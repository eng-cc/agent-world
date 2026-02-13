#[cfg(test)]
use super::{world::World, WorldError};

pub(crate) const M1_BUILTIN_WASM_ARTIFACT_BYTES: &[u8] =
    include_bytes!("world/artifacts/m1_builtin_modules.wasm");
pub(crate) const M1_BUILTIN_WASM_ARTIFACT_SHA256: &str =
    include_str!("world/artifacts/m1_builtin_modules.wasm.sha256");

#[cfg(test)]
pub(crate) fn m1_builtin_module_ids_manifest() -> Vec<&'static str> {
    include_str!("world/artifacts/m1_builtin_module_ids.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

#[cfg(test)]
pub(crate) fn m1_builtin_wasm_artifact_hash_hex() -> String {
    super::util::sha256_hex(M1_BUILTIN_WASM_ARTIFACT_BYTES)
}

#[cfg(test)]
pub(crate) fn register_m1_builtin_wasm_artifact(world: &mut World) -> Result<String, WorldError> {
    let wasm_hash = m1_builtin_wasm_artifact_hash_hex();
    world.register_module_artifact(wasm_hash.clone(), M1_BUILTIN_WASM_ARTIFACT_BYTES)?;
    Ok(wasm_hash)
}
