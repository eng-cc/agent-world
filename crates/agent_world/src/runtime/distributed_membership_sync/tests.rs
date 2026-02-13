use std::sync::Arc;

use super::super::distributed_dht::MembershipDirectorySnapshot;
use super::super::distributed_net::{DistributedNetwork, InMemoryNetwork};
use super::*;

#[test]
fn runtime_membership_exports_are_available() {
    let _ = std::any::type_name::<MembershipSyncClient>();
    let _ = std::any::type_name::<MembershipDirectorySigner>();
    let _ = std::any::type_name::<MembershipRevocationReconcilePolicy>();
    let _ = std::any::type_name::<MembershipRevocationAlertRecoveryReport>();
}

#[test]
fn membership_sync_client_subscribe_and_drain_empty() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let client = MembershipSyncClient::new(network);

    let subscription = client.subscribe("w1").expect("subscribe");
    let announcements = client
        .drain_announcements(&subscription)
        .expect("drain announcements");
    let revocations = client
        .drain_key_revocations(&subscription)
        .expect("drain revocations");

    assert!(announcements.is_empty());
    assert!(revocations.is_empty());
}

#[test]
fn membership_directory_signer_round_trip() {
    let signer = MembershipDirectorySigner::hmac_sha256("membership-secret");
    let mut snapshot = MembershipDirectorySnapshot {
        world_id: "w1".to_string(),
        requester_id: "seq-1".to_string(),
        requested_at_ms: 100,
        reason: Some("bootstrap".to_string()),
        validators: vec![
            "seq-1".to_string(),
            "seq-2".to_string(),
            "seq-3".to_string(),
        ],
        quorum_threshold: 2,
        signature_key_id: None,
        signature: None,
    };

    let signature = signer.sign_snapshot(&snapshot).expect("sign snapshot");
    snapshot.signature = Some(signature);

    signer.verify_snapshot(&snapshot).expect("verify snapshot");
}
