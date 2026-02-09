//! World Simulator module - provides the simulation kernel, agent interface, and world model.
//!
//! This module is organized into submodules:
//! - `types`: Core type definitions (IDs, constants, resources)
//! - `world_model`: World entities (Agent, Location, Asset, WorldModel)
//! - `kernel`: WorldKernel implementation (time, events, actions)
//! - `persist`: Snapshot, Journal, and persistence utilities
//! - `agent`: Agent interface trait and decision types
//! - `memory`: Agent memory system (short-term, long-term)
//! - `runner`: AgentRunner, quota, rate limiting, metrics
//! - `power`: Power system (M4 social system)

mod agent;
mod asteroid_fragment;
mod chunking;
mod fragment_physics;
mod init;
mod init_module_visual;
mod kernel;
mod llm_agent;
mod memory;
mod module_visual;
mod persist;
mod power;
mod runner;
mod scenario;
mod types;
mod world_model;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use agent::{
    ActionResult, AgentBehavior, AgentDecision, AgentDecisionTrace, LlmDecisionDiagnostics,
    LlmEffectIntentTrace, LlmEffectReceiptTrace, LlmPromptSectionTrace, LlmStepTrace,
};
pub use asteroid_fragment::generate_fragments;
pub use chunking::{
    chunk_bounds, chunk_coord_of, chunk_coords, chunk_grid_dims, chunk_seed, ChunkBounds,
    ChunkCoord, CHUNK_SIZE_X_CM, CHUNK_SIZE_Y_CM, CHUNK_SIZE_Z_CM,
};
pub use fragment_physics::{
    infer_element_ppm, mass_grams_from_volume_density, synthesize_fragment_budget,
    synthesize_fragment_profile, CompoundComposition, CuboidSizeCm, FragmentBlock,
    FragmentBlockField, FragmentCompoundKind, FragmentPhysicalProfile, GridPosCm, CM3_PER_M3,
    MIN_BLOCK_EDGE_CM,
};
pub use init::{
    build_world_model, initialize_kernel, AgentSpawnConfig, AsteroidFragmentInitConfig,
    LocationSeedConfig, OriginLocationConfig, PowerPlantSeedConfig, PowerStorageSeedConfig,
    WorldInitConfig, WorldInitError, WorldInitReport,
};
pub use kernel::ChunkRuntimeConfig;
pub use kernel::{Observation, ObservedAgent, ObservedLocation, WorldKernel};
pub use llm_agent::{
    LlmAgentBehavior, LlmAgentBuildError, LlmAgentConfig, LlmClientError,
    OpenAiChatCompletionClient, DEFAULT_CONFIG_FILE_NAME, DEFAULT_LLM_LONG_TERM_GOAL,
    DEFAULT_LLM_MAX_DECISION_STEPS, DEFAULT_LLM_MAX_MODULE_CALLS, DEFAULT_LLM_MAX_REPAIR_ROUNDS,
    DEFAULT_LLM_PROMPT_MAX_HISTORY_ITEMS, DEFAULT_LLM_SHORT_TERM_GOAL, DEFAULT_LLM_SYSTEM_PROMPT,
    ENV_LLM_API_KEY, ENV_LLM_BASE_URL, ENV_LLM_LONG_TERM_GOAL, ENV_LLM_MAX_DECISION_STEPS,
    ENV_LLM_MAX_MODULE_CALLS, ENV_LLM_MAX_REPAIR_ROUNDS, ENV_LLM_MODEL,
    ENV_LLM_PROMPT_MAX_HISTORY_ITEMS, ENV_LLM_PROMPT_PROFILE, ENV_LLM_SHORT_TERM_GOAL,
    ENV_LLM_SYSTEM_PROMPT, ENV_LLM_TIMEOUT_MS,
};
pub use memory::{
    AgentMemory, LongTermMemory, LongTermMemoryEntry, MemoryEntry, MemoryEntryKind, ShortTermMemory,
};
pub use module_visual::{ModuleVisualAnchor, ModuleVisualEntity};
pub use persist::{PersistError, WorldJournal, WorldSnapshot};
pub use runner::{
    AgentQuota, AgentRunner, AgentStats, AgentTickResult, RateLimitPolicy, RateLimitState,
    RegisteredAgent, RunnerLogEntry, RunnerLogKind, RunnerMetrics, SkippedReason,
};
pub use scenario::{ScenarioSpecError, WorldScenario, WorldScenarioSpec};
pub use types::{
    Action, ActionEnvelope, ActionId, AgentId, AssetId, ChunkResourceBudget, ElementBudgetError,
    ElementComposition, FacilityId, FragmentElementKind, FragmentResourceBudget, LocationId,
    LocationProfile, MaterialKind, ResourceKind, ResourceOwner, ResourceStock, StockError,
    WorldEventId, WorldTime, CHUNK_GENERATION_SCHEMA_VERSION, CM_PER_KM,
    DEFAULT_ELEMENT_RECOVERABILITY_PPM, DEFAULT_MOVE_COST_PER_KM_ELECTRICITY,
    DEFAULT_VISIBILITY_RANGE_CM, JOURNAL_VERSION, PPM_BASE, SNAPSHOT_VERSION,
};
pub use world_model::{
    physics_parameter_specs, Agent, Asset, AssetKind, AsteroidFragmentConfig, BoundaryReservation,
    ChunkState, EconomyConfig, FragmentResourceError, Location, MaterialRadiationFactors,
    MaterialWeights, PhysicsConfig, PhysicsParameterSpec, SpaceConfig, ThermalStatus, WorldConfig,
    WorldModel,
};

// Re-export power system types
pub use power::{
    AgentPowerState, AgentPowerStatus, ConsumeReason, PlantStatus, PowerConfig, PowerEvent,
    PowerPlant, PowerStorage,
};

// Re-export event types from kernel
pub use kernel::{ChunkGenerationCause, RejectReason, WorldEvent, WorldEventKind};
