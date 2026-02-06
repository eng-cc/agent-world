//! Gateway API for submitting actions to the network.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use super::distributed::{topic_action, ActionEnvelope};
use super::distributed_net::DistributedNetwork;
use super::error::WorldError;
use super::util::to_canonical_cbor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmitActionReceipt {
    pub action_id: String,
    pub accepted_at_ms: i64,
}

pub trait ActionGateway {
    fn submit_action(&self, action: ActionEnvelope) -> Result<SubmitActionReceipt, WorldError>;
}

#[derive(Clone)]
pub struct NetworkGateway {
    network: Arc<dyn DistributedNetwork + Send + Sync>,
    now_fn: Arc<dyn Fn() -> i64 + Send + Sync>,
}

impl NetworkGateway {
    pub fn new(network: Arc<dyn DistributedNetwork + Send + Sync>) -> Self {
        Self {
            network,
            now_fn: Arc::new(now_ms),
        }
    }

    pub fn new_with_clock(
        network: Arc<dyn DistributedNetwork + Send + Sync>,
        now_fn: Arc<dyn Fn() -> i64 + Send + Sync>,
    ) -> Self {
        Self { network, now_fn }
    }
}

impl ActionGateway for NetworkGateway {
    fn submit_action(&self, action: ActionEnvelope) -> Result<SubmitActionReceipt, WorldError> {
        let topic = topic_action(&action.world_id);
        let payload = to_canonical_cbor(&action)?;
        self.network.publish(&topic, &payload)?;
        Ok(SubmitActionReceipt {
            action_id: action.action_id,
            accepted_at_ms: (self.now_fn)(),
        })
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::super::distributed_net::InMemoryNetwork;
    use super::*;

    fn sample_action() -> ActionEnvelope {
        ActionEnvelope {
            world_id: "w1".to_string(),
            action_id: "a1".to_string(),
            actor_id: "actor-1".to_string(),
            action_kind: "test".to_string(),
            payload_cbor: vec![1, 2, 3],
            payload_hash: "hash".to_string(),
            nonce: 1,
            timestamp_ms: 10,
            signature: "sig".to_string(),
        }
    }

    #[test]
    fn gateway_publishes_action() {
        let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
        let subscription = network.subscribe("aw.w1.action").expect("subscribe");
        let gateway = NetworkGateway::new_with_clock(Arc::clone(&network), Arc::new(|| 1234));

        let receipt = gateway.submit_action(sample_action()).expect("submit");
        assert_eq!(receipt.action_id, "a1");
        assert_eq!(receipt.accepted_at_ms, 1234);

        let messages = subscription.drain();
        assert_eq!(messages.len(), 1);
        let decoded: ActionEnvelope = serde_cbor::from_slice(&messages[0]).expect("decode");
        assert_eq!(decoded.action_id, "a1");
    }
}
