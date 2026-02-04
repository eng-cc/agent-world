//! WorldKernel: time, events, actions, and observation.

use crate::geometry::{great_circle_distance_cm, GeoPos};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::Path;

use super::persist::{PersistError, WorldJournal, WorldSnapshot};
use super::power::{
    AgentPowerState, ConsumeReason, PlantStatus, PowerEvent, PowerPlant, PowerStorage,
};
use super::types::{
    Action, ActionEnvelope, ActionId, AgentId, FacilityId, LocationId, ResourceKind, ResourceOwner,
    StockError, WorldEventId, WorldTime, JOURNAL_VERSION, SNAPSHOT_VERSION,
};
use super::world_model::{movement_cost, Agent, Location, WorldConfig, WorldModel};

// ============================================================================
// Observation Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    pub time: WorldTime,
    pub agent_id: AgentId,
    pub pos: GeoPos,
    pub visibility_range_cm: i64,
    pub visible_agents: Vec<ObservedAgent>,
    pub visible_locations: Vec<ObservedLocation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedAgent {
    pub agent_id: AgentId,
    pub location_id: LocationId,
    pub pos: GeoPos,
    pub distance_cm: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedLocation {
    pub location_id: LocationId,
    pub name: String,
    pub pos: GeoPos,
    pub distance_cm: i64,
}

// ============================================================================
// Event Types
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldEvent {
    pub id: WorldEventId,
    pub time: WorldTime,
    pub kind: WorldEventKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WorldEventKind {
    LocationRegistered {
        location_id: LocationId,
        name: String,
        pos: GeoPos,
    },
    AgentRegistered {
        agent_id: AgentId,
        location_id: LocationId,
        pos: GeoPos,
    },
    AgentMoved {
        agent_id: AgentId,
        from: LocationId,
        to: LocationId,
        distance_cm: i64,
        electricity_cost: i64,
    },
    ResourceTransferred {
        from: ResourceOwner,
        to: ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    },
    ActionRejected {
        reason: RejectReason,
    },
    // Power system events
    Power(PowerEvent),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum RejectReason {
    AgentAlreadyExists { agent_id: AgentId },
    AgentNotFound { agent_id: AgentId },
    LocationAlreadyExists { location_id: LocationId },
    LocationNotFound { location_id: LocationId },
    FacilityAlreadyExists { facility_id: FacilityId },
    AgentAlreadyAtLocation { agent_id: AgentId, location_id: LocationId },
    InvalidAmount { amount: i64 },
    InsufficientResource {
        owner: ResourceOwner,
        kind: ResourceKind,
        requested: i64,
        available: i64,
    },
    LocationTransferNotAllowed { from: LocationId, to: LocationId },
    AgentNotAtLocation { agent_id: AgentId, location_id: LocationId },
    AgentsNotCoLocated {
        agent_id: AgentId,
        other_agent_id: AgentId,
    },
    AgentShutdown {
        agent_id: AgentId,
    },
}

// ============================================================================
// WorldKernel
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorldKernel {
    time: WorldTime,
    config: WorldConfig,
    next_event_id: WorldEventId,
    next_action_id: ActionId,
    pending_actions: VecDeque<ActionEnvelope>,
    journal: Vec<WorldEvent>,
    model: WorldModel,
}

impl WorldKernel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: WorldConfig) -> Self {
        let mut kernel = Self::default();
        kernel.config = config.sanitized();
        kernel
    }

    pub fn time(&self) -> WorldTime {
        self.time
    }

    pub fn config(&self) -> &WorldConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: WorldConfig) {
        self.config = config.sanitized();
    }

    pub fn model(&self) -> &WorldModel {
        &self.model
    }

    pub fn journal(&self) -> &[WorldEvent] {
        &self.journal
    }

    pub fn snapshot(&self) -> WorldSnapshot {
        WorldSnapshot {
            version: SNAPSHOT_VERSION,
            time: self.time,
            config: self.config.clone(),
            model: self.model.clone(),
            next_event_id: self.next_event_id,
            next_action_id: self.next_action_id,
            pending_actions: self.pending_actions.iter().cloned().collect(),
            journal_len: self.journal.len(),
        }
    }

    pub fn journal_snapshot(&self) -> WorldJournal {
        WorldJournal {
            version: JOURNAL_VERSION,
            events: self.journal.clone(),
        }
    }

    pub fn from_snapshot(
        snapshot: WorldSnapshot,
        journal: WorldJournal,
    ) -> Result<Self, PersistError> {
        snapshot.validate_version()?;
        journal.validate_version()?;
        if snapshot.journal_len != journal.events.len() {
            return Err(PersistError::SnapshotMismatch {
                expected: snapshot.journal_len,
                actual: journal.events.len(),
            });
        }
        Ok(Self {
            time: snapshot.time,
            config: snapshot.config.sanitized(),
            next_event_id: snapshot.next_event_id,
            next_action_id: snapshot.next_action_id,
            pending_actions: VecDeque::from(snapshot.pending_actions),
            journal: journal.events,
            model: snapshot.model,
        })
    }

    pub fn replay_from_snapshot(
        snapshot: WorldSnapshot,
        journal: WorldJournal,
    ) -> Result<Self, PersistError> {
        snapshot.validate_version()?;
        journal.validate_version()?;
        if journal.events.len() < snapshot.journal_len {
            return Err(PersistError::SnapshotMismatch {
                expected: snapshot.journal_len,
                actual: journal.events.len(),
            });
        }
        if !snapshot.pending_actions.is_empty() && journal.events.len() > snapshot.journal_len {
            return Err(PersistError::ReplayConflict {
                message: "cannot replay with pending actions in snapshot".to_string(),
            });
        }

        let mut kernel = Self {
            time: snapshot.time,
            config: snapshot.config.sanitized(),
            next_event_id: snapshot.next_event_id,
            next_action_id: snapshot.next_action_id,
            pending_actions: VecDeque::from(snapshot.pending_actions),
            journal: journal.events.clone(),
            model: snapshot.model,
        };

        for event in journal.events.iter().skip(snapshot.journal_len) {
            kernel.apply_event(event)?;
        }
        let events_after_snapshot = journal.events.len() - snapshot.journal_len;
        if events_after_snapshot > 0 {
            kernel.next_action_id = kernel
                .next_action_id
                .saturating_add(events_after_snapshot as u64);
        }

        Ok(kernel)
    }

    pub fn save_to_dir(&self, dir: impl AsRef<Path>) -> Result<(), PersistError> {
        let dir = dir.as_ref();
        fs::create_dir_all(dir)?;
        let snapshot_path = dir.join("snapshot.json");
        let journal_path = dir.join("journal.json");
        self.snapshot().save_json(&snapshot_path)?;
        self.journal_snapshot().save_json(&journal_path)?;
        Ok(())
    }

    pub fn load_from_dir(dir: impl AsRef<Path>) -> Result<Self, PersistError> {
        let dir = dir.as_ref();
        let snapshot_path = dir.join("snapshot.json");
        let journal_path = dir.join("journal.json");
        let snapshot = WorldSnapshot::load_json(&snapshot_path)?;
        let journal = WorldJournal::load_json(&journal_path)?;
        Self::from_snapshot(snapshot, journal)
    }

    pub fn observe(&self, agent_id: &str) -> Result<Observation, RejectReason> {
        let Some(agent) = self.model.agents.get(agent_id) else {
            return Err(RejectReason::AgentNotFound {
                agent_id: agent_id.to_string(),
            });
        };
        let visibility_range_cm = self.config.visibility_range_cm;
        let mut visible_agents = Vec::new();
        for (other_id, other) in &self.model.agents {
            if other_id == agent_id {
                continue;
            }
            let distance_cm = great_circle_distance_cm(agent.pos, other.pos);
            if distance_cm <= visibility_range_cm {
                visible_agents.push(ObservedAgent {
                    agent_id: other_id.clone(),
                    location_id: other.location_id.clone(),
                    pos: other.pos,
                    distance_cm,
                });
            }
        }

        let mut visible_locations = Vec::new();
        for (location_id, location) in &self.model.locations {
            let distance_cm = great_circle_distance_cm(agent.pos, location.pos);
            if distance_cm <= visibility_range_cm {
                visible_locations.push(ObservedLocation {
                    location_id: location_id.clone(),
                    name: location.name.clone(),
                    pos: location.pos,
                    distance_cm,
                });
            }
        }

        Ok(Observation {
            time: self.time,
            agent_id: agent_id.to_string(),
            pos: agent.pos,
            visibility_range_cm,
            visible_agents,
            visible_locations,
        })
    }

    pub fn submit_action(&mut self, action: Action) -> ActionId {
        let id = self.next_action_id;
        self.next_action_id = self.next_action_id.saturating_add(1);
        self.pending_actions.push_back(ActionEnvelope { id, action });
        id
    }

    pub fn pending_actions(&self) -> usize {
        self.pending_actions.len()
    }

    pub fn step(&mut self) -> Option<WorldEvent> {
        let envelope = self.pending_actions.pop_front()?;
        self.time = self.time.saturating_add(1);
        let kind = self.apply_action(envelope.action);
        let event = WorldEvent {
            id: self.next_event_id,
            time: self.time,
            kind,
        };
        self.next_event_id = self.next_event_id.saturating_add(1);
        self.journal.push(event.clone());
        Some(event)
    }

    pub fn step_until_empty(&mut self) -> Vec<WorldEvent> {
        let mut events = Vec::new();
        while let Some(event) = self.step() {
            events.push(event);
        }
        events
    }

    // -------------------------------------------------------------------------
    // Power System
    // -------------------------------------------------------------------------

    /// Process power consumption for all agents (idle consumption).
    /// Returns a list of power events generated.
    pub fn process_power_tick(&mut self) -> Vec<WorldEvent> {
        let mut events = Vec::new();
        let idle_cost = self.config.power.idle_cost_per_tick;
        let power_config = self.config.power.clone();

        // Collect agent IDs to avoid borrow issues
        let agent_ids: Vec<AgentId> = self.model.agents.keys().cloned().collect();

        for agent_id in agent_ids {
            let (consumed, remaining, old_state, new_state) = {
                let agent = match self.model.agents.get_mut(&agent_id) {
                    Some(a) => a,
                    None => continue,
                };

                // Skip already shutdown agents
                if agent.power.is_shutdown() {
                    continue;
                }

                let old_state = agent.power.state;
                let consumed = agent.power.consume(idle_cost, &power_config);
                let new_state = agent.power.state;
                (consumed, agent.power.level, old_state, new_state)
            };

            // Record consumption event
            if consumed > 0 {
                let power_event = PowerEvent::PowerConsumed {
                    agent_id: agent_id.clone(),
                    amount: consumed,
                    reason: ConsumeReason::Idle,
                    remaining,
                };
                let event = self.record_event(WorldEventKind::Power(power_event));
                events.push(event);
            }

            // Check for state change
            if old_state != new_state {
                let power_event = PowerEvent::PowerStateChanged {
                    agent_id: agent_id.clone(),
                    from: old_state,
                    to: new_state,
                    trigger_level: remaining,
                };
                let event = self.record_event(WorldEventKind::Power(power_event));
                events.push(event);
            }
        }

        events
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

        // Record consumption event
        let power_event = PowerEvent::PowerConsumed {
            agent_id: agent_id.clone(),
            amount: consumed,
            reason,
            remaining,
        };
        let event = self.record_event(WorldEventKind::Power(power_event));

        // Check for state change
        if old_state != new_state {
            let state_event = PowerEvent::PowerStateChanged {
                agent_id: agent_id.clone(),
                from: old_state,
                to: new_state,
                trigger_level: remaining,
            };
            self.record_event(WorldEventKind::Power(state_event));
        }

        Some(event)
    }

    /// Charge an agent's power.
    /// Returns the power event if power was added.
    pub fn charge_agent_power(
        &mut self,
        agent_id: &AgentId,
        amount: i64,
    ) -> Option<WorldEvent> {
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

        // Record charge event
        let power_event = PowerEvent::PowerCharged {
            agent_id: agent_id.clone(),
            amount: added,
            new_level,
        };
        let event = self.record_event(WorldEventKind::Power(power_event));

        // Check for state change (e.g., recovering from shutdown)
        if old_state != new_state {
            let state_event = PowerEvent::PowerStateChanged {
                agent_id: agent_id.clone(),
                from: old_state,
                to: new_state,
                trigger_level: new_level,
            };
            self.record_event(WorldEventKind::Power(state_event));
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

    /// Helper to record an event and return it.
    fn record_event(&mut self, kind: WorldEventKind) -> WorldEvent {
        let event = WorldEvent {
            id: self.next_event_id,
            time: self.time,
            kind,
        };
        self.next_event_id = self.next_event_id.saturating_add(1);
        self.journal.push(event.clone());
        event
    }

    fn apply_action(&mut self, action: Action) -> WorldEventKind {
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
                // Use power config from world config
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
                // Reject if agent is shutdown
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

    fn apply_event(&mut self, event: &WorldEvent) -> Result<(), PersistError> {
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
            } => {
                if self.model.locations.contains_key(location_id) {
                    return Err(PersistError::ReplayConflict {
                        message: format!("location already exists: {location_id}"),
                    });
                }
                self.model.locations.insert(
                    location_id.clone(),
                    Location::new(location_id.clone(), name.clone(), *pos),
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
                let mut agent = Agent::new_with_power(
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
                self.ensure_owner_exists(from).map_err(|reason| {
                    PersistError::ReplayConflict {
                        message: format!("invalid transfer source: {reason:?}"),
                    }
                })?;
                self.ensure_owner_exists(to).map_err(|reason| PersistError::ReplayConflict {
                    message: format!("invalid transfer target: {reason:?}"),
                })?;
                self.remove_from_owner_for_replay(from, *kind, *amount)?;
                self.add_to_owner_for_replay(to, *kind, *amount)?;
            }
            WorldEventKind::ActionRejected { .. } => {}
            WorldEventKind::Power(power_event) => {
                // Replay power events by applying their effects
                match power_event {
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
                        self.model.power_plants.insert(plant.id.clone(), plant.clone());
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
                    PowerEvent::PowerConsumed { agent_id, amount, .. } => {
                        if let Some(agent) = self.model.agents.get_mut(agent_id) {
                            let power_config = self.config.power.clone();
                            agent.power.consume(*amount, &power_config);
                        }
                    }
                    PowerEvent::PowerCharged { agent_id, amount, .. } => {
                        if let Some(agent) = self.model.agents.get_mut(agent_id) {
                            let power_config = self.config.power.clone();
                            agent.power.charge(*amount, &power_config);
                        }
                    }
                    PowerEvent::PowerStateChanged { .. } => {
                        // State changes are derived, no action needed
                    }
                    PowerEvent::PowerTransferred { from_agent, to_agent, amount } => {
                        let power_config = self.config.power.clone();
                        if let Some(from) = self.model.agents.get_mut(from_agent) {
                            from.power.consume(*amount, &power_config);
                        }
                        if let Some(to) = self.model.agents.get_mut(to_agent) {
                            to.power.charge(*amount, &power_config);
                        }
                    }
                }
            }
        }

        Ok(())
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

    fn ensure_owner_exists(&self, owner: &ResourceOwner) -> Result<(), RejectReason> {
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

    fn owner_stock(&self, owner: &ResourceOwner) -> Option<&super::types::ResourceStock> {
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
