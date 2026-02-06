use crate::geometry::space_distance_cm;

use super::types::{ChunkGenerationCause, RejectReason, WorldEventKind};
use super::WorldKernel;
use super::super::power::{PlantStatus, PowerEvent, PowerPlant, PowerStorage};
use super::super::types::{Action, ResourceKind, ResourceOwner, StockError, CM_PER_KM};
use super::super::world_model::{movement_cost, Agent, Location};

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
            Action::DrawPower { storage_id, amount } => {
                match self.discharge_power_storage_event(&storage_id, amount) {
                    Ok(power_event) => WorldEventKind::Power(power_event),
                    Err(reason) => WorldEventKind::ActionRejected { reason },
                }
            }
            Action::StorePower { storage_id, amount } => {
                match self.charge_power_storage_event(&storage_id, amount) {
                    Ok(power_event) => WorldEventKind::Power(power_event),
                    Err(reason) => WorldEventKind::ActionRejected { reason },
                }
            }
            Action::MoveAgent { agent_id, to } => {
                let to_pos = match self.model.locations.get(&to) {
                    Some(location) => location.pos,
                    None => {
                        return WorldEventKind::ActionRejected {
                            reason: RejectReason::LocationNotFound { location_id: to },
                        };
                    }
                };
                if let Err(reason) = self.ensure_chunk_generated_at(to_pos, ChunkGenerationCause::Action) {
                    return WorldEventKind::ActionRejected {
                        reason,
                    };
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
                let electricity_cost = movement_cost(
                    distance_cm,
                    self.config.move_cost_per_km_electricity,
                );
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
                if let Err(reason) = self.ensure_chunk_generated_at(location_pos, ChunkGenerationCause::Action) {
                    return WorldEventKind::ActionRejected { reason };
                }
                let (emission, radius_cm) = match self.model.locations.get(&location_id) {
                    Some(location) => (
                        location.profile.radiation_emission_per_tick,
                        location.profile.radius_cm,
                    ),
                    None => {
                        return WorldEventKind::ActionRejected {
                            reason: RejectReason::LocationNotFound { location_id },
                        };
                    }
                };
                let physics = &self.config.physics;
                let path_cm = radius_cm.max(0) as f64;
                let decay = (-physics.radiation_decay_k * path_cm).exp();
                let local_available = ((emission.max(0) as f64) * decay
                    + (physics.radiation_floor.max(0) as f64))
                    .floor() as i64;
                if local_available <= 0 {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::RadiationUnavailable { location_id },
                    };
                }
                let mut available_for_harvest = local_available;
                if physics.max_harvest_per_tick > 0 {
                    available_for_harvest =
                        available_for_harvest.min(physics.max_harvest_per_tick);
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
                        agent.thermal.heat =
                            agent.thermal.heat.saturating_add(harvested * physics.heat_factor);
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
        }
    }

    fn transfer_power(
        &mut self,
        from: &ResourceOwner,
        to: &ResourceOwner,
        amount: i64,
        price_per_pu: i64,
    ) -> Result<PowerEvent, RejectReason> {
        if amount <= 0 {
            return Err(RejectReason::InvalidAmount { amount });
        }
        if price_per_pu < 0 {
            return Err(RejectReason::InvalidAmount {
                amount: price_per_pu,
            });
        }
        self.ensure_owner_exists(from)?;
        self.ensure_owner_exists(to)?;
        self.ensure_owner_chunks_generated(from, to)?;

        let from_location = self.owner_location_id(from)?;
        let to_location = self.owner_location_id(to)?;

        if matches!(from, ResourceOwner::Agent { .. })
            || matches!(to, ResourceOwner::Agent { .. })
        {
            self.ensure_colocated(from, to)?;
        }

        let mut loss = 0;
        if from_location != to_location {
            let distance_km = self.power_transfer_distance_km(&from_location, &to_location)?;
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

        let delivered = amount - loss;
        self.remove_from_owner(from, ResourceKind::Electricity, amount)?;
        if delivered > 0 {
            self.add_to_owner(to, ResourceKind::Electricity, delivered)?;
        } else {
            return Err(RejectReason::PowerTransferLossExceedsAmount { amount, loss });
        }

        Ok(PowerEvent::PowerTransferred {
            from: from.clone(),
            to: to.clone(),
            amount,
            loss,
            price_per_pu,
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

        let available = self.owner_stock(from).map(|stock| stock.get(kind)).unwrap_or(0);
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

    fn ensure_colocated(
        &self,
        from: &ResourceOwner,
        to: &ResourceOwner,
    ) -> Result<(), RejectReason> {
        match (from, to) {
            (
                ResourceOwner::Agent { agent_id },
                ResourceOwner::Location { location_id },
            ) => {
                let agent = self.model.agents.get(agent_id).ok_or_else(|| {
                    RejectReason::AgentNotFound {
                        agent_id: agent_id.clone(),
                    }
                })?;
                if agent.location_id != *location_id {
                    return Err(RejectReason::AgentNotAtLocation {
                        agent_id: agent_id.clone(),
                        location_id: location_id.clone(),
                    });
                }
            }
            (
                ResourceOwner::Location { location_id },
                ResourceOwner::Agent { agent_id },
            ) => {
                let agent = self.model.agents.get(agent_id).ok_or_else(|| {
                    RejectReason::AgentNotFound {
                        agent_id: agent_id.clone(),
                    }
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
                let agent = self.model.agents.get(agent_id).ok_or_else(|| {
                    RejectReason::AgentNotFound {
                        agent_id: agent_id.clone(),
                    }
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

    fn ensure_owner_chunks_generated(
        &mut self,
        from: &ResourceOwner,
        to: &ResourceOwner,
    ) -> Result<(), RejectReason> {
        if let Some(pos) = self.owner_pos(from)? {
            self.ensure_chunk_generated_at(pos, ChunkGenerationCause::Action)?;
        }
        if let Some(pos) = self.owner_pos(to)? {
            self.ensure_chunk_generated_at(pos, ChunkGenerationCause::Action)?;
        }
        Ok(())
    }

    fn owner_pos(&self, owner: &ResourceOwner) -> Result<Option<crate::geometry::GeoPos>, RejectReason> {
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

    fn owner_stock(&self, owner: &ResourceOwner) -> Option<&super::super::types::ResourceStock> {
        match owner {
            ResourceOwner::Agent { agent_id } => self.model.agents.get(agent_id).map(|a| &a.resources),
            ResourceOwner::Location { location_id } => {
                self.model.locations.get(location_id).map(|l| &l.resources)
            }
        }
    }

    fn remove_from_owner(
        &mut self,
        owner: &ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    ) -> Result<(), RejectReason> {
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

    fn add_to_owner(
        &mut self,
        owner: &ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    ) -> Result<(), RejectReason> {
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
