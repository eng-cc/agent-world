use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use agent_world::runtime::distributed as dist;
use agent_world::runtime::{
    replay_validate_head, store_execution_result, ActionBatchRules, ActionGateway, ActionMempool,
    ActionMempoolConfig, DistributedClient, DistributedNetwork, ExecutionWriteConfig,
    InMemoryNetwork, NetworkGateway,
};
use agent_world::{Action, BlobStore, LocalCasStore, World};

fn temp_dir(prefix: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("agent-world-{prefix}-{unique}"))
}

#[test]
fn consistency_check_detects_corrupt_chunk() {
    let dir = temp_dir("consistency");
    let store = LocalCasStore::new(&dir);
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());

    let gateway = NetworkGateway::new_with_clock(Arc::clone(&network), Arc::new(|| 1000));
    let runtime_action = Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: agent_world::GeoPos::new(0.0, 0.0, 0.0),
    };
    let payload = serde_cbor::to_vec(&runtime_action).expect("encode action");
    let envelope = dist::ActionEnvelope {
        world_id: "w1".to_string(),
        action_id: "a1".to_string(),
        actor_id: "agent-1".to_string(),
        action_kind: "register_agent".to_string(),
        payload_cbor: payload.clone(),
        payload_hash: agent_world::blake3_hex(&payload),
        nonce: 1,
        timestamp_ms: 1000,
        signature: "sig".to_string(),
    };
    gateway.submit_action(envelope).expect("submit action");

    let subscription = network
        .subscribe(&dist::topic_action("w1"))
        .expect("subscribe");
    let messages = subscription.drain();
    let decoded: dist::ActionEnvelope =
        serde_cbor::from_slice(&messages[0]).expect("decode envelope");
    let mut mempool = ActionMempool::new(ActionMempoolConfig::default());
    mempool.add_action(decoded);
    let batch = mempool
        .take_batch_with_rules(
            "w1",
            "seq-1",
            ActionBatchRules {
                max_actions: 10,
                max_payload_bytes: 1024 * 1024,
            },
            1100,
        )
        .expect("batch")
        .expect("batch");

    let mut world = World::new();
    for action in &batch.actions {
        let runtime_action: Action =
            serde_cbor::from_slice(&action.payload_cbor).expect("decode action");
        world.submit_action(runtime_action);
    }
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
    .expect("store execution");

    // Corrupt a snapshot chunk by overwriting the blob contents.
    let corrupt_hash = write.snapshot_manifest.chunks[0].content_hash.clone();
    let blob_path = store.blobs_dir().join(format!("{corrupt_hash}.blob"));
    std::fs::write(&blob_path, b"corrupt").expect("overwrite chunk");

    let head_clone = write.head_announce.clone();
    let block_clone = write.block.clone();
    let snap_ref = write.snapshot_manifest_ref.content_hash.clone();
    let journal_ref = write.journal_segments_ref.content_hash.clone();
    let store_clone = store.clone();

    network
        .register_handler(
            dist::RR_GET_WORLD_HEAD,
            Box::new(move |_payload| {
                let response = dist::GetWorldHeadResponse {
                    head: head_clone.clone(),
                };
                Ok(serde_cbor::to_vec(&response).unwrap())
            }),
        )
        .expect("register head");

    network
        .register_handler(
            dist::RR_GET_BLOCK,
            Box::new(move |_payload| {
                let response = dist::GetBlockResponse {
                    block: block_clone.clone(),
                    journal_ref: journal_ref.clone(),
                    snapshot_ref: snap_ref.clone(),
                };
                Ok(serde_cbor::to_vec(&response).unwrap())
            }),
        )
        .expect("register block");

    network
        .register_handler(
            dist::RR_FETCH_BLOB,
            Box::new(move |payload| {
                let request: dist::FetchBlobRequest = serde_cbor::from_slice(payload).unwrap();
                let bytes = store_clone.get(&request.content_hash).unwrap();
                let response = dist::FetchBlobResponse {
                    blob: bytes,
                    content_hash: request.content_hash,
                };
                Ok(serde_cbor::to_vec(&response).unwrap())
            }),
        )
        .expect("register fetch");

    let client = DistributedClient::new(Arc::clone(&network));
    let err = replay_validate_head("w1", &client, &store).expect_err("expect failure");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("mismatch")
            || msg.contains("Serde")
            || msg.contains("Deserialize")
            || msg.contains("BlobHashMismatch"),
        "unexpected error: {msg}"
    );

    let _ = fs::remove_dir_all(&dir);
}
