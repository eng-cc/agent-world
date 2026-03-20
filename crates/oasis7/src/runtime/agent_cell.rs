//! Agent cell representation - wraps agent state with mailbox and activity tracking.

use crate::models::AgentState;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::events::DomainEvent;
use super::types::WorldTime;

/// A cell that holds an agent's state along with its mailbox and activity tracking.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentCell {
    pub state: AgentState,
    pub mailbox: VecDeque<DomainEvent>,
    pub last_active: WorldTime,
}

impl AgentCell {
    pub fn new(state: AgentState, now: WorldTime) -> Self {
        Self {
            state,
            mailbox: VecDeque::new(),
            last_active: now,
        }
    }
}
