use agent_world_proto::distributed::{ActionBatch, ActionEnvelope};

use super::error::WorldError;

pub use agent_world_consensus::{ActionBatchRules, ActionMempoolConfig};

#[derive(Debug, Default)]
pub struct ActionMempool {
    inner: agent_world_consensus::ActionMempool,
}

impl ActionMempool {
    pub fn new(config: ActionMempoolConfig) -> Self {
        Self {
            inner: agent_world_consensus::ActionMempool::new(config),
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn add_action(&mut self, action: ActionEnvelope) -> bool {
        self.inner.add_action(action)
    }

    pub fn remove_action(&mut self, action_id: &str) -> Option<ActionEnvelope> {
        self.inner.remove_action(action_id)
    }

    pub fn take_batch(
        &mut self,
        world_id: &str,
        proposer_id: &str,
        max_actions: usize,
        timestamp_ms: i64,
    ) -> Result<Option<ActionBatch>, WorldError> {
        Ok(self
            .inner
            .take_batch(world_id, proposer_id, max_actions, timestamp_ms)?)
    }

    pub fn take_batch_with_rules(
        &mut self,
        world_id: &str,
        proposer_id: &str,
        rules: ActionBatchRules,
        timestamp_ms: i64,
    ) -> Result<Option<ActionBatch>, WorldError> {
        Ok(self
            .inner
            .take_batch_with_rules(world_id, proposer_id, rules, timestamp_ms)?)
    }
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
        assert!(pool.remove_action("a2").is_some());
        assert!(pool.remove_action("a3").is_some());
        assert!(pool.remove_action("a1").is_none());
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
