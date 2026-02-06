//! Governance types - proposals, decisions, and events.

use serde::{Deserialize, Serialize};

use super::events::DomainEvent;
use super::manifest::{Manifest, ManifestPatch};
use super::types::ProposalId;

/// A proposal for manifest changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Proposal {
    pub id: ProposalId,
    pub author: String,
    pub base_manifest_hash: String,
    pub manifest: Manifest,
    pub patch: Option<ManifestPatch>,
    pub status: ProposalStatus,
}

/// The current status of a proposal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", content = "data")]
pub enum ProposalStatus {
    Proposed,
    Shadowed {
        manifest_hash: String,
    },
    Approved {
        manifest_hash: String,
        approver: String,
    },
    Rejected {
        reason: String,
    },
    Applied {
        manifest_hash: String,
    },
}

impl ProposalStatus {
    pub fn label(&self) -> String {
        match self {
            ProposalStatus::Proposed => "proposed".to_string(),
            ProposalStatus::Shadowed { .. } => "shadowed".to_string(),
            ProposalStatus::Approved { .. } => "approved".to_string(),
            ProposalStatus::Rejected { .. } => "rejected".to_string(),
            ProposalStatus::Applied { .. } => "applied".to_string(),
        }
    }
}

/// A decision on a proposal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "decision", content = "data")]
pub enum ProposalDecision {
    Approve,
    Reject { reason: String },
}

/// Events related to governance actions.
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
        #[serde(default)]
        manifest_hash: Option<String>,
    },
}

/// Schedule entry for agent activation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentSchedule {
    pub agent_id: String,
    pub event: DomainEvent,
}
