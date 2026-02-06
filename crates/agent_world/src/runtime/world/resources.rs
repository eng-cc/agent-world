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
