use serde::{Deserialize, Serialize};

pub const AGENT_CLAIM_BASE_ACTIVATION_FEE_AMOUNT: u64 = 100;
pub const AGENT_CLAIM_BASE_BOND_AMOUNT: u64 = 200;
pub const AGENT_CLAIM_BASE_UPKEEP_PER_EPOCH: u64 = 25;
pub const AGENT_CLAIM_ACTIVATION_FEE_BURN_BPS: u16 = 5_000;
pub const AGENT_CLAIM_RELEASE_COOLDOWN_EPOCHS: u64 = 3;
pub const AGENT_CLAIM_GRACE_EPOCHS: u64 = 2;
pub const AGENT_CLAIM_IDLE_WARNING_EPOCHS: u64 = 7;
pub const AGENT_CLAIM_IDLE_FORCED_RECLAIM_EPOCHS: u64 = 10;
pub const AGENT_CLAIM_FORCED_RECLAIM_PENALTY_BPS: u16 = 2_000;
pub const AGENT_CLAIM_TIER1_MIN_REPUTATION_SCORE: i64 = 10;
pub const AGENT_CLAIM_TIER2_MIN_REPUTATION_SCORE: i64 = 25;

const SLOT_MULTIPLIER_DENOMINATOR: u64 = 10;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentClaimCostQuote {
    pub reputation_tier: u8,
    pub claim_cap: u8,
    pub slot_index: u8,
    pub activation_fee_amount: u64,
    pub activation_fee_burn_amount: u64,
    pub activation_fee_treasury_amount: u64,
    pub claim_bond_amount: u64,
    pub upkeep_per_epoch: u64,
    pub release_cooldown_epochs: u64,
    pub grace_epochs: u64,
    pub idle_warning_epochs: u64,
    pub forced_idle_reclaim_epochs: u64,
    pub forced_reclaim_penalty_bps: u16,
}

pub fn agent_claim_reputation_tier(reputation_score: i64) -> u8 {
    if reputation_score >= AGENT_CLAIM_TIER2_MIN_REPUTATION_SCORE {
        2
    } else if reputation_score >= AGENT_CLAIM_TIER1_MIN_REPUTATION_SCORE {
        1
    } else {
        0
    }
}

pub fn agent_claim_cap_for_tier(reputation_tier: u8) -> u8 {
    match reputation_tier {
        0 => 1,
        1 => 2,
        _ => 3,
    }
}

pub fn agent_claim_quote(
    reputation_score: i64,
    owned_claim_count: usize,
) -> Result<AgentClaimCostQuote, String> {
    let reputation_tier = agent_claim_reputation_tier(reputation_score);
    let claim_cap = agent_claim_cap_for_tier(reputation_tier);
    let next_slot_index = owned_claim_count
        .checked_add(1)
        .ok_or_else(|| "agent claim slot overflow".to_string())?;
    if next_slot_index > usize::from(claim_cap) {
        return Err(format!(
            "agent claim cap exceeded: owned={} cap={}",
            owned_claim_count, claim_cap
        ));
    }

    let slot_index =
        u8::try_from(next_slot_index).map_err(|_| "agent claim slot exceeds u8".to_string())?;
    let multiplier_numerator = slot_multiplier_numerator(slot_index)?;
    let activation_fee_amount =
        scale_amount_ceil(AGENT_CLAIM_BASE_ACTIVATION_FEE_AMOUNT, multiplier_numerator)?;
    let claim_bond_amount = scale_amount_ceil(AGENT_CLAIM_BASE_BOND_AMOUNT, multiplier_numerator)?;
    let upkeep_per_epoch =
        scale_amount_ceil(AGENT_CLAIM_BASE_UPKEEP_PER_EPOCH, multiplier_numerator)?;
    let activation_fee_burn_amount = activation_fee_amount
        .saturating_mul(u64::from(AGENT_CLAIM_ACTIVATION_FEE_BURN_BPS))
        / 10_000;
    let activation_fee_treasury_amount = activation_fee_amount
        .checked_sub(activation_fee_burn_amount)
        .ok_or_else(|| "agent claim activation fee split underflow".to_string())?;

    Ok(AgentClaimCostQuote {
        reputation_tier,
        claim_cap,
        slot_index,
        activation_fee_amount,
        activation_fee_burn_amount,
        activation_fee_treasury_amount,
        claim_bond_amount,
        upkeep_per_epoch,
        release_cooldown_epochs: AGENT_CLAIM_RELEASE_COOLDOWN_EPOCHS,
        grace_epochs: AGENT_CLAIM_GRACE_EPOCHS,
        idle_warning_epochs: AGENT_CLAIM_IDLE_WARNING_EPOCHS,
        forced_idle_reclaim_epochs: AGENT_CLAIM_IDLE_FORCED_RECLAIM_EPOCHS,
        forced_reclaim_penalty_bps: AGENT_CLAIM_FORCED_RECLAIM_PENALTY_BPS,
    })
}

fn slot_multiplier_numerator(slot_index: u8) -> Result<u64, String> {
    match slot_index {
        1 => Ok(10),
        2 => Ok(15),
        3 => Ok(20),
        _ => Err(format!("unsupported agent claim slot index: {slot_index}")),
    }
}

fn scale_amount_ceil(base: u64, multiplier_numerator: u64) -> Result<u64, String> {
    let scaled = base
        .checked_mul(multiplier_numerator)
        .ok_or_else(|| format!("agent claim amount overflow: base={base}"))?;
    scaled
        .checked_add(SLOT_MULTIPLIER_DENOMINATOR - 1)
        .ok_or_else(|| format!("agent claim amount ceil overflow: scaled={scaled}"))
        .map(|value| value / SLOT_MULTIPLIER_DENOMINATOR)
}
