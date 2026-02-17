use std::path::Path;

use agent_world::runtime::LocalCasStore;
use agent_world_distfs::{
    StorageChallengeProbeConfig, StorageChallengeProbeCursorState, StorageChallengeProbeReport,
};
use serde::Serialize;

pub(super) const DEFAULT_REWARD_RUNTIME_DISTFS_PROBE_MAX_SAMPLE_BYTES: u32 = 64 * 1024;
pub(super) const DEFAULT_REWARD_RUNTIME_DISTFS_PROBE_PER_TICK: u32 = 1;
pub(super) const DEFAULT_REWARD_RUNTIME_DISTFS_PROBE_TTL_MS: i64 = 30_000;
pub(super) const DEFAULT_REWARD_RUNTIME_DISTFS_PROBE_ALLOWED_CLOCK_SKEW_MS: i64 = 5_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(super) struct DistfsProbeRuntimeConfig {
    pub max_sample_bytes: u32,
    pub challenges_per_tick: u32,
    pub challenge_ttl_ms: i64,
    pub allowed_clock_skew_ms: i64,
}

impl Default for DistfsProbeRuntimeConfig {
    fn default() -> Self {
        Self {
            max_sample_bytes: DEFAULT_REWARD_RUNTIME_DISTFS_PROBE_MAX_SAMPLE_BYTES,
            challenges_per_tick: DEFAULT_REWARD_RUNTIME_DISTFS_PROBE_PER_TICK,
            challenge_ttl_ms: DEFAULT_REWARD_RUNTIME_DISTFS_PROBE_TTL_MS,
            allowed_clock_skew_ms: DEFAULT_REWARD_RUNTIME_DISTFS_PROBE_ALLOWED_CLOCK_SKEW_MS,
        }
    }
}

pub(super) fn load_reward_runtime_distfs_probe_state(
    path: &Path,
) -> Result<StorageChallengeProbeCursorState, String> {
    if !path.exists() {
        return Ok(StorageChallengeProbeCursorState::default());
    }
    let bytes = std::fs::read(path)
        .map_err(|err| format!("read distfs probe state {} failed: {}", path.display(), err))?;
    serde_json::from_slice::<StorageChallengeProbeCursorState>(bytes.as_slice())
        .map_err(|err| format!("parse distfs probe state {} failed: {}", path.display(), err))
}

pub(super) fn persist_reward_runtime_distfs_probe_state(
    path: &Path,
    state: &StorageChallengeProbeCursorState,
) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(state)
        .map_err(|err| format!("serialize distfs probe state failed: {}", err))?;
    super::write_bytes_atomic(path, bytes.as_slice())
}

pub(super) fn collect_distfs_challenge_report(
    storage_root: &Path,
    world_id: &str,
    node_id: &str,
    observed_at_unix_ms: i64,
    state: &mut StorageChallengeProbeCursorState,
) -> Result<StorageChallengeProbeReport, String> {
    collect_distfs_challenge_report_with_config(
        storage_root,
        world_id,
        node_id,
        observed_at_unix_ms,
        state,
        &DistfsProbeRuntimeConfig::default(),
    )
}

pub(super) fn collect_distfs_challenge_report_with_config(
    storage_root: &Path,
    world_id: &str,
    node_id: &str,
    observed_at_unix_ms: i64,
    state: &mut StorageChallengeProbeCursorState,
    config: &DistfsProbeRuntimeConfig,
) -> Result<StorageChallengeProbeReport, String> {
    let store = LocalCasStore::new(storage_root);
    store
        .probe_storage_challenges_with_cursor(
            world_id,
            node_id,
            observed_at_unix_ms,
            &StorageChallengeProbeConfig {
                max_sample_bytes: config.max_sample_bytes,
                challenges_per_tick: config.challenges_per_tick,
                challenge_ttl_ms: config.challenge_ttl_ms,
                allowed_clock_skew_ms: config.allowed_clock_skew_ms,
            },
            state,
        )
        .map_err(|err| format!("{err:?}"))
}
