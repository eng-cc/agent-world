fn start_reward_runtime_worker(
    options: &CliOptions,
    node_handle: Option<LiveNodeHandle>,
) -> Result<Option<RewardRuntimeWorker>, String> {
    if !options.reward_runtime_enabled {
        return Ok(None);
    }
    let handle = node_handle.ok_or_else(|| {
        "reward runtime requires embedded node runtime; disable --no-node or reward runtime"
            .to_string()
    })?;
    let runtime = Arc::clone(&handle.primary_runtime);

    let signer_node_id = options
        .reward_runtime_signer_node_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| handle.primary_node_id.clone());
    if signer_node_id.trim().is_empty() {
        return Err("reward runtime signer node id cannot be empty".to_string());
    }

    let report_dir = options.reward_runtime_report_dir.trim().to_string();
    if report_dir.is_empty() {
        return Err("reward runtime report dir cannot be empty".to_string());
    }
    let signer_keypair = ensure_node_keypair_in_config(Path::new(DEFAULT_CONFIG_FILE_NAME))
        .map_err(|err| format!("failed to load reward runtime signer keypair: {err}"))?;
    let world_id = handle.world_id.clone();
    let primary_node_id = handle.primary_node_id.clone();
    let settlement_leader_node_id = options
        .reward_runtime_leader_node_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| infer_default_reward_runtime_leader_node_id(primary_node_id.as_str()));
    let reward_runtime_node_identity_bindings = reward_runtime_node_identity_bindings(
        options,
        primary_node_id.as_str(),
        signer_node_id.as_str(),
        settlement_leader_node_id.as_str(),
        &signer_keypair,
    )?;
    let node_report_root = reward_runtime_node_report_root(
        options.reward_runtime_report_dir.as_str(),
        primary_node_id.as_str(),
    );
    let reward_network = handle.reward_network.clone();

    let config = RewardRuntimeLoopConfig {
        world_id,
        local_node_id: primary_node_id.clone(),
        settlement_leader_node_id,
        settlement_leader_stale_ms: options.reward_runtime_leader_stale_ms,
        settlement_failover_enabled: options.reward_runtime_failover_enabled,
        reward_network,
        poll_interval: Duration::from_millis(options.node_tick_ms),
        signer_node_id,
        signer_private_key_hex: signer_keypair.private_key_hex,
        signer_public_key_hex: signer_keypair.public_key_hex,
        report_dir,
        state_path: node_report_root.join(DEFAULT_REWARD_RUNTIME_STATE_FILE),
        distfs_probe_state_path: node_report_root
            .join(DEFAULT_REWARD_RUNTIME_DISTFS_PROBE_STATE_FILE),
        execution_bridge_state_path: node_report_root
            .join(DEFAULT_REWARD_RUNTIME_EXECUTION_BRIDGE_STATE_FILE),
        execution_world_dir: node_report_root.join(DEFAULT_REWARD_RUNTIME_EXECUTION_WORLD_DIR),
        execution_records_dir: node_report_root.join(DEFAULT_REWARD_RUNTIME_EXECUTION_RECORDS_DIR),
        storage_root: Path::new("output")
            .join("node-distfs")
            .join(primary_node_id.as_str())
            .join("store"),
        distfs_probe_config: options.reward_distfs_probe_config,
        auto_redeem: options.reward_runtime_auto_redeem,
        reward_asset_config: RewardAssetConfig {
            points_per_credit: options.reward_points_per_credit,
            credits_per_power_unit: options.reward_credits_per_power_unit,
            max_redeem_power_per_epoch: options.reward_max_redeem_power_per_epoch,
            min_redeem_power_unit: options.reward_min_redeem_power_unit,
        },
        initial_reserve_power_units: options.reward_initial_reserve_power_units,
        min_observer_traces: options.reward_runtime_min_observer_traces,
        reward_runtime_epoch_duration_secs: options.reward_runtime_epoch_duration_secs,
        reward_runtime_node_identity_bindings,
    };

    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let join_handle = thread::Builder::new()
        .name("reward-runtime".to_string())
        .spawn(move || reward_runtime_loop(runtime, config, stop_rx))
        .map_err(|err| format!("failed to spawn reward runtime worker: {err}"))?;

    Ok(Some(RewardRuntimeWorker {
        stop_tx,
        join_handle,
    }))
}

fn stop_reward_runtime_worker(worker: Option<RewardRuntimeWorker>) {
    let Some(worker) = worker else {
        return;
    };
    let _ = worker.stop_tx.send(());
    if worker.join_handle.join().is_err() {
        eprintln!("reward runtime worker join failed");
    }
}

fn reward_runtime_node_identity_bindings(
    options: &CliOptions,
    local_node_id: &str,
    signer_node_id: &str,
    settlement_leader_node_id: &str,
    root_keypair: &node_keypair_config::NodeKeypairConfig,
) -> Result<BTreeMap<String, String>, String> {
    let mut node_ids = BTreeSet::new();
    node_ids.insert(local_node_id.trim().to_string());
    node_ids.insert(signer_node_id.trim().to_string());
    node_ids.insert(settlement_leader_node_id.trim().to_string());
    for validator in &options.node_validators {
        node_ids.insert(validator.validator_id.trim().to_string());
    }
    node_ids.retain(|node_id| !node_id.is_empty());

    let mut bindings = BTreeMap::new();
    for node_id in node_ids {
        if node_id == signer_node_id {
            bindings.insert(node_id, root_keypair.public_key_hex.clone());
            continue;
        }
        let keypair = derive_node_consensus_signer_keypair(node_id.as_str(), root_keypair)?;
        bindings.insert(node_id, keypair.public_key_hex);
    }
    Ok(bindings)
}

fn reward_runtime_loop(
    node_runtime: Arc<Mutex<NodeRuntime>>,
    config: RewardRuntimeLoopConfig,
    stop_rx: mpsc::Receiver<()>,
) {
    if let Err(err) = fs::create_dir_all(config.report_dir.as_str()) {
        eprintln!(
            "reward runtime create report dir failed {}: {}",
            config.report_dir, err
        );
    }

    let points_config = reward_runtime_points_config(config.reward_runtime_epoch_duration_secs);
    let mut collector = match load_reward_runtime_collector_snapshot(config.state_path.as_path()) {
        Ok(Some(mut restored)) => {
            if let Some(epoch_duration_secs) = config.reward_runtime_epoch_duration_secs {
                restored.ledger.config.epoch_duration_seconds = epoch_duration_secs;
            }
            NodePointsRuntimeCollector::from_snapshot(restored)
        }
        Ok(None) => NodePointsRuntimeCollector::new(
            points_config.clone(),
            NodePointsRuntimeHeuristics::default(),
        ),
        Err(err) => {
            eprintln!("reward runtime load collector state failed: {err}");
            NodePointsRuntimeCollector::new(points_config, NodePointsRuntimeHeuristics::default())
        }
    };
    let mut distfs_probe_state =
        match load_reward_runtime_distfs_probe_state(config.distfs_probe_state_path.as_path()) {
            Ok(state) => state,
            Err(err) => {
                eprintln!("reward runtime load distfs probe state failed: {err}");
                StorageChallengeProbeCursorState::default()
            }
        };
    let mut execution_bridge_state =
        match load_execution_bridge_state(config.execution_bridge_state_path.as_path()) {
            Ok(state) => state,
            Err(err) => {
                eprintln!("reward runtime load execution bridge state failed: {err}");
                execution_bridge::ExecutionBridgeState::default()
            }
        };
    let mut execution_world = match load_execution_world(config.execution_world_dir.as_path()) {
        Ok(world) => world,
        Err(err) => {
            eprintln!("reward runtime load execution world failed: {err}");
            RuntimeWorld::new()
        }
    };
    let mut execution_sandbox = WasmExecutor::new(WasmExecutorConfig::default());
    let execution_store = LocalCasStore::new(config.storage_root.as_path());
    let mut reward_world = RuntimeWorld::new();
    reward_world.set_reward_asset_config(config.reward_asset_config.clone());
    reward_world.set_reward_signature_governance_policy(RewardSignatureGovernancePolicy {
        require_mintsig_v2: true,
        allow_mintsig_v1_fallback: false,
        require_redeem_signature: true,
        require_redeem_signer_match_node_id: true,
    });
    reward_world.set_protocol_power_reserve(ProtocolPowerReserve {
        epoch_index: 0,
        available_power_units: config.initial_reserve_power_units.max(0),
        redeemed_power_units: 0,
    });
    for (node_id, public_key_hex) in &config.reward_runtime_node_identity_bindings {
        if let Err(err) = reward_world.bind_node_identity(node_id.as_str(), public_key_hex.as_str())
        {
            eprintln!(
                "reward runtime bind node identity failed node={} err={:?}",
                node_id, err
            );
        }
    }
    let settlement_topic = reward_settlement_topic(config.world_id.as_str());
    let settlement_subscription = match config.reward_network.as_ref() {
        Some(network) => match network.subscribe(settlement_topic.as_str()) {
            Ok(subscription) => Some(subscription),
            Err(err) => {
                eprintln!(
                    "reward runtime subscribe settlement topic failed {}: {:?}",
                    settlement_topic, err
                );
                None
            }
        },
        None => None,
    };
    let settlement_network_enabled =
        config.reward_network.is_some() && settlement_subscription.is_some();
    let observation_topic = reward_observation_topic(config.world_id.as_str());
    let observation_subscription = match config.reward_network.as_ref() {
        Some(network) => match network.subscribe(observation_topic.as_str()) {
            Ok(subscription) => Some(subscription),
            Err(err) => {
                eprintln!(
                    "reward runtime subscribe observation topic failed {}: {:?}",
                    observation_topic, err
                );
                None
            }
        },
        None => None,
    };
    let observation_network_enabled =
        config.reward_network.is_some() && observation_subscription.is_some();
    let mut distfs_challenge_network = match config.reward_network.as_ref() {
        Some(network) => match DistfsChallengeNetworkDriver::new(
            config.world_id.as_str(),
            config.local_node_id.as_str(),
            config.signer_private_key_hex.as_str(),
            config.signer_public_key_hex.as_str(),
            config.storage_root.clone(),
            config.distfs_probe_config,
            Arc::clone(network),
        ) {
            Ok(driver) => Some(driver),
            Err(err) => {
                eprintln!("reward runtime init distfs challenge network driver failed: {err}");
                None
            }
        },
        None => None,
    };
    let mut applied_settlement_envelope_ids = BTreeSet::new();
    let mut applied_observation_trace_ids = BTreeSet::new();
    let mut epoch_observer_nodes = BTreeSet::new();

    loop {
        match stop_rx.recv_timeout(config.poll_interval) {
            Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }

        let snapshot = match node_runtime.lock() {
            Ok(locked) => locked.snapshot(),
            Err(_) => {
                eprintln!("reward runtime failed to read node snapshot: lock poisoned");
                break;
            }
        };
        let observed_at_unix_ms = now_unix_ms();
        let effective_storage_bytes =
            measure_directory_storage_bytes(config.storage_root.as_path());
        let node_execution_available = snapshot.consensus.last_execution_height
            >= snapshot.consensus.committed_height
            && snapshot.consensus.committed_height > 0
            && snapshot.consensus.last_execution_block_hash.is_some()
            && snapshot.consensus.last_execution_state_root.is_some();
        let execution_bridge_records = if node_execution_available {
            match load_execution_bridge_state(config.execution_bridge_state_path.as_path()) {
                Ok(state) => {
                    execution_bridge_state = state;
                }
                Err(err) => {
                    eprintln!("reward runtime reload execution bridge state failed: {err}");
                }
            }
            Vec::new()
        } else {
            match bridge_committed_heights(
                &snapshot,
                observed_at_unix_ms,
                &mut execution_world,
                &mut execution_sandbox,
                &execution_store,
                config.execution_records_dir.as_path(),
                &mut execution_bridge_state,
            ) {
                Ok(records) => {
                    if !records.is_empty() {
                        if let Err(err) = persist_execution_bridge_state(
                            config.execution_bridge_state_path.as_path(),
                            &execution_bridge_state,
                        ) {
                            eprintln!(
                                "reward runtime persist execution bridge state failed: {err}"
                            );
                        }
                        if let Err(err) = persist_execution_world(
                            config.execution_world_dir.as_path(),
                            &execution_world,
                        ) {
                            eprintln!("reward runtime persist execution world failed: {err}");
                        }
                    }
                    records
                }
                Err(err) => {
                    eprintln!("reward runtime execution bridge failed: {err}");
                    Vec::new()
                }
            }
        };

        let mut observation = NodePointsRuntimeObservation::from_snapshot(
            &snapshot,
            effective_storage_bytes,
            observed_at_unix_ms,
        );
        let mut distfs_network_tick_report: Option<DistfsChallengeNetworkTickReport> = None;
        let distfs_challenge_report = if snapshot.role == NodeRole::Storage {
            if let Some(driver) = distfs_challenge_network.as_mut() {
                driver.register_observation_role(snapshot.node_id.as_str(), snapshot.role);
                let tick_report = driver.tick(observed_at_unix_ms);
                let fallback_local = tick_report.should_fallback_local();
                let probe_report = tick_report.probe_report.clone();
                distfs_network_tick_report = Some(tick_report);
                if !fallback_local {
                    if let Some(report) = probe_report.as_ref() {
                        observation.storage_checks_passed = report.passed_checks;
                        observation.storage_checks_total = report.total_checks;
                        if report.failed_checks > 0 {
                            observation.has_error = true;
                        }
                        if let Some(semantics) = report.latest_proof_semantics.as_ref() {
                            observation.storage_challenge_proof_hint = serde_json::from_value(
                                storage_proof_hint_value_from_semantics(semantics),
                            )
                            .ok();
                        }
                    }
                    probe_report
                } else {
                    match collect_distfs_challenge_report_with_config(
                        config.storage_root.as_path(),
                        snapshot.world_id.as_str(),
                        snapshot.node_id.as_str(),
                        observed_at_unix_ms,
                        &mut distfs_probe_state,
                        &config.distfs_probe_config,
                    ) {
                        Ok(report) => {
                            observation.storage_checks_passed = report.passed_checks;
                            observation.storage_checks_total = report.total_checks;
                            if report.failed_checks > 0 {
                                observation.has_error = true;
                            }
                            if let Some(semantics) = report.latest_proof_semantics.as_ref() {
                                observation.storage_challenge_proof_hint = serde_json::from_value(
                                    storage_proof_hint_value_from_semantics(semantics),
                                )
                                .ok();
                            }
                            Some(report)
                        }
                        Err(err) => {
                            eprintln!("reward runtime distfs probe failed: {err}");
                            None
                        }
                    }
                }
            } else {
                match collect_distfs_challenge_report_with_config(
                    config.storage_root.as_path(),
                    snapshot.world_id.as_str(),
                    snapshot.node_id.as_str(),
                    observed_at_unix_ms,
                    &mut distfs_probe_state,
                    &config.distfs_probe_config,
                ) {
                    Ok(report) => {
                        observation.storage_checks_passed = report.passed_checks;
                        observation.storage_checks_total = report.total_checks;
                        if report.failed_checks > 0 {
                            observation.has_error = true;
                        }
                        if let Some(semantics) = report.latest_proof_semantics.as_ref() {
                            observation.storage_challenge_proof_hint = serde_json::from_value(
                                storage_proof_hint_value_from_semantics(semantics),
                            )
                            .ok();
                        }
                        Some(report)
                    }
                    Err(err) => {
                        eprintln!("reward runtime distfs probe failed: {err}");
                        None
                    }
                }
            }
        } else {
            None
        };

        let local_trace = match sign_reward_observation_trace(
            config.world_id.as_str(),
            snapshot.node_id.as_str(),
            config.signer_private_key_hex.as_str(),
            config.signer_public_key_hex.as_str(),
            RewardObservationPayload::from_observation(observation),
            observed_at_unix_ms,
        ) {
            Ok(trace) => trace,
            Err(err) => {
                eprintln!("reward runtime sign local observation trace failed: {err}");
                continue;
            }
        };
        if observation_network_enabled {
            if let Some(network) = config.reward_network.as_ref() {
                match encode_reward_observation_trace(&local_trace) {
                    Ok(payload) => {
                        if let Err(err) =
                            network.publish(observation_topic.as_str(), payload.as_slice())
                        {
                            eprintln!("reward runtime publish observation trace failed: {:?}", err);
                        }
                    }
                    Err(err) => {
                        eprintln!("reward runtime encode observation trace failed: {err}");
                    }
                }
            }
        }

        let mut applied_observation_trace_ids_this_tick = Vec::new();
        let mut applied_observer_nodes_this_tick = BTreeSet::new();
        let mut applied_observation_traces_this_tick = Vec::new();
        let mut maybe_report = None;
        if let Some(applied) = observe_reward_observation_trace(
            &mut collector,
            local_trace.clone(),
            config.world_id.as_str(),
            "local",
            &mut applied_observation_trace_ids,
            &mut epoch_observer_nodes,
        ) {
            if let Some(report) = applied.report {
                maybe_report = Some(report);
            }
            if let Some(driver) = distfs_challenge_network.as_mut() {
                driver.register_observation_role(
                    applied.observer_node_id.as_str(),
                    applied.observer_role,
                );
            }
            applied_observation_trace_ids_this_tick.push(applied.trace_id.clone());
            applied_observer_nodes_this_tick.insert(applied.observer_node_id.clone());
            applied_observation_traces_this_tick.push(serde_json::json!({
                "trace_id": applied.trace_id,
                "observer_node_id": applied.observer_node_id,
                "observer_role": applied.observer_role.as_str(),
                "payload_hash": applied.payload_hash,
                "source": "local",
            }));
        }
        if observation_network_enabled {
            if let Some(subscription) = observation_subscription.as_ref() {
                for payload in subscription.drain() {
                    let trace = match decode_reward_observation_trace(payload.as_slice()) {
                        Ok(trace) => trace,
                        Err(err) => {
                            eprintln!("reward runtime decode observation trace failed: {err}");
                            continue;
                        }
                    };
                    if trace.version != 1 || trace.world_id != config.world_id {
                        continue;
                    }
                    if let Some(applied) = observe_reward_observation_trace(
                        &mut collector,
                        trace,
                        config.world_id.as_str(),
                        "network",
                        &mut applied_observation_trace_ids,
                        &mut epoch_observer_nodes,
                    ) {
                        if let Some(report) = applied.report {
                            maybe_report = Some(report);
                        }
                        if let Some(driver) = distfs_challenge_network.as_mut() {
                            driver.register_observation_role(
                                applied.observer_node_id.as_str(),
                                applied.observer_role,
                            );
                        }
                        applied_observation_trace_ids_this_tick.push(applied.trace_id.clone());
                        applied_observer_nodes_this_tick.insert(applied.observer_node_id.clone());
                        applied_observation_traces_this_tick.push(serde_json::json!({
                            "trace_id": applied.trace_id,
                            "observer_node_id": applied.observer_node_id,
                            "observer_role": applied.observer_role.as_str(),
                            "payload_hash": applied.payload_hash,
                            "source": "network",
                        }));
                    }
                }
            }
        }

        let mut applied_settlement_ids_this_tick = Vec::new();
        if let Some(subscription) = settlement_subscription.as_ref() {
            for payload in subscription.drain() {
                let envelope = match decode_reward_settlement_envelope(payload.as_slice()) {
                    Ok(envelope) => envelope,
                    Err(err) => {
                        eprintln!("reward runtime decode settlement envelope failed: {err}");
                        continue;
                    }
                };
                if envelope.version != 1 || envelope.world_id != config.world_id {
                    continue;
                }
                let envelope_id = match reward_settlement_envelope_id(&envelope) {
                    Ok(id) => id,
                    Err(err) => {
                        eprintln!("reward runtime hash settlement envelope failed: {err}");
                        continue;
                    }
                };
                if applied_settlement_envelope_ids.contains(envelope_id.as_str()) {
                    continue;
                }
                match apply_reward_settlement_envelope(&mut reward_world, &envelope) {
                    Ok(()) => {
                        applied_settlement_envelope_ids.insert(envelope_id.clone());
                        applied_settlement_ids_this_tick.push(envelope_id);
                    }
                    Err(err) => {
                        eprintln!("reward runtime apply settlement envelope failed: {err}");
                    }
                }
            }
        }

        if let Err(err) =
            persist_reward_runtime_collector_state(config.state_path.as_path(), &collector)
        {
            eprintln!("reward runtime persist collector state failed: {err}");
        }
        if let Err(err) = persist_reward_runtime_distfs_probe_state(
            config.distfs_probe_state_path.as_path(),
            &distfs_probe_state,
        ) {
            eprintln!("reward runtime persist distfs probe state failed: {err}");
        }
        let Some(report) = maybe_report else {
            continue;
        };

        let observer_trace_count_for_epoch = epoch_observer_nodes.len();
        let observer_trace_threshold_met =
            observer_trace_count_for_epoch >= config.min_observer_traces as usize;
        let observer_nodes_for_epoch: Vec<String> = epoch_observer_nodes.iter().cloned().collect();
        epoch_observer_nodes.clear();

        rollover_reward_reserve_epoch(&mut reward_world, report.epoch_index);
        let settlement_leader_node_id = config.settlement_leader_node_id.as_str();
        let leader_last_commit_at_ms = if snapshot.node_id == settlement_leader_node_id {
            snapshot.consensus.last_committed_at_ms
        } else {
            snapshot
                .consensus
                .peer_heads
                .iter()
                .find(|head| head.node_id == settlement_leader_node_id)
                .map(|head| head.committed_at_ms)
        };
        let leader_is_stale = leader_last_commit_at_ms
            .map(|committed_at_ms| {
                observed_at_unix_ms.saturating_sub(committed_at_ms)
                    > config.settlement_leader_stale_ms as i64
            })
            .unwrap_or(false);
        let failover_publisher_node_id = if settlement_network_enabled
            && config.settlement_failover_enabled
            && leader_is_stale
        {
            select_failover_publisher_node_id(&snapshot, settlement_leader_node_id)
        } else {
            None
        };
        let local_is_settlement_publisher = snapshot.node_id == settlement_leader_node_id
            || failover_publisher_node_id.as_deref() == Some(snapshot.node_id.as_str());
        let consensus_ready_for_settlement =
            reward_runtime_consensus_ready_for_settlement(&snapshot);
        let should_publish_settlement = settlement_network_enabled
            && observer_trace_threshold_met
            && consensus_ready_for_settlement
            && local_is_settlement_publisher;
        let requires_local_settlement = observer_trace_threshold_met
            && (should_publish_settlement || !settlement_network_enabled);
        let minted_records = if requires_local_settlement {
            if let Err(err) = ensure_reward_runtime_settlement_node_identity_bindings(
                &mut reward_world,
                &report,
                &config.signer_private_key_hex,
                &config.signer_public_key_hex,
                &config.reward_runtime_node_identity_bindings,
            ) {
                eprintln!("reward runtime settlement identity binding failed: {err}");
                continue;
            }
            match build_reward_settlement_mint_records(
                &reward_world,
                &report,
                config.signer_node_id.as_str(),
                config.signer_private_key_hex.as_str(),
            ) {
                Ok(records) => records,
                Err(err) => {
                    eprintln!("reward runtime settlement mint failed: {err:?}");
                    continue;
                }
            }
        } else {
            Vec::new()
        };
        let mut published_settlement_envelope_id: Option<String> = None;
        let mut settlement_skipped_reason: Option<String> = None;
        if !observer_trace_threshold_met {
            settlement_skipped_reason = Some(format!(
                "observer trace threshold not met: have {}, require {}",
                observer_trace_count_for_epoch, config.min_observer_traces
            ));
        } else if settlement_network_enabled && !should_publish_settlement {
            settlement_skipped_reason = if !consensus_ready_for_settlement {
                Some(format!(
                    "local node consensus is not ready for settlement: status={:?} committed_height={} network_committed_height={}",
                    snapshot.consensus.last_status,
                    snapshot.consensus.committed_height,
                    snapshot.consensus.network_committed_height
                ))
            } else if snapshot.node_id != settlement_leader_node_id
                && !config.settlement_failover_enabled
            {
                Some(format!(
                    "local node is not leader {} and failover is disabled",
                    settlement_leader_node_id
                ))
            } else if snapshot.node_id != settlement_leader_node_id && !leader_is_stale {
                Some(format!("leader {} is not stale", settlement_leader_node_id))
            } else if snapshot.node_id != settlement_leader_node_id {
                Some(format!(
                    "failover publisher selected as {}",
                    failover_publisher_node_id.as_deref().unwrap_or("none")
                ))
            } else {
                Some("leader publish conditions are not met".to_string())
            };
        }

        if observer_trace_threshold_met && settlement_network_enabled {
            if should_publish_settlement {
                let envelope = match sign_reward_settlement_envelope(
                    config.world_id.as_str(),
                    config.signer_node_id.as_str(),
                    config.signer_private_key_hex.as_str(),
                    config.signer_public_key_hex.as_str(),
                    report.clone(),
                    minted_records.clone(),
                    observed_at_unix_ms,
                ) {
                    Ok(envelope) => envelope,
                    Err(err) => {
                        eprintln!("reward runtime sign settlement envelope failed: {err}");
                        continue;
                    }
                };
                let envelope_id = match reward_settlement_envelope_id(&envelope) {
                    Ok(id) => id,
                    Err(err) => {
                        eprintln!("reward runtime settlement envelope id failed: {err}");
                        continue;
                    }
                };
                if let Some(network) = config.reward_network.as_ref() {
                    match encode_reward_settlement_envelope(&envelope) {
                        Ok(payload) => {
                            if let Err(err) =
                                network.publish(settlement_topic.as_str(), payload.as_slice())
                            {
                                eprintln!(
                                    "reward runtime publish settlement envelope failed: {:?}",
                                    err
                                );
                            }
                        }
                        Err(err) => {
                            eprintln!("reward runtime encode settlement envelope failed: {err}");
                            continue;
                        }
                    }
                }
                match apply_reward_settlement_envelope(&mut reward_world, &envelope) {
                    Ok(()) => {
                        applied_settlement_envelope_ids.insert(envelope_id.clone());
                        applied_settlement_ids_this_tick.push(envelope_id.clone());
                        published_settlement_envelope_id = Some(envelope_id);
                    }
                    Err(err) => {
                        eprintln!("reward runtime local settlement envelope apply failed: {err}");
                        continue;
                    }
                }
            }
        } else if observer_trace_threshold_met && !minted_records.is_empty() {
            let envelope = match sign_reward_settlement_envelope(
                config.world_id.as_str(),
                config.signer_node_id.as_str(),
                config.signer_private_key_hex.as_str(),
                config.signer_public_key_hex.as_str(),
                report.clone(),
                minted_records.clone(),
                observed_at_unix_ms,
            ) {
                Ok(envelope) => envelope,
                Err(err) => {
                    eprintln!("reward runtime sign local settlement envelope failed: {err}");
                    continue;
                }
            };
            let envelope_id = match reward_settlement_envelope_id(&envelope) {
                Ok(id) => id,
                Err(err) => {
                    eprintln!("reward runtime local settlement envelope id failed: {err}");
                    continue;
                }
            };
            match apply_reward_settlement_envelope(&mut reward_world, &envelope) {
                Ok(()) => {
                    applied_settlement_envelope_ids.insert(envelope_id.clone());
                    applied_settlement_ids_this_tick.push(envelope_id.clone());
                    published_settlement_envelope_id = Some(envelope_id);
                }
                Err(err) => {
                    eprintln!("reward runtime local settlement apply failed: {err}");
                    continue;
                }
            }
        }

        if config.auto_redeem {
            auto_redeem_runtime_rewards(
                &mut reward_world,
                minted_records.as_slice(),
                config.signer_node_id.as_str(),
                config.signer_private_key_hex.as_str(),
            );
        }
        let invariant_report = reward_world.reward_asset_invariant_report();
        if !invariant_report.is_ok() {
            eprintln!(
                "reward runtime invariant violations detected: {}",
                invariant_report.violations.len()
            );
        }
        let applied_observer_nodes_this_tick: Vec<String> =
            applied_observer_nodes_this_tick.into_iter().collect();

        let payload = serde_json::json!({
            "observed_at_unix_ms": observed_at_unix_ms,
            "node_snapshot": {
                "node_id": snapshot.node_id,
                "world_id": snapshot.world_id,
                "role": snapshot.role.as_str(),
                "running": snapshot.running,
                "tick_count": snapshot.tick_count,
                "last_tick_unix_ms": snapshot.last_tick_unix_ms,
                "last_error": snapshot.last_error,
                "consensus": {
                    "mode": format!("{:?}", snapshot.consensus.mode),
                    "slot": snapshot.consensus.slot,
                    "epoch": snapshot.consensus.epoch,
                    "latest_height": snapshot.consensus.latest_height,
                    "committed_height": snapshot.consensus.committed_height,
                    "last_committed_at_ms": snapshot.consensus.last_committed_at_ms,
                    "network_committed_height": snapshot.consensus.network_committed_height,
                    "known_peer_heads": snapshot.consensus.known_peer_heads,
                    "peer_heads": snapshot.consensus.peer_heads,
                    "last_status": snapshot.consensus.last_status.map(|status| format!("{status:?}")),
                    "last_block_hash": snapshot.consensus.last_block_hash,
                    "last_execution_height": snapshot.consensus.last_execution_height,
                    "last_execution_block_hash": snapshot.consensus.last_execution_block_hash,
                    "last_execution_state_root": snapshot.consensus.last_execution_state_root,
                }
            },
            "distfs_challenge_report": distfs_challenge_report,
            "distfs_probe_config": serde_json::to_value(config.distfs_probe_config).unwrap_or(serde_json::Value::Null),
            "distfs_probe_cursor_state": serde_json::to_value(distfs_probe_state.clone()).unwrap_or(serde_json::Value::Null),
            "distfs_network_challenge": serde_json::to_value(distfs_network_tick_report).unwrap_or(serde_json::Value::Null),
            "execution_bridge_state": execution_bridge_state.clone(),
            "execution_bridge_records": execution_bridge_records,
            "settlement_report": report,
            "minted_records": minted_records,
            "reward_observation_traces": {
                "network_enabled": observation_network_enabled,
                "observation_topic": observation_topic,
                "min_observer_traces": config.min_observer_traces,
                "observer_trace_count_for_epoch": observer_trace_count_for_epoch,
                "observer_nodes_for_epoch": observer_nodes_for_epoch,
                "applied_trace_ids_this_tick": applied_observation_trace_ids_this_tick,
                "applied_observer_node_ids_this_tick": applied_observer_nodes_this_tick,
                "applied_traces_this_tick": applied_observation_traces_this_tick,
            },
            "reward_settlement_transport": {
                "network_enabled": settlement_network_enabled,
                "settlement_topic": settlement_topic,
                "should_publish_settlement": should_publish_settlement,
                "settlement_leader_node_id": settlement_leader_node_id,
                "leader_last_commit_at_ms": leader_last_commit_at_ms,
                "leader_is_stale": leader_is_stale,
                "settlement_failover_enabled": config.settlement_failover_enabled,
                "failover_publisher_node_id": failover_publisher_node_id,
                "published_settlement_envelope_id": published_settlement_envelope_id,
                "applied_settlement_envelope_ids": applied_settlement_ids_this_tick,
                "observer_trace_threshold_met": observer_trace_threshold_met,
                "settlement_skipped_reason": settlement_skipped_reason,
            },
            "node_balances": reward_world.state().node_asset_balances,
            "reserve": reward_world.protocol_power_reserve(),
            "reward_asset_invariant_status": reward_invariant_status_payload(&invariant_report),
            "reward_asset_invariant_report": invariant_report,
        });
        let report_path = Path::new(config.report_dir.as_str())
            .join(format!("epoch-{}.json", report.epoch_index));
        match serde_json::to_vec_pretty(&payload) {
            Ok(bytes) => {
                if let Err(err) = fs::write(&report_path, bytes) {
                    eprintln!(
                        "reward runtime write report failed {}: {}",
                        report_path.display(),
                        err
                    );
                }
            }
            Err(err) => {
                eprintln!("reward runtime serialize report failed: {err}");
            }
        }
    }
}

fn reward_runtime_consensus_ready_for_settlement(snapshot: &NodeSnapshot) -> bool {
    if matches!(
        snapshot.consensus.last_status,
        Some(PosConsensusStatus::Committed)
    ) {
        return true;
    }
    snapshot.consensus.committed_height > 0
        && snapshot.consensus.network_committed_height >= snapshot.consensus.committed_height
}

fn ensure_reward_runtime_settlement_node_identity_bindings(
    reward_world: &mut RuntimeWorld,
    report: &EpochSettlementReport,
    signer_private_key_hex: &str,
    signer_public_key_hex: &str,
    configured_bindings: &BTreeMap<String, String>,
) -> Result<(), String> {
    let signer_root_keypair = node_keypair_config::NodeKeypairConfig {
        private_key_hex: signer_private_key_hex.to_string(),
        public_key_hex: signer_public_key_hex.to_string(),
    };
    for settlement in &report.settlements {
        let node_id = settlement.node_id.as_str();
        if reward_world.node_identity_public_key(node_id).is_some() {
            continue;
        }
        let public_key_hex = if let Some(bound) = configured_bindings.get(node_id) {
            bound.clone()
        } else {
            derive_node_consensus_signer_keypair(node_id, &signer_root_keypair)?.public_key_hex
        };
        reward_world
            .bind_node_identity(node_id, public_key_hex.as_str())
            .map_err(|err| format!("{:?}", err))?;
    }
    Ok(())
}

fn reward_runtime_points_config(epoch_duration_secs_override: Option<u64>) -> NodePointsConfig {
    let mut config = NodePointsConfig::default();
    if let Some(epoch_duration_secs) = epoch_duration_secs_override {
        config.epoch_duration_seconds = epoch_duration_secs;
    }
    config
}

fn rollover_reward_reserve_epoch(reward_world: &mut RuntimeWorld, epoch_index: u64) {
    let current = reward_world.protocol_power_reserve().clone();
    if current.epoch_index == epoch_index {
        return;
    }
    reward_world.set_protocol_power_reserve(ProtocolPowerReserve {
        epoch_index,
        available_power_units: current.available_power_units,
        redeemed_power_units: 0,
    });
}

fn apply_reward_settlement_envelope(
    reward_world: &mut RuntimeWorld,
    envelope: &RewardSettlementEnvelope,
) -> Result<(), String> {
    verify_reward_settlement_envelope(envelope)?;
    verify_settlement_envelope_signer_binding(reward_world, envelope)?;
    reward_world.submit_action(RuntimeAction::ApplyNodePointsSettlementSigned {
        report: envelope.report.clone(),
        signer_node_id: envelope.signer_node_id.clone(),
        mint_records: envelope.mint_records.clone(),
    });
    reward_world.step().map_err(|err| format!("{:?}", err))
}

fn verify_settlement_envelope_signer_binding(
    reward_world: &RuntimeWorld,
    envelope: &RewardSettlementEnvelope,
) -> Result<(), String> {
    let bound_public_key = reward_world
        .node_identity_public_key(envelope.signer_node_id.as_str())
        .ok_or_else(|| {
            format!(
                "settlement signer identity is not bound: {}",
                envelope.signer_node_id
            )
        })?;
    if bound_public_key != envelope.signer_public_key_hex {
        return Err(format!(
            "settlement signer public key mismatch: signer_node_id={} bound={} envelope={}",
            envelope.signer_node_id, bound_public_key, envelope.signer_public_key_hex
        ));
    }
    Ok(())
}

fn reward_invariant_status_payload(report: &RewardAssetInvariantReport) -> serde_json::Value {
    serde_json::json!({
        "ok": report.is_ok(),
        "violation_count": report.violations.len(),
    })
}

fn load_reward_runtime_collector_snapshot(
    path: &Path,
) -> Result<Option<NodePointsRuntimeCollectorSnapshot>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(path)
        .map_err(|err| format!("read collector state {} failed: {}", path.display(), err))?;
    let snapshot: NodePointsRuntimeCollectorSnapshot = serde_json::from_slice(bytes.as_slice())
        .map_err(|err| format!("parse collector state {} failed: {}", path.display(), err))?;
    Ok(Some(snapshot))
}

fn persist_reward_runtime_collector_state(
    path: &Path,
    collector: &NodePointsRuntimeCollector,
) -> Result<(), String> {
    let snapshot = collector.snapshot();
    let bytes = serde_json::to_vec_pretty(&snapshot)
        .map_err(|err| format!("serialize collector state failed: {}", err))?;
    write_bytes_atomic(path, bytes.as_slice())
}

fn write_bytes_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("create state dir {} failed: {}", parent.display(), err))?;
        }
    }
    let temp_path = path.with_extension("json.tmp");
    fs::write(&temp_path, bytes)
        .map_err(|err| format!("write state temp {} failed: {}", temp_path.display(), err))?;
    fs::rename(&temp_path, path).map_err(|err| {
        format!(
            "rename state temp {} -> {} failed: {}",
            temp_path.display(),
            path.display(),
            err
        )
    })
}

fn now_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
#[path = "world_viewer_live_tests.rs"]
mod world_viewer_live_tests;
