use crate::geometry::{space_distance_cm, GeoPos};
use std::collections::BTreeMap;

use super::super::chunking::CHUNK_SIZE_X_CM;
use super::super::module_visual::ModuleVisualAnchor;
use super::super::power::{PlantStatus, PowerEvent, PowerPlant, PowerStorage};
use super::super::types::{
    Action, ElementBudgetError, FragmentElementKind, PowerOrderSide, ResourceKind, ResourceOwner,
    StockError, CM_PER_KM, PPM_BASE,
};
use super::super::world_model::{Agent, Factory, FragmentResourceError, Location, PowerOrderState};
use super::types::{ChunkGenerationCause, PowerOrderFill, RejectReason, WorldEventKind};
use super::WorldKernel;

#[derive(Debug, Clone, Copy)]
struct RecipePlan {
    required_factory_kind: &'static str,
    electricity_per_batch: i64,
    hardware_per_batch: i64,
    data_output_per_batch: i64,
    finished_product_id: &'static str,
    finished_product_units_per_batch: i64,
}

const LOCATION_ELECTRICITY_POOL_REMOVED_NOTE: &str = "location electricity pool removed";
const FACTORY_KIND_ASSEMBLER_MK1: &str = "factory.assembler.mk1";
const FACTORY_KIND_RADIATION_POWER_MK1: &str = "factory.power.radiation.mk1";

#[derive(Debug, Clone, Copy)]
struct PreparedPowerTransfer {
    loss: i64,
    quoted_price_per_pu: i64,
}

impl WorldKernel {
    pub(super) fn apply_action(&mut self, action: Action) -> WorldEventKind {
        match action {
            Action::RegisterLocation {
                location_id,
                name,
                pos,
                profile,
            } => {
                if self.model.locations.contains_key(&location_id) {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::LocationAlreadyExists { location_id },
                    };
                }
                if !self.config.space.contains(pos) {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::PositionOutOfBounds { pos },
                    };
                }
                if profile.radius_cm < 0 {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InvalidAmount {
                            amount: profile.radius_cm,
                        },
                    };
                }
                if profile.radiation_emission_per_tick < 0 {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InvalidAmount {
                            amount: profile.radiation_emission_per_tick,
                        },
                    };
                }
                let location = Location::new_with_profile(
                    location_id.clone(),
                    name.clone(),
                    pos,
                    profile.clone(),
                );
                self.model.locations.insert(location_id.clone(), location);
                WorldEventKind::LocationRegistered {
                    location_id,
                    name,
                    pos,
                    profile,
                }
            }
            Action::RegisterAgent {
                agent_id,
                location_id,
            } => {
                if self.model.agents.contains_key(&agent_id) {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::AgentAlreadyExists { agent_id },
                    };
                }
                let Some(location) = self.model.locations.get(&location_id) else {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::LocationNotFound { location_id },
                    };
                };
                let agent = Agent::new_with_power(
                    agent_id.clone(),
                    location_id.clone(),
                    location.pos,
                    &self.config.power,
                );
                self.model.agents.insert(agent_id.clone(), agent);
                WorldEventKind::AgentRegistered {
                    agent_id,
                    location_id,
                    pos: location.pos,
                }
            }
            Action::RegisterPowerPlant {
                facility_id,
                location_id,
                owner,
                capacity_per_tick,
                fuel_cost_per_pu,
                maintenance_cost,
                efficiency,
                degradation,
            } => {
                if self.model.power_plants.contains_key(&facility_id)
                    || self.model.power_storages.contains_key(&facility_id)
                {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::FacilityAlreadyExists { facility_id },
                    };
                }
                if !self.model.locations.contains_key(&location_id) {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::LocationNotFound { location_id },
                    };
                }
                if let Err(reason) = self.ensure_owner_exists(&owner) {
                    return WorldEventKind::ActionRejected { reason };
                }
                if capacity_per_tick < 0 {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InvalidAmount {
                            amount: capacity_per_tick,
                        },
                    };
                }
                if fuel_cost_per_pu < 0 {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InvalidAmount {
                            amount: fuel_cost_per_pu,
                        },
                    };
                }
                if maintenance_cost < 0 {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InvalidAmount {
                            amount: maintenance_cost,
                        },
                    };
                }
                let plant = PowerPlant {
                    id: facility_id.clone(),
                    location_id,
                    owner,
                    capacity_per_tick,
                    current_output: 0,
                    fuel_cost_per_pu,
                    maintenance_cost,
                    status: PlantStatus::Running,
                    efficiency,
                    degradation,
                };
                self.model.power_plants.insert(facility_id, plant.clone());
                WorldEventKind::Power(PowerEvent::PowerPlantRegistered { plant })
            }
            Action::RegisterPowerStorage {
                facility_id,
                location_id,
                owner,
                capacity,
                current_level,
                charge_efficiency,
                discharge_efficiency,
                max_charge_rate,
                max_discharge_rate,
            } => {
                if self.model.power_plants.contains_key(&facility_id)
                    || self.model.power_storages.contains_key(&facility_id)
                {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::FacilityAlreadyExists { facility_id },
                    };
                }
                if !self.model.locations.contains_key(&location_id) {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::LocationNotFound { location_id },
                    };
                }
                if let Err(reason) = self.ensure_owner_exists(&owner) {
                    return WorldEventKind::ActionRejected { reason };
                }
                if capacity < 0 {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InvalidAmount { amount: capacity },
                    };
                }
                if current_level < 0 || current_level > capacity {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InvalidAmount {
                            amount: current_level,
                        },
                    };
                }
                if max_charge_rate < 0 {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InvalidAmount {
                            amount: max_charge_rate,
                        },
                    };
                }
                if max_discharge_rate < 0 {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InvalidAmount {
                            amount: max_discharge_rate,
                        },
                    };
                }
                let storage = PowerStorage {
                    id: facility_id.clone(),
                    location_id,
                    owner,
                    capacity,
                    current_level,
                    charge_efficiency,
                    discharge_efficiency,
                    max_charge_rate,
                    max_discharge_rate,
                };
                self.model
                    .power_storages
                    .insert(facility_id, storage.clone());
                WorldEventKind::Power(PowerEvent::PowerStorageRegistered { storage })
            }
            Action::UpsertModuleVisualEntity { entity } => {
                let entity = entity.sanitized();
                if entity.entity_id.is_empty() || entity.module_id.is_empty() {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InvalidAmount { amount: 0 },
                    };
                }
                if let Err(reason) = self.ensure_module_visual_anchor_exists(&entity.anchor) {
                    return WorldEventKind::ActionRejected { reason };
                }
                self.model
                    .module_visual_entities
                    .insert(entity.entity_id.clone(), entity.clone());
                WorldEventKind::ModuleVisualEntityUpserted { entity }
            }
            Action::RemoveModuleVisualEntity { entity_id } => {
                let entity_id = entity_id.trim().to_string();
                if entity_id.is_empty() {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InvalidAmount { amount: 0 },
                    };
                }
                if self
                    .model
                    .module_visual_entities
                    .remove(&entity_id)
                    .is_none()
                {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::FacilityNotFound {
                            facility_id: entity_id,
                        },
                    };
                }
                WorldEventKind::ModuleVisualEntityRemoved { entity_id }
            }
            Action::DrawPower { .. } => WorldEventKind::ActionRejected {
                reason: RejectReason::RuleDenied {
                    notes: vec![LOCATION_ELECTRICITY_POOL_REMOVED_NOTE.to_string()],
                },
            },
            Action::StorePower { .. } => WorldEventKind::ActionRejected {
                reason: RejectReason::RuleDenied {
                    notes: vec![LOCATION_ELECTRICITY_POOL_REMOVED_NOTE.to_string()],
                },
            },
            Action::MoveAgent { agent_id, to } => {
                let to_pos = match self.model.locations.get(&to) {
                    Some(location) => location.pos,
                    None => {
                        return WorldEventKind::ActionRejected {
                            reason: RejectReason::LocationNotFound { location_id: to },
                        };
                    }
                };
                if let Err(reason) =
                    self.ensure_chunk_generated_at(to_pos, ChunkGenerationCause::Action)
                {
                    return WorldEventKind::ActionRejected { reason };
                }
                let Some(location) = self.model.locations.get(&to) else {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::LocationNotFound { location_id: to },
                    };
                };
                let Some(agent) = self.model.agents.get_mut(&agent_id) else {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::AgentNotFound { agent_id },
                    };
                };
                if agent.power.is_shutdown() {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::AgentShutdown { agent_id },
                    };
                }
                if agent.location_id == to {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::AgentAlreadyAtLocation {
                            agent_id,
                            location_id: to,
                        },
                    };
                }
                let from = agent.location_id.clone();
                let distance_cm = space_distance_cm(agent.pos, location.pos);
                let physics = &self.config.physics;
                let max_move_distance_cm = physics.max_move_distance_cm_per_tick;
                if max_move_distance_cm > 0 && distance_cm > max_move_distance_cm {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::MoveDistanceExceeded {
                            distance_cm,
                            max_distance_cm: max_move_distance_cm,
                        },
                    };
                }
                let max_move_speed_cm_per_s = physics.max_move_speed_cm_per_s;
                if max_move_speed_cm_per_s > 0 {
                    let time_step_s = physics.time_step_s.max(1);
                    let required_speed_cm_per_s =
                        (distance_cm + time_step_s - 1).saturating_div(time_step_s);
                    if required_speed_cm_per_s > max_move_speed_cm_per_s {
                        return WorldEventKind::ActionRejected {
                            reason: RejectReason::MoveSpeedExceeded {
                                required_speed_cm_per_s,
                                max_speed_cm_per_s: max_move_speed_cm_per_s,
                                time_step_s,
                            },
                        };
                    }
                }
                let electricity_cost = self.config.movement_cost(distance_cm);
                if electricity_cost > 0 {
                    let available = agent.resources.get(ResourceKind::Electricity);
                    if available < electricity_cost {
                        return WorldEventKind::ActionRejected {
                            reason: RejectReason::InsufficientResource {
                                owner: ResourceOwner::Agent {
                                    agent_id: agent.id.clone(),
                                },
                                kind: ResourceKind::Electricity,
                                requested: electricity_cost,
                                available,
                            },
                        };
                    }
                    if let Err(err) = agent
                        .resources
                        .remove(ResourceKind::Electricity, electricity_cost)
                    {
                        return WorldEventKind::ActionRejected {
                            reason: match err {
                                StockError::NegativeAmount { amount } => {
                                    RejectReason::InvalidAmount { amount }
                                }
                                StockError::Insufficient {
                                    requested,
                                    available,
                                    ..
                                } => RejectReason::InsufficientResource {
                                    owner: ResourceOwner::Agent {
                                        agent_id: agent.id.clone(),
                                    },
                                    kind: ResourceKind::Electricity,
                                    requested,
                                    available,
                                },
                            },
                        };
                    }
                }
                agent.location_id = to.clone();
                agent.pos = location.pos;
                WorldEventKind::AgentMoved {
                    agent_id,
                    from,
                    to,
                    distance_cm,
                    electricity_cost,
                }
            }
            Action::HarvestRadiation {
                agent_id,
                max_amount,
            } => {
                if max_amount <= 0 {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InvalidAmount { amount: max_amount },
                    };
                }
                let location_id = match self.model.agents.get(&agent_id) {
                    Some(agent) => agent.location_id.clone(),
                    None => {
                        return WorldEventKind::ActionRejected {
                            reason: RejectReason::AgentNotFound { agent_id },
                        };
                    }
                };
                let location_pos = match self.model.locations.get(&location_id) {
                    Some(location) => location.pos,
                    None => {
                        return WorldEventKind::ActionRejected {
                            reason: RejectReason::LocationNotFound { location_id },
                        };
                    }
                };
                if let Err(reason) =
                    self.ensure_chunk_generated_at(location_pos, ChunkGenerationCause::Action)
                {
                    return WorldEventKind::ActionRejected { reason };
                }
                let harvest_pos = match self.model.locations.get(&location_id) {
                    Some(location) => location.pos,
                    None => {
                        return WorldEventKind::ActionRejected {
                            reason: RejectReason::LocationNotFound { location_id },
                        };
                    }
                };
                let local_available = self.radiation_available_at(harvest_pos);
                if local_available <= 0 {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::RadiationUnavailable { location_id },
                    };
                }
                let physics = &self.config.physics;
                let mut available_for_harvest = local_available;
                if physics.max_harvest_per_tick > 0 {
                    available_for_harvest = available_for_harvest.min(physics.max_harvest_per_tick);
                }
                let mut harvested = max_amount.min(available_for_harvest);
                let Some(agent) = self.model.agents.get_mut(&agent_id) else {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::AgentNotFound { agent_id },
                    };
                };
                if physics.thermal_capacity > 0 && agent.thermal.heat > physics.thermal_capacity {
                    let heat = agent.thermal.heat;
                    let capacity = physics.thermal_capacity;
                    let ratio = (capacity as f64 / heat as f64).clamp(0.1, 1.0);
                    harvested = (harvested as f64 * ratio).floor() as i64;
                    if harvested <= 0 {
                        return WorldEventKind::ActionRejected {
                            reason: RejectReason::ThermalOverload { heat, capacity },
                        };
                    }
                }
                if harvested > 0 {
                    if let Err(reason) = agent.resources.add(ResourceKind::Electricity, harvested) {
                        return WorldEventKind::ActionRejected {
                            reason: match reason {
                                StockError::NegativeAmount { amount } => {
                                    RejectReason::InvalidAmount { amount }
                                }
                                StockError::Insufficient { .. } => {
                                    RejectReason::InvalidAmount { amount: harvested }
                                }
                            },
                        };
                    }
                    if physics.heat_factor > 0 {
                        agent.thermal.heat = agent
                            .thermal
                            .heat
                            .saturating_add(harvested * physics.heat_factor);
                    }
                }
                WorldEventKind::RadiationHarvested {
                    agent_id,
                    location_id,
                    amount: harvested,
                    available: local_available,
                }
            }
            Action::BuyPower {
                buyer,
                seller,
                amount,
                price_per_pu,
            } => match self.transfer_power(&seller, &buyer, amount, price_per_pu) {
                Ok(power_event) => WorldEventKind::Power(power_event),
                Err(reason) => WorldEventKind::ActionRejected { reason },
            },
            Action::SellPower {
                seller,
                buyer,
                amount,
                price_per_pu,
            } => match self.transfer_power(&seller, &buyer, amount, price_per_pu) {
                Ok(power_event) => WorldEventKind::Power(power_event),
                Err(reason) => WorldEventKind::ActionRejected { reason },
            },
            Action::PlacePowerOrder {
                owner,
                side,
                amount,
                limit_price_per_pu,
            } => self.place_power_order(owner, side, amount, limit_price_per_pu),
            Action::CancelPowerOrder { owner, order_id } => {
                self.cancel_power_order(owner, order_id)
            }
            Action::TransferResource {
                from,
                to,
                kind,
                amount,
            } => {
                if let Err(reason) = self.ensure_owner_chunks_generated(&from, &to) {
                    return WorldEventKind::ActionRejected { reason };
                }
                match self.validate_transfer(&from, &to, kind, amount) {
                    Ok(()) => {
                        if let Err(reason) = self.apply_transfer(&from, &to, kind, amount) {
                            WorldEventKind::ActionRejected { reason }
                        } else {
                            WorldEventKind::ResourceTransferred {
                                from,
                                to,
                                kind,
                                amount,
                            }
                        }
                    }
                    Err(reason) => WorldEventKind::ActionRejected { reason },
                }
            }
            Action::DebugGrantResource {
                owner,
                kind,
                amount,
            } => {
                if amount <= 0 {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InvalidAmount { amount },
                    };
                }
                if let Err(reason) = self.ensure_owner_exists(&owner) {
                    return WorldEventKind::ActionRejected { reason };
                }
                if let Err(reason) = self.add_to_owner(&owner, kind, amount) {
                    return WorldEventKind::ActionRejected { reason };
                }
                WorldEventKind::DebugResourceGranted {
                    owner,
                    kind,
                    amount,
                }
            }
            Action::MineCompound {
                owner,
                location_id,
                compound_mass_g,
            } => {
                if compound_mass_g <= 0 {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InvalidAmount {
                            amount: compound_mass_g,
                        },
                    };
                }
                if let Err(reason) = self.ensure_owner_exists(&owner) {
                    return WorldEventKind::ActionRejected { reason };
                }
                if !self.model.locations.contains_key(&location_id) {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::LocationNotFound { location_id },
                    };
                }
                let site_owner = ResourceOwner::Location {
                    location_id: location_id.clone(),
                };
                if let Err(reason) = self.ensure_colocated(&owner, &site_owner) {
                    return WorldEventKind::ActionRejected { reason };
                }
                if let Err(reason) = self.ensure_owner_chunks_generated(&owner, &site_owner) {
                    return WorldEventKind::ActionRejected { reason };
                }

                let max_per_action = self.config.economy.mine_compound_max_per_action_g;
                if max_per_action > 0 && compound_mass_g > max_per_action {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InvalidAmount {
                            amount: compound_mass_g,
                        },
                    };
                }

                let mined_so_far = self
                    .model
                    .locations
                    .get(&location_id)
                    .map(|location| location.mined_compound_g.max(0))
                    .unwrap_or(0);
                let max_per_location = self.config.economy.mine_compound_max_per_location_g;
                if max_per_location > 0 {
                    let available = max_per_location.saturating_sub(mined_so_far).max(0);
                    if compound_mass_g > available {
                        return WorldEventKind::ActionRejected {
                            reason: RejectReason::InsufficientResource {
                                owner: site_owner.clone(),
                                kind: ResourceKind::Compound,
                                requested: compound_mass_g,
                                available,
                            },
                        };
                    }
                }

                let extraction_plan =
                    match self.plan_compound_extraction(&location_id, compound_mass_g) {
                        Ok(plan) => plan,
                        Err(reason) => return WorldEventKind::ActionRejected { reason },
                    };
                let electricity_cost = self.compute_mine_compound_electricity_cost(compound_mass_g);
                let available_electricity = self
                    .owner_stock(&owner)
                    .map(|stock| stock.get(ResourceKind::Electricity))
                    .unwrap_or(0);
                if available_electricity < electricity_cost {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InsufficientResource {
                            owner: owner.clone(),
                            kind: ResourceKind::Electricity,
                            requested: electricity_cost,
                            available: available_electricity,
                        },
                    };
                }

                for (element, amount_g) in &extraction_plan {
                    if let Err(reason) =
                        self.consume_fragment_resource_for_action(&location_id, *element, *amount_g)
                    {
                        return WorldEventKind::ActionRejected { reason };
                    }
                }
                if let Some(location) = self.model.locations.get_mut(&location_id) {
                    location.mined_compound_g = mined_so_far.saturating_add(compound_mass_g);
                }

                if let Err(reason) =
                    self.remove_from_owner(&owner, ResourceKind::Electricity, electricity_cost)
                {
                    return WorldEventKind::ActionRejected { reason };
                }
                if let Err(reason) =
                    self.add_to_owner(&owner, ResourceKind::Compound, compound_mass_g)
                {
                    return WorldEventKind::ActionRejected { reason };
                }

                WorldEventKind::CompoundMined {
                    owner,
                    location_id,
                    compound_mass_g,
                    electricity_cost,
                    extracted_elements: extraction_plan.into_iter().collect::<BTreeMap<_, _>>(),
                }
            }
            Action::RefineCompound {
                owner,
                compound_mass_g,
            } => {
                if compound_mass_g <= 0 {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InvalidAmount {
                            amount: compound_mass_g,
                        },
                    };
                }
                if let Err(reason) = self.ensure_owner_chunk_generated(&owner) {
                    return WorldEventKind::ActionRejected { reason };
                }

                let (electricity_cost, hardware_output) =
                    self.compute_refine_compound_outputs(compound_mass_g);
                if hardware_output <= 0 {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InvalidAmount {
                            amount: compound_mass_g,
                        },
                    };
                }

                let available_compound = self
                    .owner_stock(&owner)
                    .map(|stock| stock.get(ResourceKind::Compound))
                    .unwrap_or(0);
                if available_compound < compound_mass_g {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InsufficientResource {
                            owner: owner.clone(),
                            kind: ResourceKind::Compound,
                            requested: compound_mass_g,
                            available: available_compound,
                        },
                    };
                }

                let available = self
                    .owner_stock(&owner)
                    .map(|stock| stock.get(ResourceKind::Electricity))
                    .unwrap_or(0);
                if available < electricity_cost {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InsufficientResource {
                            owner: owner.clone(),
                            kind: ResourceKind::Electricity,
                            requested: electricity_cost,
                            available,
                        },
                    };
                }

                if let Err(reason) =
                    self.remove_from_owner(&owner, ResourceKind::Compound, compound_mass_g)
                {
                    return WorldEventKind::ActionRejected { reason };
                }
                if let Err(reason) =
                    self.remove_from_owner(&owner, ResourceKind::Electricity, electricity_cost)
                {
                    return WorldEventKind::ActionRejected { reason };
                }
                if let Err(reason) =
                    self.add_to_owner(&owner, ResourceKind::Hardware, hardware_output)
                {
                    return WorldEventKind::ActionRejected { reason };
                }

                WorldEventKind::CompoundRefined {
                    owner,
                    compound_mass_g,
                    electricity_cost,
                    hardware_output,
                }
            }
            Action::BuildFactory {
                owner,
                location_id,
                factory_id,
                factory_kind,
            } => {
                if factory_id.trim().is_empty() {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InvalidAmount { amount: 0 },
                    };
                }
                if factory_kind.trim().is_empty() {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::RuleDenied {
                            notes: vec!["factory_kind cannot be empty".to_string()],
                        },
                    };
                }
                if !self.model.locations.contains_key(&location_id) {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::LocationNotFound { location_id },
                    };
                }
                if self.model.factories.contains_key(&factory_id) {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::FacilityAlreadyExists {
                            facility_id: factory_id,
                        },
                    };
                }
                let is_radiation_power_factory =
                    factory_kind.eq_ignore_ascii_case(FACTORY_KIND_RADIATION_POWER_MK1);
                if is_radiation_power_factory
                    && (self.model.power_plants.contains_key(&factory_id)
                        || self.model.power_storages.contains_key(&factory_id))
                {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::FacilityAlreadyExists {
                            facility_id: factory_id,
                        },
                    };
                }
                if let Err(reason) = self.ensure_owner_exists(&owner) {
                    return WorldEventKind::ActionRejected { reason };
                }
                let site_owner = ResourceOwner::Location {
                    location_id: location_id.clone(),
                };
                if let Err(reason) = self.ensure_colocated(&owner, &site_owner) {
                    return WorldEventKind::ActionRejected { reason };
                }
                if let Err(reason) = self.ensure_owner_chunks_generated(&owner, &site_owner) {
                    return WorldEventKind::ActionRejected { reason };
                }

                let electricity_cost = self.config.economy.factory_build_electricity_cost;
                let hardware_cost = self.config.economy.factory_build_hardware_cost;

                let available_electricity = self
                    .owner_stock(&owner)
                    .map(|stock| stock.get(ResourceKind::Electricity))
                    .unwrap_or(0);
                if available_electricity < electricity_cost {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InsufficientResource {
                            owner,
                            kind: ResourceKind::Electricity,
                            requested: electricity_cost,
                            available: available_electricity,
                        },
                    };
                }
                let available_hardware = self
                    .owner_stock(&owner)
                    .map(|stock| stock.get(ResourceKind::Hardware))
                    .unwrap_or(0);
                if available_hardware < hardware_cost {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InsufficientResource {
                            owner,
                            kind: ResourceKind::Hardware,
                            requested: hardware_cost,
                            available: available_hardware,
                        },
                    };
                }

                if let Err(reason) =
                    self.remove_from_owner(&owner, ResourceKind::Electricity, electricity_cost)
                {
                    return WorldEventKind::ActionRejected { reason };
                }
                if let Err(reason) =
                    self.remove_from_owner(&owner, ResourceKind::Hardware, hardware_cost)
                {
                    return WorldEventKind::ActionRejected { reason };
                }

                self.model.factories.insert(
                    factory_id.clone(),
                    Factory {
                        id: factory_id.clone(),
                        owner: owner.clone(),
                        location_id: location_id.clone(),
                        kind: factory_kind.clone(),
                    },
                );
                if is_radiation_power_factory {
                    self.model.power_plants.insert(
                        factory_id.clone(),
                        PowerPlant {
                            id: factory_id.clone(),
                            location_id: location_id.clone(),
                            owner: owner.clone(),
                            capacity_per_tick: self
                                .config
                                .economy
                                .radiation_power_plant_output_per_tick,
                            current_output: 0,
                            fuel_cost_per_pu: 0,
                            maintenance_cost: 0,
                            status: PlantStatus::Running,
                            efficiency: 1.0,
                            degradation: 0.0,
                        },
                    );
                }
                WorldEventKind::FactoryBuilt {
                    owner,
                    location_id,
                    factory_id,
                    factory_kind,
                    electricity_cost,
                    hardware_cost,
                }
            }
            Action::ScheduleRecipe {
                owner,
                factory_id,
                recipe_id,
                batches,
            } => {
                if recipe_id.trim().is_empty() {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::RuleDenied {
                            notes: vec!["recipe_id cannot be empty".to_string()],
                        },
                    };
                }
                if batches <= 0 {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InvalidAmount { amount: batches },
                    };
                }
                if let Err(reason) = self.ensure_owner_exists(&owner) {
                    return WorldEventKind::ActionRejected { reason };
                }

                let Some(factory) = self.model.factories.get(&factory_id).cloned() else {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::FacilityNotFound {
                            facility_id: factory_id,
                        },
                    };
                };
                if factory.owner != owner {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::RuleDenied {
                            notes: vec!["factory owner mismatch".to_string()],
                        },
                    };
                }
                let site_owner = ResourceOwner::Location {
                    location_id: factory.location_id.clone(),
                };
                if let Err(reason) = self.ensure_colocated(&owner, &site_owner) {
                    return WorldEventKind::ActionRejected { reason };
                }
                if let Err(reason) = self.ensure_owner_chunks_generated(&owner, &site_owner) {
                    return WorldEventKind::ActionRejected { reason };
                }

                let Some(plan) = self.recipe_plan(recipe_id.as_str()) else {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::RuleDenied {
                            notes: vec![format!("unsupported recipe_id: {recipe_id}")],
                        },
                    };
                };
                if !factory
                    .kind
                    .eq_ignore_ascii_case(plan.required_factory_kind)
                {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::RuleDenied {
                            notes: vec![format!(
                                "recipe {recipe_id} requires factory kind {}, got {}",
                                plan.required_factory_kind, factory.kind
                            )],
                        },
                    };
                }
                let recipe_scale = batches;
                let electricity_cost = plan.electricity_per_batch.saturating_mul(recipe_scale);
                let hardware_cost = plan.hardware_per_batch.saturating_mul(recipe_scale);
                let data_output = plan.data_output_per_batch.saturating_mul(recipe_scale);
                let finished_product_units = plan
                    .finished_product_units_per_batch
                    .saturating_mul(recipe_scale);

                let available_electricity = self
                    .owner_stock(&owner)
                    .map(|stock| stock.get(ResourceKind::Electricity))
                    .unwrap_or(0);
                if available_electricity < electricity_cost {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InsufficientResource {
                            owner,
                            kind: ResourceKind::Electricity,
                            requested: electricity_cost,
                            available: available_electricity,
                        },
                    };
                }
                let available_hardware = self
                    .owner_stock(&owner)
                    .map(|stock| stock.get(ResourceKind::Hardware))
                    .unwrap_or(0);
                if available_hardware < hardware_cost {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::InsufficientResource {
                            owner,
                            kind: ResourceKind::Hardware,
                            requested: hardware_cost,
                            available: available_hardware,
                        },
                    };
                }

                if let Err(reason) =
                    self.remove_from_owner(&owner, ResourceKind::Electricity, electricity_cost)
                {
                    return WorldEventKind::ActionRejected { reason };
                }
                if let Err(reason) =
                    self.remove_from_owner(&owner, ResourceKind::Hardware, hardware_cost)
                {
                    return WorldEventKind::ActionRejected { reason };
                }
                if data_output > 0 {
                    if let Err(reason) = self.add_to_owner(&owner, ResourceKind::Data, data_output)
                    {
                        return WorldEventKind::ActionRejected { reason };
                    }
                }

                WorldEventKind::RecipeScheduled {
                    owner,
                    factory_id,
                    recipe_id,
                    batches,
                    electricity_cost,
                    hardware_cost,
                    data_output,
                    finished_product_id: plan.finished_product_id.to_string(),
                    finished_product_units,
                }
            }
            Action::CompileModuleArtifactFromSource {
                publisher_agent_id,
                module_id,
                manifest_path,
                source_files,
            } => self.apply_compile_module_artifact_from_source(
                publisher_agent_id,
                module_id,
                manifest_path,
                source_files,
            ),
            Action::DeployModuleArtifact {
                publisher_agent_id,
                wasm_hash,
                wasm_bytes,
                module_id_hint,
            } => self.apply_deploy_module_artifact(
                publisher_agent_id,
                wasm_hash,
                wasm_bytes,
                module_id_hint,
            ),
            Action::InstallModuleFromArtifact {
                installer_agent_id,
                module_id,
                module_version,
                wasm_hash,
                activate,
            } => self.apply_install_module_from_artifact(
                installer_agent_id,
                module_id,
                module_version,
                wasm_hash,
                activate,
            ),
            Action::InstallModuleToTargetFromArtifact {
                installer_agent_id,
                module_id,
                module_version,
                wasm_hash,
                activate,
                ..
            } => self.apply_install_module_from_artifact(
                installer_agent_id,
                module_id,
                module_version,
                wasm_hash,
                activate,
            ),
            Action::ListModuleArtifactForSale {
                seller_agent_id,
                wasm_hash,
                price_kind,
                price_amount,
            } => self.apply_list_module_artifact_for_sale(
                seller_agent_id,
                wasm_hash,
                price_kind,
                price_amount,
            ),
            Action::BuyModuleArtifact {
                buyer_agent_id,
                wasm_hash,
            } => self.apply_buy_module_artifact(buyer_agent_id, wasm_hash),
            Action::DelistModuleArtifact {
                seller_agent_id,
                wasm_hash,
            } => self.apply_delist_module_artifact(seller_agent_id, wasm_hash),
            Action::DestroyModuleArtifact {
                owner_agent_id,
                wasm_hash,
                reason,
            } => self.apply_destroy_module_artifact(owner_agent_id, wasm_hash, reason),
            Action::PlaceModuleArtifactBid {
                bidder_agent_id,
                wasm_hash,
                price_kind,
                price_amount,
            } => self.apply_place_module_artifact_bid(
                bidder_agent_id,
                wasm_hash,
                price_kind,
                price_amount,
            ),
            Action::CancelModuleArtifactBid {
                bidder_agent_id,
                wasm_hash,
                bid_order_id,
            } => self.apply_cancel_module_artifact_bid(bidder_agent_id, wasm_hash, bid_order_id),
            Action::PublishSocialFact {
                actor,
                schema_id,
                subject,
                object,
                claim,
                confidence_ppm,
                evidence_event_ids,
                ttl_ticks,
                stake,
            } => self.apply_publish_social_fact(
                actor,
                schema_id,
                subject,
                object,
                claim,
                confidence_ppm,
                evidence_event_ids,
                ttl_ticks,
                stake,
            ),
            Action::ChallengeSocialFact {
                challenger,
                fact_id,
                reason,
                stake,
            } => self.apply_challenge_social_fact(challenger, fact_id, reason, stake),
            Action::AdjudicateSocialFact {
                adjudicator,
                fact_id,
                decision,
                notes,
            } => self.apply_adjudicate_social_fact(adjudicator, fact_id, decision, notes),
            Action::RevokeSocialFact {
                actor,
                fact_id,
                reason,
            } => self.apply_revoke_social_fact(actor, fact_id, reason),
            Action::DeclareSocialEdge {
                declarer,
                schema_id,
                relation_kind,
                from,
                to,
                weight_bps,
                backing_fact_ids,
                ttl_ticks,
            } => self.apply_declare_social_edge(
                declarer,
                schema_id,
                relation_kind,
                from,
                to,
                weight_bps,
                backing_fact_ids,
                ttl_ticks,
            ),
        }
    }

    fn next_power_order_id(&self) -> u64 {
        self.model.power_order_book.next_order_id.max(1)
    }

    fn find_power_order_index(&self, order_id: u64) -> Option<usize> {
        self.model
            .power_order_book
            .open_orders
            .iter()
            .position(|entry| entry.order_id == order_id)
    }

    fn sorted_opposite_power_order_ids(&self, incoming_side: PowerOrderSide) -> Vec<u64> {
        let mut entries: Vec<(u64, i64)> = self
            .model
            .power_order_book
            .open_orders
            .iter()
            .filter(|entry| entry.side != incoming_side)
            .map(|entry| (entry.order_id, entry.limit_price_per_pu))
            .collect();
        entries.sort_by(
            |(lhs_order_id, lhs_price), (rhs_order_id, rhs_price)| match incoming_side {
                PowerOrderSide::Buy => lhs_price
                    .cmp(rhs_price)
                    .then_with(|| lhs_order_id.cmp(rhs_order_id)),
                PowerOrderSide::Sell => rhs_price
                    .cmp(lhs_price)
                    .then_with(|| lhs_order_id.cmp(rhs_order_id)),
            },
        );
        entries.into_iter().map(|(order_id, _)| order_id).collect()
    }

    fn power_order_limits_cross(
        incoming_side: PowerOrderSide,
        incoming_limit_price_per_pu: i64,
        opposite_limit_price_per_pu: i64,
    ) -> bool {
        match incoming_side {
            PowerOrderSide::Buy => incoming_limit_price_per_pu >= opposite_limit_price_per_pu,
            PowerOrderSide::Sell => opposite_limit_price_per_pu >= incoming_limit_price_per_pu,
        }
    }

    fn power_order_quote_within_limits(
        quoted_price_per_pu: i64,
        sell_limit_price_per_pu: i64,
        buy_limit_price_per_pu: i64,
    ) -> bool {
        quoted_price_per_pu >= sell_limit_price_per_pu
            && quoted_price_per_pu <= buy_limit_price_per_pu
    }

    fn append_auto_cancelled_order_id(auto_cancelled_order_ids: &mut Vec<u64>, order_id: u64) {
        if !auto_cancelled_order_ids.contains(&order_id) {
            auto_cancelled_order_ids.push(order_id);
        }
    }

    fn place_power_order(
        &mut self,
        owner: ResourceOwner,
        side: PowerOrderSide,
        amount: i64,
        limit_price_per_pu: i64,
    ) -> WorldEventKind {
        if amount <= 0 {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::InvalidAmount { amount },
            };
        }
        if limit_price_per_pu < 0 {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::InvalidAmount {
                    amount: limit_price_per_pu,
                },
            };
        }
        if let Err(reason) = self.ensure_owner_exists(&owner) {
            return WorldEventKind::ActionRejected { reason };
        }
        if matches!(owner, ResourceOwner::Location { .. }) {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::RuleDenied {
                    notes: vec![LOCATION_ELECTRICITY_POOL_REMOVED_NOTE.to_string()],
                },
            };
        }
        if matches!(side, PowerOrderSide::Sell) {
            let available = self
                .owner_stock(&owner)
                .map(|stock| stock.get(ResourceKind::Electricity))
                .unwrap_or(0);
            if available < amount {
                return WorldEventKind::ActionRejected {
                    reason: RejectReason::InsufficientResource {
                        owner,
                        kind: ResourceKind::Electricity,
                        requested: amount,
                        available,
                    },
                };
            }
        }

        let order_id = self.next_power_order_id();
        self.model.power_order_book.next_order_id = order_id.saturating_add(1);
        let mut remaining_amount = amount;
        let mut fills = Vec::new();
        let mut auto_cancelled_order_ids = Vec::new();

        while remaining_amount > 0 {
            let candidate_order_ids = self.sorted_opposite_power_order_ids(side);
            if candidate_order_ids.is_empty() {
                break;
            }

            let mut matched_this_round = false;
            let mut stop_matching = false;

            for candidate_order_id in candidate_order_ids {
                let Some(candidate_index) = self.find_power_order_index(candidate_order_id) else {
                    continue;
                };
                let candidate_order =
                    self.model.power_order_book.open_orders[candidate_index].clone();
                if !Self::power_order_limits_cross(
                    side,
                    limit_price_per_pu,
                    candidate_order.limit_price_per_pu,
                ) {
                    stop_matching = true;
                    break;
                }

                let fill_amount = remaining_amount.min(candidate_order.remaining_amount);
                if fill_amount <= 0 {
                    self.model
                        .power_order_book
                        .open_orders
                        .remove(candidate_index);
                    Self::append_auto_cancelled_order_id(
                        &mut auto_cancelled_order_ids,
                        candidate_order.order_id,
                    );
                    continue;
                }

                let (seller, buyer, sell_limit_price_per_pu, buy_limit_price_per_pu) = match side {
                    PowerOrderSide::Buy => (
                        candidate_order.owner.clone(),
                        owner.clone(),
                        candidate_order.limit_price_per_pu,
                        limit_price_per_pu,
                    ),
                    PowerOrderSide::Sell => (
                        owner.clone(),
                        candidate_order.owner.clone(),
                        limit_price_per_pu,
                        candidate_order.limit_price_per_pu,
                    ),
                };
                let (buy_order_id, sell_order_id) = match side {
                    PowerOrderSide::Buy => (order_id, candidate_order.order_id),
                    PowerOrderSide::Sell => (candidate_order.order_id, order_id),
                };

                let prepared = match self.prepare_power_transfer(&seller, &buyer, fill_amount) {
                    Ok(prepared) => prepared,
                    Err(reason) => {
                        if matches!(side, PowerOrderSide::Buy)
                            && matches!(
                                reason,
                                RejectReason::InsufficientResource {
                                    owner: ref rejected_owner,
                                    kind: ResourceKind::Electricity,
                                    ..
                                } if rejected_owner == &seller
                            )
                        {
                            self.model
                                .power_order_book
                                .open_orders
                                .remove(candidate_index);
                            Self::append_auto_cancelled_order_id(
                                &mut auto_cancelled_order_ids,
                                candidate_order.order_id,
                            );
                        }
                        continue;
                    }
                };

                if !Self::power_order_quote_within_limits(
                    prepared.quoted_price_per_pu,
                    sell_limit_price_per_pu,
                    buy_limit_price_per_pu,
                ) {
                    if (matches!(side, PowerOrderSide::Buy)
                        && prepared.quoted_price_per_pu < sell_limit_price_per_pu)
                        || (matches!(side, PowerOrderSide::Sell)
                            && prepared.quoted_price_per_pu > buy_limit_price_per_pu)
                    {
                        stop_matching = true;
                        break;
                    }
                    continue;
                }

                let transfer = match self.transfer_power(
                    &seller,
                    &buyer,
                    fill_amount,
                    prepared.quoted_price_per_pu,
                ) {
                    Ok(transfer) => transfer,
                    Err(reason) => {
                        if matches!(side, PowerOrderSide::Buy)
                            && matches!(
                                reason,
                                RejectReason::InsufficientResource {
                                    owner: ref rejected_owner,
                                    kind: ResourceKind::Electricity,
                                    ..
                                } if rejected_owner == &seller
                            )
                        {
                            self.model
                                .power_order_book
                                .open_orders
                                .remove(candidate_index);
                            Self::append_auto_cancelled_order_id(
                                &mut auto_cancelled_order_ids,
                                candidate_order.order_id,
                            );
                        }
                        continue;
                    }
                };

                let PowerEvent::PowerTransferred {
                    from,
                    to,
                    amount: transferred_amount,
                    loss,
                    quoted_price_per_pu,
                    price_per_pu,
                    settlement_amount,
                } = transfer
                else {
                    continue;
                };

                let Some(candidate_index) = self.find_power_order_index(candidate_order_id) else {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::RuleDenied {
                            notes: vec![format!(
                                "power orderbook inconsistent: order {} missing during fill",
                                candidate_order_id
                            )],
                        },
                    };
                };
                let candidate_state = &mut self.model.power_order_book.open_orders[candidate_index];
                if candidate_state.remaining_amount < transferred_amount {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::RuleDenied {
                            notes: vec![format!(
                                "power orderbook inconsistent: order {} remaining {} < fill {}",
                                candidate_order_id,
                                candidate_state.remaining_amount,
                                transferred_amount
                            )],
                        },
                    };
                }
                candidate_state.remaining_amount = candidate_state
                    .remaining_amount
                    .saturating_sub(transferred_amount);
                if candidate_state.remaining_amount == 0 {
                    self.model
                        .power_order_book
                        .open_orders
                        .remove(candidate_index);
                }

                remaining_amount = remaining_amount.saturating_sub(transferred_amount);
                fills.push(PowerOrderFill {
                    buy_order_id,
                    sell_order_id,
                    buyer: to,
                    seller: from,
                    amount: transferred_amount,
                    loss,
                    quoted_price_per_pu,
                    price_per_pu,
                    settlement_amount,
                });
                matched_this_round = true;
                break;
            }

            if remaining_amount <= 0 || stop_matching || !matched_this_round {
                break;
            }
        }

        if remaining_amount > 0 {
            self.model
                .power_order_book
                .open_orders
                .push(PowerOrderState {
                    order_id,
                    owner: owner.clone(),
                    side,
                    remaining_amount,
                    limit_price_per_pu,
                    created_at: self.time,
                });
        }

        WorldEventKind::PowerOrderPlaced {
            order_id,
            owner,
            side,
            requested_amount: amount,
            remaining_amount,
            limit_price_per_pu,
            fills,
            auto_cancelled_order_ids,
        }
    }

    fn cancel_power_order(&mut self, owner: ResourceOwner, order_id: u64) -> WorldEventKind {
        if order_id == 0 {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::InvalidAmount { amount: 0 },
            };
        }
        if let Err(reason) = self.ensure_owner_exists(&owner) {
            return WorldEventKind::ActionRejected { reason };
        }
        if matches!(owner, ResourceOwner::Location { .. }) {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::RuleDenied {
                    notes: vec![LOCATION_ELECTRICITY_POOL_REMOVED_NOTE.to_string()],
                },
            };
        }

        let Some(order_index) = self
            .model
            .power_order_book
            .open_orders
            .iter()
            .position(|entry| entry.order_id == order_id && entry.owner == owner)
        else {
            return WorldEventKind::ActionRejected {
                reason: RejectReason::RuleDenied {
                    notes: vec![format!(
                        "cancel power order rejected: order {} not found for owner {:?}",
                        order_id, owner
                    )],
                },
            };
        };

        let removed = self.model.power_order_book.open_orders.remove(order_index);
        WorldEventKind::PowerOrderCancelled {
            owner,
            order_id,
            side: removed.side,
            remaining_amount: removed.remaining_amount,
        }
    }

    fn prepare_power_transfer(
        &mut self,
        from: &ResourceOwner,
        to: &ResourceOwner,
        amount: i64,
    ) -> Result<PreparedPowerTransfer, RejectReason> {
        if amount <= 0 {
            return Err(RejectReason::InvalidAmount { amount });
        }
        self.ensure_owner_exists(from)?;
        self.ensure_owner_exists(to)?;
        self.ensure_owner_chunks_generated(from, to)?;
        if matches!(from, ResourceOwner::Location { .. })
            || matches!(to, ResourceOwner::Location { .. })
        {
            return Err(RejectReason::RuleDenied {
                notes: vec![LOCATION_ELECTRICITY_POOL_REMOVED_NOTE.to_string()],
            });
        }

        let from_location = self.owner_location_id(from)?;
        let to_location = self.owner_location_id(to)?;
        if matches!(from, ResourceOwner::Agent { .. }) || matches!(to, ResourceOwner::Agent { .. })
        {
            self.ensure_colocated(from, to)?;
        }

        let seller_available_before = self
            .owner_stock(from)
            .map(|stock| stock.get(ResourceKind::Electricity))
            .unwrap_or(0);
        if seller_available_before < amount {
            return Err(RejectReason::InsufficientResource {
                owner: from.clone(),
                kind: ResourceKind::Electricity,
                requested: amount,
                available: seller_available_before,
            });
        }

        let mut distance_km = 0;
        let mut loss = 0;
        if from_location != to_location {
            distance_km = self.power_transfer_distance_km(&from_location, &to_location)?;
            let max_distance_km = self.config.power.transfer_max_distance_km;
            if distance_km > max_distance_km {
                return Err(RejectReason::PowerTransferDistanceExceeded {
                    distance_km,
                    max_distance_km,
                });
            }
            loss = self.power_transfer_loss(amount, distance_km);
            if loss >= amount {
                return Err(RejectReason::PowerTransferLossExceedsAmount { amount, loss });
            }
        }

        Ok(PreparedPowerTransfer {
            loss,
            quoted_price_per_pu: self.quote_power_market_price_per_pu(
                amount,
                distance_km,
                seller_available_before,
            ),
        })
    }

    fn transfer_power(
        &mut self,
        from: &ResourceOwner,
        to: &ResourceOwner,
        amount: i64,
        requested_price_per_pu: i64,
    ) -> Result<PowerEvent, RejectReason> {
        if requested_price_per_pu < 0 {
            return Err(RejectReason::InvalidAmount {
                amount: requested_price_per_pu,
            });
        }

        let prepared = self.prepare_power_transfer(from, to, amount)?;
        let executed_price_per_pu = if self.config.power.dynamic_price_enabled {
            if requested_price_per_pu == 0 {
                prepared.quoted_price_per_pu
            } else {
                let price_band_bps = self.config.power.market_price_band_bps;
                let quote = prepared.quoted_price_per_pu.max(1) as i128;
                let deviation_bps = ((requested_price_per_pu as i128
                    - prepared.quoted_price_per_pu as i128)
                    .abs()
                    .saturating_mul(10_000))
                .saturating_div(quote);
                if deviation_bps > price_band_bps as i128 {
                    return Err(RejectReason::RuleDenied {
                        notes: vec![format!(
                            "requested power price {} out of band (quote {}, band_bps {}, deviation_bps {})",
                            requested_price_per_pu,
                            prepared.quoted_price_per_pu,
                            price_band_bps,
                            deviation_bps
                        )],
                    });
                }
                requested_price_per_pu
            }
        } else {
            requested_price_per_pu
        };

        let delivered = amount - prepared.loss;
        self.remove_from_owner(from, ResourceKind::Electricity, amount)?;
        if delivered > 0 {
            self.add_to_owner(to, ResourceKind::Electricity, delivered)?;
        } else {
            return Err(RejectReason::PowerTransferLossExceedsAmount {
                amount,
                loss: prepared.loss,
            });
        }
        let settlement_amount = delivered.saturating_mul(executed_price_per_pu);

        Ok(PowerEvent::PowerTransferred {
            from: from.clone(),
            to: to.clone(),
            amount,
            loss: prepared.loss,
            quoted_price_per_pu: prepared.quoted_price_per_pu,
            price_per_pu: executed_price_per_pu,
            settlement_amount,
        })
    }

    fn owner_location_id(&self, owner: &ResourceOwner) -> Result<String, RejectReason> {
        match owner {
            ResourceOwner::Agent { agent_id } => self
                .model
                .agents
                .get(agent_id)
                .map(|agent| agent.location_id.clone())
                .ok_or_else(|| RejectReason::AgentNotFound {
                    agent_id: agent_id.clone(),
                }),
            ResourceOwner::Location { location_id } => {
                if self.model.locations.contains_key(location_id) {
                    Ok(location_id.clone())
                } else {
                    Err(RejectReason::LocationNotFound {
                        location_id: location_id.clone(),
                    })
                }
            }
        }
    }

    fn power_transfer_distance_km(
        &self,
        from_location_id: &str,
        to_location_id: &str,
    ) -> Result<i64, RejectReason> {
        let from = self.model.locations.get(from_location_id).ok_or_else(|| {
            RejectReason::LocationNotFound {
                location_id: from_location_id.to_string(),
            }
        })?;
        let to = self.model.locations.get(to_location_id).ok_or_else(|| {
            RejectReason::LocationNotFound {
                location_id: to_location_id.to_string(),
            }
        })?;
        let distance_cm = space_distance_cm(from.pos, to.pos);
        let distance_km = (distance_cm + CM_PER_KM - 1) / CM_PER_KM;
        Ok(distance_km)
    }

    fn power_transfer_loss(&self, amount: i64, distance_km: i64) -> i64 {
        if amount <= 0 || distance_km <= 0 {
            return 0;
        }
        let bps = self.config.power.transfer_loss_per_km_bps;
        if bps <= 0 {
            return 0;
        }
        let loss = (amount as i128)
            .saturating_mul(distance_km as i128)
            .saturating_mul(bps as i128)
            / 10_000;
        loss.min(amount as i128) as i64
    }

    fn quote_power_market_price_per_pu(
        &self,
        amount: i64,
        distance_km: i64,
        seller_available_before: i64,
    ) -> i64 {
        let cfg = &self.config.power;
        let min_price = cfg.market_price_min_per_pu.max(0);
        let max_price = cfg.market_price_max_per_pu.max(min_price);
        let base_price = cfg.market_base_price_per_pu.clamp(min_price, max_price);

        let scarcity_bps = if seller_available_before <= 0 {
            cfg.market_scarcity_price_max_bps
        } else {
            ((amount as i128)
                .saturating_mul(10_000)
                .saturating_div(seller_available_before as i128))
            .clamp(0, cfg.market_scarcity_price_max_bps as i128) as i64
        };
        let scarcity_premium = ((base_price as i128)
            .saturating_mul(scarcity_bps as i128)
            .saturating_add(9_999)
            / 10_000) as i64;
        let distance_premium = ((base_price as i128)
            .saturating_mul(distance_km.max(0) as i128)
            .saturating_mul(cfg.market_distance_price_per_km_bps as i128)
            .saturating_add(9_999)
            / 10_000) as i64;

        base_price
            .saturating_add(scarcity_premium)
            .saturating_add(distance_premium)
            .clamp(min_price, max_price)
    }

    fn validate_transfer(
        &self,
        from: &ResourceOwner,
        to: &ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    ) -> Result<(), RejectReason> {
        if amount <= 0 {
            return Err(RejectReason::InvalidAmount { amount });
        }

        self.ensure_owner_exists(from)?;
        self.ensure_owner_exists(to)?;
        self.ensure_colocated(from, to)?;

        let available = self
            .owner_stock(from)
            .map(|stock| stock.get(kind))
            .unwrap_or(0);
        if available < amount {
            return Err(RejectReason::InsufficientResource {
                owner: from.clone(),
                kind,
                requested: amount,
                available,
            });
        }

        Ok(())
    }

    fn apply_transfer(
        &mut self,
        from: &ResourceOwner,
        to: &ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    ) -> Result<(), RejectReason> {
        self.remove_from_owner(from, kind, amount)?;
        self.add_to_owner(to, kind, amount)?;
        Ok(())
    }

    pub(super) fn ensure_module_visual_anchor_exists(
        &self,
        anchor: &ModuleVisualAnchor,
    ) -> Result<(), RejectReason> {
        match anchor {
            ModuleVisualAnchor::Agent { agent_id } => {
                if self.model.agents.contains_key(agent_id) {
                    Ok(())
                } else {
                    Err(RejectReason::AgentNotFound {
                        agent_id: agent_id.clone(),
                    })
                }
            }
            ModuleVisualAnchor::Location { location_id } => {
                if self.model.locations.contains_key(location_id) {
                    Ok(())
                } else {
                    Err(RejectReason::LocationNotFound {
                        location_id: location_id.clone(),
                    })
                }
            }
            ModuleVisualAnchor::Absolute { pos } => {
                if self.config.space.contains(*pos) {
                    Ok(())
                } else {
                    Err(RejectReason::PositionOutOfBounds { pos: *pos })
                }
            }
        }
    }

    pub(super) fn ensure_owner_exists(&self, owner: &ResourceOwner) -> Result<(), RejectReason> {
        match owner {
            ResourceOwner::Agent { agent_id } => {
                if self.model.agents.contains_key(agent_id) {
                    Ok(())
                } else {
                    Err(RejectReason::AgentNotFound {
                        agent_id: agent_id.clone(),
                    })
                }
            }
            ResourceOwner::Location { location_id } => {
                if self.model.locations.contains_key(location_id) {
                    Ok(())
                } else {
                    Err(RejectReason::LocationNotFound {
                        location_id: location_id.clone(),
                    })
                }
            }
        }
    }

    pub(super) fn ensure_colocated(
        &self,
        from: &ResourceOwner,
        to: &ResourceOwner,
    ) -> Result<(), RejectReason> {
        match (from, to) {
            (ResourceOwner::Agent { agent_id }, ResourceOwner::Location { location_id }) => {
                let agent =
                    self.model
                        .agents
                        .get(agent_id)
                        .ok_or_else(|| RejectReason::AgentNotFound {
                            agent_id: agent_id.clone(),
                        })?;
                if agent.location_id != *location_id {
                    return Err(RejectReason::AgentNotAtLocation {
                        agent_id: agent_id.clone(),
                        location_id: location_id.clone(),
                    });
                }
            }
            (ResourceOwner::Location { location_id }, ResourceOwner::Agent { agent_id }) => {
                let agent =
                    self.model
                        .agents
                        .get(agent_id)
                        .ok_or_else(|| RejectReason::AgentNotFound {
                            agent_id: agent_id.clone(),
                        })?;
                if agent.location_id != *location_id {
                    return Err(RejectReason::AgentNotAtLocation {
                        agent_id: agent_id.clone(),
                        location_id: location_id.clone(),
                    });
                }
            }
            (
                ResourceOwner::Agent { agent_id },
                ResourceOwner::Agent {
                    agent_id: other_agent_id,
                },
            ) => {
                let agent =
                    self.model
                        .agents
                        .get(agent_id)
                        .ok_or_else(|| RejectReason::AgentNotFound {
                            agent_id: agent_id.clone(),
                        })?;
                let other = self.model.agents.get(other_agent_id).ok_or_else(|| {
                    RejectReason::AgentNotFound {
                        agent_id: other_agent_id.clone(),
                    }
                })?;
                if agent.location_id != other.location_id {
                    return Err(RejectReason::AgentsNotCoLocated {
                        agent_id: agent_id.clone(),
                        other_agent_id: other_agent_id.clone(),
                    });
                }
            }
            (
                ResourceOwner::Location { location_id },
                ResourceOwner::Location {
                    location_id: other_location_id,
                },
            ) => {
                return Err(RejectReason::LocationTransferNotAllowed {
                    from: location_id.clone(),
                    to: other_location_id.clone(),
                });
            }
        }
        Ok(())
    }

    pub(super) fn ensure_owner_chunks_generated(
        &mut self,
        from: &ResourceOwner,
        to: &ResourceOwner,
    ) -> Result<(), RejectReason> {
        self.ensure_owner_chunk_generated(from)?;
        self.ensure_owner_chunk_generated(to)?;
        Ok(())
    }

    fn ensure_owner_chunk_generated(&mut self, owner: &ResourceOwner) -> Result<(), RejectReason> {
        if let Some(pos) = self.owner_pos(owner)? {
            self.ensure_chunk_generated_at(pos, ChunkGenerationCause::Action)?;
        }
        Ok(())
    }

    pub(super) fn radiation_available_at(&self, harvest_pos: GeoPos) -> i64 {
        let physics = &self.config.physics;
        let near_range_cm = CHUNK_SIZE_X_CM.max(1) as f64;
        let mut near_sources = 0.0;
        let mut background = 0.0;

        for source in self.model.locations.values() {
            let contribution = self.radiation_source_contribution(harvest_pos, source);
            if contribution <= 0.0 {
                continue;
            }

            let source_radius_cm = source.profile.radius_cm.max(1) as f64;
            let source_distance_cm = space_distance_cm(harvest_pos, source.pos).max(0) as f64;
            let surface_distance_cm = (source_distance_cm - source_radius_cm).max(0.0);

            if surface_distance_cm <= near_range_cm {
                near_sources += contribution;
            } else {
                background += contribution;
            }
        }

        let floor = physics.radiation_floor.max(0);
        let floor_cap = physics.radiation_floor_cap_per_tick.max(0);
        let floor_contribution = floor.min(floor_cap) as f64;
        (near_sources + background + floor_contribution).floor() as i64
    }

    fn radiation_source_contribution(&self, harvest_pos: GeoPos, source: &Location) -> f64 {
        let emission = source.profile.radiation_emission_per_tick.max(0) as f64;
        if emission <= 0.0 {
            return 0.0;
        }

        let source_radius_cm = source.profile.radius_cm.max(1) as f64;
        let source_distance_cm = space_distance_cm(harvest_pos, source.pos).max(0) as f64;
        let surface_distance_cm = (source_distance_cm - source_radius_cm).max(0.0);
        let normalized_distance = surface_distance_cm / source_radius_cm;

        let geometric_attenuation = 1.0 / (1.0 + normalized_distance * normalized_distance);
        let medium_decay = (-self.config.physics.radiation_decay_k * surface_distance_cm).exp();
        emission * geometric_attenuation * medium_decay
    }

    fn compute_mine_compound_electricity_cost(&self, compound_mass_g: i64) -> i64 {
        let mass_kg = compound_mass_g.saturating_add(999).saturating_div(1000);
        mass_kg.saturating_mul(self.config.economy.mine_electricity_cost_per_kg)
    }

    fn plan_compound_extraction(
        &self,
        location_id: &str,
        compound_mass_g: i64,
    ) -> Result<Vec<(FragmentElementKind, i64)>, RejectReason> {
        let location = self.model.locations.get(location_id).ok_or_else(|| {
            RejectReason::LocationNotFound {
                location_id: location_id.to_string(),
            }
        })?;
        let budget = location.fragment_budget.as_ref().ok_or_else(|| {
            RejectReason::InsufficientResource {
                owner: ResourceOwner::Location {
                    location_id: location_id.to_string(),
                },
                kind: ResourceKind::Compound,
                requested: compound_mass_g,
                available: 0,
            }
        })?;

        let total_available = budget
            .remaining_by_element_g
            .values()
            .copied()
            .filter(|amount| *amount > 0)
            .sum::<i64>();
        if total_available < compound_mass_g {
            return Err(RejectReason::InsufficientResource {
                owner: ResourceOwner::Location {
                    location_id: location_id.to_string(),
                },
                kind: ResourceKind::Compound,
                requested: compound_mass_g,
                available: total_available,
            });
        }

        let mut remaining = compound_mass_g;
        let mut plan = Vec::new();
        for (element, available) in &budget.remaining_by_element_g {
            if remaining <= 0 {
                break;
            }
            if *available <= 0 {
                continue;
            }
            let consume = (*available).min(remaining);
            if consume > 0 {
                plan.push((*element, consume));
                remaining = remaining.saturating_sub(consume);
            }
        }
        if remaining > 0 {
            return Err(RejectReason::InsufficientResource {
                owner: ResourceOwner::Location {
                    location_id: location_id.to_string(),
                },
                kind: ResourceKind::Compound,
                requested: compound_mass_g,
                available: compound_mass_g.saturating_sub(remaining),
            });
        }
        Ok(plan)
    }

    fn consume_fragment_resource_for_action(
        &mut self,
        location_id: &str,
        kind: FragmentElementKind,
        amount_g: i64,
    ) -> Result<(), RejectReason> {
        self.consume_fragment_resource(location_id, kind, amount_g)
            .map(|_| ())
            .map_err(|err| self.fragment_error_to_reject_reason(location_id, err))
    }

    fn fragment_error_to_reject_reason(
        &self,
        location_id: &str,
        err: FragmentResourceError,
    ) -> RejectReason {
        match err {
            FragmentResourceError::LocationNotFound { location_id } => {
                RejectReason::LocationNotFound { location_id }
            }
            FragmentResourceError::FragmentBudgetMissing { location_id } => {
                RejectReason::InsufficientResource {
                    owner: ResourceOwner::Location { location_id },
                    kind: ResourceKind::Compound,
                    requested: 1,
                    available: 0,
                }
            }
            FragmentResourceError::ChunkCoordUnavailable { location_id } => {
                RejectReason::RuleDenied {
                    notes: vec![format!(
                        "chunk coord unavailable while mining at location {location_id}"
                    )],
                }
            }
            FragmentResourceError::ChunkBudgetMissing { coord } => {
                RejectReason::ChunkGenerationFailed {
                    x: coord.x,
                    y: coord.y,
                    z: coord.z,
                }
            }
            FragmentResourceError::Budget(ElementBudgetError::InvalidAmount { amount_g }) => {
                RejectReason::InvalidAmount { amount: amount_g }
            }
            FragmentResourceError::Budget(ElementBudgetError::Insufficient {
                requested_g,
                remaining_g,
                ..
            }) => RejectReason::InsufficientResource {
                owner: ResourceOwner::Location {
                    location_id: location_id.to_string(),
                },
                kind: ResourceKind::Compound,
                requested: requested_g,
                available: remaining_g,
            },
        }
    }

    fn compute_refine_compound_outputs(&self, compound_mass_g: i64) -> (i64, i64) {
        let economy = &self.config.economy;
        let mass_kg = compound_mass_g.saturating_add(999).saturating_div(1000);
        let electricity_cost = mass_kg.saturating_mul(economy.refine_electricity_cost_per_kg);
        let hardware_output = compound_mass_g
            .saturating_mul(economy.refine_hardware_yield_ppm)
            .saturating_div(PPM_BASE);
        (electricity_cost, hardware_output)
    }

    fn recipe_plan(&self, recipe_id: &str) -> Option<RecipePlan> {
        let economy = &self.config.economy;
        let normalized = recipe_id.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "recipe.assembler.control_chip" | "recipe.control_chip" => Some(RecipePlan {
                required_factory_kind: FACTORY_KIND_ASSEMBLER_MK1,
                electricity_per_batch: economy.recipe_electricity_cost_per_batch,
                hardware_per_batch: economy.recipe_hardware_cost_per_batch,
                data_output_per_batch: economy.recipe_data_output_per_batch,
                finished_product_id: "control_chip",
                finished_product_units_per_batch: 1,
            }),
            "recipe.assembler.motor_mk1" | "recipe.motor_mk1" => Some(RecipePlan {
                required_factory_kind: FACTORY_KIND_ASSEMBLER_MK1,
                electricity_per_batch: economy.recipe_electricity_cost_per_batch.saturating_mul(2),
                hardware_per_batch: economy.recipe_hardware_cost_per_batch.saturating_mul(2),
                data_output_per_batch: economy.recipe_data_output_per_batch.saturating_mul(2),
                finished_product_id: "motor_mk1",
                finished_product_units_per_batch: 1,
            }),
            "recipe.assembler.logistics_drone" | "recipe.logistics_drone" => Some(RecipePlan {
                required_factory_kind: FACTORY_KIND_ASSEMBLER_MK1,
                electricity_per_batch: economy.recipe_electricity_cost_per_batch.saturating_mul(4),
                hardware_per_batch: economy.recipe_hardware_cost_per_batch.saturating_mul(4),
                data_output_per_batch: economy.recipe_data_output_per_batch.saturating_mul(4),
                finished_product_id: "logistics_drone",
                finished_product_units_per_batch: 1,
            }),
            _ => None,
        }
    }

    fn owner_pos(
        &self,
        owner: &ResourceOwner,
    ) -> Result<Option<crate::geometry::GeoPos>, RejectReason> {
        match owner {
            ResourceOwner::Agent { agent_id } => self
                .model
                .agents
                .get(agent_id)
                .map(|agent| Some(agent.pos))
                .ok_or_else(|| RejectReason::AgentNotFound {
                    agent_id: agent_id.clone(),
                }),
            ResourceOwner::Location { location_id } => self
                .model
                .locations
                .get(location_id)
                .map(|location| Some(location.pos))
                .ok_or_else(|| RejectReason::LocationNotFound {
                    location_id: location_id.clone(),
                }),
        }
    }

    pub(super) fn owner_stock(
        &self,
        owner: &ResourceOwner,
    ) -> Option<&super::super::types::ResourceStock> {
        match owner {
            ResourceOwner::Agent { agent_id } => {
                self.model.agents.get(agent_id).map(|a| &a.resources)
            }
            ResourceOwner::Location { location_id } => {
                self.model.locations.get(location_id).map(|l| &l.resources)
            }
        }
    }

    pub(super) fn remove_from_owner(
        &mut self,
        owner: &ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    ) -> Result<(), RejectReason> {
        if matches!(owner, ResourceOwner::Location { .. })
            && matches!(kind, ResourceKind::Electricity)
        {
            return Err(RejectReason::RuleDenied {
                notes: vec![LOCATION_ELECTRICITY_POOL_REMOVED_NOTE.to_string()],
            });
        }
        let stock = match owner {
            ResourceOwner::Agent { agent_id } => self
                .model
                .agents
                .get_mut(agent_id)
                .map(|agent| &mut agent.resources)
                .ok_or_else(|| RejectReason::AgentNotFound {
                    agent_id: agent_id.clone(),
                })?,
            ResourceOwner::Location { location_id } => self
                .model
                .locations
                .get_mut(location_id)
                .map(|location| &mut location.resources)
                .ok_or_else(|| RejectReason::LocationNotFound {
                    location_id: location_id.clone(),
                })?,
        };

        stock.remove(kind, amount).map_err(|err| match err {
            StockError::NegativeAmount { amount } => RejectReason::InvalidAmount { amount },
            StockError::Insufficient {
                requested,
                available,
                ..
            } => RejectReason::InsufficientResource {
                owner: owner.clone(),
                kind,
                requested,
                available,
            },
        })
    }

    pub(super) fn add_to_owner(
        &mut self,
        owner: &ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    ) -> Result<(), RejectReason> {
        if matches!(owner, ResourceOwner::Location { .. })
            && matches!(kind, ResourceKind::Electricity)
        {
            return Err(RejectReason::RuleDenied {
                notes: vec![LOCATION_ELECTRICITY_POOL_REMOVED_NOTE.to_string()],
            });
        }
        let stock = match owner {
            ResourceOwner::Agent { agent_id } => self
                .model
                .agents
                .get_mut(agent_id)
                .map(|agent| &mut agent.resources)
                .ok_or_else(|| RejectReason::AgentNotFound {
                    agent_id: agent_id.clone(),
                })?,
            ResourceOwner::Location { location_id } => self
                .model
                .locations
                .get_mut(location_id)
                .map(|location| &mut location.resources)
                .ok_or_else(|| RejectReason::LocationNotFound {
                    location_id: location_id.clone(),
                })?,
        };

        stock.add(kind, amount).map_err(|err| match err {
            StockError::NegativeAmount { amount } => RejectReason::InvalidAmount { amount },
            StockError::Insufficient { .. } => RejectReason::InvalidAmount { amount },
        })
    }
}
