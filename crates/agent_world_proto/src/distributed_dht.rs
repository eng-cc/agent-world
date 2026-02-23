//! Distributed DHT adapter abstractions (provider/head indexing).

use serde::{Deserialize, Serialize};

use crate::distributed::WorldHeadAnnounce;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRecord {
    pub provider_id: String,
    pub last_seen_ms: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_total_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage_available_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uptime_ratio_per_mille: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub challenge_pass_ratio_per_mille: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load_ratio_per_mille: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p50_read_latency_ms: Option<u32>,
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

#[cfg(test)]
mod tests {
    use super::ProviderRecord;

    #[test]
    fn provider_record_deserializes_legacy_json_without_capability_fields() {
        let legacy_json = r#"{"provider_id":"peer-1","last_seen_ms":42}"#;
        let record: ProviderRecord = serde_json::from_str(legacy_json).expect("deserialize");
        assert_eq!(record.provider_id, "peer-1");
        assert_eq!(record.last_seen_ms, 42);
        assert_eq!(record.storage_total_bytes, None);
        assert_eq!(record.storage_available_bytes, None);
        assert_eq!(record.uptime_ratio_per_mille, None);
        assert_eq!(record.challenge_pass_ratio_per_mille, None);
        assert_eq!(record.load_ratio_per_mille, None);
        assert_eq!(record.p50_read_latency_ms, None);
    }

    #[test]
    fn provider_record_omits_empty_capability_fields_in_json() {
        let record = ProviderRecord {
            provider_id: "peer-1".to_string(),
            last_seen_ms: 42,
            storage_total_bytes: None,
            storage_available_bytes: None,
            uptime_ratio_per_mille: None,
            challenge_pass_ratio_per_mille: None,
            load_ratio_per_mille: None,
            p50_read_latency_ms: None,
        };
        let encoded = serde_json::to_value(&record).expect("serialize");
        let object = encoded.as_object().expect("json object");
        assert_eq!(
            object.get("provider_id").and_then(|v| v.as_str()),
            Some("peer-1")
        );
        assert_eq!(
            object.get("last_seen_ms").and_then(|v| v.as_i64()),
            Some(42)
        );
        assert!(object.get("storage_total_bytes").is_none());
        assert!(object.get("storage_available_bytes").is_none());
        assert!(object.get("uptime_ratio_per_mille").is_none());
        assert!(object.get("challenge_pass_ratio_per_mille").is_none());
        assert!(object.get("load_ratio_per_mille").is_none());
        assert!(object.get("p50_read_latency_ms").is_none());
    }
}
