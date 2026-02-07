//! Agent interface: AgentBehavior trait, AgentDecision, ActionResult.

use serde::{Deserialize, Serialize};

use super::kernel::{Observation, RejectReason, WorldEvent, WorldEventKind};
use super::types::{Action, ActionId, WorldTime};

// ============================================================================
// Agent Interface (observe → decide → act)
// ============================================================================

/// Core trait for Agent behavior in the world.
///
/// An Agent follows the observe → decide → act loop:
/// 1. `observe`: Receive the current observation of the world
/// 2. `decide`: Based on observation, decide what action to take
/// 3. `act`: The kernel executes the decided action and produces events
///
/// This trait is designed to be implemented by various agent types:
/// - Simple rule-based agents
/// - LLM-powered agents
/// - Scripted agents for testing
pub trait AgentBehavior {
    /// Returns the agent's unique identifier.
    fn agent_id(&self) -> &str;

    /// Called when the agent receives a new observation.
    /// Returns an optional action to take, or None if the agent chooses to wait.
    fn decide(&mut self, observation: &Observation) -> AgentDecision;

    /// Called after an action is executed to notify the agent of the result.
    /// This allows the agent to update internal state based on action outcomes.
    fn on_action_result(&mut self, _result: &ActionResult) {
        // Default: no-op
    }

    /// Called when an event affecting this agent occurs.
    /// This allows the agent to react to external events.
    fn on_event(&mut self, _event: &WorldEvent) {
        // Default: no-op
    }

    /// Takes and clears the latest decision trace if available.
    ///
    /// Useful for live debugging/visualization pipelines that need decision internals
    /// (e.g. LLM prompt/completion I/O).
    fn take_decision_trace(&mut self) -> Option<AgentDecisionTrace> {
        None
    }
}

/// The result of an agent's decision process.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentDecision {
    /// The agent decides to perform an action.
    Act(Action),
    /// The agent decides to wait/skip this turn.
    Wait,
    /// The agent decides to wait for a specific number of ticks.
    WaitTicks(u64),
}

impl AgentDecision {
    /// Returns true if the agent decided to act.
    pub fn is_act(&self) -> bool {
        matches!(self, AgentDecision::Act(_))
    }

    /// Returns the action if the agent decided to act.
    pub fn action(&self) -> Option<&Action> {
        match self {
            AgentDecision::Act(action) => Some(action),
            _ => None,
        }
    }
}

/// Optional decision trace payload, intended for observability/diagnostics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentDecisionTrace {
    pub agent_id: String,
    pub time: WorldTime,
    pub decision: AgentDecision,
    pub llm_input: Option<String>,
    pub llm_output: Option<String>,
    pub llm_error: Option<String>,
    pub parse_error: Option<String>,
}

/// Result of an action execution, providing feedback to the agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionResult {
    /// The action that was executed.
    pub action: Action,
    /// The action ID assigned by the kernel.
    pub action_id: ActionId,
    /// Whether the action succeeded.
    pub success: bool,
    /// The resulting event (success or rejection).
    pub event: WorldEvent,
}

impl ActionResult {
    /// Returns true if the action was rejected.
    pub fn is_rejected(&self) -> bool {
        matches!(self.event.kind, WorldEventKind::ActionRejected { .. })
    }

    /// Returns the rejection reason if the action was rejected.
    pub fn reject_reason(&self) -> Option<&RejectReason> {
        match &self.event.kind {
            WorldEventKind::ActionRejected { reason } => Some(reason),
            _ => None,
        }
    }
}
