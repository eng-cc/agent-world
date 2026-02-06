//! World scenario templates (stable IDs) backed by scenario files.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

use crate::geometry::GeoPos;

use super::init::{
    AgentSpawnConfig, AsteroidFragmentInitConfig, LocationSeedConfig, OriginLocationConfig,
    PowerPlantSeedConfig, PowerStorageSeedConfig, WorldInitConfig,
};
use super::types::{LocationId, LocationProfile, ResourceStock};
use super::world_model::{SpaceConfig, WorldConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorldScenario {
    Minimal,
    TwoBases,
    PowerBootstrap,
    ResourceBootstrap,
    TwinRegionBootstrap,
    TriadRegionBootstrap,
    TriadP2pBootstrap,
    AsteroidFragmentBootstrap,
    AsteroidFragmentTwinRegionBootstrap,
    AsteroidFragmentTriadRegionBootstrap,
}

impl WorldScenario {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorldScenario::Minimal => "minimal",
            WorldScenario::TwoBases => "two_bases",
            WorldScenario::PowerBootstrap => "power_bootstrap",
            WorldScenario::ResourceBootstrap => "resource_bootstrap",
            WorldScenario::TwinRegionBootstrap => "twin_region_bootstrap",
            WorldScenario::TriadRegionBootstrap => "triad_region_bootstrap",
            WorldScenario::TriadP2pBootstrap => "triad_p2p_bootstrap",
            WorldScenario::AsteroidFragmentBootstrap => "asteroid_fragment_bootstrap",
            WorldScenario::AsteroidFragmentTwinRegionBootstrap => "asteroid_fragment_twin_region_bootstrap",
            WorldScenario::AsteroidFragmentTriadRegionBootstrap => "asteroid_fragment_triad_region_bootstrap",
        }
    }

    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_lowercase().as_str() {
            "minimal" => Some(WorldScenario::Minimal),
            "two_bases" | "two-bases" => Some(WorldScenario::TwoBases),
            "power_bootstrap" | "power-bootstrap" | "bootstrap" => {
                Some(WorldScenario::PowerBootstrap)
            }
            "resource_bootstrap" | "resource-bootstrap" | "resources" => {
                Some(WorldScenario::ResourceBootstrap)
            }
            "twin_region_bootstrap" | "twin-region-bootstrap" | "twin_regions" | "twin-regions" => {
                Some(WorldScenario::TwinRegionBootstrap)
            }
            "triad_region_bootstrap" | "triad-region-bootstrap" | "triad_regions" | "triad-regions" => {
                Some(WorldScenario::TriadRegionBootstrap)
            }
            "triad_p2p_bootstrap"
            | "triad-p2p-bootstrap"
            | "triad-p2p"
            | "p2p-triad"
            | "p2p-triad-bootstrap" => Some(WorldScenario::TriadP2pBootstrap),
            "asteroid_fragment_bootstrap"
            | "asteroid-fragment-bootstrap"
            | "asteroid_fragment"
            | "asteroid-fragment" => Some(WorldScenario::AsteroidFragmentBootstrap),
            "asteroid_fragment_twin_region_bootstrap"
            | "asteroid-fragment-twin-region-bootstrap"
            | "asteroid-fragment-twin-regions"
            | "asteroid-fragment-regions" => {
                Some(WorldScenario::AsteroidFragmentTwinRegionBootstrap)
            }
            "asteroid_fragment_triad_region_bootstrap"
            | "asteroid-fragment-triad-region-bootstrap"
            | "asteroid-fragment-triad-regions"
            | "asteroid-fragment-triad" => {
                Some(WorldScenario::AsteroidFragmentTriadRegionBootstrap)
            }
            _ => None,
        }
    }

    pub fn variants() -> &'static [&'static str] {
        &[
            "minimal",
            "two_bases",
            "power_bootstrap",
            "resource_bootstrap",
            "twin_region_bootstrap",
            "triad_region_bootstrap",
            "triad_p2p_bootstrap",
            "asteroid_fragment_bootstrap",
            "asteroid_fragment_twin_region_bootstrap",
            "asteroid_fragment_triad_region_bootstrap",
        ]
    }

    pub fn load_spec(&self) -> WorldScenarioSpec {
        let spec = scenario_spec_json(*self);
        let parsed: WorldScenarioSpec = serde_json::from_str(spec).unwrap_or_else(|err| {
            panic!(
                "failed to parse scenario spec {}: {err}",
                self.as_str()
            )
        });
        if parsed.id != self.as_str() {
            panic!(
                "scenario id mismatch: expected {}, got {}",
                self.as_str(),
                parsed.id
            );
        }
        parsed
    }

    pub fn build_init(&self, config: &WorldConfig) -> WorldInitConfig {
        let config = config.clone().sanitized();
        let spec = self.load_spec();
        spec.into_init_config(&config)
    }

    pub fn load_spec_from_path(path: impl AsRef<Path>) -> Result<WorldScenarioSpec, ScenarioSpecError> {
        WorldScenarioSpec::load_from_path(path)
    }

    pub fn build_init_from_path(
        path: impl AsRef<Path>,
        config: &WorldConfig,
    ) -> Result<WorldInitConfig, ScenarioSpecError> {
        let config = config.clone().sanitized();
        let spec = WorldScenarioSpec::load_from_path(path)?;
        Ok(spec.into_init_config(&config))
    }
}

#[derive(Debug)]
pub enum ScenarioSpecError {
    Io { path: PathBuf, source: std::io::Error },
    Parse { path: PathBuf, source: serde_json::Error },
}

impl fmt::Display for ScenarioSpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScenarioSpecError::Io { path, source } => {
                write!(f, "failed to read {}: {source}", path.display())
            }
            ScenarioSpecError::Parse { path, source } => {
                write!(f, "failed to parse {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for ScenarioSpecError {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldScenarioSpec {
    pub id: String,
    pub name: String,
    pub seed: u64,
    pub origin: ScenarioOriginConfig,
    pub locations: Vec<ScenarioLocationSeedConfig>,
    pub asteroid_fragment: AsteroidFragmentInitConfig,
    pub agents: AgentSpawnConfig,
    pub power_plants: Vec<PowerPlantSeedConfig>,
    pub power_storages: Vec<PowerStorageSeedConfig>,
}

impl Default for WorldScenarioSpec {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            seed: 0,
            origin: ScenarioOriginConfig::default(),
            locations: Vec::new(),
            asteroid_fragment: AsteroidFragmentInitConfig::default(),
            agents: AgentSpawnConfig::default(),
            power_plants: Vec::new(),
            power_storages: Vec::new(),
        }
    }
}

impl WorldScenarioSpec {
    pub fn into_init_config(self, config: &WorldConfig) -> WorldInitConfig {
        WorldInitConfig {
            seed: self.seed,
            origin: self.origin.into_origin(&config.space),
            locations: self
                .locations
                .into_iter()
                .map(|location| location.into_location(&config.space))
                .collect(),
            asteroid_fragment: self.asteroid_fragment,
            agents: self.agents,
            power_plants: self.power_plants,
            power_storages: self.power_storages,
        }
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, ScenarioSpecError> {
        let path = path.as_ref();
        let contents = std::fs::read_to_string(path).map_err(|err| ScenarioSpecError::Io {
            path: path.to_path_buf(),
            source: err,
        })?;
        serde_json::from_str(&contents).map_err(|err| ScenarioSpecError::Parse {
            path: path.to_path_buf(),
            source: err,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScenarioOriginConfig {
    pub enabled: bool,
    pub location_id: LocationId,
    pub name: String,
    pub pos: Option<ScenarioPos>,
    pub profile: LocationProfile,
    pub resources: ResourceStock,
}

impl Default for ScenarioOriginConfig {
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

impl ScenarioOriginConfig {
    fn into_origin(self, space: &SpaceConfig) -> OriginLocationConfig {
        OriginLocationConfig {
            enabled: self.enabled,
            location_id: self.location_id,
            name: self.name,
            pos: self.pos.map(|pos| pos.to_geo(space)),
            profile: self.profile,
            resources: self.resources,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ScenarioLocationSeedConfig {
    pub location_id: LocationId,
    pub name: String,
    pub pos: Option<ScenarioPos>,
    pub profile: LocationProfile,
    pub resources: ResourceStock,
}

impl Default for ScenarioLocationSeedConfig {
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

impl ScenarioLocationSeedConfig {
    fn into_location(self, space: &SpaceConfig) -> LocationSeedConfig {
        LocationSeedConfig {
            location_id: self.location_id,
            name: self.name,
            pos: self.pos.map(|pos| pos.to_geo(space)),
            profile: self.profile,
            resources: self.resources,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ScenarioPos {
    Center,
    CenterOffset {
        dx_pct: f64,
        #[serde(default)]
        dy_pct: f64,
        #[serde(default)]
        dz_pct: f64,
    },
    Absolute {
        x_cm: f64,
        y_cm: f64,
        z_cm: f64,
    },
}

impl ScenarioPos {
    fn to_geo(self, space: &SpaceConfig) -> GeoPos {
        match self {
            ScenarioPos::Center => center_pos(space),
            ScenarioPos::CenterOffset {
                dx_pct,
                dy_pct,
                dz_pct,
            } => {
                let center = center_pos(space);
                GeoPos {
                    x_cm: center.x_cm + offset_component(space.width_cm, dx_pct),
                    y_cm: center.y_cm + offset_component(space.depth_cm, dy_pct),
                    z_cm: center.z_cm + offset_component(space.height_cm, dz_pct),
                }
            }
            ScenarioPos::Absolute { x_cm, y_cm, z_cm } => GeoPos { x_cm, y_cm, z_cm },
        }
    }
}

fn offset_component(dim_cm: i64, pct: f64) -> f64 {
    if pct == 0.0 {
        return 0.0;
    }
    let raw = dim_cm as f64 * pct;
    if raw.abs() < 1.0 {
        raw.signum()
    } else {
        raw
    }
}

fn center_pos(space: &SpaceConfig) -> GeoPos {
    GeoPos {
        x_cm: space.width_cm as f64 / 2.0,
        y_cm: space.depth_cm as f64 / 2.0,
        z_cm: space.height_cm as f64 / 2.0,
    }
}

fn scenario_spec_json(scenario: WorldScenario) -> &'static str {
    match scenario {
        WorldScenario::Minimal => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/scenarios/minimal.json"
        )),
        WorldScenario::TwoBases => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/scenarios/two_bases.json"
        )),
        WorldScenario::PowerBootstrap => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/scenarios/power_bootstrap.json"
        )),
        WorldScenario::ResourceBootstrap => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/scenarios/resource_bootstrap.json"
        )),
        WorldScenario::TwinRegionBootstrap => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/scenarios/twin_region_bootstrap.json"
        )),
        WorldScenario::TriadRegionBootstrap => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/scenarios/triad_region_bootstrap.json"
        )),
        WorldScenario::TriadP2pBootstrap => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/scenarios/triad_p2p_bootstrap.json"
        )),
        WorldScenario::AsteroidFragmentBootstrap => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/scenarios/asteroid_fragment_bootstrap.json"
        )),
        WorldScenario::AsteroidFragmentTwinRegionBootstrap => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/scenarios/asteroid_fragment_twin_region_bootstrap.json"
        )),
        WorldScenario::AsteroidFragmentTriadRegionBootstrap => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/scenarios/asteroid_fragment_triad_region_bootstrap.json"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenario_specs_match_ids() {
        let scenarios = [
            WorldScenario::Minimal,
            WorldScenario::TwoBases,
            WorldScenario::PowerBootstrap,
            WorldScenario::ResourceBootstrap,
            WorldScenario::TwinRegionBootstrap,
            WorldScenario::TriadRegionBootstrap,
            WorldScenario::TriadP2pBootstrap,
            WorldScenario::AsteroidFragmentBootstrap,
            WorldScenario::AsteroidFragmentTwinRegionBootstrap,
            WorldScenario::AsteroidFragmentTriadRegionBootstrap,
        ];

        for scenario in scenarios {
            let spec = scenario.load_spec();
            assert_eq!(spec.id, scenario.as_str());
        }
    }
}
