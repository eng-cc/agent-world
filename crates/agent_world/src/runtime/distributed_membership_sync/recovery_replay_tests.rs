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

fn sample_dead_letter(
    world_id: &str,
    node_id: &str,
    detected_at_ms: i64,
    attempt: usize,
    reason: MembershipRevocationAlertDeadLetterReason,
) -> MembershipRevocationAlertDeadLetterRecord {
    MembershipRevocationAlertDeadLetterRecord {
        world_id: world_id.to_string(),
        node_id: node_id.to_string(),
        dropped_at_ms: detected_at_ms,
        reason,
        pending_alert: MembershipRevocationPendingAlert {
            alert: MembershipRevocationAnomalyAlert {
                world_id: world_id.to_string(),
                node_id: node_id.to_string(),
                detected_at_ms,
                severity: MembershipRevocationAlertSeverity::Warn,
                code: "reconcile_diverged".to_string(),
                message: "membership revocation reconcile diverged".to_string(),
                drained: 1,
                diverged: 1,
                rejected: 0,
            },
            attempt,
            next_retry_at_ms: detected_at_ms,
            last_error: None,
        },
    }
}

#[test]
fn replay_revocation_dead_letters_prioritizes_retry_limit_and_attempt() {
    let client = sample_client();
    let recovery_store = InMemoryMembershipRevocationAlertRecoveryStore::new();
    let dead_letter_store = InMemoryMembershipRevocationAlertDeadLetterStore::new();

    dead_letter_store
        .append(&sample_dead_letter(
            "w1",
            "node-a",
            1000,
            8,
            MembershipRevocationAlertDeadLetterReason::CapacityEvicted,
        ))
        .expect("append capacity record");
    dead_letter_store
        .append(&sample_dead_letter(
            "w1",
            "node-a",
            2000,
            1,
            MembershipRevocationAlertDeadLetterReason::RetryLimitExceeded,
        ))
        .expect("append retry-limit attempt1");
    dead_letter_store
        .append(&sample_dead_letter(
            "w1",
            "node-a",
            1500,
            3,
            MembershipRevocationAlertDeadLetterReason::RetryLimitExceeded,
        ))
        .expect("append retry-limit attempt3");

    let replayed = client
        .replay_revocation_dead_letters("w1", "node-a", 2, &recovery_store, &dead_letter_store)
        .expect("replay dead letters");
    assert_eq!(replayed, 2);

    let pending = recovery_store
        .load_pending("w1", "node-a")
        .expect("load pending");
    assert_eq!(pending.len(), 2);
    assert_eq!(pending[0].alert.detected_at_ms, 1500);
    assert_eq!(pending[1].alert.detected_at_ms, 2000);

    let remaining = dead_letter_store
        .list("w1", "node-a")
        .expect("load remaining dead letters");
    assert_eq!(remaining.len(), 1);
    assert_eq!(
        remaining[0].reason,
        MembershipRevocationAlertDeadLetterReason::CapacityEvicted
    );
}

#[test]
fn run_revocation_dead_letter_replay_schedule_coordinated_respects_cross_node_lease() {
    let client = sample_client();
    let recovery_store = InMemoryMembershipRevocationAlertRecoveryStore::new();
    let dead_letter_store = InMemoryMembershipRevocationAlertDeadLetterStore::new();
    let coordinator_store: Arc<dyn MembershipRevocationCoordinatorStateStore + Send + Sync> =
        Arc::new(InMemoryMembershipRevocationCoordinatorStateStore::new());
    let coordinator_a =
        StoreBackedMembershipRevocationScheduleCoordinator::new(Arc::clone(&coordinator_store));
    let coordinator_b =
        StoreBackedMembershipRevocationScheduleCoordinator::new(Arc::clone(&coordinator_store));

    dead_letter_store
        .append(&sample_dead_letter(
            "w1",
            "node-target",
            1000,
            1,
            MembershipRevocationAlertDeadLetterReason::RetryLimitExceeded,
        ))
        .expect("append dead letter");

    let coordination_world_id = "w1::revocation-dead-letter-replay::node-target";
    assert!(coordinator_a
        .acquire(coordination_world_id, "node-a", 1000, 500)
        .expect("acquire lease with node-a"));

    let mut last_replay_at_ms = None;
    let blocked = client
        .run_revocation_dead_letter_replay_schedule_coordinated(
            "w1",
            "node-target",
            "node-b",
            1001,
            100,
            1,
            &mut last_replay_at_ms,
            &recovery_store,
            &dead_letter_store,
            &coordinator_b,
            500,
        )
        .expect("run coordinated replay while lease is held");
    assert_eq!(blocked, 0);
    assert_eq!(last_replay_at_ms, None);

    coordinator_a
        .release(coordination_world_id, "node-a")
        .expect("release node-a lease");

    let replayed = client
        .run_revocation_dead_letter_replay_schedule_coordinated(
            "w1",
            "node-target",
            "node-b",
            1105,
            100,
            1,
            &mut last_replay_at_ms,
            &recovery_store,
            &dead_letter_store,
            &coordinator_b,
            500,
        )
        .expect("run coordinated replay after lease release");
    assert_eq!(replayed, 1);
    assert_eq!(last_replay_at_ms, Some(1105));

    let pending = recovery_store
        .load_pending("w1", "node-target")
        .expect("load pending");
    assert_eq!(pending.len(), 1);
    let remaining = dead_letter_store
        .list("w1", "node-target")
        .expect("load remaining dead letters");
    assert!(remaining.is_empty());
}

#[test]
fn replay_revocation_dead_letters_with_policy_rotates_capacity_evicted() {
    let client = sample_client();
    let recovery_store = InMemoryMembershipRevocationAlertRecoveryStore::new();
    let dead_letter_store = InMemoryMembershipRevocationAlertDeadLetterStore::new();
    let policy = MembershipRevocationDeadLetterReplayPolicy {
        max_replay_per_run: 1,
        max_retry_limit_exceeded_streak: 8,
    };
    let mut state = MembershipRevocationDeadLetterReplayScheduleState::default();

    dead_letter_store
        .append(&sample_dead_letter(
            "w1",
            "node-a",
            1000,
            2,
            MembershipRevocationAlertDeadLetterReason::RetryLimitExceeded,
        ))
        .expect("append retry-limit #1");
    dead_letter_store
        .append(&sample_dead_letter(
            "w1",
            "node-a",
            1001,
            2,
            MembershipRevocationAlertDeadLetterReason::RetryLimitExceeded,
        ))
        .expect("append retry-limit #2");
    dead_letter_store
        .append(&sample_dead_letter(
            "w1",
            "node-a",
            1002,
            2,
            MembershipRevocationAlertDeadLetterReason::CapacityEvicted,
        ))
        .expect("append capacity #1");
    dead_letter_store
        .append(&sample_dead_letter(
            "w1",
            "node-a",
            1003,
            2,
            MembershipRevocationAlertDeadLetterReason::CapacityEvicted,
        ))
        .expect("append capacity #2");

    let first = client
        .replay_revocation_dead_letters_with_policy(
            "w1",
            "node-a",
            &policy,
            &mut state,
            &recovery_store,
            &dead_letter_store,
        )
        .expect("first policy replay");
    assert_eq!(first, 1);

    let first_pending = recovery_store
        .load_pending("w1", "node-a")
        .expect("load pending after first replay");
    assert_eq!(first_pending.len(), 1);
    assert_eq!(
        first_pending[0].alert.detected_at_ms, 1000,
        "first replay still prioritizes retry-limit"
    );
    assert!(state.prefer_capacity_evicted);

    let second = client
        .replay_revocation_dead_letters_with_policy(
            "w1",
            "node-a",
            &policy,
            &mut state,
            &recovery_store,
            &dead_letter_store,
        )
        .expect("second policy replay");
    assert_eq!(second, 1);

    let second_pending = recovery_store
        .load_pending("w1", "node-a")
        .expect("load pending after second replay");
    assert_eq!(second_pending.len(), 2);
    assert_eq!(
        second_pending[1].alert.detected_at_ms, 1002,
        "second replay rotates into capacity-evicted queue"
    );
}

#[test]
fn run_replay_schedule_with_state_store_persists_interval_gate() {
    let root = temp_membership_dir("dead-letter-replay-state-store");
    fs::create_dir_all(&root).expect("create temp dir");

    let client = sample_client();
    let recovery_store = InMemoryMembershipRevocationAlertRecoveryStore::new();
    let dead_letter_store = InMemoryMembershipRevocationAlertDeadLetterStore::new();
    let replay_state_store =
        FileMembershipRevocationDeadLetterReplayStateStore::new(&root).expect("create state store");
    let replay_policy = MembershipRevocationDeadLetterReplayPolicy {
        max_replay_per_run: 1,
        max_retry_limit_exceeded_streak: 2,
    };

    dead_letter_store
        .append(&sample_dead_letter(
            "w1",
            "node-a",
            1000,
            1,
            MembershipRevocationAlertDeadLetterReason::RetryLimitExceeded,
        ))
        .expect("append dead letter #1");
    dead_letter_store
        .append(&sample_dead_letter(
            "w1",
            "node-a",
            1001,
            1,
            MembershipRevocationAlertDeadLetterReason::RetryLimitExceeded,
        ))
        .expect("append dead letter #2");

    let first = client
        .run_revocation_dead_letter_replay_schedule_with_state_store(
            "w1",
            "node-a",
            2000,
            100,
            &replay_policy,
            &recovery_store,
            &dead_letter_store,
            &replay_state_store,
        )
        .expect("first schedule run");
    assert_eq!(first, 1);

    let state_after_first = replay_state_store
        .load_state("w1", "node-a")
        .expect("load state after first");
    assert_eq!(state_after_first.last_replay_at_ms, Some(2000));

    let second = client
        .run_revocation_dead_letter_replay_schedule_with_state_store(
            "w1",
            "node-a",
            2050,
            100,
            &replay_policy,
            &recovery_store,
            &dead_letter_store,
            &replay_state_store,
        )
        .expect("second schedule run");
    assert_eq!(
        second, 0,
        "state store should gate run by persisted interval"
    );

    let third = client
        .run_revocation_dead_letter_replay_schedule_with_state_store(
            "w1",
            "node-a",
            2105,
            100,
            &replay_policy,
            &recovery_store,
            &dead_letter_store,
            &replay_state_store,
        )
        .expect("third schedule run");
    assert_eq!(third, 1);

    let pending = recovery_store
        .load_pending("w1", "node-a")
        .expect("load pending");
    assert_eq!(pending.len(), 2);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn run_replay_schedule_coordinated_with_state_store_respects_lease() {
    let client = sample_client();
    let recovery_store = InMemoryMembershipRevocationAlertRecoveryStore::new();
    let dead_letter_store = InMemoryMembershipRevocationAlertDeadLetterStore::new();
    let replay_state_store = InMemoryMembershipRevocationDeadLetterReplayStateStore::new();
    let coordinator_store: Arc<dyn MembershipRevocationCoordinatorStateStore + Send + Sync> =
        Arc::new(InMemoryMembershipRevocationCoordinatorStateStore::new());
    let coordinator_a =
        StoreBackedMembershipRevocationScheduleCoordinator::new(Arc::clone(&coordinator_store));
    let coordinator_b =
        StoreBackedMembershipRevocationScheduleCoordinator::new(Arc::clone(&coordinator_store));
    let replay_policy = MembershipRevocationDeadLetterReplayPolicy {
        max_replay_per_run: 2,
        max_retry_limit_exceeded_streak: 2,
    };

    dead_letter_store
        .append(&sample_dead_letter(
            "w1",
            "node-target",
            1000,
            1,
            MembershipRevocationAlertDeadLetterReason::RetryLimitExceeded,
        ))
        .expect("append dead letter");

    let coordination_world_id = "w1::revocation-dead-letter-replay::node-target";
    assert!(coordinator_a
        .acquire(coordination_world_id, "node-a", 1000, 500)
        .expect("acquire lease with node-a"));

    let blocked = client
        .run_revocation_dead_letter_replay_schedule_coordinated_with_state_store(
            "w1",
            "node-target",
            "node-b",
            1001,
            100,
            &replay_policy,
            &recovery_store,
            &dead_letter_store,
            &replay_state_store,
            &coordinator_b,
            500,
        )
        .expect("run coordinated replay while lease held");
    assert_eq!(blocked, 0);

    coordinator_a
        .release(coordination_world_id, "node-a")
        .expect("release node-a lease");

    let replayed = client
        .run_revocation_dead_letter_replay_schedule_coordinated_with_state_store(
            "w1",
            "node-target",
            "node-b",
            1105,
            100,
            &replay_policy,
            &recovery_store,
            &dead_letter_store,
            &replay_state_store,
            &coordinator_b,
            500,
        )
        .expect("run coordinated replay after lease released");
    assert_eq!(replayed, 1);

    let state = replay_state_store
        .load_state("w1", "node-target")
        .expect("load replay state");
    assert_eq!(state.last_replay_at_ms, Some(1105));
}

#[test]
fn run_replay_schedule_with_state_store_rejects_invalid_policy() {
    let client = sample_client();
    let recovery_store = InMemoryMembershipRevocationAlertRecoveryStore::new();
    let dead_letter_store = InMemoryMembershipRevocationAlertDeadLetterStore::new();
    let replay_state_store = InMemoryMembershipRevocationDeadLetterReplayStateStore::new();
    let invalid_policy = MembershipRevocationDeadLetterReplayPolicy {
        max_replay_per_run: 0,
        max_retry_limit_exceeded_streak: 1,
    };

    let error = client
        .run_revocation_dead_letter_replay_schedule_with_state_store(
            "w1",
            "node-a",
            1000,
            100,
            &invalid_policy,
            &recovery_store,
            &dead_letter_store,
            &replay_state_store,
        )
        .expect_err("invalid policy should fail");
    let reason = match error {
        WorldError::DistributedValidationFailed { reason } => reason,
        other => panic!("unexpected error: {other:?}"),
    };
    assert!(reason.contains("max_replay_per_run"));
}
