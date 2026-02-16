use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::distributed::WorldHeadAnnounce;
use super::distributed_dht::DistributedDht;
use super::error::WorldError;
use super::util::{blake3_hex, read_json_from_path, write_json_to_path};

pub const POS_CONSENSUS_SNAPSHOT_VERSION: u64 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PosValidator {
    pub validator_id: String,
    pub stake: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PosConsensusConfig {
    pub validators: Vec<PosValidator>,
    pub supermajority_numerator: u64,
    pub supermajority_denominator: u64,
    pub epoch_length_slots: u64,
}

impl PosConsensusConfig {
    pub fn ethereum_like(validators: Vec<PosValidator>) -> Self {
        Self {
            validators,
            supermajority_numerator: 2,
            supermajority_denominator: 3,
            epoch_length_slots: 32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PosConsensusStatus {
    Pending,
    Committed,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PosAttestation {
    pub validator_id: String,
    pub approve: bool,
    pub source_epoch: u64,
    pub target_epoch: u64,
    pub voted_at_ms: i64,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PosHeadRecord {
    pub head: WorldHeadAnnounce,
    pub proposer_id: String,
    pub slot: u64,
    pub epoch: u64,
    pub proposed_at_ms: i64,
    pub status: PosConsensusStatus,
    pub approved_stake: u64,
    pub rejected_stake: u64,
    pub required_stake: u64,
    pub attestations: BTreeMap<String, PosAttestation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PosConsensusDecision {
    pub world_id: String,
    pub height: u64,
    pub block_hash: String,
    pub slot: u64,
    pub epoch: u64,
    pub status: PosConsensusStatus,
    pub approved_stake: u64,
    pub rejected_stake: u64,
    pub total_stake: u64,
    pub required_stake: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PosConsensusSnapshotFile {
    version: u64,
    validators: Vec<PosValidator>,
    supermajority_numerator: u64,
    supermajority_denominator: u64,
    epoch_length_slots: u64,
    records: Vec<PosHeadRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct EpochAttestationRef {
    world_id: String,
    height: u64,
    block_hash: String,
    source_epoch: u64,
    target_epoch: u64,
}

#[derive(Debug, Clone)]
pub struct PosConsensus {
    validators: BTreeMap<String, u64>,
    total_stake: u64,
    required_stake: u64,
    supermajority_numerator: u64,
    supermajority_denominator: u64,
    epoch_length_slots: u64,
    records: BTreeMap<(String, u64), PosHeadRecord>,
    attestation_history: BTreeMap<String, Vec<EpochAttestationRef>>,
}

impl PosConsensus {
    pub fn new(config: PosConsensusConfig) -> Result<Self, WorldError> {
        if config.validators.is_empty() {
            return Err(WorldError::DistributedValidationFailed {
                reason: "pos validators cannot be empty".to_string(),
            });
        }
        if config.epoch_length_slots == 0 {
            return Err(WorldError::DistributedValidationFailed {
                reason: "epoch_length_slots must be positive".to_string(),
            });
        }
        if config.supermajority_denominator == 0
            || config.supermajority_numerator == 0
            || config.supermajority_numerator > config.supermajority_denominator
        {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "invalid supermajority ratio {}/{}",
                    config.supermajority_numerator, config.supermajority_denominator
                ),
            });
        }
        if config.supermajority_numerator.saturating_mul(2) <= config.supermajority_denominator {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "unsafe supermajority ratio {}/{}; requires > 1/2",
                    config.supermajority_numerator, config.supermajority_denominator
                ),
            });
        }

        let mut validators = BTreeMap::new();
        let mut total_stake = 0u64;
        for validator in config.validators {
            let validator_id = validator.validator_id.trim();
            if validator_id.is_empty() {
                return Err(WorldError::DistributedValidationFailed {
                    reason: "validator_id cannot be empty".to_string(),
                });
            }
            if validator.stake == 0 {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!("validator {} stake must be positive", validator_id),
                });
            }
            if validators
                .insert(validator_id.to_string(), validator.stake)
                .is_some()
            {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!("duplicate validator: {}", validator_id),
                });
            }
            total_stake = total_stake.checked_add(validator.stake).ok_or_else(|| {
                WorldError::DistributedValidationFailed {
                    reason: "total stake overflow".to_string(),
                }
            })?;
        }

        if total_stake == 0 {
            return Err(WorldError::DistributedValidationFailed {
                reason: "total stake cannot be zero".to_string(),
            });
        }
        let required_stake = required_supermajority_stake(
            total_stake,
            config.supermajority_numerator,
            config.supermajority_denominator,
        )?;

        Ok(Self {
            validators,
            total_stake,
            required_stake,
            supermajority_numerator: config.supermajority_numerator,
            supermajority_denominator: config.supermajority_denominator,
            epoch_length_slots: config.epoch_length_slots,
            records: BTreeMap::new(),
            attestation_history: BTreeMap::new(),
        })
    }

    pub fn validators(&self) -> Vec<PosValidator> {
        self.validators
            .iter()
            .map(|(validator_id, stake)| PosValidator {
                validator_id: validator_id.clone(),
                stake: *stake,
            })
            .collect()
    }

    pub fn total_stake(&self) -> u64 {
        self.total_stake
    }

    pub fn required_stake(&self) -> u64 {
        self.required_stake
    }

    pub fn epoch_length_slots(&self) -> u64 {
        self.epoch_length_slots
    }

    pub fn record(&self, world_id: &str, height: u64) -> Option<&PosHeadRecord> {
        self.records.get(&(world_id.to_string(), height))
    }

    pub fn expected_proposer(&self, slot: u64) -> Option<String> {
        if self.validators.is_empty() || self.total_stake == 0 {
            return None;
        }
        let mut slot_seed = [0u8; 8];
        slot_seed.copy_from_slice(&slot.to_le_bytes());
        let seed_hash = blake3_hex(&slot_seed);
        let mut seed_bytes = [0u8; 8];
        let decoded = hex::decode(seed_hash).ok()?;
        seed_bytes.copy_from_slice(decoded.get(..8)?);
        let mut target = u64::from_le_bytes(seed_bytes) % self.total_stake;
        for (validator_id, stake) in &self.validators {
            if target < *stake {
                return Some(validator_id.clone());
            }
            target = target.saturating_sub(*stake);
        }
        self.validators.keys().next().cloned()
    }

    pub fn slot_epoch(&self, slot: u64) -> u64 {
        slot / self.epoch_length_slots
    }

    pub fn save_snapshot_to_path(&self, path: impl AsRef<Path>) -> Result<(), WorldError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let snapshot = PosConsensusSnapshotFile {
            version: POS_CONSENSUS_SNAPSHOT_VERSION,
            validators: self.validators(),
            supermajority_numerator: self.supermajority_numerator,
            supermajority_denominator: self.supermajority_denominator,
            epoch_length_slots: self.epoch_length_slots,
            records: self.records.values().cloned().collect(),
        };
        write_json_atomic(&snapshot, path)
    }

    pub fn load_snapshot_from_path(path: impl AsRef<Path>) -> Result<Self, WorldError> {
        let snapshot: PosConsensusSnapshotFile = read_json_from_path(path.as_ref())?;
        if snapshot.version != POS_CONSENSUS_SNAPSHOT_VERSION {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "unsupported pos consensus snapshot version {} (expected {})",
                    snapshot.version, POS_CONSENSUS_SNAPSHOT_VERSION
                ),
            });
        }
        let mut consensus = Self::new(PosConsensusConfig {
            validators: snapshot.validators,
            supermajority_numerator: snapshot.supermajority_numerator,
            supermajority_denominator: snapshot.supermajority_denominator,
            epoch_length_slots: snapshot.epoch_length_slots,
        })?;
        consensus.restore_records(snapshot.records)?;
        Ok(consensus)
    }

    pub fn propose_head(
        &mut self,
        head: &WorldHeadAnnounce,
        proposer_id: &str,
        slot: u64,
        proposed_at_ms: i64,
    ) -> Result<PosConsensusDecision, WorldError> {
        self.ensure_validator(proposer_id)?;
        let expected = self.expected_proposer(slot).ok_or_else(|| {
            WorldError::DistributedValidationFailed {
                reason: "no proposer available".to_string(),
            }
        })?;
        if proposer_id != expected {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "unexpected proposer for slot {}: expected={}, got={}",
                    slot, expected, proposer_id
                ),
            });
        }

        if let Some(committed_height) = self.latest_committed_height(&head.world_id) {
            if head.height <= committed_height {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "stale pos proposal for {} at height {} (committed={})",
                        head.world_id, head.height, committed_height
                    ),
                });
            }
        }

        let epoch = self.slot_epoch(slot);
        let key = (head.world_id.clone(), head.height);
        let required_stake = self.required_stake;
        let existing = self.records.entry(key).or_insert_with(|| PosHeadRecord {
            head: head.clone(),
            proposer_id: proposer_id.to_string(),
            slot,
            epoch,
            proposed_at_ms,
            status: PosConsensusStatus::Pending,
            approved_stake: 0,
            rejected_stake: 0,
            required_stake,
            attestations: BTreeMap::new(),
        });
        if existing.head.block_hash != head.block_hash {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "conflicting pos proposal for {}@{}: existing={}, new={}",
                    head.world_id, head.height, existing.head.block_hash, head.block_hash
                ),
            });
        }
        if existing.slot != slot {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "proposal slot mismatch for {}@{}: existing={}, new={}",
                    head.world_id, head.height, existing.slot, slot
                ),
            });
        }

        self.attest_head(
            &head.world_id,
            head.height,
            &head.block_hash,
            proposer_id,
            true,
            proposed_at_ms,
            epoch.saturating_sub(1),
            epoch,
            Some("proposal accepted".to_string()),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn attest_head(
        &mut self,
        world_id: &str,
        height: u64,
        block_hash: &str,
        validator_id: &str,
        approve: bool,
        voted_at_ms: i64,
        source_epoch: u64,
        target_epoch: u64,
        reason: Option<String>,
    ) -> Result<PosConsensusDecision, WorldError> {
        self.ensure_validator(validator_id)?;
        if source_epoch > target_epoch {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "invalid attestation epochs for {}: source_epoch={} > target_epoch={}",
                    validator_id, source_epoch, target_epoch
                ),
            });
        }

        let key = (world_id.to_string(), height);
        {
            let record =
                self.records
                    .get(&key)
                    .ok_or_else(|| WorldError::DistributedValidationFailed {
                        reason: format!(
                            "pos proposal not found for {} at height {}",
                            world_id, height
                        ),
                    })?;
            if record.head.block_hash != block_hash {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "pos attestation hash mismatch for {}@{}: expected={}, got={}",
                        world_id, height, record.head.block_hash, block_hash
                    ),
                });
            }
            if record.epoch != target_epoch {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "target_epoch mismatch for {}@{}: expected={}, got={}",
                        world_id, height, record.epoch, target_epoch
                    ),
                });
            }
        }

        self.ensure_slash_free(
            validator_id,
            world_id,
            height,
            block_hash,
            source_epoch,
            target_epoch,
        )?;

        let total_stake = self.total_stake;
        let required_stake = self.required_stake;
        let validator_stake = self.validators.clone();
        let decision = {
            let record = self.records.get_mut(&key).expect("record exists");
            if let Some(existing) = record.attestations.get(validator_id) {
                if existing.approve == approve
                    && existing.source_epoch == source_epoch
                    && existing.target_epoch == target_epoch
                {
                    return Ok(decision_from_record(record, total_stake));
                }
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "conflicting attestation from {} for {}@{}",
                        validator_id, world_id, height
                    ),
                });
            }
            record.attestations.insert(
                validator_id.to_string(),
                PosAttestation {
                    validator_id: validator_id.to_string(),
                    approve,
                    source_epoch,
                    target_epoch,
                    voted_at_ms,
                    reason,
                },
            );

            let (approved_stake, rejected_stake) =
                stake_totals(&validator_stake, &record.attestations)?;
            record.approved_stake = approved_stake;
            record.rejected_stake = rejected_stake;
            record.required_stake = required_stake;
            record.status =
                decide_status(total_stake, required_stake, approved_stake, rejected_stake);
            decision_from_record(record, total_stake)
        };

        self.record_attestation_history(
            validator_id,
            world_id,
            height,
            block_hash,
            source_epoch,
            target_epoch,
        );
        Ok(decision)
    }

    fn restore_records(&mut self, records: Vec<PosHeadRecord>) -> Result<(), WorldError> {
        let mut restored = BTreeMap::new();
        for mut record in records {
            if record.head.world_id.trim().is_empty() {
                return Err(WorldError::DistributedValidationFailed {
                    reason: "pos record world_id cannot be empty".to_string(),
                });
            }
            if record.proposer_id.trim().is_empty() {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "pos record proposer cannot be empty for {}@{}",
                        record.head.world_id, record.head.height
                    ),
                });
            }
            if !self.validators.contains_key(&record.proposer_id) {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "unknown pos proposer {} for {}@{}",
                        record.proposer_id, record.head.world_id, record.head.height
                    ),
                });
            }

            let expected_epoch = self.slot_epoch(record.slot);
            if record.epoch != expected_epoch {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "invalid pos epoch for {}@{}: expected={}, actual={}",
                        record.head.world_id, record.head.height, expected_epoch, record.epoch
                    ),
                });
            }

            for (validator_id, attestation) in &record.attestations {
                if attestation.validator_id != *validator_id {
                    return Err(WorldError::DistributedValidationFailed {
                        reason: format!(
                            "invalid attestation key/payload mismatch for {}@{}",
                            record.head.world_id, record.head.height
                        ),
                    });
                }
                if !self.validators.contains_key(validator_id) {
                    return Err(WorldError::DistributedValidationFailed {
                        reason: format!(
                            "unknown attestation validator {} for {}@{}",
                            validator_id, record.head.world_id, record.head.height
                        ),
                    });
                }
                if attestation.target_epoch != record.epoch {
                    return Err(WorldError::DistributedValidationFailed {
                        reason: format!(
                            "invalid target_epoch for {}@{} from {}",
                            record.head.world_id, record.head.height, validator_id
                        ),
                    });
                }
                if attestation.source_epoch > attestation.target_epoch {
                    return Err(WorldError::DistributedValidationFailed {
                        reason: format!(
                            "invalid attestation epochs for {} on {}@{}",
                            validator_id, record.head.world_id, record.head.height
                        ),
                    });
                }
            }

            let (approved_stake, rejected_stake) =
                stake_totals(&self.validators, &record.attestations)?;
            record.approved_stake = approved_stake;
            record.rejected_stake = rejected_stake;
            record.required_stake = self.required_stake;
            record.status = decide_status(
                self.total_stake,
                self.required_stake,
                approved_stake,
                rejected_stake,
            );

            let key = (record.head.world_id.clone(), record.head.height);
            if restored.insert(key, record).is_some() {
                return Err(WorldError::DistributedValidationFailed {
                    reason: "duplicate pos record in snapshot".to_string(),
                });
            }
        }

        self.records = restored;
        self.attestation_history.clear();
        let mut historical_votes = Vec::new();
        for record in self.records.values() {
            for attestation in record.attestations.values() {
                historical_votes.push((
                    attestation.validator_id.clone(),
                    record.head.world_id.clone(),
                    record.head.height,
                    record.head.block_hash.clone(),
                    attestation.source_epoch,
                    attestation.target_epoch,
                ));
            }
        }
        for (validator_id, world_id, height, block_hash, source_epoch, target_epoch) in
            historical_votes
        {
            self.record_attestation_history(
                &validator_id,
                &world_id,
                height,
                &block_hash,
                source_epoch,
                target_epoch,
            );
        }
        Ok(())
    }

    fn ensure_validator(&self, validator_id: &str) -> Result<u64, WorldError> {
        self.validators.get(validator_id).copied().ok_or_else(|| {
            WorldError::DistributedValidationFailed {
                reason: format!("validator not allowed: {}", validator_id),
            }
        })
    }

    fn ensure_slash_free(
        &self,
        validator_id: &str,
        world_id: &str,
        height: u64,
        block_hash: &str,
        source_epoch: u64,
        target_epoch: u64,
    ) -> Result<(), WorldError> {
        let Some(history) = self.attestation_history.get(validator_id) else {
            return Ok(());
        };
        for previous in history {
            if previous.world_id != world_id {
                continue;
            }
            if previous.height == height && previous.block_hash == block_hash {
                continue;
            }
            if previous.target_epoch == target_epoch
                && (previous.block_hash != block_hash || previous.source_epoch != source_epoch)
            {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "slashable double vote detected for {} at target_epoch {}",
                        validator_id, target_epoch
                    ),
                });
            }
            let previous_surrounds_new =
                previous.source_epoch < source_epoch && previous.target_epoch > target_epoch;
            let new_surrounds_previous =
                previous.source_epoch > source_epoch && previous.target_epoch < target_epoch;
            if previous_surrounds_new || new_surrounds_previous {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "slashable surround vote detected for {} between ({},{}) and ({},{})",
                        validator_id,
                        previous.source_epoch,
                        previous.target_epoch,
                        source_epoch,
                        target_epoch
                    ),
                });
            }
        }
        Ok(())
    }

    fn record_attestation_history(
        &mut self,
        validator_id: &str,
        world_id: &str,
        height: u64,
        block_hash: &str,
        source_epoch: u64,
        target_epoch: u64,
    ) {
        let history = self
            .attestation_history
            .entry(validator_id.to_string())
            .or_default();
        let item = EpochAttestationRef {
            world_id: world_id.to_string(),
            height,
            block_hash: block_hash.to_string(),
            source_epoch,
            target_epoch,
        };
        if history.iter().any(|existing| existing == &item) {
            return;
        }
        history.push(item);
    }

    fn latest_committed_height(&self, world_id: &str) -> Option<u64> {
        self.records
            .iter()
            .filter(|((candidate_world_id, _), record)| {
                candidate_world_id == world_id
                    && matches!(record.status, PosConsensusStatus::Committed)
            })
            .map(|((_, height), _)| *height)
            .max()
    }
}

pub fn propose_world_head_with_pos(
    dht: &impl DistributedDht,
    consensus: &mut PosConsensus,
    head: &WorldHeadAnnounce,
    proposer_id: &str,
    slot: u64,
    proposed_at_ms: i64,
) -> Result<PosConsensusDecision, WorldError> {
    let decision = consensus.propose_head(head, proposer_id, slot, proposed_at_ms)?;
    if matches!(decision.status, PosConsensusStatus::Committed) {
        dht.put_world_head(&head.world_id, head)?;
    }
    Ok(decision)
}

#[allow(clippy::too_many_arguments)]
pub fn attest_world_head_with_pos(
    dht: &impl DistributedDht,
    consensus: &mut PosConsensus,
    world_id: &str,
    height: u64,
    block_hash: &str,
    validator_id: &str,
    approve: bool,
    voted_at_ms: i64,
    source_epoch: u64,
    target_epoch: u64,
    reason: Option<String>,
) -> Result<PosConsensusDecision, WorldError> {
    let decision = consensus.attest_head(
        world_id,
        height,
        block_hash,
        validator_id,
        approve,
        voted_at_ms,
        source_epoch,
        target_epoch,
        reason,
    )?;
    if matches!(decision.status, PosConsensusStatus::Committed) {
        let record = consensus.record(world_id, height).ok_or_else(|| {
            WorldError::DistributedValidationFailed {
                reason: format!(
                    "committed pos record missing for {} at height {}",
                    world_id, height
                ),
            }
        })?;
        dht.put_world_head(world_id, &record.head)?;
    }
    Ok(decision)
}

fn required_supermajority_stake(
    total_stake: u64,
    numerator: u64,
    denominator: u64,
) -> Result<u64, WorldError> {
    let multiplied = u128::from(total_stake)
        .checked_mul(u128::from(numerator))
        .ok_or_else(|| WorldError::DistributedValidationFailed {
            reason: "required stake multiplication overflow".to_string(),
        })?;
    let denominator = u128::from(denominator);
    let mut required = multiplied / denominator;
    if multiplied % denominator != 0 {
        required += 1;
    }
    let required =
        u64::try_from(required).map_err(|_| WorldError::DistributedValidationFailed {
            reason: "required stake overflow".to_string(),
        })?;
    if required == 0 || required > total_stake {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "invalid required stake {} for total stake {}",
                required, total_stake
            ),
        });
    }
    Ok(required)
}

fn stake_totals(
    validators: &BTreeMap<String, u64>,
    attestations: &BTreeMap<String, PosAttestation>,
) -> Result<(u64, u64), WorldError> {
    let mut approved_stake = 0u64;
    let mut rejected_stake = 0u64;
    let mut seen = BTreeSet::new();
    for (validator_id, attestation) in attestations {
        if !seen.insert(validator_id.clone()) {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!("duplicate attestation from {}", validator_id),
            });
        }
        let stake = validators.get(validator_id).ok_or_else(|| {
            WorldError::DistributedValidationFailed {
                reason: format!("unknown validator in attestation: {}", validator_id),
            }
        })?;
        if attestation.approve {
            approved_stake = approved_stake.checked_add(*stake).ok_or_else(|| {
                WorldError::DistributedValidationFailed {
                    reason: "approved stake overflow".to_string(),
                }
            })?;
        } else {
            rejected_stake = rejected_stake.checked_add(*stake).ok_or_else(|| {
                WorldError::DistributedValidationFailed {
                    reason: "rejected stake overflow".to_string(),
                }
            })?;
        }
    }
    Ok((approved_stake, rejected_stake))
}

fn decide_status(
    total_stake: u64,
    required_stake: u64,
    approved_stake: u64,
    rejected_stake: u64,
) -> PosConsensusStatus {
    if approved_stake >= required_stake {
        return PosConsensusStatus::Committed;
    }
    if total_stake.saturating_sub(rejected_stake) < required_stake {
        PosConsensusStatus::Rejected
    } else {
        PosConsensusStatus::Pending
    }
}

fn decision_from_record(record: &PosHeadRecord, total_stake: u64) -> PosConsensusDecision {
    PosConsensusDecision {
        world_id: record.head.world_id.clone(),
        height: record.head.height,
        block_hash: record.head.block_hash.clone(),
        slot: record.slot,
        epoch: record.epoch,
        status: record.status,
        approved_stake: record.approved_stake,
        rejected_stake: record.rejected_stake,
        total_stake,
        required_stake: record.required_stake,
    }
}

fn write_json_atomic<T: Serialize>(value: &T, path: &Path) -> Result<(), WorldError> {
    let tmp = path.with_extension("tmp");
    write_json_to_path(value, &tmp)?;
    fs::rename(tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::super::distributed_dht::InMemoryDht;
    use super::*;
    use agent_world_proto::distributed_dht::DistributedDht as _;

    fn validators() -> Vec<PosValidator> {
        vec![
            PosValidator {
                validator_id: "val-a".to_string(),
                stake: 40,
            },
            PosValidator {
                validator_id: "val-b".to_string(),
                stake: 35,
            },
            PosValidator {
                validator_id: "val-c".to_string(),
                stake: 25,
            },
        ]
    }

    fn sample_head(height: u64, block_hash: &str) -> WorldHeadAnnounce {
        WorldHeadAnnounce {
            world_id: "w1".to_string(),
            height,
            block_hash: block_hash.to_string(),
            state_root: format!("state-{}", height),
            timestamp_ms: height as i64,
            signature: "sig".to_string(),
        }
    }

    fn sample_consensus() -> PosConsensus {
        PosConsensus::new(PosConsensusConfig::ethereum_like(validators())).expect("pos consensus")
    }

    fn find_slot_with_proposer(consensus: &PosConsensus, proposer: &str, start: u64) -> u64 {
        for slot in start..(start + 256) {
            if consensus.expected_proposer(slot).as_deref() == Some(proposer) {
                return slot;
            }
        }
        panic!("slot not found for proposer {}", proposer);
    }

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("duration")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-world-pos-{prefix}-{unique}"))
    }

    #[test]
    fn pos_commit_requires_supermajority_stake() {
        let mut consensus = sample_consensus();
        let proposer = consensus.expected_proposer(0).expect("proposer");
        let head = sample_head(1, "b1");
        let pending = consensus
            .propose_head(&head, &proposer, 0, 100)
            .expect("propose");
        assert_eq!(pending.status, PosConsensusStatus::Pending);
        assert_eq!(pending.required_stake, 67);

        let next_validator = consensus
            .validators()
            .into_iter()
            .find(|validator| validator.validator_id != proposer)
            .expect("next validator");
        let decision = consensus
            .attest_head(
                "w1",
                1,
                "b1",
                &next_validator.validator_id,
                true,
                101,
                0,
                0,
                None,
            )
            .expect("attest");
        if next_validator.stake + pending.approved_stake >= pending.required_stake {
            assert_eq!(decision.status, PosConsensusStatus::Committed);
        } else {
            assert_eq!(decision.status, PosConsensusStatus::Pending);
        }
    }

    #[test]
    fn pos_rejects_when_remaining_stake_cannot_reach_supermajority() {
        let mut consensus = sample_consensus();
        let proposer = consensus.expected_proposer(1).expect("proposer");
        let head = sample_head(2, "b2");
        let _ = consensus
            .propose_head(&head, &proposer, 1, 200)
            .expect("propose");
        let rejector = consensus
            .validators()
            .into_iter()
            .filter(|validator| validator.validator_id != proposer)
            .max_by_key(|validator| validator.stake)
            .expect("rejector");
        let decision = consensus
            .attest_head(
                "w1",
                2,
                "b2",
                &rejector.validator_id,
                false,
                201,
                0,
                0,
                Some("invalid transition".to_string()),
            )
            .expect("reject");
        assert_eq!(decision.status, PosConsensusStatus::Rejected);
    }

    #[test]
    fn pos_proposer_must_match_expected_slot_proposer() {
        let mut consensus = sample_consensus();
        let expected = consensus.expected_proposer(2).expect("expected proposer");
        let wrong = consensus
            .validators()
            .into_iter()
            .find(|validator| validator.validator_id != expected)
            .expect("wrong proposer")
            .validator_id;
        let err = consensus
            .propose_head(&sample_head(3, "b3"), &wrong, 2, 300)
            .expect_err("must reject wrong proposer");
        assert!(matches!(
            err,
            WorldError::DistributedValidationFailed { .. }
        ));
    }

    #[test]
    fn pos_detects_double_vote_slashing_condition() {
        let mut consensus = sample_consensus();
        let validator = "val-c";
        let slot_a = find_slot_with_proposer(&consensus, "val-a", 0);
        let slot_b = find_slot_with_proposer(&consensus, "val-b", slot_a + 1);
        let head_a = sample_head(10, "b10");
        let head_b = sample_head(11, "b11");
        consensus
            .propose_head(&head_a, "val-a", slot_a, 1000)
            .expect("propose a");
        consensus
            .propose_head(&head_b, "val-b", slot_b, 1001)
            .expect("propose b");

        let target_epoch = consensus.slot_epoch(slot_a);
        consensus
            .attest_head(
                "w1",
                10,
                "b10",
                validator,
                true,
                1002,
                target_epoch.saturating_sub(1),
                target_epoch,
                None,
            )
            .expect("first attestation");

        let err = consensus
            .attest_head(
                "w1",
                11,
                "b11",
                validator,
                true,
                1003,
                target_epoch.saturating_sub(1),
                target_epoch,
                None,
            )
            .expect_err("double vote");
        assert!(matches!(
            err,
            WorldError::DistributedValidationFailed { .. }
        ));
    }

    #[test]
    fn pos_detects_surround_vote_slashing_condition() {
        let mut consensus = sample_consensus();
        let validator = "val-b";
        let slot_epoch4 = find_slot_with_proposer(&consensus, "val-a", 128);
        let slot_epoch3 = find_slot_with_proposer(&consensus, "val-c", 96);
        let head_a = sample_head(20, "b20");
        let head_b = sample_head(21, "b21");
        consensus
            .propose_head(&head_a, "val-a", slot_epoch4, 2000)
            .expect("propose epoch4");
        consensus
            .propose_head(&head_b, "val-c", slot_epoch3, 2001)
            .expect("propose epoch3");

        let epoch4 = consensus.slot_epoch(slot_epoch4);
        let epoch3 = consensus.slot_epoch(slot_epoch3);
        consensus
            .attest_head(
                "w1",
                20,
                "b20",
                validator,
                true,
                2002,
                epoch4.saturating_sub(2),
                epoch4,
                None,
            )
            .expect("first vote");

        let err = consensus
            .attest_head("w1", 21, "b21", validator, true, 2003, epoch3, epoch3, None)
            .expect_err("surround vote");
        assert!(matches!(
            err,
            WorldError::DistributedValidationFailed { .. }
        ));
    }

    #[test]
    fn dht_publish_happens_only_after_pos_commit() {
        let mut consensus = sample_consensus();
        let dht = InMemoryDht::new();
        let proposer = consensus.expected_proposer(5).expect("proposer");
        let head = sample_head(30, "b30");

        let pending = propose_world_head_with_pos(&dht, &mut consensus, &head, &proposer, 5, 3000)
            .expect("propose");
        assert_eq!(pending.status, PosConsensusStatus::Pending);
        assert!(dht.get_world_head("w1").expect("head").is_none());

        let approver = consensus
            .validators()
            .into_iter()
            .find(|validator| validator.validator_id != proposer)
            .expect("approver");
        let target_epoch = consensus.slot_epoch(5);
        let decision = attest_world_head_with_pos(
            &dht,
            &mut consensus,
            "w1",
            30,
            "b30",
            &approver.validator_id,
            true,
            3001,
            0,
            target_epoch,
            None,
        )
        .expect("attest");

        if matches!(decision.status, PosConsensusStatus::Committed) {
            assert_eq!(dht.get_world_head("w1").expect("head"), Some(head));
        }
    }

    #[test]
    fn pos_snapshot_round_trip_restores_records() {
        let dir = temp_dir("snapshot");
        let path = dir.join("pos-consensus.json");
        let mut consensus = sample_consensus();
        let proposer = consensus.expected_proposer(8).expect("proposer");
        consensus
            .propose_head(&sample_head(40, "b40"), &proposer, 8, 4000)
            .expect("propose");

        consensus
            .save_snapshot_to_path(&path)
            .expect("save snapshot");
        let loaded = PosConsensus::load_snapshot_from_path(&path).expect("load snapshot");
        let loaded_record = loaded.record("w1", 40).expect("record");
        assert_eq!(loaded_record.head.block_hash, "b40");
        assert_eq!(loaded_record.epoch, loaded.slot_epoch(8));

        let _ = fs::remove_dir_all(dir);
    }
}
