//! Network-focused facade for distributed runtime capabilities.

mod bootstrap;
mod client;
mod dht;
mod dht_cache;
mod execution_storage;
mod gateway;
mod head_follow;
mod head_validation;
mod index;
mod index_store;
mod network;
mod observer;
mod observer_replay;
mod provider_cache;
mod util;

#[cfg(feature = "libp2p")]
mod libp2p_net;

pub use agent_world::runtime::{ExecutionWriteConfig, ExecutionWriteResult, HeadValidationResult};
use agent_world_proto::distributed_dht as proto_dht;
use agent_world_proto::distributed_net as proto_net;
pub use bootstrap::{
    bootstrap_world_from_dht, bootstrap_world_from_head, bootstrap_world_from_head_with_dht,
};
pub use client::DistributedClient;
pub use dht::{DistributedDht, InMemoryDht};
pub use dht_cache::{CachedDht, DhtCacheConfig};
pub use execution_storage::store_execution_result;
pub use gateway::{ActionGateway, NetworkGateway, SubmitActionReceipt};
pub use head_follow::{HeadFollower, HeadUpdateDecision};
pub use head_validation::{assemble_journal, assemble_snapshot, validate_head_update};
pub use index::{
    publish_execution_providers, publish_execution_providers_cached, publish_world_head,
    query_providers, IndexPublishResult,
};
pub use index_store::{DistributedIndexStore, HeadIndexRecord, InMemoryIndexStore};
pub use network::{DistributedNetwork, InMemoryNetwork};
pub use observer::{
    HeadFollowReport, HeadSyncReport, HeadSyncResult, ObserverClient, ObserverSubscription,
};
pub use observer_replay::{
    replay_validate_head, replay_validate_head_with_dht, replay_validate_with_head,
    replay_validate_with_head_and_dht,
};
pub use proto_dht::{MembershipDirectorySnapshot, ProviderRecord};
pub use proto_net::{NetworkMessage, NetworkRequest, NetworkResponse, NetworkSubscription};
pub use provider_cache::{ProviderCache, ProviderCacheConfig};

#[cfg(feature = "libp2p")]
pub use libp2p_net::{Libp2pNetwork, Libp2pNetworkConfig};

#[cfg(test)]
mod tests;
