use crate::geometry::space_distance_cm;

use super::super::chunking::ChunkCoord;
use super::super::init::{
    generate_chunk_fragments, AsteroidFragmentInitConfig, WorldInitConfig,
};
use super::super::ChunkState;
use super::types::{
    ChunkGenerationCause, Observation, ObservedAgent, ObservedLocation, RejectReason,
    WorldEventKind,
};
use super::WorldKernel;

impl WorldKernel {
    pub fn observe(&mut self, agent_id: &str) -> Result<Observation, RejectReason> {
        let Some(agent) = self.model.agents.get(agent_id) else {
            return Err(RejectReason::AgentNotFound {
                agent_id: agent_id.to_string(),
            });
        };
        let agent_pos = agent.pos;
        self.ensure_chunk_generated_at(agent_pos, ChunkGenerationCause::Observe)?;

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

    pub(super) fn ensure_chunk_generated_at(
        &mut self,
        pos: crate::geometry::GeoPos,
        cause: ChunkGenerationCause,
    ) -> Result<(), RejectReason> {
        let Some(coord) = super::super::chunking::chunk_coord_of(pos, &self.config.space) else {
            return Ok(());
        };
        if self
            .model
            .chunks
            .get(&coord)
            .is_some_and(|state| matches!(state, ChunkState::Generated | ChunkState::Exhausted))
        {
            return Ok(());
        }

        if !self.chunk_runtime.asteroid_fragment_enabled {
            self.model.chunks.insert(coord, ChunkState::Generated);
            self.model
                .chunk_resource_budgets
                .entry(coord)
                .or_default();
            return Ok(());
        }

        let init = WorldInitConfig {
            seed: self.chunk_runtime.world_seed,
            asteroid_fragment: AsteroidFragmentInitConfig {
                enabled: self.chunk_runtime.asteroid_fragment_enabled,
                seed_offset: self.chunk_runtime.asteroid_fragment_seed_offset,
                min_fragment_spacing_cm: self.chunk_runtime.min_fragment_spacing_cm,
                bootstrap_chunks: Vec::new(),
            },
            ..WorldInitConfig::default()
        };

        let summary = generate_chunk_fragments(
            &mut self.model,
            &self.config,
            &init,
            coord,
            Some(self.chunk_runtime.asteroid_fragment_seed()),
        )
        .map_err(|_| reject_chunk_generation(coord))?;

        self.record_event(WorldEventKind::ChunkGenerated {
            coord: summary.coord,
            seed: summary.seed,
            fragment_count: summary.fragment_count,
            block_count: summary.block_count,
            chunk_budget: summary.chunk_budget,
            cause,
        });

        Ok(())
    }
}

fn reject_chunk_generation(coord: ChunkCoord) -> RejectReason {
    RejectReason::ChunkGenerationFailed {
        x: coord.x,
        y: coord.y,
        z: coord.z,
    }
}
