#![allow(improper_ctypes_definitions)]

use agent_world_wasm_sdk::{export_wasm_module, LifecycleStage, WasmModuleLifecycle};

const MODULE_ID: &str = agent_world_builtin_wasm_runtime::M4_FACTORY_ASSEMBLER_MODULE_ID;

#[derive(Default)]
struct BuiltinWasmModule;

impl WasmModuleLifecycle for BuiltinWasmModule {
    fn module_id(&self) -> &'static str {
        MODULE_ID
    }

    fn alloc(&mut self, len: i32) -> i32 {
        agent_world_builtin_wasm_runtime::builtin_alloc(len)
    }

    fn on_init(&mut self, _stage: LifecycleStage) {}

    fn on_teardown(&mut self, _stage: LifecycleStage) {}

    fn on_reduce(&mut self, input_ptr: i32, input_len: i32) -> (i32, i32) {
        agent_world_builtin_wasm_runtime::reduce_for_module(self.module_id(), input_ptr, input_len)
    }

    fn on_call(&mut self, input_ptr: i32, input_len: i32) -> (i32, i32) {
        agent_world_builtin_wasm_runtime::call_for_module(self.module_id(), input_ptr, input_len)
    }
}

export_wasm_module!(BuiltinWasmModule);
