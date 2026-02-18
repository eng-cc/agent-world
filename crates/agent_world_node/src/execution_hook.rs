use serde::{Deserialize, Serialize};

use crate::NodeConsensusAction;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeExecutionCommitContext {
    pub world_id: String,
    pub node_id: String,
    pub height: u64,
    pub slot: u64,
    pub epoch: u64,
    pub node_block_hash: String,
    pub action_root: String,
    #[serde(default)]
    pub committed_actions: Vec<NodeConsensusAction>,
    pub committed_at_unix_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeExecutionCommitResult {
    pub execution_height: u64,
    pub execution_block_hash: String,
    pub execution_state_root: String,
}

pub trait NodeExecutionHook: Send {
    fn on_commit(
        &mut self,
        context: NodeExecutionCommitContext,
    ) -> Result<NodeExecutionCommitResult, String>;
}
