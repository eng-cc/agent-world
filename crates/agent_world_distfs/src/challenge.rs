use std::convert::TryFrom;

use agent_world_proto::distributed::{
    StorageChallengeFailureReason, StorageChallengeProofSemantics, StorageChallengeSampleSource,
};
use agent_world_proto::world_error::WorldError;
use serde::{Deserialize, Serialize};

use super::{blake3_hex, validate_hash, LocalCasStore};

pub const STORAGE_CHALLENGE_VERSION: u64 = 1;
pub const STORAGE_CHALLENGE_PROOF_KIND_CHUNK_HASH_V1: &str = "chunk_hash:v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageChallengeRequest {
    pub challenge_id: String,
    pub world_id: String,
    pub node_id: String,
    pub content_hash: String,
    pub max_sample_bytes: u32,
    pub issued_at_unix_ms: i64,
    pub challenge_ttl_ms: i64,
    pub vrf_seed: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageChallenge {
    pub version: u64,
    pub challenge_id: String,
    pub world_id: String,
    pub node_id: String,
    pub content_hash: String,
    pub sample_offset: u64,
    pub sample_size_bytes: u32,
    pub expected_sample_hash: String,
    pub issued_at_unix_ms: i64,
    pub expires_at_unix_ms: i64,
    pub vrf_seed: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageChallengeReceipt {
    pub version: u64,
    pub challenge_id: String,
    pub node_id: String,
    pub content_hash: String,
    pub sample_offset: u64,
    pub sample_size_bytes: u32,
    pub sample_hash: String,
    pub responded_at_unix_ms: i64,
    pub sample_source: StorageChallengeSampleSource,
    pub failure_reason: Option<StorageChallengeFailureReason>,
    pub proof_kind: String,
}

impl LocalCasStore {
    pub fn issue_storage_challenge(
        &self,
        request: &StorageChallengeRequest,
    ) -> Result<StorageChallenge, WorldError> {
        validate_storage_challenge_request(request)?;

        let blob = self.get_verified(request.content_hash.as_str())?;
        let (sample_offset, sample_size_bytes, expected_sample_hash) = sample_window_for_blob(
            request.content_hash.as_str(),
            blob.as_slice(),
            request.max_sample_bytes,
            request.vrf_seed.as_str(),
        )?;
        let expires_at_unix_ms =
            request
                .issued_at_unix_ms
                .checked_add(request.challenge_ttl_ms)
                .ok_or_else(|| WorldError::DistributedValidationFailed {
                    reason: format!(
                        "storage challenge ttl overflow: issued_at={} ttl={}",
                        request.issued_at_unix_ms, request.challenge_ttl_ms
                    ),
                })?;

        Ok(StorageChallenge {
            version: STORAGE_CHALLENGE_VERSION,
            challenge_id: request.challenge_id.clone(),
            world_id: request.world_id.clone(),
            node_id: request.node_id.clone(),
            content_hash: request.content_hash.clone(),
            sample_offset,
            sample_size_bytes,
            expected_sample_hash,
            issued_at_unix_ms: request.issued_at_unix_ms,
            expires_at_unix_ms,
            vrf_seed: request.vrf_seed.clone(),
        })
    }

    pub fn answer_storage_challenge(
        &self,
        challenge: &StorageChallenge,
        responded_at_unix_ms: i64,
    ) -> Result<StorageChallengeReceipt, WorldError> {
        validate_storage_challenge(challenge)?;
        let blob = self.get_verified(challenge.content_hash.as_str())?;
        let sample = extract_sample_slice(
            blob.as_slice(),
            challenge.sample_offset,
            challenge.sample_size_bytes,
        )?;
        let sample_hash = blake3_hex(sample);
        let failure_reason = if sample_hash == challenge.expected_sample_hash {
            None
        } else {
            Some(StorageChallengeFailureReason::HashMismatch)
        };

        Ok(StorageChallengeReceipt {
            version: STORAGE_CHALLENGE_VERSION,
            challenge_id: challenge.challenge_id.clone(),
            node_id: challenge.node_id.clone(),
            content_hash: challenge.content_hash.clone(),
            sample_offset: challenge.sample_offset,
            sample_size_bytes: challenge.sample_size_bytes,
            sample_hash,
            responded_at_unix_ms,
            sample_source: StorageChallengeSampleSource::LocalStoreIndex,
            failure_reason,
            proof_kind: STORAGE_CHALLENGE_PROOF_KIND_CHUNK_HASH_V1.to_string(),
        })
    }
}

pub fn verify_storage_challenge_receipt(
    challenge: &StorageChallenge,
    receipt: &StorageChallengeReceipt,
    allowed_clock_skew_ms: i64,
) -> Result<(), WorldError> {
    if allowed_clock_skew_ms < 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "allowed_clock_skew_ms must be >= 0, got {}",
                allowed_clock_skew_ms
            ),
        });
    }
    validate_storage_challenge(challenge)?;
    validate_storage_challenge_receipt(receipt)?;

    if challenge.challenge_id != receipt.challenge_id {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "challenge_id mismatch: expected={} actual={}",
                challenge.challenge_id, receipt.challenge_id
            ),
        });
    }
    if challenge.node_id != receipt.node_id {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "node_id mismatch: expected={} actual={}",
                challenge.node_id, receipt.node_id
            ),
        });
    }
    if challenge.content_hash != receipt.content_hash {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "content_hash mismatch: expected={} actual={}",
                challenge.content_hash, receipt.content_hash
            ),
        });
    }
    if challenge.sample_offset != receipt.sample_offset {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "sample_offset mismatch: expected={} actual={}",
                challenge.sample_offset, receipt.sample_offset
            ),
        });
    }
    if challenge.sample_size_bytes != receipt.sample_size_bytes {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "sample_size_bytes mismatch: expected={} actual={}",
                challenge.sample_size_bytes, receipt.sample_size_bytes
            ),
        });
    }
    if receipt.sample_hash != challenge.expected_sample_hash {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "sample_hash mismatch: expected={} actual={}",
                challenge.expected_sample_hash, receipt.sample_hash
            ),
        });
    }
    if receipt.proof_kind != STORAGE_CHALLENGE_PROOF_KIND_CHUNK_HASH_V1 {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "unsupported proof kind: expected={} actual={}",
                STORAGE_CHALLENGE_PROOF_KIND_CHUNK_HASH_V1, receipt.proof_kind
            ),
        });
    }
    if receipt.sample_source == StorageChallengeSampleSource::Unknown {
        return Err(WorldError::DistributedValidationFailed {
            reason: "sample_source cannot be Unknown".to_string(),
        });
    }
    if let Some(reason) = receipt.failure_reason {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!("receipt indicates failure: {:?}", reason),
        });
    }

    let min_time = challenge
        .issued_at_unix_ms
        .saturating_sub(allowed_clock_skew_ms);
    let max_time = challenge
        .expires_at_unix_ms
        .saturating_add(allowed_clock_skew_ms);
    if receipt.responded_at_unix_ms < min_time || receipt.responded_at_unix_ms > max_time {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "response timestamp out of challenge window: responded_at={} allowed=[{}, {}]",
                receipt.responded_at_unix_ms, min_time, max_time
            ),
        });
    }

    Ok(())
}

pub fn storage_challenge_receipt_to_proof_semantics(
    challenge: &StorageChallenge,
    receipt: &StorageChallengeReceipt,
) -> StorageChallengeProofSemantics {
    StorageChallengeProofSemantics {
        node_id: receipt.node_id.clone(),
        sample_source: receipt.sample_source,
        sample_reference: challenge_sample_reference(challenge),
        failure_reason: receipt.failure_reason,
        proof_kind_hint: receipt.proof_kind.clone(),
        vrf_seed_hint: Some(challenge.vrf_seed.clone()),
        post_commitment_hint: Some(challenge.expected_sample_hash.clone()),
    }
}

fn validate_storage_challenge_request(request: &StorageChallengeRequest) -> Result<(), WorldError> {
    validate_non_empty_field(request.challenge_id.as_str(), "challenge_id")?;
    validate_non_empty_field(request.world_id.as_str(), "world_id")?;
    validate_non_empty_field(request.node_id.as_str(), "node_id")?;
    validate_non_empty_field(request.vrf_seed.as_str(), "vrf_seed")?;
    validate_hash(request.content_hash.as_str())?;
    if request.max_sample_bytes == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: "max_sample_bytes must be >= 1".to_string(),
        });
    }
    if request.challenge_ttl_ms <= 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: "challenge_ttl_ms must be > 0".to_string(),
        });
    }
    Ok(())
}

fn validate_storage_challenge(challenge: &StorageChallenge) -> Result<(), WorldError> {
    if challenge.version != STORAGE_CHALLENGE_VERSION {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "unsupported storage challenge version: expected={} actual={}",
                STORAGE_CHALLENGE_VERSION, challenge.version
            ),
        });
    }
    validate_non_empty_field(challenge.challenge_id.as_str(), "challenge_id")?;
    validate_non_empty_field(challenge.world_id.as_str(), "world_id")?;
    validate_non_empty_field(challenge.node_id.as_str(), "node_id")?;
    validate_non_empty_field(challenge.vrf_seed.as_str(), "vrf_seed")?;
    validate_hash(challenge.content_hash.as_str())?;
    validate_hash(challenge.expected_sample_hash.as_str())?;
    if challenge.expires_at_unix_ms < challenge.issued_at_unix_ms {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "storage challenge expires_at is earlier than issued_at: issued_at={} expires_at={}",
                challenge.issued_at_unix_ms, challenge.expires_at_unix_ms
            ),
        });
    }
    Ok(())
}

fn validate_storage_challenge_receipt(receipt: &StorageChallengeReceipt) -> Result<(), WorldError> {
    if receipt.version != STORAGE_CHALLENGE_VERSION {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "unsupported storage challenge receipt version: expected={} actual={}",
                STORAGE_CHALLENGE_VERSION, receipt.version
            ),
        });
    }
    validate_non_empty_field(receipt.challenge_id.as_str(), "challenge_id")?;
    validate_non_empty_field(receipt.node_id.as_str(), "node_id")?;
    validate_non_empty_field(receipt.proof_kind.as_str(), "proof_kind")?;
    validate_hash(receipt.content_hash.as_str())?;
    validate_hash(receipt.sample_hash.as_str())?;
    Ok(())
}

fn challenge_sample_reference(challenge: &StorageChallenge) -> String {
    format!(
        "distfs://{}/challenge/{}/blob/{}?offset={}&size={}",
        challenge.node_id,
        challenge.challenge_id,
        challenge.content_hash,
        challenge.sample_offset,
        challenge.sample_size_bytes
    )
}

fn validate_non_empty_field(value: &str, field_name: &str) -> Result<(), WorldError> {
    if value.trim().is_empty() {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!("storage challenge field {} cannot be empty", field_name),
        });
    }
    Ok(())
}

fn sample_window_for_blob(
    content_hash: &str,
    blob: &[u8],
    max_sample_bytes: u32,
    vrf_seed: &str,
) -> Result<(u64, u32, String), WorldError> {
    if max_sample_bytes == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: "max_sample_bytes must be >= 1".to_string(),
        });
    }
    let blob_len = blob.len();
    let sample_size = blob_len.min(max_sample_bytes as usize);
    let sample_size_bytes = u32::try_from(sample_size).map_err(|_| {
        WorldError::DistributedValidationFailed {
            reason: format!(
                "sample size conversion overflow: blob_len={} sample_size={}",
                blob_len, sample_size
            ),
        }
    })?;

    let offset = deterministic_offset(content_hash, vrf_seed, blob_len, sample_size);
    let sample = extract_sample_slice(blob, offset, sample_size_bytes)?;
    let expected_sample_hash = blake3_hex(sample);
    Ok((offset, sample_size_bytes, expected_sample_hash))
}

fn deterministic_offset(content_hash: &str, vrf_seed: &str, blob_len: usize, sample_size: usize) -> u64 {
    if blob_len <= sample_size {
        return 0;
    }
    let mut seed_material = Vec::with_capacity(content_hash.len() + vrf_seed.len() + 1);
    seed_material.extend_from_slice(content_hash.as_bytes());
    seed_material.push(b':');
    seed_material.extend_from_slice(vrf_seed.as_bytes());
    let digest = blake3::hash(seed_material.as_slice());
    let mut prefix = [0u8; 8];
    prefix.copy_from_slice(&digest.as_bytes()[..8]);
    let random_value = u64::from_le_bytes(prefix);
    let max_offset = (blob_len - sample_size) as u64;
    random_value % (max_offset + 1)
}

fn extract_sample_slice(
    blob: &[u8],
    sample_offset: u64,
    sample_size_bytes: u32,
) -> Result<&[u8], WorldError> {
    let offset = usize::try_from(sample_offset).map_err(|_| {
        WorldError::DistributedValidationFailed {
            reason: format!("sample_offset overflow: {}", sample_offset),
        }
    })?;
    let size = sample_size_bytes as usize;
    let end = offset
        .checked_add(size)
        .ok_or_else(|| WorldError::DistributedValidationFailed {
            reason: format!(
                "sample window overflow: offset={} size={}",
                sample_offset, sample_size_bytes
            ),
        })?;
    if end > blob.len() {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "sample window out of bounds: offset={} size={} blob_len={}",
                sample_offset,
                sample_size_bytes,
                blob.len()
            ),
        });
    }
    Ok(&blob[offset..end])
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::BlobStore;

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("duration")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-world-distfs-challenge-{prefix}-{unique}"))
    }

    fn make_blob(size: usize) -> Vec<u8> {
        (0..size)
            .map(|index| ((index % 251) as u8).wrapping_add(3))
            .collect()
    }

    #[test]
    fn issue_storage_challenge_is_deterministic_and_within_bounds() {
        let dir = temp_dir("issue");
        let store = LocalCasStore::new(&dir);
        let bytes = make_blob(96);
        let content_hash = store.put_bytes(bytes.as_slice()).expect("put bytes");

        let request = StorageChallengeRequest {
            challenge_id: "challenge-a".to_string(),
            world_id: "world-1".to_string(),
            node_id: "node-storage-1".to_string(),
            content_hash,
            max_sample_bytes: 32,
            issued_at_unix_ms: 100,
            challenge_ttl_ms: 2_000,
            vrf_seed: "seed-1".to_string(),
        };

        let challenge_a = store.issue_storage_challenge(&request).expect("issue a");
        let challenge_b = store.issue_storage_challenge(&request).expect("issue b");
        assert_eq!(challenge_a, challenge_b);
        assert_eq!(challenge_a.version, STORAGE_CHALLENGE_VERSION);
        assert!(challenge_a.sample_size_bytes <= request.max_sample_bytes);
        assert!(
            challenge_a.sample_offset + challenge_a.sample_size_bytes as u64 <= bytes.len() as u64
        );
        assert_eq!(challenge_a.expires_at_unix_ms, 2_100);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn answer_storage_challenge_returns_matching_receipt() {
        let dir = temp_dir("answer");
        let store = LocalCasStore::new(&dir);
        let bytes = make_blob(128);
        let content_hash = store.put_bytes(bytes.as_slice()).expect("put bytes");

        let request = StorageChallengeRequest {
            challenge_id: "challenge-b".to_string(),
            world_id: "world-1".to_string(),
            node_id: "node-storage-2".to_string(),
            content_hash,
            max_sample_bytes: 24,
            issued_at_unix_ms: 200,
            challenge_ttl_ms: 1_000,
            vrf_seed: "seed-2".to_string(),
        };
        let challenge = store.issue_storage_challenge(&request).expect("issue");
        let receipt = store
            .answer_storage_challenge(&challenge, 250)
            .expect("answer challenge");

        assert_eq!(receipt.version, STORAGE_CHALLENGE_VERSION);
        assert_eq!(receipt.challenge_id, challenge.challenge_id);
        assert_eq!(receipt.node_id, challenge.node_id);
        assert_eq!(receipt.content_hash, challenge.content_hash);
        assert_eq!(receipt.sample_offset, challenge.sample_offset);
        assert_eq!(receipt.sample_size_bytes, challenge.sample_size_bytes);
        assert_eq!(receipt.sample_hash, challenge.expected_sample_hash);
        assert_eq!(receipt.failure_reason, None);
        assert_eq!(
            receipt.proof_kind,
            STORAGE_CHALLENGE_PROOF_KIND_CHUNK_HASH_V1
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_storage_challenge_receipt_accepts_valid_receipt() {
        let dir = temp_dir("verify-valid");
        let store = LocalCasStore::new(&dir);
        let content_hash = store.put_bytes(make_blob(160).as_slice()).expect("put bytes");
        let request = StorageChallengeRequest {
            challenge_id: "challenge-verify".to_string(),
            world_id: "world-1".to_string(),
            node_id: "node-storage-3".to_string(),
            content_hash,
            max_sample_bytes: 40,
            issued_at_unix_ms: 500,
            challenge_ttl_ms: 1_000,
            vrf_seed: "seed-verify".to_string(),
        };
        let challenge = store.issue_storage_challenge(&request).expect("issue challenge");
        let receipt = store
            .answer_storage_challenge(&challenge, 900)
            .expect("answer challenge");
        verify_storage_challenge_receipt(&challenge, &receipt, 50).expect("verify receipt");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_storage_challenge_receipt_rejects_hash_mismatch() {
        let dir = temp_dir("verify-hash-mismatch");
        let store = LocalCasStore::new(&dir);
        let content_hash = store.put_bytes(make_blob(80).as_slice()).expect("put bytes");
        let request = StorageChallengeRequest {
            challenge_id: "challenge-hash-mismatch".to_string(),
            world_id: "world-1".to_string(),
            node_id: "node-storage-4".to_string(),
            content_hash,
            max_sample_bytes: 16,
            issued_at_unix_ms: 1_000,
            challenge_ttl_ms: 500,
            vrf_seed: "seed-hash".to_string(),
        };
        let challenge = store.issue_storage_challenge(&request).expect("issue challenge");
        let mut receipt = store
            .answer_storage_challenge(&challenge, 1_100)
            .expect("answer challenge");
        receipt.sample_hash = blake3_hex(b"tampered");

        let verified = verify_storage_challenge_receipt(&challenge, &receipt, 10);
        assert!(matches!(
            verified,
            Err(WorldError::DistributedValidationFailed { .. })
        ));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_storage_challenge_receipt_rejects_expired_response() {
        let dir = temp_dir("verify-expired");
        let store = LocalCasStore::new(&dir);
        let content_hash = store.put_bytes(make_blob(64).as_slice()).expect("put bytes");
        let request = StorageChallengeRequest {
            challenge_id: "challenge-expired".to_string(),
            world_id: "world-1".to_string(),
            node_id: "node-storage-5".to_string(),
            content_hash,
            max_sample_bytes: 16,
            issued_at_unix_ms: 2_000,
            challenge_ttl_ms: 100,
            vrf_seed: "seed-expired".to_string(),
        };
        let challenge = store.issue_storage_challenge(&request).expect("issue challenge");
        let receipt = store
            .answer_storage_challenge(&challenge, challenge.expires_at_unix_ms + 200)
            .expect("answer challenge");
        let verified = verify_storage_challenge_receipt(&challenge, &receipt, 50);
        assert!(matches!(
            verified,
            Err(WorldError::DistributedValidationFailed { .. })
        ));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn receipt_to_proof_semantics_projects_expected_fields() {
        let dir = temp_dir("proof-semantics");
        let store = LocalCasStore::new(&dir);
        let content_hash = store.put_bytes(make_blob(88).as_slice()).expect("put bytes");
        let request = StorageChallengeRequest {
            challenge_id: "challenge-semantics".to_string(),
            world_id: "world-1".to_string(),
            node_id: "node-storage-6".to_string(),
            content_hash,
            max_sample_bytes: 20,
            issued_at_unix_ms: 3_000,
            challenge_ttl_ms: 100,
            vrf_seed: "seed-semantics".to_string(),
        };
        let challenge = store.issue_storage_challenge(&request).expect("issue challenge");
        let receipt = store
            .answer_storage_challenge(&challenge, 3_050)
            .expect("answer challenge");
        let semantics = storage_challenge_receipt_to_proof_semantics(&challenge, &receipt);

        assert_eq!(semantics.node_id, challenge.node_id);
        assert_eq!(semantics.sample_source, StorageChallengeSampleSource::LocalStoreIndex);
        assert_eq!(
            semantics.sample_reference,
            challenge_sample_reference(&challenge)
        );
        assert_eq!(semantics.failure_reason, None);
        assert_eq!(
            semantics.proof_kind_hint,
            STORAGE_CHALLENGE_PROOF_KIND_CHUNK_HASH_V1
        );
        assert_eq!(
            semantics.vrf_seed_hint.as_deref(),
            Some(challenge.vrf_seed.as_str())
        );
        assert_eq!(
            semantics.post_commitment_hint.as_deref(),
            Some(challenge.expected_sample_hash.as_str())
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn issue_storage_challenge_rejects_invalid_request() {
        let dir = temp_dir("invalid-request");
        let store = LocalCasStore::new(&dir);
        let content_hash = store.put_bytes(b"ok").expect("put bytes");

        let request = StorageChallengeRequest {
            challenge_id: " ".to_string(),
            world_id: "world-1".to_string(),
            node_id: "node-1".to_string(),
            content_hash,
            max_sample_bytes: 0,
            issued_at_unix_ms: 0,
            challenge_ttl_ms: 0,
            vrf_seed: "seed".to_string(),
        };
        let issued = store.issue_storage_challenge(&request);
        assert!(matches!(
            issued,
            Err(WorldError::DistributedValidationFailed { .. })
        ));

        let _ = fs::remove_dir_all(&dir);
    }
}
