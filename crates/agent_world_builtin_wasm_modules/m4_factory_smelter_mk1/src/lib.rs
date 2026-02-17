#![allow(improper_ctypes_definitions)]

const MODULE_ID: &str = agent_world_builtin_wasm::M4_FACTORY_SMELTER_MODULE_ID;

#[no_mangle]
pub extern "C" fn alloc(len: i32) -> i32 {
    agent_world_builtin_wasm::builtin_alloc(len)
}

#[no_mangle]
pub extern "C" fn reduce(input_ptr: i32, input_len: i32) -> (i32, i32) {
    agent_world_builtin_wasm::reduce_for_module(MODULE_ID, input_ptr, input_len)
}

#[no_mangle]
pub extern "C" fn call(input_ptr: i32, input_len: i32) -> (i32, i32) {
    agent_world_builtin_wasm::call_for_module(MODULE_ID, input_ptr, input_len)
}
