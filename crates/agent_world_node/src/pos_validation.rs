use std::collections::BTreeMap;

use crate::{NodeError, NodePosConfig, PosConsensusStatus};

pub(crate) fn validate_pos_config(pos_config: &NodePosConfig) -> Result<(), NodeError> {
    let _ = validated_pos_state(pos_config)?;
    Ok(())
}

pub(crate) fn validated_pos_state(
    pos_config: &NodePosConfig,
) -> Result<(BTreeMap<String, u64>, BTreeMap<String, String>, u64, u64), NodeError> {
    if pos_config.validators.is_empty() {
        return Err(NodeError::InvalidConfig {
            reason: "pos validators cannot be empty".to_string(),
        });
    }
    if pos_config.epoch_length_slots == 0 {
        return Err(NodeError::InvalidConfig {
            reason: "epoch_length_slots must be positive".to_string(),
        });
    }
    if pos_config.supermajority_denominator == 0
        || pos_config.supermajority_numerator == 0
        || pos_config.supermajority_numerator > pos_config.supermajority_denominator
    {
        return Err(NodeError::InvalidConfig {
            reason: format!(
                "invalid supermajority ratio {}/{}",
                pos_config.supermajority_numerator, pos_config.supermajority_denominator
            ),
        });
    }
    if pos_config.supermajority_numerator.saturating_mul(2) <= pos_config.supermajority_denominator
    {
        return Err(NodeError::InvalidConfig {
            reason: "supermajority ratio must be greater than 1/2".to_string(),
        });
    }

    let mut validators = BTreeMap::new();
    let mut validator_players = BTreeMap::new();
    let mut player_to_validator = BTreeMap::new();
    let mut total_stake = 0u64;
    for validator in &pos_config.validators {
        let validator_id = validator.validator_id.trim();
        if validator_id.is_empty() {
            return Err(NodeError::InvalidConfig {
                reason: "validator_id cannot be empty".to_string(),
            });
        }
        if validator.stake == 0 {
            return Err(NodeError::InvalidConfig {
                reason: format!("validator {} stake must be positive", validator_id),
            });
        }
        if validators
            .insert(validator_id.to_string(), validator.stake)
            .is_some()
        {
            return Err(NodeError::InvalidConfig {
                reason: format!("duplicate validator: {}", validator_id),
            });
        }
        let player_id = pos_config
            .validator_player_ids
            .get(validator_id)
            .ok_or_else(|| NodeError::InvalidConfig {
                reason: format!("missing player binding for validator {}", validator_id),
            })?;
        let player_id = player_id.trim();
        if player_id.is_empty() {
            return Err(NodeError::InvalidConfig {
                reason: format!("validator {} has empty player_id binding", validator_id),
            });
        }
        if let Some(existing_validator) =
            player_to_validator.insert(player_id.to_string(), validator_id.to_string())
        {
            return Err(NodeError::InvalidConfig {
                reason: format!(
                    "player {} is bound to multiple validators: {} and {}",
                    player_id, existing_validator, validator_id
                ),
            });
        }
        validator_players.insert(validator_id.to_string(), player_id.to_string());
        total_stake =
            total_stake
                .checked_add(validator.stake)
                .ok_or_else(|| NodeError::InvalidConfig {
                    reason: "total stake overflow".to_string(),
                })?;
    }
    for validator_id in pos_config.validator_player_ids.keys() {
        if !validators.contains_key(validator_id.as_str()) {
            return Err(NodeError::InvalidConfig {
                reason: format!(
                    "validator player binding references unknown validator: {}",
                    validator_id
                ),
            });
        }
    }

    let required_stake = required_supermajority_stake(
        total_stake,
        pos_config.supermajority_numerator,
        pos_config.supermajority_denominator,
    )?;
    Ok((validators, validator_players, total_stake, required_stake))
}

pub(crate) fn decide_status(
    total_stake: u64,
    required_stake: u64,
    approved_stake: u64,
    rejected_stake: u64,
) -> PosConsensusStatus {
    if approved_stake >= required_stake {
        return PosConsensusStatus::Committed;
    }
    if total_stake.saturating_sub(rejected_stake) < required_stake {
        PosConsensusStatus::Rejected
    } else {
        PosConsensusStatus::Pending
    }
}

fn required_supermajority_stake(
    total_stake: u64,
    numerator: u64,
    denominator: u64,
) -> Result<u64, NodeError> {
    let multiplied = u128::from(total_stake)
        .checked_mul(u128::from(numerator))
        .ok_or_else(|| NodeError::InvalidConfig {
            reason: "required stake overflow".to_string(),
        })?;
    let denominator = u128::from(denominator);
    let mut required = multiplied / denominator;
    if multiplied % denominator != 0 {
        required += 1;
    }
    let required = u64::try_from(required).map_err(|_| NodeError::InvalidConfig {
        reason: "required stake overflow".to_string(),
    })?;
    if required == 0 || required > total_stake {
        return Err(NodeError::InvalidConfig {
            reason: format!(
                "invalid required stake {} for total stake {}",
                required, total_stake
            ),
        });
    }
    Ok(required)
}
