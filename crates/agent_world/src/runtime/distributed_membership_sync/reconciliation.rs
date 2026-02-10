use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::super::distributed::topic_membership_reconcile;
use super::super::error::WorldError;
use super::super::util::{sha256_hex, to_canonical_cbor};
use super::{
    logic, MembershipDirectorySignerKeyring, MembershipSyncClient, MembershipSyncSubscription,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipRevocationCheckpointAnnounce {
    pub world_id: String,
    pub node_id: String,
    pub announced_at_ms: i64,
    pub revoked_key_ids: Vec<String>,
    pub revoked_set_hash: String,
}

impl MembershipRevocationCheckpointAnnounce {
    pub fn from_revoked_keys(
        world_id: &str,
        node_id: &str,
        announced_at_ms: i64,
        revoked_key_ids: Vec<String>,
    ) -> Result<Self, WorldError> {
        let world_id = logic::normalized_world_id(world_id)?;
        let node_id = normalized_node_id(node_id)?;
        let revoked_key_ids = normalize_revoked_key_ids(revoked_key_ids)?;
        let revoked_set_hash = revoked_keys_hash(&revoked_key_ids)?;
        Ok(Self {
            world_id,
            node_id,
            announced_at_ms,
            revoked_key_ids,
            revoked_set_hash,
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MembershipRevocationReconcilePolicy {
    pub trusted_nodes: Vec<String>,
    pub auto_revoke_missing_keys: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipRevocationReconcileReport {
    pub drained: usize,
    pub in_sync: usize,
    pub diverged: usize,
    pub merged: usize,
    pub rejected: usize,
}

impl MembershipSyncClient {
    pub fn publish_revocation_checkpoint(
        &self,
        world_id: &str,
        node_id: &str,
        announced_at_ms: i64,
        keyring: &MembershipDirectorySignerKeyring,
    ) -> Result<MembershipRevocationCheckpointAnnounce, WorldError> {
        let checkpoint = MembershipRevocationCheckpointAnnounce::from_revoked_keys(
            world_id,
            node_id,
            announced_at_ms,
            keyring.revoked_keys(),
        )?;
        let payload = to_canonical_cbor(&checkpoint)?;
        self.network
            .publish(&topic_membership_reconcile(world_id), &payload)?;
        Ok(checkpoint)
    }

    pub fn drain_revocation_checkpoints(
        &self,
        subscription: &MembershipSyncSubscription,
    ) -> Result<Vec<MembershipRevocationCheckpointAnnounce>, WorldError> {
        let raw = subscription.reconcile_sub.drain();
        let mut checkpoints = Vec::with_capacity(raw.len());
        for bytes in raw {
            checkpoints.push(serde_cbor::from_slice(&bytes)?);
        }
        Ok(checkpoints)
    }

    pub fn reconcile_revocations_with_policy(
        &self,
        world_id: &str,
        subscription: &MembershipSyncSubscription,
        keyring: &mut MembershipDirectorySignerKeyring,
        policy: &MembershipRevocationReconcilePolicy,
    ) -> Result<MembershipRevocationReconcileReport, WorldError> {
        let checkpoints = self.drain_revocation_checkpoints(subscription)?;
        let mut report = MembershipRevocationReconcileReport {
            drained: checkpoints.len(),
            in_sync: 0,
            diverged: 0,
            merged: 0,
            rejected: 0,
        };

        for checkpoint in checkpoints {
            let remote = match validate_revocation_checkpoint(world_id, &checkpoint, policy) {
                Ok(remote) => remote,
                Err(_) => {
                    report.rejected = report.rejected.saturating_add(1);
                    continue;
                }
            };
            let local: BTreeSet<String> = keyring.revoked_keys().into_iter().collect();

            if local == remote {
                report.in_sync = report.in_sync.saturating_add(1);
                continue;
            }

            report.diverged = report.diverged.saturating_add(1);
            if policy.auto_revoke_missing_keys {
                for key_id in remote.difference(&local) {
                    if keyring.revoke_key(key_id)? {
                        report.merged = report.merged.saturating_add(1);
                    }
                }
            }
        }

        Ok(report)
    }
}

fn validate_revocation_checkpoint(
    world_id: &str,
    checkpoint: &MembershipRevocationCheckpointAnnounce,
    policy: &MembershipRevocationReconcilePolicy,
) -> Result<BTreeSet<String>, WorldError> {
    if checkpoint.world_id != world_id {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "membership revocation reconcile world mismatch: expected={world_id}, got={}",
                checkpoint.world_id
            ),
        });
    }

    let node_id = normalized_node_id(&checkpoint.node_id)?;
    if !policy.trusted_nodes.is_empty()
        && !policy
            .trusted_nodes
            .iter()
            .any(|trusted| trusted == &node_id)
    {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "membership revocation checkpoint node {} is not trusted",
                node_id
            ),
        });
    }

    let normalized_keys = normalize_revoked_key_ids(checkpoint.revoked_key_ids.clone())?;
    let expected_hash = revoked_keys_hash(&normalized_keys)?;
    if expected_hash != checkpoint.revoked_set_hash {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "membership revocation checkpoint hash mismatch for node {}",
                node_id
            ),
        });
    }

    Ok(normalized_keys.into_iter().collect())
}

fn normalize_revoked_key_ids(raw: Vec<String>) -> Result<Vec<String>, WorldError> {
    let mut normalized = BTreeSet::new();
    for key_id in raw {
        normalized.insert(logic::normalized_key_id(key_id)?);
    }
    Ok(normalized.into_iter().collect())
}

fn revoked_keys_hash(revoked_key_ids: &[String]) -> Result<String, WorldError> {
    let bytes = to_canonical_cbor(&revoked_key_ids)?;
    Ok(sha256_hex(&bytes))
}

fn normalized_node_id(raw: &str) -> Result<String, WorldError> {
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Err(WorldError::DistributedValidationFailed {
            reason: "membership revocation checkpoint node_id cannot be empty".to_string(),
        });
    }
    Ok(normalized.to_string())
}
