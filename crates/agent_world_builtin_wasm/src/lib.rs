use serde::{Deserialize, Serialize};
#[cfg(test)]
use serde_json::json;
use serde_json::Value;
#[cfg(test)]
use std::collections::BTreeMap;

mod economy_modules;
mod memory_module;
mod power_modules;
mod rule_body_modules;
mod storage_cargo_module;

use economy_modules::build_economy_module_output;
use memory_module::build_memory_module_output;
use power_modules::{build_radiation_power_module_output, build_storage_power_module_output};
#[cfg(test)]
use rule_body_modules::PositionState;
use rule_body_modules::{
    build_body_module_output, build_move_rule_output, build_transfer_rule_output,
    build_visibility_rule_output,
};
use storage_cargo_module::build_storage_cargo_module_output;

pub const M1_MOVE_RULE_MODULE_ID: &str = "m1.rule.move";
pub const M1_VISIBILITY_RULE_MODULE_ID: &str = "m1.rule.visibility";
pub const M1_TRANSFER_RULE_MODULE_ID: &str = "m1.rule.transfer";
pub const M1_BODY_MODULE_ID: &str = "m1.body.core";
pub const M1_SENSOR_MODULE_ID: &str = "m1.sensor.basic";
pub const M1_MOBILITY_MODULE_ID: &str = "m1.mobility.basic";
pub const M1_MEMORY_MODULE_ID: &str = "m1.memory.core";
pub const M1_STORAGE_CARGO_MODULE_ID: &str = "m1.storage.cargo";
pub const M1_RADIATION_POWER_MODULE_ID: &str = "m1.power.radiation_harvest";
pub const M1_STORAGE_POWER_MODULE_ID: &str = "m1.power.storage";
pub const M1_AGENT_DEFAULT_MODULE_VERSION: &str = "0.1.0";
pub const M1_POWER_MODULE_VERSION: &str = "0.1.0";
pub const M4_ECONOMY_MODULE_VERSION: &str = "0.1.0";
pub const M4_FACTORY_MINER_MODULE_ID: &str = "m4.factory.miner.mk1";
pub const M4_FACTORY_SMELTER_MODULE_ID: &str = "m4.factory.smelter.mk1";
pub const M4_FACTORY_ASSEMBLER_MODULE_ID: &str = "m4.factory.assembler.mk1";
pub const M4_RECIPE_SMELT_IRON_MODULE_ID: &str = "m4.recipe.smelter.iron_ingot";
pub const M4_RECIPE_SMELT_COPPER_WIRE_MODULE_ID: &str = "m4.recipe.smelter.copper_wire";
pub const M4_RECIPE_ASSEMBLE_GEAR_MODULE_ID: &str = "m4.recipe.assembler.gear";
pub const M4_RECIPE_ASSEMBLE_CONTROL_CHIP_MODULE_ID: &str = "m4.recipe.assembler.control_chip";
pub const M4_RECIPE_ASSEMBLE_MOTOR_MODULE_ID: &str = "m4.recipe.assembler.motor_mk1";
pub const M4_RECIPE_ASSEMBLE_DRONE_MODULE_ID: &str = "m4.recipe.assembler.logistics_drone";
pub const M4_PRODUCT_IRON_INGOT_MODULE_ID: &str = "m4.product.material.iron_ingot";
pub const M4_PRODUCT_CONTROL_CHIP_MODULE_ID: &str = "m4.product.component.control_chip";
pub const M4_PRODUCT_MOTOR_MODULE_ID: &str = "m4.product.component.motor_mk1";
pub const M4_PRODUCT_LOGISTICS_DRONE_MODULE_ID: &str = "m4.product.finished.logistics_drone";
pub const M1_BODY_ACTION_COST_ELECTRICITY: i64 = 10;
pub const M1_MEMORY_MAX_ENTRIES: usize = 256;
pub const M1_POWER_STORAGE_CAPACITY: i64 = 12;
pub const M1_POWER_STORAGE_INITIAL_LEVEL: i64 = 6;
pub const M1_POWER_STORAGE_MOVE_COST_PER_KM: i64 = 3;
pub const M1_POWER_HARVEST_BASE_PER_TICK: i64 = 1;
pub const M1_POWER_HARVEST_DISTANCE_STEP_CM: i64 = 800_000;
pub const M1_POWER_HARVEST_DISTANCE_BONUS_CAP: i64 = 1;
const DEFAULT_MOVE_COST_PER_KM_ELECTRICITY: i64 = 1;
const CM_PER_KM: i64 = 100_000;
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

fn build_module_output_for_decoded_input(module_id: &str, input: &ModuleCallInput) -> Vec<u8> {
    if let Some(output) = build_economy_module_output(input) {
        return output;
    }
    match module_id {
        M1_MOVE_RULE_MODULE_ID => build_move_rule_output(input),
        M1_VISIBILITY_RULE_MODULE_ID => build_visibility_rule_output(input),
        M1_TRANSFER_RULE_MODULE_ID => build_transfer_rule_output(input),
        M1_BODY_MODULE_ID => build_body_module_output(input),
        M1_SENSOR_MODULE_ID => build_visibility_rule_output(input),
        M1_MOBILITY_MODULE_ID => build_move_rule_output(input),
        M1_MEMORY_MODULE_ID => build_memory_module_output(input),
        M1_STORAGE_CARGO_MODULE_ID => build_storage_cargo_module_output(input),
        M1_RADIATION_POWER_MODULE_ID => build_radiation_power_module_output(input),
        M1_STORAGE_POWER_MODULE_ID => build_storage_power_module_output(input),
        _ => encode_output(empty_output()),
    }
}

#[cfg(test)]
fn build_module_output(input_bytes: &[u8]) -> Vec<u8> {
    let Some(input) = decode_input(input_bytes) else {
        return encode_output(empty_output());
    };
    build_module_output_for_decoded_input(input.ctx.module_id.as_str(), &input)
}

fn build_module_output_for_module(module_id: &str, input_bytes: &[u8]) -> Vec<u8> {
    let Some(mut input) = decode_input(input_bytes) else {
        return encode_output(empty_output());
    };
    input.ctx.module_id = module_id.to_string();
    build_module_output_for_decoded_input(module_id, &input)
}

fn read_input_bytes(input_ptr: i32, input_len: i32) -> Vec<u8> {
    if input_ptr > 0 && input_len > 0 {
        let ptr = input_ptr as *const u8;
        let len = input_len as usize;
        // SAFETY: host guarantees valid wasm linear memory pointer/len for the call.
        return unsafe { std::slice::from_raw_parts(ptr, len).to_vec() };
    }
    Vec::new()
}

fn write_bytes_to_memory(bytes: &[u8]) -> (i32, i32) {
    let len = i32::try_from(bytes.len()).unwrap_or(0);
    if len <= 0 {
        return (0, 0);
    }
    let ptr = builtin_alloc(len);
    if ptr <= 0 {
        return (0, 0);
    }
    // SAFETY: builtin_alloc returns a writable wasm linear memory region with at least len bytes.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, len as usize);
    }
    (ptr, len)
}

pub fn builtin_alloc(len: i32) -> i32 {
    if len <= 0 {
        return 0;
    }
    let capacity = len as usize;
    let mut buf = Vec::<u8>::with_capacity(capacity);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr as i32
}

pub fn reduce_for_module(module_id: &str, input_ptr: i32, input_len: i32) -> (i32, i32) {
    let input = read_input_bytes(input_ptr, input_len);
    let output = build_module_output_for_module(module_id, &input);
    write_bytes_to_memory(&output)
}

pub fn call_for_module(module_id: &str, input_ptr: i32, input_len: i32) -> (i32, i32) {
    reduce_for_module(module_id, input_ptr, input_len)
}

#[cfg(test)]
mod closed_loop_tests;

#[cfg(test)]
mod tests {
    use super::memory_module::{MemoryModuleEntry, MemoryModuleState};
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
    fn body_module_emits_emit_body_attributes_override_and_cost() {
        let action = json!({
            "id": 41u64,
            "action": {
                "type": "BodyAction",
                "data": {
                    "agent_id": "agent-1",
                    "kind": "boot",
                    "payload": {
                        "mass_kg": 120u64,
                        "radius_cm": 80u64,
                        "thrust_limit": 200u64,
                        "cross_section_cm2": 4000u64
                    }
                }
            }
        });
        let input = encode_input(M1_BODY_MODULE_ID, 0, Some(action), None, None);

        let output_bytes = build_module_output(&input);
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        assert_eq!(output.emits.len(), 1);
        let payload = &output.emits[0].payload;
        assert_eq!(payload["action_id"], json!(41u64));
        assert_eq!(payload["verdict"], json!("modify"));
        assert_eq!(
            payload["override_action"]["type"],
            json!("EmitBodyAttributes")
        );
        assert_eq!(
            payload["override_action"]["data"]["agent_id"],
            json!("agent-1")
        );
        assert_eq!(
            payload["override_action"]["data"]["reason"],
            json!("body.boot")
        );
        assert_eq!(
            payload["override_action"]["data"]["view"]["mass_kg"],
            json!(120u64)
        );
        assert_eq!(
            payload["cost"]["entries"]["electricity"],
            json!(-M1_BODY_ACTION_COST_ELECTRICITY)
        );
    }

    #[test]
    fn body_module_denies_when_payload_decode_failed() {
        let action = json!({
            "id": 42u64,
            "action": {
                "type": "BodyAction",
                "data": {
                    "agent_id": "agent-2",
                    "kind": "boot",
                    "payload": {
                        "mass_kg": 120u64,
                        "radius_cm": 80u64,
                        "thrust_limit": "bad",
                        "cross_section_cm2": 4000u64
                    }
                }
            }
        });
        let input = encode_input(M1_BODY_MODULE_ID, 0, Some(action), None, None);

        let output_bytes = build_module_output(&input);
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        assert_eq!(output.emits.len(), 1);
        let payload = &output.emits[0].payload;
        assert_eq!(payload["action_id"], json!(42u64));
        assert_eq!(payload["verdict"], json!("deny"));
        let note = payload["notes"][0].as_str().expect("note str");
        assert!(note.starts_with("body action payload decode failed:"));
    }

    #[test]
    fn sensor_module_reuses_visibility_behavior() {
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
                x_cm: 5.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
        );
        let state_bytes = serde_cbor::to_vec(&PositionState { agents }).expect("encode state");
        let action = json!({
            "id": 51u64,
            "action": {
                "type": "QueryObservation",
                "data": {"agent_id":"agent-1"}
            }
        });
        let input = encode_input(
            M1_SENSOR_MODULE_ID,
            999,
            Some(action),
            Some(state_bytes),
            None,
        );

        let output_bytes = build_module_output(&input);
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        assert_eq!(output.emits.len(), 1);
        let payload = &output.emits[0].payload;
        assert_eq!(payload["action_id"], json!(51u64));
        assert_eq!(payload["verdict"], json!("modify"));
        assert_eq!(payload["override_action"]["type"], json!("EmitObservation"));
        assert_eq!(
            payload["override_action"]["data"]["observation"]["visible_agents"][0]["agent_id"],
            json!("agent-2")
        );
    }

    #[test]
    fn mobility_module_reuses_move_rule_behavior() {
        let action = json!({
            "id": 61u64,
            "action": {
                "type": "MoveAgent",
                "data": {
                    "agent_id":"agent-1",
                    "to":{"x_cm": 100.0, "y_cm": 200.0, "z_cm": 0.0}
                }
            }
        });
        let input = encode_input(M1_MOBILITY_MODULE_ID, 1000, Some(action), None, None);

        let output_bytes = build_module_output(&input);
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        assert_eq!(output.emits.len(), 1);
        let payload = &output.emits[0].payload;
        assert_eq!(payload["action_id"], json!(61u64));
        assert_eq!(payload["verdict"], json!("allow"));
    }

    #[test]
    fn mobility_module_denies_when_move_target_equals_current_position() {
        let mut agents = BTreeMap::new();
        agents.insert(
            "agent-1".to_string(),
            GeoPos {
                x_cm: 100.0,
                y_cm: 200.0,
                z_cm: 0.0,
            },
        );
        let state_bytes = serde_cbor::to_vec(&PositionState { agents }).expect("encode state");
        let action = json!({
            "id": 62u64,
            "action": {
                "type": "MoveAgent",
                "data": {
                    "agent_id":"agent-1",
                    "to":{"x_cm": 100.0, "y_cm": 200.0, "z_cm": 0.0}
                }
            }
        });
        let input = encode_input(
            M1_MOBILITY_MODULE_ID,
            1001,
            Some(action),
            Some(state_bytes),
            None,
        );

        let output_bytes = build_module_output(&input);
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        assert_eq!(output.emits.len(), 1);
        let payload = &output.emits[0].payload;
        assert_eq!(payload["action_id"], json!(62u64));
        assert_eq!(payload["verdict"], json!("deny"));
        assert_eq!(
            payload["notes"][0],
            json!("move target equals current position")
        );
    }

    #[test]
    fn memory_module_records_domain_event_and_agent_id() {
        let event = json!({
            "id": 71u64,
            "time": 456u64,
            "caused_by": null,
            "body": {
                "kind": "Domain",
                "payload": {
                    "type": "AgentRegistered",
                    "data": {
                        "agent_id": "agent-memory",
                        "pos": {"x_cm": 1.0, "y_cm": 2.0, "z_cm": 0.0}
                    }
                }
            }
        });
        let input = encode_input(M1_MEMORY_MODULE_ID, 0, None, None, Some(event));

        let output_bytes = build_module_output(&input);
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        let state_bytes = output.new_state.expect("state bytes");
        let state: MemoryModuleState = serde_cbor::from_slice(&state_bytes).expect("decode state");
        assert_eq!(state.entries.len(), 1);
        assert_eq!(state.entries[0].time, 456u64);
        assert_eq!(state.entries[0].kind, "domain.agent_registered");
        assert_eq!(state.entries[0].agent_id.as_deref(), Some("agent-memory"));
    }

    #[test]
    fn memory_module_tracks_observation_and_action_rejected_labels() {
        let observation_event = json!({
            "id": 72u64,
            "time": 100u64,
            "caused_by": null,
            "body": {
                "kind": "Domain",
                "payload": {
                    "type": "Observation",
                    "data": {
                        "observation": {
                            "time": 100u64,
                            "agent_id": "agent-obs",
                            "pos": {"x_cm": 0.0, "y_cm": 0.0, "z_cm": 0.0},
                            "visibility_range_cm": 1000i64,
                            "visible_agents": []
                        }
                    }
                }
            }
        });
        let input_observation =
            encode_input(M1_MEMORY_MODULE_ID, 0, None, None, Some(observation_event));
        let output_observation = build_module_output(&input_observation);
        let output: ModuleOutput =
            serde_cbor::from_slice(&output_observation).expect("decode output");
        let state_bytes = output.new_state.expect("state bytes");

        let rejected_event = json!({
            "id": 73u64,
            "time": 101u64,
            "caused_by": null,
            "body": {
                "kind": "Domain",
                "payload": {
                    "type": "ActionRejected",
                    "data": {"action_id": 9u64, "reason": "invalid"}
                }
            }
        });
        let input_rejected = encode_input(
            M1_MEMORY_MODULE_ID,
            0,
            None,
            Some(state_bytes),
            Some(rejected_event),
        );
        let output_rejected = build_module_output(&input_rejected);
        let output: ModuleOutput = serde_cbor::from_slice(&output_rejected).expect("decode output");
        let state_bytes = output.new_state.expect("state bytes");
        let state: MemoryModuleState = serde_cbor::from_slice(&state_bytes).expect("decode state");

        assert_eq!(state.entries.len(), 2);
        assert_eq!(state.entries[0].kind, "domain.observation");
        assert_eq!(state.entries[0].agent_id.as_deref(), Some("agent-obs"));
        assert_eq!(state.entries[1].kind, "domain.action_rejected");
        assert_eq!(state.entries[1].agent_id, None);
    }

    #[test]
    fn memory_module_drops_old_entries_when_capacity_exceeded() {
        let mut seed_entries = Vec::new();
        for idx in 0..M1_MEMORY_MAX_ENTRIES {
            seed_entries.push(MemoryModuleEntry {
                time: idx as u64,
                kind: format!("seed.{idx}"),
                agent_id: None,
            });
        }
        let seed_state = MemoryModuleState {
            entries: seed_entries,
        };
        let state_bytes = serde_cbor::to_vec(&seed_state).expect("encode state");
        let event = json!({
            "id": 74u64,
            "time": 999u64,
            "caused_by": null,
            "body": {
                "kind": "Domain",
                "payload": {
                    "type": "BodyAttributesUpdated",
                    "data": {
                        "agent_id": "agent-77",
                        "view": {
                            "mass_kg": 1u64,
                            "radius_cm": 1u64,
                            "thrust_limit": 1u64,
                            "cross_section_cm2": 1u64
                        },
                        "reason": "test"
                    }
                }
            }
        });
        let input = encode_input(M1_MEMORY_MODULE_ID, 0, None, Some(state_bytes), Some(event));

        let output_bytes = build_module_output(&input);
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        let state_bytes = output.new_state.expect("state bytes");
        let state: MemoryModuleState = serde_cbor::from_slice(&state_bytes).expect("decode state");

        assert_eq!(state.entries.len(), M1_MEMORY_MAX_ENTRIES);
        assert_eq!(
            state.entries.first().map(|entry| entry.kind.as_str()),
            Some("seed.1")
        );
        assert_eq!(
            state.entries.last().map(|entry| entry.kind.as_str()),
            Some("domain.body_attributes_updated")
        );
        assert_eq!(
            state
                .entries
                .last()
                .and_then(|entry| entry.agent_id.as_deref()),
            Some("agent-77")
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
