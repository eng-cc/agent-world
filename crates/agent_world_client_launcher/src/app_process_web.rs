use super::*;

impl ClientLauncherApp {
    pub(super) fn current_game_url(&self) -> String {
        self.web_game_url
            .clone()
            .unwrap_or_else(|| build_game_url(&self.config))
    }

    pub(super) fn is_feedback_available(&self) -> bool {
        false
    }

    pub(super) fn maybe_auto_start_chain(&mut self) {}

    pub(super) fn update_chain_runtime_status(&mut self) {}

    pub(super) fn poll_process(&mut self) {
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
        self.request_web_start();
    }

    pub(super) fn stop_chain_process(&mut self) {
        self.append_log(
            self.tr(
                "Web 模式暂不支持独立停止区块链进程",
                "Web mode does not support standalone blockchain stop yet",
            )
            .to_string(),
        );
    }

    pub(super) fn start_chain_process(&mut self) {
        self.append_log(
            self.tr(
                "Web 模式暂不支持独立启动区块链进程",
                "Web mode does not support standalone blockchain start yet",
            )
            .to_string(),
        );
    }

    fn request_web_state(&mut self) {
        self.web_request_inflight = true;
        self.last_web_poll_at = Some(Instant::now());
        let tx = self.web_api_tx.clone();
        spawn_local(async move {
            let _ = tx.send(WebApiEvent::State(fetch_web_state().await));
        });
    }

    fn request_web_start(&mut self) {
        self.web_request_inflight = true;
        self.last_web_poll_at = Some(Instant::now());
        let tx = self.web_api_tx.clone();
        let config = self.config.clone();
        spawn_local(async move {
            let _ = tx.send(WebApiEvent::Action(post_web_start(config).await));
        });
    }

    fn request_web_stop(&mut self) {
        self.web_request_inflight = true;
        self.last_web_poll_at = Some(Instant::now());
        let tx = self.web_api_tx.clone();
        spawn_local(async move {
            let _ = tx.send(WebApiEvent::Action(post_web_stop().await));
        });
    }

    fn apply_web_snapshot(&mut self, snapshot: WebStateSnapshot) {
        self.status =
            launcher_status_from_web(snapshot.status.as_str(), snapshot.detail.as_deref());
        self.web_game_url = Some(snapshot.game_url);
        self.config = snapshot.config;
        self.logs = snapshot.logs.into_iter().collect();
        while self.logs.len() > MAX_LOG_LINES {
            self.logs.pop_front();
        }

        self.chain_runtime_status = if !self.config.chain_enabled {
            ChainRuntimeStatus::Disabled
        } else if snapshot.running {
            ChainRuntimeStatus::Ready
        } else {
            ChainRuntimeStatus::NotStarted
        };
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

async fn fetch_web_state() -> Result<WebStateSnapshot, String> {
    let response = Request::get("/api/state")
        .send()
        .await
        .map_err(|err| format!("GET /api/state failed: {err}"))?;
    if !response.ok() {
        return Err(format!(
            "GET /api/state failed with HTTP {}",
            response.status()
        ));
    }
    response
        .json::<WebStateSnapshot>()
        .await
        .map_err(|err| format!("decode /api/state response failed: {err}"))
}

async fn post_web_start(config: LaunchConfig) -> Result<WebApiResponse, String> {
    let payload = serde_json::to_string(&config)
        .map_err(|err| format!("serialize /api/start payload failed: {err}"))?;
    let request = Request::post("/api/start")
        .header("content-type", "application/json")
        .body(payload)
        .map_err(|err| format!("build /api/start request failed: {err}"))?;
    let response = request
        .send()
        .await
        .map_err(|err| format!("POST /api/start failed: {err}"))?;
    if !response.ok() {
        return Err(format!(
            "POST /api/start failed with HTTP {}",
            response.status()
        ));
    }
    response
        .json::<WebApiResponse>()
        .await
        .map_err(|err| format!("decode /api/start response failed: {err}"))
}

async fn post_web_stop() -> Result<WebApiResponse, String> {
    let response = Request::post("/api/stop")
        .send()
        .await
        .map_err(|err| format!("POST /api/stop failed: {err}"))?;
    if !response.ok() {
        return Err(format!(
            "POST /api/stop failed with HTTP {}",
            response.status()
        ));
    }
    response
        .json::<WebApiResponse>()
        .await
        .map_err(|err| format!("decode /api/stop response failed: {err}"))
}
