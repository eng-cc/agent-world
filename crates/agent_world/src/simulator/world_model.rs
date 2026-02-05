//! World model entities: Agent, Location, Asset, WorldConfig, WorldModel.

use crate::geometry::{
    GeoPos, DEFAULT_CLOUD_DEPTH_CM, DEFAULT_CLOUD_HEIGHT_CM, DEFAULT_CLOUD_WIDTH_CM,
};
use crate::models::RobotBodySpec;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::power::{AgentPowerStatus, PowerConfig, PowerPlant, PowerStorage};
use super::types::{
    AgentId, AssetId, FacilityId, LocationId, LocationProfile, MaterialKind, ResourceKind,
    ResourceStock, CM_PER_KM, DEFAULT_MOVE_COST_PER_KM_ELECTRICITY, DEFAULT_VISIBILITY_RANGE_CM,
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
    /// Thermal status (heat accumulation).
    #[serde(default)]
    pub thermal: ThermalStatus,
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
            thermal: ThermalStatus::default(),
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
            thermal: ThermalStatus::default(),
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
    pub profile: LocationProfile,
    pub resources: ResourceStock,
}

impl Location {
    pub fn new(id: impl Into<String>, name: impl Into<String>, pos: GeoPos) -> Self {
        Self::new_with_profile(id, name, pos, LocationProfile::default())
    }

    pub fn new_with_profile(
        id: impl Into<String>,
        name: impl Into<String>,
        pos: GeoPos,
        profile: LocationProfile,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            pos,
            profile,
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
#[serde(default)]
pub struct WorldConfig {
    pub visibility_range_cm: i64,
    pub move_cost_per_km_electricity: i64,
    pub space: SpaceConfig,
    /// Power system configuration.
    pub power: PowerConfig,
    /// Physics configuration (radiation/thermal/erosion).
    pub physics: PhysicsConfig,
    /// Dust cloud generation configuration.
    pub dust: DustConfig,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
            move_cost_per_km_electricity: DEFAULT_MOVE_COST_PER_KM_ELECTRICITY,
            space: SpaceConfig::default(),
            power: PowerConfig::default(),
            physics: PhysicsConfig::default(),
            dust: DustConfig::default(),
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
        self.space = self.space.sanitized();
        if self.power.transfer_loss_per_km_bps < 0 {
            self.power.transfer_loss_per_km_bps = 0;
        }
        if self.power.transfer_max_distance_km < 0 {
            self.power.transfer_max_distance_km = 0;
        }
        self.physics = self.physics.sanitized();
        self.dust = self.dust.sanitized();
        self
    }

    pub fn movement_cost(&self, distance_cm: i64) -> i64 {
        movement_cost(distance_cm, self.move_cost_per_km_electricity)
    }
}

// ============================================================================
// Space Configuration
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SpaceConfig {
    pub width_cm: i64,
    pub depth_cm: i64,
    pub height_cm: i64,
}

// ============================================================================
// Physics Configuration
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PhysicsConfig {
    pub time_step_s: i64,
    pub power_unit_j: i64,
    pub radiation_floor: i64,
    pub radiation_decay_k: f64,
    pub max_harvest_per_tick: i64,
    pub thermal_capacity: i64,
    pub thermal_dissipation: i64,
    pub heat_factor: i64,
    pub erosion_rate: f64,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            time_step_s: 10,
            power_unit_j: 1_000,
            radiation_floor: 1,
            radiation_decay_k: 1e-6,
            max_harvest_per_tick: 50,
            thermal_capacity: 100,
            thermal_dissipation: 5,
            heat_factor: 1,
            erosion_rate: 1e-6,
        }
    }
}

impl PhysicsConfig {
    pub fn sanitized(mut self) -> Self {
        if self.time_step_s < 0 {
            self.time_step_s = 0;
        }
        if self.power_unit_j < 0 {
            self.power_unit_j = 0;
        }
        if self.radiation_floor < 0 {
            self.radiation_floor = 0;
        }
        if self.radiation_decay_k < 0.0 {
            self.radiation_decay_k = 0.0;
        }
        if self.max_harvest_per_tick < 0 {
            self.max_harvest_per_tick = 0;
        }
        if self.thermal_capacity < 0 {
            self.thermal_capacity = 0;
        }
        if self.thermal_dissipation < 0 {
            self.thermal_dissipation = 0;
        }
        if self.heat_factor < 0 {
            self.heat_factor = 0;
        }
        if self.erosion_rate < 0.0 {
            self.erosion_rate = 0.0;
        }
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ThermalStatus {
    pub heat: i64,
}

// ============================================================================
// Dust Cloud Generator Configuration
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DustConfig {
    pub base_density_per_km3: f64,
    pub voxel_size_km: i64,
    pub cluster_noise: f64,
    pub layer_scale_height_km: f64,
    pub size_powerlaw_q: f64,
    pub radius_min_cm: i64,
    pub radius_max_cm: i64,
    pub material_weights: MaterialWeights,
}

impl Default for DustConfig {
    fn default() -> Self {
        Self {
            base_density_per_km3: 0.001,
            voxel_size_km: 10,
            cluster_noise: 0.5,
            layer_scale_height_km: 2.0,
            size_powerlaw_q: 3.0,
            radius_min_cm: 10,
            radius_max_cm: 10_000,
            material_weights: MaterialWeights::default(),
        }
    }
}

impl DustConfig {
    pub fn sanitized(mut self) -> Self {
        if self.base_density_per_km3 < 0.0 {
            self.base_density_per_km3 = 0.0;
        }
        if self.voxel_size_km <= 0 {
            self.voxel_size_km = 1;
        }
        if self.cluster_noise < 0.0 {
            self.cluster_noise = 0.0;
        }
        if self.layer_scale_height_km < 0.0 {
            self.layer_scale_height_km = 0.0;
        }
        if self.size_powerlaw_q <= 0.0 {
            self.size_powerlaw_q = 1.0;
        }
        if self.radius_min_cm < 0 {
            self.radius_min_cm = 0;
        }
        if self.radius_max_cm < self.radius_min_cm {
            self.radius_max_cm = self.radius_min_cm;
        }
        self.material_weights = self.material_weights.sanitized();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MaterialWeights {
    pub silicate: u32,
    pub metal: u32,
    pub ice: u32,
    pub carbon: u32,
    pub composite: u32,
}

impl Default for MaterialWeights {
    fn default() -> Self {
        Self {
            silicate: 50,
            metal: 20,
            ice: 15,
            carbon: 10,
            composite: 5,
        }
    }
}

impl MaterialWeights {
    pub fn sanitized(mut self) -> Self {
        if self.silicate == 0
            && self.metal == 0
            && self.ice == 0
            && self.carbon == 0
            && self.composite == 0
        {
            self.silicate = 1;
        }
        self
    }

    pub fn total(&self) -> u32 {
        self.silicate + self.metal + self.ice + self.carbon + self.composite
    }

    pub fn pick(&self, roll: u32) -> MaterialKind {
        let mut acc = self.silicate;
        if roll < acc {
            return MaterialKind::Silicate;
        }
        acc += self.metal;
        if roll < acc {
            return MaterialKind::Metal;
        }
        acc += self.ice;
        if roll < acc {
            return MaterialKind::Ice;
        }
        acc += self.carbon;
        if roll < acc {
            return MaterialKind::Carbon;
        }
        MaterialKind::Composite
    }
}

impl Default for SpaceConfig {
    fn default() -> Self {
        Self {
            width_cm: DEFAULT_CLOUD_WIDTH_CM,
            depth_cm: DEFAULT_CLOUD_DEPTH_CM,
            height_cm: DEFAULT_CLOUD_HEIGHT_CM,
        }
    }
}

impl SpaceConfig {
    pub fn sanitized(mut self) -> Self {
        if self.width_cm < 0 {
            self.width_cm = 0;
        }
        if self.depth_cm < 0 {
            self.depth_cm = 0;
        }
        if self.height_cm < 0 {
            self.height_cm = 0;
        }
        self
    }

    pub fn contains(&self, pos: GeoPos) -> bool {
        let max_x = self.width_cm as f64;
        let max_y = self.depth_cm as f64;
        let max_z = self.height_cm as f64;
        pos.x_cm >= 0.0
            && pos.x_cm <= max_x
            && pos.y_cm >= 0.0
            && pos.y_cm <= max_y
            && pos.z_cm >= 0.0
            && pos.z_cm <= max_z
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
