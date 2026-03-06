//! Tick-level execution consensus records.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::types::{ActionId, WorldEventId, WorldTime};
use super::util::sha256_hex;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TickBlockHeader {
    pub epoch: u64,
    pub tick: WorldTime,
    pub parent_hash: String,
    pub events_hash: String,
    pub state_root: String,
    pub executor_version: String,
    pub randomness_seed: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TickExecutionDigest {
    pub action_batch_hash: String,
    pub domain_events_hash: String,
    pub state_projection_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TickBlock {
    pub header: TickBlockHeader,
    #[serde(default)]
    pub ordered_action_ids: Vec<ActionId>,
    #[serde(default)]
    pub ordered_event_ids: Vec<WorldEventId>,
    pub event_count: u32,
    pub execution_digest: TickExecutionDigest,
}

impl TickBlock {
    pub fn block_hash(&self) -> String {
        let payload = format!(
            "tickblock:v1|{}|{}|{}|{}|{}",
            self.header.parent_hash,
            self.header.tick,
            self.header.events_hash,
            self.header.state_root,
            self.header.executor_version
        );
        sha256_hex(payload.as_bytes())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TickCertificate {
    pub block_hash: String,
    pub consensus_height: u64,
    pub threshold: u16,
    #[serde(default)]
    pub signatures: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TickConsensusRecord {
    pub block: TickBlock,
    pub certificate: TickCertificate,
}
