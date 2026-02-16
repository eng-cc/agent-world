use super::consensus_signature::{
    sign_attestation_message, sign_proposal_message, ConsensusMessageSigner,
};
use super::gossip_udp::GossipEndpoint;
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
use std::time::{Instant, SystemTime, UNIX_EPOCH};

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

fn signed_replication_config(root_dir: PathBuf, seed: u8) -> NodeReplicationConfig {
    let (private_hex, public_hex) = deterministic_keypair_hex(seed);
    NodeReplicationConfig::new(root_dir)
        .expect("replication config")
        .with_signing_keypair(private_hex, public_hex)
        .expect("signing keypair")
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
fn role_parse_roundtrip() {
    for role in [NodeRole::Sequencer, NodeRole::Storage, NodeRole::Observer] {
        let parsed = NodeRole::from_str(role.as_str()).expect("parse role");
        assert_eq!(parsed, role);
    }
}

#[test]
fn config_rejects_invalid_pos_config() {
    let result = NodeConfig::new("node-a", "world-a", NodeRole::Observer)
        .expect("base config")
        .with_pos_config(NodePosConfig::ethereum_like(vec![]));
    assert!(matches!(result, Err(NodeError::InvalidConfig { .. })));
}

#[test]
fn pos_engine_commits_single_validator_head() {
    let config = NodeConfig::new("node-a", "world-a", NodeRole::Observer).expect("config");
    let mut engine = PosNodeEngine::new(&config).expect("engine");

    let snapshot = engine
        .tick(&config.node_id, &config.world_id, 1_000, None, None, None)
        .expect("tick");
    assert_eq!(snapshot.mode, NodeConsensusMode::Pos);
    assert_eq!(snapshot.latest_height, 1);
    assert_eq!(snapshot.committed_height, 1);
    assert_eq!(snapshot.last_status, Some(PosConsensusStatus::Committed));
    assert_eq!(snapshot.slot, 1);
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
            )
            .expect("tick");
        committed_height = snapshot.committed_height;
        if committed_height > 0 {
            break;
        }
    }
    assert!(committed_height >= 1);
}

#[test]
fn pos_engine_applies_gossiped_proposal_and_attestation() {
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
    let config = NodeConfig::new("node-b", "world-gossip-proposal", NodeRole::Observer)
        .expect("config")
        .with_pos_validators(validators)
        .expect("validators")
        .with_auto_attest_all_validators(false);
    let mut engine = PosNodeEngine::new(&config).expect("engine");

    let proposal = GossipProposalMessage {
        version: 1,
        world_id: config.world_id.clone(),
        node_id: "node-a".to_string(),
        proposer_id: "node-a".to_string(),
        height: 1,
        slot: 0,
        epoch: 0,
        block_hash: format!("{}:h1:s0:p{}", config.world_id, "node-a"),
        proposed_at_ms: 1_000,
        public_key_hex: None,
        signature_hex: None,
    };
    engine
        .ingest_proposal_message(&config.world_id, &proposal)
        .expect("ingest proposal");

    let attestation = GossipAttestationMessage {
        version: 1,
        world_id: config.world_id.clone(),
        node_id: "node-b".to_string(),
        validator_id: "node-b".to_string(),
        height: 1,
        slot: 0,
        epoch: 0,
        block_hash: proposal.block_hash.clone(),
        approve: true,
        source_epoch: 0,
        target_epoch: 0,
        voted_at_ms: 1_001,
        reason: Some("gossip attestation".to_string()),
        public_key_hex: None,
        signature_hex: None,
    };
    engine
        .ingest_attestation_message(&config.world_id, &attestation)
        .expect("ingest attestation");

    let snapshot = engine
        .tick(&config.node_id, &config.world_id, 1_002, None, None, None)
        .expect("tick");
    assert_eq!(snapshot.committed_height, 1);
    assert_eq!(snapshot.last_status, Some(PosConsensusStatus::Committed));
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
        proposer_id: "node-a".to_string(),
        height: 1,
        slot: 0,
        epoch: 0,
        block_hash: format!("{}:h1:s0:p{}", config_b.world_id, "node-a"),
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
        proposer_id: "node-a".to_string(),
        height: 1,
        slot: 0,
        epoch: 0,
        block_hash: format!("{}:h1:s0:p{}", config_b.world_id, "node-a"),
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
        node_id: "node-a".to_string(),
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
fn runtime_rejects_double_start() {
    let config = NodeConfig::new("node-b", "world-b", NodeRole::Sequencer).expect("config");
    let mut runtime = NodeRuntime::new(config);
    runtime.start().expect("first start");
    let err = runtime.start().expect_err("second start must fail");
    assert!(matches!(err, NodeError::AlreadyRunning { .. }));
    runtime.stop().expect("stop");
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
