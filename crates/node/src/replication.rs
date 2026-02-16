use std::fs;
use std::path::{Path, PathBuf};

use agent_world_distfs::{
    apply_replication_record, build_replication_record, FileReplicationRecord, LocalCasStore,
    SingleWriterReplicationGuard,
};
use serde::{Deserialize, Serialize};

use crate::{NodeError, PosConsensusStatus, PosDecision};

const REPLICATION_VERSION: u8 = 1;
const COMMIT_FILE_PREFIX: &str = "consensus/commits";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeReplicationConfig {
    pub root_dir: PathBuf,
}

impl NodeReplicationConfig {
    pub fn new(root_dir: impl Into<PathBuf>) -> Result<Self, NodeError> {
        let root_dir = root_dir.into();
        if root_dir.as_os_str().is_empty() {
            return Err(NodeError::InvalidConfig {
                reason: "replication root_dir cannot be empty".to_string(),
            });
        }
        Ok(Self { root_dir })
    }

    fn store_root(&self) -> PathBuf {
        self.root_dir.join("store")
    }

    fn guard_state_path(&self) -> PathBuf {
        self.root_dir.join("replication_guard.json")
    }

    fn writer_state_path(&self, node_id: &str) -> PathBuf {
        self.root_dir
            .join(format!("replication_writer_state_{node_id}.json"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GossipReplicationMessage {
    pub version: u8,
    pub world_id: String,
    pub node_id: String,
    pub record: FileReplicationRecord,
    pub payload: Vec<u8>,
}

#[derive(Debug)]
pub(crate) struct ReplicationRuntime {
    config: NodeReplicationConfig,
    store: LocalCasStore,
    guard: SingleWriterReplicationGuard,
    writer_state: LocalWriterState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct LocalWriterState {
    last_sequence: u64,
    last_replicated_height: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ReplicatedCommitPayload {
    world_id: String,
    node_id: String,
    height: u64,
    slot: u64,
    epoch: u64,
    block_hash: String,
    committed_at_ms: i64,
}

impl ReplicationRuntime {
    pub(crate) fn new(config: &NodeReplicationConfig, node_id: &str) -> Result<Self, NodeError> {
        fs::create_dir_all(&config.root_dir).map_err(|err| NodeError::Replication {
            reason: format!(
                "create replication root {} failed: {}",
                config.root_dir.display(),
                err
            ),
        })?;

        let guard = load_json_or_default::<SingleWriterReplicationGuard>(
            config.guard_state_path().as_path(),
        )?;
        let writer_state =
            load_json_or_default::<LocalWriterState>(config.writer_state_path(node_id).as_path())?;

        Ok(Self {
            config: config.clone(),
            store: LocalCasStore::new(config.store_root()),
            guard,
            writer_state,
        })
    }

    pub(crate) fn build_local_commit_message(
        &mut self,
        node_id: &str,
        world_id: &str,
        now_ms: i64,
        decision: &PosDecision,
    ) -> Result<Option<GossipReplicationMessage>, NodeError> {
        if !matches!(decision.status, PosConsensusStatus::Committed) {
            return Ok(None);
        }
        if decision.height <= self.writer_state.last_replicated_height {
            return Ok(None);
        }

        let payload = ReplicatedCommitPayload {
            world_id: world_id.to_string(),
            node_id: node_id.to_string(),
            height: decision.height,
            slot: decision.slot,
            epoch: decision.epoch,
            block_hash: decision.block_hash.clone(),
            committed_at_ms: now_ms,
        };
        let payload_bytes = serde_json::to_vec(&payload).map_err(|err| NodeError::Replication {
            reason: format!("serialize local replication payload failed: {}", err),
        })?;
        let sequence = self
            .guard
            .last_sequence
            .max(self.writer_state.last_sequence)
            .saturating_add(1);
        let path = format!("{COMMIT_FILE_PREFIX}/{:020}.json", decision.height);
        let record = build_replication_record(
            world_id,
            node_id,
            sequence,
            path.as_str(),
            &payload_bytes,
            now_ms,
        )
        .map_err(distfs_error_to_node_error)?;

        apply_replication_record(&self.store, &mut self.guard, &record, &payload_bytes)
            .map_err(distfs_error_to_node_error)?;

        self.writer_state.last_sequence = record.sequence;
        self.writer_state.last_replicated_height = decision.height;
        self.persist_state(node_id)?;

        Ok(Some(GossipReplicationMessage {
            version: REPLICATION_VERSION,
            world_id: world_id.to_string(),
            node_id: node_id.to_string(),
            record,
            payload: payload_bytes,
        }))
    }

    pub(crate) fn apply_remote_message(
        &mut self,
        node_id: &str,
        world_id: &str,
        message: &GossipReplicationMessage,
    ) -> Result<(), NodeError> {
        if message.version != REPLICATION_VERSION {
            return Ok(());
        }
        if message.node_id == node_id {
            return Ok(());
        }
        if message.world_id != world_id || message.record.world_id != world_id {
            return Ok(());
        }

        if let Some(existing_writer) = self.guard.writer_id.as_deref() {
            if existing_writer != message.record.writer_id.as_str() {
                return Err(NodeError::Replication {
                    reason: format!(
                        "replication writer conflict: expected={}, got={}",
                        existing_writer, message.record.writer_id
                    ),
                });
            }
        }
        if message.record.sequence <= self.guard.last_sequence {
            return Ok(());
        }

        apply_replication_record(
            &self.store,
            &mut self.guard,
            &message.record,
            &message.payload,
        )
        .map_err(distfs_error_to_node_error)?;

        write_json_pretty(self.config.guard_state_path().as_path(), &self.guard)
    }

    fn persist_state(&self, node_id: &str) -> Result<(), NodeError> {
        write_json_pretty(self.config.guard_state_path().as_path(), &self.guard)?;
        write_json_pretty(
            self.config.writer_state_path(node_id).as_path(),
            &self.writer_state,
        )
    }
}

fn load_json_or_default<T>(path: &Path) -> Result<T, NodeError>
where
    T: for<'de> Deserialize<'de> + Default,
{
    if !path.exists() {
        return Ok(T::default());
    }
    let bytes = fs::read(path).map_err(|err| NodeError::Replication {
        reason: format!("read {} failed: {}", path.display(), err),
    })?;
    serde_json::from_slice::<T>(&bytes).map_err(|err| NodeError::Replication {
        reason: format!("parse {} failed: {}", path.display(), err),
    })
}

fn write_json_pretty<T: Serialize>(path: &Path, value: &T) -> Result<(), NodeError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| NodeError::Replication {
            reason: format!("create dir {} failed: {}", parent.display(), err),
        })?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|err| NodeError::Replication {
        reason: format!("serialize {} failed: {}", path.display(), err),
    })?;
    fs::write(path, bytes).map_err(|err| NodeError::Replication {
        reason: format!("write {} failed: {}", path.display(), err),
    })
}

fn distfs_error_to_node_error<E>(err: E) -> NodeError
where
    E: std::fmt::Debug,
{
    NodeError::Replication {
        reason: format!("{err:?}"),
    }
}
