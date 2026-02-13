use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::{BTreeMap, VecDeque};

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleLimits {
    pub max_mem_bytes: u64,
    pub max_gas: u64,
    pub max_call_rate: u32,
    pub max_output_bytes: u64,
    pub max_effects: u32,
    pub max_emits: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleArtifact {
    pub wasm_hash: String,
    pub bytes: Vec<u8>,
}

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleEffectIntent {
    pub kind: String,
    pub params: JsonValue,
    pub cap_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleEmit {
    pub kind: String,
    pub payload: JsonValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleOutput {
    pub new_state: Option<Vec<u8>>,
    #[serde(default)]
    pub effects: Vec<ModuleEffectIntent>,
    #[serde(default)]
    pub emits: Vec<ModuleEmit>,
    pub output_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleCallRequest {
    pub module_id: String,
    pub wasm_hash: String,
    pub trace_id: String,
    pub entrypoint: String,
    pub input: Vec<u8>,
    pub limits: ModuleLimits,
    #[serde(default)]
    pub wasm_bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleCallOrigin {
    pub kind: String,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleContext {
    pub v: String,
    pub module_id: String,
    pub trace_id: String,
    pub time: u64,
    pub origin: ModuleCallOrigin,
    pub limits: ModuleLimits,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_config_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleCallInput {
    pub ctx: ModuleContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event: Option<Vec<u8>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<Vec<u8>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleCallErrorCode {
    Trap,
    Timeout,
    OutputTooLarge,
    EffectLimitExceeded,
    EmitLimitExceeded,
    CapsDenied,
    PolicyDenied,
    SandboxUnavailable,
    InvalidOutput,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleCallFailure {
    pub module_id: String,
    pub trace_id: String,
    pub code: ModuleCallErrorCode,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleEmitEvent {
    pub module_id: String,
    pub trace_id: String,
    pub kind: String,
    pub payload: JsonValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleStateUpdate {
    pub module_id: String,
    pub trace_id: String,
    pub state: Vec<u8>,
}

pub trait ModuleSandbox {
    fn call(&mut self, request: &ModuleCallRequest) -> Result<ModuleOutput, ModuleCallFailure>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact(hash: &str, byte: u8) -> ModuleArtifact {
        ModuleArtifact {
            wasm_hash: hash.to_string(),
            bytes: vec![byte],
        }
    }

    #[test]
    fn module_cache_evicts_lru_entry() {
        let mut cache = ModuleCache::new(2);
        cache.insert(artifact("a", 1));
        cache.insert(artifact("b", 2));
        let _ = cache.get("a");
        cache.insert(artifact("c", 3));

        assert!(cache.get("a").is_some());
        assert!(cache.get("b").is_none());
        assert!(cache.get("c").is_some());
    }

    #[test]
    fn module_cache_zero_capacity_stays_empty() {
        let mut cache = ModuleCache::new(0);
        cache.insert(artifact("a", 1));
        assert_eq!(cache.len(), 0);
        assert!(cache.get("a").is_none());
    }

    #[test]
    fn module_cache_set_max_prunes_existing_entries() {
        let mut cache = ModuleCache::new(3);
        cache.insert(artifact("a", 1));
        cache.insert(artifact("b", 2));
        cache.insert(artifact("c", 3));
        cache.set_max_cached_modules(1);

        assert_eq!(cache.len(), 1);
        assert!(cache.get("c").is_some());
        assert!(cache.get("a").is_none());
        assert!(cache.get("b").is_none());
    }
}
