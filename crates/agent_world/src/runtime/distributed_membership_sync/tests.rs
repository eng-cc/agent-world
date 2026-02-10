use std::sync::Arc;

use super::super::distributed::{topic_membership, TOPIC_MEMBERSHIP_SUFFIX};
use super::super::distributed_dht::{DistributedDht, InMemoryDht};
use super::super::distributed_net::{DistributedNetwork, InMemoryNetwork};
use super::*;

fn membership_request(
    requester_id: &str,
    requested_at_ms: i64,
    change: ConsensusMembershipChange,
) -> ConsensusMembershipChangeRequest {
    ConsensusMembershipChangeRequest {
        requester_id: requester_id.to_string(),
        requested_at_ms,
        reason: None,
        change,
    }
}

fn sample_consensus() -> QuorumConsensus {
    QuorumConsensus::new(super::super::distributed_consensus::ConsensusConfig {
        validators: vec![
            "seq-1".to_string(),
            "seq-2".to_string(),
            "seq-3".to_string(),
        ],
        quorum_threshold: 0,
    })
    .expect("consensus")
}

fn sample_snapshot() -> MembershipDirectorySnapshot {
    MembershipDirectorySnapshot {
        world_id: "w1".to_string(),
        requester_id: "seq-1".to_string(),
        requested_at_ms: 400,
        reason: Some("restart".to_string()),
        validators: vec![
            "seq-1".to_string(),
            "seq-2".to_string(),
            "seq-3".to_string(),
            "seq-4".to_string(),
        ],
        quorum_threshold: 3,
        signature_key_id: None,
        signature: None,
    }
}

fn sample_keyring_with_rotation() -> MembershipDirectorySignerKeyring {
    let mut keyring = MembershipDirectorySignerKeyring::new();
    keyring
        .add_hmac_sha256_key("k1", "membership-secret-v1")
        .expect("add k1");
    keyring
        .add_hmac_sha256_key("k2", "membership-secret-v2")
        .expect("add k2");
    keyring.set_active_key("k2").expect("active k2");
    keyring
}

#[test]
fn membership_topic_suffix_matches_topic_name() {
    assert_eq!(TOPIC_MEMBERSHIP_SUFFIX, "membership");
    assert_eq!(topic_membership("w1"), "aw.w1.membership");
}

#[test]
fn membership_snapshot_signer_round_trip() {
    let signer = MembershipDirectorySigner::hmac_sha256("membership-secret");
    let mut snapshot = sample_snapshot();
    let signature = signer.sign_snapshot(&snapshot).expect("sign snapshot");
    snapshot.signature = Some(signature);
    signer.verify_snapshot(&snapshot).expect("verify snapshot");
}

#[test]
fn membership_snapshot_signer_rejects_tampered_snapshot() {
    let signer = MembershipDirectorySigner::hmac_sha256("membership-secret");
    let mut snapshot = sample_snapshot();
    let signature = signer.sign_snapshot(&snapshot).expect("sign snapshot");
    snapshot.signature = Some(signature);
    snapshot.validators.push("seq-5".to_string());

    let err = signer
        .verify_snapshot(&snapshot)
        .expect_err("verify should fail");
    assert!(matches!(
        err,
        WorldError::DistributedValidationFailed { .. }
    ));
}

#[test]
fn membership_keyring_sign_and_verify_round_trip() {
    let mut keyring = MembershipDirectorySignerKeyring::new();
    keyring
        .add_hmac_sha256_key("k1", "membership-secret-v1")
        .expect("add key");
    keyring.set_active_key("k1").expect("set active key");

    let mut snapshot = sample_snapshot();
    let (key_id, signature) = keyring
        .sign_snapshot_with_active_key(&snapshot)
        .expect("sign with active key");
    snapshot.signature_key_id = Some(key_id);
    snapshot.signature = Some(signature);

    keyring.verify_snapshot(&snapshot).expect("verify snapshot");
}

#[test]
fn membership_keyring_verifies_snapshot_signed_before_rotation() {
    let mut keyring = MembershipDirectorySignerKeyring::new();
    keyring
        .add_hmac_sha256_key("k1", "membership-secret-v1")
        .expect("add k1");
    keyring.set_active_key("k1").expect("active k1");

    let mut snapshot = sample_snapshot();
    let (key_id, signature) = keyring
        .sign_snapshot_with_active_key(&snapshot)
        .expect("sign with k1");
    snapshot.signature_key_id = Some(key_id);
    snapshot.signature = Some(signature);

    keyring
        .add_hmac_sha256_key("k2", "membership-secret-v2")
        .expect("add k2");
    keyring.set_active_key("k2").expect("rotate active key");

    keyring
        .verify_snapshot(&snapshot)
        .expect("verify legacy snapshot with old key");
}

#[test]
fn publish_and_drain_membership_change_announcement() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));
    let subscription = sync_client.subscribe("w1").expect("subscribe");

    let request = membership_request(
        "seq-1",
        100,
        ConsensusMembershipChange::AddValidator {
            validator_id: "seq-4".to_string(),
        },
    );
    let result = ConsensusMembershipChangeResult {
        applied: true,
        validators: vec![
            "seq-1".to_string(),
            "seq-2".to_string(),
            "seq-3".to_string(),
            "seq-4".to_string(),
        ],
        quorum_threshold: 3,
    };

    let published = sync_client
        .publish_membership_change("w1", &request, &result)
        .expect("publish");
    assert_eq!(published.world_id, "w1");
    assert!(published.signature.is_none());

    let drained = sync_client
        .drain_announcements(&subscription)
        .expect("drain announcements");
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0], published);
}

#[test]
fn sync_membership_directory_applies_replace_and_is_idempotent() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));
    let subscription = sync_client.subscribe("w1").expect("subscribe");

    let request = membership_request(
        "seq-1",
        200,
        ConsensusMembershipChange::AddValidator {
            validator_id: "seq-4".to_string(),
        },
    );
    let result = ConsensusMembershipChangeResult {
        applied: true,
        validators: vec![
            "seq-1".to_string(),
            "seq-2".to_string(),
            "seq-3".to_string(),
            "seq-4".to_string(),
        ],
        quorum_threshold: 3,
    };

    sync_client
        .publish_membership_change("w1", &request, &result)
        .expect("publish 1");
    sync_client
        .publish_membership_change("w1", &request, &result)
        .expect("publish 2");

    let mut consensus = sample_consensus();
    let report = sync_client
        .sync_membership_directory(&subscription, &mut consensus)
        .expect("sync directory");
    assert_eq!(report.drained, 2);
    assert_eq!(report.applied, 1);
    assert_eq!(report.ignored, 1);
    assert_eq!(consensus.validators().len(), 4);
    assert_eq!(consensus.quorum_threshold(), 3);
}

#[test]
fn publish_membership_change_with_dht_persists_snapshot() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let dht: Arc<dyn DistributedDht + Send + Sync> = Arc::new(InMemoryDht::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));

    let request = membership_request(
        "seq-1",
        300,
        ConsensusMembershipChange::AddValidator {
            validator_id: "seq-4".to_string(),
        },
    );
    let result = ConsensusMembershipChangeResult {
        applied: true,
        validators: vec![
            "seq-1".to_string(),
            "seq-2".to_string(),
            "seq-3".to_string(),
            "seq-4".to_string(),
        ],
        quorum_threshold: 3,
    };

    let announce = sync_client
        .publish_membership_change_with_dht("w1", &request, &result, dht.as_ref())
        .expect("publish with dht");
    let snapshot = dht
        .get_membership_directory("w1")
        .expect("get membership")
        .expect("snapshot exists");
    assert_eq!(MembershipDirectoryAnnounce::from(snapshot), announce);
}

#[test]
fn publish_membership_change_with_dht_signed_persists_signed_snapshot() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let dht: Arc<dyn DistributedDht + Send + Sync> = Arc::new(InMemoryDht::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));
    let signer = MembershipDirectorySigner::hmac_sha256("membership-secret");
    let subscription = sync_client.subscribe("w1").expect("subscribe");

    let request = membership_request(
        "seq-1",
        320,
        ConsensusMembershipChange::AddValidator {
            validator_id: "seq-4".to_string(),
        },
    );
    let result = ConsensusMembershipChangeResult {
        applied: true,
        validators: vec![
            "seq-1".to_string(),
            "seq-2".to_string(),
            "seq-3".to_string(),
            "seq-4".to_string(),
        ],
        quorum_threshold: 3,
    };

    let announce = sync_client
        .publish_membership_change_with_dht_signed("w1", &request, &result, dht.as_ref(), &signer)
        .expect("publish signed");
    assert!(announce.signature.is_some());
    assert!(announce.signature_key_id.is_none());

    let snapshot = dht
        .get_membership_directory("w1")
        .expect("get membership")
        .expect("snapshot exists");
    assert_eq!(snapshot.signature, announce.signature);
    assert_eq!(snapshot.signature_key_id, announce.signature_key_id);
    signer.verify_snapshot(&snapshot).expect("signature valid");

    let drained = sync_client
        .drain_announcements(&subscription)
        .expect("drain announcements");
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0], announce);
}

#[test]
fn publish_membership_change_with_dht_signed_with_keyring_persists_key_id() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let dht: Arc<dyn DistributedDht + Send + Sync> = Arc::new(InMemoryDht::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));
    let keyring = sample_keyring_with_rotation();

    let request = membership_request(
        "seq-1",
        325,
        ConsensusMembershipChange::AddValidator {
            validator_id: "seq-4".to_string(),
        },
    );
    let result = ConsensusMembershipChangeResult {
        applied: true,
        validators: vec![
            "seq-1".to_string(),
            "seq-2".to_string(),
            "seq-3".to_string(),
            "seq-4".to_string(),
        ],
        quorum_threshold: 3,
    };

    let announce = sync_client
        .publish_membership_change_with_dht_signed_with_keyring(
            "w1",
            &request,
            &result,
            dht.as_ref(),
            &keyring,
        )
        .expect("publish with keyring");
    assert_eq!(announce.signature_key_id.as_deref(), Some("k2"));
    assert!(announce.signature.is_some());

    let snapshot = dht
        .get_membership_directory("w1")
        .expect("get membership")
        .expect("snapshot exists");
    assert_eq!(snapshot.signature_key_id.as_deref(), Some("k2"));
    keyring.verify_snapshot(&snapshot).expect("signature valid");
}

#[test]
fn restore_membership_from_dht_applies_replace_snapshot() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let dht: Arc<dyn DistributedDht + Send + Sync> = Arc::new(InMemoryDht::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));

    let snapshot = sample_snapshot();
    dht.put_membership_directory("w1", &snapshot)
        .expect("put membership");

    let mut consensus = sample_consensus();
    let restored = sync_client
        .restore_membership_from_dht("w1", &mut consensus, dht.as_ref())
        .expect("restore")
        .expect("restored");
    assert!(restored.applied);
    assert_eq!(consensus.validators().len(), 4);
    assert_eq!(consensus.quorum_threshold(), 3);
}

#[test]
fn restore_membership_from_dht_verified_rejects_untrusted_requester() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let dht: Arc<dyn DistributedDht + Send + Sync> = Arc::new(InMemoryDht::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));

    let snapshot = sample_snapshot();
    dht.put_membership_directory("w1", &snapshot)
        .expect("put membership");

    let mut consensus = sample_consensus();
    let policy = MembershipSnapshotRestorePolicy {
        trusted_requesters: vec!["seq-9".to_string()],
        require_signature: false,
        require_signature_key_id: false,
        accepted_signature_key_ids: Vec::new(),
    };
    let err = sync_client
        .restore_membership_from_dht_verified("w1", &mut consensus, dht.as_ref(), None, &policy)
        .expect_err("restore should fail");
    assert!(matches!(
        err,
        WorldError::DistributedValidationFailed { .. }
    ));
}

#[test]
fn restore_membership_from_dht_verified_requires_signature_when_enabled() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let dht: Arc<dyn DistributedDht + Send + Sync> = Arc::new(InMemoryDht::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));

    let snapshot = sample_snapshot();
    dht.put_membership_directory("w1", &snapshot)
        .expect("put membership");

    let mut consensus = sample_consensus();
    let policy = MembershipSnapshotRestorePolicy {
        trusted_requesters: vec!["seq-1".to_string()],
        require_signature: true,
        require_signature_key_id: false,
        accepted_signature_key_ids: Vec::new(),
    };
    let err = sync_client
        .restore_membership_from_dht_verified(
            "w1",
            &mut consensus,
            dht.as_ref(),
            Some(&MembershipDirectorySigner::hmac_sha256("membership-secret")),
            &policy,
        )
        .expect_err("restore should fail");
    assert!(matches!(
        err,
        WorldError::DistributedValidationFailed { .. }
    ));
}

#[test]
fn restore_membership_from_dht_verified_accepts_signed_trusted_snapshot() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let dht: Arc<dyn DistributedDht + Send + Sync> = Arc::new(InMemoryDht::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));
    let signer = MembershipDirectorySigner::hmac_sha256("membership-secret");

    let mut snapshot = sample_snapshot();
    snapshot.signature = Some(signer.sign_snapshot(&snapshot).expect("sign snapshot"));
    dht.put_membership_directory("w1", &snapshot)
        .expect("put membership");

    let mut consensus = sample_consensus();
    let policy = MembershipSnapshotRestorePolicy {
        trusted_requesters: vec!["seq-1".to_string()],
        require_signature: true,
        require_signature_key_id: false,
        accepted_signature_key_ids: Vec::new(),
    };
    let restored = sync_client
        .restore_membership_from_dht_verified(
            "w1",
            &mut consensus,
            dht.as_ref(),
            Some(&signer),
            &policy,
        )
        .expect("restore")
        .expect("restored");
    assert!(restored.applied);
}

#[test]
fn restore_membership_from_dht_verified_with_keyring_accepts_rotated_key() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let dht: Arc<dyn DistributedDht + Send + Sync> = Arc::new(InMemoryDht::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));

    let old_signer = MembershipDirectorySigner::hmac_sha256("membership-secret-v1");
    let mut snapshot = sample_snapshot();
    snapshot.signature_key_id = Some("k1".to_string());
    snapshot.signature = Some(old_signer.sign_snapshot(&snapshot).expect("sign snapshot"));
    dht.put_membership_directory("w1", &snapshot)
        .expect("put membership");

    let keyring = sample_keyring_with_rotation();
    let mut consensus = sample_consensus();
    let policy = MembershipSnapshotRestorePolicy {
        trusted_requesters: vec!["seq-1".to_string()],
        require_signature: true,
        require_signature_key_id: true,
        accepted_signature_key_ids: vec!["k1".to_string(), "k2".to_string()],
    };
    let restored = sync_client
        .restore_membership_from_dht_verified_with_keyring(
            "w1",
            &mut consensus,
            dht.as_ref(),
            Some(&keyring),
            &policy,
        )
        .expect("restore")
        .expect("restored");
    assert!(restored.applied);
}

#[test]
fn restore_membership_from_dht_verified_with_keyring_rejects_unaccepted_key() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let dht: Arc<dyn DistributedDht + Send + Sync> = Arc::new(InMemoryDht::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));

    let old_signer = MembershipDirectorySigner::hmac_sha256("membership-secret-v1");
    let mut snapshot = sample_snapshot();
    snapshot.signature_key_id = Some("k1".to_string());
    snapshot.signature = Some(old_signer.sign_snapshot(&snapshot).expect("sign snapshot"));
    dht.put_membership_directory("w1", &snapshot)
        .expect("put membership");

    let keyring = sample_keyring_with_rotation();
    let mut consensus = sample_consensus();
    let policy = MembershipSnapshotRestorePolicy {
        trusted_requesters: vec!["seq-1".to_string()],
        require_signature: true,
        require_signature_key_id: true,
        accepted_signature_key_ids: vec!["k2".to_string()],
    };
    let err = sync_client
        .restore_membership_from_dht_verified_with_keyring(
            "w1",
            &mut consensus,
            dht.as_ref(),
            Some(&keyring),
            &policy,
        )
        .expect_err("restore should fail");
    assert!(matches!(
        err,
        WorldError::DistributedValidationFailed { .. }
    ));
}

#[test]
fn restore_membership_from_dht_verified_with_audit_reports_rejection() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let dht: Arc<dyn DistributedDht + Send + Sync> = Arc::new(InMemoryDht::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));

    let snapshot = sample_snapshot();
    dht.put_membership_directory("w1", &snapshot)
        .expect("put membership");

    let mut consensus = sample_consensus();
    let policy = MembershipSnapshotRestorePolicy {
        trusted_requesters: vec!["seq-9".to_string()],
        require_signature: false,
        require_signature_key_id: false,
        accepted_signature_key_ids: Vec::new(),
    };
    let report = sync_client
        .restore_membership_from_dht_verified_with_audit(
            "w1",
            &mut consensus,
            dht.as_ref(),
            None,
            None,
            &policy,
        )
        .expect("audit restore");
    assert!(report.restored.is_none());
    assert_eq!(
        report.audit.outcome,
        MembershipSnapshotAuditOutcome::Rejected
    );
    assert!(report.audit.reason.contains("not trusted"));
}

#[test]
fn restore_membership_from_dht_verified_with_audit_reports_missing_snapshot() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let dht: Arc<dyn DistributedDht + Send + Sync> = Arc::new(InMemoryDht::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));

    let mut consensus = sample_consensus();
    let report = sync_client
        .restore_membership_from_dht_verified_with_audit(
            "w1",
            &mut consensus,
            dht.as_ref(),
            None,
            None,
            &MembershipSnapshotRestorePolicy::default(),
        )
        .expect("audit restore");
    assert!(report.restored.is_none());
    assert_eq!(
        report.audit.outcome,
        MembershipSnapshotAuditOutcome::MissingSnapshot
    );
}

#[test]
fn restore_membership_from_dht_returns_none_when_missing() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let dht: Arc<dyn DistributedDht + Send + Sync> = Arc::new(InMemoryDht::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));
    let mut consensus = sample_consensus();

    let restored = sync_client
        .restore_membership_from_dht("w1", &mut consensus, dht.as_ref())
        .expect("restore result");
    assert!(restored.is_none());
    assert_eq!(consensus.validators().len(), 3);
    assert_eq!(consensus.quorum_threshold(), 2);
}
