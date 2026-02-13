//! Built-in module compatibility helpers for wasm cutover.

use std::collections::BTreeMap;

use super::sandbox::{
    ModuleCallErrorCode, ModuleCallFailure, ModuleCallRequest, ModuleOutput, ModuleSandbox,
};

pub const M1_MOVE_RULE_MODULE_ID: &str = "m1.rule.move";
pub const M1_VISIBILITY_RULE_MODULE_ID: &str = "m1.rule.visibility";
pub const M1_TRANSFER_RULE_MODULE_ID: &str = "m1.rule.transfer";
pub const M1_BODY_MODULE_ID: &str = "m1.body.core";
pub const M1_BODY_ACTION_COST_ELECTRICITY: i64 = 10;
pub const M1_SENSOR_MODULE_ID: &str = "m1.sensor.basic";
pub const M1_MOBILITY_MODULE_ID: &str = "m1.mobility.basic";
pub const M1_MEMORY_MODULE_ID: &str = "m1.memory.core";
pub const M1_STORAGE_CARGO_MODULE_ID: &str = "m1.storage.cargo";
pub const M1_AGENT_DEFAULT_MODULE_VERSION: &str = "0.1.0";
pub const M1_MEMORY_MAX_ENTRIES: usize = 256;
pub const M1_RADIATION_POWER_MODULE_ID: &str = "m1.power.radiation_harvest";
pub const M1_STORAGE_POWER_MODULE_ID: &str = "m1.power.storage";
pub const M1_POWER_MODULE_VERSION: &str = "0.1.0";
pub const M1_POWER_STORAGE_CAPACITY: i64 = 12;
pub const M1_POWER_STORAGE_INITIAL_LEVEL: i64 = 6;
pub const M1_POWER_STORAGE_MOVE_COST_PER_KM: i64 = 3;
pub const M1_POWER_HARVEST_BASE_PER_TICK: i64 = 1;
pub const M1_POWER_HARVEST_DISTANCE_STEP_CM: i64 = 800_000;
pub const M1_POWER_HARVEST_DISTANCE_BONUS_CAP: i64 = 1;

pub trait BuiltinModule {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure>;
}

pub struct BuiltinModuleSandbox {
    builtins: BTreeMap<String, Box<dyn BuiltinModule>>,
    fallback: Option<Box<dyn ModuleSandbox>>,
}

impl BuiltinModuleSandbox {
    pub fn with_preferred_fallback(fallback: Box<dyn ModuleSandbox>) -> Self {
        Self {
            builtins: BTreeMap::new(),
            fallback: Some(fallback),
        }
    }

    pub fn register_builtin(
        mut self,
        module_id: impl Into<String>,
        module: impl BuiltinModule + 'static,
    ) -> Self {
        self.builtins.insert(module_id.into(), Box::new(module));
        self
    }
}

impl ModuleSandbox for BuiltinModuleSandbox {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        if let Some(fallback) = self.fallback.as_mut() {
            match fallback.call(request) {
                Ok(output) => return Ok(output),
                Err(fallback_failure) => {
                    if let Some(module) = self.builtins.get_mut(&request.module_id) {
                        return module.call(request);
                    }
                    return Err(fallback_failure);
                }
            }
        }

        if let Some(module) = self.builtins.get_mut(&request.module_id) {
            return module.call(request);
        }
        Err(ModuleCallFailure {
            module_id: request.module_id.clone(),
            trace_id: request.trace_id.clone(),
            code: ModuleCallErrorCode::SandboxUnavailable,
            detail: "builtin module not found".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::modules::ModuleLimits;
    use super::super::sandbox::{FixedSandbox, ModuleCallFailure, ModuleOutput};
    use super::*;

    struct DummyBuiltinModule {
        output: ModuleOutput,
    }

    impl DummyBuiltinModule {
        fn new() -> Self {
            Self {
                output: ModuleOutput {
                    new_state: Some(vec![1, 2, 3]),
                    effects: Vec::new(),
                    emits: Vec::new(),
                    output_bytes: 3,
                },
            }
        }
    }

    impl BuiltinModule for DummyBuiltinModule {
        fn call(
            &mut self,
            _request: &ModuleCallRequest,
        ) -> Result<ModuleOutput, ModuleCallFailure> {
            Ok(self.output.clone())
        }
    }

    fn request(module_id: &str) -> ModuleCallRequest {
        ModuleCallRequest {
            module_id: module_id.to_string(),
            wasm_hash: "hash".to_string(),
            trace_id: "trace".to_string(),
            entrypoint: "reduce".to_string(),
            input: Vec::new(),
            limits: ModuleLimits::default(),
            wasm_bytes: Vec::new(),
        }
    }

    #[test]
    fn preferred_fallback_returns_fallback_success_without_builtin() {
        let fallback_output = ModuleOutput {
            new_state: Some(vec![9]),
            effects: Vec::new(),
            emits: Vec::new(),
            output_bytes: 1,
        };
        let mut sandbox = BuiltinModuleSandbox::with_preferred_fallback(Box::new(
            FixedSandbox::succeed(fallback_output.clone()),
        ))
        .register_builtin("m1.rule.move", DummyBuiltinModule::new());

        let output = sandbox
            .call(&request("m1.rule.move"))
            .expect("fallback success");
        assert_eq!(output, fallback_output);
    }

    #[test]
    fn preferred_fallback_uses_builtin_when_fallback_fails() {
        let fallback_failure = ModuleCallFailure {
            module_id: "m1.rule.move".to_string(),
            trace_id: "trace".to_string(),
            code: ModuleCallErrorCode::Trap,
            detail: "fallback failed".to_string(),
        };
        let mut sandbox = BuiltinModuleSandbox::with_preferred_fallback(Box::new(
            FixedSandbox::fail(fallback_failure),
        ))
        .register_builtin("m1.rule.move", DummyBuiltinModule::new());

        let output = sandbox
            .call(&request("m1.rule.move"))
            .expect("builtin fallback success");
        assert_eq!(output.new_state, Some(vec![1, 2, 3]));
    }

    #[test]
    fn preferred_fallback_returns_fallback_error_without_builtin() {
        let fallback_failure = ModuleCallFailure {
            module_id: "m1.rule.move".to_string(),
            trace_id: "trace".to_string(),
            code: ModuleCallErrorCode::Trap,
            detail: "fallback failed".to_string(),
        };
        let mut sandbox = BuiltinModuleSandbox::with_preferred_fallback(Box::new(
            FixedSandbox::fail(fallback_failure.clone()),
        ));

        let err = sandbox
            .call(&request("m1.rule.move"))
            .expect_err("fallback error");
        assert_eq!(err, fallback_failure);
    }
}
