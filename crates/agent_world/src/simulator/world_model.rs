//! World model entities: Agent, Location, Asset, WorldConfig, WorldModel.

use crate::geometry::{
    GeoPos, DEFAULT_CLOUD_DEPTH_CM, DEFAULT_CLOUD_HEIGHT_CM, DEFAULT_CLOUD_WIDTH_CM,
};
use crate::models::RobotBodySpec;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;

use super::chunking::{chunk_coord_of, ChunkCoord};
use super::fragment_physics::FragmentPhysicalProfile;
use super::module_visual::ModuleVisualEntity;
use super::power::{AgentPowerStatus, PowerConfig, PowerPlant, PowerStorage};
use super::types::{
    AgentId, AssetId, ChunkResourceBudget, ElementBudgetError, FacilityId, FragmentElementKind,
    FragmentResourceBudget, LocationId, LocationProfile, MaterialKind, ResourceKind, ResourceStock,
    CM_PER_KM, DEFAULT_MOVE_COST_PER_KM_ELECTRICITY, DEFAULT_VISIBILITY_RANGE_CM, PPM_BASE,
};
use super::ResourceOwner;

const MOVE_COST_REFERENCE_TIME_STEP_S: i64 = 10;
const MOVE_COST_REFERENCE_POWER_UNIT_J: i64 = 1_000;

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fragment_profile: Option<FragmentPhysicalProfile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fragment_budget: Option<FragmentResourceBudget>,
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
            fragment_profile: None,
            fragment_budget: None,
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
    pub module_visual_entities: BTreeMap<String, ModuleVisualEntity>,
    #[serde(default)]
    pub power_plants: BTreeMap<FacilityId, PowerPlant>,
    #[serde(default)]
    pub power_storages: BTreeMap<FacilityId, PowerStorage>,
    #[serde(
        default,
        serialize_with = "serialize_chunk_states",
        deserialize_with = "deserialize_chunk_states"
    )]
    pub chunks: BTreeMap<ChunkCoord, ChunkState>,
    #[serde(
        default,
        serialize_with = "serialize_chunk_resource_budgets",
        deserialize_with = "deserialize_chunk_resource_budgets"
    )]
    pub chunk_resource_budgets: BTreeMap<ChunkCoord, ChunkResourceBudget>,
    #[serde(
        default,
        serialize_with = "serialize_chunk_boundary_reservations",
        deserialize_with = "deserialize_chunk_boundary_reservations"
    )]
    pub chunk_boundary_reservations: BTreeMap<ChunkCoord, Vec<BoundaryReservation>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChunkState {
    #[default]
    Unexplored,
    Generated,
    Exhausted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BoundaryReservation {
    pub source_chunk: ChunkCoord,
    pub source_fragment_id: LocationId,
    pub source_pos: GeoPos,
    pub source_radius_cm: i64,
    pub min_spacing_cm: i64,
}

fn serialize_chunk_states<S>(
    chunks: &BTreeMap<ChunkCoord, ChunkState>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let encoded: BTreeMap<String, ChunkState> = chunks
        .iter()
        .map(|(coord, state)| (encode_chunk_coord(*coord), *state))
        .collect();
    encoded.serialize(serializer)
}

fn deserialize_chunk_states<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<ChunkCoord, ChunkState>, D::Error>
where
    D: Deserializer<'de>,
{
    let encoded = BTreeMap::<String, ChunkState>::deserialize(deserializer)?;
    let mut decoded = BTreeMap::new();
    for (key, state) in encoded {
        let coord = decode_chunk_coord(&key).map_err(serde::de::Error::custom)?;
        decoded.insert(coord, state);
    }
    Ok(decoded)
}

fn serialize_chunk_resource_budgets<S>(
    budgets: &BTreeMap<ChunkCoord, ChunkResourceBudget>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let encoded: BTreeMap<String, ChunkResourceBudget> = budgets
        .iter()
        .map(|(coord, budget)| (encode_chunk_coord(*coord), budget.clone()))
        .collect();
    encoded.serialize(serializer)
}

fn deserialize_chunk_resource_budgets<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<ChunkCoord, ChunkResourceBudget>, D::Error>
where
    D: Deserializer<'de>,
{
    let encoded = BTreeMap::<String, ChunkResourceBudget>::deserialize(deserializer)?;
    let mut decoded = BTreeMap::new();
    for (key, budget) in encoded {
        let coord = decode_chunk_coord(&key).map_err(serde::de::Error::custom)?;
        decoded.insert(coord, budget);
    }
    Ok(decoded)
}

fn serialize_chunk_boundary_reservations<S>(
    reservations: &BTreeMap<ChunkCoord, Vec<BoundaryReservation>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let encoded: BTreeMap<String, Vec<BoundaryReservation>> = reservations
        .iter()
        .map(|(coord, entries)| (encode_chunk_coord(*coord), entries.clone()))
        .collect();
    encoded.serialize(serializer)
}

fn deserialize_chunk_boundary_reservations<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<ChunkCoord, Vec<BoundaryReservation>>, D::Error>
where
    D: Deserializer<'de>,
{
    let encoded = BTreeMap::<String, Vec<BoundaryReservation>>::deserialize(deserializer)?;
    let mut decoded = BTreeMap::new();
    for (key, entries) in encoded {
        let coord = decode_chunk_coord(&key).map_err(serde::de::Error::custom)?;
        decoded.insert(coord, entries);
    }
    Ok(decoded)
}

fn encode_chunk_coord(coord: ChunkCoord) -> String {
    format!("{}:{}:{}", coord.x, coord.y, coord.z)
}

fn decode_chunk_coord(encoded: &str) -> Result<ChunkCoord, String> {
    let mut parts = encoded.split(':');
    let x = parts
        .next()
        .ok_or_else(|| format!("invalid chunk coord key: {encoded}"))?
        .parse::<i32>()
        .map_err(|_| format!("invalid chunk coord x: {encoded}"))?;
    let y = parts
        .next()
        .ok_or_else(|| format!("invalid chunk coord key: {encoded}"))?
        .parse::<i32>()
        .map_err(|_| format!("invalid chunk coord y: {encoded}"))?;
    let z = parts
        .next()
        .ok_or_else(|| format!("invalid chunk coord key: {encoded}"))?
        .parse::<i32>()
        .map_err(|_| format!("invalid chunk coord z: {encoded}"))?;
    if parts.next().is_some() {
        return Err(format!("invalid chunk coord key: {encoded}"));
    }
    Ok(ChunkCoord { x, y, z })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FragmentResourceError {
    LocationNotFound { location_id: LocationId },
    FragmentBudgetMissing { location_id: LocationId },
    ChunkCoordUnavailable { location_id: LocationId },
    ChunkBudgetMissing { coord: ChunkCoord },
    Budget(ElementBudgetError),
}

impl WorldModel {
    pub fn consume_fragment_resource(
        &mut self,
        location_id: &str,
        space: &SpaceConfig,
        kind: FragmentElementKind,
        amount_g: i64,
    ) -> Result<i64, FragmentResourceError> {
        if amount_g <= 0 {
            return Err(FragmentResourceError::Budget(
                ElementBudgetError::InvalidAmount { amount_g },
            ));
        }

        let location_id_owned = location_id.to_string();
        let (location_pos, location_remaining) = {
            let location = self.locations.get(location_id).ok_or_else(|| {
                FragmentResourceError::LocationNotFound {
                    location_id: location_id_owned.clone(),
                }
            })?;
            let budget = location.fragment_budget.as_ref().ok_or_else(|| {
                FragmentResourceError::FragmentBudgetMissing {
                    location_id: location_id_owned.clone(),
                }
            })?;
            (location.pos, budget.get_remaining(kind))
        };

        if location_remaining < amount_g {
            return Err(FragmentResourceError::Budget(
                ElementBudgetError::Insufficient {
                    kind,
                    requested_g: amount_g,
                    remaining_g: location_remaining,
                },
            ));
        }

        let coord = chunk_coord_of(location_pos, space).ok_or_else(|| {
            FragmentResourceError::ChunkCoordUnavailable {
                location_id: location_id_owned.clone(),
            }
        })?;

        let chunk_remaining = self
            .chunk_resource_budgets
            .get(&coord)
            .ok_or(FragmentResourceError::ChunkBudgetMissing { coord })?
            .get_remaining(kind);
        if chunk_remaining < amount_g {
            return Err(FragmentResourceError::Budget(
                ElementBudgetError::Insufficient {
                    kind,
                    requested_g: amount_g,
                    remaining_g: chunk_remaining,
                },
            ));
        }

        {
            let location = self.locations.get_mut(location_id).ok_or_else(|| {
                FragmentResourceError::LocationNotFound {
                    location_id: location_id_owned.clone(),
                }
            })?;
            let fragment_budget = location.fragment_budget.as_mut().ok_or_else(|| {
                FragmentResourceError::FragmentBudgetMissing {
                    location_id: location_id_owned.clone(),
                }
            })?;
            fragment_budget
                .consume(kind, amount_g)
                .map_err(FragmentResourceError::Budget)?;
        }

        let chunk_budget = self
            .chunk_resource_budgets
            .get_mut(&coord)
            .ok_or(FragmentResourceError::ChunkBudgetMissing { coord })?;
        chunk_budget
            .consume(kind, amount_g)
            .map_err(FragmentResourceError::Budget)?;

        if chunk_budget.is_exhausted() {
            self.chunks.insert(coord, ChunkState::Exhausted);
        }

        Ok(amount_g)
    }
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
    /// Economy configuration (refine/manufacture minimal loop).
    pub economy: EconomyConfig,
    /// Asteroid fragment belt generation configuration.
    pub asteroid_fragment: AsteroidFragmentConfig,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
            move_cost_per_km_electricity: DEFAULT_MOVE_COST_PER_KM_ELECTRICITY,
            space: SpaceConfig::default(),
            power: PowerConfig::default(),
            physics: PhysicsConfig::default(),
            economy: EconomyConfig::default(),
            asteroid_fragment: AsteroidFragmentConfig::default(),
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
        self.economy = self.economy.sanitized();
        self.asteroid_fragment = self.asteroid_fragment.sanitized();
        self
    }

    pub fn movement_cost(&self, distance_cm: i64) -> i64 {
        let per_km_cost = calibrated_move_cost_per_km(
            self.move_cost_per_km_electricity,
            self.physics.time_step_s,
            self.physics.power_unit_j,
        );
        movement_cost(distance_cm, per_km_cost)
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
    pub max_move_distance_cm_per_tick: i64,
    pub max_move_speed_cm_per_s: i64,
    pub radiation_floor: i64,
    pub radiation_floor_cap_per_tick: i64,
    pub radiation_decay_k: f64,
    pub max_harvest_per_tick: i64,
    pub thermal_capacity: i64,
    pub thermal_dissipation: i64,
    pub thermal_dissipation_gradient_bps: i64,
    pub heat_factor: i64,
    pub erosion_rate: f64,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            time_step_s: MOVE_COST_REFERENCE_TIME_STEP_S,
            power_unit_j: MOVE_COST_REFERENCE_POWER_UNIT_J,
            max_move_distance_cm_per_tick: 1_000_000,
            max_move_speed_cm_per_s: 100_000,
            radiation_floor: 1,
            radiation_floor_cap_per_tick: 5,
            radiation_decay_k: 1e-6,
            max_harvest_per_tick: 50,
            thermal_capacity: 100,
            thermal_dissipation: 5,
            thermal_dissipation_gradient_bps: 10_000,
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
        if self.max_move_distance_cm_per_tick < 0 {
            self.max_move_distance_cm_per_tick = 0;
        }
        if self.max_move_speed_cm_per_s < 0 {
            self.max_move_speed_cm_per_s = 0;
        }
        if self.radiation_floor < 0 {
            self.radiation_floor = 0;
        }
        if self.radiation_floor_cap_per_tick < 0 {
            self.radiation_floor_cap_per_tick = 0;
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
        if self.thermal_dissipation_gradient_bps < 0 {
            self.thermal_dissipation_gradient_bps = 0;
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicsParameterSpec {
    pub key: &'static str,
    pub unit: &'static str,
    pub recommended_min: f64,
    pub recommended_max: f64,
    pub tuning_impact: &'static str,
}

const PHYSICS_PARAMETER_SPECS: [PhysicsParameterSpec; 13] = [
    PhysicsParameterSpec {
        key: "time_step_s",
        unit: "s/tick",
        recommended_min: 1.0,
        recommended_max: 60.0,
        tuning_impact: "时间步越大，单次动作跨度更大、离散误差更明显。",
    },
    PhysicsParameterSpec {
        key: "power_unit_j",
        unit: "J/power_unit",
        recommended_min: 100.0,
        recommended_max: 10_000.0,
        tuning_impact: "影响电力单位到焦耳的映射，与移动/发电口径联动。",
    },
    PhysicsParameterSpec {
        key: "max_move_distance_cm_per_tick",
        unit: "cm/tick",
        recommended_min: 100.0,
        recommended_max: 5_000_000.0,
        tuning_impact: "限制单 tick 最大位移，防止瞬移跨域。",
    },
    PhysicsParameterSpec {
        key: "max_move_speed_cm_per_s",
        unit: "cm/s",
        recommended_min: 100.0,
        recommended_max: 500_000.0,
        tuning_impact: "限制速度上限，约束动力学可解释区间。",
    },
    PhysicsParameterSpec {
        key: "radiation_floor",
        unit: "power_unit/tick",
        recommended_min: 0.0,
        recommended_max: 10.0,
        tuning_impact: "外部背景辐射通量基线，抬升全局可采下限。",
    },
    PhysicsParameterSpec {
        key: "radiation_floor_cap_per_tick",
        unit: "power_unit/tick",
        recommended_min: 0.0,
        recommended_max: 50.0,
        tuning_impact: "背景通量采集上限，限制 floor 造能强度。",
    },
    PhysicsParameterSpec {
        key: "radiation_decay_k",
        unit: "cm^-1",
        recommended_min: 1e-7,
        recommended_max: 1e-4,
        tuning_impact: "介质吸收强度，数值越大近距离优势越明显。",
    },
    PhysicsParameterSpec {
        key: "max_harvest_per_tick",
        unit: "power_unit/tick",
        recommended_min: 1.0,
        recommended_max: 500.0,
        tuning_impact: "限制单 tick 采集峰值，影响高辐射区收益上限。",
    },
    PhysicsParameterSpec {
        key: "thermal_capacity",
        unit: "heat_unit",
        recommended_min: 10.0,
        recommended_max: 1_000.0,
        tuning_impact: "热惯性参数，越大越不容易过热。",
    },
    PhysicsParameterSpec {
        key: "thermal_dissipation",
        unit: "heat_unit/tick",
        recommended_min: 1.0,
        recommended_max: 50.0,
        tuning_impact: "散热基准强度，提升后稳态温度下降。",
    },
    PhysicsParameterSpec {
        key: "thermal_dissipation_gradient_bps",
        unit: "bps",
        recommended_min: 1_000.0,
        recommended_max: 50_000.0,
        tuning_impact: "温差梯度放大系数，控制高热状态散热斜率。",
    },
    PhysicsParameterSpec {
        key: "heat_factor",
        unit: "heat_unit/power_unit",
        recommended_min: 1.0,
        recommended_max: 20.0,
        tuning_impact: "采集转热系数，越高越容易触发热降效。",
    },
    PhysicsParameterSpec {
        key: "erosion_rate",
        unit: "tick^-1 (scaled)",
        recommended_min: 1e-7,
        recommended_max: 1e-4,
        tuning_impact: "磨损基准速率，影响长期维护与硬件损耗。",
    },
];

pub fn physics_parameter_specs() -> &'static [PhysicsParameterSpec] {
    &PHYSICS_PARAMETER_SPECS
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EconomyConfig {
    pub refine_electricity_cost_per_kg: i64,
    pub refine_hardware_yield_ppm: i64,
}

impl Default for EconomyConfig {
    fn default() -> Self {
        Self {
            refine_electricity_cost_per_kg: 2,
            refine_hardware_yield_ppm: 1_000,
        }
    }
}

impl EconomyConfig {
    pub fn sanitized(mut self) -> Self {
        if self.refine_electricity_cost_per_kg < 0 {
            self.refine_electricity_cost_per_kg = 0;
        }
        self.refine_hardware_yield_ppm = self.refine_hardware_yield_ppm.clamp(0, PPM_BASE);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ThermalStatus {
    pub heat: i64,
}

// ============================================================================
// Asteroid Fragment Generator Configuration
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AsteroidFragmentConfig {
    pub base_density_per_km3: f64,
    pub voxel_size_km: i64,
    pub cluster_noise: f64,
    pub layer_scale_height_km: f64,
    pub size_powerlaw_q: f64,
    pub radiation_emission_scale: f64,
    pub radiation_radius_exponent: f64,
    pub radius_min_cm: i64,
    pub radius_max_cm: i64,
    pub min_fragment_spacing_cm: i64,
    pub max_fragments_per_chunk: i64,
    pub max_blocks_per_fragment: i64,
    pub max_blocks_per_chunk: i64,
    pub material_weights: MaterialWeights,
    pub material_radiation_factors: MaterialRadiationFactors,
}

impl Default for AsteroidFragmentConfig {
    fn default() -> Self {
        Self {
            base_density_per_km3: 0.001,
            voxel_size_km: 10,
            cluster_noise: 0.5,
            layer_scale_height_km: 2.0,
            size_powerlaw_q: 3.0,
            radiation_emission_scale: 1e-12,
            radiation_radius_exponent: 3.0,
            radius_min_cm: 25_000,
            radius_max_cm: 500_000,
            min_fragment_spacing_cm: 50_000,
            max_fragments_per_chunk: 4_000,
            max_blocks_per_fragment: 64,
            max_blocks_per_chunk: 120_000,
            material_weights: MaterialWeights::default(),
            material_radiation_factors: MaterialRadiationFactors::default(),
        }
    }
}

impl AsteroidFragmentConfig {
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
        if !self.radiation_emission_scale.is_finite() || self.radiation_emission_scale < 0.0 {
            self.radiation_emission_scale = 0.0;
        }
        if !self.radiation_radius_exponent.is_finite() || self.radiation_radius_exponent < 0.0 {
            self.radiation_radius_exponent = 0.0;
        }
        if self.radius_min_cm < 0 {
            self.radius_min_cm = 0;
        }
        if self.radius_max_cm < self.radius_min_cm {
            self.radius_max_cm = self.radius_min_cm;
        }
        if self.min_fragment_spacing_cm < 0 {
            self.min_fragment_spacing_cm = 0;
        }
        if self.max_fragments_per_chunk < 0 {
            self.max_fragments_per_chunk = 0;
        }
        if self.max_blocks_per_fragment < 0 {
            self.max_blocks_per_fragment = 0;
        }
        if self.max_blocks_per_chunk < 0 {
            self.max_blocks_per_chunk = 0;
        }
        self.material_weights = self.material_weights.sanitized();
        self.material_radiation_factors = self.material_radiation_factors.sanitized();
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
            silicate: 52,
            metal: 8,
            ice: 18,
            carbon: 18,
            composite: 4,
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MaterialRadiationFactors {
    pub silicate_bps: u32,
    pub metal_bps: u32,
    pub ice_bps: u32,
    pub carbon_bps: u32,
    pub composite_bps: u32,
}

impl Default for MaterialRadiationFactors {
    fn default() -> Self {
        Self {
            silicate_bps: 7_500,
            metal_bps: 13_000,
            ice_bps: 4_500,
            carbon_bps: 6_000,
            composite_bps: 11_000,
        }
    }
}

impl MaterialRadiationFactors {
    pub fn sanitized(mut self) -> Self {
        self.silicate_bps = self.silicate_bps.min(50_000);
        self.metal_bps = self.metal_bps.min(50_000);
        self.ice_bps = self.ice_bps.min(50_000);
        self.carbon_bps = self.carbon_bps.min(50_000);
        self.composite_bps = self.composite_bps.min(50_000);

        if self.silicate_bps == 0
            && self.metal_bps == 0
            && self.ice_bps == 0
            && self.carbon_bps == 0
            && self.composite_bps == 0
        {
            self.silicate_bps = 1;
        }
        self
    }

    pub fn factor_for(self, material: MaterialKind) -> f64 {
        let bps = match material {
            MaterialKind::Silicate => self.silicate_bps,
            MaterialKind::Metal => self.metal_bps,
            MaterialKind::Ice => self.ice_bps,
            MaterialKind::Carbon => self.carbon_bps,
            MaterialKind::Composite => self.composite_bps,
        };
        bps as f64 / 10_000.0
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

fn calibrated_move_cost_per_km(base_per_km_cost: i64, time_step_s: i64, power_unit_j: i64) -> i64 {
    if base_per_km_cost <= 0 {
        return 0;
    }

    let time_step_s = time_step_s.max(1) as i128;
    let power_unit_j = power_unit_j.max(1) as i128;
    let scaled = (base_per_km_cost as i128)
        .saturating_mul(time_step_s)
        .saturating_mul(MOVE_COST_REFERENCE_POWER_UNIT_J as i128);
    let denom = (MOVE_COST_REFERENCE_TIME_STEP_S as i128).saturating_mul(power_unit_j);
    let adjusted = scaled
        .saturating_add(denom.saturating_sub(1))
        .saturating_div(denom);
    adjusted.clamp(0, i64::MAX as i128) as i64
}

/// Calculate movement cost based on distance and per-km cost.
pub fn movement_cost(distance_cm: i64, per_km_cost: i64) -> i64 {
    if distance_cm <= 0 || per_km_cost <= 0 {
        return 0;
    }
    let km = (distance_cm + CM_PER_KM - 1) / CM_PER_KM;
    km.saturating_mul(per_km_cost)
}
