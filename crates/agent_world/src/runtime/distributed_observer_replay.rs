use agent_world_net::observer_replay_flow::load_manifest_and_segments;

use super::blob_store::{blake3_hex, BlobStore};
use super::distributed::WorldHeadAnnounce;
use super::distributed_client::DistributedClient;
use super::distributed_dht::DistributedDht;
use super::distributed_validation::{validate_head_update, HeadValidationResult};
use super::error::WorldError;

pub fn replay_validate_head(
    world_id: &str,
    client: &DistributedClient,
    store: &impl BlobStore,
) -> Result<HeadValidationResult, WorldError> {
    let head = client.get_world_head(world_id)?;
    replay_validate_with_head(&head, client, store)
}

#[allow(dead_code)]
pub fn replay_validate_head_with_dht(
    world_id: &str,
    dht: &impl DistributedDht,
    client: &DistributedClient,
    store: &impl BlobStore,
) -> Result<HeadValidationResult, WorldError> {
    let head =
        dht.get_world_head(world_id)?
            .ok_or_else(|| WorldError::DistributedValidationFailed {
                reason: format!("world head not found for {world_id}"),
            })?;
    replay_validate_with_head_and_dht(&head, dht, client, store)
}

pub fn replay_validate_with_head(
    head: &WorldHeadAnnounce,
    client: &DistributedClient,
    store: &impl BlobStore,
) -> Result<HeadValidationResult, WorldError> {
    let block_response = client.get_block_response(&head.world_id, head.height)?;
    let block = block_response.block;
    let (manifest, segments) = load_manifest_and_segments(
        &block_response.snapshot_ref,
        &block_response.journal_ref,
        |content_hash| client.fetch_blob(content_hash).map_err(WorldError::from),
        verify_blob_hash,
        |content_hash, bytes| store.put(content_hash, bytes).map_err(WorldError::from),
        WorldError::from,
    )?;
    validate_head_update(head, &block, &manifest, &segments, store)
}

pub fn replay_validate_with_head_and_dht(
    head: &WorldHeadAnnounce,
    dht: &impl DistributedDht,
    client: &DistributedClient,
    store: &impl BlobStore,
) -> Result<HeadValidationResult, WorldError> {
    let block_response = client.get_block_response(&head.world_id, head.height)?;
    let block = block_response.block;
    let (manifest, segments) = load_manifest_and_segments(
        &block_response.snapshot_ref,
        &block_response.journal_ref,
        |content_hash| {
            client
                .fetch_blob_from_dht(&head.world_id, content_hash, dht)
                .map_err(WorldError::from)
        },
        verify_blob_hash,
        |content_hash, bytes| store.put(content_hash, bytes).map_err(WorldError::from),
        WorldError::from,
    )?;
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
