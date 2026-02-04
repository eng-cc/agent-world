//! Snapshot and journal types for world state persistence.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

use super::effect::{CapabilityGrant, EffectIntent};
use super::error::WorldError;
use super::events::ActionEnvelope;
use super::governance::Proposal;
use super::manifest::Manifest;
use super::modules::ModuleRegistry;
use super::policy::PolicySet;
use super::state::WorldState;
use super::types::{ActionId, IntentSeq, ProposalId, WorldEventId, WorldTime};
use super::util::{read_json_from_path, write_json_to_path};
use super::world_event::WorldEvent;

/// Policy for how many snapshots to retain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotRetentionPolicy {
    pub max_snapshots: usize,
}

impl Default for SnapshotRetentionPolicy {
    fn default() -> Self {
        Self { max_snapshots: 10 }
    }
}

/// A record of a saved snapshot for catalog purposes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotRecord {
    pub snapshot_hash: String,
    pub journal_len: usize,
    pub created_at: WorldTime,
    pub manifest_hash: String,
}

/// Catalog of all recorded snapshots with retention policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotCatalog {
    pub records: Vec<SnapshotRecord>,
    pub retention: SnapshotRetentionPolicy,
}

impl Default for SnapshotCatalog {
    fn default() -> Self {
        Self {
            records: Vec::new(),
            retention: SnapshotRetentionPolicy::default(),
        }
    }
}

/// A complete snapshot of the world state at a point in time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub snapshot_catalog: SnapshotCatalog,
    pub manifest: Manifest,
    pub module_registry: ModuleRegistry,
    pub state: WorldState,
    pub journal_len: usize,
    pub last_event_id: WorldEventId,
    pub next_action_id: ActionId,
    pub next_intent_id: IntentSeq,
    pub next_proposal_id: ProposalId,
    pub pending_actions: Vec<ActionEnvelope>,
    pub pending_effects: Vec<EffectIntent>,
    pub inflight_effects: BTreeMap<String, EffectIntent>,
    pub capabilities: BTreeMap<String, CapabilityGrant>,
    pub policies: PolicySet,
    pub proposals: BTreeMap<ProposalId, Proposal>,
    pub scheduler_cursor: Option<String>,
}

impl Snapshot {
    pub fn to_json(&self) -> Result<String, WorldError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn from_json(input: &str) -> Result<Self, WorldError> {
        Ok(serde_json::from_str(input)?)
    }

    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<(), WorldError> {
        write_json_to_path(self, path.as_ref())
    }

    pub fn load_json(path: impl AsRef<Path>) -> Result<Self, WorldError> {
        read_json_from_path(path.as_ref())
    }
}

/// Metadata about a snapshot creation event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotMeta {
    pub journal_len: usize,
}

/// The journal containing all world events since the last snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Journal {
    pub events: Vec<WorldEvent>,
}

impl Journal {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn append(&mut self, event: WorldEvent) {
        self.events.push(event);
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn to_json(&self) -> Result<String, WorldError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn from_json(input: &str) -> Result<Self, WorldError> {
        Ok(serde_json::from_str(input)?)
    }

    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<(), WorldError> {
        write_json_to_path(self, path.as_ref())
    }

    pub fn load_json(path: impl AsRef<Path>) -> Result<Self, WorldError> {
        read_json_from_path(path.as_ref())
    }
}

impl Default for Journal {
    fn default() -> Self {
        Self::new()
    }
}

/// Event recorded when a rollback is applied.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RollbackEvent {
    pub snapshot_hash: String,
    pub snapshot_journal_len: usize,
    pub prior_journal_len: usize,
    pub reason: String,
}
