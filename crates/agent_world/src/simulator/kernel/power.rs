use super::super::power::{AgentPowerState, ConsumeReason, PlantStatus, PowerEvent};
use super::super::types::{AgentId, FacilityId, ResourceKind, ResourceOwner, StockError};
use super::types::{RejectReason, WorldEvent, WorldEventKind};
use super::WorldKernel;

impl WorldKernel {
    /// Process power consumption for all agents (idle consumption).
    /// Returns a list of power events generated.
    pub fn process_power_tick(&mut self) -> Vec<WorldEvent> {
        let mut events = Vec::new();
        let idle_cost = self.config.power.idle_cost_per_tick;
        let power_config = self.config.power.clone();
        let thermal_capacity = self.config.physics.thermal_capacity;
        let thermal_dissipation = self.config.physics.thermal_dissipation;
        let thermal_dissipation_gradient_bps = self.config.physics.thermal_dissipation_gradient_bps;

        let agent_ids: Vec<AgentId> = self.model.agents.keys().cloned().collect();

        for agent_id in agent_ids {
            let (consumed, remaining, old_state, new_state) = {
                let agent = match self.model.agents.get_mut(&agent_id) {
                    Some(a) => a,
                    None => continue,
                };

                if agent.power.is_shutdown() {
                    continue;
                }

                let old_state = agent.power.state;
                let consumed = agent.power.consume(idle_cost, &power_config);
                let new_state = agent.power.state;
                let thermal_drop = Self::scaled_thermal_dissipation(
                    agent.thermal.heat,
                    thermal_capacity,
                    thermal_dissipation,
                    thermal_dissipation_gradient_bps,
                );
                if thermal_drop > 0 {
                    agent.thermal.heat = agent.thermal.heat.saturating_sub(thermal_drop);
                }
                (consumed, agent.power.level, old_state, new_state)
            };

            if consumed > 0 {
                let power_event = PowerEvent::PowerConsumed {
                    agent_id: agent_id.clone(),
                    amount: consumed,
                    reason: ConsumeReason::Idle,
                    remaining,
                };
                let event = self.record_event(super::types::WorldEventKind::Power(power_event));
                events.push(event);
            }

            if old_state != new_state {
                let power_event = PowerEvent::PowerStateChanged {
                    agent_id: agent_id.clone(),
                    from: old_state,
                    to: new_state,
                    trigger_level: remaining,
                };
                let event = self.record_event(super::types::WorldEventKind::Power(power_event));
                events.push(event);
            }
        }

        events
    }

    fn scaled_thermal_dissipation(
        heat: i64,
        thermal_capacity: i64,
        thermal_dissipation: i64,
        thermal_dissipation_gradient_bps: i64,
    ) -> i64 {
        if heat <= 0 || thermal_dissipation <= 0 || thermal_dissipation_gradient_bps <= 0 {
            return 0;
        }

        let reference_heat = thermal_capacity.max(1) as i128;
        let numerator = (thermal_dissipation as i128)
            .saturating_mul(heat as i128)
            .saturating_mul(thermal_dissipation_gradient_bps as i128);
        let denominator = reference_heat.saturating_mul(10_000);
        let scaled = numerator
            .saturating_add(denominator.saturating_sub(1))
            .saturating_div(denominator);
        scaled.clamp(1, i64::MAX as i128) as i64
    }

    /// Process power generation for all power plants.
    /// Returns a list of power events generated.
    pub fn process_power_generation_tick(&mut self) -> Vec<WorldEvent> {
        let mut events = Vec::new();
        let plant_ids: Vec<FacilityId> = self.model.power_plants.keys().cloned().collect();

        for plant_id in plant_ids {
            let (output, location_id) = {
                let plant = match self.model.power_plants.get_mut(&plant_id) {
                    Some(plant) => plant,
                    None => continue,
                };
                if plant.status != PlantStatus::Running {
                    plant.current_output = 0;
                    continue;
                }
                let output = plant.effective_output();
                plant.current_output = output;
                (output, plant.location_id.clone())
            };

            if output <= 0 {
                continue;
            }

            if let Some(location) = self.model.locations.get_mut(&location_id) {
                if location
                    .resources
                    .add(ResourceKind::Electricity, output)
                    .is_err()
                {
                    continue;
                }
                let power_event = PowerEvent::PowerGenerated {
                    plant_id: plant_id.clone(),
                    location_id: location_id.clone(),
                    amount: output,
                };
                let event = self.record_event(super::types::WorldEventKind::Power(power_event));
                events.push(event);
            }
        }

        events
    }

    /// Charge a power storage facility using electricity at its location.
    /// Returns the power event if charging occurred.
    pub fn charge_power_storage(
        &mut self,
        storage_id: &FacilityId,
        requested_input: i64,
    ) -> Option<WorldEvent> {
        let power_event = self
            .charge_power_storage_event(storage_id, requested_input)
            .ok()?;
        Some(self.record_event(WorldEventKind::Power(power_event)))
    }

    /// Discharge a power storage facility, adding electricity to its location.
    /// Returns the power event if discharging occurred.
    pub fn discharge_power_storage(
        &mut self,
        storage_id: &FacilityId,
        requested_output: i64,
    ) -> Option<WorldEvent> {
        let power_event = self
            .discharge_power_storage_event(storage_id, requested_output)
            .ok()?;
        Some(self.record_event(WorldEventKind::Power(power_event)))
    }

    pub(super) fn charge_power_storage_event(
        &mut self,
        storage_id: &FacilityId,
        requested_input: i64,
    ) -> Result<PowerEvent, RejectReason> {
        if requested_input <= 0 {
            return Err(RejectReason::InvalidAmount {
                amount: requested_input,
            });
        }
        let location_id = self
            .model
            .power_storages
            .get(storage_id)
            .map(|storage| storage.location_id.clone())
            .ok_or_else(|| RejectReason::FacilityNotFound {
                facility_id: storage_id.clone(),
            })?;
        let available = self
            .model
            .locations
            .get(&location_id)
            .map(|location| location.resources.get(ResourceKind::Electricity))
            .ok_or_else(|| RejectReason::LocationNotFound {
                location_id: location_id.clone(),
            })?;
        if available <= 0 {
            return Err(RejectReason::InsufficientResource {
                owner: ResourceOwner::Location { location_id },
                kind: ResourceKind::Electricity,
                requested: requested_input,
                available,
            });
        }
        let (input, stored) = {
            let storage = self
                .model
                .power_storages
                .get_mut(storage_id)
                .ok_or_else(|| RejectReason::FacilityNotFound {
                    facility_id: storage_id.clone(),
                })?;
            let remaining_capacity = storage.capacity - storage.current_level;
            if remaining_capacity <= 0 {
                return Err(RejectReason::InvalidAmount {
                    amount: requested_input,
                });
            }
            if storage.charge_efficiency <= 0.0 {
                return Err(RejectReason::InvalidAmount {
                    amount: requested_input,
                });
            }
            let max_input_by_capacity =
                (remaining_capacity as f64 / storage.charge_efficiency).floor() as i64;
            let input = requested_input
                .min(available)
                .min(storage.max_charge_rate)
                .min(max_input_by_capacity);
            if input <= 0 {
                return Err(RejectReason::InvalidAmount {
                    amount: requested_input,
                });
            }
            let stored = (input as f64 * storage.charge_efficiency).floor() as i64;
            if stored <= 0 {
                return Err(RejectReason::InvalidAmount {
                    amount: requested_input,
                });
            }
            storage.current_level = storage.current_level.saturating_add(stored);
            (input, stored)
        };

        let location = self.model.locations.get_mut(&location_id).ok_or_else(|| {
            RejectReason::LocationNotFound {
                location_id: location_id.clone(),
            }
        })?;
        location
            .resources
            .remove(ResourceKind::Electricity, input)
            .map_err(|err| match err {
                StockError::NegativeAmount { amount } => RejectReason::InvalidAmount { amount },
                StockError::Insufficient {
                    requested,
                    available,
                    ..
                } => RejectReason::InsufficientResource {
                    owner: ResourceOwner::Location {
                        location_id: location_id.clone(),
                    },
                    kind: ResourceKind::Electricity,
                    requested,
                    available,
                },
            })?;

        Ok(PowerEvent::PowerStored {
            storage_id: storage_id.clone(),
            location_id,
            input,
            stored,
        })
    }

    pub(super) fn discharge_power_storage_event(
        &mut self,
        storage_id: &FacilityId,
        requested_output: i64,
    ) -> Result<PowerEvent, RejectReason> {
        if requested_output <= 0 {
            return Err(RejectReason::InvalidAmount {
                amount: requested_output,
            });
        }
        let location_id = self
            .model
            .power_storages
            .get(storage_id)
            .map(|storage| storage.location_id.clone())
            .ok_or_else(|| RejectReason::FacilityNotFound {
                facility_id: storage_id.clone(),
            })?;
        if !self.model.locations.contains_key(&location_id) {
            return Err(RejectReason::LocationNotFound { location_id });
        }
        let (output, drawn) = {
            let storage = self
                .model
                .power_storages
                .get_mut(storage_id)
                .ok_or_else(|| RejectReason::FacilityNotFound {
                    facility_id: storage_id.clone(),
                })?;
            if storage.discharge_efficiency <= 0.0 {
                return Err(RejectReason::InvalidAmount {
                    amount: requested_output,
                });
            }
            let max_output_by_storage =
                (storage.current_level as f64 * storage.discharge_efficiency).floor() as i64;
            let output = requested_output
                .min(storage.max_discharge_rate)
                .min(max_output_by_storage);
            if output <= 0 {
                return Err(RejectReason::InvalidAmount {
                    amount: requested_output,
                });
            }
            let drawn = (output as f64 / storage.discharge_efficiency).ceil() as i64;
            let drawn = drawn.min(storage.current_level);
            if drawn <= 0 {
                return Err(RejectReason::InvalidAmount {
                    amount: requested_output,
                });
            }
            storage.current_level = storage.current_level.saturating_sub(drawn);
            (output, drawn)
        };

        let location = self.model.locations.get_mut(&location_id).ok_or_else(|| {
            RejectReason::LocationNotFound {
                location_id: location_id.clone(),
            }
        })?;
        location
            .resources
            .add(ResourceKind::Electricity, output)
            .map_err(|err| match err {
                StockError::NegativeAmount { amount } => RejectReason::InvalidAmount { amount },
                StockError::Insufficient { .. } => RejectReason::InvalidAmount { amount: output },
            })?;

        Ok(PowerEvent::PowerDischarged {
            storage_id: storage_id.clone(),
            location_id,
            output,
            drawn,
        })
    }

    /// Consume power from an agent for a specific reason.
    /// Returns the power event if power was consumed.
    pub fn consume_agent_power(
        &mut self,
        agent_id: &AgentId,
        amount: i64,
        reason: ConsumeReason,
    ) -> Option<WorldEvent> {
        let power_config = self.config.power.clone();

        let (consumed, remaining, old_state, new_state) = {
            let agent = self.model.agents.get_mut(agent_id)?;

            if agent.power.is_shutdown() {
                return None;
            }

            let old_state = agent.power.state;
            let consumed = agent.power.consume(amount, &power_config);
            let new_state = agent.power.state;

            if consumed == 0 {
                return None;
            }

            (consumed, agent.power.level, old_state, new_state)
        };

        let power_event = PowerEvent::PowerConsumed {
            agent_id: agent_id.clone(),
            amount: consumed,
            reason,
            remaining,
        };
        let event = self.record_event(super::types::WorldEventKind::Power(power_event));

        if old_state != new_state {
            let state_event = PowerEvent::PowerStateChanged {
                agent_id: agent_id.clone(),
                from: old_state,
                to: new_state,
                trigger_level: remaining,
            };
            self.record_event(super::types::WorldEventKind::Power(state_event));
        }

        Some(event)
    }

    /// Charge an agent's power.
    /// Returns the power event if power was added.
    pub fn charge_agent_power(&mut self, agent_id: &AgentId, amount: i64) -> Option<WorldEvent> {
        let power_config = self.config.power.clone();

        let (added, new_level, old_state, new_state) = {
            let agent = self.model.agents.get_mut(agent_id)?;

            let old_state = agent.power.state;
            let added = agent.power.charge(amount, &power_config);
            let new_state = agent.power.state;

            if added == 0 {
                return None;
            }

            (added, agent.power.level, old_state, new_state)
        };

        let power_event = PowerEvent::PowerCharged {
            agent_id: agent_id.clone(),
            amount: added,
            new_level,
        };
        let event = self.record_event(super::types::WorldEventKind::Power(power_event));

        if old_state != new_state {
            let state_event = PowerEvent::PowerStateChanged {
                agent_id: agent_id.clone(),
                from: old_state,
                to: new_state,
                trigger_level: new_level,
            };
            self.record_event(super::types::WorldEventKind::Power(state_event));
        }

        Some(event)
    }

    /// Get the power state of an agent.
    pub fn agent_power_state(&self, agent_id: &AgentId) -> Option<AgentPowerState> {
        self.model.agents.get(agent_id).map(|a| a.power.state)
    }

    /// Check if an agent is shut down.
    pub fn is_agent_shutdown(&self, agent_id: &AgentId) -> bool {
        self.model
            .agents
            .get(agent_id)
            .map(|a| a.power.is_shutdown())
            .unwrap_or(false)
    }

    /// Get all shutdown agents.
    pub fn shutdown_agents(&self) -> Vec<AgentId> {
        self.model
            .agents
            .iter()
            .filter(|(_, a)| a.power.is_shutdown())
            .map(|(id, _)| id.clone())
            .collect()
    }
}
