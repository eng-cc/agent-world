use super::super::ResourceDelta;
use super::super::WorldError;
use super::World;
use crate::simulator::ResourceKind;
use crate::simulator::StockError;

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
        self.state
            .materials
            .get(material_kind)
            .copied()
            .unwrap_or_default()
    }

    pub fn set_material_balance(
        &mut self,
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
        if amount == 0 {
            self.state.materials.remove(&material_kind);
        } else {
            self.state.materials.insert(material_kind, amount);
        }
        Ok(())
    }

    pub fn adjust_material_balance(
        &mut self,
        material_kind: impl Into<String>,
        delta: i64,
    ) -> Result<i64, WorldError> {
        let material_kind = material_kind.into();
        if material_kind.trim().is_empty() {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: "material kind cannot be empty".to_string(),
            });
        }
        let current = self.material_balance(material_kind.as_str());
        let next = current.saturating_add(delta);
        if next < 0 {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: format!(
                    "material balance cannot be negative: kind={} current={} delta={}",
                    material_kind, current, delta
                ),
            });
        }
        if next == 0 {
            self.state.materials.remove(&material_kind);
        } else {
            self.state.materials.insert(material_kind, next);
        }
        Ok(next)
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
}
