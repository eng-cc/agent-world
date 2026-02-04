//! World event types that wrap all event kinds.

use serde::{Deserialize, Serialize};

use super::audit::AuditEventKind;
use super::effect::{EffectIntent, EffectReceipt};
use super::events::{CausedBy, DomainEvent};
use super::governance::GovernanceEvent;
use super::manifest::ManifestUpdate;
use super::modules::ModuleEvent;
use super::sandbox::{ModuleCallFailure, ModuleEmitEvent, ModuleStateUpdate};
use super::policy::PolicyDecisionRecord;
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
            WorldEventBody::Governance(_) => AuditEventKind::Governance,
            WorldEventBody::ModuleEvent(_) => AuditEventKind::ModuleEvent,
            WorldEventBody::ModuleCallFailed(_) => AuditEventKind::ModuleCallFailed,
            WorldEventBody::ModuleEmitted(_) => AuditEventKind::ModuleEmitted,
            WorldEventBody::ModuleStateUpdated(_) => AuditEventKind::ModuleStateUpdated,
            WorldEventBody::SnapshotCreated(_) => AuditEventKind::SnapshotCreated,
            WorldEventBody::ManifestUpdated(_) => AuditEventKind::ManifestUpdated,
            WorldEventBody::RollbackApplied(_) => AuditEventKind::RollbackApplied,
        }
    }
}

/// The body/payload of a world event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum WorldEventBody {
    Domain(DomainEvent),
    EffectQueued(EffectIntent),
    ReceiptAppended(EffectReceipt),
    PolicyDecisionRecorded(PolicyDecisionRecord),
    Governance(GovernanceEvent),
    ModuleEvent(ModuleEvent),
    ModuleCallFailed(ModuleCallFailure),
    ModuleEmitted(ModuleEmitEvent),
    ModuleStateUpdated(ModuleStateUpdate),
    SnapshotCreated(SnapshotMeta),
    ManifestUpdated(ManifestUpdate),
    RollbackApplied(RollbackEvent),
}
