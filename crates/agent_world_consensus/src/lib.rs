//! Consensus-focused facade for distributed runtime capabilities.

mod lease;
mod membership;
mod membership_logic;
mod membership_reconciliation;
mod membership_recovery;
mod mempool;
mod pos;
mod quorum;
mod sequencer_mainloop;
mod signature;

pub mod distributed {
    pub use agent_world_proto::distributed::*;
}

pub mod distributed_dht {
    pub use agent_world_net::{DistributedDht, InMemoryDht, MembershipDirectorySnapshot};
}

pub mod distributed_net {
    pub use agent_world_net::{DistributedNetwork, NetworkSubscription};
}

pub mod distributed_consensus {
    pub use super::quorum::{
        ConsensusMembershipChange, ConsensusMembershipChangeRequest,
        ConsensusMembershipChangeResult, QuorumConsensus,
    };
}

pub mod distributed_pos_consensus {
    pub use super::pos::{
        attest_world_head_with_pos, propose_world_head_with_pos, PosAttestation, PosConsensus,
        PosConsensusConfig, PosConsensusDecision, PosConsensusStatus, PosHeadRecord, PosValidator,
        POS_CONSENSUS_SNAPSHOT_VERSION,
    };
}

pub mod distributed_lease {
    pub use super::lease::*;
}

pub mod error {
    pub use agent_world_proto::world_error::WorldError;
}

pub mod util {
    use serde::de::DeserializeOwned;
    use serde::Serialize;
    use std::fs;
    use std::path::Path;

    use super::error::WorldError;

    pub fn write_json_to_path<T: Serialize>(value: &T, path: &Path) -> Result<(), WorldError> {
        let bytes = serde_json::to_vec_pretty(value)?;
        fs::write(path, bytes)?;
        Ok(())
    }

    pub fn read_json_from_path<T: DeserializeOwned>(path: &Path) -> Result<T, WorldError> {
        let bytes = fs::read(path)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub fn to_canonical_cbor<T: Serialize>(value: &T) -> Result<Vec<u8>, WorldError> {
        let mut buf = Vec::with_capacity(256);
        let canonical_value = serde_cbor::value::to_value(value)?;
        let mut serializer = serde_cbor::ser::Serializer::new(&mut buf);
        serializer.self_describe()?;
        canonical_value.serialize(&mut serializer)?;
        Ok(buf)
    }

    pub fn blake3_hex(bytes: &[u8]) -> String {
        blake3::hash(bytes).to_hex().to_string()
    }
}

pub use lease::{LeaseDecision, LeaseManager, LeaseState};
pub use membership::{
    FileMembershipAuditStore, InMemoryMembershipAuditStore, MembershipAuditStore,
    MembershipDirectoryAnnounce, MembershipDirectorySigner, MembershipDirectorySignerKeyring,
    MembershipKeyRevocationAnnounce, MembershipRestoreAuditReport, MembershipRevocationSyncPolicy,
    MembershipRevocationSyncReport, MembershipSnapshotAuditOutcome, MembershipSnapshotAuditRecord,
    MembershipSnapshotRestorePolicy, MembershipSyncClient, MembershipSyncReport,
    MembershipSyncSubscription,
};
pub use membership_reconciliation::{
    FileMembershipRevocationAlertSink, FileMembershipRevocationScheduleStateStore,
    InMemoryMembershipRevocationAlertSink, InMemoryMembershipRevocationScheduleCoordinator,
    InMemoryMembershipRevocationScheduleStateStore, MembershipRevocationAlertDedupPolicy,
    MembershipRevocationAlertDedupState, MembershipRevocationAlertPolicy,
    MembershipRevocationAlertSeverity, MembershipRevocationAlertSink,
    MembershipRevocationAnomalyAlert, MembershipRevocationCheckpointAnnounce,
    MembershipRevocationCoordinatedRunReport, MembershipRevocationReconcilePolicy,
    MembershipRevocationReconcileReport, MembershipRevocationReconcileSchedulePolicy,
    MembershipRevocationReconcileScheduleState, MembershipRevocationScheduleCoordinator,
    MembershipRevocationScheduleStateStore, MembershipRevocationScheduledRunReport,
};
pub use membership_recovery::{
    FileMembershipRevocationAlertDeadLetterStore, FileMembershipRevocationAlertRecoveryStore,
    FileMembershipRevocationCoordinatorStateStore,
    FileMembershipRevocationDeadLetterReplayPolicyAuditStore,
    FileMembershipRevocationDeadLetterReplayPolicyStore,
    FileMembershipRevocationDeadLetterReplayRollbackAlertStateStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventCompositeSequenceCursorStateStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertStateStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleStateStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceStateStore,
    FileMembershipRevocationDeadLetterReplayStateStore,
    InMemoryMembershipRevocationAlertDeadLetterStore,
    InMemoryMembershipRevocationAlertRecoveryStore,
    InMemoryMembershipRevocationCoordinatorStateStore,
    InMemoryMembershipRevocationDeadLetterReplayPolicyAuditStore,
    InMemoryMembershipRevocationDeadLetterReplayPolicyStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackAlertStateStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventCompositeSequenceCursorStateStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertStateStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleStateStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceStateStore,
    InMemoryMembershipRevocationDeadLetterReplayStateStore,
    MembershipRevocationAlertAckRetryPolicy, MembershipRevocationAlertDeadLetterReason,
    MembershipRevocationAlertDeadLetterRecord, MembershipRevocationAlertDeadLetterStore,
    MembershipRevocationAlertDeliveryMetrics, MembershipRevocationAlertRecoveryReport,
    MembershipRevocationAlertRecoveryStore, MembershipRevocationCoordinatedRecoveryRunReport,
    MembershipRevocationCoordinatorLeaseState, MembershipRevocationCoordinatorStateStore,
    MembershipRevocationDeadLetterReplayPolicy,
    MembershipRevocationDeadLetterReplayPolicyAdoptionAuditDecision,
    MembershipRevocationDeadLetterReplayPolicyAdoptionAuditRecord,
    MembershipRevocationDeadLetterReplayPolicyAuditStore,
    MembershipRevocationDeadLetterReplayPolicyState,
    MembershipRevocationDeadLetterReplayPolicyStore,
    MembershipRevocationDeadLetterReplayRollbackAlertPolicy,
    MembershipRevocationDeadLetterReplayRollbackAlertState,
    MembershipRevocationDeadLetterReplayRollbackAlertStateStore,
    MembershipRevocationDeadLetterReplayRollbackGovernanceArchiveDrillScheduledRunReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceArchiveTieredOffloadDrillAlertEventBusRunReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceArchiveTieredOffloadDrillAlertRunReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryPolicy,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryRecord,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditArchiveTier,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditPruneReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionPolicy,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditTieredOffloadPolicy,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditTieredOffloadReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceLevel,
    MembershipRevocationDeadLetterReplayRollbackGovernancePolicy,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventCompositeSequenceCursorState,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventCompositeSequenceCursorStateStore,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertPolicy,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertRunReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertState,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertStateStore,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillSchedulePolicy,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleState,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleStateStore,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduledRunReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceState,
    MembershipRevocationDeadLetterReplayRollbackGovernanceStateStore,
    MembershipRevocationDeadLetterReplayRollbackGuard,
    MembershipRevocationDeadLetterReplayScheduleState,
    MembershipRevocationDeadLetterReplayStateStore, MembershipRevocationPendingAlert,
    NoopMembershipRevocationAlertDeadLetterStore,
    StoreBackedMembershipRevocationScheduleCoordinator,
};
pub use mempool::{ActionBatchRules, ActionMempool, ActionMempoolConfig};
pub use pos::{
    attest_world_head_with_pos, propose_world_head_with_pos, PosAttestation, PosConsensus,
    PosConsensusConfig, PosConsensusDecision, PosConsensusStatus, PosHeadRecord, PosValidator,
    POS_CONSENSUS_SNAPSHOT_VERSION,
};
pub use quorum::{
    ensure_lease_holder_validator, propose_world_head_with_quorum, vote_world_head_with_quorum,
    ConsensusConfig, ConsensusDecision, ConsensusMembershipChange,
    ConsensusMembershipChangeRequest, ConsensusMembershipChangeResult, ConsensusStatus,
    ConsensusVote, HeadConsensusRecord, QuorumConsensus, CONSENSUS_SNAPSHOT_VERSION,
};
pub use sequencer_mainloop::{
    SequencerMainloop, SequencerMainloopConfig, SequencerTickReport, SequencerTickState,
};
pub use signature::{Ed25519SignatureSigner, HmacSha256Signer};

#[cfg(test)]
mod membership_dead_letter_replay_archive_tests;
#[cfg(test)]
mod membership_dead_letter_replay_audit_tests;
#[cfg(test)]
mod membership_dead_letter_replay_persistence_tests;
#[cfg(test)]
mod membership_dead_letter_replay_tests;
#[cfg(test)]
mod membership_reconciliation_tests;
#[cfg(test)]
mod membership_recovery_tests;
#[cfg(test)]
mod membership_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consensus_exports_are_available() {
        let _ = std::any::type_name::<ConsensusConfig>();
        let _ = std::any::type_name::<QuorumConsensus>();
        let _ = std::any::type_name::<PosConsensusConfig>();
        let _ = std::any::type_name::<PosConsensus>();
        let _ = std::any::type_name::<SequencerMainloop>();
        let _ = std::any::type_name::<HmacSha256Signer>();
        let _ = std::any::type_name::<Ed25519SignatureSigner>();
        let _ = std::any::type_name::<MembershipSyncClient>();
        let _ = std::any::type_name::<MembershipRevocationSyncReport>();
    }
}
