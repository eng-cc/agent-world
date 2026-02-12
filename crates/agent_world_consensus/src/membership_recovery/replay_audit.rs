use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use agent_world::runtime::WorldError;

use super::{
    normalized_schedule_key, MembershipRevocationAlertDeadLetterStore,
    MembershipRevocationAlertDeliveryMetrics, MembershipRevocationAlertRecoveryStore,
    MembershipRevocationDeadLetterReplayPolicy, MembershipRevocationDeadLetterReplayPolicyStore,
    MembershipRevocationDeadLetterReplayRollbackGuard,
    MembershipRevocationDeadLetterReplayStateStore, MembershipRevocationScheduleCoordinator,
    MembershipSyncClient,
};
use crate::membership_reconciliation::{
    MembershipRevocationAlertSeverity, MembershipRevocationAlertSink,
    MembershipRevocationAnomalyAlert,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipRevocationDeadLetterReplayPolicyAdoptionAuditDecision {
    Adopted,
    RolledBackToStable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipRevocationDeadLetterReplayPolicyAdoptionAuditRecord {
    pub world_id: String,
    pub node_id: String,
    pub audited_at_ms: i64,
    pub decision: MembershipRevocationDeadLetterReplayPolicyAdoptionAuditDecision,
    pub recommended_policy: MembershipRevocationDeadLetterReplayPolicy,
    pub applied_policy: MembershipRevocationDeadLetterReplayPolicy,
    pub stable_policy: MembershipRevocationDeadLetterReplayPolicy,
    pub backlog_dead_letters: usize,
    pub backlog_pending: usize,
    pub metrics: MembershipRevocationAlertDeliveryMetrics,
    pub rollback_triggered: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipRevocationDeadLetterReplayRollbackAlertPolicy {
    pub rollback_window_ms: i64,
    pub max_rollbacks_per_window: usize,
    pub min_attempted: usize,
    pub alert_cooldown_ms: i64,
}

impl Default for MembershipRevocationDeadLetterReplayRollbackAlertPolicy {
    fn default() -> Self {
        Self {
            rollback_window_ms: 120_000,
            max_rollbacks_per_window: 3,
            min_attempted: 8,
            alert_cooldown_ms: 60_000,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipRevocationDeadLetterReplayRollbackAlertState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_alert_at_ms: Option<i64>,
}

pub trait MembershipRevocationDeadLetterReplayRollbackAlertStateStore {
    fn load_alert_state(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<MembershipRevocationDeadLetterReplayRollbackAlertState, WorldError>;

    fn save_alert_state(
        &self,
        world_id: &str,
        node_id: &str,
        state: &MembershipRevocationDeadLetterReplayRollbackAlertState,
    ) -> Result<(), WorldError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MembershipRevocationDeadLetterReplayRollbackGovernanceLevel {
    #[default]
    Normal,
    Stable,
    Emergency,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipRevocationDeadLetterReplayRollbackGovernancePolicy {
    pub level_one_rollback_streak: usize,
    pub level_two_rollback_streak: usize,
    pub level_two_emergency_policy: MembershipRevocationDeadLetterReplayPolicy,
}

impl Default for MembershipRevocationDeadLetterReplayRollbackGovernancePolicy {
    fn default() -> Self {
        Self {
            level_one_rollback_streak: 2,
            level_two_rollback_streak: 4,
            level_two_emergency_policy: MembershipRevocationDeadLetterReplayPolicy {
                max_replay_per_run: 2,
                max_retry_limit_exceeded_streak: 1,
            },
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipRevocationDeadLetterReplayRollbackGovernanceState {
    #[serde(default)]
    pub rollback_streak: usize,
    #[serde(default)]
    pub last_level: MembershipRevocationDeadLetterReplayRollbackGovernanceLevel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_level_updated_at_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord {
    pub world_id: String,
    pub node_id: String,
    pub audited_at_ms: i64,
    pub governance_level: MembershipRevocationDeadLetterReplayRollbackGovernanceLevel,
    pub rollback_streak: usize,
    pub rolled_back: bool,
    pub applied_policy: MembershipRevocationDeadLetterReplayPolicy,
    pub alert_emitted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillReport {
    pub world_id: String,
    pub node_id: String,
    pub drill_at_ms: i64,
    pub alert_state: MembershipRevocationDeadLetterReplayRollbackAlertState,
    pub governance_state: MembershipRevocationDeadLetterReplayRollbackGovernanceState,
    pub recent_audits: Vec<MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord>,
    pub has_emergency_history: bool,
}

type MembershipRevocationDeadLetterReplayRollbackGovernanceRunResult = (
    usize,
    MembershipRevocationDeadLetterReplayPolicy,
    bool,
    bool,
    MembershipRevocationDeadLetterReplayRollbackGovernanceLevel,
);

pub trait MembershipRevocationDeadLetterReplayRollbackGovernanceStateStore {
    fn load_governance_state(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<MembershipRevocationDeadLetterReplayRollbackGovernanceState, WorldError>;

    fn save_governance_state(
        &self,
        world_id: &str,
        node_id: &str,
        state: &MembershipRevocationDeadLetterReplayRollbackGovernanceState,
    ) -> Result<(), WorldError>;
}

pub trait MembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore {
    fn append(
        &self,
        world_id: &str,
        node_id: &str,
        record: &MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord,
    ) -> Result<(), WorldError>;

    fn list(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<Vec<MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord>, WorldError>;
}

pub trait MembershipRevocationDeadLetterReplayPolicyAuditStore {
    fn append(
        &self,
        world_id: &str,
        node_id: &str,
        record: &MembershipRevocationDeadLetterReplayPolicyAdoptionAuditRecord,
    ) -> Result<(), WorldError>;

    fn list(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<Vec<MembershipRevocationDeadLetterReplayPolicyAdoptionAuditRecord>, WorldError>;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryMembershipRevocationDeadLetterReplayPolicyAuditStore {
    records: Arc<
        Mutex<
            BTreeMap<
                (String, String),
                Vec<MembershipRevocationDeadLetterReplayPolicyAdoptionAuditRecord>,
            >,
        >,
    >,
}

impl InMemoryMembershipRevocationDeadLetterReplayPolicyAuditStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MembershipRevocationDeadLetterReplayPolicyAuditStore
    for InMemoryMembershipRevocationDeadLetterReplayPolicyAuditStore
{
    fn append(
        &self,
        world_id: &str,
        node_id: &str,
        record: &MembershipRevocationDeadLetterReplayPolicyAdoptionAuditRecord,
    ) -> Result<(), WorldError> {
        let key = normalized_schedule_key(world_id, node_id)?;
        let mut guard = self.records.lock().map_err(|_| {
            WorldError::Io(
                "membership revocation dead-letter replay policy audit store lock poisoned".into(),
            )
        })?;
        guard.entry(key).or_default().push(record.clone());
        Ok(())
    }

    fn list(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<Vec<MembershipRevocationDeadLetterReplayPolicyAdoptionAuditRecord>, WorldError>
    {
        let key = normalized_schedule_key(world_id, node_id)?;
        let guard = self.records.lock().map_err(|_| {
            WorldError::Io(
                "membership revocation dead-letter replay policy audit store lock poisoned".into(),
            )
        })?;
        Ok(guard.get(&key).cloned().unwrap_or_default())
    }
}

#[derive(Debug, Clone)]
pub struct FileMembershipRevocationDeadLetterReplayPolicyAuditStore {
    root_dir: PathBuf,
}

impl FileMembershipRevocationDeadLetterReplayPolicyAuditStore {
    pub fn new(root_dir: impl Into<PathBuf>) -> Result<Self, WorldError> {
        let root_dir = root_dir.into();
        fs::create_dir_all(&root_dir)?;
        Ok(Self { root_dir })
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    fn audit_path(&self, world_id: &str, node_id: &str) -> Result<PathBuf, WorldError> {
        let (world_id, node_id) = normalized_schedule_key(world_id, node_id)?;
        Ok(self.root_dir.join(format!(
            "{world_id}.{node_id}.revocation-dead-letter-replay-policy-audit.jsonl"
        )))
    }
}

impl MembershipRevocationDeadLetterReplayPolicyAuditStore
    for FileMembershipRevocationDeadLetterReplayPolicyAuditStore
{
    fn append(
        &self,
        world_id: &str,
        node_id: &str,
        record: &MembershipRevocationDeadLetterReplayPolicyAdoptionAuditRecord,
    ) -> Result<(), WorldError> {
        let path = self.audit_path(world_id, node_id)?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let line = serde_json::to_string(record)?;
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    fn list(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<Vec<MembershipRevocationDeadLetterReplayPolicyAdoptionAuditRecord>, WorldError>
    {
        let path = self.audit_path(world_id, node_id)?;
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            records.push(serde_json::from_str(&line)?);
        }
        Ok(records)
    }
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryMembershipRevocationDeadLetterReplayRollbackAlertStateStore {
    states: Arc<
        Mutex<BTreeMap<(String, String), MembershipRevocationDeadLetterReplayRollbackAlertState>>,
    >,
}

impl InMemoryMembershipRevocationDeadLetterReplayRollbackAlertStateStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MembershipRevocationDeadLetterReplayRollbackAlertStateStore
    for InMemoryMembershipRevocationDeadLetterReplayRollbackAlertStateStore
{
    fn load_alert_state(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<MembershipRevocationDeadLetterReplayRollbackAlertState, WorldError> {
        let key = normalized_schedule_key(world_id, node_id)?;
        let guard = self.states.lock().map_err(|_| {
            WorldError::Io(
                "membership revocation dead-letter replay rollback alert state lock poisoned"
                    .into(),
            )
        })?;
        Ok(guard.get(&key).cloned().unwrap_or_default())
    }

    fn save_alert_state(
        &self,
        world_id: &str,
        node_id: &str,
        state: &MembershipRevocationDeadLetterReplayRollbackAlertState,
    ) -> Result<(), WorldError> {
        let key = normalized_schedule_key(world_id, node_id)?;
        let mut guard = self.states.lock().map_err(|_| {
            WorldError::Io(
                "membership revocation dead-letter replay rollback alert state lock poisoned"
                    .into(),
            )
        })?;
        guard.insert(key, state.clone());
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileMembershipRevocationDeadLetterReplayRollbackAlertStateStore {
    root_dir: PathBuf,
}

impl FileMembershipRevocationDeadLetterReplayRollbackAlertStateStore {
    pub fn new(root_dir: impl Into<PathBuf>) -> Result<Self, WorldError> {
        let root_dir = root_dir.into();
        fs::create_dir_all(&root_dir)?;
        Ok(Self { root_dir })
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    fn state_path(&self, world_id: &str, node_id: &str) -> Result<PathBuf, WorldError> {
        let (world_id, node_id) = normalized_schedule_key(world_id, node_id)?;
        Ok(self.root_dir.join(format!(
            "{world_id}.{node_id}.revocation-dead-letter-replay-rollback-alert-state.json"
        )))
    }
}

impl MembershipRevocationDeadLetterReplayRollbackAlertStateStore
    for FileMembershipRevocationDeadLetterReplayRollbackAlertStateStore
{
    fn load_alert_state(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<MembershipRevocationDeadLetterReplayRollbackAlertState, WorldError> {
        let path = self.state_path(world_id, node_id)?;
        if !path.exists() {
            return Ok(MembershipRevocationDeadLetterReplayRollbackAlertState::default());
        }
        let bytes = fs::read(path)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    fn save_alert_state(
        &self,
        world_id: &str,
        node_id: &str,
        state: &MembershipRevocationDeadLetterReplayRollbackAlertState,
    ) -> Result<(), WorldError> {
        let path = self.state_path(world_id, node_id)?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, serde_json::to_vec(state)?)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceStateStore {
    states: Arc<
        Mutex<
            BTreeMap<(String, String), MembershipRevocationDeadLetterReplayRollbackGovernanceState>,
        >,
    >,
}

impl InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceStateStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MembershipRevocationDeadLetterReplayRollbackGovernanceStateStore
    for InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceStateStore
{
    fn load_governance_state(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<MembershipRevocationDeadLetterReplayRollbackGovernanceState, WorldError> {
        let key = normalized_schedule_key(world_id, node_id)?;
        let guard = self.states.lock().map_err(|_| {
            WorldError::Io(
                "membership revocation dead-letter replay rollback governance state lock poisoned"
                    .into(),
            )
        })?;
        Ok(guard.get(&key).cloned().unwrap_or_default())
    }

    fn save_governance_state(
        &self,
        world_id: &str,
        node_id: &str,
        state: &MembershipRevocationDeadLetterReplayRollbackGovernanceState,
    ) -> Result<(), WorldError> {
        let key = normalized_schedule_key(world_id, node_id)?;
        let mut guard = self.states.lock().map_err(|_| {
            WorldError::Io(
                "membership revocation dead-letter replay rollback governance state lock poisoned"
                    .into(),
            )
        })?;
        guard.insert(key, state.clone());
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileMembershipRevocationDeadLetterReplayRollbackGovernanceStateStore {
    root_dir: PathBuf,
}

impl FileMembershipRevocationDeadLetterReplayRollbackGovernanceStateStore {
    pub fn new(root_dir: impl Into<PathBuf>) -> Result<Self, WorldError> {
        let root_dir = root_dir.into();
        fs::create_dir_all(&root_dir)?;
        Ok(Self { root_dir })
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    fn state_path(&self, world_id: &str, node_id: &str) -> Result<PathBuf, WorldError> {
        let (world_id, node_id) = normalized_schedule_key(world_id, node_id)?;
        Ok(self.root_dir.join(format!(
            "{world_id}.{node_id}.revocation-dead-letter-replay-rollback-governance-state.json"
        )))
    }
}

impl MembershipRevocationDeadLetterReplayRollbackGovernanceStateStore
    for FileMembershipRevocationDeadLetterReplayRollbackGovernanceStateStore
{
    fn load_governance_state(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<MembershipRevocationDeadLetterReplayRollbackGovernanceState, WorldError> {
        let path = self.state_path(world_id, node_id)?;
        if !path.exists() {
            return Ok(MembershipRevocationDeadLetterReplayRollbackGovernanceState::default());
        }
        let bytes = fs::read(path)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    fn save_governance_state(
        &self,
        world_id: &str,
        node_id: &str,
        state: &MembershipRevocationDeadLetterReplayRollbackGovernanceState,
    ) -> Result<(), WorldError> {
        let path = self.state_path(world_id, node_id)?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(path, serde_json::to_vec(state)?)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore {
    records: Arc<
        Mutex<
            BTreeMap<
                (String, String),
                Vec<MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord>,
            >,
        >,
    >,
}

impl InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore
    for InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore
{
    fn append(
        &self,
        world_id: &str,
        node_id: &str,
        record: &MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord,
    ) -> Result<(), WorldError> {
        let key = normalized_schedule_key(world_id, node_id)?;
        let mut guard = self.records.lock().map_err(|_| {
            WorldError::Io(
                "membership revocation dead-letter replay rollback governance audit store lock poisoned"
                    .into(),
            )
        })?;
        guard.entry(key).or_default().push(record.clone());
        Ok(())
    }

    fn list(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<Vec<MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord>, WorldError>
    {
        let key = normalized_schedule_key(world_id, node_id)?;
        let guard = self.records.lock().map_err(|_| {
            WorldError::Io(
                "membership revocation dead-letter replay rollback governance audit store lock poisoned"
                    .into(),
            )
        })?;
        Ok(guard.get(&key).cloned().unwrap_or_default())
    }
}

#[derive(Debug, Clone)]
pub struct FileMembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore {
    root_dir: PathBuf,
}

impl FileMembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore {
    pub fn new(root_dir: impl Into<PathBuf>) -> Result<Self, WorldError> {
        let root_dir = root_dir.into();
        fs::create_dir_all(&root_dir)?;
        Ok(Self { root_dir })
    }

    fn audit_path(&self, world_id: &str, node_id: &str) -> Result<PathBuf, WorldError> {
        let (world_id, node_id) = normalized_schedule_key(world_id, node_id)?;
        Ok(self.root_dir.join(format!(
            "{world_id}.{node_id}.revocation-dead-letter-replay-rollback-governance-audit.jsonl"
        )))
    }
}

impl MembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore
    for FileMembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore
{
    fn append(
        &self,
        world_id: &str,
        node_id: &str,
        record: &MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord,
    ) -> Result<(), WorldError> {
        let path = self.audit_path(world_id, node_id)?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let line = serde_json::to_string(record)?;
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    fn list(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<Vec<MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord>, WorldError>
    {
        let path = self.audit_path(world_id, node_id)?;
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            records.push(serde_json::from_str(&line)?);
        }
        Ok(records)
    }
}

impl MembershipSyncClient {
    #[allow(clippy::too_many_arguments)]
    pub fn run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_persisted_guarded_policy_with_audit_and_alert(
        &self,
        world_id: &str,
        target_node_id: &str,
        coordinator_node_id: &str,
        now_ms: i64,
        replay_interval_ms: i64,
        fallback_policy: &MembershipRevocationDeadLetterReplayPolicy,
        replay_state_store: &(dyn MembershipRevocationDeadLetterReplayStateStore + Send + Sync),
        replay_policy_store: &(dyn MembershipRevocationDeadLetterReplayPolicyStore + Send + Sync),
        replay_policy_audit_store: &(dyn MembershipRevocationDeadLetterReplayPolicyAuditStore
              + Send
              + Sync),
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        dead_letter_store: &(dyn MembershipRevocationAlertDeadLetterStore + Send + Sync),
        coordinator: &(dyn MembershipRevocationScheduleCoordinator + Send + Sync),
        coordinator_lease_ttl_ms: i64,
        metrics_lookback: usize,
        min_replay_per_run: usize,
        max_replay_per_run: usize,
        max_retry_limit_exceeded_streak: usize,
        policy_cooldown_ms: i64,
        max_replay_step_change: usize,
        max_retry_streak_step_change: usize,
        rollback_guard: &MembershipRevocationDeadLetterReplayRollbackGuard,
        rollback_alert_policy: &MembershipRevocationDeadLetterReplayRollbackAlertPolicy,
        rollback_alert_state: &mut MembershipRevocationDeadLetterReplayRollbackAlertState,
        alert_sink: &(dyn MembershipRevocationAlertSink + Send + Sync),
    ) -> Result<
        (
            usize,
            MembershipRevocationDeadLetterReplayPolicy,
            bool,
            bool,
        ),
        WorldError,
    > {
        validate_dead_letter_replay_rollback_alert_policy(rollback_alert_policy)?;
        let (world_id, node_id) = normalized_schedule_key(world_id, target_node_id)?;

        let mut before_state = replay_policy_store.load_policy_state(&world_id, &node_id)?;
        if before_state.last_policy_update_at_ms.is_none()
            && before_state.last_stable_at_ms.is_none()
            && before_state.last_rollback_at_ms.is_none()
        {
            before_state.active_policy = fallback_policy.clone();
            before_state.last_stable_policy = fallback_policy.clone();
        }

        let recommended_policy = self
            .recommend_revocation_dead_letter_replay_policy_with_adaptive_guard(
                &world_id,
                &node_id,
                now_ms,
                &before_state.active_policy,
                replay_state_store,
                recovery_store,
                dead_letter_store,
                metrics_lookback,
                min_replay_per_run,
                max_replay_per_run,
                max_retry_limit_exceeded_streak,
                policy_cooldown_ms,
                max_replay_step_change,
                max_retry_streak_step_change,
            )?;

        let (replayed, applied_policy, rolled_back) = self
            .run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_persisted_guarded_policy(
                &world_id,
                &node_id,
                coordinator_node_id,
                now_ms,
                replay_interval_ms,
                fallback_policy,
                replay_state_store,
                replay_policy_store,
                recovery_store,
                dead_letter_store,
                coordinator,
                coordinator_lease_ttl_ms,
                metrics_lookback,
                min_replay_per_run,
                max_replay_per_run,
                max_retry_limit_exceeded_streak,
                policy_cooldown_ms,
                max_replay_step_change,
                max_retry_streak_step_change,
                rollback_guard,
            )?;

        let policy_state = replay_policy_store.load_policy_state(&world_id, &node_id)?;
        let dead_letters = dead_letter_store.list(&world_id, &node_id)?;
        let pending = recovery_store.load_pending(&world_id, &node_id)?;
        let metric_lines = dead_letter_store.list_delivery_metrics(&world_id, &node_id)?;
        let metrics =
            super::replay::aggregate_recent_delivery_metrics(&metric_lines, metrics_lookback);
        let audit_record = MembershipRevocationDeadLetterReplayPolicyAdoptionAuditRecord {
            world_id: world_id.clone(),
            node_id: node_id.clone(),
            audited_at_ms: now_ms,
            decision: if rolled_back {
                MembershipRevocationDeadLetterReplayPolicyAdoptionAuditDecision::RolledBackToStable
            } else {
                MembershipRevocationDeadLetterReplayPolicyAdoptionAuditDecision::Adopted
            },
            recommended_policy,
            applied_policy: applied_policy.clone(),
            stable_policy: policy_state.last_stable_policy,
            backlog_dead_letters: dead_letters.len(),
            backlog_pending: pending.len(),
            metrics: metrics.clone(),
            rollback_triggered: rolled_back,
        };
        replay_policy_audit_store.append(&world_id, &node_id, &audit_record)?;
        let alert_emitted = emit_dead_letter_replay_rollback_alert_if_needed(
            &world_id,
            &node_id,
            now_ms,
            &audit_record,
            rollback_alert_policy,
            rollback_alert_state,
            replay_policy_audit_store,
            alert_sink,
        )?;
        Ok((replayed, applied_policy, rolled_back, alert_emitted))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_persisted_guarded_policy_with_audit_alert_store_and_governance(
        &self,
        world_id: &str,
        target_node_id: &str,
        coordinator_node_id: &str,
        now_ms: i64,
        replay_interval_ms: i64,
        fallback_policy: &MembershipRevocationDeadLetterReplayPolicy,
        replay_state_store: &(dyn MembershipRevocationDeadLetterReplayStateStore + Send + Sync),
        replay_policy_store: &(dyn MembershipRevocationDeadLetterReplayPolicyStore + Send + Sync),
        replay_policy_audit_store: &(dyn MembershipRevocationDeadLetterReplayPolicyAuditStore
              + Send
              + Sync),
        rollback_alert_state_store: &(dyn MembershipRevocationDeadLetterReplayRollbackAlertStateStore
              + Send
              + Sync),
        rollback_governance_state_store: &(dyn MembershipRevocationDeadLetterReplayRollbackGovernanceStateStore
              + Send
              + Sync),
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        dead_letter_store: &(dyn MembershipRevocationAlertDeadLetterStore + Send + Sync),
        coordinator: &(dyn MembershipRevocationScheduleCoordinator + Send + Sync),
        coordinator_lease_ttl_ms: i64,
        metrics_lookback: usize,
        min_replay_per_run: usize,
        max_replay_per_run: usize,
        max_retry_limit_exceeded_streak: usize,
        policy_cooldown_ms: i64,
        max_replay_step_change: usize,
        max_retry_streak_step_change: usize,
        rollback_guard: &MembershipRevocationDeadLetterReplayRollbackGuard,
        rollback_alert_policy: &MembershipRevocationDeadLetterReplayRollbackAlertPolicy,
        rollback_governance_policy: &MembershipRevocationDeadLetterReplayRollbackGovernancePolicy,
        alert_sink: &(dyn MembershipRevocationAlertSink + Send + Sync),
    ) -> Result<MembershipRevocationDeadLetterReplayRollbackGovernanceRunResult, WorldError> {
        validate_dead_letter_replay_rollback_governance_policy(rollback_governance_policy)?;
        let mut rollback_alert_state =
            rollback_alert_state_store.load_alert_state(world_id, target_node_id)?;
        let (replayed, applied_policy, rolled_back, alert_emitted) = self
            .run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_persisted_guarded_policy_with_audit_and_alert(
                world_id,
                target_node_id,
                coordinator_node_id,
                now_ms,
                replay_interval_ms,
                fallback_policy,
                replay_state_store,
                replay_policy_store,
                replay_policy_audit_store,
                recovery_store,
                dead_letter_store,
                coordinator,
                coordinator_lease_ttl_ms,
                metrics_lookback,
                min_replay_per_run,
                max_replay_per_run,
                max_retry_limit_exceeded_streak,
                policy_cooldown_ms,
                max_replay_step_change,
                max_retry_streak_step_change,
                rollback_guard,
                rollback_alert_policy,
                &mut rollback_alert_state,
                alert_sink,
            )?;
        rollback_alert_state_store.save_alert_state(
            world_id,
            target_node_id,
            &rollback_alert_state,
        )?;

        let mut policy_state = replay_policy_store.load_policy_state(world_id, target_node_id)?;
        let mut governance_state =
            rollback_governance_state_store.load_governance_state(world_id, target_node_id)?;
        let (governed_policy, governance_level) =
            apply_dead_letter_replay_rollback_governance_policy(
                now_ms,
                rolled_back,
                &applied_policy,
                &policy_state.last_stable_policy,
                rollback_governance_policy,
                &mut governance_state,
            );
        if governed_policy != applied_policy {
            policy_state.active_policy = governed_policy.clone();
            policy_state.last_policy_update_at_ms = Some(now_ms);
            replay_policy_store.save_policy_state(world_id, target_node_id, &policy_state)?;
        }
        rollback_governance_state_store.save_governance_state(
            world_id,
            target_node_id,
            &governance_state,
        )?;
        Ok((
            replayed,
            governed_policy,
            rolled_back,
            alert_emitted,
            governance_level,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_persisted_guarded_policy_with_audit_alert_store_governance_and_archive(
        &self,
        world_id: &str,
        target_node_id: &str,
        coordinator_node_id: &str,
        now_ms: i64,
        replay_interval_ms: i64,
        fallback_policy: &MembershipRevocationDeadLetterReplayPolicy,
        replay_state_store: &(dyn MembershipRevocationDeadLetterReplayStateStore + Send + Sync),
        replay_policy_store: &(dyn MembershipRevocationDeadLetterReplayPolicyStore + Send + Sync),
        replay_policy_audit_store: &(dyn MembershipRevocationDeadLetterReplayPolicyAuditStore
              + Send
              + Sync),
        rollback_alert_state_store: &(dyn MembershipRevocationDeadLetterReplayRollbackAlertStateStore
              + Send
              + Sync),
        rollback_governance_state_store: &(dyn MembershipRevocationDeadLetterReplayRollbackGovernanceStateStore
              + Send
              + Sync),
        rollback_governance_audit_store: &(dyn MembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore
              + Send
              + Sync),
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        dead_letter_store: &(dyn MembershipRevocationAlertDeadLetterStore + Send + Sync),
        coordinator: &(dyn MembershipRevocationScheduleCoordinator + Send + Sync),
        coordinator_lease_ttl_ms: i64,
        metrics_lookback: usize,
        min_replay_per_run: usize,
        max_replay_per_run: usize,
        max_retry_limit_exceeded_streak: usize,
        policy_cooldown_ms: i64,
        max_replay_step_change: usize,
        max_retry_streak_step_change: usize,
        rollback_guard: &MembershipRevocationDeadLetterReplayRollbackGuard,
        rollback_alert_policy: &MembershipRevocationDeadLetterReplayRollbackAlertPolicy,
        rollback_governance_policy: &MembershipRevocationDeadLetterReplayRollbackGovernancePolicy,
        alert_sink: &(dyn MembershipRevocationAlertSink + Send + Sync),
    ) -> Result<MembershipRevocationDeadLetterReplayRollbackGovernanceRunResult, WorldError> {
        let (replayed, policy, rolled_back, alert_emitted, governance_level) = self
            .run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_persisted_guarded_policy_with_audit_alert_store_and_governance(
                world_id,
                target_node_id,
                coordinator_node_id,
                now_ms,
                replay_interval_ms,
                fallback_policy,
                replay_state_store,
                replay_policy_store,
                replay_policy_audit_store,
                rollback_alert_state_store,
                rollback_governance_state_store,
                recovery_store,
                dead_letter_store,
                coordinator,
                coordinator_lease_ttl_ms,
                metrics_lookback,
                min_replay_per_run,
                max_replay_per_run,
                max_retry_limit_exceeded_streak,
                policy_cooldown_ms,
                max_replay_step_change,
                max_retry_streak_step_change,
                rollback_guard,
                rollback_alert_policy,
                rollback_governance_policy,
                alert_sink,
            )?;
        let (world_id, node_id) = normalized_schedule_key(world_id, target_node_id)?;
        let governance_state =
            rollback_governance_state_store.load_governance_state(&world_id, &node_id)?;
        let record = MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord {
            world_id: world_id.clone(),
            node_id: node_id.clone(),
            audited_at_ms: now_ms,
            governance_level,
            rollback_streak: governance_state.rollback_streak,
            rolled_back,
            applied_policy: policy.clone(),
            alert_emitted,
        };
        rollback_governance_audit_store.append(&world_id, &node_id, &record)?;
        Ok((
            replayed,
            policy,
            rolled_back,
            alert_emitted,
            governance_level,
        ))
    }

    pub fn run_revocation_dead_letter_replay_rollback_governance_recovery_drill(
        &self,
        world_id: &str,
        node_id: &str,
        drill_at_ms: i64,
        recent_audit_limit: usize,
        rollback_alert_state_store: &(dyn MembershipRevocationDeadLetterReplayRollbackAlertStateStore
              + Send
              + Sync),
        rollback_governance_state_store: &(dyn MembershipRevocationDeadLetterReplayRollbackGovernanceStateStore
              + Send
              + Sync),
        rollback_governance_audit_store: &(dyn MembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore
              + Send
              + Sync),
    ) -> Result<MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillReport, WorldError>
    {
        if recent_audit_limit == 0 {
            return Err(WorldError::DistributedValidationFailed {
                reason:
                    "membership revocation dead-letter governance recovery recent_audit_limit must be positive"
                        .to_string(),
            });
        }
        let (world_id, node_id) = normalized_schedule_key(world_id, node_id)?;
        let alert_state = rollback_alert_state_store.load_alert_state(&world_id, &node_id)?;
        let governance_state =
            rollback_governance_state_store.load_governance_state(&world_id, &node_id)?;
        let audits = rollback_governance_audit_store.list(&world_id, &node_id)?;
        let start = audits.len().saturating_sub(recent_audit_limit);
        let recent_audits = audits[start..].to_vec();
        let has_emergency_history = audits.iter().any(|record| {
            record.governance_level
                == MembershipRevocationDeadLetterReplayRollbackGovernanceLevel::Emergency
        });
        Ok(
            MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillReport {
                world_id,
                node_id,
                drill_at_ms,
                alert_state,
                governance_state,
                recent_audits,
                has_emergency_history,
            },
        )
    }
}

fn validate_dead_letter_replay_rollback_alert_policy(
    policy: &MembershipRevocationDeadLetterReplayRollbackAlertPolicy,
) -> Result<(), WorldError> {
    if policy.rollback_window_ms <= 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "membership revocation dead-letter rollback alert rollback_window_ms must be positive, got {}",
                policy.rollback_window_ms
            ),
        });
    }
    if policy.max_rollbacks_per_window == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: "membership revocation dead-letter rollback alert max_rollbacks_per_window must be positive".to_string(),
        });
    }
    if policy.min_attempted == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason:
                "membership revocation dead-letter rollback alert min_attempted must be positive"
                    .to_string(),
        });
    }
    if policy.alert_cooldown_ms <= 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "membership revocation dead-letter rollback alert alert_cooldown_ms must be positive, got {}",
                policy.alert_cooldown_ms
            ),
        });
    }
    Ok(())
}

fn validate_dead_letter_replay_rollback_governance_policy(
    policy: &MembershipRevocationDeadLetterReplayRollbackGovernancePolicy,
) -> Result<(), WorldError> {
    if policy.level_one_rollback_streak == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: "membership revocation dead-letter rollback governance level_one_rollback_streak must be positive".to_string(),
        });
    }
    if policy.level_two_rollback_streak < policy.level_one_rollback_streak {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "membership revocation dead-letter rollback governance thresholds are invalid: level_two={} < level_one={}",
                policy.level_two_rollback_streak,
                policy.level_one_rollback_streak
            ),
        });
    }
    if policy.level_two_emergency_policy.max_replay_per_run == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: "membership revocation dead-letter rollback governance emergency policy max_replay_per_run must be positive".to_string(),
        });
    }
    if policy
        .level_two_emergency_policy
        .max_retry_limit_exceeded_streak
        == 0
    {
        return Err(WorldError::DistributedValidationFailed {
            reason: "membership revocation dead-letter rollback governance emergency policy max_retry_limit_exceeded_streak must be positive".to_string(),
        });
    }
    Ok(())
}

fn apply_dead_letter_replay_rollback_governance_policy(
    now_ms: i64,
    rolled_back: bool,
    applied_policy: &MembershipRevocationDeadLetterReplayPolicy,
    stable_policy: &MembershipRevocationDeadLetterReplayPolicy,
    policy: &MembershipRevocationDeadLetterReplayRollbackGovernancePolicy,
    state: &mut MembershipRevocationDeadLetterReplayRollbackGovernanceState,
) -> (
    MembershipRevocationDeadLetterReplayPolicy,
    MembershipRevocationDeadLetterReplayRollbackGovernanceLevel,
) {
    state.rollback_streak = if rolled_back {
        state.rollback_streak.saturating_add(1)
    } else {
        0
    };
    let level = if state.rollback_streak >= policy.level_two_rollback_streak {
        MembershipRevocationDeadLetterReplayRollbackGovernanceLevel::Emergency
    } else if state.rollback_streak >= policy.level_one_rollback_streak {
        MembershipRevocationDeadLetterReplayRollbackGovernanceLevel::Stable
    } else {
        MembershipRevocationDeadLetterReplayRollbackGovernanceLevel::Normal
    };
    state.last_level = level;
    state.last_level_updated_at_ms = Some(now_ms);

    let governed_policy = match level {
        MembershipRevocationDeadLetterReplayRollbackGovernanceLevel::Normal => {
            applied_policy.clone()
        }
        MembershipRevocationDeadLetterReplayRollbackGovernanceLevel::Stable => {
            stable_policy.clone()
        }
        MembershipRevocationDeadLetterReplayRollbackGovernanceLevel::Emergency => {
            policy.level_two_emergency_policy.clone()
        }
    };
    (governed_policy, level)
}

#[allow(clippy::too_many_arguments)]
fn emit_dead_letter_replay_rollback_alert_if_needed(
    world_id: &str,
    node_id: &str,
    now_ms: i64,
    audit_record: &MembershipRevocationDeadLetterReplayPolicyAdoptionAuditRecord,
    policy: &MembershipRevocationDeadLetterReplayRollbackAlertPolicy,
    state: &mut MembershipRevocationDeadLetterReplayRollbackAlertState,
    replay_policy_audit_store: &(dyn MembershipRevocationDeadLetterReplayPolicyAuditStore
          + Send
          + Sync),
    alert_sink: &(dyn MembershipRevocationAlertSink + Send + Sync),
) -> Result<bool, WorldError> {
    if !audit_record.rollback_triggered || audit_record.metrics.attempted < policy.min_attempted {
        return Ok(false);
    }

    let records = replay_policy_audit_store.list(world_id, node_id)?;
    let rollback_count = records
        .iter()
        .filter(|record| {
            record.rollback_triggered
                && now_ms.saturating_sub(record.audited_at_ms) <= policy.rollback_window_ms
        })
        .count();
    if rollback_count < policy.max_rollbacks_per_window {
        return Ok(false);
    }

    let in_cooldown = state
        .last_alert_at_ms
        .map(|last| now_ms.saturating_sub(last) < policy.alert_cooldown_ms)
        .unwrap_or(false);
    if in_cooldown {
        return Ok(false);
    }

    let alert = MembershipRevocationAnomalyAlert {
        world_id: world_id.to_string(),
        node_id: node_id.to_string(),
        detected_at_ms: now_ms,
        severity: MembershipRevocationAlertSeverity::Critical,
        code: "dead_letter_replay_policy_rollback_anomaly".to_string(),
        message: format!(
            "membership revocation dead-letter replay rollback anomaly: {rollback_count} rollbacks within {}ms (attempted={}, failed={}, dead_lettered={})",
            policy.rollback_window_ms,
            audit_record.metrics.attempted,
            audit_record.metrics.failed,
            audit_record.metrics.dead_lettered
        ),
        drained: audit_record.backlog_pending,
        diverged: rollback_count,
        rejected: audit_record.metrics.dead_lettered,
    };
    alert_sink.emit(&alert)?;
    state.last_alert_at_ms = Some(now_ms);
    Ok(true)
}
