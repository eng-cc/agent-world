use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{NodeError, NodeReplicationConfig, PosNodeEngine};

const POS_STATE_FILE_NAME: &str = "node_pos_state.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct PosNodeStateSnapshot {
    pub next_height: u64,
    pub next_slot: u64,
    pub committed_height: u64,
    pub network_committed_height: u64,
    pub last_broadcast_proposal_height: u64,
    pub last_broadcast_local_attestation_height: u64,
    pub last_broadcast_committed_height: u64,
    #[serde(default)]
    pub last_committed_block_hash: Option<String>,
    #[serde(default)]
    pub last_execution_height: u64,
    #[serde(default)]
    pub last_execution_block_hash: Option<String>,
    #[serde(default)]
    pub last_execution_state_root: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PosNodeStateStore {
    path: PathBuf,
}

impl PosNodeStateStore {
    pub(crate) fn from_replication(replication: &NodeReplicationConfig) -> Self {
        Self {
            path: replication.root_dir.join(POS_STATE_FILE_NAME),
        }
    }

    pub(crate) fn load(&self) -> Result<Option<PosNodeStateSnapshot>, NodeError> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(None),
            Err(err) => {
                return Err(NodeError::Replication {
                    reason: format!(
                        "read node pos state {} failed: {}",
                        self.path.display(),
                        err
                    ),
                });
            }
        };
        let snapshot = serde_json::from_slice::<PosNodeStateSnapshot>(&bytes).map_err(|err| {
            NodeError::Replication {
                reason: format!(
                    "parse node pos state {} failed: {}",
                    self.path.display(),
                    err
                ),
            }
        })?;
        Ok(Some(snapshot))
    }

    pub(crate) fn save_engine_state(&self, engine: &PosNodeEngine) -> Result<(), NodeError> {
        self.save_snapshot(&engine.export_state_snapshot())
    }

    fn save_snapshot(&self, snapshot: &PosNodeStateSnapshot) -> Result<(), NodeError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|err| NodeError::Replication {
                reason: format!(
                    "create node pos state dir {} failed: {}",
                    parent.display(),
                    err
                ),
            })?;
        }
        let bytes = serde_json::to_vec_pretty(snapshot).map_err(|err| NodeError::Replication {
            reason: format!("serialize node pos state failed: {}", err),
        })?;
        let temp_path = self.path.with_extension("json.tmp");
        fs::write(&temp_path, bytes).map_err(|err| NodeError::Replication {
            reason: format!(
                "write node pos state temp {} failed: {}",
                temp_path.display(),
                err
            ),
        })?;
        fs::rename(&temp_path, &self.path).map_err(|err| NodeError::Replication {
            reason: format!(
                "rename node pos state temp {} -> {} failed: {}",
                temp_path.display(),
                self.path.display(),
                err
            ),
        })?;
        Ok(())
    }
}

impl PosNodeEngine {
    pub(super) fn export_state_snapshot(&self) -> PosNodeStateSnapshot {
        PosNodeStateSnapshot {
            next_height: self.next_height,
            next_slot: self.next_slot,
            committed_height: self.committed_height,
            network_committed_height: self.network_committed_height,
            last_broadcast_proposal_height: self.last_broadcast_proposal_height,
            last_broadcast_local_attestation_height: self.last_broadcast_local_attestation_height,
            last_broadcast_committed_height: self.last_broadcast_committed_height,
            last_committed_block_hash: self.last_committed_block_hash.clone(),
            last_execution_height: self.last_execution_height,
            last_execution_block_hash: self.last_execution_block_hash.clone(),
            last_execution_state_root: self.last_execution_state_root.clone(),
        }
    }

    pub(super) fn restore_state_snapshot(
        &mut self,
        snapshot: PosNodeStateSnapshot,
    ) -> Result<(), NodeError> {
        let PosNodeStateSnapshot {
            next_height: snapshot_next_height,
            next_slot,
            committed_height,
            network_committed_height: snapshot_network_committed_height,
            last_broadcast_proposal_height,
            last_broadcast_local_attestation_height,
            last_broadcast_committed_height,
            last_committed_block_hash,
            last_execution_height,
            last_execution_block_hash,
            last_execution_state_root,
        } = snapshot;
        let committed_successor =
            committed_height
                .checked_add(1)
                .ok_or_else(|| NodeError::Replication {
                    reason: format!(
                        "restore node pos state overflow: committed_height={} has no successor",
                        committed_height
                    ),
                })?;
        let restored_next_height = snapshot_next_height.max(committed_successor).max(1);
        let restored_network_committed_height =
            snapshot_network_committed_height.max(committed_height);
        let restored_committed_hash = last_committed_block_hash.or_else(|| {
            if committed_height > 0 {
                Some(format!("legacy-height-{}", committed_height))
            } else {
                None
            }
        });

        self.pending = None;
        self.pending_consensus_actions.clear();
        self.committed_height = committed_height;
        self.network_committed_height = restored_network_committed_height;
        self.next_height = restored_next_height;
        self.next_slot = next_slot;
        self.last_broadcast_proposal_height = last_broadcast_proposal_height;
        self.last_broadcast_local_attestation_height = last_broadcast_local_attestation_height;
        self.last_broadcast_committed_height = last_broadcast_committed_height;
        self.last_committed_block_hash = restored_committed_hash;
        self.last_execution_height = last_execution_height;
        self.last_execution_block_hash = last_execution_block_hash;
        self.last_execution_state_root = last_execution_state_root;
        self.execution_bindings.clear();
        self.remember_execution_binding_for_height(self.last_execution_height);
        Ok(())
    }
}
