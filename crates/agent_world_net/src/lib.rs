//! Network-focused facade for distributed runtime capabilities.

mod client;
mod dht;
mod dht_cache;
mod gateway;
mod index;
mod index_store;
mod network;
mod provider_cache;
mod util;

pub use agent_world::runtime::{
    HeadFollowReport, HeadFollower, HeadSyncReport, HeadSyncResult, HeadUpdateDecision,
    ObserverClient, ObserverSubscription,
};
use agent_world_proto::distributed_dht as proto_dht;
use agent_world_proto::distributed_net as proto_net;
pub use client::DistributedClient;
pub use dht::{DistributedDht, InMemoryDht};
pub use dht_cache::{CachedDht, DhtCacheConfig};
pub use gateway::{ActionGateway, NetworkGateway, SubmitActionReceipt};
pub use index::{
    publish_execution_providers, publish_execution_providers_cached, publish_world_head,
    query_providers, IndexPublishResult,
};
pub use index_store::{DistributedIndexStore, HeadIndexRecord, InMemoryIndexStore};
pub use network::{DistributedNetwork, InMemoryNetwork};
pub use proto_dht::{MembershipDirectorySnapshot, ProviderRecord};
pub use proto_net::{NetworkMessage, NetworkRequest, NetworkResponse, NetworkSubscription};
pub use provider_cache::{ProviderCache, ProviderCacheConfig};

#[cfg(feature = "libp2p")]
pub use agent_world::runtime::{Libp2pNetwork, Libp2pNetworkConfig};

#[cfg(test)]
mod tests;
