use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::process;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use agent_world_distfs::StorageChallengeProbeCursorState;
use agent_world::geometry::GeoPos;
use agent_world::runtime::{
    measure_directory_storage_bytes, reward_redeem_signature_v1, Action as RuntimeAction,
    NodePointsConfig, NodePointsRuntimeCollector, NodePointsRuntimeCollectorSnapshot,
    NodePointsRuntimeHeuristics, NodePointsRuntimeObservation, ProtocolPowerReserve,
    RewardAssetConfig, RewardAssetInvariantReport, RewardSignatureGovernancePolicy,
    World as RuntimeWorld,
};
use agent_world::simulator::WorldScenario;
use agent_world::viewer::{
    ViewerLiveDecisionMode, ViewerLiveServer, ViewerLiveServerConfig, ViewerWebBridge,
    ViewerWebBridgeConfig,
};
use agent_world_node::{
    Libp2pReplicationNetwork, Libp2pReplicationNetworkConfig, NodeConfig, NodeReplicationConfig,
    NodeReplicationNetworkHandle, NodeRole, NodeRuntime, PosValidator,
};
use ed25519_dalek::SigningKey;
use rand_core::OsRng;
#[path = "world_viewer_live/distfs_probe_runtime.rs"]
mod distfs_probe_runtime;
use distfs_probe_runtime::{
    collect_distfs_challenge_report_with_config, load_reward_runtime_distfs_probe_state,
    parse_distfs_probe_runtime_option, persist_reward_runtime_distfs_probe_state,
    DistfsProbeRuntimeConfig,
};
#[cfg(test)]
use distfs_probe_runtime::collect_distfs_challenge_report;

const DEFAULT_CONFIG_FILE_NAME: &str = "config.toml";
const NODE_TABLE_KEY: &str = "node";
const NODE_PRIVATE_KEY_FIELD: &str = "private_key";
const NODE_PUBLIC_KEY_FIELD: &str = "public_key";
const DEFAULT_REWARD_RUNTIME_REPORT_DIR: &str = "output/node-reward-runtime";
const DEFAULT_REWARD_RUNTIME_STATE_FILE: &str = "reward-runtime-state.json";
const DEFAULT_REWARD_RUNTIME_DISTFS_PROBE_STATE_FILE: &str = "reward-runtime-distfs-probe-state.json";
const DEFAULT_REWARD_RUNTIME_RESERVE_UNITS: i64 = 100_000;

#[derive(Debug, Clone, PartialEq)]
struct CliOptions {
    scenario: WorldScenario,
    bind_addr: String,
    web_bind_addr: Option<String>,
    tick_ms: u64,
    llm_mode: bool,
    node_enabled: bool,
    node_id: String,
    node_role: NodeRole,
    node_tick_ms: u64,
    node_auto_attest_all_validators: bool,
    node_validators: Vec<PosValidator>,
    node_gossip_bind: Option<SocketAddr>,
    node_gossip_peers: Vec<SocketAddr>,
    node_repl_libp2p_listen: Vec<String>,
    node_repl_libp2p_peers: Vec<String>,
    node_repl_topic: Option<String>,
    reward_runtime_enabled: bool,
    reward_runtime_auto_redeem: bool,
    reward_runtime_signer_node_id: Option<String>,
    reward_runtime_report_dir: String,
    reward_points_per_credit: u64,
    reward_credits_per_power_unit: u64,
    reward_max_redeem_power_per_epoch: i64,
    reward_min_redeem_power_unit: i64,
    reward_initial_reserve_power_units: i64,
    reward_distfs_probe_config: DistfsProbeRuntimeConfig,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            scenario: WorldScenario::TwinRegionBootstrap,
            bind_addr: "127.0.0.1:5010".to_string(),
            web_bind_addr: None,
            tick_ms: 200,
            llm_mode: false,
            node_enabled: true,
            node_id: "viewer-live-node".to_string(),
            node_role: NodeRole::Observer,
            node_tick_ms: 200,
            node_auto_attest_all_validators: true,
            node_validators: Vec::new(),
            node_gossip_bind: None,
            node_gossip_peers: Vec::new(),
            node_repl_libp2p_listen: Vec::new(),
            node_repl_libp2p_peers: Vec::new(),
            node_repl_topic: None,
            reward_runtime_enabled: false,
            reward_runtime_auto_redeem: false,
            reward_runtime_signer_node_id: None,
            reward_runtime_report_dir: DEFAULT_REWARD_RUNTIME_REPORT_DIR.to_string(),
            reward_points_per_credit: RewardAssetConfig::default().points_per_credit,
            reward_credits_per_power_unit: RewardAssetConfig::default().credits_per_power_unit,
            reward_max_redeem_power_per_epoch: RewardAssetConfig::default()
                .max_redeem_power_per_epoch,
            reward_min_redeem_power_unit: RewardAssetConfig::default().min_redeem_power_unit,
            reward_initial_reserve_power_units: DEFAULT_REWARD_RUNTIME_RESERVE_UNITS,
            reward_distfs_probe_config: DistfsProbeRuntimeConfig::default(),
        }
    }
}

#[derive(Debug, Clone)]
struct RewardRuntimeLoopConfig {
    poll_interval: Duration,
    signer_node_id: String,
    signer_private_key_hex: String,
    signer_public_key_hex: String,
    report_dir: String,
    state_path: std::path::PathBuf,
    distfs_probe_state_path: std::path::PathBuf,
    storage_root: std::path::PathBuf,
    distfs_probe_config: DistfsProbeRuntimeConfig,
    auto_redeem: bool,
    reward_asset_config: RewardAssetConfig,
    initial_reserve_power_units: i64,
}

#[derive(Debug)]
struct RewardRuntimeWorker {
    stop_tx: mpsc::Sender<()>,
    join_handle: thread::JoinHandle<()>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let options = match parse_options(args.iter().skip(1).map(|arg| arg.as_str())) {
        Ok(options) => options,
        Err(err) => {
            eprintln!("{err}");
            print_help();
            process::exit(1);
        }
    };

    let node_runtime = match start_live_node(&options) {
        Ok(runtime) => runtime,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };
    let mut reward_runtime_worker =
        match start_reward_runtime_worker(&options, node_runtime.clone()) {
            Ok(worker) => worker,
            Err(err) => {
                eprintln!("{err}");
                stop_live_node(node_runtime.as_ref());
                process::exit(1);
            }
        };

    if let Some(web_bind_addr) = options.web_bind_addr.clone() {
        let upstream_addr = options.bind_addr.clone();
        thread::spawn(move || {
            let bridge = ViewerWebBridge::new(ViewerWebBridgeConfig::new(
                web_bind_addr.clone(),
                upstream_addr,
            ));
            if let Err(err) = bridge.run() {
                eprintln!("viewer web bridge failed on {}: {err:?}", web_bind_addr);
            }
        });
    }

    let config = ViewerLiveServerConfig::new(options.scenario)
        .with_bind_addr(options.bind_addr)
        .with_tick_interval(Duration::from_millis(options.tick_ms))
        .with_decision_mode(if options.llm_mode {
            ViewerLiveDecisionMode::Llm
        } else {
            ViewerLiveDecisionMode::Script
        });

    let mut server = match ViewerLiveServer::new(config) {
        Ok(server) => server,
        Err(err) => {
            eprintln!("failed to start live viewer server: {err:?}");
            stop_reward_runtime_worker(reward_runtime_worker.take());
            stop_live_node(node_runtime.as_ref());
            process::exit(1);
        }
    };

    let run_result = server.run();
    stop_reward_runtime_worker(reward_runtime_worker.take());
    stop_live_node(node_runtime.as_ref());

    if let Err(err) = run_result {
        eprintln!("live viewer server failed: {err:?}");
        process::exit(1);
    }
}

fn start_live_node(options: &CliOptions) -> Result<Option<Arc<Mutex<NodeRuntime>>>, String> {
    if !options.node_enabled {
        return Ok(None);
    }

    let keypair = ensure_node_keypair_in_config(Path::new(DEFAULT_CONFIG_FILE_NAME))
        .map_err(|err| format!("failed to ensure node keypair in config.toml: {err}"))?;

    let world_id = format!("live-{}", options.scenario.as_str());
    let mut config = NodeConfig::new(options.node_id.clone(), world_id, options.node_role)
        .and_then(|config| config.with_tick_interval(Duration::from_millis(options.node_tick_ms)))
        .map_err(|err| format!("failed to build node config: {err:?}"))?;
    if !options.node_validators.is_empty() {
        config = config
            .with_pos_validators(options.node_validators.clone())
            .map_err(|err| format!("failed to apply node validators: {err:?}"))?;
    }
    config = config.with_auto_attest_all_validators(options.node_auto_attest_all_validators);
    if !options.node_gossip_peers.is_empty() && options.node_gossip_bind.is_none() {
        return Err("node gossip peers require --node-gossip-bind".to_string());
    }
    if let Some(bind_addr) = options.node_gossip_bind {
        config = config.with_gossip_optional(bind_addr, options.node_gossip_peers.clone());
    }
    let replication_root = Path::new("output")
        .join("node-distfs")
        .join(options.node_id.as_str());
    let replication = NodeReplicationConfig::new(replication_root)
        .and_then(|cfg| {
            cfg.with_signing_keypair(
                keypair.private_key_hex.clone(),
                keypair.public_key_hex.clone(),
            )
        })
        .map_err(|err| format!("failed to build node replication config: {err:?}"))?;
    config = config.with_replication(replication);

    let mut runtime = NodeRuntime::new(config);
    if !options.node_repl_libp2p_listen.is_empty() || !options.node_repl_libp2p_peers.is_empty() {
        let mut net_config = Libp2pReplicationNetworkConfig::default();
        for raw in &options.node_repl_libp2p_listen {
            let addr = raw.parse().map_err(|err| {
                format!("--node-repl-libp2p-listen invalid multiaddr `{raw}`: {err}")
            })?;
            net_config.listen_addrs.push(addr);
        }
        for raw in &options.node_repl_libp2p_peers {
            let addr = raw.parse().map_err(|err| {
                format!("--node-repl-libp2p-peer invalid multiaddr `{raw}`: {err}")
            })?;
            net_config.bootstrap_peers.push(addr);
        }

        let network = Arc::new(Libp2pReplicationNetwork::new(net_config));
        let mut handle = NodeReplicationNetworkHandle::new(network);
        if let Some(topic) = options.node_repl_topic.as_deref() {
            handle = handle
                .with_topic(topic)
                .map_err(|err| format!("failed to configure replication topic: {err}"))?;
        }
        runtime = runtime.with_replication_network(handle);
    }
    runtime
        .start()
        .map_err(|err| format!("failed to start node runtime: {err:?}"))?;
    Ok(Some(Arc::new(Mutex::new(runtime))))
}

fn stop_live_node(node_runtime: Option<&Arc<Mutex<NodeRuntime>>>) {
    let Some(runtime) = node_runtime else {
        return;
    };
    let mut locked = match runtime.lock() {
        Ok(locked) => locked,
        Err(_) => {
            eprintln!("failed to stop node runtime: lock poisoned");
            return;
        }
    };
    if let Err(stop_err) = locked.stop() {
        eprintln!("failed to stop node runtime: {stop_err:?}");
    }
}

fn start_reward_runtime_worker(
    options: &CliOptions,
    node_runtime: Option<Arc<Mutex<NodeRuntime>>>,
) -> Result<Option<RewardRuntimeWorker>, String> {
    if !options.reward_runtime_enabled {
        return Ok(None);
    }
    let runtime = node_runtime.ok_or_else(|| {
        "reward runtime requires embedded node runtime; disable --no-node or reward runtime"
            .to_string()
    })?;

    let signer_node_id = options
        .reward_runtime_signer_node_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| options.node_id.clone());
    if signer_node_id.trim().is_empty() {
        return Err("reward runtime signer node id cannot be empty".to_string());
    }

    let report_dir = options.reward_runtime_report_dir.trim().to_string();
    if report_dir.is_empty() {
        return Err("reward runtime report dir cannot be empty".to_string());
    }
    let signer_keypair = ensure_node_keypair_in_config(Path::new(DEFAULT_CONFIG_FILE_NAME))
        .map_err(|err| format!("failed to load reward runtime signer keypair: {err}"))?;

    let config = RewardRuntimeLoopConfig {
        poll_interval: Duration::from_millis(options.tick_ms),
        signer_node_id,
        signer_private_key_hex: signer_keypair.private_key_hex,
        signer_public_key_hex: signer_keypair.public_key_hex,
        report_dir,
        state_path: Path::new(options.reward_runtime_report_dir.as_str())
            .join(DEFAULT_REWARD_RUNTIME_STATE_FILE),
        distfs_probe_state_path: Path::new(options.reward_runtime_report_dir.as_str())
            .join(DEFAULT_REWARD_RUNTIME_DISTFS_PROBE_STATE_FILE),
        storage_root: Path::new("output")
            .join("node-distfs")
            .join(options.node_id.as_str())
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

    let mut collector = match load_reward_runtime_collector_state(config.state_path.as_path()) {
        Ok(Some(restored)) => restored,
        Ok(None) => NodePointsRuntimeCollector::new(
            NodePointsConfig::default(),
            NodePointsRuntimeHeuristics::default(),
        ),
        Err(err) => {
            eprintln!("reward runtime load collector state failed: {err}");
            NodePointsRuntimeCollector::new(
                NodePointsConfig::default(),
                NodePointsRuntimeHeuristics::default(),
            )
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
    let _ = reward_world.bind_node_identity(
        config.signer_node_id.as_str(),
        config.signer_public_key_hex.as_str(),
    );

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

        let mut observation = NodePointsRuntimeObservation::from_snapshot(
            &snapshot,
            effective_storage_bytes,
            observed_at_unix_ms,
        );
        let distfs_challenge_report = if snapshot.role == NodeRole::Storage {
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
                    Some(report)
                }
                Err(err) => {
                    eprintln!("reward runtime distfs probe failed: {err}");
                    None
                }
            }
        } else {
            None
        };

        let maybe_report = collector.observe(observation);
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

        rollover_reward_reserve_epoch(&mut reward_world, report.epoch_index);
        let minted_records = match reward_world.apply_node_points_settlement_mint_v2(
            &report,
            config.signer_node_id.as_str(),
            config.signer_private_key_hex.as_str(),
        ) {
            Ok(records) => records,
            Err(err) => {
                eprintln!("reward runtime settlement mint failed: {err:?}");
                continue;
            }
        };

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
                    "network_committed_height": snapshot.consensus.network_committed_height,
                    "known_peer_heads": snapshot.consensus.known_peer_heads,
                    "last_status": snapshot.consensus.last_status.map(|status| format!("{status:?}")),
                    "last_block_hash": snapshot.consensus.last_block_hash,
                }
            },
            "distfs_challenge_report": distfs_challenge_report,
            "distfs_probe_config": serde_json::to_value(config.distfs_probe_config).unwrap_or(serde_json::Value::Null),
            "distfs_probe_cursor_state": serde_json::to_value(distfs_probe_state.clone()).unwrap_or(serde_json::Value::Null),
            "settlement_report": report,
            "minted_records": minted_records,
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

fn auto_redeem_runtime_rewards(
    reward_world: &mut RuntimeWorld,
    minted_records: &[agent_world::runtime::NodeRewardMintRecord],
    signer_node_id: &str,
    signer_private_key_hex: &str,
) {
    let signer_public_key = match reward_world.node_identity_public_key(signer_node_id) {
        Some(key) => key.to_string(),
        None => {
            eprintln!(
                "reward runtime auto-redeem skipped: signer identity not bound: {}",
                signer_node_id
            );
            return;
        }
    };

    for record in minted_records {
        let node_id = record.node_id.as_str();
        if !reward_world.state().agents.contains_key(node_id) {
            reward_world.submit_action(RuntimeAction::RegisterAgent {
                agent_id: node_id.to_string(),
                pos: GeoPos::new(0.0, 0.0, 0.0),
            });
            if let Err(err) = reward_world.step() {
                eprintln!("reward runtime register auto-redeem agent failed: {err:?}");
                continue;
            }
        }

        let redeem_credits = reward_world.node_power_credit_balance(node_id);
        if redeem_credits == 0 {
            continue;
        }
        let nonce = reward_world
            .node_last_redeem_nonce(node_id)
            .unwrap_or(0)
            .saturating_add(1);
        let signature = match reward_redeem_signature_v1(
            node_id,
            node_id,
            redeem_credits,
            nonce,
            signer_node_id,
            signer_public_key.as_str(),
            signer_private_key_hex,
        ) {
            Ok(signature) => signature,
            Err(err) => {
                eprintln!(
                    "reward runtime auto-redeem skipped for {}: sign failed: {}",
                    node_id, err
                );
                continue;
            }
        };
        reward_world.submit_action(RuntimeAction::RedeemPowerSigned {
            node_id: node_id.to_string(),
            target_agent_id: node_id.to_string(),
            redeem_credits,
            nonce,
            signer_node_id: signer_node_id.to_string(),
            signature,
        });
        if let Err(err) = reward_world.step() {
            eprintln!("reward runtime auto-redeem failed: {err:?}");
        }
    }
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

fn reward_invariant_status_payload(report: &RewardAssetInvariantReport) -> serde_json::Value {
    serde_json::json!({
        "ok": report.is_ok(),
        "violation_count": report.violations.len(),
    })
}

fn load_reward_runtime_collector_state(
    path: &Path,
) -> Result<Option<NodePointsRuntimeCollector>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(path)
        .map_err(|err| format!("read collector state {} failed: {}", path.display(), err))?;
    let snapshot: NodePointsRuntimeCollectorSnapshot = serde_json::from_slice(bytes.as_slice())
        .map_err(|err| format!("parse collector state {} failed: {}", path.display(), err))?;
    Ok(Some(NodePointsRuntimeCollector::from_snapshot(snapshot)))
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

fn parse_options<'a>(args: impl Iterator<Item = &'a str>) -> Result<CliOptions, String> {
    let mut options = CliOptions::default();
    let mut scenario_arg: Option<&str> = None;
    let mut iter = args.peekable();

    while let Some(arg) = iter.next() {
        match arg {
            "--help" | "-h" => {
                print_help();
                process::exit(0);
            }
            "--bind" => {
                options.bind_addr = iter
                    .next()
                    .ok_or_else(|| "--bind requires an address".to_string())?
                    .to_string();
            }
            "--tick-ms" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| "--tick-ms requires a positive integer".to_string())?;
                options.tick_ms = raw
                    .parse::<u64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or_else(|| "--tick-ms requires a positive integer".to_string())?;
            }
            "--web-bind" => {
                options.web_bind_addr = Some(
                    iter.next()
                        .ok_or_else(|| "--web-bind requires an address".to_string())?
                        .to_string(),
                );
            }
            "--scenario" => {
                scenario_arg = Some(
                    iter.next()
                        .ok_or_else(|| "--scenario requires a name".to_string())?,
                );
            }
            "--llm" => {
                options.llm_mode = true;
            }
            "--no-node" => {
                options.node_enabled = false;
            }
            "--node-id" => {
                options.node_id = iter
                    .next()
                    .ok_or_else(|| "--node-id requires a value".to_string())?
                    .to_string();
            }
            "--node-role" => {
                let role = iter
                    .next()
                    .ok_or_else(|| "--node-role requires a value".to_string())?;
                options.node_role = role.parse::<NodeRole>().map_err(|_| {
                    "--node-role must be one of: sequencer, storage, observer".to_string()
                })?;
            }
            "--node-tick-ms" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| "--node-tick-ms requires a positive integer".to_string())?;
                options.node_tick_ms = raw
                    .parse::<u64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or_else(|| "--node-tick-ms requires a positive integer".to_string())?;
            }
            "--node-validator" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| "--node-validator requires <validator_id:stake>".to_string())?;
                options.node_validators.push(parse_validator_spec(raw)?);
            }
            "--node-no-auto-attest-all" => {
                options.node_auto_attest_all_validators = false;
            }
            "--node-gossip-bind" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| "--node-gossip-bind requires <addr:port>".to_string())?;
                options.node_gossip_bind = Some(parse_socket_addr(raw, "--node-gossip-bind")?);
            }
            "--node-gossip-peer" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| "--node-gossip-peer requires <addr:port>".to_string())?;
                options
                    .node_gossip_peers
                    .push(parse_socket_addr(raw, "--node-gossip-peer")?);
            }
            "--node-repl-libp2p-listen" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| "--node-repl-libp2p-listen requires <multiaddr>".to_string())?;
                options.node_repl_libp2p_listen.push(raw.to_string());
            }
            "--node-repl-libp2p-peer" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| "--node-repl-libp2p-peer requires <multiaddr>".to_string())?;
                options.node_repl_libp2p_peers.push(raw.to_string());
            }
            "--node-repl-topic" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| "--node-repl-topic requires <topic>".to_string())?;
                let topic = raw.trim();
                if topic.is_empty() {
                    return Err("--node-repl-topic requires non-empty value".to_string());
                }
                options.node_repl_topic = Some(topic.to_string());
            }
            "--reward-runtime-enable" => {
                options.reward_runtime_enabled = true;
            }
            "--reward-runtime-auto-redeem" => {
                options.reward_runtime_auto_redeem = true;
            }
            "--reward-runtime-signer" => {
                let signer = iter
                    .next()
                    .ok_or_else(|| "--reward-runtime-signer requires <node_id>".to_string())?;
                let signer = signer.trim();
                if signer.is_empty() {
                    return Err("--reward-runtime-signer requires non-empty <node_id>".to_string());
                }
                options.reward_runtime_signer_node_id = Some(signer.to_string());
            }
            "--reward-runtime-report-dir" => {
                let dir = iter
                    .next()
                    .ok_or_else(|| "--reward-runtime-report-dir requires <path>".to_string())?;
                let dir = dir.trim();
                if dir.is_empty() {
                    return Err("--reward-runtime-report-dir requires non-empty <path>".to_string());
                }
                options.reward_runtime_report_dir = dir.to_string();
            }
            "--reward-points-per-credit" => {
                let raw = iter.next().ok_or_else(|| {
                    "--reward-points-per-credit requires a positive integer".to_string()
                })?;
                options.reward_points_per_credit = raw
                    .parse::<u64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or_else(|| {
                        "--reward-points-per-credit requires a positive integer".to_string()
                    })?;
            }
            "--reward-credits-per-power-unit" => {
                let raw = iter.next().ok_or_else(|| {
                    "--reward-credits-per-power-unit requires a positive integer".to_string()
                })?;
                options.reward_credits_per_power_unit = raw
                    .parse::<u64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or_else(|| {
                        "--reward-credits-per-power-unit requires a positive integer".to_string()
                    })?;
            }
            "--reward-max-redeem-power-per-epoch" => {
                let raw = iter.next().ok_or_else(|| {
                    "--reward-max-redeem-power-per-epoch requires a positive integer".to_string()
                })?;
                options.reward_max_redeem_power_per_epoch = raw
                    .parse::<i64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or_else(|| {
                        "--reward-max-redeem-power-per-epoch requires a positive integer"
                            .to_string()
                    })?;
            }
            "--reward-min-redeem-power-unit" => {
                let raw = iter.next().ok_or_else(|| {
                    "--reward-min-redeem-power-unit requires a positive integer".to_string()
                })?;
                options.reward_min_redeem_power_unit = raw
                    .parse::<i64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or_else(|| {
                        "--reward-min-redeem-power-unit requires a positive integer".to_string()
                    })?;
            }
            "--reward-initial-reserve-power-units" => {
                let raw = iter.next().ok_or_else(|| {
                    "--reward-initial-reserve-power-units requires a non-negative integer"
                        .to_string()
                })?;
                options.reward_initial_reserve_power_units = raw
                    .parse::<i64>()
                    .ok()
                    .filter(|value| *value >= 0)
                    .ok_or_else(|| {
                        "--reward-initial-reserve-power-units requires a non-negative integer"
                            .to_string()
                    })?;
            }
            _ => {
                if parse_distfs_probe_runtime_option(
                    arg,
                    &mut iter,
                    &mut options.reward_distfs_probe_config,
                )? {
                    continue;
                }
                if scenario_arg.is_none() {
                    scenario_arg = Some(arg);
                } else {
                    return Err(format!("unexpected argument: {arg}"));
                }
            }
        }
    }

    if let Some(name) = scenario_arg {
        options.scenario = WorldScenario::parse(name).ok_or_else(|| {
            format!(
                "Unknown scenario: {name}. available: {}",
                WorldScenario::variants().join(", ")
            )
        })?;
    }
    if options.node_repl_topic.is_some()
        && options.node_repl_libp2p_listen.is_empty()
        && options.node_repl_libp2p_peers.is_empty()
    {
        return Err(
            "--node-repl-topic requires --node-repl-libp2p-listen or --node-repl-libp2p-peer"
                .to_string(),
        );
    }
    if options
        .reward_distfs_probe_config
        .adaptive_policy
        .failure_backoff_max_ms
        < options
            .reward_distfs_probe_config
            .adaptive_policy
            .failure_backoff_base_ms
    {
        return Err(
            "--reward-distfs-adaptive-backoff-max-ms must be >= --reward-distfs-adaptive-backoff-base-ms"
                .to_string(),
        );
    }
    if options.reward_runtime_enabled && !options.node_enabled {
        return Err(
            "--reward-runtime-enable requires embedded node runtime (remove --no-node)".to_string(),
        );
    }

    Ok(options)
}

fn print_help() {
    println!(
        "Usage: world_viewer_live [scenario] [--bind <addr>] [--web-bind <addr>] [--tick-ms <ms>] [--llm] [--no-node] [--node-validator <id:stake>...] [--node-gossip-bind <addr:port>] [--node-gossip-peer <addr:port>...]"
    );
    println!("Options:");
    println!("  --bind <addr>     Bind address (default: 127.0.0.1:5010)");
    println!("  --web-bind <addr> WebSocket bridge bind address (optional)");
    println!("  --tick-ms <ms>    Tick interval in milliseconds (default: 200)");
    println!("  --scenario <name> Scenario name (default: twin_region_bootstrap)");
    println!("  --llm             Use LLM decisions instead of built-in script");
    println!("  --no-node         Disable embedded node runtime startup");
    println!("  --node-id <id>    Node identifier (default: viewer-live-node)");
    println!("  --node-role <r>   Node role: sequencer|storage|observer (default: observer)");
    println!("  --node-tick-ms <ms> Node runtime tick interval (default: 200)");
    println!("  --node-validator <id:stake> Add PoS validator stake (repeatable)");
    println!("  --node-no-auto-attest-all Disable auto-attesting all validators per tick");
    println!("  --node-gossip-bind <addr:port> Bind UDP endpoint for node gossip");
    println!("  --node-gossip-peer <addr:port> Add UDP peer endpoint for node gossip");
    println!(
        "  --node-repl-libp2p-listen <multiaddr> Add libp2p listen addr for replication network"
    );
    println!(
        "  --node-repl-libp2p-peer <multiaddr> Add libp2p bootstrap peer for replication network"
    );
    println!("  --node-repl-topic <topic> Override replication pubsub topic when libp2p replication is enabled");
    println!("  --reward-runtime-enable Enable reward runtime settlement loop (default: off)");
    println!("  --reward-runtime-auto-redeem Auto redeem minted credits to node-mapped runtime agent");
    println!("  --reward-runtime-signer <node_id> Settlement signer node id (default: --node-id)");
    println!("  --reward-runtime-report-dir <path> Reward runtime report directory (default: output/node-reward-runtime)");
    println!(
        "  --reward-distfs-probe-max-sample-bytes <n> DistFS probe sample size upper bound bytes (default: 65536)"
    );
    println!(
        "  --reward-distfs-probe-per-tick <n> DistFS challenge count per reward tick (default: 1)"
    );
    println!(
        "  --reward-distfs-probe-ttl-ms <n> DistFS challenge ttl milliseconds (default: 30000)"
    );
    println!(
        "  --reward-distfs-probe-allowed-clock-skew-ms <n> DistFS challenge allowed clock skew milliseconds (default: 5000)"
    );
    println!(
        "  --reward-distfs-adaptive-max-checks-per-round <n> DistFS adaptive per-round check cap (default: u32::MAX)"
    );
    println!(
        "  --reward-distfs-adaptive-backoff-base-ms <n> DistFS adaptive backoff base milliseconds (default: 0)"
    );
    println!(
        "  --reward-distfs-adaptive-backoff-max-ms <n> DistFS adaptive backoff max milliseconds (default: 0)"
    );
    println!("  --reward-points-per-credit <n> points -> credit conversion ratio");
    println!("  --reward-credits-per-power-unit <n> credit -> power conversion ratio");
    println!("  --reward-max-redeem-power-per-epoch <n> per-epoch redeem power cap");
    println!("  --reward-min-redeem-power-unit <n> minimum redeem power unit");
    println!("  --reward-initial-reserve-power-units <n> initial protocol power reserve");
    println!(
        "Available scenarios: {}",
        WorldScenario::variants().join(", ")
    );
}

fn parse_validator_spec(raw: &str) -> Result<PosValidator, String> {
    let (validator_id_raw, stake_raw) = raw
        .split_once(':')
        .ok_or_else(|| "--node-validator requires <validator_id:stake>".to_string())?;
    let validator_id = validator_id_raw.trim();
    if validator_id.is_empty() {
        return Err("--node-validator validator_id cannot be empty".to_string());
    }
    let stake = stake_raw.trim().parse::<u64>().ok().filter(|value| *value > 0).ok_or_else(
        || "--node-validator stake must be a positive integer".to_string(),
    )?;
    Ok(PosValidator { validator_id: validator_id.to_string(), stake })
}

fn parse_socket_addr(raw: &str, flag: &str) -> Result<SocketAddr, String> {
    raw.parse::<SocketAddr>().map_err(|_| format!("{flag} requires a valid <addr:port>, got: {raw}"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NodeKeypairConfig {
    private_key_hex: String,
    public_key_hex: String,
}

fn ensure_node_keypair_in_config(path: &Path) -> Result<NodeKeypairConfig, String> {
    let mut table = load_config_table(path)?;
    let mut wrote = false;

    let node_table = table
        .entry(NODE_TABLE_KEY.to_string())
        .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    let node_table = node_table
        .as_table_mut()
        .ok_or_else(|| "config field 'node' must be a table".to_string())?;

    let existing_private = node_table
        .get(NODE_PRIVATE_KEY_FIELD)
        .and_then(toml::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let existing_public = node_table
        .get(NODE_PUBLIC_KEY_FIELD)
        .and_then(toml::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);

    let keypair = match (existing_private, existing_public) {
        (Some(private_hex), Some(public_hex)) => {
            validate_node_keypair_hex(private_hex.as_str(), public_hex.as_str())?;
            NodeKeypairConfig {
                private_key_hex: private_hex,
                public_key_hex: public_hex,
            }
        }
        (Some(private_hex), None) => {
            let signing_key = signing_key_from_hex(private_hex.as_str())?;
            let public_key_hex = hex::encode(signing_key.verifying_key().to_bytes());
            node_table.insert(
                NODE_PUBLIC_KEY_FIELD.to_string(),
                toml::Value::String(public_key_hex.clone()),
            );
            wrote = true;
            NodeKeypairConfig {
                private_key_hex: private_hex,
                public_key_hex,
            }
        }
        _ => {
            let signing_key = SigningKey::generate(&mut OsRng);
            let private_key_hex = hex::encode(signing_key.to_bytes());
            let public_key_hex = hex::encode(signing_key.verifying_key().to_bytes());
            node_table.insert(
                NODE_PRIVATE_KEY_FIELD.to_string(),
                toml::Value::String(private_key_hex.clone()),
            );
            node_table.insert(
                NODE_PUBLIC_KEY_FIELD.to_string(),
                toml::Value::String(public_key_hex.clone()),
            );
            wrote = true;
            NodeKeypairConfig {
                private_key_hex,
                public_key_hex,
            }
        }
    };

    if wrote {
        write_config_table(path, &table)?;
    }
    Ok(keypair)
}

fn load_config_table(path: &Path) -> Result<toml::map::Map<String, toml::Value>, String> {
    if !path.exists() {
        return Ok(toml::map::Map::new());
    }

    let content = fs::read_to_string(path)
        .map_err(|err| format!("read {} failed: {}", path.display(), err))?;
    if content.trim().is_empty() {
        return Ok(toml::map::Map::new());
    }

    let value: toml::Value = toml::from_str(content.as_str())
        .map_err(|err| format!("parse {} failed: {}", path.display(), err))?;
    value
        .as_table()
        .cloned()
        .ok_or_else(|| format!("{} root must be a table", path.display()))
}

fn write_config_table(
    path: &Path,
    table: &toml::map::Map<String, toml::Value>,
) -> Result<(), String> {
    let content = toml::to_string_pretty(table)
        .map_err(|err| format!("serialize {} failed: {}", path.display(), err))?;
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| {
                format!(
                    "create config parent dir {} failed: {}",
                    parent.display(),
                    err
                )
            })?;
        }
    }
    fs::write(path, content).map_err(|err| format!("write {} failed: {}", path.display(), err))
}

fn validate_node_keypair_hex(private_key_hex: &str, public_key_hex: &str) -> Result<(), String> {
    let signing_key = signing_key_from_hex(private_key_hex)?;
    let expected_public_hex = hex::encode(signing_key.verifying_key().to_bytes());
    if expected_public_hex != public_key_hex {
        return Err("node.public_key does not match node.private_key".to_string());
    }
    Ok(())
}

fn signing_key_from_hex(private_key_hex: &str) -> Result<SigningKey, String> {
    let private_bytes = hex::decode(private_key_hex)
        .map_err(|_| "node.private_key must be valid hex".to_string())?;
    let private_array: [u8; 32] = private_bytes
        .try_into()
        .map_err(|_| "node.private_key must be 32-byte hex".to_string())?;
    Ok(SigningKey::from_bytes(&private_array))
}

#[cfg(test)]
#[path = "world_viewer_live/world_viewer_live_tests.rs"]
mod world_viewer_live_tests;
