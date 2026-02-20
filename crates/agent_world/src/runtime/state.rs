//! World state management.

use crate::models::AgentState;
use crate::simulator::{ModuleInstallTarget, ResourceKind};
use agent_world_wasm_abi::{FactoryModuleSpec, MaterialStack};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::agent_cell::AgentCell;
use super::error::WorldError;
use super::events::DomainEvent;
use super::gameplay_state::{
    AllianceState, CrisisState, CrisisStatus, EconomicContractState, EconomicContractStatus,
    GameplayPolicyState, GovernanceProposalState, GovernanceProposalStatus,
    GovernanceVoteBallotState, GovernanceVoteState, MetaProgressState, WarState,
};
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

fn default_next_module_instance_id() -> u64 {
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

/// Installed module instance tracked independently from global module_id activation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleInstanceState {
    pub instance_id: String,
    pub module_id: String,
    pub module_version: String,
    #[serde(default)]
    pub wasm_hash: String,
    pub owner_agent_id: String,
    #[serde(default)]
    pub install_target: ModuleInstallTarget,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub installed_at: WorldTime,
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
    pub alliances: BTreeMap<String, AllianceState>,
    #[serde(default)]
    pub gameplay_policy: GameplayPolicyState,
    #[serde(default)]
    pub economic_contracts: BTreeMap<String, EconomicContractState>,
    #[serde(default)]
    pub reputation_scores: BTreeMap<String, i64>,
    #[serde(default)]
    pub wars: BTreeMap<String, WarState>,
    #[serde(default)]
    pub governance_votes: BTreeMap<String, GovernanceVoteState>,
    #[serde(default)]
    pub governance_proposals: BTreeMap<String, GovernanceProposalState>,
    #[serde(default)]
    pub crises: BTreeMap<String, CrisisState>,
    #[serde(default)]
    pub meta_progress: BTreeMap<String, MetaProgressState>,
    #[serde(default)]
    pub module_states: BTreeMap<String, Vec<u8>>,
    #[serde(default)]
    pub module_artifact_owners: BTreeMap<String, String>,
    #[serde(default)]
    pub module_artifact_listings: BTreeMap<String, ModuleArtifactListingState>,
    #[serde(default)]
    pub module_artifact_bids: BTreeMap<String, Vec<ModuleArtifactBidState>>,
    #[serde(default)]
    pub module_instances: BTreeMap<String, ModuleInstanceState>,
    #[serde(default)]
    pub installed_module_targets: BTreeMap<String, ModuleInstallTarget>,
    #[serde(default = "default_next_module_instance_id")]
    pub next_module_instance_id: u64,
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
            alliances: BTreeMap::new(),
            gameplay_policy: GameplayPolicyState::default(),
            economic_contracts: BTreeMap::new(),
            reputation_scores: BTreeMap::new(),
            wars: BTreeMap::new(),
            governance_votes: BTreeMap::new(),
            governance_proposals: BTreeMap::new(),
            crises: BTreeMap::new(),
            meta_progress: BTreeMap::new(),
            module_states: BTreeMap::new(),
            module_artifact_owners: BTreeMap::new(),
            module_artifact_listings: BTreeMap::new(),
            module_artifact_bids: BTreeMap::new(),
            module_instances: BTreeMap::new(),
            installed_module_targets: BTreeMap::new(),
            next_module_instance_id: default_next_module_instance_id(),
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
            DomainEvent::GameplayPolicyUpdated {
                operator_agent_id,
                electricity_tax_bps,
                data_tax_bps,
                max_open_contracts_per_agent,
                blocked_agents,
            } => {
                if !self.agents.contains_key(operator_agent_id) {
                    return Err(WorldError::AgentNotFound {
                        agent_id: operator_agent_id.clone(),
                    });
                }
                let mut normalized_blocked_agents = blocked_agents
                    .iter()
                    .filter_map(|value| {
                        let normalized = value.trim();
                        if normalized.is_empty() {
                            None
                        } else {
                            Some(normalized.to_string())
                        }
                    })
                    .collect::<Vec<_>>();
                normalized_blocked_agents.sort();
                normalized_blocked_agents.dedup();
                self.gameplay_policy = GameplayPolicyState {
                    electricity_tax_bps: *electricity_tax_bps,
                    data_tax_bps: *data_tax_bps,
                    max_open_contracts_per_agent: *max_open_contracts_per_agent,
                    blocked_agents: normalized_blocked_agents,
                    updated_at: now,
                };
                if let Some(cell) = self.agents.get_mut(operator_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::EconomicContractOpened {
                creator_agent_id,
                contract_id,
                counterparty_agent_id,
                settlement_kind,
                settlement_amount,
                reputation_stake,
                expires_at,
                description,
            } => {
                if !self.agents.contains_key(creator_agent_id) {
                    return Err(WorldError::AgentNotFound {
                        agent_id: creator_agent_id.clone(),
                    });
                }
                if !self.agents.contains_key(counterparty_agent_id) {
                    return Err(WorldError::AgentNotFound {
                        agent_id: counterparty_agent_id.clone(),
                    });
                }
                self.economic_contracts.insert(
                    contract_id.clone(),
                    EconomicContractState {
                        contract_id: contract_id.clone(),
                        creator_agent_id: creator_agent_id.clone(),
                        counterparty_agent_id: counterparty_agent_id.clone(),
                        settlement_kind: *settlement_kind,
                        settlement_amount: *settlement_amount,
                        reputation_stake: *reputation_stake,
                        expires_at: *expires_at,
                        description: description.clone(),
                        status: EconomicContractStatus::Open,
                        accepted_at: None,
                        settled_at: None,
                        settlement_success: None,
                        transfer_amount: 0,
                        tax_amount: 0,
                        settlement_notes: None,
                    },
                );
                if let Some(cell) = self.agents.get_mut(creator_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::EconomicContractAccepted {
                accepter_agent_id,
                contract_id,
            } => {
                let contract = self
                    .economic_contracts
                    .get_mut(contract_id)
                    .ok_or_else(|| WorldError::ResourceBalanceInvalid {
                        reason: format!("economic contract not found: {contract_id}"),
                    })?;
                if contract.status != EconomicContractStatus::Open {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "economic contract status invalid for acceptance: {:?}",
                            contract.status
                        ),
                    });
                }
                if contract.counterparty_agent_id != *accepter_agent_id {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "economic contract accepter mismatch expected={} actual={}",
                            contract.counterparty_agent_id, accepter_agent_id
                        ),
                    });
                }
                contract.status = EconomicContractStatus::Accepted;
                contract.accepted_at = Some(now);
                if let Some(cell) = self.agents.get_mut(accepter_agent_id) {
                    cell.last_active = now;
                } else {
                    return Err(WorldError::AgentNotFound {
                        agent_id: accepter_agent_id.clone(),
                    });
                }
            }
            DomainEvent::EconomicContractSettled {
                operator_agent_id,
                contract_id,
                success,
                transfer_amount,
                tax_amount,
                notes,
                creator_reputation_delta,
                counterparty_reputation_delta,
            } => {
                let (creator_agent_id, counterparty_agent_id, settlement_kind, status) = {
                    let contract = self.economic_contracts.get(contract_id).ok_or_else(|| {
                        WorldError::ResourceBalanceInvalid {
                            reason: format!("economic contract not found: {contract_id}"),
                        }
                    })?;
                    (
                        contract.creator_agent_id.clone(),
                        contract.counterparty_agent_id.clone(),
                        contract.settlement_kind,
                        contract.status,
                    )
                };
                if status != EconomicContractStatus::Accepted {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "economic contract status invalid for settlement: {:?}",
                            status
                        ),
                    });
                }
                if *success {
                    if *transfer_amount <= 0 {
                        return Err(WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "economic contract settlement transfer must be > 0, got {}",
                                transfer_amount
                            ),
                        });
                    }
                    if *tax_amount < 0 {
                        return Err(WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "economic contract settlement tax must be >= 0, got {}",
                                tax_amount
                            ),
                        });
                    }
                    let debit_total = transfer_amount.saturating_add(*tax_amount);
                    let creator_cell = self.agents.get_mut(&creator_agent_id).ok_or_else(|| {
                        WorldError::AgentNotFound {
                            agent_id: creator_agent_id.clone(),
                        }
                    })?;
                    creator_cell
                        .state
                        .resources
                        .remove(settlement_kind, debit_total)
                        .map_err(|err| WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "economic contract settlement debit failed agent={} kind={:?} amount={} err={:?}",
                                creator_agent_id, settlement_kind, debit_total, err
                            ),
                        })?;

                    let counterparty_cell = self
                        .agents
                        .get_mut(&counterparty_agent_id)
                        .ok_or_else(|| WorldError::AgentNotFound {
                            agent_id: counterparty_agent_id.clone(),
                        })?;
                    counterparty_cell
                        .state
                        .resources
                        .add(settlement_kind, *transfer_amount)
                        .map_err(|err| WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "economic contract settlement credit failed agent={} kind={:?} amount={} err={:?}",
                                counterparty_agent_id, settlement_kind, transfer_amount, err
                            ),
                        })?;
                    let treasury = self.resources.entry(settlement_kind).or_insert(0);
                    *treasury = treasury.saturating_add(*tax_amount);
                }

                if *creator_reputation_delta != 0 {
                    let score = self
                        .reputation_scores
                        .entry(creator_agent_id.clone())
                        .or_insert(0);
                    *score = score.saturating_add(*creator_reputation_delta);
                }
                if *counterparty_reputation_delta != 0 {
                    let score = self
                        .reputation_scores
                        .entry(counterparty_agent_id.clone())
                        .or_insert(0);
                    *score = score.saturating_add(*counterparty_reputation_delta);
                }

                let contract = self
                    .economic_contracts
                    .get_mut(contract_id)
                    .ok_or_else(|| WorldError::ResourceBalanceInvalid {
                        reason: format!("economic contract not found: {contract_id}"),
                    })?;
                contract.status = EconomicContractStatus::Settled;
                contract.settled_at = Some(now);
                contract.settlement_success = Some(*success);
                contract.transfer_amount = *transfer_amount;
                contract.tax_amount = *tax_amount;
                contract.settlement_notes = Some(notes.clone());

                if let Some(cell) = self.agents.get_mut(operator_agent_id) {
                    cell.last_active = now;
                } else {
                    return Err(WorldError::AgentNotFound {
                        agent_id: operator_agent_id.clone(),
                    });
                }
            }
            DomainEvent::EconomicContractExpired {
                contract_id,
                creator_agent_id,
                counterparty_agent_id,
                creator_reputation_delta,
                counterparty_reputation_delta,
            } => {
                let contract = self
                    .economic_contracts
                    .get_mut(contract_id)
                    .ok_or_else(|| WorldError::ResourceBalanceInvalid {
                        reason: format!("economic contract not found: {contract_id}"),
                    })?;
                match contract.status {
                    EconomicContractStatus::Open | EconomicContractStatus::Accepted => {
                        contract.status = EconomicContractStatus::Expired;
                        contract.settled_at = Some(now);
                        contract.settlement_success = Some(false);
                        contract.transfer_amount = 0;
                        contract.tax_amount = 0;
                        contract.settlement_notes =
                            Some("auto expired by gameplay lifecycle".to_string());
                    }
                    EconomicContractStatus::Settled | EconomicContractStatus::Expired => {
                        return Err(WorldError::ResourceBalanceInvalid {
                            reason: format!(
                                "economic contract already finalized before expiry: {}",
                                contract_id
                            ),
                        });
                    }
                }
                if *creator_reputation_delta != 0 {
                    let score = self
                        .reputation_scores
                        .entry(creator_agent_id.clone())
                        .or_insert(0);
                    *score = score.saturating_add(*creator_reputation_delta);
                }
                if *counterparty_reputation_delta != 0 {
                    let score = self
                        .reputation_scores
                        .entry(counterparty_agent_id.clone())
                        .or_insert(0);
                    *score = score.saturating_add(*counterparty_reputation_delta);
                }
                if let Some(cell) = self.agents.get_mut(creator_agent_id) {
                    cell.last_active = now;
                }
                if let Some(cell) = self.agents.get_mut(counterparty_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::AllianceFormed {
                proposer_agent_id,
                alliance_id,
                members,
                charter,
            } => {
                for member in members {
                    if !self.agents.contains_key(member) {
                        return Err(WorldError::AgentNotFound {
                            agent_id: member.clone(),
                        });
                    }
                }
                self.alliances.insert(
                    alliance_id.clone(),
                    AllianceState {
                        alliance_id: alliance_id.clone(),
                        members: members.clone(),
                        charter: charter.clone(),
                        formed_by_agent_id: proposer_agent_id.clone(),
                        formed_at: now,
                    },
                );
                if let Some(cell) = self.agents.get_mut(proposer_agent_id) {
                    cell.last_active = now;
                } else {
                    return Err(WorldError::AgentNotFound {
                        agent_id: proposer_agent_id.clone(),
                    });
                }
                for member in members {
                    if let Some(cell) = self.agents.get_mut(member) {
                        cell.last_active = now;
                    }
                }
            }
            DomainEvent::WarDeclared {
                initiator_agent_id,
                war_id,
                aggressor_alliance_id,
                defender_alliance_id,
                objective,
                intensity,
            } => {
                if !self.alliances.contains_key(aggressor_alliance_id) {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "war declare aggressor alliance missing: {}",
                            aggressor_alliance_id
                        ),
                    });
                }
                if !self.alliances.contains_key(defender_alliance_id) {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "war declare defender alliance missing: {}",
                            defender_alliance_id
                        ),
                    });
                }
                self.wars.insert(
                    war_id.clone(),
                    WarState {
                        war_id: war_id.clone(),
                        initiator_agent_id: initiator_agent_id.clone(),
                        aggressor_alliance_id: aggressor_alliance_id.clone(),
                        defender_alliance_id: defender_alliance_id.clone(),
                        objective: objective.clone(),
                        intensity: *intensity,
                        active: true,
                        max_duration_ticks: 6_u64.saturating_add(u64::from(*intensity) * 2),
                        aggressor_score: 0,
                        defender_score: 0,
                        concluded_at: None,
                        winner_alliance_id: None,
                        settlement_summary: None,
                        declared_at: now,
                    },
                );
                if let Some(cell) = self.agents.get_mut(initiator_agent_id) {
                    cell.last_active = now;
                } else {
                    return Err(WorldError::AgentNotFound {
                        agent_id: initiator_agent_id.clone(),
                    });
                }
            }
            DomainEvent::WarConcluded {
                war_id,
                winner_alliance_id,
                aggressor_score,
                defender_score,
                summary,
            } => {
                let Some(state) = self.wars.get_mut(war_id) else {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!("war not found for conclusion: {war_id}"),
                    });
                };
                state.active = false;
                state.aggressor_score = *aggressor_score;
                state.defender_score = *defender_score;
                state.concluded_at = Some(now);
                state.winner_alliance_id = Some(winner_alliance_id.clone());
                state.settlement_summary = Some(summary.clone());
            }
            DomainEvent::GovernanceProposalOpened {
                proposer_agent_id,
                proposal_key,
                title,
                description,
                options,
                voting_window_ticks,
                closes_at,
                quorum_weight,
                pass_threshold_bps,
            } => {
                if !self.agents.contains_key(proposer_agent_id) {
                    return Err(WorldError::AgentNotFound {
                        agent_id: proposer_agent_id.clone(),
                    });
                }
                self.governance_proposals.insert(
                    proposal_key.clone(),
                    GovernanceProposalState {
                        proposal_key: proposal_key.clone(),
                        proposer_agent_id: proposer_agent_id.clone(),
                        title: title.clone(),
                        description: description.clone(),
                        options: options.clone(),
                        voting_window_ticks: *voting_window_ticks,
                        quorum_weight: *quorum_weight,
                        pass_threshold_bps: *pass_threshold_bps,
                        opened_at: now,
                        closes_at: *closes_at,
                        status: GovernanceProposalStatus::Open,
                        finalized_at: None,
                        winning_option: None,
                        winning_weight: 0,
                        total_weight_at_finalize: 0,
                    },
                );
                self.governance_votes
                    .entry(proposal_key.clone())
                    .or_insert_with(|| GovernanceVoteState {
                        proposal_key: proposal_key.clone(),
                        votes_by_agent: BTreeMap::new(),
                        tallies: BTreeMap::new(),
                        total_weight: 0,
                        last_updated_at: now,
                    });
                if let Some(cell) = self.agents.get_mut(proposer_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::GovernanceVoteCast {
                voter_agent_id,
                proposal_key,
                option,
                weight,
            } => {
                if !self.agents.contains_key(voter_agent_id) {
                    return Err(WorldError::AgentNotFound {
                        agent_id: voter_agent_id.clone(),
                    });
                }
                let Some(proposal) = self.governance_proposals.get(proposal_key) else {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "governance vote references unknown proposal: {proposal_key}"
                        ),
                    });
                };
                if proposal.status != GovernanceProposalStatus::Open {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!("governance proposal is not open: {proposal_key}"),
                    });
                }
                if now > proposal.closes_at {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!(
                            "governance proposal already closed at {}: {}",
                            proposal.closes_at, proposal_key
                        ),
                    });
                }

                let state = self
                    .governance_votes
                    .entry(proposal_key.clone())
                    .or_insert_with(|| GovernanceVoteState {
                        proposal_key: proposal_key.clone(),
                        votes_by_agent: BTreeMap::new(),
                        tallies: BTreeMap::new(),
                        total_weight: 0,
                        last_updated_at: now,
                    });

                if let Some(previous_ballot) = state.votes_by_agent.get(voter_agent_id).cloned() {
                    let previous_weight = u64::from(previous_ballot.weight);
                    state.total_weight = state.total_weight.saturating_sub(previous_weight);
                    if let Some(entry) = state.tallies.get_mut(&previous_ballot.option) {
                        *entry = entry.saturating_sub(previous_weight);
                        if *entry == 0 {
                            state.tallies.remove(&previous_ballot.option);
                        }
                    }
                }

                state.votes_by_agent.insert(
                    voter_agent_id.clone(),
                    GovernanceVoteBallotState {
                        option: option.clone(),
                        weight: *weight,
                        voted_at: now,
                    },
                );
                let vote_weight = u64::from(*weight);
                let current_tally = state.tallies.get(option).copied().unwrap_or(0);
                *state.tallies.entry(option.clone()).or_insert(0) =
                    current_tally.saturating_add(vote_weight);
                state.total_weight = state.total_weight.saturating_add(vote_weight);
                state.last_updated_at = now;

                if let Some(cell) = self.agents.get_mut(voter_agent_id) {
                    cell.last_active = now;
                }
            }
            DomainEvent::GovernanceProposalFinalized {
                proposal_key,
                winning_option,
                winning_weight,
                total_weight,
                passed,
            } => {
                let Some(state) = self.governance_proposals.get_mut(proposal_key) else {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!("governance proposal missing: {proposal_key}"),
                    });
                };
                state.status = if *passed {
                    GovernanceProposalStatus::Passed
                } else {
                    GovernanceProposalStatus::Rejected
                };
                state.finalized_at = Some(now);
                state.winning_option = winning_option.clone();
                state.winning_weight = *winning_weight;
                state.total_weight_at_finalize = *total_weight;
                if let Some(vote_state) = self.governance_votes.get_mut(proposal_key) {
                    vote_state.last_updated_at = now;
                }
            }
            DomainEvent::CrisisSpawned {
                crisis_id,
                kind,
                severity,
                expires_at,
            } => {
                self.crises.insert(
                    crisis_id.clone(),
                    CrisisState {
                        crisis_id: crisis_id.clone(),
                        kind: kind.clone(),
                        severity: *severity,
                        status: CrisisStatus::Active,
                        opened_at: now,
                        expires_at: *expires_at,
                        resolver_agent_id: None,
                        strategy: None,
                        success: None,
                        impact: 0,
                        resolved_at: None,
                    },
                );
            }
            DomainEvent::CrisisResolved {
                resolver_agent_id,
                crisis_id,
                strategy,
                success,
                impact,
            } => {
                let entry = self
                    .crises
                    .entry(crisis_id.clone())
                    .or_insert_with(|| CrisisState {
                        crisis_id: crisis_id.clone(),
                        kind: "legacy".to_string(),
                        severity: 1,
                        status: CrisisStatus::Resolved,
                        opened_at: now,
                        expires_at: now,
                        resolver_agent_id: None,
                        strategy: None,
                        success: None,
                        impact: 0,
                        resolved_at: None,
                    });
                entry.status = CrisisStatus::Resolved;
                entry.resolver_agent_id = Some(resolver_agent_id.clone());
                entry.strategy = Some(strategy.clone());
                entry.success = Some(*success);
                entry.impact = *impact;
                entry.resolved_at = Some(now);
                if let Some(cell) = self.agents.get_mut(resolver_agent_id) {
                    cell.last_active = now;
                } else {
                    return Err(WorldError::AgentNotFound {
                        agent_id: resolver_agent_id.clone(),
                    });
                }
            }
            DomainEvent::CrisisTimedOut {
                crisis_id,
                penalty_impact,
            } => {
                let Some(entry) = self.crises.get_mut(crisis_id) else {
                    return Err(WorldError::ResourceBalanceInvalid {
                        reason: format!("crisis not found for timeout: {crisis_id}"),
                    });
                };
                entry.status = CrisisStatus::TimedOut;
                entry.success = Some(false);
                entry.impact = *penalty_impact;
                entry.resolved_at = Some(now);
            }
            DomainEvent::MetaProgressGranted {
                operator_agent_id,
                target_agent_id,
                track,
                points,
                achievement_id,
            } => {
                if !self.agents.contains_key(operator_agent_id) {
                    return Err(WorldError::AgentNotFound {
                        agent_id: operator_agent_id.clone(),
                    });
                }
                if !self.agents.contains_key(target_agent_id) {
                    return Err(WorldError::AgentNotFound {
                        agent_id: target_agent_id.clone(),
                    });
                }

                let progress = self
                    .meta_progress
                    .entry(target_agent_id.clone())
                    .or_insert_with(|| MetaProgressState {
                        agent_id: target_agent_id.clone(),
                        track_points: BTreeMap::new(),
                        total_points: 0,
                        achievements: Vec::new(),
                        unlocked_tiers: BTreeMap::new(),
                        last_granted_at: now,
                    });
                let next_track_points = progress
                    .track_points
                    .get(track)
                    .copied()
                    .unwrap_or(0)
                    .saturating_add(*points);
                progress
                    .track_points
                    .insert(track.clone(), next_track_points);
                progress.total_points = progress.total_points.saturating_add(*points);
                progress.last_granted_at = now;
                if let Some(achievement_id) = achievement_id {
                    if !progress
                        .achievements
                        .iter()
                        .any(|item| item == achievement_id)
                    {
                        progress.achievements.push(achievement_id.clone());
                        progress.achievements.sort();
                    }
                }
                unlock_meta_track_tiers(track, next_track_points, progress);

                if let Some(cell) = self.agents.get_mut(operator_agent_id) {
                    cell.last_active = now;
                }
                if let Some(cell) = self.agents.get_mut(target_agent_id) {
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

fn unlock_meta_track_tiers(track: &str, track_points: i64, progress: &mut MetaProgressState) {
    const META_TIER_THRESHOLDS: [(&str, i64); 3] = [("bronze", 20), ("silver", 50), ("gold", 100)];
    let unlocked_tiers = progress
        .unlocked_tiers
        .entry(track.to_string())
        .or_default();
    for (tier, threshold) in META_TIER_THRESHOLDS {
        if track_points < threshold {
            continue;
        }
        if !unlocked_tiers.iter().any(|value| value == tier) {
            unlocked_tiers.push(tier.to_string());
        }
        let achievement_id = format!("tier.{track}.{tier}");
        if !progress
            .achievements
            .iter()
            .any(|value| value == &achievement_id)
        {
            progress.achievements.push(achievement_id);
        }
    }
    unlocked_tiers.sort();
    unlocked_tiers.dedup();
    progress.achievements.sort();
    progress.achievements.dedup();
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
