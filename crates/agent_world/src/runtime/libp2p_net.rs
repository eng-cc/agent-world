//! Libp2p-based network adapter skeleton (gossipsub + request/response).

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use futures::channel::{mpsc, oneshot};
use futures::{FutureExt, StreamExt};
use libp2p::gossipsub::{self, IdentTopic, MessageAuthenticity, TopicHash};
use libp2p::identity::Keypair;
use libp2p::kad::{self, store::MemoryStore, Quorum, RecordKey};
use libp2p::noise;
use libp2p::request_response::{self, ProtocolSupport};
use libp2p::swarm::{NetworkBehaviour, Swarm, SwarmEvent};
use libp2p::{Multiaddr, PeerId, StreamProtocol, SwarmBuilder};

use super::distributed::{
    dht_provider_key, dht_world_head_key, ErrorResponse, WorldHeadAnnounce, RR_PROTOCOL_PREFIX,
};
use super::distributed_dht::{DistributedDht, ProviderRecord};
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
        providers: Vec<String>,
        response: oneshot::Sender<Result<Vec<u8>, WorldError>>,
    },
    RegisterHandler {
        protocol: String,
        handler: Handler,
    },
    PublishProvider {
        key: String,
        response: oneshot::Sender<Result<(), WorldError>>,
    },
    GetProviders {
        key: String,
        response: oneshot::Sender<Result<Vec<ProviderRecord>, WorldError>>,
    },
    PutWorldHead {
        key: String,
        payload: Vec<u8>,
        response: oneshot::Sender<Result<(), WorldError>>,
    },
    GetWorldHead {
        key: String,
        response: oneshot::Sender<Result<Option<WorldHeadAnnounce>, WorldError>>,
    },
    Shutdown,
}

enum PendingDhtQuery {
    PublishProvider {
        response: Option<oneshot::Sender<Result<(), WorldError>>>,
    },
    GetProviders {
        response: Option<oneshot::Sender<Result<Vec<ProviderRecord>, WorldError>>>,
        providers: HashSet<PeerId>,
        error: Option<WorldError>,
    },
    PutWorldHead {
        response: Option<oneshot::Sender<Result<(), WorldError>>>,
    },
    GetWorldHead {
        response: Option<oneshot::Sender<Result<Option<WorldHeadAnnounce>, WorldError>>>,
        head: Option<WorldHeadAnnounce>,
        error: Option<WorldError>,
    },
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
            let mut pending_dht: HashMap<kad::QueryId, PendingDhtQuery> = HashMap::new();
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
                                Some(Command::Request { protocol, payload, providers, response }) => {
                                    if peers.is_empty() {
                                        if let Some(handler) = handlers.get(&protocol) {
                                            let reply = handler(&payload).map_err(|err| err);
                                            let _ = response.send(reply);
                                        } else {
                                            let _ = response.send(Err(WorldError::NetworkProtocolUnavailable { protocol }));
                                        }
                                        continue;
                                    }
                                    let mut selected_peer = None;
                                    if !providers.is_empty() {
                                        for provider in providers {
                                            if let Ok(peer_id) = provider.parse::<PeerId>() {
                                                if peers.contains(&peer_id) {
                                                    selected_peer = Some(peer_id);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    let peer = selected_peer.unwrap_or_else(|| peers[0]);
                                    let request = NetworkRequest { protocol: protocol.clone(), payload };
                                    let request_id = swarm.behaviour_mut().request_response.send_request(&peer, request);
                                    pending.insert(request_id, response);
                                }
                                Some(Command::RegisterHandler { protocol, handler }) => {
                                    handlers.insert(protocol, handler);
                                }
                                Some(Command::PublishProvider { key, response }) => {
                                    let dht_key = RecordKey::new(&key);
                                    match swarm.behaviour_mut().kademlia.start_providing(dht_key) {
                                        Ok(query_id) => {
                                            pending_dht.insert(
                                                query_id,
                                                PendingDhtQuery::PublishProvider {
                                                    response: Some(response),
                                                },
                                            );
                                        }
                                        Err(err) => {
                                            let _ = response.send(Err(WorldError::NetworkProtocolUnavailable {
                                                protocol: format!("kad start_providing failed: {err}"),
                                            }));
                                        }
                                    }
                                }
                                Some(Command::GetProviders { key, response }) => {
                                    let dht_key = RecordKey::new(&key);
                                    let query_id = swarm.behaviour_mut().kademlia.get_providers(dht_key);
                                    pending_dht.insert(
                                        query_id,
                                        PendingDhtQuery::GetProviders {
                                            response: Some(response),
                                            providers: HashSet::new(),
                                            error: None,
                                        },
                                    );
                                }
                                Some(Command::PutWorldHead { key, payload, response }) => {
                                    let dht_key = RecordKey::new(&key);
                                    let record = kad::Record {
                                        key: dht_key,
                                        value: payload,
                                        publisher: None,
                                        expires: None,
                                    };
                                    match swarm.behaviour_mut().kademlia.put_record(record, Quorum::One) {
                                        Ok(query_id) => {
                                            pending_dht.insert(
                                                query_id,
                                                PendingDhtQuery::PutWorldHead {
                                                    response: Some(response),
                                                },
                                            );
                                        }
                                        Err(err) => {
                                            let _ = response.send(Err(WorldError::NetworkProtocolUnavailable {
                                                protocol: format!("kad put_record failed: {err}"),
                                            }));
                                        }
                                    }
                                }
                                Some(Command::GetWorldHead { key, response }) => {
                                    let dht_key = RecordKey::new(&key);
                                    let query_id = swarm.behaviour_mut().kademlia.get_record(dht_key);
                                    pending_dht.insert(
                                        query_id,
                                        PendingDhtQuery::GetWorldHead {
                                            response: Some(response),
                                            head: None,
                                            error: None,
                                        },
                                    );
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
                                SwarmEvent::Behaviour(BehaviourEvent::Kademlia(event)) => {
                                    if let kad::Event::OutboundQueryProgressed { id, result, step, .. } = event {
                                        if let Some(pending) = pending_dht.get_mut(&id) {
                                            handle_dht_progress(pending, result, step.last);
                                        }
                                        if step.last {
                                            pending_dht.remove(&id);
                                        }
                                    }
                                }
                                SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                                    if !peers.contains(&peer_id) {
                                        peers.push(peer_id);
                                    }
                                    match endpoint {
                                        libp2p::core::connection::ConnectedPoint::Dialer { address, .. } => {
                                            swarm
                                                .behaviour_mut()
                                                .kademlia
                                                .add_address(&peer_id, address.clone());
                                        }
                                        libp2p::core::connection::ConnectedPoint::Listener { send_back_addr, .. } => {
                                            swarm
                                                .behaviour_mut()
                                                .kademlia
                                                .add_address(&peer_id, send_back_addr.clone());
                                        }
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

impl Drop for Libp2pNetwork {
    fn drop(&mut self) {
        let _ = self.command_tx.unbounded_send(Command::Shutdown);
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
        self.request_with_providers(protocol, payload, &[])
    }

    fn request_with_providers(
        &self,
        protocol: &str,
        payload: &[u8],
        providers: &[String],
    ) -> Result<Vec<u8>, WorldError> {
        let (sender, receiver) = oneshot::channel();
        self.command_tx
            .unbounded_send(Command::Request {
                protocol: protocol.to_string(),
                payload: payload.to_vec(),
                providers: providers.to_vec(),
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

impl DistributedDht for Libp2pNetwork {
    fn publish_provider(
        &self,
        world_id: &str,
        content_hash: &str,
        _provider_id: &str,
    ) -> Result<(), WorldError> {
        let key = dht_provider_key(world_id, content_hash);
        let (sender, receiver) = oneshot::channel();
        self.command_tx
            .unbounded_send(Command::PublishProvider { key, response: sender })
            .map_err(|_| WorldError::NetworkProtocolUnavailable {
                protocol: "libp2p".to_string(),
            })?;
        futures::executor::block_on(receiver).map_err(|_| WorldError::NetworkProtocolUnavailable {
            protocol: "libp2p".to_string(),
        })?
    }

    fn get_providers(
        &self,
        world_id: &str,
        content_hash: &str,
    ) -> Result<Vec<ProviderRecord>, WorldError> {
        let key = dht_provider_key(world_id, content_hash);
        let (sender, receiver) = oneshot::channel();
        self.command_tx
            .unbounded_send(Command::GetProviders { key, response: sender })
            .map_err(|_| WorldError::NetworkProtocolUnavailable {
                protocol: "libp2p".to_string(),
            })?;
        futures::executor::block_on(receiver).map_err(|_| WorldError::NetworkProtocolUnavailable {
            protocol: "libp2p".to_string(),
        })?
    }

    fn put_world_head(
        &self,
        world_id: &str,
        head: &WorldHeadAnnounce,
    ) -> Result<(), WorldError> {
        let key = dht_world_head_key(world_id);
        let payload = to_canonical_cbor(head)?;
        let (sender, receiver) = oneshot::channel();
        self.command_tx
            .unbounded_send(Command::PutWorldHead {
                key,
                payload,
                response: sender,
            })
            .map_err(|_| WorldError::NetworkProtocolUnavailable {
                protocol: "libp2p".to_string(),
            })?;
        futures::executor::block_on(receiver).map_err(|_| WorldError::NetworkProtocolUnavailable {
            protocol: "libp2p".to_string(),
        })?
    }

    fn get_world_head(&self, world_id: &str) -> Result<Option<WorldHeadAnnounce>, WorldError> {
        let key = dht_world_head_key(world_id);
        let (sender, receiver) = oneshot::channel();
        self.command_tx
            .unbounded_send(Command::GetWorldHead { key, response: sender })
            .map_err(|_| WorldError::NetworkProtocolUnavailable {
                protocol: "libp2p".to_string(),
            })?;
        futures::executor::block_on(receiver).map_err(|_| WorldError::NetworkProtocolUnavailable {
            protocol: "libp2p".to_string(),
        })?
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

fn handle_dht_progress(pending: &mut PendingDhtQuery, result: kad::QueryResult, is_last: bool) {
    match pending {
        PendingDhtQuery::PublishProvider { response } => {
            if is_last {
                let outcome = match result {
                    kad::QueryResult::StartProviding(Ok(_))
                    | kad::QueryResult::RepublishProvider(Ok(_)) => Ok(()),
                    kad::QueryResult::StartProviding(Err(err))
                    | kad::QueryResult::RepublishProvider(Err(err)) => Err(
                        WorldError::NetworkProtocolUnavailable {
                            protocol: format!("kad start_providing failed: {err}"),
                        },
                    ),
                    _ => Ok(()),
                };
                if let Some(response) = response.take() {
                    let _ = response.send(outcome);
                }
            }
        }
        PendingDhtQuery::PutWorldHead { response } => {
            if is_last {
                let outcome = match result {
                    kad::QueryResult::PutRecord(Ok(_))
                    | kad::QueryResult::RepublishRecord(Ok(_)) => Ok(()),
                    kad::QueryResult::PutRecord(Err(err))
                    | kad::QueryResult::RepublishRecord(Err(err)) => Err(
                        WorldError::NetworkProtocolUnavailable {
                            protocol: format!("kad put_record failed: {err}"),
                        },
                    ),
                    _ => Ok(()),
                };
                if let Some(response) = response.take() {
                    let _ = response.send(outcome);
                }
            }
        }
        PendingDhtQuery::GetProviders {
            response,
            providers,
            error,
        } => {
            match result {
                kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders {
                    providers: found,
                    ..
                })) => {
                    providers.extend(found);
                }
                kad::QueryResult::GetProviders(Ok(
                    kad::GetProvidersOk::FinishedWithNoAdditionalRecord { .. },
                )) => {}
                kad::QueryResult::GetProviders(Err(err)) => {
                    *error = Some(WorldError::NetworkProtocolUnavailable {
                        protocol: format!("kad get_providers failed: {err}"),
                    });
                }
                _ => {}
            }
            if is_last {
                let outcome = if !providers.is_empty() {
                    Ok(providers
                        .iter()
                        .map(|peer| ProviderRecord {
                            provider_id: peer.to_string(),
                            last_seen_ms: now_ms(),
                        })
                        .collect())
                } else if let Some(err) = error.take() {
                    Err(err)
                } else {
                    Ok(Vec::new())
                };
                if let Some(response) = response.take() {
                    let _ = response.send(outcome);
                }
            }
        }
        PendingDhtQuery::GetWorldHead {
            response,
            head,
            error,
        } => {
            match result {
                kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(record))) => {
                    if let Ok(decoded) = decode_world_head(&record.record.value) {
                        *head = Some(decoded);
                    }
                }
                kad::QueryResult::GetRecord(Ok(
                    kad::GetRecordOk::FinishedWithNoAdditionalRecord { .. },
                )) => {}
                kad::QueryResult::GetRecord(Err(kad::GetRecordError::NotFound { .. })) => {
                    *error = None;
                }
                kad::QueryResult::GetRecord(Err(kad::GetRecordError::QuorumFailed { records, .. })) => {
                    if let Some(record) = records.first() {
                        if let Ok(decoded) = decode_world_head(&record.record.value) {
                            *head = Some(decoded);
                        }
                    } else {
                        *error = Some(WorldError::NetworkProtocolUnavailable {
                            protocol: "kad get_record quorum failed".to_string(),
                        });
                    }
                }
                kad::QueryResult::GetRecord(Err(err)) => {
                    *error = Some(WorldError::NetworkProtocolUnavailable {
                        protocol: format!("kad get_record failed: {err}"),
                    });
                }
                _ => {}
            }
            if is_last {
                let outcome = if head.is_some() {
                    Ok(head.clone())
                } else if let Some(err) = error.take() {
                    Err(err)
                } else {
                    Ok(None)
                };
                if let Some(response) = response.take() {
                    let _ = response.send(outcome);
                }
            }
        }
    }
}

fn decode_world_head(bytes: &[u8]) -> Result<WorldHeadAnnounce, WorldError> {
    Ok(serde_cbor::from_slice(bytes)?)
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn libp2p_network_generates_peer_id() {
        let network = Libp2pNetwork::new(Libp2pNetworkConfig::default());
        assert!(!network.peer_id().to_string().is_empty());
    }

    #[test]
    fn dht_get_providers_collects_results() {
        let (sender, receiver) = oneshot::channel();
        let mut pending = PendingDhtQuery::GetProviders {
            response: Some(sender),
            providers: HashSet::new(),
            error: None,
        };
        let key_label = "providers".to_string();
        let key = RecordKey::new(&key_label);
        let mut providers = HashSet::new();
        providers.insert(PeerId::random());
        providers.insert(PeerId::random());
        let expected: HashSet<String> = providers.iter().map(|peer| peer.to_string()).collect();
        let result = kad::QueryResult::GetProviders(Ok(kad::GetProvidersOk::FoundProviders {
            key,
            providers,
        }));
        handle_dht_progress(&mut pending, result, true);
        let records = futures::executor::block_on(receiver)
            .expect("oneshot")
            .expect("get providers");
        let actual: HashSet<String> = records
            .into_iter()
            .map(|record| record.provider_id)
            .collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn dht_get_world_head_decodes_record() {
        let head = WorldHeadAnnounce {
            world_id: "w1".to_string(),
            height: 9,
            block_hash: "b1".to_string(),
            state_root: "s1".to_string(),
            timestamp_ms: 42,
            signature: "sig".to_string(),
        };
        let payload = to_canonical_cbor(&head).expect("encode head");
        let key_label = "head".to_string();
        let record = kad::Record {
            key: RecordKey::new(&key_label),
            value: payload,
            publisher: None,
            expires: None,
        };
        let peer_record = kad::PeerRecord {
            peer: None,
            record,
        };
        let (sender, receiver) = oneshot::channel();
        let mut pending = PendingDhtQuery::GetWorldHead {
            response: Some(sender),
            head: None,
            error: None,
        };
        let result = kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(peer_record)));
        handle_dht_progress(&mut pending, result, true);
        let loaded = futures::executor::block_on(receiver)
            .expect("oneshot")
            .expect("get head");
        assert_eq!(loaded, Some(head));
    }
}
