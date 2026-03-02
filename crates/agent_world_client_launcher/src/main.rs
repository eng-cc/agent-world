use std::collections::VecDeque;
use std::env;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::time::Duration;

use eframe::egui;

const DEFAULT_SCENARIO: &str = "llm_bootstrap";
const DEFAULT_LIVE_BIND: &str = "127.0.0.1:5023";
const DEFAULT_WEB_BIND: &str = "127.0.0.1:5011";
const DEFAULT_VIEWER_HOST: &str = "127.0.0.1";
const DEFAULT_VIEWER_PORT: &str = "4173";
const DEFAULT_CHAIN_STATUS_BIND: &str = "127.0.0.1:5121";
const DEFAULT_CHAIN_NODE_ID: &str = "viewer-live-node";
const DEFAULT_CHAIN_NODE_ROLE: &str = "sequencer";
const DEFAULT_CHAIN_NODE_TICK_MS: &str = "200";
const MAX_LOG_LINES: usize = 2000;
const EGUI_CJK_FONT_NAME: &str = "agent-world-cjk";
const EGUI_CJK_FONT_BYTES: &[u8] =
    include_bytes!("../../agent_world_viewer/assets/fonts/ms-yahei.ttf");
const CLIENT_LAUNCHER_FONT_ENV: &str = "AGENT_WORLD_CLIENT_LAUNCHER_FONT";
const CLIENT_LAUNCHER_LANG_ENV: &str = "AGENT_WORLD_CLIENT_LAUNCHER_LANG";

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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    chain_node_validators: String,
    auto_open_browser: bool,
    launcher_bin: String,
}

impl Default for LaunchConfig {
    fn default() -> Self {
        let launcher_bin = resolve_launcher_binary_path().to_string_lossy().to_string();
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
            chain_node_validators: String::new(),
            auto_open_browser: true,
            launcher_bin,
        }
    }
}

#[derive(Debug)]
struct RunningProcess {
    child: Child,
    log_rx: Receiver<String>,
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
    ChainStatusBindInvalid,
    ChainNodeIdRequired,
    ChainRoleInvalid,
    ChainTickMsInvalid,
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
                "链 Tick 毫秒（chain tick ms）必须是正整数"
            }
            (Self::ChainTickMsInvalid, UiLanguage::EnUs) => {
                "Chain tick milliseconds must be a positive integer"
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

#[derive(Debug)]
struct ClientLauncherApp {
    config: LaunchConfig,
    ui_language: UiLanguage,
    status: LauncherStatus,
    running: Option<RunningProcess>,
    logs: VecDeque<String>,
}

impl Default for ClientLauncherApp {
    fn default() -> Self {
        Self {
            config: LaunchConfig::default(),
            ui_language: UiLanguage::detect_from_env(),
            status: LauncherStatus::Idle,
            running: None,
            logs: VecDeque::new(),
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

    fn append_log<S: Into<String>>(&mut self, line: S) {
        self.logs.push_back(line.into());
        while self.logs.len() > MAX_LOG_LINES {
            self.logs.pop_front();
        }
    }

    fn current_game_url(&self) -> String {
        build_game_url(&self.config)
    }

    fn poll_process(&mut self) {
        let mut running = match self.running.take() {
            Some(process) => process,
            None => return,
        };

        loop {
            match running.log_rx.try_recv() {
                Ok(line) => self.append_log(line),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => break,
            }
        }

        match running.child.try_wait() {
            Ok(Some(status)) => {
                self.status = LauncherStatus::Exited(status.to_string());
                self.append_log(format!("launcher exited: {status}"));
                self.running = None;
            }
            Ok(None) => {
                self.running = Some(running);
            }
            Err(err) => {
                self.status = LauncherStatus::QueryFailed;
                self.append_log(format!("query child status failed: {err}"));
                self.running = None;
            }
        }
    }

    fn stop_process(&mut self) {
        let mut running = match self.running.take() {
            Some(process) => process,
            None => {
                let message = self
                    .tr("无需停止：当前未运行", "no running process to stop")
                    .to_string();
                self.append_log(message);
                return;
            }
        };

        match stop_child_process(&mut running.child) {
            Ok(()) => {
                self.status = LauncherStatus::Stopped;
                self.append_log("launcher stopped");
            }
            Err(err) => {
                self.status = LauncherStatus::StopFailed;
                self.append_log(format!("launcher stop failed: {err}"));
            }
        }
    }

    fn start_process(&mut self) {
        if self.running.is_some() {
            let message = self
                .tr(
                    "启动忽略：进程已运行",
                    "skip start: process already running",
                )
                .to_string();
            self.append_log(message);
            return;
        }

        let config_issues = collect_required_config_issues(&self.config);
        if !config_issues.is_empty() {
            self.status = LauncherStatus::InvalidArgs;
            let message = self
                .tr(
                    "启动前校验失败：请先修复必填配置项",
                    "preflight validation failed: fix required configuration issues first",
                )
                .to_string();
            self.append_log(message);
            for issue in config_issues {
                self.append_log(format!("- {}", issue.text(self.ui_language)));
            }
            return;
        }

        let launch_args = match build_launcher_args(&self.config) {
            Ok(args) => args,
            Err(err) => {
                self.status = LauncherStatus::InvalidArgs;
                self.append_log(format!("invalid launcher args: {err}"));
                return;
            }
        };

        match spawn_launcher_process(self.config.launcher_bin.as_str(), launch_args.as_slice()) {
            Ok(process) => {
                self.status = LauncherStatus::Running;
                self.append_log("launcher started");
                self.running = Some(process);
            }
            Err(err) => {
                self.status = LauncherStatus::StartFailed;
                self.append_log(format!("launcher start failed: {err}"));
            }
        }
    }
}

impl Drop for ClientLauncherApp {
    fn drop(&mut self) {
        if let Some(mut running) = self.running.take() {
            let _ = stop_child_process(&mut running.child);
        }
    }
}

impl eframe::App for ClientLauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_process();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(self.tr("Agent World 客户端启动器", "Agent World Client Launcher"));
                ui.separator();
                ui.label(format!(
                    "{}: {}",
                    self.tr("状态", "Status"),
                    self.status.text(self.ui_language)
                ));
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
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let llm_label = self.tr("启用 LLM", "Enable LLM").to_string();
            let chain_runtime_label = self.tr("启用链运行时", "Enable Chain Runtime").to_string();
            let auto_open_browser_label = self
                .tr("自动打开浏览器", "Open Browser Automatically")
                .to_string();
            let required_issues = collect_required_config_issues(&self.config);
            let can_start = self.running.is_none() && required_issues.is_empty();

            ui.horizontal_wrapped(|ui| {
                ui.label(self.tr("场景", "Scenario"));
                ui.text_edit_singleline(&mut self.config.scenario);
                ui.label(self.tr("实时服务绑定", "Live Bind"));
                ui.text_edit_singleline(&mut self.config.live_bind);
                ui.label(self.tr("WebSocket 绑定", "Web Bind"));
                ui.text_edit_singleline(&mut self.config.web_bind);
            });

            ui.horizontal_wrapped(|ui| {
                ui.label(self.tr("游戏页面主机", "Viewer Host"));
                ui.text_edit_singleline(&mut self.config.viewer_host);
                ui.label(self.tr("游戏页面端口", "Viewer Port"));
                ui.text_edit_singleline(&mut self.config.viewer_port);
                ui.checkbox(&mut self.config.llm_enabled, llm_label);
                ui.checkbox(&mut self.config.chain_enabled, chain_runtime_label);
                ui.checkbox(&mut self.config.auto_open_browser, auto_open_browser_label);
            });

            ui.horizontal_wrapped(|ui| {
                ui.label(self.tr("链状态服务绑定", "Chain Status Bind"));
                ui.text_edit_singleline(&mut self.config.chain_status_bind);
                ui.label(self.tr("链节点 ID", "Chain Node ID"));
                ui.text_edit_singleline(&mut self.config.chain_node_id);
                ui.label(self.tr("链世界 ID", "Chain World ID"));
                ui.text_edit_singleline(&mut self.config.chain_world_id);
            });

            ui.horizontal_wrapped(|ui| {
                ui.label(self.tr("链节点角色", "Chain Role"));
                ui.text_edit_singleline(&mut self.config.chain_node_role);
                ui.label(self.tr("链 Tick 毫秒", "Chain Tick Milliseconds"));
                ui.text_edit_singleline(&mut self.config.chain_node_tick_ms);
                ui.label(self.tr("链验证者", "Chain Validators"));
                ui.text_edit_singleline(&mut self.config.chain_node_validators);
            });

            ui.horizontal_wrapped(|ui| {
                ui.label(self.tr("启动器二进制路径", "Launcher Binary"));
                ui.text_edit_singleline(&mut self.config.launcher_bin);
            });
            ui.horizontal_wrapped(|ui| {
                ui.label(self.tr("前端静态资源目录", "Viewer Static Directory"));
                ui.text_edit_singleline(&mut self.config.viewer_static_dir);
            });

            if required_issues.is_empty() {
                ui.colored_label(
                    egui::Color32::from_rgb(36, 130, 78),
                    self.tr(
                        "必填配置项已通过校验，可启动游戏",
                        "Required configuration check passed; launcher can start",
                    ),
                );
            } else {
                ui.group(|ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(188, 60, 60),
                        self.tr(
                            "启动前请先修复以下必填配置项：",
                            "Fix the required configuration issues before starting:",
                        ),
                    );
                    for issue in &required_issues {
                        ui.label(format!("- {}", issue.text(self.ui_language)));
                    }
                });
            }

            ui.separator();

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(can_start, egui::Button::new(self.tr("启动", "Start")))
                    .clicked()
                {
                    self.start_process();
                }
                if ui
                    .add_enabled(
                        self.running.is_some(),
                        egui::Button::new(self.tr("停止", "Stop")),
                    )
                    .clicked()
                {
                    self.stop_process();
                }
                if ui.button(self.tr("打开游戏页", "Open Game Page")).clicked() {
                    let url = self.current_game_url();
                    if let Err(err) = open_browser(url.as_str()) {
                        self.append_log(format!("open browser failed: {err}"));
                    } else {
                        self.append_log(format!("open browser: {url}"));
                    }
                }
                if ui.button(self.tr("清空日志", "Clear Logs")).clicked() {
                    self.logs.clear();
                }
            });

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

        ctx.request_repaint_after(Duration::from_millis(120));
    }
}

fn collect_required_config_issues(config: &LaunchConfig) -> Vec<ConfigIssue> {
    let mut issues = Vec::new();

    if config.scenario.trim().is_empty() {
        issues.push(ConfigIssue::ScenarioRequired);
    }
    if parse_host_port(config.live_bind.as_str(), "live bind").is_err() {
        issues.push(ConfigIssue::LiveBindInvalid);
    }
    if parse_host_port(config.web_bind.as_str(), "web bind").is_err() {
        issues.push(ConfigIssue::WebBindInvalid);
    }
    if config.viewer_host.trim().is_empty() {
        issues.push(ConfigIssue::ViewerHostRequired);
    }
    if parse_port(config.viewer_port.as_str(), "viewer port").is_err() {
        issues.push(ConfigIssue::ViewerPortInvalid);
    }

    let viewer_static_dir = config.viewer_static_dir.trim();
    if viewer_static_dir.is_empty() {
        issues.push(ConfigIssue::ViewerStaticDirRequired);
    } else if !Path::new(viewer_static_dir).is_dir() {
        issues.push(ConfigIssue::ViewerStaticDirMissing);
    }

    let launcher_bin = config.launcher_bin.trim();
    if launcher_bin.is_empty() {
        issues.push(ConfigIssue::LauncherBinRequired);
    } else if !Path::new(launcher_bin).is_file() {
        issues.push(ConfigIssue::LauncherBinMissing);
    }

    if config.chain_enabled {
        if parse_host_port(config.chain_status_bind.as_str(), "chain status bind").is_err() {
            issues.push(ConfigIssue::ChainStatusBindInvalid);
        }
        if config.chain_node_id.trim().is_empty() {
            issues.push(ConfigIssue::ChainNodeIdRequired);
        }
        if parse_chain_role(config.chain_node_role.as_str()).is_err() {
            issues.push(ConfigIssue::ChainRoleInvalid);
        }
        if parse_port(config.chain_node_tick_ms.as_str(), "chain tick ms").is_err() {
            issues.push(ConfigIssue::ChainTickMsInvalid);
        }
        if parse_chain_validators(config.chain_node_validators.as_str()).is_err() {
            issues.push(ConfigIssue::ChainValidatorsInvalid);
        }
    }

    issues
}

fn build_launcher_args(config: &LaunchConfig) -> Result<Vec<String>, String> {
    if config.scenario.trim().is_empty() {
        return Err("scenario cannot be empty".to_string());
    }
    parse_host_port(config.live_bind.as_str(), "live bind")?;
    parse_host_port(config.web_bind.as_str(), "web bind")?;
    let viewer_port = parse_port(config.viewer_port.as_str(), "viewer port")?;
    if config.viewer_host.trim().is_empty() {
        return Err("viewer host cannot be empty".to_string());
    }
    if config.viewer_static_dir.trim().is_empty() {
        return Err("viewer static dir cannot be empty".to_string());
    }
    if config.chain_enabled {
        parse_host_port(config.chain_status_bind.as_str(), "chain status bind")?;
        if config.chain_node_id.trim().is_empty() {
            return Err("chain node id cannot be empty".to_string());
        }
        parse_chain_role(config.chain_node_role.as_str())?;
        parse_port(config.chain_node_tick_ms.as_str(), "chain tick ms")?;
        let _ = parse_chain_validators(config.chain_node_validators.as_str())?;
    }

    let mut args = vec![
        "--scenario".to_string(),
        config.scenario.trim().to_string(),
        "--live-bind".to_string(),
        config.live_bind.trim().to_string(),
        "--web-bind".to_string(),
        config.web_bind.trim().to_string(),
        "--viewer-host".to_string(),
        config.viewer_host.trim().to_string(),
        "--viewer-port".to_string(),
        viewer_port.to_string(),
        "--viewer-static-dir".to_string(),
        config.viewer_static_dir.trim().to_string(),
    ];

    if config.chain_enabled {
        args.push("--chain-enable".to_string());
        args.push("--chain-status-bind".to_string());
        args.push(config.chain_status_bind.trim().to_string());
        args.push("--chain-node-id".to_string());
        args.push(config.chain_node_id.trim().to_string());
        if !config.chain_world_id.trim().is_empty() {
            args.push("--chain-world-id".to_string());
            args.push(config.chain_world_id.trim().to_string());
        }
        args.push("--chain-node-role".to_string());
        args.push(parse_chain_role(config.chain_node_role.as_str())?);
        args.push("--chain-node-tick-ms".to_string());
        args.push(parse_port(config.chain_node_tick_ms.as_str(), "chain tick ms")?.to_string());
        for validator in parse_chain_validators(config.chain_node_validators.as_str())? {
            args.push("--chain-node-validator".to_string());
            args.push(validator);
        }
    } else {
        args.push("--chain-disable".to_string());
    }

    if config.llm_enabled {
        args.push("--with-llm".to_string());
    }
    if !config.auto_open_browser {
        args.push("--no-open-browser".to_string());
    }

    Ok(args)
}

fn build_game_url(config: &LaunchConfig) -> String {
    let viewer_host = normalize_host_for_url(config.viewer_host.as_str());
    let viewer_host = host_for_url(viewer_host.as_str());
    let viewer_port = parse_port(config.viewer_port.as_str(), "viewer port").unwrap_or(4173);
    let (web_host, web_port) = parse_host_port(config.web_bind.as_str(), "web bind")
        .unwrap_or(("127.0.0.1".to_string(), 5011));
    let web_host = normalize_host_for_url(web_host.as_str());
    let web_host = host_for_url(web_host.as_str());

    format!("http://{viewer_host}:{viewer_port}/?ws=ws://{web_host}:{web_port}")
}

fn normalize_host_for_url(host: &str) -> String {
    let host = host.trim();
    if host == "0.0.0.0" || host == "::" || host == "[::]" || host.is_empty() {
        "127.0.0.1".to_string()
    } else {
        host.to_string()
    }
}

fn host_for_url(host: &str) -> String {
    if host.contains(':') && !host.starts_with('[') && !host.ends_with(']') {
        format!("[{host}]")
    } else {
        host.to_string()
    }
}

fn parse_port(raw: &str, label: &str) -> Result<u16, String> {
    let value = raw.trim();
    let port = value
        .parse::<u16>()
        .map_err(|_| format!("{label} must be integer in 1..=65535"))?;
    if port == 0 {
        return Err(format!("{label} must be in 1..=65535"));
    }
    Ok(port)
}

fn parse_host_port(raw: &str, label: &str) -> Result<(String, u16), String> {
    let value = raw.trim();
    let (host_raw, port_raw) = if let Some(rest) = value.strip_prefix('[') {
        let (host, remainder) = rest
            .split_once(']')
            .ok_or_else(|| format!("{label} IPv6 host must be in [addr]:port format"))?;
        let port_raw = remainder
            .strip_prefix(':')
            .ok_or_else(|| format!("{label} must be in <host:port> format"))?;
        (host, port_raw)
    } else {
        let (host, port_raw) = value
            .rsplit_once(':')
            .ok_or_else(|| format!("{label} must be in <host:port> format"))?;
        if host.contains(':') {
            return Err(format!("{label} IPv6 host must be wrapped in []"));
        }
        (host, port_raw)
    };
    let host = host_raw.trim();
    if host.trim().is_empty() {
        return Err(format!("{label} host cannot be empty"));
    }
    let port = parse_port(port_raw, label)?;
    Ok((host.trim().to_string(), port))
}

fn parse_chain_role(raw: &str) -> Result<String, String> {
    let role = raw.trim().to_ascii_lowercase();
    match role.as_str() {
        "sequencer" | "storage" | "observer" => Ok(role),
        _ => Err("chain role must be one of: sequencer|storage|observer".to_string()),
    }
}

fn parse_chain_validators(raw: &str) -> Result<Vec<String>, String> {
    let mut validators = Vec::new();
    for token in raw.split([',', ';', ' ']) {
        let token = token.trim();
        if token.is_empty() {
            continue;
        }
        let (validator_id, stake) = token
            .rsplit_once(':')
            .ok_or_else(|| "chain validators must be <validator_id:stake>".to_string())?;
        if validator_id.trim().is_empty() {
            return Err("chain validators cannot contain empty validator_id".to_string());
        }
        let stake = stake
            .parse::<u64>()
            .ok()
            .filter(|value| *value > 0)
            .ok_or_else(|| "chain validator stake must be positive integer".to_string())?;
        validators.push(format!("{}:{}", validator_id.trim(), stake));
    }
    Ok(validators)
}

fn spawn_launcher_process(bin: &str, args: &[String]) -> Result<RunningProcess, String> {
    let mut child = Command::new(bin)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("spawn launcher `{bin}` failed: {err}"))?;

    let (log_tx, log_rx) = mpsc::channel::<String>();
    if let Some(stdout) = child.stdout.take() {
        spawn_log_reader(stdout, "stdout", log_tx.clone());
    }
    if let Some(stderr) = child.stderr.take() {
        spawn_log_reader(stderr, "stderr", log_tx.clone());
    }

    Ok(RunningProcess { child, log_rx })
}

fn spawn_log_reader<R: Read + Send + 'static>(reader: R, source: &'static str, tx: Sender<String>) {
    std::thread::spawn(move || {
        let buffered = BufReader::new(reader);
        for line in buffered.lines() {
            match line {
                Ok(content) => {
                    let _ = tx.send(format!("[{source}] {content}"));
                }
                Err(err) => {
                    let _ = tx.send(format!("[{source}] <read error: {err}>"));
                    break;
                }
            }
        }
    });
}

fn stop_child_process(child: &mut Child) -> Result<(), String> {
    if let Ok(None) = child.try_wait() {
        child
            .kill()
            .map_err(|err| format!("kill child failed: {err}"))?;
    }
    child
        .wait()
        .map_err(|err| format!("wait child failed: {err}"))?;
    Ok(())
}

fn resolve_launcher_binary_path() -> PathBuf {
    if let Ok(path) = env::var("AGENT_WORLD_GAME_LAUNCHER_BIN") {
        return PathBuf::from(path);
    }

    if let Ok(current_exe) = env::current_exe() {
        if let Some(bin_dir) = current_exe.parent() {
            return bin_dir.join(binary_name("world_game_launcher"));
        }
    }

    PathBuf::from(binary_name("world_game_launcher"))
}

fn resolve_static_dir_path() -> PathBuf {
    if let Ok(path) = env::var("AGENT_WORLD_GAME_STATIC_DIR") {
        return PathBuf::from(path);
    }

    if let Ok(current_exe) = env::current_exe() {
        if let Some(bin_dir) = current_exe.parent() {
            return bin_dir.join("..").join("web");
        }
    }

    PathBuf::from("web")
}

fn binary_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}

fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let status = Command::new("open")
            .arg(url)
            .status()
            .map_err(|err| format!("run open failed: {err}"))?;
        if status.success() {
            return Ok(());
        }
        return Err(format!("open exited with {status}"));
    }

    #[cfg(target_os = "windows")]
    {
        let status = Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg("")
            .arg(url)
            .status()
            .map_err(|err| format!("run cmd /C start failed: {err}"))?;
        if status.success() {
            return Ok(());
        }
        return Err(format!("cmd /C start exited with {status}"));
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        let status = Command::new("xdg-open")
            .arg(url)
            .status()
            .map_err(|err| format!("run xdg-open failed: {err}"))?;
        if status.success() {
            return Ok(());
        }
        Err(format!("xdg-open exited with {status}"))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_game_url, build_launcher_args, collect_required_config_issues, install_cjk_font,
        normalize_host_for_url, parse_chain_role, parse_chain_validators, parse_host_port,
        parse_port, ConfigIssue, LaunchConfig, LauncherStatus, UiLanguage, EGUI_CJK_FONT_NAME,
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
}
