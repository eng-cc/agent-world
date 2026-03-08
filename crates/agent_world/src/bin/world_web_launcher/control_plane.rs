use super::*;

use std::io::{BufRead, BufReader, Read};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Sender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

const CHAIN_TRANSFER_SUBMIT_PATH: &str = "/v1/chain/transfer/submit";
const CHAIN_FEEDBACK_SUBMIT_PATH: &str = "/v1/chain/feedback/submit";
const CHAIN_TRANSFER_PROXY_TIMEOUT_MS: u64 = 1_500;

pub(super) fn parse_config_request(body: &[u8], action: &str) -> Result<LauncherConfig, String> {
    serde_json::from_slice(body).map_err(|err| format!("parse {action} request JSON failed: {err}"))
}

pub(super) fn parse_chain_transfer_request(
    body: &[u8],
) -> Result<ChainTransferSubmitRequest, String> {
    serde_json::from_slice(body)
        .map_err(|err| format!("parse chain transfer request JSON failed: {err}"))
}

pub(super) fn parse_chain_feedback_request(
    body: &[u8],
) -> Result<ChainFeedbackSubmitRequest, String> {
    serde_json::from_slice(body)
        .map_err(|err| format!("parse chain feedback request JSON failed: {err}"))
}

pub(super) fn submit_chain_transfer(
    state: &mut ServiceState,
    request: &ChainTransferSubmitRequest,
) -> ChainTransferSubmitResponse {
    if !state.config.chain_enabled {
        let response =
            ChainTransferSubmitResponse::error("chain_disabled", "chain runtime is disabled");
        state.append_log("chain transfer submit rejected: chain runtime is disabled");
        state.mark_updated();
        return response;
    }

    let chain_status_bind = state.config.chain_status_bind.clone();
    match submit_chain_transfer_remote(chain_status_bind.as_str(), request) {
        Ok(response) => {
            if response.ok {
                let action_id = response
                    .action_id
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "n/a".to_string());
                state.append_log(format!(
                    "chain transfer submitted via control plane (action_id={action_id})"
                ));
            } else {
                let error_code = response
                    .error_code
                    .as_deref()
                    .map(|code| format!(" ({code})"))
                    .unwrap_or_default();
                let error_text = response.error.as_deref().unwrap_or("unknown error");
                state.append_log(format!(
                    "chain transfer rejected by runtime{error_code}: {error_text}"
                ));
            }
            state.mark_updated();
            response
        }
        Err(err) => {
            let response = ChainTransferSubmitResponse::error("proxy_error", err.clone());
            state.append_log(format!("chain transfer proxy failed: {err}"));
            state.mark_updated();
            response
        }
    }
}

pub(super) fn submit_chain_feedback(
    state: &mut ServiceState,
    request: &ChainFeedbackSubmitRequest,
) -> ChainFeedbackSubmitResponse {
    if !state.config.chain_enabled {
        let response = ChainFeedbackSubmitResponse::error("chain runtime is disabled");
        state.append_log("chain feedback submit rejected: chain runtime is disabled");
        state.mark_updated();
        return response;
    }

    let chain_status_bind = state.config.chain_status_bind.clone();
    match submit_chain_feedback_remote(chain_status_bind.as_str(), request) {
        Ok(response) => {
            if response.ok {
                let feedback_id = response.feedback_id.as_deref().unwrap_or("n/a");
                let event_id = response.event_id.as_deref().unwrap_or("n/a");
                state.append_log(format!(
                    "chain feedback submitted via control plane (feedback_id={feedback_id}, event_id={event_id})"
                ));
            } else {
                let error_text = response.error.as_deref().unwrap_or("unknown error");
                state.append_log(format!("chain feedback rejected by runtime: {error_text}"));
            }
            state.mark_updated();
            response
        }
        Err(err) => {
            let response = ChainFeedbackSubmitResponse::error(err.clone());
            state.append_log(format!("chain feedback proxy failed: {err}"));
            state.mark_updated();
            response
        }
    }
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

fn submit_chain_transfer_remote(
    chain_status_bind: &str,
    request: &ChainTransferSubmitRequest,
) -> Result<ChainTransferSubmitResponse, String> {
    let (host, port) = parse_host_port(chain_status_bind, "chain status bind")?;
    let host = runtime_paths::normalize_bind_host_for_local_access(host.as_str());
    let socket_addr = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|err| format!("resolve chain status server failed: {err}"))?
        .next()
        .ok_or_else(|| "resolve chain status server failed: no socket address".to_string())?;

    let mut stream = TcpStream::connect_timeout(
        &socket_addr,
        Duration::from_millis(CHAIN_TRANSFER_PROXY_TIMEOUT_MS),
    )
    .map_err(|err| format!("connect chain status server failed: {err}"))?;
    let timeout = Some(Duration::from_millis(CHAIN_TRANSFER_PROXY_TIMEOUT_MS));
    let _ = stream.set_read_timeout(timeout);
    let _ = stream.set_write_timeout(timeout);

    let payload = serde_json::to_vec(request)
        .map_err(|err| format!("serialize chain transfer request failed: {err}"))?;
    let host_header = host_for_url(host.as_str());
    let request_head = format!(
        "POST {CHAIN_TRANSFER_SUBMIT_PATH} HTTP/1.1\r\nHost: {host_header}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        payload.len()
    );
    std::io::Write::write_all(&mut stream, request_head.as_bytes())
        .map_err(|err| format!("write chain transfer request header failed: {err}"))?;
    std::io::Write::write_all(&mut stream, payload.as_slice())
        .map_err(|err| format!("write chain transfer request body failed: {err}"))?;
    std::io::Write::flush(&mut stream)
        .map_err(|err| format!("flush chain transfer request failed: {err}"))?;

    let mut response_bytes = Vec::new();
    std::io::Read::read_to_end(&mut stream, &mut response_bytes)
        .map_err(|err| format!("read chain transfer response failed: {err}"))?;
    let (status_code, response) = parse_chain_transfer_submit_response(response_bytes.as_slice())?;

    if !(200..=299).contains(&status_code) && response.ok {
        return Err(format!(
            "chain transfer submit returned HTTP {status_code} with invalid success payload"
        ));
    }
    Ok(response)
}

fn parse_chain_transfer_submit_response(
    response_bytes: &[u8],
) -> Result<(u16, ChainTransferSubmitResponse), String> {
    let Some(boundary) = response_bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
    else {
        return Err("invalid HTTP response: missing header terminator".to_string());
    };
    let header = std::str::from_utf8(&response_bytes[..boundary])
        .map_err(|_| "invalid HTTP response: header is not UTF-8".to_string())?;
    let body = &response_bytes[(boundary + 4)..];

    let status_code = header
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|token| token.parse::<u16>().ok())
        .ok_or_else(|| "invalid HTTP response: missing status code".to_string())?;

    let response: ChainTransferSubmitResponse = serde_json::from_slice(body)
        .map_err(|err| format!("parse chain transfer response JSON failed: {err}"))?;
    Ok((status_code, response))
}

fn submit_chain_feedback_remote(
    chain_status_bind: &str,
    request: &ChainFeedbackSubmitRequest,
) -> Result<ChainFeedbackSubmitResponse, String> {
    let (host, port) = parse_host_port(chain_status_bind, "chain status bind")?;
    let host = runtime_paths::normalize_bind_host_for_local_access(host.as_str());
    let socket_addr = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|err| format!("resolve chain status server failed: {err}"))?
        .next()
        .ok_or_else(|| "resolve chain status server failed: no socket address".to_string())?;

    let mut stream = TcpStream::connect_timeout(
        &socket_addr,
        Duration::from_millis(CHAIN_TRANSFER_PROXY_TIMEOUT_MS),
    )
    .map_err(|err| format!("connect chain status server failed: {err}"))?;
    let timeout = Some(Duration::from_millis(CHAIN_TRANSFER_PROXY_TIMEOUT_MS));
    let _ = stream.set_read_timeout(timeout);
    let _ = stream.set_write_timeout(timeout);

    let payload = serde_json::to_vec(request)
        .map_err(|err| format!("serialize chain feedback request failed: {err}"))?;
    let host_header = host_for_url(host.as_str());
    let request_head = format!(
        "POST {CHAIN_FEEDBACK_SUBMIT_PATH} HTTP/1.1\r\nHost: {host_header}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        payload.len()
    );
    std::io::Write::write_all(&mut stream, request_head.as_bytes())
        .map_err(|err| format!("write chain feedback request header failed: {err}"))?;
    std::io::Write::write_all(&mut stream, payload.as_slice())
        .map_err(|err| format!("write chain feedback request body failed: {err}"))?;
    std::io::Write::flush(&mut stream)
        .map_err(|err| format!("flush chain feedback request failed: {err}"))?;

    let mut response_bytes = Vec::new();
    std::io::Read::read_to_end(&mut stream, &mut response_bytes)
        .map_err(|err| format!("read chain feedback response failed: {err}"))?;
    let (status_code, response) = parse_chain_feedback_submit_response(response_bytes.as_slice())?;

    if !(200..=299).contains(&status_code) && response.ok {
        return Err(format!(
            "chain feedback submit returned HTTP {status_code} with invalid success payload"
        ));
    }
    Ok(response)
}

fn parse_chain_feedback_submit_response(
    response_bytes: &[u8],
) -> Result<(u16, ChainFeedbackSubmitResponse), String> {
    let Some(boundary) = response_bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
    else {
        return Err("invalid HTTP response: missing header terminator".to_string());
    };
    let header = std::str::from_utf8(&response_bytes[..boundary])
        .map_err(|_| "invalid HTTP response: header is not UTF-8".to_string())?;
    let body = &response_bytes[(boundary + 4)..];

    let status_code = header
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|token| token.parse::<u16>().ok())
        .ok_or_else(|| "invalid HTTP response: missing status code".to_string())?;

    let response: ChainFeedbackSubmitResponse = serde_json::from_slice(body)
        .map_err(|err| format!("parse chain feedback response JSON failed: {err}"))?;
    Ok((status_code, response))
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
        if matches!(state.process_state, ProcessState::Running { .. }) {
            state.process_state = ProcessState::Stopped;
        }
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
        if !matches!(
            state.chain_runtime_status,
            ChainRuntimeStatus::Unreachable(_) | ChainRuntimeStatus::ConfigError(_)
        ) {
            state.chain_runtime_status = if state.config.chain_enabled {
                ChainRuntimeStatus::NotStarted
            } else {
                ChainRuntimeStatus::Disabled
            };
        }
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
    if parse_positive_u64(
        config.chain_node_tick_ms.as_str(),
        "chain node poll interval ms",
    )
    .is_err()
    {
        issues.push("chain node poll interval ms must be a positive integer".to_string());
    }
    if parse_positive_u64(
        config.chain_pos_slot_duration_ms.as_str(),
        "chain pos slot duration ms",
    )
    .is_err()
    {
        issues.push("chain pos slot duration ms must be a positive integer".to_string());
    }
    let ticks_per_slot = parse_positive_u64(
        config.chain_pos_ticks_per_slot.as_str(),
        "chain pos ticks per slot",
    );
    if ticks_per_slot.is_err() {
        issues.push("chain pos ticks per slot must be a positive integer".to_string());
    }
    let proposal_tick_phase = parse_non_negative_u64(
        config.chain_pos_proposal_tick_phase.as_str(),
        "chain pos proposal tick phase",
    );
    if proposal_tick_phase.is_err() {
        issues.push("chain pos proposal tick phase must be a non-negative integer".to_string());
    }
    if let (Ok(ticks_per_slot), Ok(proposal_tick_phase)) = (ticks_per_slot, proposal_tick_phase) {
        if proposal_tick_phase >= ticks_per_slot {
            issues.push(
                "chain pos proposal tick phase must be less than chain pos ticks per slot"
                    .to_string(),
            );
        }
    }
    if parse_optional_i64(
        config.chain_pos_slot_clock_genesis_unix_ms.as_str(),
        "chain pos slot clock genesis unix ms",
    )
    .is_err()
    {
        issues.push("chain pos slot clock genesis unix ms must be an integer or empty".to_string());
    }
    if parse_non_negative_u64(
        config.chain_pos_max_past_slot_lag.as_str(),
        "chain pos max past slot lag",
    )
    .is_err()
    {
        issues.push("chain pos max past slot lag must be a non-negative integer".to_string());
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
    let chain_tick_ms = parse_positive_u64(
        config.chain_node_tick_ms.as_str(),
        "chain node poll interval ms",
    )?;
    let pos_slot_duration_ms = parse_positive_u64(
        config.chain_pos_slot_duration_ms.as_str(),
        "chain pos slot duration ms",
    )?;
    let pos_ticks_per_slot = parse_positive_u64(
        config.chain_pos_ticks_per_slot.as_str(),
        "chain pos ticks per slot",
    )?;
    let pos_proposal_tick_phase = parse_non_negative_u64(
        config.chain_pos_proposal_tick_phase.as_str(),
        "chain pos proposal tick phase",
    )?;
    if pos_proposal_tick_phase >= pos_ticks_per_slot {
        return Err(format!(
            "chain pos proposal tick phase={} must be less than chain pos ticks per slot={}",
            pos_proposal_tick_phase, pos_ticks_per_slot
        ));
    }
    let pos_slot_clock_genesis_unix_ms = parse_optional_i64(
        config.chain_pos_slot_clock_genesis_unix_ms.as_str(),
        "chain pos slot clock genesis unix ms",
    )?;
    let pos_max_past_slot_lag = parse_non_negative_u64(
        config.chain_pos_max_past_slot_lag.as_str(),
        "chain pos max past slot lag",
    )?;
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
        "--pos-slot-duration-ms".to_string(),
        pos_slot_duration_ms.to_string(),
        "--pos-ticks-per-slot".to_string(),
        pos_ticks_per_slot.to_string(),
        "--pos-proposal-tick-phase".to_string(),
        pos_proposal_tick_phase.to_string(),
        if config.chain_pos_adaptive_tick_scheduler_enabled {
            "--pos-adaptive-tick-scheduler".to_string()
        } else {
            "--pos-no-adaptive-tick-scheduler".to_string()
        },
        "--pos-max-past-slot-lag".to_string(),
        pos_max_past_slot_lag.to_string(),
    ];
    if let Some(genesis) = pos_slot_clock_genesis_unix_ms {
        args.push("--pos-slot-clock-genesis-unix-ms".to_string());
        args.push(genesis.to_string());
    }
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

#[cfg(test)]
mod tests {
    use super::{
        parse_chain_feedback_request, parse_chain_transfer_request, submit_chain_feedback_remote,
        submit_chain_transfer_remote, ChainFeedbackSubmitRequest, ChainTransferSubmitRequest,
    };
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::time::Duration;

    fn read_http_request(stream: &mut std::net::TcpStream) -> Vec<u8> {
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("set read timeout");
        let mut bytes = Vec::new();
        let mut buffer = [0_u8; 1024];

        loop {
            let read = stream.read(&mut buffer).expect("read request");
            if read == 0 {
                break;
            }
            bytes.extend_from_slice(&buffer[..read]);
            let Some(boundary) = bytes.windows(4).position(|window| window == b"\r\n\r\n") else {
                continue;
            };
            let header =
                std::str::from_utf8(&bytes[..boundary]).expect("request header should be UTF-8");
            let content_length = header
                .lines()
                .find_map(|line| line.strip_prefix("Content-Length:"))
                .and_then(|value| value.trim().parse::<usize>().ok())
                .unwrap_or(0);
            if bytes.len() >= boundary + 4 + content_length {
                break;
            }
        }

        bytes
    }

    fn write_http_json_response(stream: &mut std::net::TcpStream, status: &str, body: &str) {
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response should succeed");
    }

    #[test]
    fn parse_chain_transfer_request_rejects_invalid_json() {
        let err = parse_chain_transfer_request(br#"{"from_account_id":"player:alice"}"#)
            .expect_err("invalid payload should fail");
        assert!(err.contains("parse chain transfer request JSON failed"));
    }

    #[test]
    fn parse_chain_feedback_request_rejects_invalid_json() {
        let err = parse_chain_feedback_request(br#"{"category":"bug","title":"x"}"#)
            .expect_err("invalid payload should fail");
        assert!(err.contains("parse chain feedback request JSON failed"));
    }

    #[test]
    fn submit_chain_transfer_remote_posts_expected_payload_and_reads_success() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let bind = listener.local_addr().expect("local addr");

        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let request_bytes = read_http_request(&mut stream);
            let request_text = String::from_utf8_lossy(&request_bytes);
            assert!(request_text.starts_with("POST /v1/chain/transfer/submit HTTP/1.1"));
            assert!(request_text.contains("\"from_account_id\":\"player:alice\""));
            assert!(request_text.contains("\"to_account_id\":\"player:bob\""));
            assert!(request_text.contains("\"amount\":7"));
            assert!(request_text.contains("\"nonce\":2"));
            write_http_json_response(
                &mut stream,
                "200 OK",
                r#"{"ok":true,"action_id":11,"submitted_at_unix_ms":1700000000}"#,
            );
        });

        let request = ChainTransferSubmitRequest {
            from_account_id: "player:alice".to_string(),
            to_account_id: "player:bob".to_string(),
            amount: 7,
            nonce: 2,
        };
        let response =
            submit_chain_transfer_remote(format!("127.0.0.1:{}", bind.port()).as_str(), &request)
                .expect("submit should succeed");
        assert!(response.ok);
        assert_eq!(response.action_id, Some(11));
        server.join().expect("server thread should finish");
    }

    #[test]
    fn submit_chain_transfer_remote_returns_rejected_payload_for_http_400() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let bind = listener.local_addr().expect("local addr");

        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let _ = read_http_request(&mut stream);
            write_http_json_response(
                &mut stream,
                "400 Bad Request",
                r#"{"ok":false,"error_code":"invalid_request","error":"bad payload"}"#,
            );
        });

        let request = ChainTransferSubmitRequest {
            from_account_id: "player:alice".to_string(),
            to_account_id: "player:bob".to_string(),
            amount: 7,
            nonce: 2,
        };
        let response =
            submit_chain_transfer_remote(format!("127.0.0.1:{}", bind.port()).as_str(), &request)
                .expect("proxy should return rejected payload");
        assert!(!response.ok);
        assert_eq!(response.error_code.as_deref(), Some("invalid_request"));
        assert_eq!(response.error.as_deref(), Some("bad payload"));
        server.join().expect("server thread should finish");
    }

    #[test]
    fn submit_chain_feedback_remote_posts_expected_payload_and_reads_success() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let bind = listener.local_addr().expect("local addr");

        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let request_bytes = read_http_request(&mut stream);
            let request_text = String::from_utf8_lossy(&request_bytes);
            assert!(request_text.starts_with("POST /v1/chain/feedback/submit HTTP/1.1"));
            assert!(request_text.contains("\"category\":\"bug\""));
            assert!(request_text.contains("\"title\":\"web feedback\""));
            assert!(request_text.contains("\"description\":\"looks good\""));
            write_http_json_response(
                &mut stream,
                "200 OK",
                r#"{"ok":true,"feedback_id":"fb-1","event_id":"evt-1"}"#,
            );
        });

        let request = ChainFeedbackSubmitRequest {
            category: "bug".to_string(),
            title: "web feedback".to_string(),
            description: "looks good".to_string(),
            platform: "client_launcher_web".to_string(),
            game_version: "unknown".to_string(),
        };
        let response =
            submit_chain_feedback_remote(format!("127.0.0.1:{}", bind.port()).as_str(), &request)
                .expect("submit should succeed");
        assert!(response.ok);
        assert_eq!(response.feedback_id.as_deref(), Some("fb-1"));
        assert_eq!(response.event_id.as_deref(), Some("evt-1"));
        server.join().expect("server thread should finish");
    }

    #[test]
    fn submit_chain_feedback_remote_returns_rejected_payload_for_http_400() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
        let bind = listener.local_addr().expect("local addr");

        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let _ = read_http_request(&mut stream);
            write_http_json_response(
                &mut stream,
                "400 Bad Request",
                r#"{"ok":false,"error":"invalid category"}"#,
            );
        });

        let request = ChainFeedbackSubmitRequest {
            category: "bug".to_string(),
            title: "web feedback".to_string(),
            description: "looks good".to_string(),
            platform: "client_launcher_web".to_string(),
            game_version: "unknown".to_string(),
        };
        let response =
            submit_chain_feedback_remote(format!("127.0.0.1:{}", bind.port()).as_str(), &request)
                .expect("proxy should return rejected payload");
        assert!(!response.ok);
        assert_eq!(response.error.as_deref(), Some("invalid category"));
        server.join().expect("server thread should finish");
    }
}
