use super::*;

fn empty_action_root() -> String {
    compute_consensus_action_root(&[]).expect("empty action root")
}

#[test]
fn pos_engine_applies_gossiped_proposal_and_attestation() {
    let validators = vec![
        PosValidator {
            validator_id: "node-a".to_string(),
            stake: 60,
        },
        PosValidator {
            validator_id: "node-b".to_string(),
            stake: 40,
        },
    ];
    let config = NodeConfig::new("node-b", "world-gossip-proposal", NodeRole::Observer)
        .expect("config")
        .with_pos_validators(validators)
        .expect("validators")
        .with_auto_attest_all_validators(false);
    let mut engine = PosNodeEngine::new(&config).expect("engine");

    let proposal = GossipProposalMessage {
        version: 1,
        world_id: config.world_id.clone(),
        node_id: "node-a".to_string(),
        player_id: "node-a".to_string(),
        proposer_id: "node-a".to_string(),
        height: 1,
        slot: 0,
        epoch: 0,
        block_hash: format!("{}:h1:s0:p{}", config.world_id, "node-a"),
        action_root: empty_action_root(),
        actions: Vec::new(),
        proposed_at_ms: 1_000,
        public_key_hex: None,
        signature_hex: None,
    };
    engine
        .ingest_proposal_message(&config.world_id, &proposal)
        .expect("ingest proposal");

    let attestation = GossipAttestationMessage {
        version: 1,
        world_id: config.world_id.clone(),
        node_id: "node-b".to_string(),
        player_id: "node-b".to_string(),
        validator_id: "node-b".to_string(),
        height: 1,
        slot: 0,
        epoch: 0,
        block_hash: proposal.block_hash.clone(),
        approve: true,
        source_epoch: 0,
        target_epoch: 0,
        voted_at_ms: 1_001,
        reason: Some("gossip attestation".to_string()),
        public_key_hex: None,
        signature_hex: None,
    };
    engine
        .ingest_attestation_message(&config.world_id, &attestation)
        .expect("ingest attestation");

    let snapshot = engine
        .tick(
            &config.node_id,
            &config.world_id,
            1_002,
            None,
            None,
            None,
            None,
            Vec::new(),
            None,
        )
        .expect("tick");
    assert_eq!(snapshot.consensus_snapshot.committed_height, 1);
    assert_eq!(
        snapshot.consensus_snapshot.last_status,
        Some(PosConsensusStatus::Committed)
    );
}

#[test]
fn pos_engine_ignores_gossiped_proposal_when_player_binding_mismatches() {
    let validators = vec![
        PosValidator {
            validator_id: "node-a".to_string(),
            stake: 60,
        },
        PosValidator {
            validator_id: "node-b".to_string(),
            stake: 40,
        },
    ];
    let config = NodeConfig::new("node-b", "world-gossip-player-mismatch", NodeRole::Observer)
        .expect("config")
        .with_pos_validators(validators)
        .expect("validators")
        .with_auto_attest_all_validators(false);
    let mut engine = PosNodeEngine::new(&config).expect("engine");

    let proposal = GossipProposalMessage {
        version: 1,
        world_id: config.world_id.clone(),
        node_id: "node-a".to_string(),
        player_id: "other-player".to_string(),
        proposer_id: "node-a".to_string(),
        height: 1,
        slot: 0,
        epoch: 0,
        block_hash: format!("{}:h1:s0:p{}", config.world_id, "node-a"),
        action_root: empty_action_root(),
        actions: Vec::new(),
        proposed_at_ms: 1_000,
        public_key_hex: None,
        signature_hex: None,
    };
    engine
        .ingest_proposal_message(&config.world_id, &proposal)
        .expect("ingest proposal");
    assert!(engine.pending.is_none());
}
