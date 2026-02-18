use std::path::{Path, PathBuf};

use super::{load_builtin_wasm_with_fetch_fallback, WorldError};

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

fn hash_value_from_manifest_token(token: &'static str) -> Option<&'static str> {
    let value = token
        .split_once('=')
        .map(|(_, hash)| hash)
        .unwrap_or(token)
        .trim();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn hash_manifest_for_module(module_id: &str) -> Option<Vec<&'static str>> {
    for line in M4_BUILTIN_HASH_MANIFEST.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(id) = parts.next() else {
            continue;
        };
        if id == module_id {
            let hashes: Vec<&'static str> =
                parts.filter_map(hash_value_from_manifest_token).collect();
            if !hashes.is_empty() {
                return Some(hashes);
            }
        }
    }
    None
}

pub(crate) fn m4_builtin_wasm_module_artifact_bytes(
    module_id: &str,
) -> Result<Vec<u8>, WorldError> {
    let expected_hashes =
        hash_manifest_for_module(module_id).ok_or_else(|| WorldError::ModuleChangeInvalid {
            reason: format!("missing builtin wasm hash manifest entry for module_id={module_id}"),
        })?;
    let distfs_root = builtin_wasm_distfs_root();
    let wasm_bytes =
        load_builtin_wasm_with_fetch_fallback(module_id, &expected_hashes, &distfs_root).map_err(
            |error| WorldError::ModuleChangeInvalid {
            reason: format!(
                "failed to materialize builtin wasm artifact module_id={module_id}, hashes=[{}], distfs_root={}, err={error:?}",
                expected_hashes.join(","),
                distfs_root.display()
            ),
        },
        )?;

    Ok(wasm_bytes)
}
