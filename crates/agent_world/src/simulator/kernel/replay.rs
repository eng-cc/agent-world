use super::super::init::{
    generate_chunk_fragments, summarize_chunk_generation, AsteroidFragmentInitConfig,
    WorldInitConfig,
};
use super::super::persist::PersistError;
use super::super::power::PowerEvent;
use super::super::types::{ResourceKind, ResourceOwner, StockError};
use super::super::world_model::Location;
use super::super::ChunkState;
use super::types::{WorldEvent, WorldEventKind};
use super::WorldKernel;

impl WorldKernel {
    pub(super) fn apply_event(&mut self, event: &WorldEvent) -> Result<(), PersistError> {
        if event.id != self.next_event_id {
            return Err(PersistError::ReplayConflict {
                message: format!(
                    "event id mismatch: expected {}, got {}",
                    self.next_event_id, event.id
                ),
            });
        }
        if event.time < self.time {
            return Err(PersistError::ReplayConflict {
                message: format!(
                    "event time regression: current {}, got {}",
                    self.time, event.time
                ),
            });
        }
        self.time = event.time;
        self.next_event_id = self.next_event_id.saturating_add(1);

        match &event.kind {
            WorldEventKind::LocationRegistered {
                location_id,
                name,
                pos,
                profile,
            } => {
                if self.model.locations.contains_key(location_id) {
                    return Err(PersistError::ReplayConflict {
                        message: format!("location already exists: {location_id}"),
                    });
                }
                self.model.locations.insert(
                    location_id.clone(),
                    Location::new_with_profile(
                        location_id.clone(),
                        name.clone(),
                        *pos,
                        profile.clone(),
                    ),
                );
            }
            WorldEventKind::AgentRegistered {
                agent_id,
                location_id,
                pos,
            } => {
                if self.model.agents.contains_key(agent_id) {
                    return Err(PersistError::ReplayConflict {
                        message: format!("agent already exists: {agent_id}"),
                    });
                }
                if !self.model.locations.contains_key(location_id) {
                    return Err(PersistError::ReplayConflict {
                        message: format!("location not found: {location_id}"),
                    });
                }
                let mut agent = super::super::world_model::Agent::new_with_power(
                    agent_id.clone(),
                    location_id.clone(),
                    *pos,
                    &self.config.power,
                );
                agent.pos = *pos;
                self.model.agents.insert(agent_id.clone(), agent);
            }
            WorldEventKind::AgentMoved {
                agent_id,
                from,
                to,
                electricity_cost,
                ..
            } => {
                let Some(location) = self.model.locations.get(to) else {
                    return Err(PersistError::ReplayConflict {
                        message: format!("location not found: {to}"),
                    });
                };
                let Some(agent) = self.model.agents.get_mut(agent_id) else {
                    return Err(PersistError::ReplayConflict {
                        message: format!("agent not found: {agent_id}"),
                    });
                };
                if &agent.location_id != from {
                    return Err(PersistError::ReplayConflict {
                        message: format!("agent {agent_id} not at expected location {from}"),
                    });
                }
                if *electricity_cost > 0 {
                    let available = agent.resources.get(ResourceKind::Electricity);
                    if available < *electricity_cost {
                        return Err(PersistError::ReplayConflict {
                            message: format!(
                                "insufficient electricity for move: requested {electricity_cost}, available {available}"
                            ),
                        });
                    }
                    if let Err(err) = agent
                        .resources
                        .remove(ResourceKind::Electricity, *electricity_cost)
                    {
                        return Err(PersistError::ReplayConflict {
                            message: format!("failed to apply move cost: {err:?}"),
                        });
                    }
                }
                agent.location_id = to.clone();
                agent.pos = location.pos;
            }
            WorldEventKind::ResourceTransferred {
                from,
                to,
                kind,
                amount,
            } => {
                if *amount <= 0 {
                    return Err(PersistError::ReplayConflict {
                        message: "transfer amount must be positive".to_string(),
                    });
                }
                self.ensure_owner_exists(from)
                    .map_err(|reason| PersistError::ReplayConflict {
                        message: format!("invalid transfer source: {reason:?}"),
                    })?;
                self.ensure_owner_exists(to)
                    .map_err(|reason| PersistError::ReplayConflict {
                        message: format!("invalid transfer target: {reason:?}"),
                    })?;
                self.remove_from_owner_for_replay(from, *kind, *amount)?;
                self.add_to_owner_for_replay(to, *kind, *amount)?;
            }
            WorldEventKind::RadiationHarvested {
                agent_id, amount, ..
            } => {
                let Some(agent) = self.model.agents.get_mut(agent_id) else {
                    return Err(PersistError::ReplayConflict {
                        message: format!("agent not found: {agent_id}"),
                    });
                };
                agent
                    .resources
                    .add(ResourceKind::Electricity, *amount)
                    .map_err(|err| PersistError::ReplayConflict {
                        message: format!("failed to apply radiation harvest: {err:?}"),
                    })?;
            }
            WorldEventKind::CompoundRefined {
                owner,
                compound_mass_g,
                electricity_cost,
                hardware_output,
            } => {
                if *compound_mass_g <= 0 || *electricity_cost < 0 || *hardware_output <= 0 {
                    return Err(PersistError::ReplayConflict {
                        message: format!(
                            "invalid refine event values: mass={}, electricity_cost={}, hardware_output={}",
                            compound_mass_g, electricity_cost, hardware_output
                        ),
                    });
                }
                self.ensure_owner_exists(owner)
                    .map_err(|reason| PersistError::ReplayConflict {
                        message: format!("invalid refine owner: {reason:?}"),
                    })?;
                self.remove_from_owner_for_replay(
                    owner,
                    ResourceKind::Electricity,
                    *electricity_cost,
                )?;
                self.add_to_owner_for_replay(owner, ResourceKind::Hardware, *hardware_output)?;
            }
            WorldEventKind::ChunkGenerated {
                coord,
                seed,
                fragment_count,
                block_count,
                chunk_budget,
                ..
            } => {
                if !self.model.chunks.contains_key(coord) {
                    self.model.chunks.insert(*coord, ChunkState::Unexplored);
                }

                let actual = if self.chunk_runtime.asteroid_fragment_enabled {
                    let init = WorldInitConfig {
                        seed: self.chunk_runtime.world_seed,
                        asteroid_fragment: AsteroidFragmentInitConfig {
                            enabled: self.chunk_runtime.asteroid_fragment_enabled,
                            seed_offset: self.chunk_runtime.asteroid_fragment_seed_offset,
                            min_fragment_spacing_cm: self.chunk_runtime.min_fragment_spacing_cm,
                            bootstrap_chunks: Vec::new(),
                        },
                        ..WorldInitConfig::default()
                    };
                    generate_chunk_fragments(
                        &mut self.model,
                        &self.config,
                        &init,
                        *coord,
                        Some(self.chunk_runtime.asteroid_fragment_seed()),
                    )
                    .map_err(|err| PersistError::ReplayConflict {
                        message: format!(
                            "chunk generation failed during replay at ({}, {}, {}): {err:?}",
                            coord.x, coord.y, coord.z
                        ),
                    })?
                } else {
                    self.model.chunks.insert(*coord, ChunkState::Generated);
                    self.model.chunk_resource_budgets.entry(*coord).or_default();
                    summarize_chunk_generation(&self.model, &self.config, *coord, *seed)
                };

                if actual.seed != *seed
                    || actual.fragment_count != *fragment_count
                    || actual.block_count != *block_count
                    || actual.chunk_budget != *chunk_budget
                {
                    return Err(PersistError::ReplayConflict {
                        message: format!(
                            "chunk replay mismatch at ({}, {}, {}): expected seed={}, fragments={}, blocks={}, budget={:?}; actual seed={}, fragments={}, blocks={}, budget={:?}",
                            coord.x,
                            coord.y,
                            coord.z,
                            seed,
                            fragment_count,
                            block_count,
                            chunk_budget,
                            actual.seed,
                            actual.fragment_count,
                            actual.block_count,
                            actual.chunk_budget
                        ),
                    });
                }
            }
            WorldEventKind::FragmentsReplenished { entries } => {
                self.apply_fragment_replenished_entries(entries)
                    .map_err(|err| PersistError::ReplayConflict {
                        message: format!("failed to apply fragment replenish event: {err}"),
                    })?;
            }
            WorldEventKind::AgentPromptUpdated { profile, .. } => {
                if !self.model.agents.contains_key(&profile.agent_id) {
                    return Err(PersistError::ReplayConflict {
                        message: format!(
                            "agent not found for prompt profile: {}",
                            profile.agent_id
                        ),
                    });
                }
                self.model
                    .agent_prompt_profiles
                    .insert(profile.agent_id.clone(), profile.clone());
            }
            WorldEventKind::ActionRejected { .. } => {}
            WorldEventKind::ModuleVisualEntityUpserted { entity } => {
                if entity.entity_id.trim().is_empty() || entity.module_id.trim().is_empty() {
                    return Err(PersistError::ReplayConflict {
                        message: "invalid module visual entity payload".to_string(),
                    });
                }
                self.ensure_module_visual_anchor_exists(&entity.anchor)
                    .map_err(|reason| PersistError::ReplayConflict {
                        message: format!(
                            "module visual entity anchor missing for {}: {reason:?}",
                            entity.entity_id
                        ),
                    })?;
                self.model
                    .module_visual_entities
                    .insert(entity.entity_id.clone(), entity.clone());
            }
            WorldEventKind::ModuleVisualEntityRemoved { entity_id } => {
                if self
                    .model
                    .module_visual_entities
                    .remove(entity_id)
                    .is_none()
                {
                    return Err(PersistError::ReplayConflict {
                        message: format!("module visual entity not found: {entity_id}"),
                    });
                }
            }
            WorldEventKind::Power(power_event) => match power_event {
                PowerEvent::PowerPlantRegistered { plant } => {
                    if self.model.power_plants.contains_key(&plant.id)
                        || self.model.power_storages.contains_key(&plant.id)
                    {
                        return Err(PersistError::ReplayConflict {
                            message: format!("power plant already exists: {}", plant.id),
                        });
                    }
                    if !self.model.locations.contains_key(&plant.location_id) {
                        return Err(PersistError::ReplayConflict {
                            message: format!(
                                "location not found for power plant: {}",
                                plant.location_id
                            ),
                        });
                    }
                    self.ensure_owner_exists(&plant.owner).map_err(|reason| {
                        PersistError::ReplayConflict {
                            message: format!("invalid power plant owner: {reason:?}"),
                        }
                    })?;
                    self.model
                        .power_plants
                        .insert(plant.id.clone(), plant.clone());
                }
                PowerEvent::PowerStorageRegistered { storage } => {
                    if self.model.power_plants.contains_key(&storage.id)
                        || self.model.power_storages.contains_key(&storage.id)
                    {
                        return Err(PersistError::ReplayConflict {
                            message: format!("power storage already exists: {}", storage.id),
                        });
                    }
                    if !self.model.locations.contains_key(&storage.location_id) {
                        return Err(PersistError::ReplayConflict {
                            message: format!(
                                "location not found for power storage: {}",
                                storage.location_id
                            ),
                        });
                    }
                    self.ensure_owner_exists(&storage.owner).map_err(|reason| {
                        PersistError::ReplayConflict {
                            message: format!("invalid power storage owner: {reason:?}"),
                        }
                    })?;
                    self.model
                        .power_storages
                        .insert(storage.id.clone(), storage.clone());
                }
                PowerEvent::PowerGenerated {
                    plant_id,
                    location_id,
                    amount,
                } => {
                    if *amount < 0 {
                        return Err(PersistError::ReplayConflict {
                            message: format!("invalid power generated amount: {amount}"),
                        });
                    }
                    let plant = self.model.power_plants.get_mut(plant_id).ok_or_else(|| {
                        PersistError::ReplayConflict {
                            message: format!("power plant not found: {plant_id}"),
                        }
                    })?;
                    if &plant.location_id != location_id {
                        return Err(PersistError::ReplayConflict {
                            message: format!(
                                "power plant location mismatch: expected {}, got {}",
                                plant.location_id, location_id
                            ),
                        });
                    }
                    let location = self.model.locations.get_mut(location_id).ok_or_else(|| {
                        PersistError::ReplayConflict {
                            message: format!("location not found: {location_id}"),
                        }
                    })?;
                    location
                        .resources
                        .add(ResourceKind::Electricity, *amount)
                        .map_err(|err| PersistError::ReplayConflict {
                            message: format!("failed to apply power generation: {err:?}"),
                        })?;
                    plant.current_output = *amount;
                }
                PowerEvent::PowerStored {
                    storage_id,
                    location_id,
                    input,
                    stored,
                } => {
                    if *input < 0 || *stored < 0 {
                        return Err(PersistError::ReplayConflict {
                            message: format!(
                                "invalid power stored values: input {input}, stored {stored}"
                            ),
                        });
                    }
                    let storage =
                        self.model
                            .power_storages
                            .get_mut(storage_id)
                            .ok_or_else(|| PersistError::ReplayConflict {
                                message: format!("power storage not found: {storage_id}"),
                            })?;
                    if &storage.location_id != location_id {
                        return Err(PersistError::ReplayConflict {
                            message: format!(
                                "power storage location mismatch: expected {}, got {}",
                                storage.location_id, location_id
                            ),
                        });
                    }
                    let location = self.model.locations.get_mut(location_id).ok_or_else(|| {
                        PersistError::ReplayConflict {
                            message: format!("location not found: {location_id}"),
                        }
                    })?;
                    location
                        .resources
                        .remove(ResourceKind::Electricity, *input)
                        .map_err(|err| PersistError::ReplayConflict {
                            message: format!("failed to apply power storage input: {err:?}"),
                        })?;
                    if storage.current_level.saturating_add(*stored) > storage.capacity {
                        return Err(PersistError::ReplayConflict {
                            message: "power storage capacity exceeded".to_string(),
                        });
                    }
                    storage.current_level = storage.current_level.saturating_add(*stored);
                }
                PowerEvent::PowerDischarged {
                    storage_id,
                    location_id,
                    output,
                    drawn,
                } => {
                    if *output < 0 || *drawn < 0 {
                        return Err(PersistError::ReplayConflict {
                            message: format!(
                                "invalid power discharged values: output {output}, drawn {drawn}"
                            ),
                        });
                    }
                    let storage =
                        self.model
                            .power_storages
                            .get_mut(storage_id)
                            .ok_or_else(|| PersistError::ReplayConflict {
                                message: format!("power storage not found: {storage_id}"),
                            })?;
                    if &storage.location_id != location_id {
                        return Err(PersistError::ReplayConflict {
                            message: format!(
                                "power storage location mismatch: expected {}, got {}",
                                storage.location_id, location_id
                            ),
                        });
                    }
                    if storage.current_level < *drawn {
                        return Err(PersistError::ReplayConflict {
                            message: "power storage underflow".to_string(),
                        });
                    }
                    storage.current_level = storage.current_level.saturating_sub(*drawn);
                    let location = self.model.locations.get_mut(location_id).ok_or_else(|| {
                        PersistError::ReplayConflict {
                            message: format!("location not found: {location_id}"),
                        }
                    })?;
                    location
                        .resources
                        .add(ResourceKind::Electricity, *output)
                        .map_err(|err| PersistError::ReplayConflict {
                            message: format!("failed to apply power discharge output: {err:?}"),
                        })?;
                }
                PowerEvent::PowerConsumed {
                    agent_id, amount, ..
                } => {
                    if let Some(agent) = self.model.agents.get_mut(agent_id) {
                        let power_config = self.config.power.clone();
                        agent.power.consume(*amount, &power_config);
                    }
                }
                PowerEvent::PowerCharged {
                    agent_id, amount, ..
                } => {
                    if let Some(agent) = self.model.agents.get_mut(agent_id) {
                        let power_config = self.config.power.clone();
                        agent.power.charge(*amount, &power_config);
                    }
                }
                PowerEvent::PowerStateChanged { .. } => {}
                PowerEvent::PowerTransferred {
                    from,
                    to,
                    amount,
                    loss,
                    ..
                } => {
                    if *amount < 0 || *loss < 0 || *loss > *amount {
                        return Err(PersistError::ReplayConflict {
                            message: format!(
                                "invalid power transfer values: amount {amount}, loss {loss}"
                            ),
                        });
                    }
                    self.ensure_owner_exists(from).map_err(|reason| {
                        PersistError::ReplayConflict {
                            message: format!("invalid power transfer source: {reason:?}"),
                        }
                    })?;
                    self.ensure_owner_exists(to).map_err(|reason| {
                        PersistError::ReplayConflict {
                            message: format!("invalid power transfer target: {reason:?}"),
                        }
                    })?;
                    self.remove_from_owner_for_replay(from, ResourceKind::Electricity, *amount)?;
                    let delivered = amount.saturating_sub(*loss);
                    if delivered <= 0 {
                        return Err(PersistError::ReplayConflict {
                            message: format!(
                                "power transfer delivered amount must be positive: {delivered}"
                            ),
                        });
                    }
                    self.add_to_owner_for_replay(to, ResourceKind::Electricity, delivered)?;
                }
            },
            WorldEventKind::LlmEffectQueued { .. } => {}
            WorldEventKind::LlmReceiptAppended { .. } => {}
        }

        Ok(())
    }

    fn remove_from_owner_for_replay(
        &mut self,
        owner: &ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    ) -> Result<(), PersistError> {
        let stock = match owner {
            ResourceOwner::Agent { agent_id } => self
                .model
                .agents
                .get_mut(agent_id)
                .map(|agent| &mut agent.resources)
                .ok_or_else(|| PersistError::ReplayConflict {
                    message: format!("agent not found: {agent_id}"),
                })?,
            ResourceOwner::Location { location_id } => self
                .model
                .locations
                .get_mut(location_id)
                .map(|location| &mut location.resources)
                .ok_or_else(|| PersistError::ReplayConflict {
                    message: format!("location not found: {location_id}"),
                })?,
        };

        stock.remove(kind, amount).map_err(|err| match err {
            StockError::NegativeAmount { amount } => PersistError::ReplayConflict {
                message: format!("invalid transfer amount: {amount}"),
            },
            StockError::Insufficient {
                requested,
                available,
                ..
            } => PersistError::ReplayConflict {
                message: format!(
                    "insufficient resource {:?}: requested {requested}, available {available}",
                    kind
                ),
            },
        })
    }

    fn add_to_owner_for_replay(
        &mut self,
        owner: &ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    ) -> Result<(), PersistError> {
        let stock = match owner {
            ResourceOwner::Agent { agent_id } => self
                .model
                .agents
                .get_mut(agent_id)
                .map(|agent| &mut agent.resources)
                .ok_or_else(|| PersistError::ReplayConflict {
                    message: format!("agent not found: {agent_id}"),
                })?,
            ResourceOwner::Location { location_id } => self
                .model
                .locations
                .get_mut(location_id)
                .map(|location| &mut location.resources)
                .ok_or_else(|| PersistError::ReplayConflict {
                    message: format!("location not found: {location_id}"),
                })?,
        };

        stock.add(kind, amount).map_err(|err| match err {
            StockError::NegativeAmount { amount } => PersistError::ReplayConflict {
                message: format!("invalid transfer amount: {amount}"),
            },
            StockError::Insufficient { .. } => PersistError::ReplayConflict {
                message: format!("invalid transfer amount: {amount}"),
            },
        })
    }
}
