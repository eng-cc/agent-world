use super::super::{
    m4_builtin_wasm_module_artifact_bytes, util, Manifest, ModuleActivation,
    ModuleArtifactIdentity, ModuleChangeSet, ModuleKind, ModuleLimits, ModuleManifest,
    ModuleRegistry, ModuleRole, ProposalDecision, WorldError, M4_ECONOMY_MODULE_VERSION,
    M4_FACTORY_ASSEMBLER_MODULE_ID, M4_FACTORY_MINER_MODULE_ID, M4_FACTORY_SMELTER_MODULE_ID,
    M4_PRODUCT_CONTROL_CHIP_MODULE_ID, M4_PRODUCT_IRON_INGOT_MODULE_ID,
    M4_PRODUCT_LOGISTICS_DRONE_MODULE_ID, M4_PRODUCT_MOTOR_MODULE_ID,
    M4_RECIPE_ASSEMBLE_CONTROL_CHIP_MODULE_ID, M4_RECIPE_ASSEMBLE_DRONE_MODULE_ID,
    M4_RECIPE_ASSEMBLE_GEAR_MODULE_ID, M4_RECIPE_ASSEMBLE_MOTOR_MODULE_ID,
    M4_RECIPE_SMELT_COPPER_WIRE_MODULE_ID, M4_RECIPE_SMELT_IRON_MODULE_ID,
};
use super::World;

const M4_BOOTSTRAP_WASM_MAX_MEM_BYTES: u64 = 64 * 1024 * 1024;
const M4_BOOTSTRAP_WASM_MAX_GAS: u64 = 2_000_000;
const M4_BOOTSTRAP_BUILD_MANIFEST: &str =
    "toolchain=1.92.0;target=wasm32-unknown-unknown;profile=release;crate=agent_world_builtin_wasm";

impl World {
    pub fn install_m4_economy_bootstrap_modules(
        &mut self,
        actor: impl Into<String>,
    ) -> Result<(), WorldError> {
        let actor = actor.into();
        let mut changes = ModuleChangeSet::default();

        ensure_bootstrap_module(
            self,
            &mut changes,
            M4_FACTORY_MINER_MODULE_ID,
            &m4_builtin_wasm_artifact_for_module(M4_FACTORY_MINER_MODULE_ID)?,
            M4_ECONOMY_MODULE_VERSION,
            m4_factory_miner_manifest,
        )?;
        ensure_bootstrap_module(
            self,
            &mut changes,
            M4_FACTORY_SMELTER_MODULE_ID,
            &m4_builtin_wasm_artifact_for_module(M4_FACTORY_SMELTER_MODULE_ID)?,
            M4_ECONOMY_MODULE_VERSION,
            m4_factory_smelter_manifest,
        )?;
        ensure_bootstrap_module(
            self,
            &mut changes,
            M4_FACTORY_ASSEMBLER_MODULE_ID,
            &m4_builtin_wasm_artifact_for_module(M4_FACTORY_ASSEMBLER_MODULE_ID)?,
            M4_ECONOMY_MODULE_VERSION,
            m4_factory_assembler_manifest,
        )?;
        ensure_bootstrap_module(
            self,
            &mut changes,
            M4_RECIPE_SMELT_IRON_MODULE_ID,
            &m4_builtin_wasm_artifact_for_module(M4_RECIPE_SMELT_IRON_MODULE_ID)?,
            M4_ECONOMY_MODULE_VERSION,
            m4_recipe_smelt_iron_manifest,
        )?;
        ensure_bootstrap_module(
            self,
            &mut changes,
            M4_RECIPE_SMELT_COPPER_WIRE_MODULE_ID,
            &m4_builtin_wasm_artifact_for_module(M4_RECIPE_SMELT_COPPER_WIRE_MODULE_ID)?,
            M4_ECONOMY_MODULE_VERSION,
            m4_recipe_smelt_copper_wire_manifest,
        )?;
        ensure_bootstrap_module(
            self,
            &mut changes,
            M4_RECIPE_ASSEMBLE_GEAR_MODULE_ID,
            &m4_builtin_wasm_artifact_for_module(M4_RECIPE_ASSEMBLE_GEAR_MODULE_ID)?,
            M4_ECONOMY_MODULE_VERSION,
            m4_recipe_assemble_gear_manifest,
        )?;
        ensure_bootstrap_module(
            self,
            &mut changes,
            M4_RECIPE_ASSEMBLE_CONTROL_CHIP_MODULE_ID,
            &m4_builtin_wasm_artifact_for_module(M4_RECIPE_ASSEMBLE_CONTROL_CHIP_MODULE_ID)?,
            M4_ECONOMY_MODULE_VERSION,
            m4_recipe_assemble_control_chip_manifest,
        )?;
        ensure_bootstrap_module(
            self,
            &mut changes,
            M4_RECIPE_ASSEMBLE_MOTOR_MODULE_ID,
            &m4_builtin_wasm_artifact_for_module(M4_RECIPE_ASSEMBLE_MOTOR_MODULE_ID)?,
            M4_ECONOMY_MODULE_VERSION,
            m4_recipe_assemble_motor_manifest,
        )?;
        ensure_bootstrap_module(
            self,
            &mut changes,
            M4_RECIPE_ASSEMBLE_DRONE_MODULE_ID,
            &m4_builtin_wasm_artifact_for_module(M4_RECIPE_ASSEMBLE_DRONE_MODULE_ID)?,
            M4_ECONOMY_MODULE_VERSION,
            m4_recipe_assemble_drone_manifest,
        )?;
        ensure_bootstrap_module(
            self,
            &mut changes,
            M4_PRODUCT_IRON_INGOT_MODULE_ID,
            &m4_builtin_wasm_artifact_for_module(M4_PRODUCT_IRON_INGOT_MODULE_ID)?,
            M4_ECONOMY_MODULE_VERSION,
            m4_product_iron_ingot_manifest,
        )?;
        ensure_bootstrap_module(
            self,
            &mut changes,
            M4_PRODUCT_CONTROL_CHIP_MODULE_ID,
            &m4_builtin_wasm_artifact_for_module(M4_PRODUCT_CONTROL_CHIP_MODULE_ID)?,
            M4_ECONOMY_MODULE_VERSION,
            m4_product_control_chip_manifest,
        )?;
        ensure_bootstrap_module(
            self,
            &mut changes,
            M4_PRODUCT_MOTOR_MODULE_ID,
            &m4_builtin_wasm_artifact_for_module(M4_PRODUCT_MOTOR_MODULE_ID)?,
            M4_ECONOMY_MODULE_VERSION,
            m4_product_motor_manifest,
        )?;
        ensure_bootstrap_module(
            self,
            &mut changes,
            M4_PRODUCT_LOGISTICS_DRONE_MODULE_ID,
            &m4_builtin_wasm_artifact_for_module(M4_PRODUCT_LOGISTICS_DRONE_MODULE_ID)?,
            M4_ECONOMY_MODULE_VERSION,
            m4_product_logistics_drone_manifest,
        )?;

        if changes.is_empty() {
            return Ok(());
        }

        let mut content = serde_json::Map::new();
        content.insert(
            "module_changes".to_string(),
            serde_json::to_value(&changes)?,
        );
        let manifest = Manifest {
            version: self.manifest.version.saturating_add(1),
            content: serde_json::Value::Object(content),
        };

        let proposal_id = self.propose_manifest_update(manifest, actor.clone())?;
        self.shadow_proposal(proposal_id)?;
        self.approve_proposal(proposal_id, actor.clone(), ProposalDecision::Approve)?;
        self.apply_proposal(proposal_id)?;

        Ok(())
    }
}

fn m4_builtin_wasm_artifact_for_module(module_id: &str) -> Result<Vec<u8>, WorldError> {
    m4_builtin_wasm_module_artifact_bytes(module_id)
}

fn m4_bootstrap_artifact_identity(module_id: &str, wasm_hash: &str) -> ModuleArtifactIdentity {
    let source_hash = util::sha256_hex(format!("builtin-source:{module_id}").as_bytes());
    let build_manifest_hash = util::sha256_hex(M4_BOOTSTRAP_BUILD_MANIFEST.as_bytes());
    ModuleArtifactIdentity::unsigned(wasm_hash, source_hash, build_manifest_hash)
}

fn ensure_bootstrap_module(
    world: &mut World,
    changes: &mut ModuleChangeSet,
    module_id: &str,
    artifact: &[u8],
    version: &str,
    make_manifest: fn(String) -> ModuleManifest,
) -> Result<(), WorldError> {
    if world
        .module_registry
        .active
        .get(module_id)
        .is_some_and(|active_version| active_version == version)
    {
        return Ok(());
    }

    let record_key = ModuleRegistry::record_key(module_id, version);
    if !world.module_registry.records.contains_key(&record_key) {
        let wasm_hash = util::sha256_hex(artifact);
        world.register_module_artifact(wasm_hash.clone(), artifact)?;
        changes.register.push(make_manifest(wasm_hash));
    }

    changes.activate.push(ModuleActivation {
        module_id: module_id.to_string(),
        version: version.to_string(),
    });

    Ok(())
}

fn m4_manifest(
    module_id: &str,
    name: &str,
    wasm_hash: String,
    max_call_rate: u32,
) -> ModuleManifest {
    ModuleManifest {
        module_id: module_id.to_string(),
        name: name.to_string(),
        version: M4_ECONOMY_MODULE_VERSION.to_string(),
        kind: ModuleKind::Pure,
        role: ModuleRole::Domain,
        artifact_identity: Some(m4_bootstrap_artifact_identity(module_id, &wasm_hash)),
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        exports: vec!["call".to_string()],
        subscriptions: Vec::new(),
        required_caps: Vec::new(),
        limits: ModuleLimits {
            max_mem_bytes: M4_BOOTSTRAP_WASM_MAX_MEM_BYTES,
            max_gas: M4_BOOTSTRAP_WASM_MAX_GAS,
            max_call_rate,
            max_output_bytes: 4096,
            max_effects: 0,
            max_emits: 2,
        },
    }
}

fn m4_factory_miner_manifest(wasm_hash: String) -> ModuleManifest {
    m4_manifest(
        M4_FACTORY_MINER_MODULE_ID,
        "M4FactoryMinerMk1",
        wasm_hash,
        32,
    )
}

fn m4_factory_smelter_manifest(wasm_hash: String) -> ModuleManifest {
    m4_manifest(
        M4_FACTORY_SMELTER_MODULE_ID,
        "M4FactorySmelterMk1",
        wasm_hash,
        32,
    )
}

fn m4_factory_assembler_manifest(wasm_hash: String) -> ModuleManifest {
    m4_manifest(
        M4_FACTORY_ASSEMBLER_MODULE_ID,
        "M4FactoryAssemblerMk1",
        wasm_hash,
        32,
    )
}

fn m4_recipe_smelt_iron_manifest(wasm_hash: String) -> ModuleManifest {
    m4_manifest(
        M4_RECIPE_SMELT_IRON_MODULE_ID,
        "M4RecipeSmelterIronIngot",
        wasm_hash,
        128,
    )
}

fn m4_recipe_smelt_copper_wire_manifest(wasm_hash: String) -> ModuleManifest {
    m4_manifest(
        M4_RECIPE_SMELT_COPPER_WIRE_MODULE_ID,
        "M4RecipeSmelterCopperWire",
        wasm_hash,
        128,
    )
}

fn m4_recipe_assemble_gear_manifest(wasm_hash: String) -> ModuleManifest {
    m4_manifest(
        M4_RECIPE_ASSEMBLE_GEAR_MODULE_ID,
        "M4RecipeAssemblerGear",
        wasm_hash,
        128,
    )
}

fn m4_recipe_assemble_control_chip_manifest(wasm_hash: String) -> ModuleManifest {
    m4_manifest(
        M4_RECIPE_ASSEMBLE_CONTROL_CHIP_MODULE_ID,
        "M4RecipeAssemblerControlChip",
        wasm_hash,
        128,
    )
}

fn m4_recipe_assemble_motor_manifest(wasm_hash: String) -> ModuleManifest {
    m4_manifest(
        M4_RECIPE_ASSEMBLE_MOTOR_MODULE_ID,
        "M4RecipeAssemblerMotorMk1",
        wasm_hash,
        128,
    )
}

fn m4_recipe_assemble_drone_manifest(wasm_hash: String) -> ModuleManifest {
    m4_manifest(
        M4_RECIPE_ASSEMBLE_DRONE_MODULE_ID,
        "M4RecipeAssemblerLogisticsDrone",
        wasm_hash,
        128,
    )
}

fn m4_product_iron_ingot_manifest(wasm_hash: String) -> ModuleManifest {
    m4_manifest(
        M4_PRODUCT_IRON_INGOT_MODULE_ID,
        "M4ProductIronIngot",
        wasm_hash,
        64,
    )
}

fn m4_product_control_chip_manifest(wasm_hash: String) -> ModuleManifest {
    m4_manifest(
        M4_PRODUCT_CONTROL_CHIP_MODULE_ID,
        "M4ProductControlChip",
        wasm_hash,
        64,
    )
}

fn m4_product_motor_manifest(wasm_hash: String) -> ModuleManifest {
    m4_manifest(
        M4_PRODUCT_MOTOR_MODULE_ID,
        "M4ProductMotorMk1",
        wasm_hash,
        64,
    )
}

fn m4_product_logistics_drone_manifest(wasm_hash: String) -> ModuleManifest {
    m4_manifest(
        M4_PRODUCT_LOGISTICS_DRONE_MODULE_ID,
        "M4ProductLogisticsDrone",
        wasm_hash,
        64,
    )
}
