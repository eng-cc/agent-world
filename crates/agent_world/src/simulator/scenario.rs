//! World scenario templates (stable IDs).

use serde::{Deserialize, Serialize};

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
    DustyBootstrap,
    DustyTwinRegionBootstrap,
    DustyTriadRegionBootstrap,
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
            WorldScenario::DustyBootstrap => "dusty_bootstrap",
            WorldScenario::DustyTwinRegionBootstrap => "dusty_twin_region_bootstrap",
            WorldScenario::DustyTriadRegionBootstrap => "dusty_triad_region_bootstrap",
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
            "dusty_bootstrap" | "dusty-bootstrap" | "dusty" => Some(WorldScenario::DustyBootstrap),
            "dusty_twin_region_bootstrap"
            | "dusty-twin-region-bootstrap"
            | "dusty-twin-regions"
            | "dusty-regions" => Some(WorldScenario::DustyTwinRegionBootstrap),
            "dusty_triad_region_bootstrap"
            | "dusty-triad-region-bootstrap"
            | "dusty-triad-regions"
            | "dusty-triad" => Some(WorldScenario::DustyTriadRegionBootstrap),
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
            "dusty_bootstrap",
            "dusty_twin_region_bootstrap",
            "dusty_triad_region_bootstrap",
        ]
    }
}
