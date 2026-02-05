use agent_world::geometry::GeoPos;
use agent_world::runtime::distributed::{
    topic_head, FetchBlobRequest, FetchBlobResponse, GetBlockRequest, GetBlockResponse,
    RR_FETCH_BLOB, RR_GET_BLOCK,
};
use agent_world::runtime::{
    store_execution_result, Action, BlobStore, DistributedClient, ExecutionWriteConfig,
    HeadFollower, InMemoryNetwork, LocalCasStore, ObserverClient, World, WorldError,
};
use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(prefix: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("agent-world-{prefix}-{unique}"))
}

#[test]
fn observer_sync_heads_bootstraps_world() {
    let dir = temp_dir("observer-sync");
    let store = LocalCasStore::new(&dir);
    let mut world = World::new();

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: GeoPos::new(0.0, 0.0, 0.0),
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

    let network: Arc<dyn agent_world::runtime::DistributedNetwork + Send + Sync> =
        Arc::new(InMemoryNetwork::new());
    let observer = ObserverClient::new(Arc::clone(&network));
    let subscription = observer.subscribe("w1").expect("subscribe");
    let client = DistributedClient::new(Arc::clone(&network));

    let block = write.block.clone();
    let snapshot_ref = write.snapshot_manifest_ref.content_hash.clone();
    let journal_ref = write.journal_segments_ref.content_hash.clone();
    let store_clone = store.clone();

    network
        .register_handler(RR_GET_BLOCK, Box::new(move |payload| {
            let request: GetBlockRequest = serde_cbor::from_slice(payload).unwrap();
            if request.height != 1 {
                return Err(WorldError::DistributedValidationFailed {
                    reason: format!("unknown block height {}", request.height),
                });
            }
            let response = GetBlockResponse {
                block: block.clone(),
                journal_ref: journal_ref.clone(),
                snapshot_ref: snapshot_ref.clone(),
            };
            Ok(serde_cbor::to_vec(&response).unwrap())
        }))
        .expect("register block");

    network
        .register_handler(RR_FETCH_BLOB, Box::new(move |payload| {
            let request: FetchBlobRequest = serde_cbor::from_slice(payload).unwrap();
            let bytes = store_clone.get(&request.content_hash).unwrap();
            let response = FetchBlobResponse {
                blob: bytes,
                content_hash: request.content_hash,
            };
            Ok(serde_cbor::to_vec(&response).unwrap())
        }))
        .expect("register blob");

    let payload = serde_cbor::to_vec(&write.head_announce).expect("head cbor");
    network
        .publish(&topic_head("w1"), &payload)
        .expect("publish head");

    let mut follower = HeadFollower::new("w1");
    let synced = observer
        .sync_heads(&subscription, &mut follower, &client, &store)
        .expect("sync")
        .expect("world");
    assert_eq!(synced.state(), &snapshot.state);

    let _ = fs::remove_dir_all(&dir);
}
