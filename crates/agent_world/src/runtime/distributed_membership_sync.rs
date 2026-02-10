//! Distributed membership directory broadcast and sync helpers.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::distributed::topic_membership;
use super::distributed_consensus::{
    ConsensusMembershipChange, ConsensusMembershipChangeRequest, ConsensusMembershipChangeResult,
    QuorumConsensus,
};
use super::distributed_net::{DistributedNetwork, NetworkSubscription};
use super::error::WorldError;
use super::util::to_canonical_cbor;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipDirectoryAnnounce {
    pub world_id: String,
    pub requester_id: String,
    pub requested_at_ms: i64,
    pub reason: Option<String>,
    pub validators: Vec<String>,
    pub quorum_threshold: usize,
}

impl MembershipDirectoryAnnounce {
    pub fn from_membership_change(
        world_id: &str,
        request: &ConsensusMembershipChangeRequest,
        result: &ConsensusMembershipChangeResult,
    ) -> Self {
        Self {
            world_id: world_id.to_string(),
            requester_id: request.requester_id.clone(),
            requested_at_ms: request.requested_at_ms,
            reason: request.reason.clone(),
            validators: result.validators.clone(),
            quorum_threshold: result.quorum_threshold,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MembershipSyncSubscription {
    pub membership_sub: NetworkSubscription,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipSyncReport {
    pub drained: usize,
    pub applied: usize,
    pub ignored: usize,
}

#[derive(Clone)]
pub struct MembershipSyncClient {
    network: Arc<dyn DistributedNetwork + Send + Sync>,
}

impl MembershipSyncClient {
    pub fn new(network: Arc<dyn DistributedNetwork + Send + Sync>) -> Self {
        Self { network }
    }

    pub fn subscribe(&self, world_id: &str) -> Result<MembershipSyncSubscription, WorldError> {
        let membership_sub = self.network.subscribe(&topic_membership(world_id))?;
        Ok(MembershipSyncSubscription { membership_sub })
    }

    pub fn publish_membership_change(
        &self,
        world_id: &str,
        request: &ConsensusMembershipChangeRequest,
        result: &ConsensusMembershipChangeResult,
    ) -> Result<MembershipDirectoryAnnounce, WorldError> {
        let announce =
            MembershipDirectoryAnnounce::from_membership_change(world_id, request, result);
        let payload = to_canonical_cbor(&announce)?;
        self.network
            .publish(&topic_membership(world_id), &payload)?;
        Ok(announce)
    }

    pub fn drain_announcements(
        &self,
        subscription: &MembershipSyncSubscription,
    ) -> Result<Vec<MembershipDirectoryAnnounce>, WorldError> {
        let raw = subscription.membership_sub.drain();
        let mut announcements = Vec::with_capacity(raw.len());
        for bytes in raw {
            announcements.push(serde_cbor::from_slice(&bytes)?);
        }
        Ok(announcements)
    }

    pub fn sync_membership_directory(
        &self,
        subscription: &MembershipSyncSubscription,
        consensus: &mut QuorumConsensus,
    ) -> Result<MembershipSyncReport, WorldError> {
        let announcements = self.drain_announcements(subscription)?;
        let mut report = MembershipSyncReport {
            drained: announcements.len(),
            applied: 0,
            ignored: 0,
        };

        for announce in announcements {
            let request = ConsensusMembershipChangeRequest {
                requester_id: announce.requester_id,
                requested_at_ms: announce.requested_at_ms,
                reason: announce.reason,
                change: ConsensusMembershipChange::ReplaceValidators {
                    validators: announce.validators,
                    quorum_threshold: announce.quorum_threshold,
                },
            };
            let result = consensus.apply_membership_change(&request)?;
            if result.applied {
                report.applied = report.applied.saturating_add(1);
            } else {
                report.ignored = report.ignored.saturating_add(1);
            }
        }

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::super::distributed::{topic_membership, TOPIC_MEMBERSHIP_SUFFIX};
    use super::super::distributed_net::{DistributedNetwork, InMemoryNetwork};
    use super::*;

    fn membership_request(
        requester_id: &str,
        requested_at_ms: i64,
        change: ConsensusMembershipChange,
    ) -> ConsensusMembershipChangeRequest {
        ConsensusMembershipChangeRequest {
            requester_id: requester_id.to_string(),
            requested_at_ms,
            reason: None,
            change,
        }
    }

    fn sample_consensus() -> QuorumConsensus {
        QuorumConsensus::new(super::super::distributed_consensus::ConsensusConfig {
            validators: vec![
                "seq-1".to_string(),
                "seq-2".to_string(),
                "seq-3".to_string(),
            ],
            quorum_threshold: 0,
        })
        .expect("consensus")
    }

    #[test]
    fn membership_topic_suffix_matches_topic_name() {
        assert_eq!(TOPIC_MEMBERSHIP_SUFFIX, "membership");
        assert_eq!(topic_membership("w1"), "aw.w1.membership");
    }

    #[test]
    fn publish_and_drain_membership_change_announcement() {
        let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
        let sync_client = MembershipSyncClient::new(Arc::clone(&network));
        let subscription = sync_client.subscribe("w1").expect("subscribe");

        let request = membership_request(
            "seq-1",
            100,
            ConsensusMembershipChange::AddValidator {
                validator_id: "seq-4".to_string(),
            },
        );
        let result = ConsensusMembershipChangeResult {
            applied: true,
            validators: vec![
                "seq-1".to_string(),
                "seq-2".to_string(),
                "seq-3".to_string(),
                "seq-4".to_string(),
            ],
            quorum_threshold: 3,
        };

        let published = sync_client
            .publish_membership_change("w1", &request, &result)
            .expect("publish");
        assert_eq!(published.world_id, "w1");

        let drained = sync_client
            .drain_announcements(&subscription)
            .expect("drain announcements");
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0], published);
    }

    #[test]
    fn sync_membership_directory_applies_replace_and_is_idempotent() {
        let network: Arc<dyn DistributedNetwork + Send + Sync> = Arc::new(InMemoryNetwork::new());
        let sync_client = MembershipSyncClient::new(Arc::clone(&network));
        let subscription = sync_client.subscribe("w1").expect("subscribe");

        let request = membership_request(
            "seq-1",
            200,
            ConsensusMembershipChange::AddValidator {
                validator_id: "seq-4".to_string(),
            },
        );
        let result = ConsensusMembershipChangeResult {
            applied: true,
            validators: vec![
                "seq-1".to_string(),
                "seq-2".to_string(),
                "seq-3".to_string(),
                "seq-4".to_string(),
            ],
            quorum_threshold: 3,
        };

        sync_client
            .publish_membership_change("w1", &request, &result)
            .expect("publish 1");
        sync_client
            .publish_membership_change("w1", &request, &result)
            .expect("publish 2");

        let mut consensus = sample_consensus();
        let report = sync_client
            .sync_membership_directory(&subscription, &mut consensus)
            .expect("sync directory");
        assert_eq!(report.drained, 2);
        assert_eq!(report.applied, 1);
        assert_eq!(report.ignored, 1);
        assert_eq!(consensus.validators().len(), 4);
        assert_eq!(consensus.quorum_threshold(), 3);
    }
}
