use crate::geometry::great_circle_distance_cm;

use super::types::{RejectReason, WorldEventKind};
use super::WorldKernel;
use super::super::power::{PlantStatus, PowerEvent, PowerPlant, PowerStorage};
use super::super::types::{Action, ResourceKind, ResourceOwner, StockError};
use super::super::world_model::{movement_cost, Agent, Location};

impl WorldKernel {
    pub(super) fn apply_action(&mut self, action: Action) -> WorldEventKind {
        match action {
            Action::RegisterLocation {
                location_id,
                name,
                pos,
            } => {
                if self.model.locations.contains_key(&location_id) {
                    return WorldEventKind::ActionRejected {
                        reason: RejectReason::LocationAlreadyExists { location_id },
                    };
                }
                let location = Location::new(location_id.clone(), name.clone(), pos);
                self.model.locations.insert(location_id.clone(), location);
                WorldEventKind::LocationRegistered {
                    location_id,
                    name,
                    pos,
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
            Action::MoveAgent { agent_id, to } => {
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
                let distance_cm = great_circle_distance_cm(agent.pos, location.pos);
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
            Action::TransferResource {
                from,
                to,
                kind,
                amount,
            } => match self.validate_transfer(&from, &to, kind, amount) {
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
            },
        }
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
