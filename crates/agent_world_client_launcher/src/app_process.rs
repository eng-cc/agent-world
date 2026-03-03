use super::*;

impl ClientLauncherApp {
    pub(super) fn current_game_url(&self) -> String {
        build_game_url(&self.config)
    }

    pub(super) fn is_feedback_available(&self) -> bool {
        self.config.chain_enabled && matches!(self.chain_runtime_status, ChainRuntimeStatus::Ready)
    }

    pub(super) fn maybe_auto_start_chain(&mut self) {
        if self.chain_auto_start_attempted {
            return;
        }
        self.chain_auto_start_attempted = true;
        if !self.config.chain_enabled {
            self.chain_runtime_status = ChainRuntimeStatus::Disabled;
            return;
        }
        self.start_chain_process();
    }

    pub(super) fn update_chain_runtime_status(&mut self) {
        if !self.config.chain_enabled {
            self.chain_runtime_status = ChainRuntimeStatus::Disabled;
            self.last_chain_probe_at = None;
            return;
        }

        if self.chain_running.is_none() {
            if !matches!(
                self.chain_runtime_status,
                ChainRuntimeStatus::ConfigError(_) | ChainRuntimeStatus::Unreachable(_)
            ) {
                self.chain_runtime_status = ChainRuntimeStatus::NotStarted;
            }
            self.last_chain_probe_at = None;
            return;
        }

        let now = Instant::now();
        let should_probe = self.last_chain_probe_at.is_none_or(|last| {
            now.duration_since(last) >= Duration::from_millis(CHAIN_STATUS_PROBE_INTERVAL_MS)
        });
        if !should_probe {
            return;
        }

        self.last_chain_probe_at = Some(now);
        match probe_chain_status_endpoint(self.config.chain_status_bind.as_str()) {
            Ok(()) => {
                self.chain_runtime_status = ChainRuntimeStatus::Ready;
            }
            Err(err) => {
                let within_grace = self.chain_started_at.is_some_and(|started_at| {
                    now.duration_since(started_at)
                        < Duration::from_secs(CHAIN_STATUS_STARTING_GRACE_SECS)
                });
                if within_grace {
                    self.chain_runtime_status = ChainRuntimeStatus::Starting;
                } else if err.contains("chain status bind") {
                    self.chain_runtime_status = ChainRuntimeStatus::ConfigError(err);
                } else {
                    self.chain_runtime_status = ChainRuntimeStatus::Unreachable(err);
                }
            }
        }
    }

    pub(super) fn poll_process(&mut self) {
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
                self.launcher_started_at = None;
                self.append_log(format!("game launcher exited: {status}"));
                self.running = None;
            }
            Ok(None) => {
                self.running = Some(running);
            }
            Err(err) => {
                self.status = LauncherStatus::QueryFailed;
                self.launcher_started_at = None;
                self.append_log(format!("query game launcher status failed: {err}"));
                self.running = None;
            }
        }
    }

    pub(super) fn poll_chain_process(&mut self) {
        let mut running = match self.chain_running.take() {
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
                self.chain_started_at = None;
                self.last_chain_probe_at = None;
                self.chain_runtime_status = ChainRuntimeStatus::Unreachable(format!(
                    "world_chain_runtime exited: {status}"
                ));
                self.append_log(format!("chain runtime exited: {status}"));
                self.chain_running = None;
            }
            Ok(None) => {
                self.chain_running = Some(running);
            }
            Err(err) => {
                self.chain_started_at = None;
                self.last_chain_probe_at = None;
                self.chain_runtime_status =
                    ChainRuntimeStatus::Unreachable(format!("query chain runtime failed: {err}"));
                self.append_log(format!("query chain runtime status failed: {err}"));
                self.chain_running = None;
            }
        }
    }

    pub(super) fn stop_process(&mut self) {
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
                self.launcher_started_at = None;
                self.append_log("game launcher stopped");
            }
            Err(err) => {
                self.status = LauncherStatus::StopFailed;
                self.append_log(format!("game launcher stop failed: {err}"));
            }
        }
    }

    pub(super) fn start_process(&mut self) {
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

        let launch_args = match build_launcher_args(&self.config) {
            Ok(args) => args,
            Err(err) => {
                self.status = LauncherStatus::InvalidArgs;
                self.append_log(format!("invalid game launcher args: {err}"));
                return;
            }
        };

        match spawn_child_process(
            self.config.launcher_bin.as_str(),
            launch_args.as_slice(),
            "game",
        ) {
            Ok(process) => {
                self.status = LauncherStatus::Running;
                self.launcher_started_at = Some(Instant::now());
                self.append_log("game launcher started");
                self.running = Some(process);
            }
            Err(err) => {
                self.status = LauncherStatus::StartFailed;
                self.launcher_started_at = None;
                self.append_log(format!("game launcher start failed: {err}"));
            }
        }
    }

    pub(super) fn stop_chain_process(&mut self) {
        let mut running = match self.chain_running.take() {
            Some(process) => process,
            None => {
                let message = self
                    .tr(
                        "无需停止：区块链未运行",
                        "no running blockchain process to stop",
                    )
                    .to_string();
                self.append_log(message);
                return;
            }
        };

        match stop_child_process(&mut running.child) {
            Ok(()) => {
                self.chain_started_at = None;
                self.last_chain_probe_at = None;
                self.chain_runtime_status = if self.config.chain_enabled {
                    ChainRuntimeStatus::NotStarted
                } else {
                    ChainRuntimeStatus::Disabled
                };
                self.append_log("chain runtime stopped");
            }
            Err(err) => {
                self.chain_runtime_status =
                    ChainRuntimeStatus::Unreachable(format!("stop chain runtime failed: {err}"));
                self.append_log(format!("chain runtime stop failed: {err}"));
            }
        }
    }

    pub(super) fn start_chain_process(&mut self) {
        if !self.config.chain_enabled {
            self.chain_runtime_status = ChainRuntimeStatus::Disabled;
            self.append_log("chain runtime start skipped: chain runtime disabled");
            return;
        }

        if self.chain_running.is_some() {
            self.append_log("chain runtime start skipped: process already running");
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

        let launch_args = match build_chain_runtime_args(&self.config) {
            Ok(args) => args,
            Err(err) => {
                self.chain_runtime_status = ChainRuntimeStatus::ConfigError(err.clone());
                self.append_log(format!("invalid chain runtime args: {err}"));
                return;
            }
        };

        match spawn_child_process(
            self.config.chain_runtime_bin.as_str(),
            launch_args.as_slice(),
            "chain",
        ) {
            Ok(process) => {
                self.chain_started_at = Some(Instant::now());
                self.last_chain_probe_at = None;
                self.chain_runtime_status = ChainRuntimeStatus::Starting;
                self.append_log("chain runtime started");
                self.chain_running = Some(process);
            }
            Err(err) => {
                self.chain_started_at = None;
                self.chain_runtime_status = ChainRuntimeStatus::Unreachable(err.clone());
                self.append_log(format!("chain runtime start failed: {err}"));
            }
        }
    }
}
