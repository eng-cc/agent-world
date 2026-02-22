use super::super::{Action, ActionEnvelope, ActionId};
use super::World;

impl World {
    // ---------------------------------------------------------------------
    // Action submission
    // ---------------------------------------------------------------------

    pub fn submit_action(&mut self, action: Action) -> ActionId {
        let action_id = self.allocate_next_action_id();
        self.pending_actions.push_back(ActionEnvelope {
            id: action_id,
            action,
        });
        action_id
    }

    pub fn pending_actions_len(&self) -> usize {
        self.pending_actions.len()
    }

    pub fn pending_effects_len(&self) -> usize {
        self.pending_effects.len()
    }
}
