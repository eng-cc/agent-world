//! Module types and registry for WASM runtime integration.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::types::{ProposalId, WorldEventId, WorldTime};
pub use agent_world_wasm_abi::{
    ModuleArtifact, ModuleCache, ModuleKind, ModuleLimits, ModuleSubscription,
    ModuleSubscriptionStage,
};

/// Roles for modules in the runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleRole {
    Rule,
    Domain,
    Body,
    AgentInternal,
}

impl Default for ModuleRole {
    fn default() -> Self {
        ModuleRole::Domain
    }
}

/// Manifest entry describing a module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleManifest {
    pub module_id: String,
    pub name: String,
    pub version: String,
    pub kind: ModuleKind,
    #[serde(default)]
    pub role: ModuleRole,
    pub wasm_hash: String,
    pub interface_version: String,
    #[serde(default)]
    pub exports: Vec<String>,
    #[serde(default)]
    pub subscriptions: Vec<ModuleSubscription>,
    #[serde(default)]
    pub required_caps: Vec<String>,
    #[serde(default)]
    pub limits: ModuleLimits,
}

/// Planned module changes for governance apply.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ModuleChangeSet {
    #[serde(default)]
    pub register: Vec<ModuleManifest>,
    #[serde(default)]
    pub activate: Vec<ModuleActivation>,
    #[serde(default)]
    pub deactivate: Vec<ModuleDeactivation>,
    #[serde(default)]
    pub upgrade: Vec<ModuleUpgrade>,
}

impl ModuleChangeSet {
    pub fn is_empty(&self) -> bool {
        self.register.is_empty()
            && self.activate.is_empty()
            && self.deactivate.is_empty()
            && self.upgrade.is_empty()
    }
}

/// Activation request for a module version.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleActivation {
    pub module_id: String,
    pub version: String,
}

/// Deactivation request for a module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleDeactivation {
    pub module_id: String,
    pub reason: String,
}

/// Upgrade request for a module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleUpgrade {
    pub module_id: String,
    pub from_version: String,
    pub to_version: String,
    pub wasm_hash: String,
    pub manifest: ModuleManifest,
}

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
