//! Index publishing helpers for DHT.

use std::collections::HashSet;

use super::distributed::WorldHeadAnnounce;
use super::distributed_dht::{DistributedDht, ProviderRecord};
use super::distributed_provider_cache::ProviderCache;
use super::distributed_storage::ExecutionWriteResult;
use super::error::WorldError;

#[derive(Debug, Clone)]
pub struct IndexPublishResult {
    pub world_id: String,
    pub published: usize,
}

pub fn publish_world_head(
    dht: &impl DistributedDht,
    head: &WorldHeadAnnounce,
) -> Result<(), WorldError> {
    dht.put_world_head(&head.world_id, head)
}

pub fn publish_execution_providers(
    dht: &impl DistributedDht,
    world_id: &str,
    provider_id: &str,
    result: &ExecutionWriteResult,
) -> Result<IndexPublishResult, WorldError> {
    let hashes = collect_execution_hashes(result);

    let mut published = 0usize;
    for hash in hashes {
        dht.publish_provider(world_id, &hash, provider_id)?;
        published = published.saturating_add(1);
    }

    Ok(IndexPublishResult {
        world_id: world_id.to_string(),
        published,
    })
}

pub fn publish_execution_providers_cached(
    cache: &ProviderCache,
    world_id: &str,
    result: &ExecutionWriteResult,
) -> Result<IndexPublishResult, WorldError> {
    let hashes = collect_execution_hashes(result);

    let mut published = 0usize;
    for hash in hashes {
        cache.register_local_content(world_id, &hash)?;
        published = published.saturating_add(1);
    }

    Ok(IndexPublishResult {
        world_id: world_id.to_string(),
        published,
    })
}

pub fn query_providers(
    dht: &impl DistributedDht,
    world_id: &str,
    content_hash: &str,
) -> Result<Vec<ProviderRecord>, WorldError> {
    dht.get_providers(world_id, content_hash)
}

fn collect_execution_hashes(result: &ExecutionWriteResult) -> HashSet<String> {
    let mut hashes: HashSet<String> = HashSet::new();
    hashes.insert(result.block_ref.content_hash.clone());
    hashes.insert(result.snapshot_manifest_ref.content_hash.clone());
    hashes.insert(result.journal_segments_ref.content_hash.clone());
    for chunk in &result.snapshot_manifest.chunks {
        hashes.insert(chunk.content_hash.clone());
    }
    for segment in &result.journal_segments {
        hashes.insert(segment.content_hash.clone());
    }
    hashes
}

#[cfg(test)]
mod tests {
    use super::super::distributed_storage::{store_execution_result, ExecutionWriteConfig};
    use super::super::{
        Action, InMemoryDht, InMemoryIndexStore, LocalCasStore, ProviderCache, ProviderCacheConfig,
        World,
    };
    use super::*;
    use std::collections::HashSet;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("agent-world-{prefix}-{unique}"))
    }

    #[test]
    fn publish_world_head_round_trip() {
        let dht = InMemoryDht::new();
        let head = WorldHeadAnnounce {
            world_id: "w1".to_string(),
            height: 3,
            block_hash: "b1".to_string(),
            state_root: "s1".to_string(),
            timestamp_ms: 1,
            signature: "sig".to_string(),
        };
        publish_world_head(&dht, &head).expect("publish head");
        let loaded = dht.get_world_head("w1").expect("get head");
        assert_eq!(loaded, Some(head));
    }

    #[test]
    fn publish_execution_providers_indexes_all_hashes() {
        let dir = temp_dir("index");
        let store = LocalCasStore::new(&dir);
        let mut world = World::new();
        world.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            pos: crate::geometry::GeoPos::new(0.0, 0.0, 0.0),
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

        let dht = InMemoryDht::new();
        let result =
            publish_execution_providers(&dht, "w1", "store-1", &write).expect("publish providers");
        assert!(result.published > 0);

        let mut expected = HashSet::new();
        expected.insert(write.block_ref.content_hash.clone());
        expected.insert(write.snapshot_manifest_ref.content_hash.clone());
        expected.insert(write.journal_segments_ref.content_hash.clone());
        for chunk in &write.snapshot_manifest.chunks {
            expected.insert(chunk.content_hash.clone());
        }
        for segment in &write.journal_segments {
            expected.insert(segment.content_hash.clone());
        }

        for hash in expected {
            let providers = query_providers(&dht, "w1", &hash).expect("get providers");
            assert!(!providers.is_empty());
            assert_eq!(providers[0].provider_id, "store-1");
        }

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn publish_execution_providers_cached_indexes_all_hashes() {
        let dir = temp_dir("index-cache");
        let store = LocalCasStore::new(&dir);
        let mut world = World::new();
        world.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            pos: crate::geometry::GeoPos::new(0.0, 0.0, 0.0),
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

        let dht = InMemoryDht::new();
        let index_store = InMemoryIndexStore::new();
        let cache = ProviderCache::new(
            std::sync::Arc::new(dht.clone()),
            std::sync::Arc::new(index_store),
            "store-1",
            ProviderCacheConfig::default(),
        );

        let result =
            publish_execution_providers_cached(&cache, "w1", &write).expect("publish providers");
        assert!(result.published > 0);

        let mut expected = HashSet::new();
        expected.insert(write.block_ref.content_hash.clone());
        expected.insert(write.snapshot_manifest_ref.content_hash.clone());
        expected.insert(write.journal_segments_ref.content_hash.clone());
        for chunk in &write.snapshot_manifest.chunks {
            expected.insert(chunk.content_hash.clone());
        }
        for segment in &write.journal_segments {
            expected.insert(segment.content_hash.clone());
        }

        for hash in expected {
            let providers = query_providers(&dht, "w1", &hash).expect("get providers");
            assert!(!providers.is_empty());
            assert_eq!(providers[0].provider_id, "store-1");
        }

        let _ = fs::remove_dir_all(&dir);
    }
}
