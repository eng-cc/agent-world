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
use std::time::Duration;
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
mod platform_ops;
mod self_guided;
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
            chain_node_id: DEFAULT_CHAIN_NODE_ID.to_string(),
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

#[derive(Debug, Clone, Deserialize)]
struct WebStateSnapshot {
    status: String,
    detail: Option<String>,
    chain_status: String,
    chain_detail: Option<String>,
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
            Self::Unreachable(_) | Self::ConfigError(_) => egui::Color32::from_rgb(196, 84, 84),
        }
    }

    fn detail(&self) -> Option<&str> {
        match self {
            Self::Unreachable(detail) | Self::ConfigError(detail) => Some(detail.as_str()),
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
        let onboarding_state = OnboardingState::from_persisted(ux_state.onboarding_completed);
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

impl ClientLauncherApp {
    fn tr<'a>(&self, zh: &'a str, en: &'a str) -> &'a str {
        match self.ui_language {
            UiLanguage::ZhCn => zh,
            UiLanguage::EnUs => en,
        }
    }

    fn glossary_term_text(&self, term: GlossaryTerm) -> &'static str {
        match term {
            GlossaryTerm::Nonce => "nonce",
            GlossaryTerm::Slot => "slot",
            GlossaryTerm::Mempool => "mempool",
            GlossaryTerm::ActionId => "action_id",
        }
    }

    fn glossary_term_definition(&self, term: GlossaryTerm) -> &'static str {
        match (term, self.ui_language) {
            (GlossaryTerm::Nonce, UiLanguage::ZhCn) => {
                "每个账户的递增序号，用于防重放；通常使用 next_nonce_hint。"
            }
            (GlossaryTerm::Nonce, UiLanguage::EnUs) => {
                "Per-account increasing sequence to prevent replay; usually use next_nonce_hint."
            }
            (GlossaryTerm::Slot, UiLanguage::ZhCn) => {
                "链出块时间片编号；多个 tick 组成一个 slot，用于排序区块时间。"
            }
            (GlossaryTerm::Slot, UiLanguage::EnUs) => {
                "Block time window index; multiple ticks form one slot for chain ordering."
            }
            (GlossaryTerm::Mempool, UiLanguage::ZhCn) => {
                "待打包交易池，包含 accepted/pending 状态的交易。"
            }
            (GlossaryTerm::Mempool, UiLanguage::EnUs) => {
                "Queue of transactions waiting to be packed, including accepted/pending states."
            }
            (GlossaryTerm::ActionId, UiLanguage::ZhCn) => {
                "链内动作编号，可用于精确追踪单笔转账状态与查询。"
            }
            (GlossaryTerm::ActionId, UiLanguage::EnUs) => {
                "On-chain action identifier for tracking one transfer lifecycle and queries."
            }
        }
    }

    fn render_glossary_term_chip(&self, ui: &mut egui::Ui, term: GlossaryTerm) {
        ui.label(
            egui::RichText::new(self.glossary_term_text(term))
                .underline()
                .color(egui::Color32::from_rgb(74, 116, 168)),
        )
        .on_hover_text(self.glossary_term_definition(term));
    }

    fn append_log<S: Into<String>>(&mut self, line: S) {
        self.logs.push_back(line.into());
        while self.logs.len() > MAX_LOG_LINES {
            self.logs.pop_front();
        }
    }

    fn web_request_inflight_for(&self, domain: WebRequestDomain) -> bool {
        match domain {
            WebRequestDomain::StatePoll => self.web_request_inflight.state_poll,
            WebRequestDomain::ControlAction => self.web_request_inflight.control_action,
            WebRequestDomain::FeedbackSubmit => self.web_request_inflight.feedback_submit,
            WebRequestDomain::TransferSubmit => self.web_request_inflight.transfer_submit,
            WebRequestDomain::TransferQuery => self.web_request_inflight.transfer_query,
            WebRequestDomain::ExplorerQuery => self.web_request_inflight.explorer_query,
        }
    }

    fn set_web_request_inflight(&mut self, domain: WebRequestDomain, inflight: bool) {
        match domain {
            WebRequestDomain::StatePoll => self.web_request_inflight.state_poll = inflight,
            WebRequestDomain::ControlAction => self.web_request_inflight.control_action = inflight,
            WebRequestDomain::FeedbackSubmit => {
                self.web_request_inflight.feedback_submit = inflight;
            }
            WebRequestDomain::TransferSubmit => {
                self.web_request_inflight.transfer_submit = inflight;
            }
            WebRequestDomain::TransferQuery => self.web_request_inflight.transfer_query = inflight,
            WebRequestDomain::ExplorerQuery => self.web_request_inflight.explorer_query = inflight,
        }
    }

    #[cfg(test)]
    fn any_web_request_inflight(&self) -> bool {
        self.web_request_inflight.any()
    }

    fn any_transfer_request_inflight(&self) -> bool {
        self.web_request_inflight.transfer_any()
    }

    #[cfg(target_arch = "wasm32")]
    fn apply_web_feedback_submit_result(
        &mut self,
        result: Result<WebFeedbackSubmitResponse, String>,
    ) {
        match result {
            Ok(response) => {
                if response.ok {
                    let feedback_id = response.feedback_id.unwrap_or_else(|| "n/a".to_string());
                    let event_id = response.event_id.unwrap_or_else(|| "n/a".to_string());
                    let message = format!(
                        "{}: feedback_id={feedback_id}, event_id={event_id}",
                        self.tr(
                            "反馈已提交到分布式网络",
                            "Feedback submitted to distributed network"
                        )
                    );
                    self.append_log(message.clone());
                    self.feedback_submit_state = FeedbackSubmitState::Success(message);
                } else {
                    let error_text = response
                        .error
                        .unwrap_or_else(|| self.tr("未知错误", "Unknown error").to_string());
                    let message = format!(
                        "{}: {error_text}",
                        self.tr("反馈提交被拒绝", "Feedback submit rejected")
                    );
                    self.append_log(message.clone());
                    self.feedback_submit_state = FeedbackSubmitState::Failed(message);
                }
            }
            Err(err) => {
                let message = format!(
                    "{}: {err}",
                    self.tr("反馈提交失败", "Feedback submit failed")
                );
                self.append_log(message.clone());
                self.feedback_submit_state = FeedbackSubmitState::Failed(message);
            }
        }
    }

    fn ui_field_label(&self, field: &LauncherUiField) -> &'static str {
        match self.ui_language {
            UiLanguage::ZhCn => field.label_zh,
            UiLanguage::EnUs => field.label_en,
        }
    }

    fn render_config_field(
        &mut self,
        ui: &mut egui::Ui,
        field: &LauncherUiField,
        stack_text_fields: bool,
    ) {
        let label = self.ui_field_label(field);
        match field.kind {
            LauncherUiFieldKind::Text => {
                if let Some(value) = launcher_text_field_mut(&mut self.config, field.id) {
                    if stack_text_fields {
                        ui.vertical(|ui| {
                            ui.label(label);
                            let response = ui.add_sized(
                                [ui.available_width(), 0.0],
                                egui::TextEdit::singleline(value),
                            );
                            if response.changed() {
                                self.config_dirty = true;
                            }
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label(label);
                            if ui.text_edit_singleline(value).changed() {
                                self.config_dirty = true;
                            }
                        });
                    }
                }
            }
            LauncherUiFieldKind::Checkbox => {
                if let Some(value) = launcher_checkbox_field_mut(&mut self.config, field.id) {
                    if ui.checkbox(value, label).changed() {
                        self.config_dirty = true;
                    }
                }
            }
        }
    }

    fn render_config_section(&mut self, ui: &mut egui::Ui, section: &str) {
        let stack_text_fields = ui.available_width() <= 560.0;
        ui.vertical(|ui| {
            #[cfg(not(target_arch = "wasm32"))]
            {
                for field in
                    launcher_ui_fields_for_native().filter(|field| field.section == section)
                {
                    self.render_config_field(ui, field, stack_text_fields);
                }
            }

            #[cfg(target_arch = "wasm32")]
            {
                for field in launcher_ui_fields_for_web().filter(|field| field.section == section) {
                    self.render_config_field(ui, field, stack_text_fields);
                }
            }
        });
    }

    fn render_config_validation_summary(
        &mut self,
        ui: &mut egui::Ui,
        game_required_issues: &[ConfigIssue],
        chain_required_issues: &[ConfigIssue],
    ) {
        let chain_issue_count = if self.config.chain_enabled {
            chain_required_issues.len()
        } else {
            0
        };
        let has_issue = !game_required_issues.is_empty() || chain_issue_count > 0;

        ui.horizontal_wrapped(|ui| {
            ui.label(self.tr(
                "低频配置已收口到高级配置弹窗。",
                "Low-frequency settings are grouped in Advanced Config.",
            ));
            if ui.button(self.tr("高级配置", "Advanced Config")).clicked() {
                self.config_window_open = true;
            }
        });

        if self.config_dirty {
            ui.small(
                egui::RichText::new(self.tr(
                    "检测到本地配置改动：轮询快照不会覆盖当前编辑，直到配置与服务端一致。",
                    "Local config edits detected: polling snapshots will not overwrite current edits until they match server config.",
                ))
                .color(egui::Color32::from_rgb(201, 146, 44)),
            );
        }

        if !has_issue {
            ui.colored_label(
                egui::Color32::from_rgb(36, 130, 78),
                self.tr(
                    "当前配置校验通过，可直接执行高频操作。",
                    "Configuration checks passed; quick actions are ready.",
                ),
            );
            return;
        }

        let summary = if self.config.chain_enabled {
            match self.ui_language {
                UiLanguage::ZhCn => format!(
                    "存在配置问题：游戏 {} 项，区块链 {} 项",
                    game_required_issues.len(),
                    chain_issue_count
                ),
                UiLanguage::EnUs => format!(
                    "Configuration issues detected: game {}, blockchain {}",
                    game_required_issues.len(),
                    chain_issue_count
                ),
            }
        } else {
            match self.ui_language {
                UiLanguage::ZhCn => format!("存在配置问题：游戏 {} 项", game_required_issues.len()),
                UiLanguage::EnUs => format!(
                    "Configuration issues detected: game {}",
                    game_required_issues.len()
                ),
            }
        };
        ui.colored_label(egui::Color32::from_rgb(188, 60, 60), summary);
        ui.small(self.tr(
            "请点击“高级配置”查看并修复具体字段。",
            "Open Advanced Config to review and fix specific fields.",
        ));
    }

    fn show_config_window(
        &mut self,
        ctx: &egui::Context,
        game_required_issues: &[ConfigIssue],
        chain_required_issues: &[ConfigIssue],
    ) {
        if !self.config_window_open {
            return;
        }

        let mut keep_open = self.config_window_open;
        egui::Window::new(self.tr("高级配置", "Advanced Config"))
            .collapsible(false)
            .resizable(true)
            .default_width(780.0)
            .default_height(640.0)
            .open(&mut keep_open)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for section in NATIVE_UI_SECTIONS {
                            self.render_config_section(ui, section);
                        }
                    });

                ui.separator();

                if game_required_issues.is_empty() {
                    ui.colored_label(
                        egui::Color32::from_rgb(36, 130, 78),
                        self.tr(
                            "必填配置项已通过校验，可启动游戏",
                            "Required configuration check passed; game can start",
                        ),
                    );
                } else {
                    ui.group(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(188, 60, 60),
                            self.tr(
                                "游戏启动前请先修复以下必填配置项：",
                                "Fix the required game configuration issues before starting:",
                            ),
                        );
                        for issue in game_required_issues {
                            ui.label(format!("- {}", issue.text(self.ui_language)));
                        }
                    });
                }

                if self.config.chain_enabled && !chain_required_issues.is_empty() {
                    ui.group(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(188, 60, 60),
                            self.tr(
                                "区块链启动前请先修复以下配置项：",
                                "Fix the blockchain configuration issues before starting:",
                            ),
                        );
                        for issue in chain_required_issues {
                            ui.label(format!("- {}", issue.text(self.ui_language)));
                        }
                    });
                }
            });
        self.config_window_open = keep_open;
    }

    fn feedback_unavailable_hint(&self) -> Option<String> {
        if self.is_feedback_available() {
            return None;
        }
        let message = match (&self.chain_runtime_status, self.ui_language) {
            (ChainRuntimeStatus::Disabled, UiLanguage::ZhCn) => {
                "反馈/转账/浏览器功能已禁用：区块链功能关闭".to_string()
            }
            (ChainRuntimeStatus::Disabled, UiLanguage::EnUs) => {
                "Feedback/Transfer/Explorer are disabled because blockchain is disabled".to_string()
            }
            (ChainRuntimeStatus::NotStarted, UiLanguage::ZhCn) => {
                "反馈/转账/浏览器功能暂不可用：区块链未启动".to_string()
            }
            (ChainRuntimeStatus::NotStarted, UiLanguage::EnUs) => {
                "Feedback/Transfer/Explorer are unavailable because blockchain is not started"
                    .to_string()
            }
            (ChainRuntimeStatus::Starting, UiLanguage::ZhCn) => {
                "反馈/转账/浏览器功能暂不可用：区块链启动中".to_string()
            }
            (ChainRuntimeStatus::Starting, UiLanguage::EnUs) => {
                "Feedback/Transfer/Explorer are unavailable while blockchain is starting"
                    .to_string()
            }
            (ChainRuntimeStatus::Unreachable(detail), UiLanguage::ZhCn) => {
                format!("反馈/转账/浏览器功能暂不可用：区块链不可达（{detail}）")
            }
            (ChainRuntimeStatus::Unreachable(detail), UiLanguage::EnUs) => {
                format!(
                    "Feedback/Transfer/Explorer are unavailable: blockchain unreachable ({detail})"
                )
            }
            (ChainRuntimeStatus::ConfigError(detail), UiLanguage::ZhCn) => {
                format!("反馈/转账/浏览器功能暂不可用：区块链配置错误（{detail}）")
            }
            (ChainRuntimeStatus::ConfigError(detail), UiLanguage::EnUs) => {
                format!("Feedback/Transfer/Explorer are unavailable: blockchain config error ({detail})")
            }
            (ChainRuntimeStatus::Ready, UiLanguage::ZhCn) => {
                "反馈/转账/浏览器功能暂不可用：区块链功能关闭".to_string()
            }
            (ChainRuntimeStatus::Ready, UiLanguage::EnUs) => {
                "Feedback/Transfer/Explorer are unavailable: blockchain is disabled".to_string()
            }
        };
        Some(message)
    }
}

impl Drop for ClientLauncherApp {
    fn drop(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(mut running) = self.running.take() {
                let _ = stop_child_process(&mut running.child);
            }
        }
    }
}

impl eframe::App for ClientLauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_process();
        self.poll_chain_process();
        self.maybe_auto_start_chain();
        self.update_chain_runtime_status();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading(self.tr("Agent World 客户端启动器", "Agent World Client Launcher"));
                ui.separator();
                ui.label(format!(
                    "{}: {}",
                    self.tr("游戏", "Game"),
                    self.status.text(self.ui_language)
                ));
                ui.separator();
                let chain_status = format!(
                    "{}: {}",
                    self.tr("区块链", "Blockchain"),
                    self.chain_runtime_status.text(self.ui_language)
                );
                let response =
                    ui.colored_label(self.chain_runtime_status.color(), chain_status.as_str());
                if let Some(detail) = self.chain_runtime_status.detail() {
                    response.on_hover_text(detail);
                }
                ui.separator();
                ui.label(self.tr("语言", "Language"));
                egui::ComboBox::from_id_salt("launcher_language")
                    .selected_text(self.ui_language.display_name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.ui_language,
                            UiLanguage::ZhCn,
                            UiLanguage::ZhCn.display_name(),
                        );
                        ui.selectable_value(
                            &mut self.ui_language,
                            UiLanguage::EnUs,
                            UiLanguage::EnUs.display_name(),
                        );
                    });
                ui.separator();
                let mut expert_mode = self.is_expert_mode();
                if ui
                    .checkbox(&mut expert_mode, self.tr("专家模式", "Expert Mode"))
                    .changed()
                {
                    self.set_expert_mode(expert_mode);
                }
            });
        });

        let game_required_issues = collect_required_config_issues(&self.config);
        let chain_required_issues = collect_chain_required_config_issues(&self.config);
        let game_running = matches!(self.status, LauncherStatus::Running);
        let chain_running = matches!(
            self.chain_runtime_status,
            ChainRuntimeStatus::Starting | ChainRuntimeStatus::Ready
        );
        self.maybe_save_last_successful_config_profile(game_running);
        let can_click_start_game = !game_running;
        let can_click_start_chain = self.config.chain_enabled && !chain_running;
        self.maybe_open_onboarding_on_first_visit(
            &game_required_issues,
            &chain_required_issues,
            game_running,
            chain_running,
        );
        if self.onboarding_state.completed {
            self.maybe_open_startup_guide_on_first_check(
                &game_required_issues,
                &chain_required_issues,
            );
        }
        self.advance_demo_mode(
            &game_required_issues,
            &chain_required_issues,
            game_running,
            chain_running,
        );

        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_config_validation_summary(
                ui,
                &game_required_issues,
                &chain_required_issues,
            );

            ui.separator();
            if self.is_expert_mode() {
                ui.small(self.tr(
                    "专家模式已开启：已隐藏新手任务流卡片。",
                    "Expert mode enabled: guided task cards are hidden.",
                ));
            } else {
                self.render_task_flow_cards(
                    ui,
                    &game_required_issues,
                    &chain_required_issues,
                    game_running,
                    chain_running,
                );
            }
            ui.separator();

            ui.horizontal_wrapped(|ui| {
                if ui
                    .add_enabled(
                        can_click_start_game,
                        egui::Button::new(self.tr("启动游戏", "Start Game")),
                    )
                    .clicked()
                {
                    self.handle_start_game_click(&game_required_issues);
                }
                if ui
                    .add_enabled(
                        game_running,
                        egui::Button::new(self.tr("停止游戏", "Stop Game")),
                    )
                    .clicked()
                {
                    self.stop_process();
                }
                if ui
                    .add_enabled(
                        can_click_start_chain,
                        egui::Button::new(self.tr("启动区块链", "Start Blockchain")),
                    )
                    .clicked()
                {
                    self.handle_start_chain_click(&chain_required_issues);
                }
                if ui
                    .add_enabled(
                        chain_running,
                        egui::Button::new(self.tr("停止区块链", "Stop Blockchain")),
                    )
                    .clicked()
                {
                    self.stop_chain_process();
                }
                if ui.button(self.tr("高级配置", "Advanced Config")).clicked() {
                    self.config_window_open = true;
                }
                if !self.is_expert_mode() {
                    if ui.button(self.tr("新手引导", "Onboarding")).clicked() {
                        self.open_onboarding_manual();
                    }
                    if ui.button(self.tr("重置引导", "Reset Guide")).clicked() {
                        self.reset_onboarding();
                    }
                }
                let has_saved_profile = self.ux_state.last_successful_config.is_some();
                if ui
                    .add_enabled(
                        has_saved_profile,
                        egui::Button::new(
                            self.tr("恢复最近成功配置", "Restore Last Successful Config"),
                        ),
                    )
                    .clicked()
                {
                    self.restore_last_successful_config_profile();
                }
                if ui
                    .add_enabled(
                        has_saved_profile,
                        egui::Button::new(self.tr("清空成功配置", "Clear Saved Config")),
                    )
                    .clicked()
                {
                    self.clear_last_successful_config_profile();
                }
                if let Some(saved_at) = self.ux_state.last_successful_saved_at_unix_ms {
                    ui.small(format!(
                        "{}={saved_at}",
                        self.tr("最近成功配置时间戳", "Saved Profile Timestamp")
                    ));
                }
                let demo_running = matches!(
                    self.demo_mode_phase,
                    DemoModePhase::StartChainRequested
                        | DemoModePhase::WaitChainReady
                        | DemoModePhase::StartGameRequested
                        | DemoModePhase::WaitGameRunning
                );
                if ui
                    .add_enabled(
                        !demo_running,
                        egui::Button::new(self.tr("演示模式一键启动", "Demo Mode One-Click Start")),
                    )
                    .clicked()
                {
                    self.start_demo_mode_one_click();
                }
                if matches!(
                    self.demo_mode_phase,
                    DemoModePhase::Done | DemoModePhase::Failed
                ) && ui
                    .button(self.tr("重置演示状态", "Reset Demo State"))
                    .clicked()
                {
                    self.reset_demo_mode();
                }
                ui.small(format!(
                    "{}={}",
                    self.tr("演示模式状态", "Demo Mode Status"),
                    self.demo_mode_phase_text()
                ));
                if ui
                    .button(self.tr("引导洞察", "Guidance Insights"))
                    .clicked()
                {
                    self.guidance_insights_open = true;
                }
                if ui.button(self.tr("打开游戏页", "Open Game Page")).clicked() {
                    let url = self.current_game_url();
                    if let Err(err) = open_browser(url.as_str()) {
                        self.append_log(format!("open browser failed: {err}"));
                    } else {
                        self.append_log(format!("open browser: {url}"));
                    }
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if ui.button(self.tr("设置", "Settings")).clicked() {
                        self.llm_settings_panel.open();
                    }
                    if ui
                        .add_enabled(
                            self.is_feedback_available(),
                            egui::Button::new(self.tr("反馈", "Feedback")),
                        )
                        .clicked()
                    {
                        self.feedback_window_open = true;
                    }
                    if ui
                        .add_enabled(
                            self.is_feedback_available(),
                            egui::Button::new(self.tr("转账", "Transfer")),
                        )
                        .clicked()
                    {
                        self.transfer_window_open = true;
                    }
                    if ui
                        .add_enabled(
                            self.is_feedback_available(),
                            egui::Button::new(self.tr("浏览器", "Explorer")),
                        )
                        .clicked()
                    {
                        self.explorer_window_open = true;
                    }
                }
                #[cfg(target_arch = "wasm32")]
                {
                    if ui.button(self.tr("设置", "Settings")).clicked() {
                        self.llm_settings_panel.open();
                    }
                    if ui
                        .add_enabled(
                            self.is_feedback_available(),
                            egui::Button::new(self.tr("反馈", "Feedback")),
                        )
                        .clicked()
                    {
                        self.feedback_window_open = true;
                    }
                    if ui
                        .add_enabled(
                            self.is_feedback_available(),
                            egui::Button::new(self.tr("转账", "Transfer")),
                        )
                        .clicked()
                    {
                        self.transfer_window_open = true;
                    }
                    if ui
                        .add_enabled(
                            self.is_feedback_available(),
                            egui::Button::new(self.tr("浏览器", "Explorer")),
                        )
                        .clicked()
                    {
                        self.explorer_window_open = true;
                    }
                }
                if ui.button(self.tr("清空日志", "Clear Logs")).clicked() {
                    self.logs.clear();
                }
            });
            self.render_disabled_action_ctas(
                ui,
                &game_required_issues,
                &chain_required_issues,
                chain_running,
            );

            let url = self.current_game_url();
            ui.label(format!("{}: {url}", self.tr("游戏地址", "Game URL")));

            ui.separator();
            ui.label(self.tr("日志（stdout/stderr）", "Logs (stdout/stderr)"));

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in &self.logs {
                        ui.label(line);
                    }
                });
        });

        self.show_config_window(ctx, &game_required_issues, &chain_required_issues);
        self.show_onboarding_window(
            ctx,
            &game_required_issues,
            &chain_required_issues,
            game_running,
            chain_running,
        );
        self.show_guidance_insights_window(ctx);
        self.show_startup_guide_window(ctx, &game_required_issues, &chain_required_issues);
        self.llm_settings_panel
            .show(ctx, self.ui_language, &mut self.config);
        self.show_feedback_window(ctx);
        self.show_transfer_window(ctx);
        self.show_explorer_window(ctx);
        ctx.request_repaint_after(Duration::from_millis(120));
    }
}

#[cfg(test)]
#[path = "main_tests.rs"]
mod tests;
