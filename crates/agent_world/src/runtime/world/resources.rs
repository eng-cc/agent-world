use super::super::util::hash_json;
use super::super::ResourceDelta;
use super::super::WorldError;
use super::super::{
    EpochSettlementReport, MaterialLedgerId, MaterialStack, NodeAssetBalance, NodeRewardMintRecord,
    ProtocolPowerReserve, RewardAssetConfig,
};
use super::World;
use crate::simulator::ResourceKind;
use crate::simulator::StockError;
use std::collections::BTreeMap;

impl World {
    // ---------------------------------------------------------------------
    // Reward asset ledger
    // ---------------------------------------------------------------------

    pub fn reward_asset_config(&self) -> &RewardAssetConfig {
        &self.state.reward_asset_config
    }

    pub fn set_reward_asset_config(&mut self, config: RewardAssetConfig) {
        self.state.reward_asset_config = config;
    }

    pub fn protocol_power_reserve(&self) -> &ProtocolPowerReserve {
        &self.state.protocol_power_reserve
    }

    pub fn set_protocol_power_reserve(&mut self, reserve: ProtocolPowerReserve) {
        self.state.protocol_power_reserve = reserve;
    }

    pub fn node_asset_balance(&self, node_id: &str) -> Option<&NodeAssetBalance> {
        self.state.node_asset_balances.get(node_id)
    }

    pub fn node_power_credit_balance(&self, node_id: &str) -> u64 {
        self.state
            .node_asset_balances
            .get(node_id)
            .map(|balance| balance.power_credit_balance)
            .unwrap_or(0)
    }

    pub fn reward_mint_records(&self) -> &[NodeRewardMintRecord] {
        self.state.reward_mint_records.as_slice()
    }

    pub fn apply_node_points_settlement_mint(
        &mut self,
        report: &EpochSettlementReport,
        signer_node_id: &str,
    ) -> Result<Vec<NodeRewardMintRecord>, WorldError> {
        if signer_node_id.trim().is_empty() {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: "signer_node_id cannot be empty".to_string(),
            });
        }
        let points_per_credit = self.state.reward_asset_config.points_per_credit;
        if points_per_credit == 0 {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: "points_per_credit must be positive".to_string(),
            });
        }

        let settlement_hash = hash_json(report)?;
        let mut minted_records = Vec::new();
        for settlement in &report.settlements {
            if self.state.reward_mint_records.iter().any(|record| {
                record.epoch_index == report.epoch_index && record.node_id == settlement.node_id
            }) {
                continue;
            }

            let minted_power_credits = settlement.awarded_points / points_per_credit;
            if minted_power_credits == 0 {
                continue;
            }
            self.mint_node_power_credits(settlement.node_id.as_str(), minted_power_credits)?;

            let record = NodeRewardMintRecord {
                epoch_index: report.epoch_index,
                node_id: settlement.node_id.clone(),
                source_awarded_points: settlement.awarded_points,
                minted_power_credits,
                settlement_hash: settlement_hash.clone(),
                signer_node_id: signer_node_id.to_string(),
                signature: String::new(),
            };
            self.state.reward_mint_records.push(record.clone());
            minted_records.push(record);
        }

        Ok(minted_records)
    }

    pub fn mint_node_power_credits(
        &mut self,
        node_id: &str,
        amount: u64,
    ) -> Result<u64, WorldError> {
        let balance = self.node_asset_balance_entry_mut(node_id)?;
        balance.power_credit_balance = balance.power_credit_balance.saturating_add(amount);
        balance.total_minted_credits = balance.total_minted_credits.saturating_add(amount);
        Ok(balance.power_credit_balance)
    }

    pub fn burn_node_power_credits(
        &mut self,
        node_id: &str,
        amount: u64,
    ) -> Result<u64, WorldError> {
        let balance = self.node_asset_balance_entry_mut(node_id)?;
        if amount > balance.power_credit_balance {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: format!(
                    "insufficient power credits for {}: balance={} burn={}",
                    node_id, balance.power_credit_balance, amount
                ),
            });
        }
        balance.power_credit_balance -= amount;
        balance.total_burned_credits = balance.total_burned_credits.saturating_add(amount);
        Ok(balance.power_credit_balance)
    }

    fn node_asset_balance_entry_mut(
        &mut self,
        node_id: &str,
    ) -> Result<&mut NodeAssetBalance, WorldError> {
        if node_id.trim().is_empty() {
            return Err(WorldError::ResourceBalanceInvalid {
                reason: "node_id cannot be empty".to_string(),
            });
        }
        Ok(self
            .state
            .node_asset_balances
            .entry(node_id.to_string())
            .or_insert_with(|| NodeAssetBalance {
                node_id: node_id.to_string(),
                ..NodeAssetBalance::default()
            }))
    }

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
