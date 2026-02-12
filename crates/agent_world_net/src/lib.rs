//! Network-focused facade for distributed runtime capabilities.

mod bootstrap;
mod client;
mod dht;
mod dht_cache;
mod gateway;
mod head_follow;
mod index;
mod index_store;
mod network;
mod observer;
mod observer_replay;
mod provider_cache;
mod util;

use agent_world_proto::distributed_dht as proto_dht;
use agent_world_proto::distributed_net as proto_net;
pub use bootstrap::{
    bootstrap_world_from_dht, bootstrap_world_from_head, bootstrap_world_from_head_with_dht,
};
pub use client::DistributedClient;
pub use dht::{DistributedDht, InMemoryDht};
pub use dht_cache::{CachedDht, DhtCacheConfig};
pub use gateway::{ActionGateway, NetworkGateway, SubmitActionReceipt};
pub use head_follow::{HeadFollower, HeadUpdateDecision};
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
pub use agent_world::runtime::{Libp2pNetwork, Libp2pNetworkConfig};

#[cfg(test)]
mod tests;
