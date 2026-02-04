//! Utility functions for the runtime module.

use serde::de::DeserializeOwned;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

use super::error::WorldError;

/// Compute SHA256 hash of a serializable value.
pub fn hash_json<T: Serialize>(value: &T) -> Result<String, WorldError> {
    let bytes = serde_json::to_vec(value)?;
    Ok(sha256_hex(&bytes))
}

/// Compute SHA256 hash of bytes and return as hex string.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Write a serializable value to a JSON file.
pub fn write_json_to_path<T: Serialize>(value: &T, path: &Path) -> Result<(), WorldError> {
    let data = serde_json::to_vec_pretty(value)?;
    fs::write(path, data)?;
    Ok(())
}

/// Read a JSON file and deserialize it.
pub fn read_json_from_path<T: DeserializeOwned>(path: &Path) -> Result<T, WorldError> {
    let data = fs::read(path)?;
    Ok(serde_json::from_slice(&data)?)
}
