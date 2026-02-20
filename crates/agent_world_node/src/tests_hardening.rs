use super::gossip_udp::{GossipEndpoint, GossipProposalMessage};
use super::*;
use agent_world_consensus::node_consensus_signature::{
    sign_proposal_message, NodeConsensusMessageSigner as ConsensusMessageSigner,
};
use agent_world_distfs::{blake3_hex, FileReplicationRecord};
use agent_world_proto::distributed_net::NetworkSubscription;
use agent_world_proto::world_error::WorldError;
use ed25519_dalek::SigningKey;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::net::UdpSocket;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("duration")
        .as_nanos();
    std::env::temp_dir().join(format!("agent-world-node-hardening-{prefix}-{unique}"))
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

fn empty_action_root() -> String {
    compute_consensus_action_root(&[]).expect("empty action root")
}

fn wait_until(deadline: Instant, mut predicate: impl FnMut() -> bool) -> bool {
    while Instant::now() < deadline {
        if predicate() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

#[derive(Clone, Default)]
struct TestInMemoryNetwork {
    retained: Arc<Mutex<HashMap<String, Vec<Vec<u8>>>>>,
    subscribers: Arc<Mutex<Vec<TestNetworkInbox>>>,
    handlers: Arc<
        Mutex<HashMap<String, Arc<dyn Fn(&[u8]) -> Result<Vec<u8>, WorldError> + Send + Sync>>>,
    >,
}

type TestNetworkInbox = Arc<Mutex<HashMap<String, Vec<Vec<u8>>>>>;

impl agent_world_proto::distributed_net::DistributedNetwork<WorldError> for TestInMemoryNetwork {
    fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), WorldError> {
        self.retained
            .lock()
            .expect("lock retained")
            .entry(topic.to_string())
            .or_default()
            .push(payload.to_vec());
        let subscribers = self.subscribers.lock().expect("lock subscribers");
        for inbox in subscribers.iter() {
            let mut topic_inbox = inbox.lock().expect("lock subscriber inbox");
            topic_inbox
                .entry(topic.to_string())
                .or_default()
                .push(payload.to_vec());
        }
        Ok(())
    }

    fn subscribe(&self, topic: &str) -> Result<NetworkSubscription, WorldError> {
        let inbox = Arc::new(Mutex::new(HashMap::<String, Vec<Vec<u8>>>::new()));
        let retained = self.retained.lock().expect("lock retained");
        let seeded = retained.get(topic).cloned().unwrap_or_default();
        drop(retained);
        {
            let mut topic_inbox = inbox.lock().expect("lock subscriber inbox");
            topic_inbox.insert(topic.to_string(), seeded);
        }
        self.subscribers
            .lock()
            .expect("lock subscribers")
            .push(Arc::clone(&inbox));
        Ok(NetworkSubscription::new(topic.to_string(), inbox))
    }

    fn request(&self, protocol: &str, payload: &[u8]) -> Result<Vec<u8>, WorldError> {
        let handlers = self.handlers.lock().expect("lock handlers");
        let Some(handler) = handlers.get(protocol) else {
            return Err(WorldError::NetworkProtocolUnavailable {
                protocol: protocol.to_string(),
            });
        };
        handler(payload)
    }

    fn register_handler(
        &self,
        protocol: &str,
        handler: Box<dyn Fn(&[u8]) -> Result<Vec<u8>, WorldError> + Send + Sync>,
    ) -> Result<(), WorldError> {
        self.handlers
            .lock()
            .expect("lock handlers")
            .insert(protocol.to_string(), Arc::from(handler));
        Ok(())
    }
}

#[test]
fn config_rejects_duplicate_validator_signer_bindings() {
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
    let (_, signer_public_key) = deterministic_keypair_hex(41);
    let mut signer_map = BTreeMap::new();
    signer_map.insert("node-a".to_string(), signer_public_key.clone());
    signer_map.insert("node-b".to_string(), signer_public_key);

    let result =
        NodePosConfig::ethereum_like(validators).with_validator_signer_public_keys(signer_map);
    assert!(matches!(result, Err(NodeError::InvalidConfig { .. })));
}

#[test]
fn pos_engine_rejects_signed_proposal_when_signer_binding_mismatches_validator() {
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
    let (_, node_a_expected_pub) = deterministic_keypair_hex(51);
    let (_, node_b_expected_pub) = deterministic_keypair_hex(52);
    let mut signer_map = BTreeMap::new();
    signer_map.insert("node-a".to_string(), node_a_expected_pub);
    signer_map.insert("node-b".to_string(), node_b_expected_pub);

    let pos_config = NodePosConfig::ethereum_like(validators)
        .with_validator_signer_public_keys(signer_map)
        .expect("pos config");
    let config_b = NodeConfig::new("node-b", "world-signer-binding", NodeRole::Observer)
        .expect("config b")
        .with_pos_config(pos_config)
        .expect("validators")
        .with_replication(signed_replication_config(temp_dir("signer-binding"), 53));
    let mut engine = PosNodeEngine::new(&config_b).expect("engine");

    let (wrong_private_hex, wrong_public_hex) = deterministic_keypair_hex(61);
    let wrong_signing_key = SigningKey::from_bytes(
        &hex::decode(wrong_private_hex)
            .expect("private decode")
            .try_into()
            .expect("private len"),
    );
    let wrong_signer =
        ConsensusMessageSigner::new(wrong_signing_key, wrong_public_hex).expect("wrong signer");

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
        proposed_at_ms: 1_000,
        public_key_hex: None,
        signature_hex: None,
    };
    sign_proposal_message(&mut proposal, &wrong_signer).expect("sign proposal");
    endpoint_a
        .broadcast_proposal(&proposal)
        .expect("broadcast proposal");
    std::thread::sleep(Duration::from_millis(20));

    engine
        .ingest_peer_messages(&endpoint_b, &config_b.node_id, &config_b.world_id, None)
        .expect("ingest");
    assert!(engine.pending.is_none());
}

#[test]
fn runtime_start_fails_when_pos_state_snapshot_is_corrupted() {
    let dir = temp_dir("pos-state-corrupt");
    fs::create_dir_all(&dir).expect("create dir");
    fs::write(dir.join("node_pos_state.json"), b"not-json").expect("write corrupted snapshot");

    let config = NodeConfig::new("node-a", "world-pos-corrupt", NodeRole::Observer)
        .expect("config")
        .with_replication(signed_replication_config(dir.clone(), 71));
    let mut runtime = NodeRuntime::new(config);

    let err = runtime.start().expect_err("start should fail");
    assert!(matches!(err, NodeError::Replication { .. }));
    assert!(!runtime.snapshot().running);

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn runtime_replication_ingest_reports_error_and_does_not_advance_network_height_on_invalid_message()
{
    let world_id = "world-repl-hardening";
    let dir = temp_dir("repl-ingest-hardening");
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

    let network_impl = Arc::new(TestInMemoryNetwork::default());
    let network: Arc<
        dyn agent_world_proto::distributed_net::DistributedNetwork<WorldError> + Send + Sync,
    > = network_impl.clone();

    let config = NodeConfig::new("node-b", world_id, NodeRole::Observer)
        .expect("config")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick")
        .with_pos_validators(validators)
        .expect("validators")
        .with_replication(signed_replication_config(dir.clone(), 81));
    let mut runtime = NodeRuntime::new(config)
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));
    runtime.start().expect("start");

    let payload = b"payload-actual".to_vec();
    let bad_message = super::replication::GossipReplicationMessage {
        version: 1,
        world_id: world_id.to_string(),
        node_id: "node-a".to_string(),
        record: FileReplicationRecord {
            world_id: world_id.to_string(),
            writer_id: "writer-a".to_string(),
            writer_epoch: 1,
            sequence: 1,
            path: "consensus/commits/00000000000000000001.json".to_string(),
            content_hash: blake3_hex(b"payload-expected"),
            size_bytes: payload.len() as u64,
            updated_at_ms: 1,
        },
        payload,
        public_key_hex: None,
        signature_hex: None,
    };
    let encoded = serde_json::to_vec(&bad_message).expect("encode message");
    let topic = super::network_bridge::default_replication_topic(world_id);
    network
        .publish(topic.as_str(), encoded.as_slice())
        .expect("publish invalid message");

    let mut last_republish_at = Instant::now();
    let has_error = wait_until(Instant::now() + Duration::from_secs(2), || {
        if runtime
            .snapshot()
            .last_error
            .as_ref()
            .map(|reason| reason.contains("replication ingest rejected"))
            .unwrap_or(false)
        {
            return true;
        }

        if last_republish_at.elapsed() >= Duration::from_millis(30) {
            network
                .publish(topic.as_str(), encoded.as_slice())
                .expect("republish invalid message");
            last_republish_at = Instant::now();
        }
        false
    });
    assert!(
        has_error,
        "runtime did not report replication ingest rejection"
    );

    runtime.stop().expect("stop");
    let snapshot = runtime.snapshot();
    assert_eq!(snapshot.consensus.network_committed_height, 0);

    let _ = fs::remove_dir_all(dir);
}
