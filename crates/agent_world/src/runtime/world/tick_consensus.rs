use super::super::util::{hash_json, sha256_hex};
use super::super::{
    CausedBy, TickBlock, TickBlockHeader, TickCertificate, TickConsensusRecord,
    TickExecutionDigest, WorldError, WorldEvent, WorldEventBody, WorldEventId, WorldTime,
};
use super::World;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

const TICK_CONSENSUS_EPOCH_LEN: u64 = 360;
const TICK_CONSENSUS_GENESIS_PARENT_HASH: &str = "genesis";
const TICK_CERT_SIGNATURE_PREFIX: &str = "sha256:";

#[derive(Serialize)]
struct TickEventHashInput<'a> {
    id: WorldEventId,
    caused_by: &'a Option<CausedBy>,
    body: &'a WorldEventBody,
}

#[derive(Serialize)]
struct StateRootProjection<'a> {
    state: &'a super::super::WorldState,
    manifest_hash: &'a str,
    policy_hash: &'a str,
}

impl World {
    pub fn latest_tick_consensus_record(&self) -> Option<&TickConsensusRecord> {
        self.tick_consensus_records.last()
    }

    pub fn verify_tick_consensus_chain(&self) -> Result<(), WorldError> {
        let mut expected_parent = TICK_CONSENSUS_GENESIS_PARENT_HASH.to_string();
        let mut expected_height = 1_u64;
        for record in &self.tick_consensus_records {
            if record.block.header.parent_hash != expected_parent {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "tick consensus parent hash mismatch tick={} expected={} found={}",
                        record.block.header.tick, expected_parent, record.block.header.parent_hash
                    ),
                });
            }
            if record.certificate.consensus_height != expected_height {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "tick consensus height mismatch tick={} expected={} found={}",
                        record.block.header.tick,
                        expected_height,
                        record.certificate.consensus_height
                    ),
                });
            }
            let block_hash = record.block.block_hash();
            if record.certificate.block_hash != block_hash {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "tick consensus block hash mismatch tick={} expected={} found={}",
                        record.block.header.tick, block_hash, record.certificate.block_hash
                    ),
                });
            }
            let events_hash = self.hash_tick_events_from_ids(
                record.block.header.tick,
                &record.block.ordered_event_ids,
            )?;
            if events_hash != record.block.header.events_hash {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "tick events hash mismatch tick={} expected={} found={}",
                        record.block.header.tick, events_hash, record.block.header.events_hash
                    ),
                });
            }
            expected_parent = record.certificate.block_hash.clone();
            expected_height = expected_height.saturating_add(1);
        }
        if let Some(record) = self.tick_consensus_records.last() {
            let current_state_root = self.current_state_root_hash()?;
            if record.block.header.state_root != current_state_root {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "latest state root mismatch tick={} expected={} found={}",
                        record.block.header.tick,
                        current_state_root,
                        record.block.header.state_root
                    ),
                });
            }
        }
        Ok(())
    }

    pub(super) fn record_tick_consensus(&mut self) -> Result<(), WorldError> {
        self.record_tick_consensus_for_tick(self.state.time)
    }

    pub(super) fn record_tick_consensus_for_tick(
        &mut self,
        tick: WorldTime,
    ) -> Result<(), WorldError> {
        let tick_events: Vec<WorldEvent> = self
            .journal
            .events
            .iter()
            .filter(|event| event.time == tick)
            .cloned()
            .collect();
        let ordered_event_ids: Vec<WorldEventId> =
            tick_events.iter().map(|event| event.id).collect();
        let ordered_action_ids = Self::extract_ordered_action_ids(&tick_events);
        let events_hash = self.hash_tick_events(&tick_events)?;
        let state_root = self.current_state_root_hash()?;
        let parent_hash = self.parent_hash_for_tick(tick);
        let executor_version = env!("CARGO_PKG_VERSION").to_string();
        let randomness_seed = Self::derive_tick_randomness_seed(parent_hash.as_str(), tick);
        let consensus_height = self.consensus_height_for_tick(tick);
        let header = TickBlockHeader {
            epoch: tick / TICK_CONSENSUS_EPOCH_LEN,
            tick,
            parent_hash,
            events_hash: events_hash.clone(),
            state_root: state_root.clone(),
            executor_version,
            randomness_seed,
        };
        let execution_digest = TickExecutionDigest {
            action_batch_hash: hash_json(&ordered_action_ids)?,
            domain_events_hash: Self::hash_tick_domain_events(&tick_events)?,
            state_projection_hash: state_root,
        };
        let block = TickBlock {
            header,
            ordered_action_ids,
            ordered_event_ids: ordered_event_ids.clone(),
            event_count: ordered_event_ids.len() as u32,
            execution_digest,
        };
        let block_hash = block.block_hash();
        let mut signatures = BTreeMap::new();
        signatures.insert(
            super::BUILTIN_MODULE_SIGNER_NODE_ID.to_string(),
            format!(
                "{}{}",
                TICK_CERT_SIGNATURE_PREFIX,
                sha256_hex(
                    format!(
                        "tickcert:v1|{}|{}",
                        super::BUILTIN_MODULE_SIGNER_NODE_ID,
                        block_hash
                    )
                    .as_bytes()
                )
            ),
        );
        let record = TickConsensusRecord {
            block,
            certificate: TickCertificate {
                block_hash,
                consensus_height,
                threshold: 1,
                signatures,
            },
        };
        self.upsert_tick_consensus_record(record);
        self.verify_latest_tick_consensus_record()
    }

    fn verify_latest_tick_consensus_record(&self) -> Result<(), WorldError> {
        let Some(record) = self.tick_consensus_records.last() else {
            return Ok(());
        };
        let events_hash = self.hash_tick_events_from_ids(
            record.block.header.tick,
            record.block.ordered_event_ids.as_slice(),
        )?;
        if events_hash != record.block.header.events_hash {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "latest tick events hash mismatch tick={} expected={} found={}",
                    record.block.header.tick, events_hash, record.block.header.events_hash
                ),
            });
        }
        let block_hash = record.block.block_hash();
        if block_hash != record.certificate.block_hash {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "latest tick block hash mismatch tick={} expected={} found={}",
                    record.block.header.tick, block_hash, record.certificate.block_hash
                ),
            });
        }
        let current_state_root = self.current_state_root_hash()?;
        if record.block.header.tick == self.state.time
            && current_state_root != record.block.header.state_root
        {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "latest tick state root mismatch tick={} expected={} found={}",
                    record.block.header.tick, current_state_root, record.block.header.state_root
                ),
            });
        }
        Ok(())
    }

    fn hash_tick_events_from_ids(
        &self,
        tick: WorldTime,
        ordered_event_ids: &[WorldEventId],
    ) -> Result<String, WorldError> {
        if ordered_event_ids.is_empty() {
            let empty_inputs: Vec<TickEventHashInput<'_>> = Vec::new();
            return hash_json(&empty_inputs);
        }
        let mut events = Vec::with_capacity(ordered_event_ids.len());
        for event_id in ordered_event_ids {
            let event = self
                .journal
                .events
                .iter()
                .find(|event| event.id == *event_id && event.time == tick)
                .ok_or_else(|| WorldError::DistributedValidationFailed {
                    reason: format!(
                        "tick event missing during hash rebuild tick={} event_id={}",
                        tick, event_id
                    ),
                })?;
            events.push(event.clone());
        }
        self.hash_tick_events(events.as_slice())
    }

    fn hash_tick_events(&self, events: &[WorldEvent]) -> Result<String, WorldError> {
        let inputs: Vec<TickEventHashInput<'_>> = events
            .iter()
            .map(|event| TickEventHashInput {
                id: event.id,
                caused_by: &event.caused_by,
                body: &event.body,
            })
            .collect();
        hash_json(&inputs)
    }

    fn hash_tick_domain_events(events: &[WorldEvent]) -> Result<String, WorldError> {
        let inputs: Vec<TickEventHashInput<'_>> = events
            .iter()
            .filter(|event| matches!(event.body, WorldEventBody::Domain(_)))
            .map(|event| TickEventHashInput {
                id: event.id,
                caused_by: &event.caused_by,
                body: &event.body,
            })
            .collect();
        hash_json(&inputs)
    }

    fn extract_ordered_action_ids(events: &[WorldEvent]) -> Vec<u64> {
        let mut seen = BTreeSet::new();
        let mut ordered = Vec::new();
        for event in events {
            let Some(CausedBy::Action(action_id)) = &event.caused_by else {
                continue;
            };
            if seen.insert(*action_id) {
                ordered.push(*action_id);
            }
        }
        ordered
    }

    fn current_state_root_hash(&self) -> Result<String, WorldError> {
        let manifest_hash = self.current_manifest_hash()?;
        let policy_hash = hash_json(&self.policies)?;
        let projection = StateRootProjection {
            state: &self.state,
            manifest_hash: manifest_hash.as_str(),
            policy_hash: policy_hash.as_str(),
        };
        hash_json(&projection)
    }

    fn consensus_height_for_tick(&self, tick: WorldTime) -> u64 {
        match self
            .tick_consensus_records
            .iter()
            .position(|record| record.block.header.tick == tick)
        {
            Some(index) => index as u64 + 1,
            None => self.tick_consensus_records.len() as u64 + 1,
        }
    }

    fn parent_hash_for_tick(&self, tick: WorldTime) -> String {
        self.tick_consensus_records
            .iter()
            .rev()
            .find(|record| record.block.header.tick < tick)
            .map(|record| record.certificate.block_hash.clone())
            .unwrap_or_else(|| TICK_CONSENSUS_GENESIS_PARENT_HASH.to_string())
    }

    fn derive_tick_randomness_seed(parent_hash: &str, tick: WorldTime) -> String {
        sha256_hex(format!("tick-rand:v1|{parent_hash}|{tick}").as_bytes())
    }

    fn upsert_tick_consensus_record(&mut self, record: TickConsensusRecord) {
        if let Some(index) = self
            .tick_consensus_records
            .iter()
            .position(|existing| existing.block.header.tick == record.block.header.tick)
        {
            self.tick_consensus_records[index] = record;
            return;
        }
        self.tick_consensus_records.push(record);
    }
}
