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

#[test]
fn evaluate_revocation_reconcile_alerts_reports_diverged_and_rejected() {
    let client = sample_client();
    let report = MembershipRevocationReconcileReport {
        drained: 4,
        in_sync: 1,
        diverged: 2,
        merged: 1,
        rejected: 1,
    };
    let policy = MembershipRevocationAlertPolicy {
        warn_diverged_threshold: 2,
        critical_rejected_threshold: 1,
    };

    let alerts = client
        .evaluate_revocation_reconcile_alerts("w1", "node-a", 1000, &report, &policy)
        .expect("evaluate alerts");

    assert_eq!(alerts.len(), 2);
    assert_eq!(
        alerts[0].severity,
        MembershipRevocationAlertSeverity::Critical
    );
    assert_eq!(alerts[0].code, "reconcile_rejected");
    assert_eq!(alerts[1].severity, MembershipRevocationAlertSeverity::Warn);
    assert_eq!(alerts[1].code, "reconcile_diverged");
}

#[test]
fn run_revocation_reconcile_schedule_runs_on_first_tick() {
    let client = sample_client();
    let subscription = client.subscribe("w1").expect("subscribe");
    let mut keyring = sample_keyring();

    let reconcile_policy = MembershipRevocationReconcilePolicy {
        trusted_nodes: vec!["node-b".to_string()],
        auto_revoke_missing_keys: false,
    };
    let schedule_policy = MembershipRevocationReconcileSchedulePolicy {
        checkpoint_interval_ms: 300,
        reconcile_interval_ms: 300,
    };
    let mut schedule_state = MembershipRevocationReconcileScheduleState::default();

    let run_report = client
        .run_revocation_reconcile_schedule(
            "w1",
            "node-b",
            1000,
            &subscription,
            &mut keyring,
            &reconcile_policy,
            &schedule_policy,
            &mut schedule_state,
        )
        .expect("run schedule");

    assert!(run_report.checkpoint_published);
    assert!(run_report.reconcile_executed);
    let reconcile_report = run_report.reconcile_report.expect("reconcile report");
    assert_eq!(reconcile_report.drained, 1);
    assert_eq!(reconcile_report.in_sync, 1);
    assert_eq!(schedule_state.last_checkpoint_at_ms, Some(1000));
    assert_eq!(schedule_state.last_reconcile_at_ms, Some(1000));
}

#[test]
fn run_revocation_reconcile_schedule_skips_when_not_due() {
    let client = sample_client();
    let subscription = client.subscribe("w1").expect("subscribe");
    let mut keyring = sample_keyring();

    let reconcile_policy = MembershipRevocationReconcilePolicy {
        trusted_nodes: Vec::new(),
        auto_revoke_missing_keys: true,
    };
    let schedule_policy = MembershipRevocationReconcileSchedulePolicy {
        checkpoint_interval_ms: 300,
        reconcile_interval_ms: 300,
    };
    let mut schedule_state = MembershipRevocationReconcileScheduleState {
        last_checkpoint_at_ms: Some(1000),
        last_reconcile_at_ms: Some(1000),
    };

    let run_report = client
        .run_revocation_reconcile_schedule(
            "w1",
            "node-b",
            1200,
            &subscription,
            &mut keyring,
            &reconcile_policy,
            &schedule_policy,
            &mut schedule_state,
        )
        .expect("run schedule");

    assert!(!run_report.checkpoint_published);
    assert!(!run_report.reconcile_executed);
    assert!(run_report.reconcile_report.is_none());
    assert_eq!(schedule_state.last_checkpoint_at_ms, Some(1000));
    assert_eq!(schedule_state.last_reconcile_at_ms, Some(1000));
}

#[test]
fn run_revocation_reconcile_schedule_reconciles_due_and_merges() {
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
        auto_revoke_missing_keys: true,
    };
    let schedule_policy = MembershipRevocationReconcileSchedulePolicy {
        checkpoint_interval_ms: 1000,
        reconcile_interval_ms: 300,
    };
    let mut schedule_state = MembershipRevocationReconcileScheduleState {
        last_checkpoint_at_ms: Some(1300),
        last_reconcile_at_ms: Some(1000),
    };

    let run_report = client
        .run_revocation_reconcile_schedule(
            "w1",
            "node-b",
            1305,
            &subscription,
            &mut local_keyring,
            &reconcile_policy,
            &schedule_policy,
            &mut schedule_state,
        )
        .expect("run schedule");

    assert!(!run_report.checkpoint_published);
    assert!(run_report.reconcile_executed);

    let reconcile_report = run_report.reconcile_report.expect("reconcile report");
    assert_eq!(reconcile_report.drained, 1);
    assert_eq!(reconcile_report.diverged, 1);
    assert_eq!(reconcile_report.merged, 1);
    assert!(local_keyring.is_key_revoked("k2"));
    assert_eq!(schedule_state.last_checkpoint_at_ms, Some(1300));
    assert_eq!(schedule_state.last_reconcile_at_ms, Some(1305));
}

#[test]
fn run_revocation_reconcile_schedule_rejects_invalid_policy() {
    let client = sample_client();
    let subscription = client.subscribe("w1").expect("subscribe");
    let mut keyring = sample_keyring();

    let reconcile_policy = MembershipRevocationReconcilePolicy {
        trusted_nodes: Vec::new(),
        auto_revoke_missing_keys: true,
    };
    let schedule_policy = MembershipRevocationReconcileSchedulePolicy {
        checkpoint_interval_ms: 0,
        reconcile_interval_ms: 300,
    };
    let mut schedule_state = MembershipRevocationReconcileScheduleState::default();

    let err = client
        .run_revocation_reconcile_schedule(
            "w1",
            "node-b",
            1000,
            &subscription,
            &mut keyring,
            &reconcile_policy,
            &schedule_policy,
            &mut schedule_state,
        )
        .expect_err("invalid policy should fail");

    assert!(matches!(
        err,
        WorldError::DistributedValidationFailed { .. }
    ));
}
