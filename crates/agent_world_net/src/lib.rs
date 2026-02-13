//! Network-focused facade for distributed runtime capabilities.

#[cfg(feature = "runtime_bridge")]
mod bootstrap;
mod client;
mod dht;
mod dht_cache;
#[cfg(feature = "runtime_bridge")]
mod execution_storage;
mod gateway;
#[cfg(feature = "runtime_bridge")]
mod head_follow;
#[cfg(feature = "runtime_bridge")]
mod head_validation;
mod index;
mod index_store;
mod network;
#[cfg(feature = "runtime_bridge")]
mod observer;
#[cfg(feature = "runtime_bridge")]
mod observer_replay;
mod provider_cache;
mod util;

#[cfg(feature = "libp2p")]
mod libp2p_net;

pub mod distributed_net {
    pub use super::network::*;
}

pub mod distributed {
    pub use agent_world_proto::distributed::*;
}

pub mod distributed_dht {
    pub use super::dht::*;
}

pub mod distributed_client {
    pub use super::client::*;
}

#[cfg(feature = "runtime_bridge")]
pub mod distributed_bootstrap {
    pub use super::bootstrap::*;
}

#[cfg(feature = "runtime_bridge")]
pub mod distributed_head_follow {
    pub use super::head_follow::*;
}

#[cfg(feature = "runtime_bridge")]
pub mod distributed_observer_replay {
    pub use super::observer_replay::*;
}

pub mod distributed_index_store {
    pub use super::index_store::*;
}

pub mod distributed_provider_cache {
    pub use super::provider_cache::*;
}

pub mod distributed_storage {
    use agent_world_proto::distributed::{
        BlobRef, BlockAnnounce, SnapshotManifest, WorldBlock, WorldHeadAnnounce,
    };

    #[derive(Debug, Clone)]
    pub struct ExecutionWriteConfig {
        pub codec: String,
    }

    impl Default for ExecutionWriteConfig {
        fn default() -> Self {
            Self {
                codec: "dag-cbor".to_string(),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct ExecutionWriteResult {
        pub block: WorldBlock,
        pub block_hash: String,
        pub block_ref: BlobRef,
        pub block_announce: BlockAnnounce,
        pub head_announce: WorldHeadAnnounce,
        pub snapshot_manifest: SnapshotManifest,
        pub snapshot_manifest_ref: BlobRef,
        pub journal_segments: Vec<BlobRef>,
        pub journal_segments_ref: BlobRef,
    }

    #[cfg(feature = "runtime_bridge")]
    pub use super::execution_storage::store_execution_result;
}

#[cfg(feature = "runtime_bridge")]
pub mod distributed_validation {
    pub use super::head_validation::{assemble_journal, assemble_snapshot, validate_head_update};
    pub use super::HeadValidationResult;
}

pub mod error {
    pub use agent_world_proto::world_error::WorldError;
}

pub mod modules {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ModuleArtifact {
        pub wasm_hash: String,
        pub bytes: Vec<u8>,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    pub struct ModuleManifest {
        pub module_id: String,
        pub name: String,
        pub version: String,
        pub wasm_hash: String,
    }
}

pub use agent_world_proto::distributed_dht as proto_dht;
pub use agent_world_proto::distributed_net as proto_net;
pub use client::DistributedClient;
pub use dht::{DistributedDht, InMemoryDht};
pub use dht_cache::{CachedDht, DhtCacheConfig};
pub use distributed_storage::{ExecutionWriteConfig, ExecutionWriteResult};
pub use error::WorldError;
pub use gateway::{ActionGateway, NetworkGateway, SubmitActionReceipt};
pub use index::{
    publish_execution_providers, publish_execution_providers_cached, publish_world_head,
    query_providers, IndexPublishResult,
};
pub use index_store::{DistributedIndexStore, HeadIndexRecord, InMemoryIndexStore};
pub use modules::{ModuleArtifact, ModuleManifest};
pub use network::{DistributedNetwork, InMemoryNetwork};
pub use proto_dht::{MembershipDirectorySnapshot, ProviderRecord};
pub use proto_net::{NetworkMessage, NetworkRequest, NetworkResponse, NetworkSubscription};
pub use provider_cache::{ProviderCache, ProviderCacheConfig};

#[cfg(feature = "runtime_bridge")]
pub use bootstrap::{
    bootstrap_world_from_dht, bootstrap_world_from_head, bootstrap_world_from_head_with_dht,
};
#[cfg(feature = "runtime_bridge")]
pub use distributed_storage::store_execution_result;
#[cfg(feature = "runtime_bridge")]
pub use head_follow::{HeadFollower, HeadUpdateDecision};
#[cfg(feature = "runtime_bridge")]
pub use head_validation::{assemble_journal, assemble_snapshot, validate_head_update};
#[cfg(feature = "runtime_bridge")]
pub use observer::{
    HeadFollowReport, HeadSyncReport, HeadSyncResult, ObserverClient, ObserverSubscription,
};
#[cfg(feature = "runtime_bridge")]
pub use observer_replay::{
    replay_validate_head, replay_validate_head_with_dht, replay_validate_with_head,
    replay_validate_with_head_and_dht,
};

#[cfg(feature = "runtime_bridge")]
pub use head_validation::HeadValidationResult;

#[cfg(feature = "libp2p")]
pub use libp2p_net::{Libp2pNetwork, Libp2pNetworkConfig};

#[cfg(test)]
mod tests;
