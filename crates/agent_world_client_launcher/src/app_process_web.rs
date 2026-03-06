use super::*;

impl ClientLauncherApp {
    pub(super) fn current_game_url(&self) -> String {
        self.web_game_url
            .clone()
            .unwrap_or_else(|| build_game_url(&self.config))
    }

    pub(super) fn is_feedback_available(&self) -> bool {
        matches!(self.chain_runtime_status, ChainRuntimeStatus::Ready)
    }

    pub(super) fn maybe_auto_start_chain(&mut self) {
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
                WebApiEvent::Feedback(result) => self.apply_web_feedback_submit_result(result),
                WebApiEvent::Transfer(result) => self.apply_web_transfer_submit_result(result),
                WebApiEvent::TransferQuery(result) => self.apply_web_transfer_query_result(result),
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
        if self.web_request_inflight {
            self.append_log("skip chain stop: previous web request still in flight".to_string());
            return;
        }
        self.request_web_chain_stop();
    }

    pub(super) fn start_chain_process(&mut self) {
        if self.web_request_inflight {
            self.append_log("skip chain start: previous web request still in flight".to_string());
            return;
        }
        self.request_web_chain_start();
    }

    pub(super) fn request_web_chain_transfer(&mut self, request: WebTransferSubmitRequest) {
        if self.web_request_inflight {
            self.append_log("skip transfer submit: previous web request still in flight");
            return;
        }
        self.web_request_inflight = true;
        self.last_web_poll_at = Some(Instant::now());
        let tx = self.web_api_tx.clone();
        spawn_local(async move {
            let _ = tx.send(WebApiEvent::Transfer(
                post_web_chain_transfer(request).await,
            ));
        });
    }

    pub(super) fn request_web_chain_transfer_accounts(&mut self) {
        if self.web_request_inflight {
            self.append_log("skip transfer accounts query: previous web request still in flight");
            return;
        }
        self.web_request_inflight = true;
        self.last_web_poll_at = Some(Instant::now());
        let tx = self.web_api_tx.clone();
        spawn_local(async move {
            let _ = tx.send(WebApiEvent::TransferQuery(
                fetch_web_transfer_accounts()
                    .await
                    .map(transfer_window::TransferQueryResponse::Accounts),
            ));
        });
    }

    pub(super) fn request_web_chain_transfer_history(
        &mut self,
        account_filter: String,
        action_filter: String,
    ) {
        if self.web_request_inflight {
            self.append_log("skip transfer history query: previous web request still in flight");
            return;
        }
        self.web_request_inflight = true;
        self.last_web_poll_at = Some(Instant::now());
        let tx = self.web_api_tx.clone();
        spawn_local(async move {
            let _ = tx.send(WebApiEvent::TransferQuery(
                fetch_web_transfer_history(account_filter, action_filter)
                    .await
                    .map(transfer_window::TransferQueryResponse::History),
            ));
        });
    }

    pub(super) fn request_web_chain_transfer_status(&mut self, action_id: u64) {
        if self.web_request_inflight {
            self.append_log("skip transfer status query: previous web request still in flight");
            return;
        }
        self.web_request_inflight = true;
        self.last_web_poll_at = Some(Instant::now());
        let tx = self.web_api_tx.clone();
        spawn_local(async move {
            let _ = tx.send(WebApiEvent::TransferQuery(
                fetch_web_transfer_status(action_id)
                    .await
                    .map(transfer_window::TransferQueryResponse::Status),
            ));
        });
    }

    pub(super) fn request_web_chain_feedback(&mut self, request: WebFeedbackSubmitRequest) {
        if self.web_request_inflight {
            self.append_log("skip feedback submit: previous web request still in flight");
            return;
        }
        self.web_request_inflight = true;
        self.last_web_poll_at = Some(Instant::now());
        let tx = self.web_api_tx.clone();
        spawn_local(async move {
            let _ = tx.send(WebApiEvent::Feedback(
                post_web_chain_feedback(request).await,
            ));
        });
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

    fn request_web_chain_start(&mut self) {
        self.web_request_inflight = true;
        self.last_web_poll_at = Some(Instant::now());
        let tx = self.web_api_tx.clone();
        let config = self.config.clone();
        spawn_local(async move {
            let _ = tx.send(WebApiEvent::Action(post_web_chain_start(config).await));
        });
    }

    fn request_web_chain_stop(&mut self) {
        self.web_request_inflight = true;
        self.last_web_poll_at = Some(Instant::now());
        let tx = self.web_api_tx.clone();
        spawn_local(async move {
            let _ = tx.send(WebApiEvent::Action(post_web_chain_stop().await));
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

async fn post_web_chain_start(config: LaunchConfig) -> Result<WebApiResponse, String> {
    let payload = serde_json::to_string(&config)
        .map_err(|err| format!("serialize /api/chain/start payload failed: {err}"))?;
    let request = Request::post("/api/chain/start")
        .header("content-type", "application/json")
        .body(payload)
        .map_err(|err| format!("build /api/chain/start request failed: {err}"))?;
    let response = request
        .send()
        .await
        .map_err(|err| format!("POST /api/chain/start failed: {err}"))?;
    if !response.ok() {
        return Err(format!(
            "POST /api/chain/start failed with HTTP {}",
            response.status()
        ));
    }
    response
        .json::<WebApiResponse>()
        .await
        .map_err(|err| format!("decode /api/chain/start response failed: {err}"))
}

async fn post_web_chain_stop() -> Result<WebApiResponse, String> {
    let response = Request::post("/api/chain/stop")
        .send()
        .await
        .map_err(|err| format!("POST /api/chain/stop failed: {err}"))?;
    if !response.ok() {
        return Err(format!(
            "POST /api/chain/stop failed with HTTP {}",
            response.status()
        ));
    }
    response
        .json::<WebApiResponse>()
        .await
        .map_err(|err| format!("decode /api/chain/stop response failed: {err}"))
}

async fn post_web_chain_transfer(
    request_payload: WebTransferSubmitRequest,
) -> Result<WebTransferSubmitResponse, String> {
    let payload = serde_json::to_string(&request_payload)
        .map_err(|err| format!("serialize /api/chain/transfer payload failed: {err}"))?;
    let request = Request::post("/api/chain/transfer")
        .header("content-type", "application/json")
        .body(payload)
        .map_err(|err| format!("build /api/chain/transfer request failed: {err}"))?;
    let response = request
        .send()
        .await
        .map_err(|err| format!("POST /api/chain/transfer failed: {err}"))?;
    if !response.ok() {
        return Err(format!(
            "POST /api/chain/transfer failed with HTTP {}",
            response.status()
        ));
    }
    response
        .json::<WebTransferSubmitResponse>()
        .await
        .map_err(|err| format!("decode /api/chain/transfer response failed: {err}"))
}

async fn fetch_web_transfer_accounts(
) -> Result<transfer_window::WebTransferAccountsResponse, String> {
    let response = Request::get("/api/chain/transfer/accounts")
        .send()
        .await
        .map_err(|err| format!("GET /api/chain/transfer/accounts failed: {err}"))?;
    if !response.ok() {
        return Err(format!(
            "GET /api/chain/transfer/accounts failed with HTTP {}",
            response.status()
        ));
    }
    response
        .json::<transfer_window::WebTransferAccountsResponse>()
        .await
        .map_err(|err| format!("decode /api/chain/transfer/accounts response failed: {err}"))
}

async fn fetch_web_transfer_status(
    action_id: u64,
) -> Result<transfer_window::WebTransferStatusResponse, String> {
    let path = format!("/api/chain/transfer/status?action_id={action_id}");
    let response = Request::get(path.as_str())
        .send()
        .await
        .map_err(|err| format!("GET /api/chain/transfer/status failed: {err}"))?;
    if !response.ok() {
        return Err(format!(
            "GET /api/chain/transfer/status failed with HTTP {}",
            response.status()
        ));
    }
    response
        .json::<transfer_window::WebTransferStatusResponse>()
        .await
        .map_err(|err| format!("decode /api/chain/transfer/status response failed: {err}"))
}

async fn fetch_web_transfer_history(
    account_filter: String,
    action_filter: String,
) -> Result<transfer_window::WebTransferHistoryResponse, String> {
    let mut query = vec!["limit=50".to_string()];
    let account_filter = account_filter.trim();
    if !account_filter.is_empty() {
        query.push(format!("account_id={account_filter}"));
    }
    let action_filter = action_filter.trim();
    if !action_filter.is_empty() {
        query.push(format!("action_id={action_filter}"));
    }
    let path = format!("/api/chain/transfer/history?{}", query.join("&"));
    let response = Request::get(path.as_str())
        .send()
        .await
        .map_err(|err| format!("GET /api/chain/transfer/history failed: {err}"))?;
    if !response.ok() {
        return Err(format!(
            "GET /api/chain/transfer/history failed with HTTP {}",
            response.status()
        ));
    }
    response
        .json::<transfer_window::WebTransferHistoryResponse>()
        .await
        .map_err(|err| format!("decode /api/chain/transfer/history response failed: {err}"))
}

async fn post_web_chain_feedback(
    request_payload: WebFeedbackSubmitRequest,
) -> Result<WebFeedbackSubmitResponse, String> {
    let payload = serde_json::to_string(&request_payload)
        .map_err(|err| format!("serialize /api/chain/feedback payload failed: {err}"))?;
    let request = Request::post("/api/chain/feedback")
        .header("content-type", "application/json")
        .body(payload)
        .map_err(|err| format!("build /api/chain/feedback request failed: {err}"))?;
    let response = request
        .send()
        .await
        .map_err(|err| format!("POST /api/chain/feedback failed: {err}"))?;
    if !response.ok() {
        return Err(format!(
            "POST /api/chain/feedback failed with HTTP {}",
            response.status()
        ));
    }
    response
        .json::<WebFeedbackSubmitResponse>()
        .await
        .map_err(|err| format!("decode /api/chain/feedback response failed: {err}"))
}
