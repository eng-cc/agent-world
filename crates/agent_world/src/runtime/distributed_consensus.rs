use std::path::Path;

use super::distributed::WorldHeadAnnounce;
use super::distributed_dht::DistributedDht;
use super::distributed_lease::LeaseState;
use super::error::WorldError;

pub use agent_world_consensus::{
    ConsensusConfig, ConsensusDecision, ConsensusMembershipChange,
    ConsensusMembershipChangeRequest, ConsensusMembershipChangeResult, ConsensusStatus,
    ConsensusVote, HeadConsensusRecord,
};

pub const CONSENSUS_SNAPSHOT_VERSION: u64 = agent_world_consensus::CONSENSUS_SNAPSHOT_VERSION;

#[derive(Debug, Clone)]
pub struct QuorumConsensus {
    inner: agent_world_consensus::QuorumConsensus,
}

impl QuorumConsensus {
    pub fn new(config: ConsensusConfig) -> Result<Self, WorldError> {
        Ok(Self {
            inner: agent_world_consensus::QuorumConsensus::new(config)?,
        })
    }

    pub fn validators(&self) -> Vec<String> {
        self.inner.validators()
    }

    pub fn quorum_threshold(&self) -> usize {
        self.inner.quorum_threshold()
    }

    pub fn record(&self, world_id: &str, height: u64) -> Option<&HeadConsensusRecord> {
        self.inner.record(world_id, height)
    }

    pub fn export_records(&self) -> Vec<HeadConsensusRecord> {
        self.inner.export_records()
    }

    pub fn import_records(&mut self, records: Vec<HeadConsensusRecord>) -> Result<(), WorldError> {
        self.inner.import_records(records)?;
        Ok(())
    }

    pub fn save_snapshot_to_path(&self, path: impl AsRef<Path>) -> Result<(), WorldError> {
        self.inner.save_snapshot_to_path(path)?;
        Ok(())
    }

    pub fn load_snapshot_from_path(path: impl AsRef<Path>) -> Result<Self, WorldError> {
        Ok(Self {
            inner: agent_world_consensus::QuorumConsensus::load_snapshot_from_path(path)?,
        })
    }

    pub fn apply_membership_change(
        &mut self,
        request: &ConsensusMembershipChangeRequest,
    ) -> Result<ConsensusMembershipChangeResult, WorldError> {
        Ok(self.inner.apply_membership_change(request)?)
    }

    pub fn apply_membership_change_with_lease(
        &mut self,
        request: &ConsensusMembershipChangeRequest,
        lease: Option<&LeaseState>,
    ) -> Result<ConsensusMembershipChangeResult, WorldError> {
        Ok(self
            .inner
            .apply_membership_change_with_lease(request, lease)?)
    }

    pub fn propose_head(
        &mut self,
        head: &WorldHeadAnnounce,
        proposer_id: &str,
        proposed_at_ms: i64,
    ) -> Result<ConsensusDecision, WorldError> {
        Ok(self.inner.propose_head(head, proposer_id, proposed_at_ms)?)
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
        Ok(self.inner.vote_head(
            world_id,
            height,
            block_hash,
            validator_id,
            approve,
            voted_at_ms,
            reason,
        )?)
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
    Ok(agent_world_consensus::ensure_lease_holder_validator(
        &mut consensus.inner,
        lease,
        requested_at_ms,
    )?)
}
