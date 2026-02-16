use super::super::*;

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

    let balance = world.node_asset_balance("node-a").expect("node asset balance");
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
