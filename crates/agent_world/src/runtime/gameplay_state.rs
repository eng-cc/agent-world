//! Persisted gameplay-layer state models.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::types::WorldTime;

/// Persisted alliance relationship.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllianceState {
    pub alliance_id: String,
    pub members: Vec<String>,
    pub charter: String,
    pub formed_by_agent_id: String,
    pub formed_at: WorldTime,
}

/// Persisted war declaration state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WarState {
    pub war_id: String,
    pub initiator_agent_id: String,
    pub aggressor_alliance_id: String,
    pub defender_alliance_id: String,
    pub objective: String,
    pub intensity: u32,
    #[serde(default)]
    pub active: bool,
    pub declared_at: WorldTime,
}

/// Persisted ballot for one voter in one governance proposal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceVoteBallotState {
    pub option: String,
    pub weight: u32,
    pub voted_at: WorldTime,
}

/// Aggregated governance vote state by proposal key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GovernanceVoteState {
    pub proposal_key: String,
    #[serde(default)]
    pub votes_by_agent: BTreeMap<String, GovernanceVoteBallotState>,
    #[serde(default)]
    pub tallies: BTreeMap<String, u64>,
    #[serde(default)]
    pub total_weight: u64,
    pub last_updated_at: WorldTime,
}

/// Persisted crisis resolution state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrisisState {
    pub crisis_id: String,
    pub resolver_agent_id: String,
    pub strategy: String,
    pub success: bool,
    pub impact: i64,
    pub resolved_at: WorldTime,
}

/// Persisted meta progression state for one agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetaProgressState {
    pub agent_id: String,
    #[serde(default)]
    pub track_points: BTreeMap<String, i64>,
    #[serde(default)]
    pub total_points: i64,
    #[serde(default)]
    pub achievements: Vec<String>,
    pub last_granted_at: WorldTime,
}
