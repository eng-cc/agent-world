//! Execution node bootstrap helpers for distributed runtime.

use super::blob_store::BlobStore;
use super::distributed::WorldHeadAnnounce;
use super::distributed_client::DistributedClient;
use super::distributed_dht::DistributedDht;
use super::distributed_observer_replay::{
    replay_validate_with_head, replay_validate_with_head_and_dht,
};
use super::error::WorldError;
use super::world::World;

pub fn bootstrap_world_from_head(
    head: &WorldHeadAnnounce,
    client: &DistributedClient,
    store: &impl BlobStore,
) -> Result<World, WorldError> {
    let result = replay_validate_with_head(head, client, store)?;
    World::from_snapshot(result.snapshot, result.journal)
}

pub fn bootstrap_world_from_head_with_dht(
    head: &WorldHeadAnnounce,
    dht: &impl DistributedDht,
    client: &DistributedClient,
    store: &impl BlobStore,
) -> Result<World, WorldError> {
    let result = replay_validate_with_head_and_dht(head, dht, client, store)?;
    World::from_snapshot(result.snapshot, result.journal)
}

pub fn bootstrap_world_from_dht(
    world_id: &str,
    dht: &impl DistributedDht,
    client: &DistributedClient,
    store: &impl BlobStore,
) -> Result<World, WorldError> {
    let head = dht
        .get_world_head(world_id)?
        .ok_or_else(|| WorldError::DistributedValidationFailed {
            reason: format!("world head not found for {world_id}"),
        })?;
    bootstrap_world_from_head_with_dht(&head, dht, client, store)
}
