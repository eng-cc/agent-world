use super::super::*;

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

#[test]
fn reward_asset_mint_and_burn_updates_balance() {
    let mut world = World::new();
    assert_eq!(world.node_power_credit_balance("node-a"), 0);

    world
        .mint_node_power_credits("node-a", 120)
        .expect("mint power credits");
    assert_eq!(world.node_power_credit_balance("node-a"), 120);

    world
        .burn_node_power_credits("node-a", 20)
        .expect("burn power credits");
    assert_eq!(world.node_power_credit_balance("node-a"), 100);

    let balance = world
        .node_asset_balance("node-a")
        .expect("node asset balance");
    assert_eq!(balance.node_id, "node-a");
    assert_eq!(balance.power_credit_balance, 100);
    assert_eq!(balance.total_minted_credits, 120);
    assert_eq!(balance.total_burned_credits, 20);

    let err = world
        .burn_node_power_credits("node-a", 101)
        .expect_err("insufficient credits must fail");
    assert!(matches!(err, WorldError::ResourceBalanceInvalid { .. }));
}

#[test]
fn reward_asset_rejects_empty_node_id() {
    let mut world = World::new();
    let err = world
        .mint_node_power_credits("", 10)
        .expect_err("empty node id must fail");
    assert!(matches!(err, WorldError::ResourceBalanceInvalid { .. }));
}

#[test]
fn reward_asset_snapshot_roundtrip_persists_state() {
    let mut world = World::new();
    let config = RewardAssetConfig {
        points_per_credit: 5,
        credits_per_power_unit: 2,
        max_redeem_power_per_epoch: 5000,
        min_redeem_power_unit: 10,
    };
    world.set_reward_asset_config(config.clone());

    let reserve = ProtocolPowerReserve {
        epoch_index: 7,
        available_power_units: 3200,
        redeemed_power_units: 100,
    };
    world.set_protocol_power_reserve(reserve.clone());
    world
        .mint_node_power_credits("node-1", 90)
        .expect("mint node-1");
    world
        .burn_node_power_credits("node-1", 15)
        .expect("burn node-1");

    let snapshot = world.snapshot();
    let restored = World::from_snapshot(snapshot, world.journal().clone()).expect("restore");

    assert_eq!(restored.reward_asset_config(), &config);
    assert_eq!(restored.protocol_power_reserve(), &reserve);
    assert_eq!(restored.node_power_credit_balance("node-1"), 75);
    let balance = restored
        .node_asset_balance("node-1")
        .expect("node-1 balance should exist");
    assert_eq!(balance.total_minted_credits, 90);
    assert_eq!(balance.total_burned_credits, 15);
}

#[test]
fn reward_asset_settlement_mint_records_balance_changes() {
    let mut world = World::new();
    world.set_reward_asset_config(RewardAssetConfig {
        points_per_credit: 10,
        ..RewardAssetConfig::default()
    });

    let report = settlement_report(3, vec![settlement("node-a", 27), settlement("node-b", 9)]);

    let minted = world
        .apply_node_points_settlement_mint(&report, "node-signer")
        .expect("apply settlement mint");
    assert_eq!(minted.len(), 1);
    let record = &minted[0];
    assert_eq!(record.epoch_index, 3);
    assert_eq!(record.node_id, "node-a");
    assert_eq!(record.source_awarded_points, 27);
    assert_eq!(record.minted_power_credits, 2);
    assert_eq!(record.signer_node_id, "node-signer");
    assert!(!record.settlement_hash.is_empty());

    assert_eq!(world.node_power_credit_balance("node-a"), 2);
    assert_eq!(world.node_power_credit_balance("node-b"), 0);
    assert_eq!(world.reward_mint_records().len(), 1);
}

#[test]
fn reward_asset_settlement_mint_is_idempotent_per_epoch_node() {
    let mut world = World::new();
    world.set_reward_asset_config(RewardAssetConfig {
        points_per_credit: 5,
        ..RewardAssetConfig::default()
    });

    let report = settlement_report(5, vec![settlement("node-a", 25), settlement("node-b", 11)]);

    let first = world
        .apply_node_points_settlement_mint(&report, "node-signer")
        .expect("first apply");
    assert_eq!(first.len(), 2);
    assert_eq!(world.node_power_credit_balance("node-a"), 5);
    assert_eq!(world.node_power_credit_balance("node-b"), 2);

    let second = world
        .apply_node_points_settlement_mint(&report, "node-signer")
        .expect("second apply should be idempotent");
    assert!(second.is_empty());
    assert_eq!(world.node_power_credit_balance("node-a"), 5);
    assert_eq!(world.node_power_credit_balance("node-b"), 2);
    assert_eq!(world.reward_mint_records().len(), 2);
}

#[test]
fn reward_asset_settlement_rejects_zero_points_per_credit() {
    let mut world = World::new();
    world.set_reward_asset_config(RewardAssetConfig {
        points_per_credit: 0,
        ..RewardAssetConfig::default()
    });
    let report = settlement_report(1, vec![settlement("node-a", 10)]);

    let err = world
        .apply_node_points_settlement_mint(&report, "node-signer")
        .expect_err("points_per_credit=0 should fail");
    assert!(matches!(err, WorldError::ResourceBalanceInvalid { .. }));
}

#[test]
fn reward_asset_settlement_rejects_empty_signer_node_id() {
    let mut world = World::new();
    let report = settlement_report(1, vec![settlement("node-a", 10)]);

    let err = world
        .apply_node_points_settlement_mint(&report, "   ")
        .expect_err("empty signer_node_id should fail");
    assert!(matches!(err, WorldError::ResourceBalanceInvalid { .. }));
}

#[test]
fn reward_asset_snapshot_roundtrip_persists_mint_records() {
    let mut world = World::new();
    world.set_reward_asset_config(RewardAssetConfig {
        points_per_credit: 10,
        ..RewardAssetConfig::default()
    });
    let report = settlement_report(9, vec![settlement("node-a", 30)]);
    let minted = world
        .apply_node_points_settlement_mint(&report, "node-signer")
        .expect("mint should succeed");
    assert_eq!(minted.len(), 1);

    let snapshot = world.snapshot();
    let restored = World::from_snapshot(snapshot, world.journal().clone()).expect("restore");
    assert_eq!(restored.reward_mint_records().len(), 1);
    assert_eq!(restored.node_power_credit_balance("node-a"), 3);
    let record = &restored.reward_mint_records()[0];
    assert_eq!(record.epoch_index, 9);
    assert_eq!(record.node_id, "node-a");
    assert_eq!(record.minted_power_credits, 3);
}
