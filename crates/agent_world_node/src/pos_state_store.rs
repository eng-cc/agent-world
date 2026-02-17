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
        }
    }

    pub(super) fn restore_state_snapshot(&mut self, snapshot: PosNodeStateSnapshot) {
        self.pending = None;
        self.committed_height = snapshot.committed_height;
        self.network_committed_height = snapshot
            .network_committed_height
            .max(snapshot.committed_height);
        self.next_height = snapshot
            .next_height
            .max(snapshot.committed_height.saturating_add(1))
            .max(1);
        self.next_slot = snapshot.next_slot;
        self.last_broadcast_proposal_height = snapshot.last_broadcast_proposal_height;
        self.last_broadcast_local_attestation_height =
            snapshot.last_broadcast_local_attestation_height;
        self.last_broadcast_committed_height = snapshot.last_broadcast_committed_height;
        self.last_committed_block_hash = snapshot.last_committed_block_hash.or_else(|| {
            if snapshot.committed_height > 0 {
                Some(format!("legacy-height-{}", snapshot.committed_height))
            } else {
                None
            }
        });
    }
}
