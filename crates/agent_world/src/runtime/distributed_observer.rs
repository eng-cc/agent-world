use std::sync::Arc;

use agent_world_net::observer_flow::{
    compose_head_sync_report, follow_head_sync, HeadFollowReport as NetHeadFollowReport,
    HeadSyncReport as NetHeadSyncReport, HeadSyncResult as NetHeadSyncResult,
};

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

pub type HeadSyncResult = NetHeadSyncResult<World>;
pub type HeadSyncReport = NetHeadSyncReport<World>;
pub type HeadFollowReport = NetHeadFollowReport<World>;

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
        Ok(ObserverSubscription {
            event_sub,
            head_sub,
        })
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

    pub fn sync_heads_report(
        &self,
        subscription: &ObserverSubscription,
        follower: &mut HeadFollower,
        client: &DistributedClient,
        store: &impl BlobStore,
    ) -> Result<HeadSyncReport, WorldError> {
        let heads = self.drain_heads(subscription)?;
        let drained = heads.len();
        let world = follower.sync_from_heads(&heads, client, store)?;
        compose_head_sync_report(drained, world, follower.current_head().cloned(), || {
            WorldError::DistributedValidationFailed {
                reason: "head follower did not record applied head".to_string(),
            }
        })
    }

    pub fn sync_heads_with_result(
        &self,
        subscription: &ObserverSubscription,
        follower: &mut HeadFollower,
        client: &DistributedClient,
        store: &impl BlobStore,
    ) -> Result<Option<HeadSyncResult>, WorldError> {
        let world = self.sync_heads(subscription, follower, client, store)?;
        match world {
            Some(world) => {
                let head = follower.current_head().cloned().ok_or_else(|| {
                    WorldError::DistributedValidationFailed {
                        reason: "head follower did not record applied head".to_string(),
                    }
                })?;
                Ok(Some(HeadSyncResult { head, world }))
            }
            None => Ok(None),
        }
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

    pub fn sync_heads_with_dht_report(
        &self,
        subscription: &ObserverSubscription,
        follower: &mut HeadFollower,
        dht: &impl DistributedDht,
        client: &DistributedClient,
        store: &impl BlobStore,
    ) -> Result<HeadSyncReport, WorldError> {
        let heads = self.drain_heads(subscription)?;
        let drained = heads.len();
        let world = follower.sync_from_heads_with_dht(&heads, dht, client, store)?;
        compose_head_sync_report(drained, world, follower.current_head().cloned(), || {
            WorldError::DistributedValidationFailed {
                reason: "head follower did not record applied head".to_string(),
            }
        })
    }

    pub fn sync_heads_with_dht_result(
        &self,
        subscription: &ObserverSubscription,
        follower: &mut HeadFollower,
        dht: &impl DistributedDht,
        client: &DistributedClient,
        store: &impl BlobStore,
    ) -> Result<Option<HeadSyncResult>, WorldError> {
        let world = self.sync_heads_with_dht(subscription, follower, dht, client, store)?;
        match world {
            Some(world) => {
                let head = follower.current_head().cloned().ok_or_else(|| {
                    WorldError::DistributedValidationFailed {
                        reason: "head follower did not record applied head".to_string(),
                    }
                })?;
                Ok(Some(HeadSyncResult { head, world }))
            }
            None => Ok(None),
        }
    }

    pub fn follow_heads(
        &self,
        subscription: &ObserverSubscription,
        follower: &mut HeadFollower,
        client: &DistributedClient,
        store: &impl BlobStore,
        max_rounds: usize,
    ) -> Result<HeadFollowReport, WorldError> {
        follow_head_sync(max_rounds, || {
            self.sync_heads_report(subscription, follower, client, store)
        })
    }

    pub fn follow_heads_with_dht(
        &self,
        subscription: &ObserverSubscription,
        follower: &mut HeadFollower,
        dht: &impl DistributedDht,
        client: &DistributedClient,
        store: &impl BlobStore,
        max_rounds: usize,
    ) -> Result<HeadFollowReport, WorldError> {
        follow_head_sync(max_rounds, || {
            self.sync_heads_with_dht_report(subscription, follower, dht, client, store)
        })
    }
}
