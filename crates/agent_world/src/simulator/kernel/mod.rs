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

use super::types::{ActionEnvelope, ActionId, WorldEventId, WorldTime};
use super::world_model::{WorldConfig, WorldModel};

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
