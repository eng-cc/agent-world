//! Content-addressed blob storage primitives.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::error::WorldError;
use super::util::{read_json_from_path, write_json_to_path};

const BLOBS_DIR: &str = "blobs";
const PINS_FILE: &str = "pins.json";

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
}

impl LocalCasStore {
    pub fn new(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        let blobs_dir = root.join(BLOBS_DIR);
        let pins_path = root.join(PINS_FILE);
        Self {
            root,
            blobs_dir,
            pins_path,
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
        let actual_hash = blake3_hex(bytes);
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
    write_json_to_path(value, &tmp)?;
    fs::rename(tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("agent-world-{prefix}-{unique}"))
    }

    #[test]
    fn local_cas_put_and_get_round_trip() {
        let dir = temp_dir("blob-store");
        let store = LocalCasStore::new(&dir);
        let bytes = b"hello-blob".to_vec();
        let hash = blake3_hex(&bytes);

        store.put(&hash, &bytes).expect("put blob");
        assert!(store.has(&hash).expect("has blob"));

        let loaded = store.get(&hash).expect("get blob");
        assert_eq!(loaded, bytes);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn local_cas_pin_persists() {
        let dir = temp_dir("blob-store-pin");
        let store = LocalCasStore::new(&dir);
        let bytes = b"pin-blob".to_vec();
        let hash = blake3_hex(&bytes);

        store.put(&hash, &bytes).expect("put blob");
        store.pin(&hash).expect("pin blob");

        let reopened = LocalCasStore::new(&dir);
        assert!(reopened.is_pinned(&hash).expect("is pinned"));

        let removed = reopened.unpin(&hash).expect("unpin");
        assert!(removed);
        assert!(!reopened.is_pinned(&hash).expect("is pinned"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn local_cas_prune_unpinned_keeps_pins() {
        let dir = temp_dir("blob-store-prune");
        let store = LocalCasStore::new(&dir);
        let pinned_bytes = b"pinned".to_vec();
        let pinned_hash = blake3_hex(&pinned_bytes);
        let loose_bytes = b"loose".to_vec();
        let loose_hash = blake3_hex(&loose_bytes);

        store.put(&pinned_hash, &pinned_bytes).expect("put pinned");
        store.put(&loose_hash, &loose_bytes).expect("put loose");
        store.pin(&pinned_hash).expect("pin");

        let freed = store.prune_unpinned(0).expect("prune");
        assert!(freed > 0);
        assert!(store.has(&pinned_hash).expect("has pinned"));
        assert!(!store.has(&loose_hash).expect("has loose"));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn local_cas_put_rejects_hash_mismatch() {
        let dir = temp_dir("blob-store-mismatch");
        let store = LocalCasStore::new(&dir);
        let bytes = b"hello-blob".to_vec();

        let err = store
            .put("deadbeef", &bytes)
            .expect_err("expected hash mismatch");
        assert!(matches!(err, WorldError::BlobHashMismatch { .. }));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn local_cas_get_missing_returns_error() {
        let dir = temp_dir("blob-store-missing");
        let store = LocalCasStore::new(&dir);

        let err = store.get("missing").expect_err("expected missing blob");
        assert!(matches!(err, WorldError::BlobNotFound { .. }));

        let _ = fs::remove_dir_all(&dir);
    }
}
