use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use agent_world::runtime::{NodeAssetBalance, NodeRewardMintRecord, World as RuntimeWorld};
use agent_world_node::{
    NodeConfig, NodePosConfig, NodeReplicationConfig, NodeRole, NodeRuntime, NodeSnapshot,
    PosConsensusStatus, PosValidator,
};
use ed25519_dalek::SigningKey;
use serde::Serialize;
use sha2::{Digest, Sha256};

#[cfg(not(test))]
#[allow(dead_code)]
#[path = "world_viewer_live/execution_bridge.rs"]
mod execution_bridge;
#[path = "world_viewer_live/node_keypair_config.rs"]
mod node_keypair_config;
use execution_bridge::NodeRuntimeExecutionDriver;

#[cfg(test)]
mod execution_bridge {
    use std::path::Path;

    use agent_world::runtime::World as RuntimeWorld;
    use agent_world_node::{
        NodeExecutionCommitContext, NodeExecutionCommitResult, NodeExecutionHook,
    };

    #[derive(Debug)]
    pub(super) struct NodeRuntimeExecutionDriver;

    impl NodeRuntimeExecutionDriver {
        pub(super) fn new(
            _state_path: std::path::PathBuf,
            _world_dir: std::path::PathBuf,
            _records_dir: std::path::PathBuf,
            _storage_root: std::path::PathBuf,
        ) -> Result<Self, String> {
            Ok(Self)
        }
    }

    impl NodeExecutionHook for NodeRuntimeExecutionDriver {
        fn on_commit(
            &mut self,
            context: NodeExecutionCommitContext,
        ) -> Result<NodeExecutionCommitResult, String> {
            Ok(NodeExecutionCommitResult {
                execution_height: context.height,
                execution_block_hash: String::new(),
                execution_state_root: String::new(),
            })
        }
    }

    pub(super) fn load_execution_world(world_dir: &Path) -> Result<RuntimeWorld, String> {
        let snapshot_path = world_dir.join("snapshot.json");
        let journal_path = world_dir.join("journal.json");
        if !snapshot_path.exists() || !journal_path.exists() {
            return Ok(RuntimeWorld::new());
        }
        RuntimeWorld::load_from_dir(world_dir).map_err(|err| {
            format!(
                "load execution world from {} failed: {:?}",
                world_dir.display(),
                err
            )
        })
    }
}

const DEFAULT_NODE_ID: &str = "viewer-live-node";
const DEFAULT_WORLD_ID: &str = "live-llm_bootstrap";
const DEFAULT_STATUS_BIND: &str = "127.0.0.1:5121";
const DEFAULT_CONFIG_FILE: &str = "config.toml";
const DEFAULT_NODE_TICK_MS: u64 = 200;
const DEFAULT_RECENT_MINT_RECORD_LIMIT: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliOptions {
    node_id: String,
    world_id: String,
    status_bind: String,
    node_role: NodeRole,
    node_tick_ms: u64,
    node_auto_attest_all_validators: bool,
    node_validators: Vec<PosValidator>,
    node_gossip_bind: Option<SocketAddr>,
    node_gossip_peers: Vec<SocketAddr>,
    config_path: String,
    execution_bridge_state_path: Option<PathBuf>,
    execution_world_dir: Option<PathBuf>,
    execution_records_dir: Option<PathBuf>,
    storage_root: Option<PathBuf>,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            node_id: DEFAULT_NODE_ID.to_string(),
            world_id: DEFAULT_WORLD_ID.to_string(),
            status_bind: DEFAULT_STATUS_BIND.to_string(),
            node_role: NodeRole::Sequencer,
            node_tick_ms: DEFAULT_NODE_TICK_MS,
            node_auto_attest_all_validators: false,
            node_validators: Vec::new(),
            node_gossip_bind: None,
            node_gossip_peers: Vec::new(),
            config_path: DEFAULT_CONFIG_FILE.to_string(),
            execution_bridge_state_path: None,
            execution_world_dir: None,
            execution_records_dir: None,
            storage_root: None,
        }
    }
}

#[derive(Debug)]
struct RuntimePaths {
    execution_bridge_state_path: PathBuf,
    execution_world_dir: PathBuf,
    execution_records_dir: PathBuf,
    storage_root: PathBuf,
}

#[derive(Debug)]
struct ChainStatusServer {
    stop_tx: Sender<()>,
    error_rx: Receiver<String>,
    join_handle: Option<thread::JoinHandle<()>>,
}

#[derive(Debug, Serialize)]
struct ChainStatusResponse {
    ok: bool,
    observed_at_unix_ms: i64,
    node_id: String,
    world_id: String,
    role: String,
    running: bool,
    tick_count: u64,
    last_tick_unix_ms: Option<i64>,
    consensus: ChainConsensusStatus,
    last_error: Option<String>,
    execution_world_dir: String,
}

#[derive(Debug, Serialize)]
struct ChainConsensusStatus {
    slot: u64,
    epoch: u64,
    latest_height: u64,
    committed_height: u64,
    network_committed_height: u64,
    last_status: Option<String>,
    last_block_hash: Option<String>,
    last_execution_height: u64,
    last_execution_block_hash: Option<String>,
    last_execution_state_root: Option<String>,
    known_peer_heads: usize,
}

#[derive(Debug, Serialize)]
struct ChainBalancesResponse {
    ok: bool,
    observed_at_unix_ms: i64,
    node_id: String,
    world_id: String,
    execution_world_dir: String,
    load_error: Option<String>,
    node_asset_balance: Option<NodeAssetBalance>,
    node_power_credit_balance: u64,
    node_main_token_account: Option<String>,
    node_main_token_liquid_balance: u64,
    reward_mint_record_count: usize,
    recent_reward_mint_records: Vec<NodeRewardMintRecord>,
}

fn main() {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    if raw_args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return;
    }

    let options = match parse_options(raw_args.iter().map(|arg| arg.as_str())) {
        Ok(options) => options,
        Err(err) => {
            eprintln!("{err}");
            print_help();
            process::exit(1);
        }
    };

    if let Err(err) = run_chain_runtime(options) {
        eprintln!("world_chain_runtime failed: {err}");
        process::exit(1);
    }
}

fn run_chain_runtime(options: CliOptions) -> Result<(), String> {
    let paths = resolve_runtime_paths(&options);
    let keypair = node_keypair_config::ensure_node_keypair_in_config(Path::new(
        options.config_path.as_str(),
    ))?;

    let mut config = NodeConfig::new(
        options.node_id.clone(),
        options.world_id.clone(),
        options.node_role,
    )
    .and_then(|cfg| cfg.with_tick_interval(Duration::from_millis(options.node_tick_ms)))
    .map_err(|err| format!("failed to build node config: {err:?}"))?;

    let validators = if options.node_validators.is_empty() {
        config.pos_config.validators.clone()
    } else {
        options.node_validators.clone()
    };
    let pos_config = build_signed_pos_config(validators, &keypair)?;
    config = config
        .with_pos_config(pos_config)
        .map_err(|err| format!("failed to apply node pos config: {err:?}"))?;
    config = config.with_auto_attest_all_validators(options.node_auto_attest_all_validators);

    let require_execution = matches!(options.node_role, NodeRole::Sequencer);
    config = config
        .with_require_execution_on_commit(require_execution)
        .with_require_peer_execution_hashes(require_execution);

    if !options.node_gossip_peers.is_empty() && options.node_gossip_bind.is_none() {
        return Err("--node-gossip-peer requires --node-gossip-bind".to_string());
    }
    if let Some(bind_addr) = options.node_gossip_bind {
        config = config.with_gossip_optional(bind_addr, options.node_gossip_peers.clone());
    }

    config = config.with_replication(build_node_replication_config(
        options.node_id.as_str(),
        &keypair,
    )?);

    let mut runtime = NodeRuntime::new(config);
    if require_execution {
        let execution_driver = NodeRuntimeExecutionDriver::new(
            paths.execution_bridge_state_path.clone(),
            paths.execution_world_dir.clone(),
            paths.execution_records_dir.clone(),
            paths.storage_root.clone(),
        )
        .map_err(|err| format!("failed to initialize execution driver: {err}"))?;
        runtime = runtime.with_execution_hook(execution_driver);
    }

    runtime
        .start()
        .map_err(|err| format!("failed to start node runtime: {err:?}"))?;

    let runtime = Arc::new(Mutex::new(runtime));
    let (status_host, status_port) =
        parse_host_port(options.status_bind.as_str(), "--status-bind")?;
    let mut status_server = start_chain_status_server(
        status_host.as_str(),
        status_port,
        Arc::clone(&runtime),
        options.node_id.clone(),
        options.world_id.clone(),
        paths.execution_world_dir.clone(),
    )?;

    println!("world_chain_runtime ready.");
    println!("- node_id: {}", options.node_id);
    println!("- world_id: {}", options.world_id);
    println!("- role: {}", options.node_role.as_str());
    println!(
        "- status: http://{}:{}/v1/chain/status",
        status_host, status_port
    );
    println!(
        "- balances: http://{}:{}/v1/chain/balances",
        status_host, status_port
    );
    println!("Press Ctrl+C to stop.");

    let mut last_error = String::new();
    loop {
        if let Some(server_err) = poll_chain_status_server_error(&mut status_server)? {
            stop_chain_status_server(&mut status_server);
            stop_runtime(&runtime);
            return Err(server_err);
        }

        if let Ok(snapshot) = runtime.lock().map(|locked| locked.snapshot()) {
            if let Some(err) = snapshot.last_error {
                if err != last_error {
                    eprintln!("node runtime reported error: {err}");
                    last_error = err;
                }
            }
        }

        thread::sleep(Duration::from_millis(300));
    }
}

fn resolve_runtime_paths(options: &CliOptions) -> RuntimePaths {
    let root = Path::new("output")
        .join("chain-runtime")
        .join(options.node_id.as_str());
    RuntimePaths {
        execution_bridge_state_path: options
            .execution_bridge_state_path
            .clone()
            .unwrap_or_else(|| root.join("reward-runtime-execution-bridge-state.json")),
        execution_world_dir: options
            .execution_world_dir
            .clone()
            .unwrap_or_else(|| root.join("reward-runtime-execution-world")),
        execution_records_dir: options
            .execution_records_dir
            .clone()
            .unwrap_or_else(|| root.join("reward-runtime-execution-records")),
        storage_root: options
            .storage_root
            .clone()
            .unwrap_or_else(|| root.join("store")),
    }
}

fn stop_runtime(runtime: &Arc<Mutex<NodeRuntime>>) {
    let mut locked = match runtime.lock() {
        Ok(locked) => locked,
        Err(_) => {
            eprintln!("failed to stop node runtime: lock poisoned");
            return;
        }
    };
    if let Err(err) = locked.stop() {
        eprintln!("failed to stop node runtime: {err:?}");
    }
}

fn start_chain_status_server(
    host: &str,
    port: u16,
    runtime: Arc<Mutex<NodeRuntime>>,
    node_id: String,
    world_id: String,
    execution_world_dir: PathBuf,
) -> Result<ChainStatusServer, String> {
    let listener = TcpListener::bind((host, port))
        .map_err(|err| format!("failed to bind status server at {host}:{port}: {err}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|err| format!("failed to set status server listener nonblocking: {err}"))?;

    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let (error_tx, error_rx) = mpsc::channel::<String>();

    let join_handle = thread::spawn(move || {
        if let Err(err) = run_chain_status_server_loop(
            listener,
            stop_rx,
            runtime,
            node_id,
            world_id,
            execution_world_dir,
        ) {
            let _ = error_tx.send(err);
        }
    });

    Ok(ChainStatusServer {
        stop_tx,
        error_rx,
        join_handle: Some(join_handle),
    })
}

fn run_chain_status_server_loop(
    listener: TcpListener,
    stop_rx: Receiver<()>,
    runtime: Arc<Mutex<NodeRuntime>>,
    node_id: String,
    world_id: String,
    execution_world_dir: PathBuf,
) -> Result<(), String> {
    loop {
        match stop_rx.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => return Ok(()),
            Err(TryRecvError::Empty) => {}
        }

        match listener.accept() {
            Ok((stream, _addr)) => {
                let runtime = Arc::clone(&runtime);
                let node_id = node_id.clone();
                let world_id = world_id.clone();
                let execution_world_dir = execution_world_dir.clone();
                thread::spawn(move || {
                    if let Err(err) = handle_chain_status_connection(
                        stream,
                        runtime,
                        node_id.as_str(),
                        world_id.as_str(),
                        execution_world_dir.as_path(),
                    ) {
                        eprintln!("warning: chain status connection failed: {err}");
                    }
                });
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(err) => {
                return Err(format!("chain status server accept failed: {err}"));
            }
        }
    }
}

fn handle_chain_status_connection(
    mut stream: TcpStream,
    runtime: Arc<Mutex<NodeRuntime>>,
    node_id: &str,
    world_id: &str,
    execution_world_dir: &Path,
) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|err| format!("failed to set read timeout: {err}"))?;

    let mut buffer = [0_u8; 8192];
    let bytes = stream
        .read(&mut buffer)
        .map_err(|err| format!("failed to read request: {err}"))?;
    if bytes == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes]);
    let Some(line) = request.lines().next() else {
        write_json_response(&mut stream, 400, b"{\"error\":\"bad request\"}", false)
            .map_err(|err| format!("failed to write 400 response: {err}"))?;
        return Ok(());
    };

    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or_default();
    let head_only = method.eq_ignore_ascii_case("HEAD");

    if !method.eq_ignore_ascii_case("GET") && !head_only {
        write_json_response(
            &mut stream,
            405,
            b"{\"error\":\"method not allowed\"}",
            head_only,
        )
        .map_err(|err| format!("failed to write 405 response: {err}"))?;
        return Ok(());
    }

    match target.split('?').next().unwrap_or(target) {
        "/healthz" => {
            write_json_response(&mut stream, 200, b"{\"ok\":true}", head_only)
                .map_err(|err| format!("failed to write /healthz response: {err}"))?;
        }
        "/v1/chain/status" => {
            let snapshot = runtime
                .lock()
                .map_err(|_| "failed to read node runtime snapshot: lock poisoned".to_string())?
                .snapshot();
            let payload = build_chain_status_payload(snapshot, execution_world_dir);
            let body = serde_json::to_vec_pretty(&payload)
                .map_err(|err| format!("failed to encode status payload: {err}"))?;
            write_json_response(&mut stream, 200, body.as_slice(), head_only)
                .map_err(|err| format!("failed to write /v1/chain/status response: {err}"))?;
        }
        "/v1/chain/balances" => {
            let payload = build_chain_balances_payload(node_id, world_id, execution_world_dir);
            let body = serde_json::to_vec_pretty(&payload)
                .map_err(|err| format!("failed to encode balances payload: {err}"))?;
            write_json_response(&mut stream, 200, body.as_slice(), head_only)
                .map_err(|err| format!("failed to write /v1/chain/balances response: {err}"))?;
        }
        _ => {
            write_json_response(&mut stream, 404, b"{\"error\":\"not found\"}", head_only)
                .map_err(|err| format!("failed to write 404 response: {err}"))?;
        }
    }

    Ok(())
}

fn build_chain_status_payload(
    snapshot: NodeSnapshot,
    execution_world_dir: &Path,
) -> ChainStatusResponse {
    let last_status = snapshot
        .consensus
        .last_status
        .map(consensus_status_to_string);

    ChainStatusResponse {
        ok: true,
        observed_at_unix_ms: now_unix_ms(),
        node_id: snapshot.node_id,
        world_id: snapshot.world_id,
        role: snapshot.role.as_str().to_string(),
        running: snapshot.running,
        tick_count: snapshot.tick_count,
        last_tick_unix_ms: snapshot.last_tick_unix_ms,
        consensus: ChainConsensusStatus {
            slot: snapshot.consensus.slot,
            epoch: snapshot.consensus.epoch,
            latest_height: snapshot.consensus.latest_height,
            committed_height: snapshot.consensus.committed_height,
            network_committed_height: snapshot.consensus.network_committed_height,
            last_status,
            last_block_hash: snapshot.consensus.last_block_hash,
            last_execution_height: snapshot.consensus.last_execution_height,
            last_execution_block_hash: snapshot.consensus.last_execution_block_hash,
            last_execution_state_root: snapshot.consensus.last_execution_state_root,
            known_peer_heads: snapshot.consensus.known_peer_heads,
        },
        last_error: snapshot.last_error,
        execution_world_dir: execution_world_dir.display().to_string(),
    }
}

fn build_chain_balances_payload(
    node_id: &str,
    world_id: &str,
    execution_world_dir: &Path,
) -> ChainBalancesResponse {
    match execution_bridge::load_execution_world(execution_world_dir) {
        Ok(world) => {
            build_chain_balances_payload_from_world(node_id, world_id, execution_world_dir, &world)
        }
        Err(err) => ChainBalancesResponse {
            ok: true,
            observed_at_unix_ms: now_unix_ms(),
            node_id: node_id.to_string(),
            world_id: world_id.to_string(),
            execution_world_dir: execution_world_dir.display().to_string(),
            load_error: Some(err),
            node_asset_balance: None,
            node_power_credit_balance: 0,
            node_main_token_account: None,
            node_main_token_liquid_balance: 0,
            reward_mint_record_count: 0,
            recent_reward_mint_records: Vec::new(),
        },
    }
}

fn build_chain_balances_payload_from_world(
    node_id: &str,
    world_id: &str,
    execution_world_dir: &Path,
    world: &RuntimeWorld,
) -> ChainBalancesResponse {
    let node_asset_balance = world.node_asset_balance(node_id).cloned();
    let node_power_credit_balance = world.node_power_credit_balance(node_id);
    let node_main_token_account = world
        .node_main_token_account(node_id)
        .map(|value| value.to_string());
    let node_main_token_liquid_balance = node_main_token_account
        .as_deref()
        .map(|account_id| world.main_token_liquid_balance(account_id))
        .unwrap_or(0);

    let all_records = world.reward_mint_records();
    let record_count = all_records.len();
    let keep = record_count.min(DEFAULT_RECENT_MINT_RECORD_LIMIT);
    let recent_reward_mint_records = all_records
        .iter()
        .skip(record_count.saturating_sub(keep))
        .cloned()
        .collect::<Vec<_>>();

    ChainBalancesResponse {
        ok: true,
        observed_at_unix_ms: now_unix_ms(),
        node_id: node_id.to_string(),
        world_id: world_id.to_string(),
        execution_world_dir: execution_world_dir.display().to_string(),
        load_error: None,
        node_asset_balance,
        node_power_credit_balance,
        node_main_token_account,
        node_main_token_liquid_balance,
        reward_mint_record_count: record_count,
        recent_reward_mint_records,
    }
}

fn poll_chain_status_server_error(
    server: &mut ChainStatusServer,
) -> Result<Option<String>, String> {
    match server.error_rx.try_recv() {
        Ok(err) => Ok(Some(format!("status server failed: {err}"))),
        Err(TryRecvError::Disconnected) => Ok(Some(
            "status server channel disconnected unexpectedly".to_string(),
        )),
        Err(TryRecvError::Empty) => {
            if let Some(handle) = server.join_handle.as_ref() {
                if handle.is_finished() {
                    return Ok(Some("status server exited unexpectedly".to_string()));
                }
            }
            Ok(None)
        }
    }
}

fn stop_chain_status_server(server: &mut ChainStatusServer) {
    let _ = server.stop_tx.send(());
    if let Some(handle) = server.join_handle.take() {
        let _ = handle.join();
    }
}

fn write_json_response(
    stream: &mut TcpStream,
    status_code: u16,
    body: &[u8],
    head_only: bool,
) -> std::io::Result<()> {
    let status_text = match status_code {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Internal Server Error",
    };
    let headers = format!(
        "HTTP/1.1 {status_code} {status_text}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(headers.as_bytes())?;
    if !head_only {
        stream.write_all(body)?;
    }
    stream.flush()?;
    Ok(())
}

fn consensus_status_to_string(status: PosConsensusStatus) -> String {
    match status {
        PosConsensusStatus::Pending => "pending".to_string(),
        PosConsensusStatus::Committed => "committed".to_string(),
        PosConsensusStatus::Rejected => "rejected".to_string(),
    }
}

fn parse_options<'a>(args: impl Iterator<Item = &'a str>) -> Result<CliOptions, String> {
    let mut options = CliOptions::default();
    let mut iter = args.peekable();

    while let Some(arg) = iter.next() {
        match arg {
            "--node-id" => {
                options.node_id = parse_required_value(&mut iter, "--node-id")?;
            }
            "--world-id" => {
                options.world_id = parse_required_value(&mut iter, "--world-id")?;
            }
            "--status-bind" => {
                options.status_bind = parse_required_value(&mut iter, "--status-bind")?;
            }
            "--node-role" => {
                let raw = parse_required_value(&mut iter, "--node-role")?;
                options.node_role = raw.parse::<NodeRole>().map_err(|_| {
                    "--node-role must be one of: sequencer, storage, observer".to_string()
                })?;
            }
            "--node-tick-ms" => {
                let raw = parse_required_value(&mut iter, "--node-tick-ms")?;
                options.node_tick_ms = raw
                    .parse::<u64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or_else(|| "--node-tick-ms requires a positive integer".to_string())?;
            }
            "--node-validator" => {
                let raw = parse_required_value(&mut iter, "--node-validator")?;
                options
                    .node_validators
                    .push(parse_validator_spec(raw.as_str())?);
            }
            "--node-auto-attest-all" => {
                options.node_auto_attest_all_validators = true;
            }
            "--node-no-auto-attest-all" => {
                options.node_auto_attest_all_validators = false;
            }
            "--node-gossip-bind" => {
                let raw = parse_required_value(&mut iter, "--node-gossip-bind")?;
                options.node_gossip_bind =
                    Some(parse_socket_addr(raw.as_str(), "--node-gossip-bind")?);
            }
            "--node-gossip-peer" => {
                let raw = parse_required_value(&mut iter, "--node-gossip-peer")?;
                options
                    .node_gossip_peers
                    .push(parse_socket_addr(raw.as_str(), "--node-gossip-peer")?);
            }
            "--config" => {
                options.config_path = parse_required_value(&mut iter, "--config")?;
            }
            "--execution-bridge-state" => {
                let raw = parse_required_value(&mut iter, "--execution-bridge-state")?;
                options.execution_bridge_state_path = Some(PathBuf::from(raw));
            }
            "--execution-world-dir" => {
                let raw = parse_required_value(&mut iter, "--execution-world-dir")?;
                options.execution_world_dir = Some(PathBuf::from(raw));
            }
            "--execution-records-dir" => {
                let raw = parse_required_value(&mut iter, "--execution-records-dir")?;
                options.execution_records_dir = Some(PathBuf::from(raw));
            }
            "--storage-root" => {
                let raw = parse_required_value(&mut iter, "--storage-root")?;
                options.storage_root = Some(PathBuf::from(raw));
            }
            _ => return Err(format!("unknown option: {arg}")),
        }
    }

    parse_host_port(options.status_bind.as_str(), "--status-bind")?;
    if options.node_id.trim().is_empty() {
        return Err("--node-id requires a non-empty value".to_string());
    }
    if options.world_id.trim().is_empty() {
        return Err("--world-id requires a non-empty value".to_string());
    }
    if options.config_path.trim().is_empty() {
        return Err("--config requires a non-empty value".to_string());
    }

    if !options.node_gossip_peers.is_empty() && options.node_gossip_bind.is_none() {
        return Err("--node-gossip-peer requires --node-gossip-bind".to_string());
    }

    Ok(options)
}

fn parse_required_value<'a, I>(
    iter: &mut std::iter::Peekable<I>,
    flag: &str,
) -> Result<String, String>
where
    I: Iterator<Item = &'a str>,
{
    let Some(value) = iter.next() else {
        return Err(format!("{flag} requires a value"));
    };
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{flag} requires a non-empty value"));
    }
    Ok(value.to_string())
}

fn parse_socket_addr(raw: &str, label: &str) -> Result<SocketAddr, String> {
    raw.parse::<SocketAddr>()
        .map_err(|_| format!("{label} requires <addr:port>"))
}

fn parse_host_port(raw: &str, label: &str) -> Result<(String, u16), String> {
    let trimmed = raw.trim();
    let (host, port_text) = trimmed
        .rsplit_once(':')
        .ok_or_else(|| format!("{label} must be in <host:port> format"))?;
    if host.trim().is_empty() {
        return Err(format!("{label} host cannot be empty"));
    }
    let port = port_text
        .parse::<u16>()
        .map_err(|_| format!("{label} port must be an integer in 1..=65535"))?;
    if port == 0 {
        return Err(format!("{label} port must be in 1..=65535"));
    }
    Ok((host.trim().to_string(), port))
}

fn parse_validator_spec(raw: &str) -> Result<PosValidator, String> {
    let (validator_id, stake_text) = raw
        .rsplit_once(':')
        .ok_or_else(|| "--node-validator requires <validator_id:stake>".to_string())?;
    let validator_id = validator_id.trim();
    if validator_id.is_empty() {
        return Err("--node-validator validator_id cannot be empty".to_string());
    }
    let stake = stake_text
        .parse::<u64>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| "--node-validator stake must be a positive integer".to_string())?;
    Ok(PosValidator {
        validator_id: validator_id.to_string(),
        stake,
    })
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
    let signing_key = SigningKey::from_bytes(&derived_private);
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

fn print_help() {
    println!(
        "Usage: world_chain_runtime [options]\n\n\
Starts standalone chain/node runtime with status HTTP endpoints.\n\n\
Options:\n\
  --node-id <id>                    node identifier (default: {DEFAULT_NODE_ID})\n\
  --world-id <id>                   world identifier (default: {DEFAULT_WORLD_ID})\n\
  --status-bind <host:port>         status HTTP bind (default: {DEFAULT_STATUS_BIND})\n\
  --node-role <role>                sequencer|storage|observer (default: sequencer)\n\
  --node-tick-ms <n>                node tick interval ms (default: {DEFAULT_NODE_TICK_MS})\n\
  --node-validator <id:stake>       add validator stake (repeatable)\n\
  --node-auto-attest-all            enable auto attesting validators\n\
  --node-no-auto-attest-all         disable auto attesting validators (default)\n\
  --node-gossip-bind <addr:port>    UDP gossip bind\n\
  --node-gossip-peer <addr:port>    UDP gossip peer (repeatable, requires --node-gossip-bind)\n\
  --config <path>                   config file path for node keypair (default: {DEFAULT_CONFIG_FILE})\n\
  --execution-bridge-state <path>   override execution bridge state file path\n\
  --execution-world-dir <path>      override execution world directory\n\
  --execution-records-dir <path>    override execution records directory\n\
  --storage-root <path>             override execution CAS/storage root\n\
  -h, --help                        show help"
    );
}

#[allow(dead_code)]
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
mod tests {
    use super::{
        build_chain_balances_payload_from_world, parse_options, parse_validator_spec, CliOptions,
        DEFAULT_NODE_ID, DEFAULT_STATUS_BIND,
    };
    use agent_world::runtime::World as RuntimeWorld;

    #[test]
    fn parse_options_defaults() {
        let options = parse_options(std::iter::empty()).expect("parse should succeed");
        assert_eq!(options.node_id, DEFAULT_NODE_ID);
        assert_eq!(options.status_bind, DEFAULT_STATUS_BIND);
        assert!(!options.node_auto_attest_all_validators);
        assert!(options.node_validators.is_empty());
    }

    #[test]
    fn parse_options_reads_custom_values() {
        let options = parse_options(
            [
                "--node-id",
                "node-a",
                "--world-id",
                "live-foo",
                "--status-bind",
                "127.0.0.1:6221",
                "--node-role",
                "storage",
                "--node-tick-ms",
                "350",
                "--node-validator",
                "node-a:55",
                "--node-validator",
                "node-b:45",
                "--node-auto-attest-all",
                "--execution-world-dir",
                "custom/world",
            ]
            .into_iter(),
        )
        .expect("parse should succeed");

        assert_eq!(options.node_id, "node-a");
        assert_eq!(options.world_id, "live-foo");
        assert_eq!(options.status_bind, "127.0.0.1:6221");
        assert_eq!(options.node_role.as_str(), "storage");
        assert_eq!(options.node_tick_ms, 350);
        assert_eq!(options.node_validators.len(), 2);
        assert!(options.node_auto_attest_all_validators);
        assert_eq!(
            options
                .execution_world_dir
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            Some("custom/world".to_string())
        );
    }

    #[test]
    fn parse_options_rejects_invalid_status_bind() {
        let err = parse_options(["--status-bind", "127.0.0.1"].into_iter())
            .expect_err("should reject invalid bind");
        assert!(err.contains("<host:port>"));
    }

    #[test]
    fn parse_options_rejects_peer_without_bind() {
        let err = parse_options(["--node-gossip-peer", "127.0.0.1:9001"].into_iter())
            .expect_err("should reject peer without bind");
        assert!(err.contains("requires --node-gossip-bind"));
    }

    #[test]
    fn parse_validator_spec_rejects_zero_stake() {
        let err = parse_validator_spec("node-a:0").expect_err("should reject");
        assert!(err.contains("positive integer"));
    }

    #[test]
    fn balances_payload_reports_empty_world_without_error() {
        let world = RuntimeWorld::new();
        let payload = build_chain_balances_payload_from_world(
            "node-a",
            "live-a",
            std::path::Path::new("/tmp/empty"),
            &world,
        );
        assert!(payload.ok);
        assert!(payload.load_error.is_none());
        assert_eq!(payload.node_power_credit_balance, 0);
        assert_eq!(payload.reward_mint_record_count, 0);
        assert!(payload.recent_reward_mint_records.is_empty());
    }

    #[test]
    fn parse_options_rejects_unknown_option() {
        let err = parse_options(["--unknown"].into_iter()).expect_err("should fail");
        assert!(err.contains("unknown option"));
    }

    #[test]
    fn default_runtime_paths_depend_on_node_id() {
        let options = CliOptions {
            node_id: "node-z".to_string(),
            ..CliOptions::default()
        };
        let paths = super::resolve_runtime_paths(&options);
        assert!(paths
            .execution_world_dir
            .to_string_lossy()
            .contains("output/chain-runtime/node-z"));
    }
}
