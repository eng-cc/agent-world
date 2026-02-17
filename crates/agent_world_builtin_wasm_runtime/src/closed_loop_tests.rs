use super::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

const TRACKING_MODULE_IDS: [&str; 9] = [
    M1_MOVE_RULE_MODULE_ID,
    M1_VISIBILITY_RULE_MODULE_ID,
    M1_TRANSFER_RULE_MODULE_ID,
    M1_SENSOR_MODULE_ID,
    M1_MOBILITY_MODULE_ID,
    M1_MEMORY_MODULE_ID,
    M1_STORAGE_CARGO_MODULE_ID,
    M1_RADIATION_POWER_MODULE_ID,
    M1_STORAGE_POWER_MODULE_ID,
];

#[derive(Debug, Clone, Serialize)]
struct ModuleContextInput {
    module_id: String,
    time: u64,
}

#[derive(Debug, Clone, Serialize)]
struct ModuleCallInput {
    ctx: ModuleContextInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    event: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    action: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Deserialize)]
struct ModuleEmit {
    kind: String,
    payload: Value,
}

#[derive(Debug, Clone, Deserialize)]
struct ModuleOutputView {
    new_state: Option<Vec<u8>>,
    #[serde(default)]
    emits: Vec<ModuleEmit>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
struct GeoPosView {
    x_cm: f64,
    y_cm: f64,
    z_cm: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct PositionStateView {
    #[serde(default)]
    agents: BTreeMap<String, GeoPosView>,
}

#[derive(Debug, Clone, Deserialize)]
struct MemoryModuleEntryView {
    time: u64,
    kind: String,
    #[serde(default)]
    agent_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct MemoryModuleStateView {
    #[serde(default)]
    entries: Vec<MemoryModuleEntryView>,
}

#[derive(Debug, Clone, Deserialize)]
struct CargoLedgerStateView {
    #[serde(default)]
    consumed_interface_items: BTreeMap<String, u64>,
    #[serde(default)]
    agent_expansion_levels: BTreeMap<String, u16>,
    #[serde(default)]
    reject_count: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct AgentPowerStateView {
    pos: GeoPosView,
    level: i64,
}

#[derive(Debug, Clone, Deserialize)]
struct PowerStateView {
    #[serde(default)]
    agents: BTreeMap<String, AgentPowerStateView>,
}

#[derive(Default)]
struct ScenarioHarness {
    module_states: BTreeMap<String, Vec<u8>>,
}

impl ScenarioHarness {
    fn invoke(
        &mut self,
        module_id: &str,
        time: u64,
        action: Option<Value>,
        event: Option<Value>,
    ) -> ModuleOutputView {
        let action = action
            .as_ref()
            .map(|value| serde_cbor::to_vec(value).expect("encode action"));
        let event = event
            .as_ref()
            .map(|value| serde_cbor::to_vec(value).expect("encode event"));
        let input = ModuleCallInput {
            ctx: ModuleContextInput {
                module_id: module_id.to_string(),
                time,
            },
            event,
            action,
            state: self.module_states.get(module_id).cloned(),
        };
        let input_bytes = serde_cbor::to_vec(&input).expect("encode input");
        let output_bytes = build_module_output(&input_bytes);
        let output: ModuleOutputView =
            serde_cbor::from_slice(&output_bytes).expect("decode output");
        if let Some(state) = output.new_state.clone() {
            self.module_states.insert(module_id.to_string(), state);
        }
        output
    }

    fn broadcast_event(&mut self, time: u64, event: Value, module_ids: &[&str]) {
        for module_id in module_ids {
            self.invoke(module_id, time, None, Some(event.clone()));
        }
    }

    fn decode_state<T: DeserializeOwned>(&self, module_id: &str) -> T {
        let state = self
            .module_states
            .get(module_id)
            .unwrap_or_else(|| panic!("missing state for module {module_id}"));
        serde_cbor::from_slice(state).expect("decode state")
    }
}

fn domain_event(id: u64, time: u64, event_type: &str, data: Value) -> Value {
    json!({
        "id": id,
        "time": time,
        "caused_by": Value::Null,
        "body": {
            "kind": "Domain",
            "payload": {
                "type": event_type,
                "data": data,
            }
        }
    })
}

fn action_envelope(id: u64, action_type: &str, data: Value) -> Value {
    json!({
        "id": id,
        "action": {
            "type": action_type,
            "data": data,
        }
    })
}

fn assert_rule_decision(output: &ModuleOutputView, action_id: u64, verdict: &str) {
    assert_eq!(output.emits.len(), 1);
    assert_eq!(output.emits[0].kind, RULE_DECISION_EMIT_KIND);
    assert_eq!(output.emits[0].payload["action_id"], json!(action_id));
    assert_eq!(output.emits[0].payload["verdict"], json!(verdict));
}

#[test]
fn wasm_builtin_modules_support_closed_loop_world_scenario() {
    let mut harness = ScenarioHarness::default();

    let agent_1_pos = json!({"x_cm": 0.0, "y_cm": 0.0, "z_cm": 0.0});
    let agent_2_pos = json!({"x_cm": 200_000.0, "y_cm": 0.0, "z_cm": 0.0});

    harness.broadcast_event(
        1,
        domain_event(
            1,
            1,
            "AgentRegistered",
            json!({"agent_id": "agent-1", "pos": agent_1_pos}),
        ),
        &TRACKING_MODULE_IDS,
    );
    harness.broadcast_event(
        2,
        domain_event(
            2,
            2,
            "AgentRegistered",
            json!({"agent_id": "agent-2", "pos": agent_2_pos}),
        ),
        &TRACKING_MODULE_IDS,
    );

    let move_action = action_envelope(
        101,
        "MoveAgent",
        json!({"agent_id": "agent-1", "to": agent_2_pos}),
    );
    let move_rule_output =
        harness.invoke(M1_MOVE_RULE_MODULE_ID, 3, Some(move_action.clone()), None);
    assert_rule_decision(&move_rule_output, 101, "allow");
    let mobility_output = harness.invoke(M1_MOBILITY_MODULE_ID, 3, Some(move_action.clone()), None);
    assert_rule_decision(&mobility_output, 101, "allow");
    let storage_power_move_output =
        harness.invoke(M1_STORAGE_POWER_MODULE_ID, 3, Some(move_action), None);
    assert_rule_decision(&storage_power_move_output, 101, "allow");

    let storage_power_after_move: PowerStateView = harness.decode_state(M1_STORAGE_POWER_MODULE_ID);
    assert_eq!(storage_power_after_move.agents["agent-1"].level, 1);

    harness.broadcast_event(
        3,
        domain_event(
            3,
            3,
            "AgentMoved",
            json!({
                "agent_id": "agent-1",
                "from": {"x_cm": 0.0, "y_cm": 0.0, "z_cm": 0.0},
                "to": {"x_cm": 200_000.0, "y_cm": 0.0, "z_cm": 0.0}
            }),
        ),
        &TRACKING_MODULE_IDS,
    );

    let observation_action =
        action_envelope(102, "QueryObservation", json!({"agent_id": "agent-1"}));
    let visibility_output = harness.invoke(
        M1_VISIBILITY_RULE_MODULE_ID,
        4,
        Some(observation_action.clone()),
        None,
    );
    assert_rule_decision(&visibility_output, 102, "modify");
    assert_eq!(
        visibility_output.emits[0].payload["override_action"]["type"],
        json!("EmitObservation")
    );
    let sensor_output = harness.invoke(M1_SENSOR_MODULE_ID, 4, Some(observation_action), None);
    assert_rule_decision(&sensor_output, 102, "modify");

    harness.broadcast_event(
        4,
        domain_event(
            4,
            4,
            "Observation",
            visibility_output.emits[0].payload["override_action"]["data"].clone(),
        ),
        &[M1_MEMORY_MODULE_ID],
    );

    let transfer_action = action_envelope(
        103,
        "TransferResource",
        json!({
            "from_agent_id": "agent-1",
            "to_agent_id": "agent-2",
            "kind": "electricity",
            "amount": 2
        }),
    );
    let transfer_output =
        harness.invoke(M1_TRANSFER_RULE_MODULE_ID, 5, Some(transfer_action), None);
    assert_rule_decision(&transfer_output, 103, "modify");
    assert_eq!(
        transfer_output.emits[0].payload["override_action"]["type"],
        json!("EmitResourceTransfer")
    );

    harness.broadcast_event(
        5,
        domain_event(
            5,
            5,
            "ResourceTransferred",
            transfer_output.emits[0].payload["override_action"]["data"].clone(),
        ),
        &[M1_MEMORY_MODULE_ID],
    );

    let body_action = action_envelope(
        104,
        "BodyAction",
        json!({
            "agent_id": "agent-1",
            "kind": "bootstrap",
            "payload": {
                "mass_kg": 120,
                "radius_cm": 80,
                "thrust_limit": 200,
                "cross_section_cm2": 4000
            }
        }),
    );
    let body_output = harness.invoke(M1_BODY_MODULE_ID, 6, Some(body_action), None);
    assert_rule_decision(&body_output, 104, "modify");
    assert_eq!(
        body_output.emits[0].payload["override_action"]["type"],
        json!("EmitBodyAttributes")
    );

    harness.broadcast_event(
        6,
        domain_event(
            6,
            6,
            "BodyAttributesUpdated",
            body_output.emits[0].payload["override_action"]["data"].clone(),
        ),
        &[M1_MEMORY_MODULE_ID],
    );

    let body_expand_event = domain_event(
        7,
        7,
        "BodyInterfaceExpanded",
        json!({
            "agent_id": "agent-1",
            "slot_capacity": 2,
            "expansion_level": 1,
            "consumed_item_id": "iface-kit-1",
            "new_slot_id": "slot-1",
            "slot_type": "Tool"
        }),
    );
    harness.broadcast_event(
        7,
        body_expand_event,
        &[M1_STORAGE_CARGO_MODULE_ID, M1_MEMORY_MODULE_ID],
    );

    let radiation_harvest_output = harness.invoke(
        M1_RADIATION_POWER_MODULE_ID,
        8,
        Some(action_envelope(105, "PowerTick", json!({}))),
        None,
    );
    assert_eq!(radiation_harvest_output.emits.len(), 1);
    assert_eq!(
        radiation_harvest_output.emits[0].kind,
        "power.radiation_harvest"
    );
    assert_eq!(
        radiation_harvest_output.emits[0].payload["agents"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );

    let storage_power_tick_output = harness.invoke(
        M1_STORAGE_POWER_MODULE_ID,
        8,
        Some(action_envelope(106, "PowerTick", json!({}))),
        None,
    );
    assert!(storage_power_tick_output.emits.is_empty());

    let visibility_state: PositionStateView = harness.decode_state(M1_VISIBILITY_RULE_MODULE_ID);
    assert_eq!(
        visibility_state.agents.get("agent-1"),
        Some(&GeoPosView {
            x_cm: 200_000.0,
            y_cm: 0.0,
            z_cm: 0.0,
        })
    );

    let cargo_state: CargoLedgerStateView = harness.decode_state(M1_STORAGE_CARGO_MODULE_ID);
    assert_eq!(
        cargo_state.consumed_interface_items.get("iface-kit-1"),
        Some(&1)
    );
    assert_eq!(cargo_state.agent_expansion_levels.get("agent-1"), Some(&1));
    assert_eq!(cargo_state.reject_count, 0);

    let storage_power_state: PowerStateView = harness.decode_state(M1_STORAGE_POWER_MODULE_ID);
    assert_eq!(storage_power_state.agents["agent-1"].level, 2);
    assert_eq!(storage_power_state.agents["agent-2"].level, 8);
    assert_eq!(
        storage_power_state.agents["agent-1"].pos,
        GeoPosView {
            x_cm: 200_000.0,
            y_cm: 0.0,
            z_cm: 0.0,
        }
    );

    let memory_state: MemoryModuleStateView = harness.decode_state(M1_MEMORY_MODULE_ID);
    let memory_kinds: Vec<&str> = memory_state
        .entries
        .iter()
        .map(|entry| entry.kind.as_str())
        .collect();
    assert_eq!(
        memory_kinds,
        vec![
            "domain.agent_registered",
            "domain.agent_registered",
            "domain.agent_moved",
            "domain.observation",
            "domain.resource_transferred",
            "domain.body_attributes_updated",
            "domain.body_interface_expanded",
        ]
    );
    assert_eq!(memory_state.entries.last().map(|entry| entry.time), Some(7));
    assert_eq!(
        memory_state
            .entries
            .last()
            .and_then(|entry| entry.agent_id.as_deref()),
        Some("agent-1")
    );
}
