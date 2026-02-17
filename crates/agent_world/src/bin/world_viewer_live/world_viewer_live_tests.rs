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
    assert_eq!(report.failure_reasons.get("HASH_MISMATCH").copied(), Some(1));
    assert!(report.latest_proof_semantics.is_none());
    assert_eq!(state.cumulative_total_checks, 1);
    assert_eq!(state.cumulative_failed_checks, 1);
    assert_eq!(
        state.cumulative_failure_reasons.get("HASH_MISMATCH").copied(),
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
    };

    persist_reward_runtime_distfs_probe_state(path.as_path(), &expected)
        .expect("persist probe state");
    let loaded = load_reward_runtime_distfs_probe_state(path.as_path()).expect("load probe state");
    assert_eq!(loaded, expected);

    let _ = fs::remove_dir_all(root);
}
