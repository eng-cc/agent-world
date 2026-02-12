use std::cmp::Reverse;

use agent_world::runtime::WorldError;

use crate::membership::{
    MembershipDirectorySignerKeyring, MembershipSyncClient, MembershipSyncSubscription,
};
use crate::membership_logic;
use crate::membership_reconciliation::{
    MembershipRevocationAlertDedupPolicy, MembershipRevocationAlertDedupState,
    MembershipRevocationAlertPolicy, MembershipRevocationAlertSink,
    MembershipRevocationAnomalyAlert, MembershipRevocationReconcilePolicy,
    MembershipRevocationReconcileSchedulePolicy, MembershipRevocationScheduleCoordinator,
    MembershipRevocationScheduleStateStore,
};

mod dead_letter;
mod replay;
mod replay_archive;
mod replay_archive_federated;
mod replay_archive_tiered;
mod replay_audit;
mod stores;
mod types;

pub use dead_letter::{
    FileMembershipRevocationAlertDeadLetterStore, InMemoryMembershipRevocationAlertDeadLetterStore,
    MembershipRevocationAlertDeadLetterStore, NoopMembershipRevocationAlertDeadLetterStore,
};
#[rustfmt::skip]
pub use replay::{
    FileMembershipRevocationDeadLetterReplayPolicyStore, FileMembershipRevocationDeadLetterReplayStateStore,
    InMemoryMembershipRevocationDeadLetterReplayPolicyStore, InMemoryMembershipRevocationDeadLetterReplayStateStore,
    MembershipRevocationDeadLetterReplayPolicy, MembershipRevocationDeadLetterReplayPolicyState,
    MembershipRevocationDeadLetterReplayPolicyStore, MembershipRevocationDeadLetterReplayRollbackGuard,
    MembershipRevocationDeadLetterReplayScheduleState, MembershipRevocationDeadLetterReplayStateStore,
};
#[rustfmt::skip]
pub use replay_audit::{
    FileMembershipRevocationDeadLetterReplayPolicyAuditStore,
    FileMembershipRevocationDeadLetterReplayRollbackAlertStateStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceStateStore,
    InMemoryMembershipRevocationDeadLetterReplayPolicyAuditStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackAlertStateStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceStateStore,
    MembershipRevocationDeadLetterReplayPolicyAdoptionAuditDecision,
    MembershipRevocationDeadLetterReplayPolicyAdoptionAuditRecord,
    MembershipRevocationDeadLetterReplayPolicyAuditStore,
    MembershipRevocationDeadLetterReplayRollbackAlertPolicy,
    MembershipRevocationDeadLetterReplayRollbackAlertState,
    MembershipRevocationDeadLetterReplayRollbackAlertStateStore,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore,
    MembershipRevocationDeadLetterReplayRollbackGovernanceLevel,
    MembershipRevocationDeadLetterReplayRollbackGovernancePolicy,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceState,
    MembershipRevocationDeadLetterReplayRollbackGovernanceStateStore,
};
#[rustfmt::skip]
pub use replay_archive::{
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleStateStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleStateStore,
    MembershipRevocationDeadLetterReplayRollbackGovernanceArchiveDrillScheduledRunReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditPruneReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionPolicy,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillSchedulePolicy,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleState,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleStateStore,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduledRunReport,
};
#[rustfmt::skip]
pub use replay_archive_tiered::{
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertStateStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertStateStore,
    MembershipRevocationDeadLetterReplayRollbackGovernanceArchiveTieredOffloadDrillAlertRunReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditTieredOffloadPolicy,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditTieredOffloadReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertPolicy,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertRunReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertState,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertStateStore,
};
#[rustfmt::skip]
pub use replay_archive_federated::{
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventCompositeSequenceCursorStateStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventCompositeSequenceCursorStateStore,
    MembershipRevocationDeadLetterReplayRollbackGovernanceArchiveTieredOffloadDrillAlertEventBusRunReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryPolicy,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryRecord,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditArchiveTier,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventCompositeSequenceCursorState,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventCompositeSequenceCursorStateStore,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome,
};
pub use stores::{
    FileMembershipRevocationAlertRecoveryStore, FileMembershipRevocationCoordinatorStateStore,
    InMemoryMembershipRevocationAlertRecoveryStore,
    InMemoryMembershipRevocationCoordinatorStateStore, MembershipRevocationAlertRecoveryStore,
    MembershipRevocationCoordinatorStateStore, StoreBackedMembershipRevocationScheduleCoordinator,
};
pub use types::{
    MembershipRevocationAlertAckRetryPolicy, MembershipRevocationAlertDeadLetterReason,
    MembershipRevocationAlertDeadLetterRecord, MembershipRevocationAlertDeliveryMetrics,
    MembershipRevocationAlertRecoveryReport, MembershipRevocationCoordinatedRecoveryRunReport,
    MembershipRevocationCoordinatorLeaseState, MembershipRevocationPendingAlert,
};

impl MembershipSyncClient {
    pub fn emit_revocation_reconcile_alerts_with_recovery(
        &self,
        world_id: &str,
        node_id: &str,
        sink: &(dyn MembershipRevocationAlertSink + Send + Sync),
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        new_alerts: Vec<MembershipRevocationAnomalyAlert>,
    ) -> Result<MembershipRevocationAlertRecoveryReport, WorldError> {
        let policy = MembershipRevocationAlertAckRetryPolicy::legacy_compatible();
        let dead_letter_store = NoopMembershipRevocationAlertDeadLetterStore;
        self.emit_revocation_reconcile_alerts_with_recovery_and_ack_retry_with_dead_letter(
            world_id,
            node_id,
            0,
            sink,
            recovery_store,
            new_alerts,
            &policy,
            &dead_letter_store,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn emit_revocation_reconcile_alerts_with_recovery_and_ack_retry(
        &self,
        world_id: &str,
        node_id: &str,
        now_ms: i64,
        sink: &(dyn MembershipRevocationAlertSink + Send + Sync),
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        new_alerts: Vec<MembershipRevocationAnomalyAlert>,
        policy: &MembershipRevocationAlertAckRetryPolicy,
    ) -> Result<MembershipRevocationAlertRecoveryReport, WorldError> {
        let dead_letter_store = NoopMembershipRevocationAlertDeadLetterStore;
        self.emit_revocation_reconcile_alerts_with_recovery_and_ack_retry_with_dead_letter(
            world_id,
            node_id,
            now_ms,
            sink,
            recovery_store,
            new_alerts,
            policy,
            &dead_letter_store,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn emit_revocation_reconcile_alerts_with_recovery_and_ack_retry_with_dead_letter(
        &self,
        world_id: &str,
        node_id: &str,
        now_ms: i64,
        sink: &(dyn MembershipRevocationAlertSink + Send + Sync),
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        new_alerts: Vec<MembershipRevocationAnomalyAlert>,
        policy: &MembershipRevocationAlertAckRetryPolicy,
        dead_letter_store: &(dyn MembershipRevocationAlertDeadLetterStore + Send + Sync),
    ) -> Result<MembershipRevocationAlertRecoveryReport, WorldError> {
        validate_ack_retry_policy(policy)?;
        let (world_id, node_id) = normalized_schedule_key(world_id, node_id)?;

        let mut pending = recovery_store.load_pending(&world_id, &node_id)?;
        let mut buffered = Vec::with_capacity(pending.len().saturating_add(new_alerts.len()));
        let mut report = MembershipRevocationAlertRecoveryReport {
            recovered: 0,
            emitted_new: 0,
            buffered: 0,
            deferred: 0,
            dropped_capacity: 0,
            dropped_retry_limit: 0,
            delivery_metrics: MembershipRevocationAlertDeliveryMetrics::default(),
        };
        let mut metrics = MembershipRevocationAlertDeliveryMetrics::default();
        let mut transport_failed = false;

        for item in pending.drain(..) {
            if item.attempt >= policy.max_retry_attempts {
                report.dropped_retry_limit = report.dropped_retry_limit.saturating_add(1);
                archive_dead_letter(
                    dead_letter_store,
                    &world_id,
                    &node_id,
                    now_ms,
                    MembershipRevocationAlertDeadLetterReason::RetryLimitExceeded,
                    item,
                    &mut metrics,
                )?;
                continue;
            }
            if item.next_retry_at_ms > now_ms {
                report.deferred = report.deferred.saturating_add(1);
                buffered.push(item);
                continue;
            }
            if transport_failed {
                buffered.push(item);
                continue;
            }

            metrics.attempted = metrics.attempted.saturating_add(1);
            match sink.emit(&item.alert) {
                Ok(()) => {
                    report.recovered = report.recovered.saturating_add(1);
                    metrics.succeeded = metrics.succeeded.saturating_add(1);
                }
                Err(error) => {
                    transport_failed = true;
                    metrics.failed = metrics.failed.saturating_add(1);
                    let retried = item.with_retry_failure(
                        now_ms,
                        policy.retry_backoff_ms,
                        format!("{error:?}"),
                    );
                    if retried.attempt >= policy.max_retry_attempts {
                        report.dropped_retry_limit = report.dropped_retry_limit.saturating_add(1);
                        archive_dead_letter(
                            dead_letter_store,
                            &world_id,
                            &node_id,
                            now_ms,
                            MembershipRevocationAlertDeadLetterReason::RetryLimitExceeded,
                            retried,
                            &mut metrics,
                        )?;
                    } else {
                        buffered.push(retried);
                    }
                }
            }
        }

        for alert in new_alerts {
            if transport_failed {
                buffered.push(MembershipRevocationPendingAlert::new(alert, now_ms));
                continue;
            }

            metrics.attempted = metrics.attempted.saturating_add(1);
            match sink.emit(&alert) {
                Ok(()) => {
                    report.emitted_new = report.emitted_new.saturating_add(1);
                    metrics.succeeded = metrics.succeeded.saturating_add(1);
                }
                Err(error) => {
                    transport_failed = true;
                    metrics.failed = metrics.failed.saturating_add(1);
                    let retried = MembershipRevocationPendingAlert::new(alert, now_ms)
                        .with_retry_failure(now_ms, policy.retry_backoff_ms, format!("{error:?}"));
                    if retried.attempt >= policy.max_retry_attempts {
                        report.dropped_retry_limit = report.dropped_retry_limit.saturating_add(1);
                        archive_dead_letter(
                            dead_letter_store,
                            &world_id,
                            &node_id,
                            now_ms,
                            MembershipRevocationAlertDeadLetterReason::RetryLimitExceeded,
                            retried,
                            &mut metrics,
                        )?;
                    } else {
                        buffered.push(retried);
                    }
                }
            }
        }

        if buffered.len() > policy.max_pending_alerts {
            let dropped = buffered.split_off(policy.max_pending_alerts);
            report.dropped_capacity = dropped.len();
            for item in dropped {
                archive_dead_letter(
                    dead_letter_store,
                    &world_id,
                    &node_id,
                    now_ms,
                    MembershipRevocationAlertDeadLetterReason::CapacityEvicted,
                    item,
                    &mut metrics,
                )?;
            }
        }

        report.buffered = buffered.len();
        metrics.deferred = report.deferred;
        metrics.buffered = report.buffered;
        metrics.dropped_capacity = report.dropped_capacity;
        metrics.dropped_retry_limit = report.dropped_retry_limit;
        report.delivery_metrics = metrics;

        recovery_store.save_pending(&world_id, &node_id, &buffered)?;
        Ok(report)
    }

    pub fn replay_revocation_dead_letters(
        &self,
        world_id: &str,
        node_id: &str,
        max_replay: usize,
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        dead_letter_store: &(dyn MembershipRevocationAlertDeadLetterStore + Send + Sync),
    ) -> Result<usize, WorldError> {
        if max_replay == 0 {
            return Err(WorldError::DistributedValidationFailed {
                reason: "membership revocation dead-letter max_replay must be positive".to_string(),
            });
        }

        let (world_id, node_id) = normalized_schedule_key(world_id, node_id)?;
        let mut dead_letters = dead_letter_store.list(&world_id, &node_id)?;
        if dead_letters.is_empty() {
            return Ok(0);
        }

        let replay_count = dead_letters.len().min(max_replay);
        let replay_indices = prioritized_dead_letter_indices(&dead_letters, replay_count);
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
        Ok(replay_count)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run_revocation_dead_letter_replay_schedule(
        &self,
        world_id: &str,
        node_id: &str,
        now_ms: i64,
        replay_interval_ms: i64,
        max_replay: usize,
        last_replay_at_ms: &mut Option<i64>,
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        dead_letter_store: &(dyn MembershipRevocationAlertDeadLetterStore + Send + Sync),
    ) -> Result<usize, WorldError> {
        if replay_interval_ms <= 0 {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "membership revocation dead-letter replay_interval_ms must be positive, got {}",
                    replay_interval_ms
                ),
            });
        }

        let should_run = last_replay_at_ms
            .map(|last| now_ms.saturating_sub(last) >= replay_interval_ms)
            .unwrap_or(true);
        if !should_run {
            return Ok(0);
        }

        let replayed = self.replay_revocation_dead_letters(
            world_id,
            node_id,
            max_replay,
            recovery_store,
            dead_letter_store,
        )?;
        *last_replay_at_ms = Some(now_ms);
        Ok(replayed)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run_revocation_dead_letter_replay_schedule_coordinated(
        &self,
        world_id: &str,
        target_node_id: &str,
        coordinator_node_id: &str,
        now_ms: i64,
        replay_interval_ms: i64,
        max_replay: usize,
        last_replay_at_ms: &mut Option<i64>,
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        dead_letter_store: &(dyn MembershipRevocationAlertDeadLetterStore + Send + Sync),
        coordinator: &(dyn MembershipRevocationScheduleCoordinator + Send + Sync),
        coordinator_lease_ttl_ms: i64,
    ) -> Result<usize, WorldError> {
        validate_coordinator_lease_ttl_ms(coordinator_lease_ttl_ms)?;
        let coordination_world_id =
            normalized_dead_letter_replay_coordination_world_id(world_id, target_node_id)?;
        if !coordinator.acquire(
            &coordination_world_id,
            coordinator_node_id,
            now_ms,
            coordinator_lease_ttl_ms,
        )? {
            return Ok(0);
        }

        let replay_outcome = self.run_revocation_dead_letter_replay_schedule(
            world_id,
            target_node_id,
            now_ms,
            replay_interval_ms,
            max_replay,
            last_replay_at_ms,
            recovery_store,
            dead_letter_store,
        );
        let release_outcome = coordinator.release(&coordination_world_id, coordinator_node_id);
        match (replay_outcome, release_outcome) {
            (Ok(replayed), Ok(())) => Ok(replayed),
            (Err(err), Ok(())) => Err(err),
            (Ok(_), Err(release_err)) => Err(release_err),
            (Err(err), Err(_)) => Err(err),
        }
    }

    pub fn export_revocation_alert_delivery_metrics(
        &self,
        world_id: &str,
        node_id: &str,
        exported_at_ms: i64,
        metrics: &MembershipRevocationAlertDeliveryMetrics,
        dead_letter_store: &(dyn MembershipRevocationAlertDeadLetterStore + Send + Sync),
    ) -> Result<(), WorldError> {
        let (world_id, node_id) = normalized_schedule_key(world_id, node_id)?;
        dead_letter_store.append_delivery_metrics(&world_id, &node_id, exported_at_ms, metrics)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run_revocation_reconcile_coordinated_with_recovery_and_ack_retry_with_dead_letter_and_metrics_export(
        &self,
        world_id: &str,
        node_id: &str,
        now_ms: i64,
        subscription: &MembershipSyncSubscription,
        keyring: &mut MembershipDirectorySignerKeyring,
        reconcile_policy: &MembershipRevocationReconcilePolicy,
        schedule_policy: &MembershipRevocationReconcileSchedulePolicy,
        alert_policy: &MembershipRevocationAlertPolicy,
        dedup_policy: Option<&MembershipRevocationAlertDedupPolicy>,
        dedup_state: Option<&mut MembershipRevocationAlertDedupState>,
        schedule_store: &(dyn MembershipRevocationScheduleStateStore + Send + Sync),
        alert_sink: &(dyn MembershipRevocationAlertSink + Send + Sync),
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        recovery_policy: &MembershipRevocationAlertAckRetryPolicy,
        dead_letter_store: &(dyn MembershipRevocationAlertDeadLetterStore + Send + Sync),
        coordinator: &(dyn MembershipRevocationScheduleCoordinator + Send + Sync),
        coordinator_lease_ttl_ms: i64,
    ) -> Result<MembershipRevocationCoordinatedRecoveryRunReport, WorldError> {
        let report = self
            .run_revocation_reconcile_coordinated_with_recovery_and_ack_retry_with_dead_letter(
                world_id,
                node_id,
                now_ms,
                subscription,
                keyring,
                reconcile_policy,
                schedule_policy,
                alert_policy,
                dedup_policy,
                dedup_state,
                schedule_store,
                alert_sink,
                recovery_store,
                recovery_policy,
                dead_letter_store,
                coordinator,
                coordinator_lease_ttl_ms,
            )?;

        if report.acquired {
            self.export_revocation_alert_delivery_metrics(
                world_id,
                node_id,
                now_ms,
                &report.delivery_metrics,
                dead_letter_store,
            )?;
        }

        Ok(report)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run_revocation_reconcile_coordinated_with_recovery(
        &self,
        world_id: &str,
        node_id: &str,
        now_ms: i64,
        subscription: &MembershipSyncSubscription,
        keyring: &mut MembershipDirectorySignerKeyring,
        reconcile_policy: &MembershipRevocationReconcilePolicy,
        schedule_policy: &MembershipRevocationReconcileSchedulePolicy,
        alert_policy: &MembershipRevocationAlertPolicy,
        dedup_policy: Option<&MembershipRevocationAlertDedupPolicy>,
        dedup_state: Option<&mut MembershipRevocationAlertDedupState>,
        schedule_store: &(dyn MembershipRevocationScheduleStateStore + Send + Sync),
        alert_sink: &(dyn MembershipRevocationAlertSink + Send + Sync),
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        coordinator: &(dyn MembershipRevocationScheduleCoordinator + Send + Sync),
        coordinator_lease_ttl_ms: i64,
    ) -> Result<MembershipRevocationCoordinatedRecoveryRunReport, WorldError> {
        let policy = MembershipRevocationAlertAckRetryPolicy::legacy_compatible();
        let dead_letter_store = NoopMembershipRevocationAlertDeadLetterStore;
        self.run_revocation_reconcile_coordinated_with_recovery_and_ack_retry_with_dead_letter(
            world_id,
            node_id,
            now_ms,
            subscription,
            keyring,
            reconcile_policy,
            schedule_policy,
            alert_policy,
            dedup_policy,
            dedup_state,
            schedule_store,
            alert_sink,
            recovery_store,
            &policy,
            &dead_letter_store,
            coordinator,
            coordinator_lease_ttl_ms,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run_revocation_reconcile_coordinated_with_recovery_and_ack_retry(
        &self,
        world_id: &str,
        node_id: &str,
        now_ms: i64,
        subscription: &MembershipSyncSubscription,
        keyring: &mut MembershipDirectorySignerKeyring,
        reconcile_policy: &MembershipRevocationReconcilePolicy,
        schedule_policy: &MembershipRevocationReconcileSchedulePolicy,
        alert_policy: &MembershipRevocationAlertPolicy,
        dedup_policy: Option<&MembershipRevocationAlertDedupPolicy>,
        dedup_state: Option<&mut MembershipRevocationAlertDedupState>,
        schedule_store: &(dyn MembershipRevocationScheduleStateStore + Send + Sync),
        alert_sink: &(dyn MembershipRevocationAlertSink + Send + Sync),
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        recovery_policy: &MembershipRevocationAlertAckRetryPolicy,
        coordinator: &(dyn MembershipRevocationScheduleCoordinator + Send + Sync),
        coordinator_lease_ttl_ms: i64,
    ) -> Result<MembershipRevocationCoordinatedRecoveryRunReport, WorldError> {
        let dead_letter_store = NoopMembershipRevocationAlertDeadLetterStore;
        self.run_revocation_reconcile_coordinated_with_recovery_and_ack_retry_with_dead_letter(
            world_id,
            node_id,
            now_ms,
            subscription,
            keyring,
            reconcile_policy,
            schedule_policy,
            alert_policy,
            dedup_policy,
            dedup_state,
            schedule_store,
            alert_sink,
            recovery_store,
            recovery_policy,
            &dead_letter_store,
            coordinator,
            coordinator_lease_ttl_ms,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run_revocation_reconcile_coordinated_with_recovery_and_ack_retry_with_dead_letter(
        &self,
        world_id: &str,
        node_id: &str,
        now_ms: i64,
        subscription: &MembershipSyncSubscription,
        keyring: &mut MembershipDirectorySignerKeyring,
        reconcile_policy: &MembershipRevocationReconcilePolicy,
        schedule_policy: &MembershipRevocationReconcileSchedulePolicy,
        alert_policy: &MembershipRevocationAlertPolicy,
        dedup_policy: Option<&MembershipRevocationAlertDedupPolicy>,
        mut dedup_state: Option<&mut MembershipRevocationAlertDedupState>,
        schedule_store: &(dyn MembershipRevocationScheduleStateStore + Send + Sync),
        alert_sink: &(dyn MembershipRevocationAlertSink + Send + Sync),
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        recovery_policy: &MembershipRevocationAlertAckRetryPolicy,
        dead_letter_store: &(dyn MembershipRevocationAlertDeadLetterStore + Send + Sync),
        coordinator: &(dyn MembershipRevocationScheduleCoordinator + Send + Sync),
        coordinator_lease_ttl_ms: i64,
    ) -> Result<MembershipRevocationCoordinatedRecoveryRunReport, WorldError> {
        if !coordinator.acquire(world_id, node_id, now_ms, coordinator_lease_ttl_ms)? {
            return Ok(MembershipRevocationCoordinatedRecoveryRunReport {
                acquired: false,
                recovered_alerts: 0,
                emitted_alerts: 0,
                buffered_alerts: 0,
                deferred_alerts: 0,
                dropped_alerts_capacity: 0,
                dropped_alerts_retry_limit: 0,
                delivery_metrics: MembershipRevocationAlertDeliveryMetrics::default(),
                run_report: None,
            });
        }

        let run_outcome = (|| {
            let mut schedule_state = schedule_store.load(world_id, node_id)?;
            let run_report = self.run_revocation_reconcile_schedule(
                world_id,
                node_id,
                now_ms,
                subscription,
                keyring,
                reconcile_policy,
                schedule_policy,
                &mut schedule_state,
            )?;
            schedule_store.save(world_id, node_id, &schedule_state)?;

            let mut alerts = Vec::new();
            if let Some(reconcile_report) = run_report.reconcile_report.as_ref() {
                alerts = self.evaluate_revocation_reconcile_alerts(
                    world_id,
                    node_id,
                    now_ms,
                    reconcile_report,
                    alert_policy,
                )?;
                if let Some(dedup_policy) = dedup_policy {
                    let state = dedup_state.as_deref_mut().ok_or_else(|| {
                        WorldError::DistributedValidationFailed {
                            reason: "membership revocation dedup_state is required when dedup_policy is configured"
                                .to_string(),
                        }
                    })?;
                    alerts =
                        self.deduplicate_revocation_alerts(alerts, now_ms, dedup_policy, state)?;
                }
            }

            let recovery_report = self
                .emit_revocation_reconcile_alerts_with_recovery_and_ack_retry_with_dead_letter(
                    world_id,
                    node_id,
                    now_ms,
                    alert_sink,
                    recovery_store,
                    alerts,
                    recovery_policy,
                    dead_letter_store,
                )?;

            Ok(MembershipRevocationCoordinatedRecoveryRunReport {
                acquired: true,
                recovered_alerts: recovery_report.recovered,
                emitted_alerts: recovery_report.emitted_new,
                buffered_alerts: recovery_report.buffered,
                deferred_alerts: recovery_report.deferred,
                dropped_alerts_capacity: recovery_report.dropped_capacity,
                dropped_alerts_retry_limit: recovery_report.dropped_retry_limit,
                delivery_metrics: recovery_report.delivery_metrics.clone(),
                run_report: Some(run_report),
            })
        })();

        let release_outcome = coordinator.release(world_id, node_id);
        match (run_outcome, release_outcome) {
            (Ok(report), Ok(())) => Ok(report),
            (Err(err), Ok(())) => Err(err),
            (Ok(_), Err(release_err)) => Err(release_err),
            (Err(err), Err(_)) => Err(err),
        }
    }
}

fn archive_dead_letter(
    dead_letter_store: &(dyn MembershipRevocationAlertDeadLetterStore + Send + Sync),
    world_id: &str,
    node_id: &str,
    dropped_at_ms: i64,
    reason: MembershipRevocationAlertDeadLetterReason,
    pending_alert: MembershipRevocationPendingAlert,
    metrics: &mut MembershipRevocationAlertDeliveryMetrics,
) -> Result<(), WorldError> {
    dead_letter_store.append(&MembershipRevocationAlertDeadLetterRecord {
        world_id: world_id.to_string(),
        node_id: node_id.to_string(),
        dropped_at_ms,
        reason,
        pending_alert,
    })?;
    metrics.dead_lettered = metrics.dead_lettered.saturating_add(1);
    Ok(())
}

fn dead_letter_reason_priority(reason: MembershipRevocationAlertDeadLetterReason) -> u8 {
    match reason {
        MembershipRevocationAlertDeadLetterReason::RetryLimitExceeded => 2,
        MembershipRevocationAlertDeadLetterReason::CapacityEvicted => 1,
    }
}

fn prioritized_dead_letter_indices(
    dead_letters: &[MembershipRevocationAlertDeadLetterRecord],
    replay_count: usize,
) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..dead_letters.len()).collect();
    indices.sort_by_key(|index| {
        let record = &dead_letters[*index];
        (
            Reverse(dead_letter_reason_priority(record.reason)),
            Reverse(record.pending_alert.attempt),
            record.dropped_at_ms,
            *index,
        )
    });
    indices.truncate(replay_count);
    indices
}

pub(super) fn validate_coordinator_lease_ttl_ms(lease_ttl_ms: i64) -> Result<(), WorldError> {
    if lease_ttl_ms <= 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "membership revocation coordinator lease_ttl_ms must be positive, got {}",
                lease_ttl_ms
            ),
        });
    }
    Ok(())
}

fn validate_ack_retry_policy(
    policy: &MembershipRevocationAlertAckRetryPolicy,
) -> Result<(), WorldError> {
    if policy.max_pending_alerts == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: "membership revocation max_pending_alerts must be positive".to_string(),
        });
    }
    if policy.max_retry_attempts == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: "membership revocation max_retry_attempts must be positive".to_string(),
        });
    }
    if policy.retry_backoff_ms < 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "membership revocation retry_backoff_ms must be non-negative, got {}",
                policy.retry_backoff_ms
            ),
        });
    }
    Ok(())
}

fn normalized_node_id(raw: &str) -> Result<String, WorldError> {
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Err(WorldError::DistributedValidationFailed {
            reason: "membership revocation checkpoint node_id cannot be empty".to_string(),
        });
    }
    if normalized.contains('/') || normalized.contains('\\') || normalized.contains("..") {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!("membership revocation checkpoint node_id is invalid: {normalized}"),
        });
    }
    Ok(normalized.to_string())
}

pub(super) fn normalized_schedule_key(
    world_id: &str,
    node_id: &str,
) -> Result<(String, String), WorldError> {
    Ok((
        membership_logic::normalized_world_id(world_id)?,
        normalized_node_id(node_id)?,
    ))
}

fn normalized_dead_letter_replay_coordination_world_id(
    world_id: &str,
    target_node_id: &str,
) -> Result<String, WorldError> {
    let (world_id, target_node_id) = normalized_schedule_key(world_id, target_node_id)?;
    membership_logic::normalized_world_id(&format!(
        "{world_id}::revocation-dead-letter-replay::{target_node_id}"
    ))
}
