//! World initialization utilities.

use serde::{Deserialize, Serialize};

use crate::geometry::GeoPos;

use super::dust::generate_fragments;
use super::kernel::WorldKernel;
use super::types::{AgentId, LocationId, LocationProfile, ResourceKind, ResourceStock};
use super::world_model::{Agent, Location, WorldConfig, WorldModel};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldInitConfig {
    pub seed: u64,
    pub origin: OriginLocationConfig,
    pub locations: Vec<LocationSeedConfig>,
    pub dust: DustInitConfig,
    pub agents: AgentSpawnConfig,
}

impl Default for WorldInitConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            origin: OriginLocationConfig::default(),
            locations: Vec::new(),
            dust: DustInitConfig::default(),
            agents: AgentSpawnConfig::default(),
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
        self.dust = self.dust.sanitized();
        self.agents = self.agents.sanitized();
        self
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
pub struct DustInitConfig {
    pub enabled: bool,
    pub seed_offset: u64,
}

impl Default for DustInitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            seed_offset: 1,
        }
    }
}

impl DustInitConfig {
    pub fn sanitized(self) -> Self {
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
    pub resources: ResourceStock,
}

impl Default for AgentSpawnConfig {
    fn default() -> Self {
        Self {
            count: 1,
            id_prefix: "agent-".to_string(),
            start_index: 0,
            location_id: None,
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
pub struct WorldInitReport {
    pub seed: u64,
    pub dust_seed: Option<u64>,
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
    SpawnLocationMissing,
    SpawnLocationNotFound { location_id: LocationId },
    InvalidResourceAmount { kind: ResourceKind, amount: i64 },
}

pub fn build_world_model(
    config: &WorldConfig,
    init: &WorldInitConfig,
) -> Result<(WorldModel, WorldInitReport), WorldInitError> {
    let config = config.clone().sanitized();
    let init = init.clone().sanitized();
    let mut model = WorldModel::default();

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

    let dust_seed = if init.dust.enabled {
        let seed = init.seed.wrapping_add(init.dust.seed_offset);
        let fragments = generate_fragments(seed, &config.space, &config.dust);
        for frag in fragments {
            insert_location(&mut model, frag)?;
        }
        Some(seed)
    } else {
        None
    };

    if init.agents.count > 0 {
        ensure_valid_stock(&init.agents.resources)?;
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
        let (spawn_location_id, spawn_pos) = match model.locations.get(&spawn_location_id) {
            Some(location) => (location.id.clone(), location.pos),
            None => {
                return Err(WorldInitError::SpawnLocationNotFound {
                    location_id: spawn_location_id,
                })
            }
        };

        for offset in 0..init.agents.count {
            let idx = init.agents.start_index as u64 + offset as u64;
            let agent_id = format!("{}{}", init.agents.id_prefix, idx);
            let mut agent = Agent::new_with_power(
                agent_id.clone(),
                spawn_location_id.clone(),
                spawn_pos,
                &config.power,
            );
            agent.resources = init.agents.resources.clone();
            insert_agent(&mut model, agent)?;
        }
    }

    let report = WorldInitReport {
        seed: init.seed,
        dust_seed,
        locations: model.locations.len(),
        agents: model.agents.len(),
    };

    Ok((model, report))
}

pub fn initialize_kernel(
    config: WorldConfig,
    init: WorldInitConfig,
) -> Result<(WorldKernel, WorldInitReport), WorldInitError> {
    let (model, report) = build_world_model(&config, &init)?;
    Ok((WorldKernel::with_model(config, model), report))
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
