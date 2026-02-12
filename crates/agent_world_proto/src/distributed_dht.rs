//! Distributed DHT adapter abstractions (provider/head indexing).

use serde::{Deserialize, Serialize};

use crate::distributed::WorldHeadAnnounce;

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

pub trait DistributedDht<E> {
    fn publish_provider(
        &self,
        world_id: &str,
        content_hash: &str,
        provider_id: &str,
    ) -> Result<(), E>;

    fn get_providers(&self, world_id: &str, content_hash: &str) -> Result<Vec<ProviderRecord>, E>;

    fn put_world_head(&self, world_id: &str, head: &WorldHeadAnnounce) -> Result<(), E>;

    fn get_world_head(&self, world_id: &str) -> Result<Option<WorldHeadAnnounce>, E>;

    fn put_membership_directory(
        &self,
        world_id: &str,
        snapshot: &MembershipDirectorySnapshot,
    ) -> Result<(), E>;

    fn get_membership_directory(
        &self,
        world_id: &str,
    ) -> Result<Option<MembershipDirectorySnapshot>, E>;
}
