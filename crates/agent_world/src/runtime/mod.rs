//! Runtime module - the core world execution engine.
//!
//! This module contains the World struct and all supporting types for:
//! - World state management
//! - Event processing and journaling
//! - Snapshot persistence and recovery
//! - Effect system with capabilities and policies
//! - Governance and manifest management
//! - Agent scheduling

mod agent_cell;
mod audit;
mod blob_store;
pub mod distributed;
mod distributed_bootstrap;
mod distributed_client;
mod distributed_consensus;
mod distributed_dht;
mod distributed_dht_cache;
mod distributed_gateway;
mod distributed_head_follow;
mod distributed_index;
mod distributed_index_store;
mod distributed_lease;
mod distributed_membership_sync;
mod distributed_mempool;
mod distributed_net;
mod distributed_observer;
mod distributed_observer_replay;
mod distributed_provider_cache;
mod distributed_storage;
mod distributed_validation;
mod effect;
mod error;
mod events;
mod governance;
#[cfg(feature = "libp2p")]
mod libp2p_net;
mod m1_builtin_wasm_artifact;
mod manifest;
mod module_store;
mod modules;
mod policy;
mod rules;
mod sandbox;
mod segmenter;
mod signer;
mod snapshot;
mod state;
mod types;
mod util;
mod world;
mod world_event;

#[cfg(test)]
mod tests;

// Re-export all public types

// Types
pub use types::{ActionId, IntentSeq, PatchPath, ProposalId, WorldEventId, WorldTime};

// Agent cell
pub use agent_cell::AgentCell;

// Audit
pub use audit::{AuditCausedBy, AuditEventKind, AuditFilter};

// Effect system
pub use effect::{
    CapabilityGrant, EffectIntent, EffectOrigin, EffectReceipt, OriginKind, ReceiptSignature,
    SignatureAlgorithm,
};

// Error
pub use error::WorldError;

// Events
pub use events::{Action, ActionEnvelope, CausedBy, DomainEvent, RejectReason};

// Governance
pub use governance::{AgentSchedule, GovernanceEvent, Proposal, ProposalDecision, ProposalStatus};

// Manifest
pub use manifest::{
    apply_manifest_patch, diff_manifest, merge_manifest_patches,
    merge_manifest_patches_with_conflicts, ConflictKind, Manifest, ManifestPatch, ManifestPatchOp,
    ManifestUpdate, PatchConflict, PatchMergeResult, PatchOpKind, PatchOpSummary,
};

// Modules
pub use modules::{
    ModuleActivation, ModuleArtifact, ModuleCache, ModuleChangeSet, ModuleDeactivation,
    ModuleEvent, ModuleEventKind, ModuleKind, ModuleLimits, ModuleManifest, ModuleRecord,
    ModuleRegistry, ModuleRole, ModuleSubscription, ModuleSubscriptionStage, ModuleUpgrade,
};

// Blob store
pub use blob_store::{blake3_hex, BlobStore, LocalCasStore};

#[cfg(test)]
pub(crate) use m1_builtin_wasm_artifact::{
    m1_builtin_module_ids_manifest, m1_builtin_wasm_artifact_hash_hex,
    register_m1_builtin_wasm_module_artifact,
};
pub(crate) use m1_builtin_wasm_artifact::{
    m1_builtin_wasm_module_artifact_bytes, M1_BUILTIN_WASM_ARTIFACT_BYTES,
    M1_BUILTIN_WASM_ARTIFACT_SHA256,
};

// Built-in module constants
pub use agent_world_builtin_wasm::{
    M1_AGENT_DEFAULT_MODULE_VERSION, M1_BODY_ACTION_COST_ELECTRICITY, M1_BODY_MODULE_ID,
    M1_MEMORY_MAX_ENTRIES, M1_MEMORY_MODULE_ID, M1_MOBILITY_MODULE_ID, M1_MOVE_RULE_MODULE_ID,
    M1_POWER_HARVEST_BASE_PER_TICK, M1_POWER_HARVEST_DISTANCE_BONUS_CAP,
    M1_POWER_HARVEST_DISTANCE_STEP_CM, M1_POWER_MODULE_VERSION, M1_POWER_STORAGE_CAPACITY,
    M1_POWER_STORAGE_INITIAL_LEVEL, M1_POWER_STORAGE_MOVE_COST_PER_KM,
    M1_RADIATION_POWER_MODULE_ID, M1_SENSOR_MODULE_ID, M1_STORAGE_CARGO_MODULE_ID,
    M1_STORAGE_POWER_MODULE_ID, M1_TRANSFER_RULE_MODULE_ID, M1_VISIBILITY_RULE_MODULE_ID,
};

// Module store
pub use module_store::ModuleStore;

// Distributed network adapter
pub use distributed_net::{
    DistributedNetwork, InMemoryNetwork, NetworkMessage, NetworkRequest, NetworkResponse,
    NetworkSubscription,
};

// Distributed mempool
pub use distributed_mempool::{ActionBatchRules, ActionMempool, ActionMempoolConfig};
// Lease manager
pub use distributed_lease::{LeaseDecision, LeaseManager, LeaseState};

// Distributed client
pub use distributed_client::DistributedClient;

// Distributed DHT adapter
pub use distributed_dht::{
    DistributedDht, InMemoryDht, MembershipDirectorySnapshot, ProviderRecord,
};
// Quorum consensus helpers for head commits
pub use distributed_consensus::{
    ensure_lease_holder_validator, propose_world_head_with_quorum, vote_world_head_with_quorum,
    ConsensusConfig, ConsensusDecision, ConsensusMembershipChange,
    ConsensusMembershipChangeRequest, ConsensusMembershipChangeResult, ConsensusStatus,
    ConsensusVote, HeadConsensusRecord, QuorumConsensus, CONSENSUS_SNAPSHOT_VERSION,
};
// Cached DHT wrapper
pub use distributed_dht_cache::{CachedDht, DhtCacheConfig};
// Distributed index publishing
pub use distributed_index::{
    publish_execution_providers, publish_execution_providers_cached, publish_world_head,
    query_providers, IndexPublishResult,
};
// Distributed index store
pub use distributed_index_store::{DistributedIndexStore, HeadIndexRecord, InMemoryIndexStore};
// Provider cache
pub use distributed_provider_cache::{ProviderCache, ProviderCacheConfig};
// Distributed gateway
pub use distributed_gateway::{ActionGateway, NetworkGateway, SubmitActionReceipt};
// Distributed observer
pub use distributed_observer::{
    HeadFollowReport, HeadSyncReport, HeadSyncResult, ObserverClient, ObserverSubscription,
};
// Distributed membership directory sync
pub use distributed_membership_sync::{
    FileMembershipAuditStore, FileMembershipRevocationAlertDeadLetterStore,
    FileMembershipRevocationAlertRecoveryStore, FileMembershipRevocationAlertSink,
    FileMembershipRevocationCoordinatorStateStore,
    FileMembershipRevocationDeadLetterReplayPolicyAuditStore,
    FileMembershipRevocationDeadLetterReplayPolicyStore,
    FileMembershipRevocationDeadLetterReplayRollbackAlertStateStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertStateStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleStateStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceStateStore,
    FileMembershipRevocationDeadLetterReplayStateStore, FileMembershipRevocationScheduleStateStore,
    InMemoryMembershipAuditStore, InMemoryMembershipRevocationAlertDeadLetterStore,
    InMemoryMembershipRevocationAlertRecoveryStore, InMemoryMembershipRevocationAlertSink,
    InMemoryMembershipRevocationCoordinatorStateStore,
    InMemoryMembershipRevocationDeadLetterReplayPolicyAuditStore,
    InMemoryMembershipRevocationDeadLetterReplayPolicyStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackAlertStateStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertStateStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleStateStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceStateStore,
    InMemoryMembershipRevocationDeadLetterReplayStateStore,
    InMemoryMembershipRevocationScheduleCoordinator,
    InMemoryMembershipRevocationScheduleStateStore, MembershipAuditStore,
    MembershipDirectoryAnnounce, MembershipDirectorySigner, MembershipDirectorySignerKeyring,
    MembershipKeyRevocationAnnounce, MembershipRestoreAuditReport,
    MembershipRevocationAlertAckRetryPolicy, MembershipRevocationAlertDeadLetterReason,
    MembershipRevocationAlertDeadLetterRecord, MembershipRevocationAlertDeadLetterStore,
    MembershipRevocationAlertDedupPolicy, MembershipRevocationAlertDedupState,
    MembershipRevocationAlertDeliveryMetrics, MembershipRevocationAlertPolicy,
    MembershipRevocationAlertRecoveryReport, MembershipRevocationAlertRecoveryStore,
    MembershipRevocationAlertSeverity, MembershipRevocationAlertSink,
    MembershipRevocationAnomalyAlert, MembershipRevocationCheckpointAnnounce,
    MembershipRevocationCoordinatedRecoveryRunReport, MembershipRevocationCoordinatedRunReport,
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
    MembershipRevocationReconcilePolicy, MembershipRevocationReconcileReport,
    MembershipRevocationReconcileSchedulePolicy, MembershipRevocationReconcileScheduleState,
    MembershipRevocationScheduleCoordinator, MembershipRevocationScheduleStateStore,
    MembershipRevocationScheduledRunReport, MembershipRevocationSyncPolicy,
    MembershipRevocationSyncReport, MembershipSnapshotAuditOutcome, MembershipSnapshotAuditRecord,
    MembershipSnapshotRestorePolicy, MembershipSyncClient, MembershipSyncReport,
    MembershipSyncSubscription, NoopMembershipRevocationAlertDeadLetterStore,
    StoreBackedMembershipRevocationScheduleCoordinator,
};
// Distributed observer replay validation
pub use distributed_observer_replay::{replay_validate_head, replay_validate_with_head};
// Distributed bootstrap helpers
pub use distributed_bootstrap::{
    bootstrap_world_from_dht, bootstrap_world_from_head, bootstrap_world_from_head_with_dht,
};
// Distributed head follower
pub use distributed_head_follow::{HeadFollower, HeadUpdateDecision};

// Distributed storage helpers
pub use distributed_storage::{store_execution_result, ExecutionWriteConfig, ExecutionWriteResult};
pub use distributed_validation::{
    assemble_journal, assemble_snapshot, validate_head_update, HeadValidationResult,
};

// Libp2p adapter
#[cfg(feature = "libp2p")]
pub use libp2p_net::{Libp2pNetwork, Libp2pNetworkConfig};

// Segmenter
pub use segmenter::{segment_journal, segment_snapshot, JournalSegmentRef, SegmentConfig};

// Policy
pub use policy::{PolicyDecision, PolicyDecisionRecord, PolicyRule, PolicySet, PolicyWhen};

// Rules
pub use rules::{
    merge_rule_decisions, ActionOverrideRecord, ResourceBalanceError, ResourceDelta, RuleDecision,
    RuleDecisionMergeError, RuleDecisionRecord, RuleVerdict,
};

// Signer
pub use signer::ReceiptSigner;

// Sandbox
pub use sandbox::{
    FixedSandbox, ModuleCallErrorCode, ModuleCallFailure, ModuleCallInput, ModuleCallOrigin,
    ModuleCallRequest, ModuleContext, ModuleEffectIntent, ModuleEmit, ModuleEmitEvent,
    ModuleOutput, ModuleSandbox, ModuleStateUpdate, WasmEngineKind, WasmExecutor,
    WasmExecutorConfig,
};

// Snapshot
pub use snapshot::{
    Journal, RollbackEvent, Snapshot, SnapshotCatalog, SnapshotMeta, SnapshotRecord,
    SnapshotRetentionPolicy,
};

// State
pub use state::WorldState;

// World
pub use world::{M1ScenarioBootstrapConfig, World};

// World event
pub use world_event::{WorldEvent, WorldEventBody};
