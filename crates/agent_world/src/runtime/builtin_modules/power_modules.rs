use std::collections::{btree_map::Entry, BTreeMap};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::geometry::{space_distance_cm, GeoPos};
use crate::simulator::CM_PER_KM;

use super::super::events::{Action, ActionEnvelope};
use super::super::rules::{ResourceDelta, RuleDecision, RuleVerdict};
use super::super::sandbox::{
    ModuleCallErrorCode, ModuleCallFailure, ModuleCallInput, ModuleCallRequest, ModuleEmit,
    ModuleOutput,
};
use super::super::util::to_canonical_cbor;
use super::super::world_event::{WorldEvent, WorldEventBody};

pub const M1_RADIATION_POWER_MODULE_ID: &str = "m1.power.radiation_harvest";
pub const M1_STORAGE_POWER_MODULE_ID: &str = "m1.power.storage";
pub const M1_POWER_MODULE_VERSION: &str = "0.1.0";
pub const M1_POWER_STORAGE_CAPACITY: i64 = 12;
pub const M1_POWER_STORAGE_INITIAL_LEVEL: i64 = 6;
pub const M1_POWER_STORAGE_MOVE_COST_PER_KM: i64 = 3;
pub const M1_POWER_HARVEST_BASE_PER_TICK: i64 = 1;
pub const M1_POWER_HARVEST_DISTANCE_STEP_CM: i64 = 800_000;
pub const M1_POWER_HARVEST_DISTANCE_BONUS_CAP: i64 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AgentPowerState {
    pos: GeoPos,
    level: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PowerState {
    agents: BTreeMap<String, AgentPowerState>,
}

#[derive(Debug, Clone)]
pub struct M1RadiationPowerModule {
    base_per_tick: i64,
    distance_step_cm: i64,
    distance_bonus_cap: i64,
}

impl Default for M1RadiationPowerModule {
    fn default() -> Self {
        Self {
            base_per_tick: M1_POWER_HARVEST_BASE_PER_TICK,
            distance_step_cm: M1_POWER_HARVEST_DISTANCE_STEP_CM,
            distance_bonus_cap: M1_POWER_HARVEST_DISTANCE_BONUS_CAP,
        }
    }
}

impl M1RadiationPowerModule {
    fn handle_action(
        &self,
        request: &ModuleCallRequest,
        envelope: ActionEnvelope,
        mut state: PowerState,
    ) -> Result<ModuleOutput, ModuleCallFailure> {
        let mut changed = false;
        for agent_state in state.agents.values_mut() {
            let harvested = radiation_harvest_per_tick(
                agent_state.pos,
                self.base_per_tick,
                self.distance_step_cm,
                self.distance_bonus_cap,
            );
            if harvested <= 0 {
                continue;
            }
            agent_state.level = agent_state.level.saturating_add(harvested);
            changed = true;
        }

        let new_state = if changed {
            Some(encode_state(&state, request)?)
        } else {
            None
        };

        let emit_payload = json!({
            "action_id": envelope.id,
            "agents": state
                .agents
                .iter()
                .map(|(agent_id, power)| {
                    json!({
                        "agent_id": agent_id,
                        "level": power.level,
                    })
                })
                .collect::<Vec<_>>()
        });

        finalize_output(
            ModuleOutput {
                new_state,
                effects: Vec::new(),
                emits: vec![ModuleEmit {
                    kind: "power.radiation_harvest".to_string(),
                    payload: emit_payload,
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
        mut state: PowerState,
    ) -> Result<ModuleOutput, ModuleCallFailure> {
        let changed = update_power_state_positions(&mut state, event);
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

#[derive(Debug, Clone)]
pub struct M1StoragePowerModule {
    capacity: i64,
    initial_level: i64,
    move_cost_per_km: i64,
    harvest_base_per_tick: i64,
    harvest_distance_step_cm: i64,
    harvest_distance_bonus_cap: i64,
}

impl Default for M1StoragePowerModule {
    fn default() -> Self {
        Self {
            capacity: M1_POWER_STORAGE_CAPACITY,
            initial_level: M1_POWER_STORAGE_INITIAL_LEVEL,
            move_cost_per_km: M1_POWER_STORAGE_MOVE_COST_PER_KM,
            harvest_base_per_tick: M1_POWER_HARVEST_BASE_PER_TICK,
            harvest_distance_step_cm: M1_POWER_HARVEST_DISTANCE_STEP_CM,
            harvest_distance_bonus_cap: M1_POWER_HARVEST_DISTANCE_BONUS_CAP,
        }
    }
}

impl M1StoragePowerModule {
    fn handle_action(
        &self,
        request: &ModuleCallRequest,
        envelope: ActionEnvelope,
        mut state: PowerState,
    ) -> Result<ModuleOutput, ModuleCallFailure> {
        let ActionEnvelope { id, action } = envelope;

        let mut changed = false;
        for agent_state in state.agents.values_mut() {
            let harvested = radiation_harvest_per_tick(
                agent_state.pos,
                self.harvest_base_per_tick,
                self.harvest_distance_step_cm,
                self.harvest_distance_bonus_cap,
            );
            if harvested <= 0 {
                continue;
            }
            let next_level = agent_state
                .level
                .saturating_add(harvested)
                .min(self.capacity.max(0));
            if next_level != agent_state.level {
                agent_state.level = next_level;
                changed = true;
            }
        }

        let (agent_id, to) = match action {
            Action::MoveAgent { agent_id, to } => (agent_id, to),
            _ => {
                let new_state = if changed {
                    Some(encode_state(&state, request)?)
                } else {
                    None
                };
                return finalize_output(
                    ModuleOutput {
                        new_state,
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

        let Some(agent_state) = state.agents.get_mut(&agent_id) else {
            decision.verdict = RuleVerdict::Deny;
            decision.notes.push("agent power state missing".to_string());
            let new_state = if changed {
                Some(encode_state(&state, request)?)
            } else {
                None
            };
            return finalize_output(
                ModuleOutput {
                    new_state,
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

        let distance_cm = space_distance_cm(agent_state.pos, to);
        let move_cost = movement_cost(distance_cm, self.move_cost_per_km);
        if move_cost > agent_state.level {
            decision.verdict = RuleVerdict::Deny;
            decision.notes.push(format!(
                "storage insufficient for move: need {move_cost}, have {}",
                agent_state.level
            ));
        } else {
            agent_state.level = agent_state.level.saturating_sub(move_cost);
            agent_state.pos = to;
            changed = true;
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
        mut state: PowerState,
    ) -> Result<ModuleOutput, ModuleCallFailure> {
        let mut changed = match event.body {
            WorldEventBody::Domain(super::super::events::DomainEvent::AgentRegistered {
                agent_id,
                pos,
            }) => match state.agents.entry(agent_id) {
                Entry::Vacant(vacant) => {
                    vacant.insert(AgentPowerState {
                        pos,
                        level: self.initial_level.min(self.capacity).max(0),
                    });
                    true
                }
                Entry::Occupied(mut occupied) => {
                    if occupied.get().pos != pos {
                        occupied.get_mut().pos = pos;
                        true
                    } else {
                        false
                    }
                }
            },
            WorldEventBody::Domain(super::super::events::DomainEvent::AgentMoved {
                agent_id,
                to,
                ..
            }) => {
                if let Some(agent_state) = state.agents.get_mut(&agent_id) {
                    if agent_state.pos != to {
                        agent_state.pos = to;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            _ => false,
        };

        for agent_state in state.agents.values_mut() {
            if agent_state.level > self.capacity {
                agent_state.level = self.capacity;
                changed = true;
            }
            if agent_state.level < 0 {
                agent_state.level = 0;
                changed = true;
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

impl super::BuiltinModule for M1RadiationPowerModule {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        let input = decode_input::<ModuleCallInput>(request, &request.input)?;
        let state: PowerState = decode_state(input.state.as_deref(), request)?;

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

impl super::BuiltinModule for M1StoragePowerModule {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        let input = decode_input::<ModuleCallInput>(request, &request.input)?;
        let state: PowerState = decode_state(input.state.as_deref(), request)?;

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

fn update_power_state_positions(state: &mut PowerState, event: WorldEvent) -> bool {
    let mut changed = false;
    if let WorldEventBody::Domain(domain) = event.body {
        match domain {
            super::super::events::DomainEvent::AgentRegistered { agent_id, pos } => {
                state
                    .agents
                    .entry(agent_id)
                    .and_modify(|entry| entry.pos = pos)
                    .or_insert(AgentPowerState { pos, level: 0 });
                changed = true;
            }
            super::super::events::DomainEvent::AgentMoved { agent_id, to, .. } => {
                if let Some(entry) = state.agents.get_mut(&agent_id) {
                    entry.pos = to;
                    changed = true;
                }
            }
            super::super::events::DomainEvent::ActionRejected { .. } => {}
            super::super::events::DomainEvent::Observation { .. } => {}
            super::super::events::DomainEvent::BodyAttributesUpdated { .. } => {}
            super::super::events::DomainEvent::BodyAttributesRejected { .. } => {}
            super::super::events::DomainEvent::ResourceTransferred { .. } => {}
        }
    }
    changed
}

fn radiation_harvest_per_tick(
    pos: GeoPos,
    base_per_tick: i64,
    distance_step_cm: i64,
    bonus_cap: i64,
) -> i64 {
    if base_per_tick <= 0 {
        return 0;
    }
    let axis_sum_cm = pos.x_cm.abs() + pos.y_cm.abs() + pos.z_cm.abs();
    let step = distance_step_cm.max(1) as f64;
    let bonus = (axis_sum_cm / step).floor() as i64;
    let bounded_bonus = bonus.clamp(0, bonus_cap.max(0));
    base_per_tick.saturating_add(bounded_bonus)
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
