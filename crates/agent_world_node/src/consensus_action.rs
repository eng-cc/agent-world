use std::collections::BTreeMap;

use agent_world_distfs::blake3_hex;
use serde::{Deserialize, Serialize};

use crate::NodeError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeConsensusAction {
    pub action_id: u64,
    #[serde(default = "default_submitter_player_id")]
    pub submitter_player_id: String,
    pub payload_cbor: Vec<u8>,
    pub payload_hash: String,
}

impl NodeConsensusAction {
    pub fn from_payload(
        action_id: u64,
        submitter_player_id: impl Into<String>,
        payload_cbor: Vec<u8>,
    ) -> Result<Self, NodeError> {
        if action_id == 0 {
            return Err(NodeError::Consensus {
                reason: "consensus action_id must be > 0".to_string(),
            });
        }
        let submitter_player_id = submitter_player_id.into();
        let submitter_player_id = submitter_player_id.trim();
        if submitter_player_id.is_empty() {
            return Err(NodeError::Consensus {
                reason: "consensus action submitter_player_id cannot be empty".to_string(),
            });
        }
        let payload_hash = blake3_hex(payload_cbor.as_slice());
        Ok(Self {
            action_id,
            submitter_player_id: submitter_player_id.to_string(),
            payload_cbor,
            payload_hash,
        })
    }

    pub fn validate(&self) -> Result<(), NodeError> {
        if self.action_id == 0 {
            return Err(NodeError::Consensus {
                reason: "consensus action_id must be > 0".to_string(),
            });
        }
        if self.submitter_player_id.trim().is_empty() {
            return Err(NodeError::Consensus {
                reason: format!(
                    "consensus action submitter_player_id is empty action_id={}",
                    self.action_id
                ),
            });
        }
        let expected_hash = blake3_hex(self.payload_cbor.as_slice());
        if expected_hash != self.payload_hash {
            return Err(NodeError::Consensus {
                reason: format!(
                    "consensus action payload hash mismatch action_id={} expected={} actual={}",
                    self.action_id, expected_hash, self.payload_hash
                ),
            });
        }
        Ok(())
    }
}

fn default_submitter_player_id() -> String {
    "legacy".to_string()
}

#[derive(Debug, Serialize)]
struct ActionRootPayload<'a> {
    version: u8,
    actions: Vec<ActionRootEntry<'a>>,
}

#[derive(Debug, Serialize)]
struct ActionRootEntry<'a> {
    action_id: u64,
    submitter_player_id: &'a str,
    payload_hash: &'a str,
}

#[derive(Debug, Serialize)]
struct ActionRootLegacyPayload<'a> {
    version: u8,
    actions: Vec<ActionRootLegacyEntry<'a>>,
}

#[derive(Debug, Serialize)]
struct ActionRootLegacyEntry<'a> {
    action_id: u64,
    payload_hash: &'a str,
}

pub(crate) fn merge_pending_consensus_actions(
    pending: &mut BTreeMap<u64, NodeConsensusAction>,
    incoming: Vec<NodeConsensusAction>,
) -> Result<(), NodeError> {
    for action in incoming {
        action.validate()?;
        match pending.get(&action.action_id) {
            Some(existing) if existing.payload_hash != action.payload_hash => {
                return Err(NodeError::Consensus {
                    reason: format!(
                        "conflicting consensus action payload for action_id={}",
                        action.action_id
                    ),
                });
            }
            Some(existing) if existing.submitter_player_id != action.submitter_player_id => {
                return Err(NodeError::Consensus {
                    reason: format!(
                        "conflicting consensus action submitter for action_id={}",
                        action.action_id
                    ),
                });
            }
            Some(_) => {}
            None => {
                pending.insert(action.action_id, action);
            }
        }
    }
    Ok(())
}

pub(crate) fn drain_ordered_consensus_actions(
    pending: &mut BTreeMap<u64, NodeConsensusAction>,
) -> Vec<NodeConsensusAction> {
    let mut drained = Vec::with_capacity(pending.len());
    let taken = std::mem::take(pending);
    for (_, action) in taken {
        drained.push(action);
    }
    drained
}

pub fn compute_consensus_action_root(actions: &[NodeConsensusAction]) -> Result<String, NodeError> {
    compute_consensus_action_root_v2(actions)
}

fn compute_consensus_action_root_v2(actions: &[NodeConsensusAction]) -> Result<String, NodeError> {
    let mut last_action_id = 0_u64;
    let mut entries = Vec::with_capacity(actions.len());
    for action in actions {
        action.validate()?;
        if action.action_id <= last_action_id {
            return Err(NodeError::Consensus {
                reason: format!(
                    "consensus actions must be strictly ordered action_id={} last_action_id={}",
                    action.action_id, last_action_id
                ),
            });
        }
        last_action_id = action.action_id;
        entries.push(ActionRootEntry {
            action_id: action.action_id,
            submitter_player_id: action.submitter_player_id.as_str(),
            payload_hash: action.payload_hash.as_str(),
        });
    }

    let payload = ActionRootPayload {
        version: 2,
        actions: entries,
    };
    let bytes = serde_cbor::to_vec(&payload).map_err(|err| NodeError::Consensus {
        reason: format!("encode consensus action root payload failed: {err}"),
    })?;
    Ok(blake3_hex(bytes.as_slice()))
}

fn compute_consensus_action_root_v1_legacy(
    actions: &[NodeConsensusAction],
) -> Result<String, NodeError> {
    let mut last_action_id = 0_u64;
    let mut entries = Vec::with_capacity(actions.len());
    for action in actions {
        action.validate()?;
        if action.action_id <= last_action_id {
            return Err(NodeError::Consensus {
                reason: format!(
                    "consensus actions must be strictly ordered action_id={} last_action_id={}",
                    action.action_id, last_action_id
                ),
            });
        }
        last_action_id = action.action_id;
        entries.push(ActionRootLegacyEntry {
            action_id: action.action_id,
            payload_hash: action.payload_hash.as_str(),
        });
    }

    let payload = ActionRootLegacyPayload {
        version: 1,
        actions: entries,
    };
    let bytes = serde_cbor::to_vec(&payload).map_err(|err| NodeError::Consensus {
        reason: format!("encode legacy consensus action root payload failed: {err}"),
    })?;
    Ok(blake3_hex(bytes.as_slice()))
}

pub(crate) fn validate_consensus_action_root(
    action_root: &str,
    actions: &[NodeConsensusAction],
) -> Result<(), NodeError> {
    if action_root.trim().is_empty() {
        return Err(NodeError::Consensus {
            reason: "consensus action_root is empty".to_string(),
        });
    }
    let computed_v2 = compute_consensus_action_root_v2(actions)?;
    if computed_v2 == action_root {
        return Ok(());
    }
    let computed_v1 = compute_consensus_action_root_v1_legacy(actions)?;
    if computed_v1 == action_root {
        return Ok(());
    }
    Err(NodeError::Consensus {
        reason: format!(
            "consensus action_root mismatch expected_v2={} expected_v1={} actual={}",
            computed_v2, computed_v1, action_root
        ),
    })
}
