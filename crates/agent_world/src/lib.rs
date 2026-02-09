pub mod geometry;
pub mod models;
pub mod runtime;
pub mod simulator;
pub mod viewer;

pub use geometry::{
    space_distance_cm, space_distance_m, GeoPos, DEFAULT_CLOUD_DEPTH_CM, DEFAULT_CLOUD_DEPTH_KM,
    DEFAULT_CLOUD_HEIGHT_CM, DEFAULT_CLOUD_HEIGHT_KM, DEFAULT_CLOUD_WIDTH_CM,
    DEFAULT_CLOUD_WIDTH_KM, SPACE_UNIT_CM,
};
pub use models::{AgentState, BodyKernelView, RobotBodySpec, DEFAULT_AGENT_HEIGHT_CM};
pub use runtime::{
    Action, ActionEnvelope, ActionId, AgentCell, AgentSchedule, AuditCausedBy, AuditEventKind,
    AuditFilter, BlobStore, CapabilityGrant, CausedBy, ConflictKind, DistributedClient,
    DistributedDht, DistributedNetwork, DomainEvent, EffectIntent, EffectOrigin, EffectReceipt,
    GovernanceEvent, InMemoryDht, InMemoryNetwork, IntentSeq, Journal, JournalSegmentRef,
    LocalCasStore, Manifest, ManifestPatch, ManifestPatchOp, ManifestUpdate, ModuleActivation,
    ModuleArtifact, ModuleCache, ModuleChangeSet, ModuleDeactivation, ModuleEvent, ModuleEventKind,
    ModuleKind, ModuleLimits, ModuleManifest, ModuleRecord, ModuleRegistry, ModuleRole,
    ModuleStore, ModuleSubscription, ModuleSubscriptionStage, ModuleUpgrade, NetworkMessage,
    NetworkRequest, NetworkResponse, NetworkSubscription, OriginKind, PatchConflict,
    PatchMergeResult, PatchOpKind, PatchOpSummary, PatchPath, PolicyDecision, PolicyDecisionRecord,
    PolicyRule, PolicySet, PolicyWhen, Proposal, ProposalDecision, ProposalId, ProposalStatus,
    ProviderRecord, ReceiptSignature, ReceiptSigner, RejectReason, ResourceBalanceError,
    ResourceDelta, RollbackEvent, RuleDecision, RuleDecisionMergeError, RuleVerdict, SegmentConfig,
    SignatureAlgorithm, Snapshot, SnapshotCatalog, SnapshotMeta, SnapshotRecord,
    SnapshotRetentionPolicy, World, WorldError, WorldEvent, WorldEventBody, WorldEventId,
    WorldState, WorldTime,
};

#[cfg(feature = "libp2p")]
pub use runtime::{Libp2pNetwork, Libp2pNetworkConfig};

pub use runtime::{
    apply_manifest_patch, blake3_hex, diff_manifest, merge_manifest_patches,
    merge_manifest_patches_with_conflicts, merge_rule_decisions, segment_journal, segment_snapshot,
};

pub use runtime::{
    FixedSandbox, ModuleCallErrorCode, ModuleCallFailure, ModuleCallInput, ModuleCallOrigin,
    ModuleCallRequest, ModuleContext, ModuleEffectIntent, ModuleEmit, ModuleEmitEvent,
    ModuleOutput, ModuleSandbox, WasmEngineKind, WasmExecutor, WasmExecutorConfig,
};

// Agent interface (observe → decide → act)
pub use simulator::{
    ActionResult, AgentBehavior, AgentDecision, AgentDecisionTrace, AgentMemory, AgentQuota,
    AgentRunner, AgentStats, AgentTickResult, ChunkRuntimeConfig, LlmAgentBehavior,
    LlmAgentBuildError, LlmAgentConfig, LlmClientError, LongTermMemory, LongTermMemoryEntry,
    MemoryEntry, MemoryEntryKind, OpenAiChatCompletionClient, RateLimitPolicy, RateLimitState,
    RegisteredAgent, RunnerLogEntry, RunnerLogKind, RunnerMetrics, ShortTermMemory, SkippedReason,
    DEFAULT_CONFIG_FILE_NAME, DEFAULT_LLM_LONG_TERM_GOAL, DEFAULT_LLM_MAX_MODULE_CALLS,
    DEFAULT_LLM_SHORT_TERM_GOAL, DEFAULT_LLM_SYSTEM_PROMPT, ENV_LLM_API_KEY, ENV_LLM_BASE_URL,
    ENV_LLM_LONG_TERM_GOAL, ENV_LLM_MAX_MODULE_CALLS, ENV_LLM_MODEL, ENV_LLM_SHORT_TERM_GOAL,
    ENV_LLM_SYSTEM_PROMPT, ENV_LLM_TIMEOUT_MS,
};

// World initialization
pub use simulator::{
    build_world_model, chunk_bounds, chunk_coord_of, chunk_grid_dims, chunk_seed,
    initialize_kernel, AgentSpawnConfig, AsteroidFragmentInitConfig, ChunkBounds, ChunkCoord,
    ChunkResourceBudget, ChunkState, CompoundComposition, CuboidSizeCm, ElementBudgetError,
    FragmentBlock, FragmentBlockField, FragmentCompoundKind, FragmentPhysicalProfile,
    FragmentResourceBudget, FragmentResourceError, GridPosCm, LocationSeedConfig,
    OriginLocationConfig, PowerPlantSeedConfig, PowerStorageSeedConfig, WorldInitConfig,
    WorldInitError, WorldInitReport, WorldScenario, CHUNK_SIZE_X_CM, CHUNK_SIZE_Y_CM,
    CHUNK_SIZE_Z_CM, CM3_PER_M3, DEFAULT_ELEMENT_RECOVERABILITY_PPM, MIN_BLOCK_EDGE_CM,
};

pub use simulator::{
    infer_element_ppm, mass_grams_from_volume_density, synthesize_fragment_budget,
    synthesize_fragment_profile, ElementComposition, FragmentElementKind,
};

// Power system (M4 social system)
pub use simulator::{AgentPowerState, AgentPowerStatus, ConsumeReason, PowerConfig, PowerEvent};
