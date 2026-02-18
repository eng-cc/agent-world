//! World state management.

use crate::models::AgentState;
use crate::simulator::ResourceKind;
use agent_world_wasm_abi::{FactoryModuleSpec, MaterialStack};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::agent_cell::AgentCell;
use super::error::WorldError;
use super::events::DomainEvent;
use super::node_points::EpochSettlementReport;
use super::reward_asset::{
    reward_mint_signature_v1, verify_reward_mint_signature_v2, NodeAssetBalance,
    NodeRewardMintRecord, ProtocolPowerReserve, RewardAssetConfig, RewardSignatureGovernancePolicy,
    SystemOrderPoolBudget, REWARD_MINT_SIGNATURE_V1_PREFIX, REWARD_MINT_SIGNATURE_V2_PREFIX,
};
use super::types::{ActionId, MaterialLedgerId, WorldTime};
use super::util::hash_json;

fn default_world_material_ledger() -> MaterialLedgerId {
    MaterialLedgerId::world()
}

fn default_material_ledgers() -> BTreeMap<MaterialLedgerId, BTreeMap<String, i64>> {
    let mut ledgers = BTreeMap::new();
    ledgers.insert(MaterialLedgerId::world(), BTreeMap::new());
    ledgers
}

fn default_module_market_order_id() -> u64 {
    1
}

fn default_module_market_sale_id() -> u64 {
    1
}

/// Persisted factory instance state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactoryState {
    pub factory_id: String,
    pub site_id: String,
    pub builder_agent_id: String,
    pub spec: FactoryModuleSpec,
    #[serde(default = "default_world_material_ledger")]
    pub input_ledger: MaterialLedgerId,
    #[serde(default = "default_world_material_ledger")]
    pub output_ledger: MaterialLedgerId,
    pub built_at: WorldTime,
}

/// In-flight factory construction tracked by job id.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactoryBuildJobState {
    pub job_id: ActionId,
    pub builder_agent_id: String,
    pub site_id: String,
    pub spec: FactoryModuleSpec,
    #[serde(default = "default_world_material_ledger")]
    pub consume_ledger: MaterialLedgerId,
    pub ready_at: WorldTime,
}

/// In-flight recipe execution tracked by job id.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeJobState {
    pub job_id: ActionId,
    pub requester_agent_id: String,
    pub factory_id: String,
    pub recipe_id: String,
    pub accepted_batches: u32,
    pub consume: Vec<MaterialStack>,
    pub produce: Vec<MaterialStack>,
    pub byproducts: Vec<MaterialStack>,
    pub power_required: i64,
    pub duration_ticks: u32,
    #[serde(default = "default_world_material_ledger")]
    pub consume_ledger: MaterialLedgerId,
    #[serde(default = "default_world_material_ledger")]
    pub output_ledger: MaterialLedgerId,
    pub ready_at: WorldTime,
}

/// In-flight material transit tracked by job id.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MaterialTransitJobState {
    pub job_id: ActionId,
    pub requester_agent_id: String,
    pub from_ledger: MaterialLedgerId,
    pub to_ledger: MaterialLedgerId,
    pub kind: String,
    pub amount: i64,
    pub distance_km: i64,
    pub loss_bps: i64,
    pub ready_at: WorldTime,
}

/// Active market listing for one module artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleArtifactListingState {
    #[serde(default)]
    pub order_id: u64,
    pub seller_agent_id: String,
    pub price_kind: ResourceKind,
    pub price_amount: i64,
    pub listed_at: WorldTime,
}

/// Active bid order for one module artifact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleArtifactBidState {
    pub order_id: u64,
    pub bidder_agent_id: String,
    pub price_kind: ResourceKind,
    pub price_amount: i64,
    pub bid_at: WorldTime,
}

/// The mutable state of the world.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldState {
    pub time: WorldTime,
    pub agents: BTreeMap<String, AgentCell>,
    #[serde(default)]
    pub resources: BTreeMap<ResourceKind, i64>,
    #[serde(default)]
    pub materials: BTreeMap<String, i64>,
    #[serde(default = "default_material_ledgers")]
    pub material_ledgers: BTreeMap<MaterialLedgerId, BTreeMap<String, i64>>,
    #[serde(default)]
    pub factories: BTreeMap<String, FactoryState>,
    #[serde(default)]
    pub pending_factory_builds: BTreeMap<ActionId, FactoryBuildJobState>,
    #[serde(default)]
    pub pending_recipe_jobs: BTreeMap<ActionId, RecipeJobState>,
    #[serde(default)]
    pub pending_material_transits: BTreeMap<ActionId, MaterialTransitJobState>,
    #[serde(default)]
    pub module_states: BTreeMap<String, Vec<u8>>,
    #[serde(default)]
    pub module_artifact_owners: BTreeMap<String, String>,
    #[serde(default)]
    pub module_artifact_listings: BTreeMap<String, ModuleArtifactListingState>,
    #[serde(default)]
    pub module_artifact_bids: BTreeMap<String, Vec<ModuleArtifactBidState>>,
    #[serde(default = "default_module_market_order_id")]
    pub next_module_market_order_id: u64,
    #[serde(default = "default_module_market_sale_id")]
    pub next_module_market_sale_id: u64,
    #[serde(default)]
    pub reward_asset_config: RewardAssetConfig,
    #[serde(default)]
    pub node_asset_balances: BTreeMap<String, NodeAssetBalance>,
    #[serde(default)]
    pub protocol_power_reserve: ProtocolPowerReserve,
    #[serde(default)]
    pub reward_mint_records: Vec<NodeRewardMintRecord>,
    #[serde(default)]
    pub node_redeem_nonces: BTreeMap<String, u64>,
    #[serde(default)]
    pub system_order_pool_budgets: BTreeMap<u64, SystemOrderPoolBudget>,
    #[serde(default)]
    pub node_identity_bindings: BTreeMap<String, String>,
    #[serde(default)]
    pub reward_signature_governance_policy: RewardSignatureGovernancePolicy,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            time: 0,
            agents: BTreeMap::new(),
            resources: BTreeMap::new(),
            materials: BTreeMap::new(),
            material_ledgers: default_material_ledgers(),
            factories: BTreeMap::new(),
            pending_factory_builds: BTreeMap::new(),
            pending_recipe_jobs: BTreeMap::new(),
            pending_material_transits: BTreeMap::new(),
            module_states: BTreeMap::new(),
            module_artifact_owners: BTreeMap::new(),
            module_artifact_listings: BTreeMap::new(),
            module_artifact_bids: BTreeMap::new(),
            next_module_market_order_id: default_module_market_order_id(),
            next_module_market_sale_id: default_module_market_sale_id(),
            reward_asset_config: RewardAssetConfig::default(),
            node_asset_balances: BTreeMap::new(),
            protocol_power_reserve: ProtocolPowerReserve::default(),
            reward_mint_records: Vec::new(),
            node_redeem_nonces: BTreeMap::new(),
            system_order_pool_budgets: BTreeMap::new(),
            node_identity_bindings: BTreeMap::new(),
            reward_signature_governance_policy: RewardSignatureGovernancePolicy::default(),
        }
    }
}

impl WorldState {
    pub fn migrate_legacy_material_ledgers(&mut self) {
        self.material_ledgers
            .entry(MaterialLedgerId::world())
            .or_default();

        let world_ledger = self
            .material_ledgers
            .get(&MaterialLedgerId::world())
            .cloned()
            .unwrap_or_default();
        if world_ledger.is_empty() && !self.materials.is_empty() {
            self.material_ledgers
                .insert(MaterialLedgerId::world(), self.materials.clone());
        }

        sync_legacy_world_materials(&self.material_ledgers, &mut self.materials);
    }

    fn settle_module_action_fee(
        &mut self,
        agent_id: &str,
        fee_kind: ResourceKind,
        fee_amount: i64,
        now: WorldTime,
    ) -> Result<(), WorldError> {
        if fee_amount < 0 {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: format!("module action fee must be >= 0, got {}", fee_amount),
            });
        }

        let cell = self
            .agents
            .get_mut(agent_id)
            .ok_or_else(|| WorldError::AgentNotFound {
                agent_id: agent_id.to_string(),
            })?;
        if fee_amount > 0 {
            cell.state
                .resources
                .remove(fee_kind, fee_amount)
                .map_err(|err| WorldError::ResourceBalanceInvalid {
                    reason: format!(
                        "module action fee debit failed: agent={} kind={:?} amount={} err={:?}",
                        agent_id, fee_kind, fee_amount, err
                    ),
                })?;
            let treasury = self.resources.entry(fee_kind).or_insert(0);
            *treasury = treasury.saturating_add(fee_amount);
        }
        cell.last_active = now;
        Ok(())
    }

    pub fn apply_domain_event(
        &mut self,
        event: &DomainEvent,
        now: WorldTime,
    ) -> Result<(), WorldError> {
        self.migrate_legacy_material_ledgers();
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
                    let mut from = self.agents.remove(from_agent_id).ok_or_else(|| {
                        WorldError::AgentNotFound {
                            agent_id: from_agent_id.clone(),
                        }
                    })?;
                    let mut to = self.agents.remove(to_agent_id).ok_or_else(|| {
                        WorldError::AgentNotFound {
                            agent_id: to_agent_id.clone(),
                        }
                    })?;

                    from.state.resources.remove(*kind, *amount).map_err(|err| {
                        WorldError::ResourceBalanceInvalid {
                            reason: format!("transfer remove failed: {err:?}"),
                        }
                    })?;
                    to.state.resources.add(*kind, *amount).map_err(|err| {
                        WorldError::ResourceBalanceInvalid {
                            reason: format!("transfer add failed: {err:?}"),
                        }
                    })?;
                    from.last_active = now;
                    to.last_active = now;

                    self.agents.insert(from_agent_id.clone(), from);
                    self.agents.insert(to_agent_id.clone(), to);
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
                remove_node_power_credits(
                    &mut self.node_asset_balances,
                    node_id.as_str(),
                    *burned_credits,
                )
                .map_err(|reason| WorldError::ResourceBalanceInvalid {
                    reason: format!("power redeem burn failed: {reason}"),
                })?;

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
                self.protocol_power_reserve.available_power_units = next_reserve;
                self.protocol_power_reserve.redeemed_power_units = next_redeemed;
                self.node_redeem_nonces.insert(node_id.clone(), *nonce);

                let target = self.agents.get_mut(target_agent_id).ok_or_else(|| {
                    WorldError::AgentNotFound {
                        agent_id: target_agent_id.clone(),
                    }
                })?;
                target
                    .state
                    .resources
                    .add(ResourceKind::Electricity, *granted_power_units)
                    .map_err(|err| WorldError::ResourceBalanceInvalid {
                        reason: format!("power redeem add electricity failed: {err:?}"),
                    })?;
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
            } => {
                apply_node_points_settlement_event(
                    self,
                    report,
                    signer_node_id.as_str(),
                    settlement_hash.as_str(),
                    minted_records.as_slice(),
                )?;
            }
            DomainEvent::MaterialTransferred {
                requester_agent_id,
                from_ledger,
                to_ledger,
                kind,
                amount,
                ..
            } => {
                remove_material_balance_for_ledger(
                    &mut self.material_ledgers,
                    from_ledger,
                    kind.as_str(),
                    *amount,
                )
                .map_err(|reason| WorldError::ResourceBalanceInvalid {
                    reason: format!("material transfer remove failed: {reason}"),
                })?;
                add_material_balance_for_ledger(
                    &mut self.material_ledgers,
                    to_ledger,
                    kind.as_str(),
                    *amount,
                )
                .map_err(|reason| WorldError::ResourceBalanceInvalid {
                    reason: format!("material transfer add failed: {reason}"),
                })?;
                if let Some(cell) = self.agents.get_mut(requester_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::MaterialTransitStarted {
                job_id,
                requester_agent_id,
                from_ledger,
                to_ledger,
                kind,
                amount,
                distance_km,
                loss_bps,
                ready_at,
            } => {
                remove_material_balance_for_ledger(
                    &mut self.material_ledgers,
                    from_ledger,
                    kind.as_str(),
                    *amount,
                )
                .map_err(|reason| WorldError::ResourceBalanceInvalid {
                    reason: format!("material transit reserve failed: {reason}"),
                })?;
                self.pending_material_transits.insert(
                    *job_id,
                    MaterialTransitJobState {
                        job_id: *job_id,
                        requester_agent_id: requester_agent_id.clone(),
                        from_ledger: from_ledger.clone(),
                        to_ledger: to_ledger.clone(),
                        kind: kind.clone(),
                        amount: *amount,
                        distance_km: *distance_km,
                        loss_bps: *loss_bps,
                        ready_at: *ready_at,
                    },
                );
                if let Some(cell) = self.agents.get_mut(requester_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::MaterialTransitCompleted {
                job_id,
                requester_agent_id,
                to_ledger,
                kind,
                received_amount,
                ..
            } => {
                self.pending_material_transits.remove(job_id);
                if *received_amount > 0 {
                    add_material_balance_for_ledger(
                        &mut self.material_ledgers,
                        to_ledger,
                        kind.as_str(),
                        *received_amount,
                    )
                    .map_err(|reason| WorldError::ResourceBalanceInvalid {
                        reason: format!("material transit completion failed: {reason}"),
                    })?;
                }
                if let Some(cell) = self.agents.get_mut(requester_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::FactoryBuildStarted {
                job_id,
                builder_agent_id,
                site_id,
                spec,
                consume_ledger,
                ready_at,
            } => {
                for stack in &spec.build_cost {
                    remove_material_balance_for_ledger(
                        &mut self.material_ledgers,
                        consume_ledger,
                        stack.kind.as_str(),
                        stack.amount,
                    )
                    .map_err(|reason| WorldError::ResourceBalanceInvalid {
                        reason: format!("factory build consume failed: {reason}"),
                    })?;
                }
                self.pending_factory_builds.insert(
                    *job_id,
                    FactoryBuildJobState {
                        job_id: *job_id,
                        builder_agent_id: builder_agent_id.clone(),
                        site_id: site_id.clone(),
                        spec: spec.clone(),
                        consume_ledger: consume_ledger.clone(),
                        ready_at: *ready_at,
                    },
                );
                if let Some(cell) = self.agents.get_mut(builder_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::FactoryBuilt {
                job_id,
                builder_agent_id,
                site_id,
                spec,
            } => {
                self.pending_factory_builds.remove(job_id);
                let site_ledger = MaterialLedgerId::site(site_id.clone());
                self.factories.insert(
                    spec.factory_id.clone(),
                    FactoryState {
                        factory_id: spec.factory_id.clone(),
                        site_id: site_id.clone(),
                        builder_agent_id: builder_agent_id.clone(),
                        spec: spec.clone(),
                        input_ledger: site_ledger.clone(),
                        output_ledger: site_ledger,
                        built_at: now,
                    },
                );
                if let Some(cell) = self.agents.get_mut(builder_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::RecipeStarted {
                job_id,
                requester_agent_id,
                factory_id,
                recipe_id,
                accepted_batches,
                consume,
                produce,
                byproducts,
                power_required,
                duration_ticks,
                consume_ledger,
                output_ledger,
                ready_at,
            } => {
                for stack in consume {
                    remove_material_balance_for_ledger(
                        &mut self.material_ledgers,
                        consume_ledger,
                        stack.kind.as_str(),
                        stack.amount,
                    )
                    .map_err(|reason| WorldError::ResourceBalanceInvalid {
                        reason: format!("recipe consume failed: {reason}"),
                    })?;
                }
                remove_resource_balance(
                    &mut self.resources,
                    ResourceKind::Electricity,
                    *power_required,
                )
                .map_err(|reason| WorldError::ResourceBalanceInvalid {
                    reason: format!("recipe power consume failed: {reason}"),
                })?;
                self.pending_recipe_jobs.insert(
                    *job_id,
                    RecipeJobState {
                        job_id: *job_id,
                        requester_agent_id: requester_agent_id.clone(),
                        factory_id: factory_id.clone(),
                        recipe_id: recipe_id.clone(),
                        accepted_batches: *accepted_batches,
                        consume: consume.clone(),
                        produce: produce.clone(),
                        byproducts: byproducts.clone(),
                        power_required: *power_required,
                        duration_ticks: *duration_ticks,
                        consume_ledger: consume_ledger.clone(),
                        output_ledger: output_ledger.clone(),
                        ready_at: *ready_at,
                    },
                );
                if let Some(cell) = self.agents.get_mut(requester_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::RecipeCompleted {
                job_id,
                requester_agent_id,
                produce,
                byproducts,
                output_ledger,
                ..
            } => {
                self.pending_recipe_jobs.remove(job_id);
                for stack in produce {
                    add_material_balance_for_ledger(
                        &mut self.material_ledgers,
                        output_ledger,
                        stack.kind.as_str(),
                        stack.amount,
                    )
                    .map_err(|reason| WorldError::ResourceBalanceInvalid {
                        reason: format!("recipe produce failed: {reason}"),
                    })?;
                }
                for stack in byproducts {
                    add_material_balance_for_ledger(
                        &mut self.material_ledgers,
                        output_ledger,
                        stack.kind.as_str(),
                        stack.amount,
                    )
                    .map_err(|reason| WorldError::ResourceBalanceInvalid {
                        reason: format!("recipe byproduct failed: {reason}"),
                    })?;
                }
                if let Some(cell) = self.agents.get_mut(requester_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::ProductValidated {
                requester_agent_id, ..
            } => {
                if let Some(cell) = self.agents.get_mut(requester_agent_id) {
                    cell.last_active = now;
                }
            }
        }
        sync_legacy_world_materials(&self.material_ledgers, &mut self.materials);
        Ok(())
    }

    pub fn route_domain_event(&mut self, event: &DomainEvent) {
        match event {
            DomainEvent::ResourceTransferred {
                from_agent_id,
                to_agent_id,
                ..
            } => {
                if let Some(cell) = self.agents.get_mut(from_agent_id) {
                    cell.mailbox.push_back(event.clone());
                }
                if from_agent_id != to_agent_id {
                    if let Some(cell) = self.agents.get_mut(to_agent_id) {
                        cell.mailbox.push_back(event.clone());
                    }
                }
            }
            _ => {
                let Some(agent_id) = event.agent_id() else {
                    return;
                };
                if let Some(cell) = self.agents.get_mut(agent_id) {
                    cell.mailbox.push_back(event.clone());
                }
            }
        }
    }
}

fn apply_node_points_settlement_event(
    state: &mut WorldState,
    report: &EpochSettlementReport,
    signer_node_id: &str,
    settlement_hash: &str,
    minted_records: &[NodeRewardMintRecord],
) -> Result<(), WorldError> {
    if signer_node_id.trim().is_empty() {
        return Err(WorldError::ResourceBalanceInvalid {
            reason: "settlement signer_node_id cannot be empty".to_string(),
        });
    }
    let expected_hash = hash_json(report)?;
    if expected_hash != settlement_hash {
        return Err(WorldError::ResourceBalanceInvalid {
            reason: format!(
                "settlement_hash mismatch: expected={} actual={}",
                expected_hash, settlement_hash
            ),
        });
    }
    let points_per_credit = state.reward_asset_config.points_per_credit;
    if points_per_credit == 0 {
        return Err(WorldError::ResourceBalanceInvalid {
            reason: "points_per_credit must be positive".to_string(),
        });
    }
    if !state.node_identity_bindings.contains_key(signer_node_id) {
        return Err(WorldError::ResourceBalanceInvalid {
            reason: format!("node identity is not bound: {signer_node_id}"),
        });
    }

    let mut settlement_points = BTreeMap::new();
    for settlement in &report.settlements {
        if settlement.node_id.trim().is_empty() {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: "report settlement contains empty node_id".to_string(),
            });
        }
        if settlement_points
            .insert(settlement.node_id.clone(), settlement.awarded_points)
            .is_some()
        {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: format!(
                    "duplicate settlement node in report: {}",
                    settlement.node_id
                ),
            });
        }
        if !state
            .node_identity_bindings
            .contains_key(settlement.node_id.as_str())
        {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: format!("node identity is not bound: {}", settlement.node_id),
            });
        }
    }

    let mut budget = state
        .system_order_pool_budgets
        .get(&report.epoch_index)
        .cloned();
    if let Some(item) = budget.as_mut() {
        ensure_system_order_budget_caps_for_epoch(report, item);
    }

    let mut seen_nodes = BTreeMap::<String, ()>::new();
    for record in minted_records {
        if record.epoch_index != report.epoch_index {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: format!(
                    "mint record epoch mismatch: report={} record={}",
                    report.epoch_index, record.epoch_index
                ),
            });
        }
        if record.signer_node_id != signer_node_id {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: format!(
                    "mint record signer mismatch: event={} record={}",
                    signer_node_id, record.signer_node_id
                ),
            });
        }
        if record.settlement_hash != settlement_hash {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: format!(
                    "mint record settlement_hash mismatch for node {}",
                    record.node_id
                ),
            });
        }
        let Some(awarded_points) = settlement_points.get(record.node_id.as_str()) else {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: format!(
                    "mint record node is missing in report settlements: {}",
                    record.node_id
                ),
            });
        };
        if record.source_awarded_points != *awarded_points {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: format!(
                    "mint record awarded points mismatch for node {}: report={} record={}",
                    record.node_id, awarded_points, record.source_awarded_points
                ),
            });
        }
        if record.minted_power_credits == 0 {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: format!(
                    "mint record has zero minted_power_credits for node {}",
                    record.node_id
                ),
            });
        }
        let max_minted = record.source_awarded_points / points_per_credit;
        if record.minted_power_credits > max_minted {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: format!(
                    "minted credits exceed settlement cap for node {}: minted={} cap={}",
                    record.node_id, record.minted_power_credits, max_minted
                ),
            });
        }
        if seen_nodes.insert(record.node_id.clone(), ()).is_some() {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: format!(
                    "duplicate mint record node in one action: {}",
                    record.node_id
                ),
            });
        }
        if state.reward_mint_records.iter().any(|existing| {
            existing.epoch_index == record.epoch_index && existing.node_id == record.node_id
        }) {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: format!(
                    "mint record already exists for epoch={} node={}",
                    record.epoch_index, record.node_id
                ),
            });
        }
        verify_reward_mint_record_signature_with_state(state, record).map_err(|reason| {
            WorldError::ResourceBalanceInvalid {
                reason: format!(
                    "mint record signature invalid (epoch={} node={}): {}",
                    record.epoch_index, record.node_id, reason
                ),
            }
        })?;

        if let Some(item) = budget.as_mut() {
            let node_cap = item
                .node_credit_caps
                .get(record.node_id.as_str())
                .copied()
                .unwrap_or(0);
            let node_allocated = item
                .node_credit_allocated
                .get(record.node_id.as_str())
                .copied()
                .unwrap_or(0);
            let node_remaining = node_cap.saturating_sub(node_allocated);
            if record.minted_power_credits > node_remaining {
                return Err(WorldError::ResourceBalanceInvalid {
                    reason: format!(
                        "minted credits exceed node budget cap for {}: minted={} remaining={}",
                        record.node_id, record.minted_power_credits, node_remaining
                    ),
                });
            }
            if record.minted_power_credits > item.remaining_credit_budget {
                return Err(WorldError::ResourceBalanceInvalid {
                    reason: format!(
                        "minted credits exceed remaining system order budget: minted={} remaining={}",
                        record.minted_power_credits, item.remaining_credit_budget
                    ),
                });
            }
            item.remaining_credit_budget = item
                .remaining_credit_budget
                .saturating_sub(record.minted_power_credits);
            item.node_credit_allocated
                .entry(record.node_id.clone())
                .and_modify(|value| *value = value.saturating_add(record.minted_power_credits))
                .or_insert(record.minted_power_credits);
        }
    }

    for record in minted_records {
        let balance = state
            .node_asset_balances
            .entry(record.node_id.clone())
            .or_insert_with(|| NodeAssetBalance {
                node_id: record.node_id.clone(),
                ..NodeAssetBalance::default()
            });
        balance.power_credit_balance = balance
            .power_credit_balance
            .saturating_add(record.minted_power_credits);
        balance.total_minted_credits = balance
            .total_minted_credits
            .saturating_add(record.minted_power_credits);
        state.reward_mint_records.push(record.clone());
    }
    if let Some(item) = budget {
        state
            .system_order_pool_budgets
            .insert(report.epoch_index, item);
    }
    Ok(())
}

fn add_material_balance(
    balances: &mut BTreeMap<String, i64>,
    kind: &str,
    amount: i64,
) -> Result<(), String> {
    if amount < 0 {
        return Err(format!("negative material amount not allowed: {amount}"));
    }
    let entry = balances.entry(kind.to_string()).or_insert(0);
    *entry = entry.saturating_add(amount);
    if *entry == 0 {
        balances.remove(kind);
    }
    Ok(())
}

fn add_material_balance_for_ledger(
    ledgers: &mut BTreeMap<MaterialLedgerId, BTreeMap<String, i64>>,
    ledger: &MaterialLedgerId,
    kind: &str,
    amount: i64,
) -> Result<(), String> {
    let balances = ledgers.entry(ledger.clone()).or_default();
    add_material_balance(balances, kind, amount)
}

fn remove_material_balance(
    balances: &mut BTreeMap<String, i64>,
    kind: &str,
    amount: i64,
) -> Result<(), String> {
    if amount < 0 {
        return Err(format!("negative material amount not allowed: {amount}"));
    }
    let current = balances.get(kind).copied().unwrap_or(0);
    if current < amount {
        return Err(format!(
            "insufficient material {kind}: requested={amount} available={current}"
        ));
    }
    let next = current - amount;
    if next == 0 {
        balances.remove(kind);
    } else {
        balances.insert(kind.to_string(), next);
    }
    Ok(())
}

fn remove_material_balance_for_ledger(
    ledgers: &mut BTreeMap<MaterialLedgerId, BTreeMap<String, i64>>,
    ledger: &MaterialLedgerId,
    kind: &str,
    amount: i64,
) -> Result<(), String> {
    let balances = ledgers.entry(ledger.clone()).or_default();
    remove_material_balance(balances, kind, amount)
}

fn sync_legacy_world_materials(
    ledgers: &BTreeMap<MaterialLedgerId, BTreeMap<String, i64>>,
    legacy_world_materials: &mut BTreeMap<String, i64>,
) {
    let world_materials = ledgers
        .get(&MaterialLedgerId::world())
        .cloned()
        .unwrap_or_default();
    *legacy_world_materials = world_materials;
}

fn remove_resource_balance(
    balances: &mut BTreeMap<ResourceKind, i64>,
    kind: ResourceKind,
    amount: i64,
) -> Result<(), String> {
    if amount < 0 {
        return Err(format!("negative resource amount not allowed: {amount}"));
    }
    let current = balances.get(&kind).copied().unwrap_or(0);
    if current < amount {
        return Err(format!(
            "insufficient resource {:?}: requested={amount} available={current}",
            kind
        ));
    }
    let next = current - amount;
    if next == 0 {
        balances.remove(&kind);
    } else {
        balances.insert(kind, next);
    }
    Ok(())
}

fn remove_node_power_credits(
    balances: &mut BTreeMap<String, NodeAssetBalance>,
    node_id: &str,
    amount: u64,
) -> Result<(), String> {
    let Some(balance) = balances.get_mut(node_id) else {
        return Err(format!("node balance not found: {node_id}"));
    };
    if balance.power_credit_balance < amount {
        return Err(format!(
            "insufficient power credits: balance={} burn={}",
            balance.power_credit_balance, amount
        ));
    }
    balance.power_credit_balance -= amount;
    balance.total_burned_credits = balance.total_burned_credits.saturating_add(amount);
    Ok(())
}

fn verify_reward_mint_record_signature_with_state(
    state: &WorldState,
    record: &NodeRewardMintRecord,
) -> Result<(), String> {
    let signer_public_key = state
        .node_identity_bindings
        .get(record.signer_node_id.as_str())
        .map(String::as_str)
        .ok_or_else(|| {
            format!(
                "reward mint signer identity is not bound: {}",
                record.signer_node_id
            )
        })?;
    if record
        .signature
        .starts_with(REWARD_MINT_SIGNATURE_V2_PREFIX)
    {
        return verify_reward_mint_signature_v2(
            record.signature.as_str(),
            record.epoch_index,
            record.node_id.as_str(),
            record.source_awarded_points,
            record.minted_power_credits,
            record.settlement_hash.as_str(),
            record.signer_node_id.as_str(),
            signer_public_key,
        );
    }
    if record
        .signature
        .starts_with(REWARD_MINT_SIGNATURE_V1_PREFIX)
    {
        if !state
            .reward_signature_governance_policy
            .allow_mintsig_v1_fallback
        {
            return Err("mintsig:v1 is disabled by governance policy".to_string());
        }
        let expected_signature = reward_mint_signature_v1(
            record.epoch_index,
            record.node_id.as_str(),
            record.source_awarded_points,
            record.minted_power_credits,
            record.settlement_hash.as_str(),
            record.signer_node_id.as_str(),
            signer_public_key,
        );
        if expected_signature != record.signature {
            return Err(format!(
                "reward mint signature mismatch for node {} at epoch {}",
                record.node_id, record.epoch_index
            ));
        }
        return Ok(());
    }
    Err(format!(
        "unsupported reward mint signature version for node {} at epoch {}",
        record.node_id, record.epoch_index
    ))
}

fn ensure_system_order_budget_caps_for_epoch(
    report: &EpochSettlementReport,
    budget: &mut SystemOrderPoolBudget,
) {
    if !budget.node_credit_caps.is_empty() {
        return;
    }
    if budget.total_credit_budget == 0 || report.settlements.is_empty() {
        return;
    }

    let total_awarded_points = report
        .settlements
        .iter()
        .map(|settlement| settlement.awarded_points)
        .sum::<u64>();
    if total_awarded_points == 0 {
        return;
    }

    let mut distributed = 0_u64;
    for settlement in &report.settlements {
        let cap = budget
            .total_credit_budget
            .saturating_mul(settlement.awarded_points)
            / total_awarded_points;
        distributed = distributed.saturating_add(cap);
        budget
            .node_credit_caps
            .insert(settlement.node_id.clone(), cap);
    }

    let mut remainder = budget.total_credit_budget.saturating_sub(distributed);
    if remainder == 0 {
        return;
    }
    let mut ranked = report
        .settlements
        .iter()
        .map(|settlement| (settlement.node_id.as_str(), settlement.awarded_points))
        .collect::<Vec<_>>();
    ranked.sort_by(|(a_node_id, a_points), (b_node_id, b_points)| {
        b_points
            .cmp(a_points)
            .then_with(|| a_node_id.cmp(b_node_id))
    });
    let mut index = 0_usize;
    while remainder > 0 && !ranked.is_empty() {
        let node_id = ranked[index % ranked.len()].0;
        if let Some(cap) = budget.node_credit_caps.get_mut(node_id) {
            *cap = cap.saturating_add(1);
            remainder -= 1;
        }
        index = index.saturating_add(1);
    }
}
