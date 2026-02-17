use super::*;
use agent_world::runtime::BlobStore;
use agent_world::runtime::LocalCasStore;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_config_path(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("duration")
        .as_nanos();
    std::env::temp_dir().join(format!("agent-world-{prefix}-{unique}.toml"))
}

fn temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("duration")
        .as_nanos();
    std::env::temp_dir().join(format!("agent-world-{prefix}-{unique}"))
}

#[test]
fn parse_options_defaults() {
    let options = parse_options([].into_iter()).expect("defaults");
    assert_eq!(options.scenario, WorldScenario::TwinRegionBootstrap);
    assert_eq!(options.bind_addr, "127.0.0.1:5010");
    assert!(options.web_bind_addr.is_none());
    assert_eq!(options.tick_ms, 200);
    assert!(!options.llm_mode);
    assert!(options.node_enabled);
    assert_eq!(options.node_id, "viewer-live-node");
    assert_eq!(options.node_role, NodeRole::Observer);
    assert_eq!(options.node_tick_ms, 200);
    assert!(options.node_auto_attest_all_validators);
    assert!(options.node_validators.is_empty());
    assert!(options.node_gossip_bind.is_none());
    assert!(options.node_gossip_peers.is_empty());
    assert!(options.node_repl_libp2p_listen.is_empty());
    assert!(options.node_repl_libp2p_peers.is_empty());
    assert!(options.node_repl_topic.is_none());
    assert!(!options.reward_runtime_enabled);
    assert!(!options.reward_runtime_auto_redeem);
    assert!(options.reward_runtime_signer_node_id.is_none());
    assert_eq!(
        options.reward_runtime_report_dir,
        DEFAULT_REWARD_RUNTIME_REPORT_DIR
    );
    assert_eq!(
        options.reward_points_per_credit,
        RewardAssetConfig::default().points_per_credit
    );
    assert_eq!(
        options.reward_credits_per_power_unit,
        RewardAssetConfig::default().credits_per_power_unit
    );
    assert_eq!(
        options.reward_max_redeem_power_per_epoch,
        RewardAssetConfig::default().max_redeem_power_per_epoch
    );
    assert_eq!(
        options.reward_min_redeem_power_unit,
        RewardAssetConfig::default().min_redeem_power_unit
    );
    assert_eq!(
        options.reward_initial_reserve_power_units,
        DEFAULT_REWARD_RUNTIME_RESERVE_UNITS
    );
    assert_eq!(
        options.reward_distfs_probe_config,
        DistfsProbeRuntimeConfig::default()
    );
}

#[test]
fn parse_options_enables_llm_mode() {
    let options = parse_options(["--llm"].into_iter()).expect("llm mode");
    assert!(options.llm_mode);
}

#[test]
fn parse_options_reads_custom_values() {
    let options = parse_options(
        [
            "llm_bootstrap",
            "--bind",
            "127.0.0.1:9001",
            "--web-bind",
            "127.0.0.1:9002",
            "--tick-ms",
            "50",
            "--node-id",
            "viewer-live-1",
            "--node-role",
            "storage",
            "--node-tick-ms",
            "30",
            "--node-validator",
            "node-a:60",
            "--node-validator",
            "node-b:40",
            "--node-no-auto-attest-all",
            "--node-gossip-bind",
            "127.0.0.1:6001",
            "--node-gossip-peer",
            "127.0.0.1:6002",
            "--node-gossip-peer",
            "127.0.0.1:6003",
            "--node-repl-libp2p-listen",
            "/ip4/127.0.0.1/tcp/7001",
            "--node-repl-libp2p-peer",
            "/ip4/127.0.0.1/tcp/7002/p2p/12D3KooWR6f1fVQqfJ9WQnB8GL9QykgjM7RzQ2xZQW6hUGNfj9t7",
            "--node-repl-topic",
            "aw.custom.replication",
            "--reward-runtime-enable",
            "--reward-runtime-auto-redeem",
            "--reward-runtime-signer",
            "reward-signer-1",
            "--reward-runtime-report-dir",
            "output/reward-custom",
            "--reward-distfs-probe-max-sample-bytes",
            "8192",
            "--reward-distfs-probe-per-tick",
            "3",
            "--reward-distfs-probe-ttl-ms",
            "45000",
            "--reward-distfs-probe-allowed-clock-skew-ms",
            "2000",
            "--reward-distfs-adaptive-max-checks-per-round",
            "9",
            "--reward-distfs-adaptive-backoff-base-ms",
            "250",
            "--reward-distfs-adaptive-backoff-max-ms",
            "2500",
            "--reward-distfs-adaptive-multiplier-hash-mismatch",
            "5",
            "--reward-distfs-adaptive-multiplier-missing-sample",
            "2",
            "--reward-distfs-adaptive-multiplier-timeout",
            "3",
            "--reward-distfs-adaptive-multiplier-read-io-error",
            "4",
            "--reward-distfs-adaptive-multiplier-signature-invalid",
            "6",
            "--reward-distfs-adaptive-multiplier-unknown",
            "7",
            "--reward-points-per-credit",
            "7",
            "--reward-credits-per-power-unit",
            "3",
            "--reward-max-redeem-power-per-epoch",
            "1200",
            "--reward-min-redeem-power-unit",
            "2",
            "--reward-initial-reserve-power-units",
            "888",
        ]
        .into_iter(),
    )
    .expect("custom");
    assert_eq!(options.scenario, WorldScenario::LlmBootstrap);
    assert_eq!(options.bind_addr, "127.0.0.1:9001");
    assert_eq!(options.web_bind_addr.as_deref(), Some("127.0.0.1:9002"));
    assert_eq!(options.tick_ms, 50);
    assert_eq!(options.node_id, "viewer-live-1");
    assert_eq!(options.node_role, NodeRole::Storage);
    assert_eq!(options.node_tick_ms, 30);
    assert!(!options.node_auto_attest_all_validators);
    assert_eq!(options.node_validators.len(), 2);
    assert_eq!(
        options.node_gossip_bind,
        Some("127.0.0.1:6001".parse::<SocketAddr>().expect("addr"))
    );
    assert_eq!(
        options.node_gossip_peers,
        vec![
            "127.0.0.1:6002".parse::<SocketAddr>().expect("addr"),
            "127.0.0.1:6003".parse::<SocketAddr>().expect("addr"),
        ]
    );
    assert_eq!(
        options.node_repl_libp2p_listen,
        vec!["/ip4/127.0.0.1/tcp/7001".to_string()]
    );
    assert_eq!(
        options.node_repl_libp2p_peers,
        vec![
            "/ip4/127.0.0.1/tcp/7002/p2p/12D3KooWR6f1fVQqfJ9WQnB8GL9QykgjM7RzQ2xZQW6hUGNfj9t7"
                .to_string()
        ]
    );
    assert_eq!(
        options.node_repl_topic.as_deref(),
        Some("aw.custom.replication")
    );
    assert!(options.reward_runtime_enabled);
    assert!(options.reward_runtime_auto_redeem);
    assert_eq!(
        options.reward_runtime_signer_node_id.as_deref(),
        Some("reward-signer-1")
    );
    assert_eq!(options.reward_runtime_report_dir, "output/reward-custom");
    assert_eq!(options.reward_points_per_credit, 7);
    assert_eq!(options.reward_credits_per_power_unit, 3);
    assert_eq!(options.reward_max_redeem_power_per_epoch, 1200);
    assert_eq!(options.reward_min_redeem_power_unit, 2);
    assert_eq!(options.reward_initial_reserve_power_units, 888);
    assert_eq!(options.reward_distfs_probe_config.max_sample_bytes, 8192);
    assert_eq!(options.reward_distfs_probe_config.challenges_per_tick, 3);
    assert_eq!(options.reward_distfs_probe_config.challenge_ttl_ms, 45000);
    assert_eq!(
        options.reward_distfs_probe_config.allowed_clock_skew_ms,
        2000
    );
    assert_eq!(
        options
            .reward_distfs_probe_config
            .adaptive_policy
            .max_checks_per_round,
        9
    );
    assert_eq!(
        options
            .reward_distfs_probe_config
            .adaptive_policy
            .failure_backoff_base_ms,
        250
    );
    assert_eq!(
        options
            .reward_distfs_probe_config
            .adaptive_policy
            .failure_backoff_max_ms,
        2500
    );
    assert_eq!(
        options
            .reward_distfs_probe_config
            .adaptive_policy
            .backoff_multiplier_hash_mismatch,
        5
    );
    assert_eq!(
        options
            .reward_distfs_probe_config
            .adaptive_policy
            .backoff_multiplier_missing_sample,
        2
    );
    assert_eq!(
        options
            .reward_distfs_probe_config
            .adaptive_policy
            .backoff_multiplier_timeout,
        3
    );
    assert_eq!(
        options
            .reward_distfs_probe_config
            .adaptive_policy
            .backoff_multiplier_read_io_error,
        4
    );
    assert_eq!(
        options
            .reward_distfs_probe_config
            .adaptive_policy
            .backoff_multiplier_signature_invalid,
        6
    );
    assert_eq!(
        options
            .reward_distfs_probe_config
            .adaptive_policy
            .backoff_multiplier_unknown,
        7
    );
    assert_eq!(
        options.node_validators,
        vec![
            PosValidator {
                validator_id: "node-a".to_string(),
                stake: 60,
            },
            PosValidator {
                validator_id: "node-b".to_string(),
                stake: 40,
            }
        ]
    );
}

#[test]
fn parse_options_rejects_zero_tick_ms() {
    let err = parse_options(["--tick-ms", "0"].into_iter()).expect_err("reject zero");
    assert!(err.contains("positive integer"));
}

#[test]
fn parse_options_rejects_zero_reward_distfs_probe_per_tick() {
    let err = parse_options(["--reward-distfs-probe-per-tick", "0"].into_iter())
        .expect_err("reject zero probe per tick");
    assert!(err.contains("--reward-distfs-probe-per-tick"));
}

#[test]
fn parse_options_rejects_negative_reward_distfs_probe_allowed_clock_skew_ms() {
    let err = parse_options(["--reward-distfs-probe-allowed-clock-skew-ms", "-1"].into_iter())
        .expect_err("reject negative probe clock skew");
    assert!(err.contains("--reward-distfs-probe-allowed-clock-skew-ms"));
}

#[test]
fn parse_options_rejects_zero_reward_distfs_adaptive_max_checks_per_round() {
    let err = parse_options(["--reward-distfs-adaptive-max-checks-per-round", "0"].into_iter())
        .expect_err("reject zero adaptive max checks");
    assert!(err.contains("--reward-distfs-adaptive-max-checks-per-round"));
}

#[test]
fn parse_options_rejects_reward_distfs_adaptive_backoff_max_less_than_base() {
    let err = parse_options(
        [
            "--reward-distfs-adaptive-backoff-base-ms",
            "200",
            "--reward-distfs-adaptive-backoff-max-ms",
            "100",
        ]
        .into_iter(),
    )
    .expect_err("reject max < base");
    assert!(err.contains("--reward-distfs-adaptive-backoff-max-ms"));
}

#[test]
fn parse_options_rejects_zero_reward_distfs_adaptive_multiplier_hash_mismatch() {
    let err = parse_options(["--reward-distfs-adaptive-multiplier-hash-mismatch", "0"].into_iter())
        .expect_err("reject zero hash mismatch multiplier");
    assert!(err.contains("--reward-distfs-adaptive-multiplier-hash-mismatch"));
}

#[test]
fn parse_options_rejects_zero_reward_distfs_adaptive_multiplier_unknown() {
    let err = parse_options(["--reward-distfs-adaptive-multiplier-unknown", "0"].into_iter())
        .expect_err("reject zero unknown multiplier");
    assert!(err.contains("--reward-distfs-adaptive-multiplier-unknown"));
}

#[test]
fn parse_options_disables_node() {
    let options = parse_options(["--no-node"].into_iter()).expect("parse");
    assert!(!options.node_enabled);
}

#[test]
fn parse_options_rejects_reward_runtime_with_no_node() {
    let err = parse_options(["--no-node", "--reward-runtime-enable"].into_iter())
        .expect_err("reward runtime requires node");
    assert!(err.contains("--reward-runtime-enable"));
}

#[test]
fn reward_invariant_status_payload_reflects_violation_count() {
    let clean = RewardAssetInvariantReport::default();
    let clean_payload = reward_invariant_status_payload(&clean);
    assert_eq!(
        clean_payload.get("ok").and_then(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(
        clean_payload
            .get("violation_count")
            .and_then(|value| value.as_u64()),
        Some(0)
    );

    let mut violated = RewardAssetInvariantReport::default();
    violated
        .violations
        .push(agent_world::runtime::RewardAssetInvariantViolation {
            code: "mint_signature_invalid".to_string(),
            message: "tampered".to_string(),
        });
    let violated_payload = reward_invariant_status_payload(&violated);
    assert_eq!(
        violated_payload.get("ok").and_then(|value| value.as_bool()),
        Some(false)
    );
    assert_eq!(
        violated_payload
            .get("violation_count")
            .and_then(|value| value.as_u64()),
        Some(1)
    );
}

#[test]
fn parse_options_rejects_invalid_node_role() {
    let err = parse_options(["--node-role", "unknown"].into_iter()).expect_err("invalid node role");
    assert!(err.contains("--node-role"));
}

#[test]
fn parse_options_rejects_invalid_node_validator_spec() {
    let err = parse_options(["--node-validator", "missing_stake"].into_iter()).expect_err("spec");
    assert!(err.contains("--node-validator"));
}

#[test]
fn parse_options_rejects_invalid_node_gossip_addr() {
    let err = parse_options(["--node-gossip-bind", "invalid"].into_iter()).expect_err("invalid");
    assert!(err.contains("--node-gossip-bind"));
}

#[test]
fn start_live_node_applies_pos_options() {
    let options = parse_options(
        [
            "--node-id",
            "node-main",
            "--node-tick-ms",
            "20",
            "--node-validator",
            "node-main:70",
            "--node-validator",
            "node-backup:30",
            "--node-no-auto-attest-all",
            "--node-gossip-bind",
            "127.0.0.1:6101",
            "--node-gossip-peer",
            "127.0.0.1:6102",
        ]
        .into_iter(),
    )
    .expect("options");

    let runtime = start_live_node(&options)
        .expect("start")
        .expect("runtime exists");
    let mut locked = runtime.lock().expect("lock runtime");
    let config = locked.config();
    assert_eq!(config.pos_config.validators.len(), 2);
    assert_eq!(config.pos_config.validators[0].validator_id, "node-main");
    assert_eq!(config.pos_config.validators[0].stake, 70);
    assert_eq!(config.pos_config.validators[1].validator_id, "node-backup");
    assert_eq!(config.pos_config.validators[1].stake, 30);
    assert!(!config.auto_attest_all_validators);
    let gossip = config.gossip.as_ref().expect("gossip config");
    assert_eq!(
        gossip.bind_addr,
        "127.0.0.1:6101".parse::<SocketAddr>().expect("addr")
    );
    assert_eq!(gossip.peers.len(), 1);
    assert_eq!(
        gossip.peers[0],
        "127.0.0.1:6102".parse::<SocketAddr>().expect("addr")
    );

    locked.stop().expect("stop");
}

#[test]
fn start_live_node_rejects_gossip_peers_without_bind() {
    let options =
        parse_options(["--node-gossip-peer", "127.0.0.1:6202"].into_iter()).expect("options");
    let err = start_live_node(&options).expect_err("must fail");
    assert!(err.contains("--node-gossip-bind"));
}

#[test]
fn parse_options_rejects_repl_topic_without_repl_network() {
    let err = parse_options(["--node-repl-topic", "aw.topic"].into_iter())
        .expect_err("repl topic should require network");
    assert!(err.contains("--node-repl-topic"));
}

#[test]
fn start_live_node_supports_libp2p_replication_injection() {
    let options = parse_options(
        [
            "--node-repl-libp2p-listen",
            "/ip4/127.0.0.1/tcp/0",
            "--node-repl-topic",
            "aw.test.replication",
        ]
        .into_iter(),
    )
    .expect("options");

    let runtime = start_live_node(&options)
        .expect("start")
        .expect("runtime exists");
    runtime.lock().expect("lock runtime").stop().expect("stop");
}

#[test]
fn ensure_node_keypair_in_config_creates_file_when_missing() {
    let path = temp_config_path("node-key-create");
    let keypair = ensure_node_keypair_in_config(&path).expect("ensure keypair");
    assert_eq!(keypair.private_key_hex.len(), 64);
    assert_eq!(keypair.public_key_hex.len(), 64);
    assert!(path.exists());

    let content = fs::read_to_string(&path).expect("read config");
    assert!(content.contains("[node]"));
    assert!(content.contains("private_key"));
    assert!(content.contains("public_key"));

    let _ = fs::remove_file(path);
}

#[test]
fn ensure_node_keypair_in_config_preserves_existing_keypair() {
    let path = temp_config_path("node-key-preserve");
    let first = ensure_node_keypair_in_config(&path).expect("first ensure");
    let second = ensure_node_keypair_in_config(&path).expect("second ensure");
    assert_eq!(first, second);
    let _ = fs::remove_file(path);
}

#[test]
fn ensure_node_keypair_in_config_fills_missing_public_key() {
    let path = temp_config_path("node-key-fill-public");
    let generated = ensure_node_keypair_in_config(&path).expect("first ensure");

    let content = fs::read_to_string(&path).expect("read config");
    let mut value: toml::Value = toml::from_str(content.as_str()).expect("parse config");
    let node = value
        .as_table_mut()
        .and_then(|table| table.get_mut(NODE_TABLE_KEY))
        .and_then(toml::Value::as_table_mut)
        .expect("node table");
    node.remove(NODE_PUBLIC_KEY_FIELD);
    fs::write(&path, toml::to_string_pretty(&value).expect("serialize")).expect("write");

    let filled = ensure_node_keypair_in_config(&path).expect("fill public");
    assert_eq!(filled.private_key_hex, generated.private_key_hex);
    assert_eq!(filled.public_key_hex, generated.public_key_hex);
    let _ = fs::remove_file(path);
}

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
