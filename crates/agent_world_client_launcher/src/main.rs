use std::collections::VecDeque;
use std::env;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{BufRead, BufReader, Read, Write};
#[cfg(not(target_arch = "wasm32"))]
use std::net::{TcpStream, ToSocketAddrs};
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::process::{Child, Command, Stdio};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use eframe::egui;
#[cfg(not(target_arch = "wasm32"))]
use feedback_entry::FeedbackDraft;
#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
use llm_settings::LlmSettingsPanel;
use platform_ops::open_browser;
use platform_ops::resolve_static_dir_path;
#[cfg(not(target_arch = "wasm32"))]
use platform_ops::{resolve_chain_runtime_binary_path, resolve_launcher_binary_path};
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use transfer_entry::TransferDraft;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use web_sys::wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::HtmlCanvasElement;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

#[cfg(not(target_arch = "wasm32"))]
mod app_process;
#[cfg(target_arch = "wasm32")]
mod app_process_web;
mod config_ui;
mod explorer_window;
#[cfg(not(target_arch = "wasm32"))]
mod feedback_entry;
#[cfg(not(target_arch = "wasm32"))]
mod feedback_window;
#[cfg(target_arch = "wasm32")]
mod feedback_window_web;
mod launcher_core;
#[cfg(not(target_arch = "wasm32"))]
mod llm_settings;
#[cfg(target_arch = "wasm32")]
#[path = "llm_settings_web.rs"]
mod llm_settings;
mod main_app_shell;
mod main_ui_helpers;
mod platform_ops;
mod self_guided;
mod self_guided_blocked_actions;
mod self_guided_error_cards;
mod self_guided_onboarding_reminder;
mod self_guided_preflight;
#[cfg(not(target_arch = "wasm32"))]
mod transfer_entry;
mod transfer_window;

use config_ui::StartupGuideState;
use launcher_core::*;
use self_guided::{DemoModePhase, LauncherUxState, OnboardingState};

const DEFAULT_SCENARIO: &str = "llm_bootstrap";
const DEFAULT_LIVE_BIND: &str = "127.0.0.1:5023";
const DEFAULT_WEB_BIND: &str = "127.0.0.1:5011";
const DEFAULT_VIEWER_HOST: &str = "127.0.0.1";
const DEFAULT_VIEWER_PORT: &str = "4173";
const DEFAULT_CHAIN_STATUS_BIND: &str = "127.0.0.1:5121";
const DEFAULT_CHAIN_NODE_ID: &str = "viewer-live-node";
const DEFAULT_CHAIN_NODE_ROLE: &str = "sequencer";
const DEFAULT_CHAIN_NODE_TICK_MS: &str = "200";
const DEFAULT_CHAIN_POS_SLOT_DURATION_MS: &str = "12000";
const DEFAULT_CHAIN_POS_TICKS_PER_SLOT: &str = "10";
const DEFAULT_CHAIN_POS_PROPOSAL_TICK_PHASE: &str = "9";
const DEFAULT_CHAIN_POS_SLOT_CLOCK_GENESIS_UNIX_MS: &str = "";
const DEFAULT_CHAIN_POS_MAX_PAST_SLOT_LAG: &str = "256";
const MAX_LOG_LINES: usize = 2000;
const EGUI_CJK_FONT_NAME: &str = "agent-world-cjk";
const EGUI_CJK_FONT_BYTES: &[u8] =
    include_bytes!("../../agent_world_viewer/assets/fonts/ms-yahei.ttf");
const CLIENT_LAUNCHER_FONT_ENV: &str = "AGENT_WORLD_CLIENT_LAUNCHER_FONT";
const CLIENT_LAUNCHER_LANG_ENV: &str = "AGENT_WORLD_CLIENT_LAUNCHER_LANG";
#[cfg(not(target_arch = "wasm32"))]
const GRACEFUL_STOP_TIMEOUT_MS: u64 = 4000;
#[cfg(not(target_arch = "wasm32"))]
const STOP_POLL_INTERVAL_MS: u64 = 80;
#[cfg(not(target_arch = "wasm32"))]
const CHAIN_STATUS_PROBE_TIMEOUT_MS: u64 = 300;
#[cfg(not(target_arch = "wasm32"))]
const CLIENT_LAUNCHER_CONTROL_URL_ENV: &str = "AGENT_WORLD_CLIENT_LAUNCHER_CONTROL_URL";
#[cfg(not(target_arch = "wasm32"))]
const CLIENT_LAUNCHER_CONTROL_BIND_ENV: &str = "AGENT_WORLD_CLIENT_LAUNCHER_CONTROL_BIND";
#[cfg(not(target_arch = "wasm32"))]
const DEFAULT_CLIENT_LAUNCHER_CONTROL_BIND: &str = "127.0.0.1:5410";
const NATIVE_UI_SECTIONS: &[&str] = &[
    "game_core",
    "viewer_core",
    "chain_identity",
    "chain_runtime",
    "binaries",
    "static_assets",
];

#[cfg(target_arch = "wasm32")]
const WEB_CANVAS_ID: &str = "agent-world-launcher-canvas";
const WEB_POLL_INTERVAL_MS: u64 = 1000;

fn default_chain_node_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{DEFAULT_CHAIN_NODE_ID}-fresh-{}-{now}", std::process::id())
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(egui::vec2(920.0, 680.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Agent World Client Launcher",
        native_options,
        Box::new(|cc| {
            configure_egui_fonts(&cc.egui_ctx);
            Ok(Box::<ClientLauncherApp>::default())
        }),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    let web_options = eframe::WebOptions::default();
    let canvas = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(WEB_CANVAS_ID))
        .and_then(|element| element.dyn_into::<HtmlCanvasElement>().ok())
        .unwrap_or_else(|| panic!("missing launcher canvas: #{WEB_CANVAS_ID}"));
    spawn_local(async move {
        let runner = eframe::WebRunner::new();
        let start_result = runner
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    configure_egui_fonts(&cc.egui_ctx);
                    Ok(Box::<ClientLauncherApp>::default())
                }),
            )
            .await;
        if let Err(err) = start_result {
            eprintln!("failed to start launcher web app: {err:?}");
        }
    });
}

fn configure_egui_fonts(context: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    match load_font_override_from_env() {
        Some((font_name, font_data)) => install_cjk_font(&mut fonts, font_name, font_data),
        None => install_cjk_font(
            &mut fonts,
            EGUI_CJK_FONT_NAME.to_string(),
            egui::FontData::from_static(EGUI_CJK_FONT_BYTES),
        ),
    }
    context.set_fonts(fonts);
}

fn load_font_override_from_env() -> Option<(String, egui::FontData)> {
    let path = env::var(CLIENT_LAUNCHER_FONT_ENV).ok()?;
    let path = path.trim();
    if path.is_empty() {
        return None;
    }

    match std::fs::read(path) {
        Ok(bytes) => Some((
            format!("{EGUI_CJK_FONT_NAME}-custom"),
            egui::FontData::from_owned(bytes),
        )),
        Err(err) => {
            eprintln!(
                "warning: failed to read font from {CLIENT_LAUNCHER_FONT_ENV}={path}: {err}; fallback to embedded CJK font"
            );
            None
        }
    }
}

fn install_cjk_font(
    fonts: &mut egui::FontDefinitions,
    font_name: String,
    font_data: egui::FontData,
) {
    fonts
        .font_data
        .insert(font_name.clone(), Arc::new(font_data));

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, font_name.clone());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push(font_name);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UiLanguage {
    ZhCn,
    EnUs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GlossaryTerm {
    Nonce,
    Slot,
    Mempool,
    ActionId,
}

impl UiLanguage {
    fn from_tag(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "zh" | "zh-cn" | "zh_hans" | "zh-hans" | "cn" => Some(Self::ZhCn),
            "en" | "en-us" | "en_us" | "english" => Some(Self::EnUs),
            _ => None,
        }
    }

    fn detect_from_env() -> Self {
        if let Ok(raw) = env::var(CLIENT_LAUNCHER_LANG_ENV) {
            if let Some(language) = Self::from_tag(raw.as_str()) {
                return language;
            }
        }

        if let Ok(raw) = env::var("LANG") {
            if let Some(language) = Self::from_tag(raw.as_str()) {
                return language;
            }
        }

        Self::ZhCn
    }

    fn display_name(self) -> &'static str {
        match self {
            Self::ZhCn => "中文",
            Self::EnUs => "English",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
struct LaunchConfig {
    scenario: String,
    live_bind: String,
    web_bind: String,
    viewer_host: String,
    viewer_port: String,
    viewer_static_dir: String,
    llm_enabled: bool,
    chain_enabled: bool,
    chain_status_bind: String,
    chain_node_id: String,
    chain_world_id: String,
    chain_node_role: String,
    chain_node_tick_ms: String,
    chain_pos_slot_duration_ms: String,
    chain_pos_ticks_per_slot: String,
    chain_pos_proposal_tick_phase: String,
    chain_pos_adaptive_tick_scheduler_enabled: bool,
    chain_pos_slot_clock_genesis_unix_ms: String,
    chain_pos_max_past_slot_lag: String,
    chain_node_validators: String,
    auto_open_browser: bool,
    launcher_bin: String,
    chain_runtime_bin: String,
}

impl Default for LaunchConfig {
    fn default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let launcher_bin = resolve_launcher_binary_path().to_string_lossy().to_string();
        #[cfg(target_arch = "wasm32")]
        let launcher_bin = String::new();
        #[cfg(not(target_arch = "wasm32"))]
        let chain_runtime_bin = resolve_chain_runtime_binary_path()
            .to_string_lossy()
            .to_string();
        #[cfg(target_arch = "wasm32")]
        let chain_runtime_bin = String::new();
        let viewer_static_dir = resolve_static_dir_path().to_string_lossy().to_string();

        Self {
            scenario: DEFAULT_SCENARIO.to_string(),
            live_bind: DEFAULT_LIVE_BIND.to_string(),
            web_bind: DEFAULT_WEB_BIND.to_string(),
            viewer_host: DEFAULT_VIEWER_HOST.to_string(),
            viewer_port: DEFAULT_VIEWER_PORT.to_string(),
            viewer_static_dir,
            llm_enabled: true,
            chain_enabled: true,
            chain_status_bind: DEFAULT_CHAIN_STATUS_BIND.to_string(),
            chain_node_id: default_chain_node_id(),
            chain_world_id: String::new(),
            chain_node_role: DEFAULT_CHAIN_NODE_ROLE.to_string(),
            chain_node_tick_ms: DEFAULT_CHAIN_NODE_TICK_MS.to_string(),
            chain_pos_slot_duration_ms: DEFAULT_CHAIN_POS_SLOT_DURATION_MS.to_string(),
            chain_pos_ticks_per_slot: DEFAULT_CHAIN_POS_TICKS_PER_SLOT.to_string(),
            chain_pos_proposal_tick_phase: DEFAULT_CHAIN_POS_PROPOSAL_TICK_PHASE.to_string(),
            chain_pos_adaptive_tick_scheduler_enabled: false,
            chain_pos_slot_clock_genesis_unix_ms: DEFAULT_CHAIN_POS_SLOT_CLOCK_GENESIS_UNIX_MS
                .to_string(),
            chain_pos_max_past_slot_lag: DEFAULT_CHAIN_POS_MAX_PAST_SLOT_LAG.to_string(),
            chain_node_validators: String::new(),
            auto_open_browser: true,
            launcher_bin,
            chain_runtime_bin,
        }
    }
}

#[derive(Debug)]
#[cfg(not(target_arch = "wasm32"))]
struct RunningProcess {
    child: Child,
    log_rx: Receiver<String>,
}

#[derive(Debug, Clone)]
enum WebApiEvent {
    State(Result<WebStateSnapshot, String>),
    Action(Result<WebApiResponse, String>),
    #[cfg(target_arch = "wasm32")]
    Feedback(Result<WebFeedbackSubmitResponse, String>),
    Transfer(Result<WebTransferSubmitResponse, String>),
    TransferQuery(Result<transfer_window::TransferQueryResponse, String>),
    ExplorerQuery(Result<explorer_window::ExplorerQueryResponse, String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WebRequestDomain {
    StatePoll,
    ControlAction,
    FeedbackSubmit,
    TransferSubmit,
    TransferQuery,
    ExplorerQuery,
}

#[derive(Debug, Clone, Copy, Default)]
struct WebRequestInflight {
    state_poll: bool,
    control_action: bool,
    feedback_submit: bool,
    transfer_submit: bool,
    transfer_query: bool,
    explorer_query: bool,
}

impl WebRequestInflight {
    #[cfg(test)]
    fn any(self) -> bool {
        self.state_poll
            || self.control_action
            || self.feedback_submit
            || self.transfer_submit
            || self.transfer_query
            || self.explorer_query
    }

    fn transfer_any(self) -> bool {
        self.transfer_submit || self.transfer_query
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct WebChainRecoverySnapshot {
    error_code: String,
    reason: String,
    node_id: String,
    execution_world_dir: String,
    recovery_mode: String,
    reset_required: bool,
    fresh_node_id: String,
    fresh_chain_status_bind: String,
    suggested_config: LaunchConfig,
}

#[derive(Debug, Clone, Deserialize)]
struct WebStateSnapshot {
    status: String,
    detail: Option<String>,
    chain_status: String,
    chain_detail: Option<String>,
    chain_recovery: Option<WebChainRecoverySnapshot>,
    game_url: String,
    config: LaunchConfig,
    logs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct WebApiResponse {
    ok: bool,
    error: Option<String>,
    state: WebStateSnapshot,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Serialize)]
struct WebFeedbackSubmitRequest {
    category: String,
    title: String,
    description: String,
    platform: String,
    game_version: String,
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Deserialize)]
struct WebFeedbackSubmitResponse {
    ok: bool,
    feedback_id: Option<String>,
    event_id: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct WebTransferSubmitRequest {
    from_account_id: String,
    to_account_id: String,
    amount: u64,
    nonce: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct WebTransferSubmitResponse {
    ok: bool,
    action_id: Option<u64>,
    submitted_at_unix_ms: Option<i64>,
    lifecycle_status: Option<transfer_window::WebTransferLifecycleStatus>,
    error_code: Option<String>,
    error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LauncherStatus {
    Idle,
    Running,
    Stopped,
    InvalidArgs,
    StartFailed,
    StopFailed,
    QueryFailed,
    Exited(String),
}

impl LauncherStatus {
    fn text(&self, language: UiLanguage) -> String {
        match (self, language) {
            (Self::Idle, UiLanguage::ZhCn) => "未启动".to_string(),
            (Self::Idle, UiLanguage::EnUs) => "Not Started".to_string(),
            (Self::Running, UiLanguage::ZhCn) => "运行中".to_string(),
            (Self::Running, UiLanguage::EnUs) => "Running".to_string(),
            (Self::Stopped, UiLanguage::ZhCn) => "已停止".to_string(),
            (Self::Stopped, UiLanguage::EnUs) => "Stopped".to_string(),
            (Self::InvalidArgs, UiLanguage::ZhCn) => "参数非法".to_string(),
            (Self::InvalidArgs, UiLanguage::EnUs) => "Invalid Config".to_string(),
            (Self::StartFailed, UiLanguage::ZhCn) => "启动失败".to_string(),
            (Self::StartFailed, UiLanguage::EnUs) => "Start Failed".to_string(),
            (Self::StopFailed, UiLanguage::ZhCn) => "停止失败".to_string(),
            (Self::StopFailed, UiLanguage::EnUs) => "Stop Failed".to_string(),
            (Self::QueryFailed, UiLanguage::ZhCn) => "状态查询失败".to_string(),
            (Self::QueryFailed, UiLanguage::EnUs) => "Status Query Failed".to_string(),
            (Self::Exited(reason), UiLanguage::ZhCn) => format!("已退出: {reason}"),
            (Self::Exited(reason), UiLanguage::EnUs) => format!("Exited: {reason}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ChainRuntimeStatus {
    Disabled,
    NotStarted,
    Starting,
    Ready,
    StaleExecutionWorld(String),
    Unreachable(String),
    ConfigError(String),
}

impl ChainRuntimeStatus {
    fn text(&self, language: UiLanguage) -> &'static str {
        match (self, language) {
            (Self::Disabled, UiLanguage::ZhCn) => "已禁用",
            (Self::Disabled, UiLanguage::EnUs) => "Disabled",
            (Self::NotStarted, UiLanguage::ZhCn) => "未启动",
            (Self::NotStarted, UiLanguage::EnUs) => "Not Started",
            (Self::Starting, UiLanguage::ZhCn) => "启动中",
            (Self::Starting, UiLanguage::EnUs) => "Starting",
            (Self::Ready, UiLanguage::ZhCn) => "已就绪",
            (Self::Ready, UiLanguage::EnUs) => "Ready",
            (Self::StaleExecutionWorld(_), UiLanguage::ZhCn) => "旧执行世界冲突",
            (Self::StaleExecutionWorld(_), UiLanguage::EnUs) => "Stale World Conflict",
            (Self::Unreachable(_), UiLanguage::ZhCn) => "不可达",
            (Self::Unreachable(_), UiLanguage::EnUs) => "Unreachable",
            (Self::ConfigError(_), UiLanguage::ZhCn) => "配置错误",
            (Self::ConfigError(_), UiLanguage::EnUs) => "Config Error",
        }
    }

    fn color(&self) -> egui::Color32 {
        match self {
            Self::Disabled | Self::NotStarted => egui::Color32::from_rgb(130, 130, 130),
            Self::Starting => egui::Color32::from_rgb(201, 146, 44),
            Self::Ready => egui::Color32::from_rgb(62, 152, 92),
            Self::StaleExecutionWorld(_) => egui::Color32::from_rgb(196, 84, 84),
            Self::Unreachable(_) | Self::ConfigError(_) => egui::Color32::from_rgb(196, 84, 84),
        }
    }

    fn detail(&self) -> Option<&str> {
        match self {
            Self::StaleExecutionWorld(detail)
            | Self::Unreachable(detail)
            | Self::ConfigError(detail) => Some(detail.as_str()),
            Self::Disabled | Self::NotStarted | Self::Starting | Self::Ready => None,
        }
    }
}

fn launcher_status_from_web(status: &str, detail: Option<&str>) -> LauncherStatus {
    match status {
        "idle" => LauncherStatus::Idle,
        "running" => LauncherStatus::Running,
        "stopped" => LauncherStatus::Stopped,
        "invalid_config" => LauncherStatus::InvalidArgs,
        "start_failed" => LauncherStatus::StartFailed,
        "stop_failed" => LauncherStatus::StopFailed,
        "exited" => LauncherStatus::Exited(detail.unwrap_or("unknown").to_string()),
        _ => LauncherStatus::QueryFailed,
    }
}

fn chain_runtime_status_from_web(status: &str, detail: Option<&str>) -> ChainRuntimeStatus {
    match status {
        "disabled" => ChainRuntimeStatus::Disabled,
        "not_started" => ChainRuntimeStatus::NotStarted,
        "starting" => ChainRuntimeStatus::Starting,
        "ready" => ChainRuntimeStatus::Ready,
        "stale_execution_world" => {
            ChainRuntimeStatus::StaleExecutionWorld(detail.unwrap_or("unknown").to_string())
        }
        "unreachable" => ChainRuntimeStatus::Unreachable(detail.unwrap_or("unknown").to_string()),
        "config_error" => ChainRuntimeStatus::ConfigError(detail.unwrap_or("unknown").to_string()),
        _ => ChainRuntimeStatus::Unreachable(format!("unknown chain status: {status}")),
    }
}

fn encode_query_value(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push(hex_upper(byte >> 4));
            encoded.push(hex_upper(byte & 0x0f));
        }
    }
    encoded
}

fn encoded_query_pair(key: &str, value: &str) -> String {
    format!("{key}={}", encode_query_value(value))
}

fn hex_upper(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        10..=15 => (b'A' + (nibble - 10)) as char,
        _ => unreachable!("nibble must be in 0..=15"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigIssue {
    ScenarioRequired,
    LiveBindInvalid,
    WebBindInvalid,
    ViewerHostRequired,
    ViewerPortInvalid,
    ViewerStaticDirRequired,
    ViewerStaticDirMissing,
    LauncherBinRequired,
    LauncherBinMissing,
    ChainRuntimeBinRequired,
    ChainRuntimeBinMissing,
    ChainStatusBindInvalid,
    ChainNodeIdRequired,
    ChainRoleInvalid,
    ChainTickMsInvalid,
    ChainPosSlotDurationMsInvalid,
    ChainPosTicksPerSlotInvalid,
    ChainPosProposalTickPhaseInvalid,
    ChainPosProposalTickPhaseOutOfRange,
    ChainPosSlotClockGenesisUnixMsInvalid,
    ChainPosMaxPastSlotLagInvalid,
    ChainValidatorsInvalid,
}

impl ConfigIssue {
    fn text(self, language: UiLanguage) -> &'static str {
        match (self, language) {
            (Self::ScenarioRequired, UiLanguage::ZhCn) => "场景（scenario）是必填项",
            (Self::ScenarioRequired, UiLanguage::EnUs) => "Scenario is required",
            (Self::LiveBindInvalid, UiLanguage::ZhCn) => "实时服务绑定必须是 <host:port>",
            (Self::LiveBindInvalid, UiLanguage::EnUs) => "Live bind must be in <host:port> format",
            (Self::WebBindInvalid, UiLanguage::ZhCn) => "WebSocket 绑定必须是 <host:port>",
            (Self::WebBindInvalid, UiLanguage::EnUs) => "Web bind must be in <host:port> format",
            (Self::ViewerHostRequired, UiLanguage::ZhCn) => "游戏页面主机（viewer host）是必填项",
            (Self::ViewerHostRequired, UiLanguage::EnUs) => "Viewer host is required",
            (Self::ViewerPortInvalid, UiLanguage::ZhCn) => {
                "游戏页面端口（viewer port）必须在 1..=65535"
            }
            (Self::ViewerPortInvalid, UiLanguage::EnUs) => {
                "Viewer port must be an integer in 1..=65535"
            }
            (Self::ViewerStaticDirRequired, UiLanguage::ZhCn) => {
                "前端静态资源目录（viewer static dir）是必填项"
            }
            (Self::ViewerStaticDirRequired, UiLanguage::EnUs) => {
                "Viewer static directory is required"
            }
            (Self::ViewerStaticDirMissing, UiLanguage::ZhCn) => "前端静态资源目录不存在或不是目录",
            (Self::ViewerStaticDirMissing, UiLanguage::EnUs) => {
                "Viewer static directory does not exist or is not a directory"
            }
            (Self::LauncherBinRequired, UiLanguage::ZhCn) => {
                "启动器二进制路径（launcher bin）是必填项"
            }
            (Self::LauncherBinRequired, UiLanguage::EnUs) => "Launcher binary path is required",
            (Self::LauncherBinMissing, UiLanguage::ZhCn) => "启动器二进制文件不存在",
            (Self::LauncherBinMissing, UiLanguage::EnUs) => "Launcher binary file does not exist",
            (Self::ChainRuntimeBinRequired, UiLanguage::ZhCn) => {
                "链运行时二进制路径（chain runtime bin）是必填项"
            }
            (Self::ChainRuntimeBinRequired, UiLanguage::EnUs) => {
                "Chain runtime binary path is required"
            }
            (Self::ChainRuntimeBinMissing, UiLanguage::ZhCn) => "链运行时二进制文件不存在",
            (Self::ChainRuntimeBinMissing, UiLanguage::EnUs) => {
                "Chain runtime binary file does not exist"
            }
            (Self::ChainStatusBindInvalid, UiLanguage::ZhCn) => "链状态服务绑定必须是 <host:port>",
            (Self::ChainStatusBindInvalid, UiLanguage::EnUs) => {
                "Chain status bind must be in <host:port> format"
            }
            (Self::ChainNodeIdRequired, UiLanguage::ZhCn) => "链节点 ID（chain node id）是必填项",
            (Self::ChainNodeIdRequired, UiLanguage::EnUs) => "Chain node id is required",
            (Self::ChainRoleInvalid, UiLanguage::ZhCn) => {
                "链节点角色必须是 sequencer/storage/observer"
            }
            (Self::ChainRoleInvalid, UiLanguage::EnUs) => {
                "Chain role must be one of: sequencer/storage/observer"
            }
            (Self::ChainTickMsInvalid, UiLanguage::ZhCn) => {
                "链节点轮询间隔毫秒（chain node poll interval ms）必须是正整数"
            }
            (Self::ChainTickMsInvalid, UiLanguage::EnUs) => {
                "Chain node poll interval milliseconds must be a positive integer"
            }
            (Self::ChainPosSlotDurationMsInvalid, UiLanguage::ZhCn) => {
                "链 PoS 槽时长（slot duration ms）必须是正整数"
            }
            (Self::ChainPosSlotDurationMsInvalid, UiLanguage::EnUs) => {
                "Chain PoS slot duration ms must be a positive integer"
            }
            (Self::ChainPosTicksPerSlotInvalid, UiLanguage::ZhCn) => {
                "链 PoS 每槽 tick 数（ticks per slot）必须是正整数"
            }
            (Self::ChainPosTicksPerSlotInvalid, UiLanguage::EnUs) => {
                "Chain PoS ticks per slot must be a positive integer"
            }
            (Self::ChainPosProposalTickPhaseInvalid, UiLanguage::ZhCn) => {
                "链 PoS 提案相位（proposal tick phase）必须是非负整数"
            }
            (Self::ChainPosProposalTickPhaseInvalid, UiLanguage::EnUs) => {
                "Chain PoS proposal tick phase must be a non-negative integer"
            }
            (Self::ChainPosProposalTickPhaseOutOfRange, UiLanguage::ZhCn) => {
                "链 PoS 提案相位必须小于每槽 tick 数"
            }
            (Self::ChainPosProposalTickPhaseOutOfRange, UiLanguage::EnUs) => {
                "Chain PoS proposal tick phase must be less than ticks per slot"
            }
            (Self::ChainPosSlotClockGenesisUnixMsInvalid, UiLanguage::ZhCn) => {
                "链 PoS 槽时钟起点（slot clock genesis unix ms）必须是整数或留空"
            }
            (Self::ChainPosSlotClockGenesisUnixMsInvalid, UiLanguage::EnUs) => {
                "Chain PoS slot clock genesis unix ms must be an integer or empty"
            }
            (Self::ChainPosMaxPastSlotLagInvalid, UiLanguage::ZhCn) => {
                "链 PoS 允许过旧槽滞后（max past slot lag）必须是非负整数"
            }
            (Self::ChainPosMaxPastSlotLagInvalid, UiLanguage::EnUs) => {
                "Chain PoS max past slot lag must be a non-negative integer"
            }
            (Self::ChainValidatorsInvalid, UiLanguage::ZhCn) => {
                "链验证者（chain validators）格式必须是 <validator_id:stake>"
            }
            (Self::ChainValidatorsInvalid, UiLanguage::EnUs) => {
                "Chain validators must be in <validator_id:stake> format"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(target_arch = "wasm32")]
enum FeedbackKind {
    Bug,
    Suggestion,
}

#[cfg(target_arch = "wasm32")]
impl FeedbackKind {
    fn slug(self) -> &'static str {
        match self {
            Self::Bug => "bug",
            Self::Suggestion => "suggestion",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(target_arch = "wasm32")]
struct FeedbackDraft {
    kind: FeedbackKind,
    title: String,
    description: String,
}

#[cfg(target_arch = "wasm32")]
impl Default for FeedbackDraft {
    fn default() -> Self {
        Self {
            kind: FeedbackKind::Bug,
            title: String::new(),
            description: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FeedbackSubmitState {
    None,
    Success(String),
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(target_arch = "wasm32")]
struct TransferDraft {
    from_account_id: String,
    to_account_id: String,
    amount: String,
    nonce: String,
}

#[cfg(target_arch = "wasm32")]
impl Default for TransferDraft {
    fn default() -> Self {
        Self {
            from_account_id: String::new(),
            to_account_id: String::new(),
            amount: "1".to_string(),
            nonce: "1".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TransferSubmitState {
    None,
    Success(String),
    Failed(String),
}

#[derive(Debug)]
struct ClientLauncherApp {
    config: LaunchConfig,
    config_dirty: bool,
    llm_settings_panel: LlmSettingsPanel,
    ui_language: UiLanguage,
    status: LauncherStatus,
    chain_runtime_status: ChainRuntimeStatus,
    chain_recovery: Option<WebChainRecoverySnapshot>,
    #[cfg(not(target_arch = "wasm32"))]
    running: Option<RunningProcess>,
    chain_auto_start_attempted: bool,
    logs: VecDeque<String>,
    feedback_draft: FeedbackDraft,
    feedback_submit_state: FeedbackSubmitState,
    feedback_window_open: bool,
    onboarding_state: OnboardingState,
    ux_state: LauncherUxState,
    demo_mode_phase: DemoModePhase,
    guidance_insights_open: bool,
    startup_guide_state: StartupGuideState,
    config_window_open: bool,
    transfer_draft: TransferDraft,
    transfer_submit_state: TransferSubmitState,
    transfer_window_open: bool,
    transfer_panel_state: transfer_window::TransferPanelState,
    explorer_window_open: bool,
    explorer_panel_state: explorer_window::ExplorerPanelState,
    web_api_tx: Sender<WebApiEvent>,
    web_api_rx: Receiver<WebApiEvent>,
    web_request_inflight: WebRequestInflight,
    last_web_poll_at: Option<Instant>,
    web_game_url: Option<String>,
    #[cfg(not(target_arch = "wasm32"))]
    control_api_base: String,
    #[cfg(not(target_arch = "wasm32"))]
    control_listen_bind: String,
    #[cfg(not(target_arch = "wasm32"))]
    control_manage_service: bool,
}

impl Default for ClientLauncherApp {
    fn default() -> Self {
        let config = LaunchConfig::default();
        let ux_state = self_guided::load_launcher_ux_state();
        let onboarding_state = OnboardingState::from_persisted(
            ux_state.onboarding_completed,
            ux_state.onboarding_dismissed,
        );
        let (web_api_tx, web_api_rx) = mpsc::channel::<WebApiEvent>();
        #[cfg(not(target_arch = "wasm32"))]
        let control_url_from_env = env::var(CLIENT_LAUNCHER_CONTROL_URL_ENV)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        #[cfg(not(target_arch = "wasm32"))]
        let control_listen_bind = env::var(CLIENT_LAUNCHER_CONTROL_BIND_ENV)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| DEFAULT_CLIENT_LAUNCHER_CONTROL_BIND.to_string());
        #[cfg(not(target_arch = "wasm32"))]
        let control_api_base = control_url_from_env.clone().unwrap_or_else(|| {
            let (host, port) = parse_host_port(
                control_listen_bind.as_str(),
                CLIENT_LAUNCHER_CONTROL_BIND_ENV,
            )
            .unwrap_or(("127.0.0.1".to_string(), 5410));
            let host = normalize_host_for_url(host.as_str());
            let host = host_for_url(host.as_str());
            format!("http://{host}:{port}")
        });
        #[cfg(not(target_arch = "wasm32"))]
        let control_manage_service = control_url_from_env.is_none();
        let chain_runtime_status = if config.chain_enabled {
            ChainRuntimeStatus::NotStarted
        } else {
            ChainRuntimeStatus::Disabled
        };
        Self {
            config,
            config_dirty: false,
            llm_settings_panel: LlmSettingsPanel::new(LlmSettingsPanel::default_path()),
            ui_language: UiLanguage::detect_from_env(),
            status: LauncherStatus::Idle,
            chain_runtime_status,
            chain_recovery: None,
            #[cfg(not(target_arch = "wasm32"))]
            running: None,
            chain_auto_start_attempted: false,
            logs: VecDeque::new(),
            feedback_draft: FeedbackDraft::default(),
            feedback_submit_state: FeedbackSubmitState::None,
            feedback_window_open: false,
            onboarding_state,
            ux_state,
            demo_mode_phase: DemoModePhase::Idle,
            guidance_insights_open: false,
            startup_guide_state: StartupGuideState::default(),
            config_window_open: false,
            transfer_draft: TransferDraft::default(),
            transfer_submit_state: TransferSubmitState::None,
            transfer_window_open: false,
            transfer_panel_state: transfer_window::TransferPanelState::default(),
            explorer_window_open: false,
            explorer_panel_state: explorer_window::ExplorerPanelState::default(),
            web_api_tx,
            web_api_rx,
            web_request_inflight: WebRequestInflight::default(),
            last_web_poll_at: None,
            web_game_url: None,
            #[cfg(not(target_arch = "wasm32"))]
            control_api_base,
            #[cfg(not(target_arch = "wasm32"))]
            control_listen_bind,
            #[cfg(not(target_arch = "wasm32"))]
            control_manage_service,
        }
    }
}

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;
