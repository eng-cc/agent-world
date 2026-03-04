use super::*;

use std::io::{BufRead, BufReader, Read};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

pub(super) fn parse_config_request(body: &[u8], action: &str) -> Result<LauncherConfig, String> {
    serde_json::from_slice(body).map_err(|err| format!("parse {action} request JSON failed: {err}"))
}

pub(super) fn poll_service_state(state: &mut ServiceState) {
    poll_process_state(state);
    poll_chain_process_state(state);
    update_chain_runtime_status(state);
}

pub(super) fn poll_process_state(state: &mut ServiceState) {
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

pub(super) fn poll_chain_process_state(state: &mut ServiceState) {
    let Some(mut running) = state.chain_running.take() else {
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
            state.chain_started_at = None;
            state.last_chain_probe_at = None;
            state.chain_runtime_status =
                ChainRuntimeStatus::Unreachable(format!("world_chain_runtime exited: {status}"));
            state.append_log(format!("world_chain_runtime exited: {status}"));
            state.mark_updated();
        }
        Ok(None) => {
            state.chain_running = Some(running);
        }
        Err(err) => {
            state.chain_started_at = None;
            state.last_chain_probe_at = None;
            state.chain_runtime_status =
                ChainRuntimeStatus::Unreachable(format!("query chain runtime failed: {err}"));
            state.append_log(format!("query chain runtime status failed: {err}"));
            state.mark_updated();
        }
    }
}

pub(super) fn update_chain_runtime_status(state: &mut ServiceState) {
    if !state.config.chain_enabled {
        state.chain_runtime_status = ChainRuntimeStatus::Disabled;
        state.last_chain_probe_at = None;
        return;
    }

    if state.chain_running.is_none() {
        if !matches!(
            state.chain_runtime_status,
            ChainRuntimeStatus::ConfigError(_) | ChainRuntimeStatus::Unreachable(_)
        ) {
            state.chain_runtime_status = ChainRuntimeStatus::NotStarted;
        }
        state.last_chain_probe_at = None;
        return;
    }

    let now = Instant::now();
    let should_probe = state.last_chain_probe_at.is_none_or(|last| {
        now.duration_since(last) >= Duration::from_millis(CHAIN_STATUS_PROBE_INTERVAL_MS)
    });
    if !should_probe {
        return;
    }

    state.last_chain_probe_at = Some(now);
    match probe_chain_status_endpoint(state.config.chain_status_bind.as_str()) {
        Ok(()) => {
            state.chain_runtime_status = ChainRuntimeStatus::Ready;
        }
        Err(err) => {
            let within_grace = state.chain_started_at.is_some_and(|started_at| {
                now.duration_since(started_at)
                    < Duration::from_secs(CHAIN_STATUS_STARTING_GRACE_SECS)
            });
            if within_grace {
                state.chain_runtime_status = ChainRuntimeStatus::Starting;
            } else if err.contains("chain status bind") {
                state.chain_runtime_status = ChainRuntimeStatus::ConfigError(err);
            } else {
                state.chain_runtime_status = ChainRuntimeStatus::Unreachable(err);
            }
        }
    }
}

fn probe_chain_status_endpoint(bind: &str) -> Result<(), String> {
    let (host, port) = parse_host_port(bind, "chain status bind")?;
    let host = runtime_paths::normalize_bind_host_for_local_access(host.as_str());
    let socket_addr = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|err| format!("resolve chain status server failed: {err}"))?
        .next()
        .ok_or_else(|| "resolve chain status server failed: no socket address".to_string())?;

    let mut stream = TcpStream::connect_timeout(
        &socket_addr,
        Duration::from_millis(CHAIN_STATUS_PROBE_TIMEOUT_MS),
    )
    .map_err(|err| format!("connect chain status server failed: {err}"))?;
    let timeout = Some(Duration::from_millis(CHAIN_STATUS_PROBE_TIMEOUT_MS));
    let _ = stream.set_read_timeout(timeout);
    let _ = stream.set_write_timeout(timeout);

    let host_header = host_for_url(host.as_str());
    let request = format!(
        "GET /v1/chain/status HTTP/1.1\r\nHost: {host_header}:{port}\r\nConnection: close\r\n\r\n"
    );
    std::io::Write::write_all(&mut stream, request.as_bytes())
        .map_err(|err| format!("write chain status probe failed: {err}"))?;

    let mut buffer = [0_u8; 256];
    let bytes = std::io::Read::read(&mut stream, &mut buffer)
        .map_err(|err| format!("read chain status probe failed: {err}"))?;
    if bytes == 0 {
        return Err("chain status probe returned empty response".to_string());
    }
    let response = String::from_utf8_lossy(&buffer[..bytes]);
    let status_line = response.lines().next().unwrap_or_default();
    if !status_line.starts_with("HTTP/") {
        return Err("chain status probe received non-HTTP response".to_string());
    }
    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|token| token.parse::<u16>().ok())
        .ok_or_else(|| format!("invalid chain status probe status line: {status_line}"))?;
    if !(200..=299).contains(&status_code) {
        return Err(format!("chain status probe returned HTTP {status_code}"));
    }
    Ok(())
}

pub(super) fn start_process(
    state: &mut ServiceState,
    config: LauncherConfig,
) -> Result<(), String> {
    if state.running.is_some() {
        return Err("world_game_launcher is already running".to_string());
    }

    let issues = validate_game_config(&config);
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

    let launcher_bin = resolve_launcher_bin_from_config(&config, state.launcher_bin.as_str());
    if !Path::new(launcher_bin.as_str()).is_file() {
        let err = format!("launcher binary does not exist: {launcher_bin}");
        state.process_state = ProcessState::StartFailed(err.clone());
        state.append_log(format!("start failed: {err}"));
        state.mark_updated();
        return Err(err);
    }

    match spawn_child_process(launcher_bin.as_str(), args.as_slice(), "game") {
        Ok(process) => {
            let pid = process.child.id();
            state.running = Some(process);
            state.config = config;
            state.process_state = ProcessState::Running { pid };
            state.append_log(format!(
                "world_game_launcher started (pid={pid}, bin={launcher_bin})"
            ));
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

pub(super) fn start_chain_process(
    state: &mut ServiceState,
    config: LauncherConfig,
) -> Result<(), String> {
    if !config.chain_enabled {
        state.config = config;
        state.chain_runtime_status = ChainRuntimeStatus::Disabled;
        state.chain_running = None;
        state.chain_started_at = None;
        state.last_chain_probe_at = None;
        state.mark_updated();
        return Err("chain runtime is disabled".to_string());
    }

    if state.chain_running.is_some() {
        return Err("world_chain_runtime is already running".to_string());
    }

    let issues = validate_chain_config(&config);
    if !issues.is_empty() {
        let detail = issues.join("; ");
        state.chain_runtime_status = ChainRuntimeStatus::ConfigError(detail.clone());
        state.append_log(format!("chain config validation failed: {detail}"));
        state.mark_updated();
        return Err(detail);
    }

    let args = match build_chain_runtime_args(&config) {
        Ok(args) => args,
        Err(err) => {
            state.chain_runtime_status = ChainRuntimeStatus::ConfigError(err.clone());
            state.append_log(format!("invalid chain runtime args: {err}"));
            state.mark_updated();
            return Err(err);
        }
    };

    let chain_runtime_bin =
        resolve_chain_runtime_bin_from_config(&config, state.chain_runtime_bin.as_str());
    if !Path::new(chain_runtime_bin.as_str()).is_file() {
        let err = format!("chain runtime binary does not exist: {chain_runtime_bin}");
        state.chain_runtime_status = ChainRuntimeStatus::Unreachable(err.clone());
        state.append_log(format!("chain runtime start failed: {err}"));
        state.mark_updated();
        return Err(err);
    }

    match spawn_child_process(chain_runtime_bin.as_str(), args.as_slice(), "chain") {
        Ok(process) => {
            let pid = process.child.id();
            state.chain_running = Some(process);
            state.config = config;
            state.chain_started_at = Some(Instant::now());
            state.last_chain_probe_at = None;
            state.chain_runtime_status = ChainRuntimeStatus::Starting;
            state.append_log(format!(
                "world_chain_runtime started (pid={pid}, bin={chain_runtime_bin})"
            ));
            state.mark_updated();
            Ok(())
        }
        Err(err) => {
            state.chain_started_at = None;
            state.chain_runtime_status = ChainRuntimeStatus::Unreachable(err.clone());
            state.append_log(format!("chain runtime start failed: {err}"));
            state.mark_updated();
            Err(err)
        }
    }
}

pub(super) fn stop_process(state: &mut ServiceState) -> Result<(), String> {
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

pub(super) fn stop_chain_process(state: &mut ServiceState) -> Result<(), String> {
    let Some(mut running) = state.chain_running.take() else {
        state.chain_runtime_status = if state.config.chain_enabled {
            ChainRuntimeStatus::NotStarted
        } else {
            ChainRuntimeStatus::Disabled
        };
        state.chain_started_at = None;
        state.last_chain_probe_at = None;
        state.append_log("world_chain_runtime stop requested but process is not running");
        state.mark_updated();
        return Ok(());
    };

    match stop_child_process(&mut running.child) {
        Ok(()) => {
            state.chain_started_at = None;
            state.last_chain_probe_at = None;
            state.chain_runtime_status = if state.config.chain_enabled {
                ChainRuntimeStatus::NotStarted
            } else {
                ChainRuntimeStatus::Disabled
            };
            state.append_log("world_chain_runtime stopped");
            state.mark_updated();
            Ok(())
        }
        Err(err) => {
            state.chain_runtime_status = ChainRuntimeStatus::Unreachable(err.clone());
            state.append_log(format!("world_chain_runtime stop failed: {err}"));
            state.mark_updated();
            Err(err)
        }
    }
}

pub(super) fn snapshot_from_state(
    state: &ServiceState,
    request_host: Option<&str>,
) -> StateSnapshot {
    let game_url = build_game_url(&state.config, request_host);
    StateSnapshot {
        status: state.process_state.code().to_string(),
        detail: state.process_state.detail(),
        pid: state.process_state.pid(),
        running: matches!(state.process_state, ProcessState::Running { .. }),
        launcher_bin: resolve_launcher_bin_from_config(&state.config, state.launcher_bin.as_str()),
        chain_status: state.chain_runtime_status.code().to_string(),
        chain_detail: state.chain_runtime_status.detail(),
        chain_pid: state
            .chain_running
            .as_ref()
            .map(|process| process.child.id()),
        chain_running: state.chain_running.is_some(),
        chain_runtime_bin: resolve_chain_runtime_bin_from_config(
            &state.config,
            state.chain_runtime_bin.as_str(),
        ),
        game_url,
        config: state.config.clone(),
        logs: state.logs.iter().cloned().collect(),
        updated_at_unix_ms: state.updated_at_unix_ms,
    }
}

pub(super) fn build_game_url(config: &LauncherConfig, request_host: Option<&str>) -> String {
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

pub(super) fn host_for_url(host: &str) -> String {
    if host.contains(':') && !host.starts_with('[') && !host.ends_with(']') {
        format!("[{host}]")
    } else {
        host.to_string()
    }
}

pub(super) fn validate_game_config(config: &LauncherConfig) -> Vec<String> {
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

    issues
}

pub(super) fn validate_chain_config(config: &LauncherConfig) -> Vec<String> {
    let mut issues = Vec::new();
    if !config.chain_enabled {
        return issues;
    }

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

    issues
}

pub(super) fn build_launcher_args(config: &LauncherConfig) -> Result<Vec<String>, String> {
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
    args.push("--chain-disable".to_string());

    Ok(args)
}

pub(super) fn build_chain_runtime_args(config: &LauncherConfig) -> Result<Vec<String>, String> {
    parse_host_port(config.chain_status_bind.as_str(), "chain status bind")?;
    if config.chain_node_id.trim().is_empty() {
        return Err("chain node id cannot be empty".to_string());
    }
    let chain_role = parse_chain_role(config.chain_node_role.as_str())?;
    let chain_tick_ms = parse_port(config.chain_node_tick_ms.as_str(), "chain node tick ms")?;
    let validators = parse_chain_validators(config.chain_node_validators.as_str())?;

    let mut args = vec![
        "--node-id".to_string(),
        config.chain_node_id.trim().to_string(),
        "--world-id".to_string(),
        resolve_chain_world_id(config),
        "--status-bind".to_string(),
        config.chain_status_bind.trim().to_string(),
        "--node-role".to_string(),
        chain_role,
        "--node-tick-ms".to_string(),
        chain_tick_ms.to_string(),
    ];
    for validator in validators {
        args.push("--node-validator".to_string());
        args.push(validator);
    }
    Ok(args)
}

fn resolve_chain_world_id(config: &LauncherConfig) -> String {
    if config.chain_world_id.trim().is_empty() {
        let scenario = if config.scenario.trim().is_empty() {
            DEFAULT_SCENARIO
        } else {
            config.scenario.trim()
        };
        format!("live-{scenario}")
    } else {
        config.chain_world_id.trim().to_string()
    }
}

fn resolve_launcher_bin_from_config(config: &LauncherConfig, default_bin: &str) -> String {
    let value = config.launcher_bin.trim();
    if value.is_empty() {
        default_bin.to_string()
    } else {
        value.to_string()
    }
}

fn resolve_chain_runtime_bin_from_config(config: &LauncherConfig, default_bin: &str) -> String {
    let value = config.chain_runtime_bin.trim();
    if value.is_empty() {
        default_bin.to_string()
    } else {
        value.to_string()
    }
}

fn spawn_child_process(
    bin: &str,
    args: &[String],
    process_label: &'static str,
) -> Result<RunningProcess, String> {
    let mut child = Command::new(bin)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("spawn process `{bin}` failed: {err}"))?;

    let (log_tx, log_rx) = mpsc::channel::<String>();
    if let Some(stdout) = child.stdout.take() {
        spawn_log_reader(stdout, process_label, "stdout", log_tx.clone());
    }
    if let Some(stderr) = child.stderr.take() {
        spawn_log_reader(stderr, process_label, "stderr", log_tx.clone());
    }

    Ok(RunningProcess { child, log_rx })
}

fn spawn_log_reader<R: Read + Send + 'static>(
    reader: R,
    process_label: &'static str,
    source: &'static str,
    tx: Sender<String>,
) {
    thread::spawn(move || {
        let buffered = BufReader::new(reader);
        for line in buffered.lines() {
            match line {
                Ok(content) => {
                    let _ = tx.send(format!("[{process_label} {source}] {content}"));
                }
                Err(err) => {
                    let _ = tx.send(format!("[{process_label} {source}] <read error: {err}>"));
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
