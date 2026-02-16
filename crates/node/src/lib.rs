use std::collections::{hash_map::DefaultHasher, BTreeMap};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeConsensusSnapshot {
    pub mode: NodeConsensusMode,
    pub slot: u64,
    pub epoch: u64,
    pub latest_height: u64,
    pub committed_height: u64,
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
            running: Arc::new(AtomicBool::new(false)),
            state: Arc::new(Mutex::new(RuntimeState::default())),
            stop_tx: None,
            worker: None,
        }
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

        let mut engine = PosNodeEngine::new(&self.config)?;
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

                            let tick_result = engine.tick(&node_id, &world_id, now_ms);
                            let mut current = lock_state(&state);
                            match tick_result {
                                Ok(consensus_snapshot) => {
                                    current.consensus = consensus_snapshot;
                                    current.last_error = None;
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
    pending: Option<PendingProposal>,
    auto_attest_all_validators: bool,
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

impl PosNodeEngine {
    fn new(config: &NodeConfig) -> Result<Self, NodeError> {
        let (validators, total_stake, required_stake) = validated_pos_state(&config.pos_config)?;
        Ok(Self {
            validators,
            total_stake,
            required_stake,
            epoch_length_slots: config.pos_config.epoch_length_slots,
            next_height: 1,
            next_slot: 0,
            committed_height: 0,
            pending: None,
            auto_attest_all_validators: config.auto_attest_all_validators,
        })
    }

    fn tick(
        &mut self,
        node_id: &str,
        world_id: &str,
        now_ms: i64,
    ) -> Result<NodeConsensusSnapshot, NodeError> {
        let mut decision = if self.pending.is_some() {
            self.advance_pending_attestations(now_ms)?
        } else {
            self.propose_next_head(node_id, world_id, now_ms)?
        };

        if matches!(decision.status, PosConsensusStatus::Pending) {
            decision = self.advance_pending_attestations(now_ms)?;
        }

        self.apply_decision(&decision);
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
            last_status: Some(decision.status),
            last_block_hash: Some(decision.block_hash.clone()),
        }
    }

    fn expected_proposer(&self, slot: u64) -> Option<String> {
        if self.validators.is_empty() || self.total_stake == 0 {
            return None;
        }
        let mut hasher = DefaultHasher::new();
        slot.hash(&mut hasher);
        let mut target = hasher.finish() % self.total_stake;
        for (validator_id, stake) in &self.validators {
            if target < *stake {
                return Some(validator_id.clone());
            }
            target = target.saturating_sub(*stake);
        }
        self.validators.keys().next().cloned()
    }

    fn slot_epoch(&self, slot: u64) -> u64 {
        slot / self.epoch_length_slots
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeError {
    InvalidRole { role: String },
    InvalidConfig { reason: String },
    Consensus { reason: String },
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

fn validate_pos_config(pos_config: &NodePosConfig) -> Result<(), NodeError> {
    let _ = validated_pos_state(pos_config)?;
    Ok(())
}

fn validated_pos_state(
    pos_config: &NodePosConfig,
) -> Result<(BTreeMap<String, u64>, u64, u64), NodeError> {
    if pos_config.validators.is_empty() {
        return Err(NodeError::InvalidConfig {
            reason: "pos validators cannot be empty".to_string(),
        });
    }
    if pos_config.epoch_length_slots == 0 {
        return Err(NodeError::InvalidConfig {
            reason: "epoch_length_slots must be positive".to_string(),
        });
    }
    if pos_config.supermajority_denominator == 0
        || pos_config.supermajority_numerator == 0
        || pos_config.supermajority_numerator > pos_config.supermajority_denominator
    {
        return Err(NodeError::InvalidConfig {
            reason: format!(
                "invalid supermajority ratio {}/{}",
                pos_config.supermajority_numerator, pos_config.supermajority_denominator
            ),
        });
    }
    if pos_config.supermajority_numerator.saturating_mul(2) <= pos_config.supermajority_denominator
    {
        return Err(NodeError::InvalidConfig {
            reason: "supermajority ratio must be greater than 1/2".to_string(),
        });
    }

    let mut validators = BTreeMap::new();
    let mut total_stake = 0u64;
    for validator in &pos_config.validators {
        let validator_id = validator.validator_id.trim();
        if validator_id.is_empty() {
            return Err(NodeError::InvalidConfig {
                reason: "validator_id cannot be empty".to_string(),
            });
        }
        if validator.stake == 0 {
            return Err(NodeError::InvalidConfig {
                reason: format!("validator {} stake must be positive", validator_id),
            });
        }
        if validators
            .insert(validator_id.to_string(), validator.stake)
            .is_some()
        {
            return Err(NodeError::InvalidConfig {
                reason: format!("duplicate validator: {}", validator_id),
            });
        }
        total_stake =
            total_stake
                .checked_add(validator.stake)
                .ok_or_else(|| NodeError::InvalidConfig {
                    reason: "total stake overflow".to_string(),
                })?;
    }

    let required_stake = required_supermajority_stake(
        total_stake,
        pos_config.supermajority_numerator,
        pos_config.supermajority_denominator,
    )?;
    Ok((validators, total_stake, required_stake))
}

fn required_supermajority_stake(
    total_stake: u64,
    numerator: u64,
    denominator: u64,
) -> Result<u64, NodeError> {
    let multiplied = u128::from(total_stake)
        .checked_mul(u128::from(numerator))
        .ok_or_else(|| NodeError::InvalidConfig {
            reason: "required stake overflow".to_string(),
        })?;
    let denominator = u128::from(denominator);
    let mut required = multiplied / denominator;
    if multiplied % denominator != 0 {
        required += 1;
    }
    let required = u64::try_from(required).map_err(|_| NodeError::InvalidConfig {
        reason: "required stake overflow".to_string(),
    })?;
    if required == 0 || required > total_stake {
        return Err(NodeError::InvalidConfig {
            reason: format!(
                "invalid required stake {} for total stake {}",
                required, total_stake
            ),
        });
    }
    Ok(required)
}

fn decide_status(
    total_stake: u64,
    required_stake: u64,
    approved_stake: u64,
    rejected_stake: u64,
) -> PosConsensusStatus {
    if approved_stake >= required_stake {
        return PosConsensusStatus::Committed;
    }
    if total_stake.saturating_sub(rejected_stake) < required_stake {
        PosConsensusStatus::Rejected
    } else {
        PosConsensusStatus::Pending
    }
}

fn lock_state<'a>(state: &'a Arc<Mutex<RuntimeState>>) -> std::sync::MutexGuard<'a, RuntimeState> {
    state
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn now_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

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
            .tick(&config.node_id, &config.world_id, 1_000)
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
                .tick(&config.node_id, &config.world_id, 2_000 + offset)
                .expect("tick");
            committed_height = snapshot.committed_height;
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
    fn runtime_rejects_double_start() {
        let config = NodeConfig::new("node-b", "world-b", NodeRole::Sequencer).expect("config");
        let mut runtime = NodeRuntime::new(config);
        runtime.start().expect("first start");
        let err = runtime.start().expect_err("second start must fail");
        assert!(matches!(err, NodeError::AlreadyRunning { .. }));
        runtime.stop().expect("stop");
    }
}
