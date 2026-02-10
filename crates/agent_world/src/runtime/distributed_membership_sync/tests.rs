use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use super::super::distributed::{
    topic_membership, topic_membership_reconcile, topic_membership_revocation,
    TOPIC_MEMBERSHIP_RECONCILE_SUFFIX, TOPIC_MEMBERSHIP_REVOKE_SUFFIX, TOPIC_MEMBERSHIP_SUFFIX,
};
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

fn temp_membership_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("agent_world-{prefix}-{nanos}"))
}

#[test]
fn membership_topic_suffix_matches_topic_name() {
    assert_eq!(TOPIC_MEMBERSHIP_SUFFIX, "membership");
    assert_eq!(TOPIC_MEMBERSHIP_REVOKE_SUFFIX, "membership.revoke");
    assert_eq!(TOPIC_MEMBERSHIP_RECONCILE_SUFFIX, "membership.reconcile");
    assert_eq!(topic_membership("w1"), "aw.w1.membership");
    assert_eq!(topic_membership_revocation("w1"), "aw.w1.membership.revoke");
    assert_eq!(
        topic_membership_reconcile("w1"),
        "aw.w1.membership.reconcile"
    );
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
fn membership_keyring_revoke_key_blocks_sign_and_verify() {
    let signer = MembershipDirectorySigner::hmac_sha256("membership-secret-v1");
    let mut snapshot = sample_snapshot();
    snapshot.signature_key_id = Some("k1".to_string());
    snapshot.signature = Some(signer.sign_snapshot(&snapshot).expect("sign snapshot"));

    let mut keyring = MembershipDirectorySignerKeyring::new();
    keyring
        .add_hmac_sha256_key("k1", "membership-secret-v1")
        .expect("add key");
    keyring.set_active_key("k1").expect("set active key");

    assert!(keyring.revoke_key("k1").expect("revoke key"));
    assert!(keyring.is_key_revoked("k1"));

    let sign_err = keyring
        .sign_snapshot_with_key_id("k1", &sample_snapshot())
        .expect_err("revoked key should fail sign");
    assert!(matches!(
        sign_err,
        WorldError::DistributedValidationFailed { .. }
    ));

    let verify_err = keyring
        .verify_snapshot(&snapshot)
        .expect_err("revoked key should fail verify");
    assert!(matches!(
        verify_err,
        WorldError::DistributedValidationFailed { .. }
    ));
}

#[test]
fn membership_revocation_signer_round_trip() {
    let signer = MembershipDirectorySigner::hmac_sha256("membership-secret-v1");
    let mut announce = MembershipKeyRevocationAnnounce {
        world_id: "w1".to_string(),
        requester_id: "seq-1".to_string(),
        requested_at_ms: 800,
        key_id: "k1".to_string(),
        reason: Some("rotate".to_string()),
        signature_key_id: Some("k1".to_string()),
        signature: None,
    };
    announce.signature = Some(signer.sign_revocation(&announce).expect("sign revocation"));

    signer
        .verify_revocation(&announce)
        .expect("verify revocation signature");
}

#[test]
fn membership_keyring_sign_and_verify_revocation_round_trip() {
    let mut keyring = MembershipDirectorySignerKeyring::new();
    keyring
        .add_hmac_sha256_key("k1", "membership-secret-v1")
        .expect("add key");
    keyring.set_active_key("k1").expect("set active key");

    let mut announce = MembershipKeyRevocationAnnounce {
        world_id: "w1".to_string(),
        requester_id: "seq-1".to_string(),
        requested_at_ms: 801,
        key_id: "k1".to_string(),
        reason: Some("compromised".to_string()),
        signature_key_id: None,
        signature: None,
    };

    let (key_id, signature) = keyring
        .sign_revocation_with_active_key(&announce)
        .expect("sign revocation with active key");
    announce.signature_key_id = Some(key_id);
    announce.signature = Some(signature);

    keyring
        .verify_revocation(&announce)
        .expect("verify keyring revocation signature");
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
fn publish_and_drain_membership_key_revocation_announcement() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));
    let subscription = sync_client.subscribe("w1").expect("subscribe");

    let published = sync_client
        .publish_key_revocation(
            "w1",
            "seq-1",
            300,
            "k1",
            Some("key compromised".to_string()),
        )
        .expect("publish revocation");

    assert_eq!(published.world_id, "w1");
    assert_eq!(published.requester_id, "seq-1");
    assert_eq!(published.key_id, "k1");

    let drained = sync_client
        .drain_key_revocations(&subscription)
        .expect("drain revocations");
    assert_eq!(drained, vec![published]);
}

#[test]
fn sync_key_revocations_updates_keyring() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));
    let subscription = sync_client.subscribe("w1").expect("subscribe");

    sync_client
        .publish_key_revocation("w1", "seq-1", 300, "k1", Some("rotate".to_string()))
        .expect("publish revocation");
    sync_client
        .publish_key_revocation("w1", "seq-2", 301, "k1", Some("duplicate".to_string()))
        .expect("publish duplicate revocation");

    let mut keyring = sample_keyring_with_rotation();
    let applied = sync_client
        .sync_key_revocations(&subscription, &mut keyring)
        .expect("sync revocations");
    assert_eq!(applied, 1);
    assert!(keyring.is_key_revoked("k1"));
}

#[test]
fn sync_key_revocations_with_policy_accepts_signed_trusted_announce() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));
    let subscription = sync_client.subscribe("w1").expect("subscribe");

    let mut signer_keyring = MembershipDirectorySignerKeyring::new();
    signer_keyring
        .add_hmac_sha256_key("kr", "membership-secret-revoke")
        .expect("add signer key");
    signer_keyring.set_active_key("kr").expect("set active key");

    sync_client
        .publish_key_revocation_signed_with_keyring(
            "w1",
            "seq-1",
            320,
            "k1",
            Some("security incident".to_string()),
            &signer_keyring,
        )
        .expect("publish signed revocation");

    let mut apply_keyring = MembershipDirectorySignerKeyring::new();
    apply_keyring
        .add_hmac_sha256_key("kr", "membership-secret-revoke")
        .expect("add verify key");
    apply_keyring
        .set_active_key("kr")
        .expect("set verify key active");
    apply_keyring
        .add_hmac_sha256_key("k1", "membership-secret-v1")
        .expect("add revoked target key");

    let policy = MembershipRevocationSyncPolicy {
        trusted_requesters: vec!["seq-1".to_string()],
        authorized_requesters: Vec::new(),
        require_signature: true,
        require_signature_key_id: true,
        accepted_signature_key_ids: vec!["kr".to_string()],
        revoked_signature_key_ids: Vec::new(),
    };
    let report = sync_client
        .sync_key_revocations_with_policy("w1", &subscription, &mut apply_keyring, None, &policy)
        .expect("sync with policy");

    assert_eq!(report.drained, 1);
    assert_eq!(report.applied, 1);
    assert_eq!(report.ignored, 0);
    assert_eq!(report.rejected, 0);
    assert!(apply_keyring.is_key_revoked("k1"));
}

#[test]
fn sync_key_revocations_with_policy_rejects_untrusted_requester() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));
    let subscription = sync_client.subscribe("w1").expect("subscribe");

    let mut signer_keyring = MembershipDirectorySignerKeyring::new();
    signer_keyring
        .add_hmac_sha256_key("kr", "membership-secret-revoke")
        .expect("add signer key");
    signer_keyring.set_active_key("kr").expect("set active key");

    sync_client
        .publish_key_revocation_signed_with_keyring(
            "w1",
            "seq-9",
            321,
            "k1",
            Some("untrusted requester".to_string()),
            &signer_keyring,
        )
        .expect("publish signed revocation");

    let mut apply_keyring = MembershipDirectorySignerKeyring::new();
    apply_keyring
        .add_hmac_sha256_key("kr", "membership-secret-revoke")
        .expect("add verify key");
    apply_keyring
        .set_active_key("kr")
        .expect("set verify key active");
    apply_keyring
        .add_hmac_sha256_key("k1", "membership-secret-v1")
        .expect("add revoked target key");

    let policy = MembershipRevocationSyncPolicy {
        trusted_requesters: vec!["seq-1".to_string()],
        authorized_requesters: Vec::new(),
        require_signature: true,
        require_signature_key_id: true,
        accepted_signature_key_ids: vec!["kr".to_string()],
        revoked_signature_key_ids: Vec::new(),
    };
    let report = sync_client
        .sync_key_revocations_with_policy("w1", &subscription, &mut apply_keyring, None, &policy)
        .expect("sync with policy");

    assert_eq!(report.drained, 1);
    assert_eq!(report.applied, 0);
    assert_eq!(report.ignored, 0);
    assert_eq!(report.rejected, 1);
    assert!(!apply_keyring.is_key_revoked("k1"));
}

#[test]
fn sync_key_revocations_with_policy_rejects_unauthorized_requester() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));
    let subscription = sync_client.subscribe("w1").expect("subscribe");

    let mut signer_keyring = MembershipDirectorySignerKeyring::new();
    signer_keyring
        .add_hmac_sha256_key("kr", "membership-secret-revoke")
        .expect("add signer key");
    signer_keyring.set_active_key("kr").expect("set active key");

    sync_client
        .publish_key_revocation_signed_with_keyring(
            "w1",
            "seq-1",
            322,
            "k1",
            Some("unauthorized requester".to_string()),
            &signer_keyring,
        )
        .expect("publish signed revocation");

    let mut apply_keyring = MembershipDirectorySignerKeyring::new();
    apply_keyring
        .add_hmac_sha256_key("kr", "membership-secret-revoke")
        .expect("add verify key");
    apply_keyring
        .set_active_key("kr")
        .expect("set verify key active");
    apply_keyring
        .add_hmac_sha256_key("k1", "membership-secret-v1")
        .expect("add revoked target key");

    let policy = MembershipRevocationSyncPolicy {
        trusted_requesters: vec!["seq-1".to_string()],
        authorized_requesters: vec!["seq-2".to_string()],
        require_signature: true,
        require_signature_key_id: true,
        accepted_signature_key_ids: vec!["kr".to_string()],
        revoked_signature_key_ids: Vec::new(),
    };
    let report = sync_client
        .sync_key_revocations_with_policy("w1", &subscription, &mut apply_keyring, None, &policy)
        .expect("sync with policy");

    assert_eq!(report.drained, 1);
    assert_eq!(report.applied, 0);
    assert_eq!(report.ignored, 0);
    assert_eq!(report.rejected, 1);
    assert!(!apply_keyring.is_key_revoked("k1"));
}

#[test]
fn publish_and_drain_membership_revocation_checkpoint() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));
    let subscription = sync_client.subscribe("w1").expect("subscribe");

    let mut keyring = MembershipDirectorySignerKeyring::new();
    keyring
        .add_hmac_sha256_key("k1", "membership-secret-v1")
        .expect("add key");
    keyring.set_active_key("k1").expect("set active key");
    keyring.revoke_key("k1").expect("revoke key");

    let published = sync_client
        .publish_revocation_checkpoint("w1", "node-a", 900, &keyring)
        .expect("publish checkpoint");
    assert_eq!(published.world_id, "w1");
    assert_eq!(published.node_id, "node-a");
    assert_eq!(published.revoked_key_ids, vec!["k1".to_string()]);
    assert!(!published.revoked_set_hash.is_empty());

    let drained = sync_client
        .drain_revocation_checkpoints(&subscription)
        .expect("drain checkpoints");
    assert_eq!(drained, vec![published]);
}

#[test]
fn reconcile_revocations_with_policy_merges_missing_keys() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let publisher = MembershipSyncClient::new(Arc::clone(&network));
    let consumer = MembershipSyncClient::new(Arc::clone(&network));
    let subscription = consumer.subscribe("w1").expect("subscribe");

    let mut publisher_keyring = MembershipDirectorySignerKeyring::new();
    publisher_keyring
        .add_hmac_sha256_key("k1", "membership-secret-v1")
        .expect("add key1");
    publisher_keyring
        .add_hmac_sha256_key("k2", "membership-secret-v2")
        .expect("add key2");
    publisher_keyring
        .set_active_key("k2")
        .expect("set active key");
    publisher_keyring.revoke_key("k1").expect("revoke k1");
    publisher_keyring.revoke_key("k2").expect("revoke k2");

    publisher
        .publish_revocation_checkpoint("w1", "node-a", 901, &publisher_keyring)
        .expect("publish checkpoint");

    let mut consumer_keyring = MembershipDirectorySignerKeyring::new();
    consumer_keyring
        .add_hmac_sha256_key("k1", "membership-secret-v1")
        .expect("add key1");
    consumer_keyring
        .add_hmac_sha256_key("k2", "membership-secret-v2")
        .expect("add key2");
    consumer_keyring
        .set_active_key("k2")
        .expect("set active key");
    consumer_keyring.revoke_key("k1").expect("revoke k1");

    let policy = MembershipRevocationReconcilePolicy {
        trusted_nodes: vec!["node-a".to_string()],
        auto_revoke_missing_keys: true,
    };
    let report = consumer
        .reconcile_revocations_with_policy("w1", &subscription, &mut consumer_keyring, &policy)
        .expect("reconcile revocations");

    assert_eq!(report.drained, 1);
    assert_eq!(report.in_sync, 0);
    assert_eq!(report.diverged, 1);
    assert_eq!(report.merged, 1);
    assert_eq!(report.rejected, 0);
    assert!(consumer_keyring.is_key_revoked("k2"));
}

#[test]
fn reconcile_revocations_with_policy_rejects_untrusted_node() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let publisher = MembershipSyncClient::new(Arc::clone(&network));
    let consumer = MembershipSyncClient::new(Arc::clone(&network));
    let subscription = consumer.subscribe("w1").expect("subscribe");

    let mut publisher_keyring = MembershipDirectorySignerKeyring::new();
    publisher_keyring
        .add_hmac_sha256_key("k1", "membership-secret-v1")
        .expect("add key1");
    publisher_keyring
        .set_active_key("k1")
        .expect("set active key");
    publisher_keyring.revoke_key("k1").expect("revoke k1");

    publisher
        .publish_revocation_checkpoint("w1", "node-z", 902, &publisher_keyring)
        .expect("publish checkpoint");

    let mut consumer_keyring = MembershipDirectorySignerKeyring::new();
    consumer_keyring
        .add_hmac_sha256_key("k1", "membership-secret-v1")
        .expect("add key1");
    consumer_keyring
        .set_active_key("k1")
        .expect("set active key");

    let policy = MembershipRevocationReconcilePolicy {
        trusted_nodes: vec!["node-a".to_string()],
        auto_revoke_missing_keys: true,
    };
    let report = consumer
        .reconcile_revocations_with_policy("w1", &subscription, &mut consumer_keyring, &policy)
        .expect("reconcile revocations");

    assert_eq!(report.drained, 1);
    assert_eq!(report.in_sync, 0);
    assert_eq!(report.diverged, 0);
    assert_eq!(report.merged, 0);
    assert_eq!(report.rejected, 1);
    assert!(!consumer_keyring.is_key_revoked("k1"));
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
        revoked_signature_key_ids: Vec::new(),
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
        revoked_signature_key_ids: Vec::new(),
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
        revoked_signature_key_ids: Vec::new(),
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
        revoked_signature_key_ids: Vec::new(),
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
        revoked_signature_key_ids: Vec::new(),
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
        revoked_signature_key_ids: Vec::new(),
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
fn restore_membership_from_dht_verified_with_audit_store_persists_record() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let dht: Arc<dyn DistributedDht + Send + Sync> = Arc::new(InMemoryDht::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));

    let snapshot = sample_snapshot();
    dht.put_membership_directory("w1", &snapshot)
        .expect("put membership");

    let audit_store = InMemoryMembershipAuditStore::new();
    let mut consensus = sample_consensus();
    let report = sync_client
        .restore_membership_from_dht_verified_with_audit_store(
            "w1",
            &mut consensus,
            dht.as_ref(),
            None,
            None,
            &MembershipSnapshotRestorePolicy::default(),
            &audit_store,
        )
        .expect("audit restore with store");

    assert_eq!(
        report.audit.outcome,
        MembershipSnapshotAuditOutcome::Applied
    );
    let records = audit_store.list("w1").expect("list audit records");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0], report.audit);
}

#[test]
fn file_membership_audit_store_appends_and_lists_by_world() {
    let dir = temp_membership_dir("membership-audit-file-store");
    fs::create_dir_all(&dir).expect("create temp audit dir");

    let store = FileMembershipAuditStore::new(&dir);
    assert_eq!(store.root_dir(), dir.as_path());

    let record_w1 = MembershipSnapshotAuditRecord {
        world_id: "w1".to_string(),
        requester_id: Some("seq-1".to_string()),
        requested_at_ms: Some(100),
        signature_key_id: Some("k1".to_string()),
        outcome: MembershipSnapshotAuditOutcome::Applied,
        reason: "ok".to_string(),
    };
    let record_w2 = MembershipSnapshotAuditRecord {
        world_id: "w2".to_string(),
        requester_id: Some("seq-2".to_string()),
        requested_at_ms: Some(101),
        signature_key_id: None,
        outcome: MembershipSnapshotAuditOutcome::Rejected,
        reason: "bad signature".to_string(),
    };

    store.append(&record_w1).expect("append w1");
    store.append(&record_w2).expect("append w2");

    let w1_records = store.list("w1").expect("list w1");
    assert_eq!(w1_records, vec![record_w1]);

    let w2_records = store.list("w2").expect("list w2");
    assert_eq!(w2_records, vec![record_w2]);

    let missing = store.list("missing").expect("list missing");
    assert!(missing.is_empty());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn restore_membership_from_dht_verified_rejects_revoked_key_id() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let dht: Arc<dyn DistributedDht + Send + Sync> = Arc::new(InMemoryDht::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));

    let signer = MembershipDirectorySigner::hmac_sha256("membership-secret-v1");
    let mut snapshot = sample_snapshot();
    snapshot.signature_key_id = Some("k1".to_string());
    snapshot.signature = Some(signer.sign_snapshot(&snapshot).expect("sign snapshot"));
    dht.put_membership_directory("w1", &snapshot)
        .expect("put membership");

    let mut consensus = sample_consensus();
    let policy = MembershipSnapshotRestorePolicy {
        trusted_requesters: vec!["seq-1".to_string()],
        require_signature: true,
        require_signature_key_id: true,
        accepted_signature_key_ids: vec!["k1".to_string()],
        revoked_signature_key_ids: vec!["k1".to_string()],
    };

    let err = sync_client
        .restore_membership_from_dht_verified(
            "w1",
            &mut consensus,
            dht.as_ref(),
            Some(&signer),
            &policy,
        )
        .expect_err("revoked key should be rejected");
    assert!(matches!(
        err,
        WorldError::DistributedValidationFailed { .. }
    ));
}

#[test]
fn restore_membership_from_dht_verified_with_keyring_rejects_revoked_key_after_sync() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let dht: Arc<dyn DistributedDht + Send + Sync> = Arc::new(InMemoryDht::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));
    let subscription = sync_client.subscribe("w1").expect("subscribe");

    let signer = MembershipDirectorySigner::hmac_sha256("membership-secret-v1");
    let mut snapshot = sample_snapshot();
    snapshot.signature_key_id = Some("k1".to_string());
    snapshot.signature = Some(signer.sign_snapshot(&snapshot).expect("sign snapshot"));
    dht.put_membership_directory("w1", &snapshot)
        .expect("put membership");

    sync_client
        .publish_key_revocation("w1", "seq-1", 700, "k1", Some("compromised".to_string()))
        .expect("publish key revocation");

    let mut keyring = MembershipDirectorySignerKeyring::new();
    keyring
        .add_hmac_sha256_key("k1", "membership-secret-v1")
        .expect("add key");
    keyring.set_active_key("k1").expect("set active key");

    let revoked = sync_client
        .sync_key_revocations(&subscription, &mut keyring)
        .expect("sync revocations");
    assert_eq!(revoked, 1);

    let mut consensus = sample_consensus();
    let policy = MembershipSnapshotRestorePolicy {
        trusted_requesters: vec!["seq-1".to_string()],
        require_signature: true,
        require_signature_key_id: true,
        accepted_signature_key_ids: vec!["k1".to_string()],
        revoked_signature_key_ids: Vec::new(),
    };
    let err = sync_client
        .restore_membership_from_dht_verified_with_keyring(
            "w1",
            &mut consensus,
            dht.as_ref(),
            Some(&keyring),
            &policy,
        )
        .expect_err("restore should fail after key revocation sync");
    assert!(matches!(
        err,
        WorldError::DistributedValidationFailed { .. }
    ));
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
