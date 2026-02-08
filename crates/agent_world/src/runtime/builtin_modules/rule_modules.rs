use crate::geometry::space_distance_cm;
use crate::simulator::{
    ResourceKind, DEFAULT_MOVE_COST_PER_KM_ELECTRICITY, DEFAULT_VISIBILITY_RANGE_CM,
};

use super::super::events::{Action, ActionEnvelope, Observation, ObservedAgent};
use super::super::rules::{ResourceDelta, RuleDecision, RuleVerdict};
use super::super::sandbox::{
    ModuleCallErrorCode, ModuleCallFailure, ModuleCallInput, ModuleCallRequest, ModuleEmit,
    ModuleOutput,
};
use super::super::world_event::WorldEvent;
use super::{
    decode_input, decode_state, encode_state, failure, finalize_output, movement_cost,
    update_position_state, BuiltinModule, PositionState,
};

#[derive(Debug, Clone)]
pub struct M1MoveRuleModule {
    per_km_cost: i64,
}

impl Default for M1MoveRuleModule {
    fn default() -> Self {
        Self {
            per_km_cost: DEFAULT_MOVE_COST_PER_KM_ELECTRICITY,
        }
    }
}

impl M1MoveRuleModule {
    pub fn new(per_km_cost: i64) -> Self {
        Self { per_km_cost }
    }

    fn handle_action(
        &self,
        request: &ModuleCallRequest,
        envelope: ActionEnvelope,
        state: PositionState,
    ) -> Result<ModuleOutput, ModuleCallFailure> {
        let ActionEnvelope { id, action } = envelope;
        let (agent_id, to) = match action {
            Action::MoveAgent { agent_id, to } => (agent_id, to),
            _ => {
                return finalize_output(
                    ModuleOutput {
                        new_state: None,
                        effects: Vec::new(),
                        emits: Vec::new(),
                        output_bytes: 0,
                    },
                    request,
                );
            }
        };

        let mut decision = RuleDecision {
            action_id: id,
            verdict: RuleVerdict::Allow,
            override_action: None,
            cost: ResourceDelta::default(),
            notes: Vec::new(),
        };

        match state.agents.get(&agent_id) {
            Some(from) => {
                let distance_cm = space_distance_cm(*from, to);
                if distance_cm == 0 {
                    decision.verdict = RuleVerdict::Deny;
                    decision
                        .notes
                        .push("move target equals current position".to_string());
                } else {
                    let cost = movement_cost(distance_cm, self.per_km_cost);
                    if cost > 0 {
                        decision
                            .cost
                            .entries
                            .insert(ResourceKind::Electricity, -cost);
                    }
                }
            }
            None => {
                decision
                    .notes
                    .push("agent position missing for move rule".to_string());
            }
        }

        finalize_output(
            ModuleOutput {
                new_state: None,
                effects: Vec::new(),
                emits: vec![ModuleEmit {
                    kind: "rule.decision".to_string(),
                    payload: serde_json::to_value(decision).map_err(|err| {
                        failure(
                            request,
                            ModuleCallErrorCode::InvalidOutput,
                            format!("rule decision encode failed: {err}"),
                        )
                    })?,
                }],
                output_bytes: 0,
            },
            request,
        )
    }

    fn handle_event(
        &self,
        request: &ModuleCallRequest,
        event: WorldEvent,
        mut state: PositionState,
    ) -> Result<ModuleOutput, ModuleCallFailure> {
        let changed = update_position_state(&mut state, event);

        let new_state = if changed {
            Some(encode_state(&state, request)?)
        } else {
            None
        };
        finalize_output(
            ModuleOutput {
                new_state,
                effects: Vec::new(),
                emits: Vec::new(),
                output_bytes: 0,
            },
            request,
        )
    }
}

impl BuiltinModule for M1MoveRuleModule {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        let input = decode_input::<ModuleCallInput>(request, &request.input)?;
        let state: PositionState = decode_state(input.state.as_deref(), request)?;

        if let Some(action_bytes) = input.action.as_deref() {
            let envelope = decode_input::<ActionEnvelope>(request, action_bytes)?;
            return self.handle_action(request, envelope, state);
        }

        if let Some(event_bytes) = input.event.as_deref() {
            let event = decode_input::<WorldEvent>(request, event_bytes)?;
            return self.handle_event(request, event, state);
        }

        finalize_output(
            ModuleOutput {
                new_state: None,
                effects: Vec::new(),
                emits: Vec::new(),
                output_bytes: 0,
            },
            request,
        )
    }
}

#[derive(Debug, Clone)]
pub struct M1VisibilityRuleModule {
    visibility_range_cm: i64,
}

impl Default for M1VisibilityRuleModule {
    fn default() -> Self {
        Self {
            visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
        }
    }
}

impl M1VisibilityRuleModule {
    pub fn new(visibility_range_cm: i64) -> Self {
        Self {
            visibility_range_cm,
        }
    }

    fn handle_action(
        &self,
        request: &ModuleCallRequest,
        envelope: ActionEnvelope,
        state: PositionState,
        now: u64,
    ) -> Result<ModuleOutput, ModuleCallFailure> {
        let ActionEnvelope { id, action } = envelope;
        let agent_id = match action {
            Action::QueryObservation { agent_id } => agent_id,
            _ => {
                return finalize_output(
                    ModuleOutput {
                        new_state: None,
                        effects: Vec::new(),
                        emits: Vec::new(),
                        output_bytes: 0,
                    },
                    request,
                );
            }
        };

        let mut decision = RuleDecision {
            action_id: id,
            verdict: RuleVerdict::Modify,
            override_action: None,
            cost: ResourceDelta::default(),
            notes: Vec::new(),
        };

        let Some(origin) = state.agents.get(&agent_id) else {
            decision.verdict = RuleVerdict::Deny;
            decision
                .notes
                .push("agent position missing for visibility rule".to_string());
            return finalize_output(
                ModuleOutput {
                    new_state: None,
                    effects: Vec::new(),
                    emits: vec![ModuleEmit {
                        kind: "rule.decision".to_string(),
                        payload: serde_json::to_value(decision).map_err(|err| {
                            failure(
                                request,
                                ModuleCallErrorCode::InvalidOutput,
                                format!("rule decision encode failed: {err}"),
                            )
                        })?,
                    }],
                    output_bytes: 0,
                },
                request,
            );
        };

        let mut visible_agents = Vec::new();
        for (other_id, other_pos) in &state.agents {
            if other_id == &agent_id {
                continue;
            }
            let distance_cm = space_distance_cm(*origin, *other_pos);
            if distance_cm <= self.visibility_range_cm {
                visible_agents.push(ObservedAgent {
                    agent_id: other_id.clone(),
                    pos: *other_pos,
                    distance_cm,
                });
            }
        }

        let observation = Observation {
            time: now,
            agent_id: agent_id.clone(),
            pos: *origin,
            visibility_range_cm: self.visibility_range_cm,
            visible_agents,
        };

        decision.override_action = Some(Action::EmitObservation { observation });

        finalize_output(
            ModuleOutput {
                new_state: None,
                effects: Vec::new(),
                emits: vec![ModuleEmit {
                    kind: "rule.decision".to_string(),
                    payload: serde_json::to_value(decision).map_err(|err| {
                        failure(
                            request,
                            ModuleCallErrorCode::InvalidOutput,
                            format!("rule decision encode failed: {err}"),
                        )
                    })?,
                }],
                output_bytes: 0,
            },
            request,
        )
    }

    fn handle_event(
        &self,
        request: &ModuleCallRequest,
        event: WorldEvent,
        mut state: PositionState,
    ) -> Result<ModuleOutput, ModuleCallFailure> {
        let changed = update_position_state(&mut state, event);
        let new_state = if changed {
            Some(encode_state(&state, request)?)
        } else {
            None
        };

        finalize_output(
            ModuleOutput {
                new_state,
                effects: Vec::new(),
                emits: Vec::new(),
                output_bytes: 0,
            },
            request,
        )
    }
}

impl BuiltinModule for M1VisibilityRuleModule {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        let input = decode_input::<ModuleCallInput>(request, &request.input)?;
        let state: PositionState = decode_state(input.state.as_deref(), request)?;

        if let Some(action_bytes) = input.action.as_deref() {
            let envelope = decode_input::<ActionEnvelope>(request, action_bytes)?;
            return self.handle_action(request, envelope, state, input.ctx.time);
        }

        if let Some(event_bytes) = input.event.as_deref() {
            let event = decode_input::<WorldEvent>(request, event_bytes)?;
            return self.handle_event(request, event, state);
        }

        finalize_output(
            ModuleOutput {
                new_state: None,
                effects: Vec::new(),
                emits: Vec::new(),
                output_bytes: 0,
            },
            request,
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct M1TransferRuleModule;

impl M1TransferRuleModule {
    fn handle_action(
        &self,
        request: &ModuleCallRequest,
        envelope: ActionEnvelope,
        state: PositionState,
    ) -> Result<ModuleOutput, ModuleCallFailure> {
        let ActionEnvelope { id, action } = envelope;
        let (from_agent_id, to_agent_id, kind, amount) = match action {
            Action::TransferResource {
                from_agent_id,
                to_agent_id,
                kind,
                amount,
            } => (from_agent_id, to_agent_id, kind, amount),
            _ => {
                return finalize_output(
                    ModuleOutput {
                        new_state: None,
                        effects: Vec::new(),
                        emits: Vec::new(),
                        output_bytes: 0,
                    },
                    request,
                );
            }
        };

        let mut decision = RuleDecision {
            action_id: id,
            verdict: RuleVerdict::Modify,
            override_action: None,
            cost: ResourceDelta::default(),
            notes: Vec::new(),
        };

        if amount <= 0 {
            decision.verdict = RuleVerdict::Deny;
            decision
                .notes
                .push("transfer amount must be positive".to_string());
        } else {
            let from_pos = state.agents.get(&from_agent_id);
            let to_pos = state.agents.get(&to_agent_id);
            match (from_pos, to_pos) {
                (Some(from_pos), Some(to_pos)) => {
                    let distance_cm = space_distance_cm(*from_pos, *to_pos);
                    if distance_cm > 0 {
                        decision.verdict = RuleVerdict::Deny;
                        decision
                            .notes
                            .push("transfer requires co-located agents".to_string());
                    } else {
                        decision.override_action = Some(Action::EmitResourceTransfer {
                            from_agent_id: from_agent_id.clone(),
                            to_agent_id: to_agent_id.clone(),
                            kind,
                            amount,
                        });
                    }
                }
                _ => {
                    decision.verdict = RuleVerdict::Deny;
                    decision
                        .notes
                        .push("agent position missing for transfer rule".to_string());
                }
            }
        }

        finalize_output(
            ModuleOutput {
                new_state: None,
                effects: Vec::new(),
                emits: vec![ModuleEmit {
                    kind: "rule.decision".to_string(),
                    payload: serde_json::to_value(decision).map_err(|err| {
                        failure(
                            request,
                            ModuleCallErrorCode::InvalidOutput,
                            format!("rule decision encode failed: {err}"),
                        )
                    })?,
                }],
                output_bytes: 0,
            },
            request,
        )
    }

    fn handle_event(
        &self,
        request: &ModuleCallRequest,
        event: WorldEvent,
        mut state: PositionState,
    ) -> Result<ModuleOutput, ModuleCallFailure> {
        let changed = update_position_state(&mut state, event);
        let new_state = if changed {
            Some(encode_state(&state, request)?)
        } else {
            None
        };

        finalize_output(
            ModuleOutput {
                new_state,
                effects: Vec::new(),
                emits: Vec::new(),
                output_bytes: 0,
            },
            request,
        )
    }
}

impl BuiltinModule for M1TransferRuleModule {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        let input = decode_input::<ModuleCallInput>(request, &request.input)?;
        let state: PositionState = decode_state(input.state.as_deref(), request)?;

        if let Some(action_bytes) = input.action.as_deref() {
            let envelope = decode_input::<ActionEnvelope>(request, action_bytes)?;
            return self.handle_action(request, envelope, state);
        }

        if let Some(event_bytes) = input.event.as_deref() {
            let event = decode_input::<WorldEvent>(request, event_bytes)?;
            return self.handle_event(request, event, state);
        }

        finalize_output(
            ModuleOutput {
                new_state: None,
                effects: Vec::new(),
                emits: Vec::new(),
                output_bytes: 0,
            },
            request,
        )
    }
}
