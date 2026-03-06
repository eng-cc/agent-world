use agent_world_wasm_abi::ModuleSandbox;

use super::super::{
    Action, ActionEnvelope, CausedBy, DomainEvent, ModuleSubscriptionStage, RejectReason,
    RuleVerdict, WorldError, WorldEventBody,
};
use super::economy::EconomyActionResolution;
use super::World;

impl World {
    fn should_emit_action_accepted(action: &Action) -> bool {
        matches!(
            action,
            Action::FormAlliance { .. }
                | Action::JoinAlliance { .. }
                | Action::LeaveAlliance { .. }
                | Action::DissolveAlliance { .. }
                | Action::DeclareWar { .. }
                | Action::OpenGovernanceProposal { .. }
                | Action::CastGovernanceVote { .. }
                | Action::ResolveCrisis { .. }
                | Action::GrantMetaProgress { .. }
                | Action::UpdateGameplayPolicy { .. }
                | Action::OpenEconomicContract { .. }
                | Action::AcceptEconomicContract { .. }
                | Action::SettleEconomicContract { .. }
        )
    }

    fn append_action_accepted_event(
        &mut self,
        envelope: &ActionEnvelope,
    ) -> Result<(), WorldError> {
        if !Self::should_emit_action_accepted(&envelope.action) {
            return Ok(());
        }
        let actor_id = envelope.action.actor_id().unwrap_or("system");
        self.append_event(
            WorldEventBody::Domain(DomainEvent::ActionAccepted {
                action_id: envelope.id,
                action_kind: super::module_runtime_labels::action_kind_label(&envelope.action)
                    .to_string(),
                actor_id: actor_id.to_string(),
                eta_ticks: 0,
                notes: vec!["accepted_for_gameplay_processing".to_string()],
            }),
            Some(CausedBy::Action(envelope.id)),
        )?;
        Ok(())
    }

    fn preflight_domain_event(&self, body: &WorldEventBody) -> Result<(), WorldError> {
        let WorldEventBody::Domain(event) = body else {
            return Ok(());
        };
        let mut preview_state = self.state.clone();
        preview_state.apply_domain_event(event, self.state.time)
    }

    // ---------------------------------------------------------------------
    // Simulation step
    // ---------------------------------------------------------------------

    pub fn step(&mut self) -> Result<(), WorldError> {
        self.state.time = self.state.time.saturating_add(1);
        let _ = self.process_factory_depreciation()?;
        while let Some(envelope) = self.pending_actions.pop_front() {
            if self.try_apply_runtime_module_action(&envelope)? {
                continue;
            }
            let event_body = self.action_to_event(&envelope)?;
            self.preflight_domain_event(&event_body)?;
            self.append_action_accepted_event(&envelope)?;
            self.append_event(event_body, Some(CausedBy::Action(envelope.id)))?;
        }
        let _ = self.process_due_economy_jobs()?;
        let _ = self.process_due_material_transits()?;
        let _ = self.process_gameplay_cycles()?;
        self.refresh_threat_heatmap();
        Ok(())
    }

    pub fn step_with_modules(&mut self, sandbox: &mut dyn ModuleSandbox) -> Result<(), WorldError> {
        self.state.time = self.state.time.saturating_add(1);
        for event in self.process_factory_depreciation()? {
            self.route_event_to_modules(&event, sandbox)?;
        }
        while let Some(envelope) = self.pending_actions.pop_front() {
            let mut action_envelope = envelope.clone();
            match self.resolve_module_backed_economy_action(&envelope, sandbox)? {
                EconomyActionResolution::Resolved(action) => {
                    action_envelope.action = action;
                }
                EconomyActionResolution::Rejected(reason) => {
                    self.append_action_accepted_event(&envelope)?;
                    self.append_event(
                        WorldEventBody::Domain(super::super::DomainEvent::ActionRejected {
                            action_id: envelope.id,
                            reason,
                        }),
                        Some(CausedBy::Action(envelope.id)),
                    )?;
                    self.route_action_to_modules_with_stage(
                        &envelope,
                        ModuleSubscriptionStage::PostAction,
                        sandbox,
                    )?;
                    if let Some(event) = self.journal.events.last() {
                        let event = event.clone();
                        self.route_event_to_modules(&event, sandbox)?;
                    }
                    continue;
                }
            }

            let decision = self.evaluate_rule_decisions(&action_envelope, sandbox)?;
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
                self.append_action_accepted_event(&envelope)?;
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
                    self.append_action_accepted_event(&envelope)?;
                    self.append_event(
                        WorldEventBody::Domain(super::super::DomainEvent::ActionRejected {
                            action_id: envelope.id,
                            reason: RejectReason::InsufficientResources { deficits },
                        }),
                        Some(CausedBy::Action(envelope.id)),
                    )?;
                } else {
                    match self.apply_resource_delta(&decision.cost) {
                        Ok(()) => {
                            if !self.try_apply_runtime_module_action(&action_envelope)? {
                                let event_body = self.action_to_event(&action_envelope)?;
                                self.preflight_domain_event(&event_body)?;
                                self.append_action_accepted_event(&envelope)?;
                                self.append_event(event_body, Some(CausedBy::Action(envelope.id)))?;
                            }
                        }
                        Err(err) => {
                            self.append_action_accepted_event(&envelope)?;
                            self.append_event(
                                WorldEventBody::Domain(super::super::DomainEvent::ActionRejected {
                                    action_id: envelope.id,
                                    reason: RejectReason::RuleDenied {
                                        notes: vec![format!(
                                            "rule decision cost apply rejected: {err:?}"
                                        )],
                                    },
                                }),
                                Some(CausedBy::Action(envelope.id)),
                            )?;
                        }
                    }
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
        for event in self.process_due_economy_jobs_with_modules(sandbox)? {
            self.route_event_to_modules(&event, sandbox)?;
        }
        for event in self.process_due_material_transits()? {
            self.route_event_to_modules(&event, sandbox)?;
        }
        for event in self.process_gameplay_cycles_with_modules(sandbox)? {
            self.route_event_to_modules(&event, sandbox)?;
        }
        self.refresh_threat_heatmap();
        Ok(())
    }
}
