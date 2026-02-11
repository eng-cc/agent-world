use std::sync::Arc;

use super::super::distributed_net::{DistributedNetwork, InMemoryNetwork};
use super::*;

fn sample_client() -> MembershipSyncClient {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    MembershipSyncClient::new(network)
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
