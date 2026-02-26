//! Governance types - proposals, decisions, and events.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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

/// Finality certificate bound to consensus height and signer threshold.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GovernanceFinalityCertificate {
    pub proposal_id: ProposalId,
    pub manifest_hash: String,
    pub consensus_height: u64,
    pub threshold: u16,
    pub signatures: BTreeMap<String, String>,
}

impl GovernanceFinalityCertificate {
    pub const SIGNATURE_PREFIX_ED25519_V1: &'static str = "govsig:ed25519:v1:";

    pub fn signing_payload_v1(
        proposal_id: ProposalId,
        manifest_hash: &str,
        consensus_height: u64,
        threshold: u16,
        signer_node_id: &str,
    ) -> Vec<u8> {
        format!(
            "govfinal:ed25519:v1|{proposal_id}|{manifest_hash}|{consensus_height}|{threshold}|{signer_node_id}"
        )
        .into_bytes()
    }
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        consensus_height: Option<u64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        threshold: Option<u16>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        signer_node_ids: Vec<String>,
    },
}

/// Schedule entry for agent activation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentSchedule {
    pub agent_id: String,
    pub event: DomainEvent,
}
