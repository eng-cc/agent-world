//! Network-focused facade for distributed runtime capabilities.

mod client;
mod dht;
mod dht_cache;
mod gateway;
mod head_follow;
mod index;
mod index_store;
mod network;
mod observer;
mod provider_cache;
mod util;

use agent_world_proto::distributed_dht as proto_dht;
use agent_world_proto::distributed_net as proto_net;
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
pub use proto_dht::{MembershipDirectorySnapshot, ProviderRecord};
pub use proto_net::{NetworkMessage, NetworkRequest, NetworkResponse, NetworkSubscription};
pub use provider_cache::{ProviderCache, ProviderCacheConfig};

#[cfg(feature = "libp2p")]
pub use agent_world::runtime::{Libp2pNetwork, Libp2pNetworkConfig};

#[cfg(test)]
mod tests;
