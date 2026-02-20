#![allow(improper_ctypes_definitions)]

use agent_world_wasm_sdk::{
    export_wasm_module,
    wire::{
        decode_input, empty_output, encode_output, ModuleCallInput, ModuleEmit, ModuleOutput,
        ModuleTickLifecycleDirective,
    },
    LifecycleStage, WasmModuleLifecycle,
};
use serde::{Deserialize, Serialize};

const MODULE_ID: &str = "m5.gameplay.economic.overlay";
const DIRECTIVE_EMIT_KIND: &str = "gameplay.lifecycle.directives";
const OVERLAY_OPERATOR_ID: &str = "system.economic.overlay";
const RESILIENCE_TRACK: &str = "resilience";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetaGrantDirective {
    operator_agent_id: String,
    target_agent_id: String,
    track: String,
    points: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    achievement_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct EconomicOverlayState {
    #[serde(default)]
    pending_meta_grants: Vec<MetaGrantDirective>,
}

#[derive(Debug, Clone, Deserialize)]
struct DomainEventEnvelope {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    data: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
struct CrisisResolvedData {
    resolver_agent_id: String,
    success: bool,
    impact: i64,
}

#[derive(Debug, Clone, Serialize)]
struct DirectiveEnvelope {
    directives: Vec<LifecycleDirective>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum LifecycleDirective {
    MetaGrant {
        operator_agent_id: String,
        target_agent_id: String,
        track: String,
        points: i64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        achievement_id: Option<String>,
    },
}

fn decode_state(input: &ModuleCallInput) -> EconomicOverlayState {
    input
        .state
        .as_deref()
        .and_then(|bytes| serde_cbor::from_slice::<EconomicOverlayState>(bytes).ok())
        .unwrap_or_default()
}

fn encode_state(state: &EconomicOverlayState) -> Option<Vec<u8>> {
    serde_cbor::to_vec(state).ok()
}

fn parse_domain_event(input: &ModuleCallInput) -> Option<DomainEventEnvelope> {
    let event_bytes = input.event.as_deref()?;
    let event_value = serde_cbor::from_slice::<serde_json::Value>(event_bytes).ok()?;
    if event_value.get("body")?.get("kind")?.as_str()? != "Domain" {
        return None;
    }
    let payload = event_value.get("body")?.get("payload")?.clone();
    serde_json::from_value(payload).ok()
}

fn apply_domain_event(state: &mut EconomicOverlayState, event: DomainEventEnvelope) {
    if event.event_type != "CrisisResolved" {
        return;
    }
    let Ok(data) = serde_json::from_value::<CrisisResolvedData>(event.data) else {
        return;
    };
    if !data.success || data.impact <= 0 {
        return;
    }
    let points = (data.impact / 4).max(1);
    state.pending_meta_grants.push(MetaGrantDirective {
        operator_agent_id: OVERLAY_OPERATOR_ID.to_string(),
        target_agent_id: data.resolver_agent_id,
        track: RESILIENCE_TRACK.to_string(),
        points,
        achievement_id: None,
    });
}

fn run_tick(state: &mut EconomicOverlayState) -> Vec<LifecycleDirective> {
    if state.pending_meta_grants.is_empty() {
        return Vec::new();
    }
    let pending = std::mem::take(&mut state.pending_meta_grants);
    pending
        .into_iter()
        .map(|grant| LifecycleDirective::MetaGrant {
            operator_agent_id: grant.operator_agent_id,
            target_agent_id: grant.target_agent_id,
            track: grant.track,
            points: grant.points,
            achievement_id: grant.achievement_id,
        })
        .collect()
}

fn build_output(state: &EconomicOverlayState, directives: Vec<LifecycleDirective>) -> ModuleOutput {
    let emits = if directives.is_empty() {
        Vec::new()
    } else {
        let payload = serde_json::to_value(DirectiveEnvelope { directives })
            .unwrap_or_else(|_| serde_json::json!({ "directives": [] }));
        vec![ModuleEmit {
            kind: DIRECTIVE_EMIT_KIND.to_string(),
            payload,
        }]
    };

    ModuleOutput {
        new_state: encode_state(state),
        effects: Vec::new(),
        emits,
        tick_lifecycle: Some(ModuleTickLifecycleDirective::WakeAfterTicks { ticks: 1 }),
        output_bytes: 2048,
    }
}

fn reduce_output(input: &ModuleCallInput) -> ModuleOutput {
    let mut state = decode_state(input);
    if let Some(event) = parse_domain_event(input) {
        apply_domain_event(&mut state, event);
    }

    let directives = if input.ctx.stage.as_deref() == Some("tick") {
        run_tick(&mut state)
    } else {
        Vec::new()
    };

    build_output(&state, directives)
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
    let output = reduce_output(&decoded);
    write_bytes_to_memory(&encode_output(output))
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
