use super::super::types::{Action, ActionEnvelope, ActionId, ActionSubmitter, ResourceOwner};
use super::types::{
    KernelRuleDecision, KernelRuleModuleContext, KernelRuleModuleInput, KernelRuleVerdict,
    RejectReason, WorldEvent, WorldEventKind,
};
use super::WorldKernel;

impl WorldKernel {
    pub fn submit_action(&mut self, action: Action) -> ActionId {
        self.submit_action_from_system(action)
    }

    pub fn submit_action_from_system(&mut self, action: Action) -> ActionId {
        self.submit_action_with_submitter(action, ActionSubmitter::System)
    }

    pub fn submit_action_from_agent(
        &mut self,
        agent_id: impl Into<String>,
        action: Action,
    ) -> ActionId {
        self.submit_action_with_submitter(
            action,
            ActionSubmitter::Agent {
                agent_id: agent_id.into(),
            },
        )
    }

    pub fn submit_action_from_player(
        &mut self,
        player_id: impl Into<String>,
        action: Action,
    ) -> ActionId {
        self.submit_action_with_submitter(
            action,
            ActionSubmitter::Player {
                player_id: player_id.into(),
            },
        )
    }

    fn submit_action_with_submitter(
        &mut self,
        action: Action,
        submitter: ActionSubmitter,
    ) -> ActionId {
        let id = self.next_action_id;
        self.next_action_id = self.next_action_id.saturating_add(1);
        self.pending_actions.push_back(ActionEnvelope {
            id,
            action,
            submitter,
        });
        id
    }

    pub fn pending_actions(&self) -> usize {
        self.pending_actions.len()
    }

    pub fn step(&mut self) -> Option<WorldEvent> {
        let envelope = self.pending_actions.pop_front()?;
        let action_id = envelope.id;
        let action = envelope.action;

        if let Some(reason) = reject_reason_for_submitter(&envelope.submitter, &action) {
            self.time = self.time.saturating_add(1);
            let event = WorldEvent {
                id: self.next_event_id,
                time: self.time,
                kind: WorldEventKind::ActionRejected { reason },
            };
            self.next_event_id = self.next_event_id.saturating_add(1);
            self.journal.push(event.clone());
            return Some(event);
        }

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
        self.maintain_social_lifecycle();

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

fn reject_reason_for_submitter(
    submitter: &ActionSubmitter,
    action: &Action,
) -> Option<RejectReason> {
    match submitter {
        ActionSubmitter::System => None,
        ActionSubmitter::Player { player_id } => Some(RejectReason::RuleDenied {
            notes: vec![format!(
                "player {} cannot submit world actions directly; use prompt/chat indirect control",
                player_id
            )],
        }),
        ActionSubmitter::Agent { agent_id } => reject_reason_for_agent_submitter(agent_id, action),
    }
}

fn reject_reason_for_agent_submitter(agent_id: &str, action: &Action) -> Option<RejectReason> {
    let denied = |detail: &str| -> Option<RejectReason> {
        Some(RejectReason::RuleDenied {
            notes: vec![format!(
                "agent submitter {} cannot submit action: {}",
                agent_id, detail
            )],
        })
    };

    match action {
        Action::RegisterLocation { .. }
        | Action::RegisterAgent { .. }
        | Action::RegisterPowerPlant { .. }
        | Action::RegisterPowerStorage { .. }
        | Action::UpsertModuleVisualEntity { .. }
        | Action::RemoveModuleVisualEntity { .. }
        | Action::DebugGrantResource { .. } => denied("system-only action"),
        Action::MoveAgent {
            agent_id: action_agent_id,
            ..
        }
        | Action::HarvestRadiation {
            agent_id: action_agent_id,
            ..
        } => {
            if action_agent_id == agent_id {
                None
            } else {
                denied("action agent_id mismatch")
            }
        }
        Action::MineCompound { owner, .. }
        | Action::RefineCompound { owner, .. }
        | Action::BuildFactory { owner, .. }
        | Action::ScheduleRecipe { owner, .. }
        | Action::PlacePowerOrder { owner, .. }
        | Action::CancelPowerOrder { owner, .. }
        | Action::PublishSocialFact { actor: owner, .. }
        | Action::RevokeSocialFact { actor: owner, .. } => {
            if resource_owner_is_agent(owner, agent_id) {
                None
            } else {
                denied("owner must be the submitter agent")
            }
        }
        Action::ChallengeSocialFact { challenger, .. }
        | Action::AdjudicateSocialFact {
            adjudicator: challenger,
            ..
        }
        | Action::DeclareSocialEdge {
            declarer: challenger,
            ..
        } => {
            if resource_owner_is_agent(challenger, agent_id) {
                None
            } else {
                denied("social actor must be the submitter agent")
            }
        }
        Action::FormAlliance {
            proposer_agent_id, ..
        }
        | Action::OpenGovernanceProposal {
            proposer_agent_id, ..
        } => {
            if proposer_agent_id == agent_id {
                None
            } else {
                denied("proposer_agent_id must be the submitter agent")
            }
        }
        Action::DeclareWar {
            initiator_agent_id, ..
        } => {
            if initiator_agent_id == agent_id {
                None
            } else {
                denied("initiator_agent_id must be the submitter agent")
            }
        }
        Action::CastGovernanceVote { voter_agent_id, .. } => {
            if voter_agent_id == agent_id {
                None
            } else {
                denied("voter_agent_id must be the submitter agent")
            }
        }
        Action::ResolveCrisis {
            resolver_agent_id, ..
        } => {
            if resolver_agent_id == agent_id {
                None
            } else {
                denied("resolver_agent_id must be the submitter agent")
            }
        }
        Action::GrantMetaProgress {
            operator_agent_id, ..
        } => {
            if operator_agent_id == agent_id {
                None
            } else {
                denied("operator_agent_id must be the submitter agent")
            }
        }
        Action::TransferResource { from, .. } => {
            if resource_owner_is_agent(from, agent_id) {
                None
            } else {
                denied("from owner must be the submitter agent")
            }
        }
        Action::BuyPower { buyer, .. } => {
            if resource_owner_is_agent(buyer, agent_id) {
                None
            } else {
                denied("buyer must be the submitter agent")
            }
        }
        Action::SellPower { seller, .. } => {
            if resource_owner_is_agent(seller, agent_id) {
                None
            } else {
                denied("seller must be the submitter agent")
            }
        }
        Action::CompileModuleArtifactFromSource {
            publisher_agent_id, ..
        }
        | Action::DeployModuleArtifact {
            publisher_agent_id, ..
        } => {
            if publisher_agent_id == agent_id {
                None
            } else {
                denied("publisher_agent_id must be the submitter agent")
            }
        }
        Action::InstallModuleFromArtifact {
            installer_agent_id, ..
        }
        | Action::InstallModuleToTargetFromArtifact {
            installer_agent_id, ..
        } => {
            if installer_agent_id == agent_id {
                None
            } else {
                denied("installer_agent_id must be the submitter agent")
            }
        }
        Action::ListModuleArtifactForSale {
            seller_agent_id, ..
        }
        | Action::DelistModuleArtifact {
            seller_agent_id, ..
        } => {
            if seller_agent_id == agent_id {
                None
            } else {
                denied("seller_agent_id must be the submitter agent")
            }
        }
        Action::BuyModuleArtifact { buyer_agent_id, .. } => {
            if buyer_agent_id == agent_id {
                None
            } else {
                denied("buyer_agent_id must be the submitter agent")
            }
        }
        Action::DestroyModuleArtifact { owner_agent_id, .. } => {
            if owner_agent_id == agent_id {
                None
            } else {
                denied("owner_agent_id must be the submitter agent")
            }
        }
        Action::PlaceModuleArtifactBid {
            bidder_agent_id, ..
        }
        | Action::CancelModuleArtifactBid {
            bidder_agent_id, ..
        } => {
            if bidder_agent_id == agent_id {
                None
            } else {
                denied("bidder_agent_id must be the submitter agent")
            }
        }
        Action::DrawPower { .. } | Action::StorePower { .. } => None,
    }
}

fn resource_owner_is_agent(owner: &ResourceOwner, agent_id: &str) -> bool {
    matches!(owner, ResourceOwner::Agent { agent_id: value } if value == agent_id)
}
