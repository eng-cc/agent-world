use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use agent_world::runtime::WorldError;
use agent_world_proto::distributed as proto_distributed;

use super::distributed_net::DistributedNetwork;
use super::util::to_canonical_cbor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmitActionReceipt {
    pub action_id: String,
    pub accepted_at_ms: i64,
}

pub trait ActionGateway {
    fn submit_action(
        &self,
        action: proto_distributed::ActionEnvelope,
    ) -> Result<SubmitActionReceipt, WorldError>;
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
    fn submit_action(
        &self,
        action: proto_distributed::ActionEnvelope,
    ) -> Result<SubmitActionReceipt, WorldError> {
        let topic = proto_distributed::topic_action(&action.world_id);
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
