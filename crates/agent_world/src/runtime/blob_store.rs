//! Content-addressed blob storage primitives.

use std::fs;
use std::path::{Path, PathBuf};

use super::error::WorldError;

const BLOBS_DIR: &str = "blobs";

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
}

impl LocalCasStore {
    pub fn new(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        let blobs_dir = root.join(BLOBS_DIR);
        Self { root, blobs_dir }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn blobs_dir(&self) -> &Path {
        &self.blobs_dir
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

fn write_bytes_atomic(path: &Path, bytes: &[u8]) -> Result<(), WorldError> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes)?;
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

        let err = store
            .get("missing")
            .expect_err("expected missing blob");
        assert!(matches!(err, WorldError::BlobNotFound { .. }));

        let _ = fs::remove_dir_all(&dir);
    }
}
