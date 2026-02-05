//! Libp2p-based network adapter skeleton.

use super::distributed_net::{DistributedNetwork, NetworkSubscription};
use super::error::WorldError;
use libp2p::identity::Keypair;
use libp2p::PeerId;

#[derive(Debug, Clone, Default)]
pub struct Libp2pNetworkConfig {
    pub keypair: Option<Keypair>,
}

pub struct Libp2pNetwork {
    peer_id: PeerId,
    keypair: Keypair,
}

impl Libp2pNetwork {
    pub fn new(config: Libp2pNetworkConfig) -> Self {
        let keypair = config.keypair.unwrap_or_else(Keypair::generate_ed25519);
        let peer_id = PeerId::from(keypair.public());
        Self { peer_id, keypair }
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }
}

impl DistributedNetwork for Libp2pNetwork {
    fn publish(&self, _topic: &str, _payload: &[u8]) -> Result<(), WorldError> {
        Err(WorldError::NetworkProtocolUnavailable {
            protocol: "libp2p".to_string(),
        })
    }

    fn subscribe(&self, _topic: &str) -> Result<NetworkSubscription, WorldError> {
        Err(WorldError::NetworkProtocolUnavailable {
            protocol: "libp2p".to_string(),
        })
    }

    fn request(&self, _protocol: &str, _payload: &[u8]) -> Result<Vec<u8>, WorldError> {
        Err(WorldError::NetworkProtocolUnavailable {
            protocol: "libp2p".to_string(),
        })
    }

    fn register_handler(
        &self,
        _protocol: &str,
        _handler: Box<dyn Fn(&[u8]) -> Result<Vec<u8>, WorldError> + Send + Sync>,
    ) -> Result<(), WorldError> {
        Err(WorldError::NetworkProtocolUnavailable {
            protocol: "libp2p".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn libp2p_network_generates_peer_id() {
        let network = Libp2pNetwork::new(Libp2pNetworkConfig::default());
        assert!(!network.peer_id().to_string().is_empty());
    }
}
