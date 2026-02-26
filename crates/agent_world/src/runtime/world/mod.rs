//! The World struct - core runtime implementation.

mod actions;
mod audit;
mod base_layer;
mod body;
mod bootstrap_economy;
mod bootstrap_gameplay;
mod bootstrap_power;
mod economy;
mod effects;
mod event_processing;
mod gameplay_layer;
mod gameplay_loop;
mod governance;
mod logistics;
mod module_actions;
mod module_runtime;
mod module_runtime_labels;
mod module_runtime_metering;
mod module_tick_runtime;
mod persistence;
mod policy;
mod resources;
mod rules;
mod scheduling;
mod snapshot;
mod step;

#[cfg(all(test, feature = "wasmtime", feature = "test_tier_full"))]
pub(crate) use bootstrap_economy::m4_bootstrap_module_ids;
pub use bootstrap_power::M1ScenarioBootstrapConfig;

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::effect::{CapabilityGrant, EffectIntent};
use super::events::ActionEnvelope;
use super::governance::Proposal;
use super::manifest::Manifest;
use super::modules::{ModuleCache, ModuleLimits, ModuleRegistry};
use super::policy::PolicySet;
use super::signer::ReceiptSigner;
use super::snapshot::{Journal, SnapshotCatalog};
use super::state::WorldState;
use super::types::{ActionId, IntentSeq, ProposalId, WorldEventId};

const DEFAULT_MAX_PENDING_ACTIONS: usize = 8_192;
const DEFAULT_MAX_PENDING_EFFECTS: usize = 8_192;
const DEFAULT_MAX_INFLIGHT_EFFECTS: usize = 8_192;
const DEFAULT_MAX_JOURNAL_EVENTS: usize = 65_536;
pub(super) const BUILTIN_MODULE_SIGNER_NODE_ID: &str = "builtin.module.release.signer";
pub(super) const BUILTIN_MODULE_SIGNER_PUBLIC_KEY_HEX: &str =
    "4b97aa20b3abd613401d4f5778eab8b6c019bd2ea912d1ce2234868536389ebb";
#[cfg(any(test, feature = "test_tier_required", feature = "test_tier_full"))]
pub(super) const TEST_MODULE_SIGNER_NODE_ID: &str = "test.module.release.signer";

#[cfg(any(test, feature = "test_tier_required", feature = "test_tier_full"))]
fn test_module_signer_public_key_hex() -> String {
    use ed25519_dalek::SigningKey;

    let seed = crate::runtime::util::sha256_hex(b"agent-world-test-module-artifact-signer-v1");
    let seed_bytes = hex::decode(seed).expect("decode test module signing seed");
    let private_key_bytes: [u8; 32] = seed_bytes
        .as_slice()
        .try_into()
        .expect("test module signing seed is 32 bytes");
    let signing_key = SigningKey::from_bytes(&private_key_bytes);
    hex::encode(signing_key.verifying_key().to_bytes())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldRuntimeMemoryLimits {
    pub max_pending_actions: usize,
    pub max_pending_effects: usize,
    pub max_inflight_effects: usize,
    pub max_journal_events: usize,
}

impl Default for WorldRuntimeMemoryLimits {
    fn default() -> Self {
        Self {
            max_pending_actions: DEFAULT_MAX_PENDING_ACTIONS,
            max_pending_effects: DEFAULT_MAX_PENDING_EFFECTS,
            max_inflight_effects: DEFAULT_MAX_INFLIGHT_EFFECTS,
            max_journal_events: DEFAULT_MAX_JOURNAL_EVENTS,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldRuntimeBackpressureStats {
    pub pending_actions_evicted: u64,
    pub pending_effects_evicted: u64,
    pub inflight_effects_evicted: u64,
    pub inflight_effect_dispatch_blocked: u64,
    pub journal_events_evicted: u64,
}

/// The main World runtime that orchestrates the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    manifest: Manifest,
    module_registry: ModuleRegistry,
    module_artifacts: BTreeSet<String>,
    #[serde(skip)]
    module_artifact_bytes: BTreeMap<String, Vec<u8>>,
    #[serde(skip)]
    module_cache: ModuleCache,
    module_limits_max: ModuleLimits,
    snapshot_catalog: SnapshotCatalog,
    state: WorldState,
    journal: Journal,
    next_event_id: WorldEventId,
    #[serde(default)]
    next_event_id_era: u64,
    next_action_id: ActionId,
    #[serde(default)]
    next_action_id_era: u64,
    next_intent_id: IntentSeq,
    #[serde(default)]
    next_intent_id_era: u64,
    next_proposal_id: ProposalId,
    #[serde(default)]
    next_proposal_id_era: u64,
    pending_actions: VecDeque<ActionEnvelope>,
    pending_effects: VecDeque<EffectIntent>,
    inflight_effects: BTreeMap<String, EffectIntent>,
    #[serde(default)]
    module_tick_schedule: BTreeMap<String, u64>,
    capabilities: BTreeMap<String, CapabilityGrant>,
    policies: PolicySet,
    proposals: BTreeMap<ProposalId, Proposal>,
    scheduler_cursor: Option<String>,
    #[serde(skip)]
    receipt_signer: Option<ReceiptSigner>,
    #[serde(default)]
    runtime_memory_limits: WorldRuntimeMemoryLimits,
    #[serde(default)]
    runtime_backpressure_stats: WorldRuntimeBackpressureStats,
}

impl World {
    pub fn new() -> Self {
        Self::new_with_state(WorldState::default())
    }

    pub fn new_with_state(mut state: WorldState) -> Self {
        state.migrate_legacy_material_ledgers();
        state
            .node_identity_bindings
            .entry(BUILTIN_MODULE_SIGNER_NODE_ID.to_string())
            .or_insert_with(|| BUILTIN_MODULE_SIGNER_PUBLIC_KEY_HEX.to_string());
        #[cfg(any(test, feature = "test_tier_required", feature = "test_tier_full"))]
        state
            .node_identity_bindings
            .entry(TEST_MODULE_SIGNER_NODE_ID.to_string())
            .or_insert_with(test_module_signer_public_key_hex);
        Self {
            manifest: Manifest::default(),
            module_registry: ModuleRegistry::default(),
            module_artifacts: BTreeSet::new(),
            module_artifact_bytes: BTreeMap::new(),
            module_cache: ModuleCache::default(),
            module_limits_max: ModuleLimits::unbounded(),
            snapshot_catalog: SnapshotCatalog::default(),
            state,
            journal: Journal::new(),
            next_event_id: 1,
            next_event_id_era: 0,
            next_action_id: 1,
            next_action_id_era: 0,
            next_intent_id: 1,
            next_intent_id_era: 0,
            next_proposal_id: 1,
            next_proposal_id_era: 0,
            pending_actions: VecDeque::new(),
            pending_effects: VecDeque::new(),
            inflight_effects: BTreeMap::new(),
            module_tick_schedule: BTreeMap::new(),
            capabilities: BTreeMap::new(),
            policies: PolicySet::default(),
            proposals: BTreeMap::new(),
            scheduler_cursor: None,
            receipt_signer: None,
            runtime_memory_limits: WorldRuntimeMemoryLimits::default(),
            runtime_backpressure_stats: WorldRuntimeBackpressureStats::default(),
        }
    }

    // ---------------------------------------------------------------------
    // Accessors
    // ---------------------------------------------------------------------

    pub fn state(&self) -> &WorldState {
        &self.state
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn module_registry(&self) -> &ModuleRegistry {
        &self.module_registry
    }

    pub fn module_limits_max(&self) -> &ModuleLimits {
        &self.module_limits_max
    }

    pub fn module_cache_len(&self) -> usize {
        self.module_cache.len()
    }

    pub fn snapshot_catalog(&self) -> &SnapshotCatalog {
        &self.snapshot_catalog
    }

    pub fn journal(&self) -> &Journal {
        &self.journal
    }

    pub fn policies(&self) -> &PolicySet {
        &self.policies
    }

    pub fn capabilities(&self) -> &BTreeMap<String, CapabilityGrant> {
        &self.capabilities
    }

    pub fn proposals(&self) -> &BTreeMap<ProposalId, Proposal> {
        &self.proposals
    }

    pub fn runtime_backpressure_stats(&self) -> &WorldRuntimeBackpressureStats {
        &self.runtime_backpressure_stats
    }

    pub fn with_runtime_memory_limits(mut self, limits: WorldRuntimeMemoryLimits) -> Self {
        self.runtime_memory_limits = limits;
        self.enforce_runtime_memory_limits();
        self
    }

    pub(super) fn allocate_next_event_id(&mut self) -> WorldEventId {
        Self::allocate_rolling_sequence_id(&mut self.next_event_id, &mut self.next_event_id_era)
    }

    pub(super) fn allocate_next_action_id(&mut self) -> ActionId {
        Self::allocate_rolling_sequence_id(&mut self.next_action_id, &mut self.next_action_id_era)
    }

    pub(super) fn allocate_next_intent_seq(&mut self) -> IntentSeq {
        Self::allocate_rolling_sequence_id(&mut self.next_intent_id, &mut self.next_intent_id_era)
    }

    pub(super) fn allocate_next_proposal_id(&mut self) -> ProposalId {
        Self::allocate_rolling_sequence_id(
            &mut self.next_proposal_id,
            &mut self.next_proposal_id_era,
        )
    }

    fn allocate_rolling_sequence_id(next_id: &mut u64, era: &mut u64) -> u64 {
        if *next_id == 0 {
            *next_id = 1;
        }
        let allocated = *next_id;
        if allocated == u64::MAX {
            *next_id = 1;
            *era = era.saturating_add(1);
        } else {
            *next_id = allocated + 1;
        }
        allocated
    }

    pub(super) fn enforce_pending_action_limit(&mut self) {
        let max_len = self.runtime_memory_limits.max_pending_actions.max(1);
        while self.pending_actions.len() > max_len {
            let _ = self.pending_actions.pop_front();
            self.runtime_backpressure_stats.pending_actions_evicted = self
                .runtime_backpressure_stats
                .pending_actions_evicted
                .saturating_add(1);
        }
    }

    pub(super) fn push_pending_effect_bounded(&mut self, intent: EffectIntent) {
        self.pending_effects.push_back(intent);
        self.enforce_pending_effect_limit();
    }

    pub(super) fn enforce_pending_effect_limit(&mut self) {
        let max_len = self.runtime_memory_limits.max_pending_effects.max(1);
        while self.pending_effects.len() > max_len {
            let _ = self.pending_effects.pop_front();
            self.runtime_backpressure_stats.pending_effects_evicted = self
                .runtime_backpressure_stats
                .pending_effects_evicted
                .saturating_add(1);
        }
    }

    pub(super) fn inflight_effect_capacity_reached(&self) -> bool {
        self.inflight_effects.len() >= self.runtime_memory_limits.max_inflight_effects.max(1)
    }

    pub(super) fn record_inflight_effect_dispatch_blocked(&mut self) {
        self.runtime_backpressure_stats
            .inflight_effect_dispatch_blocked = self
            .runtime_backpressure_stats
            .inflight_effect_dispatch_blocked
            .saturating_add(1);
    }

    pub(super) fn enforce_inflight_effect_limit(&mut self) {
        let max_len = self.runtime_memory_limits.max_inflight_effects.max(1);
        while self.inflight_effects.len() > max_len {
            if let Some(first_key) = self.inflight_effects.keys().next().cloned() {
                self.inflight_effects.remove(first_key.as_str());
                self.runtime_backpressure_stats.inflight_effects_evicted = self
                    .runtime_backpressure_stats
                    .inflight_effects_evicted
                    .saturating_add(1);
            } else {
                break;
            }
        }
    }

    pub(super) fn enforce_journal_event_limit(&mut self) {
        let max_len = self.runtime_memory_limits.max_journal_events.max(1);
        let overflow = self.journal.events.len().saturating_sub(max_len);
        if overflow > 0 {
            self.journal.events.drain(0..overflow);
            self.runtime_backpressure_stats.journal_events_evicted = self
                .runtime_backpressure_stats
                .journal_events_evicted
                .saturating_add(overflow as u64);
        }
    }

    pub(super) fn enforce_runtime_memory_limits(&mut self) {
        self.enforce_pending_action_limit();
        self.enforce_pending_effect_limit();
        self.enforce_inflight_effect_limit();
        self.enforce_journal_event_limit();
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
