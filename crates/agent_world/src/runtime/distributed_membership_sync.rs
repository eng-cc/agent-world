//! Distributed membership directory broadcast and sync helpers.

use std::collections::BTreeMap;
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
    pub signature_key_id: Option<String>,
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
            signature_key_id: None,
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
            signature_key_id: self.signature_key_id,
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
            signature_key_id: value.signature_key_id,
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

#[derive(Debug, Clone, Default)]
pub struct MembershipDirectorySignerKeyring {
    active_key_id: Option<String>,
    signers: BTreeMap<String, MembershipDirectorySigner>,
}

impl MembershipDirectorySignerKeyring {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_hmac_sha256_key(
        &mut self,
        key_id: impl Into<String>,
        key: impl Into<Vec<u8>>,
    ) -> Result<(), WorldError> {
        let key_id = normalized_key_id(key_id.into())?;
        if self.signers.contains_key(&key_id) {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!("membership signing key already exists: {key_id}"),
            });
        }
        self.signers
            .insert(key_id, MembershipDirectorySigner::hmac_sha256(key));
        Ok(())
    }

    pub fn set_active_key(&mut self, key_id: &str) -> Result<(), WorldError> {
        let key_id = normalized_key_id(key_id.to_string())?;
        if !self.signers.contains_key(&key_id) {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!("membership signing key not found: {key_id}"),
            });
        }
        self.active_key_id = Some(key_id);
        Ok(())
    }

    pub fn active_key_id(&self) -> Option<&str> {
        self.active_key_id.as_deref()
    }

    pub fn sign_snapshot_with_active_key(
        &self,
        snapshot: &MembershipDirectorySnapshot,
    ) -> Result<(String, String), WorldError> {
        let active_key_id = self.active_key_id.as_deref().ok_or_else(|| {
            WorldError::DistributedValidationFailed {
                reason: "membership signing keyring has no active key".to_string(),
            }
        })?;
        let signature = self.sign_snapshot_with_key_id(active_key_id, snapshot)?;
        Ok((active_key_id.to_string(), signature))
    }

    pub fn sign_snapshot_with_key_id(
        &self,
        key_id: &str,
        snapshot: &MembershipDirectorySnapshot,
    ) -> Result<String, WorldError> {
        let key_id = normalized_key_id(key_id.to_string())?;
        let signer =
            self.signers
                .get(&key_id)
                .ok_or_else(|| WorldError::DistributedValidationFailed {
                    reason: format!("membership signing key not found: {key_id}"),
                })?;
        let mut signable = snapshot.clone();
        signable.signature_key_id = Some(key_id);
        signable.signature = None;
        signer.sign_snapshot(&signable)
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
        if signature_hex.is_empty() {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "membership snapshot signature is empty for requester {}",
                    snapshot.requester_id
                ),
            });
        }

        if let Some(key_id) = snapshot.signature_key_id.as_deref() {
            let key_id = normalized_key_id(key_id.to_string())?;
            let signer = self.signers.get(&key_id).ok_or_else(|| {
                WorldError::DistributedValidationFailed {
                    reason: format!("membership signature key_id is unknown: {key_id}"),
                }
            })?;
            return signer.verify_snapshot(snapshot);
        }

        let mut try_order: Vec<&MembershipDirectorySigner> = Vec::new();
        if let Some(active_key_id) = self.active_key_id.as_deref() {
            if let Some(active_signer) = self.signers.get(active_key_id) {
                try_order.push(active_signer);
            }
        }
        for (key_id, signer) in &self.signers {
            if self.active_key_id.as_deref() != Some(key_id.as_str()) {
                try_order.push(signer);
            }
        }

        for signer in try_order {
            if signer.verify_snapshot(snapshot).is_ok() {
                return Ok(());
            }
        }

        Err(WorldError::DistributedValidationFailed {
            reason: "membership snapshot verification failed for all keys in keyring".to_string(),
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MembershipSnapshotRestorePolicy {
    pub trusted_requesters: Vec<String>,
    pub require_signature: bool,
    pub require_signature_key_id: bool,
    pub accepted_signature_key_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipSnapshotAuditOutcome {
    MissingSnapshot,
    Applied,
    Ignored,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipSnapshotAuditRecord {
    pub world_id: String,
    pub requester_id: Option<String>,
    pub requested_at_ms: Option<i64>,
    pub signature_key_id: Option<String>,
    pub outcome: MembershipSnapshotAuditOutcome,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipRestoreAuditReport {
    pub restored: Option<ConsensusMembershipChangeResult>,
    pub audit: MembershipSnapshotAuditRecord,
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
        self.publish_membership_change_with_dht_signed_by_key_id(
            world_id, request, result, dht, signer, None,
        )
    }

    pub fn publish_membership_change_with_dht_signed_by_key_id(
        &self,
        world_id: &str,
        request: &ConsensusMembershipChangeRequest,
        result: &ConsensusMembershipChangeResult,
        dht: &(dyn DistributedDht + Send + Sync),
        signer: &MembershipDirectorySigner,
        signature_key_id: Option<&str>,
    ) -> Result<MembershipDirectoryAnnounce, WorldError> {
        let mut announce =
            MembershipDirectoryAnnounce::from_membership_change(world_id, request, result);
        let mut snapshot = announce.clone().into_snapshot();
        if let Some(key_id) = signature_key_id {
            let key_id = normalized_key_id(key_id.to_string())?;
            announce.signature_key_id = Some(key_id.clone());
            snapshot.signature_key_id = Some(key_id);
        }
        let signature = signer.sign_snapshot(&snapshot)?;
        announce.signature = Some(signature.clone());
        snapshot.signature = Some(signature);

        self.publish_announcement(world_id, &announce)?;
        dht.put_membership_directory(world_id, &snapshot)?;
        Ok(announce)
    }

    pub fn publish_membership_change_with_dht_signed_with_keyring(
        &self,
        world_id: &str,
        request: &ConsensusMembershipChangeRequest,
        result: &ConsensusMembershipChangeResult,
        dht: &(dyn DistributedDht + Send + Sync),
        keyring: &MembershipDirectorySignerKeyring,
    ) -> Result<MembershipDirectoryAnnounce, WorldError> {
        let mut announce =
            MembershipDirectoryAnnounce::from_membership_change(world_id, request, result);
        let mut snapshot = announce.clone().into_snapshot();
        let (signature_key_id, signature) = keyring.sign_snapshot_with_active_key(&snapshot)?;
        announce.signature_key_id = Some(signature_key_id.clone());
        announce.signature = Some(signature.clone());
        snapshot.signature_key_id = Some(signature_key_id);
        snapshot.signature = Some(signature);

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
        let report = self.restore_membership_from_dht_verified_with_audit(
            world_id,
            consensus,
            dht,
            None,
            None,
            &MembershipSnapshotRestorePolicy::default(),
        )?;
        restore_result_from_audit(report)
    }

    pub fn restore_membership_from_dht_verified(
        &self,
        world_id: &str,
        consensus: &mut QuorumConsensus,
        dht: &(dyn DistributedDht + Send + Sync),
        signer: Option<&MembershipDirectorySigner>,
        policy: &MembershipSnapshotRestorePolicy,
    ) -> Result<Option<ConsensusMembershipChangeResult>, WorldError> {
        let report = self.restore_membership_from_dht_verified_with_audit(
            world_id, consensus, dht, signer, None, policy,
        )?;
        restore_result_from_audit(report)
    }

    pub fn restore_membership_from_dht_verified_with_keyring(
        &self,
        world_id: &str,
        consensus: &mut QuorumConsensus,
        dht: &(dyn DistributedDht + Send + Sync),
        keyring: Option<&MembershipDirectorySignerKeyring>,
        policy: &MembershipSnapshotRestorePolicy,
    ) -> Result<Option<ConsensusMembershipChangeResult>, WorldError> {
        let report = self.restore_membership_from_dht_verified_with_audit(
            world_id, consensus, dht, None, keyring, policy,
        )?;
        restore_result_from_audit(report)
    }

    pub fn restore_membership_from_dht_verified_with_audit(
        &self,
        world_id: &str,
        consensus: &mut QuorumConsensus,
        dht: &(dyn DistributedDht + Send + Sync),
        signer: Option<&MembershipDirectorySigner>,
        keyring: Option<&MembershipDirectorySignerKeyring>,
        policy: &MembershipSnapshotRestorePolicy,
    ) -> Result<MembershipRestoreAuditReport, WorldError> {
        let snapshot = dht.get_membership_directory(world_id)?;
        let Some(snapshot) = snapshot else {
            return Ok(MembershipRestoreAuditReport {
                restored: None,
                audit: MembershipSnapshotAuditRecord {
                    world_id: world_id.to_string(),
                    requester_id: None,
                    requested_at_ms: None,
                    signature_key_id: None,
                    outcome: MembershipSnapshotAuditOutcome::MissingSnapshot,
                    reason: "membership snapshot not found in dht".to_string(),
                },
            });
        };

        if let Err(err) = validate_membership_snapshot(world_id, &snapshot, signer, keyring, policy)
        {
            return Ok(MembershipRestoreAuditReport {
                restored: None,
                audit: MembershipSnapshotAuditRecord {
                    world_id: world_id.to_string(),
                    requester_id: Some(snapshot.requester_id.clone()),
                    requested_at_ms: Some(snapshot.requested_at_ms),
                    signature_key_id: snapshot.signature_key_id.clone(),
                    outcome: MembershipSnapshotAuditOutcome::Rejected,
                    reason: world_error_reason(&err),
                },
            });
        }

        let request = ConsensusMembershipChangeRequest {
            requester_id: snapshot.requester_id.clone(),
            requested_at_ms: snapshot.requested_at_ms,
            reason: snapshot.reason.clone(),
            change: ConsensusMembershipChange::ReplaceValidators {
                validators: snapshot.validators.clone(),
                quorum_threshold: snapshot.quorum_threshold,
            },
        };

        match consensus.apply_membership_change(&request) {
            Ok(result) => {
                let outcome = if result.applied {
                    MembershipSnapshotAuditOutcome::Applied
                } else {
                    MembershipSnapshotAuditOutcome::Ignored
                };
                let reason = if result.applied {
                    "membership snapshot applied".to_string()
                } else {
                    "membership snapshot ignored (already in sync)".to_string()
                };
                Ok(MembershipRestoreAuditReport {
                    restored: Some(result),
                    audit: MembershipSnapshotAuditRecord {
                        world_id: world_id.to_string(),
                        requester_id: Some(snapshot.requester_id),
                        requested_at_ms: Some(snapshot.requested_at_ms),
                        signature_key_id: snapshot.signature_key_id,
                        outcome,
                        reason,
                    },
                })
            }
            Err(err) => Ok(MembershipRestoreAuditReport {
                restored: None,
                audit: MembershipSnapshotAuditRecord {
                    world_id: world_id.to_string(),
                    requester_id: Some(snapshot.requester_id),
                    requested_at_ms: Some(snapshot.requested_at_ms),
                    signature_key_id: snapshot.signature_key_id,
                    outcome: MembershipSnapshotAuditOutcome::Rejected,
                    reason: world_error_reason(&err),
                },
            }),
        }
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
    signature_key_id: &'a Option<String>,
}

fn snapshot_signing_bytes(snapshot: &MembershipDirectorySnapshot) -> Result<Vec<u8>, WorldError> {
    let payload = MembershipDirectorySigningPayload {
        world_id: &snapshot.world_id,
        requester_id: &snapshot.requester_id,
        requested_at_ms: snapshot.requested_at_ms,
        reason: &snapshot.reason,
        validators: &snapshot.validators,
        quorum_threshold: snapshot.quorum_threshold,
        signature_key_id: &snapshot.signature_key_id,
    };
    to_canonical_cbor(&payload)
}

fn restore_result_from_audit(
    report: MembershipRestoreAuditReport,
) -> Result<Option<ConsensusMembershipChangeResult>, WorldError> {
    match report.audit.outcome {
        MembershipSnapshotAuditOutcome::MissingSnapshot => Ok(None),
        MembershipSnapshotAuditOutcome::Applied | MembershipSnapshotAuditOutcome::Ignored => {
            Ok(report.restored)
        }
        MembershipSnapshotAuditOutcome::Rejected => Err(WorldError::DistributedValidationFailed {
            reason: report.audit.reason,
        }),
    }
}

fn world_error_reason(error: &WorldError) -> String {
    match error {
        WorldError::DistributedValidationFailed { reason } => reason.clone(),
        _ => format!("{error:?}"),
    }
}

fn normalized_key_id(raw: String) -> Result<String, WorldError> {
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Err(WorldError::DistributedValidationFailed {
            reason: "membership signature key_id cannot be empty".to_string(),
        });
    }
    Ok(normalized.to_string())
}

fn validate_membership_snapshot(
    world_id: &str,
    snapshot: &MembershipDirectorySnapshot,
    signer: Option<&MembershipDirectorySigner>,
    keyring: Option<&MembershipDirectorySignerKeyring>,
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

    let has_signature = snapshot.signature.is_some();
    if !has_signature && snapshot.signature_key_id.is_some() {
        return Err(WorldError::DistributedValidationFailed {
            reason: "membership snapshot contains signature_key_id without signature".to_string(),
        });
    }

    if policy.require_signature && !has_signature {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "membership snapshot missing signature for requester {}",
                snapshot.requester_id
            ),
        });
    }

    if policy.require_signature_key_id
        && has_signature
        && snapshot.signature_key_id.as_deref().is_none()
    {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "membership snapshot missing signature_key_id for requester {}",
                snapshot.requester_id
            ),
        });
    }

    if !policy.accepted_signature_key_ids.is_empty() {
        let Some(signature_key_id) = snapshot.signature_key_id.as_deref() else {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "membership snapshot signature_key_id is required for requester {}",
                    snapshot.requester_id
                ),
            });
        };
        if !policy
            .accepted_signature_key_ids
            .iter()
            .any(|key_id| key_id == signature_key_id)
        {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "membership snapshot signature_key_id {} is not accepted",
                    signature_key_id
                ),
            });
        }
    }

    if has_signature {
        if let Some(keyring) = keyring {
            keyring.verify_snapshot(snapshot)?;
        } else if let Some(signer) = signer {
            signer.verify_snapshot(snapshot)?;
        } else if policy.require_signature
            || policy.require_signature_key_id
            || !policy.accepted_signature_key_ids.is_empty()
        {
            return Err(WorldError::DistributedValidationFailed {
                reason: "membership snapshot verification requires signer or keyring".to_string(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
