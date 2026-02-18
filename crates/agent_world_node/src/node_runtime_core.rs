use std::fmt;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use crate::{
    NodeCommittedActionBatch, NodeConfig, NodeConsensusAction, NodeConsensusSnapshot, NodeError,
    NodeExecutionHook, NodeReplicationNetworkHandle, NodeRuntime,
};

#[derive(Debug, Clone)]
pub(super) struct RuntimeState {
    pub(super) tick_count: u64,
    pub(super) last_tick_unix_ms: Option<i64>,
    pub(super) consensus: NodeConsensusSnapshot,
    pub(super) last_error: Option<String>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            tick_count: 0,
            last_tick_unix_ms: None,
            consensus: NodeConsensusSnapshot::default(),
            last_error: None,
        }
    }
}

impl fmt::Debug for NodeRuntime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeRuntime")
            .field("config", &self.config)
            .field(
                "has_replication_network",
                &self.replication_network.is_some(),
            )
            .field("has_execution_hook", &self.execution_hook.is_some())
            .field("running", &self.running.load(Ordering::SeqCst))
            .finish()
    }
}

impl NodeRuntime {
    pub fn new(config: NodeConfig) -> Self {
        Self {
            config,
            replication_network: None,
            execution_hook: None,
            pending_consensus_actions: Arc::new(Mutex::new(Vec::new())),
            committed_action_batches: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            state: Arc::new(Mutex::new(RuntimeState::default())),
            stop_tx: None,
            worker: None,
        }
    }

    pub fn with_replication_network(mut self, network: NodeReplicationNetworkHandle) -> Self {
        self.replication_network = Some(network);
        self
    }

    pub fn with_execution_hook<T>(mut self, hook: T) -> Self
    where
        T: NodeExecutionHook + 'static,
    {
        self.execution_hook = Some(Arc::new(Mutex::new(Box::new(hook))));
        self
    }

    pub fn config(&self) -> &NodeConfig {
        &self.config
    }

    pub fn submit_consensus_action_payload(
        &self,
        action_id: u64,
        payload_cbor: Vec<u8>,
    ) -> Result<(), NodeError> {
        self.submit_consensus_action_payload_as_player(
            self.config.player_id.clone(),
            action_id,
            payload_cbor,
        )
    }

    pub fn submit_consensus_action_payload_as_player(
        &self,
        player_id: impl Into<String>,
        action_id: u64,
        payload_cbor: Vec<u8>,
    ) -> Result<(), NodeError> {
        let player_id = player_id.into();
        let player_id = player_id.trim();
        if player_id.is_empty() {
            return Err(NodeError::Consensus {
                reason: "submitter player_id cannot be empty".to_string(),
            });
        }
        if player_id != self.config.player_id {
            return Err(NodeError::Consensus {
                reason: format!(
                    "submitter player_id mismatch expected={} actual={}",
                    self.config.player_id, player_id
                ),
            });
        }
        let action = NodeConsensusAction::from_payload(
            action_id,
            self.config.player_id.clone(),
            payload_cbor,
        )?;
        let mut pending = self
            .pending_consensus_actions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        pending.push(action);
        Ok(())
    }

    pub fn drain_committed_action_batches(&self) -> Vec<NodeCommittedActionBatch> {
        let mut committed = self
            .committed_action_batches
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::mem::take(&mut *committed)
    }
}
