use super::super::ResourceDelta;
use super::super::WorldError;
use super::super::{MaterialLedgerId, MaterialStack};
use super::World;
use crate::simulator::ResourceKind;
use crate::simulator::StockError;
use std::collections::BTreeMap;

impl World {
    // ---------------------------------------------------------------------
    // Resource ledger
    // ---------------------------------------------------------------------

    pub fn resource_balance(&self, kind: ResourceKind) -> i64 {
        self.state.resources.get(&kind).copied().unwrap_or(0)
    }

    pub fn set_resource_balance(&mut self, kind: ResourceKind, amount: i64) {
        self.state.resources.insert(kind, amount);
    }

    pub fn adjust_resource_balance(&mut self, kind: ResourceKind, delta: i64) -> i64 {
        let entry = self.state.resources.entry(kind).or_insert(0);
        *entry += delta;
        *entry
    }

    pub fn material_balance(&self, material_kind: &str) -> i64 {
        self.ledger_material_balance(&MaterialLedgerId::world(), material_kind)
    }

    pub fn ledger_material_balance(
        &self,
        ledger_id: &MaterialLedgerId,
        material_kind: &str,
    ) -> i64 {
        self.state
            .material_ledgers
            .get(ledger_id)
            .and_then(|ledger| ledger.get(material_kind))
            .copied()
            .unwrap_or_default()
    }

    pub fn has_materials_in_ledger(
        &self,
        ledger_id: &MaterialLedgerId,
        consume: &[MaterialStack],
    ) -> bool {
        consume.iter().all(|stack| {
            stack.amount > 0
                && self.ledger_material_balance(ledger_id, stack.kind.as_str()) >= stack.amount
        })
    }

    pub fn ledger_material_stacks(&self, ledger_id: &MaterialLedgerId) -> Vec<MaterialStack> {
        self.state
            .material_ledgers
            .get(ledger_id)
            .map(|ledger| {
                ledger
                    .iter()
                    .filter(|(_, amount)| **amount > 0)
                    .map(|(kind, amount)| MaterialStack::new(kind.clone(), *amount))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn set_material_balance(
        &mut self,
        material_kind: impl Into<String>,
        amount: i64,
    ) -> Result<(), WorldError> {
        self.set_ledger_material_balance(MaterialLedgerId::world(), material_kind, amount)
    }

    pub fn set_ledger_material_balance(
        &mut self,
        ledger_id: MaterialLedgerId,
        material_kind: impl Into<String>,
        amount: i64,
    ) -> Result<(), WorldError> {
        if amount < 0 {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: format!("material balance must be >= 0, got {amount}"),
            });
        }
        let material_kind = material_kind.into();
        if material_kind.trim().is_empty() {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: "material kind cannot be empty".to_string(),
            });
        }
        let ledger = self
            .state
            .material_ledgers
            .entry(ledger_id)
            .or_insert_with(BTreeMap::new);
        if amount == 0 {
            ledger.remove(&material_kind);
        } else {
            ledger.insert(material_kind, amount);
        }
        self.sync_legacy_world_materials_cache();
        Ok(())
    }

    pub fn adjust_material_balance(
        &mut self,
        material_kind: impl Into<String>,
        delta: i64,
    ) -> Result<i64, WorldError> {
        self.adjust_ledger_material_balance(MaterialLedgerId::world(), material_kind, delta)
    }

    pub fn adjust_ledger_material_balance(
        &mut self,
        ledger_id: MaterialLedgerId,
        material_kind: impl Into<String>,
        delta: i64,
    ) -> Result<i64, WorldError> {
        let material_kind = material_kind.into();
        if material_kind.trim().is_empty() {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: "material kind cannot be empty".to_string(),
            });
        }
        let current = self.ledger_material_balance(&ledger_id, material_kind.as_str());
        let next = current.saturating_add(delta);
        if next < 0 {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: format!(
                    "material balance cannot be negative: kind={} current={} delta={}",
                    material_kind, current, delta
                ),
            });
        }
        let ledger = self
            .state
            .material_ledgers
            .entry(ledger_id)
            .or_insert_with(BTreeMap::new);
        if next == 0 {
            ledger.remove(&material_kind);
        } else {
            ledger.insert(material_kind, next);
        }
        self.sync_legacy_world_materials_cache();
        Ok(next)
    }

    pub fn transfer_material_between_ledgers(
        &mut self,
        from_ledger: &MaterialLedgerId,
        to_ledger: &MaterialLedgerId,
        material_kind: &str,
        amount: i64,
    ) -> Result<(), WorldError> {
        if amount <= 0 {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: format!("material transfer amount must be > 0, got {amount}"),
            });
        }
        self.adjust_ledger_material_balance(
            from_ledger.clone(),
            material_kind.to_string(),
            -amount,
        )?;
        self.adjust_ledger_material_balance(to_ledger.clone(), material_kind.to_string(), amount)?;
        Ok(())
    }

    pub(super) fn apply_resource_delta(&mut self, delta: &ResourceDelta) {
        for (kind, amount) in &delta.entries {
            self.adjust_resource_balance(*kind, *amount);
        }
    }

    pub fn agent_resource_balance(
        &self,
        agent_id: &str,
        kind: ResourceKind,
    ) -> Result<i64, WorldError> {
        let cell = self
            .state
            .agents
            .get(agent_id)
            .ok_or_else(|| WorldError::AgentNotFound {
                agent_id: agent_id.to_string(),
            })?;
        Ok(cell.state.resources.get(kind))
    }

    pub fn set_agent_resource_balance(
        &mut self,
        agent_id: &str,
        kind: ResourceKind,
        amount: i64,
    ) -> Result<(), WorldError> {
        let cell =
            self.state
                .agents
                .get_mut(agent_id)
                .ok_or_else(|| WorldError::AgentNotFound {
                    agent_id: agent_id.to_string(),
                })?;
        cell.state
            .resources
            .set(kind, amount)
            .map_err(|err| WorldError::ResourceBalanceInvalid {
                reason: format!("set resource failed: {err:?}"),
            })
    }

    pub fn adjust_agent_resource_balance(
        &mut self,
        agent_id: &str,
        kind: ResourceKind,
        delta: i64,
    ) -> Result<i64, WorldError> {
        let cell =
            self.state
                .agents
                .get_mut(agent_id)
                .ok_or_else(|| WorldError::AgentNotFound {
                    agent_id: agent_id.to_string(),
                })?;
        if delta >= 0 {
            cell.state.resources.add(kind, delta).map_err(|err| {
                WorldError::ResourceBalanceInvalid {
                    reason: format!("add resource failed: {err:?}"),
                }
            })?;
        } else {
            let amount = delta.saturating_abs();
            cell.state
                .resources
                .remove(kind, amount)
                .map_err(|err| match err {
                    StockError::NegativeAmount { .. } | StockError::Insufficient { .. } => {
                        WorldError::ResourceBalanceInvalid {
                            reason: format!("remove resource failed: {err:?}"),
                        }
                    }
                })?;
        }
        Ok(cell.state.resources.get(kind))
    }

    fn sync_legacy_world_materials_cache(&mut self) {
        self.state.materials = self
            .state
            .material_ledgers
            .get(&MaterialLedgerId::world())
            .cloned()
            .unwrap_or_default();
    }
}
