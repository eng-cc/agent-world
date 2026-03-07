use std::collections::VecDeque;
use std::env;
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{self, Child};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use agent_world_launcher_ui::launcher_ui_fields_for_web;
use serde::{Deserialize, Serialize};

#[path = "world_web_launcher/control_plane.rs"]
mod control_plane;
#[path = "world_web_launcher/http_codec.rs"]
mod http_codec;
#[path = "world_web_launcher/parse_utils.rs"]
mod parse_utils;
#[path = "world_web_launcher/runtime_paths.rs"]
mod runtime_paths;
#[path = "world_web_launcher/static_files.rs"]
mod static_files;
#[path = "world_web_launcher/transfer_query_proxy.rs"]
mod transfer_query_proxy;

use control_plane::*;
use http_codec::{read_http_request, write_http_response, write_json_response};
use parse_utils::{
    next_value, parse_chain_role, parse_chain_validators, parse_host_port, parse_non_negative_u64,
    parse_optional_i64, parse_port, parse_positive_u64,
};
use runtime_paths::{
    normalize_bind_host_for_local_access, now_unix_ms, resolve_console_static_dir_path,
    resolve_static_dir_path, resolve_world_game_launcher_binary,
};
use static_files::{load_console_static_asset, StaticAsset};
use transfer_query_proxy::query_chain_transfer_json;

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
const DEFAULT_CHAIN_POS_SLOT_DURATION_MS: u64 = 1;
const DEFAULT_CHAIN_POS_TICKS_PER_SLOT: u64 = 1;
const DEFAULT_CHAIN_POS_PROPOSAL_TICK_PHASE: u64 = 0;
const DEFAULT_CHAIN_POS_MAX_PAST_SLOT_LAG: u64 = 256;
const MAX_LOG_LINES: usize = 2000;
const GRACEFUL_STOP_TIMEOUT_MS: u64 = 4000;
const STOP_POLL_INTERVAL_MS: u64 = 80;
const CHAIN_STATUS_PROBE_INTERVAL_MS: u64 = 1000;
const CHAIN_STATUS_PROBE_TIMEOUT_MS: u64 = 300;
const CHAIN_STATUS_STARTING_GRACE_SECS: u64 = 8;

static TERMINATION_REQUESTED: AtomicBool = AtomicBool::new(false);
static SIGNAL_HANDLER_INSTALL: OnceLock<Result<(), String>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
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

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            scenario: DEFAULT_SCENARIO.to_string(),
            live_bind: DEFAULT_LIVE_BIND.to_string(),
            web_bind: DEFAULT_WEB_BIND.to_string(),
            viewer_host: DEFAULT_VIEWER_HOST.to_string(),
            viewer_port: DEFAULT_VIEWER_PORT.to_string(),
            viewer_static_dir: resolve_static_dir_path(DEFAULT_VIEWER_STATIC_DIR)
                .to_string_lossy()
                .to_string(),
            llm_enabled: false,
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
            chain_pos_slot_clock_genesis_unix_ms: String::new(),
            chain_pos_max_past_slot_lag: DEFAULT_CHAIN_POS_MAX_PAST_SLOT_LAG.to_string(),
            chain_node_validators: String::new(),
            auto_open_browser: false,
            launcher_bin: String::new(),
            chain_runtime_bin: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliOptions {
    listen_bind: String,
    launcher_bin: String,
    chain_runtime_bin: String,
    console_static_dir: PathBuf,
    initial_config: LauncherConfig,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            listen_bind: DEFAULT_LISTEN_BIND.to_string(),
            launcher_bin: resolve_world_game_launcher_binary()
                .to_string_lossy()
                .to_string(),
            chain_runtime_bin: runtime_paths::resolve_world_chain_runtime_binary()
                .to_string_lossy()
                .to_string(),
            console_static_dir: resolve_console_static_dir_path(),
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

#[derive(Debug, Clone)]
enum ChainRuntimeStatus {
    Disabled,
    NotStarted,
    Starting,
    Ready,
    Unreachable(String),
    ConfigError(String),
}

impl ChainRuntimeStatus {
    fn code(&self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::NotStarted => "not_started",
            Self::Starting => "starting",
            Self::Ready => "ready",
            Self::Unreachable(_) => "unreachable",
            Self::ConfigError(_) => "config_error",
        }
    }

    fn detail(&self) -> Option<String> {
        match self {
            Self::Unreachable(detail) | Self::ConfigError(detail) => Some(detail.clone()),
            Self::Disabled | Self::NotStarted | Self::Starting | Self::Ready => None,
        }
    }
}

#[derive(Debug)]
struct ServiceState {
    launcher_bin: String,
    chain_runtime_bin: String,
    console_static_dir: PathBuf,
    config: LauncherConfig,
    process_state: ProcessState,
    running: Option<RunningProcess>,
    chain_runtime_status: ChainRuntimeStatus,
    chain_running: Option<RunningProcess>,
    chain_started_at: Option<Instant>,
    last_chain_probe_at: Option<Instant>,
    logs: VecDeque<String>,
    updated_at_unix_ms: u64,
}

impl ServiceState {
    fn new(
        launcher_bin: String,
        chain_runtime_bin: String,
        console_static_dir: PathBuf,
        config: LauncherConfig,
    ) -> Self {
        let chain_runtime_status = if config.chain_enabled {
            ChainRuntimeStatus::NotStarted
        } else {
            ChainRuntimeStatus::Disabled
        };
        Self {
            launcher_bin,
            chain_runtime_bin,
            console_static_dir,
            config,
            process_state: ProcessState::Idle,
            running: None,
            chain_runtime_status,
            chain_running: None,
            chain_started_at: None,
            last_chain_probe_at: None,
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

#[derive(Debug, Serialize)]
struct StateSnapshot {
    status: String,
    detail: Option<String>,
    pid: Option<u32>,
    running: bool,
    launcher_bin: String,
    chain_status: String,
    chain_detail: Option<String>,
    chain_pid: Option<u32>,
    chain_running: bool,
    chain_runtime_bin: String,
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ChainTransferSubmitRequest {
    from_account_id: String,
    to_account_id: String,
    amount: u64,
    nonce: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ChainTransferSubmitResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    action_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    submitted_at_unix_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl ChainTransferSubmitResponse {
    fn error(error_code: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            ok: false,
            action_id: None,
            submitted_at_unix_ms: None,
            error_code: Some(error_code.into()),
            error: Some(error.into()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ChainFeedbackSubmitRequest {
    category: String,
    title: String,
    description: String,
    platform: String,
    game_version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ChainFeedbackSubmitResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    feedback_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl ChainFeedbackSubmitResponse {
    fn error(error: impl Into<String>) -> Self {
        Self {
            ok: false,
            feedback_id: None,
            event_id: None,
            error: Some(error.into()),
        }
    }
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
  --chain-runtime-bin <path>      world_chain_runtime binary path\n\
  --console-static-dir <path>     launcher web static directory (default: ../web-launcher)\n\
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
  --chain-node-tick-ms <ms>       worker poll/fallback interval ms\n\
  --chain-pos-slot-duration-ms <n>\n\
  --chain-pos-ticks-per-slot <n>\n\
  --chain-pos-proposal-tick-phase <n>\n\
  --chain-pos-adaptive-tick-scheduler / --chain-pos-no-adaptive-tick-scheduler\n\
  --chain-pos-slot-clock-genesis-unix-ms <n>\n\
  --chain-pos-max-past-slot-lag <n>\n\
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
        options.chain_runtime_bin,
        options.console_static_dir,
        options.initial_config,
    )));

    println!("world_web_launcher started");
    println!(
        "- console: http://{}:{}",
        normalize_bind_host_for_local_access(listen_host.as_str()),
        listen_port
    );
    println!("- listen bind: {listen_host}:{listen_port}");
    println!(
        "- console static dir: {}",
        lock_state(&state).console_static_dir.display()
    );
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
    poll_service_state(&mut state_guard);
    let _ = stop_process(&mut state_guard);
    let _ = stop_chain_process(&mut state_guard);
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
        ("GET", "/healthz") => {
            write_http_response(&mut stream, 200, "text/plain; charset=utf-8", b"ok", false)?;
            Ok(())
        }
        ("GET", "/api/state") => {
            let request_host = extract_host_header(request.headers.as_slice());
            let mut state = lock_state(&shared_state);
            poll_service_state(&mut state);
            let snapshot = snapshot_from_state(&state, request_host.as_deref());
            write_json_response(&mut stream, 200, &snapshot)
        }
        ("GET", "/api/ui/schema") => {
            let schema: Vec<_> = launcher_ui_fields_for_web().copied().collect();
            write_json_response(&mut stream, 200, &schema)
        }
        ("POST", "/api/start") => {
            let request_host = extract_host_header(request.headers.as_slice());
            let config = parse_config_request(request.body.as_slice(), "start")?;
            let mut state = lock_state(&shared_state);
            poll_service_state(&mut state);
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
            poll_service_state(&mut state);
            let outcome = stop_process(&mut state);
            poll_service_state(&mut state);
            let snapshot = snapshot_from_state(&state, request_host.as_deref());
            let response = ApiResponse {
                ok: outcome.is_ok(),
                error: outcome.err(),
                state: snapshot,
            };
            write_json_response(&mut stream, 200, &response)
        }
        ("POST", "/api/chain/start") => {
            let request_host = extract_host_header(request.headers.as_slice());
            let config = parse_config_request(request.body.as_slice(), "chain start")?;
            let mut state = lock_state(&shared_state);
            poll_service_state(&mut state);
            let outcome = start_chain_process(&mut state, config);
            poll_service_state(&mut state);
            let snapshot = snapshot_from_state(&state, request_host.as_deref());
            let response = ApiResponse {
                ok: outcome.is_ok(),
                error: outcome.err(),
                state: snapshot,
            };
            write_json_response(&mut stream, 200, &response)
        }
        ("POST", "/api/chain/stop") => {
            let request_host = extract_host_header(request.headers.as_slice());
            let mut state = lock_state(&shared_state);
            poll_service_state(&mut state);
            let outcome = stop_chain_process(&mut state);
            poll_service_state(&mut state);
            let snapshot = snapshot_from_state(&state, request_host.as_deref());
            let response = ApiResponse {
                ok: outcome.is_ok(),
                error: outcome.err(),
                state: snapshot,
            };
            write_json_response(&mut stream, 200, &response)
        }
        ("POST", "/api/chain/transfer") => {
            let submit_request = match parse_chain_transfer_request(request.body.as_slice()) {
                Ok(request) => request,
                Err(err) => {
                    let response = ChainTransferSubmitResponse::error("invalid_request", err);
                    return write_json_response(&mut stream, 200, &response);
                }
            };
            let mut state = lock_state(&shared_state);
            poll_service_state(&mut state);
            let response = submit_chain_transfer(&mut state, &submit_request);
            write_json_response(&mut stream, 200, &response)
        }
        ("GET", "/api/chain/transfer/accounts") => {
            let mut state = lock_state(&shared_state);
            poll_service_state(&mut state);
            let response = query_chain_transfer_json(&mut state, "/v1/chain/transfer/accounts");
            write_json_response(&mut stream, 200, &response)
        }
        ("GET", "/api/chain/transfer/status") => {
            let mut state = lock_state(&shared_state);
            poll_service_state(&mut state);
            let runtime_target = remap_transfer_runtime_target(
                request.path.as_str(),
                "/api/chain/transfer/status",
                "/v1/chain/transfer/status",
            );
            let response = query_chain_transfer_json(&mut state, runtime_target.as_str());
            write_json_response(&mut stream, 200, &response)
        }
        ("GET", "/api/chain/transfer/history") => {
            let mut state = lock_state(&shared_state);
            poll_service_state(&mut state);
            let runtime_target = remap_transfer_runtime_target(
                request.path.as_str(),
                "/api/chain/transfer/history",
                "/v1/chain/transfer/history",
            );
            let response = query_chain_transfer_json(&mut state, runtime_target.as_str());
            write_json_response(&mut stream, 200, &response)
        }
        ("POST", "/api/chain/feedback") => {
            let submit_request = match parse_chain_feedback_request(request.body.as_slice()) {
                Ok(request) => request,
                Err(err) => {
                    let response = ChainFeedbackSubmitResponse::error(err);
                    return write_json_response(&mut stream, 200, &response);
                }
            };
            let mut state = lock_state(&shared_state);
            poll_service_state(&mut state);
            let response = submit_chain_feedback(&mut state, &submit_request);
            write_json_response(&mut stream, 200, &response)
        }
        ("OPTIONS", _) => {
            write_http_response(&mut stream, 204, "text/plain", b"", false)?;
            Ok(())
        }
        ("GET", request_path) if !request_path.starts_with("/api/") => {
            serve_console_static_request(&mut stream, request_path, &shared_state)
        }
        (method, "/api/state")
        | (method, "/api/start")
        | (method, "/api/stop")
        | (method, "/api/chain/start")
        | (method, "/api/chain/stop")
        | (method, "/api/chain/transfer")
        | (method, "/api/chain/transfer/accounts")
        | (method, "/api/chain/transfer/status")
        | (method, "/api/chain/transfer/history")
        | (method, "/api/chain/feedback")
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

fn serve_console_static_request(
    stream: &mut TcpStream,
    request_path: &str,
    shared_state: &Arc<Mutex<ServiceState>>,
) -> Result<(), String> {
    let console_static_dir = {
        let state = lock_state(shared_state);
        state.console_static_dir.clone()
    };

    match load_console_static_asset(console_static_dir.as_path(), request_path) {
        StaticAsset::Ok { content_type, body } => {
            write_http_response(stream, 200, content_type, body.as_slice(), false)
        }
        StaticAsset::NotFound => write_http_response(
            stream,
            404,
            "text/plain; charset=utf-8",
            b"Not Found",
            false,
        ),
        StaticAsset::InvalidPath => write_http_response(
            stream,
            400,
            "text/plain; charset=utf-8",
            b"Bad Request",
            false,
        ),
    }
}

fn strip_query(path: &str) -> &str {
    path.split('?').next().unwrap_or(path)
}

fn remap_transfer_runtime_target(
    request_path: &str,
    api_prefix: &str,
    runtime_prefix: &str,
) -> String {
    let suffix = request_path.strip_prefix(api_prefix).unwrap_or_default();
    format!("{runtime_prefix}{suffix}")
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
            "--chain-runtime-bin" => {
                options.chain_runtime_bin = next_value(&mut iter, "--chain-runtime-bin")?;
            }
            "--console-static-dir" => {
                options.console_static_dir =
                    PathBuf::from(next_value(&mut iter, "--console-static-dir")?);
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
            "--chain-pos-slot-duration-ms" => {
                options.initial_config.chain_pos_slot_duration_ms =
                    next_value(&mut iter, "--chain-pos-slot-duration-ms")?;
            }
            "--chain-pos-ticks-per-slot" => {
                options.initial_config.chain_pos_ticks_per_slot =
                    next_value(&mut iter, "--chain-pos-ticks-per-slot")?;
            }
            "--chain-pos-proposal-tick-phase" => {
                options.initial_config.chain_pos_proposal_tick_phase =
                    next_value(&mut iter, "--chain-pos-proposal-tick-phase")?;
            }
            "--chain-pos-adaptive-tick-scheduler" => {
                options
                    .initial_config
                    .chain_pos_adaptive_tick_scheduler_enabled = true;
            }
            "--chain-pos-no-adaptive-tick-scheduler" => {
                options
                    .initial_config
                    .chain_pos_adaptive_tick_scheduler_enabled = false;
            }
            "--chain-pos-slot-clock-genesis-unix-ms" => {
                options.initial_config.chain_pos_slot_clock_genesis_unix_ms =
                    next_value(&mut iter, "--chain-pos-slot-clock-genesis-unix-ms")?;
            }
            "--chain-pos-max-past-slot-lag" => {
                options.initial_config.chain_pos_max_past_slot_lag =
                    next_value(&mut iter, "--chain-pos-max-past-slot-lag")?;
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
    options.initial_config.launcher_bin = options.launcher_bin.trim().to_string();
    options.initial_config.chain_runtime_bin = options.chain_runtime_bin.trim().to_string();

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
        parse_positive_u64(
            options.initial_config.chain_node_tick_ms.as_str(),
            "--chain-node-tick-ms",
        )?;
        parse_positive_u64(
            options.initial_config.chain_pos_slot_duration_ms.as_str(),
            "--chain-pos-slot-duration-ms",
        )?;
        let ticks_per_slot = parse_positive_u64(
            options.initial_config.chain_pos_ticks_per_slot.as_str(),
            "--chain-pos-ticks-per-slot",
        )?;
        let proposal_tick_phase = parse_non_negative_u64(
            options
                .initial_config
                .chain_pos_proposal_tick_phase
                .as_str(),
            "--chain-pos-proposal-tick-phase",
        )?;
        if proposal_tick_phase >= ticks_per_slot {
            return Err(format!(
                "--chain-pos-proposal-tick-phase={} must be less than --chain-pos-ticks-per-slot={}",
                proposal_tick_phase, ticks_per_slot
            ));
        }
        parse_optional_i64(
            options
                .initial_config
                .chain_pos_slot_clock_genesis_unix_ms
                .as_str(),
            "--chain-pos-slot-clock-genesis-unix-ms",
        )?;
        parse_non_negative_u64(
            options.initial_config.chain_pos_max_past_slot_lag.as_str(),
            "--chain-pos-max-past-slot-lag",
        )?;
        parse_chain_validators(options.initial_config.chain_node_validators.as_str())?;
    }
    if options.launcher_bin.trim().is_empty() {
        return Err("--launcher-bin must not be empty".to_string());
    }
    if options.chain_runtime_bin.trim().is_empty() {
        return Err("--chain-runtime-bin must not be empty".to_string());
    }
    if options.console_static_dir.as_os_str().is_empty() {
        return Err("--console-static-dir must not be empty".to_string());
    }

    Ok(options)
}

#[cfg(test)]
#[path = "world_web_launcher/world_web_launcher_tests.rs"]
mod world_web_launcher_tests;
