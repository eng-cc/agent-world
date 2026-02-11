use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
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

fn sample_alert(
    world_id: &str,
    node_id: &str,
    detected_at_ms: i64,
) -> MembershipRevocationAnomalyAlert {
    MembershipRevocationAnomalyAlert {
        world_id: world_id.to_string(),
        node_id: node_id.to_string(),
        detected_at_ms,
        severity: MembershipRevocationAlertSeverity::Warn,
        code: "reconcile_diverged".to_string(),
        message: "membership revocation reconcile diverged".to_string(),
        drained: 1,
        diverged: 1,
        rejected: 0,
    }
}

#[derive(Default, Clone)]
struct FailOnceAlertSink {
    fail_once: Arc<Mutex<bool>>,
    emitted: Arc<Mutex<Vec<MembershipRevocationAnomalyAlert>>>,
}

impl FailOnceAlertSink {
    fn new() -> Self {
        Self {
            fail_once: Arc::new(Mutex::new(true)),
            emitted: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn emitted(&self) -> Vec<MembershipRevocationAnomalyAlert> {
        self.emitted.lock().expect("lock emitted").clone()
    }
}

impl MembershipRevocationAlertSink for FailOnceAlertSink {
    fn emit(&self, alert: &MembershipRevocationAnomalyAlert) -> Result<(), WorldError> {
        let mut fail_once = self.fail_once.lock().expect("lock fail_once");
        if *fail_once {
            *fail_once = false;
            return Err(WorldError::Io("simulated alert sink failure".to_string()));
        }

        let mut emitted = self.emitted.lock().expect("lock emitted");
        emitted.push(alert.clone());
        Ok(())
    }
}

#[test]
fn file_coordinator_state_store_round_trip() {
    let root = temp_membership_dir("revocation-coordinator-state-store");
    fs::create_dir_all(&root).expect("create temp dir");

    let store = FileMembershipRevocationCoordinatorStateStore::new(&root).expect("create store");
    let state = MembershipRevocationCoordinatorLeaseState {
        holder_node_id: "node-a".to_string(),
        expires_at_ms: 1200,
    };

    store.save("w1", &state).expect("save state");
    let loaded = store.load("w1").expect("load state");
    assert_eq!(loaded, Some(state.clone()));

    store.clear("w1").expect("clear state");
    let missing = store.load("w1").expect("load missing");
    assert_eq!(missing, None);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn store_backed_schedule_coordinator_blocks_until_expired_or_released() {
    let store: Arc<dyn MembershipRevocationCoordinatorStateStore + Send + Sync> =
        Arc::new(InMemoryMembershipRevocationCoordinatorStateStore::new());
    let coordinator_a = StoreBackedMembershipRevocationScheduleCoordinator::new(Arc::clone(&store));
    let coordinator_b = StoreBackedMembershipRevocationScheduleCoordinator::new(Arc::clone(&store));

    assert!(coordinator_a
        .acquire("w1", "node-a", 1000, 500)
        .expect("acquire node-a"));
    assert!(!coordinator_b
        .acquire("w1", "node-b", 1200, 500)
        .expect("acquire node-b while held"));
    assert!(coordinator_b
        .acquire("w1", "node-b", 1601, 500)
        .expect("acquire node-b after expiry"));

    coordinator_b
        .release("w1", "node-b")
        .expect("release node-b");
    assert!(coordinator_a
        .acquire("w1", "node-a", 1602, 500)
        .expect("acquire node-a after release"));
}

#[test]
fn emit_revocation_reconcile_alerts_with_recovery_buffers_and_recovers() {
    let client = sample_client();
    let sink = FailOnceAlertSink::new();
    let recovery_store = InMemoryMembershipRevocationAlertRecoveryStore::new();

    let first = client
        .emit_revocation_reconcile_alerts_with_recovery(
            "w1",
            "node-a",
            &sink,
            &recovery_store,
            vec![
                sample_alert("w1", "node-a", 1000),
                sample_alert("w1", "node-a", 1001),
            ],
        )
        .expect("first emit with recovery");
    assert_eq!(first.recovered, 0);
    assert_eq!(first.emitted_new, 0);
    assert_eq!(first.buffered, 2);

    let pending_after_first = recovery_store
        .load_pending("w1", "node-a")
        .expect("load pending after first");
    assert_eq!(pending_after_first.len(), 2);

    let second = client
        .emit_revocation_reconcile_alerts_with_recovery(
            "w1",
            "node-a",
            &sink,
            &recovery_store,
            Vec::new(),
        )
        .expect("second emit with recovery");
    assert_eq!(second.recovered, 2);
    assert_eq!(second.emitted_new, 0);
    assert_eq!(second.buffered, 0);

    let pending_after_second = recovery_store
        .load_pending("w1", "node-a")
        .expect("load pending after second");
    assert!(pending_after_second.is_empty());
    assert_eq!(sink.emitted().len(), 2);
}

#[test]
fn run_revocation_reconcile_coordinated_with_recovery_replays_pending_alerts() {
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

    let alert_sink = FailOnceAlertSink::new();
    let recovery_store = InMemoryMembershipRevocationAlertRecoveryStore::new();
    let coordinator_store: Arc<dyn MembershipRevocationCoordinatorStateStore + Send + Sync> =
        Arc::new(InMemoryMembershipRevocationCoordinatorStateStore::new());
    let coordinator = StoreBackedMembershipRevocationScheduleCoordinator::new(coordinator_store);

    let first = client
        .run_revocation_reconcile_coordinated_with_recovery(
            "w1",
            "node-b",
            1305,
            &subscription,
            &mut local_keyring,
            &reconcile_policy,
            &schedule_policy,
            &alert_policy,
            None,
            None,
            &schedule_store,
            &alert_sink,
            &recovery_store,
            &coordinator,
            1000,
        )
        .expect("first coordinated recovery run");
    assert!(first.acquired);
    assert_eq!(first.recovered_alerts, 0);
    assert_eq!(first.emitted_alerts, 0);
    assert_eq!(first.buffered_alerts, 1);

    let second = client
        .run_revocation_reconcile_coordinated_with_recovery(
            "w1",
            "node-b",
            1310,
            &subscription,
            &mut local_keyring,
            &reconcile_policy,
            &schedule_policy,
            &alert_policy,
            None,
            None,
            &schedule_store,
            &alert_sink,
            &recovery_store,
            &coordinator,
            1000,
        )
        .expect("second coordinated recovery run");

    assert!(second.acquired);
    assert_eq!(second.recovered_alerts, 1);
    assert_eq!(second.emitted_alerts, 0);
    assert_eq!(second.buffered_alerts, 0);

    let pending = recovery_store
        .load_pending("w1", "node-b")
        .expect("load pending");
    assert!(pending.is_empty());
    assert_eq!(alert_sink.emitted().len(), 1);
}
