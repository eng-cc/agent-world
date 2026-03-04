use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    build_game_url, build_launcher_args, parse_chain_validators, parse_host_port, parse_options,
    parse_port, validate_config, CliOptions, LauncherConfig, DEFAULT_CHAIN_STATUS_BIND,
    DEFAULT_LISTEN_BIND, DEFAULT_SCENARIO,
};

#[test]
fn parse_options_defaults() {
    let options = parse_options(std::iter::empty()).expect("parse options");
    assert_eq!(options.listen_bind, DEFAULT_LISTEN_BIND);
    assert_eq!(options.initial_config.scenario, DEFAULT_SCENARIO);
    assert_eq!(
        options.initial_config.chain_status_bind,
        DEFAULT_CHAIN_STATUS_BIND
    );
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
        ]
        .into_iter(),
    )
    .expect("parse overrides");

    assert_eq!(options.listen_bind, "127.0.0.1:7510");
    assert_eq!(options.launcher_bin, "/tmp/world_game_launcher");
    assert_eq!(options.initial_config.scenario, "sandbox");
    assert_eq!(options.initial_config.live_bind, "127.0.0.1:6200");
    assert_eq!(options.initial_config.web_bind, "127.0.0.1:6201");
    assert_eq!(options.initial_config.viewer_host, "127.0.0.1");
    assert_eq!(options.initial_config.viewer_port, "4777");
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
    assert!(args.contains(&"--no-llm".to_string()));
    assert!(args.contains(&"--no-open-browser".to_string()));
}

#[test]
fn build_launcher_args_includes_chain_overrides_when_on() {
    let config = LauncherConfig {
        viewer_static_dir: ".".to_string(),
        chain_enabled: true,
        chain_status_bind: "127.0.0.1:6121".to_string(),
        chain_node_id: "chain-a".to_string(),
        chain_world_id: "live-chain-a".to_string(),
        chain_node_role: "storage".to_string(),
        chain_node_tick_ms: "300".to_string(),
        chain_node_validators: "chain-a:55,chain-b:45".to_string(),
        ..LauncherConfig::default()
    };
    let args = build_launcher_args(&config).expect("args");
    assert!(args.contains(&"--chain-enable".to_string()));
    assert!(args.contains(&"--chain-status-bind".to_string()));
    assert!(args.contains(&"127.0.0.1:6121".to_string()));
    assert!(args.contains(&"--chain-node-validator".to_string()));
    assert!(args.contains(&"chain-a:55".to_string()));
    assert!(args.contains(&"chain-b:45".to_string()));
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
fn validate_config_reports_missing_required_fields() {
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
        chain_node_role: "invalid".to_string(),
        chain_node_tick_ms: "0".to_string(),
        chain_node_validators: "node-a".to_string(),
        ..LauncherConfig::default()
    };
    let issues = validate_config(&config);
    assert!(!issues.is_empty());
    assert!(issues.iter().any(|item| item.contains("scenario")));
    assert!(issues.iter().any(|item| item.contains("live bind")));
    assert!(issues.iter().any(|item| item.contains("viewer host")));
    assert!(issues
        .iter()
        .any(|item| item.contains("viewer static directory")));
}

#[test]
fn validate_config_accepts_minimal_valid_setup() {
    let static_dir = make_temp_dir("world_web_launcher_valid");
    let config = LauncherConfig {
        viewer_static_dir: static_dir.to_string_lossy().to_string(),
        chain_enabled: false,
        ..LauncherConfig::default()
    };
    let issues = validate_config(&config);
    assert!(issues.is_empty());
    let _ = fs::remove_dir_all(static_dir);
}

#[test]
fn cli_options_default_launcher_bin_is_not_empty() {
    let options = CliOptions::default();
    assert!(!options.launcher_bin.trim().is_empty());
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
