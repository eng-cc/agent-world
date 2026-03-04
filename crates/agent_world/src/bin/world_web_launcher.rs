use std::collections::VecDeque;
use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{self, Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const DEFAULT_LISTEN_BIND: &str = "0.0.0.0:5410";
const DEFAULT_SCENARIO: &str = "llm_bootstrap";
const DEFAULT_LIVE_BIND: &str = "0.0.0.0:5023";
const DEFAULT_WEB_BIND: &str = "0.0.0.0:5011";
const DEFAULT_VIEWER_HOST: &str = "0.0.0.0";
const DEFAULT_VIEWER_PORT: u16 = 4173;
const DEFAULT_VIEWER_STATIC_DIR: &str = "web";
const DEFAULT_CHAIN_STATUS_BIND: &str = "127.0.0.1:5121";
const DEFAULT_CHAIN_NODE_ID: &str = "viewer-live-node";
const DEFAULT_CHAIN_NODE_ROLE: &str = "sequencer";
const DEFAULT_CHAIN_NODE_TICK_MS: u64 = 200;
const MAX_LOG_LINES: usize = 2000;
const GRACEFUL_STOP_TIMEOUT_MS: u64 = 4000;
const STOP_POLL_INTERVAL_MS: u64 = 80;
const HTTP_READ_TIMEOUT_SECS: u64 = 3;
const MAX_HTTP_HEADER_BYTES: usize = 32 * 1024;
const MAX_HTTP_BODY_BYTES: usize = 1024 * 1024;

static TERMINATION_REQUESTED: AtomicBool = AtomicBool::new(false);
static SIGNAL_HANDLER_INSTALL: OnceLock<Result<(), String>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct LauncherConfig {
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
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            scenario: DEFAULT_SCENARIO.to_string(),
            live_bind: DEFAULT_LIVE_BIND.to_string(),
            web_bind: DEFAULT_WEB_BIND.to_string(),
            viewer_host: DEFAULT_VIEWER_HOST.to_string(),
            viewer_port: DEFAULT_VIEWER_PORT.to_string(),
            viewer_static_dir: resolve_static_dir_path().to_string_lossy().to_string(),
            llm_enabled: false,
            chain_enabled: true,
            chain_status_bind: DEFAULT_CHAIN_STATUS_BIND.to_string(),
            chain_node_id: DEFAULT_CHAIN_NODE_ID.to_string(),
            chain_world_id: String::new(),
            chain_node_role: DEFAULT_CHAIN_NODE_ROLE.to_string(),
            chain_node_tick_ms: DEFAULT_CHAIN_NODE_TICK_MS.to_string(),
            chain_node_validators: String::new(),
            auto_open_browser: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliOptions {
    listen_bind: String,
    launcher_bin: String,
    initial_config: LauncherConfig,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            listen_bind: DEFAULT_LISTEN_BIND.to_string(),
            launcher_bin: resolve_world_game_launcher_binary()
                .to_string_lossy()
                .to_string(),
            initial_config: LauncherConfig::default(),
        }
    }
}

#[derive(Debug)]
struct RunningProcess {
    child: Child,
    log_rx: Receiver<String>,
}

#[derive(Debug, Clone)]
enum ProcessState {
    Idle,
    Running { pid: u32 },
    Stopped,
    InvalidConfig(String),
    StartFailed(String),
    StopFailed(String),
    Exited(String),
}

impl ProcessState {
    fn code(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Running { .. } => "running",
            Self::Stopped => "stopped",
            Self::InvalidConfig(_) => "invalid_config",
            Self::StartFailed(_) => "start_failed",
            Self::StopFailed(_) => "stop_failed",
            Self::Exited(_) => "exited",
        }
    }

    fn detail(&self) -> Option<String> {
        match self {
            Self::InvalidConfig(detail)
            | Self::StartFailed(detail)
            | Self::StopFailed(detail)
            | Self::Exited(detail) => Some(detail.clone()),
            Self::Idle | Self::Running { .. } | Self::Stopped => None,
        }
    }

    fn pid(&self) -> Option<u32> {
        match self {
            Self::Running { pid } => Some(*pid),
            Self::Idle
            | Self::Stopped
            | Self::InvalidConfig(_)
            | Self::StartFailed(_)
            | Self::StopFailed(_)
            | Self::Exited(_) => None,
        }
    }
}

#[derive(Debug)]
struct ServiceState {
    launcher_bin: String,
    config: LauncherConfig,
    process_state: ProcessState,
    running: Option<RunningProcess>,
    logs: VecDeque<String>,
    updated_at_unix_ms: u64,
}

impl ServiceState {
    fn new(launcher_bin: String, config: LauncherConfig) -> Self {
        Self {
            launcher_bin,
            config,
            process_state: ProcessState::Idle,
            running: None,
            logs: VecDeque::new(),
            updated_at_unix_ms: now_unix_ms(),
        }
    }

    fn append_log<S: Into<String>>(&mut self, line: S) {
        self.logs.push_back(line.into());
        while self.logs.len() > MAX_LOG_LINES {
            self.logs.pop_front();
        }
    }

    fn mark_updated(&mut self) {
        self.updated_at_unix_ms = now_unix_ms();
    }
}

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

#[derive(Debug, Serialize)]
struct StateSnapshot {
    status: String,
    detail: Option<String>,
    pid: Option<u32>,
    running: bool,
    launcher_bin: String,
    game_url: String,
    config: LauncherConfig,
    logs: Vec<String>,
    updated_at_unix_ms: u64,
}

#[derive(Debug, Serialize)]
struct ApiResponse {
    ok: bool,
    error: Option<String>,
    state: StateSnapshot,
}

fn main() {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    if raw_args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return;
    }

    let options = match parse_options(raw_args.iter().map(String::as_str)) {
        Ok(options) => options,
        Err(err) => {
            eprintln!("{err}");
            print_help();
            process::exit(1);
        }
    };

    if let Err(err) = run_server(options) {
        eprintln!("world_web_launcher failed: {err}");
        process::exit(1);
    }
}

fn print_help() {
    println!(
        "Usage: world_web_launcher [options]\n\n\
Options:\n\
  --listen-bind <host:port>       web console listen bind (default: {DEFAULT_LISTEN_BIND})\n\
  --launcher-bin <path>           world_game_launcher binary path\n\
  --scenario <name>               default scenario for web form\n\
  --live-bind <host:port>         default --live-bind for world_game_launcher\n\
  --web-bind <host:port>          default --web-bind for world_game_launcher\n\
  --viewer-host <host>            default viewer host bind\n\
  --viewer-port <port>            default viewer port\n\
  --viewer-static-dir <path>      default viewer static directory\n\
  --with-llm / --no-llm           default LLM toggle\n\
  --chain-enable / --chain-disable\n\
  --chain-status-bind <host:port>\n\
  --chain-node-id <id>\n\
  --chain-world-id <id>\n\
  --chain-node-role <role>        sequencer|storage|observer\n\
  --chain-node-tick-ms <ms>\n\
  --chain-node-validator <id:stake> (repeatable)\n\
  --open-browser / --no-open-browser\n\
  -h, --help                      show this help\n"
    );
}

fn run_server(options: CliOptions) -> Result<(), String> {
    install_signal_handler()?;
    TERMINATION_REQUESTED.store(false, Ordering::SeqCst);

    let (listen_host, listen_port) =
        parse_host_port(options.listen_bind.as_str(), "--listen-bind")?;
    let listener = TcpListener::bind((listen_host.as_str(), listen_port)).map_err(|err| {
        format!(
            "failed to bind world_web_launcher at {}:{}: {}",
            listen_host, listen_port, err
        )
    })?;
    listener
        .set_nonblocking(true)
        .map_err(|err| format!("failed to set listener nonblocking: {err}"))?;

    let state = Arc::new(Mutex::new(ServiceState::new(
        options.launcher_bin,
        options.initial_config,
    )));

    println!("world_web_launcher started");
    println!(
        "- console: http://{}:{}",
        normalize_bind_host_for_local_access(listen_host.as_str()),
        listen_port
    );
    println!("- listen bind: {listen_host}:{listen_port}");
    println!("Press Ctrl+C to stop.");

    loop {
        if TERMINATION_REQUESTED.load(Ordering::SeqCst) {
            break;
        }

        match listener.accept() {
            Ok((stream, _addr)) => {
                let shared = Arc::clone(&state);
                thread::spawn(move || {
                    if let Err(err) = handle_connection(stream, shared) {
                        eprintln!("warning: world_web_launcher request failed: {err}");
                    }
                });
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(30));
            }
            Err(err) => return Err(format!("accept failed: {err}")),
        }
    }

    let mut state_guard = lock_state(&state);
    let _ = stop_process(&mut state_guard);
    Ok(())
}

fn install_signal_handler() -> Result<(), String> {
    SIGNAL_HANDLER_INSTALL
        .get_or_init(|| {
            ctrlc::set_handler(|| {
                TERMINATION_REQUESTED.store(true, Ordering::SeqCst);
            })
            .map_err(|err| format!("failed to install signal handler: {err}"))
        })
        .clone()
}

fn handle_connection(
    mut stream: TcpStream,
    shared_state: Arc<Mutex<ServiceState>>,
) -> Result<(), String> {
    let request = read_http_request(&mut stream)?;
    let path = strip_query(request.path.as_str());

    match (request.method.as_str(), path) {
        ("GET", "/") => {
            write_http_response(
                &mut stream,
                200,
                "text/html; charset=utf-8",
                WEB_CONSOLE_HTML.as_bytes(),
                false,
            )?;
            Ok(())
        }
        ("GET", "/healthz") => {
            write_http_response(&mut stream, 200, "text/plain; charset=utf-8", b"ok", false)?;
            Ok(())
        }
        ("GET", "/api/state") => {
            let request_host = extract_host_header(request.headers.as_slice());
            let mut state = lock_state(&shared_state);
            poll_process_state(&mut state);
            let snapshot = snapshot_from_state(&state, request_host.as_deref());
            write_json_response(&mut stream, 200, &snapshot)
        }
        ("POST", "/api/start") => {
            let request_host = extract_host_header(request.headers.as_slice());
            let config: LauncherConfig = serde_json::from_slice(request.body.as_slice())
                .map_err(|err| format!("parse start request JSON failed: {err}"))?;
            let mut state = lock_state(&shared_state);
            poll_process_state(&mut state);
            let outcome = start_process(&mut state, config);
            let snapshot = snapshot_from_state(&state, request_host.as_deref());
            let response = ApiResponse {
                ok: outcome.is_ok(),
                error: outcome.err(),
                state: snapshot,
            };
            write_json_response(&mut stream, 200, &response)
        }
        ("POST", "/api/stop") => {
            let request_host = extract_host_header(request.headers.as_slice());
            let mut state = lock_state(&shared_state);
            poll_process_state(&mut state);
            let outcome = stop_process(&mut state);
            let snapshot = snapshot_from_state(&state, request_host.as_deref());
            let response = ApiResponse {
                ok: outcome.is_ok(),
                error: outcome.err(),
                state: snapshot,
            };
            write_json_response(&mut stream, 200, &response)
        }
        ("OPTIONS", _) => {
            write_http_response(&mut stream, 204, "text/plain", b"", false)?;
            Ok(())
        }
        (method, "/api/state") | (method, "/api/start") | (method, "/api/stop")
            if method != "GET" && method != "POST" =>
        {
            write_http_response(
                &mut stream,
                405,
                "text/plain; charset=utf-8",
                b"Method Not Allowed",
                false,
            )?;
            Ok(())
        }
        _ => {
            write_http_response(
                &mut stream,
                404,
                "text/plain; charset=utf-8",
                b"Not Found",
                false,
            )?;
            Ok(())
        }
    }
}

fn lock_state(shared: &Arc<Mutex<ServiceState>>) -> std::sync::MutexGuard<'_, ServiceState> {
    shared
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn read_http_request(stream: &mut TcpStream) -> Result<HttpRequest, String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(HTTP_READ_TIMEOUT_SECS)))
        .map_err(|err| format!("set read timeout failed: {err}"))?;

    let mut buffer = Vec::with_capacity(1024);
    let header_end = loop {
        if buffer.len() > MAX_HTTP_HEADER_BYTES {
            return Err("HTTP header is too large".to_string());
        }
        let mut chunk = [0_u8; 1024];
        let bytes = stream
            .read(&mut chunk)
            .map_err(|err| format!("read request failed: {err}"))?;
        if bytes == 0 {
            return Err("empty request".to_string());
        }
        buffer.extend_from_slice(&chunk[..bytes]);
        if let Some(end) = find_header_end(buffer.as_slice()) {
            break end;
        }
    };

    let header_bytes = &buffer[..header_end];
    let header_text = String::from_utf8_lossy(header_bytes);
    let mut lines = header_text.split("\r\n");
    let request_line = lines
        .next()
        .ok_or_else(|| "missing request line".to_string())?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts
        .next()
        .ok_or_else(|| "missing request method".to_string())?
        .to_ascii_uppercase();
    let path = request_parts
        .next()
        .ok_or_else(|| "missing request target".to_string())?
        .to_string();

    let mut headers = Vec::new();
    let mut content_length = 0usize;
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| format!("invalid header line: {line}"))?;
        let name = name.trim().to_ascii_lowercase();
        let value = value.trim().to_string();
        if name == "content-length" {
            content_length = value
                .parse::<usize>()
                .map_err(|_| format!("invalid content-length: {value}"))?;
            if content_length > MAX_HTTP_BODY_BYTES {
                return Err("HTTP body is too large".to_string());
            }
        }
        headers.push((name, value));
    }

    let mut body = buffer[(header_end + 4)..].to_vec();
    while body.len() < content_length {
        let remaining = content_length - body.len();
        let mut chunk = vec![0_u8; remaining.min(4096)];
        let bytes = stream
            .read(chunk.as_mut_slice())
            .map_err(|err| format!("read request body failed: {err}"))?;
        if bytes == 0 {
            return Err("unexpected EOF while reading request body".to_string());
        }
        body.extend_from_slice(&chunk[..bytes]);
    }
    body.truncate(content_length);

    Ok(HttpRequest {
        method,
        path,
        headers,
        body,
    })
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|window| window == b"\r\n\r\n")
}

fn strip_query(path: &str) -> &str {
    path.split('?').next().unwrap_or(path)
}

fn extract_host_header(headers: &[(String, String)]) -> Option<String> {
    headers
        .iter()
        .find(|(name, _)| name == "host")
        .map(|(_, value)| normalize_host_header(value.as_str()))
        .filter(|value| !value.is_empty())
}

fn normalize_host_header(raw: &str) -> String {
    let value = raw.trim();
    if value.starts_with('[') {
        if let Some((host, _)) = value.rsplit_once(']') {
            return host.trim_start_matches('[').to_string();
        }
    }
    if let Some((host, _port)) = value.rsplit_once(':') {
        if host.contains(':') {
            return value.to_string();
        }
        return host.to_string();
    }
    value.to_string()
}

fn write_json_response<T: Serialize>(
    stream: &mut TcpStream,
    status_code: u16,
    payload: &T,
) -> Result<(), String> {
    let body =
        serde_json::to_vec(payload).map_err(|err| format!("serialize JSON failed: {err}"))?;
    write_http_response(
        stream,
        status_code,
        "application/json; charset=utf-8",
        body.as_slice(),
        false,
    )
}

fn write_http_response(
    stream: &mut TcpStream,
    status_code: u16,
    content_type: &str,
    body: &[u8],
    head_only: bool,
) -> Result<(), String> {
    let status_text = match status_code {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Internal Server Error",
    };
    let headers = format!(
        "HTTP/1.1 {status_code} {status_text}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(headers.as_bytes())
        .map_err(|err| format!("write response header failed: {err}"))?;
    if !head_only {
        stream
            .write_all(body)
            .map_err(|err| format!("write response body failed: {err}"))?;
    }
    stream
        .flush()
        .map_err(|err| format!("flush response failed: {err}"))?;
    Ok(())
}

fn poll_process_state(state: &mut ServiceState) {
    let Some(mut running) = state.running.take() else {
        return;
    };

    loop {
        match running.log_rx.try_recv() {
            Ok(line) => state.append_log(line),
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => break,
        }
    }

    match running.child.try_wait() {
        Ok(Some(status)) => {
            state.process_state = ProcessState::Exited(status.to_string());
            state.append_log(format!("world_game_launcher exited: {status}"));
            state.mark_updated();
        }
        Ok(None) => {
            state.running = Some(running);
        }
        Err(err) => {
            state.process_state =
                ProcessState::Exited(format!("query process status failed: {err}"));
            state.append_log(format!("query process status failed: {err}"));
            state.mark_updated();
        }
    }
}

fn start_process(state: &mut ServiceState, config: LauncherConfig) -> Result<(), String> {
    if state.running.is_some() {
        return Err("world_game_launcher is already running".to_string());
    }

    let issues = validate_config(&config);
    if !issues.is_empty() {
        let detail = issues.join("; ");
        state.process_state = ProcessState::InvalidConfig(detail.clone());
        state.append_log(format!("config validation failed: {detail}"));
        state.mark_updated();
        return Err(detail);
    }

    let args = match build_launcher_args(&config) {
        Ok(args) => args,
        Err(err) => {
            state.process_state = ProcessState::InvalidConfig(err.clone());
            state.append_log(format!("invalid launch args: {err}"));
            state.mark_updated();
            return Err(err);
        }
    };

    match spawn_child_process(state.launcher_bin.as_str(), args.as_slice()) {
        Ok(process) => {
            let pid = process.child.id();
            state.running = Some(process);
            state.config = config;
            state.process_state = ProcessState::Running { pid };
            state.append_log(format!("world_game_launcher started (pid={pid})"));
            state.mark_updated();
            Ok(())
        }
        Err(err) => {
            state.process_state = ProcessState::StartFailed(err.clone());
            state.append_log(format!("start failed: {err}"));
            state.mark_updated();
            Err(err)
        }
    }
}

fn stop_process(state: &mut ServiceState) -> Result<(), String> {
    let Some(mut running) = state.running.take() else {
        state.process_state = ProcessState::Stopped;
        state.append_log("world_game_launcher stop requested but process is not running");
        state.mark_updated();
        return Ok(());
    };

    match stop_child_process(&mut running.child) {
        Ok(()) => {
            state.process_state = ProcessState::Stopped;
            state.append_log("world_game_launcher stopped");
            state.mark_updated();
            Ok(())
        }
        Err(err) => {
            state.process_state = ProcessState::StopFailed(err.clone());
            state.append_log(format!("stop failed: {err}"));
            state.mark_updated();
            Err(err)
        }
    }
}

fn snapshot_from_state(state: &ServiceState, request_host: Option<&str>) -> StateSnapshot {
    let game_url = build_game_url(&state.config, request_host);
    StateSnapshot {
        status: state.process_state.code().to_string(),
        detail: state.process_state.detail(),
        pid: state.process_state.pid(),
        running: matches!(state.process_state, ProcessState::Running { .. }),
        launcher_bin: state.launcher_bin.clone(),
        game_url,
        config: state.config.clone(),
        logs: state.logs.iter().cloned().collect(),
        updated_at_unix_ms: state.updated_at_unix_ms,
    }
}

fn build_game_url(config: &LauncherConfig, request_host: Option<&str>) -> String {
    let viewer_host = resolve_runtime_host(config.viewer_host.as_str(), request_host);
    let viewer_port =
        parse_port(config.viewer_port.as_str(), "viewer port").unwrap_or(DEFAULT_VIEWER_PORT);
    let (web_host, web_port) = parse_host_port(config.web_bind.as_str(), "web bind")
        .unwrap_or((DEFAULT_VIEWER_HOST.to_string(), 5011));
    let web_host = resolve_runtime_host(web_host.as_str(), request_host);
    let viewer_host = host_for_url(viewer_host.as_str());
    let web_host = host_for_url(web_host.as_str());
    format!("http://{viewer_host}:{viewer_port}/?ws=ws://{web_host}:{web_port}")
}

fn resolve_runtime_host(config_host: &str, request_host: Option<&str>) -> String {
    let config_host = config_host.trim();
    if config_host.is_empty()
        || config_host == "0.0.0.0"
        || config_host == "::"
        || config_host == "[::]"
        || config_host == "127.0.0.1"
        || config_host == "localhost"
    {
        if let Some(request_host) = request_host {
            let request_host = request_host.trim();
            if !request_host.is_empty() {
                return request_host.to_string();
            }
        }
        return "127.0.0.1".to_string();
    }
    config_host.to_string()
}

fn host_for_url(host: &str) -> String {
    if host.contains(':') && !host.starts_with('[') && !host.ends_with(']') {
        format!("[{host}]")
    } else {
        host.to_string()
    }
}

fn validate_config(config: &LauncherConfig) -> Vec<String> {
    let mut issues = Vec::new();
    if config.scenario.trim().is_empty() {
        issues.push("scenario is required".to_string());
    }
    if parse_host_port(config.live_bind.as_str(), "live bind").is_err() {
        issues.push("live bind must be in <host:port> format".to_string());
    }
    if parse_host_port(config.web_bind.as_str(), "web bind").is_err() {
        issues.push("web bind must be in <host:port> format".to_string());
    }
    if config.viewer_host.trim().is_empty() {
        issues.push("viewer host is required".to_string());
    }
    if parse_port(config.viewer_port.as_str(), "viewer port").is_err() {
        issues.push("viewer port must be integer in 1..=65535".to_string());
    }

    let viewer_static_dir = config.viewer_static_dir.trim();
    if viewer_static_dir.is_empty() {
        issues.push("viewer static directory is required".to_string());
    } else if !Path::new(viewer_static_dir).is_dir() {
        issues.push(format!(
            "viewer static directory does not exist or is not a directory: {viewer_static_dir}"
        ));
    }

    if config.chain_enabled {
        if parse_host_port(config.chain_status_bind.as_str(), "chain status bind").is_err() {
            issues.push("chain status bind must be in <host:port> format".to_string());
        }
        if config.chain_node_id.trim().is_empty() {
            issues.push("chain node id is required".to_string());
        }
        if parse_chain_role(config.chain_node_role.as_str()).is_err() {
            issues.push("chain role must be one of: sequencer|storage|observer".to_string());
        }
        if parse_port(config.chain_node_tick_ms.as_str(), "chain node tick ms").is_err() {
            issues.push("chain node tick ms must be integer in 1..=65535".to_string());
        }
        if parse_chain_validators(config.chain_node_validators.as_str()).is_err() {
            issues.push("chain validators must be in <validator_id:stake> format".to_string());
        }
    }

    issues
}

fn build_launcher_args(config: &LauncherConfig) -> Result<Vec<String>, String> {
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

    if config.llm_enabled {
        args.push("--with-llm".to_string());
    } else {
        args.push("--no-llm".to_string());
    }
    if !config.auto_open_browser {
        args.push("--no-open-browser".to_string());
    }

    if config.chain_enabled {
        parse_host_port(config.chain_status_bind.as_str(), "chain status bind")?;
        let chain_role = parse_chain_role(config.chain_node_role.as_str())?;
        let chain_tick_ms = parse_port(config.chain_node_tick_ms.as_str(), "chain node tick ms")?;
        let validators = parse_chain_validators(config.chain_node_validators.as_str())?;
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
        args.push(chain_role);
        args.push("--chain-node-tick-ms".to_string());
        args.push(chain_tick_ms.to_string());
        for validator in validators {
            args.push("--chain-node-validator".to_string());
            args.push(validator);
        }
    } else {
        args.push("--chain-disable".to_string());
    }

    Ok(args)
}

fn spawn_child_process(bin: &str, args: &[String]) -> Result<RunningProcess, String> {
    let mut child = Command::new(bin)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("spawn process `{bin}` failed: {err}"))?;

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
    thread::spawn(move || {
        let buffered = BufReader::new(reader);
        for line in buffered.lines() {
            match line {
                Ok(content) => {
                    let _ = tx.send(format!("[launcher {source}] {content}"));
                }
                Err(err) => {
                    let _ = tx.send(format!("[launcher {source}] <read error: {err}>"));
                    break;
                }
            }
        }
    });
}

fn stop_child_process(child: &mut Child) -> Result<(), String> {
    if child
        .try_wait()
        .map_err(|err| format!("query child status failed: {err}"))?
        .is_some()
    {
        return Ok(());
    }

    if let Err(err) = send_interrupt_signal(child) {
        eprintln!("warning: failed to request graceful process stop: {err}");
    } else {
        let deadline = Instant::now() + Duration::from_millis(GRACEFUL_STOP_TIMEOUT_MS);
        while Instant::now() < deadline {
            if child
                .try_wait()
                .map_err(|err| format!("query child status failed: {err}"))?
                .is_some()
            {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(STOP_POLL_INTERVAL_MS));
        }
    }

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

fn send_interrupt_signal(child: &Child) -> Result<(), String> {
    #[cfg(unix)]
    {
        let pid = child.id().to_string();
        let status = Command::new("kill")
            .arg("-INT")
            .arg(pid.as_str())
            .status()
            .map_err(|err| format!("run kill -INT failed: {err}"))?;
        if status.success() {
            return Ok(());
        }
        return Err(format!("kill -INT exited with {status}"));
    }

    #[cfg(not(unix))]
    {
        let _ = child;
        Ok(())
    }
}

fn parse_options<'a, I>(args: I) -> Result<CliOptions, String>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut options = CliOptions::default();
    let mut validators: Vec<String> = Vec::new();
    let mut iter = args.into_iter().peekable();

    while let Some(arg) = iter.next() {
        match arg {
            "--listen-bind" => {
                options.listen_bind = next_value(&mut iter, "--listen-bind")?;
            }
            "--launcher-bin" => {
                options.launcher_bin = next_value(&mut iter, "--launcher-bin")?;
            }
            "--scenario" => {
                options.initial_config.scenario = next_value(&mut iter, "--scenario")?;
            }
            "--live-bind" => {
                options.initial_config.live_bind = next_value(&mut iter, "--live-bind")?;
            }
            "--web-bind" => {
                options.initial_config.web_bind = next_value(&mut iter, "--web-bind")?;
            }
            "--viewer-host" => {
                options.initial_config.viewer_host = next_value(&mut iter, "--viewer-host")?;
            }
            "--viewer-port" => {
                options.initial_config.viewer_port = next_value(&mut iter, "--viewer-port")?;
            }
            "--viewer-static-dir" => {
                options.initial_config.viewer_static_dir =
                    next_value(&mut iter, "--viewer-static-dir")?;
            }
            "--with-llm" => {
                options.initial_config.llm_enabled = true;
            }
            "--no-llm" => {
                options.initial_config.llm_enabled = false;
            }
            "--chain-enable" => {
                options.initial_config.chain_enabled = true;
            }
            "--chain-disable" => {
                options.initial_config.chain_enabled = false;
            }
            "--chain-status-bind" => {
                options.initial_config.chain_status_bind =
                    next_value(&mut iter, "--chain-status-bind")?;
            }
            "--chain-node-id" => {
                options.initial_config.chain_node_id = next_value(&mut iter, "--chain-node-id")?;
            }
            "--chain-world-id" => {
                options.initial_config.chain_world_id = next_value(&mut iter, "--chain-world-id")?;
            }
            "--chain-node-role" => {
                options.initial_config.chain_node_role =
                    next_value(&mut iter, "--chain-node-role")?;
            }
            "--chain-node-tick-ms" => {
                options.initial_config.chain_node_tick_ms =
                    next_value(&mut iter, "--chain-node-tick-ms")?;
            }
            "--chain-node-validator" => {
                validators.push(next_value(&mut iter, "--chain-node-validator")?);
            }
            "--open-browser" => {
                options.initial_config.auto_open_browser = true;
            }
            "--no-open-browser" => {
                options.initial_config.auto_open_browser = false;
            }
            unknown => {
                return Err(format!("unknown option: {unknown}"));
            }
        }
    }

    if !validators.is_empty() {
        options.initial_config.chain_node_validators = validators.join(",");
    }

    parse_host_port(options.listen_bind.as_str(), "--listen-bind")?;
    parse_port(options.initial_config.viewer_port.as_str(), "--viewer-port")?;
    parse_host_port(options.initial_config.live_bind.as_str(), "--live-bind")?;
    parse_host_port(options.initial_config.web_bind.as_str(), "--web-bind")?;
    if options.initial_config.chain_enabled {
        parse_host_port(
            options.initial_config.chain_status_bind.as_str(),
            "--chain-status-bind",
        )?;
        parse_chain_role(options.initial_config.chain_node_role.as_str())?;
        parse_port(
            options.initial_config.chain_node_tick_ms.as_str(),
            "--chain-node-tick-ms",
        )?;
        parse_chain_validators(options.initial_config.chain_node_validators.as_str())?;
    }
    if options.launcher_bin.trim().is_empty() {
        return Err("--launcher-bin must not be empty".to_string());
    }

    Ok(options)
}

fn next_value<'a, I>(iter: &mut std::iter::Peekable<I>, flag: &str) -> Result<String, String>
where
    I: Iterator<Item = &'a str>,
{
    iter.next()
        .map(str::to_string)
        .ok_or_else(|| format!("{flag} requires a value"))
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
    if host.is_empty() {
        return Err(format!("{label} host cannot be empty"));
    }
    let port = parse_port(port_raw, label)?;
    Ok((host.to_string(), port))
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

fn resolve_world_game_launcher_binary() -> PathBuf {
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

    PathBuf::from(DEFAULT_VIEWER_STATIC_DIR)
}

fn binary_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}

fn normalize_bind_host_for_local_access(host: &str) -> String {
    let host = host.trim();
    if host == "0.0.0.0" || host == "::" || host == "[::]" {
        "127.0.0.1".to_string()
    } else {
        host.to_string()
    }
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

const WEB_CONSOLE_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Agent World Web Launcher</title>
  <style>
    :root {
      --bg: #f2f4ef;
      --panel: #ffffff;
      --line: #d0d8cc;
      --text: #1f2a22;
      --muted: #5f6d63;
      --ok: #1c7f53;
      --err: #a13a35;
      --accent: #125f9a;
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: "IBM Plex Sans", "Noto Sans", sans-serif;
      background: radial-gradient(circle at 0 0, #f9faf7, var(--bg));
      color: var(--text);
    }
    .wrap {
      max-width: 1100px;
      margin: 0 auto;
      padding: 24px 16px 32px;
      display: grid;
      gap: 16px;
    }
    .card {
      background: var(--panel);
      border: 1px solid var(--line);
      border-radius: 14px;
      padding: 16px;
      box-shadow: 0 10px 30px rgba(23, 37, 26, 0.06);
    }
    h1 {
      margin: 0 0 8px;
      font-size: 26px;
      letter-spacing: 0.2px;
    }
    .status {
      display: flex;
      gap: 12px;
      flex-wrap: wrap;
      align-items: center;
    }
    .badge {
      display: inline-flex;
      align-items: center;
      border-radius: 999px;
      padding: 4px 12px;
      font-size: 13px;
      background: #eaf1e8;
      border: 1px solid #c4d5c0;
    }
    .badge.running { color: var(--ok); border-color: #87c9a9; background: #e8f8ef; }
    .badge.failed, .badge.exited, .badge.invalid_config { color: var(--err); border-color: #e6b0ac; background: #fff0ef; }
    .row {
      display: grid;
      gap: 10px;
      grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
      margin-bottom: 10px;
    }
    label {
      display: block;
      font-size: 12px;
      color: var(--muted);
      margin-bottom: 4px;
    }
    input[type="text"], input[type="number"] {
      width: 100%;
      border: 1px solid var(--line);
      border-radius: 8px;
      padding: 8px 10px;
      font-size: 14px;
      color: var(--text);
    }
    .toggles {
      display: flex;
      gap: 14px;
      flex-wrap: wrap;
      margin-top: 8px;
    }
    .toggles label {
      display: inline-flex;
      gap: 6px;
      align-items: center;
      margin: 0;
      font-size: 13px;
      color: var(--text);
    }
    .btns {
      margin-top: 14px;
      display: flex;
      gap: 10px;
      flex-wrap: wrap;
    }
    button {
      border: 1px solid transparent;
      border-radius: 8px;
      padding: 8px 12px;
      font-size: 14px;
      cursor: pointer;
      transition: transform 120ms ease;
    }
    button:hover { transform: translateY(-1px); }
    button.start { background: var(--ok); color: #fff; }
    button.stop { background: var(--err); color: #fff; }
    button.refresh { background: var(--accent); color: #fff; }
    pre {
      margin: 0;
      padding: 12px;
      border: 1px solid var(--line);
      border-radius: 10px;
      background: #f8faf6;
      max-height: 340px;
      overflow: auto;
      font-size: 12px;
      line-height: 1.35;
      white-space: pre-wrap;
      word-break: break-word;
    }
    .meta {
      font-size: 13px;
      color: var(--muted);
    }
    a { color: var(--accent); }
  </style>
</head>
<body>
  <div class="wrap">
    <section class="card">
      <h1>Agent World Web Launcher</h1>
      <div class="status">
        <span id="status-badge" class="badge">status: idle</span>
        <span class="meta" id="status-detail"></span>
      </div>
      <div class="meta" id="game-url">game url: -</div>
      <div class="meta" id="launcher-bin">launcher bin: -</div>
    </section>

    <section class="card">
      <h2>Launch Config</h2>
      <div class="row">
        <div><label>Scenario</label><input id="scenario" type="text" /></div>
        <div><label>Live Bind</label><input id="live_bind" type="text" /></div>
        <div><label>Web Bind</label><input id="web_bind" type="text" /></div>
      </div>
      <div class="row">
        <div><label>Viewer Host</label><input id="viewer_host" type="text" /></div>
        <div><label>Viewer Port</label><input id="viewer_port" type="text" /></div>
        <div><label>Viewer Static Dir</label><input id="viewer_static_dir" type="text" /></div>
      </div>
      <div class="row">
        <div><label>Chain Status Bind</label><input id="chain_status_bind" type="text" /></div>
        <div><label>Chain Node ID</label><input id="chain_node_id" type="text" /></div>
        <div><label>Chain World ID</label><input id="chain_world_id" type="text" /></div>
      </div>
      <div class="row">
        <div><label>Chain Role</label><input id="chain_node_role" type="text" /></div>
        <div><label>Chain Tick ms</label><input id="chain_node_tick_ms" type="text" /></div>
        <div><label>Chain Validators</label><input id="chain_node_validators" type="text" /></div>
      </div>
      <div class="toggles">
        <label><input id="llm_enabled" type="checkbox" /> Enable LLM</label>
        <label><input id="chain_enabled" type="checkbox" /> Enable Chain Runtime</label>
        <label><input id="auto_open_browser" type="checkbox" /> Auto Open Browser</label>
      </div>
      <div class="btns">
        <button class="start" id="start-btn">Start</button>
        <button class="stop" id="stop-btn">Stop</button>
        <button class="refresh" id="refresh-btn">Refresh</button>
      </div>
    </section>

    <section class="card">
      <h2>Logs</h2>
      <pre id="logs">(empty)</pre>
    </section>
  </div>

  <script>
    const ids = [
      "scenario", "live_bind", "web_bind", "viewer_host", "viewer_port", "viewer_static_dir",
      "chain_status_bind", "chain_node_id", "chain_world_id", "chain_node_role",
      "chain_node_tick_ms", "chain_node_validators"
    ];
    const boolIds = ["llm_enabled", "chain_enabled", "auto_open_browser"];

    async function requestJson(path, options = {}) {
      const resp = await fetch(path, {
        headers: { "content-type": "application/json" },
        ...options,
      });
      if (!resp.ok) {
        throw new Error(`HTTP ${resp.status}`);
      }
      return await resp.json();
    }

    function applyConfig(config) {
      ids.forEach((id) => {
        const el = document.getElementById(id);
        if (el && Object.prototype.hasOwnProperty.call(config, id)) {
          el.value = config[id] ?? "";
        }
      });
      boolIds.forEach((id) => {
        const el = document.getElementById(id);
        if (el && Object.prototype.hasOwnProperty.call(config, id)) {
          el.checked = Boolean(config[id]);
        }
      });
    }

    function collectConfig() {
      const config = {};
      ids.forEach((id) => {
        const el = document.getElementById(id);
        config[id] = el ? String(el.value || "") : "";
      });
      boolIds.forEach((id) => {
        const el = document.getElementById(id);
        config[id] = Boolean(el && el.checked);
      });
      return config;
    }

    function renderState(state) {
      const badge = document.getElementById("status-badge");
      badge.textContent = `status: ${state.status}${state.pid ? ` (pid=${state.pid})` : ""}`;
      badge.className = `badge ${state.status}`;
      document.getElementById("status-detail").textContent = state.detail || "";

      const gameUrl = state.game_url || "-";
      document.getElementById("game-url").innerHTML =
        gameUrl === "-" ? "game url: -" : `game url: <a href="${gameUrl}" target="_blank" rel="noreferrer">${gameUrl}</a>`;
      document.getElementById("launcher-bin").textContent = `launcher bin: ${state.launcher_bin}`;

      const logs = state.logs && state.logs.length ? state.logs.join("\n") : "(empty)";
      document.getElementById("logs").textContent = logs;

      applyConfig(state.config || {});
    }

    async function refreshState() {
      try {
        const state = await requestJson("/api/state");
        renderState(state);
      } catch (err) {
        document.getElementById("status-detail").textContent = `refresh failed: ${err.message}`;
      }
    }

    async function startLauncher() {
      const config = collectConfig();
      try {
        const result = await requestJson("/api/start", {
          method: "POST",
          body: JSON.stringify(config),
        });
        renderState(result.state);
        if (!result.ok) {
          document.getElementById("status-detail").textContent = result.error || "start failed";
        }
      } catch (err) {
        document.getElementById("status-detail").textContent = `start failed: ${err.message}`;
      }
    }

    async function stopLauncher() {
      try {
        const result = await requestJson("/api/stop", { method: "POST" });
        renderState(result.state);
        if (!result.ok) {
          document.getElementById("status-detail").textContent = result.error || "stop failed";
        }
      } catch (err) {
        document.getElementById("status-detail").textContent = `stop failed: ${err.message}`;
      }
    }

    document.getElementById("refresh-btn").addEventListener("click", refreshState);
    document.getElementById("start-btn").addEventListener("click", startLauncher);
    document.getElementById("stop-btn").addEventListener("click", stopLauncher);

    refreshState();
    setInterval(refreshState, 1000);
  </script>
</body>
</html>
"#;

#[cfg(test)]
#[path = "world_web_launcher/world_web_launcher_tests.rs"]
mod world_web_launcher_tests;
