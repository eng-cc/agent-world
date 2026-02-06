//! WorldKernel: time, events, actions, and observation.

mod actions;
mod observation;
mod persistence;
mod power;
mod replay;
mod step;
mod types;

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::types::{ActionEnvelope, ActionId, FragmentElementKind, WorldEventId, WorldTime};
use super::world_model::{FragmentResourceError, WorldConfig, WorldModel};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChunkRuntimeConfig {
    pub world_seed: u64,
    pub asteroid_fragment_enabled: bool,
    pub asteroid_fragment_seed_offset: u64,
    pub min_fragment_spacing_cm: Option<i64>,
}

impl Default for ChunkRuntimeConfig {
    fn default() -> Self {
        Self {
            world_seed: 0,
            asteroid_fragment_enabled: false,
            asteroid_fragment_seed_offset: 1,
            min_fragment_spacing_cm: None,
        }
    }
}

impl ChunkRuntimeConfig {
    pub fn asteroid_fragment_seed(&self) -> u64 {
        self.world_seed
            .wrapping_add(self.asteroid_fragment_seed_offset)
    }
}

pub use types::{
    Observation, ObservedAgent, ObservedLocation, RejectReason, WorldEvent, WorldEventKind,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorldKernel {
    time: WorldTime,
    config: WorldConfig,
    next_event_id: WorldEventId,
    next_action_id: ActionId,
    pending_actions: VecDeque<ActionEnvelope>,
    journal: Vec<WorldEvent>,
    model: WorldModel,
    #[serde(default)]
    chunk_runtime: ChunkRuntimeConfig,
}

impl WorldKernel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: WorldConfig) -> Self {
        let mut kernel = Self::default();
        kernel.config = config.sanitized();
        kernel
    }

    pub fn with_model(config: WorldConfig, model: WorldModel) -> Self {
        Self {
            time: 0,
            config: config.sanitized(),
            next_event_id: 0,
            next_action_id: 0,
            pending_actions: VecDeque::new(),
            journal: Vec::new(),
            model,
            chunk_runtime: ChunkRuntimeConfig::default(),
        }
    }

    pub fn with_model_and_chunk_runtime(
        config: WorldConfig,
        model: WorldModel,
        chunk_runtime: ChunkRuntimeConfig,
    ) -> Self {
        Self {
            time: 0,
            config: config.sanitized(),
            next_event_id: 0,
            next_action_id: 0,
            pending_actions: VecDeque::new(),
            journal: Vec::new(),
            model,
            chunk_runtime,
        }
    }

    pub fn time(&self) -> WorldTime {
        self.time
    }

    pub fn config(&self) -> &WorldConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: WorldConfig) {
        self.config = config.sanitized();
    }

    pub fn model(&self) -> &WorldModel {
        &self.model
    }

    pub fn consume_fragment_resource(
        &mut self,
        location_id: &str,
        kind: FragmentElementKind,
        amount_g: i64,
    ) -> Result<i64, FragmentResourceError> {
        self.model
            .consume_fragment_resource(location_id, &self.config.space, kind, amount_g)
    }

    pub fn journal(&self) -> &[WorldEvent] {
        &self.journal
    }

    pub(super) fn record_event(&mut self, kind: WorldEventKind) -> WorldEvent {
        let event = WorldEvent {
            id: self.next_event_id,
            time: self.time,
            kind,
        };
        self.next_event_id = self.next_event_id.saturating_add(1);
        self.journal.push(event.clone());
        event
    }
}
