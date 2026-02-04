//! World state management.

use crate::models::AgentState;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::agent_cell::AgentCell;
use super::error::WorldError;
use super::events::DomainEvent;
use super::types::WorldTime;

/// The mutable state of the world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldState {
    pub time: WorldTime,
    pub agents: BTreeMap<String, AgentCell>,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            time: 0,
            agents: BTreeMap::new(),
        }
    }
}

impl WorldState {
    pub fn apply_domain_event(&mut self, event: &DomainEvent, now: WorldTime) -> Result<(), WorldError> {
        match event {
            DomainEvent::AgentRegistered { agent_id, pos } => {
                let state = AgentState::new(agent_id, *pos);
                self.agents
                    .insert(agent_id.clone(), AgentCell::new(state, now));
            }
            DomainEvent::AgentMoved { agent_id, to, .. } => {
                if let Some(cell) = self.agents.get_mut(agent_id) {
                    cell.state.pos = *to;
                    cell.last_active = now;
                }
            }
            DomainEvent::ActionRejected { .. } => {}
        }
        Ok(())
    }

    pub fn route_domain_event(&mut self, event: &DomainEvent) {
        let Some(agent_id) = event.agent_id() else {
            return;
        };
        if let Some(cell) = self.agents.get_mut(agent_id) {
            cell.mailbox.push_back(event.clone());
        }
    }
}
