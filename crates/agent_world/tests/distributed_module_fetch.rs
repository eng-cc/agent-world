use std::sync::Arc;

use agent_world::runtime::distributed::{
    BlobRef, FetchBlobRequest, FetchBlobResponse, GetModuleArtifactRequest,
    GetModuleArtifactResponse, RR_FETCH_BLOB, RR_GET_MODULE_ARTIFACT,
};
use agent_world::{
    DistributedClient, DistributedDht, DistributedNetwork, InMemoryDht, InMemoryNetwork, Manifest,
    ModuleActivation, ModuleChangeSet, ModuleKind, ModuleLimits, ModuleManifest, ModuleRole,
    ModuleSubscription, ModuleSubscriptionStage, ProposalDecision, World,
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
        .register_handler(
            RR_GET_MODULE_ARTIFACT,
            Box::new(move |payload| {
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
            }),
        )
        .expect("register artifact handler");

    let wasm_bytes_for_fetch = wasm_bytes.clone();
    let wasm_hash_for_fetch = wasm_hash.clone();
    network
        .register_handler(
            RR_FETCH_BLOB,
            Box::new(move |payload| {
                let request: FetchBlobRequest = serde_cbor::from_slice(payload).unwrap();
                assert_eq!(request.content_hash, wasm_hash_for_fetch);
                let response = FetchBlobResponse {
                    blob: wasm_bytes_for_fetch.clone(),
                    content_hash: request.content_hash,
                };
                Ok(serde_cbor::to_vec(&response).unwrap())
            }),
        )
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

#[test]
fn prefetch_active_modules_with_fetch_downloads_artifacts() {
    let wasm_bytes = b"prefetch-wasm".to_vec();
    let mut hasher = Sha256::new();
    hasher.update(&wasm_bytes);
    let wasm_hash = hex::encode(hasher.finalize());

    let module_manifest = ModuleManifest {
        module_id: "m.prefetch".to_string(),
        name: "Prefetch".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash: wasm_hash.clone(),
        interface_version: "wasm-1".to_string(),
        exports: vec!["reduce".to_string()],
        subscriptions: vec![ModuleSubscription {
            event_kinds: vec!["tick".to_string()],
            action_kinds: Vec::new(),
            stage: Some(ModuleSubscriptionStage::PostEvent),
            filters: None,
        }],
        required_caps: Vec::new(),
        limits: ModuleLimits::default(),
    };

    let changes = ModuleChangeSet {
        register: vec![module_manifest.clone()],
        activate: vec![ModuleActivation {
            module_id: module_manifest.module_id.clone(),
            version: module_manifest.version.clone(),
        }],
        ..ModuleChangeSet::default()
    };

    let mut content = serde_json::Map::new();
    content.insert(
        "module_changes".to_string(),
        serde_json::to_value(&changes).unwrap(),
    );
    let manifest = Manifest {
        version: 1,
        content: serde_json::Value::Object(content),
    };

    let mut world = World::new();
    world
        .register_module_artifact(wasm_hash.clone(), &wasm_bytes)
        .expect("register artifact");
    let proposal_id = world
        .propose_manifest_update(manifest, "alice")
        .expect("propose");
    world.shadow_proposal(proposal_id).expect("shadow");
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .expect("approve");
    world.apply_proposal(proposal_id).expect("apply");

    let serialized = serde_cbor::to_vec(&world).expect("serialize");
    let mut world: World = serde_cbor::from_slice(&serialized).expect("deserialize");
    assert_eq!(world.module_cache_len(), 0);

    let network = InMemoryNetwork::new();
    let wasm_hash_for_artifact = wasm_hash.clone();
    network
        .register_handler(
            RR_GET_MODULE_ARTIFACT,
            Box::new(move |payload| {
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
            }),
        )
        .expect("register artifact handler");

    let wasm_bytes_for_fetch = wasm_bytes.clone();
    let wasm_hash_for_fetch = wasm_hash.clone();
    network
        .register_handler(
            RR_FETCH_BLOB,
            Box::new(move |payload| {
                let request: FetchBlobRequest = serde_cbor::from_slice(payload).unwrap();
                assert_eq!(request.content_hash, wasm_hash_for_fetch);
                let response = FetchBlobResponse {
                    blob: wasm_bytes_for_fetch.clone(),
                    content_hash: request.content_hash,
                };
                Ok(serde_cbor::to_vec(&response).unwrap())
            }),
        )
        .expect("register fetch handler");

    let client = DistributedClient::new(Arc::new(network));
    let dht = InMemoryDht::new();
    dht.publish_provider("w1", &wasm_hash, "peer-1")
        .expect("publish provider");

    let loaded = world
        .prefetch_active_modules_with_fetch("w1", &client, &dht)
        .expect("prefetch");
    assert_eq!(loaded, 1);
    assert_eq!(world.module_cache_len(), 1);

    let artifact = world.load_module(&wasm_hash).expect("load");
    assert_eq!(artifact.bytes, wasm_bytes);
}

#[test]
fn shadow_and_apply_with_fetch_downloads_artifact() {
    let wasm_bytes = b"governance-fetch-wasm".to_vec();
    let mut hasher = Sha256::new();
    hasher.update(&wasm_bytes);
    let wasm_hash = hex::encode(hasher.finalize());

    let module_manifest = ModuleManifest {
        module_id: "m.gov".to_string(),
        name: "Gov".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Domain,
        wasm_hash: wasm_hash.clone(),
        interface_version: "wasm-1".to_string(),
        exports: vec!["reduce".to_string()],
        subscriptions: vec![ModuleSubscription {
            event_kinds: vec!["tick".to_string()],
            action_kinds: Vec::new(),
            stage: Some(ModuleSubscriptionStage::PostEvent),
            filters: None,
        }],
        required_caps: Vec::new(),
        limits: ModuleLimits::default(),
    };

    let changes = ModuleChangeSet {
        register: vec![module_manifest.clone()],
        activate: vec![ModuleActivation {
            module_id: module_manifest.module_id.clone(),
            version: module_manifest.version.clone(),
        }],
        ..ModuleChangeSet::default()
    };

    let mut content = serde_json::Map::new();
    content.insert(
        "module_changes".to_string(),
        serde_json::to_value(&changes).unwrap(),
    );
    let manifest = Manifest {
        version: 1,
        content: serde_json::Value::Object(content),
    };

    let network = InMemoryNetwork::new();
    let wasm_hash_for_artifact = wasm_hash.clone();
    network
        .register_handler(
            RR_GET_MODULE_ARTIFACT,
            Box::new(move |payload| {
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
            }),
        )
        .expect("register artifact handler");

    let wasm_bytes_for_fetch = wasm_bytes.clone();
    let wasm_hash_for_fetch = wasm_hash.clone();
    network
        .register_handler(
            RR_FETCH_BLOB,
            Box::new(move |payload| {
                let request: FetchBlobRequest = serde_cbor::from_slice(payload).unwrap();
                assert_eq!(request.content_hash, wasm_hash_for_fetch);
                let response = FetchBlobResponse {
                    blob: wasm_bytes_for_fetch.clone(),
                    content_hash: request.content_hash,
                };
                Ok(serde_cbor::to_vec(&response).unwrap())
            }),
        )
        .expect("register fetch handler");

    let client = DistributedClient::new(Arc::new(network));
    let dht = InMemoryDht::new();
    dht.publish_provider("w1", &wasm_hash, "peer-1")
        .expect("publish provider");

    let mut world = World::new();
    let proposal_id = world
        .propose_manifest_update(manifest, "alice")
        .expect("propose");
    world
        .shadow_proposal_with_fetch(proposal_id, "w1", &client, &dht)
        .expect("shadow");
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .expect("approve");
    world
        .apply_proposal_with_fetch(proposal_id, "w1", &client, &dht)
        .expect("apply");

    let active = world.module_registry().active.get("m.gov");
    assert_eq!(active.map(String::as_str), Some("0.1.0"));
    let artifact = world.load_module(&wasm_hash).expect("load");
    assert_eq!(artifact.bytes, wasm_bytes);
}
