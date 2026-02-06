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
pub use models::{AgentState, BodyKernelView, RobotBodySpec, DEFAULT_AGENT_HEIGHT_CM};
pub use runtime::{
    Action, ActionEnvelope, ActionId, AgentCell, AgentSchedule, CausedBy, CapabilityGrant,
    AuditCausedBy, AuditEventKind, AuditFilter, ConflictKind, DomainEvent, EffectIntent,
    EffectOrigin, EffectReceipt, GovernanceEvent, IntentSeq, Journal, Manifest, ManifestPatch,
    ManifestPatchOp, ManifestUpdate, ModuleActivation, ModuleArtifact, ModuleCache,
    ModuleChangeSet, ModuleDeactivation, ModuleEvent, ModuleEventKind, ModuleKind, ModuleLimits,
    ModuleManifest, ModuleRegistry, ModuleRecord, ModuleRole, ModuleStore, BlobStore, LocalCasStore,
    ModuleSubscription,
    ModuleSubscriptionStage, ModuleUpgrade,
    DistributedNetwork, InMemoryNetwork, NetworkMessage, NetworkRequest, NetworkResponse,
    NetworkSubscription, DistributedClient, DistributedDht, InMemoryDht, ProviderRecord,
    OriginKind, PatchConflict, PatchMergeResult, PatchOpKind, PatchOpSummary, PatchPath,
    PolicyDecision, PolicyDecisionRecord, PolicyRule, PolicySet, PolicyWhen, Proposal,
    ProposalDecision, ProposalId, ProposalStatus, ReceiptSignature, ReceiptSigner, RejectReason,
    RollbackEvent, RuleDecision, RuleDecisionMergeError, RuleVerdict, ResourceBalanceError,
    ResourceDelta,
    SignatureAlgorithm, Snapshot, SnapshotCatalog, SnapshotMeta, SnapshotRecord, SegmentConfig,
    JournalSegmentRef,
    SnapshotRetentionPolicy, World, WorldError, WorldEvent, WorldEventBody, WorldEventId,
    WorldState, WorldTime,
};

#[cfg(feature = "libp2p")]
pub use runtime::{Libp2pNetwork, Libp2pNetworkConfig};

pub use runtime::{
    apply_manifest_patch, blake3_hex, diff_manifest, merge_manifest_patches,
    merge_manifest_patches_with_conflicts,
    merge_rule_decisions, segment_journal, segment_snapshot,
};

pub use runtime::{
    FixedSandbox, ModuleCallErrorCode, ModuleCallFailure, ModuleCallRequest, ModuleEmit,
    ModuleEmitEvent, ModuleEffectIntent, ModuleOutput, ModuleSandbox, ModuleCallInput,
    ModuleCallOrigin, ModuleContext, WasmEngineKind, WasmExecutor, WasmExecutorConfig,
};

// Agent interface (observe → decide → act)
pub use simulator::{
    AgentBehavior, AgentDecision, AgentMemory, AgentQuota, AgentRunner, AgentStats,
    AgentTickResult, ActionResult, ChunkRuntimeConfig, LongTermMemory, LongTermMemoryEntry, MemoryEntry,
    MemoryEntryKind, RateLimitPolicy, RateLimitState, RegisteredAgent, RunnerLogEntry,
    RunnerLogKind, RunnerMetrics, ShortTermMemory, SkippedReason,
};

// World initialization
pub use simulator::{
    build_world_model, chunk_bounds, chunk_coord_of, chunk_grid_dims, chunk_seed,
    initialize_kernel, AgentSpawnConfig, AsteroidFragmentInitConfig, ChunkBounds, ChunkCoord,
    ChunkState,
    CompoundComposition, CuboidSizeCm,
    FragmentBlock, FragmentBlockField, FragmentCompoundKind, FragmentPhysicalProfile, GridPosCm,
    LocationSeedConfig, OriginLocationConfig, PowerPlantSeedConfig, PowerStorageSeedConfig,
    WorldInitConfig, WorldInitError, WorldInitReport, WorldScenario, CHUNK_SIZE_X_CM,
    CM3_PER_M3,
    MIN_BLOCK_EDGE_CM,
    CHUNK_SIZE_Y_CM, CHUNK_SIZE_Z_CM,
};

pub use simulator::{
    infer_element_ppm, mass_grams_from_volume_density, synthesize_fragment_profile,
    ElementComposition, FragmentElementKind,
};

// Power system (M4 social system)
pub use simulator::{
    AgentPowerState, AgentPowerStatus, ConsumeReason, PowerConfig, PowerEvent,
};
