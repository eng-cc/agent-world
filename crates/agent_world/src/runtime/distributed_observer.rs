//! Observer utilities for event subscription and head tracking.

use std::sync::Arc;

use super::blob_store::BlobStore;
use super::distributed::{topic_event, topic_head, WorldHeadAnnounce};
use super::distributed_client::DistributedClient;
use super::distributed_dht::DistributedDht;
use super::distributed_head_follow::HeadFollower;
use super::distributed_net::{DistributedNetwork, NetworkSubscription};
use super::error::WorldError;
use super::world::World;

#[derive(Debug, Clone)]
pub struct ObserverSubscription {
    pub event_sub: NetworkSubscription,
    pub head_sub: NetworkSubscription,
}

#[derive(Clone)]
pub struct ObserverClient {
    network: Arc<dyn DistributedNetwork + Send + Sync>,
}

impl ObserverClient {
    pub fn new(network: Arc<dyn DistributedNetwork + Send + Sync>) -> Self {
        Self { network }
    }

    pub fn subscribe(&self, world_id: &str) -> Result<ObserverSubscription, WorldError> {
        let event_topic = topic_event(world_id);
        let head_topic = topic_head(world_id);
        let event_sub = self.network.subscribe(&event_topic)?;
        let head_sub = self.network.subscribe(&head_topic)?;
        Ok(ObserverSubscription { event_sub, head_sub })
    }

    pub fn drain_events(
        &self,
        subscription: &ObserverSubscription,
    ) -> Result<Vec<Vec<u8>>, WorldError> {
        Ok(subscription.event_sub.drain())
    }

    pub fn drain_heads(
        &self,
        subscription: &ObserverSubscription,
    ) -> Result<Vec<WorldHeadAnnounce>, WorldError> {
        let raw = subscription.head_sub.drain();
        let mut heads = Vec::new();
        for bytes in raw {
            heads.push(serde_cbor::from_slice(&bytes)?);
        }
        Ok(heads)
    }

    pub fn sync_heads(
        &self,
        subscription: &ObserverSubscription,
        follower: &mut HeadFollower,
        client: &DistributedClient,
        store: &impl BlobStore,
    ) -> Result<Option<World>, WorldError> {
        let heads = self.drain_heads(subscription)?;
        follower.sync_from_heads(&heads, client, store)
    }

    pub fn sync_heads_with_dht(
        &self,
        subscription: &ObserverSubscription,
        follower: &mut HeadFollower,
        dht: &impl DistributedDht,
        client: &DistributedClient,
        store: &impl BlobStore,
    ) -> Result<Option<World>, WorldError> {
        let heads = self.drain_heads(subscription)?;
        follower.sync_from_heads_with_dht(&heads, dht, client, store)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::distributed_net::InMemoryNetwork;
    use super::super::util::to_canonical_cbor;

    #[test]
    fn observer_subscribes_and_drains_head_updates() {
        let network: Arc<dyn DistributedNetwork + Send + Sync> =
            Arc::new(InMemoryNetwork::new());
        let observer = ObserverClient::new(Arc::clone(&network));
        let subscription = observer.subscribe("w1").expect("subscribe");

        let head = WorldHeadAnnounce {
            world_id: "w1".to_string(),
            height: 2,
            block_hash: "b1".to_string(),
            state_root: "s1".to_string(),
            timestamp_ms: 1,
            signature: "sig".to_string(),
        };
        let payload = to_canonical_cbor(&head).expect("cbor");
        network
            .publish(&topic_head("w1"), &payload)
            .expect("publish");

        let heads = observer.drain_heads(&subscription).expect("drain");
        assert_eq!(heads.len(), 1);
        assert_eq!(heads[0], head);
    }
}
