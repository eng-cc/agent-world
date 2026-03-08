use super::*;
pub(super) fn launcher_text_field_mut<'a>(
    config: &'a mut LaunchConfig,
    field_id: &str,
) -> Option<&'a mut String> {
    match field_id {
        "scenario" => Some(&mut config.scenario),
        "live_bind" => Some(&mut config.live_bind),
        "web_bind" => Some(&mut config.web_bind),
        "viewer_host" => Some(&mut config.viewer_host),
        "viewer_port" => Some(&mut config.viewer_port),
        "chain_status_bind" => Some(&mut config.chain_status_bind),
        "chain_node_id" => Some(&mut config.chain_node_id),
        "chain_world_id" => Some(&mut config.chain_world_id),
        "chain_node_role" => Some(&mut config.chain_node_role),
        "chain_node_tick_ms" => Some(&mut config.chain_node_tick_ms),
        "chain_pos_slot_duration_ms" => Some(&mut config.chain_pos_slot_duration_ms),
        "chain_pos_ticks_per_slot" => Some(&mut config.chain_pos_ticks_per_slot),
        "chain_pos_proposal_tick_phase" => Some(&mut config.chain_pos_proposal_tick_phase),
        "chain_pos_slot_clock_genesis_unix_ms" => {
            Some(&mut config.chain_pos_slot_clock_genesis_unix_ms)
        }
        "chain_pos_max_past_slot_lag" => Some(&mut config.chain_pos_max_past_slot_lag),
        "chain_node_validators" => Some(&mut config.chain_node_validators),
        "launcher_bin" => Some(&mut config.launcher_bin),
        "chain_runtime_bin" => Some(&mut config.chain_runtime_bin),
        "viewer_static_dir" => Some(&mut config.viewer_static_dir),
        _ => None,
    }
}

pub(super) fn launcher_checkbox_field_mut<'a>(
    config: &'a mut LaunchConfig,
    field_id: &str,
) -> Option<&'a mut bool> {
    match field_id {
        "llm_enabled" => Some(&mut config.llm_enabled),
        "chain_enabled" => Some(&mut config.chain_enabled),
        "chain_pos_adaptive_tick_scheduler_enabled" => {
            Some(&mut config.chain_pos_adaptive_tick_scheduler_enabled)
        }
        "auto_open_browser" => Some(&mut config.auto_open_browser),
        _ => None,
    }
}

pub(super) fn collect_required_config_issues(config: &LaunchConfig) -> Vec<ConfigIssue> {
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
    }
    #[cfg(not(target_arch = "wasm32"))]
    if !viewer_static_dir.is_empty() && !Path::new(viewer_static_dir).is_dir() {
        issues.push(ConfigIssue::ViewerStaticDirMissing);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let launcher_bin = config.launcher_bin.trim();
        if launcher_bin.is_empty() {
            issues.push(ConfigIssue::LauncherBinRequired);
        } else if !Path::new(launcher_bin).is_file() {
            issues.push(ConfigIssue::LauncherBinMissing);
        }
    }

    issues
}

pub(super) fn collect_chain_required_config_issues(config: &LaunchConfig) -> Vec<ConfigIssue> {
    let mut issues = Vec::new();
    if !config.chain_enabled {
        return issues;
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let chain_runtime_bin = config.chain_runtime_bin.trim();
        if chain_runtime_bin.is_empty() {
            issues.push(ConfigIssue::ChainRuntimeBinRequired);
        } else if !Path::new(chain_runtime_bin).is_file() {
            issues.push(ConfigIssue::ChainRuntimeBinMissing);
        }
    }

    if parse_host_port(config.chain_status_bind.as_str(), "chain status bind").is_err() {
        issues.push(ConfigIssue::ChainStatusBindInvalid);
    }
    if config.chain_node_id.trim().is_empty() {
        issues.push(ConfigIssue::ChainNodeIdRequired);
    }
    if parse_chain_role(config.chain_node_role.as_str()).is_err() {
        issues.push(ConfigIssue::ChainRoleInvalid);
    }
    if parse_positive_u64(
        config.chain_node_tick_ms.as_str(),
        "chain node poll interval ms",
    )
    .is_err()
    {
        issues.push(ConfigIssue::ChainTickMsInvalid);
    }
    if parse_positive_u64(
        config.chain_pos_slot_duration_ms.as_str(),
        "chain pos slot duration ms",
    )
    .is_err()
    {
        issues.push(ConfigIssue::ChainPosSlotDurationMsInvalid);
    }
    let ticks_per_slot = parse_positive_u64(
        config.chain_pos_ticks_per_slot.as_str(),
        "chain pos ticks per slot",
    );
    if ticks_per_slot.is_err() {
        issues.push(ConfigIssue::ChainPosTicksPerSlotInvalid);
    }
    let proposal_tick_phase = parse_non_negative_u64(
        config.chain_pos_proposal_tick_phase.as_str(),
        "chain pos proposal tick phase",
    );
    if proposal_tick_phase.is_err() {
        issues.push(ConfigIssue::ChainPosProposalTickPhaseInvalid);
    }
    if let (Ok(ticks_per_slot), Ok(proposal_tick_phase)) = (ticks_per_slot, proposal_tick_phase) {
        if proposal_tick_phase >= ticks_per_slot {
            issues.push(ConfigIssue::ChainPosProposalTickPhaseOutOfRange);
        }
    }
    if parse_optional_i64(
        config.chain_pos_slot_clock_genesis_unix_ms.as_str(),
        "chain pos slot clock genesis unix ms",
    )
    .is_err()
    {
        issues.push(ConfigIssue::ChainPosSlotClockGenesisUnixMsInvalid);
    }
    if parse_non_negative_u64(
        config.chain_pos_max_past_slot_lag.as_str(),
        "chain pos max past slot lag",
    )
    .is_err()
    {
        issues.push(ConfigIssue::ChainPosMaxPastSlotLagInvalid);
    }
    if parse_chain_validators(config.chain_node_validators.as_str()).is_err() {
        issues.push(ConfigIssue::ChainValidatorsInvalid);
    }
    issues
}

pub(super) fn build_launcher_args(config: &LaunchConfig) -> Result<Vec<String>, String> {
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
        "--chain-disable".to_string(),
    ];

    if config.llm_enabled {
        args.push("--with-llm".to_string());
    }
    if !config.auto_open_browser {
        args.push("--no-open-browser".to_string());
    }

    Ok(args)
}

pub(super) fn build_chain_runtime_args(config: &LaunchConfig) -> Result<Vec<String>, String> {
    let chain_runtime_bin = config.chain_runtime_bin.trim();
    if chain_runtime_bin.is_empty() {
        return Err("chain runtime bin cannot be empty".to_string());
    }
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
    let scenario = config.scenario.trim();
    let default_world_id = if scenario.is_empty() {
        format!("live-{DEFAULT_SCENARIO}")
    } else {
        format!("live-{scenario}")
    };
    let chain_world_id = if config.chain_world_id.trim().is_empty() {
        default_world_id
    } else {
        config.chain_world_id.trim().to_string()
    };

    let mut args = vec![
        "--node-id".to_string(),
        config.chain_node_id.trim().to_string(),
        "--world-id".to_string(),
        chain_world_id,
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

pub(super) fn build_game_url(config: &LaunchConfig) -> String {
    let viewer_host = normalize_host_for_url(config.viewer_host.as_str());
    let viewer_host = host_for_url(viewer_host.as_str());
    let viewer_port = parse_port(config.viewer_port.as_str(), "viewer port").unwrap_or(4173);
    let (web_host, web_port) = parse_host_port(config.web_bind.as_str(), "web bind")
        .unwrap_or(("127.0.0.1".to_string(), 5011));
    let web_host = normalize_host_for_url(web_host.as_str());
    let web_host = host_for_url(web_host.as_str());

    format!("http://{viewer_host}:{viewer_port}/?ws=ws://{web_host}:{web_port}")
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn probe_chain_status_endpoint(bind: &str) -> Result<(), String> {
    let (host, port) = parse_host_port(bind, "chain status bind")?;
    let host = normalize_host_for_connect(host.as_str());
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
    stream
        .write_all(request.as_bytes())
        .map_err(|err| format!("write chain status probe failed: {err}"))?;

    let mut buffer = [0_u8; 256];
    let bytes = stream
        .read(&mut buffer)
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

pub(super) fn normalize_host_for_connect(host: &str) -> String {
    let host = host.trim();
    if host == "0.0.0.0" {
        "127.0.0.1".to_string()
    } else if host == "::" || host == "[::]" {
        "::1".to_string()
    } else {
        host.to_string()
    }
}

pub(super) fn normalize_host_for_url(host: &str) -> String {
    let host = host.trim();
    if host == "0.0.0.0" || host == "::" || host == "[::]" || host.is_empty() {
        "127.0.0.1".to_string()
    } else {
        host.to_string()
    }
}

pub(super) fn host_for_url(host: &str) -> String {
    if host.contains(':') && !host.starts_with('[') && !host.ends_with(']') {
        format!("[{host}]")
    } else {
        host.to_string()
    }
}

pub(super) fn parse_port(raw: &str, label: &str) -> Result<u16, String> {
    let value = raw.trim();
    let port = value
        .parse::<u16>()
        .map_err(|_| format!("{label} must be integer in 1..=65535"))?;
    if port == 0 {
        return Err(format!("{label} must be in 1..=65535"));
    }
    Ok(port)
}

pub(super) fn parse_positive_u64(raw: &str, label: &str) -> Result<u64, String> {
    let value = raw.trim();
    let parsed = value
        .parse::<u64>()
        .map_err(|_| format!("{label} must be a positive integer"))?;
    if parsed == 0 {
        return Err(format!("{label} must be a positive integer"));
    }
    Ok(parsed)
}

pub(super) fn parse_non_negative_u64(raw: &str, label: &str) -> Result<u64, String> {
    let value = raw.trim();
    value
        .parse::<u64>()
        .map_err(|_| format!("{label} must be a non-negative integer"))
}

pub(super) fn parse_optional_i64(raw: &str, label: &str) -> Result<Option<i64>, String> {
    let value = raw.trim();
    if value.is_empty() {
        return Ok(None);
    }
    value
        .parse::<i64>()
        .map(Some)
        .map_err(|_| format!("{label} must be an integer"))
}

pub(super) fn parse_host_port(raw: &str, label: &str) -> Result<(String, u16), String> {
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

pub(super) fn parse_chain_role(raw: &str) -> Result<String, String> {
    let role = raw.trim().to_ascii_lowercase();
    match role.as_str() {
        "sequencer" | "storage" | "observer" => Ok(role),
        _ => Err("chain role must be one of: sequencer|storage|observer".to_string()),
    }
}

pub(super) fn parse_chain_validators(raw: &str) -> Result<Vec<String>, String> {
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

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn spawn_child_process(
    bin: &str,
    args: &[String],
    process_label: &str,
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

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn spawn_log_reader<R: Read + Send + 'static>(
    reader: R,
    process_label: &str,
    source: &'static str,
    tx: Sender<String>,
) {
    let process_label = process_label.to_string();
    std::thread::spawn(move || {
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

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn stop_child_process(child: &mut Child) -> Result<(), String> {
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
            std::thread::sleep(Duration::from_millis(STOP_POLL_INTERVAL_MS));
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

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn send_interrupt_signal(child: &Child) -> Result<(), String> {
    #[cfg(unix)]
    {
        let pid = child.id() as i32;
        // SAFETY: libc::kill is called with a pid from std::process::Child.
        let rc = unsafe { libc::kill(pid, libc::SIGINT) };
        if rc == 0 {
            return Ok(());
        }
        return Err(format!(
            "send SIGINT failed: {}",
            std::io::Error::last_os_error()
        ));
    }

    #[cfg(not(unix))]
    {
        let _ = child;
        Ok(())
    }
}
