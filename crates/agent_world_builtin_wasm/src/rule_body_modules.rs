use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

use super::{
    action_envelope, empty_output, encode_output, parse_geo_pos, space_distance_cm, GeoPos,
    ModuleCallInput, ModuleEmit, ModuleOutput, DEFAULT_MOVE_COST_PER_KM_ELECTRICITY,
    DEFAULT_VISIBILITY_RANGE_CM, M1_BODY_ACTION_COST_ELECTRICITY, RULE_DECISION_EMIT_KIND,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct BodyKernelView {
    mass_kg: u64,
    radius_cm: u64,
    thrust_limit: u64,
    cross_section_cm2: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub(crate) struct PositionState {
    #[serde(default)]
    pub(crate) agents: BTreeMap<String, GeoPos>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct Observation {
    time: u64,
    agent_id: String,
    pos: GeoPos,
    visibility_range_cm: i64,
    visible_agents: Vec<ObservedAgent>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
struct ObservedAgent {
    agent_id: String,
    pos: GeoPos,
    distance_cm: i64,
}

fn decode_state(state_bytes: Option<&[u8]>) -> PositionState {
    let Some(state_bytes) = state_bytes else {
        return PositionState::default();
    };
    if state_bytes.is_empty() {
        return PositionState::default();
    }
    serde_cbor::from_slice(state_bytes).unwrap_or_default()
}

fn encode_state(state: &PositionState) -> Option<Vec<u8>> {
    serde_cbor::to_vec(state).ok()
}

fn rule_emit_output(decision_payload: Value) -> Vec<u8> {
    let output = ModuleOutput {
        new_state: None,
        effects: Vec::new(),
        emits: vec![ModuleEmit {
            kind: RULE_DECISION_EMIT_KIND.to_string(),
            payload: decision_payload,
        }],
        output_bytes: 0,
    };
    encode_output(output)
}

fn update_position_state_from_event(state: &mut PositionState, event_bytes: &[u8]) -> bool {
    let event: Value = match serde_cbor::from_slice(event_bytes) {
        Ok(value) => value,
        Err(_) => return false,
    };
    let body = match event.get("body") {
        Some(body) => body,
        None => return false,
    };
    if body.get("kind").and_then(Value::as_str) != Some("Domain") {
        return false;
    }
    let payload = match body.get("payload") {
        Some(payload) => payload,
        None => return false,
    };
    let event_type = match payload.get("type").and_then(Value::as_str) {
        Some(event_type) => event_type,
        None => return false,
    };
    let data = match payload.get("data") {
        Some(data) => data,
        None => return false,
    };

    match event_type {
        "AgentRegistered" => {
            let Some(agent_id) = data.get("agent_id").and_then(Value::as_str) else {
                return false;
            };
            let Some(pos) = data.get("pos").and_then(parse_geo_pos) else {
                return false;
            };
            state.agents.insert(agent_id.to_string(), pos);
            true
        }
        "AgentMoved" => {
            let Some(agent_id) = data.get("agent_id").and_then(Value::as_str) else {
                return false;
            };
            let Some(pos) = data.get("to").and_then(parse_geo_pos) else {
                return false;
            };
            state.agents.insert(agent_id.to_string(), pos);
            true
        }
        _ => false,
    }
}

fn movement_cost(distance_cm: i64, per_km_cost: i64) -> i64 {
    if distance_cm <= 0 || per_km_cost <= 0 {
        return 0;
    }
    let km = (distance_cm + super::CM_PER_KM - 1) / super::CM_PER_KM;
    km.saturating_mul(per_km_cost)
}

fn build_move_rule_action_output(input: &ModuleCallInput, state: &PositionState) -> Vec<u8> {
    let Some((action_id, action)) = action_envelope(input) else {
        return encode_output(empty_output());
    };
    if action.get("type").and_then(Value::as_str) != Some("MoveAgent") {
        return encode_output(empty_output());
    }
    let Some(data) = action.get("data") else {
        return encode_output(empty_output());
    };
    let Some(agent_id) = data.get("agent_id").and_then(Value::as_str) else {
        return encode_output(empty_output());
    };
    let Some(to) = data.get("to").and_then(parse_geo_pos) else {
        return encode_output(empty_output());
    };

    let mut decision = json!({
        "action_id": action_id,
        "verdict": "allow",
        "cost": { "entries": {} },
        "notes": [],
    });

    match state.agents.get(agent_id).copied() {
        Some(from) => {
            let distance_cm = space_distance_cm(from, to);
            if distance_cm == 0 {
                decision["verdict"] = json!("deny");
                decision["notes"] = json!(["move target equals current position"]);
            } else {
                let cost = movement_cost(distance_cm, DEFAULT_MOVE_COST_PER_KM_ELECTRICITY);
                if cost > 0 {
                    decision["cost"] = json!({
                        "entries": {
                            "electricity": -cost,
                        }
                    });
                }
            }
        }
        None => {
            decision["notes"] = json!(["agent position missing for move rule"]);
        }
    }

    rule_emit_output(decision)
}

pub(super) fn build_move_rule_output(input: &ModuleCallInput) -> Vec<u8> {
    let state = decode_state(input.state.as_deref());
    if input.action.is_some() {
        return build_move_rule_action_output(input, &state);
    }
    if input.event.is_some() {
        return build_state_tracking_event_output(input.event.as_deref(), state);
    }
    encode_output(empty_output())
}

fn build_visibility_rule_action_output(input: &ModuleCallInput, state: &PositionState) -> Vec<u8> {
    let Some((action_id, action)) = action_envelope(input) else {
        return encode_output(empty_output());
    };
    if action.get("type").and_then(Value::as_str) != Some("QueryObservation") {
        return encode_output(empty_output());
    }
    let Some(agent_id) = action
        .get("data")
        .and_then(|data| data.get("agent_id"))
        .and_then(Value::as_str)
    else {
        return encode_output(empty_output());
    };

    let mut decision = json!({
        "action_id": action_id,
        "verdict": "modify",
        "cost": { "entries": {} },
        "notes": [],
    });

    let Some(origin) = state.agents.get(agent_id).copied() else {
        decision["verdict"] = json!("deny");
        decision["notes"] = json!(["agent position missing for visibility rule"]);
        return rule_emit_output(decision);
    };

    let mut visible_agents = Vec::new();
    for (other_id, other_pos) in &state.agents {
        if other_id == agent_id {
            continue;
        }
        let distance_cm = space_distance_cm(origin, *other_pos);
        if distance_cm <= DEFAULT_VISIBILITY_RANGE_CM {
            visible_agents.push(ObservedAgent {
                agent_id: other_id.clone(),
                pos: *other_pos,
                distance_cm,
            });
        }
    }

    let observation = Observation {
        time: input.ctx.time,
        agent_id: agent_id.to_string(),
        pos: origin,
        visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
        visible_agents,
    };

    decision["override_action"] = json!({
        "type": "EmitObservation",
        "data": {
            "observation": observation
        }
    });
    rule_emit_output(decision)
}

fn build_state_tracking_event_output(
    event_bytes: Option<&[u8]>,
    mut state: PositionState,
) -> Vec<u8> {
    let Some(event_bytes) = event_bytes else {
        return encode_output(empty_output());
    };
    let changed = update_position_state_from_event(&mut state, event_bytes);
    let new_state = if changed { encode_state(&state) } else { None };
    encode_output(ModuleOutput {
        new_state,
        effects: Vec::new(),
        emits: Vec::new(),
        output_bytes: 0,
    })
}

pub(super) fn build_visibility_rule_output(input: &ModuleCallInput) -> Vec<u8> {
    let state = decode_state(input.state.as_deref());
    if input.action.is_some() {
        return build_visibility_rule_action_output(input, &state);
    }
    if input.event.is_some() {
        return build_state_tracking_event_output(input.event.as_deref(), state);
    }
    encode_output(empty_output())
}

fn build_transfer_rule_action_output(input: &ModuleCallInput, state: &PositionState) -> Vec<u8> {
    let Some((action_id, action)) = action_envelope(input) else {
        return encode_output(empty_output());
    };
    if action.get("type").and_then(Value::as_str) != Some("TransferResource") {
        return encode_output(empty_output());
    }
    let Some(data) = action.get("data") else {
        return encode_output(empty_output());
    };
    let Some(from_agent_id) = data.get("from_agent_id").and_then(Value::as_str) else {
        return encode_output(empty_output());
    };
    let Some(to_agent_id) = data.get("to_agent_id").and_then(Value::as_str) else {
        return encode_output(empty_output());
    };
    let Some(kind) = data.get("kind").and_then(Value::as_str) else {
        return encode_output(empty_output());
    };
    let Some(amount) = data.get("amount").and_then(Value::as_i64) else {
        return encode_output(empty_output());
    };

    let mut decision = json!({
        "action_id": action_id,
        "verdict": "modify",
        "cost": { "entries": {} },
        "notes": [],
    });

    if amount <= 0 {
        decision["verdict"] = json!("deny");
        decision["notes"] = json!(["transfer amount must be positive"]);
        return rule_emit_output(decision);
    }

    let from_pos = state.agents.get(from_agent_id).copied();
    let to_pos = state.agents.get(to_agent_id).copied();
    match (from_pos, to_pos) {
        (Some(from_pos), Some(to_pos)) => {
            let distance_cm = space_distance_cm(from_pos, to_pos);
            if distance_cm > 0 {
                decision["verdict"] = json!("deny");
                decision["notes"] = json!(["transfer requires co-located agents"]);
            } else {
                decision["override_action"] = json!({
                    "type": "EmitResourceTransfer",
                    "data": {
                        "from_agent_id": from_agent_id,
                        "to_agent_id": to_agent_id,
                        "kind": kind,
                        "amount": amount,
                    }
                });
            }
        }
        _ => {
            decision["verdict"] = json!("deny");
            decision["notes"] = json!(["agent position missing for transfer rule"]);
        }
    }
    rule_emit_output(decision)
}

pub(super) fn build_transfer_rule_output(input: &ModuleCallInput) -> Vec<u8> {
    let state = decode_state(input.state.as_deref());
    if input.action.is_some() {
        return build_transfer_rule_action_output(input, &state);
    }
    if input.event.is_some() {
        return build_state_tracking_event_output(input.event.as_deref(), state);
    }
    encode_output(empty_output())
}

fn build_body_module_action_output(input: &ModuleCallInput) -> Vec<u8> {
    let Some((action_id, action)) = action_envelope(input) else {
        return encode_output(empty_output());
    };
    if action.get("type").and_then(Value::as_str) != Some("BodyAction") {
        return encode_output(empty_output());
    }
    let Some(data) = action.get("data") else {
        return encode_output(empty_output());
    };
    let Some(agent_id) = data.get("agent_id").and_then(Value::as_str) else {
        return encode_output(empty_output());
    };
    let Some(kind) = data.get("kind").and_then(Value::as_str) else {
        return encode_output(empty_output());
    };
    let Some(payload) = data.get("payload") else {
        return encode_output(empty_output());
    };

    let mut decision = json!({
        "action_id": action_id,
        "verdict": "allow",
        "cost": { "entries": {} },
        "notes": [],
    });

    let view: BodyKernelView = match serde_json::from_value(payload.clone()) {
        Ok(view) => view,
        Err(err) => {
            decision["verdict"] = json!("deny");
            decision["notes"] = json!([format!("body action payload decode failed: {err}")]);
            return rule_emit_output(decision);
        }
    };

    decision["verdict"] = json!("modify");
    decision["override_action"] = json!({
        "type": "EmitBodyAttributes",
        "data": {
            "agent_id": agent_id,
            "view": view,
            "reason": format!("body.{kind}"),
        }
    });
    if M1_BODY_ACTION_COST_ELECTRICITY > 0 {
        decision["cost"] = json!({
            "entries": {
                "electricity": -M1_BODY_ACTION_COST_ELECTRICITY
            }
        });
    }
    rule_emit_output(decision)
}

pub(super) fn build_body_module_output(input: &ModuleCallInput) -> Vec<u8> {
    if input.action.is_some() {
        return build_body_module_action_output(input);
    }
    encode_output(empty_output())
}
