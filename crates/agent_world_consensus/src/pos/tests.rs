use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use super::super::distributed_dht::InMemoryDht;
use super::*;
use agent_world_proto::distributed_dht::DistributedDht as _;

fn validators() -> Vec<PosValidator> {
    vec![
        PosValidator {
            validator_id: "val-a".to_string(),
            stake: 40,
        },
        PosValidator {
            validator_id: "val-b".to_string(),
            stake: 35,
        },
        PosValidator {
            validator_id: "val-c".to_string(),
            stake: 25,
        },
    ]
}

fn sample_head(height: u64, block_hash: &str) -> WorldHeadAnnounce {
    WorldHeadAnnounce {
        world_id: "w1".to_string(),
        height,
        block_hash: block_hash.to_string(),
        state_root: format!("state-{}", height),
        timestamp_ms: height as i64,
        signature: "sig".to_string(),
    }
}

fn sample_consensus() -> PosConsensus {
    PosConsensus::new(PosConsensusConfig::ethereum_like(validators())).expect("pos consensus")
}

fn find_slot_with_proposer(consensus: &PosConsensus, proposer: &str, start: u64) -> u64 {
    for slot in start..(start + 256) {
        if consensus.expected_proposer(slot).as_deref() == Some(proposer) {
            return slot;
        }
    }
    panic!("slot not found for proposer {}", proposer);
}

fn temp_dir(prefix: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("duration")
        .as_nanos();
    std::env::temp_dir().join(format!("agent-world-pos-{prefix}-{unique}"))
}

#[test]
fn pos_commit_requires_supermajority_stake() {
    let mut consensus = sample_consensus();
    let proposer = consensus.expected_proposer(0).expect("proposer");
    let head = sample_head(1, "b1");
    let pending = consensus
        .propose_head(&head, &proposer, 0, 100)
        .expect("propose");
    assert_eq!(pending.status, PosConsensusStatus::Pending);
    assert_eq!(pending.required_stake, 67);

    let next_validator = consensus
        .validators()
        .into_iter()
        .find(|validator| validator.validator_id != proposer)
        .expect("next validator");
    let decision = consensus
        .attest_head(
            "w1",
            1,
            "b1",
            &next_validator.validator_id,
            true,
            101,
            0,
            0,
            None,
        )
        .expect("attest");
    if next_validator.stake + pending.approved_stake >= pending.required_stake {
        assert_eq!(decision.status, PosConsensusStatus::Committed);
    } else {
        assert_eq!(decision.status, PosConsensusStatus::Pending);
    }
}

#[test]
fn pos_accepts_extreme_supermajority_ratio_just_above_half() {
    let denominator = u64::MAX;
    let numerator = denominator / 2 + 1;
    let consensus = PosConsensus::new(PosConsensusConfig {
        validators: validators(),
        supermajority_numerator: numerator,
        supermajority_denominator: denominator,
        epoch_length_slots: 32,
    })
    .expect("extreme ratio should be valid");
    assert_eq!(consensus.required_stake(), 51);
}

#[test]
fn pos_rejects_when_remaining_stake_cannot_reach_supermajority() {
    let mut consensus = sample_consensus();
    let proposer = consensus.expected_proposer(1).expect("proposer");
    let head = sample_head(2, "b2");
    let _ = consensus
        .propose_head(&head, &proposer, 1, 200)
        .expect("propose");
    let rejector = consensus
        .validators()
        .into_iter()
        .filter(|validator| validator.validator_id != proposer)
        .max_by_key(|validator| validator.stake)
        .expect("rejector");
    let decision = consensus
        .attest_head(
            "w1",
            2,
            "b2",
            &rejector.validator_id,
            false,
            201,
            0,
            0,
            Some("invalid transition".to_string()),
        )
        .expect("reject");
    assert_eq!(decision.status, PosConsensusStatus::Rejected);
}

#[test]
fn pos_proposer_must_match_expected_slot_proposer() {
    let mut consensus = sample_consensus();
    let expected = consensus.expected_proposer(2).expect("expected proposer");
    let wrong = consensus
        .validators()
        .into_iter()
        .find(|validator| validator.validator_id != expected)
        .expect("wrong proposer")
        .validator_id;
    let err = consensus
        .propose_head(&sample_head(3, "b3"), &wrong, 2, 300)
        .expect_err("must reject wrong proposer");
    assert!(matches!(
        err,
        WorldError::DistributedValidationFailed { .. }
    ));
}

#[test]
fn pos_detects_double_vote_slashing_condition() {
    let mut consensus = sample_consensus();
    let validator = "val-c";
    let slot_a = find_slot_with_proposer(&consensus, "val-a", 0);
    let slot_b = find_slot_with_proposer(&consensus, "val-b", slot_a + 1);
    let head_a = sample_head(10, "b10");
    let head_b = sample_head(11, "b11");
    consensus
        .propose_head(&head_a, "val-a", slot_a, 1000)
        .expect("propose a");
    consensus
        .propose_head(&head_b, "val-b", slot_b, 1001)
        .expect("propose b");

    let target_epoch = consensus.slot_epoch(slot_a);
    consensus
        .attest_head(
            "w1",
            10,
            "b10",
            validator,
            true,
            1002,
            target_epoch.saturating_sub(1),
            target_epoch,
            None,
        )
        .expect("first attestation");

    let err = consensus
        .attest_head(
            "w1",
            11,
            "b11",
            validator,
            true,
            1003,
            target_epoch.saturating_sub(1),
            target_epoch,
            None,
        )
        .expect_err("double vote");
    assert!(matches!(
        err,
        WorldError::DistributedValidationFailed { .. }
    ));
}

#[test]
fn pos_detects_surround_vote_slashing_condition() {
    let mut consensus = sample_consensus();
    let validator = "val-b";
    let slot_epoch4 = find_slot_with_proposer(&consensus, "val-a", 128);
    let slot_epoch3 = find_slot_with_proposer(&consensus, "val-c", 96);
    let head_a = sample_head(20, "b20");
    let head_b = sample_head(21, "b21");
    consensus
        .propose_head(&head_a, "val-a", slot_epoch4, 2000)
        .expect("propose epoch4");
    consensus
        .propose_head(&head_b, "val-c", slot_epoch3, 2001)
        .expect("propose epoch3");

    let epoch4 = consensus.slot_epoch(slot_epoch4);
    let epoch3 = consensus.slot_epoch(slot_epoch3);
    consensus
        .attest_head(
            "w1",
            20,
            "b20",
            validator,
            true,
            2002,
            epoch4.saturating_sub(2),
            epoch4,
            None,
        )
        .expect("first vote");

    let err = consensus
        .attest_head("w1", 21, "b21", validator, true, 2003, epoch3, epoch3, None)
        .expect_err("surround vote");
    assert!(matches!(
        err,
        WorldError::DistributedValidationFailed { .. }
    ));
}

#[test]
fn dht_publish_happens_only_after_pos_commit() {
    let mut consensus = sample_consensus();
    let dht = InMemoryDht::new();
    let proposer = consensus.expected_proposer(5).expect("proposer");
    let head = sample_head(30, "b30");

    let pending = propose_world_head_with_pos(&dht, &mut consensus, &head, &proposer, 5, 3000)
        .expect("propose");
    assert_eq!(pending.status, PosConsensusStatus::Pending);
    assert!(dht.get_world_head("w1").expect("head").is_none());

    let approver = consensus
        .validators()
        .into_iter()
        .find(|validator| validator.validator_id != proposer)
        .expect("approver");
    let target_epoch = consensus.slot_epoch(5);
    let decision = attest_world_head_with_pos(
        &dht,
        &mut consensus,
        "w1",
        30,
        "b30",
        &approver.validator_id,
        true,
        3001,
        0,
        target_epoch,
        None,
    )
    .expect("attest");

    if matches!(decision.status, PosConsensusStatus::Committed) {
        assert_eq!(dht.get_world_head("w1").expect("head"), Some(head));
    }
}

#[test]
fn pos_snapshot_round_trip_restores_records() {
    let dir = temp_dir("snapshot");
    let path = dir.join("pos-consensus.json");
    let mut consensus = sample_consensus();
    let proposer = consensus.expected_proposer(8).expect("proposer");
    consensus
        .propose_head(&sample_head(40, "b40"), &proposer, 8, 4000)
        .expect("propose");

    consensus
        .save_snapshot_to_path(&path)
        .expect("save snapshot");
    let loaded = PosConsensus::load_snapshot_from_path(&path).expect("load snapshot");
    let loaded_record = loaded.record("w1", 40).expect("record");
    assert_eq!(loaded_record.head.block_hash, "b40");
    assert_eq!(loaded_record.epoch, loaded.slot_epoch(8));

    let _ = fs::remove_dir_all(dir);
}
