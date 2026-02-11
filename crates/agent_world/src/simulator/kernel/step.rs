use super::super::types::{Action, ActionEnvelope, ActionId};
use super::types::WorldEvent;
use super::WorldKernel;

impl WorldKernel {
    pub fn submit_action(&mut self, action: Action) -> ActionId {
        let id = self.next_action_id;
        self.next_action_id = self.next_action_id.saturating_add(1);
        self.pending_actions
            .push_back(ActionEnvelope { id, action });
        id
    }

    pub fn pending_actions(&self) -> usize {
        self.pending_actions.len()
    }

    pub fn step(&mut self) -> Option<WorldEvent> {
        let envelope = self.pending_actions.pop_front()?;
        let action_id = envelope.id;
        let action = envelope.action;

        for hook in &self.rule_hooks.pre_action {
            hook(action_id, &action);
        }

        self.time = self.time.saturating_add(1);
        let kind = self.apply_action(action.clone());
        let event = WorldEvent {
            id: self.next_event_id,
            time: self.time,
            kind,
        };
        self.next_event_id = self.next_event_id.saturating_add(1);
        self.journal.push(event.clone());

        for hook in &self.rule_hooks.post_action {
            hook(action_id, &action, &event);
        }

        Some(event)
    }

    pub fn step_until_empty(&mut self) -> Vec<WorldEvent> {
        let mut events = Vec::new();
        while let Some(event) = self.step() {
            events.push(event);
        }
        events
    }
}
