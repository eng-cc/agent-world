use crate::geometry::GeoPos;
use crate::simulator::ResourceStock;
use serde::{Deserialize, Serialize};

pub const DEFAULT_AGENT_HEIGHT_CM: i64 = 100;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BodyKernelView {
    pub mass_kg: u64,
    pub radius_cm: u64,
    pub thrust_limit: u64,
    pub cross_section_cm2: u64,
}

impl Default for BodyKernelView {
    fn default() -> Self {
        Self {
            mass_kg: 0,
            radius_cm: 0,
            thrust_limit: 0,
            cross_section_cm2: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RobotBodySpec {
    pub kind: String,
    pub height_cm: i64,
}

impl Default for RobotBodySpec {
    fn default() -> Self {
        Self {
            kind: "humanoid".to_string(),
            height_cm: DEFAULT_AGENT_HEIGHT_CM,
        }
    }
}

impl RobotBodySpec {
    pub fn height_m(&self) -> f64 {
        self.height_cm as f64 / 100.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentState {
    pub agent_id: String,
    pub pos: GeoPos,
    pub body: RobotBodySpec,
    #[serde(default)]
    pub resources: ResourceStock,
    #[serde(default)]
    pub body_view: BodyKernelView,
}

impl AgentState {
    pub fn new(agent_id: impl Into<String>, pos: GeoPos) -> Self {
        Self {
            agent_id: agent_id.into(),
            pos,
            body: RobotBodySpec::default(),
            resources: ResourceStock::default(),
            body_view: BodyKernelView::default(),
        }
    }
}
