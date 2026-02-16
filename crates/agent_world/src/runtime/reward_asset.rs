use serde::{Deserialize, Serialize};

const DEFAULT_POINTS_PER_CREDIT: u64 = 10;
const DEFAULT_CREDITS_PER_POWER_UNIT: u64 = 1;
const DEFAULT_MAX_REDEEM_POWER_PER_EPOCH: i64 = 10_000;
const DEFAULT_MIN_REDEEM_POWER_UNIT: i64 = 1;

/// Redeemable reward asset configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewardAssetConfig {
    pub points_per_credit: u64,
    pub credits_per_power_unit: u64,
    pub max_redeem_power_per_epoch: i64,
    pub min_redeem_power_unit: i64,
}

impl Default for RewardAssetConfig {
    fn default() -> Self {
        Self {
            points_per_credit: DEFAULT_POINTS_PER_CREDIT,
            credits_per_power_unit: DEFAULT_CREDITS_PER_POWER_UNIT,
            max_redeem_power_per_epoch: DEFAULT_MAX_REDEEM_POWER_PER_EPOCH,
            min_redeem_power_unit: DEFAULT_MIN_REDEEM_POWER_UNIT,
        }
    }
}

/// Node-owned reward asset balance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct NodeAssetBalance {
    pub node_id: String,
    pub power_credit_balance: u64,
    pub total_minted_credits: u64,
    pub total_burned_credits: u64,
}

/// Protocol reserve for power redemptions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProtocolPowerReserve {
    pub epoch_index: u64,
    pub available_power_units: i64,
    pub redeemed_power_units: i64,
}

/// On-chain mint record derived from node points settlement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct NodeRewardMintRecord {
    pub epoch_index: u64,
    pub node_id: String,
    pub source_awarded_points: u64,
    pub minted_power_credits: u64,
    pub settlement_hash: String,
    pub signer_node_id: String,
    pub signature: String,
}
