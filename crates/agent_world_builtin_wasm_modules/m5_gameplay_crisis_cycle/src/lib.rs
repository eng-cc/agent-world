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
use std::collections::BTreeMap;

const MODULE_ID: &str = "m5.gameplay.crisis.cycle";
const DIRECTIVE_EMIT_KIND: &str = "gameplay.lifecycle.directives";
const CRISIS_AUTO_INTERVAL_TICKS: u64 = 8;
const CRISIS_DEFAULT_DURATION_TICKS: u64 = 6;
const CRISIS_TIMEOUT_PENALTY_PER_SEVERITY: i64 = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CrisisSnapshot {
    crisis_id: String,
    severity: u32,
    expires_at: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CrisisModuleState {
    #[serde(default)]
    active_crises: BTreeMap<String, CrisisSnapshot>,
}

#[derive(Debug, Clone, Deserialize)]
struct DomainEventEnvelope {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    data: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
struct CrisisSpawnedData {
    crisis_id: String,
    severity: u32,
    expires_at: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct CrisisResolvedData {
    crisis_id: String,
}

#[derive(Debug, Clone, Deserialize)]
struct CrisisTimedOutData {
    crisis_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct DirectiveEnvelope {
    directives: Vec<LifecycleDirective>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum LifecycleDirective {
    CrisisSpawn {
        crisis_id: String,
        kind: String,
        severity: u32,
        expires_at: u64,
    },
    CrisisTimeout {
        crisis_id: String,
        penalty_impact: i64,
    },
}

fn decode_state(input: &ModuleCallInput) -> CrisisModuleState {
    input
        .state
        .as_deref()
        .and_then(|bytes| serde_cbor::from_slice::<CrisisModuleState>(bytes).ok())
        .unwrap_or_default()
}

fn encode_state(state: &CrisisModuleState) -> Option<Vec<u8>> {
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

fn apply_domain_event(state: &mut CrisisModuleState, event: DomainEventEnvelope) {
    match event.event_type.as_str() {
        "CrisisSpawned" => {
            if let Ok(data) = serde_json::from_value::<CrisisSpawnedData>(event.data) {
                state.active_crises.insert(
                    data.crisis_id.clone(),
                    CrisisSnapshot {
                        crisis_id: data.crisis_id,
                        severity: data.severity,
                        expires_at: data.expires_at,
                    },
                );
            }
        }
        "CrisisResolved" => {
            if let Ok(data) = serde_json::from_value::<CrisisResolvedData>(event.data) {
                state.active_crises.remove(&data.crisis_id);
            }
        }
        "CrisisTimedOut" => {
            if let Ok(data) = serde_json::from_value::<CrisisTimedOutData>(event.data) {
                state.active_crises.remove(&data.crisis_id);
            }
        }
        _ => {}
    }
}

fn run_tick(state: &mut CrisisModuleState, now: u64) -> Vec<LifecycleDirective> {
    let mut directives = Vec::new();

    if state.active_crises.is_empty() && now > 0 && now % CRISIS_AUTO_INTERVAL_TICKS == 0 {
        let sequence = now / CRISIS_AUTO_INTERVAL_TICKS;
        let severity = ((sequence % 3) + 1) as u32;
        let kind = match severity {
            1 => "supply_shock",
            2 => "solar_storm",
            _ => "network_outage",
        }
        .to_string();
        let crisis_id = format!("crisis.auto.{now}");
        let expires_at = now
            .saturating_add(CRISIS_DEFAULT_DURATION_TICKS)
            .saturating_add(u64::from(severity));
        state.active_crises.insert(
            crisis_id.clone(),
            CrisisSnapshot {
                crisis_id: crisis_id.clone(),
                severity,
                expires_at,
            },
        );
        directives.push(LifecycleDirective::CrisisSpawn {
            crisis_id,
            kind,
            severity,
            expires_at,
        });
    }

    let mut due_ids = state
        .active_crises
        .iter()
        .filter_map(|(crisis_id, crisis)| (crisis.expires_at <= now).then_some(crisis_id.clone()))
        .collect::<Vec<_>>();
    due_ids.sort();
    for crisis_id in due_ids {
        let Some(crisis) = state.active_crises.remove(&crisis_id) else {
            continue;
        };
        let severity = crisis.severity.max(1);
        let penalty_impact =
            -i64::from(severity).saturating_mul(CRISIS_TIMEOUT_PENALTY_PER_SEVERITY);
        directives.push(LifecycleDirective::CrisisTimeout {
            crisis_id,
            penalty_impact,
        });
    }

    directives
}

fn build_output(state: &CrisisModuleState, directives: Vec<LifecycleDirective>) -> ModuleOutput {
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
        run_tick(&mut state, input.ctx.time)
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
