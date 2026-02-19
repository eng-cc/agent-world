use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use agent_world_proto::distributed::DistributedErrorCode;
use agent_world_proto::distributed_net::{DistributedNetwork, NetworkSubscription};
use agent_world_proto::world_error::WorldError;
use futures::channel::{mpsc, oneshot};
use futures::{FutureExt, StreamExt};
use libp2p::gossipsub::{self, IdentTopic, MessageAuthenticity, TopicHash};
use libp2p::identity::Keypair;
use libp2p::noise;
use libp2p::request_response::{self, ProtocolSupport};
use libp2p::swarm::{dial_opts::DialOpts, NetworkBehaviour, Swarm, SwarmEvent};
use libp2p::{Multiaddr, PeerId, StreamProtocol, Transport as _};
use serde::{Deserialize, Serialize};

const RR_STREAM_PROTOCOL: &str = "/aw/node/replication/rr/1.0.0";

type Handler = Arc<dyn Fn(&[u8]) -> Result<Vec<u8>, WorldError> + Send + Sync>;

#[derive(Debug, Clone)]
pub struct Libp2pReplicationNetworkConfig {
    pub keypair: Option<Keypair>,
    pub listen_addrs: Vec<Multiaddr>,
    pub bootstrap_peers: Vec<Multiaddr>,
    pub allow_local_handler_fallback_when_no_peers: bool,
}

impl Default for Libp2pReplicationNetworkConfig {
    fn default() -> Self {
        Self {
            keypair: None,
            listen_addrs: Vec::new(),
            bootstrap_peers: Vec::new(),
            allow_local_handler_fallback_when_no_peers: false,
        }
    }
}

#[derive(Clone)]
pub struct Libp2pReplicationNetwork {
    peer_id: PeerId,
    command_tx: mpsc::UnboundedSender<Command>,
    inbox: Arc<Mutex<HashMap<String, Vec<Vec<u8>>>>>,
    #[cfg_attr(not(test), allow(dead_code))]
    listening_addrs: Arc<Mutex<Vec<Multiaddr>>>,
    #[cfg_attr(not(test), allow(dead_code))]
    connected_peers: Arc<Mutex<HashSet<PeerId>>>,
    #[cfg_attr(not(test), allow(dead_code))]
    errors: Arc<Mutex<Vec<String>>>,
}

enum Command {
    Publish {
        topic: String,
        payload: Vec<u8>,
    },
    Subscribe {
        topic: String,
    },
    Dial {
        addr: Multiaddr,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ReplicationRequest {
    protocol: String,
    payload: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ReplicationResponse {
    ok: bool,
    payload: Vec<u8>,
    error: Option<String>,
}

struct PendingRequest {
    request: ReplicationRequest,
    remaining_peers: Vec<PeerId>,
    response: oneshot::Sender<Result<Vec<u8>, WorldError>>,
}

impl Libp2pReplicationNetwork {
    pub fn new(config: Libp2pReplicationNetworkConfig) -> Self {
        let keypair = config
            .keypair
            .clone()
            .unwrap_or_else(Keypair::generate_ed25519);
        let peer_id = PeerId::from(keypair.public());
        let inbox = Arc::new(Mutex::new(HashMap::<String, Vec<Vec<u8>>>::new()));
        let listening_addrs = Arc::new(Mutex::new(Vec::<Multiaddr>::new()));
        let connected_peers = Arc::new(Mutex::new(HashSet::<PeerId>::new()));
        let errors = Arc::new(Mutex::new(Vec::<String>::new()));
        let (command_tx, command_rx) = mpsc::unbounded();
        let inbox_for_thread = Arc::clone(&inbox);
        let listening_addrs_for_thread = Arc::clone(&listening_addrs);
        let connected_peers_for_thread = Arc::clone(&connected_peers);
        let errors_for_thread = Arc::clone(&errors);
        let bootstrap_peers = config.bootstrap_peers.clone();
        let listen_addrs = config.listen_addrs.clone();
        let allow_local_handler_fallback_when_no_peers =
            config.allow_local_handler_fallback_when_no_peers;

        std::thread::spawn(move || {
            let mut swarm = build_swarm(&keypair);
            let mut subscribed = HashSet::<String>::new();
            let mut topic_map: HashMap<TopicHash, String> = HashMap::new();
            let mut handlers: HashMap<String, Handler> = HashMap::new();
            let mut pending: HashMap<request_response::OutboundRequestId, PendingRequest> =
                HashMap::new();
            let mut peers = Vec::<PeerId>::new();
            let mut request_peer_cursor = 0usize;

            for addr in listen_addrs {
                if let Err(err) = swarm.listen_on(addr) {
                    errors_for_thread
                        .lock()
                        .expect("lock libp2p errors")
                        .push(format!("libp2p replication listen failed: {err}"));
                }
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
                                    if let Err(err) = dial_addr_with_optional_peer_id(&mut swarm, addr) {
                                        errors_for_thread
                                            .lock()
                                            .expect("lock libp2p errors")
                                            .push(format!("libp2p replication dial failed: {err}"));
                                    }
                                }
                                Some(Command::Request { protocol, payload, response }) => {
                                    if peers.is_empty() {
                                        if allow_local_handler_fallback_when_no_peers {
                                            if let Some(handler) = handlers.get(protocol.as_str()) {
                                                let _ = response.send(handler(payload.as_slice()));
                                            } else {
                                                let _ = response.send(Err(WorldError::NetworkProtocolUnavailable {
                                                    protocol: format!(
                                                        "libp2p-replication handler missing: {protocol}"
                                                    ),
                                                }));
                                            }
                                        } else {
                                            let _ = response.send(Err(WorldError::NetworkProtocolUnavailable {
                                                protocol: format!(
                                                    "libp2p-replication no connected peers for protocol {protocol}"
                                                ),
                                            }));
                                        }
                                        continue;
                                    }

                                    let mut request_peers =
                                        rotated_peers(peers.as_slice(), request_peer_cursor);
                                    request_peer_cursor = request_peer_cursor.wrapping_add(1);
                                    let Some(first_peer) = request_peers.first().copied() else {
                                        let _ = response.send(Err(WorldError::NetworkProtocolUnavailable {
                                            protocol: format!(
                                                "libp2p-replication no connected peers for protocol {protocol}"
                                            ),
                                        }));
                                        continue;
                                    };
                                    request_peers.remove(0);

                                    let request = ReplicationRequest {
                                        protocol,
                                        payload,
                                    };
                                    let request_id = swarm
                                        .behaviour_mut()
                                        .request_response
                                        .send_request(&first_peer, request.clone());
                                    pending.insert(
                                        request_id,
                                        PendingRequest {
                                            request,
                                            remaining_peers: request_peers,
                                            response,
                                        },
                                    );
                                }
                                Some(Command::RegisterHandler { protocol, handler }) => {
                                    handlers.insert(protocol, handler);
                                }
                                Some(Command::Shutdown) | None => break,
                            }
                        }
                        event = swarm.select_next_some().fuse() => {
                            match event {
                                SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(
                                    gossipsub::Event::Message { message, .. }
                                )) => {
                                    let topic = topic_map
                                        .get(&message.topic)
                                        .cloned()
                                        .unwrap_or_else(|| message.topic.as_str().to_string());
                                    let mut inbox = inbox_for_thread.lock().expect("lock inbox");
                                    inbox.entry(topic).or_default().push(message.data);
                                }
                                SwarmEvent::Behaviour(BehaviourEvent::RequestResponse(event)) => {
                                    match event {
                                        request_response::Event::Message { message, .. } => {
                                            match message {
                                                request_response::Message::Request { request, channel, .. } => {
                                                    let response = match handlers.get(request.protocol.as_str()) {
                                                        Some(handler) => match handler(request.payload.as_slice()) {
                                                            Ok(payload) => ReplicationResponse {
                                                                ok: true,
                                                                payload,
                                                                error: None,
                                                            },
                                                            Err(err) => ReplicationResponse {
                                                                ok: false,
                                                                payload: Vec::new(),
                                                                error: Some(format!("{err:?}")),
                                                            },
                                                        },
                                                        None => ReplicationResponse {
                                                            ok: false,
                                                            payload: Vec::new(),
                                                            error: Some(format!(
                                                                "libp2p-replication handler missing: {}",
                                                                request.protocol
                                                            )),
                                                        },
                                                    };
                                                    if swarm
                                                        .behaviour_mut()
                                                        .request_response
                                                        .send_response(channel, response)
                                                        .is_err()
                                                    {
                                                        errors_for_thread
                                                            .lock()
                                                            .expect("lock libp2p errors")
                                                            .push(
                                                                "libp2p replication send_response failed"
                                                                    .to_string(),
                                                            );
                                                    }
                                                }
                                                request_response::Message::Response { request_id, response } => {
                                                    if let Some(mut pending_request) = pending.remove(&request_id) {
                                                        if response.ok {
                                                            let _ = pending_request.response.send(Ok(response.payload));
                                                        } else {
                                                            let retry_peer = pending_request.remaining_peers.first().copied();
                                                            if let Some(peer_id) = retry_peer {
                                                                pending_request.remaining_peers.remove(0);
                                                                let next_request_id = swarm
                                                                    .behaviour_mut()
                                                                    .request_response
                                                                    .send_request(&peer_id, pending_request.request.clone());
                                                                pending.insert(next_request_id, pending_request);
                                                            } else {
                                                                let _ = pending_request.response.send(Err(WorldError::NetworkRequestFailed {
                                                                    code: DistributedErrorCode::ErrNotFound,
                                                                    message: response.error.unwrap_or_else(|| {
                                                                        "libp2p replication remote handler failed".to_string()
                                                                    }),
                                                                    retryable: false,
                                                                }));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        request_response::Event::OutboundFailure { request_id, error, .. } => {
                                            if let Some(mut pending_request) = pending.remove(&request_id) {
                                                let retry_peer = pending_request.remaining_peers.first().copied();
                                                if let Some(peer_id) = retry_peer {
                                                    pending_request.remaining_peers.remove(0);
                                                    let next_request_id = swarm
                                                        .behaviour_mut()
                                                        .request_response
                                                        .send_request(&peer_id, pending_request.request.clone());
                                                    pending.insert(next_request_id, pending_request);
                                                } else {
                                                    let _ = pending_request.response.send(Err(WorldError::NetworkProtocolUnavailable {
                                                        protocol: format!(
                                                            "libp2p-replication outbound request failed: {error:?}"
                                                        ),
                                                    }));
                                                }
                                            }
                                        }
                                        request_response::Event::InboundFailure { peer, error, .. } => {
                                            errors_for_thread
                                                .lock()
                                                .expect("lock libp2p errors")
                                                .push(format!(
                                                    "libp2p replication inbound failure peer={peer}: {error:?}"
                                                ));
                                        }
                                        request_response::Event::ResponseSent { .. } => {}
                                    }
                                }
                                SwarmEvent::NewListenAddr { address, .. } => {
                                    listening_addrs_for_thread
                                        .lock()
                                        .expect("lock listening addrs")
                                        .push(address);
                                }
                                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                                    if !peers.contains(&peer_id) {
                                        peers.push(peer_id);
                                    }
                                    connected_peers_for_thread
                                        .lock()
                                        .expect("lock connected peers")
                                        .insert(peer_id);
                                }
                                SwarmEvent::ConnectionClosed { peer_id, .. } => {
                                    peers.retain(|peer| peer != &peer_id);
                                    connected_peers_for_thread
                                        .lock()
                                        .expect("lock connected peers")
                                        .remove(&peer_id);
                                }
                                SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                                    errors_for_thread
                                        .lock()
                                        .expect("lock libp2p errors")
                                        .push(format!(
                                            "libp2p replication outgoing connection error peer={peer_id:?}: {error:?}"
                                        ));
                                }
                                SwarmEvent::IncomingConnectionError { error, .. } => {
                                    errors_for_thread
                                        .lock()
                                        .expect("lock libp2p errors")
                                        .push(format!(
                                            "libp2p replication incoming connection error: {error:?}"
                                        ));
                                }
                                _ => {}
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
            listening_addrs,
            connected_peers,
            errors,
        }
    }

    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    #[cfg(test)]
    pub fn listening_addrs(&self) -> Vec<Multiaddr> {
        self.listening_addrs
            .lock()
            .expect("lock listening addrs")
            .clone()
    }

    #[cfg(test)]
    pub fn connected_peers(&self) -> Vec<PeerId> {
        self.connected_peers
            .lock()
            .expect("lock connected peers")
            .iter()
            .copied()
            .collect()
    }

    #[cfg(test)]
    pub fn debug_errors(&self) -> Vec<String> {
        self.errors.lock().expect("lock errors").clone()
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

    fn request(&self, protocol: &str, payload: &[u8]) -> Result<Vec<u8>, WorldError> {
        let (response_tx, response_rx) = oneshot::channel();
        self.command_tx
            .unbounded_send(Command::Request {
                protocol: protocol.to_string(),
                payload: payload.to_vec(),
                response: response_tx,
            })
            .map_err(|_| WorldError::NetworkProtocolUnavailable {
                protocol: "libp2p-replication".to_string(),
            })?;
        futures::executor::block_on(response_rx).map_err(|_| {
            WorldError::NetworkProtocolUnavailable {
                protocol: "libp2p-replication".to_string(),
            }
        })?
    }

    fn register_handler(
        &self,
        protocol: &str,
        handler: Box<dyn Fn(&[u8]) -> Result<Vec<u8>, WorldError> + Send + Sync>,
    ) -> Result<(), WorldError> {
        let handler: Handler = Arc::from(handler);
        self.command_tx
            .unbounded_send(Command::RegisterHandler {
                protocol: protocol.to_string(),
                handler,
            })
            .map_err(|_| WorldError::NetworkProtocolUnavailable {
                protocol: "libp2p-replication".to_string(),
            })
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(out_event = "BehaviourEvent")]
struct Behaviour {
    gossipsub: gossipsub::Behaviour,
    request_response: request_response::cbor::Behaviour<ReplicationRequest, ReplicationResponse>,
}

#[derive(Debug)]
enum BehaviourEvent {
    Gossipsub(gossipsub::Event),
    RequestResponse(request_response::Event<ReplicationRequest, ReplicationResponse>),
}

impl From<gossipsub::Event> for BehaviourEvent {
    fn from(event: gossipsub::Event) -> Self {
        BehaviourEvent::Gossipsub(event)
    }
}

impl From<request_response::Event<ReplicationRequest, ReplicationResponse>> for BehaviourEvent {
    fn from(event: request_response::Event<ReplicationRequest, ReplicationResponse>) -> Self {
        BehaviourEvent::RequestResponse(event)
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
    let protocols = vec![(
        StreamProtocol::new(RR_STREAM_PROTOCOL),
        ProtocolSupport::Full,
    )];
    let request_response =
        request_response::cbor::Behaviour::new(protocols, request_response::Config::default());
    let behaviour = Behaviour {
        gossipsub,
        request_response,
    };

    let transport = libp2p::tcp::async_io::Transport::new(libp2p::tcp::Config::default())
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(noise::Config::new(keypair).expect("noise config"))
        .multiplex(libp2p::yamux::Config::default())
        .boxed();

    Swarm::new(transport, behaviour, peer_id, swarm_config)
}

fn dial_addr_with_optional_peer_id(
    swarm: &mut Swarm<Behaviour>,
    addr: Multiaddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (peer_id, dial_addr) = split_peer_id(addr);
    if let Some(peer_id) = peer_id {
        let opts = DialOpts::peer_id(peer_id)
            .addresses(vec![dial_addr])
            .build();
        swarm.dial(opts)?;
    } else {
        swarm.dial(dial_addr)?;
    }
    Ok(())
}

fn split_peer_id(mut addr: Multiaddr) -> (Option<PeerId>, Multiaddr) {
    let peer_id = match addr.pop() {
        Some(libp2p::multiaddr::Protocol::P2p(peer_id)) => Some(peer_id),
        Some(protocol) => {
            addr.push(protocol);
            None
        }
        None => None,
    };
    (peer_id, addr)
}

fn rotated_peers(peers: &[PeerId], cursor: usize) -> Vec<PeerId> {
    if peers.is_empty() {
        return Vec::new();
    }
    let start = cursor % peers.len();
    peers[start..]
        .iter()
        .chain(peers[..start].iter())
        .copied()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn wait_until(what: &str, deadline: Instant, mut condition: impl FnMut() -> bool) {
        while Instant::now() < deadline {
            if condition() {
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("timed out waiting for condition: {what}");
    }

    fn listening_addr_with_peer_id(network: &Libp2pReplicationNetwork) -> Multiaddr {
        network
            .listening_addrs()
            .into_iter()
            .find(|addr| addr.to_string().contains("127.0.0.1"))
            .expect("listener visible addr")
            .with(libp2p::multiaddr::Protocol::P2p(network.peer_id().into()))
    }

    #[test]
    fn libp2p_replication_network_generates_peer_id() {
        let network = Libp2pReplicationNetwork::new(Libp2pReplicationNetworkConfig::default());
        assert!(!network.peer_id().to_string().is_empty());
    }

    #[test]
    fn libp2p_replication_network_request_rejects_without_connected_peers_by_default() {
        let network = Libp2pReplicationNetwork::new(Libp2pReplicationNetworkConfig::default());
        let result = network.request("/aw/node/replication/ping", b"hello");
        match result {
            Err(WorldError::NetworkProtocolUnavailable { protocol }) => {
                assert!(protocol.contains("no connected peers"));
            }
            other => panic!("expected NetworkProtocolUnavailable, got {other:?}"),
        }
    }

    #[test]
    fn libp2p_replication_network_request_falls_back_to_local_handler_when_enabled() {
        let network = Libp2pReplicationNetwork::new(Libp2pReplicationNetworkConfig {
            allow_local_handler_fallback_when_no_peers: true,
            ..Libp2pReplicationNetworkConfig::default()
        });
        network
            .register_handler(
                "/aw/node/replication/ping",
                Box::new(|payload| {
                    let mut out = payload.to_vec();
                    out.extend_from_slice(b"-ok");
                    Ok(out)
                }),
            )
            .expect("register local handler");

        let response = network
            .request("/aw/node/replication/ping", b"hello")
            .expect("local request");
        assert_eq!(response, b"hello-ok".to_vec());
    }

    #[test]
    fn libp2p_replication_network_request_response_between_peers() {
        let listener = Libp2pReplicationNetwork::new(Libp2pReplicationNetworkConfig {
            listen_addrs: vec!["/ip4/127.0.0.1/tcp/0".parse().expect("listener addr")],
            ..Libp2pReplicationNetworkConfig::default()
        });
        let listen_deadline = Instant::now() + Duration::from_secs(10);
        wait_until("listener bind", listen_deadline, || {
            !listener.listening_addrs().is_empty()
        });

        let dial_addr = listening_addr_with_peer_id(&listener);
        listener
            .register_handler(
                "/aw/node/replication/ping",
                Box::new(|payload| {
                    let mut out = payload.to_vec();
                    out.extend_from_slice(b"-pong");
                    Ok(out)
                }),
            )
            .expect("register listener handler");

        let dialer = Libp2pReplicationNetwork::new(Libp2pReplicationNetworkConfig {
            listen_addrs: vec!["/ip4/127.0.0.1/tcp/0".parse().expect("dialer addr")],
            bootstrap_peers: vec![dial_addr],
            ..Libp2pReplicationNetworkConfig::default()
        });
        let connect_deadline = Instant::now() + Duration::from_secs(10);
        wait_until("dialer connection", connect_deadline, || {
            !dialer.connected_peers().is_empty()
        });

        let request_deadline = Instant::now() + Duration::from_secs(10);
        wait_until("request response", request_deadline, || {
            match dialer.request("/aw/node/replication/ping", b"node") {
                Ok(payload) => payload == b"node-pong".to_vec(),
                Err(WorldError::NetworkProtocolUnavailable { .. }) => false,
                Err(WorldError::NetworkRequestFailed { .. }) => false,
                Err(err) => panic!(
                    "unexpected request error: {err:?}; dialer_errors={:?}; listener_errors={:?}",
                    dialer.debug_errors(),
                    listener.debug_errors(),
                ),
            }
        });
    }

    #[test]
    fn libp2p_replication_network_request_round_robins_across_connected_peers() {
        let listener_a = Libp2pReplicationNetwork::new(Libp2pReplicationNetworkConfig {
            listen_addrs: vec!["/ip4/127.0.0.1/tcp/0".parse().expect("listener a addr")],
            ..Libp2pReplicationNetworkConfig::default()
        });
        let listener_b = Libp2pReplicationNetwork::new(Libp2pReplicationNetworkConfig {
            listen_addrs: vec!["/ip4/127.0.0.1/tcp/0".parse().expect("listener b addr")],
            ..Libp2pReplicationNetworkConfig::default()
        });
        let listen_deadline = Instant::now() + Duration::from_secs(10);
        wait_until("listener a bind", listen_deadline, || {
            !listener_a.listening_addrs().is_empty()
        });
        wait_until("listener b bind", listen_deadline, || {
            !listener_b.listening_addrs().is_empty()
        });

        listener_a
            .register_handler(
                "/aw/node/replication/ping",
                Box::new(|payload| {
                    let mut out = payload.to_vec();
                    out.extend_from_slice(b"-a");
                    Ok(out)
                }),
            )
            .expect("register listener a handler");
        listener_b
            .register_handler(
                "/aw/node/replication/ping",
                Box::new(|payload| {
                    let mut out = payload.to_vec();
                    out.extend_from_slice(b"-b");
                    Ok(out)
                }),
            )
            .expect("register listener b handler");

        let dialer = Libp2pReplicationNetwork::new(Libp2pReplicationNetworkConfig {
            listen_addrs: vec!["/ip4/127.0.0.1/tcp/0".parse().expect("dialer addr")],
            bootstrap_peers: vec![
                listening_addr_with_peer_id(&listener_a),
                listening_addr_with_peer_id(&listener_b),
            ],
            ..Libp2pReplicationNetworkConfig::default()
        });
        let connect_deadline = Instant::now() + Duration::from_secs(10);
        wait_until("dialer connects to two peers", connect_deadline, || {
            dialer.connected_peers().len() >= 2
        });

        let first = dialer
            .request("/aw/node/replication/ping", b"node")
            .expect("first request");
        let second = dialer
            .request("/aw/node/replication/ping", b"node")
            .expect("second request");

        assert_ne!(
            first, second,
            "expected round-robin request targets to differ"
        );
        let mut responses = vec![first, second];
        responses.sort();
        assert_eq!(responses, vec![b"node-a".to_vec(), b"node-b".to_vec()]);
    }

    #[test]
    fn libp2p_replication_network_request_retries_next_peer_when_remote_handler_fails() {
        let listener_fail = Libp2pReplicationNetwork::new(Libp2pReplicationNetworkConfig {
            listen_addrs: vec!["/ip4/127.0.0.1/tcp/0".parse().expect("listener fail addr")],
            ..Libp2pReplicationNetworkConfig::default()
        });
        let listener_ok = Libp2pReplicationNetwork::new(Libp2pReplicationNetworkConfig {
            listen_addrs: vec!["/ip4/127.0.0.1/tcp/0".parse().expect("listener ok addr")],
            ..Libp2pReplicationNetworkConfig::default()
        });
        let listen_deadline = Instant::now() + Duration::from_secs(10);
        wait_until("listener fail bind", listen_deadline, || {
            !listener_fail.listening_addrs().is_empty()
        });
        wait_until("listener ok bind", listen_deadline, || {
            !listener_ok.listening_addrs().is_empty()
        });

        listener_fail
            .register_handler(
                "/aw/node/replication/ping",
                Box::new(|_payload| {
                    Err(WorldError::NetworkRequestFailed {
                        code: DistributedErrorCode::ErrUnsupported,
                        message: "forced failure".to_string(),
                        retryable: false,
                    })
                }),
            )
            .expect("register listener fail handler");
        listener_ok
            .register_handler(
                "/aw/node/replication/ping",
                Box::new(|payload| {
                    let mut out = payload.to_vec();
                    out.extend_from_slice(b"-ok");
                    Ok(out)
                }),
            )
            .expect("register listener ok handler");

        let dialer = Libp2pReplicationNetwork::new(Libp2pReplicationNetworkConfig {
            listen_addrs: vec!["/ip4/127.0.0.1/tcp/0".parse().expect("dialer addr")],
            bootstrap_peers: vec![
                listening_addr_with_peer_id(&listener_fail),
                listening_addr_with_peer_id(&listener_ok),
            ],
            ..Libp2pReplicationNetworkConfig::default()
        });
        let connect_deadline = Instant::now() + Duration::from_secs(10);
        wait_until("dialer connects to two peers", connect_deadline, || {
            dialer.connected_peers().len() >= 2
        });

        let first = dialer
            .request("/aw/node/replication/ping", b"node")
            .expect("first request should succeed via retry");
        let second = dialer
            .request("/aw/node/replication/ping", b"node")
            .expect("second request should succeed via retry");

        assert_eq!(first, b"node-ok".to_vec());
        assert_eq!(second, b"node-ok".to_vec());
    }
}
