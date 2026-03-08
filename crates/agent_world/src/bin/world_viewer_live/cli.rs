use std::collections::BTreeSet;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum NodeTopologyMode {
    Single,
    Triad,
    TriadDistributed,
}

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
    pub pos_slot_duration_ms: u64,
    pub pos_ticks_per_slot: u64,
    pub pos_proposal_tick_phase: u64,
    pub pos_adaptive_tick_scheduler_enabled: bool,
    pub pos_slot_clock_genesis_unix_ms: Option<i64>,
    pub pos_max_past_slot_lag: u64,
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
    pub reward_runtime_epoch_duration_secs: Option<u64>,
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
            pos_slot_duration_ms: 12_000,
            pos_ticks_per_slot: 10,
            pos_proposal_tick_phase: 9,
            pos_adaptive_tick_scheduler_enabled: false,
            pos_slot_clock_genesis_unix_ms: None,
            pos_max_past_slot_lag: 256,
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
            reward_runtime_epoch_duration_secs: None,
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

pub(super) fn parse_launch_options<'a>(
    args: impl Iterator<Item = &'a str>,
) -> Result<CliOptions, String> {
    let argv = args.map(str::to_string).collect::<Vec<_>>();
    for arg in &argv {
        if arg == "--release-config" || arg.starts_with("--node-") {
            return Err(removed_control_plane_flag_error(arg.as_str()));
        }
    }
    parse_options(argv.iter().map(|arg| arg.as_str()))
}

fn removed_control_plane_flag_error(flag: &str) -> String {
    format!(
        "{flag} is no longer supported by world_viewer_live; use world_chain_runtime (or world_game_launcher/world_web_launcher/agent_world_client_launcher) for chain control-plane options"
    )
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
            "--pos-slot-duration-ms" => {
                let raw = iter.next().ok_or_else(|| {
                    "--pos-slot-duration-ms requires a positive integer".to_string()
                })?;
                options.pos_slot_duration_ms = raw
                    .parse::<u64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or_else(|| {
                    "--pos-slot-duration-ms requires a positive integer".to_string()
                })?;
            }
            "--pos-ticks-per-slot" => {
                let raw = iter.next().ok_or_else(|| {
                    "--pos-ticks-per-slot requires a positive integer".to_string()
                })?;
                options.pos_ticks_per_slot = raw
                    .parse::<u64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or_else(|| {
                        "--pos-ticks-per-slot requires a positive integer".to_string()
                    })?;
            }
            "--pos-proposal-tick-phase" => {
                let raw = iter.next().ok_or_else(|| {
                    "--pos-proposal-tick-phase requires a non-negative integer".to_string()
                })?;
                options.pos_proposal_tick_phase = raw.parse::<u64>().map_err(|_| {
                    "--pos-proposal-tick-phase requires a non-negative integer".to_string()
                })?;
            }
            "--pos-adaptive-tick-scheduler" => {
                options.pos_adaptive_tick_scheduler_enabled = true;
            }
            "--pos-no-adaptive-tick-scheduler" => {
                options.pos_adaptive_tick_scheduler_enabled = false;
            }
            "--pos-slot-clock-genesis-unix-ms" => {
                let raw = iter.next().ok_or_else(|| {
                    "--pos-slot-clock-genesis-unix-ms requires an integer".to_string()
                })?;
                options.pos_slot_clock_genesis_unix_ms =
                    Some(raw.parse::<i64>().map_err(|_| {
                        "--pos-slot-clock-genesis-unix-ms requires an integer".to_string()
                    })?);
            }
            "--pos-max-past-slot-lag" => {
                let raw = iter.next().ok_or_else(|| {
                    "--pos-max-past-slot-lag requires a non-negative integer".to_string()
                })?;
                options.pos_max_past_slot_lag = raw.parse::<u64>().map_err(|_| {
                    "--pos-max-past-slot-lag requires a non-negative integer".to_string()
                })?;
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
            "--reward-runtime-epoch-duration-secs" => {
                let raw = iter.next().ok_or_else(|| {
                    "--reward-runtime-epoch-duration-secs requires a positive integer".to_string()
                })?;
                let value = raw
                    .parse::<u64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or_else(|| {
                        "--reward-runtime-epoch-duration-secs requires a positive integer"
                            .to_string()
                    })?;
                options.reward_runtime_epoch_duration_secs = Some(value);
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
    if options.pos_proposal_tick_phase >= options.pos_ticks_per_slot {
        return Err(format!(
            "--pos-proposal-tick-phase={} must be less than --pos-ticks-per-slot={}",
            options.pos_proposal_tick_phase, options.pos_ticks_per_slot
        ));
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
        "Usage: world_viewer_live [scenario] [--bind <addr>] [--web-bind <addr>] [--llm|--no-llm] [--no-node]"
    );
    println!("Options:");
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
    println!("  --pos-slot-duration-ms <n> PoS slot duration in milliseconds (default: 12000)");
    println!("  --pos-ticks-per-slot <n> PoS logical ticks per slot (default: 10)");
    println!("  --pos-proposal-tick-phase <n> Proposal phase within slot tick window (default: 9)");
    println!("  --pos-adaptive-tick-scheduler Enable adaptive wait to next logical tick boundary");
    println!("  --pos-no-adaptive-tick-scheduler Disable adaptive scheduler (default)");
    println!(
        "  --pos-slot-clock-genesis-unix-ms <n> Fixed slot clock genesis unix ms (default: auto)"
    );
    println!("  --pos-max-past-slot-lag <n> Max accepted inbound stale slot lag (default: 256)");
    println!("  --reward-runtime-enable Enable reward runtime settlement loop (default: off)");
    println!(
        "  --reward-runtime-auto-redeem Auto redeem minted credits to node-mapped runtime agent"
    );
    println!("  --reward-runtime-signer <node_id> Settlement signer node id (default: embedded node id)");
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
        "  --reward-runtime-epoch-duration-secs <n> Override reward settlement epoch duration seconds (default: 3600)"
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
