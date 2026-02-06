use super::World;
use super::super::ResourceDelta;
use crate::simulator::ResourceKind;

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
}
