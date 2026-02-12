//! Network-focused facade for distributed runtime capabilities.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use agent_world::runtime::{ModuleArtifact, ModuleManifest, WorldError};
use agent_world_proto::distributed as proto_distributed;
use agent_world_proto::distributed::WorldHeadAnnounce;
use agent_world_proto::distributed_dht as proto_dht;
use agent_world_proto::distributed_net as proto_net;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub use agent_world::runtime::{
    publish_execution_providers, publish_execution_providers_cached, publish_world_head,
    query_providers, CachedDht, DhtCacheConfig, DistributedIndexStore, HeadFollowReport,
    HeadFollower, HeadIndexRecord, HeadSyncReport, HeadSyncResult, HeadUpdateDecision,
    InMemoryIndexStore, IndexPublishResult, ObserverClient, ObserverSubscription, ProviderCache,
    ProviderCacheConfig,
};
pub use proto_dht::{MembershipDirectorySnapshot, ProviderRecord};
pub use proto_net::{NetworkMessage, NetworkRequest, NetworkResponse, NetworkSubscription};

pub trait DistributedNetwork: proto_net::DistributedNetwork<WorldError> {}

impl<T> DistributedNetwork for T where T: proto_net::DistributedNetwork<WorldError> {}

#[derive(Clone, Default)]
pub struct InMemoryNetwork {
    inbox: Arc<Mutex<HashMap<String, Vec<Vec<u8>>>>>,
    published: Arc<Mutex<Vec<NetworkMessage>>>,
    handlers: Arc<Mutex<HashMap<String, Handler>>>,
}

type Handler = Arc<dyn Fn(&[u8]) -> Result<Vec<u8>, WorldError> + Send + Sync>;

impl InMemoryNetwork {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn published(&self) -> Vec<NetworkMessage> {
        self.published.lock().expect("lock published").clone()
    }
}

impl proto_net::DistributedNetwork<WorldError> for InMemoryNetwork {
    fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), WorldError> {
        let message = NetworkMessage {
            topic: topic.to_string(),
            payload: payload.to_vec(),
        };
        {
            let mut published = self.published.lock().expect("lock published");
            published.push(message.clone());
        }
        let mut inbox = self.inbox.lock().expect("lock inbox");
        inbox
            .entry(topic.to_string())
            .or_default()
            .push(message.payload);
        Ok(())
    }

    fn subscribe(&self, topic: &str) -> Result<NetworkSubscription, WorldError> {
        let mut inbox = self.inbox.lock().expect("lock inbox");
        inbox.entry(topic.to_string()).or_default();
        Ok(NetworkSubscription::new(
            topic.to_string(),
            Arc::clone(&self.inbox),
        ))
    }

    fn request(&self, protocol: &str, payload: &[u8]) -> Result<Vec<u8>, WorldError> {
        let handler = {
            let handlers = self.handlers.lock().expect("lock handlers");
            handlers.get(protocol).cloned()
        };
        let handler = handler.ok_or_else(|| WorldError::NetworkProtocolUnavailable {
            protocol: protocol.to_string(),
        })?;
        handler(payload)
    }

    fn register_handler(
        &self,
        protocol: &str,
        handler: Box<dyn Fn(&[u8]) -> Result<Vec<u8>, WorldError> + Send + Sync>,
    ) -> Result<(), WorldError> {
        let mut handlers = self.handlers.lock().expect("lock handlers");
        handlers.insert(protocol.to_string(), Arc::from(handler));
        Ok(())
    }
}

pub trait DistributedDht: proto_dht::DistributedDht<WorldError> {}

impl<T> DistributedDht for T where T: proto_dht::DistributedDht<WorldError> {}

#[derive(Debug, Clone, Default)]
pub struct InMemoryDht {
    providers: Arc<Mutex<BTreeMap<(String, String), BTreeMap<String, ProviderRecord>>>>,
    heads: Arc<Mutex<BTreeMap<String, WorldHeadAnnounce>>>,
    memberships: Arc<Mutex<BTreeMap<String, MembershipDirectorySnapshot>>>,
}

impl InMemoryDht {
    pub fn new() -> Self {
        Self::default()
    }
}

impl proto_dht::DistributedDht<WorldError> for InMemoryDht {
    fn publish_provider(
        &self,
        world_id: &str,
        content_hash: &str,
        provider_id: &str,
    ) -> Result<(), WorldError> {
        let mut providers = self.providers.lock().expect("lock providers");
        let key = (world_id.to_string(), content_hash.to_string());
        let record = ProviderRecord {
            provider_id: provider_id.to_string(),
            last_seen_ms: now_ms(),
        };
        providers
            .entry(key)
            .or_default()
            .insert(provider_id.to_string(), record);
        Ok(())
    }

    fn get_providers(
        &self,
        world_id: &str,
        content_hash: &str,
    ) -> Result<Vec<ProviderRecord>, WorldError> {
        let providers = self.providers.lock().expect("lock providers");
        let key = (world_id.to_string(), content_hash.to_string());
        Ok(providers
            .get(&key)
            .map(|records| records.values().cloned().collect())
            .unwrap_or_default())
    }

    fn put_world_head(&self, world_id: &str, head: &WorldHeadAnnounce) -> Result<(), WorldError> {
        let mut heads = self.heads.lock().expect("lock heads");
        heads.insert(world_id.to_string(), head.clone());
        Ok(())
    }

    fn get_world_head(&self, world_id: &str) -> Result<Option<WorldHeadAnnounce>, WorldError> {
        let heads = self.heads.lock().expect("lock heads");
        Ok(heads.get(world_id).cloned())
    }

    fn put_membership_directory(
        &self,
        world_id: &str,
        snapshot: &MembershipDirectorySnapshot,
    ) -> Result<(), WorldError> {
        let mut memberships = self.memberships.lock().expect("lock memberships");
        memberships.insert(world_id.to_string(), snapshot.clone());
        Ok(())
    }

    fn get_membership_directory(
        &self,
        world_id: &str,
    ) -> Result<Option<MembershipDirectorySnapshot>, WorldError> {
        let memberships = self.memberships.lock().expect("lock memberships");
        Ok(memberships.get(world_id).cloned())
    }
}

#[derive(Clone)]
pub struct DistributedClient {
    network: Arc<dyn DistributedNetwork + Send + Sync>,
}

impl DistributedClient {
    pub fn new(network: Arc<dyn DistributedNetwork + Send + Sync>) -> Self {
        Self { network }
    }

    pub fn get_world_head(&self, world_id: &str) -> Result<WorldHeadAnnounce, WorldError> {
        let request = proto_distributed::GetWorldHeadRequest {
            world_id: world_id.to_string(),
        };
        let response: proto_distributed::GetWorldHeadResponse =
            self.request(proto_distributed::RR_GET_WORLD_HEAD, &request)?;
        Ok(response.head)
    }

    pub fn get_block(
        &self,
        world_id: &str,
        height: u64,
    ) -> Result<proto_distributed::WorldBlock, WorldError> {
        Ok(self.get_block_response(world_id, height)?.block)
    }

    pub fn get_block_response(
        &self,
        world_id: &str,
        height: u64,
    ) -> Result<proto_distributed::GetBlockResponse, WorldError> {
        let request = proto_distributed::GetBlockRequest {
            world_id: world_id.to_string(),
            height,
        };
        self.request(proto_distributed::RR_GET_BLOCK, &request)
    }

    pub fn get_snapshot_manifest(
        &self,
        world_id: &str,
        epoch: u64,
    ) -> Result<proto_distributed::SnapshotManifest, WorldError> {
        let request = proto_distributed::GetSnapshotRequest {
            world_id: world_id.to_string(),
            epoch,
        };
        let response: proto_distributed::GetSnapshotResponse =
            self.request(proto_distributed::RR_GET_SNAPSHOT, &request)?;
        Ok(response.manifest)
    }

    pub fn fetch_blob(&self, content_hash: &str) -> Result<Vec<u8>, WorldError> {
        let request = proto_distributed::FetchBlobRequest {
            content_hash: content_hash.to_string(),
        };
        let response: proto_distributed::FetchBlobResponse =
            self.request(proto_distributed::RR_FETCH_BLOB, &request)?;
        Ok(response.blob)
    }

    pub fn fetch_blob_with_providers(
        &self,
        content_hash: &str,
        providers: &[String],
    ) -> Result<Vec<u8>, WorldError> {
        let request = proto_distributed::FetchBlobRequest {
            content_hash: content_hash.to_string(),
        };
        let response: proto_distributed::FetchBlobResponse =
            self.request_with_providers(proto_distributed::RR_FETCH_BLOB, &request, providers)?;
        Ok(response.blob)
    }

    pub fn fetch_blob_from_dht(
        &self,
        world_id: &str,
        content_hash: &str,
        dht: &impl DistributedDht,
    ) -> Result<Vec<u8>, WorldError> {
        let providers = dht.get_providers(world_id, content_hash)?;
        if providers.is_empty() {
            return self.fetch_blob(content_hash);
        }

        let provider_ids: Vec<String> = providers
            .into_iter()
            .map(|record| record.provider_id)
            .collect();
        match self.fetch_blob_with_providers(content_hash, &provider_ids) {
            Ok(bytes) => Ok(bytes),
            Err(_) => self.fetch_blob(content_hash),
        }
    }

    pub fn get_journal_segment(
        &self,
        world_id: &str,
        from_event_id: u64,
    ) -> Result<proto_distributed::BlobRef, WorldError> {
        let request = proto_distributed::GetJournalSegmentRequest {
            world_id: world_id.to_string(),
            from_event_id,
        };
        let response: proto_distributed::GetJournalSegmentResponse =
            self.request(proto_distributed::RR_GET_JOURNAL_SEGMENT, &request)?;
        Ok(response.segment)
    }

    pub fn get_receipt_segment(
        &self,
        world_id: &str,
        from_event_id: u64,
    ) -> Result<proto_distributed::BlobRef, WorldError> {
        let request = proto_distributed::GetReceiptSegmentRequest {
            world_id: world_id.to_string(),
            from_event_id,
        };
        let response: proto_distributed::GetReceiptSegmentResponse =
            self.request(proto_distributed::RR_GET_RECEIPT_SEGMENT, &request)?;
        Ok(response.segment)
    }

    pub fn get_module_manifest(
        &self,
        module_id: &str,
        manifest_hash: &str,
    ) -> Result<proto_distributed::BlobRef, WorldError> {
        let request = proto_distributed::GetModuleManifestRequest {
            module_id: module_id.to_string(),
            manifest_hash: manifest_hash.to_string(),
        };
        let response: proto_distributed::GetModuleManifestResponse =
            self.request(proto_distributed::RR_GET_MODULE_MANIFEST, &request)?;
        Ok(response.manifest_ref)
    }

    pub fn get_module_artifact(
        &self,
        wasm_hash: &str,
    ) -> Result<proto_distributed::BlobRef, WorldError> {
        let request = proto_distributed::GetModuleArtifactRequest {
            wasm_hash: wasm_hash.to_string(),
        };
        let response: proto_distributed::GetModuleArtifactResponse =
            self.request(proto_distributed::RR_GET_MODULE_ARTIFACT, &request)?;
        Ok(response.artifact_ref)
    }

    pub fn fetch_module_manifest_from_dht(
        &self,
        world_id: &str,
        module_id: &str,
        manifest_hash: &str,
        dht: &impl DistributedDht,
    ) -> Result<ModuleManifest, WorldError> {
        let manifest_ref = self.get_module_manifest(module_id, manifest_hash)?;
        let bytes = self.fetch_blob_from_dht(world_id, &manifest_ref.content_hash, dht)?;
        Ok(serde_cbor::from_slice(&bytes)?)
    }

    pub fn fetch_module_artifact_from_dht(
        &self,
        world_id: &str,
        wasm_hash: &str,
        dht: &impl DistributedDht,
    ) -> Result<ModuleArtifact, WorldError> {
        let artifact_ref = self.get_module_artifact(wasm_hash)?;
        let bytes = self.fetch_blob_from_dht(world_id, &artifact_ref.content_hash, dht)?;
        Ok(ModuleArtifact {
            wasm_hash: wasm_hash.to_string(),
            bytes,
        })
    }

    fn request<T: Serialize, R: DeserializeOwned>(
        &self,
        protocol: &str,
        request: &T,
    ) -> Result<R, WorldError> {
        let payload = to_canonical_cbor(request)?;
        let response_bytes = self.network.request(protocol, &payload)?;
        decode_response(&response_bytes)
    }

    fn request_with_providers<T: Serialize, R: DeserializeOwned>(
        &self,
        protocol: &str,
        request: &T,
        providers: &[String],
    ) -> Result<R, WorldError> {
        let payload = to_canonical_cbor(request)?;
        let response_bytes = self
            .network
            .request_with_providers(protocol, &payload, providers)?;
        decode_response(&response_bytes)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmitActionReceipt {
    pub action_id: String,
    pub accepted_at_ms: i64,
}

pub trait ActionGateway {
    fn submit_action(
        &self,
        action: proto_distributed::ActionEnvelope,
    ) -> Result<SubmitActionReceipt, WorldError>;
}

#[derive(Clone)]
pub struct NetworkGateway {
    network: Arc<dyn DistributedNetwork + Send + Sync>,
    now_fn: Arc<dyn Fn() -> i64 + Send + Sync>,
}

impl NetworkGateway {
    pub fn new(network: Arc<dyn DistributedNetwork + Send + Sync>) -> Self {
        Self {
            network,
            now_fn: Arc::new(now_ms),
        }
    }

    pub fn new_with_clock(
        network: Arc<dyn DistributedNetwork + Send + Sync>,
        now_fn: Arc<dyn Fn() -> i64 + Send + Sync>,
    ) -> Self {
        Self { network, now_fn }
    }
}

impl ActionGateway for NetworkGateway {
    fn submit_action(
        &self,
        action: proto_distributed::ActionEnvelope,
    ) -> Result<SubmitActionReceipt, WorldError> {
        let topic = proto_distributed::topic_action(&action.world_id);
        let payload = to_canonical_cbor(&action)?;
        self.network.publish(&topic, &payload)?;
        Ok(SubmitActionReceipt {
            action_id: action.action_id,
            accepted_at_ms: (self.now_fn)(),
        })
    }
}

fn decode_response<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, WorldError> {
    if let Ok(error) = serde_cbor::from_slice::<proto_distributed::ErrorResponse>(bytes) {
        return Err(WorldError::NetworkRequestFailed {
            code: error.code,
            message: error.message,
            retryable: error.retryable,
        });
    }
    Ok(serde_cbor::from_slice(bytes)?)
}

fn to_canonical_cbor<T: Serialize>(value: &T) -> Result<Vec<u8>, WorldError> {
    let mut buf = Vec::with_capacity(256);
    let canonical_value = serde_cbor::value::to_value(value)?;
    let mut serializer = serde_cbor::ser::Serializer::new(&mut buf);
    serializer.self_describe()?;
    canonical_value.serialize(&mut serializer)?;
    Ok(buf)
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(feature = "libp2p")]
pub use agent_world::runtime::{Libp2pNetwork, Libp2pNetworkConfig};

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use agent_world::runtime::{ModuleKind, ModuleLimits, ModuleRole};
    use agent_world_proto::distributed_dht::DistributedDht as _;
    use agent_world_proto::distributed_net::DistributedNetwork as _;

    use super::*;

    #[test]
    fn net_exports_are_available() {
        let _ = std::any::type_name::<NetworkMessage>();
        let _ = std::any::type_name::<DistributedClient>();
        let _ = std::any::type_name::<SubmitActionReceipt>();
        let _ = std::any::type_name::<HeadFollower>();
    }

    #[test]
    fn in_memory_publish_delivers_to_subscribers() {
        let network = InMemoryNetwork::new();
        let subscription = network.subscribe("aw.w1.action").expect("subscribe");

        network
            .publish("aw.w1.action", b"payload")
            .expect("publish");

        let messages = subscription.drain();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], b"payload".to_vec());
    }

    fn sample_action() -> proto_distributed::ActionEnvelope {
        proto_distributed::ActionEnvelope {
            world_id: "w1".to_string(),
            action_id: "a1".to_string(),
            actor_id: "actor-1".to_string(),
            action_kind: "test".to_string(),
            payload_cbor: vec![1, 2, 3],
            payload_hash: "hash".to_string(),
            nonce: 1,
            timestamp_ms: 10,
            signature: "sig".to_string(),
        }
    }

    #[test]
    fn gateway_publishes_action() {
        let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
        let subscription = network.subscribe("aw.w1.action").expect("subscribe");
        let gateway = NetworkGateway::new_with_clock(Arc::clone(&network), Arc::new(|| 1234));

        let receipt = gateway.submit_action(sample_action()).expect("submit");
        assert_eq!(receipt.action_id, "a1");
        assert_eq!(receipt.accepted_at_ms, 1234);

        let messages = subscription.drain();
        assert_eq!(messages.len(), 1);
        let decoded: proto_distributed::ActionEnvelope =
            serde_cbor::from_slice(&messages[0]).expect("decode");
        assert_eq!(decoded.action_id, "a1");
    }

    #[test]
    fn in_memory_request_invokes_handler() {
        let network = InMemoryNetwork::new();
        network
            .register_handler(
                "/aw/rr/1.0.0/get_world_head",
                Box::new(|payload| {
                    let mut out = payload.to_vec();
                    out.extend_from_slice(b"-ok");
                    Ok(out)
                }),
            )
            .expect("register handler");

        let response = network
            .request("/aw/rr/1.0.0/get_world_head", b"ping")
            .expect("request");
        assert_eq!(response, b"ping-ok".to_vec());
    }

    #[test]
    fn in_memory_dht_stores_providers() {
        let dht = InMemoryDht::new();
        dht.publish_provider("w1", "hash", "peer-1")
            .expect("publish provider");
        dht.publish_provider("w1", "hash", "peer-2")
            .expect("publish provider");

        let providers = dht.get_providers("w1", "hash").expect("get providers");
        assert_eq!(providers.len(), 2);
    }

    #[test]
    fn in_memory_dht_tracks_world_head() {
        let dht = InMemoryDht::new();
        let head = WorldHeadAnnounce {
            world_id: "w1".to_string(),
            height: 1,
            block_hash: "b1".to_string(),
            state_root: "s1".to_string(),
            timestamp_ms: 1,
            signature: "sig".to_string(),
        };
        dht.put_world_head("w1", &head).expect("put head");

        let loaded = dht.get_world_head("w1").expect("get head");
        assert_eq!(loaded, Some(head));
    }

    #[test]
    fn in_memory_dht_tracks_membership_directory_snapshot() {
        let dht = InMemoryDht::new();
        let snapshot = MembershipDirectorySnapshot {
            world_id: "w1".to_string(),
            requester_id: "seq-1".to_string(),
            requested_at_ms: 1,
            reason: Some("bootstrap".to_string()),
            validators: vec![
                "seq-1".to_string(),
                "seq-2".to_string(),
                "seq-3".to_string(),
            ],
            quorum_threshold: 2,
            signature_key_id: Some("k1".to_string()),
            signature: Some("deadbeef".to_string()),
        };
        dht.put_membership_directory("w1", &snapshot)
            .expect("put membership");

        let loaded = dht.get_membership_directory("w1").expect("get membership");
        assert_eq!(loaded, Some(snapshot));
    }

    #[derive(Default)]
    struct SpyNetwork {
        providers: Arc<Mutex<Vec<String>>>,
        blobs: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    }

    impl SpyNetwork {
        fn providers(&self) -> Vec<String> {
            self.providers.lock().expect("lock providers").clone()
        }

        fn set_blob(&self, content_hash: &str, bytes: Vec<u8>) {
            let mut blobs = self.blobs.lock().expect("lock blobs");
            blobs.insert(content_hash.to_string(), bytes);
        }
    }

    impl proto_net::DistributedNetwork<WorldError> for SpyNetwork {
        fn publish(&self, _topic: &str, _payload: &[u8]) -> Result<(), WorldError> {
            Ok(())
        }

        fn subscribe(&self, _topic: &str) -> Result<NetworkSubscription, WorldError> {
            Err(WorldError::NetworkProtocolUnavailable {
                protocol: "spy".to_string(),
            })
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
            let mut captured = self.providers.lock().expect("lock providers");
            *captured = providers.to_vec();

            match protocol {
                proto_distributed::RR_FETCH_BLOB => {
                    let request: proto_distributed::FetchBlobRequest =
                        serde_cbor::from_slice(payload)?;
                    let blob = self
                        .blobs
                        .lock()
                        .expect("lock blobs")
                        .get(&request.content_hash)
                        .cloned()
                        .unwrap_or_else(|| b"data".to_vec());
                    let response = proto_distributed::FetchBlobResponse {
                        blob,
                        content_hash: request.content_hash,
                    };
                    Ok(to_canonical_cbor(&response)?)
                }
                proto_distributed::RR_GET_MODULE_MANIFEST => {
                    let request: proto_distributed::GetModuleManifestRequest =
                        serde_cbor::from_slice(payload)?;
                    let response = proto_distributed::GetModuleManifestResponse {
                        manifest_ref: proto_distributed::BlobRef {
                            content_hash: request.manifest_hash,
                            size_bytes: 0,
                            codec: "raw".to_string(),
                            links: Vec::new(),
                        },
                    };
                    Ok(to_canonical_cbor(&response)?)
                }
                proto_distributed::RR_GET_MODULE_ARTIFACT => {
                    let request: proto_distributed::GetModuleArtifactRequest =
                        serde_cbor::from_slice(payload)?;
                    let response = proto_distributed::GetModuleArtifactResponse {
                        artifact_ref: proto_distributed::BlobRef {
                            content_hash: request.wasm_hash,
                            size_bytes: 0,
                            codec: "raw".to_string(),
                            links: Vec::new(),
                        },
                    };
                    Ok(to_canonical_cbor(&response)?)
                }
                _ => Err(WorldError::NetworkProtocolUnavailable {
                    protocol: protocol.to_string(),
                }),
            }
        }

        fn register_handler(
            &self,
            _protocol: &str,
            _handler: Box<dyn Fn(&[u8]) -> Result<Vec<u8>, WorldError> + Send + Sync>,
        ) -> Result<(), WorldError> {
            Ok(())
        }
    }

    #[test]
    fn client_get_world_head_round_trip() {
        let network = InMemoryNetwork::new();
        network
            .register_handler(
                proto_distributed::RR_GET_WORLD_HEAD,
                Box::new(|payload| {
                    let request: proto_distributed::GetWorldHeadRequest =
                        serde_cbor::from_slice(payload).unwrap();
                    assert_eq!(request.world_id, "w1");
                    let response = proto_distributed::GetWorldHeadResponse {
                        head: WorldHeadAnnounce {
                            world_id: "w1".to_string(),
                            height: 7,
                            block_hash: "b1".to_string(),
                            state_root: "s1".to_string(),
                            timestamp_ms: 123,
                            signature: "sig".to_string(),
                        },
                    };
                    Ok(to_canonical_cbor(&response).unwrap())
                }),
            )
            .expect("register handler");

        let client = DistributedClient::new(Arc::new(network));
        let head = client.get_world_head("w1").expect("get world head");
        assert_eq!(head.height, 7);
    }

    #[test]
    fn client_maps_error_response() {
        let network = InMemoryNetwork::new();
        network
            .register_handler(
                proto_distributed::RR_FETCH_BLOB,
                Box::new(|_payload| {
                    let response = proto_distributed::ErrorResponse {
                        code: proto_distributed::DistributedErrorCode::ErrNotFound,
                        message: "missing".to_string(),
                        retryable: false,
                    };
                    Ok(to_canonical_cbor(&response).unwrap())
                }),
            )
            .expect("register handler");

        let client = DistributedClient::new(Arc::new(network));
        let err = client.fetch_blob("missing").expect_err("expect error");
        assert!(matches!(err, WorldError::NetworkRequestFailed { .. }));
    }

    #[test]
    fn client_fetch_blob_with_providers_passes_list() {
        let spy = Arc::new(SpyNetwork::default());
        let network: Arc<dyn DistributedNetwork + Send + Sync> = spy.clone();
        let client = DistributedClient::new(network);
        let providers = vec!["p1".to_string(), "p2".to_string()];
        let blob = client
            .fetch_blob_with_providers("hash", &providers)
            .expect("fetch");
        assert_eq!(blob, b"data".to_vec());

        let seen = spy.providers();
        assert_eq!(seen, providers);
    }

    #[test]
    fn client_fetch_blob_from_dht_uses_provider_list() {
        let spy = Arc::new(SpyNetwork::default());
        let network: Arc<dyn DistributedNetwork + Send + Sync> = spy.clone();
        let client = DistributedClient::new(network);
        let dht = InMemoryDht::new();
        dht.publish_provider("w1", "hash", "peer-1")
            .expect("publish provider");

        let blob = client
            .fetch_blob_from_dht("w1", "hash", &dht)
            .expect("fetch");
        assert_eq!(blob, b"data".to_vec());

        let seen = spy.providers();
        assert_eq!(seen, vec!["peer-1".to_string()]);
    }

    #[test]
    fn client_fetch_module_manifest_from_dht_uses_provider_list() {
        let spy = Arc::new(SpyNetwork::default());
        let network: Arc<dyn DistributedNetwork + Send + Sync> = spy.clone();
        let client = DistributedClient::new(network);
        let dht = InMemoryDht::new();
        dht.publish_provider("w1", "manifest-hash", "peer-9")
            .expect("publish provider");

        let manifest = ModuleManifest {
            module_id: "m.weather".to_string(),
            name: "Weather".to_string(),
            version: "0.1.0".to_string(),
            kind: ModuleKind::Reducer,
            role: ModuleRole::Rule,
            wasm_hash: "wasm-hash".to_string(),
            interface_version: "v1".to_string(),
            exports: vec![],
            subscriptions: vec![],
            required_caps: vec![],
            limits: ModuleLimits::default(),
        };
        let bytes = to_canonical_cbor(&manifest).expect("cbor");
        spy.set_blob("manifest-hash", bytes);

        let loaded = client
            .fetch_module_manifest_from_dht("w1", "m.weather", "manifest-hash", &dht)
            .expect("fetch manifest");
        assert_eq!(loaded.module_id, "m.weather");

        let seen = spy.providers();
        assert_eq!(seen, vec!["peer-9".to_string()]);
    }

    #[test]
    fn client_fetch_module_artifact_from_dht_uses_provider_list() {
        let spy = Arc::new(SpyNetwork::default());
        let network: Arc<dyn DistributedNetwork + Send + Sync> = spy.clone();
        let client = DistributedClient::new(network);
        let dht = InMemoryDht::new();
        dht.publish_provider("w1", "wasm-hash", "peer-7")
            .expect("publish provider");

        let artifact = client
            .fetch_module_artifact_from_dht("w1", "wasm-hash", &dht)
            .expect("fetch artifact");
        assert_eq!(artifact.wasm_hash, "wasm-hash");
        assert_eq!(artifact.bytes, b"data".to_vec());

        let seen = spy.providers();
        assert_eq!(seen, vec!["peer-7".to_string()]);
    }
}
