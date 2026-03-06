//! Governance types - proposals, decisions, and events.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::events::DomainEvent;
use super::gameplay_state::GovernanceIdentityStatus;
use super::manifest::{Manifest, ManifestPatch};
use super::types::{ProposalId, WorldTime};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceExecutionPolicy {
    pub timelock_ticks: u64,
    pub epoch_length_ticks: u64,
    pub activation_delay_epochs: u64,
    pub emergency_brake_guardian_threshold: u16,
    pub emergency_veto_guardian_threshold: u16,
    pub emergency_brake_max_ticks: u64,
}

impl Default for GovernanceExecutionPolicy {
    fn default() -> Self {
        Self {
            timelock_ticks: 0,
            epoch_length_ticks: 120,
            activation_delay_epochs: 0,
            emergency_brake_guardian_threshold: 2,
            emergency_veto_guardian_threshold: 2,
            emergency_brake_max_ticks: 720,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceIdentityPenaltyStatus {
    Applied,
    Appealed,
    AppealAccepted,
    AppealRejected,
}

impl Default for GovernanceIdentityPenaltyStatus {
    fn default() -> Self {
        Self::Applied
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GovernanceIdentityPenaltyRecord {
    pub penalty_id: u64,
    pub target_agent_id: String,
    pub evidence_hash: String,
    pub reason: String,
    #[serde(default)]
    pub slash_stake: u64,
    #[serde(default)]
    pub appeal_deadline_tick: WorldTime,
    #[serde(default)]
    pub status: GovernanceIdentityPenaltyStatus,
    #[serde(default)]
    pub identity_status_before: GovernanceIdentityStatus,
    #[serde(default)]
    pub detection_source: String,
    #[serde(default)]
    pub detection_risk_score: i64,
    #[serde(default)]
    pub detection_incident_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub evidence_chain_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub appeal_evidence_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution_evidence_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub appellant: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub appeal_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at_tick: Option<WorldTime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GovernanceIdentityPenaltyMonitorStats {
    pub total_penalties: u64,
    pub appealed_penalties: u64,
    pub resolved_appeals: u64,
    pub appeal_accepted_penalties: u64,
    pub high_risk_open_penalties: u64,
    pub false_positive_rate_bps: u16,
}

/// A proposal for manifest changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Proposal {
    pub id: ProposalId,
    pub author: String,
    pub base_manifest_hash: String,
    pub manifest: Manifest,
    pub patch: Option<ManifestPatch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queued_at_tick: Option<WorldTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub not_before_tick: Option<WorldTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activate_epoch: Option<u64>,
    #[serde(default)]
    pub timelock_ticks: u64,
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
    Queued {
        proposal_id: ProposalId,
        manifest_hash: String,
        queued_at_tick: WorldTime,
        not_before_tick: WorldTime,
        activate_epoch: u64,
        timelock_ticks: u64,
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
    EmergencyBrakeActivated {
        initiator: String,
        reason: String,
        active_until_tick: WorldTime,
        threshold: u16,
        signer_node_ids: Vec<String>,
    },
    EmergencyBrakeReleased {
        initiator: String,
        reason: String,
        #[serde(default)]
        threshold: u16,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        signer_node_ids: Vec<String>,
    },
    EmergencyVetoed {
        proposal_id: ProposalId,
        initiator: String,
        reason: String,
        threshold: u16,
        signer_node_ids: Vec<String>,
    },
    IdentityPenaltyApplied {
        penalty_id: u64,
        target_agent_id: String,
        evidence_hash: String,
        initiator: String,
        reason: String,
        slash_stake: u64,
        appeal_deadline_tick: WorldTime,
        threshold: u16,
        signer_node_ids: Vec<String>,
    },
    IdentityPenaltyAppealed {
        penalty_id: u64,
        appellant: String,
        reason: String,
    },
    IdentityPenaltyResolved {
        penalty_id: u64,
        resolver: String,
        accepted: bool,
        reason: String,
    },
}

/// Schedule entry for agent activation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentSchedule {
    pub agent_id: String,
    pub event: DomainEvent,
}
