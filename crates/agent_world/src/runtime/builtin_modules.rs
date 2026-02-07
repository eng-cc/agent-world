//! Built-in module implementations for development and testing.

use std::collections::BTreeMap;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::geometry::{space_distance_cm, GeoPos};
use crate::models::BodyKernelView;
use crate::simulator::{
    ResourceKind, CM_PER_KM, DEFAULT_MOVE_COST_PER_KM_ELECTRICITY, DEFAULT_VISIBILITY_RANGE_CM,
};

use super::events::{Action, ActionEnvelope, Observation, ObservedAgent};
use super::rules::{ResourceDelta, RuleDecision, RuleVerdict};
use super::sandbox::{
    ModuleCallErrorCode, ModuleCallFailure, ModuleCallInput, ModuleCallRequest, ModuleEmit,
    ModuleOutput, ModuleSandbox,
};
use super::util::to_canonical_cbor;
use super::world_event::{WorldEvent, WorldEventBody};

mod power_modules;

pub const M1_MOVE_RULE_MODULE_ID: &str = "m1.rule.move";
pub const M1_VISIBILITY_RULE_MODULE_ID: &str = "m1.rule.visibility";
pub const M1_TRANSFER_RULE_MODULE_ID: &str = "m1.rule.transfer";
pub const M1_BODY_MODULE_ID: &str = "m1.body.core";
pub const M1_BODY_ACTION_COST_ELECTRICITY: i64 = 10;

pub use power_modules::{
    M1RadiationPowerModule, M1StoragePowerModule, M1_POWER_HARVEST_BASE_PER_TICK,
    M1_POWER_HARVEST_DISTANCE_BONUS_CAP, M1_POWER_HARVEST_DISTANCE_STEP_CM,
    M1_POWER_MODULE_VERSION, M1_POWER_STORAGE_CAPACITY, M1_POWER_STORAGE_INITIAL_LEVEL,
    M1_POWER_STORAGE_MOVE_COST_PER_KM, M1_RADIATION_POWER_MODULE_ID, M1_STORAGE_POWER_MODULE_ID,
};

pub trait BuiltinModule {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure>;
}

pub struct BuiltinModuleSandbox {
    builtins: BTreeMap<String, Box<dyn BuiltinModule>>,
    fallback: Option<Box<dyn ModuleSandbox>>,
}

impl BuiltinModuleSandbox {
    pub fn new() -> Self {
        Self {
            builtins: BTreeMap::new(),
            fallback: None,
        }
    }

    pub fn with_fallback(fallback: Box<dyn ModuleSandbox>) -> Self {
        Self {
            builtins: BTreeMap::new(),
            fallback: Some(fallback),
        }
    }

    pub fn register_builtin(
        mut self,
        module_id: impl Into<String>,
        module: impl BuiltinModule + 'static,
    ) -> Self {
        self.builtins.insert(module_id.into(), Box::new(module));
        self
    }
}

impl ModuleSandbox for BuiltinModuleSandbox {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        if let Some(module) = self.builtins.get_mut(&request.module_id) {
            return module.call(request);
        }
        if let Some(fallback) = self.fallback.as_mut() {
            return fallback.call(request);
        }
        Err(ModuleCallFailure {
            module_id: request.module_id.clone(),
            trace_id: request.trace_id.clone(),
            code: ModuleCallErrorCode::SandboxUnavailable,
            detail: "builtin module not found".to_string(),
        })
    }
}

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PositionState {
    agents: BTreeMap<String, GeoPos>,
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

#[derive(Debug, Clone)]
pub struct M1BodyModule {
    cost_electricity: i64,
}

impl Default for M1BodyModule {
    fn default() -> Self {
        Self {
            cost_electricity: M1_BODY_ACTION_COST_ELECTRICITY,
        }
    }
}

impl M1BodyModule {
    fn handle_action(
        &self,
        request: &ModuleCallRequest,
        envelope: ActionEnvelope,
    ) -> Result<ModuleOutput, ModuleCallFailure> {
        let ActionEnvelope { id, action } = envelope;
        let (agent_id, kind, payload) = match action {
            Action::BodyAction {
                agent_id,
                kind,
                payload,
            } => (agent_id, kind, payload),
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

        let view: BodyKernelView = match serde_json::from_value(payload) {
            Ok(view) => view,
            Err(err) => {
                decision.verdict = RuleVerdict::Deny;
                decision
                    .notes
                    .push(format!("body action payload decode failed: {err}"));
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
            }
        };

        decision.verdict = RuleVerdict::Modify;
        decision.override_action = Some(Action::EmitBodyAttributes {
            agent_id,
            view,
            reason: format!("body.{kind}"),
        });

        if self.cost_electricity > 0 {
            decision
                .cost
                .entries
                .insert(ResourceKind::Electricity, -self.cost_electricity);
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
}

impl BuiltinModule for M1BodyModule {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        let input = decode_input::<ModuleCallInput>(request, &request.input)?;

        if let Some(action_bytes) = input.action.as_deref() {
            let envelope = decode_input::<ActionEnvelope>(request, action_bytes)?;
            return self.handle_action(request, envelope);
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

fn decode_state<T: DeserializeOwned + Default>(
    state: Option<&[u8]>,
    request: &ModuleCallRequest,
) -> Result<T, ModuleCallFailure> {
    let Some(state) = state else {
        return Ok(T::default());
    };
    if state.is_empty() {
        return Ok(T::default());
    }
    decode_input(request, state)
}

fn encode_state<T: Serialize>(
    state: &T,
    request: &ModuleCallRequest,
) -> Result<Vec<u8>, ModuleCallFailure> {
    to_canonical_cbor(state).map_err(|err| {
        failure(
            request,
            ModuleCallErrorCode::InvalidOutput,
            format!("state encode failed: {err:?}"),
        )
    })
}

fn update_position_state(state: &mut PositionState, event: WorldEvent) -> bool {
    let mut changed = false;
    if let WorldEventBody::Domain(domain) = event.body {
        match domain {
            super::events::DomainEvent::AgentRegistered { agent_id, pos } => {
                state.agents.insert(agent_id, pos);
                changed = true;
            }
            super::events::DomainEvent::AgentMoved { agent_id, to, .. } => {
                state.agents.insert(agent_id, to);
                changed = true;
            }
            super::events::DomainEvent::ActionRejected { .. } => {}
            super::events::DomainEvent::Observation { .. } => {}
            super::events::DomainEvent::BodyAttributesUpdated { .. } => {}
            super::events::DomainEvent::BodyAttributesRejected { .. } => {}
            super::events::DomainEvent::ResourceTransferred { .. } => {}
        }
    }
    changed
}

fn decode_input<T: DeserializeOwned>(
    request: &ModuleCallRequest,
    bytes: &[u8],
) -> Result<T, ModuleCallFailure> {
    serde_cbor::from_slice(bytes).map_err(|err| {
        failure(
            request,
            ModuleCallErrorCode::InvalidOutput,
            format!("input CBOR decode failed: {err}"),
        )
    })
}

fn finalize_output(
    mut output: ModuleOutput,
    request: &ModuleCallRequest,
) -> Result<ModuleOutput, ModuleCallFailure> {
    output.output_bytes = 0;
    let encoded = serde_cbor::to_vec(&output).map_err(|err| {
        failure(
            request,
            ModuleCallErrorCode::InvalidOutput,
            format!("output encode failed: {err}"),
        )
    })?;
    output.output_bytes = encoded.len() as u64;
    Ok(output)
}

fn failure(
    request: &ModuleCallRequest,
    code: ModuleCallErrorCode,
    detail: String,
) -> ModuleCallFailure {
    ModuleCallFailure {
        module_id: request.module_id.clone(),
        trace_id: request.trace_id.clone(),
        code,
        detail,
    }
}

fn movement_cost(distance_cm: i64, per_km_cost: i64) -> i64 {
    if distance_cm <= 0 || per_km_cost <= 0 {
        return 0;
    }
    let km = (distance_cm + CM_PER_KM - 1) / CM_PER_KM;
    km.saturating_mul(per_km_cost)
}
