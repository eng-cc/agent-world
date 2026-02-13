use agent_world::geometry::GeoPos;
use agent_world::runtime::distributed::{
    FetchBlobRequest, FetchBlobResponse, GetBlockRequest, GetBlockResponse, RR_FETCH_BLOB,
    RR_GET_BLOCK,
};
use agent_world::runtime::{
    store_execution_result, Action, BlobStore, DistributedClient, ExecutionWriteConfig,
    HeadFollower, InMemoryNetwork, LocalCasStore, World,
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
fn head_follower_applies_and_ignores_heads() {
    let dir = temp_dir("head-follow");
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
    let block1 = write1.block.clone();
    let block2 = write2.block.clone();
    let block1_snapshot = write1.snapshot_manifest_ref.content_hash.clone();
    let block1_journal = write1.journal_segments_ref.content_hash.clone();
    let block2_snapshot = write2.snapshot_manifest_ref.content_hash.clone();
    let block2_journal = write2.journal_segments_ref.content_hash.clone();
    let store_clone = store.clone();

    network
        .register_handler(
            RR_GET_BLOCK,
            Box::new(move |payload| {
                let request: GetBlockRequest = serde_cbor::from_slice(payload).unwrap();
                let response = match request.height {
                    1 => GetBlockResponse {
                        block: block1.clone(),
                        journal_ref: block1_journal.clone(),
                        snapshot_ref: block1_snapshot.clone(),
                    },
                    2 => GetBlockResponse {
                        block: block2.clone(),
                        journal_ref: block2_journal.clone(),
                        snapshot_ref: block2_snapshot.clone(),
                    },
                    _ => {
                        return Err(agent_world_net::WorldError::DistributedValidationFailed {
                            reason: format!("unknown block height {}", request.height),
                        })
                    }
                };
                Ok(serde_cbor::to_vec(&response).unwrap())
            }),
        )
        .expect("register block handler");

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
        .expect("register blob handler");

    let client = DistributedClient::new(Arc::clone(&network));
    let mut follower = HeadFollower::new("w1");

    let world1 = follower
        .apply_head(&write1.head_announce, &client, &store)
        .expect("apply head1")
        .expect("world1");
    assert_eq!(world1.state(), &snapshot1.state);

    let duplicate = follower
        .apply_head(&write1.head_announce, &client, &store)
        .expect("apply head1 duplicate");
    assert!(duplicate.is_none());

    let mut conflict = write1.head_announce.clone();
    conflict.block_hash = "conflict".to_string();
    assert!(follower.apply_head(&conflict, &client, &store).is_err());

    let world2 = follower
        .apply_head(&write2.head_announce, &client, &store)
        .expect("apply head2")
        .expect("world2");
    assert_eq!(world2.state(), &snapshot2.state);

    let stale = follower
        .apply_head(&write1.head_announce, &client, &store)
        .expect("apply stale");
    assert!(stale.is_none());

    let _ = fs::remove_dir_all(&dir);
}
