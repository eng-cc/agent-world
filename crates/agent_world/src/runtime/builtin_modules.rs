//! Built-in module compatibility helpers for wasm cutover.

use super::sandbox::{ModuleCallFailure, ModuleCallRequest, ModuleOutput, ModuleSandbox};

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

pub struct BuiltinModuleSandbox {
    fallback: Box<dyn ModuleSandbox>,
}

impl BuiltinModuleSandbox {
    pub fn with_preferred_fallback(fallback: Box<dyn ModuleSandbox>) -> Self {
        Self { fallback }
    }
}

impl ModuleSandbox for BuiltinModuleSandbox {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure> {
        self.fallback.call(request)
    }
}

#[cfg(test)]
mod tests {
    use super::super::modules::ModuleLimits;
    use super::super::sandbox::{FixedSandbox, ModuleCallFailure, ModuleOutput};
    use super::*;

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
    fn preferred_fallback_returns_fallback_success() {
        let fallback_output = ModuleOutput {
            new_state: Some(vec![9]),
            effects: Vec::new(),
            emits: Vec::new(),
            output_bytes: 1,
        };
        let mut sandbox = BuiltinModuleSandbox::with_preferred_fallback(Box::new(
            FixedSandbox::succeed(fallback_output.clone()),
        ));

        let output = sandbox
            .call(&request("m1.rule.move"))
            .expect("fallback success");
        assert_eq!(output, fallback_output);
    }

    #[test]
    fn preferred_fallback_returns_fallback_error() {
        use super::super::sandbox::ModuleCallErrorCode;

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
