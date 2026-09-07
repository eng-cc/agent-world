use super::*;

impl WorldState {
    pub(super) fn apply_domain_event_core_late(
        &mut self,
        event: &DomainEvent,
        now: WorldTime,
    ) -> Result<(), WorldError> {
        match event {
            DomainEvent::ModuleArtifactListed { .. }
            | DomainEvent::ModuleArtifactDelisted { .. }
            | DomainEvent::ModuleArtifactDestroyed { .. }
            | DomainEvent::ModuleArtifactBidPlaced { .. }
            | DomainEvent::ModuleArtifactBidCancelled { .. }
            | DomainEvent::ModuleArtifactSaleCompleted { .. } => {
                self.prepare_module_marketplace_event(event, now)?
                    .install_infallible(self);
            }
            event @ (DomainEvent::ResourceTransferred { .. }
            | DomainEvent::DataCollected { .. }
            | DomainEvent::DataCollectedAuthenticated { .. }
            | DomainEvent::DataAccessGranted { .. }
            | DomainEvent::DataAccessRevoked { .. }) => {
                crate::runtime::world::economy_data_publication::PreparedEconomyDataEvent::prepare(
                    self, event, now,
                )?
                .install_infallible(self);
            }
            DomainEvent::PowerRedeemed {
                node_id,
                target_agent_id,
                burned_credits,
                granted_power_units,
                reserve_remaining,
                nonce,
                ..
            } => {
                if *burned_credits == 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: "burned_credits must be > 0".to_string(),
                    });
                }
                if *granted_power_units <= 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "granted_power_units must be > 0, got {}",
                            granted_power_units
                        ),
                    });
                }
                let min_redeem_power_unit = self.reward_asset_config.min_redeem_power_unit;
                if min_redeem_power_unit <= 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: "min_redeem_power_unit must be positive".to_string(),
                    });
                }
                if *granted_power_units < min_redeem_power_unit {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "granted_power_units below minimum: granted={} min={}",
                            granted_power_units, min_redeem_power_unit
                        ),
                    });
                }
                if *nonce == 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: "nonce must be > 0".to_string(),
                    });
                }
                if let Some(last_nonce) = self.node_redeem_nonces.get(node_id) {
                    if *nonce <= *last_nonce {
                        return Err(WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "nonce replay detected: node_id={} nonce={} last_nonce={}",
                                node_id, nonce, last_nonce
                            ),
                        });
                    }
                }
                let (next_power_credit_balance, next_total_burned_credits) = {
                    let node_balance = self.node_asset_balances.get(node_id).ok_or_else(|| {
                        WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "power redeem burn failed: node balance not found: {node_id}"
                            ),
                        }
                    })?;
                    if node_balance.power_credit_balance < *burned_credits {
                        return Err(WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "power redeem burn failed: insufficient power credits: balance={} burn={}",
                                node_balance.power_credit_balance, burned_credits
                            ),
                        });
                    }
                    let next_total_burned_credits = node_balance
                        .total_burned_credits
                        .checked_add(*burned_credits)
                        .ok_or_else(|| WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "power redeem burn failed: total_burned_credits overflow: current={} burn={}",
                                node_balance.total_burned_credits, burned_credits
                            ),
                        })?;
                    (
                        node_balance.power_credit_balance - *burned_credits,
                        next_total_burned_credits,
                    )
                };
                if self.protocol_power_reserve.available_power_units < *granted_power_units {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "insufficient protocol power reserve: available={} requested={}",
                            self.protocol_power_reserve.available_power_units, granted_power_units
                        ),
                    });
                }
                let next_reserve =
                    self.protocol_power_reserve.available_power_units - *granted_power_units;
                if next_reserve != *reserve_remaining {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "reserve remaining mismatch: computed={} event={}",
                            next_reserve, reserve_remaining
                        ),
                    });
                }
                let max_redeem_power_per_epoch =
                    self.reward_asset_config.max_redeem_power_per_epoch;
                if max_redeem_power_per_epoch <= 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: "max_redeem_power_per_epoch must be positive".to_string(),
                    });
                }
                let next_redeemed = self
                    .protocol_power_reserve
                    .redeemed_power_units
                    .checked_add(*granted_power_units)
                    .ok_or_else(|| WorldError::ResourceBalanceInvalid {
                        reason: "redeemed_power_units overflow".to_string(),
                    })?;
                if next_redeemed > max_redeem_power_per_epoch {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "epoch redeem cap exceeded: next={} cap={}",
                            next_redeemed, max_redeem_power_per_epoch
                        ),
                    });
                }
                let next_target_electricity = {
                    let target = self.agents.get(target_agent_id).ok_or_else(|| {
                        WorldError::AgentNotFound {
                            agent_id: target_agent_id.clone(),
                        }
                    })?;
                    let current = target.state.resources.get(ResourceKind::Electricity);
                    current.checked_add(*granted_power_units).ok_or_else(|| {
                        WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "power redeem add electricity failed: overflow current={current} delta={}",
                                granted_power_units
                            ),
                        }
                    })?
                };

                {
                    let node_balance =
                        self.node_asset_balances.get_mut(node_id).ok_or_else(|| {
                            WorldError::ResourceBalanceInvalid {
                                reason: format!(
                                    "power redeem burn failed: node balance not found: {node_id}"
                                ),
                            }
                        })?;
                    node_balance.power_credit_balance = next_power_credit_balance;
                    node_balance.total_burned_credits = next_total_burned_credits;
                }
                self.protocol_power_reserve.available_power_units = next_reserve;
                self.protocol_power_reserve.redeemed_power_units = next_redeemed;
                self.node_redeem_nonces.insert(node_id.clone(), *nonce);

                let target = self.agents.get_mut(target_agent_id).ok_or_else(|| {
                    WorldError::AgentNotFound {
                        agent_id: target_agent_id.clone(),
                    }
                })?;
                if next_target_electricity == 0 {
                    target
                        .state
                        .resources
                        .amounts
                        .remove(&ResourceKind::Electricity);
                } else {
                    target
                        .state
                        .resources
                        .amounts
                        .insert(ResourceKind::Electricity, next_target_electricity);
                }
                target.last_active = now;
                if let Some(cell) = self.agents.get_mut(node_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::PowerRedeemRejected {
                node_id,
                target_agent_id,
                ..
            } => {
                if let Some(cell) = self.agents.get_mut(node_id) {
                    cell.last_active = now;
                }
                if let Some(cell) = self.agents.get_mut(target_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::NodePointsSettlementApplied {
                report,
                signer_node_id,
                settlement_hash,
                minted_records,
                main_token_bridge_total_amount,
                main_token_bridge_distributions,
            } => {
                apply_node_points_settlement_event(
                    self,
                    report,
                    signer_node_id.as_str(),
                    settlement_hash.as_str(),
                    minted_records.as_slice(),
                    *main_token_bridge_total_amount,
                    main_token_bridge_distributions.as_slice(),
                )?;
            }
            event @ DomainEvent::MainTokenGenesisInitialized { .. }
            | event @ DomainEvent::MainTokenVestingClaimed { .. }
            | event @ DomainEvent::MainTokenTransferred { .. }
            | event @ DomainEvent::MainTokenEpochIssued { .. }
            | event @ DomainEvent::MainTokenFeeSettled { .. }
            | event @ DomainEvent::MainTokenPolicyUpdateScheduled { .. }
            | event @ DomainEvent::MainTokenTreasuryDistributed { .. }
            | event @ DomainEvent::RestrictedStarterClaimLiveopsPoolToppedUp { .. }
            | event @ DomainEvent::RestrictedStarterClaimGrantIssued { .. }
            | event @ DomainEvent::RestrictedStarterClaimGrantExpired { .. }
            | event @ DomainEvent::RestrictedStarterClaimGrantRevoked { .. } => {
                self.apply_domain_event_main_token(event, now)?;
            }
            event @ DomainEvent::LogisticsRouteRegistered { .. }
            | event @ DomainEvent::LogisticsRouteAvailabilityChanged { .. }
            | event @ DomainEvent::LogisticsPathRerouted { .. }
            | event @ DomainEvent::MaterialTransferred { .. }
            | event @ DomainEvent::MaterialTransitStarted { .. }
            | event @ DomainEvent::MaterialTransitCompleted { .. }
            | event @ DomainEvent::FactoryBuildStarted { .. }
            | event @ DomainEvent::FactoryBuilt { .. }
            | event @ DomainEvent::FactoryDurabilityChanged { .. }
            | event @ DomainEvent::FactoryMaintained { .. }
            | event @ DomainEvent::FactoryRecycled { .. }
            | event @ DomainEvent::RecipeStarted { .. }
            | event @ DomainEvent::RecipeCompleted { .. }
            | event @ DomainEvent::FactoryProductionBlocked { .. }
            | event @ DomainEvent::FactoryProductionResumed { .. }
            | event @ DomainEvent::FactoryProductionPaused { .. } => {
                self.apply_domain_event_industry(event, now)?;
            }
            DomainEvent::MaterialProfileGoverned {
                operator_agent_id,
                proposal_id,
                profile,
            } => {
                if !self.agents.contains_key(operator_agent_id) {
                    return Err(WorldError::AgentNotFound {
                        agent_id: operator_agent_id.clone(),
                    });
                }
                if *proposal_id == 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: "material profile governed proposal_id must be > 0".to_string(),
                    });
                }
                if profile.kind.trim().is_empty() {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: "material profile kind cannot be empty".to_string(),
                    });
                }
                if profile.tier == 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!("material profile tier must be >= 1: {}", profile.kind),
                    });
                }
                if profile.category.trim().is_empty() {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "material profile category cannot be empty: {}",
                            profile.kind
                        ),
                    });
                }
                if profile.stack_limit <= 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "material profile stack_limit must be > 0: {}",
                            profile.kind
                        ),
                    });
                }
                self.material_profiles
                    .insert(profile.kind.clone(), profile.clone());
                if let Some(cell) = self.agents.get_mut(operator_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::ProductProfileGoverned { .. }
            | DomainEvent::RecipeProfileGoverned { .. }
            | DomainEvent::FactoryProfileGoverned { .. } => {
                self.prepare_module_release_event(event, now)?
                    .install_infallible(self);
            }
            _ => unreachable!("apply_domain_event_core_late received unsupported event"),
        }
        Ok(())
    }
}
