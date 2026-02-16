//! World state management.

use crate::models::AgentState;
use crate::simulator::ResourceKind;
use agent_world_wasm_abi::{FactoryModuleSpec, MaterialStack};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::agent_cell::AgentCell;
use super::error::WorldError;
use super::events::DomainEvent;
use super::reward_asset::{
    NodeAssetBalance, NodeRewardMintRecord, ProtocolPowerReserve, RewardAssetConfig,
};
use super::types::{ActionId, MaterialLedgerId, WorldTime};

fn default_world_material_ledger() -> MaterialLedgerId {
    MaterialLedgerId::world()
}

fn default_material_ledgers() -> BTreeMap<MaterialLedgerId, BTreeMap<String, i64>> {
    let mut ledgers = BTreeMap::new();
    ledgers.insert(MaterialLedgerId::world(), BTreeMap::new());
    ledgers
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
    pub reward_asset_config: RewardAssetConfig,
    #[serde(default)]
    pub node_asset_balances: BTreeMap<String, NodeAssetBalance>,
    #[serde(default)]
    pub protocol_power_reserve: ProtocolPowerReserve,
    #[serde(default)]
    pub reward_mint_records: Vec<NodeRewardMintRecord>,
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
            reward_asset_config: RewardAssetConfig::default(),
            node_asset_balances: BTreeMap::new(),
            protocol_power_reserve: ProtocolPowerReserve::default(),
            reward_mint_records: Vec::new(),
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
