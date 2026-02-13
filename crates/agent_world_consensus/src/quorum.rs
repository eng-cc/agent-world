// Quorum-based consensus helpers for distributed head commits.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::distributed::WorldHeadAnnounce;
use super::distributed_dht::DistributedDht;
use super::distributed_lease::LeaseState;
use super::error::WorldError;
use super::util::{read_json_from_path, write_json_to_path};
pub use agent_world_proto::distributed_consensus::{
    ConsensusMembershipChange, ConsensusMembershipChangeRequest, ConsensusMembershipChangeResult,
    ConsensusStatus, ConsensusVote, HeadConsensusRecord,
};

pub const CONSENSUS_SNAPSHOT_VERSION: u64 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub validators: Vec<String>,
    pub quorum_threshold: usize,
}

impl ConsensusConfig {
    pub fn majority(validators: Vec<String>) -> Self {
        Self {
            validators,
            quorum_threshold: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsensusDecision {
    pub world_id: String,
    pub height: u64,
    pub block_hash: String,
    pub status: ConsensusStatus,
    pub approvals: usize,
    pub rejections: usize,
    pub quorum_threshold: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ConsensusSnapshotFile {
    version: u64,
    validators: Vec<String>,
    quorum_threshold: usize,
    records: Vec<HeadConsensusRecord>,
}

#[derive(Debug, Clone)]
pub struct QuorumConsensus {
    validators: BTreeSet<String>,
    quorum_threshold: usize,
    records: BTreeMap<(String, u64), HeadConsensusRecord>,
}

impl QuorumConsensus {
    pub fn new(config: ConsensusConfig) -> Result<Self, WorldError> {
        let mut validators = BTreeSet::new();
        for validator in config.validators {
            let trimmed = validator.trim();
            if trimmed.is_empty() {
                continue;
            }
            validators.insert(trimmed.to_string());
        }
        if validators.is_empty() {
            return Err(WorldError::DistributedValidationFailed {
                reason: "consensus validators cannot be empty".to_string(),
            });
        }

        let validator_count = validators.len();
        let quorum_threshold = if config.quorum_threshold == 0 {
            validator_count / 2 + 1
        } else {
            config.quorum_threshold
        };
        if quorum_threshold == 0 || quorum_threshold > validator_count {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "invalid quorum threshold {quorum_threshold} for {validator_count} validators"
                ),
            });
        }
        if quorum_threshold <= validator_count / 2 {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "unsafe quorum threshold {quorum_threshold}; requires > half of {validator_count}"
                ),
            });
        }

        Ok(Self {
            validators,
            quorum_threshold,
            records: BTreeMap::new(),
        })
    }

    pub fn validators(&self) -> Vec<String> {
        self.validators.iter().cloned().collect()
    }

    pub fn quorum_threshold(&self) -> usize {
        self.quorum_threshold
    }

    pub fn record(&self, world_id: &str, height: u64) -> Option<&HeadConsensusRecord> {
        self.records.get(&(world_id.to_string(), height))
    }

    pub fn export_records(&self) -> Vec<HeadConsensusRecord> {
        self.records.values().cloned().collect()
    }

    pub fn import_records(&mut self, records: Vec<HeadConsensusRecord>) -> Result<(), WorldError> {
        self.restore_records(records)
    }

    pub fn save_snapshot_to_path(&self, path: impl AsRef<Path>) -> Result<(), WorldError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let snapshot = ConsensusSnapshotFile {
            version: CONSENSUS_SNAPSHOT_VERSION,
            validators: self.validators(),
            quorum_threshold: self.quorum_threshold,
            records: self.export_records(),
        };
        write_json_atomic(&snapshot, path)
    }

    pub fn load_snapshot_from_path(path: impl AsRef<Path>) -> Result<Self, WorldError> {
        let snapshot: ConsensusSnapshotFile = read_json_from_path(path.as_ref())?;
        if snapshot.version != CONSENSUS_SNAPSHOT_VERSION {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "unsupported consensus snapshot version {} (expected {})",
                    snapshot.version, CONSENSUS_SNAPSHOT_VERSION
                ),
            });
        }

        let mut consensus = Self::new(ConsensusConfig {
            validators: snapshot.validators,
            quorum_threshold: snapshot.quorum_threshold,
        })?;
        consensus.restore_records(snapshot.records)?;
        Ok(consensus)
    }

    pub fn apply_membership_change(
        &mut self,
        request: &ConsensusMembershipChangeRequest,
    ) -> Result<ConsensusMembershipChangeResult, WorldError> {
        if self.has_pending_records() {
            return Err(WorldError::DistributedValidationFailed {
                reason: "membership change is blocked while pending consensus records exist"
                    .to_string(),
            });
        }

        match &request.change {
            ConsensusMembershipChange::AddValidator { validator_id } => {
                let validator_id = validator_id.trim();
                if validator_id.is_empty() {
                    return Err(WorldError::DistributedValidationFailed {
                        reason: "validator_id cannot be empty".to_string(),
                    });
                }
                if self.validators.contains(validator_id) {
                    return Ok(ConsensusMembershipChangeResult {
                        applied: false,
                        validators: self.validators(),
                        quorum_threshold: self.quorum_threshold,
                    });
                }

                let mut validators = self.validators();
                validators.push(validator_id.to_string());
                let quorum_threshold =
                    derive_membership_threshold(self.quorum_threshold, validators.len());
                self.apply_membership_config(ConsensusConfig {
                    validators,
                    quorum_threshold,
                })
            }
            ConsensusMembershipChange::RemoveValidator { validator_id } => {
                let validator_id = validator_id.trim();
                if validator_id.is_empty() {
                    return Err(WorldError::DistributedValidationFailed {
                        reason: "validator_id cannot be empty".to_string(),
                    });
                }
                if !self.validators.contains(validator_id) {
                    return Err(WorldError::DistributedValidationFailed {
                        reason: format!("validator not found: {validator_id}"),
                    });
                }

                let mut validators = self.validators();
                validators.retain(|candidate| candidate != validator_id);
                let quorum_threshold =
                    derive_membership_threshold(self.quorum_threshold, validators.len());
                self.apply_membership_config(ConsensusConfig {
                    validators,
                    quorum_threshold,
                })
            }
            ConsensusMembershipChange::ReplaceValidators {
                validators,
                quorum_threshold,
            } => self.apply_membership_config(ConsensusConfig {
                validators: validators.clone(),
                quorum_threshold: *quorum_threshold,
            }),
        }
    }

    pub fn apply_membership_change_with_lease(
        &mut self,
        request: &ConsensusMembershipChangeRequest,
        lease: Option<&LeaseState>,
    ) -> Result<ConsensusMembershipChangeResult, WorldError> {
        let lease = lease.ok_or_else(|| WorldError::DistributedValidationFailed {
            reason: "membership change requires an active lease".to_string(),
        })?;
        if lease.holder_id != request.requester_id {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "membership change requester {} is not lease holder {}",
                    request.requester_id, lease.holder_id
                ),
            });
        }
        if lease.expires_at_ms <= request.requested_at_ms {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "lease {} expired at {}, request time {}",
                    lease.lease_id, lease.expires_at_ms, request.requested_at_ms
                ),
            });
        }

        self.apply_membership_change(request)
    }

    pub fn propose_head(
        &mut self,
        head: &WorldHeadAnnounce,
        proposer_id: impl Into<String>,
        proposed_at_ms: i64,
    ) -> Result<ConsensusDecision, WorldError> {
        let proposer_id = proposer_id.into();
        self.ensure_validator(&proposer_id)?;

        if let Some(committed_height) = self.latest_committed_height(&head.world_id) {
            if head.height <= committed_height {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "stale proposal for {} at height {} (committed={committed_height})",
                        head.world_id, head.height
                    ),
                });
            }
        }

        let key = (head.world_id.clone(), head.height);
        let validator_count = self.validators.len();
        let quorum_threshold = self.quorum_threshold;
        let record = self
            .records
            .entry(key)
            .or_insert_with(|| HeadConsensusRecord {
                head: head.clone(),
                proposer_id: proposer_id.clone(),
                proposed_at_ms,
                quorum_threshold,
                validator_count,
                status: ConsensusStatus::Pending,
                votes: BTreeMap::new(),
            });
        if record.head.block_hash != head.block_hash {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "conflicting proposal for {} at height {}: existing={}, new={}",
                    head.world_id, head.height, record.head.block_hash, head.block_hash
                ),
            });
        }

        Self::apply_vote(
            record,
            record.validator_count,
            record.quorum_threshold,
            &proposer_id,
            true,
            proposed_at_ms,
            Some("proposal accepted".to_string()),
        )
    }

    pub fn vote_head(
        &mut self,
        world_id: &str,
        height: u64,
        block_hash: &str,
        validator_id: &str,
        approve: bool,
        voted_at_ms: i64,
        reason: Option<String>,
    ) -> Result<ConsensusDecision, WorldError> {
        self.ensure_validator(validator_id)?;

        let key = (world_id.to_string(), height);
        let record =
            self.records
                .get_mut(&key)
                .ok_or_else(|| WorldError::DistributedValidationFailed {
                    reason: format!("proposal not found for {world_id} at height {height}"),
                })?;
        if record.head.block_hash != block_hash {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "vote block hash mismatch for {world_id} at height {height}: expected={}, got={block_hash}",
                    record.head.block_hash
                ),
            });
        }

        Self::apply_vote(
            record,
            record.validator_count,
            record.quorum_threshold,
            validator_id,
            approve,
            voted_at_ms,
            reason,
        )
    }

    fn ensure_validator(&self, validator_id: &str) -> Result<(), WorldError> {
        if self.validators.contains(validator_id) {
            return Ok(());
        }
        Err(WorldError::DistributedValidationFailed {
            reason: format!("validator not allowed: {validator_id}"),
        })
    }

    fn has_pending_records(&self) -> bool {
        self.records
            .values()
            .any(|record| matches!(record.status, ConsensusStatus::Pending))
    }

    fn apply_membership_config(
        &mut self,
        config: ConsensusConfig,
    ) -> Result<ConsensusMembershipChangeResult, WorldError> {
        let next = Self::new(config)?;
        let applied =
            self.validators != next.validators || self.quorum_threshold != next.quorum_threshold;

        self.validators = next.validators;
        self.quorum_threshold = next.quorum_threshold;

        Ok(ConsensusMembershipChangeResult {
            applied,
            validators: self.validators(),
            quorum_threshold: self.quorum_threshold,
        })
    }

    fn restore_records(&mut self, records: Vec<HeadConsensusRecord>) -> Result<(), WorldError> {
        let mut restored: BTreeMap<(String, u64), HeadConsensusRecord> = BTreeMap::new();
        for mut record in records {
            if record.head.world_id.trim().is_empty() {
                return Err(WorldError::DistributedValidationFailed {
                    reason: "consensus record world_id cannot be empty".to_string(),
                });
            }
            if record.proposer_id.trim().is_empty() {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "consensus record proposer cannot be empty for {}@{}",
                        record.head.world_id, record.head.height
                    ),
                });
            }

            if record.validator_count == 0 {
                let from_votes = record.votes.len();
                record.validator_count = self
                    .validators
                    .len()
                    .max(from_votes)
                    .max(record.quorum_threshold);
            }
            if record.validator_count == 0 {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "invalid validator_count=0 for {}@{}",
                        record.head.world_id, record.head.height
                    ),
                });
            }

            if record.quorum_threshold == 0 {
                record.quorum_threshold = record.validator_count / 2 + 1;
            }
            if record.quorum_threshold > record.validator_count {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "record quorum threshold mismatch for {}@{}: threshold={}, validator_count={}",
                        record.head.world_id,
                        record.head.height,
                        record.quorum_threshold,
                        record.validator_count
                    ),
                });
            }
            if record.quorum_threshold <= record.validator_count / 2 {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "unsafe record quorum threshold {} for validator_count {} in {}@{}",
                        record.quorum_threshold,
                        record.validator_count,
                        record.head.world_id,
                        record.head.height
                    ),
                });
            }
            if record.votes.len() > record.validator_count {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "record votes overflow for {}@{}: votes={}, validator_count={}",
                        record.head.world_id,
                        record.head.height,
                        record.votes.len(),
                        record.validator_count
                    ),
                });
            }

            for (validator_id, vote) in &record.votes {
                Self::validate_record_vote(
                    validator_id,
                    vote,
                    &record.head.world_id,
                    record.head.height,
                )?;
            }

            record.status = decide_status(
                record.validator_count,
                record.quorum_threshold,
                &record.votes,
            );
            let key = (record.head.world_id.clone(), record.head.height);
            if let Some(existing) = restored.get(&key) {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "duplicate consensus record for {}@{}: existing={}, new={}",
                        record.head.world_id,
                        record.head.height,
                        existing.head.block_hash,
                        record.head.block_hash
                    ),
                });
            }
            restored.insert(key, record);
        }
        self.records = restored;
        Ok(())
    }

    fn validate_record_vote(
        validator_id: &str,
        vote: &ConsensusVote,
        world_id: &str,
        height: u64,
    ) -> Result<(), WorldError> {
        if validator_id.trim().is_empty() {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!("invalid vote validator key in {world_id}@{height}"),
            });
        }
        if vote.validator_id != validator_id {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "invalid vote payload for {world_id}@{height}: key={validator_id}, payload={}",
                    vote.validator_id
                ),
            });
        }
        Ok(())
    }

    fn apply_vote(
        record: &mut HeadConsensusRecord,
        validator_count: usize,
        quorum_threshold: usize,
        validator_id: &str,
        approve: bool,
        voted_at_ms: i64,
        reason: Option<String>,
    ) -> Result<ConsensusDecision, WorldError> {
        if let Some(existing) = record.votes.get(validator_id) {
            if existing.approve != approve {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "conflicting vote from {validator_id} for {}@{}",
                        record.head.world_id, record.head.height
                    ),
                });
            }
            return Ok(decision_from_record(record));
        }

        if !matches!(record.status, ConsensusStatus::Pending) {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "proposal {}@{} already finalized as {:?}",
                    record.head.world_id, record.head.height, record.status
                ),
            });
        }

        record.votes.insert(
            validator_id.to_string(),
            ConsensusVote {
                validator_id: validator_id.to_string(),
                approve,
                reason,
                voted_at_ms,
            },
        );

        record.status = decide_status(validator_count, quorum_threshold, &record.votes);
        Ok(decision_from_record(record))
    }

    fn latest_committed_height(&self, world_id: &str) -> Option<u64> {
        self.records
            .iter()
            .filter(|((candidate_world_id, _), record)| {
                candidate_world_id == world_id
                    && matches!(record.status, ConsensusStatus::Committed)
            })
            .map(|((_, height), _)| *height)
            .max()
    }
}

pub fn propose_world_head_with_quorum(
    dht: &impl DistributedDht,
    consensus: &mut QuorumConsensus,
    head: &WorldHeadAnnounce,
    proposer_id: &str,
    proposed_at_ms: i64,
) -> Result<ConsensusDecision, WorldError> {
    let decision = consensus.propose_head(head, proposer_id, proposed_at_ms)?;
    if matches!(decision.status, ConsensusStatus::Committed) {
        dht.put_world_head(&head.world_id, head)?;
    }
    Ok(decision)
}

pub fn vote_world_head_with_quorum(
    dht: &impl DistributedDht,
    consensus: &mut QuorumConsensus,
    world_id: &str,
    height: u64,
    block_hash: &str,
    validator_id: &str,
    approve: bool,
    voted_at_ms: i64,
    reason: Option<String>,
) -> Result<ConsensusDecision, WorldError> {
    let decision = consensus.vote_head(
        world_id,
        height,
        block_hash,
        validator_id,
        approve,
        voted_at_ms,
        reason,
    )?;
    if matches!(decision.status, ConsensusStatus::Committed) {
        let record = consensus.record(world_id, height).ok_or_else(|| {
            WorldError::DistributedValidationFailed {
                reason: format!("committed record missing for {world_id} at height {height}"),
            }
        })?;
        dht.put_world_head(world_id, &record.head)?;
    }
    Ok(decision)
}

pub fn ensure_lease_holder_validator(
    consensus: &mut QuorumConsensus,
    lease: Option<&LeaseState>,
    requested_at_ms: i64,
) -> Result<ConsensusMembershipChangeResult, WorldError> {
    let Some(lease) = lease else {
        return Ok(ConsensusMembershipChangeResult {
            applied: false,
            validators: consensus.validators(),
            quorum_threshold: consensus.quorum_threshold(),
        });
    };

    if consensus.validators.contains(&lease.holder_id) {
        return Ok(ConsensusMembershipChangeResult {
            applied: false,
            validators: consensus.validators(),
            quorum_threshold: consensus.quorum_threshold(),
        });
    }

    let request = ConsensusMembershipChangeRequest {
        requester_id: lease.holder_id.clone(),
        requested_at_ms,
        reason: Some("auto-add active lease holder as validator".to_string()),
        change: ConsensusMembershipChange::AddValidator {
            validator_id: lease.holder_id.clone(),
        },
    };
    consensus.apply_membership_change_with_lease(&request, Some(lease))
}

fn derive_membership_threshold(current_threshold: usize, validator_count: usize) -> usize {
    let majority = validator_count / 2 + 1;
    current_threshold.max(majority).min(validator_count)
}

fn decide_status(
    validator_count: usize,
    quorum_threshold: usize,
    votes: &BTreeMap<String, ConsensusVote>,
) -> ConsensusStatus {
    let approvals = votes.values().filter(|vote| vote.approve).count();
    if approvals >= quorum_threshold {
        return ConsensusStatus::Committed;
    }

    let rejections = votes.values().filter(|vote| !vote.approve).count();
    let max_rejections_without_losing_quorum = validator_count.saturating_sub(quorum_threshold);
    if rejections > max_rejections_without_losing_quorum {
        ConsensusStatus::Rejected
    } else {
        ConsensusStatus::Pending
    }
}

fn decision_from_record(record: &HeadConsensusRecord) -> ConsensusDecision {
    let approvals = record.votes.values().filter(|vote| vote.approve).count();
    let rejections = record.votes.values().filter(|vote| !vote.approve).count();
    ConsensusDecision {
        world_id: record.head.world_id.clone(),
        height: record.head.height,
        block_hash: record.head.block_hash.clone(),
        status: record.status,
        approvals,
        rejections,
        quorum_threshold: record.quorum_threshold,
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
    use std::collections::BTreeMap;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::super::distributed_dht::InMemoryDht;
    use super::*;
    use agent_world_proto::distributed_dht::DistributedDht as _;

    fn sample_head(height: u64, block_hash: &str) -> WorldHeadAnnounce {
        WorldHeadAnnounce {
            world_id: "w1".to_string(),
            height,
            block_hash: block_hash.to_string(),
            state_root: format!("state-{height}"),
            timestamp_ms: height as i64,
            signature: "sig".to_string(),
        }
    }

    fn sample_consensus() -> QuorumConsensus {
        QuorumConsensus::new(ConsensusConfig {
            validators: vec![
                "seq-1".to_string(),
                "seq-2".to_string(),
                "seq-3".to_string(),
            ],
            quorum_threshold: 0,
        })
        .expect("consensus")
    }

    fn sample_lease(holder_id: &str, acquired_at_ms: i64, expires_at_ms: i64) -> LeaseState {
        LeaseState {
            holder_id: holder_id.to_string(),
            lease_id: format!("lease-{holder_id}-{acquired_at_ms}"),
            acquired_at_ms,
            expires_at_ms,
            term: 1,
        }
    }

    fn membership_request(
        requester_id: &str,
        requested_at_ms: i64,
        change: ConsensusMembershipChange,
    ) -> ConsensusMembershipChangeRequest {
        ConsensusMembershipChangeRequest {
            requester_id: requester_id.to_string(),
            requested_at_ms,
            reason: None,
            change,
        }
    }

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-world-{prefix}-{unique}"))
    }

    #[test]
    fn quorum_threshold_defaults_to_majority() {
        let consensus = sample_consensus();
        assert_eq!(consensus.quorum_threshold(), 2);
    }

    #[test]
    fn proposal_then_vote_commits_head() {
        let mut consensus = sample_consensus();
        let head = sample_head(1, "b1");

        let pending = consensus
            .propose_head(&head, "seq-1", 100)
            .expect("propose");
        assert_eq!(pending.status, ConsensusStatus::Pending);
        assert_eq!(pending.approvals, 1);

        let committed = consensus
            .vote_head("w1", 1, "b1", "seq-2", true, 101, None)
            .expect("vote");
        assert_eq!(committed.status, ConsensusStatus::Committed);
        assert_eq!(committed.approvals, 2);
    }

    #[test]
    fn rejections_can_finalize_proposal() {
        let mut consensus = sample_consensus();
        let head = sample_head(2, "b2");
        consensus
            .propose_head(&head, "seq-1", 200)
            .expect("propose");

        let pending = consensus
            .vote_head(
                "w1",
                2,
                "b2",
                "seq-2",
                false,
                201,
                Some("invalid".to_string()),
            )
            .expect("vote pending");
        assert_eq!(pending.status, ConsensusStatus::Pending);

        let rejected = consensus
            .vote_head(
                "w1",
                2,
                "b2",
                "seq-3",
                false,
                202,
                Some("invalid".to_string()),
            )
            .expect("vote rejected");
        assert_eq!(rejected.status, ConsensusStatus::Rejected);
        assert_eq!(rejected.rejections, 2);
    }

    #[test]
    fn proposal_conflict_is_rejected() {
        let mut consensus = sample_consensus();
        let head = sample_head(3, "b3");
        consensus
            .propose_head(&head, "seq-1", 300)
            .expect("propose");

        let conflict = sample_head(3, "b3-conflict");
        let err = consensus
            .propose_head(&conflict, "seq-2", 301)
            .expect_err("conflict");
        assert!(matches!(
            err,
            WorldError::DistributedValidationFailed { .. }
        ));
    }

    #[test]
    fn publish_happens_only_after_commit() {
        let dht = InMemoryDht::new();
        let mut consensus = sample_consensus();
        let head = sample_head(4, "b4");

        let pending = propose_world_head_with_quorum(&dht, &mut consensus, &head, "seq-1", 400)
            .expect("propose");
        assert_eq!(pending.status, ConsensusStatus::Pending);
        assert!(dht.get_world_head("w1").expect("head").is_none());

        let committed = vote_world_head_with_quorum(
            &dht,
            &mut consensus,
            "w1",
            4,
            "b4",
            "seq-2",
            true,
            401,
            None,
        )
        .expect("vote");
        assert_eq!(committed.status, ConsensusStatus::Committed);
        assert_eq!(dht.get_world_head("w1").expect("head"), Some(head.clone()));
    }

    #[test]
    fn stale_proposal_is_rejected_after_commit() {
        let mut consensus = sample_consensus();
        let head1 = sample_head(5, "b5");
        consensus
            .propose_head(&head1, "seq-1", 500)
            .expect("propose");
        consensus
            .vote_head("w1", 5, "b5", "seq-2", true, 501, None)
            .expect("commit");

        let stale = sample_head(5, "b5");
        let err = consensus
            .propose_head(&stale, "seq-3", 502)
            .expect_err("stale");
        assert!(matches!(
            err,
            WorldError::DistributedValidationFailed { .. }
        ));
    }

    #[test]
    fn membership_add_validator_updates_threshold() {
        let mut consensus = sample_consensus();
        let request = membership_request(
            "seq-1",
            1_000,
            ConsensusMembershipChange::AddValidator {
                validator_id: "seq-4".to_string(),
            },
        );
        let result = consensus
            .apply_membership_change(&request)
            .expect("membership add");

        assert!(result.applied);
        assert_eq!(result.validators.len(), 4);
        assert_eq!(result.quorum_threshold, 3);
    }

    #[test]
    fn membership_remove_validator_updates_threshold() {
        let mut consensus = sample_consensus();
        let add_request = membership_request(
            "seq-1",
            1_100,
            ConsensusMembershipChange::AddValidator {
                validator_id: "seq-4".to_string(),
            },
        );
        consensus
            .apply_membership_change(&add_request)
            .expect("membership add");

        let remove_request = membership_request(
            "seq-1",
            1_101,
            ConsensusMembershipChange::RemoveValidator {
                validator_id: "seq-4".to_string(),
            },
        );
        let result = consensus
            .apply_membership_change(&remove_request)
            .expect("membership remove");

        assert!(result.applied);
        assert_eq!(result.validators.len(), 3);
        assert_eq!(result.quorum_threshold, 3);
    }

    #[test]
    fn membership_change_is_blocked_when_pending_exists() {
        let mut consensus = sample_consensus();
        let head = sample_head(6, "b6");
        consensus
            .propose_head(&head, "seq-1", 600)
            .expect("propose pending");

        let request = membership_request(
            "seq-1",
            1_200,
            ConsensusMembershipChange::AddValidator {
                validator_id: "seq-4".to_string(),
            },
        );
        let err = consensus
            .apply_membership_change(&request)
            .expect_err("pending should block");
        assert!(matches!(
            err,
            WorldError::DistributedValidationFailed { .. }
        ));
    }

    #[test]
    fn membership_change_with_lease_requires_holder_and_active_lease() {
        let mut consensus = sample_consensus();
        let request = membership_request(
            "seq-1",
            1_300,
            ConsensusMembershipChange::AddValidator {
                validator_id: "seq-4".to_string(),
            },
        );

        let mismatched_lease = sample_lease("seq-2", 1_200, 1_400);
        let err = consensus
            .apply_membership_change_with_lease(&request, Some(&mismatched_lease))
            .expect_err("holder mismatch");
        assert!(matches!(
            err,
            WorldError::DistributedValidationFailed { .. }
        ));

        let expired_lease = sample_lease("seq-1", 1_200, 1_250);
        let err = consensus
            .apply_membership_change_with_lease(&request, Some(&expired_lease))
            .expect_err("expired lease");
        assert!(matches!(
            err,
            WorldError::DistributedValidationFailed { .. }
        ));

        let active_lease = sample_lease("seq-1", 1_200, 1_400);
        let result = consensus
            .apply_membership_change_with_lease(&request, Some(&active_lease))
            .expect("active lease apply");
        assert!(result.applied);
        assert_eq!(result.validators.len(), 4);
    }

    #[test]
    fn ensure_lease_holder_validator_auto_adds_holder() {
        let mut consensus = sample_consensus();
        let lease = sample_lease("seq-9", 1_500, 1_800);

        let first = ensure_lease_holder_validator(&mut consensus, Some(&lease), 1_600)
            .expect("auto add lease holder");
        assert!(first.applied);
        assert_eq!(first.validators.len(), 4);
        assert_eq!(first.quorum_threshold, 3);

        let second = ensure_lease_holder_validator(&mut consensus, Some(&lease), 1_601)
            .expect("already present");
        assert!(!second.applied);
        assert_eq!(second.validators.len(), 4);

        let none = ensure_lease_holder_validator(&mut consensus, None, 1_602).expect("none lease");
        assert!(!none.applied);
    }

    #[test]
    fn snapshot_round_trip_restores_consensus_records() {
        let dir = temp_dir("consensus-snapshot");
        fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("consensus_snapshot.json");

        let mut consensus = sample_consensus();
        let head = sample_head(7, "b7");
        consensus
            .propose_head(&head, "seq-1", 700)
            .expect("propose");
        consensus
            .vote_head("w1", 7, "b7", "seq-2", true, 701, None)
            .expect("commit");

        let request = membership_request(
            "seq-1",
            1_700,
            ConsensusMembershipChange::AddValidator {
                validator_id: "seq-4".to_string(),
            },
        );
        consensus
            .apply_membership_change(&request)
            .expect("membership add");

        consensus
            .save_snapshot_to_path(&path)
            .expect("save snapshot");
        let restored = QuorumConsensus::load_snapshot_from_path(&path).expect("load snapshot");

        assert_eq!(restored.quorum_threshold(), 3);
        assert_eq!(
            restored.record("w1", 7).map(|record| record.status),
            Some(ConsensusStatus::Committed)
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_snapshot_rejects_unknown_validator_vote_payload_mismatch() {
        let dir = temp_dir("consensus-invalid-snapshot");
        fs::create_dir_all(&dir).expect("mkdir");
        let path = dir.join("consensus_snapshot.json");

        let mut votes = BTreeMap::new();
        votes.insert(
            "validator-1".to_string(),
            ConsensusVote {
                validator_id: "validator-x".to_string(),
                approve: true,
                reason: None,
                voted_at_ms: 700,
            },
        );
        let snapshot = ConsensusSnapshotFile {
            version: CONSENSUS_SNAPSHOT_VERSION,
            validators: vec![
                "seq-1".to_string(),
                "seq-2".to_string(),
                "seq-3".to_string(),
            ],
            quorum_threshold: 2,
            records: vec![HeadConsensusRecord {
                head: sample_head(8, "b8"),
                proposer_id: "seq-1".to_string(),
                proposed_at_ms: 700,
                quorum_threshold: 2,
                validator_count: 3,
                status: ConsensusStatus::Pending,
                votes,
            }],
        };
        fs::write(
            &path,
            serde_json::to_vec_pretty(&snapshot).expect("serialize snapshot"),
        )
        .expect("write snapshot");

        let err = QuorumConsensus::load_snapshot_from_path(&path).expect_err("reject snapshot");
        assert!(matches!(
            err,
            WorldError::DistributedValidationFailed { .. }
        ));

        let _ = fs::remove_dir_all(&dir);
    }
}
