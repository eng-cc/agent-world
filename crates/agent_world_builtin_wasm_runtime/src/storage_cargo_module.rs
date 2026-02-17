use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use super::{empty_output, encode_output, ModuleCallInput, ModuleOutput};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(super) struct CargoLedgerState {
    #[serde(default)]
    pub(super) consumed_interface_items: BTreeMap<String, u64>,
    #[serde(default)]
    pub(super) agent_expansion_levels: BTreeMap<String, u16>,
    #[serde(default)]
    pub(super) reject_count: u64,
}

fn decode_storage_cargo_state(state_bytes: Option<&[u8]>) -> CargoLedgerState {
    let Some(state_bytes) = state_bytes else {
        return CargoLedgerState::default();
    };
    if state_bytes.is_empty() {
        return CargoLedgerState::default();
    }
    serde_cbor::from_slice(state_bytes).unwrap_or_default()
}

fn encode_storage_cargo_state(state: &CargoLedgerState) -> Option<Vec<u8>> {
    serde_cbor::to_vec(state).ok()
}

pub(super) fn build_storage_cargo_module_output(input: &ModuleCallInput) -> Vec<u8> {
    let Some(event_bytes) = input.event.as_deref() else {
        return encode_output(empty_output());
    };
    let event: Value = match serde_cbor::from_slice(event_bytes) {
        Ok(value) => value,
        Err(_) => return encode_output(empty_output()),
    };
    if event
        .get("body")
        .and_then(|body| body.get("kind"))
        .and_then(Value::as_str)
        != Some("Domain")
    {
        return encode_output(empty_output());
    }
    let Some(payload) = event.get("body").and_then(|body| body.get("payload")) else {
        return encode_output(empty_output());
    };
    let Some(event_type) = payload.get("type").and_then(Value::as_str) else {
        return encode_output(empty_output());
    };
    let Some(data) = payload.get("data") else {
        return encode_output(empty_output());
    };

    let mut state = decode_storage_cargo_state(input.state.as_deref());
    let changed = match event_type {
        "BodyInterfaceExpanded" => {
            let Some(agent_id) = data.get("agent_id").and_then(Value::as_str) else {
                return encode_output(empty_output());
            };
            let Some(consumed_item_id) = data.get("consumed_item_id").and_then(Value::as_str)
            else {
                return encode_output(empty_output());
            };
            let Some(expansion_level_raw) = data.get("expansion_level").and_then(Value::as_u64)
            else {
                return encode_output(empty_output());
            };
            let Ok(expansion_level) = u16::try_from(expansion_level_raw) else {
                return encode_output(empty_output());
            };

            state
                .agent_expansion_levels
                .insert(agent_id.to_string(), expansion_level);
            let consumed = state
                .consumed_interface_items
                .entry(consumed_item_id.to_string())
                .or_insert(0);
            *consumed = consumed.saturating_add(1);
            true
        }
        "BodyInterfaceExpandRejected" => {
            state.reject_count = state.reject_count.saturating_add(1);
            true
        }
        _ => false,
    };

    if !changed {
        return encode_output(empty_output());
    }

    encode_output(ModuleOutput {
        new_state: encode_storage_cargo_state(&state),
        effects: Vec::new(),
        emits: Vec::new(),
        output_bytes: 0,
    })
}

#[cfg(test)]
mod tests {
    use serde::Serialize;
    use serde_json::{json, Value};
    use std::collections::BTreeMap;

    use super::*;
    use crate::{build_module_output, M1_STORAGE_CARGO_MODULE_ID};

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

    fn encode_input(module_id: &str, time: u64, state: Option<Vec<u8>>, event: Value) -> Vec<u8> {
        let input = ModuleCallInputTest {
            ctx: ModuleContextTest {
                module_id: module_id.to_string(),
                time,
            },
            event: Some(serde_cbor::to_vec(&event).expect("encode event")),
            action: None,
            state,
        };
        serde_cbor::to_vec(&input).expect("encode input")
    }

    fn decode_output(output_bytes: &[u8]) -> ModuleOutput {
        serde_cbor::from_slice(output_bytes).expect("decode output")
    }

    #[test]
    fn storage_cargo_module_tracks_expand_events() {
        let expanded_event = json!({
            "id": 81u64,
            "time": 500u64,
            "caused_by": null,
            "body": {
                "kind": "Domain",
                "payload": {
                    "type": "BodyInterfaceExpanded",
                    "data": {
                        "agent_id": "agent-1",
                        "slot_capacity": 2u16,
                        "expansion_level": 1u16,
                        "consumed_item_id": "iface-kit-1",
                        "new_slot_id": "slot-2",
                        "slot_type": "Tool"
                    }
                }
            }
        });
        let input = encode_input(M1_STORAGE_CARGO_MODULE_ID, 0, None, expanded_event);
        let first_output = decode_output(&build_module_output(&input));
        let first_state_bytes = first_output.new_state.expect("state bytes");

        let first_state: CargoLedgerState =
            serde_cbor::from_slice(&first_state_bytes).expect("decode state");
        assert_eq!(
            first_state.consumed_interface_items.get("iface-kit-1"),
            Some(&1u64)
        );
        assert_eq!(
            first_state.agent_expansion_levels.get("agent-1"),
            Some(&1u16)
        );

        let expanded_event_2 = json!({
            "id": 82u64,
            "time": 501u64,
            "caused_by": null,
            "body": {
                "kind": "Domain",
                "payload": {
                    "type": "BodyInterfaceExpanded",
                    "data": {
                        "agent_id": "agent-1",
                        "slot_capacity": 3u16,
                        "expansion_level": 2u16,
                        "consumed_item_id": "iface-kit-1",
                        "new_slot_id": "slot-3",
                        "slot_type": "Tool"
                    }
                }
            }
        });
        let input_2 = encode_input(
            M1_STORAGE_CARGO_MODULE_ID,
            0,
            Some(first_state_bytes),
            expanded_event_2,
        );
        let second_output = decode_output(&build_module_output(&input_2));
        let second_state_bytes = second_output.new_state.expect("state bytes");
        let second_state: CargoLedgerState =
            serde_cbor::from_slice(&second_state_bytes).expect("decode state");

        assert_eq!(
            second_state.consumed_interface_items.get("iface-kit-1"),
            Some(&2u64)
        );
        assert_eq!(
            second_state.agent_expansion_levels.get("agent-1"),
            Some(&2u16)
        );
    }

    #[test]
    fn storage_cargo_module_counts_rejections_with_saturation() {
        let rejected_event = json!({
            "id": 83u64,
            "time": 600u64,
            "caused_by": null,
            "body": {
                "kind": "Domain",
                "payload": {
                    "type": "BodyInterfaceExpandRejected",
                    "data": {
                        "agent_id": "agent-1",
                        "consumed_item_id": "iface-kit-missing",
                        "reason": "item_missing"
                    }
                }
            }
        });
        let input = encode_input(M1_STORAGE_CARGO_MODULE_ID, 0, None, rejected_event.clone());
        let output = decode_output(&build_module_output(&input));
        let state_bytes = output.new_state.expect("state bytes");
        let state: CargoLedgerState = serde_cbor::from_slice(&state_bytes).expect("decode state");
        assert_eq!(state.reject_count, 1);

        let seeded_state = CargoLedgerState {
            consumed_interface_items: BTreeMap::new(),
            agent_expansion_levels: BTreeMap::new(),
            reject_count: u64::MAX,
        };
        let seeded_state_bytes = serde_cbor::to_vec(&seeded_state).expect("encode state");
        let input_saturated = encode_input(
            M1_STORAGE_CARGO_MODULE_ID,
            0,
            Some(seeded_state_bytes),
            rejected_event,
        );
        let saturated_output = decode_output(&build_module_output(&input_saturated));
        let saturated_state_bytes = saturated_output.new_state.expect("state bytes");
        let saturated_state: CargoLedgerState =
            serde_cbor::from_slice(&saturated_state_bytes).expect("decode state");
        assert_eq!(saturated_state.reject_count, u64::MAX);
    }

    #[test]
    fn storage_cargo_module_ignores_non_target_domain_events() {
        let unrelated_event = json!({
            "id": 84u64,
            "time": 700u64,
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

        let input = encode_input(M1_STORAGE_CARGO_MODULE_ID, 0, None, unrelated_event);
        let output = decode_output(&build_module_output(&input));

        assert!(output.new_state.is_none());
        assert!(output.effects.is_empty());
        assert!(output.emits.is_empty());
    }
}
