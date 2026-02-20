use serde::Deserialize;

use super::{ModuleArtifactIdentity, WorldError};

#[derive(Debug, Deserialize)]
struct BuiltinIdentityManifest {
    modules: Vec<BuiltinIdentityEntry>,
}

#[derive(Debug, Deserialize)]
struct BuiltinIdentityEntry {
    module_id: String,
    source_hash: String,
    build_manifest_hash: String,
    identity_hash: String,
}

pub(crate) fn module_artifact_identity_from_manifest(
    manifest_json: &str,
    manifest_name: &str,
    module_id: &str,
    wasm_hash: &str,
) -> Result<ModuleArtifactIdentity, WorldError> {
    let manifest: BuiltinIdentityManifest =
        serde_json::from_str(manifest_json).map_err(|error| WorldError::ModuleChangeInvalid {
            reason: format!(
                "failed to parse builtin identity manifest {}: {}",
                manifest_name, error
            ),
        })?;

    let Some(entry) = manifest
        .modules
        .into_iter()
        .find(|candidate| candidate.module_id == module_id)
    else {
        return Err(WorldError::ModuleChangeInvalid {
            reason: format!(
                "builtin identity manifest {} missing module_id={}",
                manifest_name, module_id
            ),
        });
    };

    let expected_identity_hash = super::util::sha256_hex(
        format!(
            "{module_id}:{}:{}",
            entry.source_hash, entry.build_manifest_hash
        )
        .as_bytes(),
    );
    if expected_identity_hash != entry.identity_hash {
        return Err(WorldError::ModuleChangeInvalid {
            reason: format!(
                "builtin identity manifest {} has invalid identity_hash module_id={} expected={} actual={}",
                manifest_name, module_id, expected_identity_hash, entry.identity_hash
            ),
        });
    }

    Ok(ModuleArtifactIdentity::unsigned(
        wasm_hash,
        entry.source_hash,
        entry.build_manifest_hash,
    ))
}
