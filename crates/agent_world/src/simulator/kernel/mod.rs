//! WorldKernel: time, events, actions, and observation.

mod actions;
mod observation;
mod persistence;
mod power;
mod replay;
mod step;
mod types;

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;

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
    ChunkGenerationCause, Observation, ObservedAgent, ObservedLocation, PromptUpdateOperation,
    RejectReason, WorldEvent, WorldEventKind,
};

type PreActionRuleHook = Arc<dyn Fn(ActionId, &Action) + Send + Sync>;
type PostActionRuleHook = Arc<dyn Fn(ActionId, &Action, &WorldEvent) + Send + Sync>;

#[derive(Default, Clone)]
struct RuleHookRegistry {
    pre_action: Vec<PreActionRuleHook>,
    post_action: Vec<PostActionRuleHook>,
}

impl std::fmt::Debug for RuleHookRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuleHookRegistry")
            .field("pre_action_len", &self.pre_action.len())
            .field("post_action_len", &self.post_action.len())
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
        F: Fn(ActionId, &Action) + Send + Sync + 'static,
    {
        self.rule_hooks.pre_action.push(Arc::new(hook));
    }

    pub fn add_post_action_rule_hook<F>(&mut self, hook: F)
    where
        F: Fn(ActionId, &Action, &WorldEvent) + Send + Sync + 'static,
    {
        self.rule_hooks.post_action.push(Arc::new(hook));
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
