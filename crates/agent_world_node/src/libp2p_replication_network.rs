use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use agent_world_proto::distributed_net::{DistributedNetwork, NetworkSubscription};
use agent_world_proto::world_error::WorldError;
use futures::channel::mpsc;
use futures::{FutureExt, StreamExt};
use libp2p::gossipsub::{self, IdentTopic, MessageAuthenticity, TopicHash};
use libp2p::identity::Keypair;
use libp2p::noise;
use libp2p::swarm::{NetworkBehaviour, Swarm, SwarmEvent};
use libp2p::{Multiaddr, PeerId, Transport as _};

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
    command_tx: mpsc::UnboundedSender<Command>,
    inbox: Arc<Mutex<HashMap<String, Vec<Vec<u8>>>>>,
}

enum Command {
    Publish { topic: String, payload: Vec<u8> },
    Subscribe { topic: String },
    Dial { addr: Multiaddr },
    Shutdown,
}

impl Libp2pReplicationNetwork {
    pub fn new(config: Libp2pReplicationNetworkConfig) -> Self {
        let keypair = config.keypair.unwrap_or_else(Keypair::generate_ed25519);
        let peer_id = PeerId::from(keypair.public());
        let inbox = Arc::new(Mutex::new(HashMap::<String, Vec<Vec<u8>>>::new()));
        let (command_tx, command_rx) = mpsc::unbounded();
        let inbox_for_thread = Arc::clone(&inbox);
        let bootstrap_peers = config.bootstrap_peers.clone();

        std::thread::spawn(move || {
            let mut swarm = build_swarm(&keypair);
            let mut subscribed = HashSet::<String>::new();
            let mut topic_map: HashMap<TopicHash, String> = HashMap::new();

            for addr in config.listen_addrs {
                let _ = swarm.listen_on(addr);
            }

            async_std::task::block_on(async move {
                let mut command_rx = command_rx;
                loop {
                    futures::select! {
                        command = command_rx.next().fuse() => {
                            match command {
                                Some(Command::Publish { topic, payload }) => {
                                    let topic_handle = IdentTopic::new(topic);
                                    let _ = swarm.behaviour_mut().gossipsub.publish(topic_handle, payload);
                                }
                                Some(Command::Subscribe { topic }) => {
                                    if subscribed.insert(topic.clone()) {
                                        let topic_handle = IdentTopic::new(topic.clone());
                                        if swarm.behaviour_mut().gossipsub.subscribe(&topic_handle).is_ok() {
                                            topic_map.insert(topic_handle.hash(), topic);
                                        }
                                    }
                                }
                                Some(Command::Dial { addr }) => {
                                    let _ = swarm.dial(addr);
                                }
                                Some(Command::Shutdown) | None => break,
                            }
                        }
                        event = swarm.select_next_some().fuse() => {
                            if let SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(
                                gossipsub::Event::Message { message, .. }
                            )) = event
                            {
                                let topic = topic_map
                                    .get(&message.topic)
                                    .cloned()
                                    .unwrap_or_else(|| message.topic.as_str().to_string());
                                let mut inbox = inbox_for_thread.lock().expect("lock inbox");
                                inbox.entry(topic).or_default().push(message.data);
                            }
                        }
                    }
                }
            });
        });

        for addr in bootstrap_peers {
            let _ = command_tx.unbounded_send(Command::Dial { addr });
        }

        Self {
            peer_id,
            command_tx,
            inbox,
        }
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }
}

impl Drop for Libp2pReplicationNetwork {
    fn drop(&mut self) {
        let _ = self.command_tx.unbounded_send(Command::Shutdown);
    }
}

impl DistributedNetwork<WorldError> for Libp2pReplicationNetwork {
    fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), WorldError> {
        self.command_tx
            .unbounded_send(Command::Publish {
                topic: topic.to_string(),
                payload: payload.to_vec(),
            })
            .map_err(|_| WorldError::NetworkProtocolUnavailable {
                protocol: "libp2p-replication".to_string(),
            })
    }

    fn subscribe(&self, topic: &str) -> Result<NetworkSubscription, WorldError> {
        self.command_tx
            .unbounded_send(Command::Subscribe {
                topic: topic.to_string(),
            })
            .map_err(|_| WorldError::NetworkProtocolUnavailable {
                protocol: "libp2p-replication".to_string(),
            })?;
        let mut inbox = self.inbox.lock().expect("lock inbox");
        inbox.entry(topic.to_string()).or_default();
        Ok(NetworkSubscription::new(
            topic.to_string(),
            Arc::clone(&self.inbox),
        ))
    }

    fn request(&self, protocol: &str, _payload: &[u8]) -> Result<Vec<u8>, WorldError> {
        Err(WorldError::NetworkProtocolUnavailable {
            protocol: format!("libp2p-replication request unsupported: {protocol}"),
        })
    }

    fn register_handler(
        &self,
        _protocol: &str,
        _handler: Box<dyn Fn(&[u8]) -> Result<Vec<u8>, WorldError> + Send + Sync>,
    ) -> Result<(), WorldError> {
        Ok(())
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "BehaviourEvent")]
struct Behaviour {
    gossipsub: gossipsub::Behaviour,
}

#[derive(Debug)]
enum BehaviourEvent {
    Gossipsub(gossipsub::Event),
}

impl From<gossipsub::Event> for BehaviourEvent {
    fn from(event: gossipsub::Event) -> Self {
        BehaviourEvent::Gossipsub(event)
    }
}

fn build_swarm(keypair: &Keypair) -> Swarm<Behaviour> {
    let swarm_config = libp2p::swarm::Config::with_async_std_executor()
        .with_idle_connection_timeout(std::time::Duration::from_secs(30));
    let peer_id = PeerId::from(keypair.public());
    let gossipsub = gossipsub::Behaviour::new(
        MessageAuthenticity::Signed(keypair.clone()),
        gossipsub::Config::default(),
    )
    .expect("gossipsub config");
    let behaviour = Behaviour { gossipsub };

    let transport = libp2p::tcp::async_io::Transport::new(libp2p::tcp::Config::default())
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(noise::Config::new(keypair).expect("noise config"))
        .multiplex(libp2p::yamux::Config::default())
        .boxed();

    Swarm::new(transport, behaviour, peer_id, swarm_config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn libp2p_replication_network_generates_peer_id() {
        let network = Libp2pReplicationNetwork::new(Libp2pReplicationNetworkConfig::default());
        assert!(!network.peer_id().to_string().is_empty());
    }
}
