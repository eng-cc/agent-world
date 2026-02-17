use std::collections::BTreeMap;

use agent_world_proto::distributed::StorageChallengeFailureReason;
use agent_world_proto::world_error::WorldError;
use serde::{Deserialize, Serialize};

use super::{
    storage_challenge_receipt_to_proof_semantics, verify_storage_challenge_receipt, LocalCasStore,
    StorageChallenge, StorageChallengeProbeConfig, StorageChallengeProbeReport,
    StorageChallengeReceipt, StorageChallengeRequest,
    STORAGE_CHALLENGE_PROOF_KIND_CHUNK_HASH_V1,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StorageChallengeProbeCursorState {
    pub next_blob_cursor: usize,
    pub rounds_executed: u64,
    pub cumulative_total_checks: u64,
    pub cumulative_passed_checks: u64,
    pub cumulative_failed_checks: u64,
    pub cumulative_failure_reasons: BTreeMap<String, u64>,
}

impl LocalCasStore {
    pub fn probe_storage_challenges_with_cursor(
        &self,
        world_id: &str,
        node_id: &str,
        observed_at_unix_ms: i64,
        config: &StorageChallengeProbeConfig,
        state: &mut StorageChallengeProbeCursorState,
    ) -> Result<StorageChallengeProbeReport, WorldError> {
        validate_probe_config(config)?;
        validate_non_empty(world_id, "world_id")?;
        validate_non_empty(node_id, "node_id")?;

        let hashes = self.list_blob_hashes()?;
        let mut report = StorageChallengeProbeReport {
            node_id: node_id.to_string(),
            world_id: world_id.to_string(),
            observed_at_unix_ms,
            total_checks: 0,
            passed_checks: 0,
            failed_checks: 0,
            failure_reasons: BTreeMap::new(),
            latest_proof_semantics: None,
        };

        if hashes.is_empty() {
            advance_probe_cursor_state(state, &report, 0);
            return Ok(report);
        }

        let checks = (config.challenges_per_tick as usize).min(hashes.len());
        let start = state.next_blob_cursor % hashes.len();
        for index in 0..checks {
            let hash = hashes[(start + index) % hashes.len()].clone();
            report.total_checks = report.total_checks.saturating_add(1);
            let request = StorageChallengeRequest {
                challenge_id: build_scheduled_challenge_id(
                    node_id,
                    observed_at_unix_ms,
                    state.next_blob_cursor,
                    index as u32,
                    hash.as_str(),
                ),
                world_id: world_id.to_string(),
                node_id: node_id.to_string(),
                content_hash: hash,
                max_sample_bytes: config.max_sample_bytes,
                issued_at_unix_ms: observed_at_unix_ms,
                challenge_ttl_ms: config.challenge_ttl_ms,
                vrf_seed: build_scheduled_challenge_seed(
                    world_id,
                    node_id,
                    observed_at_unix_ms,
                    state.next_blob_cursor,
                    index as u32,
                ),
            };

            let challenge = match self.issue_storage_challenge(&request) {
                Ok(challenge) => challenge,
                Err(err) => {
                    report.failed_checks = report.failed_checks.saturating_add(1);
                    increment_reason(
                        &mut report.failure_reasons,
                        failure_reason_key(classify_world_error_reason(&err)),
                    );
                    continue;
                }
            };

            let receipt = match self.answer_storage_challenge(&challenge, observed_at_unix_ms) {
                Ok(receipt) => receipt,
                Err(err) => {
                    report.failed_checks = report.failed_checks.saturating_add(1);
                    increment_reason(
                        &mut report.failure_reasons,
                        failure_reason_key(classify_world_error_reason(&err)),
                    );
                    continue;
                }
            };

            match verify_storage_challenge_receipt(
                &challenge,
                &receipt,
                config.allowed_clock_skew_ms,
            ) {
                Ok(()) => {
                    report.passed_checks = report.passed_checks.saturating_add(1);
                    report.latest_proof_semantics = Some(
                        storage_challenge_receipt_to_proof_semantics(&challenge, &receipt),
                    );
                }
                Err(_) => {
                    report.failed_checks = report.failed_checks.saturating_add(1);
                    increment_reason(
                        &mut report.failure_reasons,
                        failure_reason_key(classify_receipt_reason(
                            &challenge,
                            &receipt,
                            config.allowed_clock_skew_ms,
                        )),
                    );
                }
            }
        }

        advance_probe_cursor_state(state, &report, hashes.len());
        Ok(report)
    }
}

fn validate_probe_config(config: &StorageChallengeProbeConfig) -> Result<(), WorldError> {
    if config.max_sample_bytes == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: "probe max_sample_bytes must be >= 1".to_string(),
        });
    }
    if config.challenges_per_tick == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: "probe challenges_per_tick must be >= 1".to_string(),
        });
    }
    if config.challenge_ttl_ms <= 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: "probe challenge_ttl_ms must be > 0".to_string(),
        });
    }
    if config.allowed_clock_skew_ms < 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: "probe allowed_clock_skew_ms must be >= 0".to_string(),
        });
    }
    Ok(())
}

fn validate_non_empty(value: &str, field: &str) -> Result<(), WorldError> {
    if value.trim().is_empty() {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!("probe field {} cannot be empty", field),
        });
    }
    Ok(())
}

fn build_scheduled_challenge_id(
    node_id: &str,
    observed_at_unix_ms: i64,
    cursor: usize,
    probe_index: u32,
    content_hash: &str,
) -> String {
    let prefix_len = content_hash.len().min(12);
    let prefix = &content_hash[..prefix_len];
    format!(
        "sched-probe:{}:{}:{}:{}:{}",
        node_id, observed_at_unix_ms, cursor, probe_index, prefix
    )
}

fn build_scheduled_challenge_seed(
    world_id: &str,
    node_id: &str,
    observed_at_unix_ms: i64,
    cursor: usize,
    probe_index: u32,
) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        world_id, node_id, observed_at_unix_ms, cursor, probe_index
    )
}

fn advance_probe_cursor_state(
    state: &mut StorageChallengeProbeCursorState,
    report: &StorageChallengeProbeReport,
    blob_count: usize,
) {
    state.rounds_executed = state.rounds_executed.saturating_add(1);
    state.cumulative_total_checks = state
        .cumulative_total_checks
        .saturating_add(report.total_checks);
    state.cumulative_passed_checks = state
        .cumulative_passed_checks
        .saturating_add(report.passed_checks);
    state.cumulative_failed_checks = state
        .cumulative_failed_checks
        .saturating_add(report.failed_checks);
    for (reason, count) in &report.failure_reasons {
        let entry = state
            .cumulative_failure_reasons
            .entry(reason.clone())
            .or_insert(0);
        *entry = entry.saturating_add(*count);
    }

    if blob_count == 0 {
        state.next_blob_cursor = 0;
        return;
    }
    let advance = (report.total_checks as usize) % blob_count;
    state.next_blob_cursor = (state.next_blob_cursor + advance) % blob_count;
}

fn increment_reason(map: &mut BTreeMap<String, u64>, key: &str) {
    let entry = map.entry(key.to_string()).or_insert(0);
    *entry = entry.saturating_add(1);
}

fn classify_world_error_reason(error: &WorldError) -> StorageChallengeFailureReason {
    match error {
        WorldError::BlobNotFound { .. } => StorageChallengeFailureReason::MissingSample,
        WorldError::BlobHashMismatch { .. } | WorldError::BlobHashInvalid { .. } => {
            StorageChallengeFailureReason::HashMismatch
        }
        WorldError::Io(_) => StorageChallengeFailureReason::ReadIoError,
        WorldError::DistributedValidationFailed { reason } => {
            let lower = reason.to_ascii_lowercase();
            if lower.contains("timeout") || lower.contains("window") {
                StorageChallengeFailureReason::Timeout
            } else if lower.contains("hash") {
                StorageChallengeFailureReason::HashMismatch
            } else if lower.contains("sample") || lower.contains("missing") {
                StorageChallengeFailureReason::MissingSample
            } else if lower.contains("signature") || lower.contains("proof") {
                StorageChallengeFailureReason::SignatureInvalid
            } else {
                StorageChallengeFailureReason::Unknown
            }
        }
        WorldError::SignatureKeyInvalid => StorageChallengeFailureReason::SignatureInvalid,
        WorldError::NetworkProtocolUnavailable { .. }
        | WorldError::NetworkRequestFailed { .. }
        | WorldError::Serde(_) => StorageChallengeFailureReason::Unknown,
    }
}

fn classify_receipt_reason(
    challenge: &StorageChallenge,
    receipt: &StorageChallengeReceipt,
    allowed_clock_skew_ms: i64,
) -> StorageChallengeFailureReason {
    if let Some(reason) = receipt.failure_reason {
        return reason;
    }
    if receipt.proof_kind != STORAGE_CHALLENGE_PROOF_KIND_CHUNK_HASH_V1 {
        return StorageChallengeFailureReason::SignatureInvalid;
    }
    if challenge.sample_offset != receipt.sample_offset
        || challenge.sample_size_bytes != receipt.sample_size_bytes
    {
        return StorageChallengeFailureReason::MissingSample;
    }
    if challenge.content_hash != receipt.content_hash
        || challenge.expected_sample_hash != receipt.sample_hash
    {
        return StorageChallengeFailureReason::HashMismatch;
    }
    let min_time = challenge
        .issued_at_unix_ms
        .saturating_sub(allowed_clock_skew_ms);
    let max_time = challenge
        .expires_at_unix_ms
        .saturating_add(allowed_clock_skew_ms);
    if receipt.responded_at_unix_ms < min_time || receipt.responded_at_unix_ms > max_time {
        return StorageChallengeFailureReason::Timeout;
    }
    StorageChallengeFailureReason::Unknown
}

fn failure_reason_key(reason: StorageChallengeFailureReason) -> &'static str {
    match reason {
        StorageChallengeFailureReason::MissingSample => "MISSING_SAMPLE",
        StorageChallengeFailureReason::HashMismatch => "HASH_MISMATCH",
        StorageChallengeFailureReason::Timeout => "TIMEOUT",
        StorageChallengeFailureReason::ReadIoError => "READ_IO_ERROR",
        StorageChallengeFailureReason::SignatureInvalid => "SIGNATURE_INVALID",
        StorageChallengeFailureReason::Unknown => "UNKNOWN",
    }
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
        std::env::temp_dir().join(format!("agent-world-distfs-scheduler-{prefix}-{unique}"))
    }

    fn blob(size: usize) -> Vec<u8> {
        (0..size)
            .map(|index| ((index % 251) as u8).wrapping_add(11))
            .collect()
    }

    #[test]
    fn probe_with_cursor_advances_and_accumulates_state() {
        let dir = temp_dir("cursor-advance");
        let store = LocalCasStore::new(&dir);
        let _ = store.put_bytes(blob(80).as_slice()).expect("put 1");
        let _ = store.put_bytes(blob(96).as_slice()).expect("put 2");
        let _ = store.put_bytes(blob(112).as_slice()).expect("put 3");
        let config = StorageChallengeProbeConfig {
            max_sample_bytes: 16,
            challenges_per_tick: 2,
            challenge_ttl_ms: 200,
            allowed_clock_skew_ms: 10,
        };
        let mut state = StorageChallengeProbeCursorState::default();

        let first = store
            .probe_storage_challenges_with_cursor("w1", "node-a", 1_000, &config, &mut state)
            .expect("first round");
        assert_eq!(first.total_checks, 2);
        assert_eq!(first.passed_checks, 2);
        assert_eq!(first.failed_checks, 0);
        assert_eq!(state.rounds_executed, 1);
        assert_eq!(state.next_blob_cursor, 2);
        assert_eq!(state.cumulative_total_checks, 2);
        assert_eq!(state.cumulative_passed_checks, 2);
        assert_eq!(state.cumulative_failed_checks, 0);

        let second = store
            .probe_storage_challenges_with_cursor("w1", "node-a", 2_000, &config, &mut state)
            .expect("second round");
        assert_eq!(second.total_checks, 2);
        assert_eq!(second.passed_checks, 2);
        assert_eq!(second.failed_checks, 0);
        assert_eq!(state.rounds_executed, 2);
        assert_eq!(state.next_blob_cursor, 1);
        assert_eq!(state.cumulative_total_checks, 4);
        assert_eq!(state.cumulative_passed_checks, 4);
        assert_eq!(state.cumulative_failed_checks, 0);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn probe_with_cursor_records_hash_mismatch_failure() {
        let dir = temp_dir("hash-mismatch");
        let store = LocalCasStore::new(&dir);
        let hash = store.put_bytes(blob(64).as_slice()).expect("put");
        let blob_path = store.blobs_dir().join(format!("{hash}.blob"));
        fs::write(blob_path, b"tampered").expect("tamper");

        let config = StorageChallengeProbeConfig {
            max_sample_bytes: 16,
            challenges_per_tick: 1,
            challenge_ttl_ms: 200,
            allowed_clock_skew_ms: 0,
        };
        let mut state = StorageChallengeProbeCursorState::default();
        let report = store
            .probe_storage_challenges_with_cursor("w1", "node-b", 3_000, &config, &mut state)
            .expect("probe");
        assert_eq!(report.total_checks, 1);
        assert_eq!(report.passed_checks, 0);
        assert_eq!(report.failed_checks, 1);
        assert_eq!(report.failure_reasons.get("HASH_MISMATCH").copied(), Some(1));
        assert_eq!(
            state
                .cumulative_failure_reasons
                .get("HASH_MISMATCH")
                .copied(),
            Some(1)
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn probe_with_cursor_allows_empty_blob_set() {
        let dir = temp_dir("empty");
        let store = LocalCasStore::new(&dir);
        let config = StorageChallengeProbeConfig::default();
        let mut state = StorageChallengeProbeCursorState::default();
        let report = store
            .probe_storage_challenges_with_cursor("w1", "node-c", 4_000, &config, &mut state)
            .expect("probe");
        assert_eq!(report.total_checks, 0);
        assert_eq!(report.passed_checks, 0);
        assert_eq!(report.failed_checks, 0);
        assert_eq!(state.rounds_executed, 1);
        assert_eq!(state.next_blob_cursor, 0);
        assert_eq!(state.cumulative_total_checks, 0);
    }
}
