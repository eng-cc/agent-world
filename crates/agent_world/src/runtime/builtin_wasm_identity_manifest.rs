use std::collections::BTreeMap;

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
    signer_node_id: String,
    signature_scheme: String,
    artifact_signatures: BTreeMap<String, String>,
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

    let artifact_signature = entry
        .artifact_signatures
        .get(wasm_hash)
        .cloned()
        .ok_or_else(|| WorldError::ModuleChangeInvalid {
            reason: format!(
                "builtin identity manifest {} missing artifact signature module_id={} wasm_hash={}",
                manifest_name, module_id, wasm_hash
            ),
        })?;

    Ok(ModuleArtifactIdentity {
        source_hash: entry.source_hash,
        build_manifest_hash: entry.build_manifest_hash,
        signer_node_id: entry.signer_node_id,
        signature_scheme: entry.signature_scheme,
        artifact_signature,
    })
}
