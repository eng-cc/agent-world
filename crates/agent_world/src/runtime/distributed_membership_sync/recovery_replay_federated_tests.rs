use std::sync::Arc;

use super::super::distributed_net::{DistributedNetwork, InMemoryNetwork};
use super::*;

fn sample_client() -> MembershipSyncClient {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    MembershipSyncClient::new(network)
}

fn sample_alert_event(
    world_id: &str,
    node_id: &str,
    event_at_ms: i64,
    outcome: MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome,
) -> MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent {
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent {
        world_id: world_id.to_string(),
        node_id: node_id.to_string(),
        event_at_ms,
        outcome,
        reasons: vec!["rollback_streak_exceeded".to_string()],
        severity: Some(MembershipRevocationAlertSeverity::Warn),
    }
}

#[test]
fn governance_recovery_drill_alert_event_aggregate_query_filters_orders_and_pages() {
    let client = sample_client();
    let event_bus =
        InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::new();
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::publish(
        &event_bus,
        "w1",
        "node-a",
        &sample_alert_event(
            "w1",
            "node-a",
            1_000,
            MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::Emitted,
        ),
    )
    .expect("publish node-a emitted");
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::publish(
        &event_bus,
        "w1",
        "node-a",
        &sample_alert_event(
            "w1",
            "node-a",
            900,
            MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::SuppressedCooldown,
        ),
    )
    .expect("publish node-a cooldown");
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::publish(
        &event_bus,
        "w1",
        "node-b",
        &sample_alert_event(
            "w1",
            "node-b",
            1_100,
            MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::Emitted,
        ),
    )
    .expect("publish node-b emitted");
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::publish(
        &event_bus,
        "w1",
        "node-b",
        &sample_alert_event(
            "w1",
            "node-b",
            950,
            MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::SkippedNoDrill,
        ),
    )
    .expect("publish node-b skipped");

    let queried = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_aggregated(
            "w1",
            &["node-a".to_string(), "node-b".to_string()],
            Some(950),
            &[MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::Emitted],
            0,
            10,
            &event_bus,
        )
        .expect("query emitted events");
    assert_eq!(queried.len(), 2);
    assert_eq!(queried[0].node_id, "node-b");
    assert_eq!(queried[0].event_at_ms, 1_100);
    assert_eq!(queried[1].node_id, "node-a");
    assert_eq!(queried[1].event_at_ms, 1_000);

    let paged = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_aggregated(
            "w1",
            &["node-a".to_string(), "node-b".to_string()],
            Some(950),
            &[MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::Emitted],
            1,
            1,
            &event_bus,
        )
        .expect("query paged events");
    assert_eq!(paged.len(), 1);
    assert_eq!(paged[0].node_id, "node-a");
    assert_eq!(paged[0].event_at_ms, 1_000);
}

#[test]
fn governance_recovery_drill_alert_event_aggregate_query_rejects_invalid_args() {
    let client = sample_client();
    let event_bus =
        InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::new();
    let error = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_aggregated(
            "w1",
            &["node-a".to_string()],
            None,
            &[],
            0,
            0,
            &event_bus,
        )
        .expect_err("max_records=0 should be rejected");
    assert!(format!("{error:?}").contains("max_records must be positive"));

    let error = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_aggregated(
            "w1",
            &[],
            None,
            &[],
            0,
            1,
            &event_bus,
        )
        .expect_err("empty nodes should be rejected");
    assert!(format!("{error:?}").contains("requires at least one node_id"));
}
