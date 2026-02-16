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

#[test]
fn reward_asset_settlement_mint_respects_system_order_pool_budget() {
    let mut world = World::new();
    world.set_reward_asset_config(RewardAssetConfig {
        points_per_credit: 10,
        ..RewardAssetConfig::default()
    });
    world.set_system_order_pool_budget(6, 8);
    let report = settlement_report(
        6,
        vec![settlement("node-a", 100), settlement("node-b", 100)],
    );

    let minted = world
        .apply_node_points_settlement_mint(&report, "node-signer")
        .expect("apply settlement mint with budget");
    assert_eq!(minted.len(), 2);
    assert_eq!(
        minted
            .iter()
            .map(|record| record.minted_power_credits)
            .sum::<u64>(),
        8
    );
    assert_eq!(world.node_power_credit_balance("node-a"), 4);
    assert_eq!(world.node_power_credit_balance("node-b"), 4);

    let budget = world
        .system_order_pool_budget(6)
        .expect("epoch 6 budget should exist");
    assert_eq!(budget.remaining_credit_budget, 0);
    assert_eq!(budget.node_credit_caps.get("node-a"), Some(&4));
    assert_eq!(budget.node_credit_caps.get("node-b"), Some(&4));
    assert_eq!(budget.node_credit_allocated.get("node-a"), Some(&4));
    assert_eq!(budget.node_credit_allocated.get("node-b"), Some(&4));
}

#[test]
fn reward_asset_settlement_mint_budget_remainder_prefers_higher_points() {
    let mut world = World::new();
    world.set_reward_asset_config(RewardAssetConfig {
        points_per_credit: 10,
        ..RewardAssetConfig::default()
    });
    world.set_system_order_pool_budget(7, 5);
    let report = settlement_report(
        7,
        vec![settlement("node-a", 300), settlement("node-b", 100)],
    );

    let minted = world
        .apply_node_points_settlement_mint(&report, "node-signer")
        .expect("apply settlement mint with weighted budget");
    assert_eq!(minted.len(), 2);

    let node_a = minted
        .iter()
        .find(|record| record.node_id == "node-a")
        .expect("node-a record");
    let node_b = minted
        .iter()
        .find(|record| record.node_id == "node-b")
        .expect("node-b record");
    assert_eq!(node_a.minted_power_credits, 4);
    assert_eq!(node_b.minted_power_credits, 1);
    assert_eq!(world.node_power_credit_balance("node-a"), 4);
    assert_eq!(world.node_power_credit_balance("node-b"), 1);
}

#[test]
fn reward_asset_snapshot_roundtrip_persists_system_order_pool_budget() {
    let mut world = World::new();
    world.set_reward_asset_config(RewardAssetConfig {
        points_per_credit: 10,
        ..RewardAssetConfig::default()
    });
    world.set_system_order_pool_budget(8, 6);
    let report = settlement_report(
        8,
        vec![settlement("node-a", 200), settlement("node-b", 100)],
    );
    world
        .apply_node_points_settlement_mint(&report, "node-signer")
        .expect("mint with budget");

    let snapshot = world.snapshot();
    let restored = World::from_snapshot(snapshot, world.journal().clone()).expect("restore");
    let budget = restored
        .system_order_pool_budget(8)
        .expect("budget should persist");
    assert_eq!(budget.total_credit_budget, 6);
    assert_eq!(budget.remaining_credit_budget, 0);
    assert_eq!(budget.node_credit_allocated.get("node-a"), Some(&4));
    assert_eq!(budget.node_credit_allocated.get("node-b"), Some(&2));
}

#[test]
fn reward_asset_redeem_power_action_updates_balances_and_reserve() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: crate::geometry::GeoPos::new(0.0, 0.0, 0.0),
    });
    world.step().expect("register target agent");
    let initial_electricity = world
        .agent_resource_balance("agent-1", crate::simulator::ResourceKind::Electricity)
        .expect("query target electricity");

    world.set_reward_asset_config(RewardAssetConfig {
        credits_per_power_unit: 4,
        ..RewardAssetConfig::default()
    });
    world.set_protocol_power_reserve(ProtocolPowerReserve {
        epoch_index: 2,
        available_power_units: 50,
        redeemed_power_units: 0,
    });
    world
        .mint_node_power_credits("node-a", 20)
        .expect("mint node credits");

    world.submit_action(Action::RedeemPower {
        node_id: "node-a".to_string(),
        target_agent_id: "agent-1".to_string(),
        redeem_credits: 9,
        nonce: 1,
    });
    world.step().expect("redeem power");

    assert_eq!(world.node_power_credit_balance("node-a"), 11);
    assert_eq!(
        world
            .agent_resource_balance("agent-1", crate::simulator::ResourceKind::Electricity)
            .expect("query target electricity after redeem"),
        initial_electricity + 2
    );
    assert_eq!(world.protocol_power_reserve().available_power_units, 48);
    assert_eq!(world.protocol_power_reserve().redeemed_power_units, 2);

    let event = world.journal().events.last().expect("redeem event");
    match &event.body {
        WorldEventBody::Domain(DomainEvent::PowerRedeemed {
            node_id,
            target_agent_id,
            burned_credits,
            granted_power_units,
            reserve_remaining,
            nonce,
        }) => {
            assert_eq!(node_id, "node-a");
            assert_eq!(target_agent_id, "agent-1");
            assert_eq!(*burned_credits, 9);
            assert_eq!(*granted_power_units, 2);
            assert_eq!(*reserve_remaining, 48);
            assert_eq!(*nonce, 1);
        }
        other => panic!("expected PowerRedeemed, got {other:?}"),
    }
}

#[test]
fn reward_asset_redeem_power_rejected_when_reserve_insufficient() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: crate::geometry::GeoPos::new(0.0, 0.0, 0.0),
    });
    world.step().expect("register target agent");
    let initial_electricity = world
        .agent_resource_balance("agent-1", crate::simulator::ResourceKind::Electricity)
        .expect("query target electricity");

    world.set_reward_asset_config(RewardAssetConfig {
        credits_per_power_unit: 1,
        ..RewardAssetConfig::default()
    });
    world.set_protocol_power_reserve(ProtocolPowerReserve {
        epoch_index: 4,
        available_power_units: 1,
        redeemed_power_units: 0,
    });
    world
        .mint_node_power_credits("node-a", 5)
        .expect("mint node credits");

    world.submit_action(Action::RedeemPower {
        node_id: "node-a".to_string(),
        target_agent_id: "agent-1".to_string(),
        redeem_credits: 3,
        nonce: 5,
    });
    world.step().expect("redeem power rejected");

    assert_eq!(world.node_power_credit_balance("node-a"), 5);
    assert_eq!(
        world
            .agent_resource_balance("agent-1", crate::simulator::ResourceKind::Electricity)
            .expect("query target electricity after reject"),
        initial_electricity
    );
    assert_eq!(world.protocol_power_reserve().available_power_units, 1);
    assert_eq!(world.protocol_power_reserve().redeemed_power_units, 0);

    let event = world.journal().events.last().expect("reject event");
    match &event.body {
        WorldEventBody::Domain(DomainEvent::PowerRedeemRejected {
            node_id,
            target_agent_id,
            redeem_credits,
            nonce,
            reason,
        }) => {
            assert_eq!(node_id, "node-a");
            assert_eq!(target_agent_id, "agent-1");
            assert_eq!(*redeem_credits, 3);
            assert_eq!(*nonce, 5);
            assert!(reason.contains("insufficient protocol power reserve"));
        }
        other => panic!("expected PowerRedeemRejected, got {other:?}"),
    }
}

#[test]
fn reward_asset_redeem_power_rejects_below_min_redeem_unit() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: crate::geometry::GeoPos::new(0.0, 0.0, 0.0),
    });
    world.step().expect("register target agent");

    world.set_reward_asset_config(RewardAssetConfig {
        credits_per_power_unit: 4,
        max_redeem_power_per_epoch: 100,
        min_redeem_power_unit: 3,
        ..RewardAssetConfig::default()
    });
    world.set_protocol_power_reserve(ProtocolPowerReserve {
        epoch_index: 10,
        available_power_units: 50,
        redeemed_power_units: 0,
    });
    world
        .mint_node_power_credits("node-a", 12)
        .expect("mint node credits");

    world.submit_action(Action::RedeemPower {
        node_id: "node-a".to_string(),
        target_agent_id: "agent-1".to_string(),
        redeem_credits: 8,
        nonce: 1,
    });
    world.step().expect("redeem should be rejected");

    assert_eq!(world.node_power_credit_balance("node-a"), 12);
    assert_eq!(world.protocol_power_reserve().available_power_units, 50);
    assert_eq!(world.protocol_power_reserve().redeemed_power_units, 0);
    assert_eq!(world.node_last_redeem_nonce("node-a"), None);
    let event = world.journal().events.last().expect("reject event");
    match &event.body {
        WorldEventBody::Domain(DomainEvent::PowerRedeemRejected { reason, .. }) => {
            assert!(reason.contains("granted power below minimum unit"));
        }
        other => panic!("expected PowerRedeemRejected, got {other:?}"),
    }
}

#[test]
fn reward_asset_redeem_power_rejects_epoch_cap_exceeded() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: crate::geometry::GeoPos::new(0.0, 0.0, 0.0),
    });
    world.step().expect("register target agent");

    world.set_reward_asset_config(RewardAssetConfig {
        credits_per_power_unit: 1,
        max_redeem_power_per_epoch: 3,
        min_redeem_power_unit: 1,
        ..RewardAssetConfig::default()
    });
    world.set_protocol_power_reserve(ProtocolPowerReserve {
        epoch_index: 11,
        available_power_units: 50,
        redeemed_power_units: 2,
    });
    world
        .mint_node_power_credits("node-a", 10)
        .expect("mint node credits");

    world.submit_action(Action::RedeemPower {
        node_id: "node-a".to_string(),
        target_agent_id: "agent-1".to_string(),
        redeem_credits: 2,
        nonce: 1,
    });
    world.step().expect("redeem should be rejected by cap");

    assert_eq!(world.node_power_credit_balance("node-a"), 10);
    assert_eq!(world.protocol_power_reserve().available_power_units, 50);
    assert_eq!(world.protocol_power_reserve().redeemed_power_units, 2);
    assert_eq!(world.node_last_redeem_nonce("node-a"), None);
    let event = world.journal().events.last().expect("reject event");
    match &event.body {
        WorldEventBody::Domain(DomainEvent::PowerRedeemRejected { reason, .. }) => {
            assert!(reason.contains("epoch redeem cap exceeded"));
        }
        other => panic!("expected PowerRedeemRejected, got {other:?}"),
    }
}

#[test]
fn reward_asset_redeem_power_rejects_nonce_replay() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: crate::geometry::GeoPos::new(0.0, 0.0, 0.0),
    });
    world.step().expect("register target agent");

    world.set_reward_asset_config(RewardAssetConfig {
        credits_per_power_unit: 1,
        max_redeem_power_per_epoch: 100,
        min_redeem_power_unit: 1,
        ..RewardAssetConfig::default()
    });
    world.set_protocol_power_reserve(ProtocolPowerReserve {
        epoch_index: 12,
        available_power_units: 50,
        redeemed_power_units: 0,
    });
    world
        .mint_node_power_credits("node-a", 10)
        .expect("mint node credits");

    world.submit_action(Action::RedeemPower {
        node_id: "node-a".to_string(),
        target_agent_id: "agent-1".to_string(),
        redeem_credits: 2,
        nonce: 7,
    });
    world.step().expect("first redeem");
    assert_eq!(world.node_last_redeem_nonce("node-a"), Some(7));
    assert_eq!(world.node_power_credit_balance("node-a"), 8);

    world.submit_action(Action::RedeemPower {
        node_id: "node-a".to_string(),
        target_agent_id: "agent-1".to_string(),
        redeem_credits: 1,
        nonce: 7,
    });
    world.step().expect("replay nonce must be rejected");
    assert_eq!(world.node_last_redeem_nonce("node-a"), Some(7));
    assert_eq!(world.node_power_credit_balance("node-a"), 8);
    let replay_event = world.journal().events.last().expect("replay reject event");
    match &replay_event.body {
        WorldEventBody::Domain(DomainEvent::PowerRedeemRejected { reason, .. }) => {
            assert!(reason.contains("nonce replay detected"));
        }
        other => panic!("expected PowerRedeemRejected, got {other:?}"),
    }

    world.submit_action(Action::RedeemPower {
        node_id: "node-a".to_string(),
        target_agent_id: "agent-1".to_string(),
        redeem_credits: 1,
        nonce: 6,
    });
    world.step().expect("older nonce must be rejected");
    assert_eq!(world.node_last_redeem_nonce("node-a"), Some(7));
    assert_eq!(world.node_power_credit_balance("node-a"), 8);
}

#[test]
fn reward_asset_snapshot_roundtrip_persists_redeem_nonce() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: crate::geometry::GeoPos::new(0.0, 0.0, 0.0),
    });
    world.step().expect("register target agent");
    world.set_reward_asset_config(RewardAssetConfig {
        credits_per_power_unit: 1,
        max_redeem_power_per_epoch: 100,
        min_redeem_power_unit: 1,
        ..RewardAssetConfig::default()
    });
    world.set_protocol_power_reserve(ProtocolPowerReserve {
        epoch_index: 13,
        available_power_units: 30,
        redeemed_power_units: 0,
    });
    world
        .mint_node_power_credits("node-a", 5)
        .expect("mint node credits");
    world.submit_action(Action::RedeemPower {
        node_id: "node-a".to_string(),
        target_agent_id: "agent-1".to_string(),
        redeem_credits: 2,
        nonce: 3,
    });
    world.step().expect("redeem");

    let snapshot = world.snapshot();
    let restored = World::from_snapshot(snapshot, world.journal().clone()).expect("restore");
    assert_eq!(restored.node_last_redeem_nonce("node-a"), Some(3));
}
