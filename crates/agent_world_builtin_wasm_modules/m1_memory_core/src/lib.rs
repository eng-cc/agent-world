#![allow(improper_ctypes_definitions)]

use agent_world_wasm_sdk::{export_wasm_module, LifecycleStage, WasmModuleLifecycle};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const MODULE_ID: &str = "m1.memory.core";

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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
struct MemoryModuleState {
    #[serde(default)]
    entries: Vec<MemoryModuleEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct MemoryModuleEntry {
    time: u64,
    kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    agent_id: Option<String>,
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

fn build_memory_module_output(input: &ModuleCallInput) -> Vec<u8> {
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
    if state.entries.len() > 256 {
        let overflow = state.entries.len() - 256;
        state.entries.drain(0..overflow);
    }

    encode_output(ModuleOutput {
        new_state: encode_memory_state(&state),
        effects: Vec::new(),
        emits: Vec::new(),
        output_bytes: 0,
    })
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
    let ptr = agent_world_wasm_sdk::default_alloc(len);
    if ptr <= 0 {
        return (0, 0);
    }
    // SAFETY: alloc returns a writable wasm linear memory region with at least len bytes.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, len as usize);
    }
    (ptr, len)
}

fn reduce_impl(input_ptr: i32, input_len: i32) -> (i32, i32) {
    let input = read_input_bytes(input_ptr, input_len);
    let Some(mut decoded) = decode_input(&input) else {
        return write_bytes_to_memory(&encode_output(empty_output()));
    };
    decoded.ctx.module_id = MODULE_ID.to_string();
    let output = build_memory_module_output(&decoded);
    write_bytes_to_memory(&output)
}

#[derive(Default)]
struct BuiltinWasmModule;

impl WasmModuleLifecycle for BuiltinWasmModule {
    fn module_id(&self) -> &'static str {
        MODULE_ID
    }

    fn alloc(&mut self, len: i32) -> i32 {
        agent_world_wasm_sdk::default_alloc(len)
    }

    fn on_init(&mut self, _stage: LifecycleStage) {}

    fn on_teardown(&mut self, _stage: LifecycleStage) {}

    fn on_reduce(&mut self, input_ptr: i32, input_len: i32) -> (i32, i32) {
        reduce_impl(input_ptr, input_len)
    }

    fn on_call(&mut self, input_ptr: i32, input_len: i32) -> (i32, i32) {
        reduce_impl(input_ptr, input_len)
    }
}

export_wasm_module!(BuiltinWasmModule);
