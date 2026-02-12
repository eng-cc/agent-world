//! Consensus-focused facade for distributed runtime capabilities.

mod lease;
mod membership;
mod membership_logic;
mod membership_reconciliation;
mod membership_recovery;
mod mempool;
mod quorum;

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
    FileMembershipRevocationDeadLetterReplayPolicyStore,
    FileMembershipRevocationDeadLetterReplayStateStore,
    InMemoryMembershipRevocationAlertDeadLetterStore,
    InMemoryMembershipRevocationAlertRecoveryStore,
    InMemoryMembershipRevocationCoordinatorStateStore,
    InMemoryMembershipRevocationDeadLetterReplayPolicyStore,
    InMemoryMembershipRevocationDeadLetterReplayStateStore,
    MembershipRevocationAlertAckRetryPolicy, MembershipRevocationAlertDeadLetterReason,
    MembershipRevocationAlertDeadLetterRecord, MembershipRevocationAlertDeadLetterStore,
    MembershipRevocationAlertDeliveryMetrics, MembershipRevocationAlertRecoveryReport,
    MembershipRevocationAlertRecoveryStore, MembershipRevocationCoordinatedRecoveryRunReport,
    MembershipRevocationCoordinatorLeaseState, MembershipRevocationCoordinatorStateStore,
    MembershipRevocationDeadLetterReplayPolicy, MembershipRevocationDeadLetterReplayPolicyState,
    MembershipRevocationDeadLetterReplayPolicyStore,
    MembershipRevocationDeadLetterReplayRollbackGuard,
    MembershipRevocationDeadLetterReplayScheduleState,
    MembershipRevocationDeadLetterReplayStateStore, MembershipRevocationPendingAlert,
    NoopMembershipRevocationAlertDeadLetterStore,
    StoreBackedMembershipRevocationScheduleCoordinator,
};
pub use mempool::{ActionBatchRules, ActionMempool, ActionMempoolConfig};
pub use quorum::{
    ensure_lease_holder_validator, propose_world_head_with_quorum, vote_world_head_with_quorum,
    ConsensusConfig, ConsensusDecision, ConsensusMembershipChange,
    ConsensusMembershipChangeRequest, ConsensusMembershipChangeResult, ConsensusStatus,
    ConsensusVote, HeadConsensusRecord, QuorumConsensus, CONSENSUS_SNAPSHOT_VERSION,
};

pub use agent_world::runtime::{
    FileMembershipRevocationDeadLetterReplayPolicyAuditStore,
    FileMembershipRevocationDeadLetterReplayRollbackAlertStateStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertStateStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleStateStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceStateStore,
    InMemoryMembershipRevocationDeadLetterReplayPolicyAuditStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackAlertStateStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertStateStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleStateStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceStateStore,
    MembershipRevocationDeadLetterReplayPolicyAdoptionAuditDecision,
    MembershipRevocationDeadLetterReplayPolicyAdoptionAuditRecord,
    MembershipRevocationDeadLetterReplayPolicyAuditStore,
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
};

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
        let _ = std::any::type_name::<MembershipSyncClient>();
        let _ = std::any::type_name::<MembershipRevocationSyncReport>();
    }
}
