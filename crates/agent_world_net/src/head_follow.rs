use std::cmp::Ordering;

use agent_world::runtime::{
    blake3_hex, validate_head_update, BlobStore, HeadValidationResult, JournalSegmentRef, World,
    WorldError,
};
use agent_world_proto::distributed::{SnapshotManifest, WorldHeadAnnounce};

use crate::{DistributedClient, DistributedDht};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadUpdateDecision {
    Apply,
    IgnoreDuplicate,
    IgnoreStale,
}

#[derive(Debug, Clone)]
pub struct HeadFollower {
    world_id: String,
    current_head: Option<WorldHeadAnnounce>,
}

impl HeadFollower {
    pub fn new(world_id: impl Into<String>) -> Self {
        Self {
            world_id: world_id.into(),
            current_head: None,
        }
    }

    pub fn world_id(&self) -> &str {
        &self.world_id
    }

    pub fn current_head(&self) -> Option<&WorldHeadAnnounce> {
        self.current_head.as_ref()
    }

    pub fn select_best_head(&self, heads: &[WorldHeadAnnounce]) -> Option<WorldHeadAnnounce> {
        heads
            .iter()
            .filter(|head| head.world_id == self.world_id)
            .cloned()
            .max_by(compare_heads)
    }

    pub fn apply_head(
        &mut self,
        head: &WorldHeadAnnounce,
        client: &DistributedClient,
        store: &impl BlobStore,
    ) -> Result<Option<World>, WorldError> {
        match self.decide_head(head)? {
            HeadUpdateDecision::Apply => {
                let world = bootstrap_world_from_head(head, client, store)?;
                self.current_head = Some(head.clone());
                Ok(Some(world))
            }
            HeadUpdateDecision::IgnoreDuplicate | HeadUpdateDecision::IgnoreStale => Ok(None),
        }
    }

    pub fn apply_head_with_dht(
        &mut self,
        head: &WorldHeadAnnounce,
        dht: &impl DistributedDht,
        client: &DistributedClient,
        store: &impl BlobStore,
    ) -> Result<Option<World>, WorldError> {
        match self.decide_head(head)? {
            HeadUpdateDecision::Apply => {
                let world = bootstrap_world_from_head_with_dht(head, dht, client, store)?;
                self.current_head = Some(head.clone());
                Ok(Some(world))
            }
            HeadUpdateDecision::IgnoreDuplicate | HeadUpdateDecision::IgnoreStale => Ok(None),
        }
    }

    pub fn sync_from_heads(
        &mut self,
        heads: &[WorldHeadAnnounce],
        client: &DistributedClient,
        store: &impl BlobStore,
    ) -> Result<Option<World>, WorldError> {
        let Some(best) = self.select_best_head(heads) else {
            return Ok(None);
        };
        self.apply_head(&best, client, store)
    }

    pub fn sync_from_heads_with_dht(
        &mut self,
        heads: &[WorldHeadAnnounce],
        dht: &impl DistributedDht,
        client: &DistributedClient,
        store: &impl BlobStore,
    ) -> Result<Option<World>, WorldError> {
        let Some(best) = self.select_best_head(heads) else {
            return Ok(None);
        };
        self.apply_head_with_dht(&best, dht, client, store)
    }

    fn decide_head(&self, head: &WorldHeadAnnounce) -> Result<HeadUpdateDecision, WorldError> {
        if head.world_id != self.world_id {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "head world_id mismatch: expected={}, got={}",
                    self.world_id, head.world_id
                ),
            });
        }
        let Some(current) = self.current_head.as_ref() else {
            return Ok(HeadUpdateDecision::Apply);
        };
        if head.height < current.height {
            return Ok(HeadUpdateDecision::IgnoreStale);
        }
        if head.height == current.height {
            if head.block_hash == current.block_hash {
                return Ok(HeadUpdateDecision::IgnoreDuplicate);
            }
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "head conflict at height {}: current={}, new={}",
                    head.height, current.block_hash, head.block_hash
                ),
            });
        }
        Ok(HeadUpdateDecision::Apply)
    }
}

fn compare_heads(a: &WorldHeadAnnounce, b: &WorldHeadAnnounce) -> Ordering {
    a.height
        .cmp(&b.height)
        .then_with(|| a.timestamp_ms.cmp(&b.timestamp_ms))
        .then_with(|| a.block_hash.cmp(&b.block_hash))
}

fn bootstrap_world_from_head(
    head: &WorldHeadAnnounce,
    client: &DistributedClient,
    store: &impl BlobStore,
) -> Result<World, WorldError> {
    let result = replay_validate_with_head(head, client, store)?;
    World::from_snapshot(result.snapshot, result.journal)
}

fn bootstrap_world_from_head_with_dht(
    head: &WorldHeadAnnounce,
    dht: &impl DistributedDht,
    client: &DistributedClient,
    store: &impl BlobStore,
) -> Result<World, WorldError> {
    let result = replay_validate_with_head_and_dht(head, dht, client, store)?;
    World::from_snapshot(result.snapshot, result.journal)
}

fn replay_validate_with_head(
    head: &WorldHeadAnnounce,
    client: &DistributedClient,
    store: &impl BlobStore,
) -> Result<HeadValidationResult, WorldError> {
    let block_response = client.get_block_response(&head.world_id, head.height)?;
    let block = block_response.block;

    let manifest_bytes = client.fetch_blob(&block_response.snapshot_ref)?;
    verify_blob_hash(&block_response.snapshot_ref, &manifest_bytes)?;
    let manifest: SnapshotManifest = serde_cbor::from_slice(&manifest_bytes)?;

    let segments_bytes = client.fetch_blob(&block_response.journal_ref)?;
    verify_blob_hash(&block_response.journal_ref, &segments_bytes)?;
    let segments: Vec<JournalSegmentRef> = serde_cbor::from_slice(&segments_bytes)?;

    for chunk in &manifest.chunks {
        let bytes = client.fetch_blob(&chunk.content_hash)?;
        store.put(&chunk.content_hash, &bytes)?;
    }
    for segment in &segments {
        let bytes = client.fetch_blob(&segment.content_hash)?;
        store.put(&segment.content_hash, &bytes)?;
    }

    validate_head_update(head, &block, &manifest, &segments, store)
}

fn replay_validate_with_head_and_dht(
    head: &WorldHeadAnnounce,
    dht: &impl DistributedDht,
    client: &DistributedClient,
    store: &impl BlobStore,
) -> Result<HeadValidationResult, WorldError> {
    let block_response = client.get_block_response(&head.world_id, head.height)?;
    let block = block_response.block;

    let manifest_bytes =
        client.fetch_blob_from_dht(&head.world_id, &block_response.snapshot_ref, dht)?;
    verify_blob_hash(&block_response.snapshot_ref, &manifest_bytes)?;
    let manifest: SnapshotManifest = serde_cbor::from_slice(&manifest_bytes)?;

    let segments_bytes =
        client.fetch_blob_from_dht(&head.world_id, &block_response.journal_ref, dht)?;
    verify_blob_hash(&block_response.journal_ref, &segments_bytes)?;
    let segments: Vec<JournalSegmentRef> = serde_cbor::from_slice(&segments_bytes)?;

    for chunk in &manifest.chunks {
        let bytes = client.fetch_blob_from_dht(&head.world_id, &chunk.content_hash, dht)?;
        store.put(&chunk.content_hash, &bytes)?;
    }
    for segment in &segments {
        let bytes = client.fetch_blob_from_dht(&head.world_id, &segment.content_hash, dht)?;
        store.put(&segment.content_hash, &bytes)?;
    }

    validate_head_update(head, &block, &manifest, &segments, store)
}

fn verify_blob_hash(expected: &str, bytes: &[u8]) -> Result<(), WorldError> {
    let actual = blake3_hex(bytes);
    if actual != expected {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!("blob hash mismatch: expected={expected}, actual={actual}"),
        });
    }
    Ok(())
}
