mod distributed {
    pub(super) use super::super::distributed::*;
}

mod distributed_consensus {
    pub(super) use super::super::distributed_consensus::{
        ConsensusMembershipChange, ConsensusMembershipChangeRequest,
        ConsensusMembershipChangeResult, QuorumConsensus,
    };
}

mod distributed_dht {
    pub(super) use super::super::distributed_dht::{DistributedDht, MembershipDirectorySnapshot};
}

mod distributed_net {
    pub(super) use super::super::distributed_net::{DistributedNetwork, NetworkSubscription};
}

mod error {
    pub(super) use super::super::error::WorldError;
}

mod util {
    pub(super) use super::super::util::to_canonical_cbor;
}

mod membership_logic {
    include!("distributed_membership_sync/logic.rs");
}

mod shared {
    include!("../../../agent_world_consensus/src/membership.rs");
}

mod membership {
    pub(super) use super::shared::*;
}

pub use shared::*;

#[cfg(test)]
use super::distributed_consensus::{
    ConsensusMembershipChange, ConsensusMembershipChangeRequest, ConsensusMembershipChangeResult,
    QuorumConsensus,
};
#[cfg(test)]
use super::distributed_dht::MembershipDirectorySnapshot;
#[cfg(test)]
use super::error::WorldError;

#[rustfmt::skip]
pub use reconciliation::{
    FileMembershipRevocationAlertSink, FileMembershipRevocationScheduleStateStore, InMemoryMembershipRevocationAlertSink,
    InMemoryMembershipRevocationScheduleCoordinator, InMemoryMembershipRevocationScheduleStateStore,
    MembershipRevocationAlertDedupPolicy, MembershipRevocationAlertDedupState, MembershipRevocationAlertPolicy,
    MembershipRevocationAlertSeverity, MembershipRevocationAlertSink, MembershipRevocationAnomalyAlert,
    MembershipRevocationCheckpointAnnounce, MembershipRevocationCoordinatedRunReport, MembershipRevocationReconcilePolicy,
    MembershipRevocationReconcileReport, MembershipRevocationReconcileSchedulePolicy, MembershipRevocationReconcileScheduleState,
    MembershipRevocationScheduleCoordinator, MembershipRevocationScheduleStateStore, MembershipRevocationScheduledRunReport,
};
pub use recovery_exports::*;

#[cfg(test)]
mod coordination_tests;
#[cfg(test)]
mod persistence_tests;
mod reconciliation;
mod recovery;
mod recovery_exports;
#[cfg(test)]
mod recovery_replay_archive_tests;
#[cfg(test)]
mod recovery_replay_federated_tests;
#[cfg(test)]
mod recovery_replay_policy_audit_tests;
#[cfg(test)]
mod recovery_replay_policy_persistence_tests;
#[cfg(test)]
mod recovery_replay_tests;
#[cfg(test)]
mod recovery_tests;
#[cfg(test)]
mod scheduler_tests;
#[cfg(test)]
mod tests;
