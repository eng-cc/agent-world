use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

use super::{
    action_envelope, empty_output, encode_output, parse_geo_pos, space_distance_cm, GeoPos,
    ModuleCallInput, ModuleEmit, ModuleOutput, M1_POWER_HARVEST_BASE_PER_TICK,
    M1_POWER_HARVEST_DISTANCE_BONUS_CAP, M1_POWER_HARVEST_DISTANCE_STEP_CM,
    M1_POWER_STORAGE_CAPACITY, M1_POWER_STORAGE_INITIAL_LEVEL, M1_POWER_STORAGE_MOVE_COST_PER_KM,
};

const POWER_RADIATION_EMIT_KIND: &str = "power.radiation_harvest";
const CM_PER_KM: i64 = 100_000;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(super) struct AgentPowerState {
    pub(super) pos: GeoPos,
    pub(super) level: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(super) struct PowerState {
    #[serde(default)]
    pub(super) agents: BTreeMap<String, AgentPowerState>,
}

fn decode_power_state(state_bytes: Option<&[u8]>) -> PowerState {
    let Some(state_bytes) = state_bytes else {
        return PowerState::default();
    };
    if state_bytes.is_empty() {
        return PowerState::default();
    }
    serde_cbor::from_slice(state_bytes).unwrap_or_default()
}

fn encode_power_state(state: &PowerState) -> Option<Vec<u8>> {
    serde_cbor::to_vec(state).ok()
}

fn decode_domain_event(event_bytes: &[u8]) -> Option<(String, Value)> {
    let event: Value = serde_cbor::from_slice(event_bytes).ok()?;
    if event
        .get("body")
        .and_then(|body| body.get("kind"))
        .and_then(Value::as_str)
        != Some("Domain")
    {
        return None;
    }
    let payload = event.get("body")?.get("payload")?;
    let event_type = payload.get("type")?.as_str()?.to_string();
    let data = payload.get("data")?.clone();
    Some((event_type, data))
}

fn movement_cost(distance_cm: i64, per_km_cost: i64) -> i64 {
    if distance_cm <= 0 || per_km_cost <= 0 {
        return 0;
    }
    let km = (distance_cm + CM_PER_KM - 1) / CM_PER_KM;
    km.saturating_mul(per_km_cost)
}

fn radiation_harvest_per_tick(pos: GeoPos) -> i64 {
    if M1_POWER_HARVEST_BASE_PER_TICK <= 0 {
        return 0;
    }
    let axis_sum_cm = pos.x_cm.abs() + pos.y_cm.abs() + pos.z_cm.abs();
    let step = M1_POWER_HARVEST_DISTANCE_STEP_CM.max(1) as f64;
    let bonus = (axis_sum_cm / step).floor() as i64;
    let bounded_bonus = bonus.clamp(0, M1_POWER_HARVEST_DISTANCE_BONUS_CAP.max(0));
    M1_POWER_HARVEST_BASE_PER_TICK.saturating_add(bounded_bonus)
}

fn update_radiation_positions(state: &mut PowerState, event_type: &str, data: &Value) -> bool {
    match event_type {
        "AgentRegistered" => {
            let Some(agent_id) = data.get("agent_id").and_then(Value::as_str) else {
                return false;
            };
            let Some(pos) = data.get("pos").and_then(parse_geo_pos) else {
                return false;
            };
            state
                .agents
                .entry(agent_id.to_string())
                .and_modify(|entry| entry.pos = pos)
                .or_insert(AgentPowerState { pos, level: 0 });
            true
        }
        "AgentMoved" => {
            let Some(agent_id) = data.get("agent_id").and_then(Value::as_str) else {
                return false;
            };
            let Some(to) = data.get("to").and_then(parse_geo_pos) else {
                return false;
            };
            let Some(entry) = state.agents.get_mut(agent_id) else {
                return false;
            };
            entry.pos = to;
            true
        }
        _ => false,
    }
}

fn update_storage_state_from_event(state: &mut PowerState, event_type: &str, data: &Value) -> bool {
    let mut changed = match event_type {
        "AgentRegistered" => {
            let Some(agent_id) = data.get("agent_id").and_then(Value::as_str) else {
                return false;
            };
            let Some(pos) = data.get("pos").and_then(parse_geo_pos) else {
                return false;
            };
            let initial_level = M1_POWER_STORAGE_INITIAL_LEVEL
                .min(M1_POWER_STORAGE_CAPACITY)
                .max(0);
            match state.agents.get_mut(agent_id) {
                Some(entry) => {
                    if entry.pos != pos {
                        entry.pos = pos;
                        true
                    } else {
                        false
                    }
                }
                None => {
                    state.agents.insert(
                        agent_id.to_string(),
                        AgentPowerState {
                            pos,
                            level: initial_level,
                        },
                    );
                    true
                }
            }
        }
        "AgentMoved" => {
            let Some(agent_id) = data.get("agent_id").and_then(Value::as_str) else {
                return false;
            };
            let Some(to) = data.get("to").and_then(parse_geo_pos) else {
                return false;
            };
            if let Some(agent_state) = state.agents.get_mut(agent_id) {
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
        if agent_state.level > M1_POWER_STORAGE_CAPACITY {
            agent_state.level = M1_POWER_STORAGE_CAPACITY;
            changed = true;
        }
        if agent_state.level < 0 {
            agent_state.level = 0;
            changed = true;
        }
    }

    changed
}

fn apply_storage_harvest(state: &mut PowerState) -> bool {
    let mut changed = false;
    for agent_state in state.agents.values_mut() {
        let harvested = radiation_harvest_per_tick(agent_state.pos);
        if harvested <= 0 {
            continue;
        }
        let next_level = agent_state
            .level
            .saturating_add(harvested)
            .min(M1_POWER_STORAGE_CAPACITY.max(0));
        if next_level != agent_state.level {
            agent_state.level = next_level;
            changed = true;
        }
    }
    changed
}

fn build_radiation_action_output(input: &ModuleCallInput, mut state: PowerState) -> Vec<u8> {
    let Some((action_id, _)) = action_envelope(input) else {
        return encode_output(empty_output());
    };

    let mut changed = false;
    for agent_state in state.agents.values_mut() {
        let harvested = radiation_harvest_per_tick(agent_state.pos);
        if harvested <= 0 {
            continue;
        }
        agent_state.level = agent_state.level.saturating_add(harvested);
        changed = true;
    }

    let new_state = if changed {
        encode_power_state(&state)
    } else {
        None
    };
    let emit_payload = json!({
        "action_id": action_id,
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

    encode_output(ModuleOutput {
        new_state,
        effects: Vec::new(),
        emits: vec![ModuleEmit {
            kind: POWER_RADIATION_EMIT_KIND.to_string(),
            payload: emit_payload,
        }],
        output_bytes: 0,
    })
}

fn build_radiation_event_output(input: &ModuleCallInput, mut state: PowerState) -> Vec<u8> {
    let Some(event_bytes) = input.event.as_deref() else {
        return encode_output(empty_output());
    };
    let Some((event_type, data)) = decode_domain_event(event_bytes) else {
        return encode_output(empty_output());
    };
    let changed = update_radiation_positions(&mut state, &event_type, &data);
    let new_state = if changed {
        encode_power_state(&state)
    } else {
        None
    };

    encode_output(ModuleOutput {
        new_state,
        effects: Vec::new(),
        emits: Vec::new(),
        output_bytes: 0,
    })
}

pub(super) fn build_radiation_power_module_output(input: &ModuleCallInput) -> Vec<u8> {
    let state = decode_power_state(input.state.as_deref());

    if input.action.is_some() {
        return build_radiation_action_output(input, state);
    }
    if input.event.is_some() {
        return build_radiation_event_output(input, state);
    }
    encode_output(empty_output())
}

fn build_storage_action_output(input: &ModuleCallInput, mut state: PowerState) -> Vec<u8> {
    let mut changed = apply_storage_harvest(&mut state);
    let Some((action_id, action)) = action_envelope(input) else {
        return encode_output(empty_output());
    };
    let Some(action_type) = action.get("type").and_then(Value::as_str) else {
        return encode_output(empty_output());
    };
    if action_type != "MoveAgent" {
        let new_state = if changed {
            encode_power_state(&state)
        } else {
            None
        };
        return encode_output(ModuleOutput {
            new_state,
            effects: Vec::new(),
            emits: Vec::new(),
            output_bytes: 0,
        });
    }

    let Some(agent_id) = action
        .get("data")
        .and_then(|data| data.get("agent_id"))
        .and_then(Value::as_str)
    else {
        return encode_output(empty_output());
    };
    let Some(to) = action
        .get("data")
        .and_then(|data| data.get("to"))
        .and_then(parse_geo_pos)
    else {
        return encode_output(empty_output());
    };

    let mut decision = json!({
        "action_id": action_id,
        "verdict": "allow",
        "cost": { "entries": {} },
        "notes": [],
    });

    let Some(agent_state) = state.agents.get_mut(agent_id) else {
        decision["verdict"] = json!("deny");
        decision["notes"] = json!(["agent power state missing"]);
        let new_state = if changed {
            encode_power_state(&state)
        } else {
            None
        };
        return encode_output(ModuleOutput {
            new_state,
            effects: Vec::new(),
            emits: vec![ModuleEmit {
                kind: "rule.decision".to_string(),
                payload: decision,
            }],
            output_bytes: 0,
        });
    };

    let distance_cm = space_distance_cm(agent_state.pos, to);
    let move_cost = movement_cost(distance_cm, M1_POWER_STORAGE_MOVE_COST_PER_KM);
    if move_cost > agent_state.level {
        decision["verdict"] = json!("deny");
        decision["notes"] = json!([format!(
            "storage insufficient for move: need {move_cost}, have {}",
            agent_state.level
        )]);
    } else {
        agent_state.level = agent_state.level.saturating_sub(move_cost);
        agent_state.pos = to;
        changed = true;
    }

    let new_state = if changed {
        encode_power_state(&state)
    } else {
        None
    };
    encode_output(ModuleOutput {
        new_state,
        effects: Vec::new(),
        emits: vec![ModuleEmit {
            kind: "rule.decision".to_string(),
            payload: decision,
        }],
        output_bytes: 0,
    })
}

fn build_storage_event_output(input: &ModuleCallInput, mut state: PowerState) -> Vec<u8> {
    let Some(event_bytes) = input.event.as_deref() else {
        return encode_output(empty_output());
    };
    let Some((event_type, data)) = decode_domain_event(event_bytes) else {
        return encode_output(empty_output());
    };
    let changed = update_storage_state_from_event(&mut state, &event_type, &data);
    let new_state = if changed {
        encode_power_state(&state)
    } else {
        None
    };

    encode_output(ModuleOutput {
        new_state,
        effects: Vec::new(),
        emits: Vec::new(),
        output_bytes: 0,
    })
}

pub(super) fn build_storage_power_module_output(input: &ModuleCallInput) -> Vec<u8> {
    let state = decode_power_state(input.state.as_deref());

    if input.action.is_some() {
        return build_storage_action_output(input, state);
    }
    if input.event.is_some() {
        return build_storage_event_output(input, state);
    }
    encode_output(empty_output())
}

#[cfg(test)]
mod tests {
    use serde::Serialize;
    use serde_json::{json, Value};

    use super::*;
    use crate::{build_module_output, M1_RADIATION_POWER_MODULE_ID, M1_STORAGE_POWER_MODULE_ID};

    #[derive(Debug, Clone, Serialize)]
    struct ModuleContextTest {
        module_id: String,
        time: u64,
    }

    #[derive(Debug, Clone, Serialize)]
    struct ModuleCallInputTest {
        ctx: ModuleContextTest,
        #[serde(skip_serializing_if = "Option::is_none")]
        event: Option<Vec<u8>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        action: Option<Vec<u8>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        state: Option<Vec<u8>>,
    }

    fn encode_input(
        module_id: &str,
        time: u64,
        action: Option<Value>,
        state: Option<Vec<u8>>,
        event: Option<Value>,
    ) -> Vec<u8> {
        let action_bytes = action.map(|value| serde_cbor::to_vec(&value).expect("encode action"));
        let event_bytes = event.map(|value| serde_cbor::to_vec(&value).expect("encode event"));
        let input = ModuleCallInputTest {
            ctx: ModuleContextTest {
                module_id: module_id.to_string(),
                time,
            },
            event: event_bytes,
            action: action_bytes,
            state,
        };
        serde_cbor::to_vec(&input).expect("encode input")
    }

    fn decode_output(bytes: &[u8]) -> ModuleOutput {
        serde_cbor::from_slice(bytes).expect("decode output")
    }

    #[test]
    fn radiation_power_module_harvests_and_emits() {
        let seed_state = PowerState {
            agents: BTreeMap::from([
                (
                    "agent-1".to_string(),
                    AgentPowerState {
                        pos: GeoPos {
                            x_cm: 0.0,
                            y_cm: 0.0,
                            z_cm: 0.0,
                        },
                        level: 0,
                    },
                ),
                (
                    "agent-2".to_string(),
                    AgentPowerState {
                        pos: GeoPos {
                            x_cm: 800_000.0,
                            y_cm: 800_000.0,
                            z_cm: 0.0,
                        },
                        level: 5,
                    },
                ),
            ]),
        };
        let state_bytes = serde_cbor::to_vec(&seed_state).expect("encode state");
        let action = json!({
            "id": 91u64,
            "action": {
                "type": "QueryObservation",
                "data": {
                    "agent_id": "agent-1"
                }
            }
        });
        let input = encode_input(
            M1_RADIATION_POWER_MODULE_ID,
            0,
            Some(action),
            Some(state_bytes),
            None,
        );

        let output = decode_output(&build_module_output(&input));
        let state_bytes = output.new_state.expect("state bytes");
        let state: PowerState = serde_cbor::from_slice(&state_bytes).expect("decode state");
        assert_eq!(
            state.agents.get("agent-1").map(|entry| entry.level),
            Some(1)
        );
        assert_eq!(
            state.agents.get("agent-2").map(|entry| entry.level),
            Some(7)
        );
        assert_eq!(output.emits.len(), 1);
        assert_eq!(output.emits[0].kind, POWER_RADIATION_EMIT_KIND);
        assert_eq!(output.emits[0].payload["action_id"], json!(91u64));
    }

    #[test]
    fn radiation_power_module_tracks_register_and_move_events() {
        let register_event = json!({
            "id": 92u64,
            "time": 100u64,
            "caused_by": null,
            "body": {
                "kind": "Domain",
                "payload": {
                    "type": "AgentRegistered",
                    "data": {
                        "agent_id": "agent-1",
                        "pos": {"x_cm": 0.0, "y_cm": 0.0, "z_cm": 0.0}
                    }
                }
            }
        });
        let register_input = encode_input(
            M1_RADIATION_POWER_MODULE_ID,
            0,
            None,
            None,
            Some(register_event),
        );
        let register_output = decode_output(&build_module_output(&register_input));
        let state_bytes = register_output.new_state.expect("state bytes");
        let state: PowerState = serde_cbor::from_slice(&state_bytes).expect("decode state");
        assert_eq!(
            state.agents.get("agent-1"),
            Some(&AgentPowerState {
                pos: GeoPos {
                    x_cm: 0.0,
                    y_cm: 0.0,
                    z_cm: 0.0
                },
                level: 0
            })
        );

        let move_event = json!({
            "id": 93u64,
            "time": 101u64,
            "caused_by": null,
            "body": {
                "kind": "Domain",
                "payload": {
                    "type": "AgentMoved",
                    "data": {
                        "agent_id": "agent-1",
                        "from": {"x_cm": 0.0, "y_cm": 0.0, "z_cm": 0.0},
                        "to": {"x_cm": 20.0, "y_cm": 30.0, "z_cm": 0.0}
                    }
                }
            }
        });
        let move_input = encode_input(
            M1_RADIATION_POWER_MODULE_ID,
            0,
            None,
            Some(state_bytes),
            Some(move_event),
        );
        let move_output = decode_output(&build_module_output(&move_input));
        let state_bytes = move_output.new_state.expect("state bytes");
        let state: PowerState = serde_cbor::from_slice(&state_bytes).expect("decode state");
        assert_eq!(
            state.agents.get("agent-1").map(|entry| entry.pos),
            Some(GeoPos {
                x_cm: 20.0,
                y_cm: 30.0,
                z_cm: 0.0
            })
        );
    }

    #[test]
    fn storage_power_module_registers_initial_level() {
        let register_event = json!({
            "id": 94u64,
            "time": 200u64,
            "caused_by": null,
            "body": {
                "kind": "Domain",
                "payload": {
                    "type": "AgentRegistered",
                    "data": {
                        "agent_id": "agent-1",
                        "pos": {"x_cm": 10.0, "y_cm": 0.0, "z_cm": 0.0}
                    }
                }
            }
        });
        let input = encode_input(
            M1_STORAGE_POWER_MODULE_ID,
            0,
            None,
            None,
            Some(register_event),
        );
        let output = decode_output(&build_module_output(&input));
        let state_bytes = output.new_state.expect("state bytes");
        let state: PowerState = serde_cbor::from_slice(&state_bytes).expect("decode state");
        assert_eq!(
            state.agents.get("agent-1").map(|entry| entry.level),
            Some(M1_POWER_STORAGE_INITIAL_LEVEL)
        );
    }

    #[test]
    fn storage_power_module_denies_when_insufficient() {
        let seed_state = PowerState {
            agents: BTreeMap::from([(
                "agent-1".to_string(),
                AgentPowerState {
                    pos: GeoPos {
                        x_cm: 0.0,
                        y_cm: 0.0,
                        z_cm: 0.0,
                    },
                    level: 1,
                },
            )]),
        };
        let state_bytes = serde_cbor::to_vec(&seed_state).expect("encode state");
        let action = json!({
            "id": 95u64,
            "action": {
                "type": "MoveAgent",
                "data": {
                    "agent_id": "agent-1",
                    "to": {"x_cm": 100000.0, "y_cm": 0.0, "z_cm": 0.0}
                }
            }
        });
        let input = encode_input(
            M1_STORAGE_POWER_MODULE_ID,
            0,
            Some(action),
            Some(state_bytes),
            None,
        );

        let output = decode_output(&build_module_output(&input));
        assert_eq!(output.emits.len(), 1);
        assert_eq!(output.emits[0].kind, "rule.decision");
        assert_eq!(output.emits[0].payload["verdict"], json!("deny"));
        let notes = output.emits[0].payload["notes"]
            .as_array()
            .expect("notes array");
        assert!(notes.iter().any(|note| {
            note.as_str()
                .map(|text| text.contains("storage insufficient"))
                .unwrap_or(false)
        }));

        let state_bytes = output.new_state.expect("state bytes");
        let state: PowerState = serde_cbor::from_slice(&state_bytes).expect("decode state");
        assert_eq!(
            state.agents.get("agent-1").map(|entry| entry.level),
            Some(2)
        );
        assert_eq!(
            state.agents.get("agent-1").map(|entry| entry.pos),
            Some(GeoPos {
                x_cm: 0.0,
                y_cm: 0.0,
                z_cm: 0.0
            })
        );
    }

    #[test]
    fn storage_power_module_allows_move_and_consumes_power() {
        let seed_state = PowerState {
            agents: BTreeMap::from([(
                "agent-1".to_string(),
                AgentPowerState {
                    pos: GeoPos {
                        x_cm: 0.0,
                        y_cm: 0.0,
                        z_cm: 0.0,
                    },
                    level: 10,
                },
            )]),
        };
        let state_bytes = serde_cbor::to_vec(&seed_state).expect("encode state");
        let action = json!({
            "id": 96u64,
            "action": {
                "type": "MoveAgent",
                "data": {
                    "agent_id": "agent-1",
                    "to": {"x_cm": 100000.0, "y_cm": 0.0, "z_cm": 0.0}
                }
            }
        });
        let input = encode_input(
            M1_STORAGE_POWER_MODULE_ID,
            0,
            Some(action),
            Some(state_bytes),
            None,
        );

        let output = decode_output(&build_module_output(&input));
        assert_eq!(output.emits[0].payload["verdict"], json!("allow"));
        let state_bytes = output.new_state.expect("state bytes");
        let state: PowerState = serde_cbor::from_slice(&state_bytes).expect("decode state");
        assert_eq!(
            state.agents.get("agent-1").map(|entry| entry.level),
            Some(8)
        );
        assert_eq!(
            state.agents.get("agent-1").map(|entry| entry.pos),
            Some(GeoPos {
                x_cm: 100000.0,
                y_cm: 0.0,
                z_cm: 0.0
            })
        );
    }

    #[test]
    fn storage_power_module_clamps_levels_on_event() {
        let seed_state = PowerState {
            agents: BTreeMap::from([
                (
                    "agent-1".to_string(),
                    AgentPowerState {
                        pos: GeoPos {
                            x_cm: 0.0,
                            y_cm: 0.0,
                            z_cm: 0.0,
                        },
                        level: 100,
                    },
                ),
                (
                    "agent-2".to_string(),
                    AgentPowerState {
                        pos: GeoPos {
                            x_cm: 0.0,
                            y_cm: 0.0,
                            z_cm: 0.0,
                        },
                        level: -5,
                    },
                ),
            ]),
        };
        let state_bytes = serde_cbor::to_vec(&seed_state).expect("encode state");
        let moved_event = json!({
            "id": 97u64,
            "time": 210u64,
            "caused_by": null,
            "body": {
                "kind": "Domain",
                "payload": {
                    "type": "AgentMoved",
                    "data": {
                        "agent_id": "agent-1",
                        "from": {"x_cm": 0.0, "y_cm": 0.0, "z_cm": 0.0},
                        "to": {"x_cm": 10.0, "y_cm": 0.0, "z_cm": 0.0}
                    }
                }
            }
        });
        let input = encode_input(
            M1_STORAGE_POWER_MODULE_ID,
            0,
            None,
            Some(state_bytes),
            Some(moved_event),
        );
        let output = decode_output(&build_module_output(&input));
        let state_bytes = output.new_state.expect("state bytes");
        let state: PowerState = serde_cbor::from_slice(&state_bytes).expect("decode state");
        assert_eq!(
            state.agents.get("agent-1").map(|entry| entry.level),
            Some(M1_POWER_STORAGE_CAPACITY)
        );
        assert_eq!(
            state.agents.get("agent-2").map(|entry| entry.level),
            Some(0)
        );
    }
}
