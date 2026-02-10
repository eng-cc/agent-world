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

fn sample_keyring() -> MembershipDirectorySignerKeyring {
    let mut keyring = MembershipDirectorySignerKeyring::new();
    keyring
        .add_hmac_sha256_key("k1", "membership-secret-v1")
        .expect("add k1");
    keyring.set_active_key("k1").expect("set active k1");
    keyring
}

fn temp_membership_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("agent_world-{prefix}-{nanos}"))
}

fn sample_alert(world_id: &str, node_id: &str) -> MembershipRevocationAnomalyAlert {
    MembershipRevocationAnomalyAlert {
        world_id: world_id.to_string(),
        node_id: node_id.to_string(),
        detected_at_ms: 1000,
        severity: MembershipRevocationAlertSeverity::Warn,
        code: "reconcile_diverged".to_string(),
        message: "membership revocation reconcile diverged".to_string(),
        drained: 1,
        diverged: 1,
        rejected: 0,
    }
}

#[test]
fn in_memory_alert_sink_emits_and_lists() {
    let sink = InMemoryMembershipRevocationAlertSink::new();
    let alert = sample_alert("w1", "node-a");

    sink.emit(&alert).expect("emit alert");

    let alerts = sink.list().expect("list alerts");
    assert_eq!(alerts, vec![alert]);
}

#[test]
fn file_alert_sink_appends_and_lists_by_world() {
    let root = temp_membership_dir("revocation-alert-sink");
    fs::create_dir_all(&root).expect("create temp dir");

    let sink = FileMembershipRevocationAlertSink::new(&root).expect("create sink");
    let alert_w1 = sample_alert("w1", "node-a");
    let alert_w2 = sample_alert("w2", "node-b");

    sink.emit(&alert_w1).expect("emit w1 alert");
    sink.emit(&alert_w2).expect("emit w2 alert");

    let w1_alerts = sink.list("w1").expect("list w1 alerts");
    let w2_alerts = sink.list("w2").expect("list w2 alerts");
    assert_eq!(w1_alerts, vec![alert_w1]);
    assert_eq!(w2_alerts, vec![alert_w2]);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn in_memory_schedule_state_store_round_trip() {
    let store = InMemoryMembershipRevocationScheduleStateStore::new();
    let state = MembershipRevocationReconcileScheduleState {
        last_checkpoint_at_ms: Some(1100),
        last_reconcile_at_ms: Some(1200),
    };

    store
        .save("w1", "node-a", &state)
        .expect("save schedule state");

    let loaded = store.load("w1", "node-a").expect("load schedule state");
    assert_eq!(loaded, state);

    let missing = store.load("w1", "node-b").expect("load missing state");
    assert_eq!(
        missing,
        MembershipRevocationReconcileScheduleState::default()
    );
}

#[test]
fn file_schedule_state_store_round_trip() {
    let root = temp_membership_dir("revocation-schedule-store");
    fs::create_dir_all(&root).expect("create temp dir");

    let store = FileMembershipRevocationScheduleStateStore::new(&root).expect("create store");
    let state = MembershipRevocationReconcileScheduleState {
        last_checkpoint_at_ms: Some(2100),
        last_reconcile_at_ms: Some(2200),
    };

    store
        .save("w1", "node-a", &state)
        .expect("save schedule state");

    let loaded = store.load("w1", "node-a").expect("load schedule state");
    assert_eq!(loaded, state);

    let missing = store.load("w1", "node-b").expect("load missing state");
    assert_eq!(
        missing,
        MembershipRevocationReconcileScheduleState::default()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn run_schedule_with_store_and_alerts_persists_state_and_emits_alerts() {
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

    client
        .publish_revocation_checkpoint("w1", "node-a", 1300, &remote_keyring)
        .expect("publish remote checkpoint");

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

    let schedule_store = InMemoryMembershipRevocationScheduleStateStore::new();
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

    let alert_sink = InMemoryMembershipRevocationAlertSink::new();

    let run_report = client
        .run_revocation_reconcile_schedule_with_store_and_alerts(
            "w1",
            "node-b",
            1305,
            &subscription,
            &mut local_keyring,
            &reconcile_policy,
            &schedule_policy,
            &alert_policy,
            &schedule_store,
            &alert_sink,
        )
        .expect("run schedule with store and alerts");

    assert!(!run_report.checkpoint_published);
    assert!(run_report.reconcile_executed);
    let reconcile_report = run_report.reconcile_report.expect("reconcile report");
    assert_eq!(reconcile_report.drained, 1);
    assert_eq!(reconcile_report.diverged, 1);
    assert_eq!(reconcile_report.rejected, 0);

    let state = schedule_store
        .load("w1", "node-b")
        .expect("load persisted schedule state");
    assert_eq!(state.last_checkpoint_at_ms, Some(1300));
    assert_eq!(state.last_reconcile_at_ms, Some(1305));

    let alerts = alert_sink.list().expect("list emitted alerts");
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].code, "reconcile_diverged");
    assert_eq!(alerts[0].severity, MembershipRevocationAlertSeverity::Warn);
}
