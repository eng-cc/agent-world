use super::*;
use agent_world::runtime::BlobStore;
use agent_world::runtime::LocalCasStore;
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

fn temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("duration")
        .as_nanos();
    std::env::temp_dir().join(format!("agent-world-{prefix}-{unique}"))
}

fn write_release_locked_args_file(prefix: &str, locked_args: &[&str]) -> PathBuf {
    let path = temp_config_path(prefix);
    let args = locked_args
        .iter()
        .map(|arg| format!("\"{}\"", arg.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(", ");
    let body = format!("locked_args = [{args}]\n");
    fs::write(path.as_path(), body).expect("write release config");
    path
}

fn unique_triad_gossip_base_port() -> u16 {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("duration")
        .as_nanos() as u64;
    let port = 20_000 + (unique % 40_000) as u16;
    port.min(65_533)
}

fn failover_test_snapshot(
    local_node_id: &str,
    local_height: u64,
    local_committed_at_ms: Option<i64>,
    peer_heads: Vec<agent_world_node::NodePeerCommittedHead>,
) -> agent_world_node::NodeSnapshot {
    let mut consensus = agent_world_node::NodeConsensusSnapshot::default();
    consensus.committed_height = local_height;
    consensus.last_committed_at_ms = local_committed_at_ms;
    consensus.known_peer_heads = peer_heads.len();
    consensus.peer_heads = peer_heads;
    agent_world_node::NodeSnapshot {
        node_id: local_node_id.to_string(),
        player_id: local_node_id.to_string(),
        world_id: "world-failover-test".to_string(),
        role: NodeRole::Observer,
        running: true,
        tick_count: local_height,
        last_tick_unix_ms: local_committed_at_ms,
        consensus,
        last_error: None,
    }
}

#[test]
fn infer_default_reward_runtime_leader_node_id_maps_triad_roles() {
    assert_eq!(
        infer_default_reward_runtime_leader_node_id("room-1-sequencer"),
        "room-1-sequencer"
    );
    assert_eq!(
        infer_default_reward_runtime_leader_node_id("room-1-storage"),
        "room-1-sequencer"
    );
    assert_eq!(
        infer_default_reward_runtime_leader_node_id("room-1-observer"),
        "room-1-sequencer"
    );
    assert_eq!(
        infer_default_reward_runtime_leader_node_id("single-node"),
        "single-node"
    );
}

#[test]
fn select_failover_publisher_node_id_prefers_height_then_time_then_node_id() {
    let snapshot = failover_test_snapshot(
        "room-1-storage",
        9,
        Some(9_000),
        vec![
            agent_world_node::NodePeerCommittedHead {
                node_id: "room-1-sequencer".to_string(),
                height: 10,
                block_hash: "leader-block-10".to_string(),
                committed_at_ms: 10_000,
                execution_block_hash: Some("exec-leader-10".to_string()),
                execution_state_root: Some("state-leader-10".to_string()),
            },
            agent_world_node::NodePeerCommittedHead {
                node_id: "room-1-observer".to_string(),
                height: 10,
                block_hash: "observer-block-10".to_string(),
                committed_at_ms: 11_000,
                execution_block_hash: Some("exec-observer-10".to_string()),
                execution_state_root: Some("state-observer-10".to_string()),
            },
            agent_world_node::NodePeerCommittedHead {
                node_id: "room-1-archiver".to_string(),
                height: 10,
                block_hash: "archiver-block-10".to_string(),
                committed_at_ms: 11_000,
                execution_block_hash: Some("exec-archiver-10".to_string()),
                execution_state_root: Some("state-archiver-10".to_string()),
            },
        ],
    );
    let selected = select_failover_publisher_node_id(&snapshot, "room-1-sequencer");
    assert_eq!(selected.as_deref(), Some("room-1-archiver"));
}

#[test]
fn select_failover_publisher_node_id_returns_local_when_leader_excluded() {
    let snapshot = failover_test_snapshot(
        "room-2-storage",
        12,
        Some(12_000),
        vec![agent_world_node::NodePeerCommittedHead {
            node_id: "room-2-sequencer".to_string(),
            height: 13,
            block_hash: "leader-block-13".to_string(),
            committed_at_ms: 13_000,
            execution_block_hash: Some("exec-leader-13".to_string()),
            execution_state_root: Some("state-leader-13".to_string()),
        }],
    );
    let selected = select_failover_publisher_node_id(&snapshot, "room-2-sequencer");
    assert_eq!(selected.as_deref(), Some("room-2-storage"));
}

#[test]
fn parse_options_defaults() {
    let options = parse_options([].into_iter()).expect("defaults");
    assert_eq!(options.scenario, WorldScenario::TwinRegionBootstrap);
    assert_eq!(options.bind_addr, "127.0.0.1:5010");
    assert!(options.web_bind_addr.is_none());
    assert!(options.llm_mode);
    assert_eq!(options.node_topology, NodeTopologyMode::Triad);
    assert_eq!(options.triad_gossip_base_port, 5500);
    assert!(options.triad_distributed_sequencer_gossip.is_none());
    assert!(options.triad_distributed_storage_gossip.is_none());
    assert!(options.triad_distributed_observer_gossip.is_none());
    assert!(options.viewer_consensus_gate);
    assert!(options.node_enabled);
    assert_eq!(options.node_id, "viewer-live-node");
    assert_eq!(options.node_role, NodeRole::Observer);
    assert_eq!(options.node_tick_ms, 200);
    assert!(!options.node_auto_attest_all_validators);
    assert!(options.node_validators.is_empty());
    assert!(options.node_gossip_bind.is_none());
    assert!(options.node_gossip_peers.is_empty());
    assert!(options.node_repl_libp2p_listen.is_empty());
    assert!(options.node_repl_libp2p_peers.is_empty());
    assert!(options.node_repl_topic.is_none());
    assert!(!options.reward_runtime_enabled);
    assert!(!options.reward_runtime_auto_redeem);
    assert!(options.reward_runtime_signer_node_id.is_none());
    assert!(options.reward_runtime_leader_node_id.is_none());
    assert_eq!(options.reward_runtime_leader_stale_ms, 3_000);
    assert!(options.reward_runtime_failover_enabled);
    assert_eq!(
        options.reward_runtime_report_dir,
        DEFAULT_REWARD_RUNTIME_REPORT_DIR
    );
    assert_eq!(
        options.reward_runtime_min_observer_traces,
        DEFAULT_REWARD_RUNTIME_MIN_OBSERVER_TRACES
    );
    assert!(options.reward_runtime_epoch_duration_secs.is_none());
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
    assert_eq!(
        options.reward_distfs_probe_config,
        DistfsProbeRuntimeConfig::default()
    );
}

#[test]
fn parse_options_enables_llm_mode() {
    let options = parse_options(["--llm"].into_iter()).expect("llm mode");
    assert!(options.llm_mode);
}

#[test]
fn parse_options_disables_llm_mode() {
    let options = parse_options(["--no-llm"].into_iter()).expect("script mode");
    assert!(!options.llm_mode);
}

#[test]
fn parse_options_llm_flags_follow_last_one_wins() {
    let options = parse_options(["--no-llm", "--llm"].into_iter()).expect("llm after no-llm");
    assert!(options.llm_mode);

    let options = parse_options(["--llm", "--no-llm"].into_iter()).expect("no-llm after llm");
    assert!(!options.llm_mode);
}

#[test]
fn parse_options_enables_auto_attest_all_when_requested() {
    let options = parse_options(["--node-auto-attest-all"].into_iter()).expect("auto attest");
    assert!(options.node_auto_attest_all_validators);
}

#[test]
fn parse_options_reads_custom_values() {
    let options = parse_options(
        [
            "llm_bootstrap",
            "--topology",
            "single",
            "--bind",
            "127.0.0.1:9001",
            "--web-bind",
            "127.0.0.1:9002",
            "--viewer-no-consensus-gate",
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
            "--reward-runtime-leader-node",
            "reward-leader-1",
            "--reward-runtime-leader-stale-ms",
            "4500",
            "--reward-runtime-no-failover",
            "--reward-runtime-report-dir",
            "output/reward-custom",
            "--reward-runtime-min-observer-traces",
            "3",
            "--reward-runtime-epoch-duration-secs",
            "45",
            "--reward-distfs-probe-max-sample-bytes",
            "8192",
            "--reward-distfs-probe-per-tick",
            "3",
            "--reward-distfs-probe-ttl-ms",
            "45000",
            "--reward-distfs-probe-allowed-clock-skew-ms",
            "2000",
            "--reward-distfs-adaptive-max-checks-per-round",
            "9",
            "--reward-distfs-adaptive-backoff-base-ms",
            "250",
            "--reward-distfs-adaptive-backoff-max-ms",
            "2500",
            "--reward-distfs-adaptive-multiplier-hash-mismatch",
            "5",
            "--reward-distfs-adaptive-multiplier-missing-sample",
            "2",
            "--reward-distfs-adaptive-multiplier-timeout",
            "3",
            "--reward-distfs-adaptive-multiplier-read-io-error",
            "4",
            "--reward-distfs-adaptive-multiplier-signature-invalid",
            "6",
            "--reward-distfs-adaptive-multiplier-unknown",
            "7",
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
    assert_eq!(options.node_topology, NodeTopologyMode::Single);
    assert!(!options.viewer_consensus_gate);
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
    assert_eq!(
        options.reward_runtime_leader_node_id.as_deref(),
        Some("reward-leader-1")
    );
    assert_eq!(options.reward_runtime_leader_stale_ms, 4500);
    assert!(!options.reward_runtime_failover_enabled);
    assert_eq!(options.reward_runtime_report_dir, "output/reward-custom");
    assert_eq!(options.reward_runtime_min_observer_traces, 3);
    assert_eq!(options.reward_runtime_epoch_duration_secs, Some(45));
    assert_eq!(options.reward_points_per_credit, 7);
    assert_eq!(options.reward_credits_per_power_unit, 3);
    assert_eq!(options.reward_max_redeem_power_per_epoch, 1200);
    assert_eq!(options.reward_min_redeem_power_unit, 2);
    assert_eq!(options.reward_initial_reserve_power_units, 888);
    assert_eq!(options.reward_distfs_probe_config.max_sample_bytes, 8192);
    assert_eq!(options.reward_distfs_probe_config.challenges_per_tick, 3);
    assert_eq!(options.reward_distfs_probe_config.challenge_ttl_ms, 45000);
    assert_eq!(
        options.reward_distfs_probe_config.allowed_clock_skew_ms,
        2000
    );
    assert_eq!(
        options
            .reward_distfs_probe_config
            .adaptive_policy
            .max_checks_per_round,
        9
    );
    assert_eq!(
        options
            .reward_distfs_probe_config
            .adaptive_policy
            .failure_backoff_base_ms,
        250
    );
    assert_eq!(
        options
            .reward_distfs_probe_config
            .adaptive_policy
            .failure_backoff_max_ms,
        2500
    );
    assert_eq!(
        options
            .reward_distfs_probe_config
            .adaptive_policy
            .backoff_multiplier_hash_mismatch,
        5
    );
    assert_eq!(
        options
            .reward_distfs_probe_config
            .adaptive_policy
            .backoff_multiplier_missing_sample,
        2
    );
    assert_eq!(
        options
            .reward_distfs_probe_config
            .adaptive_policy
            .backoff_multiplier_timeout,
        3
    );
    assert_eq!(
        options
            .reward_distfs_probe_config
            .adaptive_policy
            .backoff_multiplier_read_io_error,
        4
    );
    assert_eq!(
        options
            .reward_distfs_probe_config
            .adaptive_policy
            .backoff_multiplier_signature_invalid,
        6
    );
    assert_eq!(
        options
            .reward_distfs_probe_config
            .adaptive_policy
            .backoff_multiplier_unknown,
        7
    );
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
fn parse_launch_options_release_config_loads_locked_args() {
    let path = write_release_locked_args_file(
        "release-locked-load",
        &[
            "llm_bootstrap",
            "--topology",
            "single",
            "--node-id",
            "release-node-a",
            "--node-role",
            "observer",
        ],
    );

    let options =
        parse_launch_options(["--release-config", path.to_string_lossy().as_ref()].into_iter())
            .expect("release launch options");
    assert_eq!(options.scenario, WorldScenario::LlmBootstrap);
    assert_eq!(options.node_topology, NodeTopologyMode::Single);
    assert_eq!(options.node_id, "release-node-a");
    assert_eq!(options.node_role, NodeRole::Observer);
    assert!(options.llm_mode);
    assert_eq!(options.bind_addr, "127.0.0.1:5010");
    assert!(options.web_bind_addr.is_none());

    let _ = fs::remove_file(path.as_path());
}

#[test]
fn parse_launch_options_release_config_respects_locked_no_llm() {
    let path = write_release_locked_args_file(
        "release-locked-no-llm",
        &[
            "llm_bootstrap",
            "--topology",
            "single",
            "--node-id",
            "release-node-script",
            "--node-role",
            "observer",
            "--no-llm",
        ],
    );

    let options =
        parse_launch_options(["--release-config", path.to_string_lossy().as_ref()].into_iter())
            .expect("release launch options");
    assert!(!options.llm_mode);

    let _ = fs::remove_file(path.as_path());
}

#[test]
fn parse_launch_options_release_config_allows_bind_overrides() {
    let path = write_release_locked_args_file(
        "release-locked-bind-overrides",
        &[
            "llm_bootstrap",
            "--topology",
            "single",
            "--bind",
            "127.0.0.1:6101",
            "--web-bind",
            "127.0.0.1:6102",
        ],
    );

    let options = parse_launch_options(
        [
            "--release-config",
            path.to_string_lossy().as_ref(),
            "--bind",
            "127.0.0.1:7201",
            "--web-bind",
            "127.0.0.1:7202",
        ]
        .into_iter(),
    )
    .expect("release launch with overrides");
    assert_eq!(options.bind_addr, "127.0.0.1:7201");
    assert_eq!(options.web_bind_addr.as_deref(), Some("127.0.0.1:7202"));

    let _ = fs::remove_file(path.as_path());
}

#[test]
fn parse_launch_options_release_config_rejects_disallowed_flags() {
    let path = write_release_locked_args_file(
        "release-locked-reject-flags",
        &["llm_bootstrap", "--topology", "single"],
    );

    let err = parse_launch_options(
        [
            "--release-config",
            path.to_string_lossy().as_ref(),
            "--node-tick-ms",
            "10",
        ]
        .into_iter(),
    )
    .expect_err("release config should reject non-whitelist flags");
    assert!(err.contains("--release-config mode only allows"));
    assert!(err.contains("--node-tick-ms"));

    let _ = fs::remove_file(path.as_path());
}

#[test]
fn parse_options_rejects_zero_reward_distfs_probe_per_tick() {
    let err = parse_options(["--reward-distfs-probe-per-tick", "0"].into_iter())
        .expect_err("reject zero probe per tick");
    assert!(err.contains("--reward-distfs-probe-per-tick"));
}

#[test]
fn parse_options_rejects_negative_reward_distfs_probe_allowed_clock_skew_ms() {
    let err = parse_options(["--reward-distfs-probe-allowed-clock-skew-ms", "-1"].into_iter())
        .expect_err("reject negative probe clock skew");
    assert!(err.contains("--reward-distfs-probe-allowed-clock-skew-ms"));
}

#[test]
fn parse_options_rejects_zero_reward_distfs_adaptive_max_checks_per_round() {
    let err = parse_options(["--reward-distfs-adaptive-max-checks-per-round", "0"].into_iter())
        .expect_err("reject zero adaptive max checks");
    assert!(err.contains("--reward-distfs-adaptive-max-checks-per-round"));
}

#[test]
fn parse_options_rejects_reward_distfs_adaptive_backoff_max_less_than_base() {
    let err = parse_options(
        [
            "--reward-distfs-adaptive-backoff-base-ms",
            "200",
            "--reward-distfs-adaptive-backoff-max-ms",
            "100",
        ]
        .into_iter(),
    )
    .expect_err("reject max < base");
    assert!(err.contains("--reward-distfs-adaptive-backoff-max-ms"));
}

#[test]
fn parse_options_rejects_zero_reward_distfs_adaptive_multiplier_hash_mismatch() {
    let err = parse_options(["--reward-distfs-adaptive-multiplier-hash-mismatch", "0"].into_iter())
        .expect_err("reject zero hash mismatch multiplier");
    assert!(err.contains("--reward-distfs-adaptive-multiplier-hash-mismatch"));
}

#[test]
fn parse_options_rejects_zero_reward_distfs_adaptive_multiplier_unknown() {
    let err = parse_options(["--reward-distfs-adaptive-multiplier-unknown", "0"].into_iter())
        .expect_err("reject zero unknown multiplier");
    assert!(err.contains("--reward-distfs-adaptive-multiplier-unknown"));
}

#[test]
fn parse_options_disables_node() {
    let options = parse_options(["--topology", "single", "--no-node"].into_iter()).expect("parse");
    assert!(!options.node_enabled);
    assert!(!options.viewer_consensus_gate);
}

#[test]
fn parse_options_disables_consensus_gate_when_requested() {
    let options = parse_options(["--topology", "single", "--viewer-no-consensus-gate"].into_iter())
        .expect("parse no gate");
    assert!(!options.viewer_consensus_gate);
    assert!(options.node_enabled);
}

#[test]
fn parse_options_rejects_no_node_in_triad_topology() {
    let err = parse_options(["--no-node"].into_iter()).expect_err("triad requires node");
    assert!(err.contains("--topology triad"));
}

#[test]
fn parse_options_rejects_no_consensus_gate_in_triad_topology() {
    let err = parse_options(["--viewer-no-consensus-gate"].into_iter())
        .expect_err("triad requires consensus gate");
    assert!(err.contains("--topology triad"));
}

#[test]
fn parse_options_reads_triad_distributed_values() {
    let options = parse_options(
        [
            "--topology",
            "triad_distributed",
            "--node-role",
            "storage",
            "--triad-sequencer-gossip",
            "127.0.0.1:7301",
            "--triad-storage-gossip",
            "127.0.0.1:7302",
        ]
        .into_iter(),
    )
    .expect("triad distributed options");
    assert_eq!(options.node_topology, NodeTopologyMode::TriadDistributed);
    assert_eq!(options.node_role, NodeRole::Storage);
    assert_eq!(
        options.triad_distributed_sequencer_gossip,
        Some("127.0.0.1:7301".parse::<SocketAddr>().expect("addr"))
    );
    assert_eq!(
        options.triad_distributed_storage_gossip,
        Some("127.0.0.1:7302".parse::<SocketAddr>().expect("addr"))
    );
    assert!(options.triad_distributed_observer_gossip.is_none());
}

#[test]
fn parse_options_rejects_triad_distributed_when_role_addrs_missing() {
    let err = parse_options(
        [
            "--topology",
            "triad_distributed",
            "--triad-sequencer-gossip",
            "127.0.0.1:7301",
            "--triad-storage-gossip",
            "127.0.0.1:7302",
        ]
        .into_iter(),
    )
    .expect_err("missing observer gossip");
    assert!(err.contains("--triad-observer-gossip"));
}

#[test]
fn parse_options_allows_triad_distributed_sequencer_without_static_peers() {
    let options = parse_options(
        [
            "--topology",
            "triad_distributed",
            "--node-role",
            "sequencer",
            "--triad-sequencer-gossip",
            "127.0.0.1:7311",
        ]
        .into_iter(),
    )
    .expect("minimal sequencer gossip config");
    assert_eq!(options.node_topology, NodeTopologyMode::TriadDistributed);
    assert_eq!(options.node_role, NodeRole::Sequencer);
    assert_eq!(
        options.triad_distributed_sequencer_gossip,
        Some("127.0.0.1:7311".parse::<SocketAddr>().expect("addr"))
    );
    assert!(options.triad_distributed_storage_gossip.is_none());
    assert!(options.triad_distributed_observer_gossip.is_none());
}

#[test]
fn parse_options_rejects_triad_distributed_storage_without_sequencer_bootstrap() {
    let err = parse_options(
        [
            "--topology",
            "triad_distributed",
            "--node-role",
            "storage",
            "--triad-storage-gossip",
            "127.0.0.1:7312",
        ]
        .into_iter(),
    )
    .expect_err("storage role requires sequencer bootstrap");
    assert!(err.contains("--triad-sequencer-gossip"));
}

#[test]
fn parse_options_rejects_reward_runtime_with_no_node() {
    let err = parse_options(
        [
            "--topology",
            "single",
            "--no-node",
            "--reward-runtime-enable",
        ]
        .into_iter(),
    )
    .expect_err("reward runtime requires node");
    assert!(err.contains("--reward-runtime-enable"));
}

#[test]
fn parse_options_rejects_zero_reward_runtime_min_observer_traces() {
    let err = parse_options(["--reward-runtime-min-observer-traces", "0"].into_iter())
        .expect_err("reject zero min observer traces");
    assert!(err.contains("--reward-runtime-min-observer-traces"));
}

#[test]
fn parse_options_rejects_zero_reward_runtime_epoch_duration_secs() {
    let err = parse_options(["--reward-runtime-epoch-duration-secs", "0"].into_iter())
        .expect_err("reject zero epoch duration");
    assert!(err.contains("--reward-runtime-epoch-duration-secs"));
}

#[test]
fn parse_options_rejects_zero_reward_runtime_leader_stale_ms() {
    let err = parse_options(["--reward-runtime-leader-stale-ms", "0"].into_iter())
        .expect_err("reject zero leader stale");
    assert!(err.contains("--reward-runtime-leader-stale-ms"));
}

#[test]
fn reward_invariant_status_payload_reflects_violation_count() {
    let clean = RewardAssetInvariantReport::default();
    let clean_payload = reward_invariant_status_payload(&clean);
    assert_eq!(
        clean_payload.get("ok").and_then(|value| value.as_bool()),
        Some(true)
    );
    assert_eq!(
        clean_payload
            .get("violation_count")
            .and_then(|value| value.as_u64()),
        Some(0)
    );

    let mut violated = RewardAssetInvariantReport::default();
    violated
        .violations
        .push(agent_world::runtime::RewardAssetInvariantViolation {
            code: "mint_signature_invalid".to_string(),
            message: "tampered".to_string(),
        });
    let violated_payload = reward_invariant_status_payload(&violated);
    assert_eq!(
        violated_payload.get("ok").and_then(|value| value.as_bool()),
        Some(false)
    );
    assert_eq!(
        violated_payload
            .get("violation_count")
            .and_then(|value| value.as_u64()),
        Some(1)
    );
}

#[test]
fn parse_options_rejects_invalid_node_role() {
    let err = parse_options(["--node-role", "unknown"].into_iter()).expect_err("invalid node role");
    assert!(err.contains("--node-role"));
}

#[test]
fn parse_options_rejects_invalid_node_validator_spec() {
    let err = parse_options(["--node-validator", "missing_stake"].into_iter()).expect_err("spec");
    assert!(err.contains("--node-validator"));
}

#[test]
fn parse_options_rejects_invalid_node_gossip_addr() {
    let err = parse_options(["--node-gossip-bind", "invalid"].into_iter()).expect_err("invalid");
    assert!(err.contains("--node-gossip-bind"));
}

#[test]
fn start_live_node_applies_pos_options() {
    let options = parse_options(
        [
            "--topology",
            "single",
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
    assert!(runtime.reward_network.is_some());
    let mut locked = runtime.primary_runtime.lock().expect("lock runtime");
    let config = locked.config();
    assert_eq!(config.pos_config.validators.len(), 2);
    assert_eq!(config.pos_config.validators[0].validator_id, "node-main");
    assert_eq!(config.pos_config.validators[0].stake, 70);
    assert_eq!(config.pos_config.validators[1].validator_id, "node-backup");
    assert_eq!(config.pos_config.validators[1].stake, 30);
    assert_eq!(config.pos_config.validator_signer_public_keys.len(), 2);
    assert!(config
        .pos_config
        .validator_signer_public_keys
        .contains_key("node-main"));
    assert!(config
        .pos_config
        .validator_signer_public_keys
        .contains_key("node-backup"));
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
    let options = parse_options(
        [
            "--topology",
            "single",
            "--node-gossip-peer",
            "127.0.0.1:6202",
        ]
        .into_iter(),
    )
    .expect("options");
    let err = start_live_node(&options).expect_err("must fail");
    assert!(err.contains("--node-gossip-bind"));
}

#[test]
fn start_live_node_starts_triad_topology_by_default() {
    let base_port = unique_triad_gossip_base_port().to_string();
    let args = vec!["--triad-gossip-base-port", base_port.as_str()];
    let options = parse_options(args.into_iter()).expect("default options");
    let runtime = start_live_node(&options)
        .expect("start triad")
        .expect("runtime exists");
    assert!(runtime.reward_network.is_some());

    let primary_snapshot = runtime
        .primary_runtime
        .lock()
        .expect("lock primary")
        .snapshot();
    assert_eq!(primary_snapshot.role, NodeRole::Sequencer);
    assert_eq!(primary_snapshot.node_id, "viewer-live-node-sequencer");
    {
        let primary_runtime = runtime.primary_runtime.lock().expect("lock primary config");
        let primary_config = primary_runtime.config();
        assert_eq!(
            primary_config.pos_config.validator_signer_public_keys.len(),
            3
        );
    }
    assert_eq!(runtime.auxiliary_runtimes.len(), 2);

    let mut aux_roles = runtime
        .auxiliary_runtimes
        .iter()
        .map(|runtime| runtime.lock().expect("lock aux").snapshot().role)
        .collect::<Vec<_>>();
    aux_roles.sort_by_key(|role| role.as_str());
    assert_eq!(aux_roles, vec![NodeRole::Observer, NodeRole::Storage]);

    stop_live_node(Some(&runtime));
}

#[test]
fn start_live_node_starts_triad_distributed_storage_role() {
    let options = parse_options(
        [
            "--topology",
            "triad_distributed",
            "--node-id",
            "triad-dist",
            "--node-role",
            "storage",
            "--triad-sequencer-gossip",
            "127.0.0.1:7401",
            "--triad-storage-gossip",
            "127.0.0.1:7402",
        ]
        .into_iter(),
    )
    .expect("options");
    let runtime = start_live_node(&options)
        .expect("start triad distributed")
        .expect("runtime exists");
    assert!(runtime.reward_network.is_some());

    let mut locked = runtime.primary_runtime.lock().expect("lock runtime");
    let snapshot = locked.snapshot();
    assert_eq!(snapshot.role, NodeRole::Storage);
    assert_eq!(snapshot.node_id, "triad-dist-storage");
    let config = locked.config();
    let gossip = config.gossip.as_ref().expect("gossip config");
    assert_eq!(
        gossip.bind_addr,
        "127.0.0.1:7402".parse::<SocketAddr>().expect("addr")
    );
    assert_eq!(
        gossip.peers,
        vec!["127.0.0.1:7401".parse::<SocketAddr>().expect("addr"),]
    );
    assert_eq!(config.pos_config.validators.len(), 3);
    assert_eq!(config.pos_config.validator_signer_public_keys.len(), 3);
    assert_eq!(runtime.auxiliary_runtimes.len(), 0);
    locked.stop().expect("stop");
}

#[test]
fn start_live_node_starts_triad_distributed_sequencer_without_static_peers() {
    let options = parse_options(
        [
            "--topology",
            "triad_distributed",
            "--node-id",
            "triad-dist",
            "--node-role",
            "sequencer",
            "--triad-sequencer-gossip",
            "127.0.0.1:7601",
        ]
        .into_iter(),
    )
    .expect("options");
    let runtime = start_live_node(&options)
        .expect("start triad distributed")
        .expect("runtime exists");
    assert!(runtime.reward_network.is_some());

    let mut locked = runtime.primary_runtime.lock().expect("lock runtime");
    let snapshot = locked.snapshot();
    assert_eq!(snapshot.role, NodeRole::Sequencer);
    assert_eq!(snapshot.node_id, "triad-dist-sequencer");
    let config = locked.config();
    let gossip = config.gossip.as_ref().expect("gossip config");
    assert_eq!(
        gossip.bind_addr,
        "127.0.0.1:7601".parse::<SocketAddr>().expect("addr")
    );
    assert!(gossip.peers.is_empty());
    assert_eq!(runtime.auxiliary_runtimes.len(), 0);
    locked.stop().expect("stop");
}

#[test]
fn parse_options_allows_repl_topic_without_explicit_repl_addrs() {
    let options = parse_options(["--node-repl-topic", "aw.topic"].into_iter()).expect("repl topic");
    assert_eq!(options.node_repl_topic.as_deref(), Some("aw.topic"));
}

#[test]
fn start_live_node_supports_libp2p_replication_injection() {
    let options = parse_options(
        [
            "--topology",
            "single",
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
    runtime
        .primary_runtime
        .lock()
        .expect("lock runtime")
        .stop()
        .expect("stop");
}

#[test]
fn start_live_node_triad_distributed_supports_libp2p_replication_injection() {
    let options = parse_options(
        [
            "--topology",
            "triad_distributed",
            "--node-role",
            "sequencer",
            "--triad-sequencer-gossip",
            "127.0.0.1:7501",
            "--node-repl-libp2p-listen",
            "/ip4/127.0.0.1/tcp/0",
            "--node-repl-topic",
            "aw.test.triad.distributed.replication",
        ]
        .into_iter(),
    )
    .expect("options");

    let runtime = start_live_node(&options)
        .expect("start")
        .expect("runtime exists");
    runtime
        .primary_runtime
        .lock()
        .expect("lock runtime")
        .stop()
        .expect("stop");
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
