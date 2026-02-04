use super::World;
use super::super::{CausedBy, ModuleSandbox, WorldError};

impl World {
    // ---------------------------------------------------------------------
    // Simulation step
    // ---------------------------------------------------------------------

    pub fn step(&mut self) -> Result<(), WorldError> {
        self.state.time = self.state.time.saturating_add(1);
        while let Some(envelope) = self.pending_actions.pop_front() {
            let event_body = self.action_to_event(&envelope)?;
            self.append_event(event_body, Some(CausedBy::Action(envelope.id)))?;
        }
        Ok(())
    }

    pub fn step_with_modules(
        &mut self,
        sandbox: &mut dyn ModuleSandbox,
    ) -> Result<(), WorldError> {
        self.state.time = self.state.time.saturating_add(1);
        while let Some(envelope) = self.pending_actions.pop_front() {
            self.route_action_to_modules(&envelope, sandbox)?;
            let event_body = self.action_to_event(&envelope)?;
            self.append_event(event_body, Some(CausedBy::Action(envelope.id)))?;
            if let Some(event) = self.journal.events.last() {
                let event = event.clone();
                self.route_event_to_modules(&event, sandbox)?;
            }
        }
        Ok(())
    }
}
