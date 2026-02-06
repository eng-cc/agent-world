//! Distributed DHT adapter abstractions (provider/head indexing).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use super::distributed::WorldHeadAnnounce;
use super::error::WorldError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRecord {
    pub provider_id: String,
    pub last_seen_ms: i64,
}

pub trait DistributedDht {
    fn publish_provider(
        &self,
        world_id: &str,
        content_hash: &str,
        provider_id: &str,
    ) -> Result<(), WorldError>;

    fn get_providers(
        &self,
        world_id: &str,
        content_hash: &str,
    ) -> Result<Vec<ProviderRecord>, WorldError>;

    fn put_world_head(&self, world_id: &str, head: &WorldHeadAnnounce) -> Result<(), WorldError>;

    fn get_world_head(&self, world_id: &str) -> Result<Option<WorldHeadAnnounce>, WorldError>;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryDht {
    providers: Arc<Mutex<BTreeMap<(String, String), BTreeMap<String, ProviderRecord>>>>,
    heads: Arc<Mutex<BTreeMap<String, WorldHeadAnnounce>>>,
}

impl InMemoryDht {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DistributedDht for InMemoryDht {
    fn publish_provider(
        &self,
        world_id: &str,
        content_hash: &str,
        provider_id: &str,
    ) -> Result<(), WorldError> {
        let mut providers = self.providers.lock().expect("lock providers");
        let key = (world_id.to_string(), content_hash.to_string());
        let record = ProviderRecord {
            provider_id: provider_id.to_string(),
            last_seen_ms: now_ms(),
        };
        providers
            .entry(key)
            .or_default()
            .insert(provider_id.to_string(), record);
        Ok(())
    }

    fn get_providers(
        &self,
        world_id: &str,
        content_hash: &str,
    ) -> Result<Vec<ProviderRecord>, WorldError> {
        let providers = self.providers.lock().expect("lock providers");
        let key = (world_id.to_string(), content_hash.to_string());
        Ok(providers
            .get(&key)
            .map(|records| records.values().cloned().collect())
            .unwrap_or_default())
    }

    fn put_world_head(&self, world_id: &str, head: &WorldHeadAnnounce) -> Result<(), WorldError> {
        let mut heads = self.heads.lock().expect("lock heads");
        heads.insert(world_id.to_string(), head.clone());
        Ok(())
    }

    fn get_world_head(&self, world_id: &str) -> Result<Option<WorldHeadAnnounce>, WorldError> {
        let heads = self.heads.lock().expect("lock heads");
        Ok(heads.get(world_id).cloned())
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_dht_stores_providers() {
        let dht = InMemoryDht::new();
        dht.publish_provider("w1", "hash", "peer-1")
            .expect("publish provider");
        dht.publish_provider("w1", "hash", "peer-2")
            .expect("publish provider");

        let providers = dht.get_providers("w1", "hash").expect("get providers");
        assert_eq!(providers.len(), 2);
    }

    #[test]
    fn in_memory_dht_tracks_world_head() {
        let dht = InMemoryDht::new();
        let head = WorldHeadAnnounce {
            world_id: "w1".to_string(),
            height: 1,
            block_hash: "b1".to_string(),
            state_root: "s1".to_string(),
            timestamp_ms: 1,
            signature: "sig".to_string(),
        };
        dht.put_world_head("w1", &head).expect("put head");

        let loaded = dht.get_world_head("w1").expect("get head");
        assert_eq!(loaded, Some(head));
    }
}
