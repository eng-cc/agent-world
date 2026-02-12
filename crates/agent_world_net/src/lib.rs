//! Network-focused facade for distributed runtime capabilities.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use agent_world::runtime::WorldError;
use agent_world_proto::distributed::WorldHeadAnnounce;
use agent_world_proto::distributed_dht as proto_dht;
use agent_world_proto::distributed_net as proto_net;

pub use agent_world::runtime::{
    publish_execution_providers, publish_execution_providers_cached, publish_world_head,
    query_providers, ActionGateway, CachedDht, DhtCacheConfig, DistributedClient,
    DistributedIndexStore, HeadFollowReport, HeadFollower, HeadIndexRecord, HeadSyncReport,
    HeadSyncResult, HeadUpdateDecision, InMemoryIndexStore, IndexPublishResult, NetworkGateway,
    ObserverClient, ObserverSubscription, ProviderCache, ProviderCacheConfig, SubmitActionReceipt,
};
pub use proto_dht::{MembershipDirectorySnapshot, ProviderRecord};
pub use proto_net::{NetworkMessage, NetworkRequest, NetworkResponse, NetworkSubscription};

pub trait DistributedNetwork: proto_net::DistributedNetwork<WorldError> {}

impl<T> DistributedNetwork for T where T: proto_net::DistributedNetwork<WorldError> {}

#[derive(Clone, Default)]
pub struct InMemoryNetwork {
    inbox: Arc<Mutex<HashMap<String, Vec<Vec<u8>>>>>,
    published: Arc<Mutex<Vec<NetworkMessage>>>,
    handlers: Arc<Mutex<HashMap<String, Handler>>>,
}

type Handler = Arc<dyn Fn(&[u8]) -> Result<Vec<u8>, WorldError> + Send + Sync>;

impl InMemoryNetwork {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn published(&self) -> Vec<NetworkMessage> {
        self.published.lock().expect("lock published").clone()
    }
}

impl proto_net::DistributedNetwork<WorldError> for InMemoryNetwork {
    fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), WorldError> {
        let message = NetworkMessage {
            topic: topic.to_string(),
            payload: payload.to_vec(),
        };
        {
            let mut published = self.published.lock().expect("lock published");
            published.push(message.clone());
        }
        let mut inbox = self.inbox.lock().expect("lock inbox");
        inbox
            .entry(topic.to_string())
            .or_default()
            .push(message.payload);
        Ok(())
    }

    fn subscribe(&self, topic: &str) -> Result<NetworkSubscription, WorldError> {
        let mut inbox = self.inbox.lock().expect("lock inbox");
        inbox.entry(topic.to_string()).or_default();
        Ok(NetworkSubscription::new(
            topic.to_string(),
            Arc::clone(&self.inbox),
        ))
    }

    fn request(&self, protocol: &str, payload: &[u8]) -> Result<Vec<u8>, WorldError> {
        let handler = {
            let handlers = self.handlers.lock().expect("lock handlers");
            handlers.get(protocol).cloned()
        };
        let handler = handler.ok_or_else(|| WorldError::NetworkProtocolUnavailable {
            protocol: protocol.to_string(),
        })?;
        handler(payload)
    }

    fn register_handler(
        &self,
        protocol: &str,
        handler: Box<dyn Fn(&[u8]) -> Result<Vec<u8>, WorldError> + Send + Sync>,
    ) -> Result<(), WorldError> {
        let mut handlers = self.handlers.lock().expect("lock handlers");
        handlers.insert(protocol.to_string(), Arc::from(handler));
        Ok(())
    }
}

pub trait DistributedDht: proto_dht::DistributedDht<WorldError> {}

impl<T> DistributedDht for T where T: proto_dht::DistributedDht<WorldError> {}

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

impl proto_dht::DistributedDht<WorldError> for InMemoryDht {
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

#[cfg(feature = "libp2p")]
pub use agent_world::runtime::{Libp2pNetwork, Libp2pNetworkConfig};

#[cfg(test)]
mod tests {
    use agent_world_proto::distributed_dht::DistributedDht as _;
    use agent_world_proto::distributed_net::DistributedNetwork as _;

    use super::*;

    #[test]
    fn net_exports_are_available() {
        let _ = std::any::type_name::<NetworkMessage>();
        let _ = std::any::type_name::<DistributedClient>();
        let _ = std::any::type_name::<HeadFollower>();
    }

    #[test]
    fn in_memory_publish_delivers_to_subscribers() {
        let network = InMemoryNetwork::new();
        let subscription = network.subscribe("aw.w1.action").expect("subscribe");

        network
            .publish("aw.w1.action", b"payload")
            .expect("publish");

        let messages = subscription.drain();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], b"payload".to_vec());
    }

    #[test]
    fn in_memory_request_invokes_handler() {
        let network = InMemoryNetwork::new();
        network
            .register_handler(
                "/aw/rr/1.0.0/get_world_head",
                Box::new(|payload| {
                    let mut out = payload.to_vec();
                    out.extend_from_slice(b"-ok");
                    Ok(out)
                }),
            )
            .expect("register handler");

        let response = network
            .request("/aw/rr/1.0.0/get_world_head", b"ping")
            .expect("request");
        assert_eq!(response, b"ping-ok".to_vec());
    }

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
