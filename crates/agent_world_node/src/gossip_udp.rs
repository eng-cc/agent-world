use std::collections::BTreeSet;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Mutex;

pub(crate) use agent_world_consensus::node_consensus_message::{
    NodeGossipAttestationMessage as GossipAttestationMessage,
    NodeGossipCommitMessage as GossipCommitMessage,
    NodeGossipProposalMessage as GossipProposalMessage,
};
use serde::{Deserialize, Serialize};

use crate::replication::GossipReplicationMessage;
use crate::{NodeError, NodeGossipConfig};

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
    bind_addr: SocketAddr,
    peers: Mutex<BTreeSet<SocketAddr>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReceivedGossipMessage {
    pub from: SocketAddr,
    pub message: GossipMessage,
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
        let peers = config
            .peers
            .iter()
            .copied()
            .filter(|peer| *peer != config.bind_addr)
            .collect::<BTreeSet<_>>();
        Ok(Self {
            socket,
            bind_addr: config.bind_addr,
            peers: Mutex::new(peers),
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
        for peer in self.snapshot_peers()? {
            self.socket
                .send_to(bytes, &peer)
                .map_err(|err| NodeError::Gossip {
                    reason: format!("send_to {} failed: {}", peer, err),
                })?;
        }
        Ok(())
    }

    pub(crate) fn remember_peer(&self, peer: SocketAddr) -> Result<(), NodeError> {
        if peer == self.bind_addr || peer.port() == 0 {
            return Ok(());
        }
        let mut peers = self.peers.lock().map_err(|_| NodeError::Gossip {
            reason: "peers mutex poisoned".to_string(),
        })?;
        peers.insert(peer);
        Ok(())
    }

    pub(crate) fn drain_messages(&self) -> Result<Vec<ReceivedGossipMessage>, NodeError> {
        let mut buf = [0u8; 4096];
        let mut messages = Vec::new();
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((size, from)) => {
                    let payload = &buf[..size];
                    if let Ok(message) = serde_json::from_slice::<GossipMessage>(payload) {
                        messages.push(ReceivedGossipMessage { from, message });
                        continue;
                    }
                    if let Ok(commit) = serde_json::from_slice::<GossipCommitMessage>(payload) {
                        messages.push(ReceivedGossipMessage {
                            from,
                            message: GossipMessage::Commit(commit),
                        });
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

    fn snapshot_peers(&self) -> Result<Vec<SocketAddr>, NodeError> {
        let peers = self.peers.lock().map_err(|_| NodeError::Gossip {
            reason: "peers mutex poisoned".to_string(),
        })?;
        Ok(peers.iter().copied().collect())
    }
}
