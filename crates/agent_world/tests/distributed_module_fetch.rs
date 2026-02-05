use std::sync::Arc;

use agent_world::runtime::distributed::{
    BlobRef, FetchBlobRequest, FetchBlobResponse, GetModuleArtifactRequest,
    GetModuleArtifactResponse, RR_FETCH_BLOB, RR_GET_MODULE_ARTIFACT,
};
use agent_world::{
    DistributedClient, DistributedDht, DistributedNetwork, InMemoryDht, InMemoryNetwork, World,
};
use sha2::{Digest, Sha256};

#[test]
fn load_module_with_fetch_downloads_artifact() {
    let wasm_bytes = b"fake-wasm".to_vec();
    let mut hasher = Sha256::new();
    hasher.update(&wasm_bytes);
    let wasm_hash = hex::encode(hasher.finalize());

    let network = InMemoryNetwork::new();
    let wasm_hash_for_artifact = wasm_hash.clone();
    network
        .register_handler(RR_GET_MODULE_ARTIFACT, Box::new(move |payload| {
            let request: GetModuleArtifactRequest = serde_cbor::from_slice(payload).unwrap();
            assert_eq!(request.wasm_hash, wasm_hash_for_artifact);
            let response = GetModuleArtifactResponse {
                artifact_ref: BlobRef {
                    content_hash: request.wasm_hash,
                    size_bytes: 0,
                    codec: "raw".to_string(),
                    links: Vec::new(),
                },
            };
            Ok(serde_cbor::to_vec(&response).unwrap())
        }))
        .expect("register artifact handler");

    let wasm_bytes_for_fetch = wasm_bytes.clone();
    let wasm_hash_for_fetch = wasm_hash.clone();
    network
        .register_handler(RR_FETCH_BLOB, Box::new(move |payload| {
            let request: FetchBlobRequest = serde_cbor::from_slice(payload).unwrap();
            assert_eq!(request.content_hash, wasm_hash_for_fetch);
            let response = FetchBlobResponse {
                blob: wasm_bytes_for_fetch.clone(),
                content_hash: request.content_hash,
            };
            Ok(serde_cbor::to_vec(&response).unwrap())
        }))
        .expect("register fetch handler");

    let client = DistributedClient::new(Arc::new(network));
    let dht = InMemoryDht::new();
    dht.publish_provider("w1", &wasm_hash, "peer-1")
        .expect("publish provider");

    let mut world = World::new();
    let artifact = world
        .load_module_with_fetch("w1", &wasm_hash, &client, &dht)
        .expect("load with fetch");
    assert_eq!(artifact.bytes, wasm_bytes);

    let cached = world.load_module(&wasm_hash).expect("cached load");
    assert_eq!(cached.bytes, artifact.bytes);
}
