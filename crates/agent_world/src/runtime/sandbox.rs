//! Sandbox execution scaffolding for WASM modules.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use super::modules::ModuleLimits;

/// Effect intent produced by a module call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleEffectIntent {
    pub kind: String,
    pub params: JsonValue,
    pub cap_ref: String,
}

/// Event emitted by a module call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleEmit {
    pub kind: String,
    pub payload: JsonValue,
}

/// Output from a module call executed in a sandbox.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleOutput {
    pub new_state: Option<Vec<u8>>,
    #[serde(default)]
    pub effects: Vec<ModuleEffectIntent>,
    #[serde(default)]
    pub emits: Vec<ModuleEmit>,
    pub output_bytes: u64,
}

/// Request for executing a module in a sandbox.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleCallRequest {
    pub module_id: String,
    pub wasm_hash: String,
    pub trace_id: String,
    pub input: Vec<u8>,
    pub limits: ModuleLimits,
}

/// Error codes for module call failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleCallErrorCode {
    Trap,
    Timeout,
    OutputTooLarge,
    EffectLimitExceeded,
    EmitLimitExceeded,
    CapsDenied,
    PolicyDenied,
    SandboxUnavailable,
    InvalidOutput,
}

/// Failure payload for module calls.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleCallFailure {
    pub module_id: String,
    pub trace_id: String,
    pub code: ModuleCallErrorCode,
    pub detail: String,
}

/// Event emitted when a module call produces an output event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleEmitEvent {
    pub module_id: String,
    pub trace_id: String,
    pub kind: String,
    pub payload: JsonValue,
}

/// Module sandbox trait for execution backends.
pub trait ModuleSandbox {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure>;
}

/// A sandbox stub that always returns a fixed result.
#[derive(Debug, Clone)]
pub struct FixedSandbox {
    result: Result<ModuleOutput, ModuleCallFailure>,
}

impl FixedSandbox {
    pub fn succeed(output: ModuleOutput) -> Self {
        Self { result: Ok(output) }
    }

    pub fn fail(failure: ModuleCallFailure) -> Self {
        Self {
            result: Err(failure),
        }
    }
}

impl ModuleSandbox for FixedSandbox {
    fn call(&mut self, _request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        self.result.clone()
    }
}

/// Configuration for a real WASM executor backend.
#[derive(Debug, Clone, PartialEq)]
pub struct WasmExecutorConfig {
    pub max_fuel: u64,
    pub max_mem_bytes: u64,
    pub max_output_bytes: u64,
    pub max_call_ms: u64,
    pub max_cache_entries: usize,
}

impl Default for WasmExecutorConfig {
    fn default() -> Self {
        Self {
            max_fuel: 10_000_000,
            max_mem_bytes: 64 * 1024 * 1024,
            max_output_bytes: 4 * 1024 * 1024,
            max_call_ms: 2_000,
            max_cache_entries: 32,
        }
    }
}

/// Placeholder WASM executor implementation.
#[derive(Debug, Clone)]
pub struct WasmExecutor {
    config: WasmExecutorConfig,
}

impl WasmExecutor {
    pub fn new(config: WasmExecutorConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &WasmExecutorConfig {
        &self.config
    }
}

impl ModuleSandbox for WasmExecutor {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        Err(ModuleCallFailure {
            module_id: request.module_id.clone(),
            trace_id: request.trace_id.clone(),
            code: ModuleCallErrorCode::SandboxUnavailable,
            detail: "wasm executor backend not configured".to_string(),
        })
    }
}
