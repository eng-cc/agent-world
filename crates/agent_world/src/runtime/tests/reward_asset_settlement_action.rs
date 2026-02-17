use super::super::*;
use ed25519_dalek::SigningKey;

fn settlement(node_id: &str, awarded_points: u64) -> NodeSettlement {
    NodeSettlement {
        node_id: node_id.to_string(),
        obligation_met: true,
        compute_score: 0.0,
        storage_score: 0.0,
        uptime_score: 0.0,
        reliability_score: 0.0,
        storage_reward_score: 0.0,
        rewardable_storage_bytes: 0,
        penalty_score: 0.0,
        total_score: 0.0,
        main_awarded_points: awarded_points,
        storage_awarded_points: 0,
        awarded_points,
        cumulative_points: awarded_points,
    }
}

fn settlement_report(epoch_index: u64, settlements: Vec<NodeSettlement>) -> EpochSettlementReport {
    let distributed_points = settlements
        .iter()
        .map(|settlement| settlement.awarded_points)
        .sum::<u64>();
    EpochSettlementReport {
        epoch_index,
        pool_points: distributed_points,
        storage_pool_points: 0,
        distributed_points,
        storage_distributed_points: 0,
        total_distributed_points: distributed_points,
        settlements,
    }
}

fn bind_node_identity(world: &mut World, node_id: &str) {
    let public_key = format!("public-key-{node_id}");
    world
        .bind_node_identity(node_id, public_key.as_str())
        .expect("bind node identity");
}

fn bind_node_identity_with_seed(world: &mut World, node_id: &str, seed: u8) -> String {
    let private = [seed; 32];
    let signing_key = SigningKey::from_bytes(&private);
    let private_key_hex = hex::encode(signing_key.to_bytes());
    let public_key_hex = hex::encode(signing_key.verifying_key().to_bytes());
    world
        .bind_node_identity(node_id, public_key_hex.as_str())
        .expect("bind node identity with keypair");
    private_key_hex
}

#[test]
fn reward_asset_settlement_action_applies_signed_records_via_step() {
    let mut world = World::new();
    bind_node_identity(&mut world, "node-a");
    let signer_private_key = bind_node_identity_with_seed(&mut world, "node-signer", 9);
    world.set_reward_signature_governance_policy(RewardSignatureGovernancePolicy {
        require_mintsig_v2: true,
        allow_mintsig_v1_fallback: false,
        require_redeem_signature: false,
        require_redeem_signer_match_node_id: false,
    });
    world.set_reward_asset_config(RewardAssetConfig {
        points_per_credit: 10,
        ..RewardAssetConfig::default()
    });

    let report = settlement_report(20, vec![settlement("node-a", 50)]);
    let mut preview = world.clone();
    let minted_records = preview
        .apply_node_points_settlement_mint_v2(&report, "node-signer", signer_private_key.as_str())
        .expect("build settlement records");
    assert_eq!(minted_records.len(), 1);

    world.submit_action(Action::ApplyNodePointsSettlementSigned {
        report: report.clone(),
        signer_node_id: "node-signer".to_string(),
        mint_records: minted_records.clone(),
    });
    world.step().expect("apply settlement action");

    assert_eq!(world.node_power_credit_balance("node-a"), 5);
    assert_eq!(world.reward_mint_records().len(), 1);
    match &world.journal().events.last().expect("event").body {
        WorldEventBody::Domain(DomainEvent::NodePointsSettlementApplied {
            signer_node_id,
            settlement_hash,
            minted_records,
            ..
        }) => {
            assert_eq!(signer_node_id, "node-signer");
            assert!(!settlement_hash.is_empty());
            assert_eq!(minted_records.len(), 1);
            assert_eq!(minted_records[0].node_id, "node-a");
            assert_eq!(minted_records[0].minted_power_credits, 5);
        }
        other => panic!("expected NodePointsSettlementApplied, got {other:?}"),
    }
}

#[test]
fn reward_asset_settlement_action_rejects_tampered_mint_record() {
    let mut world = World::new();
    bind_node_identity(&mut world, "node-a");
    let signer_private_key = bind_node_identity_with_seed(&mut world, "node-signer", 10);
    world.set_reward_signature_governance_policy(RewardSignatureGovernancePolicy {
        require_mintsig_v2: true,
        allow_mintsig_v1_fallback: false,
        require_redeem_signature: false,
        require_redeem_signer_match_node_id: false,
    });
    world.set_reward_asset_config(RewardAssetConfig {
        points_per_credit: 10,
        ..RewardAssetConfig::default()
    });

    let report = settlement_report(21, vec![settlement("node-a", 40)]);
    let mut preview = world.clone();
    let mut minted_records = preview
        .apply_node_points_settlement_mint_v2(&report, "node-signer", signer_private_key.as_str())
        .expect("build settlement records");
    minted_records[0].signature = "mintsig:v2:deadbeef".to_string();

    world.submit_action(Action::ApplyNodePointsSettlementSigned {
        report,
        signer_node_id: "node-signer".to_string(),
        mint_records: minted_records,
    });
    world.step().expect("settlement action should be rejected, not fail");

    assert_eq!(world.node_power_credit_balance("node-a"), 0);
    assert!(world.reward_mint_records().is_empty());
    match &world.journal().events.last().expect("event").body {
        WorldEventBody::Domain(DomainEvent::ActionRejected { reason, .. }) => match reason {
            RejectReason::RuleDenied { notes } => {
                assert!(
                    notes
                        .iter()
                        .any(|note| note.contains("mint record signature invalid"))
                );
            }
            other => panic!("expected rule denied reject, got {other:?}"),
        },
        other => panic!("expected ActionRejected, got {other:?}"),
    }
}
