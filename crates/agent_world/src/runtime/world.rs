//! The World struct - core runtime implementation.

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::Path;

use super::audit::AuditFilter;
use super::effect::{CapabilityGrant, EffectIntent, EffectOrigin, EffectReceipt};
use super::error::WorldError;
use super::events::{Action, ActionEnvelope, CausedBy, DomainEvent, RejectReason};
use super::governance::{AgentSchedule, GovernanceEvent, Proposal, ProposalDecision, ProposalStatus};
use super::manifest::{apply_manifest_patch, Manifest, ManifestPatch, ManifestUpdate};
use super::modules::{
    ModuleArtifact, ModuleCache, ModuleChangeSet, ModuleEvent, ModuleEventKind, ModuleLimits,
    ModuleManifest, ModuleRegistry, ModuleRecord, ModuleSubscription,
};
use super::module_store::ModuleStore;
use super::sandbox::{
    ModuleCallErrorCode, ModuleCallFailure, ModuleCallRequest, ModuleEmitEvent, ModuleOutput,
    ModuleSandbox,
};
use super::policy::{PolicyDecisionRecord, PolicySet};
use super::signer::ReceiptSigner;
use super::snapshot::{Journal, RollbackEvent, Snapshot, SnapshotCatalog, SnapshotRecord, SnapshotRetentionPolicy};
use super::state::WorldState;
use super::types::{ActionId, IntentSeq, ProposalId, WorldEventId};
use super::util::{hash_json, to_canonical_cbor, write_json_to_path};
use super::world_event::{WorldEvent, WorldEventBody};

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

    pub fn new_with_state(state: WorldState) -> Self {
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
            capabilities: BTreeMap::new(),
            policies: PolicySet::default(),
            proposals: BTreeMap::new(),
            scheduler_cursor: None,
            receipt_signer: None,
        }
    }

    // -------------------------------------------------------------------------
    // Accessors
    // -------------------------------------------------------------------------

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

    // -------------------------------------------------------------------------
    // Audit
    // -------------------------------------------------------------------------

    pub fn audit_events(&self, filter: &AuditFilter) -> Vec<WorldEvent> {
        self.journal
            .events
            .iter()
            .filter(|event| filter.matches(event))
            .cloned()
            .collect()
    }

    pub fn save_audit_log(
        &self,
        path: impl AsRef<Path>,
        filter: &AuditFilter,
    ) -> Result<(), WorldError> {
        let events = self.audit_events(filter);
        write_json_to_path(&events, path.as_ref())
    }

    // -------------------------------------------------------------------------
    // Snapshot management
    // -------------------------------------------------------------------------

    pub fn set_snapshot_retention(&mut self, policy: SnapshotRetentionPolicy) {
        self.snapshot_catalog.retention = policy;
        self.apply_snapshot_retention();
    }

    pub fn create_snapshot(&mut self) -> Result<Snapshot, WorldError> {
        let snapshot = self.snapshot();
        self.record_snapshot(&snapshot)?;
        Ok(snapshot)
    }

    pub fn record_snapshot(&mut self, snapshot: &Snapshot) -> Result<SnapshotRecord, WorldError> {
        let snapshot_hash = hash_json(snapshot)?;
        let manifest_hash = hash_json(&snapshot.manifest)?;
        let record = SnapshotRecord {
            snapshot_hash,
            journal_len: snapshot.journal_len,
            created_at: snapshot.state.time,
            manifest_hash,
        };
        self.snapshot_catalog.records.push(record.clone());
        self.apply_snapshot_retention();
        Ok(record)
    }

    pub fn save_snapshot_to_dir(
        &mut self,
        dir: impl AsRef<Path>,
    ) -> Result<SnapshotRecord, WorldError> {
        let snapshot = self.snapshot();
        let record = self.record_snapshot(&snapshot)?;

        let dir = dir.as_ref().join("snapshots");
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.json", record.snapshot_hash));
        write_json_to_path(&snapshot, &path)?;

        self.prune_snapshot_files(&dir)?;
        Ok(record)
    }

    pub fn prune_snapshot_files(&self, dir: impl AsRef<Path>) -> Result<(), WorldError> {
        let dir = dir.as_ref();
        if !dir.exists() {
            return Ok(());
        }

        let keep: BTreeSet<String> = self
            .snapshot_catalog
            .records
            .iter()
            .map(|record| format!("{}.json", record.snapshot_hash))
            .collect();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let file_name = match entry.file_name().into_string() {
                Ok(name) => name,
                Err(_) => continue,
            };
            if file_name.ends_with(".json") && !keep.contains(&file_name) {
                let _ = fs::remove_file(entry.path());
            }
        }
        Ok(())
    }

    fn apply_snapshot_retention(&mut self) {
        let max = self.snapshot_catalog.retention.max_snapshots;
        if max == 0 {
            self.snapshot_catalog.records.clear();
            return;
        }
        if self.snapshot_catalog.records.len() > max {
            let excess = self.snapshot_catalog.records.len() - max;
            self.snapshot_catalog.records.drain(0..excess);
        }
    }

    // -------------------------------------------------------------------------
    // Policy and capability management
    // -------------------------------------------------------------------------

    pub fn set_policy(&mut self, policy: PolicySet) {
        self.policies = policy;
    }

    pub fn add_capability(&mut self, grant: CapabilityGrant) {
        self.capabilities.insert(grant.name.clone(), grant);
    }

    pub fn set_receipt_signer(&mut self, signer: ReceiptSigner) {
        self.receipt_signer = Some(signer);
    }

    // -------------------------------------------------------------------------
    // Module artifact and limits
    // -------------------------------------------------------------------------

    pub fn register_module_artifact(
        &mut self,
        wasm_hash: impl Into<String>,
        bytes: &[u8],
    ) -> Result<(), WorldError> {
        let wasm_hash = wasm_hash.into();
        let computed = super::util::sha256_hex(bytes);
        if computed != wasm_hash {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("artifact hash mismatch expected {wasm_hash} found {computed}"),
            });
        }
        self.module_artifacts.insert(wasm_hash);
        self.module_artifact_bytes
            .insert(computed, bytes.to_vec());
        Ok(())
    }

    pub fn set_module_limits_max(&mut self, limits: ModuleLimits) {
        self.module_limits_max = limits;
    }

    pub fn set_module_cache_max(&mut self, max_cached_modules: usize) {
        self.module_cache.set_max_cached_modules(max_cached_modules);
    }

    pub fn load_module(&mut self, wasm_hash: &str) -> Result<ModuleArtifact, WorldError> {
        if let Some(artifact) = self.module_cache.get(wasm_hash) {
            return Ok(artifact);
        }
        let bytes = self
            .module_artifact_bytes
            .get(wasm_hash)
            .ok_or_else(|| WorldError::ModuleChangeInvalid {
                reason: format!("module artifact bytes missing {wasm_hash}"),
            })?
            .clone();
        let artifact = ModuleArtifact {
            wasm_hash: wasm_hash.to_string(),
            bytes,
        };
        self.module_cache.insert(artifact.clone());
        Ok(artifact)
    }

    pub fn validate_module_output_limits(
        &self,
        module_id: &str,
        limits: &ModuleLimits,
        effect_count: usize,
        emit_count: usize,
        output_bytes: u64,
    ) -> Result<(), WorldError> {
        if effect_count as u32 > limits.max_effects {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module output effects exceeded {module_id}"),
            });
        }
        if emit_count as u32 > limits.max_emits {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module output emits exceeded {module_id}"),
            });
        }
        if output_bytes > limits.max_output_bytes {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module output bytes exceeded {module_id}"),
            });
        }
        Ok(())
    }

    pub fn execute_module_call(
        &mut self,
        module_id: &str,
        trace_id: impl Into<String>,
        input: Vec<u8>,
        sandbox: &mut dyn ModuleSandbox,
    ) -> Result<ModuleOutput, WorldError> {
        let trace_id = trace_id.into();
        let manifest = self.active_module_manifest(module_id)?.clone();
        let wasm_hash = manifest.wasm_hash.clone();
        let artifact = self.load_module(&wasm_hash)?;

        let request = ModuleCallRequest {
            module_id: module_id.to_string(),
            wasm_hash,
            trace_id: trace_id.clone(),
            input,
            limits: manifest.limits.clone(),
            wasm_bytes: artifact.bytes,
        };

        let output = match sandbox.call(&request) {
            Ok(output) => output,
            Err(failure) => {
                self.append_event(
                    WorldEventBody::ModuleCallFailed(failure.clone()),
                    None,
                )?;
                return Err(WorldError::ModuleCallFailed {
                    module_id: failure.module_id,
                    trace_id: failure.trace_id,
                    code: failure.code,
                    detail: failure.detail,
                });
            }
        };

        self.process_module_output(module_id, &trace_id, &manifest, &output)?;
        Ok(output)
    }

    pub fn route_event_to_modules(
        &mut self,
        event: &WorldEvent,
        sandbox: &mut dyn ModuleSandbox,
    ) -> Result<usize, WorldError> {
        let event_kind = event_kind_label(&event.body);
        let mut module_ids: Vec<String> =
            self.module_registry.active.keys().cloned().collect();
        module_ids.sort();
        let mut invoked = 0;
        for module_id in module_ids {
            let subscribed = {
                let version = self
                    .module_registry
                    .active
                    .get(&module_id)
                    .ok_or_else(|| WorldError::ModuleChangeInvalid {
                        reason: format!("module not active {module_id}"),
                    })?;
                let key = ModuleRegistry::record_key(&module_id, version);
                let record = self
                    .module_registry
                    .records
                    .get(&key)
                    .ok_or_else(|| WorldError::ModuleChangeInvalid {
                        reason: format!("module record missing {key}"),
                    })?;
                module_subscribes_to_event(&record.manifest.subscriptions, event_kind)
            };
            if !subscribed {
                continue;
            }

            let input = to_canonical_cbor(event)?;
            let trace_id = format!("event-{}-{}", event.id, module_id);
            self.execute_module_call(&module_id, trace_id, input, sandbox)?;
            invoked += 1;
        }
        Ok(invoked)
    }

    pub fn route_action_to_modules(
        &mut self,
        envelope: &ActionEnvelope,
        sandbox: &mut dyn ModuleSandbox,
    ) -> Result<usize, WorldError> {
        let action_kind = action_kind_label(&envelope.action);
        let mut module_ids: Vec<String> =
            self.module_registry.active.keys().cloned().collect();
        module_ids.sort();
        let input = to_canonical_cbor(envelope)?;
        let mut invoked = 0;

        for module_id in module_ids {
            let subscribed = {
                let version = self
                    .module_registry
                    .active
                    .get(&module_id)
                    .ok_or_else(|| WorldError::ModuleChangeInvalid {
                        reason: format!("module not active {module_id}"),
                    })?;
                let key = ModuleRegistry::record_key(&module_id, version);
                let record = self
                    .module_registry
                    .records
                    .get(&key)
                    .ok_or_else(|| WorldError::ModuleChangeInvalid {
                        reason: format!("module record missing {key}"),
                    })?;
                module_subscribes_to_action(&record.manifest.subscriptions, action_kind)
            };
            if !subscribed {
                continue;
            }

            let trace_id = format!("action-{}-{}", envelope.id, module_id);
            self.execute_module_call(&module_id, trace_id, input.clone(), sandbox)?;
            invoked += 1;
        }

        Ok(invoked)
    }

    // -------------------------------------------------------------------------
    // Manifest and governance
    // -------------------------------------------------------------------------

    pub fn current_manifest_hash(&self) -> Result<String, WorldError> {
        hash_json(&self.manifest)
    }

    pub fn propose_manifest_update(
        &mut self,
        manifest: Manifest,
        author: impl Into<String>,
    ) -> Result<ProposalId, WorldError> {
        let proposal_id = self.next_proposal_id;
        self.next_proposal_id += 1;
        let base_manifest_hash = self.current_manifest_hash()?;
        let event = GovernanceEvent::Proposed {
            proposal_id,
            author: author.into(),
            base_manifest_hash,
            manifest,
            patch: None,
        };
        self.append_event(WorldEventBody::Governance(event), None)?;
        Ok(proposal_id)
    }

    pub fn propose_manifest_patch(
        &mut self,
        patch: ManifestPatch,
        author: impl Into<String>,
    ) -> Result<ProposalId, WorldError> {
        let base_manifest_hash = self.current_manifest_hash()?;
        if patch.base_manifest_hash != base_manifest_hash {
            return Err(WorldError::PatchBaseMismatch {
                expected: base_manifest_hash,
                found: patch.base_manifest_hash,
            });
        }

        let manifest = apply_manifest_patch(&self.manifest, &patch)?;
        let proposal_id = self.next_proposal_id;
        self.next_proposal_id += 1;
        let event = GovernanceEvent::Proposed {
            proposal_id,
            author: author.into(),
            base_manifest_hash,
            manifest,
            patch: Some(patch),
        };
        self.append_event(WorldEventBody::Governance(event), None)?;
        Ok(proposal_id)
    }

    pub fn shadow_proposal(&mut self, proposal_id: ProposalId) -> Result<String, WorldError> {
        let proposal = self
            .proposals
            .get(&proposal_id)
            .ok_or(WorldError::ProposalNotFound { proposal_id })?;
        if !matches!(proposal.status, ProposalStatus::Proposed) {
            return Err(WorldError::ProposalInvalidState {
                proposal_id,
                expected: "proposed".to_string(),
                found: proposal.status.label(),
            });
        }
        if let Some(changes) = proposal.manifest.module_changes()? {
            self.validate_module_changes(&changes)?;
            self.shadow_validate_module_changes(&changes)?;
        }
        let manifest_hash = hash_json(&proposal.manifest)?;
        let event = GovernanceEvent::ShadowReport {
            proposal_id,
            manifest_hash: manifest_hash.clone(),
        };
        self.append_event(WorldEventBody::Governance(event), None)?;
        Ok(manifest_hash)
    }

    pub fn approve_proposal(
        &mut self,
        proposal_id: ProposalId,
        approver: impl Into<String>,
        decision: ProposalDecision,
    ) -> Result<(), WorldError> {
        let proposal = self
            .proposals
            .get(&proposal_id)
            .ok_or(WorldError::ProposalNotFound { proposal_id })?;

        match (&decision, &proposal.status) {
            (ProposalDecision::Approve, ProposalStatus::Shadowed { .. }) => {}
            (ProposalDecision::Reject { .. }, ProposalStatus::Applied { .. })
            | (ProposalDecision::Reject { .. }, ProposalStatus::Rejected { .. }) => {
                return Err(WorldError::ProposalInvalidState {
                    proposal_id,
                    expected: "proposed".to_string(),
                    found: proposal.status.label(),
                });
            }
            (ProposalDecision::Approve, _) => {
                return Err(WorldError::ProposalInvalidState {
                    proposal_id,
                    expected: "shadowed".to_string(),
                    found: proposal.status.label(),
                });
            }
            _ => {}
        }

        let event = GovernanceEvent::Approved {
            proposal_id,
            approver: approver.into(),
            decision,
        };
        self.append_event(WorldEventBody::Governance(event), None)?;
        Ok(())
    }

    pub fn apply_proposal(&mut self, proposal_id: ProposalId) -> Result<String, WorldError> {
        let proposal = self
            .proposals
            .get(&proposal_id)
            .ok_or(WorldError::ProposalNotFound { proposal_id })?;
        let (manifest, actor) = match &proposal.status {
            ProposalStatus::Approved { .. } => (proposal.manifest.clone(), proposal.author.clone()),
            other => {
                return Err(WorldError::ProposalInvalidState {
                    proposal_id,
                    expected: "approved".to_string(),
                    found: other.label(),
                })
            }
        };

        let module_changes = manifest.module_changes()?;
        if let Some(changes) = &module_changes {
            self.validate_module_changes(changes)?;
        }
        let applied_manifest = if module_changes.is_some() {
            manifest.without_module_changes()?
        } else {
            manifest.clone()
        };
        let applied_hash = hash_json(&applied_manifest)?;

        let event = GovernanceEvent::Applied {
            proposal_id,
            manifest_hash: Some(applied_hash.clone()),
        };
        self.append_event(WorldEventBody::Governance(event), None)?;
        if let Some(changes) = module_changes {
            self.apply_module_changes(proposal_id, &changes, &actor)?;
        }
        let update = ManifestUpdate {
            manifest: applied_manifest,
            manifest_hash: applied_hash.clone(),
        };
        self.append_event(WorldEventBody::ManifestUpdated(update), None)?;
        Ok(applied_hash)
    }

    // -------------------------------------------------------------------------
    // Action submission
    // -------------------------------------------------------------------------

    pub fn submit_action(&mut self, action: Action) -> ActionId {
        let action_id = self.next_action_id;
        self.next_action_id += 1;
        self.pending_actions
            .push_back(ActionEnvelope { id: action_id, action });
        action_id
    }

    pub fn pending_actions_len(&self) -> usize {
        self.pending_actions.len()
    }

    pub fn pending_effects_len(&self) -> usize {
        self.pending_effects.len()
    }

    // -------------------------------------------------------------------------
    // Effect handling
    // -------------------------------------------------------------------------

    pub fn take_next_effect(&mut self) -> Option<EffectIntent> {
        let intent = self.pending_effects.pop_front()?;
        self.inflight_effects
            .insert(intent.intent_id.clone(), intent.clone());
        Some(intent)
    }

    pub fn emit_effect(
        &mut self,
        kind: impl Into<String>,
        params: JsonValue,
        cap_ref: impl Into<String>,
        origin: EffectOrigin,
    ) -> Result<String, WorldError> {
        let intent = self.build_effect_intent(kind, params, cap_ref, origin)?;
        let intent_id = intent.intent_id.clone();
        self.append_event(WorldEventBody::EffectQueued(intent), None)?;
        Ok(intent_id)
    }

    fn build_effect_intent(
        &mut self,
        kind: impl Into<String>,
        params: JsonValue,
        cap_ref: impl Into<String>,
        origin: EffectOrigin,
    ) -> Result<EffectIntent, WorldError> {
        let kind = kind.into();
        let cap_ref = cap_ref.into();
        let intent_id = format!("intent-{}", self.next_intent_id);
        self.next_intent_id += 1;

        let intent = EffectIntent {
            intent_id: intent_id.clone(),
            kind: kind.clone(),
            params,
            cap_ref: cap_ref.clone(),
            origin,
        };

        let grant = self
            .capabilities
            .get(&cap_ref)
            .ok_or_else(|| WorldError::CapabilityMissing { cap_ref: cap_ref.clone() })?;

        if grant.is_expired(self.state.time) {
            return Err(WorldError::CapabilityExpired { cap_ref });
        }

        if !grant.allows(&kind) {
            return Err(WorldError::CapabilityNotAllowed { cap_ref, kind });
        }

        let decision = self.policies.decide(&intent);
        let record = PolicyDecisionRecord::from_intent(&intent, decision.clone());
        self.append_event(WorldEventBody::PolicyDecisionRecorded(record), None)?;

        if !decision.is_allowed() {
            return Err(WorldError::PolicyDenied {
                intent_id,
                reason: decision.reason().unwrap_or_else(|| "policy_deny".to_string()),
            });
        }

        Ok(intent)
    }

    pub fn ingest_receipt(&mut self, mut receipt: EffectReceipt) -> Result<WorldEventId, WorldError> {
        let known = self.inflight_effects.contains_key(&receipt.intent_id)
            || self
                .pending_effects
                .iter()
                .any(|intent| intent.intent_id == receipt.intent_id);
        if !known {
            return Err(WorldError::ReceiptUnknownIntent {
                intent_id: receipt.intent_id,
            });
        }

        self.finalize_receipt_signature(&mut receipt)?;
        self.append_event(
            WorldEventBody::ReceiptAppended(receipt.clone()),
            Some(CausedBy::Effect {
                intent_id: receipt.intent_id,
            }),
        )
    }

    // -------------------------------------------------------------------------
    // Scheduling
    // -------------------------------------------------------------------------

    pub fn schedule_next(&mut self) -> Option<AgentSchedule> {
        let ready: Vec<String> = self
            .state
            .agents
            .iter()
            .filter(|(_, cell)| !cell.mailbox.is_empty())
            .map(|(id, _)| id.clone())
            .collect();

        if ready.is_empty() {
            return None;
        }

        let next_id = match &self.scheduler_cursor {
            None => ready[0].clone(),
            Some(cursor) => ready
                .iter()
                .find(|id| id.as_str() > cursor.as_str())
                .cloned()
                .unwrap_or_else(|| ready[0].clone()),
        };

        let cell = self.state.agents.get_mut(&next_id)?;
        let event = cell.mailbox.pop_front()?;
        cell.last_active = self.state.time;
        self.scheduler_cursor = Some(next_id.clone());

        Some(AgentSchedule {
            agent_id: next_id,
            event,
        })
    }

    pub fn agent_mailbox_len(&self, agent_id: &str) -> Option<usize> {
        self.state
            .agents
            .get(agent_id)
            .map(|cell| cell.mailbox.len())
    }

    // -------------------------------------------------------------------------
    // Simulation step
    // -------------------------------------------------------------------------

    pub fn step(&mut self) -> Result<(), WorldError> {
        self.state.time = self.state.time.saturating_add(1);
        while let Some(envelope) = self.pending_actions.pop_front() {
            let event_body = self.action_to_event(&envelope)?;
            self.append_event(event_body, Some(CausedBy::Action(envelope.id)))?;
        }
        Ok(())
    }

    pub fn step_with_modules(
        &mut self,
        sandbox: &mut dyn ModuleSandbox,
    ) -> Result<(), WorldError> {
        self.state.time = self.state.time.saturating_add(1);
        while let Some(envelope) = self.pending_actions.pop_front() {
            self.route_action_to_modules(&envelope, sandbox)?;
            let event_body = self.action_to_event(&envelope)?;
            self.append_event(event_body, Some(CausedBy::Action(envelope.id)))?;
            if let Some(event) = self.journal.events.last() {
                let event = event.clone();
                self.route_event_to_modules(&event, sandbox)?;
            }
        }
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Persistence
    // -------------------------------------------------------------------------

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
        world.append_event(WorldEventBody::RollbackApplied(event), None)?;
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

    // -------------------------------------------------------------------------
    // Internal helpers
    // -------------------------------------------------------------------------

    fn replay_from(&mut self, start_index: usize) -> Result<(), WorldError> {
        let events: Vec<WorldEvent> = self.journal.events[start_index..].to_vec();
        for event in events {
            self.apply_event_body(&event.body, event.time)?;
            self.state.time = event.time;
            self.next_event_id = self.next_event_id.max(event.id.saturating_add(1));
        }
        Ok(())
    }

    fn action_to_event(&self, envelope: &ActionEnvelope) -> Result<WorldEventBody, WorldError> {
        let action_id = envelope.id;
        match &envelope.action {
            Action::RegisterAgent { agent_id, pos } => {
                if self.state.agents.contains_key(agent_id) {
                    Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                        action_id,
                        reason: RejectReason::AgentAlreadyExists {
                            agent_id: agent_id.clone(),
                        },
                    }))
                } else {
                    Ok(WorldEventBody::Domain(DomainEvent::AgentRegistered {
                        agent_id: agent_id.clone(),
                        pos: *pos,
                    }))
                }
            }
            Action::MoveAgent { agent_id, to } => match self.state.agents.get(agent_id) {
                Some(cell) => Ok(WorldEventBody::Domain(DomainEvent::AgentMoved {
                    agent_id: agent_id.clone(),
                    from: cell.state.pos,
                    to: *to,
                })),
                None => Ok(WorldEventBody::Domain(DomainEvent::ActionRejected {
                    action_id,
                    reason: RejectReason::AgentNotFound {
                        agent_id: agent_id.clone(),
                    },
                })),
            },
        }
    }

    fn append_event(
        &mut self,
        body: WorldEventBody,
        caused_by: Option<CausedBy>,
    ) -> Result<WorldEventId, WorldError> {
        self.apply_event_body(&body, self.state.time)?;
        let event_id = self.next_event_id;
        self.next_event_id += 1;
        self.journal.append(WorldEvent {
            id: event_id,
            time: self.state.time,
            caused_by,
            body,
        });
        Ok(event_id)
    }

    fn apply_event_body(&mut self, body: &WorldEventBody, time: super::types::WorldTime) -> Result<(), WorldError> {
        match body {
            WorldEventBody::Domain(event) => {
                self.state.apply_domain_event(event, time)?;
                self.state.route_domain_event(event);
            }
            WorldEventBody::EffectQueued(intent) => {
                self.pending_effects.push_back(intent.clone());
            }
            WorldEventBody::ReceiptAppended(receipt) => {
                let mut removed = false;
                if self.inflight_effects.remove(&receipt.intent_id).is_some() {
                    removed = true;
                }
                let before = self.pending_effects.len();
                self.pending_effects
                    .retain(|intent| intent.intent_id != receipt.intent_id);
                if before != self.pending_effects.len() {
                    removed = true;
                }
                if !removed {
                    return Err(WorldError::ReceiptUnknownIntent {
                        intent_id: receipt.intent_id.clone(),
                    });
                }
            }
            WorldEventBody::PolicyDecisionRecorded(_) => {}
            WorldEventBody::Governance(event) => {
                self.apply_governance_event(event)?;
            }
            WorldEventBody::ModuleEvent(event) => {
                self.apply_module_event(event, time)?;
            }
            WorldEventBody::ModuleCallFailed(_) => {}
            WorldEventBody::ModuleEmitted(_) => {}
            WorldEventBody::SnapshotCreated(_) => {}
            WorldEventBody::ManifestUpdated(update) => {
                self.manifest = update.manifest.clone();
            }
            WorldEventBody::RollbackApplied(_) => {}
        }
        self.state.time = time;
        Ok(())
    }

    fn apply_governance_event(&mut self, event: &GovernanceEvent) -> Result<(), WorldError> {
        match event {
            GovernanceEvent::Proposed {
                proposal_id,
                author,
                base_manifest_hash,
                manifest,
                patch,
            } => {
                let proposal = Proposal {
                    id: *proposal_id,
                    author: author.clone(),
                    base_manifest_hash: base_manifest_hash.clone(),
                    manifest: manifest.clone(),
                    patch: patch.clone(),
                    status: ProposalStatus::Proposed,
                };
                self.proposals.insert(*proposal_id, proposal);
                self.next_proposal_id = self.next_proposal_id.max(proposal_id.saturating_add(1));
            }
            GovernanceEvent::ShadowReport {
                proposal_id,
                manifest_hash,
            } => {
                let proposal = self
                    .proposals
                    .get_mut(proposal_id)
                    .ok_or(WorldError::ProposalNotFound {
                        proposal_id: *proposal_id,
                    })?;
                proposal.status = ProposalStatus::Shadowed {
                    manifest_hash: manifest_hash.clone(),
                };
            }
            GovernanceEvent::Approved {
                proposal_id,
                approver,
                decision,
            } => {
                let proposal = self
                    .proposals
                    .get_mut(proposal_id)
                    .ok_or(WorldError::ProposalNotFound {
                        proposal_id: *proposal_id,
                    })?;
                match decision {
                    ProposalDecision::Approve => {
                        let ProposalStatus::Shadowed { manifest_hash } = &proposal.status else {
                            return Err(WorldError::ProposalInvalidState {
                                proposal_id: *proposal_id,
                                expected: "shadowed".to_string(),
                                found: proposal.status.label(),
                            });
                        };
                        proposal.status = ProposalStatus::Approved {
                            manifest_hash: manifest_hash.clone(),
                            approver: approver.clone(),
                        };
                    }
                    ProposalDecision::Reject { reason } => {
                        proposal.status = ProposalStatus::Rejected {
                            reason: reason.clone(),
                        };
                    }
                }
            }
            GovernanceEvent::Applied {
                proposal_id,
                manifest_hash,
            } => {
                let proposal = self
                    .proposals
                    .get_mut(proposal_id)
                    .ok_or(WorldError::ProposalNotFound {
                        proposal_id: *proposal_id,
                    })?;
                let ProposalStatus::Approved {
                    manifest_hash: approved_hash,
                    ..
                } = &proposal.status
                else {
                    return Err(WorldError::ProposalInvalidState {
                        proposal_id: *proposal_id,
                        expected: "approved".to_string(),
                        found: proposal.status.label(),
                    });
                };
                let applied_hash = manifest_hash
                    .clone()
                    .unwrap_or_else(|| approved_hash.clone());
                proposal.status = ProposalStatus::Applied {
                    manifest_hash: applied_hash,
                };
            }
        }
        Ok(())
    }

    fn validate_module_changes(&self, changes: &ModuleChangeSet) -> Result<(), WorldError> {
        let mut register_ids = BTreeSet::new();
        let mut activate_ids = BTreeSet::new();
        let mut deactivate_ids = BTreeSet::new();
        let mut upgrade_ids = BTreeSet::new();

        for module in &changes.register {
            if !register_ids.insert(module.module_id.clone()) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("duplicate register module_id {}", module.module_id),
                });
            }
        }

        for activation in &changes.activate {
            if !activate_ids.insert(activation.module_id.clone()) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("duplicate activate module_id {}", activation.module_id),
                });
            }
        }

        for deactivation in &changes.deactivate {
            if !deactivate_ids.insert(deactivation.module_id.clone()) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("duplicate deactivate module_id {}", deactivation.module_id),
                });
            }
        }

        for upgrade in &changes.upgrade {
            if !upgrade_ids.insert(upgrade.module_id.clone()) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("duplicate upgrade module_id {}", upgrade.module_id),
                });
            }
            if upgrade.manifest.module_id != upgrade.module_id {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!(
                        "upgrade manifest module_id mismatch {}",
                        upgrade.module_id
                    ),
                });
            }
            if upgrade.manifest.version != upgrade.to_version {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!(
                        "upgrade manifest version mismatch {}",
                        upgrade.module_id
                    ),
                });
            }
            if upgrade.manifest.wasm_hash != upgrade.wasm_hash {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!(
                        "upgrade manifest wasm_hash mismatch {}",
                        upgrade.module_id
                    ),
                });
            }
        }

        for module_id in register_ids.iter() {
            if upgrade_ids.contains(module_id) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("register and upgrade both target {module_id}"),
                });
            }
        }

        let mut planned_records = BTreeSet::new();
        for module in &changes.register {
            let key = ModuleRegistry::record_key(&module.module_id, &module.version);
            if !planned_records.insert(key.clone()) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("duplicate planned record {key}"),
                });
            }
        }
        for upgrade in &changes.upgrade {
            let key = ModuleRegistry::record_key(&upgrade.module_id, &upgrade.to_version);
            if !planned_records.insert(key.clone()) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("duplicate planned record {key}"),
                });
            }
        }

        for module in &changes.register {
            let key = ModuleRegistry::record_key(&module.module_id, &module.version);
            if self.module_registry.records.contains_key(&key) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("module already registered {key}"),
                });
            }
        }

        for upgrade in &changes.upgrade {
            let to_key = ModuleRegistry::record_key(&upgrade.module_id, &upgrade.to_version);
            if self.module_registry.records.contains_key(&to_key) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("module version already registered {to_key}"),
                });
            }

            let from_key = ModuleRegistry::record_key(&upgrade.module_id, &upgrade.from_version);
            if !self.module_registry.records.contains_key(&from_key) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("upgrade source missing {from_key}"),
                });
            }

            if let Some(active_version) = self.module_registry.active.get(&upgrade.module_id) {
                if active_version != &upgrade.from_version {
                    return Err(WorldError::ModuleChangeInvalid {
                        reason: format!(
                            "upgrade source version mismatch for {} (active {})",
                            upgrade.module_id, active_version
                        ),
                    });
                }
            }
        }

        for activation in &changes.activate {
            let key = ModuleRegistry::record_key(&activation.module_id, &activation.version);
            let exists = self.module_registry.records.contains_key(&key)
                || planned_records.contains(&key);
            if !exists {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("activate target missing {key}"),
                });
            }
        }

        let mut will_activate = BTreeSet::new();
        for activation in &changes.activate {
            will_activate.insert(activation.module_id.clone());
        }
        for deactivation in &changes.deactivate {
            let has_active = self
                .module_registry
                .active
                .contains_key(&deactivation.module_id);
            if !has_active && !will_activate.contains(&deactivation.module_id) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!(
                        "deactivate target not active {}",
                        deactivation.module_id
                    ),
                });
            }
        }

        Ok(())
    }

    fn shadow_validate_module_changes(&self, changes: &ModuleChangeSet) -> Result<(), WorldError> {
        for module in &changes.register {
            self.validate_module_manifest(module)?;
        }
        for upgrade in &changes.upgrade {
            self.validate_module_manifest(&upgrade.manifest)?;
        }
        Ok(())
    }

    fn validate_module_manifest(&self, module: &ModuleManifest) -> Result<(), WorldError> {
        if module.module_id.trim().is_empty() {
            return Err(WorldError::ModuleChangeInvalid {
                reason: "module_id is empty".to_string(),
            });
        }
        if module.version.trim().is_empty() {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module version missing for {}", module.module_id),
            });
        }
        if module.wasm_hash.trim().is_empty() {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module wasm_hash missing for {}", module.module_id),
            });
        }
        if module.interface_version.trim().is_empty() {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module interface_version missing for {}", module.module_id),
            });
        }
        if !self.module_artifacts.contains(&module.wasm_hash) {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module artifact missing {}", module.wasm_hash),
            });
        }

        if module.interface_version != "wasm-1" {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!(
                    "module interface_version unsupported {}",
                    module.interface_version
                ),
            });
        }

        let expected_export = match module.kind {
            super::modules::ModuleKind::Reducer => "reduce",
            super::modules::ModuleKind::Pure => "call",
        };
        if !module.exports.iter().any(|name| name == expected_export) {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!(
                    "module exports missing {} for {}",
                    expected_export, module.module_id
                ),
            });
        }

        self.validate_module_limits(&module.module_id, &module.limits)?;

        for cap in &module.required_caps {
            let grant = self.capabilities.get(cap).ok_or_else(|| {
                WorldError::ModuleChangeInvalid {
                    reason: format!("module cap missing {cap}"),
                }
            })?;
            if grant.is_expired(self.state.time) {
                return Err(WorldError::ModuleChangeInvalid {
                    reason: format!("module cap expired {cap}"),
                });
            }
        }

        Ok(())
    }

    fn validate_module_limits(
        &self,
        module_id: &str,
        limits: &ModuleLimits,
    ) -> Result<(), WorldError> {
        let max = &self.module_limits_max;
        if limits.max_mem_bytes > max.max_mem_bytes {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module limits max_mem_bytes exceeded {module_id}"),
            });
        }
        if limits.max_gas > max.max_gas {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module limits max_gas exceeded {module_id}"),
            });
        }
        if limits.max_call_rate > max.max_call_rate {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module limits max_call_rate exceeded {module_id}"),
            });
        }
        if limits.max_output_bytes > max.max_output_bytes {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module limits max_output_bytes exceeded {module_id}"),
            });
        }
        if limits.max_effects > max.max_effects {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module limits max_effects exceeded {module_id}"),
            });
        }
        if limits.max_emits > max.max_emits {
            return Err(WorldError::ModuleChangeInvalid {
                reason: format!("module limits max_emits exceeded {module_id}"),
            });
        }
        Ok(())
    }

    fn active_module_manifest(&self, module_id: &str) -> Result<&ModuleManifest, WorldError> {
        let version = self
            .module_registry
            .active
            .get(module_id)
            .ok_or_else(|| WorldError::ModuleChangeInvalid {
                reason: format!("module not active {module_id}"),
            })?;
        let key = ModuleRegistry::record_key(module_id, version);
        let record = self
            .module_registry
            .records
            .get(&key)
            .ok_or_else(|| WorldError::ModuleChangeInvalid {
                reason: format!("module record missing {key}"),
            })?;
        Ok(&record.manifest)
    }

    fn module_call_failed(&mut self, failure: ModuleCallFailure) -> Result<(), WorldError> {
        self.append_event(WorldEventBody::ModuleCallFailed(failure.clone()), None)?;
        Err(WorldError::ModuleCallFailed {
            module_id: failure.module_id,
            trace_id: failure.trace_id,
            code: failure.code,
            detail: failure.detail,
        })
    }

    fn apply_module_changes(
        &mut self,
        proposal_id: ProposalId,
        changes: &ModuleChangeSet,
        actor: &str,
    ) -> Result<(), WorldError> {
        let mut registers = changes.register.clone();
        registers.sort_by(|left, right| left.module_id.cmp(&right.module_id));
        for module in registers {
            let event = ModuleEvent {
                proposal_id,
                kind: ModuleEventKind::RegisterModule {
                    module,
                    registered_by: actor.to_string(),
                },
            };
            self.append_event(WorldEventBody::ModuleEvent(event), None)?;
        }

        let mut upgrades = changes.upgrade.clone();
        upgrades.sort_by(|left, right| left.module_id.cmp(&right.module_id));
        for upgrade in upgrades {
            let event = ModuleEvent {
                proposal_id,
                kind: ModuleEventKind::UpgradeModule {
                    module_id: upgrade.module_id,
                    from_version: upgrade.from_version,
                    to_version: upgrade.to_version,
                    wasm_hash: upgrade.wasm_hash,
                    manifest: upgrade.manifest,
                    upgraded_by: actor.to_string(),
                },
            };
            self.append_event(WorldEventBody::ModuleEvent(event), None)?;
        }

        let mut activations = changes.activate.clone();
        activations.sort_by(|left, right| left.module_id.cmp(&right.module_id));
        for activation in activations {
            let event = ModuleEvent {
                proposal_id,
                kind: ModuleEventKind::ActivateModule {
                    module_id: activation.module_id,
                    version: activation.version,
                    activated_by: actor.to_string(),
                },
            };
            self.append_event(WorldEventBody::ModuleEvent(event), None)?;
        }

        let mut deactivations = changes.deactivate.clone();
        deactivations.sort_by(|left, right| left.module_id.cmp(&right.module_id));
        for deactivation in deactivations {
            let event = ModuleEvent {
                proposal_id,
                kind: ModuleEventKind::DeactivateModule {
                    module_id: deactivation.module_id,
                    reason: deactivation.reason,
                    deactivated_by: actor.to_string(),
                },
            };
            self.append_event(WorldEventBody::ModuleEvent(event), None)?;
        }

        Ok(())
    }

    fn apply_module_event(
        &mut self,
        event: &ModuleEvent,
        time: super::types::WorldTime,
    ) -> Result<(), WorldError> {
        match &event.kind {
            ModuleEventKind::RegisterModule {
                module,
                registered_by,
            } => {
                let key = ModuleRegistry::record_key(&module.module_id, &module.version);
                if self.module_registry.records.contains_key(&key) {
                    return Err(WorldError::ModuleChangeInvalid {
                        reason: format!("module already registered {key}"),
                    });
                }
                let record = ModuleRecord {
                    manifest: module.clone(),
                    registered_at: time,
                    registered_by: registered_by.clone(),
                    audit_event_id: None,
                };
                self.module_registry.records.insert(key, record);
            }
            ModuleEventKind::UpgradeModule {
                module_id,
                from_version,
                to_version,
                wasm_hash,
                manifest,
                upgraded_by,
                ..
            } => {
                if manifest.module_id != *module_id || manifest.version != *to_version {
                    return Err(WorldError::ModuleChangeInvalid {
                        reason: format!("upgrade manifest mismatch for {module_id}"),
                    });
                }
                if manifest.wasm_hash != *wasm_hash {
                    return Err(WorldError::ModuleChangeInvalid {
                        reason: format!("upgrade wasm_hash mismatch for {module_id}"),
                    });
                }

                let to_key = ModuleRegistry::record_key(module_id, to_version);
                if self.module_registry.records.contains_key(&to_key) {
                    return Err(WorldError::ModuleChangeInvalid {
                        reason: format!("module already registered {to_key}"),
                    });
                }
                let record = ModuleRecord {
                    manifest: manifest.clone(),
                    registered_at: time,
                    registered_by: upgraded_by.clone(),
                    audit_event_id: None,
                };
                self.module_registry.records.insert(to_key, record);

                if let Some(active_version) = self.module_registry.active.get(module_id) {
                    if active_version == from_version {
                        self.module_registry
                            .active
                            .insert(module_id.clone(), to_version.clone());
                    }
                }
            }
            ModuleEventKind::ActivateModule {
                module_id, version, ..
            } => {
                let key = ModuleRegistry::record_key(module_id, version);
                if !self.module_registry.records.contains_key(&key) {
                    return Err(WorldError::ModuleChangeInvalid {
                        reason: format!("activate target missing {key}"),
                    });
                }
                self.module_registry
                    .active
                    .insert(module_id.clone(), version.clone());
            }
            ModuleEventKind::DeactivateModule { module_id, .. } => {
                self.module_registry.active.remove(module_id);
            }
        }
        Ok(())
    }

    fn process_module_output(
        &mut self,
        module_id: &str,
        trace_id: &str,
        manifest: &ModuleManifest,
        output: &ModuleOutput,
    ) -> Result<(), WorldError> {
        if output.effects.len() as u32 > manifest.limits.max_effects {
            return self.module_call_failed(ModuleCallFailure {
                module_id: module_id.to_string(),
                trace_id: trace_id.to_string(),
                code: ModuleCallErrorCode::EffectLimitExceeded,
                detail: "effects exceeded".to_string(),
            });
        }
        if output.emits.len() as u32 > manifest.limits.max_emits {
            return self.module_call_failed(ModuleCallFailure {
                module_id: module_id.to_string(),
                trace_id: trace_id.to_string(),
                code: ModuleCallErrorCode::EmitLimitExceeded,
                detail: "emits exceeded".to_string(),
            });
        }
        if output.output_bytes > manifest.limits.max_output_bytes {
            return self.module_call_failed(ModuleCallFailure {
                module_id: module_id.to_string(),
                trace_id: trace_id.to_string(),
                code: ModuleCallErrorCode::OutputTooLarge,
                detail: "output bytes exceeded".to_string(),
            });
        }

        for effect in &output.effects {
            if !manifest.required_caps.iter().any(|cap| cap == &effect.cap_ref) {
                return self.module_call_failed(ModuleCallFailure {
                    module_id: module_id.to_string(),
                    trace_id: trace_id.to_string(),
                    code: ModuleCallErrorCode::CapsDenied,
                    detail: format!("cap_ref not allowed {}", effect.cap_ref),
                });
            }
        }

        let mut intents = Vec::new();
        for effect in &output.effects {
            let intent = match self.build_effect_intent(
                effect.kind.clone(),
                effect.params.clone(),
                effect.cap_ref.clone(),
                EffectOrigin::Module {
                    module_id: module_id.to_string(),
                },
            ) {
                Ok(intent) => intent,
                Err(err) => {
                    let (code, detail) = match err {
                        WorldError::CapabilityMissing { cap_ref } => {
                            (ModuleCallErrorCode::CapsDenied, format!("cap missing {cap_ref}"))
                        }
                        WorldError::CapabilityExpired { cap_ref } => (
                            ModuleCallErrorCode::CapsDenied,
                            format!("cap expired {cap_ref}"),
                        ),
                        WorldError::CapabilityNotAllowed { cap_ref, kind } => (
                            ModuleCallErrorCode::CapsDenied,
                            format!("cap not allowed {cap_ref} {kind}"),
                        ),
                        WorldError::PolicyDenied { reason, .. } => {
                            (ModuleCallErrorCode::PolicyDenied, reason)
                        }
                        other => (ModuleCallErrorCode::InvalidOutput, format!("{other:?}")),
                    };
                    return self.module_call_failed(ModuleCallFailure {
                        module_id: module_id.to_string(),
                        trace_id: trace_id.to_string(),
                        code,
                        detail,
                    });
                }
            };
            intents.push(intent);
        }

        for intent in intents {
            self.append_event(WorldEventBody::EffectQueued(intent), None)?;
        }

        for emit in &output.emits {
            let event = ModuleEmitEvent {
                module_id: module_id.to_string(),
                trace_id: trace_id.to_string(),
                kind: emit.kind.clone(),
                payload: emit.payload.clone(),
            };
            self.append_event(WorldEventBody::ModuleEmitted(event), None)?;
        }

        Ok(())
    }

    fn finalize_receipt_signature(&self, receipt: &mut EffectReceipt) -> Result<(), WorldError> {
        let Some(signer) = &self.receipt_signer else {
            return Ok(());
        };

        if let Some(signature) = &receipt.signature {
            signer.verify(receipt, signature)?;
        } else {
            let signature = signer.sign(receipt)?;
            receipt.signature = Some(signature);
        }

        Ok(())
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

fn event_kind_label(body: &WorldEventBody) -> &'static str {
    match body {
        WorldEventBody::Domain(event) => match event {
            DomainEvent::AgentRegistered { .. } => "domain.agent_registered",
            DomainEvent::AgentMoved { .. } => "domain.agent_moved",
            DomainEvent::ActionRejected { .. } => "domain.action_rejected",
        },
        WorldEventBody::EffectQueued(_) => "effect.queued",
        WorldEventBody::ReceiptAppended(_) => "effect.receipt_appended",
        WorldEventBody::PolicyDecisionRecorded(_) => "policy.decision_recorded",
        WorldEventBody::Governance(_) => "governance",
        WorldEventBody::ModuleEvent(_) => "module.event",
        WorldEventBody::ModuleCallFailed(_) => "module.call_failed",
        WorldEventBody::ModuleEmitted(_) => "module.emitted",
        WorldEventBody::SnapshotCreated(_) => "snapshot.created",
        WorldEventBody::ManifestUpdated(_) => "manifest.updated",
        WorldEventBody::RollbackApplied(_) => "rollback.applied",
    }
}

fn action_kind_label(action: &Action) -> &'static str {
    match action {
        Action::RegisterAgent { .. } => "action.register_agent",
        Action::MoveAgent { .. } => "action.move_agent",
    }
}

fn module_subscribes_to_event(subscriptions: &[ModuleSubscription], event_kind: &str) -> bool {
    subscriptions.iter().any(|subscription| {
        subscription
            .event_kinds
            .iter()
            .any(|pattern| subscription_match(pattern, event_kind))
    })
}

fn module_subscribes_to_action(subscriptions: &[ModuleSubscription], action_kind: &str) -> bool {
    subscriptions.iter().any(|subscription| {
        subscription
            .action_kinds
            .iter()
            .any(|pattern| subscription_match(pattern, action_kind))
    })
}

fn subscription_match(pattern: &str, event_kind: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix(".*") {
        return event_kind.starts_with(prefix);
    }
    pattern == event_kind
}
