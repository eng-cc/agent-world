use std::collections::BTreeSet;
use std::fs;
use std::net::SocketAddr;
use std::process;

use agent_world::runtime::RewardAssetConfig;
use agent_world::simulator::WorldScenario;
use agent_world_node::{NodeRole, PosValidator};
use serde::Deserialize;

use super::{
    parse_distfs_probe_runtime_option, DistfsProbeRuntimeConfig,
    DEFAULT_REWARD_RUNTIME_MIN_OBSERVER_TRACES, DEFAULT_REWARD_RUNTIME_REPORT_DIR,
    DEFAULT_REWARD_RUNTIME_RESERVE_UNITS,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NodeTopologyMode {
    Single,
    Triad,
    TriadDistributed,
}

const RELEASE_CONFIG_FLAG: &str = "--release-config";

impl NodeTopologyMode {
    fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "single" => Ok(Self::Single),
            "triad" => Ok(Self::Triad),
            "triad_distributed" => Ok(Self::TriadDistributed),
            _ => Err("--topology must be one of: single, triad, triad_distributed".to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct CliOptions {
    pub scenario: WorldScenario,
    pub bind_addr: String,
    pub web_bind_addr: Option<String>,
    pub llm_mode: bool,
    pub node_topology: NodeTopologyMode,
    pub triad_gossip_base_port: u16,
    pub triad_distributed_sequencer_gossip: Option<SocketAddr>,
    pub triad_distributed_storage_gossip: Option<SocketAddr>,
    pub triad_distributed_observer_gossip: Option<SocketAddr>,
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
    pub reward_runtime_leader_node_id: Option<String>,
    pub reward_runtime_leader_stale_ms: u64,
    pub reward_runtime_failover_enabled: bool,
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
            llm_mode: true,
            node_topology: NodeTopologyMode::Triad,
            triad_gossip_base_port: 5500,
            triad_distributed_sequencer_gossip: None,
            triad_distributed_storage_gossip: None,
            triad_distributed_observer_gossip: None,
            viewer_consensus_gate: true,
            node_enabled: true,
            node_id: "viewer-live-node".to_string(),
            node_role: NodeRole::Observer,
            node_tick_ms: 200,
            node_auto_attest_all_validators: false,
            node_validators: Vec::new(),
            node_gossip_bind: None,
            node_gossip_peers: Vec::new(),
            node_repl_libp2p_listen: Vec::new(),
            node_repl_libp2p_peers: Vec::new(),
            node_repl_topic: None,
            reward_runtime_enabled: false,
            reward_runtime_auto_redeem: false,
            reward_runtime_signer_node_id: None,
            reward_runtime_leader_node_id: None,
            reward_runtime_leader_stale_ms: 3_000,
            reward_runtime_failover_enabled: true,
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

#[derive(Debug, Deserialize)]
struct ReleaseLockedArgsFile {
    locked_args: Vec<String>,
}

#[derive(Debug, Default)]
struct ReleaseModeCommandLine {
    release_config_path: String,
    bind_addr_override: Option<String>,
    web_bind_addr_override: Option<String>,
}

pub(super) fn parse_launch_options<'a>(
    args: impl Iterator<Item = &'a str>,
) -> Result<CliOptions, String> {
    let argv = args.map(str::to_string).collect::<Vec<_>>();
    if !argv.iter().any(|arg| arg == RELEASE_CONFIG_FLAG) {
        return parse_options(argv.iter().map(|arg| arg.as_str()));
    }

    let mode = parse_release_mode_command_line(argv.as_slice())?;
    let mut options = load_release_locked_options(mode.release_config_path.as_str())?;
    if let Some(bind_addr) = mode.bind_addr_override {
        options.bind_addr = bind_addr;
    }
    if let Some(web_bind_addr) = mode.web_bind_addr_override {
        options.web_bind_addr = Some(web_bind_addr);
    }
    Ok(options)
}

fn parse_release_mode_command_line(args: &[String]) -> Result<ReleaseModeCommandLine, String> {
    let mut mode = ReleaseModeCommandLine::default();
    let mut release_config_seen = false;
    let mut unsupported = Vec::new();
    let mut index = 0usize;

    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => {
                print_help();
                process::exit(0);
            }
            RELEASE_CONFIG_FLAG => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--release-config requires <path>".to_string())?;
                let value = value.trim();
                if value.is_empty() {
                    return Err("--release-config requires non-empty <path>".to_string());
                }
                if release_config_seen {
                    return Err("--release-config can only be specified once".to_string());
                }
                mode.release_config_path = value.to_string();
                release_config_seen = true;
                index += 2;
            }
            "--bind" => {
                mode.bind_addr_override =
                    Some(parse_release_mode_override_value(args, index, "--bind")?);
                index += 2;
            }
            "--web-bind" => {
                mode.web_bind_addr_override = Some(parse_release_mode_override_value(
                    args,
                    index,
                    "--web-bind",
                )?);
                index += 2;
            }
            _ => {
                unsupported.push(args[index].clone());
                index += 1;
            }
        }
    }

    if !release_config_seen {
        return Err("internal error: --release-config mode expected".to_string());
    }
    if !unsupported.is_empty() {
        return Err(format!(
            "--release-config mode only allows --bind/--web-bind/--help; unsupported argument(s): {}",
            unsupported.join(", ")
        ));
    }
    Ok(mode)
}

fn parse_release_mode_override_value(
    args: &[String],
    index: usize,
    flag: &str,
) -> Result<String, String> {
    let value = args
        .get(index + 1)
        .ok_or_else(|| format!("{flag} requires a value in --release-config mode"))?;
    let value = value.trim();
    if value.is_empty() {
        return Err(format!(
            "{flag} requires non-empty value in --release-config mode"
        ));
    }
    Ok(value.to_string())
}

fn load_release_locked_options(path: &str) -> Result<CliOptions, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read release config `{path}`: {err}"))?;
    let file: ReleaseLockedArgsFile = toml::from_str(text.as_str())
        .map_err(|err| format!("failed to parse release config `{path}`: {err}"))?;
    if file.locked_args.is_empty() {
        return Err(format!(
            "release config `{path}` must contain non-empty locked_args"
        ));
    }
    if file
        .locked_args
        .iter()
        .any(|arg| arg == RELEASE_CONFIG_FLAG || arg == "--help" || arg == "-h")
    {
        return Err(format!(
            "release config `{path}` locked_args cannot contain --release-config/--help/-h"
        ));
    }

    parse_options(file.locked_args.iter().map(|arg| arg.as_str()))
        .map_err(|err| format!("invalid locked_args in release config `{path}`: {err}"))
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
            "--no-llm" => {
                options.llm_mode = false;
            }
            "--topology" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| "--topology requires a value".to_string())?;
                options.node_topology = NodeTopologyMode::parse(raw)?;
            }
            "--triad-gossip-base-port" => {
                let raw = iter.next().ok_or_else(|| {
                    "--triad-gossip-base-port requires an integer in [1, 65533]".to_string()
                })?;
                options.triad_gossip_base_port = raw
                    .parse::<u16>()
                    .ok()
                    .filter(|value| (1..=65533).contains(value))
                    .ok_or_else(|| {
                        "--triad-gossip-base-port requires an integer in [1, 65533]".to_string()
                    })?;
            }
            "--triad-sequencer-gossip" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| "--triad-sequencer-gossip requires <addr:port>".to_string())?;
                options.triad_distributed_sequencer_gossip =
                    Some(parse_socket_addr(raw, "--triad-sequencer-gossip")?);
            }
            "--triad-storage-gossip" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| "--triad-storage-gossip requires <addr:port>".to_string())?;
                options.triad_distributed_storage_gossip =
                    Some(parse_socket_addr(raw, "--triad-storage-gossip")?);
            }
            "--triad-observer-gossip" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| "--triad-observer-gossip requires <addr:port>".to_string())?;
                options.triad_distributed_observer_gossip =
                    Some(parse_socket_addr(raw, "--triad-observer-gossip")?);
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
            "--node-auto-attest-all" => {
                options.node_auto_attest_all_validators = true;
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
            "--reward-runtime-leader-node" => {
                let leader = iter
                    .next()
                    .ok_or_else(|| "--reward-runtime-leader-node requires <node_id>".to_string())?;
                let leader = leader.trim();
                if leader.is_empty() {
                    return Err(
                        "--reward-runtime-leader-node requires non-empty <node_id>".to_string()
                    );
                }
                options.reward_runtime_leader_node_id = Some(leader.to_string());
            }
            "--reward-runtime-leader-stale-ms" => {
                let raw = iter.next().ok_or_else(|| {
                    "--reward-runtime-leader-stale-ms requires a positive integer".to_string()
                })?;
                options.reward_runtime_leader_stale_ms = raw
                    .parse::<u64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or_else(|| {
                        "--reward-runtime-leader-stale-ms requires a positive integer".to_string()
                    })?;
            }
            "--reward-runtime-no-failover" => {
                options.reward_runtime_failover_enabled = false;
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
    if options.node_topology == NodeTopologyMode::Triad {
        if !options.node_enabled {
            return Err(
                "--topology triad requires embedded node runtime; remove --no-node or use --topology single"
                    .to_string(),
            );
        }
        if !options.viewer_consensus_gate {
            return Err(
                "--topology triad requires viewer consensus gate; remove --viewer-no-consensus-gate or use --topology single"
                    .to_string(),
            );
        }
        if options.node_role != NodeRole::Observer {
            return Err("--node-role is only supported in --topology single".to_string());
        }
        if !options.node_validators.is_empty() {
            return Err("--node-validator is only supported in --topology single".to_string());
        }
        if options.node_gossip_bind.is_some() || !options.node_gossip_peers.is_empty() {
            return Err("--node-gossip-* is only supported in --topology single".to_string());
        }
        if !options.node_repl_libp2p_listen.is_empty() || !options.node_repl_libp2p_peers.is_empty()
        {
            return Err("--node-repl-libp2p-* is only supported in --topology single".to_string());
        }
        if options.triad_distributed_sequencer_gossip.is_some()
            || options.triad_distributed_storage_gossip.is_some()
            || options.triad_distributed_observer_gossip.is_some()
        {
            return Err(
                "--triad-*-gossip is only supported in --topology triad_distributed".to_string(),
            );
        }
    }
    if options.node_topology == NodeTopologyMode::TriadDistributed {
        if !options.node_enabled {
            return Err(
                "--topology triad_distributed requires embedded node runtime; remove --no-node or use --topology single"
                    .to_string(),
            );
        }
        if !options.viewer_consensus_gate {
            return Err(
                "--topology triad_distributed requires viewer consensus gate; remove --viewer-no-consensus-gate or use --topology single"
                    .to_string(),
            );
        }
        if !options.node_validators.is_empty() {
            return Err("--node-validator is only supported in --topology single".to_string());
        }
        if options.node_gossip_bind.is_some() || !options.node_gossip_peers.is_empty() {
            return Err("--node-gossip-* is only supported in --topology single".to_string());
        }
        let _ = resolve_triad_distributed_gossip(&options, options.node_role)?;
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
        "Usage: world_viewer_live [scenario] [--bind <addr>] [--web-bind <addr>] [--llm|--no-llm] [--no-node] [--node-validator <id:stake>...] [--node-gossip-bind <addr:port>] [--node-gossip-peer <addr:port>...]"
    );
    println!("Options:");
    println!("  --release-config <path> Enable release-locked launch from TOML locked_args");
    println!("  --bind <addr>     Bind address (default: 127.0.0.1:5010)");
    println!("  --web-bind <addr> WebSocket bridge bind address (optional)");
    println!("  --scenario <name> Scenario name (default: twin_region_bootstrap)");
    println!("  --llm             Enable LLM decisions (default)");
    println!("  --no-llm          Disable LLM decisions and use built-in script");
    println!(
        "  --topology <mode> Node topology mode: single|triad|triad_distributed (default: triad)"
    );
    println!("  --triad-gossip-base-port <port> Triad gossip base UDP port (default: 5500)");
    println!(
        "  --triad-sequencer-gossip <addr:port> Triad distributed sequencer gossip bind/bootstrap (required for all roles)"
    );
    println!(
        "  --triad-storage-gossip <addr:port> Triad distributed storage gossip bind (required when --node-role storage)"
    );
    println!(
        "  --triad-observer-gossip <addr:port> Triad distributed observer gossip bind (required when --node-role observer)"
    );
    println!(
        "  --viewer-no-consensus-gate Disable viewer tick gating by node consensus/execution height"
    );
    println!("  --no-node         Disable embedded node runtime startup");
    println!("  --node-id <id>    Node identifier (default: viewer-live-node)");
    println!("  --node-role <r>   Node role: sequencer|storage|observer (default: observer)");
    println!("  --node-tick-ms <ms> Node runtime tick interval (default: 200)");
    println!("  --node-validator <id:stake> Add PoS validator stake (repeatable)");
    println!(
        "  --node-no-auto-attest-all Disable auto-attesting all validators per tick (default)"
    );
    println!("  --node-auto-attest-all Enable auto-attesting all validators per tick");
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
    println!(
        "  --reward-runtime-leader-node <node_id> Settlement publisher leader node id (default: inferred sequencer id)"
    );
    println!(
        "  --reward-runtime-leader-stale-ms <n> Leader commit staleness window before failover publish (default: 3000)"
    );
    println!(
        "  --reward-runtime-no-failover Disable settlement publish failover when leader is stale"
    );
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

fn triad_distributed_missing_addr_error(role: NodeRole, flag: &str) -> String {
    format!(
        "--topology triad_distributed with --node-role {} requires {flag} <addr:port>",
        role.as_str()
    )
}

pub(super) fn resolve_triad_distributed_gossip(
    options: &CliOptions,
    role: NodeRole,
) -> Result<(SocketAddr, Vec<SocketAddr>), String> {
    let (bind_addr, mut peers) = match role {
        NodeRole::Sequencer => (
            options.triad_distributed_sequencer_gossip.ok_or_else(|| {
                triad_distributed_missing_addr_error(role, "--triad-sequencer-gossip")
            })?,
            vec![
                options.triad_distributed_storage_gossip,
                options.triad_distributed_observer_gossip,
            ],
        ),
        NodeRole::Storage => {
            let bind_addr = options.triad_distributed_storage_gossip.ok_or_else(|| {
                triad_distributed_missing_addr_error(role, "--triad-storage-gossip")
            })?;
            let sequencer = options.triad_distributed_sequencer_gossip.ok_or_else(|| {
                triad_distributed_missing_addr_error(role, "--triad-sequencer-gossip")
            })?;
            if sequencer == bind_addr {
                return Err(
                    "--topology triad_distributed requires --triad-storage-gossip distinct from --triad-sequencer-gossip"
                        .to_string(),
                );
            }
            (
                bind_addr,
                vec![Some(sequencer), options.triad_distributed_observer_gossip],
            )
        }
        NodeRole::Observer => {
            let bind_addr = options.triad_distributed_observer_gossip.ok_or_else(|| {
                triad_distributed_missing_addr_error(role, "--triad-observer-gossip")
            })?;
            let sequencer = options.triad_distributed_sequencer_gossip.ok_or_else(|| {
                triad_distributed_missing_addr_error(role, "--triad-sequencer-gossip")
            })?;
            if sequencer == bind_addr {
                return Err(
                    "--topology triad_distributed requires --triad-observer-gossip distinct from --triad-sequencer-gossip"
                        .to_string(),
                );
            }
            (
                bind_addr,
                vec![Some(sequencer), options.triad_distributed_storage_gossip],
            )
        }
    };

    let mut dedup = BTreeSet::new();
    peers.retain(Option::is_some);
    let peers = peers
        .into_iter()
        .flatten()
        .filter(|peer| *peer != bind_addr)
        .filter(|peer| dedup.insert(*peer))
        .collect::<Vec<_>>();
    Ok((bind_addr, peers))
}
