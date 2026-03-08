use super::{
    build_chain_runtime_args, build_game_url, build_launcher_args,
    collect_chain_required_config_issues, collect_required_config_issues,
    config_ui::{issue_field_ids, StartupGuideTarget},
    encode_query_value, encoded_query_pair, install_cjk_font, normalize_host_for_url,
    parse_chain_role, parse_chain_validators, parse_host_port, parse_port,
    probe_chain_status_endpoint,
    self_guided::OnboardingStep,
    ChainRuntimeStatus, ClientLauncherApp, ConfigIssue, LaunchConfig, LauncherStatus, UiLanguage,
    WebRequestDomain, WebStateSnapshot, EGUI_CJK_FONT_NAME,
};
use eframe::egui;
use std::io::{Read, Write};
use std::net::TcpListener;
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
fn build_launcher_args_keeps_chain_disabled_even_when_chain_config_is_set() {
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
    assert!(args.contains(&"--chain-disable".to_string()));
    assert!(!args.contains(&"--chain-enable".to_string()));
    assert!(!args.contains(&"--chain-status-bind".to_string()));
}

#[test]
fn build_chain_runtime_args_contains_chain_overrides_when_enabled() {
    let config = LaunchConfig {
        chain_enabled: true,
        chain_status_bind: "127.0.0.1:6121".to_string(),
        chain_node_id: "chain-node-a".to_string(),
        chain_world_id: "live-chain-a".to_string(),
        chain_node_role: "storage".to_string(),
        chain_node_tick_ms: "350".to_string(),
        chain_pos_slot_duration_ms: "12000".to_string(),
        chain_pos_ticks_per_slot: "10".to_string(),
        chain_pos_proposal_tick_phase: "9".to_string(),
        chain_pos_adaptive_tick_scheduler_enabled: true,
        chain_pos_slot_clock_genesis_unix_ms: "1700000000000".to_string(),
        chain_pos_max_past_slot_lag: "32".to_string(),
        chain_node_validators: "node-a:55,node-b:45".to_string(),
        chain_runtime_bin: "/tmp/world_chain_runtime".to_string(),
        ..LaunchConfig::default()
    };
    let args = build_chain_runtime_args(&config).expect("args should build");
    assert!(args.contains(&"--node-id".to_string()));
    assert!(args.contains(&"chain-node-a".to_string()));
    assert!(args.contains(&"--world-id".to_string()));
    assert!(args.contains(&"live-chain-a".to_string()));
    assert!(args.contains(&"--status-bind".to_string()));
    assert!(args.contains(&"127.0.0.1:6121".to_string()));
    assert!(args.contains(&"--node-role".to_string()));
    assert!(args.contains(&"storage".to_string()));
    assert!(args.contains(&"--node-tick-ms".to_string()));
    assert!(args.contains(&"350".to_string()));
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
    assert!(args.contains(&"--node-validator".to_string()));
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
fn chain_runtime_status_text_is_localized() {
    assert_eq!(ChainRuntimeStatus::Ready.text(UiLanguage::ZhCn), "已就绪");
    assert_eq!(
        ChainRuntimeStatus::Unreachable("x".to_string()).text(UiLanguage::EnUs),
        "Unreachable"
    );
}

#[test]
fn encode_query_value_percent_encodes_reserved_characters() {
    assert_eq!(
        encode_query_value("player:alice & bob?"),
        "player%3Aalice%20%26%20bob%3F"
    );
    assert_eq!(encode_query_value("你好"), "%E4%BD%A0%E5%A5%BD");
}

#[test]
fn encoded_query_pair_formats_key_value_pair() {
    assert_eq!(
        encoded_query_pair("account_id", "player:alice&bob"),
        "account_id=player%3Aalice%26bob"
    );
}

#[test]
fn feedback_availability_requires_chain_ready() {
    let mut app = ClientLauncherApp::default();
    app.config.chain_enabled = true;
    app.chain_runtime_status = ChainRuntimeStatus::Ready;
    assert!(app.is_feedback_available());

    app.chain_runtime_status = ChainRuntimeStatus::Starting;
    assert!(!app.is_feedback_available());

    app.chain_runtime_status = ChainRuntimeStatus::Ready;
    app.config.chain_enabled = false;
    assert!(!app.is_feedback_available());
}

#[test]
fn feedback_unavailable_hint_includes_status_reason() {
    let mut app = ClientLauncherApp::default();
    app.ui_language = UiLanguage::EnUs;
    app.chain_runtime_status = ChainRuntimeStatus::Starting;
    let hint = app
        .feedback_unavailable_hint()
        .expect("starting status should provide hint");
    assert!(hint.contains("starting"));

    app.chain_runtime_status = ChainRuntimeStatus::ConfigError("bad bind".to_string());
    let hint = app
        .feedback_unavailable_hint()
        .expect("config error status should provide hint");
    assert!(hint.contains("bad bind"));
}

#[test]
fn web_request_inflight_domains_are_independent() {
    let mut app = ClientLauncherApp::default();
    assert!(!app.any_web_request_inflight());
    assert!(!app.any_transfer_request_inflight());

    app.set_web_request_inflight(WebRequestDomain::StatePoll, true);
    assert!(app.web_request_inflight_for(WebRequestDomain::StatePoll));
    assert!(!app.web_request_inflight_for(WebRequestDomain::ExplorerQuery));
    assert!(app.any_web_request_inflight());

    app.set_web_request_inflight(WebRequestDomain::TransferQuery, true);
    assert!(app.any_transfer_request_inflight());
    assert!(app.web_request_inflight_for(WebRequestDomain::TransferQuery));
    assert!(!app.web_request_inflight_for(WebRequestDomain::TransferSubmit));

    app.set_web_request_inflight(WebRequestDomain::StatePoll, false);
    app.set_web_request_inflight(WebRequestDomain::TransferQuery, false);
    assert!(!app.any_web_request_inflight());
    assert!(!app.any_transfer_request_inflight());
}

#[test]
fn apply_web_snapshot_preserves_local_dirty_config_when_snapshot_differs() {
    let mut app = ClientLauncherApp::default();
    app.config.scenario = "local_edit".to_string();
    app.config_dirty = true;

    let mut remote_config = app.config.clone();
    remote_config.scenario = "remote_value".to_string();
    let snapshot = WebStateSnapshot {
        status: "idle".to_string(),
        detail: None,
        chain_status: "not_started".to_string(),
        chain_detail: None,
        game_url: "http://127.0.0.1:4173/".to_string(),
        config: remote_config,
        logs: vec!["snapshot".to_string()],
    };

    app.apply_web_snapshot(snapshot);
    assert_eq!(app.config.scenario, "local_edit");
    assert!(app.config_dirty);
}

#[test]
fn apply_web_snapshot_clears_dirty_flag_when_snapshot_matches_local_config() {
    let mut app = ClientLauncherApp::default();
    app.config.scenario = "same_value".to_string();
    app.config_dirty = true;
    let snapshot = WebStateSnapshot {
        status: "idle".to_string(),
        detail: None,
        chain_status: "not_started".to_string(),
        chain_detail: None,
        game_url: "http://127.0.0.1:4173/".to_string(),
        config: app.config.clone(),
        logs: vec!["snapshot".to_string()],
    };

    app.apply_web_snapshot(snapshot);
    assert!(!app.config_dirty);
}

#[test]
fn clear_transfer_history_filters_resets_filters_and_marks_refresh() {
    let mut app = ClientLauncherApp::default();
    app.transfer_panel_state.history_account_filter = "acc-1".to_string();
    app.transfer_panel_state.history_action_filter = "42".to_string();
    app.transfer_panel_state.pending_history_refresh = false;

    app.clear_transfer_history_filters();

    assert!(app.transfer_panel_state.history_account_filter.is_empty());
    assert!(app.transfer_panel_state.history_action_filter.is_empty());
    assert!(app.transfer_panel_state.pending_history_refresh);
}

#[test]
fn probe_chain_status_endpoint_accepts_http_200_response() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test listener");
    let bind = listener.local_addr().expect("listener addr");
    let serve = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept probe connection");
        let mut request = [0_u8; 512];
        let _ = stream.read(&mut request);
        let _ = stream.write_all(
            b"HTTP/1.1 200 OK\r\nContent-Length: 11\r\nConnection: close\r\n\r\n{\"ok\":true}",
        );
    });

    probe_chain_status_endpoint(bind.to_string().as_str()).expect("probe should pass");
    serve.join().expect("server thread should finish");
}

#[test]
fn probe_chain_status_endpoint_reports_connect_failure() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind temp listener");
    let bind = listener.local_addr().expect("listener addr").to_string();
    drop(listener);

    let err = probe_chain_status_endpoint(bind.as_str()).expect_err("probe should fail");
    assert!(err.contains("connect chain status server failed"));
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
        chain_runtime_bin: "".to_string(),
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
}

#[test]
fn collect_chain_required_config_issues_reports_missing_required_fields() {
    let config = LaunchConfig {
        chain_enabled: true,
        chain_runtime_bin: "".to_string(),
        chain_status_bind: "127.0.0.1".to_string(),
        chain_node_id: "".to_string(),
        chain_node_role: "invalid".to_string(),
        chain_node_tick_ms: "0".to_string(),
        chain_pos_slot_duration_ms: "0".to_string(),
        chain_pos_ticks_per_slot: "4".to_string(),
        chain_pos_proposal_tick_phase: "4".to_string(),
        chain_pos_slot_clock_genesis_unix_ms: "oops".to_string(),
        chain_pos_max_past_slot_lag: "-1".to_string(),
        chain_node_validators: "node-a".to_string(),
        ..LaunchConfig::default()
    };

    let issues = collect_chain_required_config_issues(&config);
    assert!(issues.contains(&ConfigIssue::ChainRuntimeBinRequired));
    assert!(issues.contains(&ConfigIssue::ChainStatusBindInvalid));
    assert!(issues.contains(&ConfigIssue::ChainNodeIdRequired));
    assert!(issues.contains(&ConfigIssue::ChainRoleInvalid));
    assert!(issues.contains(&ConfigIssue::ChainTickMsInvalid));
    assert!(issues.contains(&ConfigIssue::ChainPosSlotDurationMsInvalid));
    assert!(issues.contains(&ConfigIssue::ChainPosProposalTickPhaseOutOfRange));
    assert!(issues.contains(&ConfigIssue::ChainPosSlotClockGenesisUnixMsInvalid));
    assert!(issues.contains(&ConfigIssue::ChainPosMaxPastSlotLagInvalid));
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

#[test]
fn collect_chain_required_config_issues_accepts_valid_required_fields() {
    let chain_runtime_bin = std::env::current_exe()
        .expect("current exe")
        .to_string_lossy()
        .to_string();
    let config = LaunchConfig {
        chain_enabled: true,
        chain_runtime_bin,
        chain_status_bind: "127.0.0.1:6121".to_string(),
        chain_node_id: "chain-node-a".to_string(),
        chain_world_id: "live-chain-a".to_string(),
        chain_node_role: "sequencer".to_string(),
        chain_node_tick_ms: "200".to_string(),
        chain_node_validators: "node-a:100".to_string(),
        ..LaunchConfig::default()
    };

    let issues = collect_chain_required_config_issues(&config);
    assert!(issues.is_empty());
}

#[test]
fn issue_field_ids_maps_phase_out_of_range_to_related_fields() {
    let ids = issue_field_ids(ConfigIssue::ChainPosProposalTickPhaseOutOfRange);
    assert_eq!(
        ids,
        &["chain_pos_ticks_per_slot", "chain_pos_proposal_tick_phase"]
    );
}

#[test]
fn first_check_opens_startup_guide_once_for_game_issues() {
    let mut app = ClientLauncherApp::default();
    let game_issues = [ConfigIssue::ScenarioRequired];
    app.maybe_open_startup_guide_on_first_check(&game_issues, &[]);
    assert!(app.startup_guide_state.open);
    assert_eq!(app.startup_guide_state.target, StartupGuideTarget::Game);
    assert!(app.startup_guide_state.first_check_done);

    app.startup_guide_state.open = false;
    app.maybe_open_startup_guide_on_first_check(&game_issues, &[]);
    assert!(!app.startup_guide_state.open);
}

#[test]
fn handle_start_game_click_opens_startup_guide_when_invalid() {
    let mut app = ClientLauncherApp::default();
    let game_issues = [ConfigIssue::ScenarioRequired];
    app.handle_start_game_click(&game_issues);
    assert_eq!(app.status, LauncherStatus::InvalidArgs);
    assert!(app.startup_guide_state.open);
    assert_eq!(app.startup_guide_state.target, StartupGuideTarget::Game);
}

#[test]
fn handle_start_chain_click_opens_startup_guide_when_invalid() {
    let mut app = ClientLauncherApp::default();
    let chain_issues = [ConfigIssue::ChainNodeIdRequired];
    app.handle_start_chain_click(&chain_issues);
    assert!(matches!(
        app.chain_runtime_status,
        ChainRuntimeStatus::ConfigError(_)
    ));
    assert!(app.startup_guide_state.open);
    assert_eq!(app.startup_guide_state.target, StartupGuideTarget::Chain);
}

#[test]
fn onboarding_auto_open_targets_fix_config_step_when_required_fields_missing() {
    let mut app = ClientLauncherApp::default();
    app.onboarding_state.auto_open_checked = false;
    app.onboarding_state.completed = false;
    app.onboarding_state.open = false;
    let game_issues = [ConfigIssue::ScenarioRequired];
    app.maybe_open_onboarding_on_first_visit(&game_issues, &[], false, false);
    assert!(app.onboarding_state.open);
    assert_eq!(app.onboarding_state.step, OnboardingStep::FixConfig);
}

#[test]
fn onboarding_auto_open_happens_only_once_per_session() {
    let mut app = ClientLauncherApp::default();
    app.onboarding_state.auto_open_checked = false;
    app.onboarding_state.completed = false;
    app.onboarding_state.open = false;
    app.maybe_open_onboarding_on_first_visit(&[], &[], false, false);
    assert!(app.onboarding_state.open);
    assert_eq!(app.onboarding_state.step, OnboardingStep::Understand);

    app.onboarding_state.open = false;
    app.maybe_open_onboarding_on_first_visit(&[], &[], false, false);
    assert!(!app.onboarding_state.open);
}
