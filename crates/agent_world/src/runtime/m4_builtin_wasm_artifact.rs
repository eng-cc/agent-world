use std::path::{Path, PathBuf};

use super::{HashAlgorithm, LocalCasStore, WorldError};

const M4_BUILTIN_HASH_MANIFEST: &str = include_str!("world/artifacts/m4_builtin_modules.sha256");
const BUILTIN_WASM_DISTFS_ROOT_ENV: &str = "AGENT_WORLD_BUILTIN_WASM_DISTFS_ROOT";

#[cfg(all(test, feature = "wasmtime"))]
pub(crate) fn m4_builtin_module_ids_manifest() -> Vec<&'static str> {
    include_str!("world/artifacts/m4_builtin_module_ids.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

fn builtin_wasm_distfs_root() -> PathBuf {
    if let Ok(path) = std::env::var(BUILTIN_WASM_DISTFS_ROOT_ENV) {
        if !path.trim().is_empty() {
            return PathBuf::from(path);
        }
    }

    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(".distfs")
        .join("builtin_wasm")
}

fn hash_manifest_for_module(module_id: &str) -> Option<&'static str> {
    for line in M4_BUILTIN_HASH_MANIFEST.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(id) = parts.next() else {
            continue;
        };
        let Some(hash) = parts.next() else {
            continue;
        };
        if id == module_id {
            return Some(hash);
        }
    }
    None
}

pub(crate) fn m4_builtin_wasm_module_artifact_bytes(
    module_id: &str,
) -> Result<Vec<u8>, WorldError> {
    let expected_hash =
        hash_manifest_for_module(module_id).ok_or_else(|| WorldError::ModuleChangeInvalid {
            reason: format!("missing builtin wasm hash manifest entry for module_id={module_id}"),
        })?;
    let distfs_root = builtin_wasm_distfs_root();
    let store = LocalCasStore::new_with_hash_algorithm(&distfs_root, HashAlgorithm::Sha256);
    let wasm_bytes = store
        .get_verified(expected_hash)
        .map_err(|error| WorldError::ModuleChangeInvalid {
            reason: format!(
                "failed to load builtin wasm distfs blob for module_id={module_id}, hash={expected_hash}, distfs_root={}, err={error:?}",
                distfs_root.display()
            ),
        })?;

    Ok(wasm_bytes)
}
