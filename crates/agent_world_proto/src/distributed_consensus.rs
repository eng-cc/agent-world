//! Distributed consensus protocol data structures.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::distributed::WorldHeadAnnounce;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsensusStatus {
    Pending,
    Committed,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusVote {
    pub validator_id: String,
    pub approve: bool,
    pub reason: Option<String>,
    pub voted_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadConsensusRecord {
    pub head: WorldHeadAnnounce,
    pub proposer_id: String,
    pub proposed_at_ms: i64,
    pub quorum_threshold: usize,
    #[serde(default)]
    pub validator_count: usize,
    pub status: ConsensusStatus,
    pub votes: BTreeMap<String, ConsensusVote>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConsensusMembershipChange {
    AddValidator {
        validator_id: String,
    },
    RemoveValidator {
        validator_id: String,
    },
    ReplaceValidators {
        validators: Vec<String>,
        quorum_threshold: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusMembershipChangeRequest {
    pub requester_id: String,
    pub requested_at_ms: i64,
    pub reason: Option<String>,
    pub change: ConsensusMembershipChange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsensusMembershipChangeResult {
    pub applied: bool,
    pub validators: Vec<String>,
    pub quorum_threshold: usize,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn cbor_round_trip_membership_change_request() {
        let request = ConsensusMembershipChangeRequest {
            requester_id: "validator-1".to_string(),
            requested_at_ms: 42,
            reason: Some("rebalance".to_string()),
            change: ConsensusMembershipChange::AddValidator {
                validator_id: "validator-4".to_string(),
            },
        };

        let bytes = serde_cbor::to_vec(&request).expect("serialize request");
        let decoded: ConsensusMembershipChangeRequest =
            serde_cbor::from_slice(&bytes).expect("deserialize request");
        assert_eq!(decoded, request);
    }

    #[test]
    fn cbor_round_trip_head_consensus_record() {
        let record = HeadConsensusRecord {
            head: WorldHeadAnnounce {
                world_id: "w1".to_string(),
                height: 3,
                block_hash: "b3".to_string(),
                state_root: "s3".to_string(),
                timestamp_ms: 100,
                signature: "sig".to_string(),
            },
            proposer_id: "validator-1".to_string(),
            proposed_at_ms: 100,
            quorum_threshold: 2,
            validator_count: 3,
            status: ConsensusStatus::Pending,
            votes: BTreeMap::new(),
        };

        let bytes = serde_cbor::to_vec(&record).expect("serialize record");
        let decoded: HeadConsensusRecord =
            serde_cbor::from_slice(&bytes).expect("deserialize record");
        assert_eq!(decoded, record);
    }
}
