//! Built-in module implementations for development and testing.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::geometry::{space_distance_cm, GeoPos};
use crate::simulator::{CM_PER_KM, DEFAULT_MOVE_COST_PER_KM_ELECTRICITY, ResourceKind};

use super::events::ActionEnvelope;
use super::rules::{ResourceDelta, RuleDecision, RuleVerdict};
use super::sandbox::{
    ModuleCallErrorCode, ModuleCallFailure, ModuleCallInput, ModuleCallRequest, ModuleEmit,
    ModuleOutput, ModuleSandbox,
};
use super::util::to_canonical_cbor;
use super::world_event::{WorldEvent, WorldEventBody};

pub const M1_MOVE_RULE_MODULE_ID: &str = "m1.rule.move";

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
        self.builtins
            .insert(module_id.into(), Box::new(module));
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
        state: M1MoveRuleState,
    ) -> Result<ModuleOutput, ModuleCallFailure> {
        let ActionEnvelope { id, action } = envelope;
        let (agent_id, to) = match action {
            super::events::Action::MoveAgent { agent_id, to } => (agent_id, to),
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
                    payload: serde_json::to_value(decision).map_err(|err| failure(
                        request,
                        ModuleCallErrorCode::InvalidOutput,
                        format!("rule decision encode failed: {err}"),
                    ))?,
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
        mut state: M1MoveRuleState,
    ) -> Result<ModuleOutput, ModuleCallFailure> {
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
            }
        }

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
        let state = decode_state(input.state.as_deref(), request)?;

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct M1MoveRuleState {
    agents: BTreeMap<String, GeoPos>,
}

fn decode_state(
    state: Option<&[u8]>,
    request: &ModuleCallRequest,
) -> Result<M1MoveRuleState, ModuleCallFailure> {
    let Some(state) = state else {
        return Ok(M1MoveRuleState::default());
    };
    if state.is_empty() {
        return Ok(M1MoveRuleState::default());
    }
    decode_input(request, state)
}

fn encode_state(
    state: &M1MoveRuleState,
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

fn decode_input<T: DeserializeOwned>(
    request: &ModuleCallRequest,
    bytes: &[u8],
) -> Result<T, ModuleCallFailure> {
    serde_cbor::from_slice(bytes).map_err(|err| failure(
        request,
        ModuleCallErrorCode::InvalidOutput,
        format!("input CBOR decode failed: {err}"),
    ))
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
