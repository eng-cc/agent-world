use crate::geometry::GeoPos;
use crate::models::AgentState;
use hmac::{Hmac, Mac};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::io;
use std::path::Path;

type HmacSha256 = Hmac<Sha256>;

pub type WorldTime = u64;
pub type WorldEventId = u64;
pub type ActionId = u64;
pub type IntentSeq = u64;
pub type ProposalId = u64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentCell {
    pub state: AgentState,
    pub mailbox: VecDeque<DomainEvent>,
    pub last_active: WorldTime,
}

impl AgentCell {
    fn new(state: AgentState, now: WorldTime) -> Self {
        Self {
            state,
            mailbox: VecDeque::new(),
            last_active: now,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub version: u64,
    pub content: JsonValue,
}

impl Default for Manifest {
    fn default() -> Self {
        Self {
            version: 1,
            content: JsonValue::Object(serde_json::Map::new()),
        }
    }
}

pub type PatchPath = Vec<String>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManifestPatch {
    pub base_manifest_hash: String,
    pub ops: Vec<ManifestPatchOp>,
    pub new_version: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", content = "data")]
pub enum ManifestPatchOp {
    Set { path: PatchPath, value: JsonValue },
    Remove { path: PatchPath },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotRetentionPolicy {
    pub max_snapshots: usize,
}

impl Default for SnapshotRetentionPolicy {
    fn default() -> Self {
        Self { max_snapshots: 10 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotRecord {
    pub snapshot_hash: String,
    pub journal_len: usize,
    pub created_at: WorldTime,
    pub manifest_hash: String,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventKind {
    Domain,
    EffectQueued,
    ReceiptAppended,
    PolicyDecision,
    Governance,
    SnapshotCreated,
    ManifestUpdated,
    RollbackApplied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditCausedBy {
    Action,
    Effect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AuditFilter {
    pub kinds: Option<Vec<AuditEventKind>>,
    pub from_time: Option<WorldTime>,
    pub to_time: Option<WorldTime>,
    pub from_event_id: Option<WorldEventId>,
    pub to_event_id: Option<WorldEventId>,
    pub caused_by: Option<AuditCausedBy>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchConflict {
    pub path: PatchPath,
    pub kind: ConflictKind,
    pub patches: Vec<usize>,
    pub ops: Vec<PatchOpSummary>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchMergeResult {
    pub patch: ManifestPatch,
    pub conflicts: Vec<PatchConflict>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictKind {
    SamePath,
    PrefixOverlap,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatchOpKind {
    Set,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchOpSummary {
    pub patch_index: usize,
    pub kind: PatchOpKind,
    pub path: PatchPath,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldState {
    pub time: WorldTime,
    pub agents: BTreeMap<String, AgentCell>,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            time: 0,
            agents: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub snapshot_catalog: SnapshotCatalog,
    pub manifest: Manifest,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    manifest: Manifest,
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

    pub fn state(&self) -> &WorldState {
        &self.state
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
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

    pub fn set_policy(&mut self, policy: PolicySet) {
        self.policies = policy;
    }

    pub fn add_capability(&mut self, grant: CapabilityGrant) {
        self.capabilities.insert(grant.name.clone(), grant);
    }

    pub fn set_receipt_signer(&mut self, signer: ReceiptSigner) {
        self.receipt_signer = Some(signer);
    }

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
        let (manifest, manifest_hash) = match &proposal.status {
            ProposalStatus::Approved { manifest_hash, .. } => {
                (proposal.manifest.clone(), manifest_hash.clone())
            }
            other => {
                return Err(WorldError::ProposalInvalidState {
                    proposal_id,
                    expected: "approved".to_string(),
                    found: other.label(),
                })
            }
        };
        let event = GovernanceEvent::Applied { proposal_id };
        self.append_event(WorldEventBody::Governance(event), None)?;
        let update = ManifestUpdate {
            manifest,
            manifest_hash: manifest_hash.clone(),
        };
        self.append_event(WorldEventBody::ManifestUpdated(update), None)?;
        Ok(manifest_hash)
    }

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

        self.append_event(WorldEventBody::EffectQueued(intent), None)?;
        Ok(intent_id)
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

    pub fn step(&mut self) -> Result<(), WorldError> {
        self.state.time = self.state.time.saturating_add(1);
        while let Some(envelope) = self.pending_actions.pop_front() {
            let event_body = self.action_to_event(&envelope)?;
            self.append_event(event_body, Some(CausedBy::Action(envelope.id)))?;
        }
        Ok(())
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            snapshot_catalog: self.snapshot_catalog.clone(),
            manifest: self.manifest.clone(),
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

    pub fn load_from_dir(dir: impl AsRef<Path>) -> Result<Self, WorldError> {
        let dir = dir.as_ref();
        let journal_path = dir.join("journal.json");
        let snapshot_path = dir.join("snapshot.json");
        let journal = Journal::load_json(journal_path)?;
        let snapshot = Snapshot::load_json(snapshot_path)?;
        Self::from_snapshot(snapshot, journal)
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

    fn apply_event_body(&mut self, body: &WorldEventBody, time: WorldTime) -> Result<(), WorldError> {
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
            GovernanceEvent::Applied { proposal_id } => {
                let proposal = self
                    .proposals
                    .get_mut(proposal_id)
                    .ok_or(WorldError::ProposalNotFound {
                        proposal_id: *proposal_id,
                    })?;
                let ProposalStatus::Approved { manifest_hash, .. } = &proposal.status else {
                    return Err(WorldError::ProposalInvalidState {
                        proposal_id: *proposal_id,
                        expected: "approved".to_string(),
                        found: proposal.status.label(),
                    });
                };
                proposal.status = ProposalStatus::Applied {
                    manifest_hash: manifest_hash.clone(),
                };
            }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionEnvelope {
    pub id: ActionId,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Action {
    RegisterAgent { agent_id: String, pos: GeoPos },
    MoveAgent { agent_id: String, to: GeoPos },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldEvent {
    pub id: WorldEventId,
    pub time: WorldTime,
    pub caused_by: Option<CausedBy>,
    pub body: WorldEventBody,
}

impl WorldEvent {
    fn audit_kind(&self) -> AuditEventKind {
        match self.body {
            WorldEventBody::Domain(_) => AuditEventKind::Domain,
            WorldEventBody::EffectQueued(_) => AuditEventKind::EffectQueued,
            WorldEventBody::ReceiptAppended(_) => AuditEventKind::ReceiptAppended,
            WorldEventBody::PolicyDecisionRecorded(_) => AuditEventKind::PolicyDecision,
            WorldEventBody::Governance(_) => AuditEventKind::Governance,
            WorldEventBody::SnapshotCreated(_) => AuditEventKind::SnapshotCreated,
            WorldEventBody::ManifestUpdated(_) => AuditEventKind::ManifestUpdated,
            WorldEventBody::RollbackApplied(_) => AuditEventKind::RollbackApplied,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum WorldEventBody {
    Domain(DomainEvent),
    EffectQueued(EffectIntent),
    ReceiptAppended(EffectReceipt),
    PolicyDecisionRecorded(PolicyDecisionRecord),
    Governance(GovernanceEvent),
    SnapshotCreated(SnapshotMeta),
    ManifestUpdated(ManifestUpdate),
    RollbackApplied(RollbackEvent),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DomainEvent {
    AgentRegistered { agent_id: String, pos: GeoPos },
    AgentMoved { agent_id: String, from: GeoPos, to: GeoPos },
    ActionRejected { action_id: ActionId, reason: RejectReason },
}

impl DomainEvent {
    fn agent_id(&self) -> Option<&str> {
        match self {
            DomainEvent::AgentRegistered { agent_id, .. } => Some(agent_id.as_str()),
            DomainEvent::AgentMoved { agent_id, .. } => Some(agent_id.as_str()),
            DomainEvent::ActionRejected { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum RejectReason {
    AgentAlreadyExists { agent_id: String },
    AgentNotFound { agent_id: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum CausedBy {
    Action(ActionId),
    Effect { intent_id: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectIntent {
    pub intent_id: String,
    pub kind: String,
    pub params: JsonValue,
    pub cap_ref: String,
    pub origin: EffectOrigin,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EffectReceipt {
    pub intent_id: String,
    pub status: String,
    pub payload: JsonValue,
    pub cost_cents: Option<u64>,
    pub signature: Option<ReceiptSignature>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReceiptSignature {
    pub algorithm: SignatureAlgorithm,
    pub signature_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignatureAlgorithm {
    HmacSha256,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum EffectOrigin {
    Reducer { name: String },
    Plan { name: String },
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OriginKind {
    Reducer,
    Plan,
    System,
}

impl OriginKind {
    fn from_origin(origin: &EffectOrigin) -> Self {
        match origin {
            EffectOrigin::Reducer { .. } => OriginKind::Reducer,
            EffectOrigin::Plan { .. } => OriginKind::Plan,
            EffectOrigin::System => OriginKind::System,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapabilityGrant {
    pub name: String,
    pub effect_kinds: Vec<String>,
    pub expiry: Option<WorldTime>,
}

impl CapabilityGrant {
    pub fn new(name: impl Into<String>, effect_kinds: Vec<String>) -> Self {
        Self {
            name: name.into(),
            effect_kinds,
            expiry: None,
        }
    }

    pub fn allow_all(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            effect_kinds: vec!["*".to_string()],
            expiry: None,
        }
    }

    pub fn allows(&self, kind: &str) -> bool {
        self.effect_kinds.iter().any(|allowed| {
            allowed == "*"
                || allowed == kind
                || (allowed.ends_with(".*")
                    && kind.starts_with(&allowed[..allowed.len() - 1]))
        })
    }

    pub fn is_expired(&self, now: WorldTime) -> bool {
        match self.expiry {
            Some(expiry) => now > expiry,
            None => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicySet {
    pub rules: Vec<PolicyRule>,
}

impl PolicySet {
    pub fn decide(&self, intent: &EffectIntent) -> PolicyDecision {
        for rule in &self.rules {
            if rule.when.matches(intent) {
                return rule.decision.clone();
            }
        }
        PolicyDecision::Deny {
            reason: "default_deny".to_string(),
        }
    }

    pub fn allow_all() -> Self {
        Self {
            rules: vec![PolicyRule {
                when: PolicyWhen {
                    effect_kind: None,
                    origin_kind: None,
                    cap_name: None,
                },
                decision: PolicyDecision::Allow,
            }],
        }
    }
}

impl Default for PolicySet {
    fn default() -> Self {
        Self { rules: Vec::new() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyRule {
    pub when: PolicyWhen,
    pub decision: PolicyDecision,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyWhen {
    pub effect_kind: Option<String>,
    pub origin_kind: Option<OriginKind>,
    pub cap_name: Option<String>,
}

impl PolicyWhen {
    fn matches(&self, intent: &EffectIntent) -> bool {
        if let Some(effect_kind) = &self.effect_kind {
            if effect_kind != &intent.kind {
                return false;
            }
        }
        if let Some(origin_kind) = &self.origin_kind {
            if origin_kind != &OriginKind::from_origin(&intent.origin) {
                return false;
            }
        }
        if let Some(cap_name) = &self.cap_name {
            if cap_name != &intent.cap_ref {
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "decision", content = "data")]
pub enum PolicyDecision {
    Allow,
    Deny { reason: String },
}

impl PolicyDecision {
    fn is_allowed(&self) -> bool {
        matches!(self, PolicyDecision::Allow)
    }

    fn reason(&self) -> Option<String> {
        match self {
            PolicyDecision::Allow => None,
            PolicyDecision::Deny { reason } => Some(reason.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyDecisionRecord {
    pub intent_id: String,
    pub decision: PolicyDecision,
    pub effect_kind: String,
    pub cap_ref: String,
    pub origin_kind: OriginKind,
}

impl PolicyDecisionRecord {
    fn from_intent(intent: &EffectIntent, decision: PolicyDecision) -> Self {
        Self {
            intent_id: intent.intent_id.clone(),
            decision,
            effect_kind: intent.kind.clone(),
            cap_ref: intent.cap_ref.clone(),
            origin_kind: OriginKind::from_origin(&intent.origin),
        }
    }
}

impl AuditFilter {
    fn matches(&self, event: &WorldEvent) -> bool {
        if let Some(kinds) = &self.kinds {
            if !kinds.contains(&event.audit_kind()) {
                return false;
            }
        }
        if let Some(from_time) = self.from_time {
            if event.time < from_time {
                return false;
            }
        }
        if let Some(to_time) = self.to_time {
            if event.time > to_time {
                return false;
            }
        }
        if let Some(from_event_id) = self.from_event_id {
            if event.id < from_event_id {
                return false;
            }
        }
        if let Some(to_event_id) = self.to_event_id {
            if event.id > to_event_id {
                return false;
            }
        }
        if let Some(cause) = &self.caused_by {
            match cause {
                AuditCausedBy::Action => {
                    if !matches!(event.caused_by, Some(CausedBy::Action(_))) {
                        return false;
                    }
                }
                AuditCausedBy::Effect => {
                    if !matches!(event.caused_by, Some(CausedBy::Effect { .. })) {
                        return false;
                    }
                }
            }
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Proposal {
    pub id: ProposalId,
    pub author: String,
    pub base_manifest_hash: String,
    pub manifest: Manifest,
    pub patch: Option<ManifestPatch>,
    pub status: ProposalStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", content = "data")]
pub enum ProposalStatus {
    Proposed,
    Shadowed { manifest_hash: String },
    Approved { manifest_hash: String, approver: String },
    Rejected { reason: String },
    Applied { manifest_hash: String },
}

impl ProposalStatus {
    fn label(&self) -> String {
        match self {
            ProposalStatus::Proposed => "proposed".to_string(),
            ProposalStatus::Shadowed { .. } => "shadowed".to_string(),
            ProposalStatus::Approved { .. } => "approved".to_string(),
            ProposalStatus::Rejected { .. } => "rejected".to_string(),
            ProposalStatus::Applied { .. } => "applied".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "decision", content = "data")]
pub enum ProposalDecision {
    Approve,
    Reject { reason: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum GovernanceEvent {
    Proposed {
        proposal_id: ProposalId,
        author: String,
        base_manifest_hash: String,
        manifest: Manifest,
        patch: Option<ManifestPatch>,
    },
    ShadowReport {
        proposal_id: ProposalId,
        manifest_hash: String,
    },
    Approved {
        proposal_id: ProposalId,
        approver: String,
        decision: ProposalDecision,
    },
    Applied {
        proposal_id: ProposalId,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentSchedule {
    pub agent_id: String,
    pub event: DomainEvent,
}

#[derive(Debug, Clone)]
pub struct ReceiptSigner {
    algorithm: SignatureAlgorithm,
    key: Vec<u8>,
}

impl ReceiptSigner {
    pub fn hmac_sha256(key: impl Into<Vec<u8>>) -> Self {
        Self {
            algorithm: SignatureAlgorithm::HmacSha256,
            key: key.into(),
        }
    }

    pub fn sign(&self, receipt: &EffectReceipt) -> Result<ReceiptSignature, WorldError> {
        match self.algorithm {
            SignatureAlgorithm::HmacSha256 => {
                let bytes = receipt_signing_bytes(receipt)?;
                let mut mac = HmacSha256::new_from_slice(&self.key)
                    .map_err(|_| WorldError::SignatureKeyInvalid)?;
                mac.update(&bytes);
                let signature = mac.finalize().into_bytes();
                Ok(ReceiptSignature {
                    algorithm: SignatureAlgorithm::HmacSha256,
                    signature_hex: hex::encode(signature),
                })
            }
        }
    }

    pub fn verify(
        &self,
        receipt: &EffectReceipt,
        signature: &ReceiptSignature,
    ) -> Result<(), WorldError> {
        if signature.algorithm != self.algorithm {
            return Err(WorldError::ReceiptSignatureInvalid {
                intent_id: receipt.intent_id.clone(),
            });
        }
        let expected = self.sign(receipt)?;
        if signature.signature_hex != expected.signature_hex {
            return Err(WorldError::ReceiptSignatureInvalid {
                intent_id: receipt.intent_id.clone(),
            });
        }
        Ok(())
    }
}

fn receipt_signing_bytes(receipt: &EffectReceipt) -> Result<Vec<u8>, WorldError> {
    #[derive(Serialize)]
    struct ReceiptPayload<'a> {
        intent_id: &'a str,
        status: &'a str,
        payload: &'a JsonValue,
        cost_cents: Option<u64>,
    }

    let payload = ReceiptPayload {
        intent_id: &receipt.intent_id,
        status: &receipt.status,
        payload: &receipt.payload,
        cost_cents: receipt.cost_cents,
    };

    Ok(serde_json::to_vec(&payload)?)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotMeta {
    pub journal_len: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManifestUpdate {
    pub manifest: Manifest,
    pub manifest_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RollbackEvent {
    pub snapshot_hash: String,
    pub snapshot_journal_len: usize,
    pub prior_journal_len: usize,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldError {
    JournalMismatch,
    CapabilityMissing { cap_ref: String },
    CapabilityExpired { cap_ref: String },
    CapabilityNotAllowed { cap_ref: String, kind: String },
    PolicyDenied { intent_id: String, reason: String },
    ReceiptUnknownIntent { intent_id: String },
    ReceiptSignatureInvalid { intent_id: String },
    ProposalNotFound { proposal_id: ProposalId },
    ProposalInvalidState { proposal_id: ProposalId, expected: String, found: String },
    PatchBaseMismatch { expected: String, found: String },
    PatchInvalidPath { path: String },
    PatchNonObject { path: String },
    SignatureKeyInvalid,
    Io(String),
    Serde(String),
}

impl From<serde_json::Error> for WorldError {
    fn from(error: serde_json::Error) -> Self {
        WorldError::Serde(error.to_string())
    }
}

impl From<io::Error> for WorldError {
    fn from(error: io::Error) -> Self {
        WorldError::Io(error.to_string())
    }
}

impl WorldState {
    fn apply_domain_event(&mut self, event: &DomainEvent, now: WorldTime) -> Result<(), WorldError> {
        match event {
            DomainEvent::AgentRegistered { agent_id, pos } => {
                let state = AgentState::new(agent_id, *pos);
                self.agents
                    .insert(agent_id.clone(), AgentCell::new(state, now));
            }
            DomainEvent::AgentMoved { agent_id, to, .. } => {
                if let Some(cell) = self.agents.get_mut(agent_id) {
                    cell.state.pos = *to;
                    cell.last_active = now;
                }
            }
            DomainEvent::ActionRejected { .. } => {}
        }
        Ok(())
    }

    fn route_domain_event(&mut self, event: &DomainEvent) {
        let Some(agent_id) = event.agent_id() else {
            return;
        };
        if let Some(cell) = self.agents.get_mut(agent_id) {
            cell.mailbox.push_back(event.clone());
        }
    }
}

fn hash_json<T: Serialize>(value: &T) -> Result<String, WorldError> {
    let bytes = serde_json::to_vec(value)?;
    Ok(sha256_hex(&bytes))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn apply_manifest_patch_internal(
    manifest: &Manifest,
    patch: &ManifestPatch,
) -> Result<Manifest, WorldError> {
    let current_hash = hash_json(manifest)?;
    if patch.base_manifest_hash != current_hash {
        return Err(WorldError::PatchBaseMismatch {
            expected: current_hash,
            found: patch.base_manifest_hash.clone(),
        });
    }
    apply_manifest_patch_ops(manifest, patch)
}

pub fn apply_manifest_patch(
    manifest: &Manifest,
    patch: &ManifestPatch,
) -> Result<Manifest, WorldError> {
    apply_manifest_patch_internal(manifest, patch)
}

pub fn diff_manifest(base: &Manifest, target: &Manifest) -> Result<ManifestPatch, WorldError> {
    let base_hash = hash_json(base)?;
    let mut ops = Vec::new();
    diff_json(&base.content, &target.content, &mut Vec::new(), &mut ops);
    let new_version = if base.version == target.version {
        None
    } else {
        Some(target.version)
    };
    Ok(ManifestPatch {
        base_manifest_hash: base_hash,
        ops,
        new_version,
    })
}

pub fn merge_manifest_patches(
    base: &Manifest,
    patches: &[ManifestPatch],
) -> Result<ManifestPatch, WorldError> {
    let base_hash = hash_json(base)?;
    let mut current = base.clone();
    for patch in patches {
        if patch.base_manifest_hash != base_hash {
            return Err(WorldError::PatchBaseMismatch {
                expected: base_hash.clone(),
                found: patch.base_manifest_hash.clone(),
            });
        }
        current = apply_manifest_patch_ops(&current, patch)?;
    }
    diff_manifest(base, &current)
}

pub fn merge_manifest_patches_with_conflicts(
    base: &Manifest,
    patches: &[ManifestPatch],
) -> Result<PatchMergeResult, WorldError> {
    let conflicts = detect_patch_conflicts(patches);
    let patch = merge_manifest_patches(base, patches)?;
    Ok(PatchMergeResult { patch, conflicts })
}

fn apply_manifest_patch_op(root: &mut JsonValue, op: &ManifestPatchOp) -> Result<(), WorldError> {
    match op {
        ManifestPatchOp::Set { path, value } => apply_patch_set(root, path, value.clone()),
        ManifestPatchOp::Remove { path } => apply_patch_remove(root, path),
    }
}

fn apply_patch_set(
    root: &mut JsonValue,
    path: &PatchPath,
    value: JsonValue,
) -> Result<(), WorldError> {
    if path.is_empty() {
        *root = value;
        return Ok(());
    }

    let mut current = root;
    for (idx, segment) in path.iter().enumerate() {
        let is_last = idx + 1 == path.len();
        let map = current.as_object_mut().ok_or_else(|| WorldError::PatchNonObject {
            path: path[..idx].join("."),
        })?;
        if is_last {
            map.insert(segment.clone(), value);
            return Ok(());
        }
        current = map
            .entry(segment.clone())
            .or_insert_with(|| JsonValue::Object(serde_json::Map::new()));
    }
    Ok(())
}

fn apply_manifest_patch_ops(
    manifest: &Manifest,
    patch: &ManifestPatch,
) -> Result<Manifest, WorldError> {
    let mut content = manifest.content.clone();
    for op in &patch.ops {
        apply_manifest_patch_op(&mut content, op)?;
    }
    let version = patch.new_version.unwrap_or(manifest.version);
    Ok(Manifest { version, content })
}

fn apply_patch_remove(root: &mut JsonValue, path: &PatchPath) -> Result<(), WorldError> {
    if path.is_empty() {
        return Err(WorldError::PatchInvalidPath {
            path: "".to_string(),
        });
    }

    let mut current = root;
    for (idx, segment) in path.iter().enumerate() {
        let is_last = idx + 1 == path.len();
        let map = current.as_object_mut().ok_or_else(|| WorldError::PatchNonObject {
            path: path[..idx].join("."),
        })?;
        if is_last {
            if map.remove(segment).is_none() {
                return Err(WorldError::PatchInvalidPath {
                    path: path.join("."),
                });
            }
            return Ok(());
        }
        current = map.get_mut(segment).ok_or_else(|| WorldError::PatchInvalidPath {
            path: path[..=idx].join("."),
        })?;
    }
    Ok(())
}

fn diff_json(
    base: &JsonValue,
    target: &JsonValue,
    path: &mut Vec<String>,
    ops: &mut Vec<ManifestPatchOp>,
) {
    if base == target {
        return;
    }

    match (base, target) {
        (JsonValue::Object(base_map), JsonValue::Object(target_map)) => {
            let mut keys: Vec<String> = base_map
                .keys()
                .chain(target_map.keys())
                .cloned()
                .collect();
            keys.sort();
            keys.dedup();

            for key in keys {
                path.push(key.clone());
                match (base_map.get(&key), target_map.get(&key)) {
                    (Some(base_val), Some(target_val)) => {
                        diff_json(base_val, target_val, path, ops);
                    }
                    (None, Some(target_val)) => {
                        ops.push(ManifestPatchOp::Set {
                            path: path.clone(),
                            value: target_val.clone(),
                        });
                    }
                    (Some(_), None) => {
                        ops.push(ManifestPatchOp::Remove { path: path.clone() });
                    }
                    (None, None) => {}
                }
                path.pop();
            }
        }
        _ => {
            ops.push(ManifestPatchOp::Set {
                path: path.clone(),
                value: target.clone(),
            });
        }
    }
}

fn detect_patch_conflicts(patches: &[ManifestPatch]) -> Vec<PatchConflict> {
    let mut entries: Vec<(PatchPath, PatchOpSummary)> = Vec::new();
    for (idx, patch) in patches.iter().enumerate() {
        for op in &patch.ops {
            let (path, kind) = match op {
                ManifestPatchOp::Set { path, .. } => (path.clone(), PatchOpKind::Set),
                ManifestPatchOp::Remove { path } => (path.clone(), PatchOpKind::Remove),
            };
            entries.push((
                path.clone(),
                PatchOpSummary {
                    patch_index: idx,
                    kind,
                    path,
                },
            ));
        }
    }

    let mut conflicts: BTreeMap<String, PatchConflict> = BTreeMap::new();
    for i in 0..entries.len() {
        for j in (i + 1)..entries.len() {
            let (path_a, summary_a) = &entries[i];
            let (path_b, summary_b) = &entries[j];
            if path_is_prefix(path_a, path_b) || path_is_prefix(path_b, path_a) {
                let conflict_path = if path_a.len() <= path_b.len() {
                    path_a.clone()
                } else {
                    path_b.clone()
                };
                let key = conflict_path.join(".");
                let kind = if path_a == path_b {
                    ConflictKind::SamePath
                } else {
                    ConflictKind::PrefixOverlap
                };
                let entry = conflicts.entry(key.clone()).or_insert(PatchConflict {
                    path: conflict_path,
                    kind: kind.clone(),
                    patches: Vec::new(),
                    ops: Vec::new(),
                });
                if kind == ConflictKind::SamePath {
                    entry.kind = ConflictKind::SamePath;
                }
                insert_patch_index(&mut entry.patches, summary_a.patch_index);
                insert_patch_index(&mut entry.patches, summary_b.patch_index);
                insert_op_summary(&mut entry.ops, summary_a.clone());
                insert_op_summary(&mut entry.ops, summary_b.clone());
            }
        }
    }

    let mut results: Vec<PatchConflict> = conflicts.into_values().collect();
    for conflict in &mut results {
        conflict.patches.sort();
        conflict.ops.sort_by(|left, right| {
            left.patch_index
                .cmp(&right.patch_index)
                .then_with(|| left.path.cmp(&right.path))
        });
    }
    results
}

fn path_is_prefix(a: &PatchPath, b: &PatchPath) -> bool {
    if a.len() > b.len() {
        return false;
    }
    a.iter().zip(b.iter()).all(|(left, right)| left == right)
}

fn insert_patch_index(target: &mut Vec<usize>, index: usize) {
    if !target.contains(&index) {
        target.push(index);
    }
}

fn insert_op_summary(target: &mut Vec<PatchOpSummary>, summary: PatchOpSummary) {
    if !target.iter().any(|existing| {
        existing.patch_index == summary.patch_index
            && existing.kind == summary.kind
            && existing.path == summary.path
    }) {
        target.push(summary);
    }
}

fn write_json_to_path<T: Serialize>(value: &T, path: &Path) -> Result<(), WorldError> {
    let data = serde_json::to_vec_pretty(value)?;
    fs::write(path, data)?;
    Ok(())
}

fn read_json_from_path<T: DeserializeOwned>(path: &Path) -> Result<T, WorldError> {
    let data = fs::read(path)?;
    Ok(serde_json::from_slice(&data)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn pos(lat: f64, lon: f64) -> GeoPos {
        GeoPos {
            lat_deg: lat,
            lon_deg: lon,
        }
    }

    #[test]
    fn register_and_move_agent() {
        let mut world = World::new();
        world.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            pos: pos(0.0, 0.0),
        });
        world.step().unwrap();

        world.submit_action(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: pos(1.0, 1.0),
        });
        world.step().unwrap();

        let agent = world.state().agents.get("agent-1").unwrap();
        assert_eq!(agent.state.pos, pos(1.0, 1.0));
        assert_eq!(world.journal().len(), 2);
    }

    #[test]
    fn snapshot_and_replay() {
        let mut world = World::new();
        world.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            pos: pos(0.0, 0.0),
        });
        world.step().unwrap();
        let snapshot = world.snapshot();

        world.submit_action(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: pos(2.0, 2.0),
        });
        world.step().unwrap();

        let journal = world.journal().clone();
        let restored = World::from_snapshot(snapshot, journal).unwrap();
        assert_eq!(restored.state(), world.state());
    }

    #[test]
    fn rejects_invalid_actions() {
        let mut world = World::new();
        let action_id = world.submit_action(Action::MoveAgent {
            agent_id: "missing".to_string(),
            to: pos(1.0, 1.0),
        });
        world.step().unwrap();

        let event = world.journal().events.last().unwrap();
        match &event.body {
            WorldEventBody::Domain(DomainEvent::ActionRejected { action_id: id, reason }) => {
                assert_eq!(*id, action_id);
                assert!(matches!(reason, RejectReason::AgentNotFound { .. }));
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn effect_pipeline_signs_receipt() {
        let mut world = World::new();
        world.add_capability(CapabilityGrant::allow_all("cap_all"));
        world.set_policy(PolicySet::allow_all());
        world.set_receipt_signer(ReceiptSigner::hmac_sha256(b"secret"));

        let intent_id = world
            .emit_effect(
                "http.request",
                json!({"url": "https://example.com"}),
                "cap_all",
                EffectOrigin::System,
            )
            .unwrap();

        let intent = world.take_next_effect().unwrap();
        assert_eq!(intent.intent_id, intent_id);

        let receipt = EffectReceipt {
            intent_id: intent_id.clone(),
            status: "ok".to_string(),
            payload: json!({"status": 200}),
            cost_cents: Some(5),
            signature: None,
        };

        world.ingest_receipt(receipt).unwrap();

        let event = world.journal().events.last().unwrap();
        match &event.body {
            WorldEventBody::ReceiptAppended(receipt) => {
                assert!(receipt.signature.is_some());
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn policy_denies_effect() {
        let mut world = World::new();
        world.add_capability(CapabilityGrant::allow_all("cap_all"));
        world.set_policy(PolicySet {
            rules: vec![PolicyRule {
                when: PolicyWhen {
                    effect_kind: Some("http.request".to_string()),
                    origin_kind: None,
                    cap_name: None,
                },
                decision: PolicyDecision::Deny {
                    reason: "blocked".to_string(),
                },
            }],
        });

        let err = world
            .emit_effect(
                "http.request",
                json!({"url": "https://example.com"}),
                "cap_all",
                EffectOrigin::System,
            )
            .unwrap_err();

        assert!(matches!(err, WorldError::PolicyDenied { .. }));

        let event = world.journal().events.last().unwrap();
        match &event.body {
            WorldEventBody::PolicyDecisionRecorded(record) => {
                assert!(matches!(record.decision, PolicyDecision::Deny { .. }));
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn governance_flow_applies_manifest() {
        let mut world = World::new();
        let manifest = Manifest {
            version: 2,
            content: json!({ "name": "demo" }),
        };

        let proposal_id = world
            .propose_manifest_update(manifest.clone(), "alice")
            .unwrap();
        let shadow_hash = world.shadow_proposal(proposal_id).unwrap();
        world
            .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
            .unwrap();
        let applied_hash = world.apply_proposal(proposal_id).unwrap();

        assert_eq!(shadow_hash, applied_hash);
        assert_eq!(world.manifest().version, 2);
        assert_eq!(world.manifest().content, manifest.content);
    }

    #[test]
    fn governance_patch_updates_manifest() {
        let mut world = World::new();
        let base_hash = world.current_manifest_hash().unwrap();
        let patch = ManifestPatch {
            base_manifest_hash: base_hash,
            ops: vec![ManifestPatchOp::Set {
                path: vec!["settings".to_string(), "mode".to_string()],
                value: json!("fast"),
            }],
            new_version: Some(3),
        };

        let proposal_id = world
            .propose_manifest_patch(patch, "alice")
            .unwrap();
        world.shadow_proposal(proposal_id).unwrap();
        world
            .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
            .unwrap();
        world.apply_proposal(proposal_id).unwrap();

        assert_eq!(world.manifest().version, 3);
        assert_eq!(world.manifest().content, json!({ "settings": { "mode": "fast" } }));
    }

    #[test]
    fn manifest_diff_and_merge() {
        let base = Manifest {
            version: 1,
            content: json!({ "a": 1, "b": { "c": 2 } }),
        };
        let target = Manifest {
            version: 2,
            content: json!({ "a": 1, "b": { "c": 3 }, "d": 4 }),
        };

        let patch = diff_manifest(&base, &target).unwrap();
        let applied = apply_manifest_patch(&base, &patch).unwrap();
        assert_eq!(applied, target);

        let base_hash = hash_json(&base).unwrap();
        let patch1 = ManifestPatch {
            base_manifest_hash: base_hash.clone(),
            ops: vec![ManifestPatchOp::Set {
                path: vec!["b".to_string(), "c".to_string()],
                value: json!(3),
            }],
            new_version: Some(2),
        };
        let patch2 = ManifestPatch {
            base_manifest_hash: base_hash,
            ops: vec![ManifestPatchOp::Set {
                path: vec!["e".to_string()],
                value: json!(5),
            }],
            new_version: Some(3),
        };

        let merged = merge_manifest_patches(&base, &[patch1, patch2]).unwrap();
        let merged_applied = apply_manifest_patch(&base, &merged).unwrap();
        let expected = Manifest {
            version: 3,
            content: json!({ "a": 1, "b": { "c": 3 }, "e": 5 }),
        };
        assert_eq!(merged_applied, expected);
    }

    #[test]
    fn merge_reports_conflicts() {
        let base = Manifest {
            version: 1,
            content: json!({ "a": { "b": 1 }, "x": 1 }),
        };
        let base_hash = hash_json(&base).unwrap();
        let patch1 = ManifestPatch {
            base_manifest_hash: base_hash.clone(),
            ops: vec![ManifestPatchOp::Set {
                path: vec!["a".to_string(), "b".to_string()],
                value: json!(2),
            }],
            new_version: None,
        };
        let patch2 = ManifestPatch {
            base_manifest_hash: base_hash,
            ops: vec![ManifestPatchOp::Set {
                path: vec!["a".to_string()],
                value: json!({ "b": 3 }),
            }],
            new_version: None,
        };

        let result = merge_manifest_patches_with_conflicts(&base, &[patch1, patch2]).unwrap();
        assert_eq!(result.conflicts.len(), 1);
        assert_eq!(result.conflicts[0].path, vec!["a".to_string()]);
        assert_eq!(result.conflicts[0].kind, ConflictKind::PrefixOverlap);
        assert_eq!(result.conflicts[0].patches, vec![0, 1]);
        assert_eq!(result.conflicts[0].ops.len(), 2);
    }

    #[test]
    fn persist_and_restore_world() {
        let mut world = World::new();
        world.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            pos: pos(0.0, 0.0),
        });
        world.step().unwrap();

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("agent-world-{unique}"));

        world.save_to_dir(&dir).unwrap();

        let restored = World::load_from_dir(&dir).unwrap();
        assert_eq!(restored.state(), world.state());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rollback_to_snapshot_resets_state() {
        let mut world = World::new();
        world.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            pos: pos(0.0, 0.0),
        });
        world.step().unwrap();
        let snapshot = world.snapshot();

        world.submit_action(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: pos(9.0, 9.0),
        });
        world.step().unwrap();
        assert_eq!(world.state().agents.get("agent-1").unwrap().state.pos, pos(9.0, 9.0));

        let journal = world.journal().clone();
        world
            .rollback_to_snapshot(snapshot.clone(), journal, "test-rollback")
            .unwrap();

        assert_eq!(world.state(), &snapshot.state);
        let last = world.journal().events.last().unwrap();
        assert!(matches!(last.body, WorldEventBody::RollbackApplied(_)));
    }

    #[test]
    fn snapshot_retention_policy_prunes_old_entries() {
        let mut world = World::new();
        world.set_snapshot_retention(SnapshotRetentionPolicy { max_snapshots: 1 });

        world.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            pos: pos(0.0, 0.0),
        });
        world.step().unwrap();
        let snap1 = world.create_snapshot().unwrap();

        world.submit_action(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: pos(3.0, 3.0),
        });
        world.step().unwrap();
        let snap2 = world.create_snapshot().unwrap();

        assert_eq!(world.snapshot_catalog().records.len(), 1);
        let last_record = &world.snapshot_catalog().records[0];
        assert_eq!(last_record.snapshot_hash, hash_json(&snap2).unwrap());
        assert_ne!(last_record.snapshot_hash, hash_json(&snap1).unwrap());
    }

    #[test]
    fn snapshot_file_pruning_removes_old_files() {
        let mut world = World::new();
        world.set_snapshot_retention(SnapshotRetentionPolicy { max_snapshots: 1 });

        let dir = std::env::temp_dir().join(format!(
            "agent-world-snapshots-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));

        world.save_snapshot_to_dir(&dir).unwrap();
        world.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            pos: pos(0.0, 0.0),
        });
        world.step().unwrap();
        world.save_snapshot_to_dir(&dir).unwrap();

        let snapshots_dir = dir.join("snapshots");
        let file_count = fs::read_dir(&snapshots_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .count();
        assert_eq!(file_count, 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn audit_filter_by_kind_and_cause() {
        let mut world = World::new();
        world.add_capability(CapabilityGrant::allow_all("cap_all"));
        world.set_policy(PolicySet::allow_all());

        let intent_id = world
            .emit_effect(
                "http.request",
                json!({ "url": "https://example.com" }),
                "cap_all",
                EffectOrigin::System,
            )
            .unwrap();

        let intent = world.take_next_effect().unwrap();
        assert_eq!(intent.intent_id, intent_id);

        let receipt = EffectReceipt {
            intent_id: intent_id.clone(),
            status: "ok".to_string(),
            payload: json!({ "status": 200 }),
            cost_cents: None,
            signature: None,
        };
        world.ingest_receipt(receipt).unwrap();

        let filter = AuditFilter {
            kinds: Some(vec![AuditEventKind::ReceiptAppended]),
            caused_by: Some(AuditCausedBy::Effect),
            ..AuditFilter::default()
        };
        let events = world.audit_events(&filter);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0].caused_by,
            Some(CausedBy::Effect { .. })
        ));
    }

    #[test]
    fn audit_log_export_writes_file() {
        let mut world = World::new();
        world.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            pos: pos(0.0, 0.0),
        });
        world.step().unwrap();

        let dir = std::env::temp_dir().join(format!(
            "agent-world-audit-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("audit.json");

        world
            .save_audit_log(&path, &AuditFilter::default())
            .unwrap();
        let events: Vec<WorldEvent> = read_json_from_path(&path).unwrap();
        assert!(!events.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn scheduler_round_robin() {
        let mut world = World::new();
        world.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            pos: pos(0.0, 0.0),
        });
        world.submit_action(Action::RegisterAgent {
            agent_id: "agent-2".to_string(),
            pos: pos(1.0, 1.0),
        });
        world.step().unwrap();

        let first = world.schedule_next().unwrap();
        assert_eq!(first.agent_id, "agent-1");
        let second = world.schedule_next().unwrap();
        assert_eq!(second.agent_id, "agent-2");
        assert!(world.schedule_next().is_none());
    }
}
