use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

pub use agent_world_consensus::node_consensus_action::NodeConsensusAction;
use agent_world_consensus::node_consensus_action::{
    compute_consensus_action_root as core_compute_consensus_action_root,
    drain_ordered_consensus_actions as core_drain_ordered_consensus_actions,
    merge_pending_consensus_actions as core_merge_pending_consensus_actions,
    validate_consensus_action_root as core_validate_consensus_action_root,
};
use agent_world_consensus::node_consensus_error::NodeConsensusError;
use agent_world_consensus::node_consensus_signature::{
    sign_attestation_message as core_sign_attestation_message,
    sign_commit_message as core_sign_commit_message,
    sign_proposal_message as core_sign_proposal_message,
    verify_attestation_message_signature as core_verify_attestation_message_signature,
    verify_commit_message_signature as core_verify_commit_message_signature,
    verify_proposal_message_signature as core_verify_proposal_message_signature,
    NodeConsensusMessageSigner,
};
use agent_world_consensus::node_pos::{
    advance_pending_attestations as core_advance_pending_attestations,
    insert_attestation as core_insert_attestation, propose_next_head as core_propose_next_head,
    NodePosDecision, NodePosError, NodePosPendingProposal, NodePosStatusAdapter,
};
use agent_world_distfs::blake3_hex;
use agent_world_proto::distributed::DistributedErrorCode;
use agent_world_proto::world_error::WorldError as ProtoWorldError;
use serde::Deserialize;

mod error;
mod execution_hook;
mod gossip_udp;
#[cfg(not(target_arch = "wasm32"))]
mod libp2p_replication_network;
#[cfg(target_arch = "wasm32")]
mod libp2p_replication_network_wasm;
mod network_bridge;
mod node_runtime_core;
mod pos_engine_gossip;
mod pos_schedule;
mod pos_state_store;
mod pos_validation;
mod replication;
mod runtime_util;
mod types;

pub use error::NodeError;
pub use execution_hook::{
    NodeExecutionCommitContext, NodeExecutionCommitResult, NodeExecutionHook,
};
use gossip_udp::{
    GossipAttestationMessage, GossipCommitMessage, GossipEndpoint, GossipMessage,
    GossipProposalMessage,
};
#[cfg(not(target_arch = "wasm32"))]
pub use libp2p_replication_network::{Libp2pReplicationNetwork, Libp2pReplicationNetworkConfig};
#[cfg(target_arch = "wasm32")]
pub use libp2p_replication_network_wasm::{
    Libp2pReplicationNetwork, Libp2pReplicationNetworkConfig,
};
pub use network_bridge::NodeReplicationNetworkHandle;
pub use replication::NodeReplicationConfig;
pub use types::{
    NodeCommittedActionBatch, NodeConfig, NodeConsensusMode, NodeConsensusSnapshot,
    NodeGossipConfig, NodePeerCommittedHead, NodePosConfig, NodeRole, NodeSnapshot,
    PosConsensusStatus, PosValidator,
};

use network_bridge::{ConsensusNetworkEndpoint, ReplicationNetworkEndpoint};
use node_runtime_core::RuntimeState;
use pos_state_store::PosNodeStateStore;
use pos_validation::{normalize_consensus_public_key_hex, validated_pos_state};
use replication::{
    load_blob_from_root, load_commit_message_from_root, FetchBlobRequest, FetchBlobResponse,
    FetchCommitRequest, FetchCommitResponse, ReplicationRuntime, REPLICATION_FETCH_BLOB_PROTOCOL,
    REPLICATION_FETCH_COMMIT_PROTOCOL,
};
use runtime_util::{lock_state, now_unix_ms};

const STORAGE_GATE_NETWORK_SAMPLES_PER_CHECK: usize = 3;
const STORAGE_GATE_NETWORK_MIN_MATCHES_CAP: usize = 2;
const REPLICATION_GAP_SYNC_MAX_RETRIES_PER_HEIGHT: usize = 3;
const EXECUTION_BINDING_HISTORY_LIMIT: usize = 256;

fn required_network_blob_matches(sample_count: usize) -> usize {
    sample_count
        .min(STORAGE_GATE_NETWORK_MIN_MATCHES_CAP)
        .max(1)
}

impl NodePosStatusAdapter for PosConsensusStatus {
    fn pending() -> Self {
        PosConsensusStatus::Pending
    }

    fn committed() -> Self {
        PosConsensusStatus::Committed
    }

    fn rejected() -> Self {
        PosConsensusStatus::Rejected
    }
}

fn node_pos_error(err: NodePosError) -> NodeError {
    NodeError::Consensus { reason: err.reason }
}

fn node_consensus_error(err: NodeConsensusError) -> NodeError {
    NodeError::Consensus { reason: err.reason }
}

fn checked_consensus_successor(value: u64, field: &str, context: &str) -> Result<u64, NodeError> {
    value.checked_add(1).ok_or_else(|| NodeError::Consensus {
        reason: format!("{field} overflow while {context}: current={value}"),
    })
}

fn checked_replication_successor(value: u64, field: &str, context: &str) -> Result<u64, NodeError> {
    value.checked_add(1).ok_or_else(|| NodeError::Replication {
        reason: format!("{field} overflow while {context}: current={value}"),
    })
}

pub fn compute_consensus_action_root(actions: &[NodeConsensusAction]) -> Result<String, NodeError> {
    core_compute_consensus_action_root(actions).map_err(node_consensus_error)
}

fn merge_pending_consensus_actions(
    pending: &mut BTreeMap<u64, NodeConsensusAction>,
    incoming: Vec<NodeConsensusAction>,
) -> Result<(), NodeError> {
    core_merge_pending_consensus_actions(pending, incoming).map_err(node_consensus_error)
}

fn drain_ordered_consensus_actions(
    pending: &mut BTreeMap<u64, NodeConsensusAction>,
) -> Vec<NodeConsensusAction> {
    core_drain_ordered_consensus_actions(pending)
}

fn validate_consensus_action_root(
    action_root: &str,
    actions: &[NodeConsensusAction],
) -> Result<(), NodeError> {
    core_validate_consensus_action_root(action_root, actions).map_err(node_consensus_error)
}

fn sign_commit_message(
    message: &mut GossipCommitMessage,
    signer: &NodeConsensusMessageSigner,
) -> Result<(), NodeError> {
    core_sign_commit_message(message, signer).map_err(node_consensus_error)
}

fn sign_proposal_message(
    message: &mut GossipProposalMessage,
    signer: &NodeConsensusMessageSigner,
) -> Result<(), NodeError> {
    core_sign_proposal_message(message, signer).map_err(node_consensus_error)
}

fn sign_attestation_message(
    message: &mut GossipAttestationMessage,
    signer: &NodeConsensusMessageSigner,
) -> Result<(), NodeError> {
    core_sign_attestation_message(message, signer).map_err(node_consensus_error)
}

fn verify_commit_message_signature(
    message: &GossipCommitMessage,
    enforce: bool,
) -> Result<(), NodeError> {
    core_verify_commit_message_signature(message, enforce).map_err(node_consensus_error)
}

fn verify_proposal_message_signature(
    message: &GossipProposalMessage,
    enforce: bool,
) -> Result<(), NodeError> {
    core_verify_proposal_message_signature(message, enforce).map_err(node_consensus_error)
}

fn verify_attestation_message_signature(
    message: &GossipAttestationMessage,
    enforce: bool,
) -> Result<(), NodeError> {
    core_verify_attestation_message_signature(message, enforce).map_err(node_consensus_error)
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum GapSyncHeightOutcome {
    Synced {
        block_hash: String,
        committed_at_ms: i64,
    },
    NotFound,
}

pub struct NodeRuntime {
    config: NodeConfig,
    replication_network: Option<NodeReplicationNetworkHandle>,
    execution_hook: Option<std::sync::Arc<std::sync::Mutex<Box<dyn NodeExecutionHook>>>>,
    pending_consensus_actions: Arc<Mutex<Vec<NodeConsensusAction>>>,
    committed_action_batches: Arc<Mutex<Vec<NodeCommittedActionBatch>>>,
    running: Arc<AtomicBool>,
    state: Arc<Mutex<RuntimeState>>,
    stop_tx: Option<mpsc::Sender<()>>,
    worker: Option<JoinHandle<()>>,
}

impl NodeRuntime {
    pub fn start(&mut self) -> Result<(), NodeError> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Err(NodeError::AlreadyRunning {
                node_id: self.config.node_id.clone(),
            });
        }

        {
            let mut state = lock_state(&self.state);
            *state = RuntimeState::default();
        }
        {
            let mut committed = self
                .committed_action_batches
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            committed.clear();
        }

        let mut engine = match PosNodeEngine::new(&self.config) {
            Ok(engine) => engine,
            Err(err) => {
                self.running.store(false, Ordering::SeqCst);
                return Err(err);
            }
        };
        let effective_replication_config = self
            .config
            .replication
            .as_ref()
            .map(|config| {
                config.clone().with_default_remote_writer_allowlist(
                    self.config
                        .pos_config
                        .validator_signer_public_keys
                        .values()
                        .cloned(),
                )
            })
            .transpose()?;
        let pos_state_store = effective_replication_config
            .as_ref()
            .map(PosNodeStateStore::from_replication);
        if let Some(store) = pos_state_store.as_ref() {
            match store.load() {
                Ok(Some(snapshot)) => {
                    if let Err(err) = engine.restore_state_snapshot(snapshot) {
                        self.running.store(false, Ordering::SeqCst);
                        return Err(err);
                    }
                }
                Ok(None) => {}
                Err(err) => {
                    self.running.store(false, Ordering::SeqCst);
                    return Err(err);
                }
            }
        }
        let mut gossip = if let Some(config) = &self.config.gossip {
            match GossipEndpoint::bind(config) {
                Ok(endpoint) => Some(endpoint),
                Err(err) => {
                    self.running.store(false, Ordering::SeqCst);
                    return Err(err);
                }
            }
        } else {
            None
        };
        let mut replication = if let Some(config) = effective_replication_config.as_ref() {
            match ReplicationRuntime::new(config, &self.config.node_id) {
                Ok(runtime) => Some(runtime),
                Err(err) => {
                    self.running.store(false, Ordering::SeqCst);
                    return Err(err);
                }
            }
        } else {
            None
        };
        if let (Some(network), Some(replication_config)) = (
            &self.replication_network,
            effective_replication_config.as_ref(),
        ) {
            if let Err(err) = register_replication_fetch_handlers(
                network,
                replication_config,
                self.config.world_id.as_str(),
            ) {
                self.running.store(false, Ordering::SeqCst);
                return Err(err);
            }
        }
        let mut replication_network = if let Some(network) = &self.replication_network {
            let subscribe = !matches!(self.config.role, NodeRole::Sequencer);
            match ReplicationNetworkEndpoint::new(network, &self.config.world_id, subscribe) {
                Ok(endpoint) => Some(endpoint),
                Err(err) => {
                    self.running.store(false, Ordering::SeqCst);
                    return Err(err);
                }
            }
        } else {
            None
        };
        let mut consensus_network = if let Some(network) = &self.replication_network {
            match ConsensusNetworkEndpoint::new(network, &self.config.world_id, true) {
                Ok(endpoint) => Some(endpoint),
                Err(err) => {
                    self.running.store(false, Ordering::SeqCst);
                    return Err(err);
                }
            }
        } else {
            None
        };
        let tick_interval = self.config.tick_interval;
        let worker_name = format!("aw-node-{}", self.config.node_id);
        let running = Arc::clone(&self.running);
        let state = Arc::clone(&self.state);
        let execution_hook = self.execution_hook.clone();
        let pending_consensus_actions = Arc::clone(&self.pending_consensus_actions);
        let committed_action_batches = Arc::clone(&self.committed_action_batches);
        let node_id = self.config.node_id.clone();
        let world_id = self.config.world_id.clone();
        let (stop_tx, stop_rx) = mpsc::channel::<()>();

        let worker = thread::Builder::new()
            .name(worker_name)
            .spawn(move || {
                loop {
                    match stop_rx.recv_timeout(tick_interval) {
                        Ok(()) => break,
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            let now_ms = now_unix_ms();
                            {
                                let mut current = lock_state(&state);
                                current.tick_count = current.tick_count.saturating_add(1);
                                current.last_tick_unix_ms = Some(now_ms);
                            }

                            let tick_result = if let Some(execution_hook) = execution_hook.as_ref()
                            {
                                let queued_actions = {
                                    let mut pending = pending_consensus_actions
                                        .lock()
                                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                                    std::mem::take(&mut *pending)
                                };
                                match execution_hook.lock() {
                                    Ok(mut hook) => engine.tick(
                                        &node_id,
                                        &world_id,
                                        now_ms,
                                        gossip.as_mut(),
                                        replication.as_mut(),
                                        replication_network.as_mut(),
                                        consensus_network.as_mut(),
                                        queued_actions,
                                        Some(hook.as_mut()),
                                    ),
                                    Err(_) => Err(NodeError::Execution {
                                        reason: "execution hook lock poisoned".to_string(),
                                    }),
                                }
                            } else {
                                let queued_actions = {
                                    let mut pending = pending_consensus_actions
                                        .lock()
                                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                                    std::mem::take(&mut *pending)
                                };
                                engine.tick(
                                    &node_id,
                                    &world_id,
                                    now_ms,
                                    gossip.as_mut(),
                                    replication.as_mut(),
                                    replication_network.as_mut(),
                                    consensus_network.as_mut(),
                                    queued_actions,
                                    None,
                                )
                            };
                            let mut current = lock_state(&state);
                            match tick_result {
                                Ok(tick) => {
                                    current.consensus = tick.consensus_snapshot;
                                    current.last_error = None;
                                    if let Some(batch) = tick.committed_action_batch {
                                        let mut committed = committed_action_batches
                                            .lock()
                                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                                        committed.push(batch);
                                    }
                                    if let Some(store) = pos_state_store.as_ref() {
                                        if let Err(err) = store.save_engine_state(&engine) {
                                            current.last_error = Some(err.to_string());
                                        }
                                    }
                                }
                                Err(err) => {
                                    current.last_error = Some(err.to_string());
                                }
                            }
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                }
                running.store(false, Ordering::SeqCst);
            })
            .map_err(|err| {
                self.running.store(false, Ordering::SeqCst);
                NodeError::ThreadSpawnFailed {
                    reason: err.to_string(),
                }
            })?;

        self.stop_tx = Some(stop_tx);
        self.worker = Some(worker);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), NodeError> {
        if !self.running.load(Ordering::SeqCst) {
            return Err(NodeError::NotRunning {
                node_id: self.config.node_id.clone(),
            });
        }
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
        if let Some(worker) = self.worker.take() {
            worker.join().map_err(|_| NodeError::ThreadJoinFailed {
                node_id: self.config.node_id.clone(),
            })?;
        }
        self.running.store(false, Ordering::SeqCst);
        Ok(())
    }

    pub fn snapshot(&self) -> NodeSnapshot {
        let state = lock_state(&self.state);
        NodeSnapshot {
            node_id: self.config.node_id.clone(),
            player_id: self.config.player_id.clone(),
            world_id: self.config.world_id.clone(),
            role: self.config.role,
            running: self.running.load(Ordering::SeqCst),
            tick_count: state.tick_count,
            last_tick_unix_ms: state.last_tick_unix_ms,
            consensus: state.consensus.clone(),
            last_error: state.last_error.clone(),
        }
    }
}

impl Drop for NodeRuntime {
    fn drop(&mut self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        self.running.store(false, Ordering::SeqCst);
    }
}

fn register_replication_fetch_handlers(
    handle: &NodeReplicationNetworkHandle,
    replication: &NodeReplicationConfig,
    world_id: &str,
) -> Result<(), NodeError> {
    let network = handle.clone_network();

    let commit_root_dir = replication.root_dir.clone();
    let commit_world_id = world_id.to_string();
    network
        .register_handler(
            REPLICATION_FETCH_COMMIT_PROTOCOL,
            Box::new(move |payload| {
                let request =
                    serde_json::from_slice::<FetchCommitRequest>(payload).map_err(|err| {
                        network_bad_request(format!("decode fetch-commit request failed: {}", err))
                    })?;
                if request.world_id != commit_world_id {
                    return Err(network_bad_request(format!(
                        "fetch-commit world mismatch: expected={}, got={}",
                        commit_world_id, request.world_id
                    )));
                }
                let message = load_commit_message_from_root(
                    commit_root_dir.as_path(),
                    commit_world_id.as_str(),
                    request.height,
                )
                .map_err(network_internal_error)?;
                let response = FetchCommitResponse {
                    found: message.is_some(),
                    message,
                };
                serde_json::to_vec(&response).map_err(|err| {
                    network_internal_error(NodeError::Replication {
                        reason: format!("encode fetch-commit response failed: {}", err),
                    })
                })
            }),
        )
        .map_err(network_replication_error)?;

    let blob_root_dir = replication.root_dir.clone();
    network
        .register_handler(
            REPLICATION_FETCH_BLOB_PROTOCOL,
            Box::new(move |payload| {
                let request =
                    serde_json::from_slice::<FetchBlobRequest>(payload).map_err(|err| {
                        network_bad_request(format!("decode fetch-blob request failed: {}", err))
                    })?;
                let blob =
                    load_blob_from_root(blob_root_dir.as_path(), request.content_hash.as_str())
                        .map_err(network_internal_error)?;
                let response = FetchBlobResponse {
                    found: blob.is_some(),
                    blob,
                };
                serde_json::to_vec(&response).map_err(|err| {
                    network_internal_error(NodeError::Replication {
                        reason: format!("encode fetch-blob response failed: {}", err),
                    })
                })
            }),
        )
        .map_err(network_replication_error)
}

fn network_bad_request(message: impl Into<String>) -> ProtoWorldError {
    ProtoWorldError::NetworkRequestFailed {
        code: DistributedErrorCode::ErrBadRequest,
        message: message.into(),
        retryable: false,
    }
}

fn network_internal_error(err: NodeError) -> ProtoWorldError {
    ProtoWorldError::NetworkRequestFailed {
        code: DistributedErrorCode::ErrNotAvailable,
        message: err.to_string(),
        retryable: true,
    }
}

fn network_replication_error(err: ProtoWorldError) -> NodeError {
    NodeError::Replication {
        reason: format!("replication network error: {err:?}"),
    }
}

#[derive(Debug, Clone)]
struct PosNodeEngine {
    validators: BTreeMap<String, u64>,
    validator_players: BTreeMap<String, String>,
    validator_signers: BTreeMap<String, String>,
    total_stake: u64,
    required_stake: u64,
    epoch_length_slots: u64,
    local_validator_id: String,
    node_player_id: String,
    require_execution_on_commit: bool,
    next_height: u64,
    next_slot: u64,
    committed_height: u64,
    network_committed_height: u64,
    pending: Option<PendingProposal>,
    auto_attest_all_validators: bool,
    last_broadcast_proposal_height: u64,
    last_broadcast_local_attestation_height: u64,
    last_broadcast_committed_height: u64,
    replicate_local_commits: bool,
    require_peer_execution_hashes: bool,
    consensus_signer: Option<NodeConsensusMessageSigner>,
    enforce_consensus_signature: bool,
    peer_heads: BTreeMap<String, PeerCommittedHead>,
    last_committed_at_ms: Option<i64>,
    last_committed_block_hash: Option<String>,
    last_execution_height: u64,
    last_execution_block_hash: Option<String>,
    last_execution_state_root: Option<String>,
    execution_bindings: BTreeMap<u64, (String, String)>,
    pending_consensus_actions: BTreeMap<u64, NodeConsensusAction>,
}

type PendingProposal = NodePosPendingProposal<NodeConsensusAction, PosConsensusStatus>;
type PosDecision = NodePosDecision<NodeConsensusAction, PosConsensusStatus>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct PeerCommittedHead {
    height: u64,
    block_hash: String,
    committed_at_ms: i64,
    execution_block_hash: Option<String>,
    execution_state_root: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ReplicationCommitPayloadView {
    height: u64,
    block_hash: String,
    committed_at_ms: i64,
    #[serde(default)]
    execution_block_hash: Option<String>,
    #[serde(default)]
    execution_state_root: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ReplicationCommitPayload {
    world_id: String,
    node_id: String,
    height: u64,
    block_hash: String,
    action_root: String,
    actions: Vec<NodeConsensusAction>,
    committed_at_ms: i64,
    #[serde(default)]
    execution_block_hash: Option<String>,
    #[serde(default)]
    execution_state_root: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NodeEngineTickResult {
    consensus_snapshot: NodeConsensusSnapshot,
    committed_action_batch: Option<NodeCommittedActionBatch>,
}

impl PosNodeEngine {
    fn new(config: &NodeConfig) -> Result<Self, NodeError> {
        let (validators, validator_players, validator_signers, total_stake, required_stake) =
            validated_pos_state(&config.pos_config)?;
        let (consensus_signer, consensus_signer_public_key, enforce_consensus_signature) =
            if let Some(replication) = &config.replication {
                let signer_keypair = replication.consensus_signer()?;
                let signer_public_key = signer_keypair
                    .as_ref()
                    .map(|(_, public_key_hex)| public_key_hex.clone());
                let signer = signer_keypair
                    .map(|(signing_key, public_key_hex)| {
                        NodeConsensusMessageSigner::new(signing_key, public_key_hex)
                    })
                    .transpose()
                    .map_err(node_consensus_error)?;
                (
                    signer,
                    signer_public_key,
                    replication.enforce_consensus_signature(),
                )
            } else {
                (None, None, false)
            };
        if enforce_consensus_signature && validator_signers.len() != validators.len() {
            let missing_validator_signers = validators
                .keys()
                .filter(|validator_id| !validator_signers.contains_key(*validator_id))
                .cloned()
                .collect::<Vec<_>>();
            return Err(NodeError::InvalidConfig {
                reason: format!(
                    "consensus signature enforcement requires signer bindings for all validators; missing={}",
                    missing_validator_signers.join(",")
                ),
            });
        }
        if enforce_consensus_signature {
            if let Some(expected_public_key) = validator_signers.get(config.node_id.as_str()) {
                let Some(actual_public_key) = consensus_signer_public_key.as_deref() else {
                    return Err(NodeError::InvalidConfig {
                        reason: format!(
                            "consensus signer binding missing local signer keypair for validator {}",
                            config.node_id
                        ),
                    });
                };
                if actual_public_key != expected_public_key {
                    return Err(NodeError::InvalidConfig {
                        reason: format!(
                            "consensus signer binding mismatch for local validator {}: expected={} actual={}",
                            config.node_id, expected_public_key, actual_public_key
                        ),
                    });
                }
            }
        }
        if let Some(bound_player_id) = validator_players.get(config.node_id.as_str()) {
            if bound_player_id != &config.player_id {
                return Err(NodeError::InvalidConfig {
                    reason: format!(
                        "node_id {} is bound to validator player {}, but config player_id is {}",
                        config.node_id, bound_player_id, config.player_id
                    ),
                });
            }
        }
        Ok(Self {
            validators,
            validator_players,
            validator_signers,
            total_stake,
            required_stake,
            epoch_length_slots: config.pos_config.epoch_length_slots,
            local_validator_id: config.node_id.clone(),
            node_player_id: config.player_id.clone(),
            require_execution_on_commit: config.require_execution_on_commit,
            next_height: 1,
            next_slot: 0,
            committed_height: 0,
            network_committed_height: 0,
            pending: None,
            auto_attest_all_validators: config.auto_attest_all_validators,
            last_broadcast_proposal_height: 0,
            last_broadcast_local_attestation_height: 0,
            last_broadcast_committed_height: 0,
            replicate_local_commits: matches!(config.role, NodeRole::Sequencer)
                && config.replication.is_some(),
            require_peer_execution_hashes: config.require_peer_execution_hashes,
            consensus_signer,
            enforce_consensus_signature,
            peer_heads: BTreeMap::new(),
            last_committed_at_ms: None,
            last_committed_block_hash: None,
            last_execution_height: 0,
            last_execution_block_hash: None,
            last_execution_state_root: None,
            execution_bindings: BTreeMap::new(),
            pending_consensus_actions: BTreeMap::new(),
        })
    }

    fn tick(
        &mut self,
        node_id: &str,
        world_id: &str,
        now_ms: i64,
        gossip: Option<&mut GossipEndpoint>,
        mut replication: Option<&mut ReplicationRuntime>,
        replication_network: Option<&mut ReplicationNetworkEndpoint>,
        consensus_network: Option<&mut ConsensusNetworkEndpoint>,
        queued_actions: Vec<NodeConsensusAction>,
        execution_hook: Option<&mut dyn NodeExecutionHook>,
    ) -> Result<NodeEngineTickResult, NodeError> {
        merge_pending_consensus_actions(&mut self.pending_consensus_actions, queued_actions)?;

        if let Some(endpoint) = gossip.as_ref() {
            self.ingest_peer_messages(endpoint, node_id, world_id, replication.as_deref_mut())?;
        }
        if let Some(endpoint) = consensus_network.as_ref() {
            self.ingest_consensus_network_messages(endpoint, world_id)?;
        }
        if let Some(endpoint) = replication_network.as_ref() {
            self.ingest_network_replications(
                endpoint,
                node_id,
                world_id,
                replication.as_deref_mut(),
            )?;
            self.sync_missing_replication_commits(
                endpoint,
                node_id,
                world_id,
                replication.as_deref_mut(),
            )?;
        }

        let mut decision = if self.pending.is_some() {
            self.advance_pending_attestations(now_ms)?
        } else {
            self.propose_next_head(node_id, world_id, now_ms)?
        };

        if matches!(decision.status, PosConsensusStatus::Pending) {
            decision = self.advance_pending_attestations(now_ms)?;
        }

        if let Some(endpoint) = consensus_network.as_ref() {
            self.broadcast_local_proposal_network(endpoint, node_id, world_id, now_ms)?;
            self.broadcast_local_attestation_network(endpoint, node_id, world_id, now_ms)?;
        } else if let Some(endpoint) = gossip.as_ref() {
            self.broadcast_local_proposal(endpoint, node_id, world_id, now_ms)?;
            self.broadcast_local_attestation(endpoint, node_id, world_id, now_ms)?;
        }

        let prev_committed_height = self.committed_height;
        self.apply_committed_execution(node_id, world_id, now_ms, &decision, execution_hook)?;
        self.apply_decision(&decision)?;
        if matches!(decision.status, PosConsensusStatus::Committed)
            && decision.height > prev_committed_height
        {
            self.last_committed_at_ms = Some(now_ms);
        }
        if let Some(endpoint) = consensus_network.as_ref() {
            self.broadcast_local_commit_network(endpoint, node_id, world_id, now_ms, &decision)?;
        } else if let Some(endpoint) = gossip.as_ref() {
            self.broadcast_local_commit(endpoint, node_id, world_id, now_ms, &decision)?;
        }
        self.broadcast_local_replication(
            gossip.as_deref(),
            replication_network.as_deref(),
            node_id,
            world_id,
            now_ms,
            &decision,
            replication.as_deref_mut(),
        )?;
        if let Some(endpoint) = gossip.as_ref() {
            self.ingest_peer_messages(endpoint, node_id, world_id, replication.as_deref_mut())?;
        }
        if let Some(endpoint) = consensus_network.as_ref() {
            self.ingest_consensus_network_messages(endpoint, world_id)?;
        }
        if let Some(endpoint) = replication_network.as_ref() {
            self.ingest_network_replications(
                endpoint,
                node_id,
                world_id,
                replication.as_deref_mut(),
            )?;
        }
        let committed_action_batch = if matches!(decision.status, PosConsensusStatus::Committed)
            && !decision.committed_actions.is_empty()
            && decision.height > prev_committed_height
        {
            Some(NodeCommittedActionBatch {
                height: decision.height,
                slot: decision.slot,
                epoch: decision.epoch,
                block_hash: decision.block_hash.clone(),
                action_root: decision.action_root.clone(),
                committed_at_unix_ms: now_ms,
                actions: decision.committed_actions.clone(),
            })
        } else {
            None
        };

        Ok(NodeEngineTickResult {
            consensus_snapshot: self.snapshot_from_decision(&decision),
            committed_action_batch,
        })
    }

    fn propose_next_head(
        &mut self,
        node_id: &str,
        world_id: &str,
        now_ms: i64,
    ) -> Result<PosDecision, NodeError> {
        let slot = self.next_slot;
        let epoch = self.slot_epoch(slot);
        let committed_actions =
            drain_ordered_consensus_actions(&mut self.pending_consensus_actions);
        let action_root = compute_consensus_action_root(committed_actions.as_slice())?;
        let proposer_id = self
            .expected_proposer(slot)
            .ok_or_else(|| NodeError::Consensus {
                reason: "no proposer available".to_string(),
            })?;
        let parent_block_hash = self
            .last_committed_block_hash
            .as_deref()
            .unwrap_or("genesis");
        let block_hash = self.compute_block_hash(
            world_id,
            self.next_height,
            slot,
            epoch,
            proposer_id.as_str(),
            parent_block_hash,
            action_root.as_str(),
        )?;

        core_propose_next_head(
            &self.validators,
            self.total_stake,
            self.required_stake,
            self.epoch_length_slots,
            &mut self.next_height,
            &mut self.next_slot,
            &mut self.pending,
            proposer_id,
            block_hash,
            action_root,
            committed_actions,
            node_id,
            now_ms,
        )
        .map_err(node_pos_error)
    }

    fn advance_pending_attestations(&mut self, now_ms: i64) -> Result<PosDecision, NodeError> {
        core_advance_pending_attestations(
            &self.validators,
            self.total_stake,
            self.required_stake,
            self.local_validator_id.as_str(),
            self.auto_attest_all_validators,
            &mut self.pending,
            now_ms,
        )
        .map_err(node_pos_error)
    }

    fn insert_attestation(
        &self,
        proposal: &mut PendingProposal,
        validator_id: &str,
        approve: bool,
        voted_at_ms: i64,
        source_epoch: u64,
        target_epoch: u64,
        reason: Option<String>,
    ) -> Result<(), NodeError> {
        core_insert_attestation(
            &self.validators,
            self.total_stake,
            self.required_stake,
            proposal,
            validator_id,
            approve,
            voted_at_ms,
            source_epoch,
            target_epoch,
            reason,
        )
        .map_err(node_pos_error)
    }

    fn apply_decision(&mut self, decision: &PosDecision) -> Result<(), NodeError> {
        match decision.status {
            PosConsensusStatus::Pending => {}
            PosConsensusStatus::Committed => {
                let next_height = checked_consensus_successor(
                    decision.height,
                    "decision.height",
                    "applying committed decision",
                )?;
                self.committed_height = decision.height;
                self.network_committed_height = self.network_committed_height.max(decision.height);
                self.last_committed_block_hash = Some(decision.block_hash.clone());
                self.next_height = next_height;
                self.pending = None;
            }
            PosConsensusStatus::Rejected => {
                let next_height = checked_consensus_successor(
                    decision.height,
                    "decision.height",
                    "applying rejected decision",
                )?;
                let _ = merge_pending_consensus_actions(
                    &mut self.pending_consensus_actions,
                    decision.committed_actions.clone(),
                );
                self.next_height = next_height;
                self.pending = None;
            }
        }
        Ok(())
    }

    fn apply_committed_execution(
        &mut self,
        node_id: &str,
        world_id: &str,
        now_ms: i64,
        decision: &PosDecision,
        execution_hook: Option<&mut dyn NodeExecutionHook>,
    ) -> Result<(), NodeError> {
        if !matches!(decision.status, PosConsensusStatus::Committed) {
            return Ok(());
        }
        if decision.height <= self.last_execution_height {
            return Ok(());
        }
        let Some(execution_hook) = execution_hook else {
            if self.require_execution_on_commit {
                return Err(NodeError::Execution {
                    reason: format!(
                        "execution hook is required before committing height {}",
                        decision.height
                    ),
                });
            }
            return Ok(());
        };

        let result = execution_hook
            .on_commit(NodeExecutionCommitContext {
                world_id: world_id.to_string(),
                node_id: node_id.to_string(),
                height: decision.height,
                slot: decision.slot,
                epoch: decision.epoch,
                node_block_hash: decision.block_hash.clone(),
                action_root: decision.action_root.clone(),
                committed_actions: decision.committed_actions.clone(),
                committed_at_unix_ms: now_ms,
            })
            .map_err(|reason| NodeError::Execution { reason })?;

        if result.execution_height != decision.height {
            return Err(NodeError::Execution {
                reason: format!(
                    "execution hook returned mismatched height: expected {}, got {}",
                    decision.height, result.execution_height
                ),
            });
        }
        if result.execution_block_hash.trim().is_empty() {
            return Err(NodeError::Execution {
                reason: "execution hook returned empty execution_block_hash".to_string(),
            });
        }
        if result.execution_state_root.trim().is_empty() {
            return Err(NodeError::Execution {
                reason: "execution hook returned empty execution_state_root".to_string(),
            });
        }

        self.last_execution_height = result.execution_height;
        self.last_execution_block_hash = Some(result.execution_block_hash);
        self.last_execution_state_root = Some(result.execution_state_root);
        self.remember_execution_binding_for_height(decision.height);
        Ok(())
    }

    fn snapshot_from_decision(&self, decision: &PosDecision) -> NodeConsensusSnapshot {
        let peer_heads = self
            .peer_heads
            .iter()
            .map(|(node_id, head)| NodePeerCommittedHead {
                node_id: node_id.clone(),
                height: head.height,
                block_hash: head.block_hash.clone(),
                committed_at_ms: head.committed_at_ms,
                execution_block_hash: head.execution_block_hash.clone(),
                execution_state_root: head.execution_state_root.clone(),
            })
            .collect::<Vec<_>>();
        NodeConsensusSnapshot {
            mode: NodeConsensusMode::Pos,
            slot: self.next_slot,
            epoch: self.slot_epoch(self.next_slot),
            latest_height: decision.height,
            committed_height: self.committed_height,
            last_committed_at_ms: self.last_committed_at_ms,
            network_committed_height: self.network_committed_height.max(self.committed_height),
            known_peer_heads: self.peer_heads.len(),
            peer_heads,
            last_status: Some(decision.status),
            last_block_hash: Some(decision.block_hash.clone()),
            last_execution_height: self.last_execution_height,
            last_execution_block_hash: self.last_execution_block_hash.clone(),
            last_execution_state_root: self.last_execution_state_root.clone(),
        }
    }

    fn commit_execution_binding_for_height(
        &self,
        committed_height: u64,
    ) -> Result<(Option<&str>, Option<&str>), NodeError> {
        let (execution_block_hash, execution_state_root) = self
            .execution_binding_for_height(committed_height)
            .map(|(block_hash, state_root)| (Some(block_hash), Some(state_root)))
            .unwrap_or((None, None));
        if execution_block_hash.is_some() != execution_state_root.is_some() {
            return Err(NodeError::Consensus {
                reason:
                    "execution commit binding requires both execution_block_hash and execution_state_root"
                        .to_string(),
            });
        }
        Ok((execution_block_hash, execution_state_root))
    }

    fn execution_binding_for_height(&self, height: u64) -> Option<(&str, &str)> {
        if let Some((block_hash, state_root)) = self.execution_bindings.get(&height) {
            return Some((block_hash.as_str(), state_root.as_str()));
        }
        if self.last_execution_height != height {
            return None;
        }
        match (
            self.last_execution_block_hash.as_deref(),
            self.last_execution_state_root.as_deref(),
        ) {
            (Some(block_hash), Some(state_root)) => Some((block_hash, state_root)),
            _ => None,
        }
    }

    fn remember_execution_binding_for_height(&mut self, height: u64) {
        let (Some(block_hash), Some(state_root)) = (
            self.last_execution_block_hash.as_ref(),
            self.last_execution_state_root.as_ref(),
        ) else {
            return;
        };
        self.execution_bindings
            .insert(height, (block_hash.clone(), state_root.clone()));
        while self.execution_bindings.len() > EXECUTION_BINDING_HISTORY_LIMIT {
            let Some(first_height) = self.execution_bindings.keys().next().copied() else {
                break;
            };
            self.execution_bindings.remove(&first_height);
        }
    }

    fn validate_peer_commit_execution_binding(
        &self,
        height: u64,
        execution_block_hash: Option<&str>,
        execution_state_root: Option<&str>,
    ) -> Result<(), NodeError> {
        if execution_block_hash.is_some() != execution_state_root.is_some() {
            return Err(NodeError::Consensus {
                reason: format!(
                    "peer commit execution binding malformed at height {}: block/state pair mismatch",
                    height
                ),
            });
        }
        if self.require_peer_execution_hashes
            && (execution_block_hash.is_none() || execution_state_root.is_none())
        {
            return Err(NodeError::Consensus {
                reason: format!(
                    "peer commit missing required execution hashes at height {}",
                    height
                ),
            });
        }
        let Some((local_block_hash, local_state_root)) = self.execution_binding_for_height(height)
        else {
            return Ok(());
        };
        let (Some(peer_block_hash), Some(peer_state_root)) =
            (execution_block_hash, execution_state_root)
        else {
            return Err(NodeError::Consensus {
                reason: format!(
                    "peer commit missing execution hashes at locally executed height {}",
                    height
                ),
            });
        };
        if local_block_hash != peer_block_hash || local_state_root != peer_state_root {
            return Err(NodeError::Consensus {
                reason: format!(
                    "peer commit execution mismatch at height {}: local_block={} peer_block={} local_state={} peer_state={}",
                    height, local_block_hash, peer_block_hash, local_state_root, peer_state_root
                ),
            });
        }
        Ok(())
    }

    fn broadcast_local_replication(
        &mut self,
        gossip_endpoint: Option<&GossipEndpoint>,
        network_endpoint: Option<&ReplicationNetworkEndpoint>,
        node_id: &str,
        world_id: &str,
        now_ms: i64,
        decision: &PosDecision,
        replication: Option<&mut ReplicationRuntime>,
    ) -> Result<(), NodeError> {
        if !self.replicate_local_commits {
            return Ok(());
        }
        let Some(replication) = replication else {
            return Ok(());
        };
        self.enforce_storage_challenge_gate(
            replication,
            network_endpoint,
            node_id,
            world_id,
            now_ms,
        )?;
        let (execution_block_hash, execution_state_root) =
            self.commit_execution_binding_for_height(decision.height)?;
        if let Some(message) = replication.build_local_commit_message(
            node_id,
            world_id,
            now_ms,
            decision,
            execution_block_hash,
            execution_state_root,
        )? {
            if let Some(endpoint) = network_endpoint {
                endpoint.publish_replication(&message)?;
            } else if let Some(endpoint) = gossip_endpoint {
                endpoint.broadcast_replication(&message)?;
            }
        }
        Ok(())
    }

    fn enforce_storage_challenge_gate(
        &self,
        replication: &ReplicationRuntime,
        network_endpoint: Option<&ReplicationNetworkEndpoint>,
        node_id: &str,
        world_id: &str,
        now_ms: i64,
    ) -> Result<(), NodeError> {
        let report = replication.probe_storage_challenges(world_id, node_id, now_ms)?;
        if report.failed_checks > 0 {
            return Err(NodeError::Consensus {
                reason: format!(
                    "storage challenge gate failed: total_checks={} failed_checks={} reasons={:?}",
                    report.total_checks, report.failed_checks, report.failure_reasons
                ),
            });
        }

        let Some(endpoint) = network_endpoint else {
            return Ok(());
        };
        let content_hashes = replication
            .recent_replicated_content_hashes(world_id, STORAGE_GATE_NETWORK_SAMPLES_PER_CHECK)?;
        if content_hashes.is_empty() {
            return Ok(());
        }

        let mut successful_matches = 0usize;
        let mut attempted_samples = 0usize;
        let mut failure_reasons = Vec::new();
        for content_hash in content_hashes {
            attempted_samples = attempted_samples.saturating_add(1);

            let local_blob = match replication.load_blob_by_hash(content_hash.as_str())? {
                Some(blob) => blob,
                None => {
                    failure_reasons.push(format!(
                        "storage challenge gate local blob missing for hash {}",
                        content_hash
                    ));
                    continue;
                }
            };
            let response = match endpoint.request_json::<FetchBlobRequest, FetchBlobResponse>(
                REPLICATION_FETCH_BLOB_PROTOCOL,
                &FetchBlobRequest {
                    content_hash: content_hash.clone(),
                },
            ) {
                Ok(response) => response,
                Err(err) => {
                    failure_reasons.push(format!(
                        "storage challenge gate network request failed for hash {}: {:?}",
                        content_hash, err
                    ));
                    continue;
                }
            };
            if !response.found {
                failure_reasons.push(format!(
                    "storage challenge gate network blob not found for hash {}",
                    content_hash
                ));
                continue;
            }
            let Some(network_blob) = response.blob else {
                failure_reasons.push(format!(
                    "storage challenge gate network blob payload missing for hash {}",
                    content_hash
                ));
                continue;
            };
            if blake3_hex(network_blob.as_slice()) != content_hash {
                failure_reasons.push(format!(
                    "storage challenge gate network blob hash mismatch for hash {}",
                    content_hash
                ));
                continue;
            }
            if network_blob != local_blob {
                failure_reasons.push(format!(
                    "storage challenge gate network blob bytes mismatch for hash {}",
                    content_hash
                ));
                continue;
            }
            successful_matches = successful_matches.saturating_add(1);
        }

        let required_matches = required_network_blob_matches(attempted_samples);
        if successful_matches < required_matches {
            return Err(NodeError::Consensus {
                reason: format!(
                    "storage challenge gate network threshold unmet: samples={} required_matches={} successful_matches={} reasons={:?}",
                    attempted_samples,
                    required_matches,
                    successful_matches,
                    failure_reasons
                ),
            });
        }
        Ok(())
    }

    fn ingest_network_replications(
        &mut self,
        endpoint: &ReplicationNetworkEndpoint,
        node_id: &str,
        world_id: &str,
        mut replication: Option<&mut ReplicationRuntime>,
    ) -> Result<(), NodeError> {
        let Some(replication_runtime) = replication.as_deref_mut() else {
            return Ok(());
        };
        let messages = endpoint.drain_replications()?;
        let mut rejected = Vec::new();
        for message in messages {
            let committed_successor = checked_replication_successor(
                self.committed_height,
                "committed_height",
                "ingesting replication message",
            )?;
            let payload_view = parse_replication_commit_payload_view(message.payload.as_slice());
            match replication_runtime
                .validate_remote_message_for_observe(node_id, world_id, &message)
            {
                Ok(true) => {}
                Ok(false) => continue,
                Err(err) => {
                    rejected.push(format!(
                        "node_id={} world_id={} err={}",
                        message.node_id, message.world_id, err
                    ));
                    continue;
                }
            }
            if let Some(payload) = payload_view.as_ref() {
                if self
                    .validate_peer_commit_execution_binding(
                        payload.height,
                        payload.execution_block_hash.as_deref(),
                        payload.execution_state_root.as_deref(),
                    )
                    .is_err()
                {
                    rejected.push(format!(
                        "node_id={} world_id={} err=peer execution hash validation failed for height {}",
                        message.node_id, message.world_id, payload.height
                    ));
                    continue;
                }
                self.observe_network_replication_commit(message.node_id.as_str(), payload);
            }
            let should_apply = payload_view
                .as_ref()
                .map(|payload| payload.height <= committed_successor)
                .unwrap_or(true);
            if !should_apply {
                continue;
            }
            match replication_runtime.apply_remote_message(node_id, world_id, &message) {
                Ok(()) => {
                    if let Some(payload) = payload_view {
                        if payload.height == committed_successor
                            && replication_runtime
                                .load_commit_message_by_height(world_id, payload.height)?
                                .is_some()
                        {
                            self.record_synced_replication_height(
                                payload.height,
                                payload.block_hash,
                                payload.committed_at_ms,
                            )?;
                        }
                    }
                }
                Err(err) => rejected.push(format!(
                    "node_id={} world_id={} err={}",
                    message.node_id, message.world_id, err
                )),
            }
        }
        if !rejected.is_empty() {
            let rejected_count = rejected.len();
            let sample = rejected.into_iter().take(3).collect::<Vec<_>>();
            return Err(NodeError::Replication {
                reason: format!(
                    "replication ingest rejected {rejected_count} message(s); sample={sample:?}"
                ),
            });
        }
        Ok(())
    }

    fn observe_network_replication_commit(
        &mut self,
        peer_node_id: &str,
        payload: &ReplicationCommitPayloadView,
    ) {
        if payload.height == 0 {
            return;
        }
        self.network_committed_height = self.network_committed_height.max(payload.height);
        self.peer_heads.insert(
            peer_node_id.to_string(),
            PeerCommittedHead {
                height: payload.height,
                block_hash: payload.block_hash.clone(),
                committed_at_ms: payload.committed_at_ms,
                execution_block_hash: payload.execution_block_hash.clone(),
                execution_state_root: payload.execution_state_root.clone(),
            },
        );
    }

    fn sync_missing_replication_commits(
        &mut self,
        endpoint: &ReplicationNetworkEndpoint,
        node_id: &str,
        world_id: &str,
        mut replication: Option<&mut ReplicationRuntime>,
    ) -> Result<(), NodeError> {
        let Some(replication_runtime) = replication.as_deref_mut() else {
            return Ok(());
        };
        if self.network_committed_height <= self.committed_height {
            return Ok(());
        }

        let mut next_height = checked_replication_successor(
            self.committed_height,
            "committed_height",
            "starting replication gap sync",
        )?;
        while next_height <= self.network_committed_height {
            let mut synced_commit: Option<(String, i64)> = None;
            let mut not_found = false;
            let mut last_error = None;
            for attempt in 1..=REPLICATION_GAP_SYNC_MAX_RETRIES_PER_HEIGHT {
                match self.sync_replication_height_once(
                    endpoint,
                    node_id,
                    world_id,
                    replication_runtime,
                    next_height,
                ) {
                    Ok(GapSyncHeightOutcome::Synced {
                        block_hash,
                        committed_at_ms,
                    }) => {
                        synced_commit = Some((block_hash, committed_at_ms));
                        break;
                    }
                    Ok(GapSyncHeightOutcome::NotFound) => {
                        not_found = true;
                        break;
                    }
                    Err(err) => {
                        last_error = Some(format!(
                            "attempt {attempt}/{} failed: {}",
                            REPLICATION_GAP_SYNC_MAX_RETRIES_PER_HEIGHT, err
                        ));
                    }
                }
            }
            if let Some((block_hash, committed_at_ms)) = synced_commit {
                self.record_synced_replication_height(next_height, block_hash, committed_at_ms)?;
                next_height = checked_replication_successor(
                    next_height,
                    "next_height",
                    "advancing replication gap sync cursor",
                )?;
                continue;
            }
            if not_found {
                break;
            }
            return Err(NodeError::Replication {
                reason: format!(
                    "gap sync height {} failed after {} attempts: {}",
                    next_height,
                    REPLICATION_GAP_SYNC_MAX_RETRIES_PER_HEIGHT,
                    last_error.unwrap_or_else(|| "unknown error".to_string())
                ),
            });
        }
        Ok(())
    }

    fn sync_replication_height_once(
        &self,
        endpoint: &ReplicationNetworkEndpoint,
        node_id: &str,
        world_id: &str,
        replication_runtime: &mut ReplicationRuntime,
        height: u64,
    ) -> Result<GapSyncHeightOutcome, NodeError> {
        let request = FetchCommitRequest {
            world_id: world_id.to_string(),
            height,
        };
        let response = endpoint.request_json::<FetchCommitRequest, FetchCommitResponse>(
            REPLICATION_FETCH_COMMIT_PROTOCOL,
            &request,
        )?;
        if !response.found {
            return Ok(GapSyncHeightOutcome::NotFound);
        }
        let mut message = response.message.ok_or_else(|| NodeError::Replication {
            reason: format!(
                "gap sync height {} commit response missing payload (found=true)",
                height
            ),
        })?;
        if message.world_id != world_id || message.record.world_id != world_id {
            return Err(NodeError::Replication {
                reason: format!(
                    "gap sync height {} world mismatch: expected={} actual_message={} actual_record={}",
                    height, world_id, message.world_id, message.record.world_id
                ),
            });
        }

        let blob_request = FetchBlobRequest {
            content_hash: message.record.content_hash.clone(),
        };
        let blob_response = endpoint.request_json::<FetchBlobRequest, FetchBlobResponse>(
            REPLICATION_FETCH_BLOB_PROTOCOL,
            &blob_request,
        )?;
        if !blob_response.found {
            return Err(NodeError::Replication {
                reason: format!(
                    "gap sync height {} blob not found for hash {}",
                    height, message.record.content_hash
                ),
            });
        }
        let blob = blob_response.blob.ok_or_else(|| NodeError::Replication {
            reason: format!(
                "gap sync height {} blob payload missing for hash {}",
                height, message.record.content_hash
            ),
        })?;
        message.payload = blob;
        let payload =
            parse_replication_commit_payload(message.payload.as_slice()).ok_or_else(|| {
                NodeError::Replication {
                    reason: format!("gap sync height {} payload decode failed", height),
                }
            })?;
        if payload.world_id != world_id {
            return Err(NodeError::Replication {
                reason: format!(
                    "gap sync height {} payload world mismatch expected={} actual={}",
                    height, world_id, payload.world_id
                ),
            });
        }
        if payload.node_id != message.node_id {
            return Err(NodeError::Replication {
                reason: format!(
                    "gap sync height {} payload node mismatch expected={} actual={}",
                    height, message.node_id, payload.node_id
                ),
            });
        }
        if payload.height != height {
            return Err(NodeError::Replication {
                reason: format!(
                    "gap sync height {} payload mismatch actual={}",
                    height, payload.height
                ),
            });
        }
        if payload.block_hash.trim().is_empty() {
            return Err(NodeError::Replication {
                reason: format!("gap sync height {} payload block_hash is empty", height),
            });
        }
        validate_consensus_action_root(payload.action_root.as_str(), payload.actions.as_slice())
            .map_err(|err| NodeError::Replication {
                reason: format!(
                    "gap sync height {} action_root validation failed: {:?}",
                    height, err
                ),
            })?;
        self.validate_peer_commit_execution_binding(
            payload.height,
            payload.execution_block_hash.as_deref(),
            payload.execution_state_root.as_deref(),
        )
        .map_err(|err| NodeError::Replication {
            reason: format!(
                "gap sync height {} execution hash validation failed: {}",
                height, err
            ),
        })?;
        replication_runtime.apply_remote_message(node_id, world_id, &message)?;
        let persisted = replication_runtime.load_commit_message_by_height(world_id, height)?;
        if persisted
            .as_ref()
            .map(|entry| entry.record.content_hash.as_str())
            != Some(message.record.content_hash.as_str())
        {
            return Err(NodeError::Replication {
                reason: format!(
                    "gap sync height {} persisted commit hash mismatch expected={}",
                    height, message.record.content_hash
                ),
            });
        }
        Ok(GapSyncHeightOutcome::Synced {
            block_hash: payload.block_hash.clone(),
            committed_at_ms: payload.committed_at_ms,
        })
    }

    fn record_synced_replication_height(
        &mut self,
        height: u64,
        block_hash: String,
        committed_at_ms: i64,
    ) -> Result<(), NodeError> {
        if height <= self.committed_height {
            return Ok(());
        }
        let next_synced_height =
            checked_replication_successor(height, "height", "recording synced replication height")?;
        self.committed_height = height;
        self.last_committed_at_ms = Some(committed_at_ms);
        self.next_height = self.next_height.max(next_synced_height);
        self.last_committed_block_hash = Some(block_hash);
        self.pending = None;
        Ok(())
    }

    fn ingest_proposal_message(
        &mut self,
        world_id: &str,
        message: &GossipProposalMessage,
    ) -> Result<(), NodeError> {
        if message.version != 1 || message.world_id != world_id {
            return Ok(());
        }
        if message.node_id != message.proposer_id {
            return Ok(());
        }
        if self
            .validate_message_player_binding(
                message.proposer_id.as_str(),
                message.player_id.as_str(),
                "proposal",
            )
            .is_err()
        {
            return Ok(());
        }
        if self
            .validate_message_signer_binding(
                message.proposer_id.as_str(),
                message.public_key_hex.as_deref(),
                "proposal",
            )
            .is_err()
        {
            return Ok(());
        }
        if message.height < self.next_height {
            return Ok(());
        }
        if let Some(current) = self.pending.as_ref() {
            if current.height > message.height {
                return Ok(());
            }
            if current.height == message.height && current.block_hash == message.block_hash {
                return Ok(());
            }
        }
        if validate_consensus_action_root(message.action_root.as_str(), message.actions.as_slice())
            .is_err()
        {
            return Ok(());
        }

        let mut proposal = PendingProposal {
            height: message.height,
            slot: message.slot,
            epoch: message.epoch,
            proposer_id: message.proposer_id.clone(),
            block_hash: message.block_hash.clone(),
            action_root: message.action_root.clone(),
            committed_actions: message.actions.clone(),
            attestations: BTreeMap::new(),
            approved_stake: 0,
            rejected_stake: 0,
            status: PosConsensusStatus::Pending,
        };
        self.insert_attestation(
            &mut proposal,
            &message.proposer_id,
            true,
            message.proposed_at_ms,
            message.epoch.saturating_sub(1),
            message.epoch,
            Some(format!("proposal gossiped from {}", message.node_id)),
        )?;
        let next_height = self.next_height.max(proposal.height);
        let mut next_slot = self.next_slot;
        if proposal.slot >= self.next_slot {
            next_slot = checked_consensus_successor(
                proposal.slot,
                "proposal.slot",
                "ingesting proposal message",
            )?;
        }
        self.next_height = next_height;
        self.next_slot = next_slot;
        self.pending = Some(proposal);
        Ok(())
    }

    fn ingest_attestation_message(
        &mut self,
        world_id: &str,
        message: &GossipAttestationMessage,
    ) -> Result<(), NodeError> {
        if message.version != 1 || message.world_id != world_id {
            return Ok(());
        }
        if message.node_id != message.validator_id {
            return Ok(());
        }
        if self
            .validate_message_player_binding(
                message.validator_id.as_str(),
                message.player_id.as_str(),
                "attestation",
            )
            .is_err()
        {
            return Ok(());
        }
        if self
            .validate_message_signer_binding(
                message.validator_id.as_str(),
                message.public_key_hex.as_deref(),
                "attestation",
            )
            .is_err()
        {
            return Ok(());
        }
        let Some(mut proposal) = self.pending.clone() else {
            return Ok(());
        };
        if proposal.height != message.height || proposal.block_hash != message.block_hash {
            return Ok(());
        }

        self.insert_attestation(
            &mut proposal,
            &message.validator_id,
            message.approve,
            message.voted_at_ms,
            message.source_epoch,
            message.target_epoch,
            message.reason.clone(),
        )?;
        self.pending = Some(proposal);
        Ok(())
    }

    fn ingest_consensus_network_messages(
        &mut self,
        endpoint: &ConsensusNetworkEndpoint,
        world_id: &str,
    ) -> Result<(), NodeError> {
        let messages = endpoint.drain_messages()?;
        for message in messages {
            match message {
                GossipMessage::Commit(commit) => {
                    if commit.version != 1 || commit.world_id != world_id {
                        continue;
                    }
                    if verify_commit_message_signature(&commit, self.enforce_consensus_signature)
                        .is_err()
                    {
                        continue;
                    }
                    if self
                        .validate_peer_commit_execution_binding(
                            commit.height,
                            commit.execution_block_hash.as_deref(),
                            commit.execution_state_root.as_deref(),
                        )
                        .is_err()
                    {
                        continue;
                    }
                    if validate_consensus_action_root(
                        commit.action_root.as_str(),
                        commit.actions.as_slice(),
                    )
                    .is_err()
                    {
                        continue;
                    }
                    if self
                        .validate_message_player_binding(
                            commit.node_id.as_str(),
                            commit.player_id.as_str(),
                            "commit",
                        )
                        .is_err()
                    {
                        continue;
                    }
                    if self
                        .validate_message_signer_binding(
                            commit.node_id.as_str(),
                            commit.public_key_hex.as_deref(),
                            "commit",
                        )
                        .is_err()
                    {
                        continue;
                    }
                    let previous_height = self
                        .peer_heads
                        .get(commit.node_id.as_str())
                        .map(|head| head.height)
                        .unwrap_or(0);
                    if commit.height < previous_height {
                        continue;
                    }
                    self.peer_heads.insert(
                        commit.node_id.clone(),
                        PeerCommittedHead {
                            height: commit.height,
                            block_hash: commit.block_hash.clone(),
                            committed_at_ms: commit.committed_at_ms,
                            execution_block_hash: commit.execution_block_hash.clone(),
                            execution_state_root: commit.execution_state_root.clone(),
                        },
                    );
                    if commit.height > self.network_committed_height {
                        self.network_committed_height = commit.height;
                    }
                }
                GossipMessage::Proposal(proposal) => {
                    if proposal.version != 1 || proposal.world_id != world_id {
                        continue;
                    }
                    if verify_proposal_message_signature(
                        &proposal,
                        self.enforce_consensus_signature,
                    )
                    .is_err()
                    {
                        continue;
                    }
                    self.ingest_proposal_message(world_id, &proposal)?;
                }
                GossipMessage::Attestation(attestation) => {
                    if attestation.version != 1 || attestation.world_id != world_id {
                        continue;
                    }
                    if verify_attestation_message_signature(
                        &attestation,
                        self.enforce_consensus_signature,
                    )
                    .is_err()
                    {
                        continue;
                    }
                    self.ingest_attestation_message(world_id, &attestation)?;
                }
                GossipMessage::Replication(_) => {}
            }
        }
        Ok(())
    }

    fn ingest_peer_messages(
        &mut self,
        endpoint: &GossipEndpoint,
        node_id: &str,
        world_id: &str,
        mut replication: Option<&mut ReplicationRuntime>,
    ) -> Result<(), NodeError> {
        let messages = endpoint.drain_messages()?;
        for received in messages {
            let from = received.from;
            match received.message {
                GossipMessage::Commit(commit) => {
                    if commit.version != 1 || commit.world_id != world_id {
                        continue;
                    }
                    if verify_commit_message_signature(&commit, self.enforce_consensus_signature)
                        .is_err()
                    {
                        continue;
                    }
                    if self
                        .validate_peer_commit_execution_binding(
                            commit.height,
                            commit.execution_block_hash.as_deref(),
                            commit.execution_state_root.as_deref(),
                        )
                        .is_err()
                    {
                        continue;
                    }
                    if validate_consensus_action_root(
                        commit.action_root.as_str(),
                        commit.actions.as_slice(),
                    )
                    .is_err()
                    {
                        continue;
                    }
                    if self
                        .validate_message_player_binding(
                            commit.node_id.as_str(),
                            commit.player_id.as_str(),
                            "commit",
                        )
                        .is_err()
                    {
                        continue;
                    }
                    if self
                        .validate_message_signer_binding(
                            commit.node_id.as_str(),
                            commit.public_key_hex.as_deref(),
                            "commit",
                        )
                        .is_err()
                    {
                        continue;
                    }
                    endpoint.remember_peer(from)?;
                    let previous_height = self
                        .peer_heads
                        .get(commit.node_id.as_str())
                        .map(|head| head.height)
                        .unwrap_or(0);
                    if commit.height < previous_height {
                        continue;
                    }
                    self.peer_heads.insert(
                        commit.node_id.clone(),
                        PeerCommittedHead {
                            height: commit.height,
                            block_hash: commit.block_hash.clone(),
                            committed_at_ms: commit.committed_at_ms,
                            execution_block_hash: commit.execution_block_hash.clone(),
                            execution_state_root: commit.execution_state_root.clone(),
                        },
                    );
                    if commit.height > self.network_committed_height {
                        self.network_committed_height = commit.height;
                    }
                }
                GossipMessage::Proposal(proposal) => {
                    if proposal.version != 1 || proposal.world_id != world_id {
                        continue;
                    }
                    if verify_proposal_message_signature(
                        &proposal,
                        self.enforce_consensus_signature,
                    )
                    .is_err()
                    {
                        continue;
                    }
                    self.ingest_proposal_message(world_id, &proposal)?;
                    endpoint.remember_peer(from)?;
                }
                GossipMessage::Attestation(attestation) => {
                    if attestation.version != 1 || attestation.world_id != world_id {
                        continue;
                    }
                    if verify_attestation_message_signature(
                        &attestation,
                        self.enforce_consensus_signature,
                    )
                    .is_err()
                    {
                        continue;
                    }
                    self.ingest_attestation_message(world_id, &attestation)?;
                    endpoint.remember_peer(from)?;
                }
                GossipMessage::Replication(replication_msg) => {
                    if replication_msg.version != 1
                        || replication_msg.world_id != world_id
                        || replication_msg.record.world_id != world_id
                    {
                        continue;
                    }
                    if let Some(replication_runtime) = replication.as_deref_mut() {
                        if replication_runtime
                            .apply_remote_message(node_id, world_id, &replication_msg)
                            .is_ok()
                        {
                            endpoint.remember_peer(from)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn compute_block_hash(
        &self,
        world_id: &str,
        height: u64,
        slot: u64,
        epoch: u64,
        proposer_id: &str,
        parent_block_hash: &str,
        action_root: &str,
    ) -> Result<String, NodeError> {
        let payload = (
            1_u8,
            world_id,
            height,
            slot,
            epoch,
            proposer_id,
            parent_block_hash,
            action_root,
        );
        let bytes = serde_cbor::to_vec(&payload).map_err(|err| NodeError::Consensus {
            reason: format!("encode block hash payload failed: {err}"),
        })?;
        Ok(blake3_hex(bytes.as_slice()))
    }
}

fn parse_replication_commit_payload_view(payload: &[u8]) -> Option<ReplicationCommitPayloadView> {
    serde_json::from_slice::<ReplicationCommitPayloadView>(payload).ok()
}

fn parse_replication_commit_payload(payload: &[u8]) -> Option<ReplicationCommitPayload> {
    serde_json::from_slice::<ReplicationCommitPayload>(payload).ok()
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_action_payload;
#[cfg(test)]
mod tests_gossip_player;
#[cfg(test)]
mod tests_hardening;
