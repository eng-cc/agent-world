//! Core type definitions: IDs, constants, and resource types.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ============================================================================
// Type Aliases
// ============================================================================

pub type AgentId = String;
pub type LocationId = String;
pub type AssetId = String;
pub type FacilityId = String;
pub type WorldTime = u64;
pub type WorldEventId = u64;
pub type ActionId = u64;

// ============================================================================
// Constants
// ============================================================================

pub const CM_PER_KM: i64 = 100_000;
pub const DEFAULT_VISIBILITY_RANGE_CM: i64 = 10_000_000;
pub const DEFAULT_MOVE_COST_PER_KM_ELECTRICITY: i64 = 1;
pub const SNAPSHOT_VERSION: u32 = 2;
pub const JOURNAL_VERSION: u32 = 2;

// ============================================================================
// Resource Types
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceKind {
    Electricity,
    Hardware,
    Data,
}

// ============================================================================
// Location Physical Profile
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialKind {
    Silicate,
    Metal,
    Ice,
    Carbon,
    Composite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FragmentElementKind {
    Oxygen,
    Silicon,
    Magnesium,
    Aluminum,
    Calcium,
    Iron,
    Nickel,
    Cobalt,
    Titanium,
    Chromium,
    Hydrogen,
    Carbon,
    Nitrogen,
    Sulfur,
    Copper,
    Zinc,
    Lithium,
    Neodymium,
    Uranium,
    Thorium,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ElementComposition {
    pub ppm: BTreeMap<FragmentElementKind, u32>,
}

impl ElementComposition {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, kind: FragmentElementKind) -> u32 {
        *self.ppm.get(&kind).unwrap_or(&0)
    }

    pub fn set(&mut self, kind: FragmentElementKind, value: u32) {
        if value == 0 {
            self.ppm.remove(&kind);
        } else {
            self.ppm.insert(kind, value);
        }
    }

    pub fn total_ppm(&self) -> u64 {
        self.ppm.values().map(|value| *value as u64).sum()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocationProfile {
    pub material: MaterialKind,
    pub radius_cm: i64,
    pub radiation_emission_per_tick: i64,
}

impl Default for LocationProfile {
    fn default() -> Self {
        Self {
            material: MaterialKind::Silicate,
            radius_cm: 100,
            radiation_emission_per_tick: 0,
        }
    }
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

// ============================================================================
// Action Types
// ============================================================================

use crate::geometry::GeoPos;

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
        profile: LocationProfile,
    },
    RegisterAgent {
        agent_id: AgentId,
        location_id: LocationId,
    },
    RegisterPowerPlant {
        facility_id: FacilityId,
        location_id: LocationId,
        owner: ResourceOwner,
        capacity_per_tick: i64,
        fuel_cost_per_pu: i64,
        maintenance_cost: i64,
        efficiency: f64,
        degradation: f64,
    },
    RegisterPowerStorage {
        facility_id: FacilityId,
        location_id: LocationId,
        owner: ResourceOwner,
        capacity: i64,
        current_level: i64,
        charge_efficiency: f64,
        discharge_efficiency: f64,
        max_charge_rate: i64,
        max_discharge_rate: i64,
    },
    DrawPower {
        storage_id: FacilityId,
        amount: i64,
    },
    StorePower {
        storage_id: FacilityId,
        amount: i64,
    },
    MoveAgent {
        agent_id: AgentId,
        to: LocationId,
    },
    HarvestRadiation {
        agent_id: AgentId,
        max_amount: i64,
    },
    BuyPower {
        buyer: ResourceOwner,
        seller: ResourceOwner,
        amount: i64,
        price_per_pu: i64,
    },
    SellPower {
        seller: ResourceOwner,
        buyer: ResourceOwner,
        amount: i64,
        price_per_pu: i64,
    },
    TransferResource {
        from: ResourceOwner,
        to: ResourceOwner,
        kind: ResourceKind,
        amount: i64,
    },
}
