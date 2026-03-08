use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    build_chain_runtime_args, build_game_url, build_launcher_args, parse_chain_validators,
    parse_host_port, parse_options, parse_port, remap_transfer_runtime_target, stop_chain_process,
    stop_process, validate_chain_config, validate_game_config, ChainRuntimeStatus, CliOptions,
    LauncherConfig, ProcessState, ServiceState, DEFAULT_CHAIN_STATUS_BIND, DEFAULT_LISTEN_BIND,
    DEFAULT_SCENARIO,
};
use agent_world_proto::storage_profile::StorageProfile;

#[test]
fn parse_options_defaults() {
    let options = parse_options(std::iter::empty()).expect("parse options");
    assert_eq!(options.listen_bind, DEFAULT_LISTEN_BIND);
    assert_eq!(options.initial_config.scenario, DEFAULT_SCENARIO);
    assert_eq!(
        options.initial_config.chain_status_bind,
        DEFAULT_CHAIN_STATUS_BIND
    );
    assert_eq!(
        options.initial_config.chain_storage_profile,
        StorageProfile::DevLocal.as_str()
    );
    assert_eq!(options.initial_config.chain_pos_slot_duration_ms, "12000");
    assert_eq!(options.initial_config.chain_pos_ticks_per_slot, "10");
    assert_eq!(options.initial_config.chain_pos_proposal_tick_phase, "9");
    assert!(
        !options
            .initial_config
            .chain_pos_adaptive_tick_scheduler_enabled
    );
    assert_eq!(
        options.initial_config.chain_pos_slot_clock_genesis_unix_ms,
        ""
    );
    assert_eq!(options.initial_config.chain_pos_max_past_slot_lag, "256");
    assert!(!options.initial_config.llm_enabled);
    assert!(options.initial_config.chain_enabled);
    assert!(!options.initial_config.auto_open_browser);
}

#[test]
fn parse_options_accepts_overrides() {
    let options = parse_options(
        [
            "--listen-bind",
            "127.0.0.1:7510",
            "--launcher-bin",
            "/tmp/world_game_launcher",
            "--chain-runtime-bin",
            "/tmp/world_chain_runtime",
            "--console-static-dir",
            "/tmp/web-launcher-dist",
            "--scenario",
            "sandbox",
            "--live-bind",
            "127.0.0.1:6200",
            "--web-bind",
            "127.0.0.1:6201",
            "--viewer-host",
            "127.0.0.1",
            "--viewer-port",
            "4777",
            "--viewer-static-dir",
            "./web",
            "--with-llm",
            "--chain-disable",
            "--open-browser",
            "--chain-storage-profile",
            "release_default",
            "--chain-pos-slot-duration-ms",
            "12000",
            "--chain-pos-ticks-per-slot",
            "10",
            "--chain-pos-proposal-tick-phase",
            "9",
            "--chain-pos-adaptive-tick-scheduler",
            "--chain-pos-slot-clock-genesis-unix-ms",
            "1700000000000",
            "--chain-pos-max-past-slot-lag",
            "32",
        ]
        .into_iter(),
    )
    .expect("parse overrides");

    assert_eq!(options.listen_bind, "127.0.0.1:7510");
    assert_eq!(options.launcher_bin, "/tmp/world_game_launcher");
    assert_eq!(options.chain_runtime_bin, "/tmp/world_chain_runtime");
    assert_eq!(
        options.console_static_dir,
        PathBuf::from("/tmp/web-launcher-dist")
    );
    assert_eq!(options.initial_config.scenario, "sandbox");
    assert_eq!(options.initial_config.live_bind, "127.0.0.1:6200");
    assert_eq!(options.initial_config.web_bind, "127.0.0.1:6201");
    assert_eq!(options.initial_config.viewer_host, "127.0.0.1");
    assert_eq!(options.initial_config.viewer_port, "4777");
    assert_eq!(
        options.initial_config.launcher_bin,
        "/tmp/world_game_launcher"
    );
    assert_eq!(
        options.initial_config.chain_runtime_bin,
        "/tmp/world_chain_runtime"
    );
    assert_eq!(
        options.initial_config.chain_storage_profile,
        "release_default"
    );
    assert_eq!(options.initial_config.chain_pos_slot_duration_ms, "12000");
    assert_eq!(options.initial_config.chain_pos_ticks_per_slot, "10");
    assert_eq!(options.initial_config.chain_pos_proposal_tick_phase, "9");
    assert!(
        options
            .initial_config
            .chain_pos_adaptive_tick_scheduler_enabled
    );
    assert_eq!(
        options.initial_config.chain_pos_slot_clock_genesis_unix_ms,
        "1700000000000"
    );
    assert_eq!(options.initial_config.chain_pos_max_past_slot_lag, "32");
    assert!(options.initial_config.llm_enabled);
    assert!(!options.initial_config.chain_enabled);
    assert!(options.initial_config.auto_open_browser);
}

#[test]
fn parse_options_collects_repeat_validators() {
    let options = parse_options(
        [
            "--chain-node-validator",
            "node-a:40",
            "--chain-node-validator",
            "node-b:60",
        ]
        .into_iter(),
    )
    .expect("parse validators");

    assert_eq!(
        options.initial_config.chain_node_validators,
        "node-a:40,node-b:60"
    );
}

#[test]
fn parse_options_rejects_unknown_option() {
    let err = parse_options(["--unknown"].into_iter()).expect_err("unknown option should fail");
    assert!(err.contains("unknown option"));
}

#[test]
fn parse_options_rejects_out_of_range_chain_pos_proposal_tick_phase() {
    let err = parse_options(
        [
            "--chain-pos-ticks-per-slot",
            "4",
            "--chain-pos-proposal-tick-phase",
            "4",
        ]
        .into_iter(),
    )
    .expect_err("out-of-range proposal tick phase should fail");
    assert!(err.contains("--chain-pos-proposal-tick-phase"));
}

#[test]
fn parse_port_rejects_zero() {
    let err = parse_port("0", "viewer port").expect_err("zero port should fail");
    assert!(err.contains("1..=65535"));
}

#[test]
fn parse_host_port_accepts_ipv6() {
    let (host, port) = parse_host_port("[::1]:5011", "--web-bind").expect("ipv6 host:port");
    assert_eq!(host, "::1");
    assert_eq!(port, 5011);
}

#[test]
fn parse_host_port_rejects_unbracketed_ipv6() {
    let err = parse_host_port("::1:5011", "--web-bind").expect_err("should fail");
    assert!(err.contains("wrapped in []"));
}

#[test]
fn parse_chain_validators_rejects_invalid_format() {
    let err = parse_chain_validators("node-a").expect_err("should fail");
    assert!(err.contains("validator_id:stake"));
}

#[test]
fn build_launcher_args_includes_chain_disable_when_off() {
    let config = LauncherConfig {
        chain_enabled: false,
        viewer_static_dir: ".".to_string(),
        ..LauncherConfig::default()
    };
    let args = build_launcher_args(&config).expect("args");
    assert!(args.contains(&"--chain-disable".to_string()));
    assert!(!args.contains(&"--no-llm".to_string()));
    assert!(!args.contains(&"--with-llm".to_string()));
    assert!(args.contains(&"--no-open-browser".to_string()));
}

#[test]
fn build_launcher_args_keeps_chain_disabled_even_when_chain_config_is_on() {
    let config = LauncherConfig {
        viewer_static_dir: ".".to_string(),
        chain_enabled: true,
        chain_status_bind: "127.0.0.1:6121".to_string(),
        chain_node_id: "chain-a".to_string(),
        chain_storage_profile: "soak_forensics".to_string(),
        chain_world_id: "live-chain-a".to_string(),
        chain_node_role: "storage".to_string(),
        chain_node_tick_ms: "300".to_string(),
        chain_pos_slot_duration_ms: "12000".to_string(),
        chain_pos_ticks_per_slot: "10".to_string(),
        chain_pos_proposal_tick_phase: "9".to_string(),
        chain_pos_adaptive_tick_scheduler_enabled: true,
        chain_pos_slot_clock_genesis_unix_ms: "1700000000000".to_string(),
        chain_pos_max_past_slot_lag: "32".to_string(),
        chain_node_validators: "chain-a:55,chain-b:45".to_string(),
        ..LauncherConfig::default()
    };
    let args = build_launcher_args(&config).expect("args");
    assert!(args.contains(&"--chain-disable".to_string()));
    assert!(!args.contains(&"--chain-enable".to_string()));
}

#[test]
fn build_chain_runtime_args_includes_chain_overrides_when_on() {
    let config = LauncherConfig {
        viewer_static_dir: ".".to_string(),
        chain_enabled: true,
        chain_status_bind: "127.0.0.1:6121".to_string(),
        chain_node_id: "chain-a".to_string(),
        chain_storage_profile: "soak_forensics".to_string(),
        chain_world_id: "live-chain-a".to_string(),
        chain_node_role: "storage".to_string(),
        chain_node_tick_ms: "300".to_string(),
        chain_pos_slot_duration_ms: "12000".to_string(),
        chain_pos_ticks_per_slot: "10".to_string(),
        chain_pos_proposal_tick_phase: "9".to_string(),
        chain_pos_adaptive_tick_scheduler_enabled: true,
        chain_pos_slot_clock_genesis_unix_ms: "1700000000000".to_string(),
        chain_pos_max_past_slot_lag: "32".to_string(),
        chain_node_validators: "chain-a:55,chain-b:45".to_string(),
        ..LauncherConfig::default()
    };
    let args = build_chain_runtime_args(&config).expect("args");
    assert!(args.contains(&"--status-bind".to_string()));
    assert!(args.contains(&"127.0.0.1:6121".to_string()));
    assert!(args.contains(&"--node-id".to_string()));
    assert!(args.contains(&"--storage-profile".to_string()));
    assert!(args.contains(&"soak_forensics".to_string()));
    assert!(args.contains(&"chain-a".to_string()));
    assert!(args.contains(&"--node-validator".to_string()));
    assert!(args.contains(&"chain-a:55".to_string()));
    assert!(args.contains(&"chain-b:45".to_string()));
    assert!(args.contains(&"--pos-slot-duration-ms".to_string()));
    assert!(args.contains(&"12000".to_string()));
    assert!(args.contains(&"--pos-ticks-per-slot".to_string()));
    assert!(args.contains(&"10".to_string()));
    assert!(args.contains(&"--pos-proposal-tick-phase".to_string()));
    assert!(args.contains(&"9".to_string()));
    assert!(args.contains(&"--pos-adaptive-tick-scheduler".to_string()));
    assert!(args.contains(&"--pos-slot-clock-genesis-unix-ms".to_string()));
    assert!(args.contains(&"1700000000000".to_string()));
    assert!(args.contains(&"--pos-max-past-slot-lag".to_string()));
    assert!(args.contains(&"32".to_string()));
}

#[test]
fn build_game_url_uses_request_host_for_wildcard_bindings() {
    let config = LauncherConfig {
        viewer_host: "0.0.0.0".to_string(),
        viewer_port: "4173".to_string(),
        web_bind: "0.0.0.0:5011".to_string(),
        ..LauncherConfig::default()
    };
    let url = build_game_url(&config, Some("10.10.1.8"));
    assert_eq!(url, "http://10.10.1.8:4173/?ws=ws://10.10.1.8:5011");
}

#[test]
fn validate_game_config_reports_missing_required_fields() {
    let config = LauncherConfig {
        scenario: "".to_string(),
        live_bind: "127.0.0.1".to_string(),
        web_bind: "127.0.0.1".to_string(),
        viewer_host: "".to_string(),
        viewer_port: "0".to_string(),
        viewer_static_dir: "/missing/dir".to_string(),
        chain_enabled: true,
        chain_status_bind: "127.0.0.1".to_string(),
        chain_node_id: "".to_string(),
        chain_storage_profile: "invalid".to_string(),
        chain_node_role: "invalid".to_string(),
        chain_node_tick_ms: "0".to_string(),
        chain_pos_slot_duration_ms: "0".to_string(),
        chain_pos_ticks_per_slot: "0".to_string(),
        chain_pos_proposal_tick_phase: "x".to_string(),
        chain_pos_slot_clock_genesis_unix_ms: "oops".to_string(),
        chain_pos_max_past_slot_lag: "-1".to_string(),
        chain_node_validators: "node-a".to_string(),
        ..LauncherConfig::default()
    };
    let issues = validate_game_config(&config);
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|item| item.contains("scenario")));
    assert!(issues.iter().any(|item| item.contains("live bind")));
    assert!(issues.iter().any(|item| item.contains("viewer host")));
    assert!(issues
        .iter()
        .any(|item| item.contains("viewer static directory")));
}

#[test]
fn validate_game_config_accepts_minimal_valid_setup() {
    let static_dir = make_temp_dir("world_web_launcher_valid");
    let config = LauncherConfig {
        viewer_static_dir: static_dir.to_string_lossy().to_string(),
        chain_enabled: false,
        ..LauncherConfig::default()
    };
    let issues = validate_game_config(&config);
    assert!(issues.is_empty());
    let _ = fs::remove_dir_all(static_dir);
}

#[test]
fn validate_chain_config_reports_missing_required_fields() {
    let config = LauncherConfig {
        chain_enabled: true,
        chain_status_bind: "127.0.0.1".to_string(),
        chain_node_id: "".to_string(),
        chain_storage_profile: "invalid".to_string(),
        chain_node_role: "invalid".to_string(),
        chain_node_tick_ms: "0".to_string(),
        chain_pos_slot_duration_ms: "0".to_string(),
        chain_pos_ticks_per_slot: "4".to_string(),
        chain_pos_proposal_tick_phase: "4".to_string(),
        chain_pos_slot_clock_genesis_unix_ms: "oops".to_string(),
        chain_pos_max_past_slot_lag: "-1".to_string(),
        chain_node_validators: "node-a".to_string(),
        ..LauncherConfig::default()
    };
    let issues = validate_chain_config(&config);
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|item| item.contains("chain status bind")));
    assert!(issues.iter().any(|item| item.contains("chain node id")));
    assert!(issues
        .iter()
        .any(|item| item.contains("chain storage profile")));
    assert!(issues.iter().any(|item| item.contains("chain pos")));
}

#[test]
fn cli_options_default_launcher_bin_is_not_empty() {
    let options = CliOptions::default();
    assert!(!options.launcher_bin.trim().is_empty());
    assert!(!options.chain_runtime_bin.trim().is_empty());
}

#[test]
fn remap_transfer_runtime_target_preserves_query_parameters() {
    let mapped = remap_transfer_runtime_target(
        "/api/chain/explorer/transactions?status=confirmed&limit=50",
        "/api/chain/explorer/transactions",
        "/v1/chain/explorer/transactions",
    );
    assert_eq!(
        mapped,
        "/v1/chain/explorer/transactions?status=confirmed&limit=50"
    );
}

#[test]
fn remap_transfer_runtime_target_supports_explorer_blocks_pagination() {
    let mapped = remap_transfer_runtime_target(
        "/api/chain/explorer/blocks?cursor=50&limit=25",
        "/api/chain/explorer/blocks",
        "/v1/chain/explorer/blocks",
    );
    assert_eq!(mapped, "/v1/chain/explorer/blocks?cursor=50&limit=25");
}

#[test]
fn remap_transfer_runtime_target_supports_explorer_p1_address_query() {
    let mapped = remap_transfer_runtime_target(
        "/api/chain/explorer/address?account_id=player:alice&limit=20",
        "/api/chain/explorer/address",
        "/v1/chain/explorer/address",
    );
    assert_eq!(
        mapped,
        "/v1/chain/explorer/address?account_id=player:alice&limit=20"
    );
}

#[test]
fn stop_process_noop_preserves_error_state() {
    let config = LauncherConfig {
        chain_enabled: false,
        ..LauncherConfig::default()
    };
    let mut state = ServiceState::new(
        "launcher".to_string(),
        "chain".to_string(),
        PathBuf::from("."),
        config,
    );
    state.process_state = ProcessState::StartFailed("boot failed".to_string());

    stop_process(&mut state).expect("stop no-op should succeed");

    assert!(matches!(
        state.process_state,
        ProcessState::StartFailed(ref detail) if detail == "boot failed"
    ));
}

#[test]
fn stop_chain_process_noop_preserves_error_state() {
    let config = LauncherConfig {
        chain_enabled: true,
        ..LauncherConfig::default()
    };
    let mut state = ServiceState::new(
        "launcher".to_string(),
        "chain".to_string(),
        PathBuf::from("."),
        config,
    );
    state.chain_runtime_status = ChainRuntimeStatus::Unreachable("probe failed".to_string());

    stop_chain_process(&mut state).expect("chain stop no-op should succeed");

    assert!(matches!(
        state.chain_runtime_status,
        ChainRuntimeStatus::Unreachable(ref detail) if detail == "probe failed"
    ));
}

fn make_temp_dir(label: &str) -> PathBuf {
    let mut path = env::temp_dir();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    path.push(format!(
        "agent_world_world_web_launcher_test_{label}_{}_{}",
        std::process::id(),
        stamp
    ));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}
