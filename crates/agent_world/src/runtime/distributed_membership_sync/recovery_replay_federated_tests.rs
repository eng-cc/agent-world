use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

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

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let now_ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "agent_world_{prefix}_{}_{}",
        std::process::id(),
        now_ns
    ));
    std::fs::create_dir_all(&path).expect("create temp directory for test");
    path
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

#[test]
fn governance_recovery_drill_alert_event_incremental_watermark_advances_monotonically() {
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
            1_040,
            MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::SuppressedCooldown,
        ),
    )
    .expect("publish node-a 1040");
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
    .expect("publish node-b 1080");

    let (batch1, watermark1) = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since_with_next_watermark(
            "w1",
            &["node-a".to_string(), "node-b".to_string()],
            1_000,
            &[],
            1,
            &event_bus,
        )
        .expect("first incremental batch");
    assert_eq!(batch1.len(), 1);
    assert_eq!(batch1[0].event_at_ms, 1_040);
    assert_eq!(watermark1, 1_040);

    let (batch2, watermark2) = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since_with_next_watermark(
            "w1",
            &["node-a".to_string(), "node-b".to_string()],
            watermark1,
            &[],
            10,
            &event_bus,
        )
        .expect("second incremental batch");
    assert_eq!(batch2.len(), 1);
    assert_eq!(batch2[0].event_at_ms, 1_080);
    assert_eq!(watermark2, 1_080);

    let (batch3, watermark3) = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since_with_next_watermark(
            "w1",
            &["node-a".to_string(), "node-b".to_string()],
            watermark2,
            &[],
            10,
            &event_bus,
        )
        .expect("third incremental batch");
    assert!(batch3.is_empty());
    assert_eq!(watermark3, 1_080);
}

#[test]
fn governance_recovery_drill_alert_event_incremental_cursor_handles_same_timestamp_nodes() {
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
    .expect("publish node-a 1000");
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::publish(
        &event_bus,
        "w1",
        "node-b",
        &sample_alert_event(
            "w1",
            "node-b",
            1_000,
            MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::SuppressedCooldown,
        ),
    )
    .expect("publish node-b 1000");
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::publish(
        &event_bus,
        "w1",
        "node-b",
        &sample_alert_event(
            "w1",
            "node-b",
            1_050,
            MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::Emitted,
        ),
    )
    .expect("publish node-b 1050");

    let (batch1, next_ts1, next_node1) = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since_cursor(
            "w1",
            &["node-a".to_string(), "node-b".to_string()],
            999,
            None,
            &[],
            1,
            &event_bus,
        )
        .expect("first cursor batch");
    assert_eq!(batch1.len(), 1);
    assert_eq!(batch1[0].event_at_ms, 1_000);
    assert_eq!(batch1[0].node_id, "node-a");
    assert_eq!(next_ts1, 1_000);
    assert_eq!(next_node1.as_deref(), Some("node-a"));

    let (batch2, next_ts2, next_node2) = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since_cursor(
            "w1",
            &["node-a".to_string(), "node-b".to_string()],
            next_ts1,
            next_node1.as_deref(),
            &[],
            10,
            &event_bus,
        )
        .expect("second cursor batch");
    assert_eq!(batch2.len(), 2);
    assert_eq!(batch2[0].event_at_ms, 1_000);
    assert_eq!(batch2[0].node_id, "node-b");
    assert_eq!(batch2[1].event_at_ms, 1_050);
    assert_eq!(batch2[1].node_id, "node-b");
    assert_eq!(next_ts2, 1_050);
    assert_eq!(next_node2.as_deref(), Some("node-b"));

    let (batch3, next_ts3, next_node3) = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since_cursor(
            "w1",
            &["node-a".to_string(), "node-b".to_string()],
            next_ts2,
            next_node2.as_deref(),
            &[],
            10,
            &event_bus,
        )
        .expect("third cursor batch");
    assert!(batch3.is_empty());
    assert_eq!(next_ts3, 1_050);
    assert_eq!(next_node3.as_deref(), Some("node-b"));
}

#[test]
fn governance_recovery_drill_alert_event_incremental_composite_sequence_cursor_handles_same_node_same_timestamp(
) {
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
    .expect("publish node-a 1000 emitted");
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::publish(
        &event_bus,
        "w1",
        "node-a",
        &sample_alert_event(
            "w1",
            "node-a",
            1_000,
            MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::SuppressedCooldown,
        ),
    )
    .expect("publish node-a 1000 cooldown");
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::publish(
        &event_bus,
        "w1",
        "node-b",
        &sample_alert_event(
            "w1",
            "node-b",
            1_000,
            MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::SuppressedNoAnomaly,
        ),
    )
    .expect("publish node-b 1000");
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::publish(
        &event_bus,
        "w1",
        "node-b",
        &sample_alert_event(
            "w1",
            "node-b",
            1_010,
            MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::Emitted,
        ),
    )
    .expect("publish node-b 1010");

    let (batch1, next_ts1, next_node1, next_offset1) = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since_composite_sequence_cursor(
            "w1",
            &["node-a".to_string(), "node-b".to_string()],
            999,
            None,
            0,
            &[],
            1,
            &event_bus,
        )
        .expect("first composite sequence cursor batch");
    assert_eq!(batch1.len(), 1);
    assert_eq!(batch1[0].event_at_ms, 1_000);
    assert_eq!(batch1[0].node_id, "node-a");
    assert_eq!(batch1[0].outcome, MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::Emitted);
    assert_eq!(next_ts1, 1_000);
    assert_eq!(next_node1.as_deref(), Some("node-a"));
    assert_eq!(next_offset1, 0);

    let (batch2, next_ts2, next_node2, next_offset2) = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since_composite_sequence_cursor(
            "w1",
            &["node-a".to_string(), "node-b".to_string()],
            next_ts1,
            next_node1.as_deref(),
            next_offset1,
            &[],
            1,
            &event_bus,
        )
        .expect("second composite sequence cursor batch");
    assert_eq!(batch2.len(), 1);
    assert_eq!(batch2[0].event_at_ms, 1_000);
    assert_eq!(batch2[0].node_id, "node-a");
    assert_eq!(batch2[0].outcome, MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::SuppressedCooldown);
    assert_eq!(next_ts2, 1_000);
    assert_eq!(next_node2.as_deref(), Some("node-a"));
    assert_eq!(next_offset2, 1);

    let (batch3, next_ts3, next_node3, next_offset3) = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since_composite_sequence_cursor(
            "w1",
            &["node-a".to_string(), "node-b".to_string()],
            next_ts2,
            next_node2.as_deref(),
            next_offset2,
            &[],
            10,
            &event_bus,
        )
        .expect("third composite sequence cursor batch");
    assert_eq!(batch3.len(), 2);
    assert_eq!(batch3[0].event_at_ms, 1_000);
    assert_eq!(batch3[0].node_id, "node-b");
    assert_eq!(batch3[0].outcome, MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::SuppressedNoAnomaly);
    assert_eq!(batch3[1].event_at_ms, 1_010);
    assert_eq!(batch3[1].node_id, "node-b");
    assert_eq!(next_ts3, 1_010);
    assert_eq!(next_node3.as_deref(), Some("node-b"));
    assert_eq!(next_offset3, 1);

    let (batch4, next_ts4, next_node4, next_offset4) = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_since_composite_sequence_cursor(
            "w1",
            &["node-a".to_string(), "node-b".to_string()],
            next_ts3,
            next_node3.as_deref(),
            next_offset3,
            &[],
            10,
            &event_bus,
        )
        .expect("fourth composite sequence cursor batch");
    assert!(batch4.is_empty());
    assert_eq!(next_ts4, 1_010);
    assert_eq!(next_node4.as_deref(), Some("node-b"));
    assert_eq!(next_offset4, 1);
}

#[test]
fn governance_recovery_drill_alert_event_composite_sequence_cursor_state_store_file_round_trip() {
    let root_dir = unique_temp_dir("drill_alert_composite_sequence_cursor_state");
    let store =
        FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventCompositeSequenceCursorStateStore::new(
            root_dir.clone(),
        )
        .expect("create file state store");

    let initial = store.load("w1", "consumer-a").expect("load initial state");
    assert!(initial.is_none());

    let expected =
        MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventCompositeSequenceCursorState {
            world_id: "w1".to_string(),
            consumer_id: "consumer-a".to_string(),
            since_event_at_ms: 1_010,
            since_node_id: Some("node-b".to_string()),
            since_node_event_offset: 3,
        };
    store.save(&expected).expect("save cursor state");

    let loaded = store
        .load("w1", "consumer-a")
        .expect("load saved state")
        .expect("saved state should exist");
    assert_eq!(loaded, expected);

    std::fs::remove_dir_all(root_dir).expect("remove temp directory");
}

#[test]
fn governance_recovery_drill_alert_event_incremental_composite_sequence_cursor_stateful_query_continues_from_store(
) {
    let client = sample_client();
    let event_bus =
        InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::new();
    let state_store =
        InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventCompositeSequenceCursorStateStore::new();
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
    .expect("publish node-a 1000 emitted");
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::publish(
        &event_bus,
        "w1",
        "node-a",
        &sample_alert_event(
            "w1",
            "node-a",
            1_000,
            MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::SuppressedCooldown,
        ),
    )
    .expect("publish node-a 1000 cooldown");
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::publish(
        &event_bus,
        "w1",
        "node-b",
        &sample_alert_event(
            "w1",
            "node-b",
            1_000,
            MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::SuppressedNoAnomaly,
        ),
    )
    .expect("publish node-b 1000");
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus::publish(
        &event_bus,
        "w1",
        "node-b",
        &sample_alert_event(
            "w1",
            "node-b",
            1_010,
            MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::Emitted,
        ),
    )
    .expect("publish node-b 1010");

    let (batch1, state1) = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_with_composite_sequence_cursor_state(
            "w1",
            "consumer-a",
            &["node-a".to_string(), "node-b".to_string()],
            999,
            None,
            0,
            &[],
            1,
            &event_bus,
            &state_store,
        )
        .expect("first stateful query");
    assert_eq!(batch1.len(), 1);
    assert_eq!(batch1[0].node_id, "node-a");
    assert_eq!(batch1[0].outcome, MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::Emitted);
    assert_eq!(state1.since_event_at_ms, 1_000);
    assert_eq!(state1.since_node_id.as_deref(), Some("node-a"));
    assert_eq!(state1.since_node_event_offset, 0);

    let (batch2, state2) = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_with_composite_sequence_cursor_state(
            "w1",
            "consumer-a",
            &["node-a".to_string(), "node-b".to_string()],
            0,
            None,
            0,
            &[],
            1,
            &event_bus,
            &state_store,
        )
        .expect("second stateful query");
    assert_eq!(batch2.len(), 1);
    assert_eq!(batch2[0].node_id, "node-a");
    assert_eq!(batch2[0].outcome, MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::SuppressedCooldown);
    assert_eq!(state2.since_event_at_ms, 1_000);
    assert_eq!(state2.since_node_id.as_deref(), Some("node-a"));
    assert_eq!(state2.since_node_event_offset, 1);

    let (batch3, state3) = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_with_composite_sequence_cursor_state(
            "w1",
            "consumer-a",
            &["node-a".to_string(), "node-b".to_string()],
            0,
            None,
            0,
            &[],
            10,
            &event_bus,
            &state_store,
        )
        .expect("third stateful query");
    assert_eq!(batch3.len(), 2);
    assert_eq!(batch3[0].node_id, "node-b");
    assert_eq!(batch3[0].event_at_ms, 1_000);
    assert_eq!(batch3[1].node_id, "node-b");
    assert_eq!(batch3[1].event_at_ms, 1_010);
    assert_eq!(state3.since_event_at_ms, 1_010);
    assert_eq!(state3.since_node_id.as_deref(), Some("node-b"));
    assert_eq!(state3.since_node_event_offset, 1);

    let persisted = state_store
        .load("w1", "consumer-a")
        .expect("load persisted state")
        .expect("persisted state should exist");
    assert_eq!(persisted, state3);

    let (batch4, state4) = client
        .query_revocation_dead_letter_replay_rollback_governance_recovery_drill_alert_events_incremental_with_composite_sequence_cursor_state(
            "w1",
            "consumer-a",
            &["node-a".to_string(), "node-b".to_string()],
            0,
            None,
            0,
            &[],
            10,
            &event_bus,
            &state_store,
        )
        .expect("fourth stateful query");
    assert!(batch4.is_empty());
    assert_eq!(state4, state3);
}

#[test]
fn governance_recovery_drill_alert_event_composite_sequence_cursor_state_store_rejects_rollback() {
    let store =
        InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventCompositeSequenceCursorStateStore::new();
    let baseline =
        MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventCompositeSequenceCursorState {
            world_id: "w1".to_string(),
            consumer_id: "consumer-a".to_string(),
            since_event_at_ms: 1_000,
            since_node_id: Some("node-b".to_string()),
            since_node_event_offset: 1,
        };
    store.save(&baseline).expect("save baseline cursor");
    store
        .save(&baseline)
        .expect("save equal cursor is idempotent");

    let forward =
        MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventCompositeSequenceCursorState {
            world_id: "w1".to_string(),
            consumer_id: "consumer-a".to_string(),
            since_event_at_ms: 1_000,
            since_node_id: Some("node-b".to_string()),
            since_node_event_offset: 2,
        };
    store.save(&forward).expect("save forward cursor");

    let rollback =
        MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventCompositeSequenceCursorState {
            world_id: "w1".to_string(),
            consumer_id: "consumer-a".to_string(),
            since_event_at_ms: 999,
            since_node_id: Some("node-a".to_string()),
            since_node_event_offset: 0,
        };
    let error = store
        .save(&rollback)
        .expect_err("rollback cursor should be rejected");
    assert!(format!("{error:?}").contains("cannot rollback"));
}

#[test]
fn governance_recovery_drill_alert_event_composite_sequence_cursor_file_state_store_rejects_rollback(
) {
    let root_dir = unique_temp_dir("drill_alert_composite_sequence_cursor_state_rollback");
    let store =
        FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventCompositeSequenceCursorStateStore::new(
            root_dir.clone(),
        )
        .expect("create file state store");
    let baseline =
        MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventCompositeSequenceCursorState {
            world_id: "w1".to_string(),
            consumer_id: "consumer-a".to_string(),
            since_event_at_ms: 1_000,
            since_node_id: Some("node-b".to_string()),
            since_node_event_offset: 1,
        };
    store.save(&baseline).expect("save baseline cursor");

    let rollback =
        MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventCompositeSequenceCursorState {
            world_id: "w1".to_string(),
            consumer_id: "consumer-a".to_string(),
            since_event_at_ms: 900,
            since_node_id: Some("node-a".to_string()),
            since_node_event_offset: 0,
        };
    let error = store
        .save(&rollback)
        .expect_err("rollback cursor should be rejected");
    assert!(format!("{error:?}").contains("cannot rollback"));

    let loaded = store
        .load("w1", "consumer-a")
        .expect("load stored state")
        .expect("stored state should exist");
    assert_eq!(loaded, baseline);
    std::fs::remove_dir_all(root_dir).expect("remove temp directory");
}
