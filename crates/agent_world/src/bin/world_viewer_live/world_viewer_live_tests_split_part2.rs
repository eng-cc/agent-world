#[test]
fn build_reward_settlement_mint_records_uses_preview_world_without_mutation() {
    let mut world = RuntimeWorld::new();
    world
        .bind_node_identity("node-a", "public-key-node-a")
        .expect("bind node-a identity");
    let signer_private = [21_u8; 32];
    let signer_key = ed25519_dalek::SigningKey::from_bytes(&signer_private);
    let signer_private_key_hex = hex::encode(signer_key.to_bytes());
    let signer_public_key_hex = hex::encode(signer_key.verifying_key().to_bytes());
    world
        .bind_node_identity("node-signer", signer_public_key_hex.as_str())
        .expect("bind signer identity");

    let report = agent_world::runtime::EpochSettlementReport {
        epoch_index: 1,
        pool_points: 20,
        storage_pool_points: 0,
        distributed_points: 20,
        storage_distributed_points: 0,
        total_distributed_points: 20,
        settlements: vec![agent_world::runtime::NodeSettlement {
            node_id: "node-a".to_string(),
            obligation_met: true,
            compute_score: 0.0,
            storage_score: 0.0,
            uptime_score: 0.0,
            reliability_score: 0.0,
            storage_reward_score: 0.0,
            rewardable_storage_bytes: 0,
            penalty_score: 0.0,
            total_score: 0.0,
            main_awarded_points: 20,
            storage_awarded_points: 0,
            awarded_points: 20,
            cumulative_points: 20,
        }],
    };

    let minted = build_reward_settlement_mint_records(
        &world,
        &report,
        "node-signer",
        signer_private_key_hex.as_str(),
    )
    .expect("build mint records");
    assert_eq!(minted.len(), 1);
    assert_eq!(minted[0].node_id, "node-a");
    assert_eq!(minted[0].minted_power_credits, 2);

    assert!(world.reward_mint_records().is_empty());
    assert_eq!(world.node_power_credit_balance("node-a"), 0);
}

#[test]
fn reward_runtime_node_identity_bindings_derive_signer_binding_from_root_key() {
    let options = parse_options(
        [
            "--topology",
            "single",
            "--node-validator",
            "node-sequencer:70",
            "--node-validator",
            "node-storage:30",
            "--reward-runtime-signer",
            "node-sequencer",
            "--reward-runtime-leader-node",
            "node-sequencer",
        ]
        .into_iter(),
    )
    .expect("options");

    let root_private = [41_u8; 32];
    let root_signing_key = ed25519_dalek::SigningKey::from_bytes(&root_private);
    let root_keypair = node_keypair_config::NodeKeypairConfig {
        private_key_hex: hex::encode(root_signing_key.to_bytes()),
        public_key_hex: hex::encode(root_signing_key.verifying_key().to_bytes()),
    };

    let bindings = reward_runtime_node_identity_bindings(
        &options,
        "node-sequencer",
        "node-sequencer",
        "node-sequencer",
        &root_keypair,
    )
    .expect("bindings");

    let expected_signer =
        derive_node_consensus_signer_keypair("node-sequencer", &root_keypair).expect("derive");
    assert_eq!(
        bindings.get("node-sequencer"),
        Some(&expected_signer.public_key_hex)
    );
    assert_ne!(
        bindings.get("node-sequencer"),
        Some(&root_keypair.public_key_hex)
    );
    assert!(bindings.contains_key("node-storage"));
}

#[test]
fn ensure_reward_runtime_settlement_node_identity_bindings_rejects_missing_binding() {
    let mut world = RuntimeWorld::new();
    let report = agent_world::runtime::EpochSettlementReport {
        epoch_index: 1,
        pool_points: 10,
        storage_pool_points: 0,
        distributed_points: 10,
        storage_distributed_points: 0,
        total_distributed_points: 10,
        settlements: vec![agent_world::runtime::NodeSettlement {
            node_id: "node-missing".to_string(),
            obligation_met: true,
            compute_score: 0.0,
            storage_score: 0.0,
            uptime_score: 0.0,
            reliability_score: 0.0,
            storage_reward_score: 0.0,
            rewardable_storage_bytes: 0,
            penalty_score: 0.0,
            total_score: 0.0,
            main_awarded_points: 10,
            storage_awarded_points: 0,
            awarded_points: 10,
            cumulative_points: 10,
        }],
    };

    let err = ensure_reward_runtime_settlement_node_identity_bindings(
        &mut world,
        &report,
        &std::collections::BTreeMap::new(),
    )
    .expect_err("missing binding should fail");
    assert!(err.contains("missing configured key"));
}

#[test]
fn collect_distfs_challenge_report_returns_zero_for_empty_store() {
    let dir = temp_dir("distfs-probe-empty");
    let mut state = StorageChallengeProbeCursorState::default();
    let report =
        collect_distfs_challenge_report(dir.as_path(), "world-1", "node-a", 1_000, &mut state)
            .expect("collect challenge report");
    assert_eq!(report.total_checks, 0);
    assert_eq!(report.passed_checks, 0);
    assert_eq!(report.failed_checks, 0);
    assert!(report.failure_reasons.is_empty());
    assert!(report.latest_proof_semantics.is_none());
    assert_eq!(state.rounds_executed, 1);
}

#[test]
fn collect_distfs_challenge_report_detects_hash_mismatch() {
    let dir = temp_dir("distfs-probe-mismatch");
    fs::create_dir_all(dir.as_path()).expect("create dir");
    let store = LocalCasStore::new(dir.as_path());
    let hash = store.put_bytes(b"storage-proof-data").expect("put");
    let blob_path = store.blobs_dir().join(format!("{hash}.blob"));
    fs::write(blob_path, b"tampered").expect("tamper");

    let mut state = StorageChallengeProbeCursorState::default();
    let report =
        collect_distfs_challenge_report(dir.as_path(), "world-1", "node-b", 2_000, &mut state)
            .expect("collect challenge report");
    assert_eq!(report.total_checks, 1);
    assert_eq!(report.passed_checks, 0);
    assert_eq!(report.failed_checks, 1);
    assert_eq!(
        report.failure_reasons.get("HASH_MISMATCH").copied(),
        Some(1)
    );
    assert!(report.latest_proof_semantics.is_none());
    assert_eq!(state.cumulative_total_checks, 1);
    assert_eq!(state.cumulative_failed_checks, 1);
    assert_eq!(
        state
            .cumulative_failure_reasons
            .get("HASH_MISMATCH")
            .copied(),
        Some(1)
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn reward_runtime_distfs_probe_state_roundtrip() {
    let root = temp_dir("distfs-probe-state-roundtrip");
    fs::create_dir_all(root.as_path()).expect("create root");
    let path = root.join("probe-state.json");
    let expected = StorageChallengeProbeCursorState {
        next_blob_cursor: 3,
        rounds_executed: 7,
        cumulative_total_checks: 20,
        cumulative_passed_checks: 16,
        cumulative_failed_checks: 4,
        cumulative_failure_reasons: [("HASH_MISMATCH".to_string(), 4)].into_iter().collect(),
        consecutive_failure_rounds: 4,
        backoff_until_unix_ms: 9_999,
        last_probe_unix_ms: Some(8_888),
        cumulative_backoff_skipped_rounds: 5,
        cumulative_backoff_applied_ms: 3_600,
        last_backoff_duration_ms: 400,
        last_backoff_reason: Some("HASH_MISMATCH".to_string()),
        last_backoff_multiplier: 4,
    };

    persist_reward_runtime_distfs_probe_state(path.as_path(), &expected)
        .expect("persist probe state");
    let loaded = load_reward_runtime_distfs_probe_state(path.as_path()).expect("load probe state");
    assert_eq!(loaded, expected);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn distfs_probe_runtime_config_is_report_serializable() {
    let config = DistfsProbeRuntimeConfig {
        max_sample_bytes: 4096,
        challenges_per_tick: 4,
        challenge_ttl_ms: 60_000,
        allowed_clock_skew_ms: 1234,
        adaptive_policy: agent_world_distfs::StorageChallengeAdaptivePolicy {
            max_checks_per_round: 8,
            failure_backoff_base_ms: 100,
            failure_backoff_max_ms: 1_600,
            backoff_multiplier_hash_mismatch: 5,
            backoff_multiplier_timeout: 3,
            ..agent_world_distfs::StorageChallengeAdaptivePolicy::default()
        },
    };
    let value = serde_json::to_value(config).expect("serialize config");
    assert_eq!(
        value
            .get("max_sample_bytes")
            .and_then(serde_json::Value::as_u64),
        Some(4096)
    );
    assert_eq!(
        value
            .get("challenges_per_tick")
            .and_then(serde_json::Value::as_u64),
        Some(4)
    );
    assert_eq!(
        value
            .get("challenge_ttl_ms")
            .and_then(serde_json::Value::as_i64),
        Some(60_000)
    );
    assert_eq!(
        value
            .get("allowed_clock_skew_ms")
            .and_then(serde_json::Value::as_i64),
        Some(1234)
    );
    assert_eq!(
        value
            .get("adaptive_policy")
            .and_then(serde_json::Value::as_object)
            .and_then(|policy| policy.get("max_checks_per_round"))
            .and_then(serde_json::Value::as_u64),
        Some(8)
    );
    assert_eq!(
        value
            .get("adaptive_policy")
            .and_then(serde_json::Value::as_object)
            .and_then(|policy| policy.get("failure_backoff_base_ms"))
            .and_then(serde_json::Value::as_i64),
        Some(100)
    );
    assert_eq!(
        value
            .get("adaptive_policy")
            .and_then(serde_json::Value::as_object)
            .and_then(|policy| policy.get("failure_backoff_max_ms"))
            .and_then(serde_json::Value::as_i64),
        Some(1_600)
    );
    assert_eq!(
        value
            .get("adaptive_policy")
            .and_then(serde_json::Value::as_object)
            .and_then(|policy| policy.get("backoff_multiplier_hash_mismatch"))
            .and_then(serde_json::Value::as_u64),
        Some(5)
    );
    assert_eq!(
        value
            .get("adaptive_policy")
            .and_then(serde_json::Value::as_object)
            .and_then(|policy| policy.get("backoff_multiplier_timeout"))
            .and_then(serde_json::Value::as_u64),
        Some(3)
    );
}

#[test]
fn distfs_probe_cursor_state_is_report_serializable() {
    let state = StorageChallengeProbeCursorState {
        next_blob_cursor: 9,
        rounds_executed: 12,
        cumulative_total_checks: 30,
        cumulative_passed_checks: 25,
        cumulative_failed_checks: 5,
        cumulative_failure_reasons: [("HASH_MISMATCH".to_string(), 5)].into_iter().collect(),
        consecutive_failure_rounds: 3,
        backoff_until_unix_ms: 20_000,
        last_probe_unix_ms: Some(19_500),
        cumulative_backoff_skipped_rounds: 6,
        cumulative_backoff_applied_ms: 9_000,
        last_backoff_duration_ms: 1_600,
        last_backoff_reason: Some("TIMEOUT".to_string()),
        last_backoff_multiplier: 3,
    };
    let value = serde_json::to_value(state).expect("serialize cursor state");
    assert_eq!(
        value
            .get("next_blob_cursor")
            .and_then(serde_json::Value::as_u64),
        Some(9)
    );
    assert_eq!(
        value
            .get("cumulative_failed_checks")
            .and_then(serde_json::Value::as_u64),
        Some(5)
    );
    assert_eq!(
        value
            .get("cumulative_failure_reasons")
            .and_then(serde_json::Value::as_object)
            .and_then(|reasons| reasons.get("HASH_MISMATCH"))
            .and_then(serde_json::Value::as_u64),
        Some(5)
    );
    assert_eq!(
        value
            .get("cumulative_backoff_skipped_rounds")
            .and_then(serde_json::Value::as_u64),
        Some(6)
    );
    assert_eq!(
        value
            .get("cumulative_backoff_applied_ms")
            .and_then(serde_json::Value::as_i64),
        Some(9_000)
    );
    assert_eq!(
        value
            .get("last_backoff_duration_ms")
            .and_then(serde_json::Value::as_i64),
        Some(1_600)
    );
    assert_eq!(
        value
            .get("last_backoff_reason")
            .and_then(serde_json::Value::as_str),
        Some("TIMEOUT")
    );
    assert_eq!(
        value
            .get("last_backoff_multiplier")
            .and_then(serde_json::Value::as_u64),
        Some(3)
    );
}

#[test]
fn reward_runtime_distfs_probe_state_loads_legacy_snapshot_with_defaults() {
    let root = temp_dir("distfs-probe-state-legacy");
    fs::create_dir_all(root.as_path()).expect("create root");
    let path = root.join("probe-state-legacy.json");
    let legacy = serde_json::json!({
        "next_blob_cursor": 1,
        "rounds_executed": 2,
        "cumulative_total_checks": 3,
        "cumulative_passed_checks": 2,
        "cumulative_failed_checks": 1,
        "cumulative_failure_reasons": { "HASH_MISMATCH": 1 },
        "consecutive_failure_rounds": 1,
        "backoff_until_unix_ms": 4000,
        "last_probe_unix_ms": 3990
    });
    fs::write(
        path.as_path(),
        serde_json::to_vec_pretty(&legacy).expect("serialize legacy"),
    )
    .expect("write legacy");

    let loaded = load_reward_runtime_distfs_probe_state(path.as_path()).expect("load legacy");
    assert_eq!(loaded.next_blob_cursor, 1);
    assert_eq!(loaded.rounds_executed, 2);
    assert_eq!(loaded.cumulative_total_checks, 3);
    assert_eq!(loaded.cumulative_backoff_skipped_rounds, 0);
    assert_eq!(loaded.cumulative_backoff_applied_ms, 0);
    assert_eq!(loaded.last_backoff_duration_ms, 0);
    assert!(loaded.last_backoff_reason.is_none());
    assert_eq!(loaded.last_backoff_multiplier, 0);

    let _ = fs::remove_dir_all(root);
}
