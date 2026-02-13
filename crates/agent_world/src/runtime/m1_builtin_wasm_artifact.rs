#[cfg(test)]
use super::{world::World, WorldError};
#[cfg(test)]
use super::{
    M1_BODY_MODULE_ID, M1_MEMORY_MODULE_ID, M1_MOBILITY_MODULE_ID, M1_MOVE_RULE_MODULE_ID,
    M1_RADIATION_POWER_MODULE_ID, M1_SENSOR_MODULE_ID, M1_STORAGE_CARGO_MODULE_ID,
    M1_STORAGE_POWER_MODULE_ID, M1_TRANSFER_RULE_MODULE_ID, M1_VISIBILITY_RULE_MODULE_ID,
};

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

#[cfg(test)]
pub(crate) fn m1_builtin_wasm_module_artifact_bytes(module_id: &str) -> Option<&'static [u8]> {
    match module_id {
        M1_MOVE_RULE_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m1_builtin_modules/m1.rule.move.wasm"
        )),
        M1_VISIBILITY_RULE_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m1_builtin_modules/m1.rule.visibility.wasm"
        )),
        M1_TRANSFER_RULE_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m1_builtin_modules/m1.rule.transfer.wasm"
        )),
        M1_BODY_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m1_builtin_modules/m1.body.core.wasm"
        )),
        M1_SENSOR_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m1_builtin_modules/m1.sensor.basic.wasm"
        )),
        M1_MOBILITY_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m1_builtin_modules/m1.mobility.basic.wasm"
        )),
        M1_MEMORY_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m1_builtin_modules/m1.memory.core.wasm"
        )),
        M1_STORAGE_CARGO_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m1_builtin_modules/m1.storage.cargo.wasm"
        )),
        M1_RADIATION_POWER_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m1_builtin_modules/m1.power.radiation_harvest.wasm"
        )),
        M1_STORAGE_POWER_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m1_builtin_modules/m1.power.storage.wasm"
        )),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) fn register_m1_builtin_wasm_module_artifact(
    world: &mut World,
    module_id: &str,
) -> Result<String, WorldError> {
    let wasm_bytes = m1_builtin_wasm_module_artifact_bytes(module_id).ok_or_else(|| {
        WorldError::ModuleChangeInvalid {
            reason: format!("unsupported m1 builtin wasm module id: {module_id}"),
        }
    })?;
    let wasm_hash = super::util::sha256_hex(wasm_bytes);
    world.register_module_artifact(wasm_hash.clone(), wasm_bytes)?;
    Ok(wasm_hash)
}
