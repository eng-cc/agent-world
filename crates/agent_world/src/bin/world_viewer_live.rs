use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fmt;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use agent_world::runtime::{
    measure_directory_storage_bytes, Action as RuntimeAction, LocalCasStore, NodePointsConfig,
    NodePointsRuntimeCollector, NodePointsRuntimeCollectorSnapshot, NodePointsRuntimeHeuristics,
    NodePointsRuntimeObservation, ProtocolPowerReserve, RewardAssetConfig,
    RewardAssetInvariantReport, RewardSignatureGovernancePolicy, World as RuntimeWorld,
};
#[cfg(test)]
use agent_world::simulator::WorldScenario;
use agent_world::viewer::{
    ViewerLiveDecisionMode, ViewerLiveServer, ViewerLiveServerConfig, ViewerWebBridge,
    ViewerWebBridgeConfig,
};
use agent_world_distfs::StorageChallengeProbeCursorState;
use agent_world_node::{
    Libp2pReplicationNetwork, Libp2pReplicationNetworkConfig, NodeConfig, NodePosConfig,
    NodeReplicationConfig, NodeReplicationNetworkHandle, NodeRole, NodeRuntime, NodeSnapshot,
    PosConsensusStatus, PosValidator,
};
use agent_world_proto::distributed_net::DistributedNetwork as ProtoDistributedNetwork;
use agent_world_proto::world_error::WorldError as ProtoWorldError;
use agent_world_wasm_executor::{WasmExecutor, WasmExecutorConfig};
#[path = "world_viewer_live/cli.rs"]
mod cli;
#[path = "world_viewer_live/distfs_probe_runtime.rs"]
mod distfs_probe_runtime;
#[cfg(test)]
use distfs_probe_runtime::collect_distfs_challenge_report;
#[path = "world_viewer_live/distfs_challenge_network.rs"]
mod distfs_challenge_network;
#[path = "world_viewer_live/execution_bridge.rs"]
mod execution_bridge;
#[path = "world_viewer_live/node_keypair_config.rs"]
mod node_keypair_config;
#[path = "world_viewer_live/observation_trace_runtime.rs"]
mod observation_trace_runtime;
#[path = "world_viewer_live/reward_runtime_network.rs"]
mod reward_runtime_network;
#[path = "world_viewer_live/reward_runtime_settlement.rs"]
mod reward_runtime_settlement;
use cli::{
    parse_options, print_help, resolve_triad_distributed_gossip, CliOptions, NodeTopologyMode,
};
use distfs_challenge_network::{
    storage_proof_hint_value_from_semantics, DistfsChallengeNetworkDriver,
    DistfsChallengeNetworkTickReport,
};
use distfs_probe_runtime::{
    collect_distfs_challenge_report_with_config, load_reward_runtime_distfs_probe_state,
    parse_distfs_probe_runtime_option, persist_reward_runtime_distfs_probe_state,
    DistfsProbeRuntimeConfig,
};
use execution_bridge::{
    bridge_committed_heights, load_execution_bridge_state, load_execution_world,
    persist_execution_bridge_state, persist_execution_world, NodeRuntimeExecutionDriver,
};
use node_keypair_config::ensure_node_keypair_in_config;
use observation_trace_runtime::observe_reward_observation_trace;
use reward_runtime_network::{
    decode_reward_observation_trace, decode_reward_settlement_envelope,
    encode_reward_observation_trace, encode_reward_settlement_envelope, reward_observation_topic,
    reward_settlement_envelope_id, reward_settlement_topic, sign_reward_observation_trace,
    sign_reward_settlement_envelope, verify_reward_settlement_envelope, RewardObservationPayload,
    RewardSettlementEnvelope,
};
use reward_runtime_settlement::{
    auto_redeem_runtime_rewards, build_reward_settlement_mint_records,
};
use sha2::{Digest, Sha256};

const DEFAULT_CONFIG_FILE_NAME: &str = "config.toml";
#[cfg(test)]
const NODE_TABLE_KEY: &str = node_keypair_config::NODE_TABLE_KEY;
#[cfg(test)]
const NODE_PUBLIC_KEY_FIELD: &str = node_keypair_config::NODE_PUBLIC_KEY_FIELD;
const DEFAULT_REWARD_RUNTIME_REPORT_DIR: &str = "output/node-reward-runtime";
const DEFAULT_REWARD_RUNTIME_STATE_FILE: &str = "reward-runtime-state.json";
const DEFAULT_REWARD_RUNTIME_DISTFS_PROBE_STATE_FILE: &str =
    "reward-runtime-distfs-probe-state.json";
const DEFAULT_REWARD_RUNTIME_EXECUTION_BRIDGE_STATE_FILE: &str =
    "reward-runtime-execution-bridge-state.json";
const DEFAULT_REWARD_RUNTIME_EXECUTION_WORLD_DIR: &str = "reward-runtime-execution-world";
const DEFAULT_REWARD_RUNTIME_EXECUTION_RECORDS_DIR: &str = "reward-runtime-execution-records";
const DEFAULT_REWARD_RUNTIME_RESERVE_UNITS: i64 = 100_000;
const DEFAULT_REWARD_RUNTIME_MIN_OBSERVER_TRACES: u32 = 1;
const DEFAULT_REPLICATION_PORT_OFFSET: u16 = 100;

#[derive(Clone)]
struct LiveNodeHandle {
    primary_runtime: Arc<Mutex<NodeRuntime>>,
    auxiliary_runtimes: Vec<Arc<Mutex<NodeRuntime>>>,
    world_id: String,
    primary_node_id: String,
    reward_network: Option<Arc<dyn ProtoDistributedNetwork<ProtoWorldError> + Send + Sync>>,
}

impl fmt::Debug for LiveNodeHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LiveNodeHandle")
            .field("world_id", &self.world_id)
            .field("primary_node_id", &self.primary_node_id)
            .field("auxiliary_runtime_count", &self.auxiliary_runtimes.len())
            .field("has_reward_network", &self.reward_network.is_some())
            .finish()
    }
}

struct RewardRuntimeLoopConfig {
    world_id: String,
    local_node_id: String,
    settlement_leader_node_id: String,
    settlement_leader_stale_ms: u64,
    settlement_failover_enabled: bool,
    reward_network: Option<Arc<dyn ProtoDistributedNetwork<ProtoWorldError> + Send + Sync>>,
    poll_interval: Duration,
    signer_node_id: String,
    signer_private_key_hex: String,
    signer_public_key_hex: String,
    report_dir: String,
    state_path: std::path::PathBuf,
    distfs_probe_state_path: std::path::PathBuf,
    execution_bridge_state_path: std::path::PathBuf,
    execution_world_dir: std::path::PathBuf,
    execution_records_dir: std::path::PathBuf,
    storage_root: std::path::PathBuf,
    distfs_probe_config: DistfsProbeRuntimeConfig,
    auto_redeem: bool,
    reward_asset_config: RewardAssetConfig,
    initial_reserve_power_units: i64,
    min_observer_traces: u32,
}

#[derive(Debug)]
struct RewardRuntimeWorker {
    stop_tx: mpsc::Sender<()>,
    join_handle: thread::JoinHandle<()>,
}

#[derive(Debug)]
struct ConsensusGateWorker {
    max_tick: Arc<AtomicU64>,
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

    let node_handle = match start_live_node(&options) {
        Ok(handle) => handle,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };
    let mut reward_runtime_worker = match start_reward_runtime_worker(&options, node_handle.clone())
    {
        Ok(worker) => worker,
        Err(err) => {
            eprintln!("{err}");
            stop_live_node(node_handle.as_ref());
            process::exit(1);
        }
    };
    let mut consensus_gate_worker = match start_consensus_gate_worker(&options, node_handle.clone())
    {
        Ok(worker) => worker,
        Err(err) => {
            eprintln!("{err}");
            stop_reward_runtime_worker(reward_runtime_worker.take());
            stop_live_node(node_handle.as_ref());
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

    let mut config = ViewerLiveServerConfig::new(options.scenario)
        .with_bind_addr(options.bind_addr)
        .with_tick_interval(Duration::from_millis(options.tick_ms))
        .with_decision_mode(if options.llm_mode {
            ViewerLiveDecisionMode::Llm
        } else {
            ViewerLiveDecisionMode::Script
        });
    if let Some(worker) = consensus_gate_worker.as_ref() {
        config = config.with_consensus_gate_max_tick(Arc::clone(&worker.max_tick));
    }
    if let Some(handle) = node_handle.as_ref() {
        config = config.with_consensus_runtime(Arc::clone(&handle.primary_runtime));
    }

    let mut server = match ViewerLiveServer::new(config) {
        Ok(server) => server,
        Err(err) => {
            eprintln!("failed to start live viewer server: {err:?}");
            stop_consensus_gate_worker(consensus_gate_worker.take());
            stop_reward_runtime_worker(reward_runtime_worker.take());
            stop_live_node(node_handle.as_ref());
            process::exit(1);
        }
    };

    let run_result = server.run();
    stop_consensus_gate_worker(consensus_gate_worker.take());
    stop_reward_runtime_worker(reward_runtime_worker.take());
    stop_live_node(node_handle.as_ref());

    if let Err(err) = run_result {
        eprintln!("live viewer server failed: {err:?}");
        process::exit(1);
    }
}

fn start_live_node(options: &CliOptions) -> Result<Option<LiveNodeHandle>, String> {
    if !options.node_enabled {
        return Ok(None);
    }

    let keypair = ensure_node_keypair_in_config(Path::new(DEFAULT_CONFIG_FILE_NAME))
        .map_err(|err| format!("failed to ensure node keypair in config.toml: {err}"))?;
    let world_id = format!("live-{}", options.scenario.as_str());
    match options.node_topology {
        NodeTopologyMode::Single => {
            start_single_live_node(options, world_id.as_str(), &keypair).map(Some)
        }
        NodeTopologyMode::Triad => {
            start_triad_live_nodes(options, world_id.as_str(), &keypair).map(Some)
        }
        NodeTopologyMode::TriadDistributed => {
            start_triad_distributed_live_node(options, world_id.as_str(), &keypair).map(Some)
        }
    }
}

fn stop_live_node(node_handle: Option<&LiveNodeHandle>) {
    let Some(node_handle) = node_handle else {
        return;
    };
    for runtime in &node_handle.auxiliary_runtimes {
        stop_node_runtime("auxiliary node runtime", runtime);
    }
    stop_node_runtime("primary node runtime", &node_handle.primary_runtime);
}

fn stop_node_runtime(label: &str, runtime: &Arc<Mutex<NodeRuntime>>) {
    let mut locked = match runtime.lock() {
        Ok(locked) => locked,
        Err(_) => {
            eprintln!("failed to stop {label}: lock poisoned");
            return;
        }
    };
    if let Err(stop_err) = locked.stop() {
        eprintln!("failed to stop {label}: {stop_err:?}");
    }
}

fn build_node_replication_config(
    node_id: &str,
    keypair: &node_keypair_config::NodeKeypairConfig,
) -> Result<NodeReplicationConfig, String> {
    let signer_keypair = derive_node_consensus_signer_keypair(node_id, keypair)?;
    let replication_root = Path::new("output").join("node-distfs").join(node_id);
    NodeReplicationConfig::new(replication_root)
        .and_then(|cfg| {
            cfg.with_signing_keypair(
                signer_keypair.private_key_hex,
                signer_keypair.public_key_hex,
            )
        })
        .map_err(|err| format!("failed to build node replication config: {err:?}"))
}

fn derive_node_consensus_signer_keypair(
    node_id: &str,
    root_keypair: &node_keypair_config::NodeKeypairConfig,
) -> Result<node_keypair_config::NodeKeypairConfig, String> {
    let node_id = node_id.trim();
    if node_id.is_empty() {
        return Err("node consensus signer derivation requires non-empty node_id".to_string());
    }
    let root_private_bytes = hex::decode(root_keypair.private_key_hex.as_str())
        .map_err(|_| "root node.private_key must be valid hex".to_string())?;
    let root_private: [u8; 32] = root_private_bytes
        .try_into()
        .map_err(|_| "root node.private_key must be 32-byte hex".to_string())?;

    let mut hasher = Sha256::new();
    hasher.update(b"agent-world-node-consensus-signer-v1");
    hasher.update(root_private);
    hasher.update(b"|");
    hasher.update(node_id.as_bytes());
    let digest = hasher.finalize();

    let mut derived_private = [0_u8; 32];
    derived_private.copy_from_slice(&digest[..32]);
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&derived_private);
    Ok(node_keypair_config::NodeKeypairConfig {
        private_key_hex: hex::encode(signing_key.to_bytes()),
        public_key_hex: hex::encode(signing_key.verifying_key().to_bytes()),
    })
}

fn build_validator_signer_public_keys(
    validators: &[PosValidator],
    root_keypair: &node_keypair_config::NodeKeypairConfig,
) -> Result<BTreeMap<String, String>, String> {
    let mut bindings = BTreeMap::new();
    for validator in validators {
        let validator_id = validator.validator_id.trim();
        if validator_id.is_empty() {
            return Err("validator_id cannot be empty when deriving signer bindings".to_string());
        }
        let keypair = derive_node_consensus_signer_keypair(validator_id, root_keypair)?;
        bindings.insert(validator_id.to_string(), keypair.public_key_hex);
    }
    Ok(bindings)
}

fn build_signed_pos_config(
    validators: Vec<PosValidator>,
    root_keypair: &node_keypair_config::NodeKeypairConfig,
) -> Result<NodePosConfig, String> {
    let signer_bindings = build_validator_signer_public_keys(validators.as_slice(), root_keypair)?;
    NodePosConfig::ethereum_like(validators)
        .with_validator_signer_public_keys(signer_bindings)
        .map_err(|err| format!("failed to apply validator signer bindings: {err:?}"))
}

fn attach_optional_replication_network(
    options: &CliOptions,
    mut runtime: NodeRuntime,
    role: NodeRole,
) -> Result<
    (
        NodeRuntime,
        Option<Arc<dyn ProtoDistributedNetwork<ProtoWorldError> + Send + Sync>>,
    ),
    String,
> {
    let mut net_config = if !options.node_repl_libp2p_listen.is_empty()
        || !options.node_repl_libp2p_peers.is_empty()
    {
        Libp2pReplicationNetworkConfig::default()
    } else {
        default_replication_network_config(options, role)?
    };
    if !options.node_repl_libp2p_listen.is_empty() || !options.node_repl_libp2p_peers.is_empty() {
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
    }

    let network: Arc<dyn ProtoDistributedNetwork<ProtoWorldError> + Send + Sync> =
        Arc::new(Libp2pReplicationNetwork::new(net_config));
    let mut handle = NodeReplicationNetworkHandle::new(network);
    if let Some(topic) = options.node_repl_topic.as_deref() {
        handle = handle
            .with_topic(topic)
            .map_err(|err| format!("failed to configure replication topic: {err}"))?;
    }
    let reward_network = Some(handle.clone_network());
    runtime = runtime.with_replication_network(handle);

    Ok((runtime, reward_network))
}

fn default_replication_network_config(
    options: &CliOptions,
    role: NodeRole,
) -> Result<Libp2pReplicationNetworkConfig, String> {
    let mut config = Libp2pReplicationNetworkConfig::default();
    match options.node_topology {
        NodeTopologyMode::Single => {
            config.listen_addrs.push(
                "/ip4/127.0.0.1/tcp/0".parse().map_err(|err| {
                    format!("default replication listen multiaddr invalid: {err}")
                })?,
            );
        }
        NodeTopologyMode::Triad => {
            let ip = infer_triad_gossip_ip(options.bind_addr.as_str());
            let sequencer = SocketAddr::new(
                ip,
                checked_add_port(
                    options.triad_gossip_base_port,
                    0,
                    "triad sequencer gossip port",
                )?,
            );
            let storage = SocketAddr::new(
                ip,
                checked_add_port(
                    options.triad_gossip_base_port,
                    1,
                    "triad storage gossip port",
                )?,
            );
            let observer = SocketAddr::new(
                ip,
                checked_add_port(
                    options.triad_gossip_base_port,
                    2,
                    "triad observer gossip port",
                )?,
            );
            apply_role_replication_addrs(&mut config, role, sequencer, storage, observer)?;
        }
        NodeTopologyMode::TriadDistributed => {
            let (bind_addr, peers) = resolve_triad_distributed_gossip(options, role)?;
            apply_role_replication_addrs_from_peer_set(&mut config, role, bind_addr, peers)?;
        }
    }
    Ok(config)
}

fn apply_role_replication_addrs(
    config: &mut Libp2pReplicationNetworkConfig,
    role: NodeRole,
    sequencer_gossip: SocketAddr,
    storage_gossip: SocketAddr,
    observer_gossip: SocketAddr,
) -> Result<(), String> {
    let sequencer = with_replication_port_offset(sequencer_gossip, "triad sequencer gossip")?;
    let storage = with_replication_port_offset(storage_gossip, "triad storage gossip")?;
    let observer = with_replication_port_offset(observer_gossip, "triad observer gossip")?;
    let (listen, peers) = match role {
        NodeRole::Sequencer => (sequencer, vec![storage, observer]),
        NodeRole::Storage => (storage, vec![sequencer, observer]),
        NodeRole::Observer => (observer, vec![sequencer, storage]),
    };
    config
        .listen_addrs
        .push(socket_addr_to_multiaddr(listen).parse().map_err(|err| {
            format!(
                "default replication listen multiaddr invalid for role {}: {err}",
                role.as_str()
            )
        })?);
    for peer in peers {
        config
            .bootstrap_peers
            .push(socket_addr_to_multiaddr(peer).parse().map_err(|err| {
                format!(
                    "default replication peer multiaddr invalid for role {}: {err}",
                    role.as_str()
                )
            })?);
    }
    Ok(())
}

fn apply_role_replication_addrs_from_peer_set(
    config: &mut Libp2pReplicationNetworkConfig,
    role: NodeRole,
    bind_gossip: SocketAddr,
    peer_gossip: Vec<SocketAddr>,
) -> Result<(), String> {
    let listen = with_replication_port_offset(bind_gossip, "triad distributed gossip bind")?;
    config
        .listen_addrs
        .push(socket_addr_to_multiaddr(listen).parse().map_err(|err| {
            format!(
                "default replication listen multiaddr invalid for role {}: {err}",
                role.as_str()
            )
        })?);
    for peer_gossip_addr in peer_gossip {
        let peer = with_replication_port_offset(peer_gossip_addr, "triad distributed gossip peer")?;
        config
            .bootstrap_peers
            .push(socket_addr_to_multiaddr(peer).parse().map_err(|err| {
                format!(
                    "default replication peer multiaddr invalid for role {}: {err}",
                    role.as_str()
                )
            })?);
    }
    Ok(())
}

fn checked_add_port(base: u16, offset: u16, label: &str) -> Result<u16, String> {
    base.checked_add(offset)
        .ok_or_else(|| format!("{label} overflows u16"))
}

fn with_replication_port_offset(addr: SocketAddr, label: &str) -> Result<SocketAddr, String> {
    let port = addr
        .port()
        .checked_add(DEFAULT_REPLICATION_PORT_OFFSET)
        .ok_or_else(|| {
            format!(
                "{} + replication port offset {} overflows u16",
                label, DEFAULT_REPLICATION_PORT_OFFSET
            )
        })?;
    Ok(SocketAddr::new(addr.ip(), port))
}

fn socket_addr_to_multiaddr(addr: SocketAddr) -> String {
    match addr.ip() {
        IpAddr::V4(ip) => format!("/ip4/{ip}/tcp/{}", addr.port()),
        IpAddr::V6(ip) => format!("/ip6/{ip}/tcp/{}", addr.port()),
    }
}

fn reward_runtime_node_report_root(report_dir: &str, node_id: &str) -> PathBuf {
    Path::new(report_dir).join("nodes").join(node_id)
}

fn infer_default_reward_runtime_leader_node_id(local_node_id: &str) -> String {
    if let Some(base) = local_node_id.strip_suffix("-sequencer") {
        return format!("{base}-sequencer");
    }
    if let Some(base) = local_node_id.strip_suffix("-storage") {
        return format!("{base}-sequencer");
    }
    if let Some(base) = local_node_id.strip_suffix("-observer") {
        return format!("{base}-sequencer");
    }
    local_node_id.to_string()
}

fn select_failover_publisher_node_id(
    snapshot: &NodeSnapshot,
    leader_node_id: &str,
) -> Option<String> {
    let mut candidates = vec![(
        snapshot.node_id.clone(),
        snapshot.consensus.committed_height,
        snapshot.consensus.last_committed_at_ms.unwrap_or(0),
    )];
    for peer_head in &snapshot.consensus.peer_heads {
        if peer_head.node_id == leader_node_id {
            continue;
        }
        candidates.push((
            peer_head.node_id.clone(),
            peer_head.height,
            peer_head.committed_at_ms,
        ));
    }
    let max_height = candidates.iter().map(|(_, height, _)| *height).max()?;
    let latest_candidates = candidates
        .into_iter()
        .filter(|(_, height, _)| *height == max_height)
        .collect::<Vec<_>>();
    let max_committed_at_ms = latest_candidates
        .iter()
        .map(|(_, _, committed_at_ms)| *committed_at_ms)
        .max()
        .unwrap_or(0);
    latest_candidates
        .into_iter()
        .filter(|(_, _, committed_at_ms)| *committed_at_ms == max_committed_at_ms)
        .map(|(node_id, _, _)| node_id)
        .min()
}

fn start_single_live_node(
    options: &CliOptions,
    world_id: &str,
    keypair: &node_keypair_config::NodeKeypairConfig,
) -> Result<LiveNodeHandle, String> {
    let mut config = NodeConfig::new(
        options.node_id.clone(),
        world_id.to_string(),
        options.node_role,
    )
    .and_then(|config| config.with_tick_interval(Duration::from_millis(options.node_tick_ms)))
    .map_err(|err| format!("failed to build node config: {err:?}"))?;
    let validators = if options.node_validators.is_empty() {
        config.pos_config.validators.clone()
    } else {
        options.node_validators.clone()
    };
    let pos_config = build_signed_pos_config(validators, keypair)?;
    config = config
        .with_pos_config(pos_config)
        .map_err(|err| format!("failed to apply node pos config: {err:?}"))?;
    config = config.with_auto_attest_all_validators(options.node_auto_attest_all_validators);
    config = config
        .with_require_execution_on_commit(true)
        .with_require_peer_execution_hashes(true);
    if !options.node_gossip_peers.is_empty() && options.node_gossip_bind.is_none() {
        return Err("node gossip peers require --node-gossip-bind".to_string());
    }
    if let Some(bind_addr) = options.node_gossip_bind {
        config = config.with_gossip_optional(bind_addr, options.node_gossip_peers.clone());
    }
    config = config.with_replication(build_node_replication_config(
        options.node_id.as_str(),
        keypair,
    )?);

    let storage_root = Path::new("output")
        .join("node-distfs")
        .join(options.node_id.as_str())
        .join("store");
    let node_report_root = reward_runtime_node_report_root(
        options.reward_runtime_report_dir.as_str(),
        options.node_id.as_str(),
    );
    let mut runtime = NodeRuntime::new(config);
    let execution_driver = NodeRuntimeExecutionDriver::new(
        node_report_root.join(DEFAULT_REWARD_RUNTIME_EXECUTION_BRIDGE_STATE_FILE),
        node_report_root.join(DEFAULT_REWARD_RUNTIME_EXECUTION_WORLD_DIR),
        node_report_root.join(DEFAULT_REWARD_RUNTIME_EXECUTION_RECORDS_DIR),
        storage_root,
    )
    .map_err(|err| format!("failed to initialize node execution driver: {err}"))?;
    runtime = runtime.with_execution_hook(execution_driver);

    let (mut runtime, reward_network) =
        attach_optional_replication_network(options, runtime, options.node_role)?;
    runtime
        .start()
        .map_err(|err| format!("failed to start node runtime: {err:?}"))?;
    Ok(LiveNodeHandle {
        primary_runtime: Arc::new(Mutex::new(runtime)),
        auxiliary_runtimes: Vec::new(),
        world_id: world_id.to_string(),
        primary_node_id: options.node_id.clone(),
        reward_network,
    })
}

fn start_triad_live_nodes(
    options: &CliOptions,
    world_id: &str,
    keypair: &node_keypair_config::NodeKeypairConfig,
) -> Result<LiveNodeHandle, String> {
    let base_id = options.node_id.trim();
    if base_id.is_empty() {
        return Err("--node-id cannot be empty".to_string());
    }

    let primary_node_id = format!("{base_id}-sequencer");
    let storage_node_id = format!("{base_id}-storage");
    let observer_node_id = format!("{base_id}-observer");
    let validators = vec![
        PosValidator {
            validator_id: primary_node_id.clone(),
            stake: 34,
        },
        PosValidator {
            validator_id: storage_node_id.clone(),
            stake: 33,
        },
        PosValidator {
            validator_id: observer_node_id.clone(),
            stake: 33,
        },
    ];
    let ip = infer_triad_gossip_ip(options.bind_addr.as_str());
    let p0 = options.triad_gossip_base_port;
    let p1 = p0.saturating_add(1);
    let p2 = p0.saturating_add(2);
    let sequencer_bind = SocketAddr::new(ip, p0);
    let storage_bind = SocketAddr::new(ip, p1);
    let observer_bind = SocketAddr::new(ip, p2);

    let node_specs = vec![
        (
            primary_node_id.clone(),
            NodeRole::Sequencer,
            sequencer_bind,
            vec![storage_bind, observer_bind],
            true,
        ),
        (
            storage_node_id,
            NodeRole::Storage,
            storage_bind,
            vec![sequencer_bind, observer_bind],
            true,
        ),
        (
            observer_node_id,
            NodeRole::Observer,
            observer_bind,
            vec![sequencer_bind, storage_bind],
            true,
        ),
    ];

    let mut runtimes: Vec<Arc<Mutex<NodeRuntime>>> = Vec::new();
    let mut primary_reward_network: Option<
        Arc<dyn ProtoDistributedNetwork<ProtoWorldError> + Send + Sync>,
    > = None;
    let triad_pos_config = build_signed_pos_config(validators.clone(), keypair)?;
    for (node_id, role, bind_addr, peers, attach_execution_hook) in node_specs {
        let mut config = NodeConfig::new(node_id.clone(), world_id.to_string(), role)
            .and_then(|cfg| cfg.with_tick_interval(Duration::from_millis(options.node_tick_ms)))
            .map_err(|err| format!("failed to build triad node config {node_id}: {err:?}"))?;
        config = config
            .with_pos_config(triad_pos_config.clone())
            .map_err(|err| format!("failed to apply triad pos config for {node_id}: {err:?}"))?;
        config = config.with_auto_attest_all_validators(options.node_auto_attest_all_validators);
        config = config
            .with_require_execution_on_commit(true)
            .with_require_peer_execution_hashes(true);
        config = config.with_gossip_optional(bind_addr, peers);
        config = config.with_replication(build_node_replication_config(node_id.as_str(), keypair)?);

        let mut runtime = NodeRuntime::new(config);
        if attach_execution_hook {
            let storage_root = Path::new("output")
                .join("node-distfs")
                .join(node_id.as_str())
                .join("store");
            let node_report_root = reward_runtime_node_report_root(
                options.reward_runtime_report_dir.as_str(),
                node_id.as_str(),
            );
            let execution_driver = NodeRuntimeExecutionDriver::new(
                node_report_root.join(DEFAULT_REWARD_RUNTIME_EXECUTION_BRIDGE_STATE_FILE),
                node_report_root.join(DEFAULT_REWARD_RUNTIME_EXECUTION_WORLD_DIR),
                node_report_root.join(DEFAULT_REWARD_RUNTIME_EXECUTION_RECORDS_DIR),
                storage_root,
            )
            .map_err(|err| format!("failed to initialize triad execution driver: {err}"))?;
            runtime = runtime.with_execution_hook(execution_driver);
        }
        let (runtime_with_network, reward_network) =
            attach_optional_replication_network(options, runtime, role)?;
        runtime = runtime_with_network;
        if matches!(role, NodeRole::Sequencer) {
            primary_reward_network = reward_network;
        }

        if let Err(err) = runtime.start() {
            for started in &runtimes {
                stop_node_runtime("triad node runtime", started);
            }
            return Err(format!(
                "failed to start triad node runtime {node_id}: {err:?}"
            ));
        }
        runtimes.push(Arc::new(Mutex::new(runtime)));
    }

    let primary_runtime = match runtimes.first() {
        Some(runtime) => Arc::clone(runtime),
        None => return Err("triad startup produced no runtimes".to_string()),
    };
    let auxiliary_runtimes = runtimes.into_iter().skip(1).collect::<Vec<_>>();
    Ok(LiveNodeHandle {
        primary_runtime,
        auxiliary_runtimes,
        world_id: world_id.to_string(),
        primary_node_id,
        reward_network: primary_reward_network,
    })
}

fn start_triad_distributed_live_node(
    options: &CliOptions,
    world_id: &str,
    keypair: &node_keypair_config::NodeKeypairConfig,
) -> Result<LiveNodeHandle, String> {
    let base_id = options.node_id.trim();
    if base_id.is_empty() {
        return Err("--node-id cannot be empty".to_string());
    }
    let sequencer_node_id = format!("{base_id}-sequencer");
    let storage_node_id = format!("{base_id}-storage");
    let observer_node_id = format!("{base_id}-observer");
    let (bind_addr, peers) = resolve_triad_distributed_gossip(options, options.node_role)?;

    let validators = vec![
        PosValidator {
            validator_id: sequencer_node_id.clone(),
            stake: 34,
        },
        PosValidator {
            validator_id: storage_node_id.clone(),
            stake: 33,
        },
        PosValidator {
            validator_id: observer_node_id.clone(),
            stake: 33,
        },
    ];
    let (node_id, attach_execution_hook) = match options.node_role {
        NodeRole::Sequencer => (sequencer_node_id, true),
        NodeRole::Storage => (storage_node_id, true),
        NodeRole::Observer => (observer_node_id, true),
    };

    let mut config = NodeConfig::new(node_id.clone(), world_id.to_string(), options.node_role)
        .and_then(|cfg| cfg.with_tick_interval(Duration::from_millis(options.node_tick_ms)))
        .map_err(|err| {
            format!("failed to build triad_distributed node config {node_id}: {err:?}")
        })?;
    let pos_config = build_signed_pos_config(validators, keypair)?;
    config = config.with_pos_config(pos_config).map_err(|err| {
        format!("failed to apply triad_distributed pos config for {node_id}: {err:?}")
    })?;
    config = config.with_auto_attest_all_validators(options.node_auto_attest_all_validators);
    config = config
        .with_require_execution_on_commit(true)
        .with_require_peer_execution_hashes(true);
    config = config.with_gossip_optional(bind_addr, peers);
    config = config.with_replication(build_node_replication_config(node_id.as_str(), keypair)?);

    let mut runtime = NodeRuntime::new(config);
    if attach_execution_hook {
        let storage_root = Path::new("output")
            .join("node-distfs")
            .join(node_id.as_str())
            .join("store");
        let node_report_root = reward_runtime_node_report_root(
            options.reward_runtime_report_dir.as_str(),
            node_id.as_str(),
        );
        let execution_driver = NodeRuntimeExecutionDriver::new(
            node_report_root.join(DEFAULT_REWARD_RUNTIME_EXECUTION_BRIDGE_STATE_FILE),
            node_report_root.join(DEFAULT_REWARD_RUNTIME_EXECUTION_WORLD_DIR),
            node_report_root.join(DEFAULT_REWARD_RUNTIME_EXECUTION_RECORDS_DIR),
            storage_root,
        )
        .map_err(|err| format!("failed to initialize triad_distributed execution driver: {err}"))?;
        runtime = runtime.with_execution_hook(execution_driver);
    }
    let (mut runtime, reward_network) =
        attach_optional_replication_network(options, runtime, options.node_role)?;
    runtime
        .start()
        .map_err(|err| format!("failed to start triad_distributed runtime {node_id}: {err:?}"))?;
    Ok(LiveNodeHandle {
        primary_runtime: Arc::new(Mutex::new(runtime)),
        auxiliary_runtimes: Vec::new(),
        world_id: world_id.to_string(),
        primary_node_id: node_id,
        reward_network,
    })
}

fn infer_triad_gossip_ip(bind_addr: &str) -> IpAddr {
    bind_addr
        .parse::<SocketAddr>()
        .map(|addr| addr.ip())
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
}

fn start_consensus_gate_worker(
    options: &CliOptions,
    node_handle: Option<LiveNodeHandle>,
) -> Result<Option<ConsensusGateWorker>, String> {
    if !options.viewer_consensus_gate {
        return Ok(None);
    }
    let handle = node_handle.ok_or_else(|| {
        "viewer consensus gate requires embedded node runtime; remove --no-node or pass --viewer-no-consensus-gate"
            .to_string()
    })?;
    let runtime = Arc::clone(&handle.primary_runtime);
    let max_tick = Arc::new(AtomicU64::new(0));
    let worker_max_tick = Arc::clone(&max_tick);
    let poll_interval = Duration::from_millis(options.node_tick_ms.max(20));
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let join_handle = thread::Builder::new()
        .name("viewer-consensus-gate".to_string())
        .spawn(move || loop {
            match stop_rx.recv_timeout(poll_interval) {
                Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }

            let snapshot = match runtime.lock() {
                Ok(locked) => locked.snapshot(),
                Err(_) => break,
            };
            let committed_height = snapshot.consensus.committed_height;
            let execution_height = snapshot.consensus.last_execution_height;
            let max_allowed_tick = if execution_height > 0 {
                committed_height.min(execution_height)
            } else {
                committed_height
            };
            worker_max_tick.store(max_allowed_tick, Ordering::SeqCst);
        })
        .map_err(|err| format!("failed to spawn viewer consensus gate worker: {err}"))?;

    Ok(Some(ConsensusGateWorker {
        max_tick,
        stop_tx,
        join_handle,
    }))
}

fn stop_consensus_gate_worker(worker: Option<ConsensusGateWorker>) {
    let Some(worker) = worker else {
        return;
    };
    let _ = worker.stop_tx.send(());
    if worker.join_handle.join().is_err() {
        eprintln!("viewer consensus gate worker join failed");
    }
}

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
        poll_interval: Duration::from_millis(options.tick_ms),
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
    let _ = reward_world.bind_node_identity(
        config.signer_node_id.as_str(),
        config.signer_public_key_hex.as_str(),
    );
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
        let should_publish_settlement = settlement_network_enabled
            && observer_trace_threshold_met
            && matches!(
                snapshot.consensus.last_status,
                Some(PosConsensusStatus::Committed)
            )
            && local_is_settlement_publisher;
        let requires_local_settlement = observer_trace_threshold_met
            && (should_publish_settlement || !settlement_network_enabled);
        let minted_records = if requires_local_settlement {
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
            settlement_skipped_reason = if !matches!(
                snapshot.consensus.last_status,
                Some(PosConsensusStatus::Committed)
            ) {
                Some("local node consensus status is not committed".to_string())
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

#[cfg(test)]
#[path = "world_viewer_live/world_viewer_live_tests.rs"]
mod world_viewer_live_tests;
