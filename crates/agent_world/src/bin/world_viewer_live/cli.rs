use std::net::SocketAddr;
use std::process;

use agent_world::runtime::RewardAssetConfig;
use agent_world::simulator::WorldScenario;
use agent_world_node::{NodeRole, PosValidator};

use super::{
    parse_distfs_probe_runtime_option, DistfsProbeRuntimeConfig,
    DEFAULT_REWARD_RUNTIME_MIN_OBSERVER_TRACES, DEFAULT_REWARD_RUNTIME_REPORT_DIR,
    DEFAULT_REWARD_RUNTIME_RESERVE_UNITS,
};

#[derive(Debug, Clone, PartialEq)]
pub(super) struct CliOptions {
    pub scenario: WorldScenario,
    pub bind_addr: String,
    pub web_bind_addr: Option<String>,
    pub tick_ms: u64,
    pub llm_mode: bool,
    pub viewer_consensus_gate: bool,
    pub node_enabled: bool,
    pub node_id: String,
    pub node_role: NodeRole,
    pub node_tick_ms: u64,
    pub node_auto_attest_all_validators: bool,
    pub node_validators: Vec<PosValidator>,
    pub node_gossip_bind: Option<SocketAddr>,
    pub node_gossip_peers: Vec<SocketAddr>,
    pub node_repl_libp2p_listen: Vec<String>,
    pub node_repl_libp2p_peers: Vec<String>,
    pub node_repl_topic: Option<String>,
    pub reward_runtime_enabled: bool,
    pub reward_runtime_auto_redeem: bool,
    pub reward_runtime_signer_node_id: Option<String>,
    pub reward_runtime_report_dir: String,
    pub reward_runtime_min_observer_traces: u32,
    pub reward_points_per_credit: u64,
    pub reward_credits_per_power_unit: u64,
    pub reward_max_redeem_power_per_epoch: i64,
    pub reward_min_redeem_power_unit: i64,
    pub reward_initial_reserve_power_units: i64,
    pub reward_distfs_probe_config: DistfsProbeRuntimeConfig,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            scenario: WorldScenario::TwinRegionBootstrap,
            bind_addr: "127.0.0.1:5010".to_string(),
            web_bind_addr: None,
            tick_ms: 200,
            llm_mode: false,
            viewer_consensus_gate: true,
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
            reward_runtime_min_observer_traces: DEFAULT_REWARD_RUNTIME_MIN_OBSERVER_TRACES,
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

pub(super) fn parse_options<'a>(args: impl Iterator<Item = &'a str>) -> Result<CliOptions, String> {
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
            "--viewer-no-consensus-gate" => {
                options.viewer_consensus_gate = false;
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
            "--reward-runtime-min-observer-traces" => {
                let raw = iter.next().ok_or_else(|| {
                    "--reward-runtime-min-observer-traces requires a positive integer".to_string()
                })?;
                options.reward_runtime_min_observer_traces = raw
                    .parse::<u32>()
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or_else(|| {
                        "--reward-runtime-min-observer-traces requires a positive integer"
                            .to_string()
                    })?;
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
    if !options.node_enabled {
        options.viewer_consensus_gate = false;
    }

    Ok(options)
}

pub(super) fn print_help() {
    println!(
        "Usage: world_viewer_live [scenario] [--bind <addr>] [--web-bind <addr>] [--tick-ms <ms>] [--llm] [--no-node] [--node-validator <id:stake>...] [--node-gossip-bind <addr:port>] [--node-gossip-peer <addr:port>...]"
    );
    println!("Options:");
    println!("  --bind <addr>     Bind address (default: 127.0.0.1:5010)");
    println!("  --web-bind <addr> WebSocket bridge bind address (optional)");
    println!("  --tick-ms <ms>    Tick interval in milliseconds (default: 200)");
    println!("  --scenario <name> Scenario name (default: twin_region_bootstrap)");
    println!("  --llm             Use LLM decisions instead of built-in script");
    println!(
        "  --viewer-no-consensus-gate Disable viewer tick gating by node consensus/execution height"
    );
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
    println!(
        "  --reward-runtime-auto-redeem Auto redeem minted credits to node-mapped runtime agent"
    );
    println!("  --reward-runtime-signer <node_id> Settlement signer node id (default: --node-id)");
    println!("  --reward-runtime-report-dir <path> Reward runtime report directory (default: output/node-reward-runtime)");
    println!(
        "  --reward-runtime-min-observer-traces <n> Minimum unique observer traces per epoch before settlement (default: 1)"
    );
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
    println!(
        "  --reward-distfs-adaptive-multiplier-hash-mismatch <n> DistFS adaptive backoff multiplier for HASH_MISMATCH (default: 1)"
    );
    println!(
        "  --reward-distfs-adaptive-multiplier-missing-sample <n> DistFS adaptive backoff multiplier for MISSING_SAMPLE (default: 1)"
    );
    println!(
        "  --reward-distfs-adaptive-multiplier-timeout <n> DistFS adaptive backoff multiplier for TIMEOUT (default: 1)"
    );
    println!(
        "  --reward-distfs-adaptive-multiplier-read-io-error <n> DistFS adaptive backoff multiplier for READ_IO_ERROR (default: 1)"
    );
    println!(
        "  --reward-distfs-adaptive-multiplier-signature-invalid <n> DistFS adaptive backoff multiplier for SIGNATURE_INVALID (default: 1)"
    );
    println!(
        "  --reward-distfs-adaptive-multiplier-unknown <n> DistFS adaptive backoff multiplier for UNKNOWN (default: 1)"
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
