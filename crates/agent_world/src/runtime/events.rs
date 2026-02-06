//! Action and domain event types.

use crate::geometry::GeoPos;
use crate::simulator::ResourceKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::types::{ActionId, WorldTime};

/// An envelope wrapping an action with its ID.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionEnvelope {
    pub id: ActionId,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    pub time: WorldTime,
    pub agent_id: String,
    pub pos: GeoPos,
    pub visibility_range_cm: i64,
    pub visible_agents: Vec<ObservedAgent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedAgent {
    pub agent_id: String,
    pub pos: GeoPos,
    pub distance_cm: i64,
}

/// Actions that can be submitted to the world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Action {
    RegisterAgent { agent_id: String, pos: GeoPos },
    MoveAgent { agent_id: String, to: GeoPos },
    QueryObservation { agent_id: String },
    EmitObservation { observation: Observation },
}

/// Domain events that describe state changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DomainEvent {
    AgentRegistered { agent_id: String, pos: GeoPos },
    AgentMoved { agent_id: String, from: GeoPos, to: GeoPos },
    ActionRejected { action_id: ActionId, reason: RejectReason },
    Observation { observation: Observation },
}

impl DomainEvent {
    pub fn agent_id(&self) -> Option<&str> {
        match self {
            DomainEvent::AgentRegistered { agent_id, .. } => Some(agent_id.as_str()),
            DomainEvent::AgentMoved { agent_id, .. } => Some(agent_id.as_str()),
            DomainEvent::Observation { observation } => Some(observation.agent_id.as_str()),
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
    InsufficientResources { deficits: BTreeMap<ResourceKind, i64> },
    RuleDenied { notes: Vec<String> },
}

/// The cause of an event, for audit purposes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum CausedBy {
    Action(ActionId),
    Effect { intent_id: String },
}
