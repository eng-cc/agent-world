//! Module types and registry for WASM runtime integration.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::{BTreeMap, VecDeque};

use super::types::{ProposalId, WorldEventId, WorldTime};

/// Kinds of WASM modules supported by the runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleKind {
    Reducer,
    Pure,
}

impl ModuleKind {
    pub fn entrypoint(&self) -> &'static str {
        match self {
            ModuleKind::Reducer => "reduce",
            ModuleKind::Pure => "call",
        }
    }
}

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

/// Resource limits for module execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleLimits {
    pub max_mem_bytes: u64,
    pub max_gas: u64,
    pub max_call_rate: u32,
    pub max_output_bytes: u64,
    pub max_effects: u32,
    pub max_emits: u32,
}

impl Default for ModuleLimits {
    fn default() -> Self {
        Self {
            max_mem_bytes: 0,
            max_gas: 0,
            max_call_rate: 0,
            max_output_bytes: 0,
            max_effects: 0,
            max_emits: 0,
        }
    }
}

impl ModuleLimits {
    pub fn unbounded() -> Self {
        Self {
            max_mem_bytes: u64::MAX,
            max_gas: u64::MAX,
            max_call_rate: u32::MAX,
            max_output_bytes: u64::MAX,
            max_effects: u32::MAX,
            max_emits: u32::MAX,
        }
    }
}

/// Stored artifact bytes for a module.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleArtifact {
    pub wasm_hash: String,
    pub bytes: Vec<u8>,
}

/// In-memory LRU cache of loaded module artifacts.
#[derive(Debug, Clone, PartialEq)]
pub struct ModuleCache {
    max_cached_modules: usize,
    cache: BTreeMap<String, ModuleArtifact>,
    lru: VecDeque<String>,
}

impl ModuleCache {
    pub fn new(max_cached_modules: usize) -> Self {
        Self {
            max_cached_modules,
            cache: BTreeMap::new(),
            lru: VecDeque::new(),
        }
    }

    pub fn max_cached_modules(&self) -> usize {
        self.max_cached_modules
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn set_max_cached_modules(&mut self, max_cached_modules: usize) {
        self.max_cached_modules = max_cached_modules;
        self.prune();
    }

    pub fn get(&mut self, wasm_hash: &str) -> Option<ModuleArtifact> {
        let artifact = self.cache.get(wasm_hash)?.clone();
        self.touch(wasm_hash);
        Some(artifact)
    }

    pub fn insert(&mut self, artifact: ModuleArtifact) {
        let key = artifact.wasm_hash.clone();
        self.cache.insert(key.clone(), artifact);
        self.touch(&key);
        self.prune();
    }

    fn touch(&mut self, wasm_hash: &str) {
        self.lru.retain(|entry| entry != wasm_hash);
        self.lru.push_back(wasm_hash.to_string());
    }

    fn prune(&mut self) {
        if self.max_cached_modules == 0 {
            self.cache.clear();
            self.lru.clear();
            return;
        }
        while self.cache.len() > self.max_cached_modules {
            if let Some(evicted) = self.lru.pop_front() {
                self.cache.remove(&evicted);
            } else {
                break;
            }
        }
    }
}

impl Default for ModuleCache {
    fn default() -> Self {
        Self::new(8)
    }
}

/// Subscription specification for module event routing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleSubscription {
    #[serde(default)]
    pub event_kinds: Vec<String>,
    #[serde(default)]
    pub action_kinds: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage: Option<ModuleSubscriptionStage>,
    #[serde(default)]
    pub filters: Option<JsonValue>,
}

impl ModuleSubscription {
    pub fn resolved_stage(&self) -> ModuleSubscriptionStage {
        self.stage.unwrap_or_else(|| {
            if !self.event_kinds.is_empty() {
                ModuleSubscriptionStage::PostEvent
            } else {
                ModuleSubscriptionStage::PreAction
            }
        })
    }
}

/// Routing stage for module subscriptions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleSubscriptionStage {
    PreAction,
    PostAction,
    PostEvent,
}

impl Default for ModuleSubscriptionStage {
    fn default() -> Self {
        ModuleSubscriptionStage::PostEvent
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
