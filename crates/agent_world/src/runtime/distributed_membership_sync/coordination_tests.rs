use std::sync::Arc;

use super::super::distributed_net::{DistributedNetwork, InMemoryNetwork};
use super::*;

fn sample_client() -> MembershipSyncClient {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    MembershipSyncClient::new(network)
}

fn sample_keyring() -> MembershipDirectorySignerKeyring {
    let mut keyring = MembershipDirectorySignerKeyring::new();
    keyring
        .add_hmac_sha256_key("k1", "membership-secret-v1")
        .expect("add k1");
    keyring.set_active_key("k1").expect("set active k1");
    keyring
}

fn sample_alert(world_id: &str, node_id: &str, code: &str) -> MembershipRevocationAnomalyAlert {
    MembershipRevocationAnomalyAlert {
        world_id: world_id.to_string(),
        node_id: node_id.to_string(),
        detected_at_ms: 1000,
        severity: MembershipRevocationAlertSeverity::Warn,
        code: code.to_string(),
        message: "sample".to_string(),
        drained: 1,
        diverged: 1,
        rejected: 0,
    }
}

#[test]
fn deduplicate_revocation_alerts_suppresses_within_window() {
    let client = sample_client();
    let policy = MembershipRevocationAlertDedupPolicy {
        suppress_window_ms: 300,
    };
    let mut state = MembershipRevocationAlertDedupState::default();

    let alerts = vec![
        sample_alert("w1", "node-a", "reconcile_diverged"),
        sample_alert("w1", "node-a", "reconcile_diverged"),
        sample_alert("w1", "node-a", "reconcile_rejected"),
    ];

    let first = client
        .deduplicate_revocation_alerts(alerts.clone(), 1000, &policy, &mut state)
        .expect("first dedup");
    assert_eq!(first.len(), 2);

    let second = client
        .deduplicate_revocation_alerts(alerts.clone(), 1100, &policy, &mut state)
        .expect("second dedup");
    assert_eq!(second.len(), 0);

    let third = client
        .deduplicate_revocation_alerts(alerts, 1400, &policy, &mut state)
        .expect("third dedup");
    assert_eq!(third.len(), 2);
}

#[test]
fn in_memory_schedule_coordinator_blocks_until_expired_or_released() {
    let coordinator = InMemoryMembershipRevocationScheduleCoordinator::new();

    assert!(coordinator
        .acquire("w1", "node-a", 1000, 500)
        .expect("acquire node-a"));
    assert!(!coordinator
        .acquire("w1", "node-b", 1200, 500)
        .expect("acquire node-b while held"));
    assert!(coordinator
        .acquire("w1", "node-b", 1601, 500)
        .expect("acquire node-b after expiry"));

    coordinator.release("w1", "node-b").expect("release node-b");
    assert!(coordinator
        .acquire("w1", "node-a", 1602, 500)
        .expect("acquire node-a after release"));
}

#[test]
fn run_revocation_reconcile_coordinated_reports_not_acquired_when_locked() {
    let client = sample_client();
    let subscription = client.subscribe("w1").expect("subscribe");
    let mut keyring = sample_keyring();

    let reconcile_policy = MembershipRevocationReconcilePolicy {
        trusted_nodes: Vec::new(),
        auto_revoke_missing_keys: false,
    };
    let schedule_policy = MembershipRevocationReconcileSchedulePolicy {
        checkpoint_interval_ms: 300,
        reconcile_interval_ms: 300,
    };
    let alert_policy = MembershipRevocationAlertPolicy::default();

    let schedule_store = InMemoryMembershipRevocationScheduleStateStore::new();
    let alert_sink = InMemoryMembershipRevocationAlertSink::new();
    let coordinator = InMemoryMembershipRevocationScheduleCoordinator::new();

    assert!(coordinator
        .acquire("w1", "node-a", 1000, 1000)
        .expect("seed lock"));

    let report = client
        .run_revocation_reconcile_coordinated(
            "w1",
            "node-b",
            1100,
            &subscription,
            &mut keyring,
            &reconcile_policy,
            &schedule_policy,
            &alert_policy,
            None,
            None,
            &schedule_store,
            &alert_sink,
            &coordinator,
            1000,
        )
        .expect("run coordinated");

    assert!(!report.acquired);
    assert_eq!(report.emitted_alerts, 0);
    assert!(report.run_report.is_none());

    coordinator
        .release("w1", "node-a")
        .expect("release seed lock");
}

#[test]
fn run_revocation_reconcile_coordinated_applies_dedup_policy() {
    let client = sample_client();
    let subscription = client.subscribe("w1").expect("subscribe");

    let mut local_keyring = sample_keyring();
    local_keyring
        .add_hmac_sha256_key("k2", "membership-secret-v2")
        .expect("add local k2");

    let mut remote_keyring = MembershipDirectorySignerKeyring::new();
    remote_keyring
        .add_hmac_sha256_key("k2", "membership-secret-v2")
        .expect("add remote k2");
    assert!(remote_keyring.revoke_key("k2").expect("revoke remote k2"));

    let reconcile_policy = MembershipRevocationReconcilePolicy {
        trusted_nodes: vec!["node-a".to_string()],
        auto_revoke_missing_keys: false,
    };
    let schedule_policy = MembershipRevocationReconcileSchedulePolicy {
        checkpoint_interval_ms: 1000,
        reconcile_interval_ms: 300,
    };
    let alert_policy = MembershipRevocationAlertPolicy {
        warn_diverged_threshold: 1,
        critical_rejected_threshold: 1,
    };
    let dedup_policy = MembershipRevocationAlertDedupPolicy {
        suppress_window_ms: 500,
    };
    let mut dedup_state = MembershipRevocationAlertDedupState::default();

    let schedule_store = InMemoryMembershipRevocationScheduleStateStore::new();
    let alert_sink = InMemoryMembershipRevocationAlertSink::new();
    let coordinator = InMemoryMembershipRevocationScheduleCoordinator::new();

    schedule_store
        .save(
            "w1",
            "node-b",
            &MembershipRevocationReconcileScheduleState {
                last_checkpoint_at_ms: Some(1300),
                last_reconcile_at_ms: Some(1000),
            },
        )
        .expect("seed schedule state");

    client
        .publish_revocation_checkpoint("w1", "node-a", 1300, &remote_keyring)
        .expect("publish first checkpoint");
    let first = client
        .run_revocation_reconcile_coordinated(
            "w1",
            "node-b",
            1305,
            &subscription,
            &mut local_keyring,
            &reconcile_policy,
            &schedule_policy,
            &alert_policy,
            Some(&dedup_policy),
            Some(&mut dedup_state),
            &schedule_store,
            &alert_sink,
            &coordinator,
            1000,
        )
        .expect("first coordinated run");
    assert!(first.acquired);
    assert_eq!(first.emitted_alerts, 1);

    schedule_store
        .save(
            "w1",
            "node-b",
            &MembershipRevocationReconcileScheduleState {
                last_checkpoint_at_ms: Some(1300),
                last_reconcile_at_ms: Some(1000),
            },
        )
        .expect("reset schedule state");
    client
        .publish_revocation_checkpoint("w1", "node-a", 1310, &remote_keyring)
        .expect("publish second checkpoint");
    let second = client
        .run_revocation_reconcile_coordinated(
            "w1",
            "node-b",
            1315,
            &subscription,
            &mut local_keyring,
            &reconcile_policy,
            &schedule_policy,
            &alert_policy,
            Some(&dedup_policy),
            Some(&mut dedup_state),
            &schedule_store,
            &alert_sink,
            &coordinator,
            1000,
        )
        .expect("second coordinated run");

    assert!(second.acquired);
    assert_eq!(second.emitted_alerts, 0);

    let alerts = alert_sink.list().expect("list alerts");
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].code, "reconcile_diverged");
}
