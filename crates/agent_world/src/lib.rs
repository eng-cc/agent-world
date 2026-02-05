pub mod geometry;
pub mod models;
pub mod runtime;
pub mod simulator;

pub use geometry::{
    great_circle_distance_cm, great_circle_distance_cm_with_radius, great_circle_distance_m,
    great_circle_distance_m_with_radius, GeoPos, SPACE_UNIT_CM, WORLD_RADIUS_CM, WORLD_RADIUS_KM,
    WORLD_RADIUS_M,
};
pub use models::{AgentState, RobotBodySpec, DEFAULT_AGENT_HEIGHT_CM};
pub use runtime::{
    Action, ActionEnvelope, ActionId, AgentCell, AgentSchedule, CausedBy, CapabilityGrant,
    AuditCausedBy, AuditEventKind, AuditFilter, ConflictKind, DomainEvent, EffectIntent,
    EffectOrigin, EffectReceipt, GovernanceEvent, IntentSeq, Journal, Manifest, ManifestPatch,
    ManifestPatchOp, ManifestUpdate, ModuleActivation, ModuleArtifact, ModuleCache,
    ModuleChangeSet, ModuleDeactivation, ModuleEvent, ModuleEventKind, ModuleKind, ModuleLimits,
    ModuleManifest, ModuleRegistry, ModuleRecord, ModuleStore, ModuleSubscription, ModuleUpgrade,
    OriginKind, PatchConflict, PatchMergeResult, PatchOpKind, PatchOpSummary, PatchPath,
    PolicyDecision, PolicyDecisionRecord, PolicyRule, PolicySet, PolicyWhen, Proposal,
    ProposalDecision, ProposalId, ProposalStatus, ReceiptSignature, ReceiptSigner, RejectReason,
    RollbackEvent, SignatureAlgorithm, Snapshot, SnapshotCatalog, SnapshotMeta, SnapshotRecord,
    SnapshotRetentionPolicy, World, WorldError, WorldEvent, WorldEventBody, WorldEventId,
    WorldState, WorldTime,
};

pub use runtime::{
    apply_manifest_patch, diff_manifest, merge_manifest_patches, merge_manifest_patches_with_conflicts,
};

pub use runtime::{
    FixedSandbox, ModuleCallErrorCode, ModuleCallFailure, ModuleCallRequest, ModuleEmit,
    ModuleEmitEvent, ModuleEffectIntent, ModuleOutput, ModuleSandbox, ModuleCallInput,
    ModuleCallOrigin, ModuleContext, WasmEngineKind, WasmExecutor, WasmExecutorConfig,
};

// Agent interface (observe → decide → act)
pub use simulator::{
    AgentBehavior, AgentDecision, AgentMemory, AgentQuota, AgentRunner, AgentStats,
    AgentTickResult, ActionResult, LongTermMemory, LongTermMemoryEntry, MemoryEntry,
    MemoryEntryKind, RateLimitPolicy, RateLimitState, RegisteredAgent, RunnerLogEntry,
    RunnerLogKind, RunnerMetrics, ShortTermMemory, SkippedReason,
};

// Power system (M4 social system)
pub use simulator::{
    AgentPowerState, AgentPowerStatus, ConsumeReason, PowerConfig, PowerEvent,
};
