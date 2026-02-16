use std::sync::Arc;

use super::distributed::{topic_event, topic_head, WorldHeadAnnounce};
use super::distributed_client::DistributedClient;
use super::distributed_dht::DistributedDht;
use super::distributed_head_follow::HeadFollower;
use super::distributed_net::{DistributedNetwork, NetworkSubscription};
use super::error::WorldError;
use super::head_sync::{
    compose_head_sync_report, follow_head_sync, HeadFollowReport as GenericHeadFollowReport,
    HeadSyncReport as GenericHeadSyncReport, HeadSyncResult as GenericHeadSyncResult,
};
use agent_world::runtime::World;
use agent_world_distfs::{BlobStore, FileStore};

#[derive(Debug, Clone)]
pub struct ObserverSubscription {
    pub event_sub: NetworkSubscription,
    pub head_sub: NetworkSubscription,
}

pub type HeadSyncResult = GenericHeadSyncResult<World>;
pub type HeadSyncReport = GenericHeadSyncReport<World>;
pub type HeadFollowReport = GenericHeadFollowReport<World>;

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

    pub fn sync_heads_with_path_index(
        &self,
        subscription: &ObserverSubscription,
        follower: &mut HeadFollower,
        store: &(impl BlobStore + FileStore),
    ) -> Result<Option<World>, WorldError> {
        let heads = self.drain_heads(subscription)?;
        follower.sync_from_heads_with_path_index(&heads, store)
    }

    pub fn sync_heads_with_path_index_report(
        &self,
        subscription: &ObserverSubscription,
        follower: &mut HeadFollower,
        store: &(impl BlobStore + FileStore),
    ) -> Result<HeadSyncReport, WorldError> {
        let heads = self.drain_heads(subscription)?;
        let drained = heads.len();
        let world = follower.sync_from_heads_with_path_index(&heads, store)?;
        compose_head_sync_report(drained, world, follower.current_head().cloned(), || {
            WorldError::DistributedValidationFailed {
                reason: "head follower did not record applied head".to_string(),
            }
        })
    }

    pub fn sync_heads_with_path_index_result(
        &self,
        subscription: &ObserverSubscription,
        follower: &mut HeadFollower,
        store: &(impl BlobStore + FileStore),
    ) -> Result<Option<HeadSyncResult>, WorldError> {
        let world = self.sync_heads_with_path_index(subscription, follower, store)?;
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

    pub fn follow_heads_with_path_index(
        &self,
        subscription: &ObserverSubscription,
        follower: &mut HeadFollower,
        store: &(impl BlobStore + FileStore),
        max_rounds: usize,
    ) -> Result<HeadFollowReport, WorldError> {
        follow_head_sync(max_rounds, || {
            self.sync_heads_with_path_index_report(subscription, follower, store)
        })
    }
}

#[cfg(all(test, feature = "self_tests"))]
mod tests {
    use std::fs;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    use agent_world::runtime::{Action, World};
    use agent_world::GeoPos;
    use agent_world_distfs::LocalCasStore;

    use super::super::distributed_head_follow::HeadFollower;
    use super::super::distributed_net::InMemoryNetwork;
    use super::super::distributed_storage::{
        store_execution_result_with_path_index, ExecutionWriteConfig,
    };
    use super::super::util::to_canonical_cbor;
    use super::*;

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("duration since epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-world-net-{prefix}-{unique}"))
    }

    #[test]
    fn observer_subscribes_and_drains_head_updates() {
        let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
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

    #[test]
    fn observer_sync_heads_with_path_index_applies_world() {
        let dir = temp_dir("observer-path-index-sync");
        let store = LocalCasStore::new(&dir);

        let mut world = World::new();
        world.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            pos: GeoPos::new(0.0, 0.0, 0.0),
        });
        world.step().expect("step world");

        let snapshot = world.snapshot();
        let journal = world.journal().clone();
        let write = store_execution_result_with_path_index(
            "w1",
            1,
            "genesis",
            "exec-1",
            1,
            &snapshot,
            &journal,
            &store,
            ExecutionWriteConfig::default(),
        )
        .expect("write");

        let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
        let observer = ObserverClient::new(Arc::clone(&network));
        let subscription = observer.subscribe("w1").expect("subscribe");
        let payload = to_canonical_cbor(&write.head_announce).expect("head cbor");
        network
            .publish(&topic_head("w1"), &payload)
            .expect("publish");

        let mut follower = HeadFollower::new("w1");
        let result = observer
            .sync_heads_with_path_index(&subscription, &mut follower, &store)
            .expect("sync");
        let applied = result.expect("applied world");
        assert_eq!(applied.journal().len(), journal.len());
        assert_eq!(follower.current_head(), Some(&write.head_announce));

        let _ = fs::remove_dir_all(&dir);
    }
}
