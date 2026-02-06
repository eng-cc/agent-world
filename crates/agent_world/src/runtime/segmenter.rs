//! Snapshot/journal segmentation helpers for distributed storage.

use serde::{Deserialize, Serialize};

use super::blob_store::{blake3_hex, BlobStore};
use super::distributed::{SnapshotManifest, StateChunkRef};
use super::error::WorldError;
use super::snapshot::{Journal, Snapshot};
use super::util::to_canonical_cbor;

pub const DEFAULT_SNAPSHOT_CHUNK_BYTES: usize = 256 * 1024;
pub const DEFAULT_JOURNAL_EVENTS_PER_SEGMENT: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentConfig {
    pub snapshot_chunk_bytes: usize,
    pub journal_events_per_segment: usize,
}

impl Default for SegmentConfig {
    fn default() -> Self {
        Self {
            snapshot_chunk_bytes: DEFAULT_SNAPSHOT_CHUNK_BYTES,
            journal_events_per_segment: DEFAULT_JOURNAL_EVENTS_PER_SEGMENT,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalSegmentRef {
    pub from_event_id: u64,
    pub to_event_id: u64,
    pub content_hash: String,
    pub size_bytes: u64,
}

pub fn segment_snapshot(
    snapshot: &Snapshot,
    world_id: &str,
    epoch: u64,
    store: &impl BlobStore,
    config: SegmentConfig,
) -> Result<SnapshotManifest, WorldError> {
    let bytes = to_canonical_cbor(snapshot)?;
    let state_root = blake3_hex(&bytes);
    let chunk_size = config.snapshot_chunk_bytes.max(1);
    let mut chunks = Vec::new();

    for (index, chunk) in bytes.chunks(chunk_size).enumerate() {
        let content_hash = store.put_bytes(chunk)?;
        chunks.push(StateChunkRef {
            chunk_id: format!("{epoch}-{index:04}"),
            content_hash,
            size_bytes: chunk.len() as u64,
        });
    }

    Ok(SnapshotManifest {
        world_id: world_id.to_string(),
        epoch,
        chunks,
        state_root,
    })
}

pub fn segment_journal(
    journal: &Journal,
    store: &impl BlobStore,
    config: SegmentConfig,
) -> Result<Vec<JournalSegmentRef>, WorldError> {
    if journal.events.is_empty() {
        return Ok(Vec::new());
    }

    let max_events = config.journal_events_per_segment.max(1);
    let mut segments = Vec::new();

    for chunk in journal.events.chunks(max_events) {
        let from_event_id = chunk.first().map(|event| event.id).unwrap_or(0);
        let to_event_id = chunk.last().map(|event| event.id).unwrap_or(0);
        let bytes = to_canonical_cbor(&chunk)?;
        let content_hash = store.put_bytes(&bytes)?;
        segments.push(JournalSegmentRef {
            from_event_id,
            to_event_id,
            content_hash,
            size_bytes: bytes.len() as u64,
        });
    }

    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::super::{Action, LocalCasStore, World};
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("agent-world-{prefix}-{unique}"))
    }

    #[test]
    fn segment_snapshot_writes_chunks() {
        let dir = temp_dir("snapshot-seg");
        let store = LocalCasStore::new(&dir);
        let world = World::new();
        let snapshot = world.snapshot();

        let manifest = segment_snapshot(
            &snapshot,
            "w1",
            1,
            &store,
            SegmentConfig {
                snapshot_chunk_bytes: 64,
                ..SegmentConfig::default()
            },
        )
        .expect("segment snapshot");

        assert_eq!(manifest.world_id, "w1");
        assert_eq!(manifest.epoch, 1);
        assert!(!manifest.chunks.is_empty());
        for chunk in &manifest.chunks {
            assert!(store.has(&chunk.content_hash).expect("has chunk"));
        }

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn segment_journal_splits_by_event_count() {
        let dir = temp_dir("journal-seg");
        let store = LocalCasStore::new(&dir);
        let mut world = World::new();

        world.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            pos: crate::geometry::GeoPos::new(0.0, 0.0, 0.0),
        });
        world.step().expect("step world");
        world.submit_action(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: crate::geometry::GeoPos::new(1.0, 1.0, 0.0),
        });
        world.step().expect("step world");

        let journal = world.journal().clone();
        let segments = segment_journal(
            &journal,
            &store,
            SegmentConfig {
                journal_events_per_segment: 1,
                ..SegmentConfig::default()
            },
        )
        .expect("segment journal");

        assert_eq!(segments.len(), journal.len());
        for segment in segments {
            assert!(store.has(&segment.content_hash).expect("has segment"));
        }

        let _ = fs::remove_dir_all(&dir);
    }
}
