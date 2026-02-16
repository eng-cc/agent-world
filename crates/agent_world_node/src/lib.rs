use std::collections::BTreeMap;
use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

mod consensus_signature;
mod gossip_udp;
mod libp2p_replication_network;
mod network_bridge;
mod pos_schedule;
mod pos_state_store;
mod pos_validation;
mod replication;
mod runtime_util;

use consensus_signature::{
    sign_attestation_message, sign_commit_message, sign_proposal_message,
    verify_attestation_message_signature, verify_commit_message_signature,
    verify_proposal_message_signature, ConsensusMessageSigner,
};
use gossip_udp::{
    GossipAttestationMessage, GossipCommitMessage, GossipEndpoint, GossipMessage,
    GossipProposalMessage,
};
pub use libp2p_replication_network::{Libp2pReplicationNetwork, Libp2pReplicationNetworkConfig};
pub use network_bridge::NodeReplicationNetworkHandle;
pub use replication::NodeReplicationConfig;

use network_bridge::ReplicationNetworkEndpoint;
use pos_state_store::PosNodeStateStore;
use pos_validation::{decide_status, validate_pos_config, validated_pos_state};
use replication::ReplicationRuntime;
use runtime_util::{lock_state, now_unix_ms};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    Sequencer,
    Storage,
    Observer,
}

impl NodeRole {
    pub fn as_str(self) -> &'static str {
        match self {
            NodeRole::Sequencer => "sequencer",
            NodeRole::Storage => "storage",
            NodeRole::Observer => "observer",
        }
    }
}

impl fmt::Display for NodeRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for NodeRole {
    type Err = NodeError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "sequencer" => Ok(NodeRole::Sequencer),
            "storage" => Ok(NodeRole::Storage),
            "observer" => Ok(NodeRole::Observer),
            _ => Err(NodeError::InvalidRole {
                role: raw.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeConsensusMode {
    Pos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum PosConsensusStatus {
    Pending,
    Committed,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PosValidator {
    pub validator_id: String,
    pub stake: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodePosConfig {
    pub validators: Vec<PosValidator>,
    pub supermajority_numerator: u64,
    pub supermajority_denominator: u64,
    pub epoch_length_slots: u64,
}

impl NodePosConfig {
    pub fn ethereum_like(validators: Vec<PosValidator>) -> Self {
        Self {
            validators,
            supermajority_numerator: 2,
            supermajority_denominator: 3,
            epoch_length_slots: 32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeConfig {
    pub node_id: String,
    pub world_id: String,
    pub tick_interval: Duration,
    pub role: NodeRole,
    pub pos_config: NodePosConfig,
    pub auto_attest_all_validators: bool,
    pub gossip: Option<NodeGossipConfig>,
    pub replication: Option<NodeReplicationConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeGossipConfig {
    pub bind_addr: SocketAddr,
    pub peers: Vec<SocketAddr>,
}

impl NodeConfig {
    pub fn new(
        node_id: impl Into<String>,
        world_id: impl Into<String>,
        role: NodeRole,
    ) -> Result<Self, NodeError> {
        let node_id = node_id.into();
        let world_id = world_id.into();
        if node_id.trim().is_empty() {
            return Err(NodeError::InvalidConfig {
                reason: "node_id cannot be empty".to_string(),
            });
        }
        if world_id.trim().is_empty() {
            return Err(NodeError::InvalidConfig {
                reason: "world_id cannot be empty".to_string(),
            });
        }

        let pos_config = NodePosConfig::ethereum_like(vec![PosValidator {
            validator_id: node_id.clone(),
            stake: 100,
        }]);
        validate_pos_config(&pos_config)?;

        Ok(Self {
            node_id,
            world_id,
            tick_interval: Duration::from_millis(200),
            role,
            pos_config,
            auto_attest_all_validators: true,
            gossip: None,
            replication: None,
        })
    }

    pub fn with_tick_interval(mut self, interval: Duration) -> Result<Self, NodeError> {
        if interval.is_zero() {
            return Err(NodeError::InvalidConfig {
                reason: "tick_interval must be positive".to_string(),
            });
        }
        self.tick_interval = interval;
        Ok(self)
    }

    pub fn with_pos_config(mut self, pos_config: NodePosConfig) -> Result<Self, NodeError> {
        validate_pos_config(&pos_config)?;
        self.pos_config = pos_config;
        Ok(self)
    }

    pub fn with_pos_validators(self, validators: Vec<PosValidator>) -> Result<Self, NodeError> {
        self.with_pos_config(NodePosConfig::ethereum_like(validators))
    }

    pub fn with_auto_attest_all_validators(mut self, enabled: bool) -> Self {
        self.auto_attest_all_validators = enabled;
        self
    }

    pub fn with_gossip(
        mut self,
        bind_addr: SocketAddr,
        peers: Vec<SocketAddr>,
    ) -> Result<Self, NodeError> {
        if peers.is_empty() {
            return Err(NodeError::InvalidConfig {
                reason: "gossip peers cannot be empty".to_string(),
            });
        }
        let mut dedup = BTreeMap::new();
        for peer in peers {
            dedup.insert(peer, ());
        }
        self.gossip = Some(NodeGossipConfig {
            bind_addr,
            peers: dedup.keys().copied().collect(),
        });
        Ok(self)
    }

    pub fn with_gossip_optional(mut self, bind_addr: SocketAddr, peers: Vec<SocketAddr>) -> Self {
        let mut dedup = BTreeMap::new();
        for peer in peers {
            dedup.insert(peer, ());
        }
        self.gossip = Some(NodeGossipConfig {
            bind_addr,
            peers: dedup.keys().copied().collect(),
        });
        self
    }

    pub fn with_replication_root(
        mut self,
        root_dir: impl Into<PathBuf>,
    ) -> Result<Self, NodeError> {
        self.replication = Some(NodeReplicationConfig::new(root_dir)?);
        Ok(self)
    }

    pub fn with_replication(mut self, replication: NodeReplicationConfig) -> Self {
        self.replication = Some(replication);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeConsensusSnapshot {
    pub mode: NodeConsensusMode,
    pub slot: u64,
    pub epoch: u64,
    pub latest_height: u64,
    pub committed_height: u64,
    pub network_committed_height: u64,
    pub known_peer_heads: usize,
    pub last_status: Option<PosConsensusStatus>,
    pub last_block_hash: Option<String>,
}

impl Default for NodeConsensusSnapshot {
    fn default() -> Self {
        Self {
            mode: NodeConsensusMode::Pos,
            slot: 0,
            epoch: 0,
            latest_height: 0,
            committed_height: 0,
            network_committed_height: 0,
            known_peer_heads: 0,
            last_status: None,
            last_block_hash: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeSnapshot {
    pub node_id: String,
    pub world_id: String,
    pub role: NodeRole,
    pub running: bool,
    pub tick_count: u64,
    pub last_tick_unix_ms: Option<i64>,
    pub consensus: NodeConsensusSnapshot,
    pub last_error: Option<String>,
}

#[derive(Debug)]
pub struct NodeRuntime {
    config: NodeConfig,
    replication_network: Option<NodeReplicationNetworkHandle>,
    running: Arc<AtomicBool>,
    state: Arc<Mutex<RuntimeState>>,
    stop_tx: Option<mpsc::Sender<()>>,
    worker: Option<JoinHandle<()>>,
}

#[derive(Debug, Clone)]
struct RuntimeState {
    tick_count: u64,
    last_tick_unix_ms: Option<i64>,
    consensus: NodeConsensusSnapshot,
    last_error: Option<String>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            tick_count: 0,
            last_tick_unix_ms: None,
            consensus: NodeConsensusSnapshot::default(),
            last_error: None,
        }
    }
}

impl NodeRuntime {
    pub fn new(config: NodeConfig) -> Self {
        Self {
            config,
            replication_network: None,
            running: Arc::new(AtomicBool::new(false)),
            state: Arc::new(Mutex::new(RuntimeState::default())),
            stop_tx: None,
            worker: None,
        }
    }

    pub fn with_replication_network(mut self, network: NodeReplicationNetworkHandle) -> Self {
        self.replication_network = Some(network);
        self
    }

    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

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

        let mut engine = match PosNodeEngine::new(&self.config) {
            Ok(engine) => engine,
            Err(err) => {
                self.running.store(false, Ordering::SeqCst);
                return Err(err);
            }
        };
        let pos_state_store = self
            .config
            .replication
            .as_ref()
            .map(PosNodeStateStore::from_replication);
        if let Some(store) = pos_state_store.as_ref() {
            if let Ok(Some(snapshot)) = store.load() {
                engine.restore_state_snapshot(snapshot);
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
        let mut replication = if let Some(config) = &self.config.replication {
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
        let tick_interval = self.config.tick_interval;
        let worker_name = format!("aw-node-{}", self.config.node_id);
        let running = Arc::clone(&self.running);
        let state = Arc::clone(&self.state);
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

                            let tick_result = engine.tick(
                                &node_id,
                                &world_id,
                                now_ms,
                                gossip.as_mut(),
                                replication.as_mut(),
                                replication_network.as_mut(),
                            );
                            let mut current = lock_state(&state);
                            match tick_result {
                                Ok(consensus_snapshot) => {
                                    current.consensus = consensus_snapshot;
                                    current.last_error = None;
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

#[derive(Debug, Clone)]
struct PosNodeEngine {
    validators: BTreeMap<String, u64>,
    total_stake: u64,
    required_stake: u64,
    epoch_length_slots: u64,
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
    consensus_signer: Option<ConsensusMessageSigner>,
    enforce_consensus_signature: bool,
    peer_heads: BTreeMap<String, PeerCommittedHead>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PosAttestation {
    validator_id: String,
    approve: bool,
    source_epoch: u64,
    target_epoch: u64,
    voted_at_ms: i64,
    reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingProposal {
    height: u64,
    slot: u64,
    epoch: u64,
    proposer_id: String,
    block_hash: String,
    attestations: BTreeMap<String, PosAttestation>,
    approved_stake: u64,
    rejected_stake: u64,
    status: PosConsensusStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PosDecision {
    height: u64,
    slot: u64,
    epoch: u64,
    status: PosConsensusStatus,
    block_hash: String,
    approved_stake: u64,
    rejected_stake: u64,
    required_stake: u64,
    total_stake: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PeerCommittedHead {
    height: u64,
    block_hash: String,
    committed_at_ms: i64,
}

impl PosNodeEngine {
    fn new(config: &NodeConfig) -> Result<Self, NodeError> {
        let (validators, total_stake, required_stake) = validated_pos_state(&config.pos_config)?;
        let (consensus_signer, enforce_consensus_signature) =
            if let Some(replication) = &config.replication {
                let signer = replication
                    .consensus_signer()?
                    .map(|(signing_key, public_key_hex)| {
                        ConsensusMessageSigner::new(signing_key, public_key_hex)
                    })
                    .transpose()?;
                (signer, replication.enforce_consensus_signature())
            } else {
                (None, false)
            };
        Ok(Self {
            validators,
            total_stake,
            required_stake,
            epoch_length_slots: config.pos_config.epoch_length_slots,
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
            consensus_signer,
            enforce_consensus_signature,
            peer_heads: BTreeMap::new(),
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
    ) -> Result<NodeConsensusSnapshot, NodeError> {
        if let Some(endpoint) = gossip.as_ref() {
            self.ingest_peer_messages(endpoint, node_id, world_id, replication.as_deref_mut())?;
        }
        if let Some(endpoint) = replication_network.as_ref() {
            self.ingest_network_replications(
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

        if let Some(endpoint) = gossip.as_ref() {
            self.broadcast_local_proposal(endpoint, node_id, world_id, now_ms)?;
            self.broadcast_local_attestation(endpoint, node_id, world_id, now_ms)?;
        }

        self.apply_decision(&decision);
        if let Some(endpoint) = gossip.as_ref() {
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
        if let Some(endpoint) = replication_network.as_ref() {
            self.ingest_network_replications(
                endpoint,
                node_id,
                world_id,
                replication.as_deref_mut(),
            )?;
        }
        Ok(self.snapshot_from_decision(&decision))
    }

    fn propose_next_head(
        &mut self,
        node_id: &str,
        world_id: &str,
        now_ms: i64,
    ) -> Result<PosDecision, NodeError> {
        let slot = self.next_slot;
        let epoch = self.slot_epoch(slot);
        let proposer_id = self
            .expected_proposer(slot)
            .ok_or_else(|| NodeError::Consensus {
                reason: "no proposer available".to_string(),
            })?;
        let block_hash = format!("{world_id}:h{}:s{slot}:p{proposer_id}", self.next_height);

        let mut proposal = PendingProposal {
            height: self.next_height,
            slot,
            epoch,
            proposer_id: proposer_id.clone(),
            block_hash: block_hash.clone(),
            attestations: BTreeMap::new(),
            approved_stake: 0,
            rejected_stake: 0,
            status: PosConsensusStatus::Pending,
        };

        self.insert_attestation(
            &mut proposal,
            &proposer_id,
            true,
            now_ms,
            epoch.saturating_sub(1),
            epoch,
            Some(format!("proposal accepted by {node_id}")),
        )?;

        self.next_slot = self.next_slot.saturating_add(1);
        let decision = self.decision_from_proposal(&proposal);
        self.pending = Some(proposal);
        Ok(decision)
    }

    fn advance_pending_attestations(&mut self, now_ms: i64) -> Result<PosDecision, NodeError> {
        let mut proposal = self.pending.clone().ok_or_else(|| NodeError::Consensus {
            reason: "missing pending proposal".to_string(),
        })?;

        for validator_id in self.validators.keys() {
            if proposal.attestations.contains_key(validator_id.as_str()) {
                continue;
            }
            let epoch = proposal.epoch;
            self.insert_attestation(
                &mut proposal,
                validator_id,
                true,
                now_ms,
                epoch.saturating_sub(1),
                epoch,
                Some("node mainloop auto attestation".to_string()),
            )?;
            if matches!(
                proposal.status,
                PosConsensusStatus::Committed | PosConsensusStatus::Rejected
            ) {
                break;
            }
            if !self.auto_attest_all_validators {
                break;
            }
        }

        let decision = self.decision_from_proposal(&proposal);
        self.pending = Some(proposal);
        Ok(decision)
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
        let stake =
            self.validators
                .get(validator_id)
                .copied()
                .ok_or_else(|| NodeError::Consensus {
                    reason: format!("validator not found: {}", validator_id),
                })?;
        if proposal.attestations.contains_key(validator_id) {
            return Ok(());
        }

        proposal.attestations.insert(
            validator_id.to_string(),
            PosAttestation {
                validator_id: validator_id.to_string(),
                approve,
                source_epoch,
                target_epoch,
                voted_at_ms,
                reason,
            },
        );
        if approve {
            proposal.approved_stake = proposal.approved_stake.saturating_add(stake);
        } else {
            proposal.rejected_stake = proposal.rejected_stake.saturating_add(stake);
        }
        proposal.status = decide_status(
            self.total_stake,
            self.required_stake,
            proposal.approved_stake,
            proposal.rejected_stake,
        );
        Ok(())
    }

    fn decision_from_proposal(&self, proposal: &PendingProposal) -> PosDecision {
        PosDecision {
            height: proposal.height,
            slot: proposal.slot,
            epoch: proposal.epoch,
            status: proposal.status,
            block_hash: proposal.block_hash.clone(),
            approved_stake: proposal.approved_stake,
            rejected_stake: proposal.rejected_stake,
            required_stake: self.required_stake,
            total_stake: self.total_stake,
        }
    }

    fn apply_decision(&mut self, decision: &PosDecision) {
        match decision.status {
            PosConsensusStatus::Pending => {}
            PosConsensusStatus::Committed => {
                self.committed_height = decision.height;
                self.network_committed_height = self.network_committed_height.max(decision.height);
                self.next_height = decision.height.saturating_add(1);
                self.pending = None;
            }
            PosConsensusStatus::Rejected => {
                self.next_height = decision.height.saturating_add(1);
                self.pending = None;
            }
        }
    }

    fn snapshot_from_decision(&self, decision: &PosDecision) -> NodeConsensusSnapshot {
        NodeConsensusSnapshot {
            mode: NodeConsensusMode::Pos,
            slot: self.next_slot,
            epoch: self.slot_epoch(self.next_slot),
            latest_height: decision.height,
            committed_height: self.committed_height,
            network_committed_height: self.network_committed_height.max(self.committed_height),
            known_peer_heads: self.peer_heads.len(),
            last_status: Some(decision.status),
            last_block_hash: Some(decision.block_hash.clone()),
        }
    }

    fn broadcast_local_proposal(
        &mut self,
        endpoint: &GossipEndpoint,
        node_id: &str,
        world_id: &str,
        now_ms: i64,
    ) -> Result<(), NodeError> {
        let Some(proposal) = self.pending.as_ref() else {
            return Ok(());
        };
        if proposal.proposer_id != node_id {
            return Ok(());
        }
        if proposal.height <= self.last_broadcast_proposal_height {
            return Ok(());
        }
        let mut message = GossipProposalMessage {
            version: 1,
            world_id: world_id.to_string(),
            node_id: node_id.to_string(),
            proposer_id: proposal.proposer_id.clone(),
            height: proposal.height,
            slot: proposal.slot,
            epoch: proposal.epoch,
            block_hash: proposal.block_hash.clone(),
            proposed_at_ms: now_ms,
            public_key_hex: None,
            signature_hex: None,
        };
        if let Some(signer) = self.consensus_signer.as_ref() {
            sign_proposal_message(&mut message, signer)?;
        }
        endpoint.broadcast_proposal(&message)?;
        self.last_broadcast_proposal_height = proposal.height;
        Ok(())
    }

    fn broadcast_local_attestation(
        &mut self,
        endpoint: &GossipEndpoint,
        node_id: &str,
        world_id: &str,
        now_ms: i64,
    ) -> Result<(), NodeError> {
        let Some(proposal) = self.pending.as_ref() else {
            return Ok(());
        };
        let Some(attestation) = proposal.attestations.get(node_id) else {
            return Ok(());
        };
        if proposal.height <= self.last_broadcast_local_attestation_height {
            return Ok(());
        }

        let mut message = GossipAttestationMessage {
            version: 1,
            world_id: world_id.to_string(),
            node_id: node_id.to_string(),
            validator_id: attestation.validator_id.clone(),
            height: proposal.height,
            slot: proposal.slot,
            epoch: proposal.epoch,
            block_hash: proposal.block_hash.clone(),
            approve: attestation.approve,
            source_epoch: attestation.source_epoch,
            target_epoch: attestation.target_epoch,
            voted_at_ms: now_ms,
            reason: attestation.reason.clone(),
            public_key_hex: None,
            signature_hex: None,
        };
        if let Some(signer) = self.consensus_signer.as_ref() {
            sign_attestation_message(&mut message, signer)?;
        }
        endpoint.broadcast_attestation(&message)?;
        self.last_broadcast_local_attestation_height = proposal.height;
        Ok(())
    }

    fn broadcast_local_commit(
        &mut self,
        endpoint: &GossipEndpoint,
        node_id: &str,
        world_id: &str,
        now_ms: i64,
        decision: &PosDecision,
    ) -> Result<(), NodeError> {
        if !matches!(decision.status, PosConsensusStatus::Committed) {
            return Ok(());
        }
        if decision.height <= self.last_broadcast_committed_height {
            return Ok(());
        }
        let mut message = GossipCommitMessage {
            version: 1,
            world_id: world_id.to_string(),
            node_id: node_id.to_string(),
            height: decision.height,
            slot: decision.slot,
            epoch: decision.epoch,
            block_hash: decision.block_hash.clone(),
            committed_at_ms: now_ms,
            public_key_hex: None,
            signature_hex: None,
        };
        if let Some(signer) = self.consensus_signer.as_ref() {
            sign_commit_message(&mut message, signer)?;
        }
        endpoint.broadcast_commit(&message)?;
        self.last_broadcast_committed_height = decision.height;
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
        if let Some(message) =
            replication.build_local_commit_message(node_id, world_id, now_ms, decision)?
        {
            if let Some(endpoint) = network_endpoint {
                endpoint.publish_replication(&message)?;
            } else if let Some(endpoint) = gossip_endpoint {
                endpoint.broadcast_replication(&message)?;
            }
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
        let messages = endpoint.drain_replications()?;
        for message in messages {
            if let Some(replication_runtime) = replication.as_deref_mut() {
                let _ = replication_runtime.apply_remote_message(node_id, world_id, &message);
            }
        }
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

        let mut proposal = PendingProposal {
            height: message.height,
            slot: message.slot,
            epoch: message.epoch,
            proposer_id: message.proposer_id.clone(),
            block_hash: message.block_hash.clone(),
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
        if proposal.height > self.next_height {
            self.next_height = proposal.height;
        }
        if proposal.slot >= self.next_slot {
            self.next_slot = proposal.slot.saturating_add(1);
        }
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

    fn ingest_peer_messages(
        &mut self,
        endpoint: &GossipEndpoint,
        node_id: &str,
        world_id: &str,
        mut replication: Option<&mut ReplicationRuntime>,
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
                        },
                    );
                    if commit.height > self.network_committed_height {
                        self.network_committed_height = commit.height;
                    }
                }
                GossipMessage::Proposal(proposal) => {
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
                GossipMessage::Replication(replication_msg) => {
                    if let Some(replication_runtime) = replication.as_deref_mut() {
                        let _ = replication_runtime.apply_remote_message(
                            node_id,
                            world_id,
                            &replication_msg,
                        );
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeError {
    InvalidRole { role: String },
    InvalidConfig { reason: String },
    Consensus { reason: String },
    Gossip { reason: String },
    Replication { reason: String },
    AlreadyRunning { node_id: String },
    NotRunning { node_id: String },
    ThreadSpawnFailed { reason: String },
    ThreadJoinFailed { node_id: String },
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeError::InvalidRole { role } => {
                write!(f, "invalid node role: {}", role)
            }
            NodeError::InvalidConfig { reason } => write!(f, "invalid node config: {}", reason),
            NodeError::Consensus { reason } => write!(f, "node consensus error: {}", reason),
            NodeError::Gossip { reason } => write!(f, "node gossip error: {}", reason),
            NodeError::Replication { reason } => write!(f, "node replication error: {}", reason),
            NodeError::AlreadyRunning { node_id } => {
                write!(f, "node runtime already running: {}", node_id)
            }
            NodeError::NotRunning { node_id } => write!(f, "node runtime not running: {}", node_id),
            NodeError::ThreadSpawnFailed { reason } => {
                write!(f, "failed to spawn node thread: {}", reason)
            }
            NodeError::ThreadJoinFailed { node_id } => {
                write!(f, "failed to join node thread: {}", node_id)
            }
        }
    }
}

impl std::error::Error for NodeError {}

#[cfg(test)]
mod tests;
