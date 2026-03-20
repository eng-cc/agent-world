//! Utility functions for the runtime module.

use serde::de::{DeserializeOwned, Deserializer, Error as DeError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use super::error::WorldError;

pub fn deserialize_btreemap_u64_keys<'de, D, V>(
    deserializer: D,
) -> Result<BTreeMap<u64, V>, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de>,
{
    let raw = BTreeMap::<String, V>::deserialize(deserializer)?;
    raw.into_iter()
        .map(|(key, value)| {
            key.parse::<u64>()
                .map(|parsed| (parsed, value))
                .map_err(|err| D::Error::custom(format!("invalid numeric map key `{key}`: {err}")))
        })
        .collect()
}

/// Compute SHA256 hash of a serializable value.
pub fn hash_json<T: Serialize>(value: &T) -> Result<String, WorldError> {
    let bytes = serde_json::to_vec(value)?;
    Ok(sha256_hex(&bytes))
}

/// Serialize a value into canonical CBOR bytes using deterministic ordering.
pub fn to_canonical_cbor<T: Serialize>(value: &T) -> Result<Vec<u8>, WorldError> {
    let mut buf = Vec::with_capacity(256);
    let canonical_value = serde_cbor::value::to_value(value)?;
    let mut serializer = serde_cbor::ser::Serializer::new(&mut buf);
    serializer.self_describe()?;
    canonical_value.serialize(&mut serializer)?;
    Ok(buf)
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
