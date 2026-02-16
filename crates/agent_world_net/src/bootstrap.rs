use super::distributed::WorldHeadAnnounce;
use super::distributed_client::DistributedClient;
use super::distributed_dht::DistributedDht;
use super::distributed_observer_replay::{
    replay_validate_with_head, replay_validate_with_head_and_dht,
};
use super::error::WorldError;
use agent_world::runtime::{World, WorldError as RuntimeWorldError};
use agent_world_distfs::BlobStore;

pub fn bootstrap_world_from_head(
    head: &WorldHeadAnnounce,
    client: &DistributedClient,
    store: &impl BlobStore,
) -> Result<World, WorldError> {
    let result = replay_validate_with_head(head, client, store)?;
    World::from_snapshot(result.snapshot, result.journal).map_err(runtime_world_error_to_proto)
}

pub fn bootstrap_world_from_head_with_dht(
    head: &WorldHeadAnnounce,
    dht: &impl DistributedDht,
    client: &DistributedClient,
    store: &impl BlobStore,
) -> Result<World, WorldError> {
    let result = replay_validate_with_head_and_dht(head, dht, client, store)?;
    World::from_snapshot(result.snapshot, result.journal).map_err(runtime_world_error_to_proto)
}

pub fn bootstrap_world_from_dht(
    world_id: &str,
    dht: &impl DistributedDht,
    client: &DistributedClient,
    store: &impl BlobStore,
) -> Result<World, WorldError> {
    let head =
        dht.get_world_head(world_id)?
            .ok_or_else(|| WorldError::DistributedValidationFailed {
                reason: format!("world head not found for {world_id}"),
            })?;
    bootstrap_world_from_head_with_dht(&head, dht, client, store)
}

fn runtime_world_error_to_proto(error: RuntimeWorldError) -> WorldError {
    WorldError::DistributedValidationFailed {
        reason: format!("runtime world validation failed: {error:?}"),
    }
}

#[cfg(all(test, feature = "self_tests"))]
mod tests {
    use std::fs;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    use agent_world::runtime::{Action, World};
    use agent_world::GeoPos;
    use agent_world_distfs::{BlobStore as _, LocalCasStore};
    use agent_world_proto::distributed::{
        FetchBlobRequest, FetchBlobResponse, GetBlockRequest, GetBlockResponse, RR_FETCH_BLOB,
        RR_GET_BLOCK,
    };
    use agent_world_proto::distributed_dht::DistributedDht as _;

    use super::super::distributed_dht::InMemoryDht;
    use super::super::distributed_net::{DistributedNetwork, InMemoryNetwork};
    use super::super::distributed_storage::{store_execution_result, ExecutionWriteConfig};
    use super::super::util::to_canonical_cbor;
    use super::*;

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("duration since epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("agent-world-net-{prefix}-{unique}"))
    }

    fn register_block_fetch_handlers(
        network: &Arc<dyn DistributedNetwork + Send + Sync>,
        world_id: &'static str,
        store: &LocalCasStore,
        block: agent_world_proto::distributed::WorldBlock,
        snapshot_ref: String,
        journal_ref: String,
    ) {
        network
            .register_handler(
                RR_GET_BLOCK,
                Box::new(move |payload| {
                    let request: GetBlockRequest =
                        serde_cbor::from_slice(payload).expect("decode request");
                    assert_eq!(request.world_id, world_id);
                    let response = GetBlockResponse {
                        block: block.clone(),
                        journal_ref: journal_ref.clone(),
                        snapshot_ref: snapshot_ref.clone(),
                    };
                    Ok(to_canonical_cbor(&response).expect("encode response"))
                }),
            )
            .expect("register block");

        let store_clone = store.clone();
        network
            .register_handler(
                RR_FETCH_BLOB,
                Box::new(move |payload| {
                    let request: FetchBlobRequest =
                        serde_cbor::from_slice(payload).expect("decode request");
                    let bytes = store_clone.get(&request.content_hash).expect("load blob");
                    let response = FetchBlobResponse {
                        blob: bytes,
                        content_hash: request.content_hash,
                    };
                    Ok(to_canonical_cbor(&response).expect("encode response"))
                }),
            )
            .expect("register fetch");
    }

    #[test]
    fn bootstrap_world_from_head_round_trip() {
        let dir = temp_dir("bootstrap-head");
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

        let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
        register_block_fetch_handlers(
            &network,
            "w1",
            &store,
            write.block.clone(),
            write.snapshot_manifest_ref.content_hash.clone(),
            write.journal_segments_ref.content_hash.clone(),
        );

        let client = DistributedClient::new(Arc::clone(&network));
        let bootstrapped =
            bootstrap_world_from_head(&write.head_announce, &client, &store).expect("bootstrap");
        assert_eq!(bootstrapped.journal().len(), journal.len());
        assert_eq!(bootstrapped.manifest().version, snapshot.manifest.version);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn bootstrap_world_from_dht_round_trip() {
        let dir = temp_dir("bootstrap-dht");
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

        let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
        register_block_fetch_handlers(
            &network,
            "w1",
            &store,
            write.block.clone(),
            write.snapshot_manifest_ref.content_hash.clone(),
            write.journal_segments_ref.content_hash.clone(),
        );

        let client = DistributedClient::new(Arc::clone(&network));
        let dht = InMemoryDht::new();
        dht.put_world_head("w1", &write.head_announce)
            .expect("put head");

        let bootstrapped =
            bootstrap_world_from_dht("w1", &dht, &client, &store).expect("bootstrap");
        assert_eq!(bootstrapped.journal().len(), journal.len());
        assert_eq!(bootstrapped.manifest().version, snapshot.manifest.version);

        let _ = fs::remove_dir_all(&dir);
    }
}
