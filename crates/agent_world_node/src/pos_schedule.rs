use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::PosNodeEngine;

impl PosNodeEngine {
    pub(super) fn expected_proposer(&self, slot: u64) -> Option<String> {
        if self.validators.is_empty() || self.total_stake == 0 {
            return None;
        }
        let mut hasher = DefaultHasher::new();
        slot.hash(&mut hasher);
        let mut target = hasher.finish() % self.total_stake;
        for (validator_id, stake) in &self.validators {
            if target < *stake {
                return Some(validator_id.clone());
            }
            target = target.saturating_sub(*stake);
        }
        self.validators.keys().next().cloned()
    }

    pub(super) fn slot_epoch(&self, slot: u64) -> u64 {
        slot / self.epoch_length_slots
    }
}
