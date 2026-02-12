use serde::{Deserialize, Serialize};
use serde_json::json;

pub const M1_MOVE_RULE_MODULE_ID: &str = "m1.rule.move";
const RULE_DECISION_EMIT_KIND: &str = "rule.decision";

#[derive(Debug, Clone, Deserialize)]
struct ModuleCallInput {
    #[serde(default)]
    action: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Deserialize)]
struct ActionEnvelopeLite {
    id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ModuleEffectIntent {
    kind: String,
    params: serde_json::Value,
    cap_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ModuleEmit {
    kind: String,
    payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ModuleOutput {
    new_state: Option<Vec<u8>>,
    #[serde(default)]
    effects: Vec<ModuleEffectIntent>,
    #[serde(default)]
    emits: Vec<ModuleEmit>,
    output_bytes: u64,
}

fn action_id_from_input(input_bytes: &[u8]) -> Option<u64> {
    let input: ModuleCallInput = serde_cbor::from_slice(input_bytes).ok()?;
    let action_bytes = input.action?;
    if action_bytes.is_empty() {
        return None;
    }
    let action: ActionEnvelopeLite = serde_cbor::from_slice(&action_bytes).ok()?;
    Some(action.id)
}

fn build_allow_rule_output(input_bytes: &[u8]) -> Vec<u8> {
    let action_id = action_id_from_input(input_bytes).unwrap_or(0);
    let payload = json!({
        "action_id": action_id,
        "verdict": "allow",
    });
    let output = ModuleOutput {
        new_state: None,
        effects: Vec::new(),
        emits: vec![ModuleEmit {
            kind: RULE_DECISION_EMIT_KIND.to_string(),
            payload,
        }],
        output_bytes: 0,
    };
    serde_cbor::to_vec(&output).unwrap_or_default()
}

fn call_impl(input_ptr: i32, input_len: i32) -> (i32, i32) {
    let input = if input_ptr > 0 && input_len > 0 {
        let ptr = input_ptr as *const u8;
        let len = input_len as usize;
        // SAFETY: host guarantees valid wasm linear memory pointer/len for the call.
        unsafe { std::slice::from_raw_parts(ptr, len).to_vec() }
    } else {
        Vec::new()
    };
    let output = build_allow_rule_output(&input);
    write_bytes_to_memory(&output)
}

fn write_bytes_to_memory(bytes: &[u8]) -> (i32, i32) {
    let len = i32::try_from(bytes.len()).unwrap_or(0);
    if len <= 0 {
        return (0, 0);
    }
    let ptr = alloc(len);
    if ptr <= 0 {
        return (0, 0);
    }
    // SAFETY: alloc returns a writable wasm linear memory region with at least len bytes.
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, len as usize);
    }
    (ptr, len)
}

#[no_mangle]
pub extern "C" fn alloc(len: i32) -> i32 {
    if len <= 0 {
        return 0;
    }
    let capacity = len as usize;
    let mut buf = Vec::<u8>::with_capacity(capacity);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr as i32
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn reduce(input_ptr: i32, input_len: i32) -> (i32, i32) {
    call_impl(input_ptr, input_len)
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
pub extern "C" fn call(input_ptr: i32, input_len: i32) -> (i32, i32) {
    call_impl(input_ptr, input_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize)]
    struct ActionEnvelopeTest {
        id: u64,
        action: serde_json::Value,
    }

    #[derive(Debug, Clone, Serialize)]
    struct ModuleCallInputTest {
        #[serde(skip_serializing_if = "Option::is_none")]
        action: Option<Vec<u8>>,
    }

    #[test]
    fn allow_output_uses_action_id_from_input() {
        let action = ActionEnvelopeTest {
            id: 42,
            action: json!({"kind":"move_agent"}),
        };
        let action_bytes = serde_cbor::to_vec(&action).expect("encode action");
        let input = ModuleCallInputTest {
            action: Some(action_bytes),
        };
        let input_bytes = serde_cbor::to_vec(&input).expect("encode input");

        let output_bytes = build_allow_rule_output(&input_bytes);
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        assert_eq!(output.emits.len(), 1);
        assert_eq!(output.emits[0].kind, RULE_DECISION_EMIT_KIND);
        assert_eq!(output.emits[0].payload["action_id"], json!(42));
        assert_eq!(output.emits[0].payload["verdict"], json!("allow"));
    }

    #[test]
    fn allow_output_falls_back_to_zero_when_input_invalid() {
        let output_bytes = build_allow_rule_output(b"invalid-cbor");
        let output: ModuleOutput = serde_cbor::from_slice(&output_bytes).expect("decode output");
        assert_eq!(output.emits[0].payload["action_id"], json!(0));
    }
}
