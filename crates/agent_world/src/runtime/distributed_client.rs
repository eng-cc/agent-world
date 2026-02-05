//! Client helpers for distributed storage/network access.

use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::distributed::{
    BlobRef, ErrorResponse, FetchBlobRequest, FetchBlobResponse,
    GetBlockRequest, GetBlockResponse, GetJournalSegmentRequest, GetJournalSegmentResponse,
    GetModuleArtifactRequest, GetModuleArtifactResponse, GetModuleManifestRequest,
    GetModuleManifestResponse, GetReceiptSegmentRequest, GetReceiptSegmentResponse,
    GetSnapshotRequest, GetSnapshotResponse, GetWorldHeadRequest, GetWorldHeadResponse,
    SnapshotManifest, WorldBlock, WorldHeadAnnounce, RR_FETCH_BLOB, RR_GET_BLOCK,
    RR_GET_JOURNAL_SEGMENT, RR_GET_MODULE_ARTIFACT, RR_GET_MODULE_MANIFEST, RR_GET_RECEIPT_SEGMENT,
    RR_GET_SNAPSHOT, RR_GET_WORLD_HEAD,
};
use super::distributed_net::DistributedNetwork;
use super::error::WorldError;
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

    pub fn get_journal_segment(&self, world_id: &str, from_event_id: u64) -> Result<BlobRef, WorldError> {
        let request = GetJournalSegmentRequest {
            world_id: world_id.to_string(),
            from_event_id,
        };
        let response: GetJournalSegmentResponse =
            self.request(RR_GET_JOURNAL_SEGMENT, &request)?;
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
        let response: GetReceiptSegmentResponse =
            self.request(RR_GET_RECEIPT_SEGMENT, &request)?;
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
        let response: GetModuleManifestResponse =
            self.request(RR_GET_MODULE_MANIFEST, &request)?;
        Ok(response.manifest_ref)
    }

    pub fn get_module_artifact(&self, wasm_hash: &str) -> Result<BlobRef, WorldError> {
        let request = GetModuleArtifactRequest {
            wasm_hash: wasm_hash.to_string(),
        };
        let response: GetModuleArtifactResponse =
            self.request(RR_GET_MODULE_ARTIFACT, &request)?;
        Ok(response.artifact_ref)
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
    use super::*;
    use super::super::distributed_net::{InMemoryNetwork, NetworkSubscription};
    use crate::runtime::distributed::DistributedErrorCode;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct SpyNetwork {
        providers: Arc<Mutex<Vec<String>>>,
    }

    impl SpyNetwork {
        fn providers(&self) -> Vec<String> {
            self.providers.lock().expect("lock providers").clone()
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
            _protocol: &str,
            payload: &[u8],
            providers: &[String],
        ) -> Result<Vec<u8>, WorldError> {
            let mut captured = self.providers.lock().expect("lock providers");
            *captured = providers.to_vec();

            let request: FetchBlobRequest = serde_cbor::from_slice(payload)?;
            let response = FetchBlobResponse {
                blob: b"data".to_vec(),
                content_hash: request.content_hash,
            };
            Ok(to_canonical_cbor(&response)?)
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
            .register_handler(RR_GET_WORLD_HEAD, Box::new(|payload| {
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
            }))
            .expect("register handler");

        let client = DistributedClient::new(Arc::new(network));
        let head = client.get_world_head("w1").expect("get world head");
        assert_eq!(head.height, 7);
    }

    #[test]
    fn client_maps_error_response() {
        let network = InMemoryNetwork::new();
        network
            .register_handler(RR_FETCH_BLOB, Box::new(|_payload| {
                let response = ErrorResponse {
                    code: DistributedErrorCode::ErrNotFound,
                    message: "missing".to_string(),
                    retryable: false,
                };
                Ok(to_canonical_cbor(&response).unwrap())
            }))
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
}
