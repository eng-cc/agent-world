use crate::geometry::GeoPos;
use crate::models::RobotBodySpec;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type AgentId = String;
pub type LocationId = String;
pub type AssetId = String;
pub type WorldTime = u64;
pub type WorldEventId = u64;

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
    InvalidAmount { amount: i64 },
    InsufficientResource {
        owner: ResourceOwner,
        kind: ResourceKind,
        requested: i64,
        available: i64,
    },
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
        let pos = GeoPos {
            lat_deg: 0.0,
            lon_deg: 0.0,
        };
        let location = Location::new("loc-1", "base", pos);
        let agent = Agent::new("agent-1", "loc-1", pos);

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
}
