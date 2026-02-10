//! Distributed membership directory broadcast and sync helpers.

use std::sync::Arc;

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use super::distributed::topic_membership;
use super::distributed_consensus::{
    ConsensusMembershipChange, ConsensusMembershipChangeRequest, ConsensusMembershipChangeResult,
    QuorumConsensus,
};
use super::distributed_dht::{DistributedDht, MembershipDirectorySnapshot};
use super::distributed_net::{DistributedNetwork, NetworkSubscription};
use super::error::WorldError;
use super::util::to_canonical_cbor;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipDirectoryAnnounce {
    pub world_id: String,
    pub requester_id: String,
    pub requested_at_ms: i64,
    pub reason: Option<String>,
    pub validators: Vec<String>,
    pub quorum_threshold: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl MembershipDirectoryAnnounce {
    pub fn from_membership_change(
        world_id: &str,
        request: &ConsensusMembershipChangeRequest,
        result: &ConsensusMembershipChangeResult,
    ) -> Self {
        Self {
            world_id: world_id.to_string(),
            requester_id: request.requester_id.clone(),
            requested_at_ms: request.requested_at_ms,
            reason: request.reason.clone(),
            validators: result.validators.clone(),
            quorum_threshold: result.quorum_threshold,
            signature: None,
        }
    }

    pub fn into_snapshot(self) -> MembershipDirectorySnapshot {
        MembershipDirectorySnapshot {
            world_id: self.world_id,
            requester_id: self.requester_id,
            requested_at_ms: self.requested_at_ms,
            reason: self.reason,
            validators: self.validators,
            quorum_threshold: self.quorum_threshold,
            signature: self.signature,
        }
    }
}

impl From<MembershipDirectorySnapshot> for MembershipDirectoryAnnounce {
    fn from(value: MembershipDirectorySnapshot) -> Self {
        Self {
            world_id: value.world_id,
            requester_id: value.requester_id,
            requested_at_ms: value.requested_at_ms,
            reason: value.reason,
            validators: value.validators,
            quorum_threshold: value.quorum_threshold,
            signature: value.signature,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MembershipDirectorySigner {
    key: Vec<u8>,
}

impl MembershipDirectorySigner {
    pub fn hmac_sha256(key: impl Into<Vec<u8>>) -> Self {
        Self { key: key.into() }
    }

    pub fn sign_snapshot(
        &self,
        snapshot: &MembershipDirectorySnapshot,
    ) -> Result<String, WorldError> {
        let payload = snapshot_signing_bytes(snapshot)?;
        let mut mac =
            HmacSha256::new_from_slice(&self.key).map_err(|_| WorldError::SignatureKeyInvalid)?;
        mac.update(&payload);
        Ok(hex::encode(mac.finalize().into_bytes()))
    }

    pub fn verify_snapshot(
        &self,
        snapshot: &MembershipDirectorySnapshot,
    ) -> Result<(), WorldError> {
        let Some(signature_hex) = snapshot.signature.as_deref() else {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "membership snapshot missing signature for requester {}",
                    snapshot.requester_id
                ),
            });
        };
        let signature =
            hex::decode(signature_hex).map_err(|_| WorldError::DistributedValidationFailed {
                reason: format!(
                    "membership snapshot signature is not valid hex for requester {}",
                    snapshot.requester_id
                ),
            })?;
        let payload = snapshot_signing_bytes(snapshot)?;
        let mut mac =
            HmacSha256::new_from_slice(&self.key).map_err(|_| WorldError::SignatureKeyInvalid)?;
        mac.update(&payload);
        mac.verify_slice(&signature)
            .map_err(|_| WorldError::DistributedValidationFailed {
                reason: format!(
                    "membership snapshot signature mismatch for requester {}",
                    snapshot.requester_id
                ),
            })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MembershipSnapshotRestorePolicy {
    pub trusted_requesters: Vec<String>,
    pub require_signature: bool,
}

#[derive(Debug, Clone)]
pub struct MembershipSyncSubscription {
    pub membership_sub: NetworkSubscription,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipSyncReport {
    pub drained: usize,
    pub applied: usize,
    pub ignored: usize,
}

#[derive(Clone)]
pub struct MembershipSyncClient {
    network: Arc<dyn DistributedNetwork + Send + Sync>,
}

impl MembershipSyncClient {
    pub fn new(network: Arc<dyn DistributedNetwork + Send + Sync>) -> Self {
        Self { network }
    }

    pub fn subscribe(&self, world_id: &str) -> Result<MembershipSyncSubscription, WorldError> {
        let membership_sub = self.network.subscribe(&topic_membership(world_id))?;
        Ok(MembershipSyncSubscription { membership_sub })
    }

    pub fn publish_membership_change(
        &self,
        world_id: &str,
        request: &ConsensusMembershipChangeRequest,
        result: &ConsensusMembershipChangeResult,
    ) -> Result<MembershipDirectoryAnnounce, WorldError> {
        let announce =
            MembershipDirectoryAnnounce::from_membership_change(world_id, request, result);
        self.publish_announcement(world_id, &announce)?;
        Ok(announce)
    }

    pub fn publish_membership_change_with_dht(
        &self,
        world_id: &str,
        request: &ConsensusMembershipChangeRequest,
        result: &ConsensusMembershipChangeResult,
        dht: &(dyn DistributedDht + Send + Sync),
    ) -> Result<MembershipDirectoryAnnounce, WorldError> {
        let announce = self.publish_membership_change(world_id, request, result)?;
        dht.put_membership_directory(world_id, &announce.clone().into_snapshot())?;
        Ok(announce)
    }

    pub fn publish_membership_change_with_dht_signed(
        &self,
        world_id: &str,
        request: &ConsensusMembershipChangeRequest,
        result: &ConsensusMembershipChangeResult,
        dht: &(dyn DistributedDht + Send + Sync),
        signer: &MembershipDirectorySigner,
    ) -> Result<MembershipDirectoryAnnounce, WorldError> {
        let mut announce =
            MembershipDirectoryAnnounce::from_membership_change(world_id, request, result);
        let mut snapshot = announce.clone().into_snapshot();
        let signature = signer.sign_snapshot(&snapshot)?;
        snapshot.signature = Some(signature.clone());
        announce.signature = Some(signature);

        self.publish_announcement(world_id, &announce)?;
        dht.put_membership_directory(world_id, &snapshot)?;
        Ok(announce)
    }

    fn publish_announcement(
        &self,
        world_id: &str,
        announce: &MembershipDirectoryAnnounce,
    ) -> Result<(), WorldError> {
        let payload = to_canonical_cbor(announce)?;
        self.network.publish(&topic_membership(world_id), &payload)
    }

    pub fn drain_announcements(
        &self,
        subscription: &MembershipSyncSubscription,
    ) -> Result<Vec<MembershipDirectoryAnnounce>, WorldError> {
        let raw = subscription.membership_sub.drain();
        let mut announcements = Vec::with_capacity(raw.len());
        for bytes in raw {
            announcements.push(serde_cbor::from_slice(&bytes)?);
        }
        Ok(announcements)
    }

    pub fn sync_membership_directory(
        &self,
        subscription: &MembershipSyncSubscription,
        consensus: &mut QuorumConsensus,
    ) -> Result<MembershipSyncReport, WorldError> {
        let announcements = self.drain_announcements(subscription)?;
        let mut report = MembershipSyncReport {
            drained: announcements.len(),
            applied: 0,
            ignored: 0,
        };

        for announce in announcements {
            let request = ConsensusMembershipChangeRequest {
                requester_id: announce.requester_id,
                requested_at_ms: announce.requested_at_ms,
                reason: announce.reason,
                change: ConsensusMembershipChange::ReplaceValidators {
                    validators: announce.validators,
                    quorum_threshold: announce.quorum_threshold,
                },
            };
            let result = consensus.apply_membership_change(&request)?;
            if result.applied {
                report.applied = report.applied.saturating_add(1);
            } else {
                report.ignored = report.ignored.saturating_add(1);
            }
        }

        Ok(report)
    }

    pub fn restore_membership_from_dht(
        &self,
        world_id: &str,
        consensus: &mut QuorumConsensus,
        dht: &(dyn DistributedDht + Send + Sync),
    ) -> Result<Option<ConsensusMembershipChangeResult>, WorldError> {
        self.restore_membership_from_dht_verified(
            world_id,
            consensus,
            dht,
            None,
            &MembershipSnapshotRestorePolicy::default(),
        )
    }

    pub fn restore_membership_from_dht_verified(
        &self,
        world_id: &str,
        consensus: &mut QuorumConsensus,
        dht: &(dyn DistributedDht + Send + Sync),
        signer: Option<&MembershipDirectorySigner>,
        policy: &MembershipSnapshotRestorePolicy,
    ) -> Result<Option<ConsensusMembershipChangeResult>, WorldError> {
        let snapshot = dht.get_membership_directory(world_id)?;
        let Some(snapshot) = snapshot else {
            return Ok(None);
        };
        validate_membership_snapshot(world_id, &snapshot, signer, policy)?;

        let announce = MembershipDirectoryAnnounce::from(snapshot);
        let request = ConsensusMembershipChangeRequest {
            requester_id: announce.requester_id,
            requested_at_ms: announce.requested_at_ms,
            reason: announce.reason,
            change: ConsensusMembershipChange::ReplaceValidators {
                validators: announce.validators,
                quorum_threshold: announce.quorum_threshold,
            },
        };
        let result = consensus.apply_membership_change(&request)?;
        Ok(Some(result))
    }
}

#[derive(Serialize)]
struct MembershipDirectorySigningPayload<'a> {
    world_id: &'a str,
    requester_id: &'a str,
    requested_at_ms: i64,
    reason: &'a Option<String>,
    validators: &'a [String],
    quorum_threshold: usize,
}

fn snapshot_signing_bytes(snapshot: &MembershipDirectorySnapshot) -> Result<Vec<u8>, WorldError> {
    let payload = MembershipDirectorySigningPayload {
        world_id: &snapshot.world_id,
        requester_id: &snapshot.requester_id,
        requested_at_ms: snapshot.requested_at_ms,
        reason: &snapshot.reason,
        validators: &snapshot.validators,
        quorum_threshold: snapshot.quorum_threshold,
    };
    to_canonical_cbor(&payload)
}

fn validate_membership_snapshot(
    world_id: &str,
    snapshot: &MembershipDirectorySnapshot,
    signer: Option<&MembershipDirectorySigner>,
    policy: &MembershipSnapshotRestorePolicy,
) -> Result<(), WorldError> {
    if snapshot.world_id != world_id {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "membership snapshot world mismatch: expected={world_id}, got={}",
                snapshot.world_id
            ),
        });
    }

    if !snapshot
        .validators
        .iter()
        .any(|validator| validator == &snapshot.requester_id)
    {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "membership snapshot requester {} is not in validator set",
                snapshot.requester_id
            ),
        });
    }

    if !policy.trusted_requesters.is_empty()
        && !policy
            .trusted_requesters
            .iter()
            .any(|requester| requester == &snapshot.requester_id)
    {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "membership snapshot requester {} is not trusted",
                snapshot.requester_id
            ),
        });
    }

    if policy.require_signature && signer.is_none() {
        return Err(WorldError::DistributedValidationFailed {
            reason: "membership snapshot verification requires signer".to_string(),
        });
    }

    if let Some(signer) = signer {
        if policy.require_signature || snapshot.signature.is_some() {
            signer.verify_snapshot(snapshot)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
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
            signature: None,
        }
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
            .publish_membership_change_with_dht_signed(
                "w1",
                &request,
                &result,
                dht.as_ref(),
                &signer,
            )
            .expect("publish signed");
        assert!(announce.signature.is_some());

        let snapshot = dht
            .get_membership_directory("w1")
            .expect("get membership")
            .expect("snapshot exists");
        assert_eq!(snapshot.signature, announce.signature);
        signer.verify_snapshot(&snapshot).expect("signature valid");

        let drained = sync_client
            .drain_announcements(&subscription)
            .expect("drain announcements");
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0], announce);
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
}
