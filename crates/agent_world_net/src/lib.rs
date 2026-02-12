//! Network-focused facade for distributed runtime capabilities.

pub use agent_world::runtime::{
    publish_execution_providers, publish_execution_providers_cached, publish_world_head,
    query_providers, ActionGateway, CachedDht, DhtCacheConfig, DistributedClient, DistributedDht,
    DistributedIndexStore, DistributedNetwork, HeadFollowReport, HeadFollower, HeadIndexRecord,
    HeadSyncReport, HeadSyncResult, HeadUpdateDecision, InMemoryDht, InMemoryIndexStore,
    InMemoryNetwork, IndexPublishResult, MembershipDirectorySnapshot, NetworkGateway,
    NetworkMessage, NetworkRequest, NetworkResponse, NetworkSubscription, ObserverClient,
    ObserverSubscription, ProviderCache, ProviderCacheConfig, ProviderRecord, SubmitActionReceipt,
};

#[cfg(feature = "libp2p")]
pub use agent_world::runtime::{Libp2pNetwork, Libp2pNetworkConfig};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn net_exports_are_available() {
        let _ = std::any::type_name::<NetworkMessage>();
        let _ = std::any::type_name::<DistributedClient>();
        let _ = std::any::type_name::<HeadFollower>();
    }
}
