use super::{
    M4_FACTORY_ASSEMBLER_MODULE_ID, M4_FACTORY_MINER_MODULE_ID, M4_FACTORY_SMELTER_MODULE_ID,
    M4_PRODUCT_CONTROL_CHIP_MODULE_ID, M4_PRODUCT_IRON_INGOT_MODULE_ID,
    M4_PRODUCT_LOGISTICS_DRONE_MODULE_ID, M4_PRODUCT_MOTOR_MODULE_ID,
    M4_RECIPE_ASSEMBLE_CONTROL_CHIP_MODULE_ID, M4_RECIPE_ASSEMBLE_DRONE_MODULE_ID,
    M4_RECIPE_ASSEMBLE_GEAR_MODULE_ID, M4_RECIPE_ASSEMBLE_MOTOR_MODULE_ID,
    M4_RECIPE_SMELT_COPPER_WIRE_MODULE_ID, M4_RECIPE_SMELT_IRON_MODULE_ID,
};

#[cfg(all(test, feature = "wasmtime"))]
pub(crate) fn m4_builtin_module_ids_manifest() -> Vec<&'static str> {
    include_str!("world/artifacts/m4_builtin_module_ids.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

pub(crate) fn m4_builtin_wasm_module_artifact_bytes(module_id: &str) -> Option<&'static [u8]> {
    match module_id {
        M4_FACTORY_MINER_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m4_builtin_modules/m4.factory.miner.mk1.wasm"
        )),
        M4_FACTORY_SMELTER_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m4_builtin_modules/m4.factory.smelter.mk1.wasm"
        )),
        M4_FACTORY_ASSEMBLER_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m4_builtin_modules/m4.factory.assembler.mk1.wasm"
        )),
        M4_RECIPE_SMELT_IRON_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m4_builtin_modules/m4.recipe.smelter.iron_ingot.wasm"
        )),
        M4_RECIPE_SMELT_COPPER_WIRE_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m4_builtin_modules/m4.recipe.smelter.copper_wire.wasm"
        )),
        M4_RECIPE_ASSEMBLE_GEAR_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m4_builtin_modules/m4.recipe.assembler.gear.wasm"
        )),
        M4_RECIPE_ASSEMBLE_CONTROL_CHIP_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m4_builtin_modules/m4.recipe.assembler.control_chip.wasm"
        )),
        M4_RECIPE_ASSEMBLE_MOTOR_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m4_builtin_modules/m4.recipe.assembler.motor_mk1.wasm"
        )),
        M4_RECIPE_ASSEMBLE_DRONE_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m4_builtin_modules/m4.recipe.assembler.logistics_drone.wasm"
        )),
        M4_PRODUCT_IRON_INGOT_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m4_builtin_modules/m4.product.material.iron_ingot.wasm"
        )),
        M4_PRODUCT_CONTROL_CHIP_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m4_builtin_modules/m4.product.component.control_chip.wasm"
        )),
        M4_PRODUCT_MOTOR_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m4_builtin_modules/m4.product.component.motor_mk1.wasm"
        )),
        M4_PRODUCT_LOGISTICS_DRONE_MODULE_ID => Some(include_bytes!(
            "world/artifacts/m4_builtin_modules/m4.product.finished.logistics_drone.wasm"
        )),
        _ => None,
    }
}
