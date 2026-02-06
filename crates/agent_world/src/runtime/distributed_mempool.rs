//! Action mempool aggregation and deduplication.

use std::collections::{HashMap, VecDeque};

use super::blob_store::blake3_hex;
use super::distributed::{ActionBatch, ActionEnvelope};
use super::error::WorldError;
use super::util::to_canonical_cbor;

#[derive(Debug, Clone)]
pub struct ActionMempoolConfig {
    pub max_actions: usize,
    pub max_per_actor: usize,
}

impl Default for ActionMempoolConfig {
    fn default() -> Self {
        Self {
            max_actions: 10_000,
            max_per_actor: 256,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionBatchRules {
    pub max_actions: usize,
    pub max_payload_bytes: usize,
}

impl Default for ActionBatchRules {
    fn default() -> Self {
        Self {
            max_actions: 512,
            max_payload_bytes: 512 * 1024,
        }
    }
}

#[derive(Debug, Default)]
pub struct ActionMempool {
    config: ActionMempoolConfig,
    actions: HashMap<String, ActionEnvelope>,
    per_actor: HashMap<String, Vec<String>>,
    arrival: VecDeque<String>,
}

impl ActionMempool {
    pub fn new(config: ActionMempoolConfig) -> Self {
        Self {
            config,
            actions: HashMap::new(),
            per_actor: HashMap::new(),
            arrival: VecDeque::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.actions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    pub fn add_action(&mut self, action: ActionEnvelope) -> bool {
        if self.actions.contains_key(&action.action_id) {
            return false;
        }

        let actor_count = self
            .per_actor
            .get(&action.actor_id)
            .map(|items| items.len())
            .unwrap_or(0);
        if actor_count >= self.config.max_per_actor {
            return false;
        }

        self.evict_if_needed();

        let actor_actions = self.per_actor.entry(action.actor_id.clone()).or_default();
        actor_actions.push(action.action_id.clone());
        self.arrival.push_back(action.action_id.clone());
        self.actions.insert(action.action_id.clone(), action);
        true
    }

    pub fn remove_action(&mut self, action_id: &str) -> Option<ActionEnvelope> {
        let action = self.actions.remove(action_id)?;
        self.arrival.retain(|id| id != action_id);
        if let Some(actor_actions) = self.per_actor.get_mut(&action.actor_id) {
            actor_actions.retain(|id| id != action_id);
            if actor_actions.is_empty() {
                self.per_actor.remove(&action.actor_id);
            }
        }
        Some(action)
    }

    pub fn take_batch(
        &mut self,
        world_id: &str,
        proposer_id: &str,
        max_actions: usize,
        timestamp_ms: i64,
    ) -> Result<Option<ActionBatch>, WorldError> {
        self.take_batch_with_rules(
            world_id,
            proposer_id,
            ActionBatchRules {
                max_actions,
                max_payload_bytes: usize::MAX,
            },
            timestamp_ms,
        )
    }

    pub fn take_batch_with_rules(
        &mut self,
        world_id: &str,
        proposer_id: &str,
        rules: ActionBatchRules,
        timestamp_ms: i64,
    ) -> Result<Option<ActionBatch>, WorldError> {
        if self.actions.is_empty() {
            return Ok(None);
        }
        let mut candidates: Vec<ActionEnvelope> = self.actions.values().cloned().collect();
        candidates.sort_by(|left, right| {
            left.timestamp_ms
                .cmp(&right.timestamp_ms)
                .then_with(|| left.action_id.cmp(&right.action_id))
        });

        let mut actions = Vec::new();
        let mut total_bytes = 0usize;

        for action in candidates {
            if actions.len() >= rules.max_actions {
                break;
            }
            let size_bytes = action_size_bytes(&action)?;
            if size_bytes > rules.max_payload_bytes {
                self.remove_action(&action.action_id);
                continue;
            }
            if total_bytes.saturating_add(size_bytes) > rules.max_payload_bytes {
                break;
            }
            total_bytes = total_bytes.saturating_add(size_bytes);
            actions.push(action);
        }

        if actions.is_empty() {
            return Ok(None);
        }

        for action in &actions {
            self.remove_action(&action.action_id);
        }

        let batch_id = batch_id_for_actions(&actions)?;
        Ok(Some(ActionBatch {
            world_id: world_id.to_string(),
            batch_id,
            actions,
            proposer_id: proposer_id.to_string(),
            timestamp_ms,
            signature: String::new(),
        }))
    }

    fn evict_if_needed(&mut self) {
        while self.actions.len() >= self.config.max_actions {
            let Some(evicted_id) = self.arrival.pop_front() else {
                break;
            };
            self.remove_action(&evicted_id);
        }
    }
}

fn batch_id_for_actions(actions: &[ActionEnvelope]) -> Result<String, WorldError> {
    let mut ids: Vec<String> = actions
        .iter()
        .map(|action| action.action_id.clone())
        .collect();
    ids.sort();
    let bytes = to_canonical_cbor(&ids)?;
    Ok(blake3_hex(&bytes))
}

fn action_size_bytes(action: &ActionEnvelope) -> Result<usize, WorldError> {
    let bytes = to_canonical_cbor(action)?;
    Ok(bytes.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action(id: &str, actor: &str, ts: i64) -> ActionEnvelope {
        ActionEnvelope {
            world_id: "w1".to_string(),
            action_id: id.to_string(),
            actor_id: actor.to_string(),
            action_kind: "test".to_string(),
            payload_cbor: Vec::new(),
            payload_hash: "hash".to_string(),
            nonce: 1,
            timestamp_ms: ts,
            signature: String::new(),
        }
    }

    #[test]
    fn mempool_dedups_by_action_id() {
        let mut pool = ActionMempool::new(ActionMempoolConfig::default());
        assert!(pool.add_action(action("a1", "actor1", 1)));
        assert!(!pool.add_action(action("a1", "actor1", 2)));
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn mempool_respects_actor_limit() {
        let mut pool = ActionMempool::new(ActionMempoolConfig {
            max_actions: 10,
            max_per_actor: 1,
        });
        assert!(pool.add_action(action("a1", "actor1", 1)));
        assert!(!pool.add_action(action("a2", "actor1", 2)));
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn mempool_evicts_oldest_when_full() {
        let mut pool = ActionMempool::new(ActionMempoolConfig {
            max_actions: 2,
            max_per_actor: 10,
        });
        assert!(pool.add_action(action("a1", "actor1", 1)));
        assert!(pool.add_action(action("a2", "actor2", 2)));
        assert!(pool.add_action(action("a3", "actor3", 3)));
        assert_eq!(pool.len(), 2);
        assert!(pool.actions.contains_key("a2"));
        assert!(pool.actions.contains_key("a3"));
        assert!(!pool.actions.contains_key("a1"));
    }

    #[test]
    fn take_batch_orders_by_timestamp_then_id() {
        let mut pool = ActionMempool::new(ActionMempoolConfig::default());
        pool.add_action(action("a2", "actor1", 2));
        pool.add_action(action("a1", "actor2", 1));
        pool.add_action(action("a3", "actor3", 2));

        let batch = pool
            .take_batch("w1", "seq", 2, 10)
            .expect("batch result")
            .expect("batch");

        assert_eq!(batch.actions.len(), 2);
        assert_eq!(batch.actions[0].action_id, "a1");
        assert_eq!(batch.actions[1].action_id, "a2");
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn take_batch_respects_payload_limit() {
        let mut pool = ActionMempool::new(ActionMempoolConfig::default());
        let mut large = action("a1", "actor1", 1);
        large.payload_cbor = vec![0u8; 2048];
        let small = action("a2", "actor2", 2);

        pool.add_action(large);
        pool.add_action(small);

        let batch = pool
            .take_batch_with_rules(
                "w1",
                "seq",
                ActionBatchRules {
                    max_actions: 10,
                    max_payload_bytes: 512,
                },
                10,
            )
            .expect("batch result")
            .expect("batch");

        assert_eq!(batch.actions.len(), 1);
        assert_eq!(batch.actions[0].action_id, "a2");
    }
}
