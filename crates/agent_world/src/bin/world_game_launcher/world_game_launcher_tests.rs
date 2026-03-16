use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{
    build_game_url, build_viewer_auth_bootstrap_script, build_world_chain_runtime_args,
    content_type_for_path, parse_host_port, parse_options, resolve_static_asset_path,
    resolve_viewer_auth_bootstrap_from_path, resolve_viewer_static_dir_with_override,
    sanitize_index_html_for_embedded_server, sanitize_relative_request_path, CliOptions,
    ViewerAuthBootstrap, BUILTIN_LLM_PROVIDER_MODE, DEFAULT_CHAIN_NODE_ID,
    DEFAULT_CHAIN_STATUS_BIND, DEFAULT_LIVE_BIND, DEFAULT_OPENCLAW_AGENT_PROFILE, DEFAULT_SCENARIO,
    DEFAULT_VIEWER_STATIC_DIR, OPENCLAW_LOCAL_HTTP_PROVIDER_MODE, VIEWER_AUTH_BOOTSTRAP_OBJECT,
    VIEWER_AUTH_PRIVATE_KEY_ENV, VIEWER_AUTH_PUBLIC_KEY_ENV, VIEWER_PLAYER_ID_ENV,
};
use agent_world::simulator::ProviderExecutionMode;
use agent_world_proto::storage_profile::StorageProfile;

#[test]
fn parse_options_defaults() {
    let options = parse_options(std::iter::empty()).expect("parse should succeed");
    assert_eq!(options.scenario, DEFAULT_SCENARIO);
    assert_eq!(options.live_bind, DEFAULT_LIVE_BIND);
    assert!(!options.with_llm);
    assert_eq!(options.agent_provider_mode, BUILTIN_LLM_PROVIDER_MODE);
    assert_eq!(
        options.openclaw_agent_profile,
        DEFAULT_OPENCLAW_AGENT_PROFILE
    );
    assert_eq!(
        options.openclaw_execution_mode,
        ProviderExecutionMode::HeadlessAgent
    );
    assert!(options.open_browser);
    assert_eq!(options.viewer_static_dir, "web");
    assert!(options.chain_enabled);
    assert_eq!(options.chain_status_bind, DEFAULT_CHAIN_STATUS_BIND);
    assert!(options
        .chain_node_id
        .starts_with(&format!("{DEFAULT_CHAIN_NODE_ID}-fresh-")));
    assert_eq!(options.chain_storage_profile, StorageProfile::DevLocal);
    assert_eq!(options.chain_node_role, "sequencer");
    assert_eq!(options.chain_pos_slot_duration_ms, 12_000);
    assert_eq!(options.chain_pos_ticks_per_slot, 10);
    assert_eq!(options.chain_pos_proposal_tick_phase, 9);
    assert!(!options.chain_pos_adaptive_tick_scheduler_enabled);
    assert_eq!(options.chain_pos_slot_clock_genesis_unix_ms, None);
    assert_eq!(options.chain_pos_max_past_slot_lag, 256);
}

#[test]
fn parse_options_accepts_overrides() {
    let options = parse_options(
        [
            "--scenario",
            "twin_region_bootstrap",
            "--live-bind",
            "127.0.0.1:6200",
            "--web-bind",
            "127.0.0.1:6201",
            "--viewer-host",
            "0.0.0.0",
            "--viewer-port",
            "4777",
            "--viewer-static-dir",
            "dist",
            "--chain-status-bind",
            "127.0.0.1:6331",
            "--chain-node-id",
            "chain-a",
            "--chain-storage-profile",
            "soak_forensics",
            "--chain-world-id",
            "live-chain-a",
            "--chain-node-role",
            "storage",
            "--chain-node-tick-ms",
            "350",
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
            "--chain-node-validator",
            "chain-a:55",
            "--with-llm",
            "--agent-provider-mode",
            "openclaw_local_http",
            "--openclaw-base-url",
            "http://127.0.0.1:5841",
            "--openclaw-auth-token",
            "secret-token",
            "--openclaw-connect-timeout-ms",
            "3000",
            "--openclaw-agent-profile",
            "agent_world_p0_low_freq_npc",
            "--openclaw-execution-mode",
            "player_parity",
            "--no-open-browser",
        ]
        .into_iter(),
    )
    .expect("parse should succeed");

    assert_eq!(options.scenario, "twin_region_bootstrap");
    assert_eq!(options.live_bind, "127.0.0.1:6200");
    assert_eq!(options.web_bind, "127.0.0.1:6201");
    assert_eq!(options.viewer_host, "0.0.0.0");
    assert_eq!(options.viewer_port, 4777);
    assert_eq!(options.viewer_static_dir, "dist");
    assert_eq!(options.chain_status_bind, "127.0.0.1:6331");
    assert_eq!(options.chain_node_id, "chain-a");
    assert_eq!(options.chain_storage_profile, StorageProfile::SoakForensics);
    assert_eq!(options.chain_world_id, Some("live-chain-a".to_string()));
    assert_eq!(options.chain_node_role, "storage");
    assert_eq!(options.chain_node_tick_ms, 350);
    assert_eq!(options.chain_pos_slot_duration_ms, 12_000);
    assert_eq!(options.chain_pos_ticks_per_slot, 10);
    assert_eq!(options.chain_pos_proposal_tick_phase, 9);
    assert!(options.chain_pos_adaptive_tick_scheduler_enabled);
    assert_eq!(
        options.chain_pos_slot_clock_genesis_unix_ms,
        Some(1_700_000_000_000)
    );
    assert_eq!(options.chain_pos_max_past_slot_lag, 32);
    assert_eq!(
        options.chain_node_validators,
        vec!["chain-a:55".to_string()]
    );
    assert!(options.with_llm);
    assert_eq!(
        options.agent_provider_mode,
        OPENCLAW_LOCAL_HTTP_PROVIDER_MODE
    );
    assert_eq!(options.openclaw_base_url, "http://127.0.0.1:5841");
    assert_eq!(options.openclaw_auth_token, "secret-token");
    assert_eq!(options.openclaw_connect_timeout_ms, 3000);
    assert_eq!(
        options.openclaw_agent_profile,
        "agent_world_p0_low_freq_npc"
    );
    assert_eq!(
        options.openclaw_execution_mode,
        ProviderExecutionMode::PlayerParity
    );
    assert!(!options.open_browser);
}

#[test]
fn parse_options_accepts_chain_disable() {
    let options = parse_options(["--chain-disable"].into_iter()).expect("parse should succeed");
    assert!(!options.chain_enabled);
}

#[test]
fn parse_options_rejects_invalid_chain_role() {
    let err = parse_options(["--chain-node-role", "invalid"].into_iter()).expect_err("should fail");
    assert!(err.contains("sequencer, storage, observer"));
}

#[test]
fn parse_options_rejects_proposal_tick_phase_out_of_range() {
    let err = parse_options(
        [
            "--chain-pos-ticks-per-slot",
            "4",
            "--chain-pos-proposal-tick-phase",
            "4",
        ]
        .into_iter(),
    )
    .expect_err("should fail");
    assert!(err.contains("--chain-pos-proposal-tick-phase"));
}

#[test]
fn parse_options_rejects_unknown_option() {
    let err = parse_options(["--unknown"].into_iter()).expect_err("should fail");
    assert!(err.contains("unknown option"));
}

#[test]
fn parse_options_rejects_unknown_agent_provider_mode() {
    let err = parse_options(["--agent-provider-mode", "wat-provider"].into_iter())
        .expect_err("should fail");
    assert!(err.contains("builtin_llm"));
    assert!(err.contains("openclaw_local_http"));
}

#[test]
fn parse_options_rejects_invalid_openclaw_execution_mode() {
    let err = parse_options(
        [
            "--with-llm",
            "--agent-provider-mode",
            "openclaw_local_http",
            "--openclaw-execution-mode",
            "gpu_only",
        ]
        .into_iter(),
    )
    .expect_err("should fail");
    assert!(err.contains("player_parity"));
    assert!(err.contains("headless_agent"));
}

#[test]
fn parse_options_rejects_unknown_chain_storage_profile() {
    let err =
        parse_options(["--chain-storage-profile", "unknown"].into_iter()).expect_err("should fail");
    assert!(err.contains("dev_local"));
    assert!(err.contains("release_default"));
    assert!(err.contains("soak_forensics"));
}

#[test]
fn parse_options_rejects_missing_value() {
    let err = parse_options(["--viewer-port"].into_iter()).expect_err("should fail");
    assert!(err.contains("requires a value"));
}

#[test]
fn parse_options_rejects_invalid_port() {
    let err = parse_options(["--viewer-port", "70000"].into_iter()).expect_err("should fail");
    assert!(err.contains("integer"));
}

#[test]
fn parse_options_rejects_invalid_bind_format() {
    let err = parse_options(["--live-bind", "127.0.0.1"].into_iter()).expect_err("should fail");
    assert!(err.contains("<host:port>"));
}

#[test]
fn parse_host_port_parses_valid_value() {
    let (host, port) = parse_host_port("127.0.0.1:5011", "--web-bind").expect("ok");
    assert_eq!(host, "127.0.0.1");
    assert_eq!(port, 5011);
}

#[test]
fn parse_host_port_accepts_bracketed_ipv6() {
    let (host, port) = parse_host_port("[::1]:5011", "--web-bind").expect("ok");
    assert_eq!(host, "::1");
    assert_eq!(port, 5011);
}

#[test]
fn parse_host_port_rejects_unbracketed_ipv6() {
    let err = parse_host_port("::1:5011", "--web-bind").expect_err("should fail");
    assert!(err.contains("wrapped in []"));
}

#[test]
fn parse_host_port_rejects_zero_port() {
    let err = parse_host_port("127.0.0.1:0", "--web-bind").expect_err("should fail");
    assert!(err.contains("1..=65535"));
}

#[test]
fn build_game_url_rewrites_zero_bind_host_to_loopback() {
    let options = CliOptions {
        viewer_host: "0.0.0.0".to_string(),
        viewer_port: 4173,
        web_bind: "0.0.0.0:5011".to_string(),
        ..CliOptions::default()
    };
    let url = build_game_url(&options);
    assert_eq!(url, "http://127.0.0.1:4173/?ws=ws://127.0.0.1:5011");
}

#[test]
fn build_game_url_brackets_ipv6_hosts() {
    let options = CliOptions {
        viewer_host: "::1".to_string(),
        viewer_port: 4173,
        web_bind: "[::1]:5011".to_string(),
        ..CliOptions::default()
    };
    let url = build_game_url(&options);
    assert_eq!(url, "http://[::1]:4173/?ws=ws://[::1]:5011");
}

#[test]
fn build_world_chain_runtime_args_includes_storage_profile() {
    let options = CliOptions {
        scenario: "sandbox".to_string(),
        chain_node_id: "chain-a".to_string(),
        chain_status_bind: "127.0.0.1:6121".to_string(),
        chain_storage_profile: StorageProfile::ReleaseDefault,
        ..CliOptions::default()
    };
    let args = build_world_chain_runtime_args(&options);
    assert!(args.contains(&"--storage-profile".to_string()));
    assert!(args.contains(&"release_default".to_string()));
    assert!(args.contains(&"--world-id".to_string()));
    assert!(args.contains(&"live-sandbox".to_string()));
    assert!(args.contains(&"--execution-world-dir".to_string()));
    assert!(
        args.contains(&"output/chain-runtime/chain-a/reward-runtime-execution-world".to_string())
    );
}

#[test]
fn build_world_chain_runtime_args_supports_all_storage_profiles() {
    for (profile, expected) in [
        (StorageProfile::DevLocal, "dev_local"),
        (StorageProfile::ReleaseDefault, "release_default"),
        (StorageProfile::SoakForensics, "soak_forensics"),
    ] {
        let options = CliOptions {
            scenario: "sandbox".to_string(),
            chain_node_id: format!("chain-{expected}"),
            chain_status_bind: "127.0.0.1:6121".to_string(),
            chain_storage_profile: profile,
            ..CliOptions::default()
        };
        let args = build_world_chain_runtime_args(&options);
        assert!(args.contains(&"--storage-profile".to_string()));
        assert!(args.contains(&expected.to_string()));
    }
}

#[test]
fn sanitize_relative_request_path_rejects_traversal() {
    let err = sanitize_relative_request_path("/../etc/passwd").expect_err("should fail");
    assert!(err.contains("traversal"));
}

#[test]
fn resolve_static_asset_path_supports_spa_fallback() {
    let temp_dir = make_temp_dir("spa_fallback");
    fs::write(temp_dir.join("index.html"), "<html>ok</html>").expect("write index");
    let resolved = resolve_static_asset_path(temp_dir.as_path(), "/app/route?x=1")
        .expect("resolve should succeed")
        .expect("should fallback to index");
    assert_eq!(resolved, temp_dir.join("index.html"));
    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn resolve_static_asset_path_returns_none_for_missing_static_asset() {
    let temp_dir = make_temp_dir("missing_asset");
    fs::write(temp_dir.join("index.html"), "<html>ok</html>").expect("write index");
    let resolved = resolve_static_asset_path(temp_dir.as_path(), "/assets/missing.js")
        .expect("resolve should succeed");
    assert!(resolved.is_none());
    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn content_type_for_path_covers_wasm_and_js() {
    assert_eq!(
        content_type_for_path(Path::new("a.wasm")),
        "application/wasm"
    );
    assert_eq!(
        content_type_for_path(Path::new("a.js")),
        "text/javascript; charset=utf-8"
    );
}

#[test]
fn sanitize_index_html_for_embedded_server_removes_trunk_reload_script() {
    let html = concat!(
        "<html><body>",
        "<script>window.bootstrap = true;</script>",
        "<script>const url = 'ws://{{__TRUNK_ADDRESS__}}{{__TRUNK_WS_BASE__}}.well-known/trunk/ws';</script>",
        "</body></html>"
    );
    let sanitized =
        sanitize_index_html_for_embedded_server(Path::new("index.html"), html.as_bytes(), None);
    let sanitized = String::from_utf8(sanitized).expect("utf-8");
    assert!(sanitized.contains("window.bootstrap = true"));
    assert!(!sanitized.contains(".well-known/trunk/ws"));
    assert!(!sanitized.contains("__TRUNK_ADDRESS__"));
}

#[test]
fn sanitize_index_html_for_embedded_server_keeps_non_index_files_unchanged() {
    let body = b"<script>.well-known/trunk/ws</script>";
    let sanitized = sanitize_index_html_for_embedded_server(Path::new("app.js"), body, None);
    assert_eq!(sanitized, body);
}

#[test]
fn sanitize_index_html_for_embedded_server_injects_viewer_auth_bootstrap() {
    let html = "<html><head></head><body><div id=\"app\"></div></body></html>";
    let auth = ViewerAuthBootstrap {
        player_id: "viewer-player".to_string(),
        public_key: "pub-hex".to_string(),
        private_key: "priv-hex".to_string(),
    };
    let sanitized = sanitize_index_html_for_embedded_server(
        Path::new("index.html"),
        html.as_bytes(),
        Some(&auth),
    );
    let sanitized = String::from_utf8(sanitized).expect("utf-8");
    assert!(sanitized.contains(VIEWER_AUTH_BOOTSTRAP_OBJECT));
    assert!(sanitized.contains(VIEWER_PLAYER_ID_ENV));
    assert!(sanitized.contains(VIEWER_AUTH_PUBLIC_KEY_ENV));
    assert!(sanitized.contains(VIEWER_AUTH_PRIVATE_KEY_ENV));
    assert!(sanitized.contains("viewer-player"));
    assert!(sanitized.contains("pub-hex"));
    assert!(sanitized.contains("priv-hex"));
}

#[test]
fn sanitize_index_html_for_embedded_server_injects_viewer_auth_bootstrap_into_non_index_html() {
    let html = "<html><head></head><body><div id=\"safe\"></div></body></html>";
    let auth = ViewerAuthBootstrap {
        player_id: "viewer-player".to_string(),
        public_key: "pub-hex".to_string(),
        private_key: "priv-hex".to_string(),
    };
    let sanitized = sanitize_index_html_for_embedded_server(
        Path::new("software_safe.html"),
        html.as_bytes(),
        Some(&auth),
    );
    let sanitized = String::from_utf8(sanitized).expect("utf-8");
    assert!(sanitized.contains(VIEWER_AUTH_BOOTSTRAP_OBJECT));
    assert!(sanitized.contains("viewer-player"));
    assert!(sanitized.contains("pub-hex"));
    assert!(sanitized.contains("priv-hex"));
}

#[test]
fn build_viewer_auth_bootstrap_script_contains_expected_window_object() {
    let auth = ViewerAuthBootstrap {
        player_id: "viewer-player".to_string(),
        public_key: "public".to_string(),
        private_key: "private".to_string(),
    };
    let script = build_viewer_auth_bootstrap_script(&auth);
    assert!(script.contains("window."));
    assert!(script.contains(VIEWER_AUTH_BOOTSTRAP_OBJECT));
    assert!(script.contains(VIEWER_PLAYER_ID_ENV));
    assert!(script.contains(VIEWER_AUTH_PUBLIC_KEY_ENV));
    assert!(script.contains(VIEWER_AUTH_PRIVATE_KEY_ENV));
}

#[test]
fn resolve_viewer_auth_bootstrap_from_path_reads_node_keypair() {
    let temp_dir = make_temp_dir("viewer_auth_bootstrap");
    let config_path = temp_dir.join("config.toml");
    fs::write(
        &config_path,
        "[node]\nprivate_key = \"private-key-hex\"\npublic_key = \"public-key-hex\"\n",
    )
    .expect("write config");

    let auth =
        resolve_viewer_auth_bootstrap_from_path(config_path.as_path()).expect("resolve auth");
    assert_eq!(auth.public_key, "public-key-hex");
    assert_eq!(auth.private_key, "private-key-hex");
    assert!(!auth.player_id.trim().is_empty());
    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn resolve_viewer_static_dir_with_override_prefers_env_for_default_static_dir() {
    let override_dir = make_temp_dir("viewer_static_override");
    let override_raw = override_dir.to_string_lossy().to_string();

    let resolved = resolve_viewer_static_dir_with_override(
        DEFAULT_VIEWER_STATIC_DIR,
        Some(override_raw.as_str()),
    )
    .expect("resolve should succeed");

    assert_eq!(resolved, override_dir);
    let _ = fs::remove_dir_all(override_dir);
}

#[test]
fn resolve_viewer_static_dir_with_override_keeps_explicit_path_priority() {
    let explicit_dir = make_temp_dir("viewer_static_explicit");
    let override_dir = make_temp_dir("viewer_static_override_ignored");
    let explicit_raw = explicit_dir.to_string_lossy().to_string();
    let override_raw = override_dir.to_string_lossy().to_string();

    let resolved =
        resolve_viewer_static_dir_with_override(explicit_raw.as_str(), Some(override_raw.as_str()))
            .expect("resolve should succeed");

    assert_eq!(resolved, explicit_dir);
    let _ = fs::remove_dir_all(explicit_dir);
    let _ = fs::remove_dir_all(override_dir);
}

#[test]
fn resolve_viewer_static_dir_with_override_rejects_missing_env_dir() {
    let missing_path = make_missing_temp_path("viewer_static_missing_env");
    let missing_raw = missing_path.to_string_lossy().to_string();

    let err = resolve_viewer_static_dir_with_override(
        DEFAULT_VIEWER_STATIC_DIR,
        Some(missing_raw.as_str()),
    )
    .expect_err("missing override path should fail");

    assert!(err.contains("AGENT_WORLD_GAME_STATIC_DIR"));
    assert!(err.contains("not found"));
}

fn make_temp_dir(label: &str) -> PathBuf {
    let mut path = env::temp_dir();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    path.push(format!(
        "agent_world_launcher_test_{label}_{}_{}",
        std::process::id(),
        stamp
    ));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

fn make_missing_temp_path(label: &str) -> PathBuf {
    let mut path = env::temp_dir();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    path.push(format!(
        "agent_world_launcher_missing_{label}_{}_{}",
        std::process::id(),
        stamp
    ));
    path
}
