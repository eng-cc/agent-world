use super::super::*;

#[test]
fn main_token_queries_return_defaults_when_uninitialized() {
    let world = World::new();

    let config = world.main_token_config();
    assert_eq!(config.symbol, "AWT");
    assert_eq!(config.decimals, 9);
    assert_eq!(config.initial_supply, 0);
    assert_eq!(world.main_token_liquid_balance("missing-account"), 0);
    assert_eq!(world.main_token_treasury_balance("missing-bucket"), 0);
    assert!(world.main_token_genesis_bucket("missing-bucket").is_none());
    assert!(world.main_token_epoch_issuance_record(1).is_none());
}

#[test]
fn main_token_snapshot_roundtrip_persists_state() {
    let mut world = World::new();
    world.set_main_token_config(MainTokenConfig {
        symbol: "AWT".to_string(),
        decimals: 9,
        initial_supply: 1_000_000_000,
        max_supply: Some(5_000_000_000),
        inflation_policy: MainTokenInflationPolicy {
            base_rate_bps: 410,
            ..MainTokenInflationPolicy::default()
        },
        issuance_split: MainTokenIssuanceSplitPolicy {
            node_service_reward_bps: 2_200,
            ..MainTokenIssuanceSplitPolicy::default()
        },
        burn_policy: MainTokenBurnPolicy {
            gas_base_fee_burn_bps: 3_200,
            ..MainTokenBurnPolicy::default()
        },
    });
    world.set_main_token_supply(MainTokenSupplyState {
        total_supply: 1_050_000_000,
        circulating_supply: 820_000_000,
        total_issued: 100_000_000,
        total_burned: 50_000_000,
    });
    world
        .set_main_token_account_balance("protocol:treasury", 320_000_000, 0)
        .expect("set treasury account balance");
    world
        .set_main_token_account_balance("player:alice", 1_250_000, 350_000)
        .expect("set alice account balance");
    world
        .set_main_token_treasury_balance("ecosystem_pool", 120_000_000)
        .expect("set ecosystem pool");
    world
        .set_main_token_genesis_bucket(MainTokenGenesisAllocationBucketState {
            bucket_id: "ecosystem_growth_pool".to_string(),
            ratio_bps: 2_500,
            recipient: "protocol:treasury".to_string(),
            cliff_epochs: 30,
            linear_unlock_epochs: 360,
            start_epoch: 1,
            allocated_amount: 250_000_000,
            claimed_amount: 20_000_000,
        })
        .expect("set genesis bucket");
    world
        .record_main_token_epoch_issuance(MainTokenEpochIssuanceRecord {
            epoch_index: 12,
            inflation_rate_bps: 405,
            issued_amount: 1_337_000,
            staking_reward_amount: 802_200,
            node_service_reward_amount: 267_400,
            ecosystem_pool_amount: 200_550,
            security_reserve_amount: 66_850,
        })
        .expect("record issuance");

    let snapshot = world.snapshot();
    let restored = World::from_snapshot(snapshot, world.journal().clone()).expect("restore");

    assert_eq!(restored.main_token_config().initial_supply, 1_000_000_000);
    assert_eq!(restored.main_token_supply().total_supply, 1_050_000_000);
    assert_eq!(
        restored.main_token_liquid_balance("protocol:treasury"),
        320_000_000
    );
    assert_eq!(
        restored.main_token_liquid_balance("player:alice"),
        1_250_000
    );
    assert_eq!(
        restored.main_token_account_balance("player:alice"),
        Some(&MainTokenAccountBalance {
            account_id: "player:alice".to_string(),
            liquid_balance: 1_250_000,
            vested_balance: 350_000,
        })
    );
    assert_eq!(
        restored.main_token_treasury_balance("ecosystem_pool"),
        120_000_000
    );
    assert_eq!(
        restored
            .main_token_genesis_bucket("ecosystem_growth_pool")
            .expect("genesis bucket"),
        &MainTokenGenesisAllocationBucketState {
            bucket_id: "ecosystem_growth_pool".to_string(),
            ratio_bps: 2_500,
            recipient: "protocol:treasury".to_string(),
            cliff_epochs: 30,
            linear_unlock_epochs: 360,
            start_epoch: 1,
            allocated_amount: 250_000_000,
            claimed_amount: 20_000_000,
        }
    );
    assert_eq!(
        restored.main_token_epoch_issuance_record(12),
        Some(&MainTokenEpochIssuanceRecord {
            epoch_index: 12,
            inflation_rate_bps: 405,
            issued_amount: 1_337_000,
            staking_reward_amount: 802_200,
            node_service_reward_amount: 267_400,
            ecosystem_pool_amount: 200_550,
            security_reserve_amount: 66_850,
        })
    );
}
