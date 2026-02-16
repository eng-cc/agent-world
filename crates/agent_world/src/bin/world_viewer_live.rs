use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::process;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use agent_world::geometry::GeoPos;
use agent_world::runtime::{
    measure_directory_storage_bytes, Action as RuntimeAction, NodePointsConfig,
    NodePointsRuntimeCollector, NodePointsRuntimeHeuristics, ProtocolPowerReserve,
    RewardAssetConfig, World as RuntimeWorld,
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

const DEFAULT_CONFIG_FILE_NAME: &str = "config.toml";
const NODE_TABLE_KEY: &str = "node";
const NODE_PRIVATE_KEY_FIELD: &str = "private_key";
const NODE_PUBLIC_KEY_FIELD: &str = "public_key";
const DEFAULT_REWARD_RUNTIME_REPORT_DIR: &str = "output/node-reward-runtime";
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
        }
    }
}

#[derive(Debug, Clone)]
struct RewardRuntimeLoopConfig {
    poll_interval: Duration,
    signer_node_id: String,
    report_dir: String,
    storage_root: std::path::PathBuf,
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
    let mut reward_runtime_worker = match start_reward_runtime_worker(&options, node_runtime.clone())
    {
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

    let config = RewardRuntimeLoopConfig {
        poll_interval: Duration::from_millis(options.tick_ms),
        signer_node_id,
        report_dir,
        storage_root: Path::new("output")
            .join("node-distfs")
            .join(options.node_id.as_str())
            .join("store"),
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

    let mut collector =
        NodePointsRuntimeCollector::new(NodePointsConfig::default(), NodePointsRuntimeHeuristics::default());
    let mut reward_world = RuntimeWorld::new();
    reward_world.set_reward_asset_config(config.reward_asset_config.clone());
    reward_world.set_protocol_power_reserve(ProtocolPowerReserve {
        epoch_index: 0,
        available_power_units: config.initial_reserve_power_units.max(0),
        redeemed_power_units: 0,
    });

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
        let effective_storage_bytes = measure_directory_storage_bytes(config.storage_root.as_path());

        let Some(report) =
            collector.observe_snapshot(&snapshot, effective_storage_bytes, observed_at_unix_ms)
        else {
            continue;
        };

        rollover_reward_reserve_epoch(&mut reward_world, report.epoch_index);
        let minted_records =
            match reward_world.apply_node_points_settlement_mint(&report, config.signer_node_id.as_str()) {
                Ok(records) => records,
                Err(err) => {
                    eprintln!("reward runtime settlement mint failed: {err:?}");
                    continue;
                }
            };

        if config.auto_redeem {
            auto_redeem_runtime_rewards(&mut reward_world, minted_records.as_slice());
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
            "settlement_report": report,
            "minted_records": minted_records,
            "node_balances": reward_world.state().node_asset_balances,
            "reserve": reward_world.protocol_power_reserve(),
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
) {
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
        reward_world.submit_action(RuntimeAction::RedeemPower {
            node_id: node_id.to_string(),
            target_agent_id: node_id.to_string(),
            redeem_credits,
            nonce,
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
                let dir = iter.next().ok_or_else(|| {
                    "--reward-runtime-report-dir requires <path>".to_string()
                })?;
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
    if options.reward_runtime_enabled && !options.node_enabled {
        return Err(
            "--reward-runtime-enable requires embedded node runtime (remove --no-node)"
                .to_string(),
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
    println!(
        "  --reward-runtime-report-dir <path> Reward runtime report directory (default: output/node-reward-runtime)"
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
    let stake = stake_raw
        .trim()
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| "--node-validator stake must be a positive integer".to_string())?;
    Ok(PosValidator {
        validator_id: validator_id.to_string(),
        stake,
    })
}

fn parse_socket_addr(raw: &str, flag: &str) -> Result<SocketAddr, String> {
    raw.parse::<SocketAddr>()
        .map_err(|_| format!("{flag} requires a valid <addr:port>, got: {raw}"))
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
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_config_path(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("duration")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-world-{prefix}-{unique}.toml"))
    }

    #[test]
    fn parse_options_defaults() {
        let options = parse_options([].into_iter()).expect("defaults");
        assert_eq!(options.scenario, WorldScenario::TwinRegionBootstrap);
        assert_eq!(options.bind_addr, "127.0.0.1:5010");
        assert!(options.web_bind_addr.is_none());
        assert_eq!(options.tick_ms, 200);
        assert!(!options.llm_mode);
        assert!(options.node_enabled);
        assert_eq!(options.node_id, "viewer-live-node");
        assert_eq!(options.node_role, NodeRole::Observer);
        assert_eq!(options.node_tick_ms, 200);
        assert!(options.node_auto_attest_all_validators);
        assert!(options.node_validators.is_empty());
        assert!(options.node_gossip_bind.is_none());
        assert!(options.node_gossip_peers.is_empty());
        assert!(options.node_repl_libp2p_listen.is_empty());
        assert!(options.node_repl_libp2p_peers.is_empty());
        assert!(options.node_repl_topic.is_none());
        assert!(!options.reward_runtime_enabled);
        assert!(!options.reward_runtime_auto_redeem);
        assert!(options.reward_runtime_signer_node_id.is_none());
        assert_eq!(
            options.reward_runtime_report_dir,
            DEFAULT_REWARD_RUNTIME_REPORT_DIR
        );
        assert_eq!(
            options.reward_points_per_credit,
            RewardAssetConfig::default().points_per_credit
        );
        assert_eq!(
            options.reward_credits_per_power_unit,
            RewardAssetConfig::default().credits_per_power_unit
        );
        assert_eq!(
            options.reward_max_redeem_power_per_epoch,
            RewardAssetConfig::default().max_redeem_power_per_epoch
        );
        assert_eq!(
            options.reward_min_redeem_power_unit,
            RewardAssetConfig::default().min_redeem_power_unit
        );
        assert_eq!(
            options.reward_initial_reserve_power_units,
            DEFAULT_REWARD_RUNTIME_RESERVE_UNITS
        );
    }

    #[test]
    fn parse_options_enables_llm_mode() {
        let options = parse_options(["--llm"].into_iter()).expect("llm mode");
        assert!(options.llm_mode);
    }

    #[test]
    fn parse_options_reads_custom_values() {
        let options = parse_options(
            [
                "llm_bootstrap",
                "--bind",
                "127.0.0.1:9001",
                "--web-bind",
                "127.0.0.1:9002",
                "--tick-ms",
                "50",
                "--node-id",
                "viewer-live-1",
                "--node-role",
                "storage",
                "--node-tick-ms",
                "30",
                "--node-validator",
                "node-a:60",
                "--node-validator",
                "node-b:40",
                "--node-no-auto-attest-all",
                "--node-gossip-bind",
                "127.0.0.1:6001",
                "--node-gossip-peer",
                "127.0.0.1:6002",
                "--node-gossip-peer",
                "127.0.0.1:6003",
                "--node-repl-libp2p-listen",
                "/ip4/127.0.0.1/tcp/7001",
                "--node-repl-libp2p-peer",
                "/ip4/127.0.0.1/tcp/7002/p2p/12D3KooWR6f1fVQqfJ9WQnB8GL9QykgjM7RzQ2xZQW6hUGNfj9t7",
                "--node-repl-topic",
                "aw.custom.replication",
                "--reward-runtime-enable",
                "--reward-runtime-auto-redeem",
                "--reward-runtime-signer",
                "reward-signer-1",
                "--reward-runtime-report-dir",
                "output/reward-custom",
                "--reward-points-per-credit",
                "7",
                "--reward-credits-per-power-unit",
                "3",
                "--reward-max-redeem-power-per-epoch",
                "1200",
                "--reward-min-redeem-power-unit",
                "2",
                "--reward-initial-reserve-power-units",
                "888",
            ]
            .into_iter(),
        )
        .expect("custom");
        assert_eq!(options.scenario, WorldScenario::LlmBootstrap);
        assert_eq!(options.bind_addr, "127.0.0.1:9001");
        assert_eq!(options.web_bind_addr.as_deref(), Some("127.0.0.1:9002"));
        assert_eq!(options.tick_ms, 50);
        assert_eq!(options.node_id, "viewer-live-1");
        assert_eq!(options.node_role, NodeRole::Storage);
        assert_eq!(options.node_tick_ms, 30);
        assert!(!options.node_auto_attest_all_validators);
        assert_eq!(options.node_validators.len(), 2);
        assert_eq!(
            options.node_gossip_bind,
            Some("127.0.0.1:6001".parse::<SocketAddr>().expect("addr"))
        );
        assert_eq!(
            options.node_gossip_peers,
            vec![
                "127.0.0.1:6002".parse::<SocketAddr>().expect("addr"),
                "127.0.0.1:6003".parse::<SocketAddr>().expect("addr"),
            ]
        );
        assert_eq!(
            options.node_repl_libp2p_listen,
            vec!["/ip4/127.0.0.1/tcp/7001".to_string()]
        );
        assert_eq!(
            options.node_repl_libp2p_peers,
            vec![
                "/ip4/127.0.0.1/tcp/7002/p2p/12D3KooWR6f1fVQqfJ9WQnB8GL9QykgjM7RzQ2xZQW6hUGNfj9t7"
                    .to_string()
            ]
        );
        assert_eq!(
            options.node_repl_topic.as_deref(),
            Some("aw.custom.replication")
        );
        assert!(options.reward_runtime_enabled);
        assert!(options.reward_runtime_auto_redeem);
        assert_eq!(
            options.reward_runtime_signer_node_id.as_deref(),
            Some("reward-signer-1")
        );
        assert_eq!(options.reward_runtime_report_dir, "output/reward-custom");
        assert_eq!(options.reward_points_per_credit, 7);
        assert_eq!(options.reward_credits_per_power_unit, 3);
        assert_eq!(options.reward_max_redeem_power_per_epoch, 1200);
        assert_eq!(options.reward_min_redeem_power_unit, 2);
        assert_eq!(options.reward_initial_reserve_power_units, 888);
        assert_eq!(
            options.node_validators,
            vec![
                PosValidator {
                    validator_id: "node-a".to_string(),
                    stake: 60,
                },
                PosValidator {
                    validator_id: "node-b".to_string(),
                    stake: 40,
                }
            ]
        );
    }

    #[test]
    fn parse_options_rejects_zero_tick_ms() {
        let err = parse_options(["--tick-ms", "0"].into_iter()).expect_err("reject zero");
        assert!(err.contains("positive integer"));
    }

    #[test]
    fn parse_options_disables_node() {
        let options = parse_options(["--no-node"].into_iter()).expect("parse");
        assert!(!options.node_enabled);
    }

    #[test]
    fn parse_options_rejects_reward_runtime_with_no_node() {
        let err = parse_options(["--no-node", "--reward-runtime-enable"].into_iter())
            .expect_err("reward runtime requires node");
        assert!(err.contains("--reward-runtime-enable"));
    }

    #[test]
    fn parse_options_rejects_invalid_node_role() {
        let err =
            parse_options(["--node-role", "unknown"].into_iter()).expect_err("invalid node role");
        assert!(err.contains("--node-role"));
    }

    #[test]
    fn parse_options_rejects_invalid_node_validator_spec() {
        let err =
            parse_options(["--node-validator", "missing_stake"].into_iter()).expect_err("spec");
        assert!(err.contains("--node-validator"));
    }

    #[test]
    fn parse_options_rejects_invalid_node_gossip_addr() {
        let err =
            parse_options(["--node-gossip-bind", "invalid"].into_iter()).expect_err("invalid");
        assert!(err.contains("--node-gossip-bind"));
    }

    #[test]
    fn start_live_node_applies_pos_options() {
        let options = parse_options(
            [
                "--node-id",
                "node-main",
                "--node-tick-ms",
                "20",
                "--node-validator",
                "node-main:70",
                "--node-validator",
                "node-backup:30",
                "--node-no-auto-attest-all",
                "--node-gossip-bind",
                "127.0.0.1:6101",
                "--node-gossip-peer",
                "127.0.0.1:6102",
            ]
            .into_iter(),
        )
        .expect("options");

        let runtime = start_live_node(&options)
            .expect("start")
            .expect("runtime exists");
        let mut locked = runtime.lock().expect("lock runtime");
        let config = locked.config();
        assert_eq!(config.pos_config.validators.len(), 2);
        assert_eq!(config.pos_config.validators[0].validator_id, "node-main");
        assert_eq!(config.pos_config.validators[0].stake, 70);
        assert_eq!(config.pos_config.validators[1].validator_id, "node-backup");
        assert_eq!(config.pos_config.validators[1].stake, 30);
        assert!(!config.auto_attest_all_validators);
        let gossip = config.gossip.as_ref().expect("gossip config");
        assert_eq!(
            gossip.bind_addr,
            "127.0.0.1:6101".parse::<SocketAddr>().expect("addr")
        );
        assert_eq!(gossip.peers.len(), 1);
        assert_eq!(
            gossip.peers[0],
            "127.0.0.1:6102".parse::<SocketAddr>().expect("addr")
        );

        locked.stop().expect("stop");
    }

    #[test]
    fn start_live_node_rejects_gossip_peers_without_bind() {
        let options =
            parse_options(["--node-gossip-peer", "127.0.0.1:6202"].into_iter()).expect("options");
        let err = start_live_node(&options).expect_err("must fail");
        assert!(err.contains("--node-gossip-bind"));
    }

    #[test]
    fn parse_options_rejects_repl_topic_without_repl_network() {
        let err = parse_options(["--node-repl-topic", "aw.topic"].into_iter())
            .expect_err("repl topic should require network");
        assert!(err.contains("--node-repl-topic"));
    }

    #[test]
    fn start_live_node_supports_libp2p_replication_injection() {
        let options = parse_options(
            [
                "--node-repl-libp2p-listen",
                "/ip4/127.0.0.1/tcp/0",
                "--node-repl-topic",
                "aw.test.replication",
            ]
            .into_iter(),
        )
        .expect("options");

        let runtime = start_live_node(&options)
            .expect("start")
            .expect("runtime exists");
        runtime.lock().expect("lock runtime").stop().expect("stop");
    }

    #[test]
    fn ensure_node_keypair_in_config_creates_file_when_missing() {
        let path = temp_config_path("node-key-create");
        let keypair = ensure_node_keypair_in_config(&path).expect("ensure keypair");
        assert_eq!(keypair.private_key_hex.len(), 64);
        assert_eq!(keypair.public_key_hex.len(), 64);
        assert!(path.exists());

        let content = fs::read_to_string(&path).expect("read config");
        assert!(content.contains("[node]"));
        assert!(content.contains("private_key"));
        assert!(content.contains("public_key"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn ensure_node_keypair_in_config_preserves_existing_keypair() {
        let path = temp_config_path("node-key-preserve");
        let first = ensure_node_keypair_in_config(&path).expect("first ensure");
        let second = ensure_node_keypair_in_config(&path).expect("second ensure");
        assert_eq!(first, second);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn ensure_node_keypair_in_config_fills_missing_public_key() {
        let path = temp_config_path("node-key-fill-public");
        let generated = ensure_node_keypair_in_config(&path).expect("first ensure");

        let content = fs::read_to_string(&path).expect("read config");
        let mut value: toml::Value = toml::from_str(content.as_str()).expect("parse config");
        let node = value
            .as_table_mut()
            .and_then(|table| table.get_mut(NODE_TABLE_KEY))
            .and_then(toml::Value::as_table_mut)
            .expect("node table");
        node.remove(NODE_PUBLIC_KEY_FIELD);
        fs::write(&path, toml::to_string_pretty(&value).expect("serialize")).expect("write");

        let filled = ensure_node_keypair_in_config(&path).expect("fill public");
        assert_eq!(filled.private_key_hex, generated.private_key_hex);
        assert_eq!(filled.public_key_hex, generated.public_key_hex);
        let _ = fs::remove_file(path);
    }
}
