use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::collections::VecDeque;

use super::World;
use super::super::{Journal, ModuleCache, ModuleStore, RollbackEvent, Snapshot, WorldError};
use super::super::util::hash_json;

impl World {
    // ---------------------------------------------------------------------
    // Persistence
    // ---------------------------------------------------------------------

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            snapshot_catalog: self.snapshot_catalog.clone(),
            manifest: self.manifest.clone(),
            module_registry: self.module_registry.clone(),
            module_artifacts: self.module_artifacts.clone(),
            module_limits_max: self.module_limits_max.clone(),
            state: self.state.clone(),
            journal_len: self.journal.len(),
            last_event_id: self.next_event_id.saturating_sub(1),
            next_action_id: self.next_action_id,
            next_intent_id: self.next_intent_id,
            next_proposal_id: self.next_proposal_id,
            pending_actions: self.pending_actions.iter().cloned().collect(),
            pending_effects: self.pending_effects.iter().cloned().collect(),
            inflight_effects: self.inflight_effects.clone(),
            capabilities: self.capabilities.clone(),
            policies: self.policies.clone(),
            proposals: self.proposals.clone(),
            scheduler_cursor: self.scheduler_cursor.clone(),
        }
    }

    pub fn save_to_dir(&self, dir: impl AsRef<Path>) -> Result<(), WorldError> {
        let dir = dir.as_ref();
        fs::create_dir_all(dir)?;
        let journal_path = dir.join("journal.json");
        let snapshot_path = dir.join("snapshot.json");
        self.journal.save_json(journal_path)?;
        self.snapshot().save_json(snapshot_path)?;
        Ok(())
    }

    pub fn save_to_dir_with_modules(&self, dir: impl AsRef<Path>) -> Result<(), WorldError> {
        let dir = dir.as_ref();
        self.save_to_dir(dir)?;
        self.save_module_store_to_dir(dir)?;
        Ok(())
    }

    pub fn save_module_store_to_dir(&self, dir: impl AsRef<Path>) -> Result<(), WorldError> {
        let store = ModuleStore::new(dir);
        store.save_registry(&self.module_registry)?;
        for record in self.module_registry.records.values() {
            store.write_meta(&record.manifest)?;
            let wasm_hash = &record.manifest.wasm_hash;
            let bytes = self
                .module_artifact_bytes
                .get(wasm_hash)
                .ok_or_else(|| WorldError::ModuleStoreArtifactMissing {
                    wasm_hash: wasm_hash.clone(),
                })?;
            store.write_artifact(wasm_hash, bytes)?;
        }
        Ok(())
    }

    pub fn load_from_dir(dir: impl AsRef<Path>) -> Result<Self, WorldError> {
        let dir = dir.as_ref();
        let journal_path = dir.join("journal.json");
        let snapshot_path = dir.join("snapshot.json");
        let journal = Journal::load_json(journal_path)?;
        let snapshot = Snapshot::load_json(snapshot_path)?;
        Self::from_snapshot(snapshot, journal)
    }

    pub fn load_from_dir_with_modules(dir: impl AsRef<Path>) -> Result<Self, WorldError> {
        let dir = dir.as_ref();
        let mut world = Self::load_from_dir(dir)?;
        world.load_module_store_from_dir(dir)?;
        Ok(world)
    }

    pub fn load_module_store_from_dir(
        &mut self,
        dir: impl AsRef<Path>,
    ) -> Result<(), WorldError> {
        let store = ModuleStore::new(dir);
        let registry = store.load_registry()?;
        self.module_registry = registry;
        self.module_artifacts.clear();
        self.module_artifact_bytes.clear();

        for record in self.module_registry.records.values() {
            let wasm_hash = &record.manifest.wasm_hash;
            let meta = store.read_meta(wasm_hash)?;
            if meta != record.manifest {
                return Err(WorldError::ModuleStoreManifestMismatch {
                    wasm_hash: wasm_hash.clone(),
                });
            }
            let bytes = store.read_artifact(wasm_hash)?;
            self.module_artifacts.insert(wasm_hash.clone());
            self.module_artifact_bytes.insert(wasm_hash.clone(), bytes);
        }
        Ok(())
    }

    pub fn rollback_to_snapshot(
        &mut self,
        snapshot: Snapshot,
        mut journal: Journal,
        reason: impl Into<String>,
    ) -> Result<(), WorldError> {
        if snapshot.journal_len > journal.len() {
            return Err(WorldError::JournalMismatch);
        }

        let prior_len = journal.len();
        journal.events.truncate(snapshot.journal_len);

        let signer = self.receipt_signer.clone();
        let mut world = Self::from_snapshot(snapshot.clone(), journal)?;
        world.receipt_signer = signer;

        let snapshot_hash = hash_json(&snapshot)?;
        let event = RollbackEvent {
            snapshot_hash,
            snapshot_journal_len: snapshot.journal_len,
            prior_journal_len: prior_len,
            reason: reason.into(),
        };
        world.append_event(super::super::WorldEventBody::RollbackApplied(event), None)?;
        *self = world;
        Ok(())
    }

    pub fn from_snapshot(snapshot: Snapshot, journal: Journal) -> Result<Self, WorldError> {
        if snapshot.journal_len > journal.len() {
            return Err(WorldError::JournalMismatch);
        }

        let mut world = Self::new_with_state(snapshot.state);
        world.journal = journal;
        world.manifest = snapshot.manifest;
        world.module_registry = snapshot.module_registry;
        world.module_artifacts = snapshot.module_artifacts;
        world.module_artifact_bytes = BTreeMap::new();
        world.module_cache = ModuleCache::default();
        world.module_limits_max = snapshot.module_limits_max;
        world.snapshot_catalog = snapshot.snapshot_catalog;
        world.next_event_id = snapshot.last_event_id.saturating_add(1);
        world.next_action_id = snapshot.next_action_id;
        world.next_intent_id = snapshot.next_intent_id;
        world.next_proposal_id = snapshot.next_proposal_id;
        world.pending_actions = VecDeque::from(snapshot.pending_actions);
        world.pending_effects = VecDeque::from(snapshot.pending_effects);
        world.inflight_effects = snapshot.inflight_effects;
        world.capabilities = snapshot.capabilities;
        world.policies = snapshot.policies;
        world.proposals = snapshot.proposals;
        world.scheduler_cursor = snapshot.scheduler_cursor;
        world.replay_from(snapshot.journal_len)?;
        Ok(world)
    }
}
