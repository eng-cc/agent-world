//! The World struct - core runtime implementation.

mod actions;
mod audit;
mod body;
mod bootstrap_economy;
mod bootstrap_power;
mod economy;
mod effects;
mod event_processing;
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
    next_action_id: ActionId,
    next_intent_id: IntentSeq,
    next_proposal_id: ProposalId,
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
}

impl World {
    pub fn new() -> Self {
        Self::new_with_state(WorldState::default())
    }

    pub fn new_with_state(mut state: WorldState) -> Self {
        state.migrate_legacy_material_ledgers();
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
            next_action_id: 1,
            next_intent_id: 1,
            next_proposal_id: 1,
            pending_actions: VecDeque::new(),
            pending_effects: VecDeque::new(),
            inflight_effects: BTreeMap::new(),
            module_tick_schedule: BTreeMap::new(),
            capabilities: BTreeMap::new(),
            policies: PolicySet::default(),
            proposals: BTreeMap::new(),
            scheduler_cursor: None,
            receipt_signer: None,
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
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
