// Execution result storage helpers for distributed runtime (net crate facade).

use serde::Serialize;

use super::blob_store::{blake3_hex, BlobStore};
use super::distributed::{BlobRef, BlockAnnounce, WorldBlock, WorldHeadAnnounce};
use super::distributed_storage::{
    ExecutionWriteConfig as DistributedExecutionWriteConfig,
    ExecutionWriteResult as DistributedExecutionWriteResult,
};
use super::error::WorldError;
use super::events::CausedBy;
use super::segmenter::{segment_journal, segment_snapshot, SegmentConfig};
use super::snapshot::{Journal, Snapshot};
use super::types::ActionId;
use super::util::to_canonical_cbor;
use super::world_event::WorldEventBody;

pub fn store_execution_result(
    world_id: &str,
    height: u64,
    prev_block_hash: &str,
    proposer_id: &str,
    snapshot_epoch: u64,
    snapshot: &Snapshot,
    journal: &Journal,
    store: &impl BlobStore,
    config: DistributedExecutionWriteConfig,
) -> Result<DistributedExecutionWriteResult, WorldError> {
    let DistributedExecutionWriteConfig {
        segment: segment_config,
        codec,
    } = config;
    let segment_config: SegmentConfig = segment_config;
    let snapshot_manifest =
        segment_snapshot(snapshot, world_id, snapshot_epoch, store, segment_config)?;
    let snapshot_manifest_bytes = to_canonical_cbor(&snapshot_manifest)?;
    let snapshot_manifest_hash = store.put_bytes(&snapshot_manifest_bytes)?;
    let snapshot_manifest_ref = BlobRef {
        content_hash: snapshot_manifest_hash.clone(),
        size_bytes: snapshot_manifest_bytes.len() as u64,
        codec: codec.clone(),
        links: snapshot_manifest
            .chunks
            .iter()
            .map(|chunk| chunk.content_hash.clone())
            .collect(),
    };

    let journal_segments = segment_journal(journal, store, segment_config)?;
    let journal_segments_bytes = to_canonical_cbor(&journal_segments)?;
    let journal_segments_hash = store.put_bytes(&journal_segments_bytes)?;
    let journal_segments_ref = BlobRef {
        content_hash: journal_segments_hash.clone(),
        size_bytes: journal_segments_bytes.len() as u64,
        codec: codec.clone(),
        links: journal_segments
            .iter()
            .map(|segment| segment.content_hash.clone())
            .collect(),
    };

    let action_root = hash_actions(journal)?;
    let event_root = hash_events(journal)?;
    let receipts_root = hash_receipts(journal)?;
    let timestamp_ms = snapshot.state.time as i64;

    let block = WorldBlock {
        world_id: world_id.to_string(),
        height,
        prev_block_hash: prev_block_hash.to_string(),
        action_root,
        event_root: event_root.clone(),
        state_root: snapshot_manifest.state_root.clone(),
        journal_ref: journal_segments_hash.clone(),
        snapshot_ref: snapshot_manifest_hash.clone(),
        receipts_root,
        proposer_id: proposer_id.to_string(),
        timestamp_ms,
        signature: String::new(),
    };

    let block_bytes = to_canonical_cbor(&block)?;
    let block_hash = store.put_bytes(&block_bytes)?;
    let block_ref = BlobRef {
        content_hash: block_hash.clone(),
        size_bytes: block_bytes.len() as u64,
        codec: codec.clone(),
        links: vec![
            snapshot_manifest_hash.clone(),
            journal_segments_hash.clone(),
        ],
    };

    let block_announce = BlockAnnounce {
        world_id: world_id.to_string(),
        height,
        block_hash: block_hash.clone(),
        prev_block_hash: prev_block_hash.to_string(),
        state_root: snapshot_manifest.state_root.clone(),
        event_root,
        timestamp_ms,
        signature: String::new(),
    };

    let head_announce = WorldHeadAnnounce {
        world_id: world_id.to_string(),
        height,
        block_hash: block_hash.clone(),
        state_root: snapshot_manifest.state_root.clone(),
        timestamp_ms,
        signature: String::new(),
    };

    Ok(DistributedExecutionWriteResult {
        block,
        block_hash,
        block_ref,
        block_announce,
        head_announce,
        snapshot_manifest,
        snapshot_manifest_ref,
        journal_segments,
        journal_segments_ref,
    })
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

#[cfg(all(test, feature = "self_tests"))]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use agent_world::runtime::{Action, LocalCasStore, World};
    use agent_world::GeoPos;

    use super::super::distributed_storage::ExecutionWriteConfig;
    use super::*;

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("duration since epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-world-net-{prefix}-{unique}"))
    }

    #[test]
    fn store_execution_result_writes_block_and_refs() {
        let dir = temp_dir("exec-store");
        let store = LocalCasStore::new(&dir);
        let mut world = World::new();

        world.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            pos: GeoPos::new(0.0, 0.0, 0.0),
        });
        world.step().expect("step world");

        let snapshot = world.snapshot();
        let journal = world.journal().clone();

        let result = store_execution_result(
            "w1",
            1,
            "genesis",
            "exec-1",
            1,
            &snapshot,
            &journal,
            &store,
            ExecutionWriteConfig::default(),
        )
        .expect("store execution");

        assert!(store.has(&result.block_ref.content_hash).expect("block"));
        assert!(store
            .has(&result.snapshot_manifest_ref.content_hash)
            .expect("manifest"));
        assert!(store
            .has(&result.journal_segments_ref.content_hash)
            .expect("journal index"));
        assert_eq!(
            result.block.snapshot_ref,
            result.snapshot_manifest_ref.content_hash
        );
        assert_eq!(
            result.block.journal_ref,
            result.journal_segments_ref.content_hash
        );
        assert_eq!(result.block.world_id, "w1");

        let _ = fs::remove_dir_all(&dir);
    }
}
