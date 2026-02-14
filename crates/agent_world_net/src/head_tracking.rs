use std::cmp::Ordering;

use super::distributed::WorldHeadAnnounce;
use super::error::WorldError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeadUpdateDecision {
    Apply,
    IgnoreDuplicate,
    IgnoreStale,
}

#[derive(Debug, Clone)]
pub struct HeadTracker {
    world_id: String,
    current_head: Option<WorldHeadAnnounce>,
}

impl HeadTracker {
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

    pub fn decide_head(&self, head: &WorldHeadAnnounce) -> Result<HeadUpdateDecision, WorldError> {
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

    pub fn record_applied(&mut self, head: &WorldHeadAnnounce) {
        self.current_head = Some(head.clone());
    }
}

fn compare_heads(a: &WorldHeadAnnounce, b: &WorldHeadAnnounce) -> Ordering {
    a.height
        .cmp(&b.height)
        .then_with(|| a.timestamp_ms.cmp(&b.timestamp_ms))
        .then_with(|| a.block_hash.cmp(&b.block_hash))
}
