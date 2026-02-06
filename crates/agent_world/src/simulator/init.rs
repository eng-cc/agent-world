//! World initialization utilities.

use serde::{Deserialize, Serialize};

use crate::geometry::GeoPos;

use super::asteroid_fragment::generate_fragments;
use super::chunking::{chunk_coord_of, chunk_coords, ChunkCoord};
use super::fragment_physics::{synthesize_fragment_budget, synthesize_fragment_profile};
use super::kernel::{ChunkRuntimeConfig, WorldKernel};
use super::power::{PlantStatus, PowerPlant, PowerStorage};
use super::scenario::WorldScenario;
use super::types::{
    AgentId, ChunkResourceBudget, FacilityId, LocationId, LocationProfile, ResourceKind,
    ResourceOwner, ResourceStock,
};
use super::world_model::{Agent, ChunkState, Location, SpaceConfig, WorldConfig, WorldModel};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldInitConfig {
    pub seed: u64,
    pub origin: OriginLocationConfig,
    pub locations: Vec<LocationSeedConfig>,
    pub asteroid_fragment: AsteroidFragmentInitConfig,
    pub agents: AgentSpawnConfig,
    pub power_plants: Vec<PowerPlantSeedConfig>,
    pub power_storages: Vec<PowerStorageSeedConfig>,
}

impl Default for WorldInitConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            origin: OriginLocationConfig::default(),
            locations: Vec::new(),
            asteroid_fragment: AsteroidFragmentInitConfig::default(),
            agents: AgentSpawnConfig::default(),
            power_plants: Vec::new(),
            power_storages: Vec::new(),
        }
    }
}

impl WorldInitConfig {
    pub fn sanitized(mut self) -> Self {
        self.origin = self.origin.sanitized();
        self.locations = self
            .locations
            .into_iter()
            .map(|location| location.sanitized())
            .collect();
        self.asteroid_fragment = self.asteroid_fragment.sanitized();
        self.agents = self.agents.sanitized();
        self.power_plants = self
            .power_plants
            .into_iter()
            .map(|plant| plant.sanitized())
            .collect();
        self.power_storages = self
            .power_storages
            .into_iter()
            .map(|storage| storage.sanitized())
            .collect();
        self
    }

    pub fn from_scenario(scenario: WorldScenario, config: &WorldConfig) -> Self {
        scenario.build_init(config)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OriginLocationConfig {
    pub enabled: bool,
    pub location_id: LocationId,
    pub name: String,
    pub pos: Option<GeoPos>,
    pub profile: LocationProfile,
    pub resources: ResourceStock,
}

impl Default for OriginLocationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            location_id: "origin".to_string(),
            name: "Origin".to_string(),
            pos: None,
            profile: LocationProfile::default(),
            resources: ResourceStock::default(),
        }
    }
}

impl OriginLocationConfig {
    pub fn sanitized(mut self) -> Self {
        if self.location_id.is_empty() {
            self.location_id = "origin".to_string();
        }
        if self.name.is_empty() {
            self.name = "Origin".to_string();
        }
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LocationSeedConfig {
    pub location_id: LocationId,
    pub name: String,
    pub pos: Option<GeoPos>,
    pub profile: LocationProfile,
    pub resources: ResourceStock,
}

impl Default for LocationSeedConfig {
    fn default() -> Self {
        Self {
            location_id: String::new(),
            name: String::new(),
            pos: None,
            profile: LocationProfile::default(),
            resources: ResourceStock::default(),
        }
    }
}

impl LocationSeedConfig {
    pub fn sanitized(mut self) -> Self {
        if self.name.is_empty() {
            self.name = self.location_id.clone();
        }
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AsteroidFragmentInitConfig {
    pub enabled: bool,
    pub seed_offset: u64,
    pub min_fragment_spacing_cm: Option<i64>,
    pub bootstrap_chunks: Vec<ChunkCoord>,
}

impl Default for AsteroidFragmentInitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            seed_offset: 1,
            min_fragment_spacing_cm: None,
            bootstrap_chunks: Vec::new(),
        }
    }
}

impl AsteroidFragmentInitConfig {
    pub fn sanitized(mut self) -> Self {
        if let Some(spacing) = self.min_fragment_spacing_cm {
            if spacing < 0 {
                self.min_fragment_spacing_cm = Some(0);
            }
        }
        self.bootstrap_chunks.sort();
        self.bootstrap_chunks.dedup();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentSpawnConfig {
    pub count: usize,
    pub id_prefix: String,
    pub start_index: u32,
    pub location_id: Option<LocationId>,
    pub spawn_locations: Vec<LocationId>,
    pub resources: ResourceStock,
}

impl Default for AgentSpawnConfig {
    fn default() -> Self {
        Self {
            count: 1,
            id_prefix: "agent-".to_string(),
            start_index: 0,
            location_id: None,
            spawn_locations: Vec::new(),
            resources: ResourceStock::default(),
        }
    }
}

impl AgentSpawnConfig {
    pub fn sanitized(mut self) -> Self {
        if self.id_prefix.is_empty() {
            self.id_prefix = "agent-".to_string();
        }
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PowerPlantSeedConfig {
    pub facility_id: FacilityId,
    pub location_id: LocationId,
    pub owner: ResourceOwner,
    pub capacity_per_tick: i64,
    pub fuel_cost_per_pu: i64,
    pub maintenance_cost: i64,
    pub efficiency: f64,
    pub degradation: f64,
}

impl Default for PowerPlantSeedConfig {
    fn default() -> Self {
        Self {
            facility_id: String::new(),
            location_id: String::new(),
            owner: ResourceOwner::Location {
                location_id: String::new(),
            },
            capacity_per_tick: 0,
            fuel_cost_per_pu: 0,
            maintenance_cost: 0,
            efficiency: 1.0,
            degradation: 0.0,
        }
    }
}

impl PowerPlantSeedConfig {
    pub fn sanitized(self) -> Self {
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PowerStorageSeedConfig {
    pub facility_id: FacilityId,
    pub location_id: LocationId,
    pub owner: ResourceOwner,
    pub capacity: i64,
    pub current_level: i64,
    pub charge_efficiency: f64,
    pub discharge_efficiency: f64,
    pub max_charge_rate: i64,
    pub max_discharge_rate: i64,
}

impl Default for PowerStorageSeedConfig {
    fn default() -> Self {
        Self {
            facility_id: String::new(),
            location_id: String::new(),
            owner: ResourceOwner::Location {
                location_id: String::new(),
            },
            capacity: 0,
            current_level: 0,
            charge_efficiency: 1.0,
            discharge_efficiency: 1.0,
            max_charge_rate: 0,
            max_discharge_rate: 0,
        }
    }
}

impl PowerStorageSeedConfig {
    pub fn sanitized(self) -> Self {
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldInitReport {
    pub seed: u64,
    pub asteroid_fragment_seed: Option<u64>,
    pub locations: usize,
    pub agents: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorldInitError {
    OriginOutOfBounds { pos: GeoPos },
    LocationOutOfBounds { location_id: LocationId, pos: GeoPos },
    InvalidLocationId { location_id: LocationId },
    LocationIdConflict { location_id: LocationId },
    AgentIdConflict { agent_id: AgentId },
    InvalidFacilityId { facility_id: FacilityId },
    FacilityIdConflict { facility_id: FacilityId },
    FacilityLocationNotFound { location_id: LocationId },
    FacilityOwnerNotFound { owner: ResourceOwner },
    SpawnLocationMissing,
    SpawnLocationNotFound { location_id: LocationId },
    InvalidResourceAmount { kind: ResourceKind, amount: i64 },
    InvalidFacilityAmount { field: String, amount: i64 },
    InvalidFacilityRatio { field: String, value: f64 },
    InvalidFacilityLevel { current_level: i64, capacity: i64 },
}

pub fn build_world_model(
    config: &WorldConfig,
    init: &WorldInitConfig,
) -> Result<(WorldModel, WorldInitReport), WorldInitError> {
    let config = config.clone().sanitized();
    let init = init.clone().sanitized();
    let mut model = WorldModel::default();
    initialize_chunk_index(&mut model, &config);

    if init.origin.enabled {
        let pos = init
            .origin
            .pos
            .unwrap_or_else(|| center_pos(&config.space));
        if !config.space.contains(pos) {
            return Err(WorldInitError::OriginOutOfBounds { pos });
        }
        ensure_valid_stock(&init.origin.resources)?;
        let mut location = Location::new_with_profile(
            init.origin.location_id.clone(),
            init.origin.name.clone(),
            pos,
            init.origin.profile.clone(),
        );
        location.resources = init.origin.resources.clone();
        insert_location(&mut model, location)?;
    }

    for location_seed in &init.locations {
        if location_seed.location_id.is_empty() {
            return Err(WorldInitError::InvalidLocationId {
                location_id: location_seed.location_id.clone(),
            });
        }
        let pos = location_seed
            .pos
            .unwrap_or_else(|| center_pos(&config.space));
        if !config.space.contains(pos) {
            return Err(WorldInitError::LocationOutOfBounds {
                location_id: location_seed.location_id.clone(),
                pos,
            });
        }
        ensure_valid_stock(&location_seed.resources)?;
        let name = if location_seed.name.is_empty() {
            location_seed.location_id.clone()
        } else {
            location_seed.name.clone()
        };
        let mut location = Location::new_with_profile(
            location_seed.location_id.clone(),
            name,
            pos,
            location_seed.profile.clone(),
        );
        location.resources = location_seed.resources.clone();
        insert_location(&mut model, location)?;
    }

    let asteroid_fragment_seed = if init.asteroid_fragment.enabled {
        Some(init.seed.wrapping_add(init.asteroid_fragment.seed_offset))
    } else {
        None
    };

    if init.asteroid_fragment.enabled {
        let seed_positions = gather_seed_positions(&model);
        ensure_chunk_generated_at_positions(
            &mut model,
            &config,
            &init,
            seed_positions,
            asteroid_fragment_seed,
        )?;
        ensure_chunk_generated_at_coords(
            &mut model,
            &config,
            &init,
            init.asteroid_fragment.bootstrap_chunks.clone(),
            asteroid_fragment_seed,
        )?;
    }

    let spawn_locations = if !init.agents.spawn_locations.is_empty() {
        init.agents.spawn_locations.clone()
    } else if init.agents.count > 0 {
        let spawn_location_id = match init.agents.location_id.clone() {
            Some(location_id) => location_id,
            None => {
                if init.origin.enabled {
                    init.origin.location_id.clone()
                } else {
                    return Err(WorldInitError::SpawnLocationMissing);
                }
            }
        };
        vec![spawn_location_id; init.agents.count]
    } else {
        Vec::new()
    };

    if !spawn_locations.is_empty() {
        ensure_valid_stock(&init.agents.resources)?;
        for (offset, location_id) in spawn_locations.iter().enumerate() {
            let (spawn_location_id, spawn_pos) = match model.locations.get(location_id) {
                Some(location) => (location.id.clone(), location.pos),
                None => {
                    return Err(WorldInitError::SpawnLocationNotFound {
                        location_id: location_id.clone(),
                    })
                }
            };

            let idx = init.agents.start_index as u64 + offset as u64;
            let agent_id = format!("{}{}", init.agents.id_prefix, idx);
            let mut agent = Agent::new_with_power(
                agent_id.clone(),
                spawn_location_id,
                spawn_pos,
                &config.power,
            );
            agent.resources = init.agents.resources.clone();
            insert_agent(&mut model, agent)?;
        }
    }

    if init.asteroid_fragment.enabled {
        let agent_positions: Vec<GeoPos> = model.agents.values().map(|agent| agent.pos).collect();
        ensure_chunk_generated_at_positions(
            &mut model,
            &config,
            &init,
            agent_positions,
            asteroid_fragment_seed,
        )?;
    }

    for plant_seed in &init.power_plants {
        if plant_seed.facility_id.is_empty() {
            return Err(WorldInitError::InvalidFacilityId {
                facility_id: plant_seed.facility_id.clone(),
            });
        }
        if !model.locations.contains_key(&plant_seed.location_id) {
            return Err(WorldInitError::FacilityLocationNotFound {
                location_id: plant_seed.location_id.clone(),
            });
        }
        ensure_owner_exists(&model, &plant_seed.owner)?;
        ensure_non_negative_amount("capacity_per_tick", plant_seed.capacity_per_tick)?;
        ensure_non_negative_amount("fuel_cost_per_pu", plant_seed.fuel_cost_per_pu)?;
        ensure_non_negative_amount("maintenance_cost", plant_seed.maintenance_cost)?;
        ensure_valid_ratio("efficiency", plant_seed.efficiency)?;
        ensure_valid_ratio("degradation", plant_seed.degradation)?;

        let plant = PowerPlant {
            id: plant_seed.facility_id.clone(),
            location_id: plant_seed.location_id.clone(),
            owner: plant_seed.owner.clone(),
            capacity_per_tick: plant_seed.capacity_per_tick,
            current_output: 0,
            fuel_cost_per_pu: plant_seed.fuel_cost_per_pu,
            maintenance_cost: plant_seed.maintenance_cost,
            status: PlantStatus::Running,
            efficiency: plant_seed.efficiency,
            degradation: plant_seed.degradation,
        };
        insert_power_plant(&mut model, plant)?;
    }

    for storage_seed in &init.power_storages {
        if storage_seed.facility_id.is_empty() {
            return Err(WorldInitError::InvalidFacilityId {
                facility_id: storage_seed.facility_id.clone(),
            });
        }
        if !model.locations.contains_key(&storage_seed.location_id) {
            return Err(WorldInitError::FacilityLocationNotFound {
                location_id: storage_seed.location_id.clone(),
            });
        }
        ensure_owner_exists(&model, &storage_seed.owner)?;
        ensure_non_negative_amount("capacity", storage_seed.capacity)?;
        ensure_non_negative_amount("current_level", storage_seed.current_level)?;
        ensure_non_negative_amount("max_charge_rate", storage_seed.max_charge_rate)?;
        ensure_non_negative_amount("max_discharge_rate", storage_seed.max_discharge_rate)?;
        ensure_valid_ratio("charge_efficiency", storage_seed.charge_efficiency)?;
        ensure_valid_ratio("discharge_efficiency", storage_seed.discharge_efficiency)?;
        if storage_seed.current_level > storage_seed.capacity {
            return Err(WorldInitError::InvalidFacilityLevel {
                current_level: storage_seed.current_level,
                capacity: storage_seed.capacity,
            });
        }

        let storage = PowerStorage {
            id: storage_seed.facility_id.clone(),
            location_id: storage_seed.location_id.clone(),
            owner: storage_seed.owner.clone(),
            capacity: storage_seed.capacity,
            current_level: storage_seed.current_level,
            charge_efficiency: storage_seed.charge_efficiency,
            discharge_efficiency: storage_seed.discharge_efficiency,
            max_charge_rate: storage_seed.max_charge_rate,
            max_discharge_rate: storage_seed.max_discharge_rate,
        };
        insert_power_storage(&mut model, storage)?;
    }

    let report = WorldInitReport {
        seed: init.seed,
        asteroid_fragment_seed,
        locations: model
            .locations
            .values()
            .filter(|location| !location.id.starts_with("frag-"))
            .count(),
        agents: model.agents.len(),
    };

    Ok((model, report))
}

pub fn ensure_chunk_generated_at_positions(
    model: &mut WorldModel,
    config: &WorldConfig,
    init: &WorldInitConfig,
    positions: Vec<GeoPos>,
    asteroid_fragment_seed: Option<u64>,
) -> Result<(), WorldInitError> {
    let coords = positions
        .into_iter()
        .filter_map(|pos| chunk_coord_of(pos, &config.space))
        .collect::<Vec<_>>();
    ensure_chunk_generated_at_coords(model, config, init, coords, asteroid_fragment_seed)
}

pub fn ensure_chunk_generated_at_coords(
    model: &mut WorldModel,
    config: &WorldConfig,
    init: &WorldInitConfig,
    coords: Vec<ChunkCoord>,
    asteroid_fragment_seed: Option<u64>,
) -> Result<(), WorldInitError> {
    for coord in coords {
        if model
            .chunks
            .get(&coord)
            .is_some_and(|state| matches!(state, ChunkState::Generated | ChunkState::Exhausted))
        {
            continue;
        }
        generate_chunk_fragments(model, config, init, coord, asteroid_fragment_seed)?;
    }
    Ok(())
}

pub fn generate_chunk_fragments(
    model: &mut WorldModel,
    config: &WorldConfig,
    init: &WorldInitConfig,
    coord: super::chunking::ChunkCoord,
    asteroid_fragment_seed: Option<u64>,
) -> Result<(), WorldInitError> {
    if !init.asteroid_fragment.enabled {
        model.chunks.insert(coord, ChunkState::Generated);
        model
            .chunk_resource_budgets
            .insert(coord, ChunkResourceBudget::default());
        return Ok(());
    }

    if !model.chunks.contains_key(&coord) {
        return Ok(());
    }
    if model
        .chunks
        .get(&coord)
        .is_some_and(|state| matches!(state, ChunkState::Generated | ChunkState::Exhausted))
    {
        return Ok(());
    }

    let Some(bounds) = super::chunking::chunk_bounds(coord, &config.space) else {
        return Ok(());
    };

    let base_seed = asteroid_fragment_seed
        .unwrap_or_else(|| init.seed.wrapping_add(init.asteroid_fragment.seed_offset));
    let seed = super::chunking::chunk_seed(base_seed, coord);

    let mut asteroid_fragment_config = config.asteroid_fragment.clone();
    if let Some(spacing) = init.asteroid_fragment.min_fragment_spacing_cm {
        asteroid_fragment_config.min_fragment_spacing_cm = spacing;
    }

    let local_space = chunk_local_space(bounds);
    if local_space.width_cm <= 0 || local_space.depth_cm <= 0 || local_space.height_cm <= 0 {
        model.chunks.insert(coord, ChunkState::Generated);
        model
            .chunk_resource_budgets
            .insert(coord, ChunkResourceBudget::default());
        return Ok(());
    }

    let fragments = generate_fragments(seed, &local_space, &asteroid_fragment_config);
    let mut chunk_budget = ChunkResourceBudget::default();

    for (idx, mut frag) in fragments.into_iter().enumerate() {
        frag.id = format!("frag-{}-{}-{}-{}", coord.x, coord.y, coord.z, idx);
        frag.name = frag.id.clone();
        frag.pos.x_cm += bounds.min.x_cm;
        frag.pos.y_cm += bounds.min.y_cm;
        frag.pos.z_cm += bounds.min.z_cm;

        if model.locations.contains_key(&frag.id) {
            continue;
        }

        let profile_seed = seed
            .wrapping_add((idx as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let fragment_profile = synthesize_fragment_profile(
            profile_seed,
            frag.profile.radius_cm,
            frag.profile.material,
        );
        let fragment_budget = synthesize_fragment_budget(&fragment_profile);
        chunk_budget.accumulate_fragment(&fragment_budget);
        frag.fragment_profile = Some(fragment_profile);
        frag.fragment_budget = Some(fragment_budget);

        insert_location(model, frag)?;
    }

    model.chunks.insert(coord, ChunkState::Generated);
    model.chunk_resource_budgets.insert(coord, chunk_budget);
    Ok(())
}

fn chunk_local_space(bounds: super::chunking::ChunkBounds) -> SpaceConfig {
    SpaceConfig {
        width_cm: (bounds.max.x_cm - bounds.min.x_cm).floor() as i64,
        depth_cm: (bounds.max.y_cm - bounds.min.y_cm).floor() as i64,
        height_cm: (bounds.max.z_cm - bounds.min.z_cm).floor() as i64,
    }
}

fn initialize_chunk_index(model: &mut WorldModel, config: &WorldConfig) {
    for coord in chunk_coords(&config.space) {
        model.chunks.insert(coord, ChunkState::Unexplored);
    }
}

fn gather_seed_positions(model: &WorldModel) -> Vec<GeoPos> {
    let mut positions: Vec<GeoPos> = model.locations.values().map(|location| location.pos).collect();
    positions.extend(model.agents.values().map(|agent| agent.pos));
    positions
}

pub fn initialize_kernel(
    config: WorldConfig,
    init: WorldInitConfig,
) -> Result<(WorldKernel, WorldInitReport), WorldInitError> {
    let (model, report) = build_world_model(&config, &init)?;
    let chunk_runtime = ChunkRuntimeConfig {
        world_seed: init.seed,
        asteroid_fragment_enabled: init.asteroid_fragment.enabled,
        asteroid_fragment_seed_offset: init.asteroid_fragment.seed_offset,
        min_fragment_spacing_cm: init.asteroid_fragment.min_fragment_spacing_cm,
    };
    Ok((
        WorldKernel::with_model_and_chunk_runtime(config, model, chunk_runtime),
        report,
    ))
}

fn center_pos(space: &super::world_model::SpaceConfig) -> GeoPos {
    GeoPos {
        x_cm: space.width_cm as f64 / 2.0,
        y_cm: space.depth_cm as f64 / 2.0,
        z_cm: space.height_cm as f64 / 2.0,
    }
}

fn insert_location(model: &mut WorldModel, location: Location) -> Result<(), WorldInitError> {
    if model.locations.contains_key(&location.id) {
        return Err(WorldInitError::LocationIdConflict {
            location_id: location.id,
        });
    }
    model.locations.insert(location.id.clone(), location);
    Ok(())
}

fn insert_agent(model: &mut WorldModel, agent: Agent) -> Result<(), WorldInitError> {
    if model.agents.contains_key(&agent.id) {
        return Err(WorldInitError::AgentIdConflict {
            agent_id: agent.id,
        });
    }
    model.agents.insert(agent.id.clone(), agent);
    Ok(())
}

fn ensure_valid_stock(stock: &ResourceStock) -> Result<(), WorldInitError> {
    for (kind, amount) in &stock.amounts {
        if *amount < 0 {
            return Err(WorldInitError::InvalidResourceAmount {
                kind: *kind,
                amount: *amount,
            });
        }
    }
    Ok(())
}

fn ensure_owner_exists(model: &WorldModel, owner: &ResourceOwner) -> Result<(), WorldInitError> {
    match owner {
        ResourceOwner::Agent { agent_id } => {
            if model.agents.contains_key(agent_id) {
                Ok(())
            } else {
                Err(WorldInitError::FacilityOwnerNotFound { owner: owner.clone() })
            }
        }
        ResourceOwner::Location { location_id } => {
            if model.locations.contains_key(location_id) {
                Ok(())
            } else {
                Err(WorldInitError::FacilityOwnerNotFound { owner: owner.clone() })
            }
        }
    }
}

fn ensure_non_negative_amount(field: &str, amount: i64) -> Result<(), WorldInitError> {
    if amount < 0 {
        return Err(WorldInitError::InvalidFacilityAmount {
            field: field.to_string(),
            amount,
        });
    }
    Ok(())
}

fn ensure_valid_ratio(field: &str, value: f64) -> Result<(), WorldInitError> {
    if !value.is_finite() || value < 0.0 || value > 1.0 {
        return Err(WorldInitError::InvalidFacilityRatio {
            field: field.to_string(),
            value,
        });
    }
    Ok(())
}

fn insert_power_plant(model: &mut WorldModel, plant: PowerPlant) -> Result<(), WorldInitError> {
    if model.power_plants.contains_key(&plant.id) || model.power_storages.contains_key(&plant.id) {
        return Err(WorldInitError::FacilityIdConflict {
            facility_id: plant.id,
        });
    }
    model.power_plants.insert(plant.id.clone(), plant);
    Ok(())
}

fn insert_power_storage(
    model: &mut WorldModel,
    storage: PowerStorage,
) -> Result<(), WorldInitError> {
    if model.power_plants.contains_key(&storage.id)
        || model.power_storages.contains_key(&storage.id)
    {
        return Err(WorldInitError::FacilityIdConflict {
            facility_id: storage.id,
        });
    }
    model.power_storages.insert(storage.id.clone(), storage);
    Ok(())
}
