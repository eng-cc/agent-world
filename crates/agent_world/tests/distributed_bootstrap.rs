use std::sync::Arc;

use agent_world::runtime::distributed::{
    FetchBlobRequest, FetchBlobResponse, GetBlockRequest, GetBlockResponse, RR_FETCH_BLOB,
    RR_GET_BLOCK,
};
use agent_world::runtime::{store_execution_result, ExecutionWriteConfig};
use agent_world::Action;
use agent_world::{
    BlobStore, DistributedClient, DistributedDht, DistributedNetwork, InMemoryDht, InMemoryNetwork,
    LocalCasStore, World,
};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn bootstrap_world_from_dht_round_trip() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("agent-world-bootstrap-{unique}"));
    let store = LocalCasStore::new(&dir);

    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: agent_world::GeoPos::new(0.0, 0.0, 0.0),
    });
    world.step().expect("step world");

    let snapshot = world.snapshot();
    let journal = world.journal().clone();
    let write = store_execution_result(
        "w1",
        1,
        "genesis",
        "exec-1",
        1,
        &snapshot,
        &journal,
        &store,
        ExecutionWriteConfig::default(),
    )
    .expect("write");

    let network = InMemoryNetwork::new();
    let write_block = write.block.clone();
    let write_snapshot_ref = write.snapshot_manifest_ref.content_hash.clone();
    let write_journal_ref = write.journal_segments_ref.content_hash.clone();
    network
        .register_handler(
            RR_GET_BLOCK,
            Box::new(move |payload| {
                let request: GetBlockRequest = serde_cbor::from_slice(payload).unwrap();
                assert_eq!(request.world_id, "w1");
                let response = GetBlockResponse {
                    block: write_block.clone(),
                    journal_ref: write_journal_ref.clone(),
                    snapshot_ref: write_snapshot_ref.clone(),
                };
                Ok(serde_cbor::to_vec(&response).unwrap())
            }),
        )
        .expect("register block");

    let store_clone = store.clone();
    network
        .register_handler(
            RR_FETCH_BLOB,
            Box::new(move |payload| {
                let request: FetchBlobRequest = serde_cbor::from_slice(payload).unwrap();
                let bytes = store_clone.get(&request.content_hash).unwrap();
                let response = FetchBlobResponse {
                    blob: bytes,
                    content_hash: request.content_hash,
                };
                Ok(serde_cbor::to_vec(&response).unwrap())
            }),
        )
        .expect("register fetch");

    let client = DistributedClient::new(Arc::new(network));
    let dht = InMemoryDht::new();
    dht.put_world_head("w1", &write.head_announce)
        .expect("put head");

    let bootstrapped = agent_world::runtime::bootstrap_world_from_dht("w1", &dht, &client, &store)
        .expect("bootstrap");
    assert_eq!(bootstrapped.journal().len(), journal.len());
    assert_eq!(bootstrapped.manifest().version, snapshot.manifest.version);

    let _ = std::fs::remove_dir_all(&dir);
}
