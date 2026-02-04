use crate::geometry::{great_circle_distance_cm, GeoPos};
use crate::models::RobotBodySpec;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};

pub type AgentId = String;
pub type LocationId = String;
pub type AssetId = String;
pub type WorldTime = u64;
pub type WorldEventId = u64;
pub type ActionId = u64;

pub const CM_PER_KM: i64 = 100_000;
pub const DEFAULT_VISIBILITY_RANGE_CM: i64 = 10_000_000;
pub const MOVE_COST_PER_KM_ELECTRICITY: i64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    Electricity,
    Hardware,
    Data,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ResourceStock {
    pub amounts: BTreeMap<ResourceKind, i64>,
}

impl ResourceStock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, kind: ResourceKind) -> i64 {
        *self.amounts.get(&kind).unwrap_or(&0)
    }

    pub fn set(&mut self, kind: ResourceKind, amount: i64) -> Result<(), StockError> {
        if amount < 0 {
            return Err(StockError::NegativeAmount { amount });
        }
        if amount == 0 {
            self.amounts.remove(&kind);
        } else {
            self.amounts.insert(kind, amount);
        }
        Ok(())
    }

    pub fn add(&mut self, kind: ResourceKind, amount: i64) -> Result<(), StockError> {
        if amount < 0 {
            return Err(StockError::NegativeAmount { amount });
        }
        let current = self.get(kind);
        self.set(kind, current + amount)
    }

    pub fn remove(&mut self, kind: ResourceKind, amount: i64) -> Result<(), StockError> {
        if amount < 0 {
            return Err(StockError::NegativeAmount { amount });
        }
        let current = self.get(kind);
        if current < amount {
            return Err(StockError::Insufficient {
                kind,
                requested: amount,
                available: current,
            });
        }
        self.set(kind, current - amount)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StockError {
    NegativeAmount { amount: i64 },
    Insufficient {
        kind: ResourceKind,
        requested: i64,
        available: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ResourceOwner {
    Agent { agent_id: AgentId },
    Location { location_id: LocationId },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Agent {
    pub id: AgentId,
    pub pos: GeoPos,
    pub body: RobotBodySpec,
    pub location_id: LocationId,
    pub resources: ResourceStock,
}

impl Agent {
    pub fn new(id: impl Into<String>, location_id: impl Into<String>, pos: GeoPos) -> Self {
        Self {
            id: id.into(),
            pos,
            body: RobotBodySpec::default(),
            location_id: location_id.into(),
            resources: ResourceStock::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub id: LocationId,
    pub name: String,
    pub pos: GeoPos,
    pub resources: ResourceStock,
}

impl Location {
    pub fn new(id: impl Into<String>, name: impl Into<String>, pos: GeoPos) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            pos,
            resources: ResourceStock::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Asset {
    pub id: AssetId,
    pub owner: ResourceOwner,
    pub kind: AssetKind,
    pub quantity: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AssetKind {
    Resource { kind: ResourceKind },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorldModel {
    pub agents: BTreeMap<AgentId, Agent>,
    pub locations: BTreeMap<LocationId, Location>,
    pub assets: BTreeMap<AssetId, Asset>,
}

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionEnvelope {
    pub id: ActionId,
    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Action {
    RegisterLocation {
        location_id: LocationId,
        name: String,
        pos: GeoPos,
    },
    RegisterAgent {
        agent_id: AgentId,
        location_id: LocationId,
    },
    MoveAgent {
        agent_id: AgentId,
        to: LocationId,
    },
    TransferResource {
        from: ResourceOwner,
        to: ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct WorldKernel {
    time: WorldTime,
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

    pub fn time(&self) -> WorldTime {
        self.time
    }

    pub fn model(&self) -> &WorldModel {
        &self.model
    }

    pub fn journal(&self) -> &[WorldEvent] {
        &self.journal
    }

    pub fn observe(&self, agent_id: &str) -> Result<Observation, RejectReason> {
        let Some(agent) = self.model.agents.get(agent_id) else {
            return Err(RejectReason::AgentNotFound {
                agent_id: agent_id.to_string(),
            });
        };
        let visibility_range_cm = DEFAULT_VISIBILITY_RANGE_CM;
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
                let location = Location::new(location_id.clone(), name, pos);
                self.model.locations.insert(location_id.clone(), location);
                WorldEventKind::LocationRegistered { location_id, pos }
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
                let agent = Agent::new(agent_id.clone(), location_id.clone(), location.pos);
                self.model.agents.insert(agent_id.clone(), agent);
                WorldEventKind::AgentRegistered {
                    agent_id,
                    location_id,
                    pos: location.pos,
                }
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
                let electricity_cost = movement_cost(distance_cm);
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

    fn owner_stock(&self, owner: &ResourceOwner) -> Option<&ResourceStock> {
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

fn movement_cost(distance_cm: i64) -> i64 {
    if distance_cm <= 0 {
        return 0;
    }
    let km = (distance_cm + CM_PER_KM - 1) / CM_PER_KM;
    km.saturating_mul(MOVE_COST_PER_KM_ELECTRICITY)
}

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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum RejectReason {
    AgentAlreadyExists { agent_id: AgentId },
    AgentNotFound { agent_id: AgentId },
    LocationAlreadyExists { location_id: LocationId },
    LocationNotFound { location_id: LocationId },
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DEFAULT_AGENT_HEIGHT_CM;

    fn pos(lat: f64, lon: f64) -> GeoPos {
        GeoPos {
            lat_deg: lat,
            lon_deg: lon,
        }
    }

    #[test]
    fn resource_stock_add_remove() {
        let mut stock = ResourceStock::new();
        stock.add(ResourceKind::Electricity, 10).unwrap();
        stock.add(ResourceKind::Electricity, 5).unwrap();
        assert_eq!(stock.get(ResourceKind::Electricity), 15);

        stock.remove(ResourceKind::Electricity, 6).unwrap();
        assert_eq!(stock.get(ResourceKind::Electricity), 9);

        let err = stock.remove(ResourceKind::Electricity, 20).unwrap_err();
        assert!(matches!(err, StockError::Insufficient { .. }));
    }

    #[test]
    fn agent_and_location_defaults() {
        let position = pos(0.0, 0.0);
        let location = Location::new("loc-1", "base", position);
        let agent = Agent::new("agent-1", "loc-1", position);

        assert_eq!(location.id, "loc-1");
        assert_eq!(agent.location_id, "loc-1");
        assert_eq!(agent.body.height_cm, DEFAULT_AGENT_HEIGHT_CM);
    }

    #[test]
    fn world_model_starts_empty() {
        let model = WorldModel::default();
        assert!(model.agents.is_empty());
        assert!(model.locations.is_empty());
        assert!(model.assets.is_empty());
    }

    #[test]
    fn kernel_registers_and_moves_agent() {
        let mut kernel = WorldKernel::new();
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "outpost".to_string(),
            pos: pos(1.0, 1.0),
        });
        kernel.step_until_empty();

        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step().unwrap();
        let starting_energy = 500;
        kernel
            .model
            .agents
            .get_mut("agent-1")
            .unwrap()
            .resources
            .add(ResourceKind::Electricity, starting_energy)
            .unwrap();

        kernel.submit_action(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-2".to_string(),
        });
        let event = kernel.step().unwrap();
        let recorded_cost = match event.kind {
            WorldEventKind::AgentMoved {
                agent_id,
                from,
                to,
                distance_cm,
                electricity_cost,
            } => {
                assert_eq!(agent_id, "agent-1");
                assert_eq!(from, "loc-1");
                assert_eq!(to, "loc-2");
                assert!(distance_cm > 0);
                assert_eq!(electricity_cost, movement_cost(distance_cm));
                electricity_cost
            }
            other => panic!("unexpected event: {other:?}"),
        };

        let agent = kernel.model.agents.get("agent-1").unwrap();
        assert_eq!(agent.location_id, "loc-2");
        assert_eq!(agent.pos, pos(1.0, 1.0));
        assert_eq!(
            agent.resources.get(ResourceKind::Electricity),
            starting_energy - recorded_cost
        );
    }

    #[test]
    fn kernel_move_requires_energy() {
        let mut kernel = WorldKernel::new();
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "outpost".to_string(),
            pos: pos(1.0, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step_until_empty();

        kernel.submit_action(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-2".to_string(),
        });
        let event = kernel.step().unwrap();
        match event.kind {
            WorldEventKind::ActionRejected { reason } => {
                assert!(matches!(reason, RejectReason::InsufficientResource { .. }));
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn kernel_rejects_move_to_same_location() {
        let mut kernel = WorldKernel::new();
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.step_until_empty();

        kernel.submit_action(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-1".to_string(),
        });
        let event = kernel.step().unwrap();
        match event.kind {
            WorldEventKind::ActionRejected { reason } => {
                assert!(matches!(reason, RejectReason::AgentAlreadyAtLocation { .. }));
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn kernel_observe_visibility_range() {
        let mut kernel = WorldKernel::new();
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "near".to_string(),
            pos: pos(0.4, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-3".to_string(),
            name: "far".to_string(),
            pos: pos(1.5, 0.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-2".to_string(),
            location_id: "loc-2".to_string(),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-3".to_string(),
            location_id: "loc-3".to_string(),
        });
        kernel.step_until_empty();

        let observation = kernel.observe("agent-1").unwrap();
        assert!(
            observation
                .visible_locations
                .iter()
                .any(|loc| loc.location_id == "loc-1")
        );
        assert!(
            observation
                .visible_locations
                .iter()
                .any(|loc| loc.location_id == "loc-2")
        );
        assert!(
            !observation
                .visible_locations
                .iter()
                .any(|loc| loc.location_id == "loc-3")
        );
        assert!(
            observation
                .visible_agents
                .iter()
                .any(|agent| agent.agent_id == "agent-2")
        );
        assert!(
            !observation
                .visible_agents
                .iter()
                .any(|agent| agent.agent_id == "agent-3")
        );
        assert_eq!(observation.visibility_range_cm, DEFAULT_VISIBILITY_RANGE_CM);
    }

    #[test]
    fn kernel_transfer_requires_colocation() {
        let mut kernel = WorldKernel::new();
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "base".to_string(),
            pos: pos(0.0, 0.0),
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "outpost".to_string(),
            pos: pos(1.0, 1.0),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-2".to_string(),
            location_id: "loc-2".to_string(),
        });
        kernel.step_until_empty();

        kernel
            .model
            .agents
            .get_mut("agent-1")
            .unwrap()
            .resources
            .add(ResourceKind::Electricity, 10)
            .unwrap();

        kernel.submit_action(Action::TransferResource {
            from: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            to: ResourceOwner::Agent {
                agent_id: "agent-2".to_string(),
            },
            kind: ResourceKind::Electricity,
            amount: 5,
        });
        let event = kernel.step().unwrap();
        match event.kind {
            WorldEventKind::ActionRejected { reason } => {
                assert!(matches!(reason, RejectReason::AgentsNotCoLocated { .. }));
            }
            other => panic!("unexpected event: {other:?}"),
        }
    }

    #[test]
    fn kernel_closed_loop_example() {
        let mut kernel = WorldKernel::new();
        let loc1_pos = pos(0.0, 0.0);
        let loc2_pos = pos(2.0, 2.0);
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-1".to_string(),
            name: "plant".to_string(),
            pos: loc1_pos,
        });
        kernel.submit_action(Action::RegisterLocation {
            location_id: "loc-2".to_string(),
            name: "lab".to_string(),
            pos: loc2_pos,
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-1".to_string(),
            location_id: "loc-1".to_string(),
        });
        kernel.submit_action(Action::RegisterAgent {
            agent_id: "agent-2".to_string(),
            location_id: "loc-2".to_string(),
        });
        kernel.step_until_empty();

        kernel
            .model
            .locations
            .get_mut("loc-1")
            .unwrap()
            .resources
            .add(ResourceKind::Electricity, 1000)
            .unwrap();
        kernel
            .model
            .locations
            .get_mut("loc-2")
            .unwrap()
            .resources
            .add(ResourceKind::Electricity, 20)
            .unwrap();

        kernel.submit_action(Action::TransferResource {
            from: ResourceOwner::Location {
                location_id: "loc-1".to_string(),
            },
            to: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            kind: ResourceKind::Electricity,
            amount: 500,
        });
        kernel.step().unwrap();
        let move_cost = movement_cost(great_circle_distance_cm(loc1_pos, loc2_pos));

        kernel.submit_action(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-2".to_string(),
        });
        kernel.step().unwrap();

        kernel.submit_action(Action::TransferResource {
            from: ResourceOwner::Location {
                location_id: "loc-1".to_string(),
            },
            to: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            kind: ResourceKind::Electricity,
            amount: 10,
        });
        let event = kernel.step().unwrap();
        match event.kind {
            WorldEventKind::ActionRejected { reason } => {
                assert!(matches!(reason, RejectReason::AgentNotAtLocation { .. }));
            }
            other => panic!("unexpected event: {other:?}"),
        }

        kernel.submit_action(Action::TransferResource {
            from: ResourceOwner::Location {
                location_id: "loc-2".to_string(),
            },
            to: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            kind: ResourceKind::Electricity,
            amount: 10,
        });
        kernel.step().unwrap();

        let agent = kernel.model.agents.get("agent-1").unwrap();
        assert_eq!(
            agent.resources.get(ResourceKind::Electricity),
            500 - move_cost + 10
        );
    }
}
