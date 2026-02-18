use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::fs;
use std::path::Path;

use agent_world_distfs::{assemble_journal, assemble_snapshot};
use agent_world_proto::distributed::SnapshotManifest;

use super::super::util::{hash_json, read_json_from_path, write_json_to_path};
use super::super::{
    segment_journal, segment_snapshot, Journal, JournalSegmentRef, LocalCasStore, ModuleCache,
    ModuleStore, RollbackEvent, SegmentConfig, Snapshot, WorldError, WorldEvent,
};
use super::World;

const JOURNAL_FILE: &str = "journal.json";
const SNAPSHOT_FILE: &str = "snapshot.json";
const DISTFS_STATE_DIR: &str = ".distfs-state";
const DISTFS_SNAPSHOT_MANIFEST_FILE: &str = "snapshot.manifest.json";
const DISTFS_JOURNAL_SEGMENTS_FILE: &str = "journal.segments.json";
const DISTFS_WORLD_ID_FALLBACK: &str = "runtime-world";

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
            module_tick_schedule: self.module_tick_schedule.clone(),
            capabilities: self.capabilities.clone(),
            policies: self.policies.clone(),
            proposals: self.proposals.clone(),
            scheduler_cursor: self.scheduler_cursor.clone(),
        }
    }

    pub fn save_to_dir(&self, dir: impl AsRef<Path>) -> Result<(), WorldError> {
        let dir = dir.as_ref();
        fs::create_dir_all(dir)?;
        let snapshot = self.snapshot();
        let journal_path = dir.join(JOURNAL_FILE);
        let snapshot_path = dir.join(SNAPSHOT_FILE);
        self.journal.save_json(journal_path)?;
        snapshot.save_json(snapshot_path)?;
        self.save_distfs_sidecar(dir, &snapshot)?;
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
            let bytes = self.module_artifact_bytes.get(wasm_hash).ok_or_else(|| {
                WorldError::ModuleStoreArtifactMissing {
                    wasm_hash: wasm_hash.clone(),
                }
            })?;
            store.write_artifact(wasm_hash, bytes)?;
        }
        Ok(())
    }

    pub fn load_from_dir(dir: impl AsRef<Path>) -> Result<Self, WorldError> {
        let dir = dir.as_ref();
        if let Some((snapshot, journal)) = Self::try_load_from_distfs_sidecar(dir)? {
            return Self::from_snapshot(snapshot, journal);
        }
        let journal_path = dir.join(JOURNAL_FILE);
        let snapshot_path = dir.join(SNAPSHOT_FILE);
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

    pub fn load_module_store_from_dir(&mut self, dir: impl AsRef<Path>) -> Result<(), WorldError> {
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
        world.module_tick_schedule = snapshot.module_tick_schedule;
        world.capabilities = snapshot.capabilities;
        world.policies = snapshot.policies;
        world.proposals = snapshot.proposals;
        world.scheduler_cursor = snapshot.scheduler_cursor;
        world.replay_from(snapshot.journal_len)?;
        Ok(world)
    }

    fn save_distfs_sidecar(&self, dir: &Path, snapshot: &Snapshot) -> Result<(), WorldError> {
        let store_root = dir.join(DISTFS_STATE_DIR);
        fs::create_dir_all(store_root.as_path())?;
        let store = LocalCasStore::new(store_root.as_path());
        let config = SegmentConfig::default();
        let world_id = distfs_world_id(dir);
        let epoch = snapshot.state.time;
        let manifest = segment_snapshot(snapshot, world_id.as_str(), epoch, &store, config)?;
        let journal_segments = segment_journal(&self.journal, &store, config)?;

        let restored_snapshot: Snapshot = assemble_snapshot(&manifest, &store)?;
        if restored_snapshot != *snapshot {
            return Err(WorldError::DistributedValidationFailed {
                reason: "distfs snapshot assemble verification mismatch".to_string(),
            });
        }

        let restored_events: Vec<WorldEvent> =
            assemble_journal(&journal_segments, &store, |event: &WorldEvent| event.id)?;
        if restored_events != self.journal.events {
            return Err(WorldError::DistributedValidationFailed {
                reason: "distfs journal assemble verification mismatch".to_string(),
            });
        }

        let snapshot_manifest_path = dir.join(DISTFS_SNAPSHOT_MANIFEST_FILE);
        let journal_segments_path = dir.join(DISTFS_JOURNAL_SEGMENTS_FILE);
        write_json_to_path(&manifest, snapshot_manifest_path.as_path())?;
        write_json_to_path(&journal_segments, journal_segments_path.as_path())?;
        Ok(())
    }

    fn try_load_from_distfs_sidecar(dir: &Path) -> Result<Option<(Snapshot, Journal)>, WorldError> {
        let snapshot_manifest_path = dir.join(DISTFS_SNAPSHOT_MANIFEST_FILE);
        let journal_segments_path = dir.join(DISTFS_JOURNAL_SEGMENTS_FILE);
        let store_root = dir.join(DISTFS_STATE_DIR);
        if !snapshot_manifest_path.exists()
            || !journal_segments_path.exists()
            || !store_root.exists()
        {
            return Ok(None);
        }

        let restored = Self::load_from_distfs_sidecar(
            snapshot_manifest_path.as_path(),
            journal_segments_path.as_path(),
            store_root.as_path(),
        );
        match restored {
            Ok(value) => Ok(Some(value)),
            Err(_) => Ok(None),
        }
    }

    fn load_from_distfs_sidecar(
        snapshot_manifest_path: &Path,
        journal_segments_path: &Path,
        store_root: &Path,
    ) -> Result<(Snapshot, Journal), WorldError> {
        let manifest: SnapshotManifest = read_json_from_path(snapshot_manifest_path)?;
        let journal_segments: Vec<JournalSegmentRef> = read_json_from_path(journal_segments_path)?;
        let store = LocalCasStore::new(store_root);
        let snapshot: Snapshot = assemble_snapshot(&manifest, &store)?;
        let events: Vec<WorldEvent> =
            assemble_journal(&journal_segments, &store, |event: &WorldEvent| event.id)?;
        Ok((snapshot, Journal { events }))
    }
}

fn distfs_world_id(dir: &Path) -> String {
    dir.file_name()
        .and_then(|name| name.to_str())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or(DISTFS_WORLD_ID_FALLBACK)
        .to_string()
}
