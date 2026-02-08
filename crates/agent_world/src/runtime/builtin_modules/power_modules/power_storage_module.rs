use std::collections::btree_map::Entry;

use crate::geometry::space_distance_cm;

use super::super::super::events::{Action, ActionEnvelope};
use super::super::super::rules::{ResourceDelta, RuleDecision, RuleVerdict};
use super::super::super::sandbox::{
    ModuleCallErrorCode, ModuleCallFailure, ModuleCallInput, ModuleCallRequest, ModuleEmit,
    ModuleOutput,
};
use super::super::super::world_event::{WorldEvent, WorldEventBody};
use super::super::BuiltinModule;
use super::{
    decode_input, decode_state, encode_state, failure, finalize_output, movement_cost,
    radiation_harvest_per_tick, AgentPowerState, PowerState, M1_POWER_HARVEST_BASE_PER_TICK,
    M1_POWER_HARVEST_DISTANCE_BONUS_CAP, M1_POWER_HARVEST_DISTANCE_STEP_CM,
    M1_POWER_STORAGE_CAPACITY, M1_POWER_STORAGE_INITIAL_LEVEL, M1_POWER_STORAGE_MOVE_COST_PER_KM,
};

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
            WorldEventBody::Domain(super::super::super::events::DomainEvent::AgentRegistered {
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
            WorldEventBody::Domain(super::super::super::events::DomainEvent::AgentMoved {
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

impl BuiltinModule for M1StoragePowerModule {
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
