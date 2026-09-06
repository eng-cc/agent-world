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
            DomainEvent::ResourceTransferred {
                from_agent_id,
                to_agent_id,
                kind,
                amount,
            } => {
                if from_agent_id == to_agent_id {
                    let cell = self.agents.get_mut(from_agent_id).ok_or_else(|| {
                        WorldError::AgentNotFound {
                            agent_id: from_agent_id.clone(),
                        }
                    })?;
                    cell.last_active = now;
                } else {
                    let (next_from_resources, next_to_resources) = {
                        let from = self.agents.get(from_agent_id).ok_or_else(|| {
                            WorldError::AgentNotFound {
                                agent_id: from_agent_id.clone(),
                            }
                        })?;
                        let to = self.agents.get(to_agent_id).ok_or_else(|| {
                            WorldError::AgentNotFound {
                                agent_id: to_agent_id.clone(),
                            }
                        })?;

                        let mut next_from = from.state.resources.clone();
                        let mut next_to = to.state.resources.clone();
                        next_from.remove(*kind, *amount).map_err(|err| {
                            WorldError::ResourceBalanceInvalid {
                                reason: format!("transfer remove failed: {err:?}"),
                            }
                        })?;
                        next_to.add(*kind, *amount).map_err(|err| {
                            WorldError::ResourceBalanceInvalid {
                                reason: format!("transfer add failed: {err:?}"),
                            }
                        })?;
                        (next_from, next_to)
                    };

                    let from = self.agents.get_mut(from_agent_id).ok_or_else(|| {
                        WorldError::AgentNotFound {
                            agent_id: from_agent_id.clone(),
                        }
                    })?;
                    from.state.resources = next_from_resources;
                    from.last_active = now;

                    let to = self.agents.get_mut(to_agent_id).ok_or_else(|| {
                        WorldError::AgentNotFound {
                            agent_id: to_agent_id.clone(),
                        }
                    })?;
                    to.state.resources = next_to_resources;
                    to.last_active = now;
                }
            }
            DomainEvent::DataCollected {
                collector_agent_id,
                electricity_cost,
                data_amount,
            } => {
                if *electricity_cost <= 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "data collection electricity_cost must be > 0, got {}",
                            electricity_cost
                        ),
                    });
                }
                if *data_amount <= 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "data collection data_amount must be > 0, got {}",
                            data_amount
                        ),
                    });
                }
                let next_resources = {
                    let collector = self.agents.get(collector_agent_id).ok_or_else(|| {
                        WorldError::AgentNotFound {
                            agent_id: collector_agent_id.clone(),
                        }
                    })?;
                    let mut next = collector.state.resources.clone();
                    next.remove(ResourceKind::Electricity, *electricity_cost)
                        .map_err(|err| WorldError::ResourceBalanceInvalid {
                            reason: format!("data collection electricity debit failed: {err:?}"),
                        })?;
                    next.add(ResourceKind::Data, *data_amount).map_err(|err| {
                        WorldError::ResourceBalanceInvalid {
                            reason: format!("data collection data credit failed: {err:?}"),
                        }
                    })?;
                    next
                };
                let collector = self.agents.get_mut(collector_agent_id).ok_or_else(|| {
                    WorldError::AgentNotFound {
                        agent_id: collector_agent_id.clone(),
                    }
                })?;
                collector.state.resources = next_resources;
                collector.last_active = now;
            }
            DomainEvent::DataCollectedAuthenticated {
                collector_agent_id,
                electricity_cost,
                data_amount,
                player_id,
                public_key,
                nonce,
                signature,
            } => {
                if *electricity_cost <= 0 || *data_amount <= 0 || *nonce == 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason:
                            "authenticated data collection event contains invalid amount or nonce"
                                .to_string(),
                    });
                }
                let last_nonce = self
                    .authenticated_collect_data_last_nonces
                    .get(player_id)
                    .and_then(|by_key| by_key.get(public_key))
                    .copied()
                    .unwrap_or(0);
                if *nonce <= last_nonce {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "authenticated data collection nonce must advance: last={last_nonce} next={nonce}"
                        ),
                    });
                }
                crate::collect_data_auth::verify_authorization(
                    crate::collect_data_auth::COLLECT_DATA_SUBMIT_OPERATION,
                    *electricity_cost,
                    *data_amount,
                    player_id,
                    public_key,
                    *nonce,
                    signature,
                )
                .map_err(|error| WorldError::ResourceBalanceInvalid {
                    reason: format!("authenticated data collection signature invalid: {error}"),
                })?;
                let matching_claims = self
                    .starter_oc_claims
                    .values()
                    .filter(|claim| {
                        claim.player_id == *player_id
                            && claim.public_key.as_deref() == Some(public_key.as_str())
                    })
                    .collect::<Vec<_>>();
                if matching_claims.len() != 1 || matching_claims[0].agent_id != *collector_agent_id
                {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "authenticated data collection requires exactly one starter OC player/key binding for collector {collector_agent_id}; found {}",
                            matching_claims.len()
                        ),
                    });
                }
                let next_resources = {
                    let collector = self.agents.get(collector_agent_id).ok_or_else(|| {
                        WorldError::AgentNotFound {
                            agent_id: collector_agent_id.clone(),
                        }
                    })?;
                    let mut next = collector.state.resources.clone();
                    next.remove(ResourceKind::Electricity, *electricity_cost)
                        .map_err(|err| WorldError::ResourceBalanceInvalid {
                            reason: format!("data collection electricity debit failed: {err:?}"),
                        })?;
                    next.add(ResourceKind::Data, *data_amount).map_err(|err| {
                        WorldError::ResourceBalanceInvalid {
                            reason: format!("data collection data credit failed: {err:?}"),
                        }
                    })?;
                    next
                };
                let collector = self.agents.get_mut(collector_agent_id).ok_or_else(|| {
                    WorldError::AgentNotFound {
                        agent_id: collector_agent_id.clone(),
                    }
                })?;
                collector.state.resources = next_resources;
                collector.last_active = now;
                self.authenticated_collect_data_last_nonces
                    .entry(player_id.clone())
                    .or_default()
                    .insert(public_key.clone(), *nonce);
            }
            DomainEvent::DataAccessGranted {
                owner_agent_id,
                grantee_agent_id,
            } => {
                if !self.agents.contains_key(owner_agent_id) {
                    return Err(WorldError::AgentNotFound {
                        agent_id: owner_agent_id.clone(),
                    });
                }
                if !self.agents.contains_key(grantee_agent_id) {
                    return Err(WorldError::AgentNotFound {
                        agent_id: grantee_agent_id.clone(),
                    });
                }
                if owner_agent_id != grantee_agent_id {
                    self.data_access_permissions
                        .entry(owner_agent_id.clone())
                        .or_default()
                        .insert(grantee_agent_id.clone());
                }
                if let Some(owner) = self.agents.get_mut(owner_agent_id) {
                    owner.last_active = now;
                }
                if owner_agent_id != grantee_agent_id {
                    if let Some(grantee) = self.agents.get_mut(grantee_agent_id) {
                        grantee.last_active = now;
                    }
                }
            }
            DomainEvent::DataAccessRevoked {
                owner_agent_id,
                grantee_agent_id,
            } => {
                if !self.agents.contains_key(owner_agent_id) {
                    return Err(WorldError::AgentNotFound {
                        agent_id: owner_agent_id.clone(),
                    });
                }
                if !self.agents.contains_key(grantee_agent_id) {
                    return Err(WorldError::AgentNotFound {
                        agent_id: grantee_agent_id.clone(),
                    });
                }
                if owner_agent_id != grantee_agent_id {
                    let remove_owner_entry = if let Some(grantees) =
                        self.data_access_permissions.get_mut(owner_agent_id)
                    {
                        grantees.remove(grantee_agent_id);
                        grantees.is_empty()
                    } else {
                        false
                    };
                    if remove_owner_entry {
                        self.data_access_permissions.remove(owner_agent_id);
                    }
                }
                if let Some(owner) = self.agents.get_mut(owner_agent_id) {
                    owner.last_active = now;
                }
                if owner_agent_id != grantee_agent_id {
                    if let Some(grantee) = self.agents.get_mut(grantee_agent_id) {
                        grantee.last_active = now;
                    }
                }
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
