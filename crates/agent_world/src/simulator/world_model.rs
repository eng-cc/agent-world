//! World model entities: Agent, Location, Asset, WorldConfig, WorldModel.

use crate::geometry::GeoPos;
use crate::models::RobotBodySpec;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::power::{AgentPowerStatus, PowerConfig, PowerPlant, PowerStorage};
use super::types::{
    AgentId, AssetId, FacilityId, LocationId, ResourceKind, ResourceStock, CM_PER_KM,
    DEFAULT_MOVE_COST_PER_KM_ELECTRICITY, DEFAULT_VISIBILITY_RANGE_CM,
};
use super::ResourceOwner;

// ============================================================================
// World Entities
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub pos: GeoPos,
    pub body: RobotBodySpec,
    pub location_id: LocationId,
    pub resources: ResourceStock,
    /// Power status for M4 power system.
    pub power: AgentPowerStatus,
}

impl Agent {
    pub fn new(id: impl Into<String>, location_id: impl Into<String>, pos: GeoPos) -> Self {
        Self {
            id: id.into(),
            pos,
            body: RobotBodySpec::default(),
            location_id: location_id.into(),
            resources: ResourceStock::default(),
            power: AgentPowerStatus::default(),
        }
    }

    /// Create a new agent with custom power configuration.
    pub fn new_with_power(
        id: impl Into<String>,
        location_id: impl Into<String>,
        pos: GeoPos,
        power_config: &PowerConfig,
    ) -> Self {
        Self {
            id: id.into(),
            pos,
            body: RobotBodySpec::default(),
            location_id: location_id.into(),
            resources: ResourceStock::default(),
            power: AgentPowerStatus::from_config(power_config),
        }
    }

    /// Check if the agent is shut down due to power depletion.
    pub fn is_shutdown(&self) -> bool {
        self.power.is_shutdown()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub id: LocationId,
    pub name: String,
    pub pos: GeoPos,
    pub resources: ResourceStock,
}

impl Location {
    pub fn new(id: impl Into<String>, name: impl Into<String>, pos: GeoPos) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            pos,
            resources: ResourceStock::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    pub id: AssetId,
    pub owner: ResourceOwner,
    pub kind: AssetKind,
    pub quantity: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AssetKind {
    Resource { kind: ResourceKind },
}

// ============================================================================
// World Model (aggregate)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorldModel {
    pub agents: BTreeMap<AgentId, Agent>,
    pub locations: BTreeMap<LocationId, Location>,
    pub assets: BTreeMap<AssetId, Asset>,
    #[serde(default)]
    pub power_plants: BTreeMap<FacilityId, PowerPlant>,
    #[serde(default)]
    pub power_storages: BTreeMap<FacilityId, PowerStorage>,
}

// ============================================================================
// World Configuration
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldConfig {
    pub visibility_range_cm: i64,
    pub move_cost_per_km_electricity: i64,
    /// Power system configuration.
    pub power: PowerConfig,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
            move_cost_per_km_electricity: DEFAULT_MOVE_COST_PER_KM_ELECTRICITY,
            power: PowerConfig::default(),
        }
    }
}

impl WorldConfig {
    pub fn sanitized(mut self) -> Self {
        if self.visibility_range_cm < 0 {
            self.visibility_range_cm = 0;
        }
        if self.move_cost_per_km_electricity < 0 {
            self.move_cost_per_km_electricity = 0;
        }
        self
    }

    pub fn movement_cost(&self, distance_cm: i64) -> i64 {
        movement_cost(distance_cm, self.move_cost_per_km_electricity)
    }
}

/// Calculate movement cost based on distance and per-km cost.
pub fn movement_cost(distance_cm: i64, per_km_cost: i64) -> i64 {
    if distance_cm <= 0 || per_km_cost <= 0 {
        return 0;
    }
    let km = (distance_cm + CM_PER_KM - 1) / CM_PER_KM;
    km.saturating_mul(per_km_cost)
}
