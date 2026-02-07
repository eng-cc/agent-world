use super::super::{
    util, Manifest, ModuleActivation, ModuleChangeSet, ModuleKind, ModuleLimits, ModuleManifest,
    ModuleRegistry, ModuleRole, ModuleSubscription, ModuleSubscriptionStage, ProposalDecision,
    WorldError, M1_POWER_MODULE_VERSION, M1_RADIATION_POWER_MODULE_ID, M1_STORAGE_POWER_MODULE_ID,
};
use super::World;

const M1_RADIATION_POWER_ARTIFACT: &[u8] = b"m1-radiation-power";
const M1_STORAGE_POWER_ARTIFACT: &[u8] = b"m1-storage-power";

impl World {
    pub fn install_m1_power_bootstrap_modules(
        &mut self,
        actor: impl Into<String>,
    ) -> Result<(), WorldError> {
        let actor = actor.into();
        let mut changes = ModuleChangeSet::default();

        ensure_bootstrap_module(
            self,
            &mut changes,
            M1_RADIATION_POWER_MODULE_ID,
            M1_RADIATION_POWER_ARTIFACT,
            m1_radiation_power_manifest,
        )?;
        ensure_bootstrap_module(
            self,
            &mut changes,
            M1_STORAGE_POWER_MODULE_ID,
            M1_STORAGE_POWER_ARTIFACT,
            m1_storage_power_manifest,
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

fn ensure_bootstrap_module(
    world: &mut World,
    changes: &mut ModuleChangeSet,
    module_id: &str,
    artifact: &[u8],
    make_manifest: fn(String) -> ModuleManifest,
) -> Result<(), WorldError> {
    if world
        .module_registry
        .active
        .get(module_id)
        .is_some_and(|version| version == M1_POWER_MODULE_VERSION)
    {
        return Ok(());
    }

    let record_key = ModuleRegistry::record_key(module_id, M1_POWER_MODULE_VERSION);
    if !world.module_registry.records.contains_key(&record_key) {
        let wasm_hash = util::sha256_hex(artifact);
        world.register_module_artifact(wasm_hash.clone(), artifact)?;
        changes.register.push(make_manifest(wasm_hash));
    }

    changes.activate.push(ModuleActivation {
        module_id: module_id.to_string(),
        version: M1_POWER_MODULE_VERSION.to_string(),
    });

    Ok(())
}

fn m1_radiation_power_manifest(wasm_hash: String) -> ModuleManifest {
    ModuleManifest {
        module_id: M1_RADIATION_POWER_MODULE_ID.to_string(),
        name: "M1RadiationPower".to_string(),
        version: M1_POWER_MODULE_VERSION.to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        exports: vec!["reduce".to_string()],
        subscriptions: vec![
            ModuleSubscription {
                event_kinds: vec![
                    "domain.agent_registered".to_string(),
                    "domain.agent_moved".to_string(),
                ],
                action_kinds: Vec::new(),
                stage: Some(ModuleSubscriptionStage::PostEvent),
                filters: None,
            },
            ModuleSubscription {
                event_kinds: Vec::new(),
                action_kinds: vec!["action.*".to_string()],
                stage: Some(ModuleSubscriptionStage::PreAction),
                filters: None,
            },
        ],
        required_caps: Vec::new(),
        limits: ModuleLimits {
            max_mem_bytes: 1024,
            max_gas: 10_000,
            max_call_rate: 32,
            max_output_bytes: 4096,
            max_effects: 0,
            max_emits: 1,
        },
    }
}

fn m1_storage_power_manifest(wasm_hash: String) -> ModuleManifest {
    ModuleManifest {
        module_id: M1_STORAGE_POWER_MODULE_ID.to_string(),
        name: "M1StoragePower".to_string(),
        version: M1_POWER_MODULE_VERSION.to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Body,
        wasm_hash,
        interface_version: "wasm-1".to_string(),
        exports: vec!["reduce".to_string()],
        subscriptions: vec![
            ModuleSubscription {
                event_kinds: vec![
                    "domain.agent_registered".to_string(),
                    "domain.agent_moved".to_string(),
                ],
                action_kinds: Vec::new(),
                stage: Some(ModuleSubscriptionStage::PostEvent),
                filters: None,
            },
            ModuleSubscription {
                event_kinds: Vec::new(),
                action_kinds: vec!["action.move_agent".to_string()],
                stage: Some(ModuleSubscriptionStage::PreAction),
                filters: None,
            },
        ],
        required_caps: Vec::new(),
        limits: ModuleLimits {
            max_mem_bytes: 2048,
            max_gas: 20_000,
            max_call_rate: 64,
            max_output_bytes: 4096,
            max_effects: 0,
            max_emits: 1,
        },
    }
}
