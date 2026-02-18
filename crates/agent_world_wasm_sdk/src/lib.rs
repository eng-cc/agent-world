#![cfg_attr(not(any(test, feature = "std")), no_std)]
#![allow(improper_ctypes_definitions)]

extern crate alloc;

use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleStage {
    Reduce,
    Call,
}

pub trait WasmModuleLifecycle: Default {
    fn module_id(&self) -> &'static str;

    fn alloc(&mut self, len: i32) -> i32 {
        default_alloc(len)
    }

    fn on_init(&mut self, stage: LifecycleStage);

    fn on_teardown(&mut self, stage: LifecycleStage);

    fn on_reduce(&mut self, input_ptr: i32, input_len: i32) -> (i32, i32);

    fn on_call(&mut self, input_ptr: i32, input_len: i32) -> (i32, i32) {
        self.on_reduce(input_ptr, input_len)
    }
}

pub fn default_alloc(len: i32) -> i32 {
    if len <= 0 {
        return 0;
    }
    let capacity = len as usize;
    let mut buf = Vec::<u8>::with_capacity(capacity);
    let ptr = buf.as_mut_ptr();
    core::mem::forget(buf);
    ptr as i32
}

pub fn dispatch_reduce<M: WasmModuleLifecycle>(input_ptr: i32, input_len: i32) -> (i32, i32) {
    let mut module = M::default();
    module.on_init(LifecycleStage::Reduce);
    let output = module.on_reduce(input_ptr, input_len);
    module.on_teardown(LifecycleStage::Reduce);
    output
}

pub fn dispatch_call<M: WasmModuleLifecycle>(input_ptr: i32, input_len: i32) -> (i32, i32) {
    let mut module = M::default();
    module.on_init(LifecycleStage::Call);
    let output = module.on_call(input_ptr, input_len);
    module.on_teardown(LifecycleStage::Call);
    output
}

#[cfg(feature = "wire")]
pub mod wire {
    use alloc::string::String;
    use alloc::vec::Vec;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Deserialize)]
    pub struct ModuleCallInput {
        pub ctx: ModuleContext,
        #[serde(default)]
        pub event: Option<Vec<u8>>,
        #[serde(default)]
        pub action: Option<Vec<u8>>,
        #[serde(default)]
        pub state: Option<Vec<u8>>,
    }

    #[derive(Debug, Clone, Deserialize)]
    pub struct ModuleContext {
        pub module_id: String,
        pub time: u64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct ModuleEffectIntent {
        pub kind: String,
        pub params: serde_json::Value,
        pub cap_ref: String,
        #[serde(default)]
        pub cap_slot: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct ModuleEmit {
        pub kind: String,
        pub payload: serde_json::Value,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct ModuleOutput {
        pub new_state: Option<Vec<u8>>,
        #[serde(default)]
        pub effects: Vec<ModuleEffectIntent>,
        #[serde(default)]
        pub emits: Vec<ModuleEmit>,
        pub output_bytes: u64,
    }

    pub fn empty_output() -> ModuleOutput {
        ModuleOutput {
            new_state: None,
            effects: Vec::new(),
            emits: Vec::new(),
            output_bytes: 0,
        }
    }

    pub fn encode_output(output: ModuleOutput) -> Vec<u8> {
        serde_cbor::to_vec(&output).unwrap_or_default()
    }

    pub fn decode_input(input_bytes: &[u8]) -> Option<ModuleCallInput> {
        serde_cbor::from_slice(input_bytes).ok()
    }

    pub fn decode_action<T: for<'de> Deserialize<'de>>(input: &ModuleCallInput) -> Option<T> {
        let bytes = input.action.as_deref()?;
        serde_cbor::from_slice(bytes).ok()
    }
}

#[macro_export]
macro_rules! export_wasm_module {
    ($module_ty:ty) => {
        #[no_mangle]
        pub extern "C" fn alloc(len: i32) -> i32 {
            let mut module = <$module_ty>::default();
            module.alloc(len)
        }

        #[no_mangle]
        pub extern "C" fn reduce(input_ptr: i32, input_len: i32) -> (i32, i32) {
            $crate::dispatch_reduce::<$module_ty>(input_ptr, input_len)
        }

        #[no_mangle]
        pub extern "C" fn call(input_ptr: i32, input_len: i32) -> (i32, i32) {
            $crate::dispatch_call::<$module_ty>(input_ptr, input_len)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::{
        default_alloc, dispatch_call, dispatch_reduce, LifecycleStage, WasmModuleLifecycle,
    };
    use core::sync::atomic::{AtomicUsize, Ordering};

    static INIT_REDUCE: AtomicUsize = AtomicUsize::new(0);
    static TEARDOWN_REDUCE: AtomicUsize = AtomicUsize::new(0);
    static INIT_CALL: AtomicUsize = AtomicUsize::new(0);
    static TEARDOWN_CALL: AtomicUsize = AtomicUsize::new(0);

    #[derive(Default)]
    struct TestModule;

    impl WasmModuleLifecycle for TestModule {
        fn module_id(&self) -> &'static str {
            "test.module"
        }

        fn on_init(&mut self, stage: LifecycleStage) {
            match stage {
                LifecycleStage::Reduce => {
                    INIT_REDUCE.fetch_add(1, Ordering::SeqCst);
                }
                LifecycleStage::Call => {
                    INIT_CALL.fetch_add(1, Ordering::SeqCst);
                }
            }
        }

        fn on_teardown(&mut self, stage: LifecycleStage) {
            match stage {
                LifecycleStage::Reduce => {
                    TEARDOWN_REDUCE.fetch_add(1, Ordering::SeqCst);
                }
                LifecycleStage::Call => {
                    TEARDOWN_CALL.fetch_add(1, Ordering::SeqCst);
                }
            }
        }

        fn on_reduce(&mut self, input_ptr: i32, input_len: i32) -> (i32, i32) {
            (input_ptr + 1, input_len + 2)
        }

        fn on_call(&mut self, input_ptr: i32, input_len: i32) -> (i32, i32) {
            (input_ptr + 3, input_len + 4)
        }
    }

    #[test]
    fn default_alloc_handles_non_positive() {
        assert_eq!(default_alloc(0), 0);
        assert_eq!(default_alloc(-1), 0);
    }

    #[test]
    fn dispatch_reduce_runs_lifecycle_hooks() {
        INIT_REDUCE.store(0, Ordering::SeqCst);
        TEARDOWN_REDUCE.store(0, Ordering::SeqCst);

        let output = dispatch_reduce::<TestModule>(10, 20);

        assert_eq!(output, (11, 22));
        assert_eq!(INIT_REDUCE.load(Ordering::SeqCst), 1);
        assert_eq!(TEARDOWN_REDUCE.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn dispatch_call_runs_lifecycle_hooks() {
        INIT_CALL.store(0, Ordering::SeqCst);
        TEARDOWN_CALL.store(0, Ordering::SeqCst);

        let output = dispatch_call::<TestModule>(10, 20);

        assert_eq!(output, (13, 24));
        assert_eq!(INIT_CALL.load(Ordering::SeqCst), 1);
        assert_eq!(TEARDOWN_CALL.load(Ordering::SeqCst), 1);
    }
}
