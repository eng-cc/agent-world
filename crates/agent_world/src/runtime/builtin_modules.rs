//! Built-in module implementations for development and testing.

use std::collections::BTreeMap;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::geometry::GeoPos;
use crate::simulator::CM_PER_KM;

use super::sandbox::{
    ModuleCallErrorCode, ModuleCallFailure, ModuleCallRequest, ModuleOutput, ModuleSandbox,
};
use super::util::to_canonical_cbor;
use super::world_event::{WorldEvent, WorldEventBody};

mod body_module;
mod default_modules;
mod power_modules;
mod rule_modules;

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

pub use body_module::M1BodyModule;
pub use default_modules::{M1MemoryModule, M1MobilityModule, M1SensorModule, M1StorageCargoModule};
pub use power_modules::{
    M1RadiationPowerModule, M1StoragePowerModule, M1_POWER_HARVEST_BASE_PER_TICK,
    M1_POWER_HARVEST_DISTANCE_BONUS_CAP, M1_POWER_HARVEST_DISTANCE_STEP_CM,
    M1_POWER_MODULE_VERSION, M1_POWER_STORAGE_CAPACITY, M1_POWER_STORAGE_INITIAL_LEVEL,
    M1_POWER_STORAGE_MOVE_COST_PER_KM, M1_RADIATION_POWER_MODULE_ID, M1_STORAGE_POWER_MODULE_ID,
};
pub use rule_modules::{M1MoveRuleModule, M1TransferRuleModule, M1VisibilityRuleModule};

pub trait BuiltinModule {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure>;
}

pub struct BuiltinModuleSandbox {
    builtins: BTreeMap<String, Box<dyn BuiltinModule>>,
    fallback: Option<Box<dyn ModuleSandbox>>,
    prefer_fallback: bool,
}

impl BuiltinModuleSandbox {
    pub fn new() -> Self {
        Self {
            builtins: BTreeMap::new(),
            fallback: None,
            prefer_fallback: false,
        }
    }

    pub fn with_fallback(fallback: Box<dyn ModuleSandbox>) -> Self {
        Self {
            builtins: BTreeMap::new(),
            fallback: Some(fallback),
            prefer_fallback: false,
        }
    }

    pub fn with_preferred_fallback(fallback: Box<dyn ModuleSandbox>) -> Self {
        Self {
            builtins: BTreeMap::new(),
            fallback: Some(fallback),
            prefer_fallback: true,
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
        if self.prefer_fallback {
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
        }

        if let Some(module) = self.builtins.get_mut(&request.module_id) {
            return module.call(request);
        }
        if let Some(fallback) = self.fallback.as_mut() {
            return fallback.call(request);
        }
        Err(ModuleCallFailure {
            module_id: request.module_id.clone(),
            trace_id: request.trace_id.clone(),
            code: ModuleCallErrorCode::SandboxUnavailable,
            detail: "builtin module not found".to_string(),
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PositionState {
    agents: BTreeMap<String, GeoPos>,
}

fn decode_state<T: DeserializeOwned + Default>(
    state: Option<&[u8]>,
    request: &ModuleCallRequest,
) -> Result<T, ModuleCallFailure> {
    let Some(state) = state else {
        return Ok(T::default());
    };
    if state.is_empty() {
        return Ok(T::default());
    }
    decode_input(request, state)
}

fn encode_state<T: Serialize>(
    state: &T,
    request: &ModuleCallRequest,
) -> Result<Vec<u8>, ModuleCallFailure> {
    to_canonical_cbor(state).map_err(|err| {
        failure(
            request,
            ModuleCallErrorCode::InvalidOutput,
            format!("state encode failed: {err:?}"),
        )
    })
}

fn update_position_state(state: &mut PositionState, event: WorldEvent) -> bool {
    let mut changed = false;
    if let WorldEventBody::Domain(domain) = event.body {
        match domain {
            super::events::DomainEvent::AgentRegistered { agent_id, pos } => {
                state.agents.insert(agent_id, pos);
                changed = true;
            }
            super::events::DomainEvent::AgentMoved { agent_id, to, .. } => {
                state.agents.insert(agent_id, to);
                changed = true;
            }
            super::events::DomainEvent::ActionRejected { .. } => {}
            super::events::DomainEvent::Observation { .. } => {}
            super::events::DomainEvent::BodyAttributesUpdated { .. } => {}
            super::events::DomainEvent::BodyAttributesRejected { .. } => {}
            super::events::DomainEvent::BodyInterfaceExpanded { .. } => {}
            super::events::DomainEvent::BodyInterfaceExpandRejected { .. } => {}
            super::events::DomainEvent::ResourceTransferred { .. } => {}
        }
    }
    changed
}

fn decode_input<T: DeserializeOwned>(
    request: &ModuleCallRequest,
    bytes: &[u8],
) -> Result<T, ModuleCallFailure> {
    serde_cbor::from_slice(bytes).map_err(|err| {
        failure(
            request,
            ModuleCallErrorCode::InvalidOutput,
            format!("input CBOR decode failed: {err}"),
        )
    })
}

fn finalize_output(
    mut output: ModuleOutput,
    request: &ModuleCallRequest,
) -> Result<ModuleOutput, ModuleCallFailure> {
    output.output_bytes = 0;
    let encoded = serde_cbor::to_vec(&output).map_err(|err| {
        failure(
            request,
            ModuleCallErrorCode::InvalidOutput,
            format!("output encode failed: {err}"),
        )
    })?;
    output.output_bytes = encoded.len() as u64;
    Ok(output)
}

fn failure(
    request: &ModuleCallRequest,
    code: ModuleCallErrorCode,
    detail: String,
) -> ModuleCallFailure {
    ModuleCallFailure {
        module_id: request.module_id.clone(),
        trace_id: request.trace_id.clone(),
        code,
        detail,
    }
}

fn movement_cost(distance_cm: i64, per_km_cost: i64) -> i64 {
    if distance_cm <= 0 || per_km_cost <= 0 {
        return 0;
    }
    let km = (distance_cm + CM_PER_KM - 1) / CM_PER_KM;
    km.saturating_mul(per_km_cost)
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
