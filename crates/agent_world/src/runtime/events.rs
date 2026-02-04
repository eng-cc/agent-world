//! Action and domain event types.

use crate::geometry::GeoPos;
use serde::{Deserialize, Serialize};

use super::types::ActionId;

/// An envelope wrapping an action with its ID.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionEnvelope {
    pub id: ActionId,
    pub action: Action,
}

/// Actions that can be submitted to the world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Action {
    RegisterAgent { agent_id: String, pos: GeoPos },
    MoveAgent { agent_id: String, to: GeoPos },
}

/// Domain events that describe state changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DomainEvent {
    AgentRegistered { agent_id: String, pos: GeoPos },
    AgentMoved { agent_id: String, from: GeoPos, to: GeoPos },
    ActionRejected { action_id: ActionId, reason: RejectReason },
}

impl DomainEvent {
    pub fn agent_id(&self) -> Option<&str> {
        match self {
            DomainEvent::AgentRegistered { agent_id, .. } => Some(agent_id.as_str()),
            DomainEvent::AgentMoved { agent_id, .. } => Some(agent_id.as_str()),
            DomainEvent::ActionRejected { .. } => None,
        }
    }
}

/// Reasons why an action was rejected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum RejectReason {
    AgentAlreadyExists { agent_id: String },
    AgentNotFound { agent_id: String },
}

/// The cause of an event, for audit purposes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum CausedBy {
    Action(ActionId),
    Effect { intent_id: String },
}
