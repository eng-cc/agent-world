//! Client helpers for distributed storage/network access.

use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::distributed::{
    BlobRef, ErrorResponse, FetchBlobRequest, FetchBlobResponse, GetBlockRequest, GetBlockResponse,
    GetJournalSegmentRequest, GetJournalSegmentResponse, GetModuleArtifactRequest,
    GetModuleArtifactResponse, GetModuleManifestRequest, GetModuleManifestResponse,
    GetReceiptSegmentRequest, GetReceiptSegmentResponse, GetSnapshotRequest, GetSnapshotResponse,
    GetWorldHeadRequest, GetWorldHeadResponse, SnapshotManifest, WorldBlock, WorldHeadAnnounce,
    RR_FETCH_BLOB, RR_GET_BLOCK, RR_GET_JOURNAL_SEGMENT, RR_GET_MODULE_ARTIFACT,
    RR_GET_MODULE_MANIFEST, RR_GET_RECEIPT_SEGMENT, RR_GET_SNAPSHOT, RR_GET_WORLD_HEAD,
};
use super::distributed_dht::DistributedDht;
use super::distributed_net::DistributedNetwork;
use super::error::WorldError;
use super::modules::ModuleArtifact;
use super::modules::ModuleManifest;
use super::util::to_canonical_cbor;

#[derive(Clone)]
pub struct DistributedClient {
    network: Arc<dyn DistributedNetwork + Send + Sync>,
}

impl DistributedClient {
    pub fn new(network: Arc<dyn DistributedNetwork + Send + Sync>) -> Self {
        Self { network }
    }

    pub fn get_world_head(&self, world_id: &str) -> Result<WorldHeadAnnounce, WorldError> {
        let request = GetWorldHeadRequest {
            world_id: world_id.to_string(),
        };
        let response: GetWorldHeadResponse = self.request(RR_GET_WORLD_HEAD, &request)?;
        Ok(response.head)
    }

    pub fn get_block(&self, world_id: &str, height: u64) -> Result<WorldBlock, WorldError> {
        Ok(self.get_block_response(world_id, height)?.block)
    }

    pub fn get_block_response(
        &self,
        world_id: &str,
        height: u64,
    ) -> Result<GetBlockResponse, WorldError> {
        let request = GetBlockRequest {
            world_id: world_id.to_string(),
            height,
        };
        self.request(RR_GET_BLOCK, &request)
    }

    pub fn get_snapshot_manifest(
        &self,
        world_id: &str,
        epoch: u64,
    ) -> Result<SnapshotManifest, WorldError> {
        let request = GetSnapshotRequest {
            world_id: world_id.to_string(),
            epoch,
        };
        let response: GetSnapshotResponse = self.request(RR_GET_SNAPSHOT, &request)?;
        Ok(response.manifest)
    }

    pub fn fetch_blob(&self, content_hash: &str) -> Result<Vec<u8>, WorldError> {
        let request = FetchBlobRequest {
            content_hash: content_hash.to_string(),
        };
        let response: FetchBlobResponse = self.request(RR_FETCH_BLOB, &request)?;
        Ok(response.blob)
    }

    pub fn fetch_blob_with_providers(
        &self,
        content_hash: &str,
        providers: &[String],
    ) -> Result<Vec<u8>, WorldError> {
        let request = FetchBlobRequest {
            content_hash: content_hash.to_string(),
        };
        let response: FetchBlobResponse =
            self.request_with_providers(RR_FETCH_BLOB, &request, providers)?;
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
    ) -> Result<BlobRef, WorldError> {
        let request = GetJournalSegmentRequest {
            world_id: world_id.to_string(),
            from_event_id,
        };
        let response: GetJournalSegmentResponse = self.request(RR_GET_JOURNAL_SEGMENT, &request)?;
        Ok(response.segment)
    }

    pub fn get_receipt_segment(
        &self,
        world_id: &str,
        from_event_id: u64,
    ) -> Result<BlobRef, WorldError> {
        let request = GetReceiptSegmentRequest {
            world_id: world_id.to_string(),
            from_event_id,
        };
        let response: GetReceiptSegmentResponse = self.request(RR_GET_RECEIPT_SEGMENT, &request)?;
        Ok(response.segment)
    }

    pub fn get_module_manifest(
        &self,
        module_id: &str,
        manifest_hash: &str,
    ) -> Result<BlobRef, WorldError> {
        let request = GetModuleManifestRequest {
            module_id: module_id.to_string(),
            manifest_hash: manifest_hash.to_string(),
        };
        let response: GetModuleManifestResponse = self.request(RR_GET_MODULE_MANIFEST, &request)?;
        Ok(response.manifest_ref)
    }

    pub fn get_module_artifact(&self, wasm_hash: &str) -> Result<BlobRef, WorldError> {
        let request = GetModuleArtifactRequest {
            wasm_hash: wasm_hash.to_string(),
        };
        let response: GetModuleArtifactResponse = self.request(RR_GET_MODULE_ARTIFACT, &request)?;
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

fn decode_response<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, WorldError> {
    if let Ok(error) = serde_cbor::from_slice::<ErrorResponse>(bytes) {
        return Err(WorldError::NetworkRequestFailed {
            code: error.code,
            message: error.message,
            retryable: error.retryable,
        });
    }
    Ok(serde_cbor::from_slice(bytes)?)
}

#[cfg(test)]
mod tests {
    use super::super::distributed_net::{InMemoryNetwork, NetworkSubscription};
    use super::*;
    use crate::runtime::distributed::DistributedErrorCode;
    use crate::runtime::InMemoryDht;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

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

    impl DistributedNetwork for SpyNetwork {
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
                RR_FETCH_BLOB => {
                    let request: FetchBlobRequest = serde_cbor::from_slice(payload)?;
                    let blob = self
                        .blobs
                        .lock()
                        .expect("lock blobs")
                        .get(&request.content_hash)
                        .cloned()
                        .unwrap_or_else(|| b"data".to_vec());
                    let response = FetchBlobResponse {
                        blob,
                        content_hash: request.content_hash,
                    };
                    Ok(to_canonical_cbor(&response)?)
                }
                RR_GET_MODULE_MANIFEST => {
                    let request: GetModuleManifestRequest = serde_cbor::from_slice(payload)?;
                    let response = GetModuleManifestResponse {
                        manifest_ref: BlobRef {
                            content_hash: request.manifest_hash,
                            size_bytes: 0,
                            codec: "raw".to_string(),
                            links: Vec::new(),
                        },
                    };
                    Ok(to_canonical_cbor(&response)?)
                }
                RR_GET_MODULE_ARTIFACT => {
                    let request: GetModuleArtifactRequest = serde_cbor::from_slice(payload)?;
                    let response = GetModuleArtifactResponse {
                        artifact_ref: BlobRef {
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
                RR_GET_WORLD_HEAD,
                Box::new(|payload| {
                    let request: GetWorldHeadRequest = serde_cbor::from_slice(payload).unwrap();
                    assert_eq!(request.world_id, "w1");
                    let response = GetWorldHeadResponse {
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
                RR_FETCH_BLOB,
                Box::new(|_payload| {
                    let response = ErrorResponse {
                        code: DistributedErrorCode::ErrNotFound,
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
            kind: super::super::ModuleKind::Reducer,
            role: super::super::ModuleRole::Rule,
            wasm_hash: "wasm-hash".to_string(),
            interface_version: "v1".to_string(),
            exports: vec![],
            subscriptions: vec![],
            required_caps: vec![],
            limits: super::super::ModuleLimits::default(),
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
