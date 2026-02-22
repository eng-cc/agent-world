use agent_world_proto::distributed_net::{DistributedNetwork, NetworkSubscription};
use agent_world_proto::world_error::WorldError;
use libp2p::identity::Keypair;
use libp2p::{Multiaddr, PeerId};

// wasm32 target intentionally does not ship a full-node networking stack.
// This stub exists only to keep API shape stable for compile-time compatibility.
#[derive(Debug, Clone)]
pub struct Libp2pReplicationNetworkConfig {
    pub keypair: Option<Keypair>,
    pub listen_addrs: Vec<Multiaddr>,
    pub bootstrap_peers: Vec<Multiaddr>,
}

impl Default for Libp2pReplicationNetworkConfig {
    fn default() -> Self {
        Self {
            keypair: None,
            listen_addrs: Vec::new(),
            bootstrap_peers: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct Libp2pReplicationNetwork {
    peer_id: PeerId,
}

impl Libp2pReplicationNetwork {
    pub fn new(config: Libp2pReplicationNetworkConfig) -> Self {
        let keypair = config.keypair.unwrap_or_else(Keypair::generate_ed25519);
        let peer_id = PeerId::from(keypair.public());
        Self { peer_id }
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }
}

fn unsupported(protocol: &str) -> WorldError {
    WorldError::NetworkProtocolUnavailable {
        protocol: format!("{protocol} (wasm32 unsupported)"),
    }
}

impl DistributedNetwork<WorldError> for Libp2pReplicationNetwork {
    fn publish(&self, _topic: &str, _payload: &[u8]) -> Result<(), WorldError> {
        Err(unsupported("libp2p-replication publish"))
    }

    fn subscribe(&self, _topic: &str) -> Result<NetworkSubscription, WorldError> {
        Err(unsupported("libp2p-replication subscribe"))
    }

    fn request(&self, _protocol: &str, _payload: &[u8]) -> Result<Vec<u8>, WorldError> {
        Err(unsupported("libp2p-replication request"))
    }

    fn register_handler(
        &self,
        _protocol: &str,
        _handler: Box<dyn Fn(&[u8]) -> Result<Vec<u8>, WorldError> + Send + Sync>,
    ) -> Result<(), WorldError> {
        Err(unsupported("libp2p-replication register_handler"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_stub_generates_peer_id() {
        let network = Libp2pReplicationNetwork::new(Libp2pReplicationNetworkConfig::default());
        assert!(!network.peer_id().to_string().is_empty());
    }
}
