use std::cmp::Ordering;

use agent_world::runtime::{BlobStore, World, WorldError};
use agent_world_proto::distributed::WorldHeadAnnounce;

use crate::bootstrap::{bootstrap_world_from_head, bootstrap_world_from_head_with_dht};
use crate::{DistributedClient, DistributedDht};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadUpdateDecision {
    Apply,
    IgnoreDuplicate,
    IgnoreStale,
}

#[derive(Debug, Clone)]
pub struct HeadFollower {
    world_id: String,
    current_head: Option<WorldHeadAnnounce>,
}

impl HeadFollower {
    pub fn new(world_id: impl Into<String>) -> Self {
        Self {
            world_id: world_id.into(),
            current_head: None,
        }
    }

    pub fn world_id(&self) -> &str {
        &self.world_id
    }

    pub fn current_head(&self) -> Option<&WorldHeadAnnounce> {
        self.current_head.as_ref()
    }

    pub fn select_best_head(&self, heads: &[WorldHeadAnnounce]) -> Option<WorldHeadAnnounce> {
        heads
            .iter()
            .filter(|head| head.world_id == self.world_id)
            .cloned()
            .max_by(compare_heads)
    }

    pub fn apply_head(
        &mut self,
        head: &WorldHeadAnnounce,
        client: &DistributedClient,
        store: &impl BlobStore,
    ) -> Result<Option<World>, WorldError> {
        match self.decide_head(head)? {
            HeadUpdateDecision::Apply => {
                let world = bootstrap_world_from_head(head, client, store)?;
                self.current_head = Some(head.clone());
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
        match self.decide_head(head)? {
            HeadUpdateDecision::Apply => {
                let world = bootstrap_world_from_head_with_dht(head, dht, client, store)?;
                self.current_head = Some(head.clone());
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

    fn decide_head(&self, head: &WorldHeadAnnounce) -> Result<HeadUpdateDecision, WorldError> {
        if head.world_id != self.world_id {
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "head world_id mismatch: expected={}, got={}",
                    self.world_id, head.world_id
                ),
            });
        }
        let Some(current) = self.current_head.as_ref() else {
            return Ok(HeadUpdateDecision::Apply);
        };
        if head.height < current.height {
            return Ok(HeadUpdateDecision::IgnoreStale);
        }
        if head.height == current.height {
            if head.block_hash == current.block_hash {
                return Ok(HeadUpdateDecision::IgnoreDuplicate);
            }
            return Err(WorldError::DistributedValidationFailed {
                reason: format!(
                    "head conflict at height {}: current={}, new={}",
                    head.height, current.block_hash, head.block_hash
                ),
            });
        }
        Ok(HeadUpdateDecision::Apply)
    }
}

fn compare_heads(a: &WorldHeadAnnounce, b: &WorldHeadAnnounce) -> Ordering {
    a.height
        .cmp(&b.height)
        .then_with(|| a.timestamp_ms.cmp(&b.timestamp_ms))
        .then_with(|| a.block_hash.cmp(&b.block_hash))
}
