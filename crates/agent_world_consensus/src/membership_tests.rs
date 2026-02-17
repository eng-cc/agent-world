use std::sync::Arc;

use agent_world_net::{DistributedNetwork, InMemoryDht, InMemoryNetwork};
use agent_world_proto::distributed_dht::{DistributedDht, MembershipDirectorySnapshot};
use ed25519_dalek::SigningKey;

use crate::membership::{
    MembershipDirectorySigner, MembershipDirectorySignerKeyring, MembershipKeyRevocationAnnounce,
    MembershipRevocationSyncPolicy, MembershipSnapshotRestorePolicy, MembershipSyncClient,
};
use crate::quorum::{
    ConsensusConfig, ConsensusMembershipChange, ConsensusMembershipChangeRequest,
    ConsensusMembershipChangeResult, QuorumConsensus,
};

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
    QuorumConsensus::new(ConsensusConfig {
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

fn sample_revocation() -> MembershipKeyRevocationAnnounce {
    MembershipKeyRevocationAnnounce {
        world_id: "w1".to_string(),
        requester_id: "seq-1".to_string(),
        requested_at_ms: 401,
        key_id: "k1".to_string(),
        reason: Some("rotate".to_string()),
        signature_key_id: Some("k1".to_string()),
        signature: None,
    }
}

fn ed25519_keypair_hex(seed: u8) -> (String, String) {
    let private_key_hex = hex::encode([seed; 32]);
    let signing_key = SigningKey::from_bytes(&[seed; 32]);
    let public_key_hex = hex::encode(signing_key.verifying_key().to_bytes());
    (private_key_hex, public_key_hex)
}

fn ed25519_signer(seed: u8) -> MembershipDirectorySigner {
    let (private_key_hex, public_key_hex) = ed25519_keypair_hex(seed);
    MembershipDirectorySigner::ed25519(private_key_hex.as_str(), public_key_hex.as_str())
        .expect("ed25519 signer")
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
fn membership_snapshot_ed25519_signer_round_trip() {
    let signer = ed25519_signer(7);
    let mut snapshot = sample_snapshot();
    let signature = signer.sign_snapshot(&snapshot).expect("sign snapshot");
    snapshot.signature = Some(signature);
    signer.verify_snapshot(&snapshot).expect("verify snapshot");
}

#[test]
fn membership_revocation_ed25519_signer_round_trip() {
    let signer = ed25519_signer(9);
    let mut announce = sample_revocation();
    let signature = signer.sign_revocation(&announce).expect("sign revocation");
    announce.signature = Some(signature);
    signer
        .verify_revocation(&announce)
        .expect("verify revocation");
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
fn membership_keyring_ed25519_sign_and_verify_round_trip() {
    let mut keyring = MembershipDirectorySignerKeyring::new();
    let (private_key_hex, public_key_hex) = ed25519_keypair_hex(17);
    keyring
        .add_ed25519_key("k-ed", private_key_hex.as_str(), public_key_hex.as_str())
        .expect("add key");
    keyring.set_active_key("k-ed").expect("set active key");

    let mut snapshot = sample_snapshot();
    let (key_id, signature) = keyring
        .sign_snapshot_with_active_key(&snapshot)
        .expect("sign with active key");
    snapshot.signature_key_id = Some(key_id);
    snapshot.signature = Some(signature);

    keyring.verify_snapshot(&snapshot).expect("verify snapshot");
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

    let drained = sync_client
        .drain_announcements(&subscription)
        .expect("drain announcements");
    assert_eq!(drained, vec![published]);
}

#[test]
fn restore_membership_from_dht_applies_replace_snapshot() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let dht = InMemoryDht::new();
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));
    let mut consensus = sample_consensus();
    let snapshot = sample_snapshot();
    dht.put_membership_directory("w1", &snapshot)
        .expect("put snapshot");

    let restored = sync_client
        .restore_membership_from_dht("w1", &mut consensus, &dht)
        .expect("restore membership")
        .expect("restored result");

    assert!(restored.applied);
    assert!(restored.validators.iter().any(|id| id == "seq-4"));
}

#[test]
fn publish_membership_change_with_dht_signed_with_ed25519_keyring_restores_with_policy() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let dht = InMemoryDht::new();
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));

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

    let mut keyring = MembershipDirectorySignerKeyring::new();
    let (private_key_hex, public_key_hex) = ed25519_keypair_hex(23);
    keyring
        .add_ed25519_key("k-ed", private_key_hex.as_str(), public_key_hex.as_str())
        .expect("add key");
    keyring.set_active_key("k-ed").expect("set active key");

    let published = sync_client
        .publish_membership_change_with_dht_signed_with_keyring(
            "w1", &request, &result, &dht, &keyring,
        )
        .expect("publish signed membership change");
    assert_eq!(published.signature_key_id.as_deref(), Some("k-ed"));
    assert!(published
        .signature
        .as_deref()
        .expect("signature")
        .starts_with("ed25519:v1:"));

    let mut consensus = sample_consensus();
    let policy = MembershipSnapshotRestorePolicy {
        require_signature: true,
        require_signature_key_id: true,
        accepted_signature_key_ids: vec!["k-ed".to_string()],
        ..MembershipSnapshotRestorePolicy::default()
    };
    let restored = sync_client
        .restore_membership_from_dht_verified_with_keyring(
            "w1",
            &mut consensus,
            &dht,
            Some(&keyring),
            &policy,
        )
        .expect("restore membership")
        .expect("restored result");

    assert!(restored.applied);
    assert!(restored.validators.iter().any(|id| id == "seq-4"));
}

#[test]
fn sync_key_revocations_with_policy_accepts_ed25519_signed_revocation() {
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
    let sync_client = MembershipSyncClient::new(Arc::clone(&network));
    let subscription = sync_client.subscribe("w1").expect("subscribe");

    let mut keyring = MembershipDirectorySignerKeyring::new();
    let (private_key_hex, public_key_hex) = ed25519_keypair_hex(31);
    keyring
        .add_ed25519_key("k-ed", private_key_hex.as_str(), public_key_hex.as_str())
        .expect("add key");
    keyring
        .add_hmac_sha256_key("k-old", "legacy-hmac")
        .expect("add old key");
    keyring.set_active_key("k-ed").expect("set active key");

    let _published = sync_client
        .publish_key_revocation_signed_with_keyring(
            "w1",
            "seq-1",
            201,
            "k-old",
            Some("rotate to ed25519".to_string()),
            &keyring,
        )
        .expect("publish key revocation");

    let policy = MembershipRevocationSyncPolicy {
        trusted_requesters: vec!["seq-1".to_string()],
        require_signature: true,
        require_signature_key_id: true,
        accepted_signature_key_ids: vec!["k-ed".to_string()],
        ..MembershipRevocationSyncPolicy::default()
    };
    let report = sync_client
        .sync_key_revocations_with_policy("w1", &subscription, &mut keyring, None, &policy)
        .expect("sync revocations");
    assert_eq!(report.drained, 1);
    assert_eq!(report.applied, 1);
    assert_eq!(report.rejected, 0);
    assert!(keyring.is_key_revoked("k-old"));
}
