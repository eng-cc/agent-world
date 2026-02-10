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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipDirectorySnapshot {
    pub world_id: String,
    pub requester_id: String,
    pub requested_at_ms: i64,
    pub reason: Option<String>,
    pub validators: Vec<String>,
    pub quorum_threshold: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature_key_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
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

    fn put_membership_directory(
        &self,
        world_id: &str,
        snapshot: &MembershipDirectorySnapshot,
    ) -> Result<(), WorldError>;

    fn get_membership_directory(
        &self,
        world_id: &str,
    ) -> Result<Option<MembershipDirectorySnapshot>, WorldError>;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryDht {
    providers: Arc<Mutex<BTreeMap<(String, String), BTreeMap<String, ProviderRecord>>>>,
    heads: Arc<Mutex<BTreeMap<String, WorldHeadAnnounce>>>,
    memberships: Arc<Mutex<BTreeMap<String, MembershipDirectorySnapshot>>>,
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

    fn put_membership_directory(
        &self,
        world_id: &str,
        snapshot: &MembershipDirectorySnapshot,
    ) -> Result<(), WorldError> {
        let mut memberships = self.memberships.lock().expect("lock memberships");
        memberships.insert(world_id.to_string(), snapshot.clone());
        Ok(())
    }

    fn get_membership_directory(
        &self,
        world_id: &str,
    ) -> Result<Option<MembershipDirectorySnapshot>, WorldError> {
        let memberships = self.memberships.lock().expect("lock memberships");
        Ok(memberships.get(world_id).cloned())
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

    #[test]
    fn in_memory_dht_tracks_membership_directory_snapshot() {
        let dht = InMemoryDht::new();
        let snapshot = MembershipDirectorySnapshot {
            world_id: "w1".to_string(),
            requester_id: "seq-1".to_string(),
            requested_at_ms: 1,
            reason: Some("bootstrap".to_string()),
            validators: vec![
                "seq-1".to_string(),
                "seq-2".to_string(),
                "seq-3".to_string(),
            ],
            quorum_threshold: 2,
            signature_key_id: Some("k1".to_string()),
            signature: Some("deadbeef".to_string()),
        };
        dht.put_membership_directory("w1", &snapshot)
            .expect("put membership");

        let loaded = dht.get_membership_directory("w1").expect("get membership");
        assert_eq!(loaded, Some(snapshot));
    }
}
