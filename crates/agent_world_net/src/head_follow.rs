use super::distributed::WorldHeadAnnounce;
use super::distributed_bootstrap::{bootstrap_world_from_head, bootstrap_world_from_head_with_dht};
use super::distributed_client::DistributedClient;
use super::distributed_dht::DistributedDht;
use super::error::WorldError;
use super::head_tracking::{HeadTracker, HeadUpdateDecision};
use agent_world::runtime::World;
use agent_world_distfs::BlobStore;

#[derive(Debug, Clone)]
pub struct HeadFollower {
    tracker: HeadTracker,
}

impl HeadFollower {
    pub fn new(world_id: impl Into<String>) -> Self {
        Self {
            tracker: HeadTracker::new(world_id),
        }
    }

    pub fn world_id(&self) -> &str {
        self.tracker.world_id()
    }

    pub fn current_head(&self) -> Option<&WorldHeadAnnounce> {
        self.tracker.current_head()
    }

    pub fn select_best_head(&self, heads: &[WorldHeadAnnounce]) -> Option<WorldHeadAnnounce> {
        self.tracker.select_best_head(heads)
    }

    pub fn apply_head(
        &mut self,
        head: &WorldHeadAnnounce,
        client: &DistributedClient,
        store: &impl BlobStore,
    ) -> Result<Option<World>, WorldError> {
        match self.tracker.decide_head(head)? {
            HeadUpdateDecision::Apply => {
                let world = bootstrap_world_from_head(head, client, store)?;
                self.tracker.record_applied(head);
                Ok(Some(world))
            }
            HeadUpdateDecision::IgnoreDuplicate | HeadUpdateDecision::IgnoreStale => Ok(None),
        }
    }

    pub fn apply_head_with_dht(
        &mut self,
        head: &WorldHeadAnnounce,
        dht: &impl DistributedDht,
        client: &DistributedClient,
        store: &impl BlobStore,
    ) -> Result<Option<World>, WorldError> {
        match self.tracker.decide_head(head)? {
            HeadUpdateDecision::Apply => {
                let world = bootstrap_world_from_head_with_dht(head, dht, client, store)?;
                self.tracker.record_applied(head);
                Ok(Some(world))
            }
            HeadUpdateDecision::IgnoreDuplicate | HeadUpdateDecision::IgnoreStale => Ok(None),
        }
    }

    pub fn sync_from_heads(
        &mut self,
        heads: &[WorldHeadAnnounce],
        client: &DistributedClient,
        store: &impl BlobStore,
    ) -> Result<Option<World>, WorldError> {
        let Some(best) = self.select_best_head(heads) else {
            return Ok(None);
        };
        self.apply_head(&best, client, store)
    }

    pub fn sync_from_heads_with_dht(
        &mut self,
        heads: &[WorldHeadAnnounce],
        dht: &impl DistributedDht,
        client: &DistributedClient,
        store: &impl BlobStore,
    ) -> Result<Option<World>, WorldError> {
        let Some(best) = self.select_best_head(heads) else {
            return Ok(None);
        };
        self.apply_head_with_dht(&best, dht, client, store)
    }
}
