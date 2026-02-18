//! World event types that wrap all event kinds.

use crate::simulator::ResourceKind;
use agent_world_wasm_abi::{ModuleCallFailure, ModuleEmitEvent, ModuleStateUpdate};
use serde::{Deserialize, Serialize};

use super::audit::AuditEventKind;
use super::effect::{EffectIntent, EffectReceipt};
use super::events::{CausedBy, DomainEvent};
use super::governance::GovernanceEvent;
use super::manifest::ManifestUpdate;
use super::modules::ModuleEvent;
use super::policy::PolicyDecisionRecord;
use super::rules::{ActionOverrideRecord, RuleDecisionRecord};
use super::snapshot::{RollbackEvent, SnapshotMeta};
use super::types::{WorldEventId, WorldTime};

/// A world event with full metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldEvent {
    pub id: WorldEventId,
    pub time: WorldTime,
    pub caused_by: Option<CausedBy>,
    pub body: WorldEventBody,
}

impl WorldEvent {
    pub fn audit_kind(&self) -> AuditEventKind {
        match self.body {
            WorldEventBody::Domain(_) => AuditEventKind::Domain,
            WorldEventBody::EffectQueued(_) => AuditEventKind::EffectQueued,
            WorldEventBody::ReceiptAppended(_) => AuditEventKind::ReceiptAppended,
            WorldEventBody::PolicyDecisionRecorded(_) => AuditEventKind::PolicyDecision,
            WorldEventBody::RuleDecisionRecorded(_) => AuditEventKind::RuleDecision,
            WorldEventBody::ActionOverridden(_) => AuditEventKind::ActionOverridden,
            WorldEventBody::Governance(_) => AuditEventKind::Governance,
            WorldEventBody::ModuleEvent(_) => AuditEventKind::ModuleEvent,
            WorldEventBody::ModuleCallFailed(_) => AuditEventKind::ModuleCallFailed,
            WorldEventBody::ModuleEmitted(_) => AuditEventKind::ModuleEmitted,
            WorldEventBody::ModuleStateUpdated(_) => AuditEventKind::ModuleStateUpdated,
            WorldEventBody::ModuleRuntimeCharged(_) => AuditEventKind::ModuleRuntimeCharged,
            WorldEventBody::SnapshotCreated(_) => AuditEventKind::SnapshotCreated,
            WorldEventBody::ManifestUpdated(_) => AuditEventKind::ManifestUpdated,
            WorldEventBody::RollbackApplied(_) => AuditEventKind::RollbackApplied,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleRuntimeChargeEvent {
    pub module_id: String,
    pub trace_id: String,
    pub payer_agent_id: String,
    pub compute_fee_kind: ResourceKind,
    pub compute_fee_amount: i64,
    pub electricity_fee_kind: ResourceKind,
    pub electricity_fee_amount: i64,
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub effect_count: u32,
    pub emit_count: u32,
}

/// The body/payload of a world event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum WorldEventBody {
    Domain(DomainEvent),
    EffectQueued(EffectIntent),
    ReceiptAppended(EffectReceipt),
    PolicyDecisionRecorded(PolicyDecisionRecord),
    RuleDecisionRecorded(RuleDecisionRecord),
    ActionOverridden(ActionOverrideRecord),
    Governance(GovernanceEvent),
    ModuleEvent(ModuleEvent),
    ModuleCallFailed(ModuleCallFailure),
    ModuleEmitted(ModuleEmitEvent),
    ModuleStateUpdated(ModuleStateUpdate),
    ModuleRuntimeCharged(ModuleRuntimeChargeEvent),
    SnapshotCreated(SnapshotMeta),
    ManifestUpdated(ManifestUpdate),
    RollbackApplied(RollbackEvent),
}
