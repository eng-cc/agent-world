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
    GovernanceVoteBallotState, GovernanceVoteState, MetaProgressState, WarParticipantOutcome,
    WarState,
};
use super::node_points::EpochSettlementReport;
use super::reward_asset::{
    reward_mint_signature_v1, verify_reward_mint_signature_v2, NodeAssetBalance,
    NodeRewardMintRecord, ProtocolPowerReserve, RewardAssetConfig, RewardSignatureGovernancePolicy,
    SystemOrderPoolBudget, REWARD_MINT_SIGNATURE_V1_PREFIX, REWARD_MINT_SIGNATURE_V2_PREFIX,
};
use super::types::{ActionId, MaterialLedgerId, WorldTime};
use super::util::hash_json;

mod apply_domain_event_core;
mod apply_domain_event_gameplay;
mod apply_domain_event_governance_meta;

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

const ALLIANCE_MIN_MEMBER_COUNT: usize = 2;

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
            DomainEvent::AgentRegistered { .. }
            | DomainEvent::AgentMoved { .. }
            | DomainEvent::ActionRejected { .. }
            | DomainEvent::Observation { .. }
            | DomainEvent::BodyAttributesUpdated { .. }
            | DomainEvent::BodyAttributesRejected { .. }
            | DomainEvent::BodyInterfaceExpanded { .. }
            | DomainEvent::BodyInterfaceExpandRejected { .. }
            | DomainEvent::ModuleArtifactDeployed { .. }
            | DomainEvent::ModuleInstalled { .. }
            | DomainEvent::ModuleUpgraded { .. }
            | DomainEvent::ModuleArtifactListed { .. }
            | DomainEvent::ModuleArtifactDelisted { .. }
            | DomainEvent::ModuleArtifactDestroyed { .. }
            | DomainEvent::ModuleArtifactBidPlaced { .. }
            | DomainEvent::ModuleArtifactBidCancelled { .. }
            | DomainEvent::ModuleArtifactSaleCompleted { .. }
            | DomainEvent::ResourceTransferred { .. }
            | DomainEvent::PowerRedeemed { .. }
            | DomainEvent::PowerRedeemRejected { .. }
            | DomainEvent::NodePointsSettlementApplied { .. }
            | DomainEvent::MaterialTransferred { .. }
            | DomainEvent::MaterialTransitStarted { .. }
            | DomainEvent::MaterialTransitCompleted { .. }
            | DomainEvent::FactoryBuildStarted { .. }
            | DomainEvent::FactoryBuilt { .. }
            | DomainEvent::RecipeStarted { .. }
            | DomainEvent::RecipeCompleted { .. } => self.apply_domain_event_core(event, now)?,
            DomainEvent::GameplayPolicyUpdated { .. }
            | DomainEvent::EconomicContractOpened { .. }
            | DomainEvent::EconomicContractAccepted { .. }
            | DomainEvent::EconomicContractSettled { .. }
            | DomainEvent::EconomicContractExpired { .. }
            | DomainEvent::AllianceFormed { .. }
            | DomainEvent::AllianceJoined { .. }
            | DomainEvent::AllianceLeft { .. }
            | DomainEvent::AllianceDissolved { .. }
            | DomainEvent::WarDeclared { .. }
            | DomainEvent::WarConcluded { .. } => self.apply_domain_event_gameplay(event, now)?,
            DomainEvent::GovernanceProposalOpened { .. }
            | DomainEvent::GovernanceVoteCast { .. }
            | DomainEvent::GovernanceProposalFinalized { .. }
            | DomainEvent::CrisisSpawned { .. }
            | DomainEvent::CrisisResolved { .. }
            | DomainEvent::CrisisTimedOut { .. }
            | DomainEvent::MetaProgressGranted { .. }
            | DomainEvent::ProductValidated { .. } => {
                self.apply_domain_event_governance_meta(event, now)?
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

fn apply_war_participant_outcomes(
    agents: &mut BTreeMap<String, AgentCell>,
    reputation_scores: &mut BTreeMap<String, i64>,
    outcomes: &[WarParticipantOutcome],
    now: WorldTime,
) -> Result<(), WorldError> {
    for outcome in outcomes {
        let Some(cell) = agents.get_mut(outcome.agent_id.as_str()) else {
            return Err(WorldError::AgentNotFound {
                agent_id: outcome.agent_id.clone(),
            });
        };

        apply_agent_resource_delta(
            cell,
            ResourceKind::Electricity,
            outcome.electricity_delta,
            outcome.agent_id.as_str(),
            "war electricity outcome",
        )?;
        apply_agent_resource_delta(
            cell,
            ResourceKind::Data,
            outcome.data_delta,
            outcome.agent_id.as_str(),
            "war data outcome",
        )?;
        cell.last_active = now;

        if outcome.reputation_delta != 0 {
            let score = reputation_scores
                .entry(outcome.agent_id.clone())
                .or_insert(0);
            *score = score.saturating_add(outcome.reputation_delta);
        }
    }
    Ok(())
}

fn apply_agent_resource_delta(
    cell: &mut AgentCell,
    kind: ResourceKind,
    delta: i64,
    agent_id: &str,
    context: &str,
) -> Result<(), WorldError> {
    if delta == 0 {
        return Ok(());
    }
    if delta > 0 {
        return cell.state.resources.add(kind, delta).map_err(|err| {
            WorldError::ResourceBalanceInvalid {
                reason: format!("{context} apply failed for {agent_id}: {err:?}"),
            }
        });
    }
    cell.state
        .resources
        .remove(kind, delta.saturating_abs())
        .map_err(|err| WorldError::ResourceBalanceInvalid {
            reason: format!("{context} apply failed for {agent_id}: {err:?}"),
        })
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
