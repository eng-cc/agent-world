use std::collections::BTreeMap;
use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use crate::pos_validation::validate_pos_config;
use crate::{NodeConsensusAction, NodeError, NodeReplicationConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    Sequencer,
    Storage,
    Observer,
}

impl NodeRole {
    pub fn as_str(self) -> &'static str {
        match self {
            NodeRole::Sequencer => "sequencer",
            NodeRole::Storage => "storage",
            NodeRole::Observer => "observer",
        }
    }
}

impl fmt::Display for NodeRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for NodeRole {
    type Err = NodeError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "sequencer" => Ok(NodeRole::Sequencer),
            "storage" => Ok(NodeRole::Storage),
            "observer" => Ok(NodeRole::Observer),
            _ => Err(NodeError::InvalidRole {
                role: raw.to_string(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeConsensusMode {
    Pos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum PosConsensusStatus {
    Pending,
    Committed,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PosValidator {
    pub validator_id: String,
    pub stake: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodePosConfig {
    pub validators: Vec<PosValidator>,
    pub supermajority_numerator: u64,
    pub supermajority_denominator: u64,
    pub epoch_length_slots: u64,
}

impl NodePosConfig {
    pub fn ethereum_like(validators: Vec<PosValidator>) -> Self {
        Self {
            validators,
            supermajority_numerator: 2,
            supermajority_denominator: 3,
            epoch_length_slots: 32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeConfig {
    pub node_id: String,
    pub world_id: String,
    pub tick_interval: Duration,
    pub role: NodeRole,
    pub pos_config: NodePosConfig,
    pub auto_attest_all_validators: bool,
    pub gossip: Option<NodeGossipConfig>,
    pub replication: Option<NodeReplicationConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeGossipConfig {
    pub bind_addr: SocketAddr,
    pub peers: Vec<SocketAddr>,
}

impl NodeConfig {
    pub fn new(
        node_id: impl Into<String>,
        world_id: impl Into<String>,
        role: NodeRole,
    ) -> Result<Self, NodeError> {
        let node_id = node_id.into();
        let world_id = world_id.into();
        if node_id.trim().is_empty() {
            return Err(NodeError::InvalidConfig {
                reason: "node_id cannot be empty".to_string(),
            });
        }
        if world_id.trim().is_empty() {
            return Err(NodeError::InvalidConfig {
                reason: "world_id cannot be empty".to_string(),
            });
        }

        let pos_config = NodePosConfig::ethereum_like(vec![PosValidator {
            validator_id: node_id.clone(),
            stake: 100,
        }]);
        validate_pos_config(&pos_config)?;

        Ok(Self {
            node_id,
            world_id,
            tick_interval: Duration::from_millis(200),
            role,
            pos_config,
            auto_attest_all_validators: true,
            gossip: None,
            replication: None,
        })
    }

    pub fn with_tick_interval(mut self, interval: Duration) -> Result<Self, NodeError> {
        if interval.is_zero() {
            return Err(NodeError::InvalidConfig {
                reason: "tick_interval must be positive".to_string(),
            });
        }
        self.tick_interval = interval;
        Ok(self)
    }

    pub fn with_pos_config(mut self, pos_config: NodePosConfig) -> Result<Self, NodeError> {
        validate_pos_config(&pos_config)?;
        self.pos_config = pos_config;
        Ok(self)
    }

    pub fn with_pos_validators(self, validators: Vec<PosValidator>) -> Result<Self, NodeError> {
        self.with_pos_config(NodePosConfig::ethereum_like(validators))
    }

    pub fn with_auto_attest_all_validators(mut self, enabled: bool) -> Self {
        self.auto_attest_all_validators = enabled;
        self
    }

    pub fn with_gossip(
        mut self,
        bind_addr: SocketAddr,
        peers: Vec<SocketAddr>,
    ) -> Result<Self, NodeError> {
        if peers.is_empty() {
            return Err(NodeError::InvalidConfig {
                reason: "gossip peers cannot be empty".to_string(),
            });
        }
        let mut dedup = BTreeMap::new();
        for peer in peers {
            dedup.insert(peer, ());
        }
        self.gossip = Some(NodeGossipConfig {
            bind_addr,
            peers: dedup.keys().copied().collect(),
        });
        Ok(self)
    }

    pub fn with_gossip_optional(mut self, bind_addr: SocketAddr, peers: Vec<SocketAddr>) -> Self {
        let mut dedup = BTreeMap::new();
        for peer in peers {
            dedup.insert(peer, ());
        }
        self.gossip = Some(NodeGossipConfig {
            bind_addr,
            peers: dedup.keys().copied().collect(),
        });
        self
    }

    pub fn with_replication_root(
        mut self,
        root_dir: impl Into<PathBuf>,
    ) -> Result<Self, NodeError> {
        self.replication = Some(NodeReplicationConfig::new(root_dir)?);
        Ok(self)
    }

    pub fn with_replication(mut self, replication: NodeReplicationConfig) -> Self {
        self.replication = Some(replication);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeCommittedActionBatch {
    pub height: u64,
    pub slot: u64,
    pub epoch: u64,
    pub block_hash: String,
    pub action_root: String,
    pub committed_at_unix_ms: i64,
    pub actions: Vec<NodeConsensusAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeConsensusSnapshot {
    pub mode: NodeConsensusMode,
    pub slot: u64,
    pub epoch: u64,
    pub latest_height: u64,
    pub committed_height: u64,
    pub network_committed_height: u64,
    pub known_peer_heads: usize,
    pub last_status: Option<PosConsensusStatus>,
    pub last_block_hash: Option<String>,
    pub last_execution_height: u64,
    pub last_execution_block_hash: Option<String>,
    pub last_execution_state_root: Option<String>,
}

impl Default for NodeConsensusSnapshot {
    fn default() -> Self {
        Self {
            mode: NodeConsensusMode::Pos,
            slot: 0,
            epoch: 0,
            latest_height: 0,
            committed_height: 0,
            network_committed_height: 0,
            known_peer_heads: 0,
            last_status: None,
            last_block_hash: None,
            last_execution_height: 0,
            last_execution_block_hash: None,
            last_execution_state_root: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeSnapshot {
    pub node_id: String,
    pub world_id: String,
    pub role: NodeRole,
    pub running: bool,
    pub tick_count: u64,
    pub last_tick_unix_ms: Option<i64>,
    pub consensus: NodeConsensusSnapshot,
    pub last_error: Option<String>,
}
