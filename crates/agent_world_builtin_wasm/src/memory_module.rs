use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{empty_output, encode_output, ModuleCallInput, ModuleOutput, M1_MEMORY_MAX_ENTRIES};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub(super) struct MemoryModuleState {
    #[serde(default)]
    pub(super) entries: Vec<MemoryModuleEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub(super) struct MemoryModuleEntry {
    pub(super) time: u64,
    pub(super) kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) agent_id: Option<String>,
}

fn decode_memory_state(state_bytes: Option<&[u8]>) -> MemoryModuleState {
    let Some(state_bytes) = state_bytes else {
        return MemoryModuleState::default();
    };
    if state_bytes.is_empty() {
        return MemoryModuleState::default();
    }
    serde_cbor::from_slice(state_bytes).unwrap_or_default()
}

fn encode_memory_state(state: &MemoryModuleState) -> Option<Vec<u8>> {
    serde_cbor::to_vec(state).ok()
}

fn memory_domain_label(event_type: &str, data: &Value) -> Option<(&'static str, Option<String>)> {
    match event_type {
        "AgentRegistered" => Some((
            "domain.agent_registered",
            data.get("agent_id")
                .and_then(Value::as_str)
                .map(str::to_string),
        )),
        "AgentMoved" => Some((
            "domain.agent_moved",
            data.get("agent_id")
                .and_then(Value::as_str)
                .map(str::to_string),
        )),
        "ActionRejected" => Some(("domain.action_rejected", None)),
        "Observation" => Some((
            "domain.observation",
            data.get("observation")
                .and_then(|observation| observation.get("agent_id"))
                .and_then(Value::as_str)
                .map(str::to_string),
        )),
        "BodyAttributesUpdated" => Some((
            "domain.body_attributes_updated",
            data.get("agent_id")
                .and_then(Value::as_str)
                .map(str::to_string),
        )),
        "BodyAttributesRejected" => Some((
            "domain.body_attributes_rejected",
            data.get("agent_id")
                .and_then(Value::as_str)
                .map(str::to_string),
        )),
        "BodyInterfaceExpanded" => Some((
            "domain.body_interface_expanded",
            data.get("agent_id")
                .and_then(Value::as_str)
                .map(str::to_string),
        )),
        "BodyInterfaceExpandRejected" => Some((
            "domain.body_interface_expand_rejected",
            data.get("agent_id")
                .and_then(Value::as_str)
                .map(str::to_string),
        )),
        "ResourceTransferred" => Some((
            "domain.resource_transferred",
            data.get("from_agent_id")
                .and_then(Value::as_str)
                .map(str::to_string),
        )),
        _ => None,
    }
}

pub(super) fn build_memory_module_output(input: &ModuleCallInput) -> Vec<u8> {
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
    let Some(time) = event.get("time").and_then(Value::as_u64) else {
        return encode_output(empty_output());
    };
    let Some(payload) = event.get("body").and_then(|body| body.get("payload")) else {
        return encode_output(empty_output());
    };
    let Some(event_type) = payload.get("type").and_then(Value::as_str) else {
        return encode_output(empty_output());
    };
    let Some(data) = payload.get("data") else {
        return encode_output(empty_output());
    };
    let Some((kind, agent_id)) = memory_domain_label(event_type, data) else {
        return encode_output(empty_output());
    };

    let mut state = decode_memory_state(input.state.as_deref());
    state.entries.push(MemoryModuleEntry {
        time,
        kind: kind.to_string(),
        agent_id,
    });
    if state.entries.len() > M1_MEMORY_MAX_ENTRIES {
        let overflow = state.entries.len() - M1_MEMORY_MAX_ENTRIES;
        state.entries.drain(0..overflow);
    }

    encode_output(ModuleOutput {
        new_state: encode_memory_state(&state),
        effects: Vec::new(),
        emits: Vec::new(),
        output_bytes: 0,
    })
}
