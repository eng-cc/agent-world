#[derive(Debug, Clone)]
pub struct HeadValidationResult {
    pub block_hash: String,
    pub snapshot: super::snapshot::Snapshot,
    pub journal: super::snapshot::Journal,
}

use serde::Serialize;

use super::blob_store::{blake3_hex, BlobStore};
use super::distributed::{SnapshotManifest, WorldBlock, WorldHeadAnnounce};
use super::error::WorldError;
use super::events::CausedBy;
use super::segmenter::JournalSegmentRef;
use super::snapshot::{Journal, Snapshot};
use super::types::ActionId;
use super::util::to_canonical_cbor;
use super::world::World;
use super::world_event::{WorldEvent, WorldEventBody};

pub fn validate_head_update(
    head: &WorldHeadAnnounce,
    block: &WorldBlock,
    snapshot_manifest: &SnapshotManifest,
    journal_segments: &[JournalSegmentRef],
    store: &impl BlobStore,
) -> Result<HeadValidationResult, WorldError> {
    if head.world_id != block.world_id {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "world_id mismatch: head={}, block={}",
                head.world_id, block.world_id
            ),
        });
    }
    if head.height != block.height {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "height mismatch: head={}, block={}",
                head.height, block.height
            ),
        });
    }
    if head.state_root != block.state_root {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "state_root mismatch: head={}, block={}",
                head.state_root, block.state_root
            ),
        });
    }

    let manifest_hash = hash_cbor(snapshot_manifest)?;
    if block.snapshot_ref != manifest_hash {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "snapshot_ref mismatch: block={}, manifest={}",
                block.snapshot_ref, manifest_hash
            ),
        });
    }
    let journal_ref_hash = hash_cbor(&journal_segments)?;
    if block.journal_ref != journal_ref_hash {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "journal_ref mismatch: block={}, segments={}",
                block.journal_ref, journal_ref_hash
            ),
        });
    }

    let snapshot = assemble_snapshot(snapshot_manifest, store)?;
    let journal = assemble_journal(journal_segments, store)?;
    if snapshot.journal_len != journal.len() {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "journal length mismatch: snapshot={}, journal={}",
                snapshot.journal_len,
                journal.len()
            ),
        });
    }

    let action_root = hash_actions(&journal)?;
    if block.action_root != action_root {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "action_root mismatch: block={}, computed={}",
                block.action_root, action_root
            ),
        });
    }
    let event_root = hash_events(&journal)?;
    if block.event_root != event_root {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "event_root mismatch: block={}, computed={}",
                block.event_root, event_root
            ),
        });
    }
    let receipts_root = hash_receipts(&journal)?;
    if block.receipts_root != receipts_root {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "receipts_root mismatch: block={}, computed={}",
                block.receipts_root, receipts_root
            ),
        });
    }

    let block_hash = hash_cbor(block)?;
    if head.block_hash != block_hash {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "block_hash mismatch: head={}, computed={}",
                head.block_hash, block_hash
            ),
        });
    }

    World::from_snapshot(snapshot.clone(), journal.clone())?;

    Ok(HeadValidationResult {
        block_hash,
        snapshot,
        journal,
    })
}

pub fn assemble_snapshot(
    manifest: &SnapshotManifest,
    store: &impl BlobStore,
) -> Result<Snapshot, WorldError> {
    let mut bytes = Vec::new();
    for chunk in &manifest.chunks {
        let chunk_bytes = store.get(&chunk.content_hash)?;
        let actual_hash = blake3_hex(&chunk_bytes);
        if actual_hash != chunk.content_hash {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "snapshot chunk hash mismatch: expected={}, actual={}",
                    chunk.content_hash, actual_hash
                ),
            });
        }
        bytes.extend_from_slice(&chunk_bytes);
    }

    let actual_root = blake3_hex(&bytes);
    if actual_root != manifest.state_root {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "snapshot state_root mismatch: expected={}, actual={}",
                manifest.state_root, actual_root
            ),
        });
    }

    Ok(serde_cbor::from_slice(&bytes)?)
}

pub fn assemble_journal(
    segments: &[JournalSegmentRef],
    store: &impl BlobStore,
) -> Result<Journal, WorldError> {
    let mut journal = Journal::new();
    let mut expected_next: Option<u64> = None;

    for segment in segments {
        let segment_bytes = store.get(&segment.content_hash)?;
        let actual_hash = blake3_hex(&segment_bytes);
        if actual_hash != segment.content_hash {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "journal segment hash mismatch: expected={}, actual={}",
                    segment.content_hash, actual_hash
                ),
            });
        }
        let events: Vec<WorldEvent> = serde_cbor::from_slice(&segment_bytes)?;
        let (first, last) = match (events.first(), events.last()) {
            (Some(first), Some(last)) => (first, last),
            _ => {
                return Err(WorldError::DistributedValidationFailed {
                    reason: "journal segment empty".to_string(),
                });
            }
        };
        if first.id != segment.from_event_id || last.id != segment.to_event_id {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "journal segment range mismatch: segment={}..{}, events={}..{}",
                    segment.from_event_id, segment.to_event_id, first.id, last.id
                ),
            });
        }
        if let Some(expected) = expected_next {
            if first.id != expected {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!(
                        "journal discontinuity: expected={}, got={}",
                        expected, first.id
                    ),
                });
            }
        }
        expected_next = last.id.checked_add(1);
        for event in events {
            journal.append(event);
        }
    }

    Ok(journal)
}

fn hash_actions(journal: &Journal) -> Result<String, WorldError> {
    let actions: Vec<ActionId> = journal
        .events
        .iter()
        .filter_map(|event| match event.caused_by {
            Some(CausedBy::Action(action_id)) => Some(action_id),
            _ => None,
        })
        .collect();
    hash_cbor(&actions)
}

fn hash_events(journal: &Journal) -> Result<String, WorldError> {
    hash_cbor(&journal.events)
}

fn hash_receipts(journal: &Journal) -> Result<String, WorldError> {
    let receipts = journal
        .events
        .iter()
        .filter_map(|event| match &event.body {
            WorldEventBody::ReceiptAppended(receipt) => Some(receipt.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    hash_cbor(&receipts)
}

fn hash_cbor<T: Serialize>(value: &T) -> Result<String, WorldError> {
    let bytes = to_canonical_cbor(value)?;
    Ok(blake3_hex(&bytes))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::super::{Action, LocalCasStore, World};
    use super::*;
    use crate::GeoPos;

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("duration since epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-world-runtime-{prefix}-{unique}"))
    }

    #[test]
    fn validate_head_update_accepts_written_block() {
        let dir = temp_dir("head-validate");
        let store = LocalCasStore::new(&dir);
        let mut world = World::new();

        world.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            pos: GeoPos::new(0.0, 0.0, 0.0),
        });
        world.step().expect("step world");

        let snapshot = world.snapshot();
        let journal = world.journal().clone();
        let write = super::super::distributed_storage::store_execution_result(
            "w1",
            1,
            "genesis",
            "node-1",
            1,
            &snapshot,
            &journal,
            &store,
            super::super::distributed_storage::ExecutionWriteConfig::default(),
        )
        .expect("write");

        let result = validate_head_update(
            &write.head_announce,
            &write.block,
            &write.snapshot_manifest,
            &write.journal_segments,
            &store,
        )
        .expect("validate");

        assert_eq!(result.block_hash, write.block_hash);
        assert_eq!(result.snapshot.journal_len, snapshot.journal_len);
        assert_eq!(result.journal.len(), journal.len());

        let _ = fs::remove_dir_all(&dir);
    }
}
