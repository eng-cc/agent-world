use super::{
    build_chain_balances_payload_from_world, parse_options, parse_validator_spec, CliOptions,
    DEFAULT_NODE_ID, DEFAULT_STATUS_BIND,
};
use agent_world::runtime::World as RuntimeWorld;

#[test]
fn parse_options_defaults() {
    let options = parse_options(std::iter::empty()).expect("parse should succeed");
    assert_eq!(options.node_id, DEFAULT_NODE_ID);
    assert_eq!(options.status_bind, DEFAULT_STATUS_BIND);
    assert!(!options.node_auto_attest_all_validators);
    assert!(options.node_validators.is_empty());
    assert!(options.reward_runtime_enabled);
    assert!(options.reward_runtime_epoch_duration_secs.is_none());
}

#[test]
fn parse_options_reads_custom_values() {
    let options = parse_options(
        [
            "--node-id",
            "node-a",
            "--world-id",
            "live-foo",
            "--status-bind",
            "127.0.0.1:6221",
            "--node-role",
            "storage",
            "--node-tick-ms",
            "350",
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
    assert_eq!(options.status_bind, "127.0.0.1:6221");
    assert_eq!(options.node_role.as_str(), "storage");
    assert_eq!(options.node_tick_ms, 350);
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
