use std::net::{SocketAddr, UdpSocket};

use serde::{Deserialize, Serialize};

use crate::replication::GossipReplicationMessage;
use crate::{NodeError, NodeGossipConfig};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GossipCommitMessage {
    pub version: u8,
    pub world_id: String,
    pub node_id: String,
    pub height: u64,
    pub slot: u64,
    pub epoch: u64,
    pub block_hash: String,
    pub committed_at_ms: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_hex: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GossipProposalMessage {
    pub version: u8,
    pub world_id: String,
    pub node_id: String,
    pub proposer_id: String,
    pub height: u64,
    pub slot: u64,
    pub epoch: u64,
    pub block_hash: String,
    pub proposed_at_ms: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_hex: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct GossipAttestationMessage {
    pub version: u8,
    pub world_id: String,
    pub node_id: String,
    pub validator_id: String,
    pub height: u64,
    pub slot: u64,
    pub epoch: u64,
    pub block_hash: String,
    pub approve: bool,
    pub source_epoch: u64,
    pub target_epoch: u64,
    pub voted_at_ms: i64,
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key_hex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_hex: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum GossipMessage {
    Commit(GossipCommitMessage),
    Proposal(GossipProposalMessage),
    Attestation(GossipAttestationMessage),
    Replication(GossipReplicationMessage),
}

#[derive(Debug)]
pub(crate) struct GossipEndpoint {
    socket: UdpSocket,
    peers: Vec<SocketAddr>,
}

impl GossipEndpoint {
    pub(crate) fn bind(config: &NodeGossipConfig) -> Result<Self, NodeError> {
        let socket = UdpSocket::bind(config.bind_addr).map_err(|err| NodeError::Gossip {
            reason: format!("bind {} failed: {}", config.bind_addr, err),
        })?;
        socket
            .set_nonblocking(true)
            .map_err(|err| NodeError::Gossip {
                reason: format!("set_nonblocking failed: {}", err),
            })?;
        Ok(Self {
            socket,
            peers: config.peers.clone(),
        })
    }

    pub(crate) fn broadcast_commit(&self, message: &GossipCommitMessage) -> Result<(), NodeError> {
        self.broadcast_message(GossipMessage::Commit(message.clone()))
    }

    pub(crate) fn broadcast_proposal(
        &self,
        message: &GossipProposalMessage,
    ) -> Result<(), NodeError> {
        self.broadcast_message(GossipMessage::Proposal(message.clone()))
    }

    pub(crate) fn broadcast_attestation(
        &self,
        message: &GossipAttestationMessage,
    ) -> Result<(), NodeError> {
        self.broadcast_message(GossipMessage::Attestation(message.clone()))
    }

    pub(crate) fn broadcast_replication(
        &self,
        message: &GossipReplicationMessage,
    ) -> Result<(), NodeError> {
        self.broadcast_message(GossipMessage::Replication(message.clone()))
    }

    fn broadcast_message(&self, envelope: GossipMessage) -> Result<(), NodeError> {
        let bytes = serde_json::to_vec(&envelope).map_err(|err| NodeError::Gossip {
            reason: format!("serialize gossip message failed: {}", err),
        })?;
        self.broadcast_bytes(&bytes)
    }

    fn broadcast_bytes(&self, bytes: &[u8]) -> Result<(), NodeError> {
        for peer in &self.peers {
            self.socket
                .send_to(bytes, peer)
                .map_err(|err| NodeError::Gossip {
                    reason: format!("send_to {} failed: {}", peer, err),
                })?;
        }
        Ok(())
    }

    pub(crate) fn drain_messages(&self) -> Result<Vec<GossipMessage>, NodeError> {
        let mut buf = [0u8; 4096];
        let mut messages = Vec::new();
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((size, _from)) => {
                    let payload = &buf[..size];
                    if let Ok(message) = serde_json::from_slice::<GossipMessage>(payload) {
                        messages.push(message);
                        continue;
                    }
                    if let Ok(commit) = serde_json::from_slice::<GossipCommitMessage>(payload) {
                        messages.push(GossipMessage::Commit(commit));
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(err) => {
                    return Err(NodeError::Gossip {
                        reason: format!("recv_from failed: {}", err),
                    });
                }
            }
        }
        Ok(messages)
    }
}
