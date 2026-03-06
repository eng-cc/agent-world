use super::{
    build_transfer_submit_action_payload, maybe_handle_transfer_submit_request,
    parse_transfer_submit_request, ChainTransferHistoryResponse, ChainTransferStatusResponse,
    ChainTransferSubmitResponse, TransferLifecycleStatus,
};
use agent_world::consensus_action_payload::{
    decode_consensus_action_payload, ConsensusActionPayloadBody,
};
use agent_world::runtime::Action;
use agent_world_node::{
    NodeConfig, NodeExecutionCommitContext, NodeExecutionCommitResult, NodeExecutionHook, NodeRole,
    NodeRuntime,
};
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn tcp_stream_pair() -> (TcpStream, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback listener");
    let bind = listener.local_addr().expect("read local addr");
    let client = TcpStream::connect(bind).expect("connect loopback client");
    let (server, _) = listener.accept().expect("accept loopback connection");
    (server, client)
}

#[derive(Debug)]
struct NoopExecutionHook;

impl NodeExecutionHook for NoopExecutionHook {
    fn on_commit(
        &mut self,
        context: NodeExecutionCommitContext,
    ) -> Result<NodeExecutionCommitResult, String> {
        Ok(NodeExecutionCommitResult {
            execution_height: context.height,
            execution_block_hash: format!("noop-block-{}", context.height),
            execution_state_root: format!("noop-root-{}", context.height),
        })
    }
}

fn decode_http_json_response<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> (u16, T) {
    let boundary = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("response must include HTTP body separator");
    let header = std::str::from_utf8(&bytes[..boundary]).expect("response header utf-8");
    let status = header
        .split_whitespace()
        .nth(1)
        .and_then(|token| token.parse::<u16>().ok())
        .expect("response status code");
    let payload =
        serde_json::from_slice::<T>(&bytes[(boundary + 4)..]).expect("response json payload");
    (status, payload)
}

#[test]
fn parse_transfer_submit_request_rejects_same_account() {
    let body =
        br#"{"from_account_id":"player:alice","to_account_id":"player:alice","amount":7,"nonce":1}"#;
    let err = parse_transfer_submit_request(body).expect_err("same source and target must fail");
    assert!(err.contains("cannot be the same"));
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
fn transfer_submit_handler_returns_invalid_request_for_bad_payload() {
    let runtime = Arc::new(Mutex::new(NodeRuntime::new(
        NodeConfig::new(
            "node-transfer-submit-bad",
            "world-transfer-submit-bad",
            NodeRole::Sequencer,
        )
        .expect("node config"),
    )));

    let (mut server_stream, mut client_stream) = tcp_stream_pair();
    let body =
        r#"{"from_account_id":"player:alice","to_account_id":"player:alice","amount":7,"nonce":2}"#;
    let request = format!(
        "POST /v1/chain/transfer/submit HTTP/1.1\r\nHost: 127.0.0.1:5121\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    let handled = maybe_handle_transfer_submit_request(
        &mut server_stream,
        request.as_bytes(),
        &runtime,
        "POST",
        "/v1/chain/transfer/submit",
        "node-transfer-submit-bad",
        "world-transfer-submit-bad",
        Path::new("."),
    )
    .expect("handler should process request");
    assert!(handled);
    drop(server_stream);

    client_stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("set client timeout");
    let mut response_bytes = Vec::new();
    client_stream
        .read_to_end(&mut response_bytes)
        .expect("read handler response");
    let (status, response): (u16, ChainTransferSubmitResponse) =
        decode_http_json_response(&response_bytes);
    assert_eq!(status, 400);
    assert!(!response.ok);
    assert_eq!(response.error_code.as_deref(), Some("invalid_request"));
}

#[test]
fn transfer_status_and_history_endpoint_report_confirmed_record() {
    let config = NodeConfig::new(
        "node-transfer-query-ok",
        "world-transfer-query-ok",
        NodeRole::Sequencer,
    )
    .expect("node config")
    .with_tick_interval(Duration::from_millis(10))
    .expect("tick interval");
    let mut node_runtime = NodeRuntime::new(config).with_execution_hook(NoopExecutionHook);
    node_runtime.start().expect("start node runtime");
    let runtime = Arc::new(Mutex::new(node_runtime));

    let (mut submit_server, mut submit_client) = tcp_stream_pair();
    let submit_body =
        r#"{"from_account_id":"player:alice","to_account_id":"player:bob","amount":3,"nonce":8}"#;
    let submit_http = format!(
        "POST /v1/chain/transfer/submit HTTP/1.1\r\nHost: 127.0.0.1:5121\r\nContent-Length: {}\r\n\r\n{}",
        submit_body.len(),
        submit_body
    );
    maybe_handle_transfer_submit_request(
        &mut submit_server,
        submit_http.as_bytes(),
        &runtime,
        "POST",
        "/v1/chain/transfer/submit",
        "node-transfer-query-ok",
        "world-transfer-query-ok",
        Path::new("."),
    )
    .expect("submit should be handled");
    drop(submit_server);

    let mut submit_response_bytes = Vec::new();
    submit_client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("set timeout");
    submit_client
        .read_to_end(&mut submit_response_bytes)
        .expect("read submit response");
    let (_, submit_response): (u16, ChainTransferSubmitResponse) =
        decode_http_json_response(&submit_response_bytes);
    assert_eq!(
        submit_response.lifecycle_status,
        Some(TransferLifecycleStatus::Accepted)
    );
    let action_id = submit_response.action_id.expect("action_id");

    let deadline = Instant::now() + Duration::from_secs(3);
    let mut observed_confirmed = false;
    while Instant::now() < deadline {
        let (mut status_server, mut status_client) = tcp_stream_pair();
        let status_http = format!(
            "GET /v1/chain/transfer/status?action_id={} HTTP/1.1\r\nHost: 127.0.0.1:5121\r\n\r\n",
            action_id
        );
        maybe_handle_transfer_submit_request(
            &mut status_server,
            status_http.as_bytes(),
            &runtime,
            "GET",
            "/v1/chain/transfer/status",
            "node-transfer-query-ok",
            "world-transfer-query-ok",
            Path::new("."),
        )
        .expect("status should be handled");
        drop(status_server);

        status_client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("set timeout");
        let mut status_response_bytes = Vec::new();
        status_client
            .read_to_end(&mut status_response_bytes)
            .expect("read status response");
        let (_, status_response): (u16, ChainTransferStatusResponse) =
            decode_http_json_response(&status_response_bytes);
        let status = status_response.status.expect("status payload");
        if status.status == TransferLifecycleStatus::Confirmed {
            observed_confirmed = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(80));
    }
    assert!(
        observed_confirmed,
        "status should eventually become confirmed"
    );

    let (mut history_server, mut history_client) = tcp_stream_pair();
    let history_http = format!(
        "GET /v1/chain/transfer/history?action_id={} HTTP/1.1\r\nHost: 127.0.0.1:5121\r\n\r\n",
        action_id
    );
    maybe_handle_transfer_submit_request(
        &mut history_server,
        history_http.as_bytes(),
        &runtime,
        "GET",
        "/v1/chain/transfer/history",
        "node-transfer-query-ok",
        "world-transfer-query-ok",
        Path::new("."),
    )
    .expect("history should be handled");
    drop(history_server);

    history_client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("set timeout");
    let mut history_response_bytes = Vec::new();
    history_client
        .read_to_end(&mut history_response_bytes)
        .expect("read history response");
    let (_, history_response): (u16, ChainTransferHistoryResponse) =
        decode_http_json_response(&history_response_bytes);
    assert!(history_response.ok);
    assert_eq!(history_response.total, 1);
    assert_eq!(history_response.items[0].action_id, action_id);

    runtime
        .lock()
        .expect("lock runtime for stop")
        .stop()
        .expect("stop node runtime");
}
