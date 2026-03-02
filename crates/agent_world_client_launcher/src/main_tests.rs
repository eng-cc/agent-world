use super::{
    build_game_url, build_launcher_args, collect_required_config_issues, install_cjk_font,
    normalize_host_for_url, parse_chain_role, parse_chain_validators, parse_host_port, parse_port,
    ConfigIssue, LaunchConfig, LauncherStatus, UiLanguage, EGUI_CJK_FONT_NAME,
};
use eframe::egui;
#[test]
fn parse_port_rejects_zero() {
    let err = parse_port("0", "viewer port").expect_err("should fail");
    assert!(err.contains("1..=65535"));
}
#[test]
fn parse_host_port_requires_colon() {
    let err = parse_host_port("127.0.0.1", "web bind").expect_err("should fail");
    assert!(err.contains("<host:port>"));
}
#[test]
fn parse_host_port_accepts_bracketed_ipv6() {
    let (host, port) = parse_host_port("[::1]:5011", "web bind").expect("ok");
    assert_eq!(host, "::1");
    assert_eq!(port, 5011);
}
#[test]
fn parse_host_port_rejects_unbracketed_ipv6() {
    let err = parse_host_port("::1:5011", "web bind").expect_err("should fail");
    assert!(err.contains("wrapped in []"));
}
#[test]
fn build_launcher_args_contains_llm_and_no_open_switches() {
    let config = LaunchConfig {
        llm_enabled: true,
        auto_open_browser: false,
        chain_enabled: false,
        ..LaunchConfig::default()
    };
    let args = build_launcher_args(&config).expect("args should build");
    assert!(args.contains(&"--with-llm".to_string()));
    assert!(args.contains(&"--no-open-browser".to_string()));
    assert!(args.contains(&"--viewer-static-dir".to_string()));
    assert!(args.contains(&"--chain-disable".to_string()));
}
#[test]
fn build_launcher_args_rejects_empty_static_dir() {
    let config = LaunchConfig {
        viewer_static_dir: "".to_string(),
        ..LaunchConfig::default()
    };
    let err = build_launcher_args(&config).expect_err("should fail");
    assert!(err.contains("static dir"));
}
#[test]
fn build_game_url_rewrites_zero_host() {
    let config = LaunchConfig {
        viewer_host: "0.0.0.0".to_string(),
        viewer_port: "4173".to_string(),
        web_bind: "0.0.0.0:5011".to_string(),
        ..LaunchConfig::default()
    };
    let url = build_game_url(&config);
    assert_eq!(url, "http://127.0.0.1:4173/?ws=ws://127.0.0.1:5011");
}
#[test]
fn build_game_url_brackets_ipv6_hosts() {
    let config = LaunchConfig {
        viewer_host: "::1".to_string(),
        viewer_port: "4173".to_string(),
        web_bind: "[::1]:5011".to_string(),
        ..LaunchConfig::default()
    };
    let url = build_game_url(&config);
    assert_eq!(url, "http://[::1]:4173/?ws=ws://[::1]:5011");
}
#[test]
fn normalize_host_for_url_maps_empty_and_any() {
    assert_eq!(normalize_host_for_url("0.0.0.0"), "127.0.0.1");
    assert_eq!(normalize_host_for_url(""), "127.0.0.1");
    assert_eq!(normalize_host_for_url("192.168.0.2"), "192.168.0.2");
}
#[test]
fn launch_config_defaults_enable_llm() {
    let config = LaunchConfig::default();
    assert!(config.llm_enabled);
    assert!(config.chain_enabled);
}
#[test]
fn build_launcher_args_contains_chain_overrides_when_enabled() {
    let config = LaunchConfig {
        chain_enabled: true,
        chain_status_bind: "127.0.0.1:6121".to_string(),
        chain_node_id: "chain-node-a".to_string(),
        chain_world_id: "live-chain-a".to_string(),
        chain_node_role: "storage".to_string(),
        chain_node_tick_ms: "350".to_string(),
        chain_node_validators: "node-a:55,node-b:45".to_string(),
        ..LaunchConfig::default()
    };
    let args = build_launcher_args(&config).expect("args should build");
    assert!(args.contains(&"--chain-enable".to_string()));
    assert!(args.contains(&"--chain-status-bind".to_string()));
    assert!(args.contains(&"127.0.0.1:6121".to_string()));
    assert!(args.contains(&"--chain-node-id".to_string()));
    assert!(args.contains(&"chain-node-a".to_string()));
    assert!(args.contains(&"--chain-world-id".to_string()));
    assert!(args.contains(&"live-chain-a".to_string()));
    assert!(args.contains(&"--chain-node-role".to_string()));
    assert!(args.contains(&"storage".to_string()));
    assert!(args.contains(&"--chain-node-tick-ms".to_string()));
    assert!(args.contains(&"350".to_string()));
    assert!(args.contains(&"--chain-node-validator".to_string()));
    assert!(args.contains(&"node-a:55".to_string()));
    assert!(args.contains(&"node-b:45".to_string()));
}
#[test]
fn parse_chain_role_rejects_invalid_value() {
    let err = parse_chain_role("invalid").expect_err("should fail");
    assert!(err.contains("sequencer|storage|observer"));
}
#[test]
fn parse_chain_validators_rejects_invalid_format() {
    let err = parse_chain_validators("node-a").expect_err("should fail");
    assert!(err.contains("<validator_id:stake>"));
}
#[test]
fn install_cjk_font_registers_font_and_priority() {
    let mut fonts = egui::FontDefinitions::default();
    install_cjk_font(
        &mut fonts,
        EGUI_CJK_FONT_NAME.to_string(),
        egui::FontData::from_static(&[0u8, 1u8]),
    );

    assert!(fonts.font_data.contains_key(EGUI_CJK_FONT_NAME));

    let proportional = fonts
        .families
        .get(&egui::FontFamily::Proportional)
        .expect("proportional family");
    assert_eq!(
        proportional.first().map(String::as_str),
        Some(EGUI_CJK_FONT_NAME)
    );

    let monospace = fonts
        .families
        .get(&egui::FontFamily::Monospace)
        .expect("monospace family");
    assert!(monospace.iter().any(|name| name == EGUI_CJK_FONT_NAME));
}
#[test]
fn parse_ui_language_supports_zh_and_en_aliases() {
    assert_eq!(UiLanguage::from_tag("zh"), Some(UiLanguage::ZhCn));
    assert_eq!(UiLanguage::from_tag("zh-CN"), Some(UiLanguage::ZhCn));
    assert_eq!(UiLanguage::from_tag("en"), Some(UiLanguage::EnUs));
    assert_eq!(UiLanguage::from_tag("EN_us"), Some(UiLanguage::EnUs));
    assert_eq!(UiLanguage::from_tag("ja"), None);
}
#[test]
fn launcher_status_text_is_localized() {
    assert_eq!(LauncherStatus::Idle.text(UiLanguage::ZhCn), "未启动");
    assert_eq!(LauncherStatus::Idle.text(UiLanguage::EnUs), "Not Started");
}
#[test]
fn collect_required_config_issues_reports_missing_required_fields() {
    let config = LaunchConfig {
        scenario: "".to_string(),
        live_bind: "127.0.0.1".to_string(),
        web_bind: "127.0.0.1".to_string(),
        viewer_host: "".to_string(),
        viewer_port: "0".to_string(),
        viewer_static_dir: "".to_string(),
        launcher_bin: "".to_string(),
        chain_enabled: true,
        chain_status_bind: "127.0.0.1".to_string(),
        chain_node_id: "".to_string(),
        chain_node_role: "invalid".to_string(),
        chain_node_tick_ms: "0".to_string(),
        chain_node_validators: "node-a".to_string(),
        ..LaunchConfig::default()
    };

    let issues = collect_required_config_issues(&config);
    assert!(issues.contains(&ConfigIssue::ScenarioRequired));
    assert!(issues.contains(&ConfigIssue::LiveBindInvalid));
    assert!(issues.contains(&ConfigIssue::WebBindInvalid));
    assert!(issues.contains(&ConfigIssue::ViewerHostRequired));
    assert!(issues.contains(&ConfigIssue::ViewerPortInvalid));
    assert!(issues.contains(&ConfigIssue::ViewerStaticDirRequired));
    assert!(issues.contains(&ConfigIssue::LauncherBinRequired));
    assert!(issues.contains(&ConfigIssue::ChainStatusBindInvalid));
    assert!(issues.contains(&ConfigIssue::ChainNodeIdRequired));
    assert!(issues.contains(&ConfigIssue::ChainRoleInvalid));
    assert!(issues.contains(&ConfigIssue::ChainTickMsInvalid));
    assert!(issues.contains(&ConfigIssue::ChainValidatorsInvalid));
}
#[test]
fn collect_required_config_issues_accepts_valid_required_fields() {
    let launcher_bin = std::env::current_exe()
        .expect("current exe")
        .to_string_lossy()
        .to_string();
    let config = LaunchConfig {
        scenario: "llm_bootstrap".to_string(),
        live_bind: "127.0.0.1:5023".to_string(),
        web_bind: "127.0.0.1:5011".to_string(),
        viewer_host: "127.0.0.1".to_string(),
        viewer_port: "4173".to_string(),
        viewer_static_dir: ".".to_string(),
        chain_enabled: false,
        launcher_bin,
        ..LaunchConfig::default()
    };

    let issues = collect_required_config_issues(&config);
    assert!(issues.is_empty());
}
