use super::distributed::WorldHeadAnnounce;
use super::distributed_dht::{DistributedDht, ProviderRecord};
use super::distributed_provider_cache::ProviderCache;
use super::distributed_storage::ExecutionWriteResult;
use super::error::WorldError;

pub use agent_world_net::IndexPublishResult;

pub fn publish_world_head(
    dht: &impl DistributedDht,
    head: &WorldHeadAnnounce,
) -> Result<(), WorldError> {
    agent_world_net::publish_world_head(dht, head).map_err(WorldError::from)
}

pub fn publish_execution_providers(
    dht: &impl DistributedDht,
    world_id: &str,
    provider_id: &str,
    result: &ExecutionWriteResult,
) -> Result<IndexPublishResult, WorldError> {
    agent_world_net::publish_execution_providers(dht, world_id, provider_id, result)
        .map_err(WorldError::from)
}

pub fn publish_execution_providers_cached(
    cache: &ProviderCache,
    world_id: &str,
    result: &ExecutionWriteResult,
) -> Result<IndexPublishResult, WorldError> {
    agent_world_net::publish_execution_providers_cached(cache, world_id, result)
        .map_err(WorldError::from)
}

pub fn query_providers(
    dht: &impl DistributedDht,
    world_id: &str,
    content_hash: &str,
) -> Result<Vec<ProviderRecord>, WorldError> {
    agent_world_net::query_providers(dht, world_id, content_hash).map_err(WorldError::from)
}
