use super::super::agent::{ActionResult, AgentDecision};
use super::super::kernel::{Observation, RejectReason, WorldEventKind};
use super::super::types::Action;
use super::decision_flow::{ExecuteUntilCondition, ExecuteUntilDirective, ExecuteUntilEventKind};

#[derive(Debug, Clone, Default)]
pub(super) struct ActionReplanGuardState {
    last_action_signature: Option<String>,
    consecutive_same_action: usize,
}

impl ActionReplanGuardState {
    pub(super) fn record_decision(&mut self, decision: &AgentDecision) {
        let signature = decision_action_signature(decision);
        match signature {
            Some(signature) => {
                if self.last_action_signature.as_deref() == Some(signature.as_str()) {
                    self.consecutive_same_action = self.consecutive_same_action.saturating_add(1);
                } else {
                    self.last_action_signature = Some(signature);
                    self.consecutive_same_action = 1;
                }
            }
            None => {
                self.last_action_signature = None;
                self.consecutive_same_action = 0;
            }
        }
    }

    pub(super) fn should_force_replan(&self, threshold: usize) -> bool {
        threshold > 0 && self.consecutive_same_action >= threshold
    }

    pub(super) fn guard_summary(&self, threshold: usize) -> Option<String> {
        let action = self.last_action_signature.as_ref()?;
        Some(format!(
            "consecutive_same_action={}; threshold={}; last_action={action}",
            self.consecutive_same_action, threshold,
        ))
    }
}

#[derive(Debug, Clone)]
pub(super) struct ActiveExecuteUntil {
    action: Action,
    until_conditions: Vec<ExecuteUntilCondition>,
    remaining_ticks: u64,
    baseline_visible_agents: usize,
    baseline_visible_locations: usize,
    target_location_id: Option<String>,
    last_action_failed: bool,
    last_action_rejected: bool,
    last_reject_reason: Option<RejectReason>,
    last_harvest_amount: Option<i64>,
    last_harvest_available: Option<i64>,
}

impl ActiveExecuteUntil {
    pub(super) fn from_directive(
        directive: ExecuteUntilDirective,
        observation: &Observation,
    ) -> Self {
        let target_location_id = match &directive.action {
            Action::MoveAgent { to, .. } => Some(to.clone()),
            _ => None,
        };

        Self {
            action: directive.action,
            until_conditions: directive.until_conditions,
            remaining_ticks: directive.max_ticks,
            baseline_visible_agents: observation.visible_agents.len(),
            baseline_visible_locations: observation.visible_locations.len(),
            target_location_id,
            last_action_failed: false,
            last_action_rejected: false,
            last_reject_reason: None,
            last_harvest_amount: None,
            last_harvest_available: None,
        }
    }

    pub(super) fn action(&self) -> &Action {
        &self.action
    }

    pub(super) fn until_events_summary(&self) -> String {
        self.until_conditions
            .iter()
            .map(ExecuteUntilCondition::summary)
            .collect::<Vec<_>>()
            .join("|")
    }

    pub(super) fn remaining_ticks(&self) -> u64 {
        self.remaining_ticks
    }

    pub(super) fn update_from_action_result(&mut self, result: &ActionResult) {
        if !actions_same(self.action(), &result.action) {
            return;
        }

        self.last_action_failed = !result.success;
        self.last_action_rejected = false;
        self.last_reject_reason = None;
        self.last_harvest_amount = None;
        self.last_harvest_available = None;

        match &result.event.kind {
            WorldEventKind::ActionRejected { reason } => {
                self.last_action_rejected = true;
                self.last_reject_reason = Some(reason.clone());
            }
            WorldEventKind::RadiationHarvested {
                amount, available, ..
            } => {
                self.last_harvest_amount = Some(*amount);
                self.last_harvest_available = Some(*available);
            }
            _ => {}
        }
    }

    pub(super) fn evaluate_next_step(&mut self, observation: &Observation) -> Result<(), String> {
        for condition in &self.until_conditions {
            match condition.kind {
                ExecuteUntilEventKind::ActionRejected => {
                    if self.last_action_rejected {
                        return Err("until.event action_rejected matched".to_string());
                    }
                }
                ExecuteUntilEventKind::NewVisibleAgent => {
                    if observation.visible_agents.len() > self.baseline_visible_agents {
                        return Err(format!(
                            "until.event new_visible_agent matched: baseline={}, current={}",
                            self.baseline_visible_agents,
                            observation.visible_agents.len()
                        ));
                    }
                }
                ExecuteUntilEventKind::NewVisibleLocation => {
                    if observation.visible_locations.len() > self.baseline_visible_locations {
                        return Err(format!(
                            "until.event new_visible_location matched: baseline={}, current={}",
                            self.baseline_visible_locations,
                            observation.visible_locations.len()
                        ));
                    }
                }
                ExecuteUntilEventKind::ArriveTarget => {
                    if let Some(target_location_id) = self.target_location_id.as_ref() {
                        let arrived = observation.visible_locations.iter().any(|location| {
                            location.location_id == *target_location_id && location.distance_cm <= 0
                        });
                        if arrived {
                            return Err(format!(
                                "until.event arrive_target matched: {target_location_id}"
                            ));
                        }
                    }
                }
                ExecuteUntilEventKind::InsufficientElectricity => {
                    if self
                        .last_reject_reason
                        .as_ref()
                        .is_some_and(reject_reason_is_insufficient_electricity)
                    {
                        return Err("until.event insufficient_electricity matched".to_string());
                    }
                }
                ExecuteUntilEventKind::ThermalOverload => {
                    if self
                        .last_reject_reason
                        .as_ref()
                        .is_some_and(reject_reason_is_thermal_overload)
                    {
                        return Err("until.event thermal_overload matched".to_string());
                    }
                }
                ExecuteUntilEventKind::HarvestYieldBelow => {
                    if let (Some(amount), Some(value_lte)) =
                        (self.last_harvest_amount, condition.value_lte)
                    {
                        if amount <= value_lte {
                            return Err(format!(
                                "until.event harvest_yield_below matched: amount={}, threshold={}",
                                amount, value_lte
                            ));
                        }
                    }
                }
                ExecuteUntilEventKind::HarvestAvailableBelow => {
                    if let (Some(available), Some(value_lte)) =
                        (self.last_harvest_available, condition.value_lte)
                    {
                        if available <= value_lte {
                            return Err(format!(
                                "until.event harvest_available_below matched: available={}, threshold={}",
                                available, value_lte
                            ));
                        }
                    }
                }
            }
        }

        if self.last_action_failed {
            return Err("until plan stop: previous action failed".to_string());
        }

        if self.remaining_ticks == 0 {
            return Err("until plan stop: max_ticks reached".to_string());
        }

        self.remaining_ticks = self.remaining_ticks.saturating_sub(1);
        self.last_action_failed = false;
        self.last_action_rejected = false;
        self.last_reject_reason = None;
        self.last_harvest_amount = None;
        self.last_harvest_available = None;
        Ok(())
    }
}

fn reject_reason_is_insufficient_electricity(reason: &RejectReason) -> bool {
    matches!(
        reason,
        RejectReason::InsufficientResource { kind, .. }
            if matches!(kind, super::super::types::ResourceKind::Electricity)
    ) || matches!(reason, RejectReason::AgentShutdown { .. })
}

fn reject_reason_is_thermal_overload(reason: &RejectReason) -> bool {
    matches!(reason, RejectReason::ThermalOverload { .. })
}

fn decision_action_signature(decision: &AgentDecision) -> Option<String> {
    match decision {
        AgentDecision::Act(action) => Some(action_signature(action)),
        _ => None,
    }
}

fn action_signature(action: &Action) -> String {
    match action {
        Action::MoveAgent { to, .. } => format!("move_agent:{to}"),
        Action::HarvestRadiation { max_amount, .. } => {
            format!("harvest_radiation:{max_amount}")
        }
        other => format!("other:{other:?}"),
    }
}

fn actions_same(left: &Action, right: &Action) -> bool {
    match (left, right) {
        (Action::MoveAgent { to: left_to, .. }, Action::MoveAgent { to: right_to, .. }) => {
            left_to == right_to
        }
        (
            Action::HarvestRadiation {
                max_amount: left_amount,
                ..
            },
            Action::HarvestRadiation {
                max_amount: right_amount,
                ..
            },
        ) => left_amount == right_amount,
        _ => false,
    }
}
