use serde::{Deserialize, Serialize};

use super::node_consensus_action::NodeConsensusAction;

fn default_legacy_player_id() -> String {
    "legacy".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeGossipCommitMessage {
    pub version: u8,
    pub world_id: String,
    pub node_id: String,
    #[serde(default = "default_legacy_player_id")]
    pub player_id: String,
    pub height: u64,
    pub slot: u64,
    pub epoch: u64,
    pub block_hash: String,
    #[serde(default)]
    pub action_root: String,
    #[serde(default)]
    pub actions: Vec<NodeConsensusAction>,
    pub committed_at_ms: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_block_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_state_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_hex: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeGossipProposalMessage {
    pub version: u8,
    pub world_id: String,
    pub node_id: String,
    #[serde(default = "default_legacy_player_id")]
    pub player_id: String,
    pub proposer_id: String,
    pub height: u64,
    pub slot: u64,
    pub epoch: u64,
    pub block_hash: String,
    #[serde(default)]
    pub action_root: String,
    #[serde(default)]
    pub actions: Vec<NodeConsensusAction>,
    pub proposed_at_ms: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_hex: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeGossipAttestationMessage {
    pub version: u8,
    pub world_id: String,
    pub node_id: String,
    #[serde(default = "default_legacy_player_id")]
    pub player_id: String,
    pub validator_id: String,
    pub height: u64,
    pub slot: u64,
    pub epoch: u64,
    pub block_hash: String,
    pub approve: bool,
    pub source_epoch: u64,
    pub target_epoch: u64,
    pub voted_at_ms: i64,
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_hex: Option<String>,
}
