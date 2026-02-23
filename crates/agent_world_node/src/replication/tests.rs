use super::*;
use std::path::PathBuf;

fn temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("duration")
        .as_nanos();
    std::env::temp_dir().join(format!("agent-world-replication-tests-{prefix}-{unique}"))
}

fn deterministic_keypair_hex(seed: u8) -> (String, String) {
    let bytes = [seed; 32];
    let signing_key = SigningKey::from_bytes(&bytes);
    (
        hex::encode(signing_key.to_bytes()),
        hex::encode(signing_key.verifying_key().to_bytes()),
    )
}

fn signed_remote_message(
    seed: u8,
    world_id: &str,
    node_id: &str,
    sequence: u64,
) -> GossipReplicationMessage {
    let (private_hex, public_hex) = deterministic_keypair_hex(seed);
    let signer = ReplicationSigningKey {
        signing_key: signing_key_from_hex(private_hex.as_str()).expect("signing key"),
        public_key_hex: public_hex.clone(),
    };
    let payload = format!("payload-{seed}-{sequence}").into_bytes();
    let path = format!("{COMMIT_FILE_PREFIX}/{:020}.json", sequence.max(1));
    let record = build_replication_record_with_epoch(
        world_id,
        public_hex.as_str(),
        1,
        sequence.max(1),
        path.as_str(),
        payload.as_slice(),
        1_000,
    )
    .expect("record");
    let mut message = GossipReplicationMessage {
        version: REPLICATION_VERSION,
        world_id: world_id.to_string(),
        node_id: node_id.to_string(),
        record,
        payload,
        public_key_hex: Some(public_hex),
        signature_hex: None,
    };
    message.signature_hex = Some(sign_replication_message(&message, &signer).expect("sign"));
    message
}

#[test]
fn next_local_record_position_rejects_sequence_overflow_for_existing_writer() {
    let dir = temp_dir("existing-writer-sequence-overflow");
    let config = NodeReplicationConfig::new(&dir).expect("config");
    let mut runtime = ReplicationRuntime::new(&config, "node-a").expect("runtime");
    runtime.guard = SingleWriterReplicationGuard {
        writer_id: Some("node-a".to_string()),
        writer_epoch: 7,
        last_sequence: u64::MAX,
    };
    runtime.writer_state = LocalWriterState {
        writer_epoch: 7,
        last_sequence: u64::MAX,
        last_replicated_height: 0,
    };

    let err = runtime
        .next_local_record_position("node-a")
        .expect_err("sequence overflow should fail");
    assert!(
        matches!(err, NodeError::Replication { reason } if reason.contains("sequence overflow"))
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn next_local_record_position_rejects_writer_epoch_overflow_on_writer_switch() {
    let dir = temp_dir("writer-switch-epoch-overflow");
    let config = NodeReplicationConfig::new(&dir).expect("config");
    let mut runtime = ReplicationRuntime::new(&config, "node-a").expect("runtime");
    runtime.guard = SingleWriterReplicationGuard {
        writer_id: Some("node-b".to_string()),
        writer_epoch: u64::MAX,
        last_sequence: 8,
    };
    runtime.writer_state = LocalWriterState {
        writer_epoch: u64::MAX,
        last_sequence: 12,
        last_replicated_height: 0,
    };

    let err = runtime
        .next_local_record_position("node-a")
        .expect_err("writer epoch overflow should fail");
    assert!(
        matches!(err, NodeError::Replication { reason } if reason.contains("writer_epoch overflow"))
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn next_local_record_position_rejects_sequence_overflow_without_guard_writer() {
    let dir = temp_dir("no-guard-sequence-overflow");
    let config = NodeReplicationConfig::new(&dir).expect("config");
    let mut runtime = ReplicationRuntime::new(&config, "node-a").expect("runtime");
    runtime.guard = SingleWriterReplicationGuard {
        writer_id: None,
        writer_epoch: DEFAULT_WRITER_EPOCH,
        last_sequence: 0,
    };
    runtime.writer_state = LocalWriterState {
        writer_epoch: 19,
        last_sequence: u64::MAX,
        last_replicated_height: 0,
    };

    let err = runtime
        .next_local_record_position("node-a")
        .expect_err("sequence overflow should fail");
    assert!(
        matches!(err, NodeError::Replication { reason } if reason.contains("sequence overflow"))
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn validate_remote_message_for_observe_rejects_writer_outside_allowlist() {
    let dir = temp_dir("allowlist-reject");
    let (local_private_hex, local_public_hex) = deterministic_keypair_hex(21);
    let (_, allowed_public_hex) = deterministic_keypair_hex(22);
    let config = NodeReplicationConfig::new(&dir)
        .expect("config")
        .with_signing_keypair(local_private_hex, local_public_hex)
        .expect("signing keypair")
        .with_remote_writer_allowlist(vec![allowed_public_hex])
        .expect("allowlist");
    let runtime = ReplicationRuntime::new(&config, "node-b").expect("runtime");
    let unauthorized_message = signed_remote_message(23, "world-allowlist", "node-a", 1);

    let err = runtime
        .validate_remote_message_for_observe("node-b", "world-allowlist", &unauthorized_message)
        .expect_err("unauthorized writer should fail");
    assert!(matches!(err, NodeError::Replication { reason } if reason.contains("not authorized")));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn validate_remote_message_for_observe_accepts_writer_in_allowlist() {
    let dir = temp_dir("allowlist-accept");
    let (local_private_hex, local_public_hex) = deterministic_keypair_hex(31);
    let (_, allowed_public_hex) = deterministic_keypair_hex(32);
    let config = NodeReplicationConfig::new(&dir)
        .expect("config")
        .with_signing_keypair(local_private_hex, local_public_hex)
        .expect("signing keypair")
        .with_remote_writer_allowlist(vec![allowed_public_hex])
        .expect("allowlist");
    let runtime = ReplicationRuntime::new(&config, "node-b").expect("runtime");
    let allowed_message = signed_remote_message(32, "world-allowlist", "node-a", 1);

    let accepted = runtime
        .validate_remote_message_for_observe("node-b", "world-allowlist", &allowed_message)
        .expect("authorized writer should pass");
    assert!(accepted);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_fetch_commit_request_signs_with_runtime_signer() {
    let dir = temp_dir("fetch-commit-sign");
    let (local_private_hex, local_public_hex) = deterministic_keypair_hex(41);
    let config = NodeReplicationConfig::new(&dir)
        .expect("config")
        .with_signing_keypair(local_private_hex, local_public_hex.clone())
        .expect("signing keypair")
        .with_remote_writer_allowlist(vec![local_public_hex.clone()])
        .expect("allowlist");
    let runtime = ReplicationRuntime::new(&config, "node-a").expect("runtime");

    let request = runtime
        .build_fetch_commit_request("world-fetch-sign", 7)
        .expect("build request");
    assert_eq!(
        request.requester_public_key_hex.as_deref(),
        Some(local_public_hex.as_str())
    );
    assert!(request.requester_signature_hex.is_some());
    config
        .authorize_fetch_commit_request(&request)
        .expect("signed request should pass authorization");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn authorize_fetch_commit_request_rejects_missing_signature_when_required() {
    let dir = temp_dir("fetch-commit-missing-signature");
    let (local_private_hex, local_public_hex) = deterministic_keypair_hex(42);
    let (_, allowed_public_hex) = deterministic_keypair_hex(43);
    let config = NodeReplicationConfig::new(&dir)
        .expect("config")
        .with_signing_keypair(local_private_hex, local_public_hex)
        .expect("signing keypair")
        .with_remote_writer_allowlist(vec![allowed_public_hex.clone()])
        .expect("allowlist");
    let request = FetchCommitRequest {
        world_id: "world-fetch-sign".to_string(),
        height: 9,
        requester_public_key_hex: Some(allowed_public_hex),
        requester_signature_hex: None,
    };

    let err = config
        .authorize_fetch_commit_request(&request)
        .expect_err("unsigned request should fail");
    assert!(matches!(
        err,
        NodeError::Replication { reason }
            if reason.contains("missing requester_signature_hex")
    ));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn authorize_fetch_blob_request_rejects_requester_outside_allowlist() {
    let dir = temp_dir("fetch-blob-allowlist");
    let (local_private_hex, local_public_hex) = deterministic_keypair_hex(44);
    let (_, allowed_public_hex) = deterministic_keypair_hex(45);
    let (requester_private_hex, requester_public_hex) = deterministic_keypair_hex(46);
    let config = NodeReplicationConfig::new(&dir)
        .expect("config")
        .with_signing_keypair(local_private_hex, local_public_hex)
        .expect("signing keypair")
        .with_remote_writer_allowlist(vec![allowed_public_hex])
        .expect("allowlist");
    let signer = ReplicationSigningKey {
        signing_key: signing_key_from_hex(requester_private_hex.as_str()).expect("signing key"),
        public_key_hex: requester_public_hex.clone(),
    };
    let mut request = FetchBlobRequest {
        content_hash: "hash-1".to_string(),
        requester_public_key_hex: Some(requester_public_hex),
        requester_signature_hex: None,
    };
    request.requester_signature_hex =
        Some(sign_fetch_blob_request(&request, &signer).expect("sign"));

    let err = config
        .authorize_fetch_blob_request(&request)
        .expect_err("out-of-allowlist requester should fail");
    assert!(matches!(err, NodeError::Replication { reason } if reason.contains("not authorized")));

    let _ = std::fs::remove_dir_all(&dir);
}
