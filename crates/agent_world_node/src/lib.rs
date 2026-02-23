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
    let commit_replication_config = replication.clone();
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
                commit_replication_config
                    .authorize_fetch_commit_request(&request)
                    .map_err(|err| {
                        network_bad_request(format!("fetch-commit authorization failed: {}", err))
                    })?;
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
    let blob_replication_config = replication.clone();
    network
        .register_handler(
            REPLICATION_FETCH_BLOB_PROTOCOL,
            Box::new(move |payload| {
                let request =
                    serde_json::from_slice::<FetchBlobRequest>(payload).map_err(|err| {
                        network_bad_request(format!("decode fetch-blob request failed: {}", err))
                    })?;
                blob_replication_config
                    .authorize_fetch_blob_request(&request)
                    .map_err(|err| {
                        network_bad_request(format!("fetch-blob authorization failed: {}", err))
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

include!("lib_impl_part1.rs");
include!("lib_impl_part2.rs");

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
