use super::platform_ops::viewer_dev_dist_candidates;
use super::{
    build_chain_runtime_args, build_game_url, build_launcher_args, chain_runtime_status_from_web,
    collect_chain_required_config_issues, collect_required_config_issues,
    config_ui::{issue_field_ids, StartupGuideTarget},
    encode_query_value, encoded_query_pair,
    explorer_window::{
        resolve_explorer_my_account_candidate, ExplorerQuickShortcut, ExplorerStatusFilter,
        WebExplorerOverviewResponse,
    },
    install_cjk_font, normalize_host_for_url, parse_chain_role, parse_chain_validators,
    parse_host_port, parse_port, probe_chain_status_endpoint, probe_openclaw_local_http,
    read_named_env_value_with, resolve_control_plane_env_with,
    self_guided::{
        resolve_config_guide_target, resolve_next_task_hint, resolve_primary_disabled_cta,
        ConfigGuideTargetHint, DemoModePhase, DisabledActionCta, NextTaskHint, OnboardingStep,
    },
    self_guided_blocked_actions::resolve_disabled_cta_plan,
    self_guided_preflight::{resolve_chain_runtime_preflight_state, PreflightCheckState},
    transfer_window::{
        recommend_default_from_account, recommend_transfer_account_ids, resolve_transfer_timeline,
        transfer_amount_presets, TransferTimelineState, WebTransferAccountEntry,
        WebTransferLifecycleStatus,
    },
    ChainRuntimeStatus, ClientLauncherApp, ConfigIssue, GlossaryTerm, LaunchConfig, LauncherStatus,
    UiLanguage, WebChainRecoverySnapshot, WebRequestDomain, WebStateSnapshot,
    DEFAULT_CLIENT_LAUNCHER_CONTROL_BIND, OASIS7_CJK_FONT_NAME, OASIS7_CLIENT_LAUNCHER_LANG_ENV,
};
use eframe::egui;
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
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
    assert!(args.contains(&"--agent-provider-mode".to_string()));
    assert!(args.contains(&"builtin_llm".to_string()));
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
fn build_launcher_args_includes_openclaw_profile_flags() {
    let config = LaunchConfig {
        llm_enabled: true,
        agent_provider_mode: "openclaw_local_http".to_string(),
        openclaw_base_url: "http://127.0.0.1:5841".to_string(),
        openclaw_auth_token: "secret-token".to_string(),
        openclaw_connect_timeout_ms: "3000".to_string(),
        openclaw_agent_profile: "oasis7_p0_low_freq_npc".to_string(),
        ..LaunchConfig::default()
    };
    let args = build_launcher_args(&config).expect("args should build");
    assert!(args.contains(&"--agent-provider-mode".to_string()));
    assert!(args.contains(&"openclaw_local_http".to_string()));
    assert!(args.contains(&"--openclaw-base-url".to_string()));
    assert!(args.contains(&"http://127.0.0.1:5841".to_string()));
    assert!(args.contains(&"--openclaw-auth-token".to_string()));
    assert!(args.contains(&"secret-token".to_string()));
    assert!(args.contains(&"--openclaw-connect-timeout-ms".to_string()));
    assert!(args.contains(&"3000".to_string()));
    assert!(args.contains(&"--openclaw-agent-profile".to_string()));
    assert!(args.contains(&"oasis7_p0_low_freq_npc".to_string()));
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
fn viewer_dev_dist_candidates_only_return_current_oasis7_name() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    let candidates = viewer_dev_dist_candidates();

    assert_eq!(
        candidates,
        vec![repo_root.join("oasis7_viewer").join("dist")]
    );
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
    assert_eq!(config.agent_provider_mode, "builtin_llm");
    assert_eq!(config.openclaw_base_url, "http://127.0.0.1:5841");
    assert_eq!(config.openclaw_agent_profile, "oasis7_p0_low_freq_npc");
    assert!(config.openclaw_auto_discover);
    assert!(config.chain_node_id.starts_with("viewer-live-node-fresh-"));
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
        chain_runtime_bin: "/tmp/oasis7_chain_runtime".to_string(),
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
        OASIS7_CJK_FONT_NAME.to_string(),
        egui::FontData::from_static(&[0u8, 1u8]),
    );

    assert!(fonts.font_data.contains_key(OASIS7_CJK_FONT_NAME));

    let proportional = fonts
        .families
        .get(&egui::FontFamily::Proportional)
        .expect("proportional family");
    assert_eq!(
        proportional.first().map(String::as_str),
        Some(OASIS7_CJK_FONT_NAME)
    );

    let monospace = fonts
        .families
        .get(&egui::FontFamily::Monospace)
        .expect("monospace family");
    assert!(monospace.iter().any(|name| name == OASIS7_CJK_FONT_NAME));
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
fn read_named_env_value_with_rejects_removed_old_brand_key_names() {
    let removed_old_brand_lang_key = removed_old_brand_launcher_env("LANG");
    let values = BTreeMap::from([(removed_old_brand_lang_key.as_str(), "en-US")]);
    let resolved = read_named_env_value_with(
        &|key| values.get(key).map(|value| value.to_string()),
        &[OASIS7_CLIENT_LAUNCHER_LANG_ENV],
    );
    assert_eq!(resolved, None);
}

#[test]
fn ui_language_detect_from_values_prefers_current_launcher_value_and_falls_back_to_lang() {
    assert_eq!(
        UiLanguage::detect_from_values(Some("en-US"), Some("zh-CN")),
        UiLanguage::EnUs
    );
    assert_eq!(
        UiLanguage::detect_from_values(None, Some("en-US")),
        UiLanguage::EnUs
    );
    let removed_old_brand_lang_key = removed_old_brand_launcher_env("LANG");
    assert_eq!(
        UiLanguage::detect_from_values(Some(removed_old_brand_lang_key.as_str()), Some("zh-CN")),
        UiLanguage::ZhCn
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn resolve_control_plane_env_with_rejects_removed_old_brand_key_names() {
    let removed_old_brand_control_url = removed_old_brand_launcher_env("CONTROL_URL");
    let removed_old_brand_control_bind = removed_old_brand_launcher_env("CONTROL_BIND");
    let values = BTreeMap::from([
        (removed_old_brand_control_url.as_str(), "http://127.0.0.1:9999"),
        (removed_old_brand_control_bind.as_str(), "127.0.0.1:9998"),
    ]);
    let (control_url_from_env, control_listen_bind, control_api_base, control_manage_service) =
        resolve_control_plane_env_with(&|key| values.get(key).map(|value| value.to_string()));

    assert_eq!(control_url_from_env, None);
    assert_eq!(
        control_listen_bind,
        DEFAULT_CLIENT_LAUNCHER_CONTROL_BIND.to_string()
    );
    assert_eq!(control_api_base, "http://127.0.0.1:5410");
    assert!(control_manage_service);
}

fn removed_old_brand_launcher_env(suffix: &str) -> String {
    ["AGENT", "WORLD", "CLIENT", "LAUNCHER", suffix].join("_")
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
fn transfer_amount_presets_match_product_defaults() {
    assert_eq!(transfer_amount_presets(), &[1, 10, 100]);
}

#[test]
fn recommend_default_from_account_uses_highest_liquid_balance() {
    let accounts = vec![
        WebTransferAccountEntry {
            account_id: "player-a".to_string(),
            liquid_balance: 30,
            vested_balance: 0,
            next_nonce_hint: 3,
        },
        WebTransferAccountEntry {
            account_id: "player-b".to_string(),
            liquid_balance: 90,
            vested_balance: 0,
            next_nonce_hint: 1,
        },
        WebTransferAccountEntry {
            account_id: "player-c".to_string(),
            liquid_balance: 20,
            vested_balance: 0,
            next_nonce_hint: 5,
        },
    ];

    assert_eq!(
        recommend_default_from_account(accounts.as_slice()),
        Some("player-b".to_string())
    );
}

#[test]
fn recommend_transfer_account_ids_excludes_sender_and_sorts_by_balance() {
    let accounts = vec![
        WebTransferAccountEntry {
            account_id: "player-a".to_string(),
            liquid_balance: 50,
            vested_balance: 0,
            next_nonce_hint: 1,
        },
        WebTransferAccountEntry {
            account_id: "player-b".to_string(),
            liquid_balance: 80,
            vested_balance: 0,
            next_nonce_hint: 2,
        },
        WebTransferAccountEntry {
            account_id: "player-c".to_string(),
            liquid_balance: 20,
            vested_balance: 0,
            next_nonce_hint: 3,
        },
        WebTransferAccountEntry {
            account_id: "player-d".to_string(),
            liquid_balance: 70,
            vested_balance: 0,
            next_nonce_hint: 4,
        },
    ];

    assert_eq!(
        recommend_transfer_account_ids(accounts.as_slice(), "player-a", 3),
        vec![
            "player-b".to_string(),
            "player-d".to_string(),
            "player-c".to_string()
        ]
    );
}

#[test]
fn resolve_transfer_timeline_tracks_accepted_pending_final_states() {
    assert_eq!(
        resolve_transfer_timeline(WebTransferLifecycleStatus::Accepted),
        [
            TransferTimelineState::Active,
            TransferTimelineState::Waiting,
            TransferTimelineState::Waiting
        ]
    );
    assert_eq!(
        resolve_transfer_timeline(WebTransferLifecycleStatus::Pending),
        [
            TransferTimelineState::Done,
            TransferTimelineState::Active,
            TransferTimelineState::Waiting
        ]
    );
    assert_eq!(
        resolve_transfer_timeline(WebTransferLifecycleStatus::Confirmed),
        [
            TransferTimelineState::Done,
            TransferTimelineState::Done,
            TransferTimelineState::Done
        ]
    );
    assert_eq!(
        resolve_transfer_timeline(WebTransferLifecycleStatus::Failed),
        [
            TransferTimelineState::Done,
            TransferTimelineState::Done,
            TransferTimelineState::Failed
        ]
    );
}

#[test]
fn resolve_explorer_my_account_candidate_prefers_transfer_sender() {
    assert_eq!(
        resolve_explorer_my_account_candidate("player-a", "player-b", "player-c"),
        Some("player-a".to_string())
    );
    assert_eq!(
        resolve_explorer_my_account_candidate("", "player-b", "player-c"),
        Some("player-b".to_string())
    );
    assert_eq!(
        resolve_explorer_my_account_candidate("", "", "player-c"),
        Some("player-c".to_string())
    );
    assert_eq!(resolve_explorer_my_account_candidate("", "", ""), None);
}

#[test]
fn explorer_quick_shortcut_recent_txs_resets_filters_and_refreshes() {
    let mut app = ClientLauncherApp::default();
    app.explorer_panel_state.account_filter = "player-a".to_string();
    app.explorer_panel_state.action_filter_input = "42".to_string();
    app.explorer_panel_state.status_filter = ExplorerStatusFilter::Failed;
    app.explorer_panel_state.txs_cursor = 20;

    app.apply_explorer_quick_shortcut(ExplorerQuickShortcut::RecentTxs);

    assert!(app.explorer_panel_state.account_filter.is_empty());
    assert!(app.explorer_panel_state.action_filter_input.is_empty());
    assert_eq!(
        app.explorer_panel_state.status_filter,
        ExplorerStatusFilter::All
    );
    assert_eq!(app.explorer_panel_state.txs_cursor, 0);
    assert!(app.explorer_panel_state.pending_txs_refresh);
}

#[test]
fn explorer_quick_shortcut_my_account_logs_when_missing_candidate() {
    let mut app = ClientLauncherApp::default();
    app.ui_language = UiLanguage::EnUs;
    let logs_before = app.logs.len();

    app.apply_explorer_quick_shortcut(ExplorerQuickShortcut::MyAccount);

    assert_eq!(app.logs.len(), logs_before + 1);
    let latest_log = app.logs.back().expect("latest log should exist");
    assert!(latest_log.contains("My Account shortcut is unavailable"));
}

#[test]
fn explorer_quick_shortcut_latest_block_prefills_height_from_overview() {
    let mut app = ClientLauncherApp::default();
    app.explorer_panel_state.overview = Some(WebExplorerOverviewResponse {
        ok: true,
        observed_at_unix_ms: 1,
        node_id: "node-a".to_string(),
        world_id: "world-a".to_string(),
        latest_height: 88,
        committed_height: 88,
        network_committed_height: 88,
        last_block_hash: Some("hash-a".to_string()),
        last_execution_block_hash: Some("hash-b".to_string()),
        tracked_records: 0,
        transfer_total: 0,
        transfer_accepted: 0,
        transfer_pending: 0,
        transfer_confirmed: 0,
        transfer_failed: 0,
        transfer_timeout: 0,
        error_code: None,
        error: None,
    });

    app.apply_explorer_quick_shortcut(ExplorerQuickShortcut::LatestBlock);

    assert_eq!(app.explorer_panel_state.block_height_input, "88");
    assert_eq!(app.explorer_panel_state.pending_block_height, Some(88));
    assert!(app.explorer_panel_state.pending_block_refresh);
}

#[test]
fn glossary_terms_cover_nonce_slot_mempool_action_id() {
    let mut app = ClientLauncherApp::default();
    app.ui_language = UiLanguage::EnUs;
    assert_eq!(app.glossary_term_text(GlossaryTerm::Nonce), "nonce");
    assert_eq!(app.glossary_term_text(GlossaryTerm::Slot), "slot");
    assert_eq!(app.glossary_term_text(GlossaryTerm::Mempool), "mempool");
    assert_eq!(app.glossary_term_text(GlossaryTerm::ActionId), "action_id");

    for term in [
        GlossaryTerm::Nonce,
        GlossaryTerm::Slot,
        GlossaryTerm::Mempool,
        GlossaryTerm::ActionId,
    ] {
        assert!(!app.glossary_term_definition(term).trim().is_empty());
    }
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
fn chain_runtime_status_from_web_maps_stale_execution_world() {
    let status = chain_runtime_status_from_web(
        "stale_execution_world",
        Some("stale execution world detected"),
    );
    assert!(matches!(
        status,
        ChainRuntimeStatus::StaleExecutionWorld(ref detail)
            if detail == "stale execution world detected"
    ));
}

#[test]
fn apply_web_snapshot_tracks_chain_recovery_payload() {
    let mut app = ClientLauncherApp::default();
    let snapshot = WebStateSnapshot {
        status: "idle".to_string(),
        detail: None,
        chain_status: "stale_execution_world".to_string(),
        chain_detail: Some("stale execution world detected".to_string()),
        chain_recovery: Some(WebChainRecoverySnapshot {
            error_code: "stale_execution_world".to_string(),
            reason: "stale execution world detected".to_string(),
            node_id: "viewer-live-node".to_string(),
            execution_world_dir:
                "output/chain-runtime/viewer-live-node/reward-runtime-execution-world".to_string(),
            recovery_mode: "fresh_node_id".to_string(),
            reset_required: false,
            fresh_node_id: "viewer-live-node-fresh-1".to_string(),
            fresh_chain_status_bind: "127.0.0.1:5122".to_string(),
            suggested_config: LaunchConfig {
                chain_node_id: "viewer-live-node-fresh-1".to_string(),
                chain_status_bind: "127.0.0.1:5122".to_string(),
                ..LaunchConfig::default()
            },
        }),
        game_url: "http://127.0.0.1:4173/".to_string(),
        config: LaunchConfig::default(),
        logs: vec![],
    };

    app.apply_web_snapshot(snapshot);
    assert!(matches!(
        app.chain_runtime_status,
        ChainRuntimeStatus::StaleExecutionWorld(_)
    ));
    assert_eq!(
        app.chain_recovery
            .as_ref()
            .map(|value| value.fresh_node_id.as_str()),
        Some("viewer-live-node-fresh-1")
    );
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
        chain_recovery: None,
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
        chain_recovery: None,
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
fn probe_openclaw_local_http_accepts_info_and_health_responses() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test listener");
    let bind = listener.local_addr().expect("listener addr");
    let serve = std::thread::spawn(move || {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().expect("accept probe connection");
            let mut request = [0_u8; 1024];
            let bytes = stream.read(&mut request).expect("read request");
            let request_text = String::from_utf8_lossy(&request[..bytes]);
            let body = if request_text.contains("GET /v1/provider/info") {
                r#"{"provider_id":"openclaw-local","name":"OpenClaw","version":"0.1.0","protocol_version":"v1"}"#
            } else {
                r#"{"ok":true,"status":"ready","uptime_ms":42,"last_error":null,"queue_depth":0}"#
            };
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes());
        }
    });

    let snapshot = probe_openclaw_local_http(format!("http://{}", bind).as_str(), None, 200)
        .expect("probe should pass");
    assert_eq!(snapshot.provider_id, "openclaw-local");
    assert_eq!(snapshot.name, "OpenClaw");
    assert_eq!(snapshot.version, "0.1.0");
    assert_eq!(snapshot.protocol_version, "v1");
    assert_eq!(snapshot.status, "ready");
    assert_eq!(snapshot.queue_depth, Some(0));
    assert_eq!(snapshot.last_error, None);
    assert!(snapshot.info_latency_ms <= snapshot.total_latency_ms);
    assert!(snapshot.health_latency_ms <= snapshot.total_latency_ms);
    serve.join().expect("server thread should finish");
}

#[test]
fn collect_required_config_issues_reports_openclaw_specific_fields() {
    let config = LaunchConfig {
        agent_provider_mode: "openclaw_local_http".to_string(),
        openclaw_base_url: String::new(),
        openclaw_auto_discover: false,
        openclaw_connect_timeout_ms: "0".to_string(),
        ..LaunchConfig::default()
    };
    let issues = collect_required_config_issues(&config);
    assert!(issues.contains(&ConfigIssue::OpenClawBaseUrlRequired));
    assert!(issues.contains(&ConfigIssue::OpenClawConnectTimeoutMsInvalid));
    assert!(!issues.contains(&ConfigIssue::OpenClawAgentProfileRequired));
}

#[test]
fn collect_required_config_issues_rejects_non_loopback_openclaw_base_url() {
    let config = LaunchConfig {
        agent_provider_mode: "openclaw_local_http".to_string(),
        openclaw_base_url: "http://192.168.0.5:5841".to_string(),
        ..LaunchConfig::default()
    };
    let issues = collect_required_config_issues(&config);
    assert!(issues.contains(&ConfigIssue::OpenClawBaseUrlLoopbackRequired));
}

#[test]
fn collect_required_config_issues_requires_openclaw_agent_profile() {
    let config = LaunchConfig {
        agent_provider_mode: "openclaw_local_http".to_string(),
        openclaw_agent_profile: String::new(),
        ..LaunchConfig::default()
    };
    let issues = collect_required_config_issues(&config);
    assert!(issues.contains(&ConfigIssue::OpenClawAgentProfileRequired));
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
fn collect_required_config_issues_accepts_bundle_relative_web_path_from_launcher_bin() {
    let bundle_root = make_temp_dir("client_launcher_bundle_relative");
    let bundle_bin = bundle_root.join("bin");
    let launcher_bin = bundle_bin.join("oasis7_game_launcher");
    let bundle_web = bundle_root.join("web");
    fs::create_dir_all(&bundle_bin).expect("create bundle bin dir");
    fs::create_dir_all(&bundle_web).expect("create bundle web dir");
    fs::write(&launcher_bin, b"#!/bin/sh\n").expect("write fake launcher bin");

    let config = LaunchConfig {
        scenario: "llm_bootstrap".to_string(),
        live_bind: "127.0.0.1:5023".to_string(),
        web_bind: "127.0.0.1:5011".to_string(),
        viewer_host: "127.0.0.1".to_string(),
        viewer_port: "4173".to_string(),
        viewer_static_dir: "web".to_string(),
        chain_enabled: false,
        launcher_bin: launcher_bin.to_string_lossy().to_string(),
        ..LaunchConfig::default()
    };

    let issues = collect_required_config_issues(&config);
    assert!(!issues.contains(&ConfigIssue::ViewerStaticDirMissing));
    assert!(!issues.contains(&ConfigIssue::LauncherBinMissing));

    let _ = fs::remove_dir_all(bundle_root);
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
fn apply_safe_defaults_for_game_target_recovers_required_fields() {
    let mut app = ClientLauncherApp::default();
    app.config.scenario.clear();
    app.config.live_bind = "127.0.0.1".to_string();
    app.config.web_bind = "127.0.0.1".to_string();
    app.config.viewer_host.clear();
    app.config.viewer_port = "0".to_string();
    app.config.viewer_static_dir.clear();

    app.apply_safe_defaults_for_startup_target(StartupGuideTarget::Game);

    let game_issues = collect_required_config_issues(&app.config);
    assert!(!game_issues.contains(&ConfigIssue::ScenarioRequired));
    assert!(!game_issues.contains(&ConfigIssue::LiveBindInvalid));
    assert!(!game_issues.contains(&ConfigIssue::WebBindInvalid));
    assert!(!game_issues.contains(&ConfigIssue::ViewerHostRequired));
    assert!(!game_issues.contains(&ConfigIssue::ViewerPortInvalid));
    assert!(!game_issues.contains(&ConfigIssue::ViewerStaticDirRequired));
}

#[test]
fn apply_safe_defaults_for_chain_target_recovers_required_fields() {
    let mut app = ClientLauncherApp::default();
    app.config.chain_enabled = false;
    app.config.chain_runtime_bin.clear();
    app.config.chain_status_bind = "127.0.0.1".to_string();
    app.config.chain_node_id.clear();
    app.config.chain_node_role = "invalid".to_string();
    app.config.chain_node_tick_ms = "0".to_string();
    app.config.chain_pos_slot_duration_ms = "0".to_string();
    app.config.chain_pos_ticks_per_slot = "0".to_string();
    app.config.chain_pos_proposal_tick_phase = "99".to_string();
    app.config.chain_pos_max_past_slot_lag = "-1".to_string();

    app.apply_safe_defaults_for_startup_target(StartupGuideTarget::Chain);

    let chain_issues = collect_chain_required_config_issues(&app.config);
    assert!(app.config.chain_enabled);
    assert!(chain_issues.is_empty());
    assert_eq!(app.chain_runtime_status, ChainRuntimeStatus::NotStarted);
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

#[test]
fn onboarding_auto_open_respects_dismissed_state() {
    let mut app = ClientLauncherApp::default();
    app.onboarding_state.auto_open_checked = false;
    app.onboarding_state.completed = false;
    app.onboarding_state.dismissed = true;
    app.onboarding_state.open = false;

    app.maybe_open_onboarding_on_first_visit(&[], &[], false, false);
    assert!(!app.onboarding_state.open);
}

#[test]
fn dismiss_onboarding_with_reminder_keeps_reminder_visible() {
    let mut app = ClientLauncherApp::default();
    app.onboarding_state.open = true;

    app.dismiss_onboarding_with_reminder();

    assert!(!app.onboarding_state.completed);
    assert!(app.onboarding_state.dismissed);
    assert!(app.should_show_onboarding_reminder());
    assert_eq!(app.ux_state.onboarding_skipped_count, 1);
}

#[test]
fn should_show_onboarding_reminder_hides_when_completed_or_open() {
    let mut app = ClientLauncherApp::default();
    app.onboarding_state.completed = false;
    app.onboarding_state.open = false;
    assert!(app.should_show_onboarding_reminder());

    app.onboarding_state.open = true;
    assert!(!app.should_show_onboarding_reminder());

    app.onboarding_state.open = false;
    app.onboarding_state.completed = true;
    assert!(!app.should_show_onboarding_reminder());
}

#[test]
fn resolve_next_task_hint_prioritizes_config_fix_then_start_order() {
    assert_eq!(
        resolve_next_task_hint(true, &[], &[ConfigIssue::ChainNodeIdRequired], false, false),
        NextTaskHint::FixChainConfig
    );
    assert_eq!(
        resolve_next_task_hint(true, &[ConfigIssue::ScenarioRequired], &[], false, false),
        NextTaskHint::FixGameConfig
    );
    assert_eq!(
        resolve_next_task_hint(true, &[], &[], false, false),
        NextTaskHint::StartChain
    );
    assert_eq!(
        resolve_next_task_hint(true, &[], &[], false, true),
        NextTaskHint::StartGame
    );
    assert_eq!(
        resolve_next_task_hint(true, &[], &[], true, true),
        NextTaskHint::OpenGamePage
    );
}

#[test]
fn resolve_config_guide_target_follows_blocking_priority() {
    assert_eq!(
        resolve_config_guide_target(
            true,
            &[ConfigIssue::ScenarioRequired],
            &[ConfigIssue::ChainNodeIdRequired],
        ),
        Some(ConfigGuideTargetHint::Chain)
    );
    assert_eq!(
        resolve_config_guide_target(true, &[ConfigIssue::ScenarioRequired], &[]),
        Some(ConfigGuideTargetHint::Game)
    );
    assert_eq!(
        resolve_config_guide_target(false, &[ConfigIssue::ScenarioRequired], &[]),
        Some(ConfigGuideTargetHint::Game)
    );
    assert_eq!(resolve_config_guide_target(true, &[], &[]), None);
}

#[test]
fn resolve_primary_disabled_cta_prefers_first_blocking_action() {
    assert_eq!(
        resolve_primary_disabled_cta(false, &[], &[], false),
        Some(DisabledActionCta::EnableChain)
    );
    assert_eq!(
        resolve_primary_disabled_cta(true, &[], &[ConfigIssue::ChainNodeIdRequired], false),
        Some(DisabledActionCta::FixChainConfig)
    );
    assert_eq!(
        resolve_primary_disabled_cta(true, &[], &[], false),
        Some(DisabledActionCta::StartChain)
    );
    assert_eq!(
        resolve_primary_disabled_cta(true, &[ConfigIssue::ScenarioRequired], &[], true),
        Some(DisabledActionCta::FixGameConfig)
    );
    assert_eq!(resolve_primary_disabled_cta(true, &[], &[], true), None);
}

#[test]
fn resolve_disabled_cta_plan_prefers_retry_when_chain_is_starting() {
    let (primary, secondary) =
        resolve_disabled_cta_plan(&ChainRuntimeStatus::Starting, true, &[], &[]);
    assert_eq!(primary, Some(DisabledActionCta::RetryChainStatus));
    assert_eq!(secondary, Some(DisabledActionCta::StartChain));
}

#[test]
fn resolve_disabled_cta_plan_prioritizes_chain_fix_before_game_fix() {
    let (primary, secondary) = resolve_disabled_cta_plan(
        &ChainRuntimeStatus::ConfigError("bad bind".to_string()),
        true,
        &[ConfigIssue::ScenarioRequired],
        &[ConfigIssue::ChainNodeIdRequired],
    );
    assert_eq!(primary, Some(DisabledActionCta::FixChainConfig));
    assert_eq!(secondary, Some(DisabledActionCta::RetryChainStatus));
}

#[test]
fn resolve_chain_runtime_preflight_state_requires_ready_chain() {
    assert_eq!(
        resolve_chain_runtime_preflight_state(true, &ChainRuntimeStatus::Ready),
        PreflightCheckState::Pass
    );
    assert_eq!(
        resolve_chain_runtime_preflight_state(true, &ChainRuntimeStatus::Starting),
        PreflightCheckState::Blocked
    );
    assert_eq!(
        resolve_chain_runtime_preflight_state(false, &ChainRuntimeStatus::Disabled),
        PreflightCheckState::Blocked
    );
}

#[test]
fn expert_mode_toggle_updates_runtime_state() {
    let mut app = ClientLauncherApp::default();
    app.set_expert_mode(true);
    assert!(app.is_expert_mode());
    app.set_expert_mode(false);
    assert!(!app.is_expert_mode());
}

#[test]
fn successful_config_profile_is_saved_on_running_state() {
    let mut app = ClientLauncherApp::default();
    app.config.scenario = "profile-save".to_string();

    app.maybe_save_last_successful_config_profile(true);

    let saved = app
        .ux_state
        .last_successful_config
        .as_ref()
        .expect("saved profile");
    assert_eq!(saved.scenario, "profile-save");
    assert!(app.ux_state.last_successful_saved_at_unix_ms.is_some());
}

#[test]
fn restore_last_successful_config_profile_replaces_runtime_config() {
    let mut app = ClientLauncherApp::default();
    let mut saved = app.config.clone();
    saved.scenario = "restored-scenario".to_string();
    saved.chain_enabled = false;
    app.ux_state.last_successful_config = Some(saved);

    app.restore_last_successful_config_profile();

    assert_eq!(app.config.scenario, "restored-scenario");
    assert!(!app.config.chain_enabled);
    assert_eq!(app.chain_runtime_status, ChainRuntimeStatus::Disabled);
}

#[test]
fn clear_last_successful_config_profile_clears_saved_snapshot() {
    let mut app = ClientLauncherApp::default();
    app.ux_state.last_successful_config = Some(app.config.clone());
    app.ux_state.last_successful_saved_at_unix_ms = Some(7);

    app.clear_last_successful_config_profile();

    assert!(app.ux_state.last_successful_config.is_none());
    assert!(app.ux_state.last_successful_saved_at_unix_ms.is_none());
}

#[test]
fn start_demo_mode_one_click_applies_safe_defaults() {
    let mut app = ClientLauncherApp::default();
    app.config.chain_enabled = false;
    app.config.scenario = "custom".to_string();

    app.start_demo_mode_one_click();

    assert_eq!(app.demo_mode_phase, DemoModePhase::StartChainRequested);
    assert!(app.config.chain_enabled);
    assert_eq!(app.config.scenario, "llm_bootstrap");
}

#[test]
fn advance_demo_mode_reaches_done_when_chain_and_game_are_ready() {
    let mut app = ClientLauncherApp::default();
    app.start_demo_mode_one_click();
    app.advance_demo_mode(&[], &[], false, true);
    assert_eq!(app.demo_mode_phase, DemoModePhase::StartGameRequested);

    app.advance_demo_mode(&[], &[], true, true);
    assert_eq!(app.demo_mode_phase, DemoModePhase::Done);
}

#[test]
fn advance_demo_mode_fails_when_chain_config_is_blocked() {
    let mut app = ClientLauncherApp::default();
    app.start_demo_mode_one_click();
    app.advance_demo_mode(&[], &[ConfigIssue::ChainNodeIdRequired], false, false);
    assert_eq!(app.demo_mode_phase, DemoModePhase::Failed);
}

#[test]
fn guidance_counters_increase_for_open_demo_and_quick_actions() {
    let mut app = ClientLauncherApp::default();
    app.open_onboarding_manual();
    assert_eq!(app.ux_state.onboarding_opened_count, 1);

    app.start_demo_mode_one_click();
    assert_eq!(app.ux_state.demo_mode_runs_count, 1);

    app.apply_explorer_quick_shortcut(ExplorerQuickShortcut::RecentTxs);
    assert_eq!(app.ux_state.quick_action_click_count, 1);
}

fn make_temp_dir(label: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    path.push(format!(
        "oasis7_client_launcher_test_{label}_{}_{}",
        std::process::id(),
        stamp
    ));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}
