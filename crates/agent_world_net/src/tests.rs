use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use agent_world_proto::distributed as proto_distributed;
use agent_world_proto::distributed_dht::DistributedDht as _;
use agent_world_proto::distributed_net as proto_net;
use agent_world_proto::distributed_net::DistributedNetwork as _;
use agent_world_proto::distributed_storage as proto_storage;

use super::*;
use crate::util::to_canonical_cbor;

#[test]
fn net_exports_are_available() {
    let _ = std::any::type_name::<NetworkMessage>();
    let _ = std::any::type_name::<DistributedClient>();
    let _ = std::any::type_name::<CachedDht>();
    let _ = std::any::type_name::<DhtCacheConfig>();
    let _ = std::any::type_name::<dyn DistributedIndexStore>();
    let _ = std::any::type_name::<HeadIndexRecord>();
    let _ = std::any::type_name::<InMemoryIndexStore>();
    let _ = std::any::type_name::<ProviderCache>();
    let _ = std::any::type_name::<ProviderCacheConfig>();
    let _ = std::any::type_name::<IndexPublishResult>();
    let _ = std::any::type_name::<SubmitActionReceipt>();
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

#[cfg(feature = "libp2p")]
#[test]
fn libp2p_smoke_request_response_and_pubsub_work_between_peers() {
    use std::time::{Duration, Instant};

    use libp2p::Multiaddr;

    fn wait_until(what: &str, deadline: Instant, mut condition: impl FnMut() -> bool) {
        while Instant::now() < deadline {
            if condition() {
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        panic!("timed out waiting for condition: {what}");
    }

    let listen_addr: Multiaddr = "/ip4/127.0.0.1/tcp/0".parse().expect("multiaddr");
    let net1 = Libp2pNetwork::new(Libp2pNetworkConfig {
        listen_addrs: vec![listen_addr],
        ..Libp2pNetworkConfig::default()
    });

    let deadline = Instant::now() + Duration::from_secs(10);
    wait_until("net1 listening addrs", deadline, || {
        !net1.listening_addrs().is_empty()
    });
    let dial_addr = net1
        .listening_addrs()
        .into_iter()
        .find(|addr| addr.to_string().contains("127.0.0.1"))
        .expect("listening addr")
        .with(libp2p::multiaddr::Protocol::P2p(net1.peer_id().into()));

    net1.register_handler(
        "/aw/rr/1.0.0/ping",
        Box::new(|payload| {
            let mut out = payload.to_vec();
            out.extend_from_slice(b"-ok");
            Ok(out)
        }),
    )
    .expect("register handler");

    let net2 = Libp2pNetwork::new(Libp2pNetworkConfig {
        listen_addrs: vec!["/ip4/127.0.0.1/tcp/0".parse().expect("listen")],
        bootstrap_peers: vec![dial_addr],
        ..Libp2pNetworkConfig::default()
    });

    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if !net2.connected_peers().is_empty() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    if net2.connected_peers().is_empty() {
        panic!(
            "timed out waiting for net2 connected peers; net2_errors={:?}; net1_errors={:?}; net1_addrs={:?}",
            net2.debug_errors(),
            net1.debug_errors(),
            net1.listening_addrs(),
        );
    }

    let deadline = Instant::now() + Duration::from_secs(10);
    wait_until("request/response", deadline, || {
        match net2.request("/aw/rr/1.0.0/ping", b"ping") {
            Ok(reply) => reply == b"ping-ok".to_vec(),
            Err(WorldError::NetworkProtocolUnavailable { .. }) => false,
            Err(err) => panic!("unexpected request error: {err:?}"),
        }
    });

    let sub2 = net2.subscribe("aw.smoke").expect("sub2");
    let _sub1 = net1.subscribe("aw.smoke").expect("sub1");
    std::thread::sleep(Duration::from_millis(200));

    net1.publish("aw.smoke", b"hello").expect("publish");

    let deadline = Instant::now() + Duration::from_secs(10);
    wait_until("gossipsub deliver", deadline, || {
        sub2.drain().iter().any(|msg| msg == b"hello")
    });
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
    let head = proto_distributed::WorldHeadAnnounce {
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

fn sample_blob_ref(content_hash: &str) -> proto_distributed::BlobRef {
    proto_distributed::BlobRef {
        content_hash: content_hash.to_string(),
        size_bytes: 1,
        codec: "dag-cbor".to_string(),
        links: Vec::new(),
    }
}

fn sample_segment_ref(
    from_event_id: u64,
    to_event_id: u64,
    content_hash: &str,
) -> proto_storage::JournalSegmentRef {
    proto_storage::JournalSegmentRef {
        from_event_id,
        to_event_id,
        content_hash: content_hash.to_string(),
        size_bytes: 1,
    }
}

fn sample_write_result() -> ExecutionWriteResult {
    ExecutionWriteResult {
        block: proto_distributed::WorldBlock {
            world_id: "w1".to_string(),
            height: 1,
            prev_block_hash: "genesis".to_string(),
            action_root: "action-root".to_string(),
            event_root: "event-root".to_string(),
            state_root: "state-root".to_string(),
            journal_ref: "journal-index".to_string(),
            snapshot_ref: "snapshot-manifest".to_string(),
            receipts_root: "receipts-root".to_string(),
            proposer_id: "node-1".to_string(),
            timestamp_ms: 1,
            signature: "sig".to_string(),
        },
        block_hash: "block-hash".to_string(),
        block_ref: sample_blob_ref("block-hash"),
        block_announce: proto_distributed::BlockAnnounce {
            world_id: "w1".to_string(),
            height: 1,
            block_hash: "block-hash".to_string(),
            prev_block_hash: "genesis".to_string(),
            state_root: "state-root".to_string(),
            event_root: "event-root".to_string(),
            timestamp_ms: 1,
            signature: "sig".to_string(),
        },
        head_announce: proto_distributed::WorldHeadAnnounce {
            world_id: "w1".to_string(),
            height: 1,
            block_hash: "block-hash".to_string(),
            state_root: "state-root".to_string(),
            timestamp_ms: 1,
            signature: "sig".to_string(),
        },
        snapshot_manifest: proto_distributed::SnapshotManifest {
            world_id: "w1".to_string(),
            epoch: 1,
            chunks: vec![
                proto_distributed::StateChunkRef {
                    chunk_id: "chunk-1".to_string(),
                    content_hash: "chunk-hash-1".to_string(),
                    size_bytes: 1,
                },
                proto_distributed::StateChunkRef {
                    chunk_id: "chunk-2".to_string(),
                    content_hash: "chunk-hash-2".to_string(),
                    size_bytes: 1,
                },
            ],
            state_root: "state-root".to_string(),
        },
        snapshot_manifest_ref: sample_blob_ref("snapshot-manifest"),
        journal_segments: vec![
            sample_segment_ref(1, 1, "journal-seg-1"),
            sample_segment_ref(2, 2, "journal-seg-2"),
        ],
        journal_segments_ref: sample_blob_ref("journal-index"),
    }
}

#[test]
fn publish_execution_providers_indexes_all_hashes() {
    let write = sample_write_result();
    let dht = InMemoryDht::new();
    let result =
        publish_execution_providers(&dht, "w1", "store-1", &write).expect("publish providers");
    assert!(result.published > 0);

    let mut expected = HashSet::new();
    expected.insert(write.block_ref.content_hash.clone());
    expected.insert(write.snapshot_manifest_ref.content_hash.clone());
    expected.insert(write.journal_segments_ref.content_hash.clone());
    for chunk in &write.snapshot_manifest.chunks {
        expected.insert(chunk.content_hash.clone());
    }
    for segment in &write.journal_segments {
        expected.insert(segment.content_hash.clone());
    }

    for hash in expected {
        let providers = query_providers(&dht, "w1", &hash).expect("get providers");
        assert!(!providers.is_empty());
        assert_eq!(providers[0].provider_id, "store-1");
    }
}

#[test]
fn publish_execution_providers_cached_indexes_all_hashes() {
    let write = sample_write_result();
    let dht = InMemoryDht::new();
    let index_store = InMemoryIndexStore::new();
    let cache = ProviderCache::new(
        Arc::new(dht.clone()),
        Arc::new(index_store),
        "store-1",
        ProviderCacheConfig::default(),
    );

    let result =
        publish_execution_providers_cached(&cache, "w1", &write).expect("publish providers");
    assert!(result.published > 0);

    for hash in [
        "block-hash",
        "snapshot-manifest",
        "journal-index",
        "chunk-hash-1",
        "chunk-hash-2",
        "journal-seg-1",
        "journal-seg-2",
    ] {
        let providers = query_providers(&dht, "w1", hash).expect("get providers");
        assert!(!providers.is_empty(), "missing providers for {hash}");
    }
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
                let request: proto_distributed::FetchBlobRequest = serde_cbor::from_slice(payload)?;
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
                    serde_cbor::from_slice(payload).expect("decode request");
                assert_eq!(request.world_id, "w1");
                let response = proto_distributed::GetWorldHeadResponse {
                    head: proto_distributed::WorldHeadAnnounce {
                        world_id: "w1".to_string(),
                        height: 7,
                        block_hash: "b1".to_string(),
                        state_root: "s1".to_string(),
                        timestamp_ms: 123,
                        signature: "sig".to_string(),
                    },
                };
                Ok(to_canonical_cbor(&response).expect("encode response"))
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
                Ok(to_canonical_cbor(&response).expect("encode"))
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
        kind: agent_world_wasm_abi::ModuleKind::Pure,
        role: agent_world_wasm_abi::ModuleRole::Domain,
        wasm_hash: "wasm-hash".to_string(),
        interface_version: "aw.abi.module.v1".to_string(),
        exports: vec!["call".to_string()],
        subscriptions: Vec::new(),
        required_caps: Vec::new(),
        artifact_identity: None,
        limits: agent_world_wasm_abi::ModuleLimits::unbounded(),
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
