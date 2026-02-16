use std::fmt;
use std::sync::Arc;

use agent_world_proto::distributed_net::{DistributedNetwork, NetworkSubscription};
use agent_world_proto::world_error::WorldError;

use crate::replication::GossipReplicationMessage;
use crate::NodeError;

pub(crate) const DEFAULT_REPLICATION_TOPIC_PREFIX: &str = "aw";

pub(crate) fn default_replication_topic(world_id: &str) -> String {
    format!("{DEFAULT_REPLICATION_TOPIC_PREFIX}.{world_id}.replication")
}

#[derive(Clone)]
pub struct NodeReplicationNetworkHandle {
    network: Arc<dyn DistributedNetwork<WorldError> + Send + Sync>,
    topic: Option<String>,
}

impl fmt::Debug for NodeReplicationNetworkHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeReplicationNetworkHandle")
            .field("topic", &self.topic)
            .finish()
    }
}

impl NodeReplicationNetworkHandle {
    pub fn new(network: Arc<dyn DistributedNetwork<WorldError> + Send + Sync>) -> Self {
        Self {
            network,
            topic: None,
        }
    }

    pub fn with_topic(mut self, topic: impl Into<String>) -> Result<Self, NodeError> {
        let topic = topic.into();
        if topic.trim().is_empty() {
            return Err(NodeError::InvalidConfig {
                reason: "replication network topic cannot be empty".to_string(),
            });
        }
        self.topic = Some(topic);
        Ok(self)
    }

    fn resolved_topic(&self, world_id: &str) -> String {
        self.topic
            .clone()
            .unwrap_or_else(|| default_replication_topic(world_id))
    }
}

pub(crate) struct ReplicationNetworkEndpoint {
    network: Arc<dyn DistributedNetwork<WorldError> + Send + Sync>,
    topic: String,
    subscription: Option<NetworkSubscription>,
}

impl ReplicationNetworkEndpoint {
    pub(crate) fn new(
        handle: &NodeReplicationNetworkHandle,
        world_id: &str,
        subscribe: bool,
    ) -> Result<Self, NodeError> {
        let topic = handle.resolved_topic(world_id);
        let subscription = if subscribe {
            Some(
                handle
                    .network
                    .subscribe(topic.as_str())
                    .map_err(network_err)?,
            )
        } else {
            None
        };
        Ok(Self {
            network: Arc::clone(&handle.network),
            topic,
            subscription,
        })
    }

    pub(crate) fn publish_replication(
        &self,
        message: &GossipReplicationMessage,
    ) -> Result<(), NodeError> {
        let payload = serde_json::to_vec(message).map_err(|err| NodeError::Replication {
            reason: format!("serialize replication network message failed: {}", err),
        })?;
        self.network
            .publish(self.topic.as_str(), payload.as_slice())
            .map_err(network_err)
    }

    pub(crate) fn drain_replications(&self) -> Result<Vec<GossipReplicationMessage>, NodeError> {
        let Some(subscription) = &self.subscription else {
            return Ok(Vec::new());
        };

        let mut messages = Vec::new();
        for payload in subscription.drain() {
            if let Ok(message) = serde_json::from_slice::<GossipReplicationMessage>(&payload) {
                messages.push(message);
            }
        }
        Ok(messages)
    }
}

fn network_err(err: WorldError) -> NodeError {
    NodeError::Replication {
        reason: format!("replication network error: {err:?}"),
    }
}
