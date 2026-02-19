use std::fmt;
use std::sync::Arc;

use agent_world_proto::distributed_net::{DistributedNetwork, NetworkSubscription};
use agent_world_proto::world_error::WorldError;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::gossip_udp::{
    GossipAttestationMessage, GossipCommitMessage, GossipMessage, GossipProposalMessage,
};
use crate::replication::GossipReplicationMessage;
use crate::NodeError;

pub(crate) const DEFAULT_REPLICATION_TOPIC_PREFIX: &str = "aw";
pub(crate) const DEFAULT_CONSENSUS_PROPOSAL_TOPIC_SUFFIX: &str = "consensus.proposal";
pub(crate) const DEFAULT_CONSENSUS_ATTESTATION_TOPIC_SUFFIX: &str = "consensus.attestation";
pub(crate) const DEFAULT_CONSENSUS_COMMIT_TOPIC_SUFFIX: &str = "consensus.commit";

pub(crate) fn default_replication_topic(world_id: &str) -> String {
    format!("{DEFAULT_REPLICATION_TOPIC_PREFIX}.{world_id}.replication")
}

pub(crate) fn default_consensus_proposal_topic(world_id: &str) -> String {
    format!(
        "{DEFAULT_REPLICATION_TOPIC_PREFIX}.{world_id}.{}",
        DEFAULT_CONSENSUS_PROPOSAL_TOPIC_SUFFIX
    )
}

pub(crate) fn default_consensus_attestation_topic(world_id: &str) -> String {
    format!(
        "{DEFAULT_REPLICATION_TOPIC_PREFIX}.{world_id}.{}",
        DEFAULT_CONSENSUS_ATTESTATION_TOPIC_SUFFIX
    )
}

pub(crate) fn default_consensus_commit_topic(world_id: &str) -> String {
    format!(
        "{DEFAULT_REPLICATION_TOPIC_PREFIX}.{world_id}.{}",
        DEFAULT_CONSENSUS_COMMIT_TOPIC_SUFFIX
    )
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

    pub fn clone_network(&self) -> Arc<dyn DistributedNetwork<WorldError> + Send + Sync> {
        Arc::clone(&self.network)
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

    pub(crate) fn request_json<Req, Resp>(
        &self,
        protocol: &str,
        request: &Req,
    ) -> Result<Resp, NodeError>
    where
        Req: Serialize,
        Resp: DeserializeOwned,
    {
        let payload = serde_json::to_vec(request).map_err(|err| NodeError::Replication {
            reason: format!("serialize replication request {} failed: {}", protocol, err),
        })?;
        let response_bytes = self
            .network
            .request(protocol, payload.as_slice())
            .map_err(network_err)?;
        serde_json::from_slice::<Resp>(&response_bytes).map_err(|err| NodeError::Replication {
            reason: format!("decode replication response {} failed: {}", protocol, err),
        })
    }
}

pub(crate) struct ConsensusNetworkEndpoint {
    network: Arc<dyn DistributedNetwork<WorldError> + Send + Sync>,
    proposal_topic: String,
    attestation_topic: String,
    commit_topic: String,
    proposal_subscription: Option<NetworkSubscription>,
    attestation_subscription: Option<NetworkSubscription>,
    commit_subscription: Option<NetworkSubscription>,
}

impl ConsensusNetworkEndpoint {
    pub(crate) fn new(
        handle: &NodeReplicationNetworkHandle,
        world_id: &str,
        subscribe: bool,
    ) -> Result<Self, NodeError> {
        let proposal_topic = default_consensus_proposal_topic(world_id);
        let attestation_topic = default_consensus_attestation_topic(world_id);
        let commit_topic = default_consensus_commit_topic(world_id);
        let proposal_subscription = if subscribe {
            Some(
                handle
                    .network
                    .subscribe(proposal_topic.as_str())
                    .map_err(network_err)?,
            )
        } else {
            None
        };
        let attestation_subscription = if subscribe {
            Some(
                handle
                    .network
                    .subscribe(attestation_topic.as_str())
                    .map_err(network_err)?,
            )
        } else {
            None
        };
        let commit_subscription = if subscribe {
            Some(
                handle
                    .network
                    .subscribe(commit_topic.as_str())
                    .map_err(network_err)?,
            )
        } else {
            None
        };
        Ok(Self {
            network: Arc::clone(&handle.network),
            proposal_topic,
            attestation_topic,
            commit_topic,
            proposal_subscription,
            attestation_subscription,
            commit_subscription,
        })
    }

    pub(crate) fn publish_proposal(
        &self,
        message: &GossipProposalMessage,
    ) -> Result<(), NodeError> {
        self.publish_json(self.proposal_topic.as_str(), message)
    }

    pub(crate) fn publish_attestation(
        &self,
        message: &GossipAttestationMessage,
    ) -> Result<(), NodeError> {
        self.publish_json(self.attestation_topic.as_str(), message)
    }

    pub(crate) fn publish_commit(&self, message: &GossipCommitMessage) -> Result<(), NodeError> {
        self.publish_json(self.commit_topic.as_str(), message)
    }

    pub(crate) fn drain_messages(&self) -> Result<Vec<GossipMessage>, NodeError> {
        let mut out = Vec::new();
        Self::drain_subscription(self.proposal_subscription.as_ref(), &mut out);
        Self::drain_subscription(self.attestation_subscription.as_ref(), &mut out);
        Self::drain_subscription(self.commit_subscription.as_ref(), &mut out);
        Ok(out)
    }

    fn publish_json<T: Serialize>(&self, topic: &str, message: &T) -> Result<(), NodeError> {
        let payload = serde_json::to_vec(message).map_err(|err| NodeError::Replication {
            reason: format!("serialize consensus network message failed: {}", err),
        })?;
        self.network
            .publish(topic, payload.as_slice())
            .map_err(network_err)
    }

    fn drain_subscription(
        subscription: Option<&NetworkSubscription>,
        out: &mut Vec<GossipMessage>,
    ) {
        let Some(subscription) = subscription else {
            return;
        };
        for payload in subscription.drain() {
            if let Some(message) = decode_consensus_message(payload.as_slice()) {
                out.push(message);
            }
        }
    }
}

fn decode_consensus_message(payload: &[u8]) -> Option<GossipMessage> {
    if let Ok(message) = serde_json::from_slice::<GossipMessage>(payload) {
        match message {
            GossipMessage::Proposal(_)
            | GossipMessage::Attestation(_)
            | GossipMessage::Commit(_) => return Some(message),
            GossipMessage::Replication(_) => {}
        }
    }
    if let Ok(message) = serde_json::from_slice::<GossipProposalMessage>(payload) {
        return Some(GossipMessage::Proposal(message));
    }
    if let Ok(message) = serde_json::from_slice::<GossipAttestationMessage>(payload) {
        return Some(GossipMessage::Attestation(message));
    }
    if let Ok(message) = serde_json::from_slice::<GossipCommitMessage>(payload) {
        return Some(GossipMessage::Commit(message));
    }
    None
}

fn network_err(err: WorldError) -> NodeError {
    NodeError::Replication {
        reason: format!("replication network error: {err:?}"),
    }
}
