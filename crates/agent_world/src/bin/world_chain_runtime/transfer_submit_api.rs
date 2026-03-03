use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use agent_world::consensus_action_payload::{
    encode_consensus_action_payload, ConsensusActionPayloadEnvelope,
};
use agent_world::runtime::Action;
use agent_world_node::NodeRuntime;
use serde::{Deserialize, Serialize};

const TRANSFER_PATH: &str = "/v1/chain/transfer/submit";
const ACCOUNT_ID_MAX_LEN: usize = 128;
const TRANSFER_ERROR_INVALID_REQUEST: &str = "invalid_request";
const TRANSFER_ERROR_INTERNAL: &str = "internal_error";
const TRANSFER_ERROR_SUBMIT_FAILED: &str = "submit_failed";

static NEXT_TRANSFER_ACTION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ChainTransferSubmitRequest {
    pub(super) from_account_id: String,
    pub(super) to_account_id: String,
    pub(super) amount: u64,
    pub(super) nonce: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct ChainTransferSubmitResponse {
    pub(super) ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) action_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) submitted_at_unix_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) error: Option<String>,
}

impl ChainTransferSubmitResponse {
    pub(super) fn success(action_id: u64, submitted_at_unix_ms: i64) -> Self {
        Self {
            ok: true,
            action_id: Some(action_id),
            submitted_at_unix_ms: Some(submitted_at_unix_ms),
            error_code: None,
            error: None,
        }
    }

    pub(super) fn error(error_code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            ok: false,
            action_id: None,
            submitted_at_unix_ms: None,
            error_code: Some(error_code.into()),
            error: Some(message.into()),
        }
    }
}

pub(super) fn maybe_handle_transfer_submit_request(
    stream: &mut TcpStream,
    request_bytes: &[u8],
    runtime: &Arc<Mutex<NodeRuntime>>,
    method: &str,
    path: &str,
) -> Result<bool, String> {
    if !method.eq_ignore_ascii_case("POST") || path != TRANSFER_PATH {
        return Ok(false);
    }

    let body = match super::feedback_submit_api::extract_http_json_body(request_bytes) {
        Ok(body) => body,
        Err(err) => {
            write_transfer_submit_error(stream, 400, TRANSFER_ERROR_INVALID_REQUEST, err.as_str())?;
            return Ok(true);
        }
    };
    let submit_request = match parse_transfer_submit_request(body) {
        Ok(request) => request,
        Err(err) => {
            write_transfer_submit_error(stream, 400, TRANSFER_ERROR_INVALID_REQUEST, err.as_str())?;
            return Ok(true);
        }
    };

    let action_id = match next_transfer_action_id() {
        Ok(action_id) => action_id,
        Err(err) => {
            write_transfer_submit_error(stream, 502, TRANSFER_ERROR_INTERNAL, err.as_str())?;
            return Ok(true);
        }
    };
    let payload = match build_transfer_submit_action_payload(&submit_request) {
        Ok(payload) => payload,
        Err(err) => {
            write_transfer_submit_error(stream, 502, TRANSFER_ERROR_INTERNAL, err.as_str())?;
            return Ok(true);
        }
    };
    if let Err(err) = runtime
        .lock()
        .map_err(|_| "failed to lock node runtime for transfer submit".to_string())?
        .submit_consensus_action_payload(action_id, payload)
    {
        write_transfer_submit_error(
            stream,
            502,
            TRANSFER_ERROR_SUBMIT_FAILED,
            format!("transfer submit failed: {err}").as_str(),
        )?;
        return Ok(true);
    }

    let response = ChainTransferSubmitResponse::success(action_id, super::now_unix_ms());
    let body = serde_json::to_vec_pretty(&response)
        .map_err(|err| format!("failed to encode transfer submit response: {err}"))?;
    super::write_json_response(stream, 200, body.as_slice(), false)
        .map_err(|err| format!("failed to write transfer submit response: {err}"))?;
    Ok(true)
}

pub(super) fn parse_transfer_submit_request(
    body: &[u8],
) -> Result<ChainTransferSubmitRequest, String> {
    let mut request: ChainTransferSubmitRequest = serde_json::from_slice(body)
        .map_err(|err| format!("invalid transfer submit payload: {err}"))?;

    request.from_account_id =
        normalize_account_id(request.from_account_id.as_str(), "from_account_id")?;
    request.to_account_id = normalize_account_id(request.to_account_id.as_str(), "to_account_id")?;

    if request.from_account_id == request.to_account_id {
        return Err("transfer from_account_id and to_account_id cannot be the same".to_string());
    }
    if request.amount == 0 {
        return Err("transfer amount must be > 0".to_string());
    }
    if request.nonce == 0 {
        return Err("transfer nonce must be > 0".to_string());
    }
    Ok(request)
}

fn normalize_account_id(raw: &str, field: &str) -> Result<String, String> {
    let account_id = raw.trim();
    if account_id.is_empty() {
        return Err(format!("transfer {field} cannot be empty"));
    }
    if account_id.len() > ACCOUNT_ID_MAX_LEN {
        return Err(format!(
            "transfer {field} exceeds max length {ACCOUNT_ID_MAX_LEN}"
        ));
    }
    if !account_id.bytes().all(is_allowed_account_id_byte) {
        return Err(format!(
            "transfer {field} must only contain ASCII letters, digits, ':', '-', '_' or '.'"
        ));
    }
    Ok(account_id.to_string())
}

fn is_allowed_account_id_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'-' | b'_' | b'.')
}

fn next_transfer_action_id() -> Result<u64, String> {
    let action_id = NEXT_TRANSFER_ACTION_ID.fetch_add(1, Ordering::Relaxed);
    if action_id == 0 {
        return Err("transfer action id allocator exhausted".to_string());
    }
    Ok(action_id)
}

pub(super) fn build_transfer_submit_action_payload(
    request: &ChainTransferSubmitRequest,
) -> Result<Vec<u8>, String> {
    let action = Action::TransferMainToken {
        from_account_id: request.from_account_id.clone(),
        to_account_id: request.to_account_id.clone(),
        amount: request.amount,
        nonce: request.nonce,
    };
    let envelope = ConsensusActionPayloadEnvelope::from_runtime_action(action);
    encode_consensus_action_payload(&envelope)
}

fn write_transfer_submit_error(
    stream: &mut TcpStream,
    status_code: u16,
    error_code: &str,
    error: &str,
) -> Result<(), String> {
    let payload = ChainTransferSubmitResponse::error(error_code, error);
    let body = serde_json::to_vec_pretty(&payload)
        .map_err(|err| format!("failed to encode transfer submit error payload: {err}"))?;
    super::write_json_response(stream, status_code, body.as_slice(), false)
        .map_err(|err| format!("failed to write transfer submit error response: {err}"))
}

#[cfg(test)]
mod tests {
    use super::{
        build_transfer_submit_action_payload, parse_transfer_submit_request,
        ChainTransferSubmitResponse,
    };
    use agent_world::consensus_action_payload::{
        decode_consensus_action_payload, ConsensusActionPayloadBody,
    };
    use agent_world::runtime::Action;

    #[test]
    fn parse_transfer_submit_request_rejects_same_account() {
        let body =
            br#"{"from_account_id":"player:alice","to_account_id":"player:alice","amount":7,"nonce":1}"#;
        let err =
            parse_transfer_submit_request(body).expect_err("same source and target must fail");
        assert!(err.contains("cannot be the same"));
    }

    #[test]
    fn parse_transfer_submit_request_rejects_invalid_account_id() {
        let body = br#"{"from_account_id":"player:alice","to_account_id":"bad account","amount":7,"nonce":1}"#;
        let err = parse_transfer_submit_request(body).expect_err("invalid account id should fail");
        assert!(err.contains("ASCII letters"));
    }

    #[test]
    fn parse_transfer_submit_request_requires_positive_amount_and_nonce() {
        let amount_err = parse_transfer_submit_request(
            br#"{"from_account_id":"player:alice","to_account_id":"player:bob","amount":0,"nonce":1}"#,
        )
        .expect_err("amount=0 should fail");
        assert!(amount_err.contains("amount"));

        let nonce_err = parse_transfer_submit_request(
            br#"{"from_account_id":"player:alice","to_account_id":"player:bob","amount":1,"nonce":0}"#,
        )
        .expect_err("nonce=0 should fail");
        assert!(nonce_err.contains("nonce"));
    }

    #[test]
    fn build_transfer_submit_action_payload_encodes_runtime_action() {
        let request = parse_transfer_submit_request(
            br#"{"from_account_id":" player:alice ","to_account_id":"player:bob","amount":7,"nonce":2}"#,
        )
        .expect("request should parse");
        let payload = build_transfer_submit_action_payload(&request).expect("payload");
        let decoded = decode_consensus_action_payload(payload.as_slice()).expect("decode payload");
        match decoded {
            ConsensusActionPayloadBody::RuntimeAction { action } => match action {
                Action::TransferMainToken {
                    from_account_id,
                    to_account_id,
                    amount,
                    nonce,
                } => {
                    assert_eq!(from_account_id, "player:alice");
                    assert_eq!(to_account_id, "player:bob");
                    assert_eq!(amount, 7);
                    assert_eq!(nonce, 2);
                }
                other => panic!("expected TransferMainToken action, got {other:?}"),
            },
            other => panic!("expected runtime action payload, got {other:?}"),
        }
    }

    #[test]
    fn chain_transfer_submit_response_error_fields() {
        let response = ChainTransferSubmitResponse::error("invalid_request", "failed");
        assert!(!response.ok);
        assert!(response.action_id.is_none());
        assert_eq!(response.error_code.as_deref(), Some("invalid_request"));
        assert_eq!(response.error.as_deref(), Some("failed"));
    }
}
