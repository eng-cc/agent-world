use super::{
    build_chain_balances_payload_from_world, build_default_replication_network_config,
    build_node_replication_config, node_keypair_config, parse_options, parse_validator_spec,
    CliOptions, DEFAULT_NODE_ID, DEFAULT_REPLICATION_NETWORK_LISTEN, DEFAULT_STATUS_BIND,
};
use agent_world::runtime::World as RuntimeWorld;
use agent_world_node::{NodeConsensusSnapshot, NodeRole, NodeSnapshot};
use agent_world_proto::storage_profile::{StorageProfile, StorageProfileConfig};
use ed25519_dalek::SigningKey;
use std::collections::BTreeMap;
use std::path::Path;

#[test]
fn parse_options_defaults() {
    let options = parse_options(std::iter::empty()).expect("parse should succeed");
    assert_eq!(options.node_id, DEFAULT_NODE_ID);
    assert_eq!(options.status_bind, DEFAULT_STATUS_BIND);
    assert_eq!(options.storage_profile, StorageProfile::DevLocal);
    assert!(!options.node_auto_attest_all_validators);
    assert!(options.node_validators.is_empty());
    assert!(options.reward_runtime_enabled);
    assert!(options.reward_runtime_epoch_duration_secs.is_none());
    assert_eq!(options.pos_slot_duration_ms, 12_000);
    assert_eq!(options.pos_ticks_per_slot, 10);
    assert_eq!(options.pos_proposal_tick_phase, 9);
    assert!(!options.pos_adaptive_tick_scheduler_enabled);
    assert!(options.pos_slot_clock_genesis_unix_ms.is_none());
    assert_eq!(options.pos_max_past_slot_lag, 256);
}

#[test]
fn parse_options_reads_custom_values() {
    let options = parse_options(
        [
            "--node-id",
            "node-a",
            "--world-id",
            "live-foo",
            "--storage-profile",
            "soak_forensics",
            "--status-bind",
            "127.0.0.1:6221",
            "--node-role",
            "storage",
            "--node-tick-ms",
            "350",
            "--pos-slot-duration-ms",
            "12000",
            "--pos-ticks-per-slot",
            "10",
            "--pos-proposal-tick-phase",
            "9",
            "--pos-adaptive-tick-scheduler",
            "--pos-slot-clock-genesis-unix-ms",
            "1700000000000",
            "--pos-max-past-slot-lag",
            "32",
            "--node-validator",
            "node-a:55",
            "--node-validator",
            "node-b:45",
            "--node-auto-attest-all",
            "--execution-world-dir",
            "custom/world",
            "--reward-runtime-epoch-duration-secs",
            "60",
            "--reward-points-per-credit",
            "100",
            "--reward-runtime-auto-redeem",
            "--reward-initial-reserve-power-units",
            "50000",
        ]
        .into_iter(),
    )
    .expect("parse should succeed");

    assert_eq!(options.node_id, "node-a");
    assert_eq!(options.world_id, "live-foo");
    assert_eq!(options.storage_profile, StorageProfile::SoakForensics);
    assert_eq!(options.status_bind, "127.0.0.1:6221");
    assert_eq!(options.node_role.as_str(), "storage");
    assert_eq!(options.node_tick_ms, 350);
    assert_eq!(options.pos_slot_duration_ms, 12_000);
    assert_eq!(options.pos_ticks_per_slot, 10);
    assert_eq!(options.pos_proposal_tick_phase, 9);
    assert!(options.pos_adaptive_tick_scheduler_enabled);
    assert_eq!(
        options.pos_slot_clock_genesis_unix_ms,
        Some(1_700_000_000_000)
    );
    assert_eq!(options.pos_max_past_slot_lag, 32);
    assert_eq!(options.node_validators.len(), 2);
    assert!(options.node_auto_attest_all_validators);
    assert_eq!(options.reward_runtime_epoch_duration_secs, Some(60));
    assert_eq!(options.reward_points_per_credit, 100);
    assert!(options.reward_runtime_auto_redeem);
    assert_eq!(options.reward_initial_reserve_power_units, 50_000);
    assert_eq!(
        options
            .execution_world_dir
            .as_ref()
            .map(|p| p.to_string_lossy().to_string()),
        Some("custom/world".to_string())
    );
}

#[test]
fn parse_options_rejects_invalid_status_bind() {
    let err = parse_options(["--status-bind", "127.0.0.1"].into_iter())
        .expect_err("should reject invalid bind");
    assert!(err.contains("<host:port>"));
}

#[test]
fn parse_options_rejects_peer_without_bind() {
    let err = parse_options(["--node-gossip-peer", "127.0.0.1:9001"].into_iter())
        .expect_err("should reject peer without bind");
    assert!(err.contains("requires --node-gossip-bind"));
}

#[test]
fn parse_options_rejects_proposal_tick_phase_out_of_range() {
    let err = parse_options(
        [
            "--pos-ticks-per-slot",
            "4",
            "--pos-proposal-tick-phase",
            "4",
        ]
        .into_iter(),
    )
    .expect_err("proposal tick phase out of range");
    assert!(err.contains("--pos-proposal-tick-phase"));
}

#[test]
fn parse_options_rejects_unknown_storage_profile() {
    let err = parse_options(["--storage-profile", "unknown"].into_iter())
        .expect_err("invalid storage profile should fail");
    assert!(err.contains("dev_local"));
    assert!(err.contains("soak_forensics"));
}

#[test]
fn parse_validator_spec_rejects_zero_stake() {
    let err = parse_validator_spec("node-a:0").expect_err("should reject");
    assert!(err.contains("positive integer"));
}

#[test]
fn balances_payload_reports_empty_world_without_error() {
    let world = RuntimeWorld::new();
    let payload = build_chain_balances_payload_from_world(
        "node-a",
        "live-a",
        std::path::Path::new("/tmp/empty"),
        &world,
    );
    assert!(payload.ok);
    assert!(payload.load_error.is_none());
    assert_eq!(payload.node_power_credit_balance, 0);
    assert_eq!(payload.reward_mint_record_count, 0);
    assert!(payload.recent_reward_mint_records.is_empty());
}

#[test]
fn parse_options_rejects_unknown_option() {
    let err = parse_options(["--unknown"].into_iter()).expect_err("should fail");
    assert!(err.contains("unknown option"));
}

#[test]
fn default_runtime_paths_depend_on_node_id() {
    let options = CliOptions {
        node_id: "node-z".to_string(),
        ..CliOptions::default()
    };
    let paths = super::resolve_runtime_paths(&options);
    assert!(paths
        .execution_world_dir
        .to_string_lossy()
        .contains("output/chain-runtime/node-z"));
}

#[test]
fn default_replication_network_config_uses_loopback_ephemeral_listen() {
    let config = build_default_replication_network_config()
        .expect("default replication network config should build");
    assert_eq!(config.listen_addrs.len(), 1);
    assert_eq!(
        config.listen_addrs[0].to_string(),
        DEFAULT_REPLICATION_NETWORK_LISTEN
    );
    assert!(config.bootstrap_peers.is_empty());
    assert!(!config.allow_local_handler_fallback_when_no_peers);
}

#[test]
fn build_node_replication_config_uses_storage_profile_budget() {
    let signing_key = SigningKey::generate(&mut rand_core::OsRng);
    let keypair = node_keypair_config::NodeKeypairConfig {
        private_key_hex: hex::encode(signing_key.to_bytes()),
        public_key_hex: hex::encode(signing_key.verifying_key().to_bytes()),
    };
    let storage_profile = StorageProfileConfig::for_profile(StorageProfile::ReleaseDefault);
    let config = build_node_replication_config("node-a", &keypair, &storage_profile)
        .expect("replication config should build");

    assert_eq!(
        config.max_hot_commit_messages(),
        storage_profile.replication_max_hot_commit_messages
    );
}

#[test]
fn build_chain_status_payload_includes_storage_metrics() {
    let snapshot = NodeSnapshot {
        node_id: "node-a".to_string(),
        player_id: "player-a".to_string(),
        world_id: "live-a".to_string(),
        role: NodeRole::Sequencer,
        running: true,
        tick_count: 42,
        last_tick_unix_ms: Some(1_700_000_000_000),
        consensus: NodeConsensusSnapshot::default(),
        last_error: None,
    };
    let reward_runtime = super::reward_runtime_worker::RewardRuntimeMetricsSnapshot {
        enabled: true,
        metrics_available: true,
        report_dir: "/tmp/reports".to_string(),
        report_count: 2,
        latest_epoch_index: 1,
        latest_report_observed_at_unix_ms: 1_700_000_000_000,
        latest_total_distributed_points: 10,
        latest_minted_record_count: 1,
        cumulative_minted_record_count: 1,
        distfs_total_checks: 0,
        distfs_failed_checks: 0,
        distfs_failure_ratio: 0.0,
        settlement_apply_attempts_total: 0,
        settlement_apply_failures_total: 0,
        settlement_apply_failure_ratio: 0.0,
        invariant_ok: true,
        last_error: None,
    };
    let storage = super::storage_metrics::StorageMetricsSnapshot {
        storage_profile: "dev_local".to_string(),
        effective_budget: StorageProfileConfig::from(StorageProfile::DevLocal),
        bytes_by_dir: BTreeMap::from([("runtime_root".to_string(), 128)]),
        blob_counts: BTreeMap::from([("execution_store_blobs".to_string(), 2)]),
        ref_count: 5,
        pin_count: 3,
        retained_heights: vec![1, 2],
        checkpoint_count: 1,
        replay_summary: super::storage_metrics::StorageReplaySummary {
            retained_height_count: 2,
            earliest_retained_height: Some(1),
            latest_retained_height: Some(2),
            earliest_checkpoint_height: Some(2),
            latest_checkpoint_height: Some(2),
            mode: "checkpoint_plus_log".to_string(),
        },
        orphan_blob_count: 0,
        last_gc_at_ms: Some(1_700_000_000_000),
        last_gc_result: "failed".to_string(),
        last_gc_error: Some("gc failed".to_string()),
        degraded_reason: Some("storage degraded".to_string()),
    };

    let payload = super::build_chain_status_payload(
        snapshot,
        Path::new("/tmp/execution-world"),
        reward_runtime,
        storage.clone(),
    );

    assert_eq!(payload.storage.storage_profile, "dev_local");
    assert_eq!(payload.storage.ref_count, 5);
    assert_eq!(payload.storage.pin_count, 3);
    assert_eq!(payload.storage.checkpoint_count, 1);
    assert_eq!(
        payload.storage.effective_budget.profile,
        StorageProfile::DevLocal
    );
    assert_eq!(payload.storage.replay_summary.mode, "checkpoint_plus_log");
    assert_eq!(
        payload.storage.replay_summary.latest_retained_height,
        Some(2)
    );
    assert_eq!(payload.storage.last_gc_result, "failed");
    assert_eq!(payload.storage.last_gc_error.as_deref(), Some("gc failed"));
    assert_eq!(
        payload.storage.degraded_reason.as_deref(),
        Some("storage degraded")
    );
}
