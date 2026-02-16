use serde::{Deserialize, Serialize};

use crate::{blake3_hex, FileMetadata, FileStore};
use agent_world_proto::world_error::WorldError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileReplicationRecord {
    pub world_id: String,
    pub writer_id: String,
    pub sequence: u64,
    pub path: String,
    pub content_hash: String,
    pub size_bytes: u64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SingleWriterReplicationGuard {
    pub writer_id: Option<String>,
    pub last_sequence: u64,
}

impl Default for SingleWriterReplicationGuard {
    fn default() -> Self {
        Self {
            writer_id: None,
            last_sequence: 0,
        }
    }
}

impl SingleWriterReplicationGuard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn validate_and_advance(
        &mut self,
        record: &FileReplicationRecord,
    ) -> Result<(), WorldError> {
        if record.world_id.trim().is_empty() {
            return Err(WorldError::DistributedValidationFailed {
                reason: "replication record world_id cannot be empty".to_string(),
            });
        }
        if record.writer_id.trim().is_empty() {
            return Err(WorldError::DistributedValidationFailed {
                reason: "replication record writer_id cannot be empty".to_string(),
            });
        }
        if record.sequence == 0 {
            return Err(WorldError::DistributedValidationFailed {
                reason: "replication record sequence must be >= 1".to_string(),
            });
        }

        if let Some(existing_writer) = &self.writer_id {
            if existing_writer != &record.writer_id {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "replication writer conflict: expected={}, got={}",
                        existing_writer, record.writer_id
                    ),
                });
            }
        }

        if record.sequence <= self.last_sequence {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "replication sequence not monotonic: last={}, got={}",
                    self.last_sequence, record.sequence
                ),
            });
        }

        self.writer_id = Some(record.writer_id.clone());
        self.last_sequence = record.sequence;
        Ok(())
    }
}

pub fn build_replication_record(
    world_id: &str,
    writer_id: &str,
    sequence: u64,
    path: &str,
    bytes: &[u8],
    updated_at_ms: i64,
) -> Result<FileReplicationRecord, WorldError> {
    if world_id.trim().is_empty() {
        return Err(WorldError::DistributedValidationFailed {
            reason: "world_id cannot be empty".to_string(),
        });
    }
    if writer_id.trim().is_empty() {
        return Err(WorldError::DistributedValidationFailed {
            reason: "writer_id cannot be empty".to_string(),
        });
    }
    if sequence == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: "sequence must be >= 1".to_string(),
        });
    }

    Ok(FileReplicationRecord {
        world_id: world_id.to_string(),
        writer_id: writer_id.to_string(),
        sequence,
        path: path.to_string(),
        content_hash: blake3_hex(bytes),
        size_bytes: bytes.len() as u64,
        updated_at_ms,
    })
}

pub fn apply_replication_record(
    store: &impl FileStore,
    guard: &mut SingleWriterReplicationGuard,
    record: &FileReplicationRecord,
    bytes: &[u8],
) -> Result<FileMetadata, WorldError> {
    guard.validate_and_advance(record)?;

    let computed_hash = blake3_hex(bytes);
    if computed_hash != record.content_hash {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "replication content hash mismatch: expected={}, got={}",
                record.content_hash, computed_hash
            ),
        });
    }

    let metadata = store.write_file(record.path.as_str(), bytes)?;
    if metadata.content_hash != record.content_hash {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "replication write hash mismatch: expected={}, got={}",
                record.content_hash, metadata.content_hash
            ),
        });
    }
    Ok(metadata)
}

pub fn replay_replication_records(
    store: &impl FileStore,
    guard: &mut SingleWriterReplicationGuard,
    entries: &[(FileReplicationRecord, Vec<u8>)],
) -> Result<Vec<FileMetadata>, WorldError> {
    let mut applied = Vec::with_capacity(entries.len());
    for (record, bytes) in entries {
        let metadata = apply_replication_record(store, guard, record, bytes)?;
        applied.push(metadata);
    }
    Ok(applied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LocalCasStore;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("duration")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-world-distfs-replication-{prefix}-{unique}"))
    }

    #[test]
    fn apply_replication_record_accepts_single_writer_monotonic_sequence() {
        let dir = temp_dir("single-writer");
        let store = LocalCasStore::new(&dir);
        let mut guard = SingleWriterReplicationGuard::new();

        let record = build_replication_record("w1", "writer-a", 1, "a/file.txt", b"hello", 10)
            .expect("record");
        let meta = apply_replication_record(&store, &mut guard, &record, b"hello").expect("apply");

        assert_eq!(meta.path, "a/file.txt");
        assert_eq!(guard.writer_id.as_deref(), Some("writer-a"));
        assert_eq!(guard.last_sequence, 1);
        assert_eq!(store.read_file("a/file.txt").expect("read"), b"hello");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_replication_record_rejects_writer_conflict() {
        let dir = temp_dir("writer-conflict");
        let store = LocalCasStore::new(&dir);
        let mut guard = SingleWriterReplicationGuard::new();

        let first =
            build_replication_record("w1", "writer-a", 1, "a/file.txt", b"v1", 10).expect("first");
        apply_replication_record(&store, &mut guard, &first, b"v1").expect("apply first");

        let conflict = build_replication_record("w1", "writer-b", 2, "a/file.txt", b"v2", 20)
            .expect("conflict");
        let result = apply_replication_record(&store, &mut guard, &conflict, b"v2");
        assert!(matches!(
            result,
            Err(WorldError::DistributedValidationFailed { .. })
        ));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_replication_record_rejects_non_monotonic_sequence() {
        let dir = temp_dir("sequence");
        let store = LocalCasStore::new(&dir);
        let mut guard = SingleWriterReplicationGuard::new();

        let first =
            build_replication_record("w1", "writer-a", 2, "a/file.txt", b"v1", 10).expect("first");
        apply_replication_record(&store, &mut guard, &first, b"v1").expect("apply first");

        let stale =
            build_replication_record("w1", "writer-a", 2, "a/file2.txt", b"v2", 20).expect("stale");
        let result = apply_replication_record(&store, &mut guard, &stale, b"v2");
        assert!(matches!(
            result,
            Err(WorldError::DistributedValidationFailed { .. })
        ));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn replay_replication_records_restores_files_in_order() {
        let dir = temp_dir("replay");
        let store = LocalCasStore::new(&dir);
        let mut guard = SingleWriterReplicationGuard::new();

        let entries = vec![
            (
                build_replication_record("w1", "writer-a", 1, "docs/a.txt", b"A", 10).expect("r1"),
                b"A".to_vec(),
            ),
            (
                build_replication_record("w1", "writer-a", 2, "docs/b.txt", b"B", 20).expect("r2"),
                b"B".to_vec(),
            ),
        ];

        let applied = replay_replication_records(&store, &mut guard, &entries).expect("replay");
        assert_eq!(applied.len(), 2);
        assert_eq!(guard.last_sequence, 2);
        assert_eq!(store.read_file("docs/a.txt").expect("read a"), b"A");
        assert_eq!(store.read_file("docs/b.txt").expect("read b"), b"B");

        let _ = fs::remove_dir_all(&dir);
    }
}
