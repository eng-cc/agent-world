use std::fs;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use agent_world::runtime::distributed as dist;
use agent_world::runtime::{
    publish_execution_providers, publish_world_head, replay_validate_head, store_execution_result,
    ActionBatchRules, ActionGateway, ActionMempool, ActionMempoolConfig, DistributedClient,
    DistributedNetwork, ExecutionWriteConfig, InMemoryDht, InMemoryNetwork, NetworkGateway,
    ObserverClient,
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
fn multi_node_integration_flow() {
    let dir = temp_dir("multi-node");
    let store = LocalCasStore::new(&dir);
    let dht = InMemoryDht::new();
    let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());

    // Gateway submits action into gossipsub.
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

    // Sequencer collects action into mempool and creates a batch.
    let subscription = network
        .subscribe(&dist::topic_action("w1"))
        .expect("subscribe");
    let messages = subscription.drain();
    assert_eq!(messages.len(), 1);
    let decoded: dist::ActionEnvelope =
        serde_cbor::from_slice(&messages[0]).expect("decode envelope");

    let mut mempool = ActionMempool::new(ActionMempoolConfig::default());
    assert!(mempool.add_action(decoded));
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

    // Execution node applies actions and writes results to storage.
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

    // Storage publishes provider records and head to DHT and gossipsub.
    publish_world_head(&dht, &write.head_announce).expect("publish head");
    publish_execution_providers(&dht, "w1", "store-1", &write).expect("publish providers");
    let head_payload = serde_cbor::to_vec(&write.head_announce).expect("head cbor");
    network
        .publish(&dist::topic_head("w1"), &head_payload)
        .expect("publish head");

    // Observer watches head updates.
    let observer = ObserverClient::new(Arc::clone(&network));
    let obs_sub = observer.subscribe("w1").expect("observer subscribe");
    let heads = observer.drain_heads(&obs_sub).expect("drain heads");
    assert_eq!(heads.len(), 1);
    assert_eq!(heads[0], write.head_announce);

    // Provide RR handlers for replay validation.
    let head_clone = write.head_announce.clone();
    let block_clone = write.block.clone();
    let snap_ref = write.snapshot_manifest_ref.content_hash.clone();
    let journal_ref = write.journal_segments_ref.content_hash.clone();
    let store_clone = store.clone();

    network
        .register_handler(
            dist::RR_GET_WORLD_HEAD,
            Box::new(move |payload| {
                let request: dist::GetWorldHeadRequest = serde_cbor::from_slice(payload).unwrap();
                assert_eq!(request.world_id, "w1");
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
            Box::new(move |payload| {
                let request: dist::GetBlockRequest = serde_cbor::from_slice(payload).unwrap();
                assert_eq!(request.world_id, "w1");
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
        .expect("register fetch blob");

    let client = DistributedClient::new(Arc::clone(&network));
    let validation = replay_validate_head("w1", &client, &store).expect("replay validate");
    assert_eq!(validation.block_hash, write.head_announce.block_hash);

    let _ = fs::remove_dir_all(&dir);
}
