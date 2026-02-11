use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use super::super::super::error::WorldError;
use super::{
    normalized_schedule_key, validate_coordinator_lease_ttl_ms,
    MembershipRevocationAlertDeadLetterReason, MembershipRevocationAlertDeadLetterRecord,
    MembershipRevocationAlertDeadLetterStore, MembershipRevocationAlertRecoveryStore,
    MembershipRevocationScheduleCoordinator, MembershipSyncClient,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct MembershipRevocationDeadLetterReplayScheduleState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_replay_at_ms: Option<i64>,
    #[serde(default)]
    pub prefer_capacity_evicted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipRevocationDeadLetterReplayPolicy {
    pub max_replay_per_run: usize,
    pub max_retry_limit_exceeded_streak: usize,
}

impl Default for MembershipRevocationDeadLetterReplayPolicy {
    fn default() -> Self {
        Self {
            max_replay_per_run: 64,
            max_retry_limit_exceeded_streak: 3,
        }
    }
}

pub trait MembershipRevocationDeadLetterReplayStateStore {
    fn load_state(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<MembershipRevocationDeadLetterReplayScheduleState, WorldError>;

    fn save_state(
        &self,
        world_id: &str,
        node_id: &str,
        state: &MembershipRevocationDeadLetterReplayScheduleState,
    ) -> Result<(), WorldError>;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryMembershipRevocationDeadLetterReplayStateStore {
    states:
        Arc<Mutex<BTreeMap<(String, String), MembershipRevocationDeadLetterReplayScheduleState>>>,
}

impl InMemoryMembershipRevocationDeadLetterReplayStateStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MembershipRevocationDeadLetterReplayStateStore
    for InMemoryMembershipRevocationDeadLetterReplayStateStore
{
    fn load_state(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<MembershipRevocationDeadLetterReplayScheduleState, WorldError> {
        let key = normalized_schedule_key(world_id, node_id)?;
        let guard = self.states.lock().map_err(|_| {
            WorldError::Io("membership revocation dead-letter replay state lock poisoned".into())
        })?;
        Ok(guard.get(&key).cloned().unwrap_or_default())
    }

    fn save_state(
        &self,
        world_id: &str,
        node_id: &str,
        state: &MembershipRevocationDeadLetterReplayScheduleState,
    ) -> Result<(), WorldError> {
        let key = normalized_schedule_key(world_id, node_id)?;
        let mut guard = self.states.lock().map_err(|_| {
            WorldError::Io("membership revocation dead-letter replay state lock poisoned".into())
        })?;
        guard.insert(key, state.clone());
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileMembershipRevocationDeadLetterReplayStateStore {
    root_dir: PathBuf,
}

impl FileMembershipRevocationDeadLetterReplayStateStore {
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
            "{world_id}.{node_id}.revocation-dead-letter-replay-state.json"
        )))
    }
}

impl MembershipRevocationDeadLetterReplayStateStore
    for FileMembershipRevocationDeadLetterReplayStateStore
{
    fn load_state(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<MembershipRevocationDeadLetterReplayScheduleState, WorldError> {
        let path = self.state_path(world_id, node_id)?;
        if !path.exists() {
            return Ok(MembershipRevocationDeadLetterReplayScheduleState::default());
        }
        let bytes = fs::read(path)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    fn save_state(
        &self,
        world_id: &str,
        node_id: &str,
        state: &MembershipRevocationDeadLetterReplayScheduleState,
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

impl MembershipSyncClient {
    #[allow(clippy::too_many_arguments)]
    pub fn replay_revocation_dead_letters_with_policy(
        &self,
        world_id: &str,
        node_id: &str,
        policy: &MembershipRevocationDeadLetterReplayPolicy,
        state: &mut MembershipRevocationDeadLetterReplayScheduleState,
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        dead_letter_store: &(dyn MembershipRevocationAlertDeadLetterStore + Send + Sync),
    ) -> Result<usize, WorldError> {
        validate_dead_letter_replay_policy(policy)?;
        let (world_id, node_id) = normalized_schedule_key(world_id, node_id)?;
        let mut dead_letters = dead_letter_store.list(&world_id, &node_id)?;
        if dead_letters.is_empty() {
            return Ok(0);
        }

        let replay_count = dead_letters.len().min(policy.max_replay_per_run);
        let (replay_indices, next_prefer_capacity_evicted) = fair_dead_letter_indices(
            &dead_letters,
            replay_count,
            policy.max_retry_limit_exceeded_streak,
            state.prefer_capacity_evicted,
        );
        let replaying: Vec<MembershipRevocationAlertDeadLetterRecord> = replay_indices
            .iter()
            .map(|index| dead_letters[*index].clone())
            .collect();
        let mut replay_selected = vec![false; dead_letters.len()];
        for index in replay_indices {
            replay_selected[index] = true;
        }
        let remaining: Vec<MembershipRevocationAlertDeadLetterRecord> = dead_letters
            .drain(..)
            .enumerate()
            .filter_map(|(index, record)| (!replay_selected[index]).then_some(record))
            .collect();

        let mut pending = recovery_store.load_pending(&world_id, &node_id)?;
        for record in replaying {
            pending.push(record.pending_alert);
        }
        recovery_store.save_pending(&world_id, &node_id, &pending)?;
        dead_letter_store.replace(&world_id, &node_id, &remaining)?;
        state.prefer_capacity_evicted = next_prefer_capacity_evicted;
        Ok(replay_count)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run_revocation_dead_letter_replay_schedule_with_state_store(
        &self,
        world_id: &str,
        node_id: &str,
        now_ms: i64,
        replay_interval_ms: i64,
        replay_policy: &MembershipRevocationDeadLetterReplayPolicy,
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        dead_letter_store: &(dyn MembershipRevocationAlertDeadLetterStore + Send + Sync),
        replay_state_store: &(dyn MembershipRevocationDeadLetterReplayStateStore + Send + Sync),
    ) -> Result<usize, WorldError> {
        validate_replay_interval_ms(replay_interval_ms)?;
        validate_dead_letter_replay_policy(replay_policy)?;

        let mut state = replay_state_store.load_state(world_id, node_id)?;
        let should_run = state
            .last_replay_at_ms
            .map(|last| now_ms.saturating_sub(last) >= replay_interval_ms)
            .unwrap_or(true);
        if !should_run {
            return Ok(0);
        }

        let replayed = self.replay_revocation_dead_letters_with_policy(
            world_id,
            node_id,
            replay_policy,
            &mut state,
            recovery_store,
            dead_letter_store,
        )?;
        state.last_replay_at_ms = Some(now_ms);
        replay_state_store.save_state(world_id, node_id, &state)?;
        Ok(replayed)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run_revocation_dead_letter_replay_schedule_coordinated_with_state_store(
        &self,
        world_id: &str,
        target_node_id: &str,
        coordinator_node_id: &str,
        now_ms: i64,
        replay_interval_ms: i64,
        replay_policy: &MembershipRevocationDeadLetterReplayPolicy,
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        dead_letter_store: &(dyn MembershipRevocationAlertDeadLetterStore + Send + Sync),
        replay_state_store: &(dyn MembershipRevocationDeadLetterReplayStateStore + Send + Sync),
        coordinator: &(dyn MembershipRevocationScheduleCoordinator + Send + Sync),
        coordinator_lease_ttl_ms: i64,
    ) -> Result<usize, WorldError> {
        validate_coordinator_lease_ttl_ms(coordinator_lease_ttl_ms)?;
        let coordination_world_id =
            super::normalized_dead_letter_replay_coordination_world_id(world_id, target_node_id)?;
        if !coordinator.acquire(
            &coordination_world_id,
            coordinator_node_id,
            now_ms,
            coordinator_lease_ttl_ms,
        )? {
            return Ok(0);
        }

        let replay_outcome = self.run_revocation_dead_letter_replay_schedule_with_state_store(
            world_id,
            target_node_id,
            now_ms,
            replay_interval_ms,
            replay_policy,
            recovery_store,
            dead_letter_store,
            replay_state_store,
        );
        let release_outcome = coordinator.release(&coordination_world_id, coordinator_node_id);
        match (replay_outcome, release_outcome) {
            (Ok(replayed), Ok(())) => Ok(replayed),
            (Err(err), Ok(())) => Err(err),
            (Ok(_), Err(release_err)) => Err(release_err),
            (Err(err), Err(_)) => Err(err),
        }
    }
}

fn validate_replay_interval_ms(replay_interval_ms: i64) -> Result<(), WorldError> {
    if replay_interval_ms <= 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "membership revocation dead-letter replay_interval_ms must be positive, got {}",
                replay_interval_ms
            ),
        });
    }
    Ok(())
}

fn validate_dead_letter_replay_policy(
    policy: &MembershipRevocationDeadLetterReplayPolicy,
) -> Result<(), WorldError> {
    if policy.max_replay_per_run == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: "membership revocation dead-letter max_replay_per_run must be positive"
                .to_string(),
        });
    }
    if policy.max_retry_limit_exceeded_streak == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason:
                "membership revocation dead-letter max_retry_limit_exceeded_streak must be positive"
                    .to_string(),
        });
    }
    Ok(())
}

fn fair_dead_letter_indices(
    dead_letters: &[MembershipRevocationAlertDeadLetterRecord],
    replay_count: usize,
    max_retry_limit_exceeded_streak: usize,
    prefer_capacity_evicted: bool,
) -> (Vec<usize>, bool) {
    let mut retry_limit_exceeded = Vec::new();
    let mut capacity_evicted = Vec::new();
    for (index, record) in dead_letters.iter().enumerate() {
        match record.reason {
            MembershipRevocationAlertDeadLetterReason::RetryLimitExceeded => {
                retry_limit_exceeded.push(index);
            }
            MembershipRevocationAlertDeadLetterReason::CapacityEvicted => {
                capacity_evicted.push(index);
            }
        }
    }
    sort_dead_letter_bucket(dead_letters, &mut retry_limit_exceeded);
    sort_dead_letter_bucket(dead_letters, &mut capacity_evicted);

    let mut selected = Vec::with_capacity(replay_count);
    let mut retry_cursor = 0usize;
    let mut capacity_cursor = 0usize;
    let mut retry_streak = 0usize;
    let mut prefer_capacity_next = prefer_capacity_evicted;

    while selected.len() < replay_count {
        let retry_available = retry_cursor < retry_limit_exceeded.len();
        let capacity_available = capacity_cursor < capacity_evicted.len();
        if !retry_available && !capacity_available {
            break;
        }

        let take_capacity = if prefer_capacity_next && capacity_available {
            true
        } else if retry_available && capacity_available {
            retry_streak >= max_retry_limit_exceeded_streak
        } else {
            !retry_available && capacity_available
        };

        if take_capacity {
            selected.push(capacity_evicted[capacity_cursor]);
            capacity_cursor = capacity_cursor.saturating_add(1);
            retry_streak = 0;
            prefer_capacity_next = false;
            continue;
        }

        if retry_available {
            selected.push(retry_limit_exceeded[retry_cursor]);
            retry_cursor = retry_cursor.saturating_add(1);
            retry_streak = retry_streak.saturating_add(1);
            if capacity_cursor < capacity_evicted.len()
                && retry_streak >= max_retry_limit_exceeded_streak
            {
                prefer_capacity_next = true;
            }
            continue;
        }

        selected.push(capacity_evicted[capacity_cursor]);
        capacity_cursor = capacity_cursor.saturating_add(1);
        retry_streak = 0;
        prefer_capacity_next = false;
    }

    let capacity_selected = selected
        .iter()
        .filter(|index| {
            dead_letters[**index].reason
                == MembershipRevocationAlertDeadLetterReason::CapacityEvicted
        })
        .count();
    let capacity_remaining = capacity_cursor < capacity_evicted.len();
    let next_prefer_capacity_evicted = capacity_remaining && capacity_selected == 0;

    (selected, next_prefer_capacity_evicted)
}

fn sort_dead_letter_bucket(
    dead_letters: &[MembershipRevocationAlertDeadLetterRecord],
    indices: &mut [usize],
) {
    indices.sort_by_key(|index| {
        let record = &dead_letters[*index];
        (
            Reverse(record.pending_alert.attempt),
            record.dropped_at_ms,
            *index,
        )
    });
}
