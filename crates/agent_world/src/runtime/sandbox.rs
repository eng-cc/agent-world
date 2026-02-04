//! Sandbox execution scaffolding for WASM modules.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
#[cfg(feature = "wasmtime")]
use std::collections::{BTreeMap, VecDeque};
#[cfg(feature = "wasmtime")]
use std::sync::{Arc, Mutex};

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
    #[serde(default)]
    pub wasm_bytes: Vec<u8>,
}

/// Origin metadata for a module call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleCallOrigin {
    pub kind: String,
    pub id: String,
}

/// Execution context passed into a module call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleContext {
    pub v: String,
    pub module_id: String,
    pub trace_id: String,
    pub time: u64,
    pub origin: ModuleCallOrigin,
    pub limits: ModuleLimits,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_config_hash: Option<String>,
}

/// Canonical input envelope passed into module calls.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleCallInput {
    pub ctx: ModuleContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<Vec<u8>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Vec<u8>>,
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
    pub engine: WasmEngineKind,
    pub max_fuel: u64,
    pub max_mem_bytes: u64,
    pub max_output_bytes: u64,
    pub max_call_ms: u64,
    pub max_cache_entries: usize,
}

/// Selected WASM engine backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmEngineKind {
    Wasmtime,
}

impl Default for WasmExecutorConfig {
    fn default() -> Self {
        Self {
            engine: WasmEngineKind::Wasmtime,
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
    #[cfg(feature = "wasmtime")]
    engine: wasmtime::Engine,
    #[cfg(feature = "wasmtime")]
    compiled_cache: Arc<Mutex<CompiledModuleCache>>,
}

impl WasmExecutor {
    pub fn new(config: WasmExecutorConfig) -> Self {
        #[cfg(feature = "wasmtime")]
        {
            let mut engine_config = wasmtime::Config::new();
            engine_config.consume_fuel(true);
            engine_config.epoch_interruption(true);
            engine_config.wasm_multi_value(true);
            engine_config.wasm_reference_types(true);
            engine_config.wasm_threads(false);
            engine_config.cranelift_nan_canonicalization(true);
            engine_config.debug_info(false);
            let engine = wasmtime::Engine::new(&engine_config)
                .expect("failed to initialize wasmtime engine");
            let compiled_cache =
                Arc::new(Mutex::new(CompiledModuleCache::new(config.max_cache_entries)));
            Self {
                config,
                engine,
                compiled_cache,
            }
        }

        #[cfg(not(feature = "wasmtime"))]
        {
            Self { config }
        }
    }

    pub fn config(&self) -> &WasmExecutorConfig {
        &self.config
    }

    #[cfg(feature = "wasmtime")]
    pub fn engine(&self) -> &wasmtime::Engine {
        &self.engine
    }

    #[cfg(feature = "wasmtime")]
    pub(crate) fn compiled_cache_len(&self) -> usize {
        self.compiled_cache
            .lock()
            .expect("compiled cache poisoned")
            .len()
    }

    #[cfg(feature = "wasmtime")]
    pub(crate) fn compile_module_cached(
        &self,
        wasm_hash: &str,
        wasm_bytes: &[u8],
    ) -> Result<Arc<wasmtime::Module>, ModuleCallFailure> {
        let mut cache = self
            .compiled_cache
            .lock()
            .expect("compiled cache poisoned");
        if let Some(module) = cache.get(wasm_hash) {
            return Ok(module);
        }
        drop(cache);

        let module = wasmtime::Module::new(&self.engine, wasm_bytes).map_err(|err| {
            self.failure(
                &ModuleCallRequest {
                    module_id: wasm_hash.to_string(),
                    wasm_hash: wasm_hash.to_string(),
                    trace_id: "compile".to_string(),
                    input: Vec::new(),
                    limits: ModuleLimits::default(),
                    wasm_bytes: Vec::new(),
                },
                ModuleCallErrorCode::Trap,
                format!("compile failed: {err}"),
            )
        })?;
        let module = Arc::new(module);
        let mut cache = self
            .compiled_cache
            .lock()
            .expect("compiled cache poisoned");
        cache.insert(wasm_hash.to_string(), module.clone());
        Ok(module)
    }

    fn failure(
        &self,
        request: &ModuleCallRequest,
        code: ModuleCallErrorCode,
        detail: impl Into<String>,
    ) -> ModuleCallFailure {
        ModuleCallFailure {
            module_id: request.module_id.clone(),
            trace_id: request.trace_id.clone(),
            code,
            detail: detail.into(),
        }
    }

    #[cfg_attr(not(feature = "wasmtime"), allow(dead_code))]
    fn validate_request_limits(
        &self,
        request: &ModuleCallRequest,
    ) -> Result<(), ModuleCallFailure> {
        let limits = &request.limits;
        if limits.max_output_bytes > self.config.max_output_bytes {
            return Err(self.failure(
                request,
                ModuleCallErrorCode::OutputTooLarge,
                "requested output limit exceeds executor max",
            ));
        }
        if limits.max_gas > self.config.max_fuel {
            return Err(self.failure(
                request,
                ModuleCallErrorCode::Timeout,
                "requested fuel limit exceeds executor max",
            ));
        }
        if limits.max_mem_bytes > self.config.max_mem_bytes {
            return Err(self.failure(
                request,
                ModuleCallErrorCode::Trap,
                "requested memory limit exceeds executor max",
            ));
        }
        Ok(())
    }

    #[allow(dead_code)]
    fn validate_output_limits(
        &self,
        request: &ModuleCallRequest,
        output: &ModuleOutput,
    ) -> Result<(), ModuleCallFailure> {
        let limits = &request.limits;
        if output.effects.len() as u32 > limits.max_effects {
            return Err(self.failure(
                request,
                ModuleCallErrorCode::EffectLimitExceeded,
                "effects exceeded",
            ));
        }
        if output.emits.len() as u32 > limits.max_emits {
            return Err(self.failure(
                request,
                ModuleCallErrorCode::EmitLimitExceeded,
                "emits exceeded",
            ));
        }
        if output.output_bytes > limits.max_output_bytes {
            return Err(self.failure(
                request,
                ModuleCallErrorCode::OutputTooLarge,
                "output bytes exceeded",
            ));
        }
        Ok(())
    }

    #[cfg(feature = "wasmtime")]
    fn map_wasmtime_error(
        &self,
        request: &ModuleCallRequest,
        err: wasmtime::Error,
    ) -> ModuleCallFailure {
        if let Some(trap) = err.downcast_ref::<wasmtime::Trap>() {
            let code = match trap {
                wasmtime::Trap::OutOfFuel => ModuleCallErrorCode::Timeout,
                _ => ModuleCallErrorCode::Trap,
            };
            return self.failure(request, code, trap.to_string());
        }
        self.failure(request, ModuleCallErrorCode::Trap, err.to_string())
    }
}

impl ModuleSandbox for WasmExecutor {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        #[cfg(feature = "wasmtime")]
        {
            if let Err(failure) = self.validate_request_limits(request) {
                return Err(failure);
            }

            if request.wasm_bytes.is_empty() {
                return Err(self.failure(
                    request,
                    ModuleCallErrorCode::Trap,
                    "missing wasm bytes",
                ));
            }

            let module =
                self.compile_module_cached(&request.wasm_hash, &request.wasm_bytes)?;
            let start = std::time::Instant::now();
            let mut store = wasmtime::Store::new(&self.engine, ());
            if request.limits.max_gas > 0 {
                store
                    .add_fuel(request.limits.max_gas)
                    .map_err(|err| self.failure(request, ModuleCallErrorCode::Trap, err.to_string()))?;
            }
            let linker = wasmtime::Linker::new(&self.engine);
            let instance = linker
                .instantiate(&mut store, &module)
                .map_err(|err| self.map_wasmtime_error(request, err))?;
            let memory = instance
                .get_memory(&mut store, "memory")
                .ok_or_else(|| {
                    self.failure(
                        request,
                        ModuleCallErrorCode::InvalidOutput,
                        "missing memory export",
                    )
                })?;
            let alloc = instance
                .get_typed_func::<i32, i32>(&mut store, "alloc")
                .map_err(|err| {
                    self.failure(
                        request,
                        ModuleCallErrorCode::InvalidOutput,
                        format!("missing alloc export: {err}"),
                    )
                })?;
            let call = instance
                .get_typed_func::<(i32, i32), (i32, i32)>(&mut store, "call")
                .map_err(|err| {
                    self.failure(
                        request,
                        ModuleCallErrorCode::InvalidOutput,
                        format!("missing call export: {err}"),
                    )
                })?;

            let input_len = i32::try_from(request.input.len()).map_err(|_| {
                self.failure(
                    request,
                    ModuleCallErrorCode::Trap,
                    "input too large for wasm32",
                )
            })?;
            let input_ptr = alloc
                .call(&mut store, input_len)
                .map_err(|err| self.map_wasmtime_error(request, err))?;
            if input_len > 0 {
                memory
                    .write(&mut store, input_ptr as usize, &request.input)
                    .map_err(|err| self.failure(request, ModuleCallErrorCode::Trap, err.to_string()))?;
            }
            let (output_ptr, output_len) = call
                .call(&mut store, (input_ptr, input_len))
                .map_err(|err| self.map_wasmtime_error(request, err))?;
            let memory_size = memory.data_size(&store) as u64;
            if memory_size > request.limits.max_mem_bytes {
                return Err(self.failure(
                    request,
                    ModuleCallErrorCode::Trap,
                    "memory limit exceeded",
                ));
            }
            let output_len = usize::try_from(output_len).map_err(|_| {
                self.failure(
                    request,
                    ModuleCallErrorCode::Trap,
                    "negative output length",
                )
            })?;
            if output_len as u64 > request.limits.max_output_bytes {
                return Err(self.failure(
                    request,
                    ModuleCallErrorCode::OutputTooLarge,
                    "output bytes exceeded",
                ));
            }
            let mut output_buf = vec![0u8; output_len];
            if output_len > 0 {
                memory
                    .read(&mut store, output_ptr as usize, &mut output_buf)
                    .map_err(|err| self.failure(request, ModuleCallErrorCode::Trap, err.to_string()))?;
            }
            if start.elapsed().as_millis() as u64 > self.config.max_call_ms {
                return Err(self.failure(
                    request,
                    ModuleCallErrorCode::Timeout,
                    "execution exceeded max_call_ms",
                ));
            }
            let mut output: ModuleOutput = serde_cbor::from_slice(&output_buf).map_err(|err| {
                self.failure(
                    request,
                    ModuleCallErrorCode::InvalidOutput,
                    format!("output CBOR decode failed: {err}"),
                )
            })?;
            output.output_bytes = output_buf.len() as u64;
            self.validate_output_limits(request, &output)?;
            return Ok(output);
        }

        let detail = if cfg!(feature = "wasmtime") {
            "wasmtime backend not implemented"
        } else {
            "wasmtime feature not enabled"
        };

        Err(ModuleCallFailure {
            module_id: request.module_id.clone(),
            trace_id: request.trace_id.clone(),
            code: ModuleCallErrorCode::SandboxUnavailable,
            detail: detail.to_string(),
        })
    }
}

#[cfg(feature = "wasmtime")]
#[derive(Debug)]
struct CompiledModuleCache {
    max_entries: usize,
    cache: BTreeMap<String, Arc<wasmtime::Module>>,
    lru: VecDeque<String>,
}

#[cfg(feature = "wasmtime")]
impl CompiledModuleCache {
    fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            cache: BTreeMap::new(),
            lru: VecDeque::new(),
        }
    }

    fn len(&self) -> usize {
        self.cache.len()
    }

    fn get(&mut self, wasm_hash: &str) -> Option<Arc<wasmtime::Module>> {
        let module = self.cache.get(wasm_hash)?.clone();
        self.touch(wasm_hash);
        Some(module)
    }

    fn insert(&mut self, wasm_hash: String, module: Arc<wasmtime::Module>) {
        self.cache.insert(wasm_hash.clone(), module);
        self.touch(&wasm_hash);
        self.prune();
    }

    fn touch(&mut self, wasm_hash: &str) {
        self.lru.retain(|entry| entry != wasm_hash);
        self.lru.push_back(wasm_hash.to_string());
    }

    fn prune(&mut self) {
        if self.max_entries == 0 {
            self.cache.clear();
            self.lru.clear();
            return;
        }
        while self.cache.len() > self.max_entries {
            if let Some(evicted) = self.lru.pop_front() {
                self.cache.remove(&evicted);
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_executor_rejects_output_limit_overflow() {
        let executor = WasmExecutor::new(WasmExecutorConfig::default());
        let request = ModuleCallRequest {
            module_id: "m.test".to_string(),
            wasm_hash: "hash".to_string(),
            trace_id: "trace-1".to_string(),
            input: vec![],
            limits: ModuleLimits {
                max_mem_bytes: executor.config().max_mem_bytes,
                max_gas: executor.config().max_fuel,
                max_call_rate: 0,
                max_output_bytes: 4,
                max_effects: 0,
                max_emits: 0,
            },
            wasm_bytes: Vec::new(),
        };
        let output = ModuleOutput {
            new_state: None,
            effects: Vec::new(),
            emits: Vec::new(),
            output_bytes: 8,
        };

        let err = executor
            .validate_output_limits(&request, &output)
            .unwrap_err();
        assert_eq!(err.code, ModuleCallErrorCode::OutputTooLarge);
    }

    #[test]
    fn wasm_executor_rejects_fuel_limit_as_timeout() {
        let executor = WasmExecutor::new(WasmExecutorConfig {
            max_fuel: 10,
            ..WasmExecutorConfig::default()
        });
        let request = ModuleCallRequest {
            module_id: "m.test".to_string(),
            wasm_hash: "hash".to_string(),
            trace_id: "trace-2".to_string(),
            input: vec![],
            limits: ModuleLimits {
                max_mem_bytes: executor.config().max_mem_bytes,
                max_gas: 11,
                max_call_rate: 0,
                max_output_bytes: executor.config().max_output_bytes,
                max_effects: 0,
                max_emits: 0,
            },
            wasm_bytes: Vec::new(),
        };

        let err = executor.validate_request_limits(&request).unwrap_err();
        assert_eq!(err.code, ModuleCallErrorCode::Timeout);
    }

    #[cfg(feature = "wasmtime")]
    #[test]
    fn wasm_executor_compiled_cache_evicts_old_entries() {
        let executor = WasmExecutor::new(WasmExecutorConfig {
            max_cache_entries: 1,
            ..WasmExecutorConfig::default()
        });
        let wasm_a = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        let wasm_b = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

        executor
            .compile_module_cached("hash-a", &wasm_a)
            .unwrap();
        assert_eq!(executor.compiled_cache_len(), 1);

        executor
            .compile_module_cached("hash-b", &wasm_b)
            .unwrap();
        assert_eq!(executor.compiled_cache_len(), 1);
    }
}
