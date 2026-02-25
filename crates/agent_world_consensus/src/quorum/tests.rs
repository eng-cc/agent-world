use std::collections::BTreeMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use super::super::distributed_dht::InMemoryDht;
use super::*;
use agent_world_proto::distributed_dht::DistributedDht as _;

fn sample_head(height: u64, block_hash: &str) -> WorldHeadAnnounce {
    WorldHeadAnnounce {
        world_id: "w1".to_string(),
        height,
        block_hash: block_hash.to_string(),
        state_root: format!("state-{height}"),
        timestamp_ms: height as i64,
        signature: "sig".to_string(),
    }
}

fn sample_consensus() -> QuorumConsensus {
    QuorumConsensus::new(ConsensusConfig {
        validators: vec![
            "seq-1".to_string(),
            "seq-2".to_string(),
            "seq-3".to_string(),
        ],
        quorum_threshold: 0,
        max_records_per_world: default_max_records_per_world(),
    })
    .expect("consensus")
}

fn sample_lease(holder_id: &str, acquired_at_ms: i64, expires_at_ms: i64) -> LeaseState {
    LeaseState {
        holder_id: holder_id.to_string(),
        lease_id: format!("lease-{holder_id}-{acquired_at_ms}"),
        acquired_at_ms,
        expires_at_ms,
        term: 1,
    }
}

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

fn temp_dir(prefix: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("agent-world-{prefix}-{unique}"))
}

#[test]
fn quorum_threshold_defaults_to_majority() {
    let consensus = sample_consensus();
    assert_eq!(consensus.quorum_threshold(), 2);
}

#[test]
fn proposal_then_vote_commits_head() {
    let mut consensus = sample_consensus();
    let head = sample_head(1, "b1");

    let pending = consensus
        .propose_head(&head, "seq-1", 100)
        .expect("propose");
    assert_eq!(pending.status, ConsensusStatus::Pending);
    assert_eq!(pending.approvals, 1);

    let committed = consensus
        .vote_head("w1", 1, "b1", "seq-2", true, 101, None)
        .expect("vote");
    assert_eq!(committed.status, ConsensusStatus::Committed);
    assert_eq!(committed.approvals, 2);
}

#[test]
fn rejections_can_finalize_proposal() {
    let mut consensus = sample_consensus();
    let head = sample_head(2, "b2");
    consensus
        .propose_head(&head, "seq-1", 200)
        .expect("propose");

    let pending = consensus
        .vote_head(
            "w1",
            2,
            "b2",
            "seq-2",
            false,
            201,
            Some("invalid".to_string()),
        )
        .expect("vote pending");
    assert_eq!(pending.status, ConsensusStatus::Pending);

    let rejected = consensus
        .vote_head(
            "w1",
            2,
            "b2",
            "seq-3",
            false,
            202,
            Some("invalid".to_string()),
        )
        .expect("vote rejected");
    assert_eq!(rejected.status, ConsensusStatus::Rejected);
    assert_eq!(rejected.rejections, 2);
}

#[test]
fn proposal_conflict_is_rejected() {
    let mut consensus = sample_consensus();
    let head = sample_head(3, "b3");
    consensus
        .propose_head(&head, "seq-1", 300)
        .expect("propose");

    let conflict = sample_head(3, "b3-conflict");
    let err = consensus
        .propose_head(&conflict, "seq-2", 301)
        .expect_err("conflict");
    assert!(matches!(
        err,
        WorldError::DistributedValidationFailed { .. }
    ));
}

#[test]
fn publish_happens_only_after_commit() {
    let dht = InMemoryDht::new();
    let mut consensus = sample_consensus();
    let head = sample_head(4, "b4");

    let pending =
        propose_world_head_with_quorum(&dht, &mut consensus, &head, "seq-1", 400).expect("propose");
    assert_eq!(pending.status, ConsensusStatus::Pending);
    assert!(dht.get_world_head("w1").expect("head").is_none());

    let committed = vote_world_head_with_quorum(
        &dht,
        &mut consensus,
        "w1",
        4,
        "b4",
        "seq-2",
        true,
        401,
        None,
    )
    .expect("vote");
    assert_eq!(committed.status, ConsensusStatus::Committed);
    assert_eq!(dht.get_world_head("w1").expect("head"), Some(head.clone()));
}

#[test]
fn stale_proposal_is_rejected_after_commit() {
    let mut consensus = sample_consensus();
    let head1 = sample_head(5, "b5");
    consensus
        .propose_head(&head1, "seq-1", 500)
        .expect("propose");
    consensus
        .vote_head("w1", 5, "b5", "seq-2", true, 501, None)
        .expect("commit");

    let stale = sample_head(5, "b5");
    let err = consensus
        .propose_head(&stale, "seq-3", 502)
        .expect_err("stale");
    assert!(matches!(
        err,
        WorldError::DistributedValidationFailed { .. }
    ));
}

#[test]
fn finalized_records_are_pruned_by_world_limit() {
    let mut consensus = QuorumConsensus::new(ConsensusConfig {
        validators: vec![
            "seq-1".to_string(),
            "seq-2".to_string(),
            "seq-3".to_string(),
        ],
        quorum_threshold: 2,
        max_records_per_world: 2,
    })
    .expect("consensus");

    for height in 1..=3 {
        let block_hash = format!("b{height}");
        let head = sample_head(height, block_hash.as_str());
        let _ = consensus
            .propose_head(&head, "seq-1", height as i64)
            .expect("propose");
        let decision = consensus
            .vote_head(
                "w1",
                height,
                block_hash.as_str(),
                "seq-2",
                true,
                height as i64 + 1,
                None,
            )
            .expect("vote");
        assert_eq!(decision.status, ConsensusStatus::Committed);
    }

    assert!(consensus.record("w1", 1).is_none());
    assert!(consensus.record("w1", 2).is_some());
    assert!(consensus.record("w1", 3).is_some());
}

#[test]
fn pending_records_are_preserved_under_pruning_pressure() {
    let mut consensus = QuorumConsensus::new(ConsensusConfig {
        validators: vec![
            "seq-1".to_string(),
            "seq-2".to_string(),
            "seq-3".to_string(),
        ],
        quorum_threshold: 2,
        max_records_per_world: 1,
    })
    .expect("consensus");

    let committed_head = sample_head(1, "b1");
    let _ = consensus
        .propose_head(&committed_head, "seq-1", 10)
        .expect("propose committed");
    let _ = consensus
        .vote_head("w1", 1, "b1", "seq-2", true, 11, None)
        .expect("commit");

    let pending_head = sample_head(2, "b2");
    let decision = consensus
        .propose_head(&pending_head, "seq-1", 12)
        .expect("propose pending");
    assert_eq!(decision.status, ConsensusStatus::Pending);

    assert!(consensus.record("w1", 1).is_none());
    let pending = consensus.record("w1", 2).expect("pending record");
    assert_eq!(pending.status, ConsensusStatus::Pending);
}

#[test]
fn membership_add_validator_updates_threshold() {
    let mut consensus = sample_consensus();
    let request = membership_request(
        "seq-1",
        1_000,
        ConsensusMembershipChange::AddValidator {
            validator_id: "seq-4".to_string(),
        },
    );
    let result = consensus
        .apply_membership_change(&request)
        .expect("membership add");

    assert!(result.applied);
    assert_eq!(result.validators.len(), 4);
    assert_eq!(result.quorum_threshold, 3);
}

#[test]
fn membership_remove_validator_updates_threshold() {
    let mut consensus = sample_consensus();
    let add_request = membership_request(
        "seq-1",
        1_100,
        ConsensusMembershipChange::AddValidator {
            validator_id: "seq-4".to_string(),
        },
    );
    consensus
        .apply_membership_change(&add_request)
        .expect("membership add");

    let remove_request = membership_request(
        "seq-1",
        1_101,
        ConsensusMembershipChange::RemoveValidator {
            validator_id: "seq-4".to_string(),
        },
    );
    let result = consensus
        .apply_membership_change(&remove_request)
        .expect("membership remove");

    assert!(result.applied);
    assert_eq!(result.validators.len(), 3);
    assert_eq!(result.quorum_threshold, 3);
}

#[test]
fn membership_change_is_blocked_when_pending_exists() {
    let mut consensus = sample_consensus();
    let head = sample_head(6, "b6");
    consensus
        .propose_head(&head, "seq-1", 600)
        .expect("propose pending");

    let request = membership_request(
        "seq-1",
        1_200,
        ConsensusMembershipChange::AddValidator {
            validator_id: "seq-4".to_string(),
        },
    );
    let err = consensus
        .apply_membership_change(&request)
        .expect_err("pending should block");
    assert!(matches!(
        err,
        WorldError::DistributedValidationFailed { .. }
    ));
}

#[test]
fn membership_change_with_lease_requires_holder_and_active_lease() {
    let mut consensus = sample_consensus();
    let request = membership_request(
        "seq-1",
        1_300,
        ConsensusMembershipChange::AddValidator {
            validator_id: "seq-4".to_string(),
        },
    );

    let mismatched_lease = sample_lease("seq-2", 1_200, 1_400);
    let err = consensus
        .apply_membership_change_with_lease(&request, Some(&mismatched_lease))
        .expect_err("holder mismatch");
    assert!(matches!(
        err,
        WorldError::DistributedValidationFailed { .. }
    ));

    let expired_lease = sample_lease("seq-1", 1_200, 1_250);
    let err = consensus
        .apply_membership_change_with_lease(&request, Some(&expired_lease))
        .expect_err("expired lease");
    assert!(matches!(
        err,
        WorldError::DistributedValidationFailed { .. }
    ));

    let active_lease = sample_lease("seq-1", 1_200, 1_400);
    let result = consensus
        .apply_membership_change_with_lease(&request, Some(&active_lease))
        .expect("active lease apply");
    assert!(result.applied);
    assert_eq!(result.validators.len(), 4);
}

#[test]
fn ensure_lease_holder_validator_auto_adds_holder() {
    let mut consensus = sample_consensus();
    let lease = sample_lease("seq-9", 1_500, 1_800);

    let first = ensure_lease_holder_validator(&mut consensus, Some(&lease), 1_600)
        .expect("auto add lease holder");
    assert!(first.applied);
    assert_eq!(first.validators.len(), 4);
    assert_eq!(first.quorum_threshold, 3);

    let second = ensure_lease_holder_validator(&mut consensus, Some(&lease), 1_601)
        .expect("already present");
    assert!(!second.applied);
    assert_eq!(second.validators.len(), 4);

    let none = ensure_lease_holder_validator(&mut consensus, None, 1_602).expect("none lease");
    assert!(!none.applied);
}

#[test]
fn snapshot_round_trip_restores_consensus_records() {
    let dir = temp_dir("consensus-snapshot");
    fs::create_dir_all(&dir).expect("mkdir");
    let path = dir.join("consensus_snapshot.json");

    let mut consensus = sample_consensus();
    let head = sample_head(7, "b7");
    consensus
        .propose_head(&head, "seq-1", 700)
        .expect("propose");
    consensus
        .vote_head("w1", 7, "b7", "seq-2", true, 701, None)
        .expect("commit");

    let request = membership_request(
        "seq-1",
        1_700,
        ConsensusMembershipChange::AddValidator {
            validator_id: "seq-4".to_string(),
        },
    );
    consensus
        .apply_membership_change(&request)
        .expect("membership add");

    consensus
        .save_snapshot_to_path(&path)
        .expect("save snapshot");
    let restored = QuorumConsensus::load_snapshot_from_path(&path).expect("load snapshot");

    assert_eq!(restored.quorum_threshold(), 3);
    assert_eq!(
        restored.record("w1", 7).map(|record| record.status),
        Some(ConsensusStatus::Committed)
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn load_snapshot_rejects_unknown_validator_vote_payload_mismatch() {
    let dir = temp_dir("consensus-invalid-snapshot");
    fs::create_dir_all(&dir).expect("mkdir");
    let path = dir.join("consensus_snapshot.json");

    let mut votes = BTreeMap::new();
    votes.insert(
        "validator-1".to_string(),
        ConsensusVote {
            validator_id: "validator-x".to_string(),
            approve: true,
            reason: None,
            voted_at_ms: 700,
        },
    );
    let snapshot = ConsensusSnapshotFile {
        version: CONSENSUS_SNAPSHOT_VERSION,
        validators: vec![
            "seq-1".to_string(),
            "seq-2".to_string(),
            "seq-3".to_string(),
        ],
        quorum_threshold: 2,
        max_records_per_world: default_max_records_per_world(),
        records: vec![HeadConsensusRecord {
            head: sample_head(8, "b8"),
            proposer_id: "seq-1".to_string(),
            proposed_at_ms: 700,
            quorum_threshold: 2,
            validator_count: 3,
            status: ConsensusStatus::Pending,
            votes,
        }],
    };
    fs::write(
        &path,
        serde_json::to_vec_pretty(&snapshot).expect("serialize snapshot"),
    )
    .expect("write snapshot");

    let err = QuorumConsensus::load_snapshot_from_path(&path).expect_err("reject snapshot");
    assert!(matches!(
        err,
        WorldError::DistributedValidationFailed { .. }
    ));

    let _ = fs::remove_dir_all(&dir);
}
