use std::collections::VecDeque;
use std::env;
use std::io::{BufRead, BufReader, Read};
use std::path::PathBuf;
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

#[derive(Debug)]
struct ClientLauncherApp {
    config: LaunchConfig,
    status_text: String,
    running: Option<RunningProcess>,
    logs: VecDeque<String>,
}

impl Default for ClientLauncherApp {
    fn default() -> Self {
        Self {
            config: LaunchConfig::default(),
            status_text: "未启动".to_string(),
            running: None,
            logs: VecDeque::new(),
        }
    }
}

impl ClientLauncherApp {
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
                self.status_text = format!("已退出: {status}");
                self.append_log(format!("launcher exited: {status}"));
                self.running = None;
            }
            Ok(None) => {
                self.running = Some(running);
            }
            Err(err) => {
                self.status_text = "状态查询失败".to_string();
                self.append_log(format!("query child status failed: {err}"));
                self.running = None;
            }
        }
    }

    fn stop_process(&mut self) {
        let mut running = match self.running.take() {
            Some(process) => process,
            None => {
                self.append_log("无需停止：当前未运行");
                return;
            }
        };

        match stop_child_process(&mut running.child) {
            Ok(()) => {
                self.status_text = "已停止".to_string();
                self.append_log("launcher stopped");
            }
            Err(err) => {
                self.status_text = "停止失败".to_string();
                self.append_log(format!("launcher stop failed: {err}"));
            }
        }
    }

    fn start_process(&mut self) {
        if self.running.is_some() {
            self.append_log("启动忽略：进程已运行");
            return;
        }

        let launch_args = match build_launcher_args(&self.config) {
            Ok(args) => args,
            Err(err) => {
                self.status_text = "参数非法".to_string();
                self.append_log(format!("invalid launcher args: {err}"));
                return;
            }
        };

        match spawn_launcher_process(self.config.launcher_bin.as_str(), launch_args.as_slice()) {
            Ok(process) => {
                self.status_text = "运行中".to_string();
                self.append_log("launcher started");
                self.running = Some(process);
            }
            Err(err) => {
                self.status_text = "启动失败".to_string();
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
                ui.heading("Agent World 客户端启动器");
                ui.separator();
                ui.label(format!("状态: {}", self.status_text));
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("scenario");
                ui.text_edit_singleline(&mut self.config.scenario);
                ui.label("live bind");
                ui.text_edit_singleline(&mut self.config.live_bind);
                ui.label("web bind");
                ui.text_edit_singleline(&mut self.config.web_bind);
            });

            ui.horizontal_wrapped(|ui| {
                ui.label("viewer host");
                ui.text_edit_singleline(&mut self.config.viewer_host);
                ui.label("viewer port");
                ui.text_edit_singleline(&mut self.config.viewer_port);
                ui.checkbox(&mut self.config.llm_enabled, "启用 LLM");
                ui.checkbox(&mut self.config.chain_enabled, "启用链运行时");
                ui.checkbox(&mut self.config.auto_open_browser, "自动打开浏览器");
            });

            ui.horizontal_wrapped(|ui| {
                ui.label("chain status bind");
                ui.text_edit_singleline(&mut self.config.chain_status_bind);
                ui.label("chain node id");
                ui.text_edit_singleline(&mut self.config.chain_node_id);
                ui.label("chain world id");
                ui.text_edit_singleline(&mut self.config.chain_world_id);
            });

            ui.horizontal_wrapped(|ui| {
                ui.label("chain role");
                ui.text_edit_singleline(&mut self.config.chain_node_role);
                ui.label("chain tick ms");
                ui.text_edit_singleline(&mut self.config.chain_node_tick_ms);
                ui.label("chain validators");
                ui.text_edit_singleline(&mut self.config.chain_node_validators);
            });

            ui.horizontal_wrapped(|ui| {
                ui.label("launcher bin");
                ui.text_edit_singleline(&mut self.config.launcher_bin);
            });
            ui.horizontal_wrapped(|ui| {
                ui.label("viewer static dir");
                ui.text_edit_singleline(&mut self.config.viewer_static_dir);
            });

            ui.separator();

            ui.horizontal(|ui| {
                if ui
                    .add_enabled(self.running.is_none(), egui::Button::new("启动"))
                    .clicked()
                {
                    self.start_process();
                }
                if ui
                    .add_enabled(self.running.is_some(), egui::Button::new("停止"))
                    .clicked()
                {
                    self.stop_process();
                }
                if ui.button("打开游戏页").clicked() {
                    let url = self.current_game_url();
                    if let Err(err) = open_browser(url.as_str()) {
                        self.append_log(format!("open browser failed: {err}"));
                    } else {
                        self.append_log(format!("open browser: {url}"));
                    }
                }
                if ui.button("清空日志").clicked() {
                    self.logs.clear();
                }
            });

            let url = self.current_game_url();
            ui.label(format!("游戏地址: {url}"));

            ui.separator();
            ui.label("日志（stdout/stderr）");

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
    let viewer_port = parse_port(config.viewer_port.as_str(), "viewer port").unwrap_or(4173);
    let (web_host, web_port) = parse_host_port(config.web_bind.as_str(), "web bind")
        .unwrap_or(("127.0.0.1".to_string(), 5011));
    let web_host = normalize_host_for_url(web_host.as_str());

    format!("http://{viewer_host}:{viewer_port}/?ws=ws://{web_host}:{web_port}")
}

fn normalize_host_for_url(host: &str) -> String {
    let host = host.trim();
    if host == "0.0.0.0" || host.is_empty() {
        "127.0.0.1".to_string()
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
    let (host, port_raw) = value
        .rsplit_once(':')
        .ok_or_else(|| format!("{label} must be in <host:port> format"))?;
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
        build_game_url, build_launcher_args, install_cjk_font, normalize_host_for_url,
        parse_chain_role, parse_chain_validators, parse_host_port, parse_port, LaunchConfig,
        EGUI_CJK_FONT_NAME,
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
}
