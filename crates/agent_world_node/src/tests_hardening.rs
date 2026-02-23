use super::gossip_udp::{GossipEndpoint, GossipProposalMessage};
use super::*;
use agent_world_consensus::node_consensus_signature::{
    sign_proposal_message, NodeConsensusMessageSigner as ConsensusMessageSigner,
};
use agent_world_distfs::{blake3_hex, build_replication_record_with_epoch, FileReplicationRecord};
use agent_world_proto::distributed::DistributedErrorCode;
use agent_world_proto::distributed_net::NetworkSubscription;
use agent_world_proto::world_error::WorldError;
use ed25519_dalek::{Signer as _, SigningKey};
use serde::Serialize;
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

fn signed_pos_config_with_signer_seeds(
    validators: Vec<PosValidator>,
    signer_seeds: &[(&str, u8)],
) -> NodePosConfig {
    let seed_map = signer_seeds
        .iter()
        .map(|(validator_id, seed)| ((*validator_id).to_string(), *seed))
        .collect::<HashMap<_, _>>();
    let mut signer_map = BTreeMap::new();
    for validator in &validators {
        let seed = seed_map
            .get(validator.validator_id.as_str())
            .unwrap_or_else(|| {
                panic!(
                    "missing signer seed for validator {}",
                    validator.validator_id
                )
            });
        let (_, public_key_hex) = deterministic_keypair_hex(*seed);
        signer_map.insert(validator.validator_id.clone(), public_key_hex);
    }
    NodePosConfig::ethereum_like(validators)
        .with_validator_signer_public_keys(signer_map)
        .expect("signed pos config")
}

#[derive(Debug, Serialize)]
struct ReplicationSigningPayload<'a> {
    version: u8,
    world_id: &'a str,
    node_id: &'a str,
    record: &'a FileReplicationRecord,
    payload: &'a [u8],
    public_key_hex: Option<&'a str>,
}

fn sign_replication_message_for_test(
    message: &super::replication::GossipReplicationMessage,
    private_key_hex: &str,
) -> String {
    let private_key: [u8; 32] = hex::decode(private_key_hex)
        .expect("private key decode")
        .try_into()
        .expect("private key length");
    let signing_key = SigningKey::from_bytes(&private_key);
    let payload = ReplicationSigningPayload {
        version: message.version,
        world_id: message.world_id.as_str(),
        node_id: message.node_id.as_str(),
        record: &message.record,
        payload: &message.payload,
        public_key_hex: message.public_key_hex.as_deref(),
    };
    let bytes = serde_json::to_vec(&payload).expect("encode signing payload");
    let signature = signing_key.sign(bytes.as_slice());
    hex::encode(signature.to_bytes())
}

fn signed_replication_message_for_writer(
    world_id: &str,
    node_id: &str,
    private_key_hex: &str,
    public_key_hex: &str,
    sequence: u64,
) -> super::replication::GossipReplicationMessage {
    let payload = format!("payload-{sequence}").into_bytes();
    let path = format!("consensus/commits/{:020}.json", sequence.max(1));
    let record = build_replication_record_with_epoch(
        world_id,
        public_key_hex,
        1,
        sequence.max(1),
        path.as_str(),
        payload.as_slice(),
        1_000,
    )
    .expect("record");
    let mut message = super::replication::GossipReplicationMessage {
        version: 1,
        world_id: world_id.to_string(),
        node_id: node_id.to_string(),
        record,
        payload,
        public_key_hex: Some(public_key_hex.to_string()),
        signature_hex: None,
    };
    message.signature_hex = Some(sign_replication_message_for_test(&message, private_key_hex));
    message
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
        .with_replication(signed_replication_config(temp_dir("signer-binding"), 52));
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
fn pos_engine_rejects_signed_mode_without_complete_validator_signer_bindings() {
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
    let pos_config = NodePosConfig::ethereum_like(validators);
    let config = NodeConfig::new("node-b", "world-signer-complete", NodeRole::Observer)
        .expect("config")
        .with_pos_config(pos_config)
        .expect("pos config")
        .with_replication(signed_replication_config(temp_dir("signer-complete"), 52));

    let err = PosNodeEngine::new(&config).expect_err("engine should reject incomplete signer map");
    assert!(matches!(
        err,
        NodeError::InvalidConfig { reason }
            if reason.contains("requires signer bindings for all validators")
    ));
}

#[test]
fn pos_engine_rejects_signed_mode_when_local_signer_binding_mismatches() {
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
    let pos_config =
        signed_pos_config_with_signer_seeds(validators, &[("node-a", 51), ("node-b", 52)]);
    let config = NodeConfig::new("node-b", "world-signer-local-mismatch", NodeRole::Observer)
        .expect("config")
        .with_pos_config(pos_config)
        .expect("pos config")
        .with_replication(signed_replication_config(
            temp_dir("signer-local-mismatch"),
            53,
        ));

    let err = PosNodeEngine::new(&config).expect_err("engine should reject local signer mismatch");
    assert!(matches!(
        err,
        NodeError::InvalidConfig { reason }
            if reason.contains("consensus signer binding mismatch for local validator")
    ));
}

#[test]
fn runtime_start_fails_when_pos_state_snapshot_is_corrupted() {
    let dir = temp_dir("pos-state-corrupt");
    fs::create_dir_all(&dir).expect("create dir");
    fs::write(dir.join("node_pos_state.json"), b"not-json").expect("write corrupted snapshot");

    let config = NodeConfig::new("node-a", "world-pos-corrupt", NodeRole::Observer)
        .expect("config")
        .with_replication_root(dir.clone())
        .expect("replication");
    let mut runtime = NodeRuntime::new(config);

    let err = runtime.start().expect_err("start should fail");
    assert!(matches!(err, NodeError::Replication { .. }));
    assert!(!runtime.snapshot().running);

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn runtime_start_fails_when_pos_state_snapshot_height_overflows() {
    let dir = temp_dir("pos-state-overflow");
    fs::create_dir_all(&dir).expect("create dir");
    let snapshot = super::pos_state_store::PosNodeStateSnapshot {
        next_height: 0,
        next_slot: 0,
        committed_height: u64::MAX,
        network_committed_height: u64::MAX,
        last_broadcast_proposal_height: 0,
        last_broadcast_local_attestation_height: 0,
        last_broadcast_committed_height: 0,
        last_committed_block_hash: None,
        last_execution_height: 0,
        last_execution_block_hash: None,
        last_execution_state_root: None,
    };
    fs::write(
        dir.join("node_pos_state.json"),
        serde_json::to_vec(&snapshot).expect("encode snapshot"),
    )
    .expect("write overflow snapshot");

    let config = NodeConfig::new("node-a", "world-pos-overflow", NodeRole::Observer)
        .expect("config")
        .with_replication_root(dir.clone())
        .expect("replication");
    let mut runtime = NodeRuntime::new(config);

    let err = runtime.start().expect_err("start should fail");
    assert!(
        matches!(err, NodeError::Replication { reason } if reason.contains("committed_height"))
    );
    let snapshot = runtime.snapshot();
    assert!(!snapshot.running);
    assert_eq!(snapshot.consensus.committed_height, 0);

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
    let pos_config =
        signed_pos_config_with_signer_seeds(validators, &[("node-a", 82), ("node-b", 81)]);

    let network_impl = Arc::new(TestInMemoryNetwork::default());
    let network: Arc<
        dyn agent_world_proto::distributed_net::DistributedNetwork<WorldError> + Send + Sync,
    > = network_impl.clone();

    let config = NodeConfig::new("node-b", world_id, NodeRole::Observer)
        .expect("config")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick")
        .with_pos_config(pos_config)
        .expect("pos config")
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

#[test]
fn runtime_replication_ingest_rejects_signed_writer_outside_allowlist() {
    let world_id = "world-repl-allowlist";
    let dir = temp_dir("repl-allowlist");
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
    let pos_config =
        signed_pos_config_with_signer_seeds(validators, &[("node-a", 91), ("node-b", 92)]);

    let network_impl = Arc::new(TestInMemoryNetwork::default());
    let network: Arc<
        dyn agent_world_proto::distributed_net::DistributedNetwork<WorldError> + Send + Sync,
    > = network_impl.clone();

    let config = NodeConfig::new("node-b", world_id, NodeRole::Observer)
        .expect("config")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick")
        .with_pos_config(pos_config)
        .expect("pos config")
        .with_replication(signed_replication_config(dir.clone(), 92));
    let mut runtime = NodeRuntime::new(config)
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));
    runtime.start().expect("start");

    let (unauthorized_private_hex, unauthorized_public_hex) = deterministic_keypair_hex(99);
    let unauthorized_message = signed_replication_message_for_writer(
        world_id,
        "node-a",
        unauthorized_private_hex.as_str(),
        unauthorized_public_hex.as_str(),
        1,
    );
    let encoded = serde_json::to_vec(&unauthorized_message).expect("encode message");
    let topic = super::network_bridge::default_replication_topic(world_id);
    network
        .publish(topic.as_str(), encoded.as_slice())
        .expect("publish unauthorized message");

    let unauthorized_rejected = wait_until(Instant::now() + Duration::from_secs(2), || {
        runtime
            .snapshot()
            .last_error
            .as_ref()
            .map(|reason| reason.contains("not authorized"))
            .unwrap_or(false)
    });
    assert!(
        unauthorized_rejected,
        "runtime did not reject unauthorized remote writer"
    );

    runtime.stop().expect("stop");
    let snapshot = runtime.snapshot();
    assert_eq!(snapshot.consensus.network_committed_height, 0);

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn runtime_fetch_handlers_reject_unsigned_fetch_request_in_signed_mode() {
    let world_id = "world-fetch-auth-hardening";
    let dir = temp_dir("fetch-auth-hardening");
    let pos_config = signed_pos_config_with_signer_seeds(
        vec![PosValidator {
            validator_id: "node-a".to_string(),
            stake: 100,
        }],
        &[("node-a", 111)],
    );
    let network: Arc<
        dyn agent_world_proto::distributed_net::DistributedNetwork<WorldError> + Send + Sync,
    > = Arc::new(TestInMemoryNetwork::default());

    let config = NodeConfig::new("node-a", world_id, NodeRole::Observer)
        .expect("config")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick")
        .with_pos_config(pos_config)
        .expect("pos config")
        .with_auto_attest_all_validators(true)
        .with_replication(signed_replication_config(dir.clone(), 111));
    let mut runtime = NodeRuntime::new(config)
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));
    runtime.start().expect("start");

    let unsigned_request = super::replication::FetchCommitRequest {
        world_id: world_id.to_string(),
        height: 1,
        requester_public_key_hex: None,
        requester_signature_hex: None,
    };
    let payload = serde_json::to_vec(&unsigned_request).expect("encode request");
    let err = network
        .request(
            super::replication::REPLICATION_FETCH_COMMIT_PROTOCOL,
            payload.as_slice(),
        )
        .expect_err("unsigned fetch request should be rejected");
    match err {
        WorldError::NetworkRequestFailed { code, message, .. } => {
            assert_eq!(code, DistributedErrorCode::ErrBadRequest);
            assert!(message.contains("authorization failed"));
        }
        other => panic!("unexpected error: {other:?}"),
    }

    runtime.stop().expect("stop");
    let _ = fs::remove_dir_all(dir);
}
