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
mod kernel;
mod memory;
mod persist;
mod power;
mod runner;
mod types;
mod world_model;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use agent::{ActionResult, AgentBehavior, AgentDecision};
pub use kernel::{WorldKernel, Observation, ObservedAgent, ObservedLocation};
pub use memory::{
    AgentMemory, LongTermMemory, LongTermMemoryEntry, MemoryEntry, MemoryEntryKind, ShortTermMemory,
};
pub use persist::{PersistError, WorldJournal, WorldSnapshot};
pub use runner::{
    AgentQuota, AgentRunner, AgentStats, AgentTickResult, RateLimitPolicy, RateLimitState,
    RegisteredAgent, RunnerLogEntry, RunnerLogKind, RunnerMetrics, SkippedReason,
};
pub use types::{
    Action, ActionEnvelope, ActionId, AgentId, AssetId, FacilityId, LocationId, ResourceKind,
    ResourceOwner, ResourceStock, StockError, WorldEventId, WorldTime, CM_PER_KM,
    DEFAULT_MOVE_COST_PER_KM_ELECTRICITY, DEFAULT_VISIBILITY_RANGE_CM, JOURNAL_VERSION,
    SNAPSHOT_VERSION,
};
pub use world_model::{Agent, Asset, AssetKind, Location, WorldConfig, WorldModel};

// Re-export power system types
pub use power::{AgentPowerState, AgentPowerStatus, ConsumeReason, PowerConfig, PowerEvent};

// Re-export event types from kernel
pub use kernel::{RejectReason, WorldEvent, WorldEventKind};
