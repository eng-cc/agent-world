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

    #[allow(clippy::too_many_arguments)]
    pub fn recommend_revocation_dead_letter_replay_policy(
        &self,
        world_id: &str,
        node_id: &str,
        current_policy: &MembershipRevocationDeadLetterReplayPolicy,
        replay_state_store: &(dyn MembershipRevocationDeadLetterReplayStateStore + Send + Sync),
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        dead_letter_store: &(dyn MembershipRevocationAlertDeadLetterStore + Send + Sync),
        metrics_lookback: usize,
        min_replay_per_run: usize,
        max_replay_per_run: usize,
        max_retry_limit_exceeded_streak: usize,
    ) -> Result<MembershipRevocationDeadLetterReplayPolicy, WorldError> {
        validate_dead_letter_replay_policy(current_policy)?;
        validate_adaptive_policy_bounds(
            metrics_lookback,
            min_replay_per_run,
            max_replay_per_run,
            max_retry_limit_exceeded_streak,
        )?;
        let (world_id, node_id) = normalized_schedule_key(world_id, node_id)?;

        let state = replay_state_store.load_state(&world_id, &node_id)?;
        let dead_letters = dead_letter_store.list(&world_id, &node_id)?;
        let pending = recovery_store.load_pending(&world_id, &node_id)?;
        let metric_lines = dead_letter_store.list_delivery_metrics(&world_id, &node_id)?;
        let metrics = aggregate_recent_delivery_metrics(&metric_lines, metrics_lookback);

        let mut recommendation = current_policy.clone();
        let backlog_total = dead_letters.len().saturating_add(pending.len());
        let retry_backlog = dead_letters
            .iter()
            .filter(|record| {
                record.reason == MembershipRevocationAlertDeadLetterReason::RetryLimitExceeded
            })
            .count();
        let capacity_backlog = dead_letters
            .iter()
            .filter(|record| {
                record.reason == MembershipRevocationAlertDeadLetterReason::CapacityEvicted
            })
            .count();
        let dead_letter_ratio_per_mille = ratio_per_mille(metrics.dead_lettered, metrics.attempted);
        let failure_ratio_per_mille = ratio_per_mille(metrics.failed, metrics.attempted);

        let high_backlog = backlog_total
            > current_policy
                .max_replay_per_run
                .saturating_mul(2)
                .max(min_replay_per_run);
        if high_backlog || retry_backlog > current_policy.max_replay_per_run {
            let step = current_policy.max_replay_per_run.max(2) / 2;
            recommendation.max_replay_per_run = recommendation
                .max_replay_per_run
                .saturating_add(step.max(1))
                .min(max_replay_per_run);
        }

        let low_backlog =
            backlog_total <= current_policy.max_replay_per_run.saturating_div(2).max(1);
        if low_backlog
            && pending.is_empty()
            && metrics.attempted >= 4
            && metrics.failed == 0
            && metrics.dead_lettered == 0
        {
            recommendation.max_replay_per_run = recommendation
                .max_replay_per_run
                .saturating_sub(1)
                .max(min_replay_per_run);
        }

        if capacity_backlog > 0 {
            if state.prefer_capacity_evicted || dead_letter_ratio_per_mille >= 250 {
                recommendation.max_retry_limit_exceeded_streak = recommendation
                    .max_retry_limit_exceeded_streak
                    .saturating_sub(1)
                    .max(1);
            } else if retry_backlog > capacity_backlog.saturating_mul(2)
                && failure_ratio_per_mille <= 350
            {
                recommendation.max_retry_limit_exceeded_streak = recommendation
                    .max_retry_limit_exceeded_streak
                    .saturating_add(1);
            }
        } else if retry_backlog > 0 && failure_ratio_per_mille <= 100 {
            recommendation.max_retry_limit_exceeded_streak = recommendation
                .max_retry_limit_exceeded_streak
                .saturating_add(1);
        }

        recommendation.max_replay_per_run = recommendation
            .max_replay_per_run
            .clamp(min_replay_per_run, max_replay_per_run);
        recommendation.max_retry_limit_exceeded_streak = recommendation
            .max_retry_limit_exceeded_streak
            .clamp(1, max_retry_limit_exceeded_streak);
        Ok(recommendation)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_adaptive_policy(
        &self,
        world_id: &str,
        target_node_id: &str,
        coordinator_node_id: &str,
        now_ms: i64,
        replay_interval_ms: i64,
        current_policy: &MembershipRevocationDeadLetterReplayPolicy,
        replay_state_store: &(dyn MembershipRevocationDeadLetterReplayStateStore + Send + Sync),
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        dead_letter_store: &(dyn MembershipRevocationAlertDeadLetterStore + Send + Sync),
        coordinator: &(dyn MembershipRevocationScheduleCoordinator + Send + Sync),
        coordinator_lease_ttl_ms: i64,
        metrics_lookback: usize,
        min_replay_per_run: usize,
        max_replay_per_run: usize,
        max_retry_limit_exceeded_streak: usize,
    ) -> Result<(usize, MembershipRevocationDeadLetterReplayPolicy), WorldError> {
        let recommended = self.recommend_revocation_dead_letter_replay_policy(
            world_id,
            target_node_id,
            current_policy,
            replay_state_store,
            recovery_store,
            dead_letter_store,
            metrics_lookback,
            min_replay_per_run,
            max_replay_per_run,
            max_retry_limit_exceeded_streak,
        )?;
        let replayed = self
            .run_revocation_dead_letter_replay_schedule_coordinated_with_state_store(
                world_id,
                target_node_id,
                coordinator_node_id,
                now_ms,
                replay_interval_ms,
                &recommended,
                recovery_store,
                dead_letter_store,
                replay_state_store,
                coordinator,
                coordinator_lease_ttl_ms,
            )?;
        Ok((replayed, recommended))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn recommend_revocation_dead_letter_replay_policy_with_adaptive_guard(
        &self,
        world_id: &str,
        node_id: &str,
        now_ms: i64,
        current_policy: &MembershipRevocationDeadLetterReplayPolicy,
        replay_state_store: &(dyn MembershipRevocationDeadLetterReplayStateStore + Send + Sync),
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        dead_letter_store: &(dyn MembershipRevocationAlertDeadLetterStore + Send + Sync),
        metrics_lookback: usize,
        min_replay_per_run: usize,
        max_replay_per_run: usize,
        max_retry_limit_exceeded_streak: usize,
        policy_cooldown_ms: i64,
        max_replay_step_change: usize,
        max_retry_streak_step_change: usize,
    ) -> Result<MembershipRevocationDeadLetterReplayPolicy, WorldError> {
        validate_adaptive_policy_guard_bounds(
            policy_cooldown_ms,
            max_replay_step_change,
            max_retry_streak_step_change,
        )?;
        let recommended = self.recommend_revocation_dead_letter_replay_policy(
            world_id,
            node_id,
            current_policy,
            replay_state_store,
            recovery_store,
            dead_letter_store,
            metrics_lookback,
            min_replay_per_run,
            max_replay_per_run,
            max_retry_limit_exceeded_streak,
        )?;
        let state = replay_state_store.load_state(world_id, node_id)?;
        let within_cooldown = state
            .last_replay_at_ms
            .map(|last| now_ms.saturating_sub(last) < policy_cooldown_ms)
            .unwrap_or(false);
        if within_cooldown {
            return Ok(current_policy.clone());
        }

        Ok(clamp_policy_change_with_step_limit(
            current_policy,
            &recommended,
            max_replay_step_change,
            max_retry_streak_step_change,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run_revocation_dead_letter_replay_schedule_coordinated_with_state_store_and_guarded_adaptive_policy(
        &self,
        world_id: &str,
        target_node_id: &str,
        coordinator_node_id: &str,
        now_ms: i64,
        replay_interval_ms: i64,
        current_policy: &MembershipRevocationDeadLetterReplayPolicy,
        replay_state_store: &(dyn MembershipRevocationDeadLetterReplayStateStore + Send + Sync),
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
    ) -> Result<(usize, MembershipRevocationDeadLetterReplayPolicy), WorldError> {
        let recommended = self.recommend_revocation_dead_letter_replay_policy_with_adaptive_guard(
            world_id,
            target_node_id,
            now_ms,
            current_policy,
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
        let replayed = self
            .run_revocation_dead_letter_replay_schedule_coordinated_with_state_store(
                world_id,
                target_node_id,
                coordinator_node_id,
                now_ms,
                replay_interval_ms,
                &recommended,
                recovery_store,
                dead_letter_store,
                replay_state_store,
                coordinator,
                coordinator_lease_ttl_ms,
            )?;
        Ok((replayed, recommended))
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

fn validate_adaptive_policy_bounds(
    metrics_lookback: usize,
    min_replay_per_run: usize,
    max_replay_per_run: usize,
    max_retry_limit_exceeded_streak: usize,
) -> Result<(), WorldError> {
    if metrics_lookback == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: "membership revocation dead-letter metrics_lookback must be positive"
                .to_string(),
        });
    }
    if min_replay_per_run == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: "membership revocation dead-letter min_replay_per_run must be positive"
                .to_string(),
        });
    }
    if min_replay_per_run > max_replay_per_run {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "membership revocation dead-letter replay bounds are invalid: min={} > max={}",
                min_replay_per_run, max_replay_per_run
            ),
        });
    }
    if max_retry_limit_exceeded_streak == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason:
                "membership revocation dead-letter max_retry_limit_exceeded_streak must be positive"
                    .to_string(),
        });
    }
    Ok(())
}

fn validate_adaptive_policy_guard_bounds(
    policy_cooldown_ms: i64,
    max_replay_step_change: usize,
    max_retry_streak_step_change: usize,
) -> Result<(), WorldError> {
    if policy_cooldown_ms <= 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "membership revocation dead-letter policy_cooldown_ms must be positive, got {}",
                policy_cooldown_ms
            ),
        });
    }
    if max_replay_step_change == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: "membership revocation dead-letter max_replay_step_change must be positive"
                .to_string(),
        });
    }
    if max_retry_streak_step_change == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason:
                "membership revocation dead-letter max_retry_streak_step_change must be positive"
                    .to_string(),
        });
    }
    Ok(())
}

fn clamp_policy_change_with_step_limit(
    current_policy: &MembershipRevocationDeadLetterReplayPolicy,
    recommended_policy: &MembershipRevocationDeadLetterReplayPolicy,
    max_replay_step_change: usize,
    max_retry_streak_step_change: usize,
) -> MembershipRevocationDeadLetterReplayPolicy {
    MembershipRevocationDeadLetterReplayPolicy {
        max_replay_per_run: clamp_usize_delta(
            current_policy.max_replay_per_run,
            recommended_policy.max_replay_per_run,
            max_replay_step_change,
        ),
        max_retry_limit_exceeded_streak: clamp_usize_delta(
            current_policy.max_retry_limit_exceeded_streak,
            recommended_policy.max_retry_limit_exceeded_streak,
            max_retry_streak_step_change,
        )
        .max(1),
    }
}

fn clamp_usize_delta(current: usize, target: usize, max_step_change: usize) -> usize {
    if current == target {
        return target;
    }
    if current < target {
        current.saturating_add(max_step_change).min(target)
    } else {
        current.saturating_sub(max_step_change).max(target)
    }
}

fn aggregate_recent_delivery_metrics(
    metric_lines: &[(i64, super::MembershipRevocationAlertDeliveryMetrics)],
    metrics_lookback: usize,
) -> super::MembershipRevocationAlertDeliveryMetrics {
    let start = metric_lines.len().saturating_sub(metrics_lookback);
    metric_lines[start..].iter().fold(
        super::MembershipRevocationAlertDeliveryMetrics::default(),
        |mut total, (_, metrics)| {
            total.attempted = total.attempted.saturating_add(metrics.attempted);
            total.succeeded = total.succeeded.saturating_add(metrics.succeeded);
            total.failed = total.failed.saturating_add(metrics.failed);
            total.deferred = total.deferred.saturating_add(metrics.deferred);
            total.buffered = total.buffered.saturating_add(metrics.buffered);
            total.dropped_capacity = total
                .dropped_capacity
                .saturating_add(metrics.dropped_capacity);
            total.dropped_retry_limit = total
                .dropped_retry_limit
                .saturating_add(metrics.dropped_retry_limit);
            total.dead_lettered = total.dead_lettered.saturating_add(metrics.dead_lettered);
            total
        },
    )
}

fn ratio_per_mille(numerator: usize, denominator: usize) -> usize {
    if denominator == 0 {
        return 0;
    }
    numerator.saturating_mul(1000) / denominator
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
