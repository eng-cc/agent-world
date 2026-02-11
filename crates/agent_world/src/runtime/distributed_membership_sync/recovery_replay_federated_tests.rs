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

#[test]
fn governance_recovery_drill_alert_event_incremental_since_returns_strictly_newer_records() {
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
    .expect("publish node-a baseline");
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::publish(
        &event_bus,
        "w1",
        "node-a",
        &sample_alert_event(
            "w1",
            "node-a",
            1_040,
            MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::SuppressedCooldown,
        ),
    )
    .expect("publish node-a incremental");
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::publish(
        &event_bus,
        "w1",
        "node-b",
        &sample_alert_event(
            "w1",
            "node-b",
            1_080,
            MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::Emitted,
        ),
    )
    .expect("publish node-b incremental");

    let incremental = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since(
            "w1",
            &["node-a".to_string(), "node-b".to_string()],
            1_000,
            &[],
            10,
            &event_bus,
        )
        .expect("query incremental events");
    assert_eq!(incremental.len(), 2);
    assert_eq!(incremental[0].event_at_ms, 1_040);
    assert_eq!(incremental[1].event_at_ms, 1_080);

    let emitted_only = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since(
            "w1",
            &["node-a".to_string(), "node-b".to_string()],
            1_000,
            &[MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::Emitted],
            10,
            &event_bus,
        )
        .expect("query emitted incremental events");
    assert_eq!(emitted_only.len(), 1);
    assert_eq!(emitted_only[0].node_id, "node-b");
    assert_eq!(emitted_only[0].event_at_ms, 1_080);

    let limited = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since(
            "w1",
            &["node-a".to_string(), "node-b".to_string()],
            1_000,
            &[],
            1,
            &event_bus,
        )
        .expect("query incremental events with limit");
    assert_eq!(limited.len(), 1);
    assert_eq!(limited[0].event_at_ms, 1_040);
}

#[test]
fn governance_recovery_drill_alert_event_summary_aggregates_outcome_counts() {
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
    .expect("publish emitted");
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::publish(
        &event_bus,
        "w1",
        "node-a",
        &sample_alert_event(
            "w1",
            "node-a",
            1_010,
            MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::Emitted,
        ),
    )
    .expect("publish emitted 2");
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::publish(
        &event_bus,
        "w1",
        "node-b",
        &sample_alert_event(
            "w1",
            "node-b",
            1_020,
            MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::SuppressedNoAnomaly,
        ),
    )
    .expect("publish suppressed");
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::publish(
        &event_bus,
        "w1",
        "node-b",
        &sample_alert_event(
            "w1",
            "node-b",
            900,
            MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::SkippedNoDrill,
        ),
    )
    .expect("publish skipped");

    let summary = client
        .summarize_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_aggregated_by_outcome(
            "w1",
            &["node-a".to_string(), "node-b".to_string()],
            Some(950),
            &event_bus,
        )
        .expect("summarize outcomes");
    assert_eq!(summary.get("emitted"), Some(&2));
    assert_eq!(summary.get("suppressed_no_anomaly"), Some(&1));
    assert_eq!(summary.get("skipped_no_drill"), None);
}
