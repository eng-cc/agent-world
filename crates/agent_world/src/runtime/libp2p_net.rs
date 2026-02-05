//! Libp2p-based network adapter skeleton (gossipsub + request/response).

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use futures::channel::{mpsc, oneshot};
use futures::{FutureExt, StreamExt};
use libp2p::gossipsub::{self, IdentTopic, MessageAuthenticity, TopicHash};
use libp2p::identity::Keypair;
use libp2p::kad::{self, store::MemoryStore};
use libp2p::noise;
use libp2p::request_response::{self, ProtocolSupport};
use libp2p::swarm::{NetworkBehaviour, Swarm, SwarmEvent};
use libp2p::{Multiaddr, PeerId, StreamProtocol, SwarmBuilder};

use super::distributed::{ErrorResponse, RR_PROTOCOL_PREFIX};
use super::distributed_net::{
    DistributedNetwork, NetworkMessage, NetworkRequest, NetworkResponse, NetworkSubscription,
};
use super::error::WorldError;
use super::util::to_canonical_cbor;

#[derive(Debug, Clone, Default)]
pub struct Libp2pNetworkConfig {
    pub keypair: Option<Keypair>,
    pub listen_addrs: Vec<Multiaddr>,
    pub bootstrap_peers: Vec<Multiaddr>,
}

#[derive(Clone)]
pub struct Libp2pNetwork {
    peer_id: PeerId,
    keypair: Keypair,
    command_tx: mpsc::UnboundedSender<Command>,
    inbox: Arc<Mutex<HashMap<String, Vec<Vec<u8>>>>>,
    published: Arc<Mutex<Vec<NetworkMessage>>>,
}

type Handler = Arc<dyn Fn(&[u8]) -> Result<Vec<u8>, WorldError> + Send + Sync>;

enum Command {
    Publish {
        topic: String,
        payload: Vec<u8>,
    },
    Subscribe {
        topic: String,
    },
    Request {
        protocol: String,
        payload: Vec<u8>,
        response: oneshot::Sender<Result<Vec<u8>, WorldError>>,
    },
    RegisterHandler {
        protocol: String,
        handler: Handler,
    },
    Shutdown,
}

impl Libp2pNetwork {
    pub fn new(config: Libp2pNetworkConfig) -> Self {
        let keypair = config
            .keypair
            .clone()
            .unwrap_or_else(Keypair::generate_ed25519);
        let peer_id = PeerId::from(keypair.public());
        let inbox = Arc::new(Mutex::new(HashMap::<String, Vec<Vec<u8>>>::new()));
        let published = Arc::new(Mutex::new(Vec::new()));
        let (command_tx, command_rx) = mpsc::unbounded();

        let event_inbox = Arc::clone(&inbox);
        let event_published = Arc::clone(&published);
        let config_clone = config.clone();
        let keypair_clone = keypair.clone();

        std::thread::spawn(move || {
            let mut swarm = build_swarm(&keypair_clone);
            let mut subscriptions = HashSet::new();
            let mut topic_map: HashMap<TopicHash, String> = HashMap::new();
            let mut handlers: HashMap<String, Handler> = HashMap::new();
            let mut pending: HashMap<
                request_response::OutboundRequestId,
                oneshot::Sender<Result<Vec<u8>, WorldError>>,
            > = HashMap::new();
            let mut peers: Vec<PeerId> = Vec::new();

            for addr in config_clone.listen_addrs {
                if let Err(err) = swarm.listen_on(addr) {
                    eprintln!("libp2p listen failed: {err}");
                }
            }

            for addr in config_clone.bootstrap_peers {
                if let Err(err) = swarm.dial(addr) {
                    eprintln!("libp2p dial failed: {err}");
                }
            }

            futures::executor::block_on(async move {
                let mut command_rx = command_rx;
                loop {
                    futures::select! {
                        command = command_rx.next().fuse() => {
                            match command {
                                Some(Command::Publish { topic, payload }) => {
                                    let message = NetworkMessage { topic: topic.clone(), payload: payload.clone() };
                                    event_published.lock().expect("lock published").push(message);
                                    let topic_handle = IdentTopic::new(topic.clone());
                                    let _ = swarm.behaviour_mut().gossipsub.publish(topic_handle, payload);
                                }
                                Some(Command::Subscribe { topic }) => {
                                    if subscriptions.insert(topic.clone()) {
                                        let topic_handle = IdentTopic::new(topic.clone());
                                        if swarm.behaviour_mut().gossipsub.subscribe(&topic_handle).is_ok() {
                                            topic_map.insert(topic_handle.hash(), topic);
                                        }
                                    }
                                }
                                Some(Command::Request { protocol, payload, response }) => {
                                    if peers.is_empty() {
                                        if let Some(handler) = handlers.get(&protocol) {
                                            let reply = handler(&payload).map_err(|err| err);
                                            let _ = response.send(reply);
                                        } else {
                                            let _ = response.send(Err(WorldError::NetworkProtocolUnavailable { protocol }));
                                        }
                                        continue;
                                    }
                                    let peer = peers[0];
                                    let request = NetworkRequest { protocol: protocol.clone(), payload };
                                    let request_id = swarm.behaviour_mut().request_response.send_request(&peer, request);
                                    pending.insert(request_id, response);
                                }
                                Some(Command::RegisterHandler { protocol, handler }) => {
                                    handlers.insert(protocol, handler);
                                }
                                Some(Command::Shutdown) | None => {
                                    break;
                                }
                            }
                        }
                        event = swarm.select_next_some().fuse() => {
                            match event {
                                SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(gossipsub::Event::Message { message, .. })) => {
                                    let topic = topic_map
                                        .get(&message.topic)
                                        .cloned()
                                        .unwrap_or_else(|| message.topic.as_str().to_string());
                                    let mut inbox = event_inbox.lock().expect("lock inbox");
                                    inbox.entry(topic).or_default().push(message.data);
                                }
                                SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(event)) => {
                                    match event {
                                        request_response::Event::Message { message, peer: _ } => {
                                            match message {
                                                request_response::Message::Request { request, channel, .. } => {
                                                    let reply = if let Some(handler) = handlers.get(&request.protocol) {
                                                        handler(&request.payload)
                                                    } else {
                                                        Err(WorldError::NetworkProtocolUnavailable { protocol: request.protocol.clone() })
                                                    };
                                                    let response_bytes = match reply {
                                                        Ok(bytes) => bytes,
                                                        Err(err) => {
                                                            let error = ErrorResponse::from_code(
                                                                super::distributed::DistributedErrorCode::ErrNotFound,
                                                                format!("{err:?}"),
                                                            );
                                                            to_canonical_cbor(&error).unwrap_or_default()
                                                        }
                                                    };
                                                    let response = NetworkResponse { payload: response_bytes };
                                                    swarm.behaviour_mut().request_response.send_response(channel, response).ok();
                                                }
                                        request_response::Message::Response { request_id, response } => {
                                            if let Some(sender) = pending.remove(&request_id) {
                                                let _ = sender.send(Ok(response.payload));
                                            }
                                        }
                                            }
                                        }
                                        request_response::Event::OutboundFailure { request_id, error, .. } => {
                                            if let Some(sender) = pending.remove(&request_id) {
                                                let _ = sender.send(Err(WorldError::NetworkProtocolUnavailable { protocol: format!("request failed: {error:?}") }));
                                            }
                                        }
                                        request_response::Event::InboundFailure { peer, error, .. } => {
                                            eprintln!("libp2p inbound failure from {peer:?}: {error:?}");
                                        }
                                        request_response::Event::ResponseSent { .. } => {}
                                    }
                                }
                                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                                    if !peers.contains(&peer_id) {
                                        peers.push(peer_id);
                                    }
                                }
                                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                                    peers.retain(|peer| peer != &peer_id);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            });
        });

        Self {
            peer_id,
            keypair,
            command_tx,
            inbox,
            published,
        }
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    pub fn published(&self) -> Vec<NetworkMessage> {
        self.published.lock().expect("lock published").clone()
    }
}

impl DistributedNetwork for Libp2pNetwork {
    fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), WorldError> {
        self.command_tx
            .unbounded_send(Command::Publish {
                topic: topic.to_string(),
                payload: payload.to_vec(),
            })
            .map_err(|_| WorldError::NetworkProtocolUnavailable {
                protocol: "libp2p".to_string(),
            })
    }

    fn subscribe(&self, topic: &str) -> Result<NetworkSubscription, WorldError> {
        self.command_tx
            .unbounded_send(Command::Subscribe {
                topic: topic.to_string(),
            })
            .map_err(|_| WorldError::NetworkProtocolUnavailable {
                protocol: "libp2p".to_string(),
            })?;
        Ok(NetworkSubscription::new(
            topic.to_string(),
            Arc::clone(&self.inbox),
        ))
    }

    fn request(&self, protocol: &str, payload: &[u8]) -> Result<Vec<u8>, WorldError> {
        let (sender, receiver) = oneshot::channel();
        self.command_tx
            .unbounded_send(Command::Request {
                protocol: protocol.to_string(),
                payload: payload.to_vec(),
                response: sender,
            })
            .map_err(|_| WorldError::NetworkProtocolUnavailable {
                protocol: "libp2p".to_string(),
            })?;
        futures::executor::block_on(receiver).map_err(|_| WorldError::NetworkProtocolUnavailable {
            protocol: "libp2p".to_string(),
        })?
    }

    fn register_handler(
        &self,
        protocol: &str,
        handler: Box<dyn Fn(&[u8]) -> Result<Vec<u8>, WorldError> + Send + Sync>,
    ) -> Result<(), WorldError> {
        self.command_tx
            .unbounded_send(Command::RegisterHandler {
                protocol: protocol.to_string(),
                handler: Arc::from(handler),
            })
            .map_err(|_| WorldError::NetworkProtocolUnavailable {
                protocol: "libp2p".to_string(),
            })
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "BehaviourEvent")]
struct Behaviour {
    gossipsub: gossipsub::Behaviour,
    request_response: request_response::cbor::Behaviour<NetworkRequest, NetworkResponse>,
    kademlia: kad::Behaviour<MemoryStore>,
}

#[derive(Debug)]
enum BehaviourEvent {
    Gossipsub(gossipsub::Event),
    RequestResponse(request_response::Event<NetworkRequest, NetworkResponse>),
    Kademlia(kad::Event),
}

impl From<gossipsub::Event> for BehaviourEvent {
    fn from(event: gossipsub::Event) -> Self {
        BehaviourEvent::Gossipsub(event)
    }
}

impl From<request_response::Event<NetworkRequest, NetworkResponse>> for BehaviourEvent {
    fn from(event: request_response::Event<NetworkRequest, NetworkResponse>) -> Self {
        BehaviourEvent::RequestResponse(event)
    }
}

impl From<kad::Event> for BehaviourEvent {
    fn from(event: kad::Event) -> Self {
        BehaviourEvent::Kademlia(event)
    }
}

fn build_swarm(keypair: &Keypair) -> Swarm<Behaviour> {
    let peer_id = PeerId::from(keypair.public());
    let gossipsub = gossipsub::Behaviour::new(
        MessageAuthenticity::Signed(keypair.clone()),
        gossipsub::Config::default(),
    )
    .expect("gossipsub config");

    let protocols = vec![(StreamProtocol::new(RR_PROTOCOL_PREFIX), ProtocolSupport::Full)];
    let request_response = request_response::cbor::Behaviour::new(
        protocols,
        request_response::Config::default(),
    );

    let store = MemoryStore::new(peer_id);
    let kademlia = kad::Behaviour::new(peer_id, store);

    let behaviour = Behaviour {
        gossipsub,
        request_response,
        kademlia,
    };

    SwarmBuilder::with_existing_identity(keypair.clone())
        .with_async_std()
        .with_tcp(
            libp2p::tcp::Config::default(),
            noise::Config::new,
            libp2p::yamux::Config::default,
        )
        .expect("tcp transport")
        .with_behaviour(|_| behaviour)
        .expect("behaviour")
        .build()
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
