use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub const M1_MOVE_RULE_MODULE_ID: &str = "m1.rule.move";
pub const M1_VISIBILITY_RULE_MODULE_ID: &str = "m1.rule.visibility";
pub const M1_TRANSFER_RULE_MODULE_ID: &str = "m1.rule.transfer";
const DEFAULT_VISIBILITY_RANGE_CM: i64 = 10_000_000;
const RULE_DECISION_EMIT_KIND: &str = "rule.decision";

#[derive(Debug, Clone, Deserialize)]
struct ModuleCallInput {
    ctx: ModuleContext,
    #[serde(default)]
    event: Option<Vec<u8>>,
    #[serde(default)]
    action: Option<Vec<u8>>,
    #[serde(default)]
    state: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Deserialize)]
struct ModuleContext {
    module_id: String,
    time: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
struct GeoPos {
    x_cm: f64,
    y_cm: f64,
    z_cm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
struct PositionState {
    #[serde(default)]
    agents: BTreeMap<String, GeoPos>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ModuleEffectIntent {
    kind: String,
    params: Value,
    cap_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ModuleEmit {
    kind: String,
    payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ModuleOutput {
    new_state: Option<Vec<u8>>,
    #[serde(default)]
    effects: Vec<ModuleEffectIntent>,
    #[serde(default)]
    emits: Vec<ModuleEmit>,
    output_bytes: u64,
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

fn empty_output() -> ModuleOutput {
    ModuleOutput {
        new_state: None,
        effects: Vec::new(),
        emits: Vec::new(),
        output_bytes: 0,
    }
}

fn encode_output(output: ModuleOutput) -> Vec<u8> {
    serde_cbor::to_vec(&output).unwrap_or_default()
}

fn decode_input(input_bytes: &[u8]) -> Option<ModuleCallInput> {
    serde_cbor::from_slice(input_bytes).ok()
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

fn action_envelope(input: &ModuleCallInput) -> Option<(u64, Value)> {
    let action_bytes = input.action.as_deref()?;
    if action_bytes.is_empty() {
        return None;
    }
    let action: Value = serde_cbor::from_slice(action_bytes).ok()?;
    let action_id = action.get("id")?.as_u64()?;
    let action_payload = action.get("action")?.clone();
    Some((action_id, action_payload))
}

fn parse_geo_pos(value: &Value) -> Option<GeoPos> {
    Some(GeoPos {
        x_cm: value.get("x_cm")?.as_f64()?,
        y_cm: value.get("y_cm")?.as_f64()?,
        z_cm: value.get("z_cm")?.as_f64()?,
    })
}

fn space_distance_cm(a: GeoPos, b: GeoPos) -> i64 {
    let dx_m = (a.x_cm - b.x_cm) / 100.0;
    let dy_m = (a.y_cm - b.y_cm) / 100.0;
    let dz_m = (a.z_cm - b.z_cm) / 100.0;
    ((dx_m * dx_m + dy_m * dy_m + dz_m * dz_m).sqrt() * 100.0)
        .round()
        .max(0.0) as i64
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

fn build_move_rule_output(input: &ModuleCallInput) -> Vec<u8> {
    let action_id = action_envelope(input).map(|(id, _)| id).unwrap_or(0);
    let payload = json!({
        "action_id": action_id,
        "verdict": "allow",
    });
    rule_emit_output(payload)
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

fn build_visibility_rule_output(input: &ModuleCallInput) -> Vec<u8> {
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

fn build_transfer_rule_output(input: &ModuleCallInput) -> Vec<u8> {
    let state = decode_state(input.state.as_deref());
    if input.action.is_some() {
        return build_transfer_rule_action_output(input, &state);
    }
    if input.event.is_some() {
        return build_state_tracking_event_output(input.event.as_deref(), state);
    }
    encode_output(empty_output())
}

fn build_module_output(input_bytes: &[u8]) -> Vec<u8> {
    let Some(input) = decode_input(input_bytes) else {
        return encode_output(empty_output());
    };
    match input.ctx.module_id.as_str() {
        M1_MOVE_RULE_MODULE_ID => build_move_rule_output(&input),
        M1_VISIBILITY_RULE_MODULE_ID => build_visibility_rule_output(&input),
        M1_TRANSFER_RULE_MODULE_ID => build_transfer_rule_output(&input),
        _ => encode_output(empty_output()),
    }
}

fn call_impl(input_ptr: i32, input_len: i32) -> (i32, i32) {
    let input = if input_ptr > 0 && input_len > 0 {
        let ptr = input_ptr as *const u8;
        let len = input_len as usize;
        // SAFETY: host guarantees valid wasm linear memory pointer/len for the call.
        unsafe { std::slice::from_raw_parts(ptr, len).to_vec() }
    } else {
        Vec::new()
    };
    let output = build_module_output(&input);
    write_bytes_to_memory(&output)
}

fn write_bytes_to_memory(bytes: &[u8]) -> (i32, i32) {
    let len = i32::try_from(bytes.len()).unwrap_or(0);
    if len <= 0 {
        return (0, 0);
    }
    let ptr = alloc(len);
    if ptr <= 0 {
        return (0, 0);
    }
    // SAFETY: alloc returns a writable wasm linear memory region with at least len bytes.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, len as usize);
    }
    (ptr, len)
}

#[no_mangle]
pub extern "C" fn alloc(len: i32) -> i32 {
    if len <= 0 {
        return 0;
    }
    let capacity = len as usize;
    let mut buf = Vec::<u8>::with_capacity(capacity);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr as i32
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn reduce(input_ptr: i32, input_len: i32) -> (i32, i32) {
    call_impl(input_ptr, input_len)
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn call(input_ptr: i32, input_len: i32) -> (i32, i32) {
    call_impl(input_ptr, input_len)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn move_rule_output_uses_action_id_from_input() {
        let action = json!({
            "id": 42u64,
            "action": {
                "type": "MoveAgent",
                "data": {"agent_id":"a-1","to":{"x_cm": 1.0, "y_cm": 2.0, "z_cm": 0.0}}
            }
        });
        let input = encode_input(M1_MOVE_RULE_MODULE_ID, 10, Some(action), None, None);
        let output_bytes = build_module_output(&input);
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        assert_eq!(output.emits.len(), 1);
        assert_eq!(output.emits[0].kind, RULE_DECISION_EMIT_KIND);
        assert_eq!(output.emits[0].payload["action_id"], json!(42u64));
        assert_eq!(output.emits[0].payload["verdict"], json!("allow"));
    }

    #[test]
    fn visibility_rule_emits_observation_override() {
        let mut agents = BTreeMap::new();
        agents.insert(
            "agent-1".to_string(),
            GeoPos {
                x_cm: 0.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
        );
        agents.insert(
            "agent-2".to_string(),
            GeoPos {
                x_cm: 10.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
        );
        let state_bytes = serde_cbor::to_vec(&PositionState { agents }).expect("encode state");
        let action = json!({
            "id": 7u64,
            "action": {
                "type": "QueryObservation",
                "data": {"agent_id":"agent-1"}
            }
        });
        let input = encode_input(
            M1_VISIBILITY_RULE_MODULE_ID,
            123,
            Some(action),
            Some(state_bytes),
            None,
        );

        let output_bytes = build_module_output(&input);
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        assert_eq!(output.emits.len(), 1);
        let payload = &output.emits[0].payload;
        assert_eq!(payload["action_id"], json!(7u64));
        assert_eq!(payload["verdict"], json!("modify"));
        assert_eq!(payload["override_action"]["type"], json!("EmitObservation"));
        assert_eq!(
            payload["override_action"]["data"]["observation"]["agent_id"],
            json!("agent-1")
        );
        assert_eq!(
            payload["override_action"]["data"]["observation"]["time"],
            json!(123u64)
        );
        assert_eq!(
            payload["override_action"]["data"]["observation"]["visible_agents"][0]["agent_id"],
            json!("agent-2")
        );
    }

    #[test]
    fn visibility_rule_denies_when_agent_missing() {
        let action = json!({
            "id": 9u64,
            "action": {
                "type": "QueryObservation",
                "data": {"agent_id":"missing-agent"}
            }
        });
        let input = encode_input(M1_VISIBILITY_RULE_MODULE_ID, 456, Some(action), None, None);

        let output_bytes = build_module_output(&input);
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        assert_eq!(output.emits.len(), 1);
        let payload = &output.emits[0].payload;
        assert_eq!(payload["action_id"], json!(9u64));
        assert_eq!(payload["verdict"], json!("deny"));
        assert_eq!(
            payload["notes"][0],
            json!("agent position missing for visibility rule")
        );
    }

    #[test]
    fn visibility_rule_updates_position_state_on_event() {
        let event = json!({
            "id": 1u64,
            "time": 1u64,
            "caused_by": null,
            "body": {
                "kind": "Domain",
                "payload": {
                    "type": "AgentRegistered",
                    "data": {
                        "agent_id": "agent-9",
                        "pos": {"x_cm": 10.0, "y_cm": 20.0, "z_cm": 0.0}
                    }
                }
            }
        });
        let input = encode_input(M1_VISIBILITY_RULE_MODULE_ID, 0, None, None, Some(event));

        let output_bytes = build_module_output(&input);
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        let state_bytes = output.new_state.expect("state bytes");
        let state: PositionState = serde_cbor::from_slice(&state_bytes).expect("decode state");
        assert_eq!(state.agents.len(), 1);
        assert_eq!(
            state.agents.get("agent-9"),
            Some(&GeoPos {
                x_cm: 10.0,
                y_cm: 20.0,
                z_cm: 0.0
            })
        );
    }

    #[test]
    fn transfer_rule_emits_resource_transfer_override_when_colocated() {
        let mut agents = BTreeMap::new();
        let pos = GeoPos {
            x_cm: 10.0,
            y_cm: 20.0,
            z_cm: 0.0,
        };
        agents.insert("agent-1".to_string(), pos);
        agents.insert("agent-2".to_string(), pos);
        let state_bytes = serde_cbor::to_vec(&PositionState { agents }).expect("encode state");
        let action = json!({
            "id": 31u64,
            "action": {
                "type": "TransferResource",
                "data": {
                    "from_agent_id": "agent-1",
                    "to_agent_id": "agent-2",
                    "kind": "electricity",
                    "amount": 3i64
                }
            }
        });
        let input = encode_input(
            M1_TRANSFER_RULE_MODULE_ID,
            100,
            Some(action),
            Some(state_bytes),
            None,
        );

        let output_bytes = build_module_output(&input);
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        assert_eq!(output.emits.len(), 1);
        let payload = &output.emits[0].payload;
        assert_eq!(payload["action_id"], json!(31u64));
        assert_eq!(payload["verdict"], json!("modify"));
        assert_eq!(
            payload["override_action"]["type"],
            json!("EmitResourceTransfer")
        );
        assert_eq!(
            payload["override_action"]["data"]["from_agent_id"],
            json!("agent-1")
        );
        assert_eq!(
            payload["override_action"]["data"]["to_agent_id"],
            json!("agent-2")
        );
        assert_eq!(
            payload["override_action"]["data"]["kind"],
            json!("electricity")
        );
        assert_eq!(payload["override_action"]["data"]["amount"], json!(3i64));
    }

    #[test]
    fn transfer_rule_denies_when_amount_not_positive() {
        let action = json!({
            "id": 32u64,
            "action": {
                "type": "TransferResource",
                "data": {
                    "from_agent_id": "agent-1",
                    "to_agent_id": "agent-2",
                    "kind": "electricity",
                    "amount": 0i64
                }
            }
        });
        let input = encode_input(M1_TRANSFER_RULE_MODULE_ID, 0, Some(action), None, None);

        let output_bytes = build_module_output(&input);
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        let payload = &output.emits[0].payload;
        assert_eq!(payload["action_id"], json!(32u64));
        assert_eq!(payload["verdict"], json!("deny"));
        assert_eq!(
            payload["notes"][0],
            json!("transfer amount must be positive")
        );
    }

    #[test]
    fn transfer_rule_denies_when_agents_not_colocated() {
        let mut agents = BTreeMap::new();
        agents.insert(
            "agent-1".to_string(),
            GeoPos {
                x_cm: 0.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
        );
        agents.insert(
            "agent-2".to_string(),
            GeoPos {
                x_cm: 100.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
        );
        let state_bytes = serde_cbor::to_vec(&PositionState { agents }).expect("encode state");
        let action = json!({
            "id": 33u64,
            "action": {
                "type": "TransferResource",
                "data": {
                    "from_agent_id": "agent-1",
                    "to_agent_id": "agent-2",
                    "kind": "electricity",
                    "amount": 1i64
                }
            }
        });
        let input = encode_input(
            M1_TRANSFER_RULE_MODULE_ID,
            0,
            Some(action),
            Some(state_bytes),
            None,
        );

        let output_bytes = build_module_output(&input);
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        let payload = &output.emits[0].payload;
        assert_eq!(payload["action_id"], json!(33u64));
        assert_eq!(payload["verdict"], json!("deny"));
        assert_eq!(
            payload["notes"][0],
            json!("transfer requires co-located agents")
        );
    }

    #[test]
    fn transfer_rule_updates_position_state_on_event() {
        let event = json!({
            "id": 2u64,
            "time": 2u64,
            "caused_by": null,
            "body": {
                "kind": "Domain",
                "payload": {
                    "type": "AgentMoved",
                    "data": {
                        "agent_id": "agent-11",
                        "from": {"x_cm": 0.0, "y_cm": 0.0, "z_cm": 0.0},
                        "to": {"x_cm": 55.0, "y_cm": 66.0, "z_cm": 0.0}
                    }
                }
            }
        });
        let input = encode_input(M1_TRANSFER_RULE_MODULE_ID, 0, None, None, Some(event));

        let output_bytes = build_module_output(&input);
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        let state_bytes = output.new_state.expect("state bytes");
        let state: PositionState = serde_cbor::from_slice(&state_bytes).expect("decode state");
        assert_eq!(
            state.agents.get("agent-11"),
            Some(&GeoPos {
                x_cm: 55.0,
                y_cm: 66.0,
                z_cm: 0.0
            })
        );
    }

    #[test]
    fn invalid_input_falls_back_to_empty_output() {
        let output_bytes = build_module_output(b"invalid-cbor");
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        assert!(output.emits.is_empty());
        assert!(output.effects.is_empty());
        assert!(output.new_state.is_none());
    }
}
