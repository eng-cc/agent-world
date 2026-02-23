use super::gossip_udp::{GossipCommitMessage, GossipEndpoint, GossipMessage};
use super::*;
use agent_world_consensus::node_consensus_signature::{
    sign_attestation_message, sign_commit_message, sign_proposal_message,
    verify_commit_message_signature, NodeConsensusMessageSigner as ConsensusMessageSigner,
};
use agent_world_distfs::{FileStore as _, LocalCasStore, SingleWriterReplicationGuard};
use agent_world_proto::distributed_net::NetworkSubscription;
use agent_world_proto::world_error::WorldError;
use ed25519_dalek::{Signer as _, SigningKey};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
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
struct FetchCommitRequestSigningPayload<'a> {
    version: u8,
    world_id: &'a str,
    height: u64,
    requester_public_key_hex: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct FetchBlobRequestSigningPayload<'a> {
    version: u8,
    content_hash: &'a str,
    requester_public_key_hex: Option<&'a str>,
}

fn signed_fetch_commit_request_for_test(
    world_id: &str,
    height: u64,
    signer_seed: u8,
) -> super::replication::FetchCommitRequest {
    let (private_hex, public_hex) = deterministic_keypair_hex(signer_seed);
    let signing_key_bytes: [u8; 32] = hex::decode(private_hex)
        .expect("private key decode")
        .try_into()
        .expect("private key len");
    let signing_key = SigningKey::from_bytes(&signing_key_bytes);
    let mut request = super::replication::FetchCommitRequest {
        world_id: world_id.to_string(),
        height,
        requester_public_key_hex: Some(public_hex),
        requester_signature_hex: None,
    };
    let payload = FetchCommitRequestSigningPayload {
        version: 1,
        world_id: request.world_id.as_str(),
        height: request.height,
        requester_public_key_hex: request.requester_public_key_hex.as_deref(),
    };
    let payload_bytes = serde_json::to_vec(&payload).expect("encode fetch-commit signing payload");
    let signature = signing_key.sign(payload_bytes.as_slice());
    request.requester_signature_hex = Some(hex::encode(signature.to_bytes()));
    request
}

fn signed_fetch_blob_request_for_test(
    content_hash: &str,
    signer_seed: u8,
) -> super::replication::FetchBlobRequest {
    let (private_hex, public_hex) = deterministic_keypair_hex(signer_seed);
    let signing_key_bytes: [u8; 32] = hex::decode(private_hex)
        .expect("private key decode")
        .try_into()
        .expect("private key len");
    let signing_key = SigningKey::from_bytes(&signing_key_bytes);
    let mut request = super::replication::FetchBlobRequest {
        content_hash: content_hash.to_string(),
        requester_public_key_hex: Some(public_hex),
        requester_signature_hex: None,
    };
    let payload = FetchBlobRequestSigningPayload {
        version: 1,
        content_hash: request.content_hash.as_str(),
        requester_public_key_hex: request.requester_public_key_hex.as_deref(),
    };
    let payload_bytes = serde_json::to_vec(&payload).expect("encode fetch-blob signing payload");
    let signature = signing_key.sign(payload_bytes.as_slice());
    request.requester_signature_hex = Some(hex::encode(signature.to_bytes()));
    request
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

fn with_noop_execution_hook(runtime: NodeRuntime) -> NodeRuntime {
    let calls: Arc<Mutex<Vec<NodeExecutionCommitContext>>> = Arc::new(Mutex::new(Vec::new()));
    runtime.with_execution_hook(RecordingExecutionHook::new(calls))
}

fn wait_until(deadline: Instant, mut predicate: impl FnMut() -> bool) -> bool {
    while Instant::now() < deadline {
        if predicate() {
            return true;
        }
        thread::sleep(Duration::from_millis(20));
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

impl TestInMemoryNetwork {
    fn clear_topic(&self, topic: &str) {
        self.retained
            .lock()
            .expect("lock retained")
            .insert(topic.to_string(), Vec::new());
        let subscribers = self.subscribers.lock().expect("lock subscribers");
        for inbox in subscribers.iter() {
            inbox
                .lock()
                .expect("lock subscriber inbox")
                .insert(topic.to_string(), Vec::new());
        }
    }
}

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
fn pos_engine_stays_pending_without_peer_votes_when_auto_attest_disabled() {
    let config = NodeConfig::new("node-a", "world-a", NodeRole::Observer)
        .expect("config")
        .with_pos_validators(multi_validators())
        .expect("validators")
        .with_auto_attest_all_validators(false);
    let mut engine = PosNodeEngine::new(&config).expect("engine");

    for offset in 0..12 {
        let snapshot = engine
            .tick(
                &config.node_id,
                &config.world_id,
                2_000 + offset,
                None,
                None,
                None,
                None,
                Vec::new(),
                None,
            )
            .expect("tick");
        assert_eq!(snapshot.consensus_snapshot.committed_height, 0);
    }
}

#[test]
fn pos_engine_apply_decision_rejects_height_overflow_without_state_mutation() {
    let config =
        NodeConfig::new("node-a", "world-overflow-apply", NodeRole::Observer).expect("config");
    let mut engine = PosNodeEngine::new(&config).expect("engine");
    engine.committed_height = 41;
    engine.network_committed_height = 43;
    engine.next_height = 44;
    engine.pending = Some(PendingProposal {
        height: 44,
        slot: 7,
        epoch: 0,
        proposer_id: "node-a".to_string(),
        block_hash: "pending-block".to_string(),
        action_root: empty_action_root(),
        committed_actions: Vec::new(),
        attestations: std::collections::BTreeMap::new(),
        approved_stake: 100,
        rejected_stake: 0,
        status: PosConsensusStatus::Pending,
    });

    let decision = PosDecision {
        height: u64::MAX,
        slot: 8,
        epoch: 0,
        status: PosConsensusStatus::Committed,
        block_hash: "overflow-block".to_string(),
        action_root: empty_action_root(),
        committed_actions: Vec::new(),
        approved_stake: 100,
        rejected_stake: 0,
        required_stake: 67,
        total_stake: 100,
    };

    let err = engine
        .apply_decision(&decision)
        .expect_err("height overflow must fail");
    assert!(
        matches!(err, NodeError::Consensus { reason } if reason.contains("decision.height overflow"))
    );
    assert_eq!(engine.committed_height, 41);
    assert_eq!(engine.network_committed_height, 43);
    assert_eq!(engine.next_height, 44);
    assert_eq!(
        engine
            .pending
            .as_ref()
            .map(|proposal| proposal.block_hash.as_str()),
        Some("pending-block")
    );
    assert!(engine.last_committed_block_hash.is_none());
}

#[test]
fn pos_engine_record_synced_replication_height_rejects_overflow_without_partial_state() {
    let config =
        NodeConfig::new("node-a", "world-overflow-sync", NodeRole::Observer).expect("config");
    let mut engine = PosNodeEngine::new(&config).expect("engine");
    engine.committed_height = 9;
    engine.next_height = 10;
    engine.pending = Some(PendingProposal {
        height: 10,
        slot: 1,
        epoch: 0,
        proposer_id: "node-a".to_string(),
        block_hash: "pending-sync".to_string(),
        action_root: empty_action_root(),
        committed_actions: Vec::new(),
        attestations: std::collections::BTreeMap::new(),
        approved_stake: 100,
        rejected_stake: 0,
        status: PosConsensusStatus::Pending,
    });

    let err = engine
        .record_synced_replication_height(u64::MAX, "overflow-block".to_string(), 7_700)
        .expect_err("height overflow must fail");
    assert!(matches!(err, NodeError::Replication { reason } if reason.contains("height overflow")));
    assert_eq!(engine.committed_height, 9);
    assert_eq!(engine.next_height, 10);
    assert_eq!(
        engine
            .pending
            .as_ref()
            .map(|proposal| proposal.block_hash.as_str()),
        Some("pending-sync")
    );
    assert!(engine.last_committed_block_hash.is_none());
    assert!(engine.last_committed_at_ms.is_none());
}

#[test]
fn pos_engine_ingest_proposal_rejects_slot_overflow_without_partial_state() {
    let config =
        NodeConfig::new("node-a", "world-overflow-proposal", NodeRole::Observer).expect("config");
    let mut engine = PosNodeEngine::new(&config).expect("engine");
    engine.next_height = 5;
    engine.next_slot = 3;

    let message = GossipProposalMessage {
        version: 1,
        world_id: config.world_id.clone(),
        node_id: config.node_id.clone(),
        player_id: config.player_id.clone(),
        proposer_id: config.node_id.clone(),
        height: 8,
        slot: u64::MAX,
        epoch: 0,
        block_hash: "proposal-overflow".to_string(),
        action_root: empty_action_root(),
        actions: Vec::new(),
        proposed_at_ms: 1_234,
        public_key_hex: None,
        signature_hex: None,
    };

    let err = engine
        .ingest_proposal_message(config.world_id.as_str(), &message)
        .expect_err("slot overflow must fail");
    assert!(
        matches!(err, NodeError::Consensus { reason } if reason.contains("proposal.slot overflow"))
    );
    assert_eq!(engine.next_height, 5);
    assert_eq!(engine.next_slot, 3);
    assert!(engine.pending.is_none());
}

#[test]
fn pos_engine_restore_state_snapshot_rejects_overflow_without_partial_state() {
    let config =
        NodeConfig::new("node-a", "world-overflow-restore", NodeRole::Observer).expect("config");
    let mut engine = PosNodeEngine::new(&config).expect("engine");
    engine.committed_height = 9;
    engine.network_committed_height = 10;
    engine.next_height = 11;
    engine.next_slot = 3;
    engine.pending = Some(PendingProposal {
        height: 11,
        slot: 3,
        epoch: 0,
        proposer_id: "node-a".to_string(),
        block_hash: "pending-restore".to_string(),
        action_root: empty_action_root(),
        committed_actions: Vec::new(),
        attestations: std::collections::BTreeMap::new(),
        approved_stake: 100,
        rejected_stake: 0,
        status: PosConsensusStatus::Pending,
    });

    let snapshot = super::pos_state_store::PosNodeStateSnapshot {
        next_height: 0,
        next_slot: 77,
        committed_height: u64::MAX,
        network_committed_height: u64::MAX,
        last_broadcast_proposal_height: 0,
        last_broadcast_local_attestation_height: 0,
        last_broadcast_committed_height: 0,
        last_committed_block_hash: Some("unexpected".to_string()),
        last_execution_height: 0,
        last_execution_block_hash: None,
        last_execution_state_root: None,
    };

    let err = engine
        .restore_state_snapshot(snapshot)
        .expect_err("committed height overflow must fail");
    assert!(
        matches!(err, NodeError::Replication { reason } if reason.contains("committed_height"))
    );
    assert_eq!(engine.committed_height, 9);
    assert_eq!(engine.network_committed_height, 10);
    assert_eq!(engine.next_height, 11);
    assert_eq!(engine.next_slot, 3);
    assert_eq!(
        engine
            .pending
            .as_ref()
            .map(|proposal| proposal.block_hash.as_str()),
        Some("pending-restore")
    );
}

#[test]
fn sequencer_commit_requires_execution_hook() {
    let config = NodeConfig::new("sequencer-a", "world-a", NodeRole::Sequencer)
        .expect("config")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick interval");
    let mut runtime = NodeRuntime::new(config);
    runtime.start().expect("start");
    thread::sleep(Duration::from_millis(80));
    runtime.stop().expect("stop");

    let snapshot = runtime.snapshot();
    assert_eq!(snapshot.consensus.committed_height, 0);
    assert!(snapshot
        .last_error
        .as_deref()
        .unwrap_or_default()
        .contains("execution hook is required"));
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
    let pos_config =
        signed_pos_config_with_signer_seeds(validators, &[("node-a", 203), ("node-b", 201)]);
    let config_b = NodeConfig::new("node-b", "world-sig-enforced", NodeRole::Observer)
        .expect("config b")
        .with_pos_config(pos_config)
        .expect("pos config")
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
    let pos_config =
        signed_pos_config_with_signer_seeds(validators, &[("node-a", 203), ("node-b", 202)]);
    let config_b = NodeConfig::new("node-b", "world-sig-accept", NodeRole::Observer)
        .expect("config b")
        .with_pos_config(pos_config)
        .expect("pos config")
        .with_replication(signed_replication_config(temp_dir("sig-accept"), 202));
    let mut engine = PosNodeEngine::new(&config_b).expect("engine");

    let (proposal_private_hex, proposal_public_hex) = deterministic_keypair_hex(203);
    let proposal_signing_key = SigningKey::from_bytes(
        &hex::decode(proposal_private_hex)
            .expect("proposal private decode")
            .try_into()
            .expect("proposal private len"),
    );
    let proposal_signer =
        ConsensusMessageSigner::new(proposal_signing_key, proposal_public_hex).expect("signer");

    let (attestation_private_hex, attestation_public_hex) = deterministic_keypair_hex(202);
    let attestation_signing_key = SigningKey::from_bytes(
        &hex::decode(attestation_private_hex)
            .expect("attestation private decode")
            .try_into()
            .expect("attestation private len"),
    );
    let attestation_signer =
        ConsensusMessageSigner::new(attestation_signing_key, attestation_public_hex)
            .expect("attestation signer");

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
    sign_proposal_message(&mut proposal, &proposal_signer).expect("sign proposal");
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
    sign_attestation_message(&mut attestation, &attestation_signer).expect("sign attestation");
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
    assert!(err.reason.contains("verify commit signature failed"));

    let mut tampered_action_root = commit.clone();
    tampered_action_root.action_root = "tampered-action-root".to_string();
    let err =
        verify_commit_message_signature(&tampered_action_root, true).expect_err("tamper must fail");
    assert!(err.reason.contains("verify commit signature failed"));
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
fn pos_engine_rejects_commit_without_execution_hashes_when_required() {
    let socket_a = UdpSocket::bind("127.0.0.1:0").expect("bind a");
    let socket_b = UdpSocket::bind("127.0.0.1:0").expect("bind b");
    let addr_a = socket_a.local_addr().expect("addr a");
    let addr_b = socket_b.local_addr().expect("addr b");
    drop(socket_a);
    drop(socket_b);

    let config = NodeConfig::new("node-b", "world-commit-exec-required", NodeRole::Observer)
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
        .with_require_peer_execution_hashes(true)
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
            execution_block_hash: None,
            execution_state_root: None,
            public_key_hex: None,
            signature_hex: None,
        })
        .expect("broadcast commit");
    thread::sleep(Duration::from_millis(20));

    engine
        .ingest_peer_messages(&endpoint_b, &config.node_id, &config.world_id, None)
        .expect("ingest");
    assert!(
        !engine.peer_heads.contains_key("node-a"),
        "peer head with missing execution hashes must be rejected"
    );
}

#[test]
fn pos_engine_rejects_commit_when_execution_binding_mismatches_local() {
    let socket_a = UdpSocket::bind("127.0.0.1:0").expect("bind a");
    let socket_b = UdpSocket::bind("127.0.0.1:0").expect("bind b");
    let addr_a = socket_a.local_addr().expect("addr a");
    let addr_b = socket_b.local_addr().expect("addr b");
    drop(socket_a);
    drop(socket_b);

    let config = NodeConfig::new("node-b", "world-commit-exec-mismatch", NodeRole::Observer)
        .expect("config")
        .with_require_peer_execution_hashes(true)
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

    let calls = Arc::new(Mutex::new(Vec::new()));
    let mut hook = RecordingExecutionHook::new(calls);
    let tick = engine
        .tick(
            &config.node_id,
            &config.world_id,
            1_000,
            None,
            None,
            None,
            None,
            Vec::new(),
            Some(&mut hook),
        )
        .expect("tick");
    assert_eq!(tick.consensus_snapshot.committed_height, 1);
    assert_eq!(engine.last_execution_height, 1);

    endpoint_a
        .broadcast_commit(&GossipCommitMessage {
            version: 1,
            world_id: config.world_id.clone(),
            node_id: "node-a".to_string(),
            player_id: "node-a".to_string(),
            height: 1,
            slot: 1,
            epoch: 0,
            block_hash: "block-peer-1".to_string(),
            action_root: empty_action_root(),
            actions: Vec::new(),
            committed_at_ms: 1_100,
            execution_block_hash: Some("exec-block-mismatch".to_string()),
            execution_state_root: Some("exec-state-mismatch".to_string()),
            public_key_hex: None,
            signature_hex: None,
        })
        .expect("broadcast commit");
    thread::sleep(Duration::from_millis(20));

    engine
        .ingest_peer_messages(&endpoint_b, &config.node_id, &config.world_id, None)
        .expect("ingest");
    assert!(
        !engine.peer_heads.contains_key("node-a"),
        "peer head with mismatched execution binding must be rejected"
    );
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
fn gossip_endpoint_learns_inbound_peer_for_followup_broadcasts() {
    let socket_a = UdpSocket::bind("127.0.0.1:0").expect("bind a");
    let socket_b = UdpSocket::bind("127.0.0.1:0").expect("bind b");
    let addr_a = socket_a.local_addr().expect("addr a");
    let addr_b = socket_b.local_addr().expect("addr b");
    drop(socket_a);
    drop(socket_b);

    let world_id = "world-gossip-discovery";
    let config_a = NodeConfig::new("node-a", world_id, NodeRole::Observer)
        .expect("config a")
        .with_pos_validators(vec![
            PosValidator {
                validator_id: "node-a".to_string(),
                stake: 50,
            },
            PosValidator {
                validator_id: "node-b".to_string(),
                stake: 50,
            },
        ])
        .expect("validators")
        .with_gossip_optional(addr_a, Vec::new());
    let mut engine_a = PosNodeEngine::new(&config_a).expect("engine a");

    let endpoint_a = GossipEndpoint::bind(&NodeGossipConfig {
        bind_addr: addr_a,
        peers: Vec::new(),
    })
    .expect("endpoint a");
    let endpoint_b = GossipEndpoint::bind(&NodeGossipConfig {
        bind_addr: addr_b,
        peers: vec![addr_a],
    })
    .expect("endpoint b");

    endpoint_b
        .broadcast_commit(&GossipCommitMessage {
            version: 1,
            world_id: world_id.to_string(),
            node_id: "node-b".to_string(),
            player_id: "node-b".to_string(),
            height: 1,
            slot: 1,
            epoch: 0,
            block_hash: "block-b-1".to_string(),
            action_root: empty_action_root(),
            actions: Vec::new(),
            committed_at_ms: 1_000,
            execution_block_hash: None,
            execution_state_root: None,
            public_key_hex: None,
            signature_hex: None,
        })
        .expect("broadcast to a");
    thread::sleep(Duration::from_millis(20));
    engine_a
        .ingest_peer_messages(&endpoint_a, "node-a", world_id, None)
        .expect("ingest from b");

    endpoint_a
        .broadcast_commit(&GossipCommitMessage {
            version: 1,
            world_id: world_id.to_string(),
            node_id: "node-a".to_string(),
            player_id: "node-a".to_string(),
            height: 2,
            slot: 2,
            epoch: 0,
            block_hash: "block-a-2".to_string(),
            action_root: empty_action_root(),
            actions: Vec::new(),
            committed_at_ms: 2_000,
            execution_block_hash: None,
            execution_state_root: None,
            public_key_hex: None,
            signature_hex: None,
        })
        .expect("rebroadcast to discovered peer");
    thread::sleep(Duration::from_millis(20));

    let echoed = endpoint_b.drain_messages().expect("drain endpoint b");
    assert!(echoed.iter().any(|received| {
        matches!(
            &received.message,
            GossipMessage::Commit(commit) if commit.node_id == "node-a" && commit.height == 2
        )
    }));
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

    let mut runtime_a = with_noop_execution_hook(NodeRuntime::new(config_a));
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
fn runtime_network_consensus_syncs_peer_heads_without_udp_gossip() {
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

    let config_a = NodeConfig::new("node-a", "world-network-consensus", NodeRole::Sequencer)
        .expect("config a")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick a")
        .with_pos_validators(validators.clone())
        .expect("validators a")
        .with_auto_attest_all_validators(true);
    let config_b = NodeConfig::new("node-b", "world-network-consensus", NodeRole::Observer)
        .expect("config b")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick b")
        .with_pos_validators(validators)
        .expect("validators b");

    let mut runtime_a = with_noop_execution_hook(NodeRuntime::new(config_a))
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));
    let mut runtime_b = NodeRuntime::new(config_b)
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));
    runtime_a.start().expect("start a");
    runtime_b.start().expect("start b");
    thread::sleep(Duration::from_millis(200));

    let snapshot_a = runtime_a.snapshot();
    let snapshot_b = runtime_b.snapshot();
    assert!(snapshot_a.consensus.committed_height >= 1);
    assert!(snapshot_b.consensus.network_committed_height >= 1);
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

    let mut runtime_a = with_noop_execution_hook(NodeRuntime::new(config_a));
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
    let pos_config =
        signed_pos_config_with_signer_seeds(validators, &[("node-a", 71), ("node-b", 72)]);
    let network: Arc<
        dyn agent_world_proto::distributed_net::DistributedNetwork<WorldError> + Send + Sync,
    > = Arc::new(TestInMemoryNetwork::default());

    let config_a = NodeConfig::new("node-a", "world-network-repl", NodeRole::Sequencer)
        .expect("config a")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick a")
        .with_pos_config(pos_config.clone())
        .expect("pos config a")
        .with_auto_attest_all_validators(true)
        .with_replication(signed_replication_config(dir_a.clone(), 71));
    let config_b = NodeConfig::new("node-b", "world-network-repl", NodeRole::Observer)
        .expect("config b")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick b")
        .with_pos_config(pos_config)
        .expect("pos config b")
        .with_replication(signed_replication_config(dir_b.clone(), 72));

    let mut runtime_a = with_noop_execution_hook(NodeRuntime::new(config_a))
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
fn runtime_network_replication_fetch_handlers_serve_commit_and_blob() {
    let dir_a = temp_dir("network-fetch-a");
    let validators = vec![PosValidator {
        validator_id: "node-a".to_string(),
        stake: 100,
    }];
    let pos_config = signed_pos_config_with_signer_seeds(validators, &[("node-a", 77)]);
    let network: Arc<
        dyn agent_world_proto::distributed_net::DistributedNetwork<WorldError> + Send + Sync,
    > = Arc::new(TestInMemoryNetwork::default());

    let config_a = NodeConfig::new("node-a", "world-network-fetch", NodeRole::Sequencer)
        .expect("config a")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick a")
        .with_pos_config(pos_config)
        .expect("pos config a")
        .with_auto_attest_all_validators(true)
        .with_replication(signed_replication_config(dir_a.clone(), 77));
    let mut runtime_a = with_noop_execution_hook(NodeRuntime::new(config_a))
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));

    runtime_a.start().expect("start a");
    thread::sleep(Duration::from_millis(180));
    let snapshot = runtime_a.snapshot();
    assert!(snapshot.consensus.committed_height >= 1);
    let target_height = snapshot.consensus.committed_height;

    let fetch_commit_request =
        signed_fetch_commit_request_for_test("world-network-fetch", target_height, 77);
    let fetch_commit_payload =
        serde_json::to_vec(&fetch_commit_request).expect("encode fetch commit request");
    let fetch_commit_response_payload = network
        .request(
            super::replication::REPLICATION_FETCH_COMMIT_PROTOCOL,
            fetch_commit_payload.as_slice(),
        )
        .expect("fetch commit response");
    let fetch_commit_response: super::replication::FetchCommitResponse =
        serde_json::from_slice(&fetch_commit_response_payload).expect("decode fetch commit");
    assert!(fetch_commit_response.found);
    let commit_message = fetch_commit_response.message.expect("commit message");
    assert_eq!(commit_message.world_id, "world-network-fetch");
    assert_eq!(commit_message.record.world_id, "world-network-fetch");
    assert_eq!(
        commit_message.record.path,
        format!("consensus/commits/{:020}.json", target_height)
    );

    let fetch_blob_request =
        signed_fetch_blob_request_for_test(commit_message.record.content_hash.as_str(), 77);
    let fetch_blob_payload =
        serde_json::to_vec(&fetch_blob_request).expect("encode fetch blob request");
    let fetch_blob_response_payload = network
        .request(
            super::replication::REPLICATION_FETCH_BLOB_PROTOCOL,
            fetch_blob_payload.as_slice(),
        )
        .expect("fetch blob response");
    let fetch_blob_response: super::replication::FetchBlobResponse =
        serde_json::from_slice(&fetch_blob_response_payload).expect("decode fetch blob");
    assert!(fetch_blob_response.found);
    assert_eq!(
        fetch_blob_response.blob.expect("blob payload"),
        commit_message.payload
    );

    runtime_a.stop().expect("stop a");
    let _ = fs::remove_dir_all(&dir_a);
}

#[test]
fn runtime_network_replication_gap_sync_fetches_missing_commits() {
    let world_id = "world-network-gap";
    let dir_a = temp_dir("network-gap-a");
    let dir_b = temp_dir("network-gap-b");
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
        signed_pos_config_with_signer_seeds(validators, &[("node-a", 78), ("node-b", 79)]);
    let network_impl = Arc::new(TestInMemoryNetwork::default());
    let network: Arc<
        dyn agent_world_proto::distributed_net::DistributedNetwork<WorldError> + Send + Sync,
    > = network_impl.clone();

    let config_a = NodeConfig::new("node-a", world_id, NodeRole::Sequencer)
        .expect("config a")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick a")
        .with_pos_config(pos_config.clone())
        .expect("pos config a")
        .with_auto_attest_all_validators(true)
        .with_replication(signed_replication_config(dir_a.clone(), 78));
    let config_b = NodeConfig::new("node-b", world_id, NodeRole::Observer)
        .expect("config b")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick b")
        .with_pos_config(pos_config)
        .expect("pos config b")
        .with_replication(signed_replication_config(dir_b.clone(), 79));

    let mut runtime_a = with_noop_execution_hook(NodeRuntime::new(config_a))
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));
    runtime_a.start().expect("start a");
    let reached = wait_until(Instant::now() + Duration::from_secs(2), || {
        runtime_a.snapshot().consensus.committed_height >= 3
    });
    assert!(reached, "sequencer did not reach target height in time");
    let target_height = runtime_a.snapshot().consensus.committed_height;
    runtime_a.stop().expect("stop a");

    let mut commit_map = HashMap::<u64, super::replication::GossipReplicationMessage>::new();
    let mut blob_map = HashMap::<String, Vec<u8>>::new();
    for height in 1..=target_height {
        let request = signed_fetch_commit_request_for_test(world_id, height, 78);
        let payload = serde_json::to_vec(&request).expect("encode commit request");
        let response_payload = network
            .request(
                super::replication::REPLICATION_FETCH_COMMIT_PROTOCOL,
                payload.as_slice(),
            )
            .expect("fetch commit");
        let response: super::replication::FetchCommitResponse =
            serde_json::from_slice(&response_payload).expect("decode commit response");
        assert!(response.found, "missing fetched commit at height {height}");
        let message = response.message.expect("commit payload");
        blob_map.insert(message.record.content_hash.clone(), message.payload.clone());
        commit_map.insert(height, message);
    }
    assert_eq!(commit_map.len() as u64, target_height);
    let high_message = commit_map
        .get(&target_height)
        .cloned()
        .expect("high commit message");

    let topic = super::network_bridge::default_replication_topic(world_id);
    network_impl.clear_topic(topic.as_str());
    network_impl
        .clear_topic(super::network_bridge::default_consensus_proposal_topic(world_id).as_str());
    network_impl
        .clear_topic(super::network_bridge::default_consensus_attestation_topic(world_id).as_str());
    network_impl
        .clear_topic(super::network_bridge::default_consensus_commit_topic(world_id).as_str());
    let high_payload = serde_json::to_vec(&high_message).expect("encode high message");
    network
        .publish(topic.as_str(), high_payload.as_slice())
        .expect("publish high message");

    let mut runtime_b = NodeRuntime::new(config_b)
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));
    runtime_b.start().expect("start b");

    let commit_map = Arc::new(commit_map);
    let blob_map = Arc::new(blob_map);
    let commit_world_id = world_id.to_string();
    network
        .register_handler(
            super::replication::REPLICATION_FETCH_COMMIT_PROTOCOL,
            Box::new(move |payload| {
                let request =
                    serde_json::from_slice::<super::replication::FetchCommitRequest>(payload)
                        .map_err(|err| WorldError::DistributedValidationFailed {
                            reason: format!("decode fetch commit request failed: {err}"),
                        })?;
                if request.world_id != commit_world_id {
                    return Err(WorldError::DistributedValidationFailed {
                        reason: format!(
                            "world mismatch expected={} actual={}",
                            commit_world_id, request.world_id
                        ),
                    });
                }
                let response = super::replication::FetchCommitResponse {
                    found: commit_map.contains_key(&request.height),
                    message: commit_map.get(&request.height).cloned(),
                };
                serde_json::to_vec(&response).map_err(|err| {
                    WorldError::DistributedValidationFailed {
                        reason: format!("encode fetch commit response failed: {err}"),
                    }
                })
            }),
        )
        .expect("register commit handler");
    network
        .register_handler(
            super::replication::REPLICATION_FETCH_BLOB_PROTOCOL,
            Box::new(move |payload| {
                let request =
                    serde_json::from_slice::<super::replication::FetchBlobRequest>(payload)
                        .map_err(|err| WorldError::DistributedValidationFailed {
                            reason: format!("decode fetch blob request failed: {err}"),
                        })?;
                let response = super::replication::FetchBlobResponse {
                    found: blob_map.contains_key(request.content_hash.as_str()),
                    blob: blob_map.get(request.content_hash.as_str()).cloned(),
                };
                serde_json::to_vec(&response).map_err(|err| {
                    WorldError::DistributedValidationFailed {
                        reason: format!("encode fetch blob response failed: {err}"),
                    }
                })
            }),
        )
        .expect("register blob handler");

    let synced = wait_until(Instant::now() + Duration::from_secs(3), || {
        runtime_b.snapshot().consensus.committed_height >= target_height
    });
    assert!(synced, "observer did not sync missing commits in time");

    runtime_b.stop().expect("stop b");
    let snapshot_b = runtime_b.snapshot();
    assert!(snapshot_b.last_error.is_none());
    assert!(snapshot_b.consensus.committed_height >= target_height);

    let store_b = LocalCasStore::new(dir_b.join("store"));
    let files = store_b.list_files().expect("list files");
    assert!(files
        .iter()
        .any(|item| item.path == "consensus/commits/00000000000000000001.json"));
    assert!(files
        .iter()
        .any(|item| { item.path == format!("consensus/commits/{:020}.json", target_height) }));

    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);
}

#[test]
fn runtime_network_replication_gap_sync_not_found_is_non_fatal() {
    let world_id = "world-network-gap-not-found";
    let dir_a = temp_dir("network-gap-not-found-a");
    let dir_b = temp_dir("network-gap-not-found-b");
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
        signed_pos_config_with_signer_seeds(validators, &[("node-a", 87), ("node-b", 88)]);
    let network_impl = Arc::new(TestInMemoryNetwork::default());
    let network: Arc<
        dyn agent_world_proto::distributed_net::DistributedNetwork<WorldError> + Send + Sync,
    > = network_impl.clone();

    let config_a = NodeConfig::new("node-a", world_id, NodeRole::Sequencer)
        .expect("config a")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick a")
        .with_pos_config(pos_config.clone())
        .expect("pos config a")
        .with_auto_attest_all_validators(true)
        .with_replication(signed_replication_config(dir_a.clone(), 87));
    let config_b = NodeConfig::new("node-b", world_id, NodeRole::Observer)
        .expect("config b")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick b")
        .with_pos_config(pos_config)
        .expect("pos config b")
        .with_replication(signed_replication_config(dir_b.clone(), 88));

    let mut runtime_a = with_noop_execution_hook(NodeRuntime::new(config_a))
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));
    runtime_a.start().expect("start a");
    let reached = wait_until(Instant::now() + Duration::from_secs(2), || {
        runtime_a.snapshot().consensus.committed_height >= 3
    });
    assert!(reached, "sequencer did not reach target height in time");
    let target_height = runtime_a.snapshot().consensus.committed_height;
    runtime_a.stop().expect("stop a");

    let request = signed_fetch_commit_request_for_test(world_id, target_height, 87);
    let payload = serde_json::to_vec(&request).expect("encode commit request");
    let response_payload = network
        .request(
            super::replication::REPLICATION_FETCH_COMMIT_PROTOCOL,
            payload.as_slice(),
        )
        .expect("fetch commit");
    let response: super::replication::FetchCommitResponse =
        serde_json::from_slice(&response_payload).expect("decode commit response");
    assert!(response.found, "missing high commit");
    let high_message = response.message.expect("high commit payload");

    let topic = super::network_bridge::default_replication_topic(world_id);
    network_impl.clear_topic(topic.as_str());
    network_impl
        .clear_topic(super::network_bridge::default_consensus_proposal_topic(world_id).as_str());
    network_impl
        .clear_topic(super::network_bridge::default_consensus_attestation_topic(world_id).as_str());
    network_impl
        .clear_topic(super::network_bridge::default_consensus_commit_topic(world_id).as_str());
    let high_payload = serde_json::to_vec(&high_message).expect("encode high message");
    network
        .publish(topic.as_str(), high_payload.as_slice())
        .expect("publish high message");

    network
        .register_handler(
            super::replication::REPLICATION_FETCH_COMMIT_PROTOCOL,
            Box::new(move |_payload| {
                let response = super::replication::FetchCommitResponse {
                    found: false,
                    message: None,
                };
                serde_json::to_vec(&response).map_err(|err| {
                    WorldError::DistributedValidationFailed {
                        reason: format!("encode fetch commit response failed: {err}"),
                    }
                })
            }),
        )
        .expect("register commit not found handler");

    let mut runtime_b = NodeRuntime::new(config_b)
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));
    runtime_b.start().expect("start b");
    thread::sleep(Duration::from_millis(250));

    let snapshot_b = runtime_b.snapshot();
    assert!(
        !snapshot_b
            .last_error
            .as_deref()
            .map(|reason| reason.contains("gap sync height"))
            .unwrap_or(false),
        "not found gap sync should not be reported as fatal error"
    );
    assert!(
        snapshot_b.consensus.committed_height < target_height,
        "observer should keep waiting when target height is not found"
    );

    runtime_b.stop().expect("stop b");
    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);
}

#[test]
fn runtime_network_replication_gap_sync_reports_error_after_retries_exhausted() {
    let world_id = "world-network-gap-retry-exhausted";
    let dir_a = temp_dir("network-gap-retry-a");
    let dir_b = temp_dir("network-gap-retry-b");
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
        signed_pos_config_with_signer_seeds(validators, &[("node-a", 89), ("node-b", 90)]);
    let network_impl = Arc::new(TestInMemoryNetwork::default());
    let network: Arc<
        dyn agent_world_proto::distributed_net::DistributedNetwork<WorldError> + Send + Sync,
    > = network_impl.clone();

    let config_a = NodeConfig::new("node-a", world_id, NodeRole::Sequencer)
        .expect("config a")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick a")
        .with_pos_config(pos_config.clone())
        .expect("pos config a")
        .with_auto_attest_all_validators(true)
        .with_replication(signed_replication_config(dir_a.clone(), 89));
    let config_b = NodeConfig::new("node-b", world_id, NodeRole::Observer)
        .expect("config b")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick b")
        .with_pos_config(pos_config)
        .expect("pos config b")
        .with_replication(signed_replication_config(dir_b.clone(), 90));

    let mut runtime_a = with_noop_execution_hook(NodeRuntime::new(config_a))
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));
    runtime_a.start().expect("start a");
    let reached = wait_until(Instant::now() + Duration::from_secs(2), || {
        runtime_a.snapshot().consensus.committed_height >= 3
    });
    assert!(reached, "sequencer did not reach target height in time");
    let target_height = runtime_a.snapshot().consensus.committed_height;
    runtime_a.stop().expect("stop a");

    let request = signed_fetch_commit_request_for_test(world_id, target_height, 89);
    let payload = serde_json::to_vec(&request).expect("encode commit request");
    let response_payload = network
        .request(
            super::replication::REPLICATION_FETCH_COMMIT_PROTOCOL,
            payload.as_slice(),
        )
        .expect("fetch commit");
    let response: super::replication::FetchCommitResponse =
        serde_json::from_slice(&response_payload).expect("decode commit response");
    assert!(response.found, "missing high commit");
    let high_message = response.message.expect("high commit payload");

    let topic = super::network_bridge::default_replication_topic(world_id);
    network_impl.clear_topic(topic.as_str());
    network_impl
        .clear_topic(super::network_bridge::default_consensus_proposal_topic(world_id).as_str());
    network_impl
        .clear_topic(super::network_bridge::default_consensus_attestation_topic(world_id).as_str());
    network_impl
        .clear_topic(super::network_bridge::default_consensus_commit_topic(world_id).as_str());
    let high_payload = serde_json::to_vec(&high_message).expect("encode high message");
    network
        .publish(topic.as_str(), high_payload.as_slice())
        .expect("publish high message");

    let mut runtime_b = NodeRuntime::new(config_b)
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));
    runtime_b.start().expect("start b");
    network
        .register_handler(
            super::replication::REPLICATION_FETCH_COMMIT_PROTOCOL,
            Box::new(move |_payload| {
                Err(WorldError::NetworkProtocolUnavailable {
                    protocol: "forced-gap-sync-retry-failure".to_string(),
                })
            }),
        )
        .expect("register commit retry-failure handler");
    let errored = wait_until(Instant::now() + Duration::from_secs(3), || {
        runtime_b
            .snapshot()
            .last_error
            .as_deref()
            .map(|reason| {
                reason.contains("gap sync height")
                    && reason.contains("failed after 3 attempts")
                    && reason.contains("attempt 3/3 failed")
            })
            .unwrap_or(false)
    });
    let snapshot_b = runtime_b.snapshot();
    assert!(
        errored,
        "observer did not report gap sync retry exhaustion: committed_height={} network_committed_height={} last_error={:?}",
        snapshot_b.consensus.committed_height,
        snapshot_b.consensus.network_committed_height,
        snapshot_b.last_error
    );

    runtime_b.stop().expect("stop b");
    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);
}

#[test]
fn runtime_replication_storage_challenge_gate_blocks_on_local_probe_failure() {
    let dir = temp_dir("challenge-gate-local");
    let pos_config = signed_pos_config_with_signer_seeds(
        vec![PosValidator {
            validator_id: "node-a".to_string(),
            stake: 100,
        }],
        &[("node-a", 83)],
    );
    let config = NodeConfig::new("node-a", "world-challenge-local", NodeRole::Sequencer)
        .expect("config")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick")
        .with_pos_config(pos_config)
        .expect("pos config")
        .with_auto_attest_all_validators(true)
        .with_replication(signed_replication_config(dir.clone(), 83));
    let mut runtime = with_noop_execution_hook(NodeRuntime::new(config));

    runtime.start().expect("start runtime");
    let committed = wait_until(Instant::now() + Duration::from_secs(2), || {
        runtime.snapshot().consensus.committed_height >= 1
    });
    assert!(committed, "runtime did not produce first commit in time");

    let store = LocalCasStore::new(dir.join("store"));
    for entry in fs::read_dir(store.blobs_dir()).expect("list blobs") {
        let entry = entry.expect("blob entry");
        if entry.file_type().expect("blob type").is_file() {
            fs::write(entry.path(), b"tampered-local-blob").expect("tamper blob");
        }
    }

    let errored = wait_until(Instant::now() + Duration::from_secs(3), || {
        runtime
            .snapshot()
            .last_error
            .as_deref()
            .map(|reason| reason.contains("storage challenge gate failed"))
            .unwrap_or(false)
    });
    assert!(
        errored,
        "runtime did not report storage challenge gate failure"
    );

    runtime.stop().expect("stop runtime");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn runtime_replication_storage_challenge_gate_blocks_on_network_blob_mismatch() {
    let dir = temp_dir("challenge-gate-network");
    let network: Arc<
        dyn agent_world_proto::distributed_net::DistributedNetwork<WorldError> + Send + Sync,
    > = Arc::new(TestInMemoryNetwork::default());
    let pos_config = signed_pos_config_with_signer_seeds(
        vec![PosValidator {
            validator_id: "node-a".to_string(),
            stake: 100,
        }],
        &[("node-a", 84)],
    );
    let config = NodeConfig::new("node-a", "world-challenge-network", NodeRole::Sequencer)
        .expect("config")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick")
        .with_pos_config(pos_config)
        .expect("pos config")
        .with_auto_attest_all_validators(true)
        .with_replication(signed_replication_config(dir.clone(), 84));
    let mut runtime = with_noop_execution_hook(NodeRuntime::new(config))
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));

    runtime.start().expect("start runtime");
    let committed = wait_until(Instant::now() + Duration::from_secs(2), || {
        runtime.snapshot().consensus.committed_height >= 1
    });
    assert!(committed, "runtime did not produce first commit in time");

    network
        .register_handler(
            super::replication::REPLICATION_FETCH_BLOB_PROTOCOL,
            Box::new(|payload| {
                let request =
                    serde_json::from_slice::<super::replication::FetchBlobRequest>(payload)
                        .map_err(|err| WorldError::DistributedValidationFailed {
                            reason: format!("decode fetch blob request failed: {err}"),
                        })?;
                let response = super::replication::FetchBlobResponse {
                    found: true,
                    blob: Some(format!("bad-{}", request.content_hash).into_bytes()),
                };
                serde_json::to_vec(&response).map_err(|err| {
                    WorldError::DistributedValidationFailed {
                        reason: format!("encode fetch blob response failed: {err}"),
                    }
                })
            }),
        )
        .expect("register mismatched blob handler");

    let errored = wait_until(Instant::now() + Duration::from_secs(3), || {
        runtime
            .snapshot()
            .last_error
            .as_deref()
            .map(|reason| {
                reason.contains("network threshold unmet")
                    && reason.contains("network blob hash mismatch")
            })
            .unwrap_or(false)
    });
    assert!(
        errored,
        "runtime did not report network blob mismatch gate failure"
    );

    runtime.stop().expect("stop runtime");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn runtime_replication_storage_challenge_gate_allows_when_network_matches_reach_threshold() {
    let dir = temp_dir("challenge-gate-threshold-pass");
    let network: Arc<
        dyn agent_world_proto::distributed_net::DistributedNetwork<WorldError> + Send + Sync,
    > = Arc::new(TestInMemoryNetwork::default());
    let pos_config = signed_pos_config_with_signer_seeds(
        vec![PosValidator {
            validator_id: "node-a".to_string(),
            stake: 100,
        }],
        &[("node-a", 86)],
    );
    let config = NodeConfig::new(
        "node-a",
        "world-challenge-threshold-pass",
        NodeRole::Sequencer,
    )
    .expect("config")
    .with_tick_interval(Duration::from_millis(10))
    .expect("tick")
    .with_pos_config(pos_config)
    .expect("pos config")
    .with_auto_attest_all_validators(true)
    .with_replication(signed_replication_config(dir.clone(), 86));
    let mut runtime = with_noop_execution_hook(NodeRuntime::new(config))
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));

    let root_for_handler = dir.clone();
    let matched_hashes = Arc::new(Mutex::new(Vec::<String>::new()));
    let matched_hashes_for_handler = Arc::clone(&matched_hashes);
    network
        .register_handler(
            super::replication::REPLICATION_FETCH_BLOB_PROTOCOL,
            Box::new(move |payload| {
                let request =
                    serde_json::from_slice::<super::replication::FetchBlobRequest>(payload)
                        .map_err(|err| WorldError::DistributedValidationFailed {
                            reason: format!("decode fetch blob request failed: {err}"),
                        })?;
                let maybe_local = super::replication::load_blob_from_root(
                    root_for_handler.as_path(),
                    request.content_hash.as_str(),
                )
                .map_err(|err| WorldError::DistributedValidationFailed {
                    reason: format!("load local blob failed: {err}"),
                })?;
                let Some(local_blob) = maybe_local else {
                    let response = super::replication::FetchBlobResponse {
                        found: false,
                        blob: None,
                    };
                    return serde_json::to_vec(&response).map_err(|err| {
                        WorldError::DistributedValidationFailed {
                            reason: format!("encode fetch blob response failed: {err}"),
                        }
                    });
                };

                let mut matched_hashes = matched_hashes_for_handler
                    .lock()
                    .expect("lock matched hashes");
                if matched_hashes.len() < 2
                    && !matched_hashes
                        .iter()
                        .any(|hash| hash == &request.content_hash)
                {
                    matched_hashes.push(request.content_hash.clone());
                }
                let should_match = matched_hashes
                    .iter()
                    .any(|hash| hash == &request.content_hash);
                drop(matched_hashes);
                let response = super::replication::FetchBlobResponse {
                    found: true,
                    blob: Some(if should_match {
                        local_blob
                    } else {
                        format!("bad-{}", request.content_hash).into_bytes()
                    }),
                };
                serde_json::to_vec(&response).map_err(|err| {
                    WorldError::DistributedValidationFailed {
                        reason: format!("encode fetch blob response failed: {err}"),
                    }
                })
            }),
        )
        .expect("register threshold pass blob handler");

    runtime.start().expect("start runtime");
    let advanced = wait_until(Instant::now() + Duration::from_secs(2), || {
        runtime.snapshot().consensus.committed_height >= 5
    });
    assert!(
        advanced,
        "runtime did not continue committing under threshold-based gate"
    );

    let snapshot = runtime.snapshot();
    assert!(
        !snapshot
            .last_error
            .as_deref()
            .map(|reason| reason.contains("network threshold unmet"))
            .unwrap_or(false),
        "runtime should not report threshold unmet when enough matches are available"
    );

    runtime.stop().expect("stop runtime");
    let _ = fs::remove_dir_all(&dir);
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
    let pos_config =
        signed_pos_config_with_signer_seeds(validators, &[("node-a", 81), ("node-b", 82)]);
    let network: Arc<
        dyn agent_world_proto::distributed_net::DistributedNetwork<WorldError> + Send + Sync,
    > = Arc::new(TestInMemoryNetwork::default());

    let config_a = NodeConfig::new("node-a", "world-topic-repl", NodeRole::Sequencer)
        .expect("config a")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick a")
        .with_pos_config(pos_config.clone())
        .expect("pos config a")
        .with_auto_attest_all_validators(true)
        .with_replication(signed_replication_config(dir_a.clone(), 81));
    let config_b = NodeConfig::new("node-b", "world-topic-repl", NodeRole::Observer)
        .expect("config b")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick b")
        .with_pos_config(pos_config)
        .expect("pos config b")
        .with_replication(signed_replication_config(dir_b.clone(), 82));

    let mut runtime_a = with_noop_execution_hook(NodeRuntime::new(config_a))
        .with_replication_network(
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
    let pos_config =
        signed_pos_config_with_signer_seeds(validators, &[("node-a", 11), ("node-b", 22)]);

    let config_a = NodeConfig::new("node-a", "world-signed", NodeRole::Sequencer)
        .expect("config a")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick a")
        .with_pos_config(pos_config.clone())
        .expect("pos config a")
        .with_gossip_optional(addr_a, vec![addr_b])
        .with_replication(signed_replication_config(dir_a.clone(), 11));
    let config_b = NodeConfig::new("node-b", "world-signed", NodeRole::Observer)
        .expect("config b")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick b")
        .with_pos_config(pos_config)
        .expect("pos config b")
        .with_gossip_optional(addr_b, vec![addr_a])
        .with_replication(signed_replication_config(dir_b.clone(), 22));

    let mut runtime_a = with_noop_execution_hook(NodeRuntime::new(config_a));
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
    let pos_config =
        signed_pos_config_with_signer_seeds(validators, &[("node-a", 11), ("node-b", 33)]);

    let config_a = NodeConfig::new("node-a", "world-enforced", NodeRole::Sequencer)
        .expect("config a")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick a")
        .with_pos_config(pos_config.clone())
        .expect("pos config a")
        .with_gossip_optional(addr_a, vec![addr_b])
        .with_replication_root(dir_a.clone())
        .expect("replication a");
    let config_b = NodeConfig::new("node-b", "world-enforced", NodeRole::Observer)
        .expect("config b")
        .with_tick_interval(Duration::from_millis(10))
        .expect("tick b")
        .with_pos_config(pos_config)
        .expect("pos config b")
        .with_gossip_optional(addr_b, vec![addr_a])
        .with_replication(signed_replication_config(dir_b.clone(), 33));

    let mut runtime_a = with_noop_execution_hook(NodeRuntime::new(config_a));
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
    let pos_config =
        signed_pos_config_with_signer_seeds(validators, &[("node-a", 55), ("node-b", 66)]);

    let build_config_a = || {
        NodeConfig::new("node-a", "world-restart", NodeRole::Sequencer)
            .expect("config a")
            .with_tick_interval(Duration::from_millis(10))
            .expect("tick a")
            .with_pos_config(pos_config.clone())
            .expect("pos config a")
            .with_auto_attest_all_validators(true)
            .with_gossip_optional(addr_a, vec![addr_b])
            .with_replication(signed_replication_config(dir_a.clone(), 55))
    };
    let build_config_b = || {
        NodeConfig::new("node-b", "world-restart", NodeRole::Observer)
            .expect("config b")
            .with_tick_interval(Duration::from_millis(10))
            .expect("tick b")
            .with_pos_config(pos_config.clone())
            .expect("pos config b")
            .with_gossip_optional(addr_b, vec![addr_a])
            .with_replication(signed_replication_config(dir_b.clone(), 66))
    };

    let mut runtime_a = with_noop_execution_hook(NodeRuntime::new(build_config_a()));
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

    let mut runtime_a = with_noop_execution_hook(NodeRuntime::new(build_config_a()));
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

#[test]
fn runtime_network_replication_accepts_writer_failover_with_epoch_rotation() {
    let dir_a = temp_dir("failover-a");
    let dir_b = temp_dir("failover-b");
    let dir_c = temp_dir("failover-c");
    let validators = vec![
        PosValidator {
            validator_id: "node-a".to_string(),
            stake: 34,
        },
        PosValidator {
            validator_id: "node-b".to_string(),
            stake: 33,
        },
        PosValidator {
            validator_id: "node-c".to_string(),
            stake: 33,
        },
    ];
    let pos_config = signed_pos_config_with_signer_seeds(
        validators,
        &[("node-a", 91), ("node-b", 92), ("node-c", 93)],
    );
    let network: Arc<
        dyn agent_world_proto::distributed_net::DistributedNetwork<WorldError> + Send + Sync,
    > = Arc::new(TestInMemoryNetwork::default());

    let build_observer = || {
        NodeConfig::new("node-b", "world-failover-repl", NodeRole::Observer)
            .expect("observer config")
            .with_tick_interval(Duration::from_millis(10))
            .expect("observer tick")
            .with_pos_config(pos_config.clone())
            .expect("observer pos config")
            .with_replication(signed_replication_config(dir_b.clone(), 92))
    };
    let build_sequencer_a = || {
        NodeConfig::new("node-a", "world-failover-repl", NodeRole::Sequencer)
            .expect("sequencer a config")
            .with_tick_interval(Duration::from_millis(10))
            .expect("sequencer a tick")
            .with_pos_config(pos_config.clone())
            .expect("sequencer a pos config")
            .with_auto_attest_all_validators(true)
            .with_replication(signed_replication_config(dir_a.clone(), 91))
    };
    let build_sequencer_c = || {
        NodeConfig::new("node-c", "world-failover-repl", NodeRole::Sequencer)
            .expect("sequencer c config")
            .with_tick_interval(Duration::from_millis(10))
            .expect("sequencer c tick")
            .with_pos_config(pos_config.clone())
            .expect("sequencer c pos config")
            .with_auto_attest_all_validators(true)
            .with_replication(signed_replication_config(dir_c.clone(), 93))
    };

    let mut runtime_a = with_noop_execution_hook(NodeRuntime::new(build_sequencer_a()))
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));
    let mut runtime_b = NodeRuntime::new(build_observer())
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));
    runtime_a.start().expect("start a");
    runtime_b.start().expect("start b with a");
    thread::sleep(Duration::from_millis(220));
    runtime_a.stop().expect("stop a");
    runtime_b.stop().expect("stop b after a");

    let guard_path = dir_b.join("replication_guard.json");
    let guard_before: SingleWriterReplicationGuard =
        serde_json::from_slice(&fs::read(&guard_path).expect("read guard before"))
            .expect("parse guard before");
    assert!(guard_before.last_sequence >= 1);
    assert!(guard_before.writer_epoch >= 1);
    let writer_before = guard_before.writer_id.clone();

    let mut runtime_c = with_noop_execution_hook(NodeRuntime::new(build_sequencer_c()))
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));
    let mut runtime_b = NodeRuntime::new(build_observer())
        .with_replication_network(NodeReplicationNetworkHandle::new(Arc::clone(&network)));
    runtime_c.start().expect("start c");
    runtime_b.start().expect("start b with c");
    thread::sleep(Duration::from_millis(260));
    runtime_c.stop().expect("stop c");
    runtime_b.stop().expect("stop b after c");

    let guard_after: SingleWriterReplicationGuard =
        serde_json::from_slice(&fs::read(&guard_path).expect("read guard after"))
            .expect("parse guard after");
    assert!(guard_after.last_sequence >= 1);
    assert!(guard_after.writer_epoch > guard_before.writer_epoch);
    assert_ne!(guard_after.writer_id, writer_before);

    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);
    let _ = fs::remove_dir_all(&dir_c);
}
