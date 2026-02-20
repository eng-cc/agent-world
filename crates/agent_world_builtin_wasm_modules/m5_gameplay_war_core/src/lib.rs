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

const MODULE_ID: &str = "m5.gameplay.war.core";
const DIRECTIVE_EMIT_KIND: &str = "gameplay.lifecycle.directives";
const WAR_SCORE_PER_MEMBER: i64 = 10;
const BASE_WAR_DURATION_TICKS: u64 = 6;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct AllianceSnapshot {
    #[serde(default)]
    members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct WarSnapshot {
    war_id: String,
    aggressor_alliance_id: String,
    defender_alliance_id: String,
    intensity: u32,
    declared_at: u64,
    max_duration_ticks: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct WarModuleState {
    #[serde(default)]
    alliances: BTreeMap<String, AllianceSnapshot>,
    #[serde(default)]
    active_wars: BTreeMap<String, WarSnapshot>,
}

#[derive(Debug, Clone, Deserialize)]
struct DomainEventEnvelope {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    data: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
struct AllianceFormedData {
    alliance_id: String,
    #[serde(default)]
    members: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct WarDeclaredData {
    war_id: String,
    aggressor_alliance_id: String,
    defender_alliance_id: String,
    intensity: u32,
}

#[derive(Debug, Clone, Deserialize)]
struct WarConcludedData {
    war_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct DirectiveEnvelope {
    directives: Vec<LifecycleDirective>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum LifecycleDirective {
    WarConclude {
        war_id: String,
        winner_alliance_id: String,
        aggressor_score: i64,
        defender_score: i64,
        summary: String,
    },
}

fn decode_state(input: &ModuleCallInput) -> WarModuleState {
    input
        .state
        .as_deref()
        .and_then(|bytes| serde_cbor::from_slice::<WarModuleState>(bytes).ok())
        .unwrap_or_default()
}

fn encode_state(state: &WarModuleState) -> Option<Vec<u8>> {
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

fn apply_domain_event(state: &mut WarModuleState, event: DomainEventEnvelope, now: u64) {
    match event.event_type.as_str() {
        "AllianceFormed" => {
            if let Ok(data) = serde_json::from_value::<AllianceFormedData>(event.data) {
                state.alliances.insert(
                    data.alliance_id,
                    AllianceSnapshot {
                        members: data.members,
                    },
                );
            }
        }
        "WarDeclared" => {
            if let Ok(data) = serde_json::from_value::<WarDeclaredData>(event.data) {
                let max_duration_ticks = BASE_WAR_DURATION_TICKS
                    .saturating_add(u64::from(data.intensity).saturating_mul(2));
                state.active_wars.insert(
                    data.war_id.clone(),
                    WarSnapshot {
                        war_id: data.war_id,
                        aggressor_alliance_id: data.aggressor_alliance_id,
                        defender_alliance_id: data.defender_alliance_id,
                        intensity: data.intensity,
                        declared_at: now,
                        max_duration_ticks,
                    },
                );
            }
        }
        "WarConcluded" => {
            if let Ok(data) = serde_json::from_value::<WarConcludedData>(event.data) {
                state.active_wars.remove(&data.war_id);
            }
        }
        _ => {}
    }
}

fn run_tick(state: &mut WarModuleState, now: u64) -> Vec<LifecycleDirective> {
    let mut due_ids = state
        .active_wars
        .iter()
        .filter_map(|(war_id, war)| {
            let due_at = war
                .declared_at
                .saturating_add(war.max_duration_ticks.max(1));
            (now >= due_at).then_some(war_id.clone())
        })
        .collect::<Vec<_>>();
    due_ids.sort();

    let mut directives = Vec::new();
    for war_id in due_ids {
        let Some(war) = state.active_wars.remove(&war_id) else {
            continue;
        };
        let aggressor_members = state
            .alliances
            .get(&war.aggressor_alliance_id)
            .map(|alliance| alliance.members.len() as i64)
            .unwrap_or(0);
        let defender_members = state
            .alliances
            .get(&war.defender_alliance_id)
            .map(|alliance| alliance.members.len() as i64)
            .unwrap_or(0);
        let aggressor_score = aggressor_members
            .saturating_mul(WAR_SCORE_PER_MEMBER)
            .saturating_add(i64::from(war.intensity));
        let defender_score = defender_members.saturating_mul(WAR_SCORE_PER_MEMBER);
        let winner_alliance_id = if aggressor_score >= defender_score {
            war.aggressor_alliance_id
        } else {
            war.defender_alliance_id
        };
        let summary = format!(
            "module settlement: aggressor_score={} defender_score={}",
            aggressor_score, defender_score
        );
        directives.push(LifecycleDirective::WarConclude {
            war_id,
            winner_alliance_id,
            aggressor_score,
            defender_score,
            summary,
        });
    }
    directives
}

fn build_output(state: &WarModuleState, directives: Vec<LifecycleDirective>) -> ModuleOutput {
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
    let now = input.ctx.time;
    if let Some(event) = parse_domain_event(input) {
        apply_domain_event(&mut state, event, now);
    }

    let directives = if input.ctx.stage.as_deref() == Some("tick") {
        run_tick(&mut state, now)
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
