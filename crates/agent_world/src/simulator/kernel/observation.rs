use crate::geometry::space_distance_cm;

use super::WorldKernel;
use super::types::{Observation, ObservedAgent, ObservedLocation, RejectReason};

impl WorldKernel {
    pub fn observe(&self, agent_id: &str) -> Result<Observation, RejectReason> {
        let Some(agent) = self.model.agents.get(agent_id) else {
            return Err(RejectReason::AgentNotFound {
                agent_id: agent_id.to_string(),
            });
        };
        let visibility_range_cm = self.config.visibility_range_cm;
        let mut visible_agents = Vec::new();
        for (other_id, other) in &self.model.agents {
            if other_id == agent_id {
                continue;
            }
            let distance_cm = space_distance_cm(agent.pos, other.pos);
            if distance_cm <= visibility_range_cm {
                visible_agents.push(ObservedAgent {
                    agent_id: other_id.clone(),
                    location_id: other.location_id.clone(),
                    pos: other.pos,
                    distance_cm,
                });
            }
        }

        let mut visible_locations = Vec::new();
        for (location_id, location) in &self.model.locations {
            let distance_cm = space_distance_cm(agent.pos, location.pos);
            if distance_cm <= visibility_range_cm {
                visible_locations.push(ObservedLocation {
                    location_id: location_id.clone(),
                    name: location.name.clone(),
                    pos: location.pos,
                    profile: location.profile.clone(),
                    distance_cm,
                });
            }
        }

        Ok(Observation {
            time: self.time,
            agent_id: agent_id.to_string(),
            pos: agent.pos,
            visibility_range_cm,
            visible_agents,
            visible_locations,
        })
    }
}
