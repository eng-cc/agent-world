use agent_world_proto::distributed::{SnapshotManifest, StateChunkRef};
use agent_world_proto::distributed_storage::{JournalSegmentRef, SegmentConfig};
use agent_world_proto::world_error::WorldError;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const BLOBS_DIR: &str = "blobs";
const PINS_FILE: &str = "pins.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Blake3,
    Sha256,
}

pub trait BlobStore {
    fn put(&self, content_hash: &str, bytes: &[u8]) -> Result<(), WorldError>;
    fn get(&self, content_hash: &str) -> Result<Vec<u8>, WorldError>;
    fn has(&self, content_hash: &str) -> Result<bool, WorldError>;

    fn put_bytes(&self, bytes: &[u8]) -> Result<String, WorldError> {
        let content_hash = blake3_hex(bytes);
        self.put(&content_hash, bytes)?;
        Ok(content_hash)
    }
}

#[derive(Debug, Clone)]
pub struct LocalCasStore {
    root: PathBuf,
    blobs_dir: PathBuf,
    pins_path: PathBuf,
    hash_algorithm: HashAlgorithm,
}

impl LocalCasStore {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self::new_with_hash_algorithm(root, HashAlgorithm::Blake3)
    }

    pub fn new_with_hash_algorithm(root: impl AsRef<Path>, hash_algorithm: HashAlgorithm) -> Self {
        let root = root.as_ref().to_path_buf();
        let blobs_dir = root.join(BLOBS_DIR);
        let pins_path = root.join(PINS_FILE);
        Self {
            root,
            blobs_dir,
            pins_path,
            hash_algorithm,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn blobs_dir(&self) -> &Path {
        &self.blobs_dir
    }

    pub fn pins_path(&self) -> &Path {
        &self.pins_path
    }

    pub fn hash_algorithm(&self) -> HashAlgorithm {
        self.hash_algorithm
    }

    pub fn get_verified(&self, content_hash: &str) -> Result<Vec<u8>, WorldError> {
        let bytes = self.get(content_hash)?;
        let actual_hash = self.hash_hex(&bytes);
        if actual_hash != content_hash {
            return Err(WorldError::BlobHashMismatch {
                expected: content_hash.to_string(),
                actual: actual_hash,
            });
        }
        Ok(bytes)
    }

    fn hash_hex(&self, bytes: &[u8]) -> String {
        match self.hash_algorithm {
            HashAlgorithm::Blake3 => blake3_hex(bytes),
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(bytes);
                format!("{:x}", hasher.finalize())
            }
        }
    }

    fn ensure_dirs(&self) -> Result<(), WorldError> {
        fs::create_dir_all(&self.root)?;
        fs::create_dir_all(&self.blobs_dir)?;
        Ok(())
    }

    fn blob_path(&self, content_hash: &str) -> Result<PathBuf, WorldError> {
        validate_hash(content_hash)?;
        Ok(self.blobs_dir.join(format!("{content_hash}.blob")))
    }

    fn load_pins(&self) -> Result<PinFile, WorldError> {
        if !self.pins_path.exists() {
            return Ok(PinFile::default());
        }
        read_json_from_path(&self.pins_path)
    }

    fn save_pins(&self, pins: &PinFile) -> Result<(), WorldError> {
        self.ensure_dirs()?;
        write_json_atomic(pins, &self.pins_path)
    }

    pub fn pin(&self, content_hash: &str) -> Result<(), WorldError> {
        validate_hash(content_hash)?;
        if !self.has(content_hash)? {
            return Err(WorldError::BlobNotFound {
                content_hash: content_hash.to_string(),
            });
        }
        let mut pins = self.load_pins()?;
        pins.pins.insert(content_hash.to_string());
        self.save_pins(&pins)
    }

    pub fn unpin(&self, content_hash: &str) -> Result<bool, WorldError> {
        validate_hash(content_hash)?;
        let mut pins = self.load_pins()?;
        let removed = pins.pins.remove(content_hash);
        self.save_pins(&pins)?;
        Ok(removed)
    }

    pub fn list_pins(&self) -> Result<Vec<String>, WorldError> {
        let pins = self.load_pins()?;
        Ok(pins.pins.into_iter().collect())
    }

    pub fn is_pinned(&self, content_hash: &str) -> Result<bool, WorldError> {
        validate_hash(content_hash)?;
        let pins = self.load_pins()?;
        Ok(pins.pins.contains(content_hash))
    }

    pub fn prune_unpinned(&self, max_bytes: u64) -> Result<u64, WorldError> {
        self.ensure_dirs()?;
        let pins = self.load_pins()?.pins;
        let mut total_bytes = 0u64;
        let mut entries = Vec::new();

        if self.blobs_dir.exists() {
            for entry in fs::read_dir(&self.blobs_dir)? {
                let entry = entry?;
                if !entry.file_type()?.is_file() {
                    continue;
                }
                let path = entry.path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("blob") {
                    continue;
                }
                let metadata = entry.metadata()?;
                let size = metadata.len();
                total_bytes = total_bytes.saturating_add(size);
                let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
                let hash = path
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or("")
                    .to_string();
                entries.push(BlobEntry {
                    hash,
                    path,
                    size,
                    modified,
                });
            }
        }

        if total_bytes <= max_bytes {
            return Ok(0);
        }

        entries.sort_by_key(|entry| entry.modified);
        let mut freed = 0u64;
        for entry in entries {
            if total_bytes <= max_bytes {
                break;
            }
            if pins.contains(&entry.hash) {
                continue;
            }
            fs::remove_file(&entry.path)?;
            total_bytes = total_bytes.saturating_sub(entry.size);
            freed = freed.saturating_add(entry.size);
        }
        Ok(freed)
    }
}

impl BlobStore for LocalCasStore {
    fn put(&self, content_hash: &str, bytes: &[u8]) -> Result<(), WorldError> {
        self.ensure_dirs()?;
        let actual_hash = self.hash_hex(bytes);
        if actual_hash != content_hash {
            return Err(WorldError::BlobHashMismatch {
                expected: content_hash.to_string(),
                actual: actual_hash,
            });
        }
        let path = self.blob_path(content_hash)?;
        if path.exists() {
            return Ok(());
        }
        write_bytes_atomic(&path, bytes)?;
        Ok(())
    }

    fn put_bytes(&self, bytes: &[u8]) -> Result<String, WorldError> {
        let content_hash = self.hash_hex(bytes);
        self.put(&content_hash, bytes)?;
        Ok(content_hash)
    }

    fn get(&self, content_hash: &str) -> Result<Vec<u8>, WorldError> {
        let path = self.blob_path(content_hash)?;
        if !path.exists() {
            return Err(WorldError::BlobNotFound {
                content_hash: content_hash.to_string(),
            });
        }
        Ok(fs::read(path)?)
    }

    fn has(&self, content_hash: &str) -> Result<bool, WorldError> {
        let path = self.blob_path(content_hash)?;
        Ok(path.exists())
    }
}

pub fn segment_snapshot<T: Serialize>(
    snapshot: &T,
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

pub fn segment_journal<E: Serialize>(
    events: &[E],
    store: &impl BlobStore,
    config: SegmentConfig,
    event_id_of: impl Fn(&E) -> u64,
) -> Result<Vec<JournalSegmentRef>, WorldError> {
    if events.is_empty() {
        return Ok(Vec::new());
    }

    let max_events = config.journal_events_per_segment.max(1);
    let mut segments = Vec::new();

    for chunk in events.chunks(max_events) {
        let from_event_id = chunk.first().map(&event_id_of).unwrap_or(0);
        let to_event_id = chunk.last().map(&event_id_of).unwrap_or(0);
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

pub fn assemble_snapshot<T: DeserializeOwned>(
    manifest: &SnapshotManifest,
    store: &impl BlobStore,
) -> Result<T, WorldError> {
    let mut bytes = Vec::new();
    for chunk in &manifest.chunks {
        let chunk_bytes = store.get(&chunk.content_hash)?;
        let actual_hash = blake3_hex(&chunk_bytes);
        if actual_hash != chunk.content_hash {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "snapshot chunk hash mismatch: expected={}, actual={}",
                    chunk.content_hash, actual_hash
                ),
            });
        }
        bytes.extend_from_slice(&chunk_bytes);
    }

    let actual_root = blake3_hex(&bytes);
    if actual_root != manifest.state_root {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "snapshot state_root mismatch: expected={}, actual={}",
                manifest.state_root, actual_root
            ),
        });
    }

    Ok(serde_cbor::from_slice(&bytes)?)
}

pub fn assemble_journal<E: DeserializeOwned>(
    segments: &[JournalSegmentRef],
    store: &impl BlobStore,
    event_id_of: impl Fn(&E) -> u64,
) -> Result<Vec<E>, WorldError> {
    let mut events = Vec::new();
    let mut expected_next: Option<u64> = None;

    for segment in segments {
        let segment_bytes = store.get(&segment.content_hash)?;
        let actual_hash = blake3_hex(&segment_bytes);
        if actual_hash != segment.content_hash {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "journal segment hash mismatch: expected={}, actual={}",
                    segment.content_hash, actual_hash
                ),
            });
        }
        let segment_events: Vec<E> = serde_cbor::from_slice(&segment_bytes)?;
        let (first, last) = match (segment_events.first(), segment_events.last()) {
            (Some(first), Some(last)) => (first, last),
            _ => {
                return Err(WorldError::DistributedValidationFailed {
                    reason: "journal segment empty".to_string(),
                });
            }
        };

        let first_id = event_id_of(first);
        let last_id = event_id_of(last);
        if first_id != segment.from_event_id || last_id != segment.to_event_id {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "journal segment range mismatch: segment={}..{}, events={}..{}",
                    segment.from_event_id, segment.to_event_id, first_id, last_id
                ),
            });
        }
        if let Some(expected) = expected_next {
            if first_id != expected {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "journal discontinuity: expected={}, got={}",
                        expected, first_id
                    ),
                });
            }
        }
        expected_next = last_id.checked_add(1);

        events.extend(segment_events);
    }

    Ok(events)
}

pub fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn validate_hash(content_hash: &str) -> Result<(), WorldError> {
    if content_hash.is_empty()
        || content_hash.contains('/')
        || content_hash.contains('\\')
        || content_hash.contains("..")
    {
        return Err(WorldError::BlobHashInvalid {
            content_hash: content_hash.to_string(),
        });
    }
    Ok(())
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct PinFile {
    #[serde(default)]
    pins: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct BlobEntry {
    hash: String,
    path: PathBuf,
    size: u64,
    modified: SystemTime,
}

fn write_bytes_atomic(path: &Path, bytes: &[u8]) -> Result<(), WorldError> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(tmp, path)?;
    Ok(())
}

fn write_json_atomic<T: Serialize>(value: &T, path: &Path) -> Result<(), WorldError> {
    let tmp = path.with_extension("tmp");
    let bytes = serde_json::to_vec_pretty(value)?;
    fs::write(&tmp, bytes)?;
    fs::rename(tmp, path)?;
    Ok(())
}

fn read_json_from_path<T: DeserializeOwned>(path: &Path) -> Result<T, WorldError> {
    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn to_canonical_cbor<T: Serialize>(value: &T) -> Result<Vec<u8>, WorldError> {
    let mut buf = Vec::with_capacity(256);
    let canonical_value = serde_cbor::value::to_value(value)?;
    let mut serializer = serde_cbor::ser::Serializer::new(&mut buf);
    serializer.self_describe()?;
    canonical_value.serialize(&mut serializer)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct DemoEvent {
        id: u64,
        kind: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct DemoSnapshot {
        tick: u64,
        world: String,
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("duration")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-world-distfs-{prefix}-{unique}"))
    }

    #[test]
    fn cas_roundtrip_and_pin() {
        let dir = temp_dir("cas");
        let store = LocalCasStore::new(&dir);

        let bytes = b"hello distfs".to_vec();
        let hash = store.put_bytes(&bytes).expect("put");
        assert!(store.has(&hash).expect("has"));
        assert_eq!(store.get(&hash).expect("get"), bytes);

        store.pin(&hash).expect("pin");
        assert!(store.is_pinned(&hash).expect("is pinned"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn cas_sha256_roundtrip_and_verify() {
        let dir = temp_dir("cas-sha256");
        let store = LocalCasStore::new_with_hash_algorithm(&dir, HashAlgorithm::Sha256);

        let bytes = b"hello sha256 distfs".to_vec();
        let hash = store.put_bytes(&bytes).expect("put");
        assert!(store.has(&hash).expect("has"));
        assert_eq!(store.get_verified(&hash).expect("verified get"), bytes);

        let blob_path = store.blobs_dir().join(format!("{hash}.blob"));
        fs::write(&blob_path, b"tampered").expect("tamper blob");
        assert!(matches!(
            store.get_verified(&hash),
            Err(WorldError::BlobHashMismatch { .. })
        ));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn segment_and_assemble_roundtrip() {
        let dir = temp_dir("segment");
        let store = LocalCasStore::new(&dir);
        let snapshot = DemoSnapshot {
            tick: 42,
            world: "w1".to_string(),
        };
        let events = vec![
            DemoEvent {
                id: 1,
                kind: "a".to_string(),
            },
            DemoEvent {
                id: 2,
                kind: "b".to_string(),
            },
            DemoEvent {
                id: 3,
                kind: "c".to_string(),
            },
        ];

        let manifest = segment_snapshot(
            &snapshot,
            "w1",
            1,
            &store,
            SegmentConfig {
                snapshot_chunk_bytes: 8,
                ..SegmentConfig::default()
            },
        )
        .expect("segment snapshot");

        let segments = segment_journal(
            &events,
            &store,
            SegmentConfig {
                journal_events_per_segment: 2,
                ..SegmentConfig::default()
            },
            |event| event.id,
        )
        .expect("segment journal");

        let snapshot_loaded: DemoSnapshot =
            assemble_snapshot(&manifest, &store).expect("assemble snapshot");
        let events_loaded: Vec<DemoEvent> =
            assemble_journal(&segments, &store, |event: &DemoEvent| event.id)
                .expect("assemble journal");

        assert_eq!(snapshot_loaded, snapshot);
        assert_eq!(events_loaded, events);

        let _ = fs::remove_dir_all(&dir);
    }
}
