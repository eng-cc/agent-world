use super::super::main_token::{
    MAIN_TOKEN_TREASURY_BUCKET_ECOSYSTEM_POOL, MAIN_TOKEN_TREASURY_BUCKET_NODE_SERVICE_REWARD,
    MAIN_TOKEN_TREASURY_BUCKET_SECURITY_RESERVE, MAIN_TOKEN_TREASURY_BUCKET_STAKING_REWARD,
};
use super::*;

impl WorldState {
    pub(super) fn apply_domain_event_main_token(
        &mut self,
        event: &DomainEvent,
        now: WorldTime,
    ) -> Result<(), WorldError> {
        match event {
            DomainEvent::MainTokenGenesisInitialized {
                total_supply,
                allocations,
            } => {
                if *total_supply == 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: "main token genesis total_supply must be > 0".to_string(),
                    });
                }
                if allocations.is_empty() {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: "main token genesis allocations cannot be empty".to_string(),
                    });
                }
                if !self.main_token_genesis_buckets.is_empty() {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: "main token genesis already initialized".to_string(),
                    });
                }
                if !self.main_token_balances.is_empty()
                    || !self.main_token_treasury_balances.is_empty()
                    || !self.main_token_claim_nonces.is_empty()
                {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: "main token ledger is not empty during genesis initialization"
                            .to_string(),
                    });
                }
                if self.main_token_supply.total_supply > 0
                    || self.main_token_supply.total_issued > 0
                    || self.main_token_supply.total_burned > 0
                    || self.main_token_supply.circulating_supply > 0
                {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: "main token supply already initialized".to_string(),
                    });
                }

                let mut ratio_sum = 0_u64;
                let mut allocated_sum = 0_u64;
                let mut buckets = BTreeMap::new();
                let mut recipient_vested = BTreeMap::<String, u64>::new();
                for allocation in allocations {
                    if allocation.bucket_id.trim().is_empty() {
                        return Err(WorldError::ResourceBalanceInvalid {
                            reason: "main token allocation bucket_id cannot be empty".to_string(),
                        });
                    }
                    if allocation.recipient.trim().is_empty() {
                        return Err(WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "main token allocation recipient cannot be empty: bucket={}",
                                allocation.bucket_id
                            ),
                        });
                    }
                    if allocation.ratio_bps == 0 {
                        return Err(WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "main token allocation ratio must be > 0: bucket={}",
                                allocation.bucket_id
                            ),
                        });
                    }
                    if allocation.claimed_amount != 0 {
                        return Err(WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "main token allocation claimed_amount must be 0 at genesis: bucket={}",
                                allocation.bucket_id
                            ),
                        });
                    }
                    ratio_sum = ratio_sum.saturating_add(u64::from(allocation.ratio_bps));
                    allocated_sum = allocated_sum.saturating_add(allocation.allocated_amount);
                    if buckets
                        .insert(allocation.bucket_id.clone(), allocation.clone())
                        .is_some()
                    {
                        return Err(WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "duplicate main token allocation bucket_id: {}",
                                allocation.bucket_id
                            ),
                        });
                    }
                    recipient_vested
                        .entry(allocation.recipient.clone())
                        .and_modify(|value| {
                            *value = value.saturating_add(allocation.allocated_amount);
                        })
                        .or_insert(allocation.allocated_amount);
                }
                if ratio_sum != 10_000 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "main token allocation ratio sum must be 10000 bps, got {}",
                            ratio_sum
                        ),
                    });
                }
                if allocated_sum != *total_supply {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "main token allocation sum mismatch: allocated={} total_supply={}",
                            allocated_sum, total_supply
                        ),
                    });
                }

                self.main_token_supply = MainTokenSupplyState {
                    total_supply: *total_supply,
                    circulating_supply: 0,
                    total_issued: 0,
                    total_burned: 0,
                };
                self.main_token_genesis_buckets = buckets;
                for (recipient, vested_amount) in recipient_vested {
                    self.main_token_balances.insert(
                        recipient.clone(),
                        MainTokenAccountBalance {
                            account_id: recipient,
                            liquid_balance: 0,
                            vested_balance: vested_amount,
                        },
                    );
                }
            }
            DomainEvent::MainTokenVestingClaimed {
                bucket_id,
                beneficiary,
                amount,
                nonce,
            } => {
                if *amount == 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: "main token vesting claim amount must be > 0".to_string(),
                    });
                }
                if *nonce == 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: "main token vesting claim nonce must be > 0".to_string(),
                    });
                }
                if let Some(last_nonce) = self.main_token_claim_nonces.get(beneficiary) {
                    if *nonce <= *last_nonce {
                        return Err(WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "main token vesting claim nonce replay: beneficiary={} nonce={} last_nonce={}",
                                beneficiary, nonce, last_nonce
                            ),
                        });
                    }
                }
                let bucket = self
                    .main_token_genesis_buckets
                    .get(bucket_id)
                    .ok_or_else(|| WorldError::ResourceBalanceInvalid {
                        reason: format!("main token genesis bucket not found: {bucket_id}"),
                    })?;
                if bucket.recipient != *beneficiary {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "main token vesting beneficiary mismatch: bucket recipient={} beneficiary={}",
                            bucket.recipient, beneficiary
                        ),
                    });
                }
                let unlocked_amount = main_token_bucket_unlocked_amount(bucket, now);
                let releasable = unlocked_amount.saturating_sub(bucket.claimed_amount);
                if releasable == 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "main token vesting has no releasable amount: bucket={} epoch={}",
                            bucket_id, now
                        ),
                    });
                }
                if *amount != releasable {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "main token vesting claim amount mismatch: expected={} actual={}",
                            releasable, amount
                        ),
                    });
                }

                let account = self
                    .main_token_balances
                    .get_mut(beneficiary)
                    .ok_or_else(|| WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "main token beneficiary account not found: {}",
                            beneficiary
                        ),
                    })?;
                if account.vested_balance < *amount {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "main token vested balance insufficient: beneficiary={} vested={} claim={}",
                            beneficiary, account.vested_balance, amount
                        ),
                    });
                }
                account.vested_balance -= *amount;
                account.liquid_balance =
                    account
                        .liquid_balance
                        .checked_add(*amount)
                        .ok_or_else(|| WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "main token liquid balance overflow: beneficiary={} current={} claim={}",
                                beneficiary, account.liquid_balance, amount
                            ),
                        })?;

                let bucket = self
                    .main_token_genesis_buckets
                    .get_mut(bucket_id)
                    .ok_or_else(|| WorldError::ResourceBalanceInvalid {
                        reason: format!("main token genesis bucket not found: {bucket_id}"),
                    })?;
                bucket.claimed_amount =
                    bucket.claimed_amount.checked_add(*amount).ok_or_else(|| {
                        WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "main token claimed amount overflow: bucket={} current={} claim={}",
                                bucket_id, bucket.claimed_amount, amount
                            ),
                        }
                    })?;
                if bucket.claimed_amount > bucket.allocated_amount {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "main token claimed exceeds allocation: bucket={} claimed={} allocated={}",
                            bucket_id, bucket.claimed_amount, bucket.allocated_amount
                        ),
                    });
                }

                self.main_token_supply.circulating_supply = self
                    .main_token_supply
                    .circulating_supply
                    .checked_add(*amount)
                    .ok_or_else(|| WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "main token circulating supply overflow: current={} delta={}",
                            self.main_token_supply.circulating_supply, amount
                        ),
                    })?;
                if self.main_token_supply.circulating_supply > self.main_token_supply.total_supply {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "main token circulating exceeds total supply: circulating={} total={}",
                            self.main_token_supply.circulating_supply,
                            self.main_token_supply.total_supply
                        ),
                    });
                }
                self.main_token_claim_nonces
                    .insert(beneficiary.clone(), *nonce);
            }
            DomainEvent::MainTokenEpochIssued {
                epoch_index,
                inflation_rate_bps,
                issued_amount,
                staking_reward_amount,
                node_service_reward_amount,
                ecosystem_pool_amount,
                security_reserve_amount,
            } => {
                if self
                    .main_token_epoch_issuance_records
                    .contains_key(epoch_index)
                {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "main token epoch issuance already exists: epoch={epoch_index}"
                        ),
                    });
                }
                let split_sum = staking_reward_amount
                    .checked_add(*node_service_reward_amount)
                    .and_then(|value| value.checked_add(*ecosystem_pool_amount))
                    .and_then(|value| value.checked_add(*security_reserve_amount))
                    .ok_or_else(|| WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "main token epoch split overflow: epoch={} staking={} node_service={} ecosystem={} security={}",
                            epoch_index,
                            staking_reward_amount,
                            node_service_reward_amount,
                            ecosystem_pool_amount,
                            security_reserve_amount
                        ),
                    })?;
                if split_sum != *issued_amount {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "main token epoch split mismatch: epoch={} issued={} split_sum={}",
                            epoch_index, issued_amount, split_sum
                        ),
                    });
                }

                let next_total_issued = self
                    .main_token_supply
                    .total_issued
                    .checked_add(*issued_amount)
                    .ok_or_else(|| WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "main token total_issued overflow: current={} issued={}",
                            self.main_token_supply.total_issued, issued_amount
                        ),
                    })?;
                let next_total_supply = self
                    .main_token_supply
                    .total_supply
                    .checked_add(*issued_amount)
                    .ok_or_else(|| WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "main token total_supply overflow: current={} issued={}",
                            self.main_token_supply.total_supply, issued_amount
                        ),
                    })?;
                if let Some(max_supply) = self.main_token_config.max_supply {
                    if next_total_supply > max_supply {
                        return Err(WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "main token total_supply exceeds max_supply: next={} max={}",
                                next_total_supply, max_supply
                            ),
                        });
                    }
                }

                add_main_token_treasury_balance(
                    &mut self.main_token_treasury_balances,
                    MAIN_TOKEN_TREASURY_BUCKET_STAKING_REWARD,
                    *staking_reward_amount,
                )?;
                add_main_token_treasury_balance(
                    &mut self.main_token_treasury_balances,
                    MAIN_TOKEN_TREASURY_BUCKET_NODE_SERVICE_REWARD,
                    *node_service_reward_amount,
                )?;
                add_main_token_treasury_balance(
                    &mut self.main_token_treasury_balances,
                    MAIN_TOKEN_TREASURY_BUCKET_ECOSYSTEM_POOL,
                    *ecosystem_pool_amount,
                )?;
                add_main_token_treasury_balance(
                    &mut self.main_token_treasury_balances,
                    MAIN_TOKEN_TREASURY_BUCKET_SECURITY_RESERVE,
                    *security_reserve_amount,
                )?;

                self.main_token_supply.total_issued = next_total_issued;
                self.main_token_supply.total_supply = next_total_supply;
                self.main_token_epoch_issuance_records.insert(
                    *epoch_index,
                    MainTokenEpochIssuanceRecord {
                        epoch_index: *epoch_index,
                        inflation_rate_bps: *inflation_rate_bps,
                        issued_amount: *issued_amount,
                        staking_reward_amount: *staking_reward_amount,
                        node_service_reward_amount: *node_service_reward_amount,
                        ecosystem_pool_amount: *ecosystem_pool_amount,
                        security_reserve_amount: *security_reserve_amount,
                    },
                );
            }
            _ => unreachable!("apply_domain_event_main_token received unsupported event variant"),
        }
        Ok(())
    }
}

fn add_main_token_treasury_balance(
    balances: &mut BTreeMap<String, u64>,
    bucket_id: &str,
    amount: u64,
) -> Result<(), WorldError> {
    let next = balances
        .get(bucket_id)
        .copied()
        .unwrap_or(0)
        .checked_add(amount)
        .ok_or_else(|| WorldError::ResourceBalanceInvalid {
            reason: format!(
                "main token treasury balance overflow: bucket={} amount={}",
                bucket_id, amount
            ),
        })?;
    balances.insert(bucket_id.to_string(), next);
    Ok(())
}
