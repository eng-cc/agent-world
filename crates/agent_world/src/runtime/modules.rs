//! Module types and registry for WASM runtime integration.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::types::{ProposalId, WorldEventId, WorldTime};
pub use agent_world_wasm_abi::{
    ModuleActivation, ModuleArtifact, ModuleCache, ModuleChangeSet, ModuleDeactivation, ModuleKind,
    ModuleLimits, ModuleManifest, ModuleRole, ModuleSubscription, ModuleSubscriptionStage,
    ModuleUpgrade,
};

/// Registry of all known modules and their activation status.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleRegistry {
    pub records: BTreeMap<String, ModuleRecord>,
    pub active: BTreeMap<String, String>,
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self {
            records: BTreeMap::new(),
            active: BTreeMap::new(),
        }
    }
}

impl ModuleRegistry {
    pub fn record_key(module_id: &str, version: &str) -> String {
        format!("{module_id}@{version}")
    }
}

/// Metadata about a registered module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleRecord {
    pub manifest: ModuleManifest,
    pub registered_at: WorldTime,
    pub registered_by: String,
    pub audit_event_id: Option<WorldEventId>,
}

/// Module lifecycle event wrapper with governance linkage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleEvent {
    pub proposal_id: ProposalId,
    #[serde(flatten)]
    pub kind: ModuleEventKind,
}

/// Module lifecycle events recorded during apply.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ModuleEventKind {
    RegisterModule {
        module: ModuleManifest,
        registered_by: String,
    },
    ActivateModule {
        module_id: String,
        version: String,
        activated_by: String,
    },
    DeactivateModule {
        module_id: String,
        reason: String,
        deactivated_by: String,
    },
    UpgradeModule {
        module_id: String,
        from_version: String,
        to_version: String,
        wasm_hash: String,
        manifest: ModuleManifest,
        upgraded_by: String,
    },
}
