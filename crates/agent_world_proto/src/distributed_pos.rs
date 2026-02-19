use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PosDecisionStatus {
    Pending,
    Committed,
    Rejected,
}

pub fn required_supermajority_stake(
    total_stake: u64,
    numerator: u64,
    denominator: u64,
) -> Result<u64, String> {
    if total_stake == 0 {
        return Err("total stake must be positive".to_string());
    }
    if denominator == 0 || numerator == 0 || numerator > denominator {
        return Err(format!(
            "invalid supermajority ratio {}/{}",
            numerator, denominator
        ));
    }
    if numerator.saturating_mul(2) <= denominator {
        return Err("supermajority ratio must be greater than 1/2".to_string());
    }

    let multiplied = u128::from(total_stake)
        .checked_mul(u128::from(numerator))
        .ok_or_else(|| "required stake overflow".to_string())?;
    let denominator = u128::from(denominator);
    let mut required = multiplied / denominator;
    if multiplied % denominator != 0 {
        required += 1;
    }
    let required = u64::try_from(required).map_err(|_| "required stake overflow".to_string())?;
    if required == 0 || required > total_stake {
        return Err(format!(
            "invalid required stake {} for total stake {}",
            required, total_stake
        ));
    }
    Ok(required)
}

pub fn decide_pos_status(
    total_stake: u64,
    required_stake: u64,
    approved_stake: u64,
    rejected_stake: u64,
) -> PosDecisionStatus {
    if approved_stake >= required_stake {
        return PosDecisionStatus::Committed;
    }
    if total_stake.saturating_sub(rejected_stake) < required_stake {
        PosDecisionStatus::Rejected
    } else {
        PosDecisionStatus::Pending
    }
}

pub fn weighted_expected_proposer(
    validators: &BTreeMap<String, u64>,
    total_stake: u64,
    slot: u64,
) -> Option<String> {
    if validators.is_empty() || total_stake == 0 {
        return None;
    }
    let slot_seed = slot.to_le_bytes();
    let seed_hash = blake3::hash(&slot_seed);
    let mut seed_bytes = [0u8; 8];
    seed_bytes.copy_from_slice(&seed_hash.as_bytes()[..8]);
    let mut target = u64::from_le_bytes(seed_bytes) % total_stake;
    for (validator_id, stake) in validators {
        if target < *stake {
            return Some(validator_id.clone());
        }
        target = target.saturating_sub(*stake);
    }
    validators.keys().next().cloned()
}

pub fn slot_epoch(epoch_length_slots: u64, slot: u64) -> u64 {
    if epoch_length_slots == 0 {
        return 0;
    }
    slot / epoch_length_slots
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn required_supermajority_rounds_up() {
        let required = required_supermajority_stake(100, 2, 3).expect("required");
        assert_eq!(required, 67);
    }

    #[test]
    fn decide_pos_status_transitions() {
        assert_eq!(
            decide_pos_status(100, 67, 67, 0),
            PosDecisionStatus::Committed
        );
        assert_eq!(
            decide_pos_status(100, 67, 20, 30),
            PosDecisionStatus::Pending
        );
        assert_eq!(
            decide_pos_status(100, 67, 20, 34),
            PosDecisionStatus::Rejected
        );
    }

    #[test]
    fn weighted_expected_proposer_is_deterministic() {
        let mut validators = BTreeMap::new();
        validators.insert("a".to_string(), 34);
        validators.insert("b".to_string(), 33);
        validators.insert("c".to_string(), 33);
        let first = weighted_expected_proposer(&validators, 100, 42).expect("first");
        let second = weighted_expected_proposer(&validators, 100, 42).expect("second");
        assert_eq!(first, second);
    }
}
