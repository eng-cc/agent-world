//! World model entities: Agent, Location, Asset, WorldConfig, WorldModel.

use crate::geometry::{
    GeoPos, DEFAULT_CLOUD_DEPTH_CM, DEFAULT_CLOUD_HEIGHT_CM, DEFAULT_CLOUD_WIDTH_CM,
};
use crate::models::RobotBodySpec;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap;

use super::fragment_physics::FragmentPhysicalProfile;
use super::power::{AgentPowerStatus, PowerConfig, PowerPlant, PowerStorage};
use super::chunking::{chunk_coord_of, ChunkCoord};
use super::types::{
    AgentId, AssetId, ChunkResourceBudget, ElementBudgetError, FacilityId, FragmentElementKind,
    FragmentResourceBudget, LocationId, LocationProfile, MaterialKind, ResourceKind, ResourceStock,
    CM_PER_KM, DEFAULT_MOVE_COST_PER_KM_ELECTRICITY, DEFAULT_VISIBILITY_RANGE_CM,
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
            return Err(FragmentResourceError::Budget(ElementBudgetError::InvalidAmount {
                amount_g,
            }));
        }

        let location_id_owned = location_id.to_string();
        let (location_pos, location_remaining) = {
            let location = self
                .locations
                .get(location_id)
                .ok_or_else(|| FragmentResourceError::LocationNotFound {
                    location_id: location_id_owned.clone(),
                })?;
            let budget = location
                .fragment_budget
                .as_ref()
                .ok_or_else(|| FragmentResourceError::FragmentBudgetMissing {
                    location_id: location_id_owned.clone(),
                })?;
            (location.pos, budget.get_remaining(kind))
        };

        if location_remaining < amount_g {
            return Err(FragmentResourceError::Budget(ElementBudgetError::Insufficient {
                kind,
                requested_g: amount_g,
                remaining_g: location_remaining,
            }));
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
            return Err(FragmentResourceError::Budget(ElementBudgetError::Insufficient {
                kind,
                requested_g: amount_g,
                remaining_g: chunk_remaining,
            }));
        }

        {
            let location = self
                .locations
                .get_mut(location_id)
                .ok_or_else(|| FragmentResourceError::LocationNotFound {
                    location_id: location_id_owned.clone(),
                })?;
            let fragment_budget = location
                .fragment_budget
                .as_mut()
                .ok_or_else(|| FragmentResourceError::FragmentBudgetMissing {
                    location_id: location_id_owned.clone(),
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
        self.asteroid_fragment = self.asteroid_fragment.sanitized();
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
    pub radius_min_cm: i64,
    pub radius_max_cm: i64,
    pub min_fragment_spacing_cm: i64,
    pub material_weights: MaterialWeights,
}

impl Default for AsteroidFragmentConfig {
    fn default() -> Self {
        Self {
            base_density_per_km3: 0.001,
            voxel_size_km: 10,
            cluster_noise: 0.5,
            layer_scale_height_km: 2.0,
            size_powerlaw_q: 3.0,
            radius_min_cm: 10,
            radius_max_cm: 10_000,
            min_fragment_spacing_cm: 50_000,
            material_weights: MaterialWeights::default(),
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
        if self.radius_min_cm < 0 {
            self.radius_min_cm = 0;
        }
        if self.radius_max_cm < self.radius_min_cm {
            self.radius_max_cm = self.radius_min_cm;
        }
        if self.min_fragment_spacing_cm < 0 {
            self.min_fragment_spacing_cm = 0;
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
