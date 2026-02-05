pub mod geometry;
pub mod models;
pub mod runtime;
pub mod simulator;
pub mod viewer;

pub use geometry::{
    space_distance_cm, space_distance_m, GeoPos, SPACE_UNIT_CM, DEFAULT_CLOUD_DEPTH_CM,
    DEFAULT_CLOUD_DEPTH_KM, DEFAULT_CLOUD_HEIGHT_CM, DEFAULT_CLOUD_HEIGHT_KM,
    DEFAULT_CLOUD_WIDTH_CM, DEFAULT_CLOUD_WIDTH_KM,
};
pub use models::{AgentState, RobotBodySpec, DEFAULT_AGENT_HEIGHT_CM};
pub use runtime::{
    Action, ActionEnvelope, ActionId, AgentCell, AgentSchedule, CausedBy, CapabilityGrant,
    AuditCausedBy, AuditEventKind, AuditFilter, ConflictKind, DomainEvent, EffectIntent,
    EffectOrigin, EffectReceipt, GovernanceEvent, IntentSeq, Journal, Manifest, ManifestPatch,
    ManifestPatchOp, ManifestUpdate, ModuleActivation, ModuleArtifact, ModuleCache,
    ModuleChangeSet, ModuleDeactivation, ModuleEvent, ModuleEventKind, ModuleKind, ModuleLimits,
    ModuleManifest, ModuleRegistry, ModuleRecord, ModuleRole, ModuleStore, BlobStore, LocalCasStore,
    ModuleSubscription,
    ModuleSubscriptionStage, ModuleUpgrade,
    OriginKind, PatchConflict, PatchMergeResult, PatchOpKind, PatchOpSummary, PatchPath,
    PolicyDecision, PolicyDecisionRecord, PolicyRule, PolicySet, PolicyWhen, Proposal,
    ProposalDecision, ProposalId, ProposalStatus, ReceiptSignature, ReceiptSigner, RejectReason,
    RollbackEvent, RuleDecision, RuleDecisionMergeError, RuleVerdict, ResourceDelta,
    SignatureAlgorithm, Snapshot, SnapshotCatalog, SnapshotMeta, SnapshotRecord,
    SnapshotRetentionPolicy, World, WorldError, WorldEvent, WorldEventBody, WorldEventId,
    WorldState, WorldTime,
};

pub use runtime::{
    apply_manifest_patch, blake3_hex, diff_manifest, merge_manifest_patches,
    merge_manifest_patches_with_conflicts,
    merge_rule_decisions,
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

// World initialization
pub use simulator::{
    build_world_model, initialize_kernel, AgentSpawnConfig, DustInitConfig, LocationSeedConfig,
    OriginLocationConfig, PowerPlantSeedConfig, PowerStorageSeedConfig, WorldInitConfig,
    WorldInitError, WorldInitReport, WorldScenario,
};

// Power system (M4 social system)
pub use simulator::{
    AgentPowerState, AgentPowerStatus, ConsumeReason, PowerConfig, PowerEvent,
};
