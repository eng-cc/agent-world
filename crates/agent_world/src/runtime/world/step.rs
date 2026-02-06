use super::World;
use super::super::{
    ActionEnvelope, CausedBy, ModuleSandbox, ModuleSubscriptionStage, RejectReason, RuleVerdict,
    WorldError, WorldEventBody,
};

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
            let decision = self.evaluate_rule_decisions(&envelope, sandbox)?;
            let mut action_envelope = envelope.clone();
            if decision.verdict == RuleVerdict::Modify {
                if let Some(override_action) = decision.override_action.clone() {
                    self.record_action_override(
                        super::super::ActionOverrideRecord {
                            action_id: envelope.id,
                            original_action: envelope.action.clone(),
                            override_action: override_action.clone(),
                        },
                        Some(CausedBy::Action(envelope.id)),
                    )?;
                    action_envelope = ActionEnvelope {
                        id: envelope.id,
                        action: override_action,
                    };
                }
            }

            if decision.verdict == RuleVerdict::Deny {
                self.append_event(
                    WorldEventBody::Domain(super::super::DomainEvent::ActionRejected {
                        action_id: envelope.id,
                        reason: RejectReason::RuleDenied {
                            notes: decision.notes.clone(),
                        },
                    }),
                    Some(CausedBy::Action(envelope.id)),
                )?;
            } else {
                let deficits = decision.cost.deficits(&self.state.resources);
                if !deficits.is_empty() {
                    self.append_event(
                        WorldEventBody::Domain(super::super::DomainEvent::ActionRejected {
                            action_id: envelope.id,
                            reason: RejectReason::InsufficientResources { deficits },
                        }),
                        Some(CausedBy::Action(envelope.id)),
                    )?;
                } else {
                    self.apply_resource_delta(&decision.cost);
                    let event_body = self.action_to_event(&action_envelope)?;
                    self.append_event(event_body, Some(CausedBy::Action(envelope.id)))?;
                }
            }

            self.route_action_to_modules_with_stage(
                &envelope,
                ModuleSubscriptionStage::PostAction,
                sandbox,
            )?;
            if let Some(event) = self.journal.events.last() {
                let event = event.clone();
                self.route_event_to_modules(&event, sandbox)?;
            }
        }
        Ok(())
    }
}
