use super::super::{
    m4_builtin_module_artifact_identity, m4_builtin_wasm_module_artifact_bytes, util, Manifest,
    ModuleAbiContract, ModuleActivation, ModuleArtifactIdentity, ModuleChangeSet, ModuleKind,
    ModuleLimits, ModuleManifest, ModuleRegistry, ModuleRole, ProposalDecision, WorldError,
    M4_ECONOMY_MODULE_VERSION, M4_FACTORY_ASSEMBLER_MODULE_ID, M4_FACTORY_MINER_MODULE_ID,
    M4_FACTORY_SMELTER_MODULE_ID, M4_PRODUCT_CONTROL_CHIP_MODULE_ID,
    M4_PRODUCT_IRON_INGOT_MODULE_ID, M4_PRODUCT_LOGISTICS_DRONE_MODULE_ID,
    M4_PRODUCT_MOTOR_MODULE_ID, M4_RECIPE_ASSEMBLE_CONTROL_CHIP_MODULE_ID,
    M4_RECIPE_ASSEMBLE_DRONE_MODULE_ID, M4_RECIPE_ASSEMBLE_GEAR_MODULE_ID,
    M4_RECIPE_ASSEMBLE_MOTOR_MODULE_ID, M4_RECIPE_SMELT_COPPER_WIRE_MODULE_ID,
    M4_RECIPE_SMELT_IRON_MODULE_ID,
};
use super::World;

const M4_BOOTSTRAP_WASM_MAX_MEM_BYTES: u64 = 64 * 1024 * 1024;
const M4_BOOTSTRAP_WASM_MAX_GAS: u64 = 2_000_000;
const M4_BUILTIN_MODULE_IDS_MANIFEST: &str = include_str!("artifacts/m4_builtin_module_ids.txt");

#[derive(Debug, Clone, Copy)]
struct M4BootstrapModuleDescriptor {
    module_id: &'static str,
    manifest_name: &'static str,
    max_call_rate: u32,
}

const M4_BOOTSTRAP_MODULES: &[M4BootstrapModuleDescriptor] = &[
    M4BootstrapModuleDescriptor {
        module_id: M4_FACTORY_MINER_MODULE_ID,
        manifest_name: "M4FactoryMinerMk1",
        max_call_rate: 32,
    },
    M4BootstrapModuleDescriptor {
        module_id: M4_FACTORY_SMELTER_MODULE_ID,
        manifest_name: "M4FactorySmelterMk1",
        max_call_rate: 32,
    },
    M4BootstrapModuleDescriptor {
        module_id: M4_FACTORY_ASSEMBLER_MODULE_ID,
        manifest_name: "M4FactoryAssemblerMk1",
        max_call_rate: 32,
    },
    M4BootstrapModuleDescriptor {
        module_id: M4_RECIPE_SMELT_IRON_MODULE_ID,
        manifest_name: "M4RecipeSmelterIronIngot",
        max_call_rate: 128,
    },
    M4BootstrapModuleDescriptor {
        module_id: M4_RECIPE_SMELT_COPPER_WIRE_MODULE_ID,
        manifest_name: "M4RecipeSmelterCopperWire",
        max_call_rate: 128,
    },
    M4BootstrapModuleDescriptor {
        module_id: M4_RECIPE_ASSEMBLE_GEAR_MODULE_ID,
        manifest_name: "M4RecipeAssemblerGear",
        max_call_rate: 128,
    },
    M4BootstrapModuleDescriptor {
        module_id: M4_RECIPE_ASSEMBLE_CONTROL_CHIP_MODULE_ID,
        manifest_name: "M4RecipeAssemblerControlChip",
        max_call_rate: 128,
    },
    M4BootstrapModuleDescriptor {
        module_id: M4_RECIPE_ASSEMBLE_MOTOR_MODULE_ID,
        manifest_name: "M4RecipeAssemblerMotorMk1",
        max_call_rate: 128,
    },
    M4BootstrapModuleDescriptor {
        module_id: M4_RECIPE_ASSEMBLE_DRONE_MODULE_ID,
        manifest_name: "M4RecipeAssemblerLogisticsDrone",
        max_call_rate: 128,
    },
    M4BootstrapModuleDescriptor {
        module_id: M4_PRODUCT_IRON_INGOT_MODULE_ID,
        manifest_name: "M4ProductIronIngot",
        max_call_rate: 64,
    },
    M4BootstrapModuleDescriptor {
        module_id: M4_PRODUCT_CONTROL_CHIP_MODULE_ID,
        manifest_name: "M4ProductControlChip",
        max_call_rate: 64,
    },
    M4BootstrapModuleDescriptor {
        module_id: M4_PRODUCT_MOTOR_MODULE_ID,
        manifest_name: "M4ProductMotorMk1",
        max_call_rate: 64,
    },
    M4BootstrapModuleDescriptor {
        module_id: M4_PRODUCT_LOGISTICS_DRONE_MODULE_ID,
        manifest_name: "M4ProductLogisticsDrone",
        max_call_rate: 64,
    },
];

pub(crate) fn m4_bootstrap_module_ids() -> Vec<&'static str> {
    M4_BOOTSTRAP_MODULES
        .iter()
        .map(|item| item.module_id)
        .collect()
}

fn m4_builtin_module_ids_from_manifest() -> Vec<&'static str> {
    M4_BUILTIN_MODULE_IDS_MANIFEST
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

fn ensure_m4_bootstrap_descriptor_consistency() -> Result<(), WorldError> {
    let descriptor_ids = m4_bootstrap_module_ids();
    let manifest_ids = m4_builtin_module_ids_from_manifest();
    if descriptor_ids != manifest_ids {
        return Err(WorldError::ModuleChangeInvalid {
            reason: format!(
                "m4 bootstrap descriptor ids mismatch module ids manifest: descriptor=[{}] manifest=[{}]",
                descriptor_ids.join(","),
                manifest_ids.join(",")
            ),
        });
    }
    Ok(())
}

impl World {
    pub fn install_m4_economy_bootstrap_modules(
        &mut self,
        actor: impl Into<String>,
    ) -> Result<(), WorldError> {
        ensure_m4_bootstrap_descriptor_consistency()?;

        let actor = actor.into();
        let mut changes = ModuleChangeSet::default();

        for descriptor in M4_BOOTSTRAP_MODULES {
            ensure_bootstrap_module(
                self,
                &mut changes,
                descriptor,
                &m4_builtin_wasm_artifact_for_module(descriptor.module_id)?,
                M4_ECONOMY_MODULE_VERSION,
            )?;
        }

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
    m4_builtin_module_artifact_identity(module_id, wasm_hash).unwrap_or_else(|error| {
        panic!(
            "builtin m4 identity invariant failed module_id={} wasm_hash={} err={:?}",
            module_id, wasm_hash, error
        )
    })
}

fn ensure_bootstrap_module(
    world: &mut World,
    changes: &mut ModuleChangeSet,
    descriptor: &M4BootstrapModuleDescriptor,
    artifact: &[u8],
    version: &str,
) -> Result<(), WorldError> {
    if world
        .module_registry
        .active
        .get(descriptor.module_id)
        .is_some_and(|active_version| active_version == version)
    {
        return Ok(());
    }

    let record_key = ModuleRegistry::record_key(descriptor.module_id, version);
    if !world.module_registry.records.contains_key(&record_key) {
        let wasm_hash = util::sha256_hex(artifact);
        world.register_module_artifact(wasm_hash.clone(), artifact)?;
        changes.register.push(m4_manifest(descriptor, wasm_hash));
    }

    changes.activate.push(ModuleActivation {
        module_id: descriptor.module_id.to_string(),
        version: version.to_string(),
    });

    Ok(())
}

fn m4_manifest(descriptor: &M4BootstrapModuleDescriptor, wasm_hash: String) -> ModuleManifest {
    ModuleManifest {
        module_id: descriptor.module_id.to_string(),
        name: descriptor.manifest_name.to_string(),
        version: M4_ECONOMY_MODULE_VERSION.to_string(),
        kind: ModuleKind::Pure,
        role: ModuleRole::Domain,
        artifact_identity: Some(m4_bootstrap_artifact_identity(
            descriptor.module_id,
            &wasm_hash,
        )),
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        abi_contract: ModuleAbiContract::default(),
        exports: vec!["call".to_string()],
        subscriptions: Vec::new(),
        required_caps: Vec::new(),
        limits: ModuleLimits {
            max_mem_bytes: M4_BOOTSTRAP_WASM_MAX_MEM_BYTES,
            max_gas: M4_BOOTSTRAP_WASM_MAX_GAS,
            max_call_rate: descriptor.max_call_rate,
            max_output_bytes: 4096,
            max_effects: 0,
            max_emits: 2,
        },
    }
}
