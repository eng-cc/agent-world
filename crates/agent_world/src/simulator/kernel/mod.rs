//! WorldKernel: time, events, actions, and observation.

mod actions;
mod observation;
mod persistence;
mod power;
mod replay;
mod step;
mod types;

use agent_world_wasm_abi::{
    ModuleCallInput, ModuleCallOrigin, ModuleCallRequest, ModuleContext, ModuleLimits,
    ModuleOutput, ModuleSandbox,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};

use super::types::{
    Action, ActionEnvelope, ActionId, FragmentElementKind, WorldEventId, WorldTime,
};
use super::world_model::{AgentPromptProfile, FragmentResourceError, WorldConfig, WorldModel};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChunkRuntimeConfig {
    pub world_seed: u64,
    pub asteroid_fragment_enabled: bool,
    pub asteroid_fragment_seed_offset: u64,
    pub min_fragment_spacing_cm: Option<i64>,
}

impl Default for ChunkRuntimeConfig {
    fn default() -> Self {
        Self {
            world_seed: 0,
            asteroid_fragment_enabled: false,
            asteroid_fragment_seed_offset: 1,
            min_fragment_spacing_cm: None,
        }
    }
}

impl ChunkRuntimeConfig {
    pub fn asteroid_fragment_seed(&self) -> u64 {
        self.world_seed
            .wrapping_add(self.asteroid_fragment_seed_offset)
    }
}

pub use types::{
    merge_kernel_rule_decisions, ChunkGenerationCause, KernelRuleCost, KernelRuleDecision,
    KernelRuleDecisionMergeError, KernelRuleModuleContext, KernelRuleModuleInput,
    KernelRuleModuleOutput, KernelRuleVerdict, Observation, ObservedAgent, ObservedLocation,
    PromptUpdateOperation, RejectReason, WorldEvent, WorldEventKind,
};

type PreActionRuleHook =
    Arc<dyn Fn(ActionId, &Action, &WorldKernel) -> KernelRuleDecision + Send + Sync>;
type PostActionRuleHook = Arc<dyn Fn(ActionId, &Action, &WorldEvent) + Send + Sync>;
type PreActionWasmRuleEvaluator =
    Arc<dyn Fn(&KernelRuleModuleInput) -> Result<KernelRuleModuleOutput, String> + Send + Sync>;
const RULE_DECISION_EMIT_KIND: &str = "rule.decision";

#[derive(Default, Clone)]
struct RuleHookRegistry {
    pre_action: Vec<PreActionRuleHook>,
    post_action: Vec<PostActionRuleHook>,
    pre_action_wasm: Option<PreActionWasmRuleEvaluator>,
    pre_action_wasm_artifacts: BTreeMap<String, Vec<u8>>,
}

impl std::fmt::Debug for RuleHookRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuleHookRegistry")
            .field("pre_action_len", &self.pre_action.len())
            .field("post_action_len", &self.post_action.len())
            .field("pre_action_wasm_enabled", &self.pre_action_wasm.is_some())
            .field(
                "pre_action_wasm_artifact_count",
                &self.pre_action_wasm_artifacts.len(),
            )
            .finish()
    }
}

impl PartialEq for RuleHookRegistry {
    fn eq(&self, _other: &Self) -> bool {
        // Runtime hooks are process-local closures and intentionally excluded from state equality.
        true
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorldKernel {
    time: WorldTime,
    config: WorldConfig,
    next_event_id: WorldEventId,
    next_action_id: ActionId,
    pending_actions: VecDeque<ActionEnvelope>,
    journal: Vec<WorldEvent>,
    model: WorldModel,
    #[serde(default)]
    chunk_runtime: ChunkRuntimeConfig,
    #[serde(skip, default)]
    rule_hooks: RuleHookRegistry,
}

impl WorldKernel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: WorldConfig) -> Self {
        let mut kernel = Self::default();
        kernel.config = config.sanitized();
        kernel
    }

    pub fn with_model(config: WorldConfig, model: WorldModel) -> Self {
        Self {
            time: 0,
            config: config.sanitized(),
            next_event_id: 0,
            next_action_id: 0,
            pending_actions: VecDeque::new(),
            journal: Vec::new(),
            model,
            chunk_runtime: ChunkRuntimeConfig::default(),
            rule_hooks: RuleHookRegistry::default(),
        }
    }

    pub fn with_model_and_chunk_runtime(
        config: WorldConfig,
        model: WorldModel,
        chunk_runtime: ChunkRuntimeConfig,
    ) -> Self {
        Self {
            time: 0,
            config: config.sanitized(),
            next_event_id: 0,
            next_action_id: 0,
            pending_actions: VecDeque::new(),
            journal: Vec::new(),
            model,
            chunk_runtime,
            rule_hooks: RuleHookRegistry::default(),
        }
    }

    pub fn add_pre_action_rule_hook<F>(&mut self, hook: F)
    where
        F: Fn(ActionId, &Action, &WorldKernel) -> KernelRuleDecision + Send + Sync + 'static,
    {
        self.rule_hooks.pre_action.push(Arc::new(hook));
    }

    pub fn add_post_action_rule_hook<F>(&mut self, hook: F)
    where
        F: Fn(ActionId, &Action, &WorldEvent) + Send + Sync + 'static,
    {
        self.rule_hooks.post_action.push(Arc::new(hook));
    }

    pub fn set_pre_action_wasm_rule_evaluator<F>(&mut self, evaluator: F)
    where
        F: Fn(&KernelRuleModuleInput) -> Result<KernelRuleModuleOutput, String>
            + Send
            + Sync
            + 'static,
    {
        self.rule_hooks.pre_action_wasm = Some(Arc::new(evaluator));
    }

    pub fn clear_pre_action_wasm_rule_evaluator(&mut self) {
        self.rule_hooks.pre_action_wasm = None;
    }

    pub fn set_pre_action_wasm_rule_module_evaluator<S>(
        &mut self,
        module_id: impl Into<String>,
        wasm_hash: impl Into<String>,
        entrypoint: impl Into<String>,
        wasm_bytes: Vec<u8>,
        limits: ModuleLimits,
        sandbox: Arc<Mutex<S>>,
    ) where
        S: ModuleSandbox + Send + 'static,
    {
        let module_id = module_id.into();
        let wasm_hash = wasm_hash.into();
        let entrypoint = entrypoint.into();
        self.set_pre_action_wasm_rule_evaluator(move |input| {
            let request = build_pre_action_wasm_call_request(
                input,
                &module_id,
                &wasm_hash,
                &entrypoint,
                &wasm_bytes,
                &limits,
            )?;
            let output = {
                let mut locked = sandbox
                    .lock()
                    .map_err(|_| "wasm sandbox mutex poisoned".to_string())?;
                locked.call(&request).map_err(|failure| {
                    format!("module call failed {:?}: {}", failure.code, failure.detail)
                })?
            };
            let decision = parse_pre_action_wasm_rule_decision(input.action_id, &output)?;
            Ok(KernelRuleModuleOutput::from_decision(decision))
        });
    }

    pub fn register_pre_action_wasm_rule_artifact(
        &mut self,
        wasm_hash: impl Into<String>,
        wasm_bytes: Vec<u8>,
    ) -> Result<(), String> {
        let wasm_hash = wasm_hash.into();
        if wasm_hash.trim().is_empty() {
            return Err("wasm hash is empty".to_string());
        }
        if wasm_bytes.is_empty() {
            return Err(format!("wasm bytes are empty for hash {wasm_hash}"));
        }
        if let Some(existing) = self.rule_hooks.pre_action_wasm_artifacts.get(&wasm_hash) {
            if existing != &wasm_bytes {
                return Err(format!(
                    "artifact hash {wasm_hash} already registered with different bytes"
                ));
            }
            return Ok(());
        }

        self.rule_hooks
            .pre_action_wasm_artifacts
            .insert(wasm_hash, wasm_bytes);
        Ok(())
    }

    pub fn remove_pre_action_wasm_rule_artifact(&mut self, wasm_hash: &str) -> bool {
        self.rule_hooks
            .pre_action_wasm_artifacts
            .remove(wasm_hash)
            .is_some()
    }

    pub fn set_pre_action_wasm_rule_module_from_registry<S>(
        &mut self,
        module_id: impl Into<String>,
        wasm_hash: impl Into<String>,
        entrypoint: impl Into<String>,
        limits: ModuleLimits,
        sandbox: Arc<Mutex<S>>,
    ) -> Result<(), String>
    where
        S: ModuleSandbox + Send + 'static,
    {
        let wasm_hash = wasm_hash.into();
        let wasm_bytes = self
            .rule_hooks
            .pre_action_wasm_artifacts
            .get(&wasm_hash)
            .cloned()
            .ok_or_else(|| format!("pre-action wasm artifact missing for hash {wasm_hash}"))?;

        self.set_pre_action_wasm_rule_module_evaluator(
            module_id, wasm_hash, entrypoint, wasm_bytes, limits, sandbox,
        );
        Ok(())
    }

    pub fn time(&self) -> WorldTime {
        self.time
    }

    pub fn config(&self) -> &WorldConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: WorldConfig) {
        self.config = config.sanitized();
    }

    pub fn model(&self) -> &WorldModel {
        &self.model
    }

    pub fn consume_fragment_resource(
        &mut self,
        location_id: &str,
        kind: FragmentElementKind,
        amount_g: i64,
    ) -> Result<i64, FragmentResourceError> {
        self.model
            .consume_fragment_resource(location_id, &self.config.space, kind, amount_g)
    }

    pub fn journal(&self) -> &[WorldEvent] {
        &self.journal
    }

    pub fn apply_agent_prompt_profile_update(
        &mut self,
        profile: AgentPromptProfile,
        operation: PromptUpdateOperation,
        applied_fields: Vec<String>,
        digest: String,
        rolled_back_to_version: Option<u64>,
    ) -> WorldEvent {
        self.model
            .agent_prompt_profiles
            .insert(profile.agent_id.clone(), profile.clone());
        self.record_event(WorldEventKind::AgentPromptUpdated {
            profile,
            operation,
            applied_fields,
            digest,
            rolled_back_to_version,
        })
    }

    pub(super) fn record_event(&mut self, kind: WorldEventKind) -> WorldEvent {
        let event = WorldEvent {
            id: self.next_event_id,
            time: self.time,
            kind,
        };
        self.next_event_id = self.next_event_id.saturating_add(1);
        self.journal.push(event.clone());
        event
    }
}

fn build_pre_action_wasm_call_request(
    input: &KernelRuleModuleInput,
    module_id: &str,
    wasm_hash: &str,
    entrypoint: &str,
    wasm_bytes: &[u8],
    limits: &ModuleLimits,
) -> Result<ModuleCallRequest, String> {
    let action_bytes = to_canonical_cbor(input)?;
    let trace_id = format!(
        "sim-kernel-action-{}-t{}",
        input.action_id, input.context.time
    );
    let call_input = ModuleCallInput {
        ctx: ModuleContext {
            v: "wasm-1".to_string(),
            module_id: module_id.to_string(),
            trace_id: trace_id.clone(),
            time: input.context.time,
            origin: ModuleCallOrigin {
                kind: "simulator_action".to_string(),
                id: input.action_id.to_string(),
            },
            limits: limits.clone(),
            world_config_hash: None,
        },
        event: None,
        action: Some(action_bytes),
        state: None,
    };
    let input_bytes = to_canonical_cbor(&call_input)?;

    Ok(ModuleCallRequest {
        module_id: module_id.to_string(),
        wasm_hash: wasm_hash.to_string(),
        trace_id,
        entrypoint: entrypoint.to_string(),
        input: input_bytes,
        limits: limits.clone(),
        wasm_bytes: wasm_bytes.to_vec(),
    })
}

fn parse_pre_action_wasm_rule_decision(
    action_id: ActionId,
    output: &ModuleOutput,
) -> Result<KernelRuleDecision, String> {
    let mut decision = None;
    for emit in &output.emits {
        if emit.kind != RULE_DECISION_EMIT_KIND {
            continue;
        }
        if decision.is_some() {
            return Err("multiple rule.decision emits in wasm module output".to_string());
        }
        let parsed: KernelRuleDecision = serde_json::from_value(emit.payload.clone())
            .map_err(|err| format!("failed to decode rule.decision payload: {err}"))?;
        if parsed.action_id != action_id {
            return Err(format!(
                "rule.decision action_id mismatch expected {action_id} got {}",
                parsed.action_id
            ));
        }
        decision = Some(parsed);
    }

    Ok(decision.unwrap_or_else(|| KernelRuleDecision::allow(action_id)))
}

fn to_canonical_cbor<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    let mut buf = Vec::with_capacity(256);
    let canonical_value = serde_cbor::value::to_value(value)
        .map_err(|err| format!("failed to convert value to canonical cbor: {err}"))?;
    let mut serializer = serde_cbor::ser::Serializer::new(&mut buf);
    serializer
        .self_describe()
        .map_err(|err| format!("failed to write cbor self describe tag: {err}"))?;
    canonical_value
        .serialize(&mut serializer)
        .map_err(|err| format!("failed to serialize canonical cbor value: {err}"))?;
    Ok(buf)
}
