use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use super::super::distributed_net::{DistributedNetwork, InMemoryNetwork};
use super::*;

fn sample_client() -> MembershipSyncClient {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    MembershipSyncClient::new(network)
}

fn temp_membership_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("agent_world-{prefix}-{nanos}"))
}

fn sample_governance_audit_record(
    world_id: &str,
    node_id: &str,
    audited_at_ms: i64,
    governance_level: MembershipRevocationDeadLetterReplayRollbackGovernanceLevel,
    rollback_streak: usize,
) -> MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord {
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord {
        world_id: world_id.to_string(),
        node_id: node_id.to_string(),
        audited_at_ms,
        governance_level,
        rollback_streak,
        rolled_back: rollback_streak > 0,
        applied_policy: MembershipRevocationDeadLetterReplayPolicy {
            max_replay_per_run: 4,
            max_retry_limit_exceeded_streak: 2,
        },
        alert_emitted: false,
    }
}

#[test]
fn governance_audit_archive_prune_rewrites_file_store() {
    let client = sample_client();
    let root = temp_membership_dir("governance-audit-retention-store");
    fs::create_dir_all(&root).expect("create temp dir");
    let store =
        FileMembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore::new(&root)
            .expect("create governance audit retention file store");
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore::append(
        &store,
        "w1",
        "node-a",
        &sample_governance_audit_record(
            "w1",
            "node-a",
            1_000,
            MembershipRevocationDeadLetterReplayRollbackGovernanceLevel::Normal,
            0,
        ),
    )
    .expect("append record 1");
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore::append(
        &store,
        "w1",
        "node-a",
        &sample_governance_audit_record(
            "w1",
            "node-a",
            1_100,
            MembershipRevocationDeadLetterReplayRollbackGovernanceLevel::Stable,
            1,
        ),
    )
    .expect("append record 2");
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore::append(
        &store,
        "w1",
        "node-a",
        &sample_governance_audit_record(
            "w1",
            "node-a",
            1_200,
            MembershipRevocationDeadLetterReplayRollbackGovernanceLevel::Emergency,
            2,
        ),
    )
    .expect("append record 3");

    let retention_policy =
        MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionPolicy {
            max_records: 1,
            max_age_ms: 250,
        };
    let report = client
        .prune_revocation_dead_letter_replay_rollback_governance_audit_archive(
            "w1",
            "node-a",
            1_400,
            &retention_policy,
            &store,
        )
        .expect("prune governance audits");
    assert_eq!(report.before, 3);
    assert_eq!(report.after, 1);
    assert_eq!(report.pruned_by_age, 2);
    assert_eq!(report.pruned_by_capacity, 0);

    let kept = MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore::list(
        &store, "w1", "node-a",
    )
    .expect("list pruned records");
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].audited_at_ms, 1_200);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn governance_recovery_drill_schedule_state_store_file_round_trip() {
    let root = temp_membership_dir("governance-recovery-drill-schedule-state-store");
    fs::create_dir_all(&root).expect("create temp dir");
    let store = FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleStateStore::new(
        &root,
    )
    .expect("create schedule state store");
    let state = MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleState {
        last_drill_at_ms: Some(1_234),
    };
    store
        .save_state("w1", "node-a", &state)
        .expect("save schedule state");
    let loaded = store
        .load_state("w1", "node-a")
        .expect("load schedule state");
    assert_eq!(loaded, state);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn governance_recovery_drill_schedule_executes_when_due_and_persists_state() {
    let client = sample_client();
    let schedule_state_store =
        InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleStateStore::new();
    let rollback_alert_state_store =
        InMemoryMembershipRevocationDeadLetterReplayRollbackAlertStateStore::new();
    let rollback_governance_state_store =
        InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceStateStore::new();
    let rollback_governance_audit_store =
        InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore::new();

    rollback_alert_state_store
        .save_alert_state(
            "w1",
            "node-a",
            &MembershipRevocationDeadLetterReplayRollbackAlertState {
                last_alert_at_ms: Some(900),
            },
        )
        .expect("save alert state");
    rollback_governance_state_store
        .save_governance_state(
            "w1",
            "node-a",
            &MembershipRevocationDeadLetterReplayRollbackGovernanceState {
                rollback_streak: 2,
                last_level: MembershipRevocationDeadLetterReplayRollbackGovernanceLevel::Emergency,
                last_level_updated_at_ms: Some(900),
            },
        )
        .expect("save governance state");
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore::append(
        &rollback_governance_audit_store,
        "w1",
        "node-a",
        &sample_governance_audit_record(
            "w1",
            "node-a",
            800,
            MembershipRevocationDeadLetterReplayRollbackGovernanceLevel::Normal,
            0,
        ),
    )
    .expect("append record 1");
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore::append(
        &rollback_governance_audit_store,
        "w1",
        "node-a",
        &sample_governance_audit_record(
            "w1",
            "node-a",
            900,
            MembershipRevocationDeadLetterReplayRollbackGovernanceLevel::Stable,
            1,
        ),
    )
    .expect("append record 2");
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore::append(
        &rollback_governance_audit_store,
        "w1",
        "node-a",
        &sample_governance_audit_record(
            "w1",
            "node-a",
            1_000,
            MembershipRevocationDeadLetterReplayRollbackGovernanceLevel::Emergency,
            2,
        ),
    )
    .expect("append record 3");

    let policy =
        MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillSchedulePolicy {
            drill_interval_ms: 100,
            recent_audit_limit: 2,
        };

    let first = client
        .run_revocation_dead_letter_replay_rollback_governance_recovery_drill_schedule(
            "w1",
            "node-a",
            1_100,
            &policy,
            &schedule_state_store,
            &rollback_alert_state_store,
            &rollback_governance_state_store,
            &rollback_governance_audit_store,
        )
        .expect("first drill schedule run");
    assert!(first.drill_due);
    assert!(first.drill_executed);
    assert_eq!(first.next_due_at_ms, Some(1_200));
    assert_eq!(
        first
            .drill_report
            .as_ref()
            .expect("first drill report")
            .recent_audits
            .len(),
        2
    );

    let second = client
        .run_revocation_dead_letter_replay_rollback_governance_recovery_drill_schedule(
            "w1",
            "node-a",
            1_150,
            &policy,
            &schedule_state_store,
            &rollback_alert_state_store,
            &rollback_governance_state_store,
            &rollback_governance_audit_store,
        )
        .expect("second drill schedule run");
    assert!(!second.drill_due);
    assert!(!second.drill_executed);
    assert!(second.drill_report.is_none());
    assert_eq!(second.next_due_at_ms, Some(1_200));

    let state = schedule_state_store
        .load_state("w1", "node-a")
        .expect("load schedule state");
    assert_eq!(state.last_drill_at_ms, Some(1_100));
}

#[test]
fn governance_recovery_drill_schedule_rejects_invalid_policy() {
    let client = sample_client();
    let schedule_state_store =
        InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleStateStore::new();
    let rollback_alert_state_store =
        InMemoryMembershipRevocationDeadLetterReplayRollbackAlertStateStore::new();
    let rollback_governance_state_store =
        InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceStateStore::new();
    let rollback_governance_audit_store =
        InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore::new();

    let invalid_policy =
        MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillSchedulePolicy {
            drill_interval_ms: 0,
            recent_audit_limit: 1,
        };
    let error = client
        .run_revocation_dead_letter_replay_rollback_governance_recovery_drill_schedule(
            "w1",
            "node-a",
            1_200,
            &invalid_policy,
            &schedule_state_store,
            &rollback_alert_state_store,
            &rollback_governance_state_store,
            &rollback_governance_audit_store,
        )
        .expect_err("invalid schedule policy should fail");
    let message = format!("{error:?}");
    assert!(
        message.contains("drill_interval_ms must be positive"),
        "unexpected error: {message}"
    );
}

#[test]
fn governance_archive_and_drill_schedule_orchestration_runs_prune_then_drill() {
    let client = sample_client();
    let schedule_state_store =
        InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleStateStore::new();
    let rollback_alert_state_store =
        InMemoryMembershipRevocationDeadLetterReplayRollbackAlertStateStore::new();
    let rollback_governance_state_store =
        InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceStateStore::new();
    let rollback_governance_audit_retention_store =
        InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore::new();

    rollback_alert_state_store
        .save_alert_state(
            "w1",
            "node-a",
            &MembershipRevocationDeadLetterReplayRollbackAlertState {
                last_alert_at_ms: Some(950),
            },
        )
        .expect("save alert state");
    rollback_governance_state_store
        .save_governance_state(
            "w1",
            "node-a",
            &MembershipRevocationDeadLetterReplayRollbackGovernanceState {
                rollback_streak: 1,
                last_level: MembershipRevocationDeadLetterReplayRollbackGovernanceLevel::Stable,
                last_level_updated_at_ms: Some(950),
            },
        )
        .expect("save governance state");
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore::append(
        &rollback_governance_audit_retention_store,
        "w1",
        "node-a",
        &sample_governance_audit_record(
            "w1",
            "node-a",
            700,
            MembershipRevocationDeadLetterReplayRollbackGovernanceLevel::Normal,
            0,
        ),
    )
    .expect("append record 1");
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore::append(
        &rollback_governance_audit_retention_store,
        "w1",
        "node-a",
        &sample_governance_audit_record(
            "w1",
            "node-a",
            900,
            MembershipRevocationDeadLetterReplayRollbackGovernanceLevel::Stable,
            1,
        ),
    )
    .expect("append record 2");

    let retention_policy =
        MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionPolicy {
            max_records: 1,
            max_age_ms: 10_000,
        };
    let drill_policy =
        MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillSchedulePolicy {
            drill_interval_ms: 100,
            recent_audit_limit: 5,
        };
    let run_report = client
        .run_revocation_dead_letter_replay_rollback_governance_archive_retention_and_recovery_drill_schedule(
            "w1",
            "node-a",
            1_000,
            &retention_policy,
            &drill_policy,
            &rollback_governance_audit_retention_store,
            &schedule_state_store,
            &rollback_alert_state_store,
            &rollback_governance_state_store,
            &rollback_governance_audit_retention_store,
        )
        .expect("run archive and drill schedule orchestration");
    assert_eq!(run_report.prune_report.before, 2);
    assert_eq!(run_report.prune_report.after, 1);
    assert!(run_report.drill_run_report.drill_executed);
    assert_eq!(
        run_report
            .drill_run_report
            .drill_report
            .expect("drill report")
            .recent_audits
            .len(),
        1
    );
}
