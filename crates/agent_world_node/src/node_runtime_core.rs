use std::fmt;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use agent_world_proto::distributed_dht as proto_dht;
use agent_world_proto::world_error::WorldError as ProtoWorldError;

use crate::{
    NodeCommittedActionBatch, NodeConfig, NodeConsensusAction, NodeConsensusSnapshot, NodeError,
    NodeExecutionHook, NodeReplicationNetworkHandle, NodeRuntime,
};

#[derive(Debug, Clone)]
pub(super) struct RuntimeState {
    pub(super) tick_count: u64,
    pub(super) last_tick_unix_ms: Option<i64>,
    pub(super) replica_maintenance_last_polled_at_ms: Option<i64>,
    pub(super) consensus: NodeConsensusSnapshot,
    pub(super) last_error: Option<String>,
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self {
            tick_count: 0,
            last_tick_unix_ms: None,
            replica_maintenance_last_polled_at_ms: None,
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
            replica_maintenance_dht: None,
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

    pub fn with_replica_maintenance_dht(
        mut self,
        dht: Arc<dyn proto_dht::DistributedDht<ProtoWorldError> + Send + Sync>,
    ) -> Self {
        self.replica_maintenance_dht = Some(dht);
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
        if payload_cbor.len() > self.config.max_consensus_action_payload_bytes {
            return Err(NodeError::Consensus {
                reason: format!(
                    "consensus action payload too large: bytes={} limit={}",
                    payload_cbor.len(),
                    self.config.max_consensus_action_payload_bytes
                ),
            });
        }
        let action = NodeConsensusAction::from_payload(
            action_id,
            self.config.player_id.clone(),
            payload_cbor,
        )
        .map_err(|err| NodeError::Consensus { reason: err.reason })?;
        let mut pending = self
            .pending_consensus_actions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if pending.len() >= self.config.max_pending_consensus_actions {
            return Err(NodeError::Consensus {
                reason: format!(
                    "pending consensus actions queue saturated: len={} limit={}",
                    pending.len(),
                    self.config.max_pending_consensus_actions
                ),
            });
        }
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
