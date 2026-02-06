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
    let report = observer
        .sync_heads_report(&subscription, &mut follower, &client, &store)
        .expect("sync");
    assert_eq!(report.drained, 1);
    let result = report.applied.expect("result");
    assert_eq!(result.head, write.head_announce);
    assert_eq!(result.world.state(), &snapshot.state);

    let payload = serde_cbor::to_vec(&write.head_announce).expect("head cbor");
    network
        .publish(&topic_head("w1"), &payload)
        .expect("publish head duplicate");
    let duplicate = observer
        .sync_heads_report(&subscription, &mut follower, &client, &store)
        .expect("sync duplicate");
    assert_eq!(duplicate.drained, 1);
    assert!(duplicate.applied.is_none());

    let follow_empty = observer
        .follow_heads(&subscription, &mut follower, &client, &store, 2)
        .expect("follow empty");
    assert_eq!(follow_empty.rounds, 1);
    assert_eq!(follow_empty.drained, 0);
    assert!(follow_empty.applied.is_none());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn observer_follow_heads_reports_last_applied() {
    let dir = temp_dir("observer-follow");
    let store = LocalCasStore::new(&dir);
    let mut world = World::new();

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: GeoPos::new(0.0, 0.0, 0.0),
    });
    world.step().expect("step world");
    let snapshot1 = world.snapshot();
    let journal1 = world.journal().clone();
    let write1 = store_execution_result(
        "w1",
        1,
        "genesis",
        "exec-1",
        1,
        &snapshot1,
        &journal1,
        &store,
        ExecutionWriteConfig::default(),
    )
    .expect("write1");

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-2".to_string(),
        pos: GeoPos::new(1.0, 0.0, 0.0),
    });
    world.step().expect("step world");
    let snapshot2 = world.snapshot();
    let journal2 = world.journal().clone();
    let write2 = store_execution_result(
        "w1",
        2,
        &write1.block_hash,
        "exec-1",
        2,
        &snapshot2,
        &journal2,
        &store,
        ExecutionWriteConfig::default(),
    )
    .expect("write2");

    let network: Arc<dyn agent_world::runtime::DistributedNetwork + Send + Sync> =
        Arc::new(InMemoryNetwork::new());
    let observer = ObserverClient::new(Arc::clone(&network));
    let subscription = observer.subscribe("w1").expect("subscribe");
    let client = DistributedClient::new(Arc::clone(&network));

    let block1 = write1.block.clone();
    let block2 = write2.block.clone();
    let snapshot1_ref = write1.snapshot_manifest_ref.content_hash.clone();
    let snapshot2_ref = write2.snapshot_manifest_ref.content_hash.clone();
    let journal1_ref = write1.journal_segments_ref.content_hash.clone();
    let journal2_ref = write2.journal_segments_ref.content_hash.clone();
    let store_clone = store.clone();

    network
        .register_handler(RR_GET_BLOCK, Box::new(move |payload| {
            let request: GetBlockRequest = serde_cbor::from_slice(payload).unwrap();
            let response = match request.height {
                1 => GetBlockResponse {
                    block: block1.clone(),
                    journal_ref: journal1_ref.clone(),
                    snapshot_ref: snapshot1_ref.clone(),
                },
                2 => GetBlockResponse {
                    block: block2.clone(),
                    journal_ref: journal2_ref.clone(),
                    snapshot_ref: snapshot2_ref.clone(),
                },
                _ => {
                    return Err(WorldError::DistributedValidationFailed {
                        reason: format!("unknown block height {}", request.height),
                    })
                }
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

    let payload1 = serde_cbor::to_vec(&write1.head_announce).expect("head1 cbor");
    let payload2 = serde_cbor::to_vec(&write2.head_announce).expect("head2 cbor");
    network
        .publish(&topic_head("w1"), &payload1)
        .expect("publish head1");
    network
        .publish(&topic_head("w1"), &payload2)
        .expect("publish head2");

    let mut follower = HeadFollower::new("w1");
    let report = observer
        .follow_heads(&subscription, &mut follower, &client, &store, 3)
        .expect("follow");
    assert_eq!(report.drained, 2);
    let applied = report.applied.expect("applied");
    assert_eq!(applied.head, write2.head_announce);
    assert_eq!(applied.world.state(), &snapshot2.state);
    assert!(report.rounds >= 1);

    let _ = fs::remove_dir_all(&dir);
}
