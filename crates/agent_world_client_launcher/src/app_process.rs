use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;

use serde::de::DeserializeOwned;

use super::*;

const CONTROL_PLANE_BOOT_TIMEOUT_MS: u64 = 4_000;
const CONTROL_PLANE_BOOT_POLL_INTERVAL_MS: u64 = 80;
const CONTROL_PLANE_HTTP_TIMEOUT_MS: u64 = 1_500;

impl ClientLauncherApp {
    pub(super) fn current_game_url(&self) -> String {
        self.web_game_url
            .clone()
            .unwrap_or_else(|| build_game_url(&self.config))
    }

    pub(super) fn is_feedback_available(&self) -> bool {
        self.config.chain_enabled && matches!(self.chain_runtime_status, ChainRuntimeStatus::Ready)
    }

    pub(super) fn maybe_auto_start_chain(&mut self) {
        self.ensure_control_plane_service();
        if self.chain_auto_start_attempted {
            return;
        }
        if !self.config.chain_enabled {
            self.chain_runtime_status = ChainRuntimeStatus::Disabled;
            self.chain_auto_start_attempted = true;
            return;
        }
        if self.web_request_inflight {
            return;
        }
        self.chain_auto_start_attempted = true;
        self.start_chain_process();
    }

    pub(super) fn update_chain_runtime_status(&mut self) {}

    pub(super) fn poll_process(&mut self) {
        self.poll_control_plane_process();

        while let Ok(event) = self.web_api_rx.try_recv() {
            self.web_request_inflight = false;
            self.last_web_poll_at = Some(Instant::now());
            match event {
                WebApiEvent::State(result) => match result {
                    Ok(snapshot) => self.apply_web_snapshot(snapshot),
                    Err(err) => {
                        self.status = LauncherStatus::QueryFailed;
                        self.append_log(format!("web state refresh failed: {err}"));
                    }
                },
                WebApiEvent::Action(result) => match result {
                    Ok(response) => {
                        if !response.ok {
                            if let Some(error) = response.error {
                                self.append_log(format!("web action failed: {error}"));
                            } else {
                                self.append_log("web action failed".to_string());
                            }
                        }
                        self.apply_web_snapshot(response.state);
                    }
                    Err(err) => {
                        self.status = LauncherStatus::QueryFailed;
                        self.append_log(format!("web action request failed: {err}"));
                    }
                },
                WebApiEvent::Transfer(result) => {
                    self.apply_web_transfer_submit_result(result);
                }
            }
        }

        let now = Instant::now();
        let should_poll = self.last_web_poll_at.is_none_or(|last| {
            now.duration_since(last) >= Duration::from_millis(WEB_POLL_INTERVAL_MS)
        });
        if should_poll && !self.web_request_inflight {
            self.request_web_state();
        }
    }

    pub(super) fn poll_chain_process(&mut self) {}

    pub(super) fn stop_process(&mut self) {
        if self.web_request_inflight {
            self.append_log("skip stop: previous web request still in flight".to_string());
            return;
        }
        self.request_web_stop();
    }

    pub(super) fn start_process(&mut self) {
        if self.web_request_inflight {
            self.append_log("skip start: previous web request still in flight".to_string());
            return;
        }

        let config_issues = collect_required_config_issues(&self.config);
        if !config_issues.is_empty() {
            self.status = LauncherStatus::InvalidArgs;
            let message = self
                .tr(
                    "游戏启动前校验失败：请先修复必填配置项",
                    "game preflight validation failed: fix required configuration issues first",
                )
                .to_string();
            self.append_log(message);
            for issue in config_issues {
                self.append_log(format!("- {}", issue.text(self.ui_language)));
            }
            return;
        }

        self.request_web_start();
    }

    pub(super) fn stop_chain_process(&mut self) {
        if self.web_request_inflight {
            self.append_log("skip chain stop: previous web request still in flight".to_string());
            return;
        }
        self.request_web_chain_stop();
    }

    pub(super) fn start_chain_process(&mut self) {
        if !self.config.chain_enabled {
            self.chain_runtime_status = ChainRuntimeStatus::Disabled;
            self.append_log("chain runtime start skipped: chain runtime disabled");
            return;
        }

        if self.web_request_inflight {
            self.append_log("skip chain start: previous web request still in flight".to_string());
            return;
        }

        let config_issues = collect_chain_required_config_issues(&self.config);
        if !config_issues.is_empty() {
            let mut details = Vec::new();
            for issue in config_issues {
                let detail = issue.text(self.ui_language).to_string();
                details.push(detail.clone());
                self.append_log(format!("- {detail}"));
            }
            self.chain_runtime_status = ChainRuntimeStatus::ConfigError(details.join("; "));
            self.append_log("chain runtime preflight validation failed");
            return;
        }

        self.request_web_chain_start();
    }

    fn ensure_control_plane_service(&mut self) {
        if !self.control_manage_service {
            return;
        }
        if self.running.is_some() {
            return;
        }

        let web_launcher_bin = platform_ops::resolve_web_launcher_binary_path()
            .to_string_lossy()
            .to_string();
        if web_launcher_bin.trim().is_empty() {
            self.status = LauncherStatus::QueryFailed;
            self.append_log("control plane bootstrap failed: web launcher binary is empty");
            return;
        }

        let launcher_bin = if self.config.launcher_bin.trim().is_empty() {
            resolve_launcher_binary_path().to_string_lossy().to_string()
        } else {
            self.config.launcher_bin.trim().to_string()
        };
        let chain_runtime_bin = if self.config.chain_runtime_bin.trim().is_empty() {
            resolve_chain_runtime_binary_path()
                .to_string_lossy()
                .to_string()
        } else {
            self.config.chain_runtime_bin.trim().to_string()
        };

        let mut args = vec![
            "--listen-bind".to_string(),
            self.control_listen_bind.clone(),
            "--launcher-bin".to_string(),
            launcher_bin,
            "--chain-runtime-bin".to_string(),
            chain_runtime_bin,
        ];
        if let Ok(static_dir) = env::var("AGENT_WORLD_WEB_LAUNCHER_STATIC_DIR") {
            let static_dir = static_dir.trim();
            if !static_dir.is_empty() {
                args.push("--console-static-dir".to_string());
                args.push(static_dir.to_string());
            }
        }

        match spawn_child_process(web_launcher_bin.as_str(), args.as_slice(), "control") {
            Ok(process) => {
                self.append_log(format!(
                    "control plane started (bind={}, bin={web_launcher_bin})",
                    self.control_listen_bind
                ));
                self.running = Some(process);
                self.wait_control_plane_ready();
            }
            Err(err) => {
                self.status = LauncherStatus::QueryFailed;
                self.append_log(format!("control plane start failed: {err}"));
            }
        }
    }

    fn wait_control_plane_ready(&mut self) {
        let deadline = Instant::now() + Duration::from_millis(CONTROL_PLANE_BOOT_TIMEOUT_MS);
        while Instant::now() < deadline {
            self.poll_control_plane_process();
            if self.running.is_none() {
                return;
            }
            if check_web_health(self.control_api_base.as_str()).is_ok() {
                self.append_log(format!("control plane ready at {}", self.control_api_base));
                return;
            }
            thread::sleep(Duration::from_millis(CONTROL_PLANE_BOOT_POLL_INTERVAL_MS));
        }
        self.append_log(format!(
            "control plane health check timeout at {}",
            self.control_api_base
        ));
    }

    fn poll_control_plane_process(&mut self) {
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
                self.status = LauncherStatus::QueryFailed;
                self.append_log(format!("control plane exited: {status}"));
                self.running = None;
            }
            Ok(None) => {
                self.running = Some(running);
            }
            Err(err) => {
                self.status = LauncherStatus::QueryFailed;
                self.append_log(format!("query control plane status failed: {err}"));
                self.running = None;
            }
        }
    }

    fn request_web_state(&mut self) {
        self.ensure_control_plane_service();
        self.web_request_inflight = true;
        self.last_web_poll_at = Some(Instant::now());
        let tx = self.web_api_tx.clone();
        let base_url = self.control_api_base.clone();
        thread::spawn(move || {
            let _ = tx.send(WebApiEvent::State(fetch_web_state_blocking(
                base_url.as_str(),
            )));
        });
    }

    fn request_web_start(&mut self) {
        self.ensure_control_plane_service();
        self.web_request_inflight = true;
        self.last_web_poll_at = Some(Instant::now());
        let tx = self.web_api_tx.clone();
        let base_url = self.control_api_base.clone();
        let config = self.config.clone();
        thread::spawn(move || {
            let _ = tx.send(WebApiEvent::Action(post_web_start_blocking(
                base_url.as_str(),
                config,
            )));
        });
    }

    fn request_web_stop(&mut self) {
        self.ensure_control_plane_service();
        self.web_request_inflight = true;
        self.last_web_poll_at = Some(Instant::now());
        let tx = self.web_api_tx.clone();
        let base_url = self.control_api_base.clone();
        thread::spawn(move || {
            let _ = tx.send(WebApiEvent::Action(post_web_stop_blocking(
                base_url.as_str(),
            )));
        });
    }

    fn request_web_chain_start(&mut self) {
        self.ensure_control_plane_service();
        self.web_request_inflight = true;
        self.last_web_poll_at = Some(Instant::now());
        let tx = self.web_api_tx.clone();
        let base_url = self.control_api_base.clone();
        let config = self.config.clone();
        thread::spawn(move || {
            let _ = tx.send(WebApiEvent::Action(post_web_chain_start_blocking(
                base_url.as_str(),
                config,
            )));
        });
    }

    fn request_web_chain_stop(&mut self) {
        self.ensure_control_plane_service();
        self.web_request_inflight = true;
        self.last_web_poll_at = Some(Instant::now());
        let tx = self.web_api_tx.clone();
        let base_url = self.control_api_base.clone();
        thread::spawn(move || {
            let _ = tx.send(WebApiEvent::Action(post_web_chain_stop_blocking(
                base_url.as_str(),
            )));
        });
    }

    fn apply_web_snapshot(&mut self, snapshot: WebStateSnapshot) {
        self.status =
            launcher_status_from_web(snapshot.status.as_str(), snapshot.detail.as_deref());
        self.chain_runtime_status = chain_runtime_status_from_web(
            snapshot.chain_status.as_str(),
            snapshot.chain_detail.as_deref(),
        );
        self.web_game_url = Some(snapshot.game_url);
        self.config = snapshot.config;
        self.logs = snapshot.logs.into_iter().collect();
        while self.logs.len() > MAX_LOG_LINES {
            self.logs.pop_front();
        }

        if matches!(
            self.chain_runtime_status,
            ChainRuntimeStatus::Starting | ChainRuntimeStatus::Ready
        ) {
            self.chain_auto_start_attempted = true;
        }
    }
}

fn fetch_web_state_blocking(base_url: &str) -> Result<WebStateSnapshot, String> {
    http_json_request(base_url, "GET", "/api/state", None)
}

fn post_web_start_blocking(base_url: &str, config: LaunchConfig) -> Result<WebApiResponse, String> {
    let payload = serde_json::to_vec(&config)
        .map_err(|err| format!("serialize /api/start payload failed: {err}"))?;
    http_json_request(base_url, "POST", "/api/start", Some(payload.as_slice()))
}

fn post_web_stop_blocking(base_url: &str) -> Result<WebApiResponse, String> {
    http_json_request(base_url, "POST", "/api/stop", None)
}

fn post_web_chain_start_blocking(
    base_url: &str,
    config: LaunchConfig,
) -> Result<WebApiResponse, String> {
    let payload = serde_json::to_vec(&config)
        .map_err(|err| format!("serialize /api/chain/start payload failed: {err}"))?;
    http_json_request(
        base_url,
        "POST",
        "/api/chain/start",
        Some(payload.as_slice()),
    )
}

fn post_web_chain_stop_blocking(base_url: &str) -> Result<WebApiResponse, String> {
    http_json_request(base_url, "POST", "/api/chain/stop", None)
}

fn check_web_health(base_url: &str) -> Result<(), String> {
    let (status_code, _body) = http_request(base_url, "GET", "/healthz", None)?;
    if (200..=299).contains(&status_code) {
        Ok(())
    } else {
        Err(format!("GET /healthz failed with HTTP {status_code}"))
    }
}

fn http_json_request<T: DeserializeOwned>(
    base_url: &str,
    method: &str,
    path: &str,
    body: Option<&[u8]>,
) -> Result<T, String> {
    let (status_code, response_body) = http_request(base_url, method, path, body)?;
    if !(200..=299).contains(&status_code) {
        let body_text = String::from_utf8_lossy(response_body.as_slice());
        return Err(format!(
            "{method} {path} failed with HTTP {status_code}: {body_text}"
        ));
    }
    serde_json::from_slice(response_body.as_slice())
        .map_err(|err| format!("decode {method} {path} response failed: {err}"))
}

fn http_request(
    base_url: &str,
    method: &str,
    path: &str,
    body: Option<&[u8]>,
) -> Result<(u16, Vec<u8>), String> {
    let (host, port) = parse_http_base_url(base_url)?;
    let connect_host = normalize_host_for_connect(host.as_str());
    let socket_host = host_for_url(connect_host.as_str());
    let mut stream = TcpStream::connect(format!("{socket_host}:{port}"))
        .map_err(|err| format!("connect control plane failed: {err}"))?;

    let timeout = Some(Duration::from_millis(CONTROL_PLANE_HTTP_TIMEOUT_MS));
    let _ = stream.set_read_timeout(timeout);
    let _ = stream.set_write_timeout(timeout);

    let payload = body.unwrap_or(&[]);
    let host_header = host_for_url(host.as_str());
    let mut request_head = String::new();
    request_head.push_str(&format!("{method} {path} HTTP/1.1\r\n"));
    request_head.push_str(&format!("Host: {host_header}:{port}\r\n"));
    request_head.push_str("Connection: close\r\n");
    if !payload.is_empty() {
        request_head.push_str("Content-Type: application/json\r\n");
        request_head.push_str(&format!("Content-Length: {}\r\n", payload.len()));
    }
    request_head.push_str("\r\n");

    stream
        .write_all(request_head.as_bytes())
        .map_err(|err| format!("write request header failed: {err}"))?;
    if !payload.is_empty() {
        stream
            .write_all(payload)
            .map_err(|err| format!("write request body failed: {err}"))?;
    }
    stream
        .flush()
        .map_err(|err| format!("flush request failed: {err}"))?;

    let mut response_bytes = Vec::new();
    stream
        .read_to_end(&mut response_bytes)
        .map_err(|err| format!("read response failed: {err}"))?;
    parse_http_response(response_bytes.as_slice())
}

fn parse_http_response(bytes: &[u8]) -> Result<(u16, Vec<u8>), String> {
    let Some(boundary) = bytes.windows(4).position(|window| window == b"\r\n\r\n") else {
        return Err("invalid HTTP response: missing header terminator".to_string());
    };
    let header = std::str::from_utf8(&bytes[..boundary])
        .map_err(|_| "invalid HTTP response: header is not UTF-8".to_string())?;
    let body = bytes[(boundary + 4)..].to_vec();

    let Some(status_line) = header.lines().next() else {
        return Err("invalid HTTP response: missing status line".to_string());
    };
    let Some(status_code) = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|token| token.parse::<u16>().ok())
    else {
        return Err(format!("invalid HTTP response status line: {status_line}"));
    };

    Ok((status_code, body))
}

fn parse_http_base_url(base_url: &str) -> Result<(String, u16), String> {
    let mut raw = base_url.trim();
    if let Some(stripped) = raw.strip_prefix("http://") {
        raw = stripped;
    }
    raw = raw.trim_end_matches('/');
    let authority = raw
        .split('/')
        .next()
        .ok_or_else(|| format!("invalid control plane URL: {base_url}"))?
        .trim();
    if authority.is_empty() {
        return Err(format!("invalid control plane URL: {base_url}"));
    }

    if authority.starts_with('[') || authority.contains(':') {
        parse_host_port(authority, CLIENT_LAUNCHER_CONTROL_URL_ENV)
    } else {
        Ok((authority.to_string(), 80))
    }
}
