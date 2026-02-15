use super::super::types::{Action, ActionEnvelope, ActionId};
use super::types::{
    KernelRuleDecision, KernelRuleModuleContext, KernelRuleModuleInput, KernelRuleVerdict,
    RejectReason, WorldEvent, WorldEventKind,
};
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

        let mut decisions = Vec::with_capacity(
            self.rule_hooks.pre_action.len()
                + usize::from(self.rule_hooks.pre_action_wasm.is_some()),
        );
        if let Some(evaluator) = &self.rule_hooks.pre_action_wasm {
            let input = self.build_pre_action_wasm_rule_input(action_id, &action);
            match evaluator(&input) {
                Ok(output) => decisions.push(output.decision),
                Err(err) => decisions.push(KernelRuleDecision::deny(
                    action_id,
                    vec![format!("wasm pre-action evaluator failed: {err}")],
                )),
            }
        }
        for hook in &self.rule_hooks.pre_action {
            decisions.push(hook(action_id, &action, self));
        }
        let merged_decision =
            self.merge_pre_action_rule_decisions(action_id, decisions.into_iter());

        self.time = self.time.saturating_add(1);
        let kind = match merged_decision.verdict {
            KernelRuleVerdict::Deny => WorldEventKind::ActionRejected {
                reason: RejectReason::RuleDenied {
                    notes: merged_decision.notes,
                },
            },
            KernelRuleVerdict::Modify => match merged_decision.override_action {
                Some(override_action) => self.apply_action(override_action),
                None => WorldEventKind::ActionRejected {
                    reason: RejectReason::RuleDenied {
                        notes: vec![format!(
                            "rule decision missing override for action {action_id}"
                        )],
                    },
                },
            },
            KernelRuleVerdict::Allow => self.apply_action(action.clone()),
        };
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

        self.maybe_replenish_fragments();

        Some(event)
    }

    fn merge_pre_action_rule_decisions<I>(
        &self,
        action_id: ActionId,
        decisions: I,
    ) -> KernelRuleDecision
    where
        I: IntoIterator<Item = KernelRuleDecision>,
    {
        match super::merge_kernel_rule_decisions(action_id, decisions) {
            Ok(merged) => merged,
            Err(err) => KernelRuleDecision::deny(action_id, vec![err.to_string()]),
        }
    }

    fn build_pre_action_wasm_rule_input(
        &self,
        action_id: ActionId,
        action: &Action,
    ) -> KernelRuleModuleInput {
        KernelRuleModuleInput {
            action_id,
            action: action.clone(),
            context: KernelRuleModuleContext {
                time: self.time,
                location_ids: self.model.locations.keys().cloned().collect(),
                agent_ids: self.model.agents.keys().cloned().collect(),
            },
        }
    }

    pub fn step_until_empty(&mut self) -> Vec<WorldEvent> {
        let mut events = Vec::new();
        while let Some(event) = self.step() {
            events.push(event);
        }
        events
    }
}
