use super::*;

impl WorldState {
    pub(super) fn apply_domain_event_core(
        &mut self,
        event: &DomainEvent,
        now: WorldTime,
    ) -> Result<(), WorldError> {
        match event {
            DomainEvent::AgentRegistered { agent_id, pos } => {
                let state = AgentState::new(agent_id, *pos);
                self.agents
                    .insert(agent_id.clone(), AgentCell::new(state, now));
            }
            DomainEvent::AgentMoved { agent_id, to, .. } => {
                if let Some(cell) = self.agents.get_mut(agent_id) {
                    cell.state.pos = *to;
                    cell.last_active = now;
                }
            }
            DomainEvent::ActionRejected { .. } => {}
            DomainEvent::Observation { .. } => {}
            DomainEvent::BodyAttributesUpdated { agent_id, view, .. } => {
                let cell =
                    self.agents
                        .get_mut(agent_id)
                        .ok_or_else(|| WorldError::AgentNotFound {
                            agent_id: agent_id.clone(),
                        })?;
                cell.state.body_view = view.clone();
                cell.last_active = now;
            }
            DomainEvent::BodyAttributesRejected { agent_id, .. } => {
                if let Some(cell) = self.agents.get_mut(agent_id) {
                    cell.last_active = now;
                } else {
                    return Err(WorldError::AgentNotFound {
                        agent_id: agent_id.clone(),
                    });
                }
            }
            DomainEvent::BodyInterfaceExpanded {
                agent_id,
                slot_capacity,
                expansion_level,
                consumed_item_id,
                new_slot_id,
                slot_type,
                ..
            } => {
                let cell =
                    self.agents
                        .get_mut(agent_id)
                        .ok_or_else(|| WorldError::AgentNotFound {
                            agent_id: agent_id.clone(),
                        })?;
                cell.state
                    .body_state
                    .consume_interface_module_item(consumed_item_id)
                    .map_err(|reason| WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "consume interface module item failed for {agent_id}: {reason}"
                        ),
                    })?;
                cell.state.body_state.slot_capacity = *slot_capacity;
                cell.state.body_state.expansion_level = *expansion_level;
                if !cell
                    .state
                    .body_state
                    .slots
                    .iter()
                    .any(|slot| slot.slot_id == *new_slot_id)
                {
                    cell.state
                        .body_state
                        .slots
                        .push(crate::models::BodyModuleSlot {
                            slot_id: new_slot_id.clone(),
                            slot_type: *slot_type,
                            installed_module: None,
                            locked: false,
                        });
                }
                cell.last_active = now;
            }
            DomainEvent::BodyInterfaceExpandRejected { agent_id, .. } => {
                if let Some(cell) = self.agents.get_mut(agent_id) {
                    cell.last_active = now;
                } else {
                    return Err(WorldError::AgentNotFound {
                        agent_id: agent_id.clone(),
                    });
                }
            }
            DomainEvent::ModuleArtifactDeployed {
                publisher_agent_id,
                wasm_hash,
                fee_kind,
                fee_amount,
                ..
            } => {
                self.settle_module_action_fee(
                    publisher_agent_id.as_str(),
                    *fee_kind,
                    *fee_amount,
                    now,
                )?;
                self.module_artifact_owners
                    .insert(wasm_hash.clone(), publisher_agent_id.clone());
                self.module_artifact_listings.remove(wasm_hash);
                self.module_artifact_bids.remove(wasm_hash);
            }
            DomainEvent::ModuleInstalled {
                installer_agent_id,
                instance_id,
                module_id,
                install_target,
                module_version,
                wasm_hash,
                active,
                fee_kind,
                fee_amount,
                ..
            } => {
                self.settle_module_action_fee(
                    installer_agent_id.as_str(),
                    *fee_kind,
                    *fee_amount,
                    now,
                )?;
                let resolved_instance_id = if instance_id.trim().is_empty() {
                    module_id.clone()
                } else {
                    instance_id.trim().to_string()
                };
                self.module_instances.insert(
                    resolved_instance_id.clone(),
                    ModuleInstanceState {
                        instance_id: resolved_instance_id,
                        module_id: module_id.clone(),
                        module_version: module_version.clone(),
                        wasm_hash: wasm_hash.clone(),
                        owner_agent_id: installer_agent_id.clone(),
                        install_target: install_target.clone(),
                        active: *active,
                        installed_at: now,
                    },
                );
                self.next_module_instance_id = self.next_module_instance_id.saturating_add(1);
                self.installed_module_targets
                    .insert(module_id.clone(), install_target.clone());
            }
            DomainEvent::ModuleUpgraded {
                upgrader_agent_id,
                instance_id,
                module_id,
                from_module_version,
                to_module_version,
                wasm_hash,
                install_target,
                active,
                fee_kind,
                fee_amount,
                ..
            } => {
                self.settle_module_action_fee(
                    upgrader_agent_id.as_str(),
                    *fee_kind,
                    *fee_amount,
                    now,
                )?;
                let instance = self.module_instances.get_mut(instance_id).ok_or_else(|| {
                    WorldError::ResourceBalanceInvalid {
                        reason: format!("module instance missing for upgrade {instance_id}"),
                    }
                })?;
                if instance.owner_agent_id != *upgrader_agent_id {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module instance owner mismatch for upgrade: instance={} owner={} upgrader={}",
                            instance_id, instance.owner_agent_id, upgrader_agent_id
                        ),
                    });
                }
                if instance.module_id != *module_id {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module instance module_id mismatch for upgrade: instance={} state_module_id={} event_module_id={}",
                            instance_id, instance.module_id, module_id
                        ),
                    });
                }
                if instance.module_version != *from_module_version {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module instance from_version mismatch for upgrade: instance={} state_version={} event_from={}",
                            instance_id, instance.module_version, from_module_version
                        ),
                    });
                }
                instance.module_version = to_module_version.clone();
                instance.wasm_hash = wasm_hash.clone();
                instance.install_target = install_target.clone();
                instance.active = *active;
                self.installed_module_targets
                    .insert(module_id.clone(), install_target.clone());
            }
            DomainEvent::ModuleArtifactListed {
                seller_agent_id,
                wasm_hash,
                price_kind,
                price_amount,
                order_id,
                fee_kind,
                fee_amount,
            } => {
                if *price_amount <= 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module artifact listing price must be > 0, got {}",
                            price_amount
                        ),
                    });
                }
                let owner = self.module_artifact_owners.get(wasm_hash).ok_or_else(|| {
                    WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module artifact owner missing for listing hash {}",
                            wasm_hash
                        ),
                    }
                })?;
                if owner != seller_agent_id {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module artifact listing seller mismatch: hash={} owner={} seller={}",
                            wasm_hash, owner, seller_agent_id
                        ),
                    });
                }
                self.settle_module_action_fee(
                    seller_agent_id.as_str(),
                    *fee_kind,
                    *fee_amount,
                    now,
                )?;
                self.module_artifact_listings.insert(
                    wasm_hash.clone(),
                    ModuleArtifactListingState {
                        order_id: *order_id,
                        seller_agent_id: seller_agent_id.clone(),
                        price_kind: *price_kind,
                        price_amount: *price_amount,
                        listed_at: now,
                    },
                );
                if *order_id > 0 {
                    self.next_module_market_order_id = self
                        .next_module_market_order_id
                        .max(order_id.saturating_add(1));
                }
            }
            DomainEvent::ModuleArtifactDelisted {
                seller_agent_id,
                wasm_hash,
                order_id,
                fee_kind,
                fee_amount,
            } => {
                let listing = self
                    .module_artifact_listings
                    .get(wasm_hash)
                    .ok_or_else(|| WorldError::ResourceBalanceInvalid {
                        reason: format!("module artifact listing missing for hash {}", wasm_hash),
                    })?;
                if listing.seller_agent_id != *seller_agent_id {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module artifact delist seller mismatch: hash={} listing_seller={} event_seller={}",
                            wasm_hash, listing.seller_agent_id, seller_agent_id
                        ),
                    });
                }
                if let Some(expected_order_id) = order_id {
                    if listing.order_id != *expected_order_id {
                        return Err(WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "module artifact delist order mismatch: hash={} listing_order_id={} event_order_id={}",
                                wasm_hash, listing.order_id, expected_order_id
                            ),
                        });
                    }
                }
                let owner = self.module_artifact_owners.get(wasm_hash).ok_or_else(|| {
                    WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module artifact owner missing for delist hash {}",
                            wasm_hash
                        ),
                    }
                })?;
                if owner != seller_agent_id {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module artifact delist seller is not owner: hash={} owner={} seller={}",
                            wasm_hash, owner, seller_agent_id
                        ),
                    });
                }
                self.settle_module_action_fee(
                    seller_agent_id.as_str(),
                    *fee_kind,
                    *fee_amount,
                    now,
                )?;
                self.module_artifact_listings.remove(wasm_hash);
            }
            DomainEvent::ModuleArtifactDestroyed {
                owner_agent_id,
                wasm_hash,
                reason,
                fee_kind,
                fee_amount,
            } => {
                if reason.trim().is_empty() {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module artifact destroy reason cannot be empty for hash {}",
                            wasm_hash
                        ),
                    });
                }
                let owner = self.module_artifact_owners.get(wasm_hash).ok_or_else(|| {
                    WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module artifact owner missing for destroy hash {}",
                            wasm_hash
                        ),
                    }
                })?;
                if owner != owner_agent_id {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module artifact destroy owner mismatch: hash={} owner={} event_owner={}",
                            wasm_hash, owner, owner_agent_id
                        ),
                    });
                }
                self.settle_module_action_fee(
                    owner_agent_id.as_str(),
                    *fee_kind,
                    *fee_amount,
                    now,
                )?;
                self.module_artifact_owners.remove(wasm_hash);
                self.module_artifact_listings.remove(wasm_hash);
                self.module_artifact_bids.remove(wasm_hash);
            }
            DomainEvent::ModuleArtifactBidPlaced {
                bidder_agent_id,
                wasm_hash,
                order_id,
                price_kind,
                price_amount,
            } => {
                if *order_id == 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module artifact bid order_id must be > 0 for hash {}",
                            wasm_hash
                        ),
                    });
                }
                if *price_amount <= 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module artifact bid price must be > 0, got {}",
                            price_amount
                        ),
                    });
                }
                if !self.agents.contains_key(bidder_agent_id) {
                    return Err(WorldError::AgentNotFound {
                        agent_id: bidder_agent_id.clone(),
                    });
                }
                self.next_module_market_order_id = self
                    .next_module_market_order_id
                    .max(order_id.saturating_add(1));
                self.module_artifact_bids
                    .entry(wasm_hash.clone())
                    .or_default()
                    .push(ModuleArtifactBidState {
                        order_id: *order_id,
                        bidder_agent_id: bidder_agent_id.clone(),
                        price_kind: *price_kind,
                        price_amount: *price_amount,
                        bid_at: now,
                    });
                if let Some(cell) = self.agents.get_mut(bidder_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::ModuleArtifactBidCancelled {
                bidder_agent_id,
                wasm_hash,
                order_id,
                ..
            } => {
                let remove_empty_entry = {
                    let bids = self
                        .module_artifact_bids
                        .get_mut(wasm_hash)
                        .ok_or_else(|| WorldError::ResourceBalanceInvalid {
                            reason: format!("module artifact bids missing for hash {}", wasm_hash),
                        })?;
                    let before = bids.len();
                    bids.retain(|entry| {
                        !(entry.order_id == *order_id && entry.bidder_agent_id == *bidder_agent_id)
                    });
                    if before == bids.len() {
                        return Err(WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "module artifact bid cancel target not found: hash={} order_id={} bidder={}",
                                wasm_hash, order_id, bidder_agent_id
                            ),
                        });
                    }
                    bids.is_empty()
                };
                if remove_empty_entry {
                    self.module_artifact_bids.remove(wasm_hash);
                }
                if let Some(cell) = self.agents.get_mut(bidder_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::ModuleArtifactSaleCompleted {
                buyer_agent_id,
                seller_agent_id,
                wasm_hash,
                price_kind,
                price_amount,
                sale_id,
                listing_order_id,
                bid_order_id,
            } => {
                if buyer_agent_id == seller_agent_id {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module artifact buyer and seller cannot be the same: {}",
                            buyer_agent_id
                        ),
                    });
                }
                if *price_amount <= 0 {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module artifact sale price must be > 0, got {}",
                            price_amount
                        ),
                    });
                }

                let listing = self
                    .module_artifact_listings
                    .get(wasm_hash)
                    .ok_or_else(|| WorldError::ResourceBalanceInvalid {
                        reason: format!("module artifact listing missing for hash {}", wasm_hash),
                    })?;
                if listing.seller_agent_id != *seller_agent_id
                    || listing.price_kind != *price_kind
                    || listing.price_amount != *price_amount
                {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!("module artifact listing mismatch for hash {}", wasm_hash),
                    });
                }
                if let Some(expected_listing_order_id) = listing_order_id {
                    if listing.order_id != *expected_listing_order_id {
                        return Err(WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "module artifact sale listing order mismatch: hash={} listing_order_id={} event_order_id={}",
                                wasm_hash, listing.order_id, expected_listing_order_id
                            ),
                        });
                    }
                }
                let owner = self.module_artifact_owners.get(wasm_hash).ok_or_else(|| {
                    WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module artifact owner missing for sale hash {}",
                            wasm_hash
                        ),
                    }
                })?;
                if owner != seller_agent_id {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "module artifact sale seller is not owner: hash={} owner={} seller={}",
                            wasm_hash, owner, seller_agent_id
                        ),
                    });
                }

                let mut seller = self.agents.remove(seller_agent_id).ok_or_else(|| {
                    WorldError::AgentNotFound {
                        agent_id: seller_agent_id.clone(),
                    }
                })?;
                let mut buyer = self.agents.remove(buyer_agent_id).ok_or_else(|| {
                    WorldError::AgentNotFound {
                        agent_id: buyer_agent_id.clone(),
                    }
                })?;

                buyer
                    .state
                    .resources
                    .remove(*price_kind, *price_amount)
                    .map_err(|err| WorldError::ResourceBalanceInvalid {
                        reason: format!("module artifact sale buyer debit failed: {err:?}"),
                    })?;
                seller
                    .state
                    .resources
                    .add(*price_kind, *price_amount)
                    .map_err(|err| WorldError::ResourceBalanceInvalid {
                        reason: format!("module artifact sale seller credit failed: {err:?}"),
                    })?;
                seller.last_active = now;
                buyer.last_active = now;

                self.agents.insert(seller_agent_id.clone(), seller);
                self.agents.insert(buyer_agent_id.clone(), buyer);
                self.module_artifact_owners
                    .insert(wasm_hash.clone(), buyer_agent_id.clone());
                self.module_artifact_listings.remove(wasm_hash);
                if let Some(expected_bid_order_id) = bid_order_id {
                    let remove_empty_entry = {
                        let bids =
                            self.module_artifact_bids
                                .get_mut(wasm_hash)
                                .ok_or_else(|| WorldError::ResourceBalanceInvalid {
                                    reason: format!(
                                        "module artifact sale bid missing for hash {} order_id {}",
                                        wasm_hash, expected_bid_order_id
                                    ),
                                })?;
                        let before = bids.len();
                        bids.retain(|entry| {
                            !(entry.order_id == *expected_bid_order_id
                                && entry.bidder_agent_id == *buyer_agent_id)
                        });
                        if before == bids.len() {
                            return Err(WorldError::ResourceBalanceInvalid {
                                reason: format!(
                                    "module artifact sale bid not found: hash={} order_id={} buyer={}",
                                    wasm_hash, expected_bid_order_id, buyer_agent_id
                                ),
                            });
                        }
                        bids.is_empty()
                    };
                    if remove_empty_entry {
                        self.module_artifact_bids.remove(wasm_hash);
                    }
                }
                if *sale_id > 0 {
                    self.next_module_market_sale_id = self
                        .next_module_market_sale_id
                        .max(sale_id.saturating_add(1));
                }
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
                    // Validate and precompute both sides first so transfer is atomic.
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
            event @ DomainEvent::MainTokenGenesisInitialized { .. } => {
                self.apply_domain_event_main_token(event, now)?;
            }
            event @ DomainEvent::MainTokenVestingClaimed { .. } => {
                self.apply_domain_event_main_token(event, now)?;
            }
            event @ DomainEvent::MainTokenEpochIssued { .. } => {
                self.apply_domain_event_main_token(event, now)?;
            }
            event @ DomainEvent::MainTokenFeeSettled { .. } => {
                self.apply_domain_event_main_token(event, now)?;
            }
            event @ DomainEvent::MainTokenPolicyUpdateScheduled { .. } => {
                self.apply_domain_event_main_token(event, now)?;
            }
            event @ DomainEvent::MaterialTransferred { .. } => {
                self.apply_domain_event_industry(event, now)?;
            }
            event @ DomainEvent::MaterialTransitStarted { .. } => {
                self.apply_domain_event_industry(event, now)?;
            }
            event @ DomainEvent::MaterialTransitCompleted { .. } => {
                self.apply_domain_event_industry(event, now)?;
            }
            event @ DomainEvent::FactoryBuildStarted { .. } => {
                self.apply_domain_event_industry(event, now)?;
            }
            event @ DomainEvent::FactoryBuilt { .. } => {
                self.apply_domain_event_industry(event, now)?;
            }
            event @ DomainEvent::FactoryDurabilityChanged { .. } => {
                self.apply_domain_event_industry(event, now)?;
            }
            event @ DomainEvent::FactoryMaintained { .. } => {
                self.apply_domain_event_industry(event, now)?;
            }
            event @ DomainEvent::FactoryRecycled { .. } => {
                self.apply_domain_event_industry(event, now)?;
            }
            event @ DomainEvent::RecipeStarted { .. } => {
                self.apply_domain_event_industry(event, now)?;
            }
            event @ DomainEvent::RecipeCompleted { .. } => {
                self.apply_domain_event_industry(event, now)?;
            }
            _ => unreachable!("apply_domain_event_core received unsupported event variant"),
        }
        Ok(())
    }
}
