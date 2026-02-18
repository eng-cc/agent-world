use super::consensus_signature::{
    sign_attestation_message, sign_commit_message, sign_proposal_message,
    verify_commit_message_signature, ConsensusMessageSigner,
};
use super::gossip_udp::{GossipCommitMessage, GossipEndpoint};
use super::*;
use agent_world_distfs::{FileStore as _, LocalCasStore, SingleWriterReplicationGuard};
use agent_world_proto::distributed_net::NetworkSubscription;
use agent_world_proto::world_error::WorldError;
use ed25519_dalek::SigningKey;
use std::collections::HashMap;
use std::fs;
use std::net::UdpSocket;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn multi_validators() -> Vec<PosValidator> {
    vec![
        PosValidator {
            validator_id: "node-a".to_string(),
            stake: 40,
        },
        PosValidator {
            validator_id: "node-b".to_string(),
            stake: 35,
        },
        PosValidator {
            validator_id: "node-c".to_string(),
            stake: 25,
        },
    ]
}

fn temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("duration")
        .as_nanos();
    std::env::temp_dir().join(format!("agent-world-node-tests-{prefix}-{unique}"))
}

fn deterministic_keypair_hex(seed: u8) -> (String, String) {
    let bytes = [seed; 32];
    let signing_key = SigningKey::from_bytes(&bytes);
    (
        hex::encode(signing_key.to_bytes()),
        hex::encode(signing_key.verifying_key().to_bytes()),
    )
}

fn empty_action_root() -> String {
    compute_consensus_action_root(&[]).expect("empty action root")
}

fn signed_replication_config(root_dir: PathBuf, seed: u8) -> NodeReplicationConfig {
    let (private_hex, public_hex) = deterministic_keypair_hex(seed);
    NodeReplicationConfig::new(root_dir)
        .expect("replication config")
        .with_signing_keypair(private_hex, public_hex)
        .expect("signing keypair")
}

#[derive(Clone)]
struct RecordingExecutionHook {
    calls: Arc<Mutex<Vec<NodeExecutionCommitContext>>>,
}

impl RecordingExecutionHook {
    fn new(calls: Arc<Mutex<Vec<NodeExecutionCommitContext>>>) -> Self {
        Self { calls }
    }
}

impl NodeExecutionHook for RecordingExecutionHook {
    fn on_commit(
        &mut self,
        context: NodeExecutionCommitContext,
    ) -> Result<NodeExecutionCommitResult, String> {
        self.calls
            .lock()
            .expect("lock execution calls")
            .push(context.clone());
        Ok(NodeExecutionCommitResult {
            execution_height: context.height,
            execution_block_hash: format!("exec-block-{:020}", context.height),
            execution_state_root: format!("exec-state-{:020}", context.height),
        })
    }
}

#[derive(Clone, Default)]
struct TestInMemoryNetwork {
    inbox: Arc<Mutex<HashMap<String, Vec<Vec<u8>>>>>,
}

impl agent_world_proto::distributed_net::DistributedNetwork<WorldError> for TestInMemoryNetwork {
    fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), WorldError> {
        let mut inbox = self.inbox.lock().expect("lock inbox");
        inbox
            .entry(topic.to_string())
            .or_default()
            .push(payload.to_vec());
        Ok(())
    }

    fn subscribe(&self, topic: &str) -> Result<NetworkSubscription, WorldError> {
        let mut inbox = self.inbox.lock().expect("lock inbox");
        inbox.entry(topic.to_string()).or_default();
        Ok(NetworkSubscription::new(
            topic.to_string(),
            Arc::clone(&self.inbox),
        ))
    }

    fn request(&self, _protocol: &str, _payload: &[u8]) -> Result<Vec<u8>, WorldError> {
        Err(WorldError::NetworkProtocolUnavailable {
            protocol: "test".to_string(),
        })
    }

    fn register_handler(
        &self,
        _protocol: &str,
        _handler: Box<dyn Fn(&[u8]) -> Result<Vec<u8>, WorldError> + Send + Sync>,
    ) -> Result<(), WorldError> {
        Ok(())
    }
}

#[test]
fn pos_engine_commits_single_validator_head() {
    let config = NodeConfig::new("node-a", "world-a", NodeRole::Observer).expect("config");
    let mut engine = PosNodeEngine::new(&config).expect("engine");

    let snapshot = engine
        .tick(
            &config.node_id,
            &config.world_id,
            1_000,
            None,
            None,
            None,
            Vec::new(),
            None,
        )
        .expect("tick");
    assert_eq!(snapshot.consensus_snapshot.mode, NodeConsensusMode::Pos);
    assert_eq!(snapshot.consensus_snapshot.latest_height, 1);
    assert_eq!(snapshot.consensus_snapshot.committed_height, 1);
    assert_eq!(
        snapshot.consensus_snapshot.last_status,
        Some(PosConsensusStatus::Committed)
    );
    assert_eq!(snapshot.consensus_snapshot.slot, 1);
}

#[test]
fn pos_engine_generates_chain_hashed_block_ids() {
    let config = NodeConfig::new("node-a", "world-hash", NodeRole::Observer).expect("config");
    let mut engine = PosNodeEngine::new(&config).expect("engine");

    let first = engine
        .tick(
            &config.node_id,
            &config.world_id,
            1_000,
            None,
            None,
            None,
            Vec::new(),
            None,
        )
        .expect("first tick");
    let second = engine
        .tick(
            &config.node_id,
            &config.world_id,
            2_000,
            None,
            None,
            None,
            Vec::new(),
            None,
        )
        .expect("second tick");

    let first_hash = first
        .consensus_snapshot
        .last_block_hash
        .as_deref()
        .expect("first hash should exist");
    let second_hash = second
        .consensus_snapshot
        .last_block_hash
        .as_deref()
        .expect("second hash should exist");
    assert_eq!(first_hash.len(), 64);
    assert_eq!(second_hash.len(), 64);
    assert!(first_hash.chars().all(|ch| ch.is_ascii_hexdigit()));
    assert!(second_hash.chars().all(|ch| ch.is_ascii_hexdigit()));
    assert_ne!(first_hash, second_hash);
    assert!(!first_hash.contains(":h"));
}

#[test]
fn pos_engine_progresses_pending_when_auto_attest_disabled() {
    let config = NodeConfig::new("node-a", "world-a", NodeRole::Observer)
        .expect("config")
        .with_pos_validators(multi_validators())
        .expect("validators")
        .with_auto_attest_all_validators(false);
    let mut engine = PosNodeEngine::new(&config).expect("engine");

    let mut committed_height = 0;
    for offset in 0..12 {
        let snapshot = engine
            .tick(
                &config.node_id,
                &config.world_id,
                2_000 + offset,
                None,
                None,
                None,
                Vec::new(),
                None,
            )
            .expect("tick");
        committed_height = snapshot.consensus_snapshot.committed_height;
        if committed_height > 0 {
            break;
        }
    }
    assert!(committed_height >= 1);
}

#[test]
fn runtime_start_and_stop_updates_snapshot() {
    let config = NodeConfig::new("node-a", "world-a", NodeRole::Observer)
        .expect("config")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick interval");
    let mut runtime = NodeRuntime::new(config);
    runtime.start().expect("start");
    thread::sleep(Duration::from_millis(40));

    let running = runtime.snapshot();
    assert!(running.running);
    assert!(running.tick_count >= 2);
    assert!(running.last_tick_unix_ms.is_some());
    assert_eq!(running.consensus.mode, NodeConsensusMode::Pos);
    assert!(running.consensus.committed_height >= 1);
    assert_eq!(
        running.consensus.last_status,
        Some(PosConsensusStatus::Committed)
    );
    assert!(running.last_error.is_none());

    runtime.stop().expect("stop");
    let stopped = runtime.snapshot();
    assert!(!stopped.running);
    assert!(stopped.tick_count >= running.tick_count);
}

#[test]
fn runtime_execution_hook_updates_consensus_snapshot() {
    let config = NodeConfig::new("node-exec", "world-exec", NodeRole::Sequencer)
        .expect("config")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick interval");
    let calls = Arc::new(Mutex::new(Vec::new()));
    let hook = RecordingExecutionHook::new(Arc::clone(&calls));
    let mut runtime = NodeRuntime::new(config).with_execution_hook(hook);
    runtime.start().expect("start");
    thread::sleep(Duration::from_millis(120));
    runtime.stop().expect("stop");

    let snapshot = runtime.snapshot();
    assert!(snapshot.consensus.committed_height >= 1);
    assert!(snapshot.consensus.last_execution_height >= 1);
    assert!(snapshot.consensus.last_execution_block_hash.is_some());
    assert!(snapshot.consensus.last_execution_state_root.is_some());

    let execution_calls = calls.lock().expect("lock calls");
    assert!(!execution_calls.is_empty());
    assert!(execution_calls
        .iter()
        .all(|call| call.world_id == "world-exec" && call.node_id == "node-exec"));
}

#[test]
fn pos_engine_signature_enforced_rejects_unsigned_proposal() {
    let socket_a = UdpSocket::bind("127.0.0.1:0").expect("bind a");
    let socket_b = UdpSocket::bind("127.0.0.1:0").expect("bind b");
    let addr_a = socket_a.local_addr().expect("addr a");
    let addr_b = socket_b.local_addr().expect("addr b");
    drop(socket_a);
    drop(socket_b);

    let validators = vec![
        PosValidator {
            validator_id: "node-a".to_string(),
            stake: 60,
        },
        PosValidator {
            validator_id: "node-b".to_string(),
            stake: 40,
        },
    ];
    let config_b = NodeConfig::new("node-b", "world-sig-enforced", NodeRole::Observer)
        .expect("config b")
        .with_pos_validators(validators)
        .expect("validators")
        .with_replication(signed_replication_config(temp_dir("sig-enforced"), 201));
    let mut engine = PosNodeEngine::new(&config_b).expect("engine");

    let endpoint_a = GossipEndpoint::bind(&NodeGossipConfig {
        bind_addr: addr_a,
        peers: vec![addr_b],
    })
    .expect("endpoint a");
    let endpoint_b = GossipEndpoint::bind(&NodeGossipConfig {
        bind_addr: addr_b,
        peers: vec![addr_a],
    })
    .expect("endpoint b");

    let unsigned_proposal = GossipProposalMessage {
        version: 1,
        world_id: config_b.world_id.clone(),
        node_id: "node-a".to_string(),
        player_id: "node-a".to_string(),
        proposer_id: "node-a".to_string(),
        height: 1,
        slot: 0,
        epoch: 0,
        block_hash: format!("{}:h1:s0:p{}", config_b.world_id, "node-a"),
        action_root: empty_action_root(),
        actions: Vec::new(),
        proposed_at_ms: 1_000,
        public_key_hex: None,
        signature_hex: None,
    };
    endpoint_a
        .broadcast_proposal(&unsigned_proposal)
        .expect("broadcast unsigned proposal");
    thread::sleep(Duration::from_millis(20));

    engine
        .ingest_peer_messages(&endpoint_b, &config_b.node_id, &config_b.world_id, None)
        .expect("ingest");
    assert!(engine.pending.is_none());
}

#[test]
fn pos_engine_signature_enforced_accepts_signed_proposal_and_attestation() {
    let socket_a = UdpSocket::bind("127.0.0.1:0").expect("bind a");
    let socket_b = UdpSocket::bind("127.0.0.1:0").expect("bind b");
    let addr_a = socket_a.local_addr().expect("addr a");
    let addr_b = socket_b.local_addr().expect("addr b");
    drop(socket_a);
    drop(socket_b);

    let validators = vec![
        PosValidator {
            validator_id: "node-a".to_string(),
            stake: 60,
        },
        PosValidator {
            validator_id: "node-b".to_string(),
            stake: 40,
        },
    ];
    let config_b = NodeConfig::new("node-b", "world-sig-accept", NodeRole::Observer)
        .expect("config b")
        .with_pos_validators(validators)
        .expect("validators")
        .with_replication(signed_replication_config(temp_dir("sig-accept"), 202));
    let mut engine = PosNodeEngine::new(&config_b).expect("engine");

    let (private_hex, public_hex) = deterministic_keypair_hex(203);
    let signing_key = SigningKey::from_bytes(
        &hex::decode(private_hex)
            .expect("private decode")
            .try_into()
            .expect("private len"),
    );
    let signer = ConsensusMessageSigner::new(signing_key, public_hex).expect("signer");

    let endpoint_a = GossipEndpoint::bind(&NodeGossipConfig {
        bind_addr: addr_a,
        peers: vec![addr_b],
    })
    .expect("endpoint a");
    let endpoint_b = GossipEndpoint::bind(&NodeGossipConfig {
        bind_addr: addr_b,
        peers: vec![addr_a],
    })
    .expect("endpoint b");

    let mut proposal = GossipProposalMessage {
        version: 1,
        world_id: config_b.world_id.clone(),
        node_id: "node-a".to_string(),
        player_id: "node-a".to_string(),
        proposer_id: "node-a".to_string(),
        height: 1,
        slot: 0,
        epoch: 0,
        block_hash: format!("{}:h1:s0:p{}", config_b.world_id, "node-a"),
        action_root: empty_action_root(),
        actions: Vec::new(),
        proposed_at_ms: 2_000,
        public_key_hex: None,
        signature_hex: None,
    };
    sign_proposal_message(&mut proposal, &signer).expect("sign proposal");
    endpoint_a
        .broadcast_proposal(&proposal)
        .expect("broadcast signed proposal");
    thread::sleep(Duration::from_millis(20));

    let mut attestation = GossipAttestationMessage {
        version: 1,
        world_id: config_b.world_id.clone(),
        node_id: "node-b".to_string(),
        player_id: "node-b".to_string(),
        validator_id: "node-b".to_string(),
        height: proposal.height,
        slot: proposal.slot,
        epoch: proposal.epoch,
        block_hash: proposal.block_hash.clone(),
        approve: true,
        source_epoch: 0,
        target_epoch: 0,
        voted_at_ms: 2_001,
        reason: Some("signed attestation".to_string()),
        public_key_hex: None,
        signature_hex: None,
    };
    sign_attestation_message(&mut attestation, &signer).expect("sign attestation");
    endpoint_a
        .broadcast_attestation(&attestation)
        .expect("broadcast signed attestation");
    thread::sleep(Duration::from_millis(20));

    engine
        .ingest_peer_messages(&endpoint_b, &config_b.node_id, &config_b.world_id, None)
        .expect("ingest");
    let pending = engine.pending.as_ref().expect("pending exists");
    assert_eq!(pending.height, 1);
    assert!(pending.attestations.contains_key("node-a"));
    assert!(pending.attestations.contains_key("node-b"));
}

#[test]
fn commit_signature_covers_execution_hashes() {
    let (private_hex, public_hex) = deterministic_keypair_hex(204);
    let signing_key = SigningKey::from_bytes(
        &hex::decode(private_hex)
            .expect("private decode")
            .try_into()
            .expect("private len"),
    );
    let signer = ConsensusMessageSigner::new(signing_key, public_hex).expect("signer");

    let mut commit = GossipCommitMessage {
        version: 1,
        world_id: "world-signature-exec".to_string(),
        node_id: "node-a".to_string(),
        player_id: "node-a".to_string(),
        height: 7,
        slot: 3,
        epoch: 0,
        block_hash: "block-7".to_string(),
        action_root: empty_action_root(),
        actions: Vec::new(),
        committed_at_ms: 3_000,
        execution_block_hash: Some("exec-block-7".to_string()),
        execution_state_root: Some("exec-state-7".to_string()),
        public_key_hex: None,
        signature_hex: None,
    };
    sign_commit_message(&mut commit, &signer).expect("sign commit");
    verify_commit_message_signature(&commit, true).expect("verify signed commit");

    let mut tampered = commit.clone();
    tampered.execution_state_root = Some("exec-state-tampered".to_string());
    let err = verify_commit_message_signature(&tampered, true).expect_err("tamper must fail");
    assert!(matches!(err, NodeError::Consensus { .. }));

    let mut tampered_action_root = commit.clone();
    tampered_action_root.action_root = "tampered-action-root".to_string();
    let err =
        verify_commit_message_signature(&tampered_action_root, true).expect_err("tamper must fail");
    assert!(matches!(err, NodeError::Consensus { .. }));
}

#[test]
fn pos_engine_ingests_commit_execution_hashes() {
    let socket_a = UdpSocket::bind("127.0.0.1:0").expect("bind a");
    let socket_b = UdpSocket::bind("127.0.0.1:0").expect("bind b");
    let addr_a = socket_a.local_addr().expect("addr a");
    let addr_b = socket_b.local_addr().expect("addr b");
    drop(socket_a);
    drop(socket_b);

    let config = NodeConfig::new("node-b", "world-commit-exec-head", NodeRole::Observer)
        .expect("config")
        .with_pos_validators(vec![
            PosValidator {
                validator_id: "node-a".to_string(),
                stake: 60,
            },
            PosValidator {
                validator_id: "node-b".to_string(),
                stake: 40,
            },
        ])
        .expect("validators")
        .with_gossip_optional(addr_b, vec![addr_a]);
    let mut engine = PosNodeEngine::new(&config).expect("engine");
    let endpoint_a = GossipEndpoint::bind(&NodeGossipConfig {
        bind_addr: addr_a,
        peers: vec![addr_b],
    })
    .expect("endpoint a");
    let endpoint_b = GossipEndpoint::bind(&NodeGossipConfig {
        bind_addr: addr_b,
        peers: vec![addr_a],
    })
    .expect("endpoint b");

    endpoint_a
        .broadcast_commit(&GossipCommitMessage {
            version: 1,
            world_id: config.world_id.clone(),
            node_id: "node-a".to_string(),
            player_id: "node-a".to_string(),
            height: 4,
            slot: 4,
            epoch: 0,
            block_hash: "block-4".to_string(),
            action_root: empty_action_root(),
            actions: Vec::new(),
            committed_at_ms: 4_000,
            execution_block_hash: Some("exec-block-4".to_string()),
            execution_state_root: Some("exec-state-4".to_string()),
            public_key_hex: None,
            signature_hex: None,
        })
        .expect("broadcast commit");
    thread::sleep(Duration::from_millis(20));

    engine
        .ingest_peer_messages(&endpoint_b, &config.node_id, &config.world_id, None)
        .expect("ingest");
    let head = engine
        .peer_heads
        .get("node-a")
        .expect("peer head should exist");
    assert_eq!(head.height, 4);
    assert_eq!(head.execution_block_hash.as_deref(), Some("exec-block-4"));
    assert_eq!(head.execution_state_root.as_deref(), Some("exec-state-4"));
}

#[test]
fn replication_commit_payload_includes_execution_hashes() {
    let dir = temp_dir("replication-payload-exec");
    let config = NodeReplicationConfig::new(dir.clone()).expect("replication config");
    let mut replication =
        super::replication::ReplicationRuntime::new(&config, "node-a").expect("runtime");
    let decision = PosDecision {
        height: 1,
        slot: 0,
        epoch: 0,
        status: PosConsensusStatus::Committed,
        block_hash: "block-1".to_string(),
        action_root: empty_action_root(),
        committed_actions: Vec::new(),
        approved_stake: 100,
        rejected_stake: 0,
        required_stake: 67,
        total_stake: 100,
    };
    let message = replication
        .build_local_commit_message(
            "node-a",
            "world-repl-exec",
            5_000,
            &decision,
            Some("exec-block-1"),
            Some("exec-state-1"),
        )
        .expect("build")
        .expect("message");
    let payload: serde_json::Value =
        serde_json::from_slice(&message.payload).expect("parse payload");
    assert_eq!(
        payload
            .get("execution_block_hash")
            .and_then(serde_json::Value::as_str),
        Some("exec-block-1")
    );
    assert_eq!(
        payload
            .get("execution_state_root")
            .and_then(serde_json::Value::as_str),
        Some("exec-state-1")
    );
    assert_eq!(
        payload
            .get("action_root")
            .and_then(serde_json::Value::as_str),
        Some(empty_action_root().as_str())
    );
    assert_eq!(
        payload
            .get("actions")
            .and_then(serde_json::Value::as_array)
            .map(Vec::len),
        Some(0)
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn runtime_rejects_double_start() {
    let config = NodeConfig::new("node-b", "world-b", NodeRole::Sequencer).expect("config");
    let mut runtime = NodeRuntime::new(config);
    runtime.start().expect("first start");
    let err = runtime.start().expect_err("second start must fail");
    assert!(matches!(err, NodeError::AlreadyRunning { .. }));
    runtime.stop().expect("stop");
}

#[test]
fn runtime_pos_state_persists_across_restart() {
    let dir = temp_dir("pos-state-restart");
    let build_config = || {
        NodeConfig::new("node-a", "world-pos-state", NodeRole::Sequencer)
            .expect("config")
            .with_tick_interval(Duration::from_millis(10))
            .expect("tick")
            .with_replication_root(dir.clone())
            .expect("replication")
    };

    let mut runtime = NodeRuntime::new(build_config()).with_execution_hook(
        RecordingExecutionHook::new(Arc::new(Mutex::new(Vec::new()))),
    );
    runtime.start().expect("start first");
    thread::sleep(Duration::from_millis(180));
    runtime.stop().expect("stop first");
    let first = runtime.snapshot();
    assert!(first.last_error.is_none());
    assert!(first.consensus.committed_height >= 8);
    assert!(first.consensus.last_execution_height >= 8);

    let state_path = dir.join("node_pos_state.json");
    assert!(state_path.exists());
    let persisted = serde_json::from_slice::<super::pos_state_store::PosNodeStateSnapshot>(
        &fs::read(&state_path).expect("read pos state"),
    )
    .expect("parse pos state");
    assert!(persisted.committed_height >= first.consensus.committed_height);
    assert!(persisted.last_execution_height >= first.consensus.last_execution_height);
    assert!(persisted.last_execution_block_hash.is_some());
    assert!(persisted.last_execution_state_root.is_some());

    let mut runtime = NodeRuntime::new(build_config()).with_execution_hook(
        RecordingExecutionHook::new(Arc::new(Mutex::new(Vec::new()))),
    );
    runtime.start().expect("start second");
    thread::sleep(Duration::from_millis(40));
    runtime.stop().expect("stop second");
    let second = runtime.snapshot();
    assert!(second.last_error.is_none());
    assert!(second.consensus.committed_height > first.consensus.committed_height);
    assert!(second.consensus.last_execution_height > first.consensus.last_execution_height);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn config_with_gossip_rejects_empty_peers() {
    let bind_socket = UdpSocket::bind("127.0.0.1:0").expect("bind");
    let bind_addr = bind_socket.local_addr().expect("addr");
    let config = NodeConfig::new("node-a", "world-a", NodeRole::Observer)
        .expect("config")
        .with_gossip(bind_addr, vec![]);
    assert!(matches!(config, Err(NodeError::InvalidConfig { .. })));
}

#[test]
fn runtime_gossip_tracks_peer_committed_heads() {
    let socket_a = UdpSocket::bind("127.0.0.1:0").expect("bind a");
    let socket_b = UdpSocket::bind("127.0.0.1:0").expect("bind b");
    let addr_a = socket_a.local_addr().expect("addr a");
    let addr_b = socket_b.local_addr().expect("addr b");
    drop(socket_a);
    drop(socket_b);

    let validators = vec![
        PosValidator {
            validator_id: "node-a".to_string(),
            stake: 60,
        },
        PosValidator {
            validator_id: "node-b".to_string(),
            stake: 40,
        },
    ];

    let config_a = NodeConfig::new("node-a", "world-sync", NodeRole::Sequencer)
        .expect("config a")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick a")
        .with_pos_validators(validators.clone())
        .expect("validators a")
        .with_gossip_optional(addr_a, vec![addr_b]);
    let config_b = NodeConfig::new("node-b", "world-sync", NodeRole::Observer)
        .expect("config b")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick b")
        .with_pos_validators(validators)
        .expect("validators b")
        .with_gossip_optional(addr_b, vec![addr_a]);

    let mut runtime_a = NodeRuntime::new(config_a);
    let mut runtime_b = NodeRuntime::new(config_b);
    runtime_a.start().expect("start a");
    runtime_b.start().expect("start b");
    thread::sleep(Duration::from_millis(180));

    let snapshot_a = runtime_a.snapshot();
    let snapshot_b = runtime_b.snapshot();
    assert!(snapshot_a.consensus.network_committed_height >= 1);
    assert!(snapshot_b.consensus.network_committed_height >= 1);
    assert!(snapshot_a.consensus.known_peer_heads >= 1);
    assert!(snapshot_b.consensus.known_peer_heads >= 1);

    runtime_a.stop().expect("stop a");
    runtime_b.stop().expect("stop b");
}

#[test]
fn runtime_gossip_replication_syncs_distfs_commit_files() {
    let socket_a = UdpSocket::bind("127.0.0.1:0").expect("bind a");
    let socket_b = UdpSocket::bind("127.0.0.1:0").expect("bind b");
    let addr_a = socket_a.local_addr().expect("addr a");
    let addr_b = socket_b.local_addr().expect("addr b");
    drop(socket_a);
    drop(socket_b);

    let dir_a = temp_dir("replication-a");
    let dir_b = temp_dir("replication-b");
    let validators = vec![
        PosValidator {
            validator_id: "node-a".to_string(),
            stake: 60,
        },
        PosValidator {
            validator_id: "node-b".to_string(),
            stake: 40,
        },
    ];

    let config_a = NodeConfig::new("node-a", "world-repl", NodeRole::Sequencer)
        .expect("config a")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick a")
        .with_pos_validators(validators.clone())
        .expect("validators a")
        .with_gossip_optional(addr_a, vec![addr_b])
        .with_replication_root(dir_a.clone())
        .expect("replication a");
    let config_b = NodeConfig::new("node-b", "world-repl", NodeRole::Observer)
        .expect("config b")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick b")
        .with_pos_validators(validators)
        .expect("validators b")
        .with_gossip_optional(addr_b, vec![addr_a])
        .with_replication_root(dir_b.clone())
        .expect("replication b");

    let mut runtime_a = NodeRuntime::new(config_a);
    let mut runtime_b = NodeRuntime::new(config_b);
    runtime_a.start().expect("start a");
    runtime_b.start().expect("start b");
    thread::sleep(Duration::from_millis(220));

    runtime_a.stop().expect("stop a");
    runtime_b.stop().expect("stop b");

    let store_b = LocalCasStore::new(dir_b.join("store"));
    let files = store_b.list_files().expect("list files");
    assert!(files
        .iter()
        .any(|item| item.path.starts_with("consensus/commits/")));
    assert!(dir_b.join("replication_guard.json").exists());

    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);
}

#[test]
fn runtime_network_replication_syncs_distfs_commit_files() {
    let dir_a = temp_dir("network-repl-a");
    let dir_b = temp_dir("network-repl-b");
    let validators = vec![
        PosValidator {
            validator_id: "node-a".to_string(),
            stake: 60,
        },
        PosValidator {
            validator_id: "node-b".to_string(),
            stake: 40,
        },
    ];
    let network: Arc<
        dyn agent_world_proto::distributed_net::DistributedNetwork<WorldError> + Send + Sync,
    > = Arc::new(TestInMemoryNetwork::default());

    let config_a = NodeConfig::new("node-a", "world-network-repl", NodeRole::Sequencer)
        .expect("config a")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick a")
        .with_pos_validators(validators.clone())
        .expect("validators a")
        .with_replication(signed_replication_config(dir_a.clone(), 71));
    let config_b = NodeConfig::new("node-b", "world-network-repl", NodeRole::Observer)
        .expect("config b")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick b")
        .with_pos_validators(validators)
        .expect("validators b")
        .with_replication(signed_replication_config(dir_b.clone(), 72));

    let mut runtime_a = NodeRuntime::new(config_a)
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));
    let mut runtime_b = NodeRuntime::new(config_b)
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));
    runtime_a.start().expect("start a");
    runtime_b.start().expect("start b");
    thread::sleep(Duration::from_millis(220));

    runtime_a.stop().expect("stop a");
    runtime_b.stop().expect("stop b");

    let store_b = LocalCasStore::new(dir_b.join("store"));
    let files = store_b.list_files().expect("list files");
    assert!(files
        .iter()
        .any(|item| item.path.starts_with("consensus/commits/")));

    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);
}

#[test]
fn replication_network_handle_rejects_empty_topic() {
    let network: Arc<
        dyn agent_world_proto::distributed_net::DistributedNetwork<WorldError> + Send + Sync,
    > = Arc::new(TestInMemoryNetwork::default());
    let err = NodeReplicationNetworkHandle::new(network)
        .with_topic("   ")
        .expect_err("empty topic");
    assert!(matches!(err, NodeError::InvalidConfig { .. }));
}

#[test]
fn runtime_network_replication_respects_topic_isolation() {
    let dir_a = temp_dir("network-topic-a");
    let dir_b = temp_dir("network-topic-b");
    let validators = vec![
        PosValidator {
            validator_id: "node-a".to_string(),
            stake: 60,
        },
        PosValidator {
            validator_id: "node-b".to_string(),
            stake: 40,
        },
    ];
    let network: Arc<
        dyn agent_world_proto::distributed_net::DistributedNetwork<WorldError> + Send + Sync,
    > = Arc::new(TestInMemoryNetwork::default());

    let config_a = NodeConfig::new("node-a", "world-topic-repl", NodeRole::Sequencer)
        .expect("config a")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick a")
        .with_pos_validators(validators.clone())
        .expect("validators a")
        .with_replication(signed_replication_config(dir_a.clone(), 81));
    let config_b = NodeConfig::new("node-b", "world-topic-repl", NodeRole::Observer)
        .expect("config b")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick b")
        .with_pos_validators(validators)
        .expect("validators b")
        .with_replication(signed_replication_config(dir_b.clone(), 82));

    let mut runtime_a = NodeRuntime::new(config_a).with_replication_network(
        NodeReplicationNetworkHandle::new(Arc::clone(&network))
            .with_topic("aw.world-topic-repl.replication.a")
            .expect("topic a"),
    );
    let mut runtime_b = NodeRuntime::new(config_b).with_replication_network(
        NodeReplicationNetworkHandle::new(Arc::clone(&network))
            .with_topic("aw.world-topic-repl.replication.b")
            .expect("topic b"),
    );
    runtime_a.start().expect("start a");
    runtime_b.start().expect("start b");
    thread::sleep(Duration::from_millis(220));

    runtime_a.stop().expect("stop a");
    runtime_b.stop().expect("stop b");

    let store_b = LocalCasStore::new(dir_b.join("store"));
    let files = store_b.list_files().expect("list files");
    assert!(files.is_empty());

    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);
}

#[test]
fn runtime_gossip_replication_with_signature_applies_files() {
    let socket_a = UdpSocket::bind("127.0.0.1:0").expect("bind a");
    let socket_b = UdpSocket::bind("127.0.0.1:0").expect("bind b");
    let addr_a = socket_a.local_addr().expect("addr a");
    let addr_b = socket_b.local_addr().expect("addr b");
    drop(socket_a);
    drop(socket_b);

    let dir_a = temp_dir("signed-repl-a");
    let dir_b = temp_dir("signed-repl-b");
    let validators = vec![
        PosValidator {
            validator_id: "node-a".to_string(),
            stake: 60,
        },
        PosValidator {
            validator_id: "node-b".to_string(),
            stake: 40,
        },
    ];

    let config_a = NodeConfig::new("node-a", "world-signed", NodeRole::Sequencer)
        .expect("config a")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick a")
        .with_pos_validators(validators.clone())
        .expect("validators a")
        .with_gossip_optional(addr_a, vec![addr_b])
        .with_replication(signed_replication_config(dir_a.clone(), 11));
    let config_b = NodeConfig::new("node-b", "world-signed", NodeRole::Observer)
        .expect("config b")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick b")
        .with_pos_validators(validators)
        .expect("validators b")
        .with_gossip_optional(addr_b, vec![addr_a])
        .with_replication(signed_replication_config(dir_b.clone(), 22));

    let mut runtime_a = NodeRuntime::new(config_a);
    let mut runtime_b = NodeRuntime::new(config_b);
    runtime_a.start().expect("start a");
    runtime_b.start().expect("start b");
    thread::sleep(Duration::from_millis(220));

    runtime_a.stop().expect("stop a");
    runtime_b.stop().expect("stop b");

    let store_b = LocalCasStore::new(dir_b.join("store"));
    let files = store_b.list_files().expect("list files");
    assert!(files
        .iter()
        .any(|item| item.path.starts_with("consensus/commits/")));

    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);
}

#[test]
fn runtime_gossip_replication_rejects_unsigned_when_signature_enforced() {
    let socket_a = UdpSocket::bind("127.0.0.1:0").expect("bind a");
    let socket_b = UdpSocket::bind("127.0.0.1:0").expect("bind b");
    let addr_a = socket_a.local_addr().expect("addr a");
    let addr_b = socket_b.local_addr().expect("addr b");
    drop(socket_a);
    drop(socket_b);

    let dir_a = temp_dir("unsigned-a");
    let dir_b = temp_dir("enforced-b");
    let validators = vec![
        PosValidator {
            validator_id: "node-a".to_string(),
            stake: 60,
        },
        PosValidator {
            validator_id: "node-b".to_string(),
            stake: 40,
        },
    ];

    let config_a = NodeConfig::new("node-a", "world-enforced", NodeRole::Sequencer)
        .expect("config a")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick a")
        .with_pos_validators(validators.clone())
        .expect("validators a")
        .with_gossip_optional(addr_a, vec![addr_b])
        .with_replication_root(dir_a.clone())
        .expect("replication a");
    let config_b = NodeConfig::new("node-b", "world-enforced", NodeRole::Observer)
        .expect("config b")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick b")
        .with_pos_validators(validators)
        .expect("validators b")
        .with_gossip_optional(addr_b, vec![addr_a])
        .with_replication(signed_replication_config(dir_b.clone(), 33));

    let mut runtime_a = NodeRuntime::new(config_a);
    let mut runtime_b = NodeRuntime::new(config_b);
    runtime_a.start().expect("start a");
    runtime_b.start().expect("start b");
    thread::sleep(Duration::from_millis(220));

    runtime_a.stop().expect("stop a");
    runtime_b.stop().expect("stop b");

    let store_b = LocalCasStore::new(dir_b.join("store"));
    let files = store_b.list_files().expect("list files");
    assert!(files.is_empty());

    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);
}

#[test]
fn runtime_gossip_replication_persists_guard_across_restart() {
    let socket_a = UdpSocket::bind("127.0.0.1:0").expect("bind a");
    let socket_b = UdpSocket::bind("127.0.0.1:0").expect("bind b");
    let addr_a = socket_a.local_addr().expect("addr a");
    let addr_b = socket_b.local_addr().expect("addr b");
    drop(socket_a);
    drop(socket_b);

    let dir_a = temp_dir("restart-a");
    let dir_b = temp_dir("restart-b");
    let validators = vec![
        PosValidator {
            validator_id: "node-a".to_string(),
            stake: 60,
        },
        PosValidator {
            validator_id: "node-b".to_string(),
            stake: 40,
        },
    ];

    let build_config_a = || {
        NodeConfig::new("node-a", "world-restart", NodeRole::Sequencer)
            .expect("config a")
            .with_tick_interval(Duration::from_millis(10))
            .expect("tick a")
            .with_pos_validators(validators.clone())
            .expect("validators a")
            .with_gossip_optional(addr_a, vec![addr_b])
            .with_replication(signed_replication_config(dir_a.clone(), 55))
    };
    let build_config_b = || {
        NodeConfig::new("node-b", "world-restart", NodeRole::Observer)
            .expect("config b")
            .with_tick_interval(Duration::from_millis(10))
            .expect("tick b")
            .with_pos_validators(validators.clone())
            .expect("validators b")
            .with_gossip_optional(addr_b, vec![addr_a])
            .with_replication(signed_replication_config(dir_b.clone(), 66))
    };

    let mut runtime_a = NodeRuntime::new(build_config_a());
    let mut runtime_b = NodeRuntime::new(build_config_b());
    runtime_a.start().expect("start a first");
    runtime_b.start().expect("start b first");
    thread::sleep(Duration::from_millis(220));
    let snapshot_b_first = runtime_b.snapshot();
    runtime_a.stop().expect("stop a first");
    runtime_b.stop().expect("stop b first");
    assert!(snapshot_b_first.last_error.is_none());

    let guard_path = dir_b.join("replication_guard.json");
    let guard_before: SingleWriterReplicationGuard =
        serde_json::from_slice(&fs::read(&guard_path).expect("read guard before"))
            .expect("parse guard before");
    assert!(guard_before.last_sequence >= 1);

    let mut runtime_a = NodeRuntime::new(build_config_a());
    let mut runtime_b = NodeRuntime::new(build_config_b());
    runtime_a.start().expect("start a second");
    runtime_b.start().expect("start b second");
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let maybe_guard = fs::read(&guard_path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<SingleWriterReplicationGuard>(&bytes).ok());
        if maybe_guard
            .as_ref()
            .is_some_and(|guard| guard.last_sequence > guard_before.last_sequence)
        {
            break;
        }
        if Instant::now() >= deadline {
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    let snapshot_b_second = runtime_b.snapshot();
    runtime_a.stop().expect("stop a second");
    runtime_b.stop().expect("stop b second");
    assert!(snapshot_b_second.last_error.is_none());

    let guard_after: SingleWriterReplicationGuard =
        serde_json::from_slice(&fs::read(&guard_path).expect("read guard after"))
            .expect("parse guard after");
    assert_eq!(guard_after.writer_id, guard_before.writer_id);
    assert!(guard_after.last_sequence > guard_before.last_sequence);

    let store_b = LocalCasStore::new(dir_b.join("store"));
    let files = store_b.list_files().expect("list files");
    assert!(files.len() >= 2);

    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);
}
