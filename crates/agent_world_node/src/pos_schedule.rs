use agent_world_proto::distributed_pos::{
    slot_epoch as shared_slot_epoch, weighted_expected_proposer,
};

use crate::PosNodeEngine;

impl PosNodeEngine {
    pub(super) fn expected_proposer(&self, slot: u64) -> Option<String> {
        weighted_expected_proposer(&self.validators, self.total_stake, slot)
    }

    pub(super) fn slot_epoch(&self, slot: u64) -> u64 {
        shared_slot_epoch(self.epoch_length_slots, slot)
    }
}
