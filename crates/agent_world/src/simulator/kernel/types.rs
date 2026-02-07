use crate::geometry::GeoPos;
use serde::{Deserialize, Serialize};

use super::super::chunking::ChunkCoord;
use super::super::module_visual::ModuleVisualEntity;
use super::super::power::PowerEvent;
use super::super::types::{
    AgentId, ChunkResourceBudget, FacilityId, LocationId, LocationProfile, ResourceKind,
    ResourceOwner, WorldEventId, WorldTime,
};

// ============================================================================
// Observation Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    pub time: WorldTime,
    pub agent_id: AgentId,
    pub pos: GeoPos,
    pub visibility_range_cm: i64,
    pub visible_agents: Vec<ObservedAgent>,
    pub visible_locations: Vec<ObservedLocation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedAgent {
    pub agent_id: AgentId,
    pub location_id: LocationId,
    pub pos: GeoPos,
    pub distance_cm: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedLocation {
    pub location_id: LocationId,
    pub name: String,
    pub pos: GeoPos,
    pub profile: LocationProfile,
    pub distance_cm: i64,
}

// ============================================================================
// Event Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldEvent {
    pub id: WorldEventId,
    pub time: WorldTime,
    pub kind: WorldEventKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WorldEventKind {
    LocationRegistered {
        location_id: LocationId,
        name: String,
        pos: GeoPos,
        profile: LocationProfile,
    },
    AgentRegistered {
        agent_id: AgentId,
        location_id: LocationId,
        pos: GeoPos,
    },
    AgentMoved {
        agent_id: AgentId,
        from: LocationId,
        to: LocationId,
        distance_cm: i64,
        electricity_cost: i64,
    },
    ResourceTransferred {
        from: ResourceOwner,
        to: ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    },
    RadiationHarvested {
        agent_id: AgentId,
        location_id: LocationId,
        amount: i64,
        available: i64,
    },
    CompoundRefined {
        owner: ResourceOwner,
        compound_mass_g: i64,
        electricity_cost: i64,
        hardware_output: i64,
    },
    ModuleVisualEntityUpserted {
        entity: ModuleVisualEntity,
    },
    ModuleVisualEntityRemoved {
        entity_id: String,
    },
    ChunkGenerated {
        coord: ChunkCoord,
        seed: u64,
        fragment_count: u32,
        block_count: u32,
        chunk_budget: ChunkResourceBudget,
        cause: ChunkGenerationCause,
    },
    ActionRejected {
        reason: RejectReason,
    },
    // Power system events
    Power(PowerEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkGenerationCause {
    Init,
    Observe,
    Action,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum RejectReason {
    AgentAlreadyExists {
        agent_id: AgentId,
    },
    AgentNotFound {
        agent_id: AgentId,
    },
    LocationAlreadyExists {
        location_id: LocationId,
    },
    LocationNotFound {
        location_id: LocationId,
    },
    FacilityAlreadyExists {
        facility_id: FacilityId,
    },
    FacilityNotFound {
        facility_id: FacilityId,
    },
    AgentAlreadyAtLocation {
        agent_id: AgentId,
        location_id: LocationId,
    },
    MoveDistanceExceeded {
        distance_cm: i64,
        max_distance_cm: i64,
    },
    MoveSpeedExceeded {
        required_speed_cm_per_s: i64,
        max_speed_cm_per_s: i64,
        time_step_s: i64,
    },
    InvalidAmount {
        amount: i64,
    },
    InsufficientResource {
        owner: ResourceOwner,
        kind: ResourceKind,
        requested: i64,
        available: i64,
    },
    LocationTransferNotAllowed {
        from: LocationId,
        to: LocationId,
    },
    AgentNotAtLocation {
        agent_id: AgentId,
        location_id: LocationId,
    },
    AgentsNotCoLocated {
        agent_id: AgentId,
        other_agent_id: AgentId,
    },
    AgentShutdown {
        agent_id: AgentId,
    },
    PositionOutOfBounds {
        pos: GeoPos,
    },
    ChunkGenerationFailed {
        x: i32,
        y: i32,
        z: i32,
    },
    RadiationUnavailable {
        location_id: LocationId,
    },
    ThermalOverload {
        heat: i64,
        capacity: i64,
    },
    PowerTransferDistanceExceeded {
        distance_km: i64,
        max_distance_km: i64,
    },
    PowerTransferLossExceedsAmount {
        amount: i64,
        loss: i64,
    },
}
