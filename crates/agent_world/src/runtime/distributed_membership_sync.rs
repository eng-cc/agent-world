//! Distributed membership directory broadcast and sync helpers.
use super::distributed::{
    topic_membership, topic_membership_reconcile, topic_membership_revocation,
};
use super::distributed_consensus::{
    ConsensusMembershipChange, ConsensusMembershipChangeRequest, ConsensusMembershipChangeResult,
    QuorumConsensus,
};
use super::distributed_dht::{DistributedDht, MembershipDirectorySnapshot};
use super::distributed_net::{DistributedNetwork, NetworkSubscription};
use super::error::WorldError;
use super::util::to_canonical_cbor;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
type HmacSha256 = Hmac<Sha256>;
#[rustfmt::skip]
pub use reconciliation::{
    FileMembershipRevocationAlertSink, FileMembershipRevocationScheduleStateStore, InMemoryMembershipRevocationAlertSink,
    InMemoryMembershipRevocationScheduleCoordinator, InMemoryMembershipRevocationScheduleStateStore,
    MembershipRevocationAlertDedupPolicy, MembershipRevocationAlertDedupState, MembershipRevocationAlertPolicy,
    MembershipRevocationAlertSeverity, MembershipRevocationAlertSink, MembershipRevocationAnomalyAlert,
    MembershipRevocationCheckpointAnnounce, MembershipRevocationCoordinatedRunReport, MembershipRevocationReconcilePolicy,
    MembershipRevocationReconcileReport, MembershipRevocationReconcileSchedulePolicy, MembershipRevocationReconcileScheduleState,
    MembershipRevocationScheduleCoordinator, MembershipRevocationScheduleStateStore, MembershipRevocationScheduledRunReport,
};
#[rustfmt::skip]
pub use recovery::{
    FileMembershipRevocationAlertDeadLetterStore, FileMembershipRevocationAlertRecoveryStore, FileMembershipRevocationCoordinatorStateStore,
    FileMembershipRevocationDeadLetterReplayPolicyAuditStore, FileMembershipRevocationDeadLetterReplayPolicyStore,
    FileMembershipRevocationDeadLetterReplayRollbackAlertStateStore, FileMembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore,
    FileMembershipRevocationDeadLetterReplayRollbackGovernanceStateStore,
    FileMembershipRevocationDeadLetterReplayStateStore, InMemoryMembershipRevocationAlertDeadLetterStore, InMemoryMembershipRevocationAlertRecoveryStore,
    InMemoryMembershipRevocationCoordinatorStateStore, InMemoryMembershipRevocationDeadLetterReplayPolicyAuditStore, InMemoryMembershipRevocationDeadLetterReplayPolicyStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackAlertStateStore, InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore,
    InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceStateStore,
    InMemoryMembershipRevocationDeadLetterReplayStateStore, MembershipRevocationAlertAckRetryPolicy, MembershipRevocationAlertDeadLetterReason,
    MembershipRevocationAlertDeadLetterRecord, MembershipRevocationAlertDeadLetterStore, MembershipRevocationAlertDeliveryMetrics,
    MembershipRevocationAlertRecoveryReport, MembershipRevocationAlertRecoveryStore, MembershipRevocationCoordinatedRecoveryRunReport,
    MembershipRevocationCoordinatorLeaseState, MembershipRevocationCoordinatorStateStore, MembershipRevocationDeadLetterReplayPolicy,
    MembershipRevocationDeadLetterReplayPolicyAdoptionAuditDecision, MembershipRevocationDeadLetterReplayPolicyAdoptionAuditRecord,
    MembershipRevocationDeadLetterReplayPolicyAuditStore, MembershipRevocationDeadLetterReplayPolicyState, MembershipRevocationDeadLetterReplayPolicyStore,
    MembershipRevocationDeadLetterReplayRollbackAlertPolicy, MembershipRevocationDeadLetterReplayRollbackAlertState,
    MembershipRevocationDeadLetterReplayRollbackAlertStateStore, MembershipRevocationDeadLetterReplayRollbackGovernanceLevel,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord, MembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore,
    MembershipRevocationDeadLetterReplayRollbackGovernancePolicy, MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceState,
    MembershipRevocationDeadLetterReplayRollbackGovernanceStateStore, MembershipRevocationDeadLetterReplayRollbackGuard,
    MembershipRevocationDeadLetterReplayScheduleState, MembershipRevocationDeadLetterReplayStateStore, MembershipRevocationPendingAlert,
    NoopMembershipRevocationAlertDeadLetterStore, StoreBackedMembershipRevocationScheduleCoordinator,
};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipKeyRevocationAnnounce {
    pub world_id: String,
    pub requester_id: String,
    pub requested_at_ms: i64,
    pub key_id: String,
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_key_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
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
        let payload = logic::snapshot_signing_bytes(snapshot)?;
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
        let payload = logic::snapshot_signing_bytes(snapshot)?;
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

    pub fn sign_revocation(
        &self,
        announce: &MembershipKeyRevocationAnnounce,
    ) -> Result<String, WorldError> {
        let payload = logic::revocation_signing_bytes(announce)?;
        let mut mac =
            HmacSha256::new_from_slice(&self.key).map_err(|_| WorldError::SignatureKeyInvalid)?;
        mac.update(&payload);
        Ok(hex::encode(mac.finalize().into_bytes()))
    }

    pub fn verify_revocation(
        &self,
        announce: &MembershipKeyRevocationAnnounce,
    ) -> Result<(), WorldError> {
        let Some(signature_hex) = announce.signature.as_deref() else {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "membership revocation missing signature for requester {}",
                    announce.requester_id
                ),
            });
        };
        let signature =
            hex::decode(signature_hex).map_err(|_| WorldError::DistributedValidationFailed {
                reason: format!(
                    "membership revocation signature is not valid hex for requester {}",
                    announce.requester_id
                ),
            })?;
        let payload = logic::revocation_signing_bytes(announce)?;
        let mut mac =
            HmacSha256::new_from_slice(&self.key).map_err(|_| WorldError::SignatureKeyInvalid)?;
        mac.update(&payload);
        mac.verify_slice(&signature)
            .map_err(|_| WorldError::DistributedValidationFailed {
                reason: format!(
                    "membership revocation signature mismatch for requester {}",
                    announce.requester_id
                ),
            })
    }
}

#[derive(Debug, Clone, Default)]
pub struct MembershipDirectorySignerKeyring {
    active_key_id: Option<String>,
    signers: BTreeMap<String, MembershipDirectorySigner>,
    revoked_key_ids: BTreeSet<String>,
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
        let key_id = logic::normalized_key_id(key_id.into())?;
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
        let key_id = logic::normalized_key_id(key_id.to_string())?;
        if !self.signers.contains_key(&key_id) {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!("membership signing key not found: {key_id}"),
            });
        }
        if self.revoked_key_ids.contains(&key_id) {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!("membership signing key is revoked: {key_id}"),
            });
        }
        self.active_key_id = Some(key_id);
        Ok(())
    }

    pub fn active_key_id(&self) -> Option<&str> {
        self.active_key_id.as_deref()
    }

    pub fn revoke_key(&mut self, key_id: &str) -> Result<bool, WorldError> {
        let key_id = logic::normalized_key_id(key_id.to_string())?;
        let inserted = self.revoked_key_ids.insert(key_id.clone());
        if self.active_key_id.as_deref() == Some(key_id.as_str()) {
            self.active_key_id = None;
        }
        Ok(inserted)
    }

    pub fn is_key_revoked(&self, key_id: &str) -> bool {
        let normalized = key_id.trim();
        if normalized.is_empty() {
            return false;
        }
        self.revoked_key_ids.contains(normalized)
    }

    pub fn revoked_keys(&self) -> Vec<String> {
        self.revoked_key_ids.iter().cloned().collect()
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
        let key_id = logic::normalized_key_id(key_id.to_string())?;
        if self.revoked_key_ids.contains(&key_id) {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!("membership signing key is revoked: {key_id}"),
            });
        }
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

    pub fn sign_revocation_with_active_key(
        &self,
        announce: &MembershipKeyRevocationAnnounce,
    ) -> Result<(String, String), WorldError> {
        let active_key_id = self.active_key_id.as_deref().ok_or_else(|| {
            WorldError::DistributedValidationFailed {
                reason: "membership signing keyring has no active key".to_string(),
            }
        })?;
        let signature = self.sign_revocation_with_key_id(active_key_id, announce)?;
        Ok((active_key_id.to_string(), signature))
    }

    pub fn sign_revocation_with_key_id(
        &self,
        key_id: &str,
        announce: &MembershipKeyRevocationAnnounce,
    ) -> Result<String, WorldError> {
        let key_id = logic::normalized_key_id(key_id.to_string())?;
        if self.revoked_key_ids.contains(&key_id) {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!("membership signing key is revoked: {key_id}"),
            });
        }
        let signer =
            self.signers
                .get(&key_id)
                .ok_or_else(|| WorldError::DistributedValidationFailed {
                    reason: format!("membership signing key not found: {key_id}"),
                })?;
        let mut signable = announce.clone();
        signable.signature_key_id = Some(key_id);
        signable.signature = None;
        signer.sign_revocation(&signable)
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
            let key_id = logic::normalized_key_id(key_id.to_string())?;
            if self.revoked_key_ids.contains(&key_id) {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!("membership signature key_id is revoked: {key_id}"),
                });
            }
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
                if !self.revoked_key_ids.contains(active_key_id) {
                    try_order.push(active_signer);
                }
            }
        }
        for (key_id, signer) in &self.signers {
            if self.active_key_id.as_deref() != Some(key_id.as_str())
                && !self.revoked_key_ids.contains(key_id)
            {
                try_order.push(signer);
            }
        }

        for signer in try_order {
            if signer.verify_snapshot(snapshot).is_ok() {
                return Ok(());
            }
        }

        Err(WorldError::DistributedValidationFailed {
            reason: "membership snapshot verification failed for all non-revoked keys in keyring"
                .to_string(),
        })
    }

    pub fn verify_revocation(
        &self,
        announce: &MembershipKeyRevocationAnnounce,
    ) -> Result<(), WorldError> {
        let Some(signature_hex) = announce.signature.as_deref() else {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "membership revocation missing signature for requester {}",
                    announce.requester_id
                ),
            });
        };
        if signature_hex.is_empty() {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "membership revocation signature is empty for requester {}",
                    announce.requester_id
                ),
            });
        }

        if let Some(key_id) = announce.signature_key_id.as_deref() {
            let key_id = logic::normalized_key_id(key_id.to_string())?;
            if self.revoked_key_ids.contains(&key_id) {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!("membership revocation signature key_id is revoked: {key_id}"),
                });
            }
            let signer = self.signers.get(&key_id).ok_or_else(|| {
                WorldError::DistributedValidationFailed {
                    reason: format!("membership revocation signature key_id is unknown: {key_id}"),
                }
            })?;
            return signer.verify_revocation(announce);
        }

        let mut try_order: Vec<&MembershipDirectorySigner> = Vec::new();
        if let Some(active_key_id) = self.active_key_id.as_deref() {
            if let Some(active_signer) = self.signers.get(active_key_id) {
                if !self.revoked_key_ids.contains(active_key_id) {
                    try_order.push(active_signer);
                }
            }
        }
        for (key_id, signer) in &self.signers {
            if self.active_key_id.as_deref() != Some(key_id.as_str())
                && !self.revoked_key_ids.contains(key_id)
            {
                try_order.push(signer);
            }
        }

        for signer in try_order {
            if signer.verify_revocation(announce).is_ok() {
                return Ok(());
            }
        }

        Err(WorldError::DistributedValidationFailed {
            reason: "membership revocation verification failed for all non-revoked keys in keyring"
                .to_string(),
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MembershipSnapshotRestorePolicy {
    pub trusted_requesters: Vec<String>,
    pub require_signature: bool,
    pub require_signature_key_id: bool,
    pub accepted_signature_key_ids: Vec<String>,
    pub revoked_signature_key_ids: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MembershipRevocationSyncPolicy {
    pub trusted_requesters: Vec<String>,
    pub authorized_requesters: Vec<String>,
    pub require_signature: bool,
    pub require_signature_key_id: bool,
    pub accepted_signature_key_ids: Vec<String>,
    pub revoked_signature_key_ids: Vec<String>,
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

pub trait MembershipAuditStore {
    fn append(&self, record: &MembershipSnapshotAuditRecord) -> Result<(), WorldError>;
    fn list(&self, world_id: &str) -> Result<Vec<MembershipSnapshotAuditRecord>, WorldError>;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryMembershipAuditStore {
    records: Arc<Mutex<Vec<MembershipSnapshotAuditRecord>>>,
}

impl InMemoryMembershipAuditStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MembershipAuditStore for InMemoryMembershipAuditStore {
    fn append(&self, record: &MembershipSnapshotAuditRecord) -> Result<(), WorldError> {
        let mut records = self.records.lock().expect("lock membership audit records");
        records.push(record.clone());
        Ok(())
    }

    fn list(&self, world_id: &str) -> Result<Vec<MembershipSnapshotAuditRecord>, WorldError> {
        let records = self.records.lock().expect("lock membership audit records");
        Ok(records
            .iter()
            .filter(|record| record.world_id == world_id)
            .cloned()
            .collect())
    }
}

#[derive(Debug, Clone)]
pub struct FileMembershipAuditStore {
    root_dir: PathBuf,
}

impl FileMembershipAuditStore {
    pub fn new(root_dir: impl Into<PathBuf>) -> Self {
        Self {
            root_dir: root_dir.into(),
        }
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    fn world_log_path(&self, world_id: &str) -> Result<PathBuf, WorldError> {
        let world_id = logic::normalized_world_id(world_id)?;
        Ok(self.root_dir.join(format!("{world_id}.jsonl")))
    }
}

impl MembershipAuditStore for FileMembershipAuditStore {
    fn append(&self, record: &MembershipSnapshotAuditRecord) -> Result<(), WorldError> {
        let path = self.world_log_path(&record.world_id)?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let line = serde_json::to_string(record)?;
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        file.write_all(line.as_bytes())?;
        file.write_all(
            b"
",
        )?;
        Ok(())
    }

    fn list(&self, world_id: &str) -> Result<Vec<MembershipSnapshotAuditRecord>, WorldError> {
        let path = self.world_log_path(world_id)?;
        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            records.push(serde_json::from_str(&line)?);
        }
        Ok(records)
    }
}

#[derive(Debug, Clone)]
pub struct MembershipSyncSubscription {
    pub membership_sub: NetworkSubscription,
    pub revocation_sub: NetworkSubscription,
    pub reconcile_sub: NetworkSubscription,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipSyncReport {
    pub drained: usize,
    pub applied: usize,
    pub ignored: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipRevocationSyncReport {
    pub drained: usize,
    pub applied: usize,
    pub ignored: usize,
    pub rejected: usize,
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
        let revocation_sub = self
            .network
            .subscribe(&topic_membership_revocation(world_id))?;
        let reconcile_sub = self
            .network
            .subscribe(&topic_membership_reconcile(world_id))?;
        Ok(MembershipSyncSubscription {
            membership_sub,
            revocation_sub,
            reconcile_sub,
        })
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
            let key_id = logic::normalized_key_id(key_id.to_string())?;
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

    pub fn publish_key_revocation(
        &self,
        world_id: &str,
        requester_id: &str,
        requested_at_ms: i64,
        key_id: &str,
        reason: Option<String>,
    ) -> Result<MembershipKeyRevocationAnnounce, WorldError> {
        let announce = Self::build_key_revocation_announce(
            world_id,
            requester_id,
            requested_at_ms,
            key_id,
            reason,
        )?;
        self.publish_revocation(world_id, &announce)?;
        Ok(announce)
    }

    pub fn publish_key_revocation_signed(
        &self,
        world_id: &str,
        requester_id: &str,
        requested_at_ms: i64,
        key_id: &str,
        reason: Option<String>,
        signer: &MembershipDirectorySigner,
    ) -> Result<MembershipKeyRevocationAnnounce, WorldError> {
        self.publish_key_revocation_signed_by_key_id(
            world_id,
            requester_id,
            requested_at_ms,
            key_id,
            reason,
            signer,
            None,
        )
    }

    pub fn publish_key_revocation_signed_by_key_id(
        &self,
        world_id: &str,
        requester_id: &str,
        requested_at_ms: i64,
        key_id: &str,
        reason: Option<String>,
        signer: &MembershipDirectorySigner,
        signature_key_id: Option<&str>,
    ) -> Result<MembershipKeyRevocationAnnounce, WorldError> {
        let mut announce = Self::build_key_revocation_announce(
            world_id,
            requester_id,
            requested_at_ms,
            key_id,
            reason,
        )?;
        if let Some(key_id) = signature_key_id {
            announce.signature_key_id = Some(logic::normalized_key_id(key_id.to_string())?);
        }
        let signature = signer.sign_revocation(&announce)?;
        announce.signature = Some(signature);
        self.publish_revocation(world_id, &announce)?;
        Ok(announce)
    }

    pub fn publish_key_revocation_signed_with_keyring(
        &self,
        world_id: &str,
        requester_id: &str,
        requested_at_ms: i64,
        key_id: &str,
        reason: Option<String>,
        keyring: &MembershipDirectorySignerKeyring,
    ) -> Result<MembershipKeyRevocationAnnounce, WorldError> {
        let mut announce = Self::build_key_revocation_announce(
            world_id,
            requester_id,
            requested_at_ms,
            key_id,
            reason,
        )?;
        let (signature_key_id, signature) = keyring.sign_revocation_with_active_key(&announce)?;
        announce.signature_key_id = Some(signature_key_id);
        announce.signature = Some(signature);
        self.publish_revocation(world_id, &announce)?;
        Ok(announce)
    }

    fn build_key_revocation_announce(
        world_id: &str,
        requester_id: &str,
        requested_at_ms: i64,
        key_id: &str,
        reason: Option<String>,
    ) -> Result<MembershipKeyRevocationAnnounce, WorldError> {
        let key_id = logic::normalized_key_id(key_id.to_string())?;
        Ok(MembershipKeyRevocationAnnounce {
            world_id: world_id.to_string(),
            requester_id: requester_id.to_string(),
            requested_at_ms,
            key_id,
            reason,
            signature_key_id: None,
            signature: None,
        })
    }

    fn publish_announcement(
        &self,
        world_id: &str,
        announce: &MembershipDirectoryAnnounce,
    ) -> Result<(), WorldError> {
        let payload = to_canonical_cbor(announce)?;
        self.network.publish(&topic_membership(world_id), &payload)
    }

    fn publish_revocation(
        &self,
        world_id: &str,
        announce: &MembershipKeyRevocationAnnounce,
    ) -> Result<(), WorldError> {
        let payload = to_canonical_cbor(announce)?;
        self.network
            .publish(&topic_membership_revocation(world_id), &payload)
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

    pub fn drain_key_revocations(
        &self,
        subscription: &MembershipSyncSubscription,
    ) -> Result<Vec<MembershipKeyRevocationAnnounce>, WorldError> {
        let raw = subscription.revocation_sub.drain();
        let mut revocations = Vec::with_capacity(raw.len());
        for bytes in raw {
            revocations.push(serde_cbor::from_slice(&bytes)?);
        }
        Ok(revocations)
    }

    pub fn sync_key_revocations(
        &self,
        subscription: &MembershipSyncSubscription,
        keyring: &mut MembershipDirectorySignerKeyring,
    ) -> Result<usize, WorldError> {
        let revocations = self.drain_key_revocations(subscription)?;
        let mut applied = 0usize;
        for revocation in revocations {
            if keyring.revoke_key(&revocation.key_id)? {
                applied = applied.saturating_add(1);
            }
        }
        Ok(applied)
    }

    pub fn sync_key_revocations_with_policy(
        &self,
        world_id: &str,
        subscription: &MembershipSyncSubscription,
        keyring: &mut MembershipDirectorySignerKeyring,
        signer: Option<&MembershipDirectorySigner>,
        policy: &MembershipRevocationSyncPolicy,
    ) -> Result<MembershipRevocationSyncReport, WorldError> {
        let revocations = self.drain_key_revocations(subscription)?;
        let mut report = MembershipRevocationSyncReport {
            drained: revocations.len(),
            applied: 0,
            ignored: 0,
            rejected: 0,
        };
        let mut verification_keyring = if signer.is_none() {
            Some(keyring.clone())
        } else {
            None
        };
        for revocation in revocations {
            let keyring_ref = verification_keyring.as_ref();
            if let Err(_err) =
                logic::validate_key_revocation(world_id, &revocation, signer, keyring_ref, policy)
            {
                report.rejected = report.rejected.saturating_add(1);
                continue;
            }
            if keyring.revoke_key(&revocation.key_id)? {
                report.applied = report.applied.saturating_add(1);
                if let Some(verifier) = verification_keyring.as_mut() {
                    let _ = verifier.revoke_key(&revocation.key_id);
                }
            } else {
                report.ignored = report.ignored.saturating_add(1);
            }
        }
        Ok(report)
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

        if let Err(err) =
            logic::validate_membership_snapshot(world_id, &snapshot, signer, keyring, policy)
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

    pub fn restore_membership_from_dht_verified_with_audit_store(
        &self,
        world_id: &str,
        consensus: &mut QuorumConsensus,
        dht: &(dyn DistributedDht + Send + Sync),
        signer: Option<&MembershipDirectorySigner>,
        keyring: Option<&MembershipDirectorySignerKeyring>,
        policy: &MembershipSnapshotRestorePolicy,
        audit_store: &(dyn MembershipAuditStore + Send + Sync),
    ) -> Result<MembershipRestoreAuditReport, WorldError> {
        let report = self.restore_membership_from_dht_verified_with_audit(
            world_id, consensus, dht, signer, keyring, policy,
        )?;
        audit_store.append(&report.audit)?;
        Ok(report)
    }
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

#[cfg(test)]
mod coordination_tests;
mod logic;
#[cfg(test)]
mod persistence_tests;
mod reconciliation;
mod recovery;
#[cfg(test)]
mod recovery_replay_policy_audit_tests;
#[cfg(test)]
mod recovery_replay_policy_persistence_tests;
#[cfg(test)]
mod recovery_replay_tests;
#[cfg(test)]
mod recovery_tests;
#[cfg(test)]
mod scheduler_tests;
#[cfg(test)]
mod tests;
