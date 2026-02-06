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
mod kernel;
mod memory;
mod persist;
mod power;
mod runner;
mod scenario;
mod types;
mod world_model;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use agent::{ActionResult, AgentBehavior, AgentDecision};
pub use asteroid_fragment::generate_fragments;
pub use chunking::{
    chunk_bounds, chunk_coord_of, chunk_coords, chunk_grid_dims, chunk_seed, ChunkBounds, ChunkCoord,
    CHUNK_SIZE_X_CM, CHUNK_SIZE_Y_CM, CHUNK_SIZE_Z_CM,
};
pub use fragment_physics::{
    infer_element_ppm, mass_grams_from_volume_density, CompoundComposition, CuboidSizeCm,
    FragmentBlock, FragmentBlockField, FragmentCompoundKind, FragmentPhysicalProfile, GridPosCm,
    synthesize_fragment_budget, synthesize_fragment_profile, CM3_PER_M3, MIN_BLOCK_EDGE_CM,
};
pub use init::{
    build_world_model, initialize_kernel, AgentSpawnConfig, AsteroidFragmentInitConfig, LocationSeedConfig,
    OriginLocationConfig, PowerPlantSeedConfig, PowerStorageSeedConfig, WorldInitConfig,
    WorldInitError, WorldInitReport,
};
pub use scenario::{ScenarioSpecError, WorldScenario, WorldScenarioSpec};
pub use kernel::{WorldKernel, Observation, ObservedAgent, ObservedLocation};
pub use kernel::ChunkRuntimeConfig;
pub use memory::{
    AgentMemory, LongTermMemory, LongTermMemoryEntry, MemoryEntry, MemoryEntryKind, ShortTermMemory,
};
pub use persist::{PersistError, WorldJournal, WorldSnapshot};
pub use runner::{
    AgentQuota, AgentRunner, AgentStats, AgentTickResult, RateLimitPolicy, RateLimitState,
    RegisteredAgent, RunnerLogEntry, RunnerLogKind, RunnerMetrics, SkippedReason,
};
pub use types::{
    Action, ActionEnvelope, ActionId, AgentId, AssetId, ChunkResourceBudget,
    DEFAULT_ELEMENT_RECOVERABILITY_PPM, ElementBudgetError, ElementComposition, FacilityId,
    FragmentElementKind, FragmentResourceBudget, LocationId, ResourceKind, LocationProfile, MaterialKind, ResourceOwner,
    ResourceStock, StockError, WorldEventId,
    WorldTime, CM_PER_KM, DEFAULT_MOVE_COST_PER_KM_ELECTRICITY, DEFAULT_VISIBILITY_RANGE_CM,
    CHUNK_GENERATION_SCHEMA_VERSION, JOURNAL_VERSION, PPM_BASE, SNAPSHOT_VERSION,
};
pub use world_model::{
    Agent, Asset, AssetKind, AsteroidFragmentConfig, BoundaryReservation, ChunkState,
    FragmentResourceError, Location, MaterialWeights,
    PhysicsConfig, SpaceConfig, ThermalStatus, WorldConfig, WorldModel,
};

// Re-export power system types
pub use power::{
    AgentPowerState, AgentPowerStatus, ConsumeReason, PlantStatus, PowerConfig, PowerEvent,
    PowerPlant, PowerStorage,
};

// Re-export event types from kernel
pub use kernel::{ChunkGenerationCause, RejectReason, WorldEvent, WorldEventKind};
