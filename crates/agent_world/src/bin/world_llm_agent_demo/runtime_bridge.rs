use agent_world::runtime::{
    Action as RuntimeAction, DomainEvent as RuntimeDomainEvent,
    RejectReason as RuntimeRejectReason, World as RuntimeWorld,
    WorldEventBody as RuntimeWorldEventBody,
};
use agent_world::simulator::{
    Action as SimulatorAction, ActionResult, RejectReason, ResourceKind, ResourceOwner, WorldEvent,
    WorldEventKind, WorldKernel,
};

fn simulator_gameplay_action_to_runtime(action: &SimulatorAction) -> Option<RuntimeAction> {
    match action {
        SimulatorAction::FormAlliance {
            proposer_agent_id,
            alliance_id,
            members,
            charter,
        } => Some(RuntimeAction::FormAlliance {
            proposer_agent_id: proposer_agent_id.clone(),
            alliance_id: alliance_id.clone(),
            members: members.clone(),
            charter: charter.clone(),
        }),
        SimulatorAction::JoinAlliance {
            operator_agent_id,
            alliance_id,
            member_agent_id,
        } => Some(RuntimeAction::JoinAlliance {
            operator_agent_id: operator_agent_id.clone(),
            alliance_id: alliance_id.clone(),
            member_agent_id: member_agent_id.clone(),
        }),
        SimulatorAction::LeaveAlliance {
            operator_agent_id,
            alliance_id,
            member_agent_id,
        } => Some(RuntimeAction::LeaveAlliance {
            operator_agent_id: operator_agent_id.clone(),
            alliance_id: alliance_id.clone(),
            member_agent_id: member_agent_id.clone(),
        }),
        SimulatorAction::DissolveAlliance {
            operator_agent_id,
            alliance_id,
            reason,
        } => Some(RuntimeAction::DissolveAlliance {
            operator_agent_id: operator_agent_id.clone(),
            alliance_id: alliance_id.clone(),
            reason: reason.clone(),
        }),
        SimulatorAction::DeclareWar {
            initiator_agent_id,
            war_id,
            aggressor_alliance_id,
            defender_alliance_id,
            objective,
            intensity,
        } => Some(RuntimeAction::DeclareWar {
            initiator_agent_id: initiator_agent_id.clone(),
            war_id: war_id.clone(),
            aggressor_alliance_id: aggressor_alliance_id.clone(),
            defender_alliance_id: defender_alliance_id.clone(),
            objective: objective.clone(),
            intensity: *intensity,
        }),
        SimulatorAction::OpenGovernanceProposal {
            proposer_agent_id,
            proposal_key,
            title,
            description,
            options,
            voting_window_ticks,
            quorum_weight,
            pass_threshold_bps,
        } => Some(RuntimeAction::OpenGovernanceProposal {
            proposer_agent_id: proposer_agent_id.clone(),
            proposal_key: proposal_key.clone(),
            title: title.clone(),
            description: description.clone(),
            options: options.clone(),
            voting_window_ticks: *voting_window_ticks,
            quorum_weight: *quorum_weight,
            pass_threshold_bps: *pass_threshold_bps,
        }),
        SimulatorAction::CastGovernanceVote {
            voter_agent_id,
            proposal_key,
            option,
            weight,
        } => Some(RuntimeAction::CastGovernanceVote {
            voter_agent_id: voter_agent_id.clone(),
            proposal_key: proposal_key.clone(),
            option: option.clone(),
            weight: *weight,
        }),
        SimulatorAction::ResolveCrisis {
            resolver_agent_id,
            crisis_id,
            strategy,
            success,
        } => Some(RuntimeAction::ResolveCrisis {
            resolver_agent_id: resolver_agent_id.clone(),
            crisis_id: crisis_id.clone(),
            strategy: strategy.clone(),
            success: *success,
        }),
        SimulatorAction::GrantMetaProgress {
            operator_agent_id,
            target_agent_id,
            track,
            points,
            achievement_id,
        } => Some(RuntimeAction::GrantMetaProgress {
            operator_agent_id: operator_agent_id.clone(),
            target_agent_id: target_agent_id.clone(),
            track: track.clone(),
            points: *points,
            achievement_id: achievement_id.clone(),
        }),
        SimulatorAction::OpenEconomicContract {
            creator_agent_id,
            contract_id,
            counterparty_agent_id,
            settlement_kind,
            settlement_amount,
            reputation_stake,
            expires_at,
            description,
        } => Some(RuntimeAction::OpenEconomicContract {
            creator_agent_id: creator_agent_id.clone(),
            contract_id: contract_id.clone(),
            counterparty_agent_id: counterparty_agent_id.clone(),
            settlement_kind: *settlement_kind,
            settlement_amount: *settlement_amount,
            reputation_stake: *reputation_stake,
            expires_at: *expires_at,
            description: description.clone(),
        }),
        SimulatorAction::AcceptEconomicContract {
            accepter_agent_id,
            contract_id,
        } => Some(RuntimeAction::AcceptEconomicContract {
            accepter_agent_id: accepter_agent_id.clone(),
            contract_id: contract_id.clone(),
        }),
        SimulatorAction::SettleEconomicContract {
            operator_agent_id,
            contract_id,
            success,
            notes,
        } => Some(RuntimeAction::SettleEconomicContract {
            operator_agent_id: operator_agent_id.clone(),
            contract_id: contract_id.clone(),
            success: *success,
            notes: notes.clone(),
        }),
        _ => None,
    }
}

fn runtime_reject_reason_to_simulator(reason: &RuntimeRejectReason) -> RejectReason {
    match reason {
        RuntimeRejectReason::AgentAlreadyExists { agent_id } => RejectReason::AgentAlreadyExists {
            agent_id: agent_id.clone(),
        },
        RuntimeRejectReason::AgentNotFound { agent_id } => RejectReason::AgentNotFound {
            agent_id: agent_id.clone(),
        },
        RuntimeRejectReason::AgentsNotCoLocated {
            agent_id,
            other_agent_id,
        } => RejectReason::AgentsNotCoLocated {
            agent_id: agent_id.clone(),
            other_agent_id: other_agent_id.clone(),
        },
        RuntimeRejectReason::InvalidAmount { amount } => {
            RejectReason::InvalidAmount { amount: *amount }
        }
        RuntimeRejectReason::InsufficientResource {
            agent_id,
            kind,
            requested,
            available,
        } => RejectReason::InsufficientResource {
            owner: ResourceOwner::Agent {
                agent_id: agent_id.clone(),
            },
            kind: *kind,
            requested: *requested,
            available: *available,
        },
        RuntimeRejectReason::FactoryNotFound { factory_id } => RejectReason::FacilityNotFound {
            facility_id: factory_id.clone(),
        },
        RuntimeRejectReason::RuleDenied { notes } => RejectReason::RuleDenied {
            notes: notes.clone(),
        },
        other => RejectReason::RuleDenied {
            notes: vec![format!("runtime bridge reject: {:?}", other)],
        },
    }
}

pub(crate) fn is_bridgeable_action(action: &SimulatorAction) -> bool {
    simulator_gameplay_action_to_runtime(action).is_some()
}

pub(crate) fn execute_action_in_kernel(
    kernel: &mut WorldKernel,
    agent_id: &str,
    action: SimulatorAction,
) -> ActionResult {
    let action_id = kernel.submit_action_from_agent(agent_id.to_string(), action.clone());
    if let Some(event) = kernel.step() {
        let success = !matches!(event.kind, WorldEventKind::ActionRejected { .. });
        return ActionResult {
            action,
            action_id,
            success,
            event,
        };
    }

    ActionResult {
        action,
        action_id,
        success: false,
        event: WorldEvent {
            id: action_id,
            time: kernel.time(),
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::RuleDenied {
                    notes: vec!["kernel.step returned no event".to_string()],
                },
            },
        },
    }
}

pub(crate) fn advance_kernel_time_with_noop_move(kernel: &mut WorldKernel, agent_id: &str) {
    let Some(current_location_id) = kernel
        .model()
        .agents
        .get(agent_id)
        .map(|agent| agent.location_id.clone())
    else {
        return;
    };
    kernel.submit_action_from_system(SimulatorAction::MoveAgent {
        agent_id: agent_id.to_string(),
        to: current_location_id,
    });
    let _ = kernel.step();
}

#[derive(Debug)]
pub(crate) struct RuntimeGameplayBridge {
    world: RuntimeWorld,
    next_simulator_event_id: u64,
}

impl RuntimeGameplayBridge {
    pub(crate) fn from_kernel(kernel: &WorldKernel) -> Result<Self, String> {
        let mut world = RuntimeWorld::new();
        let mut agent_ids: Vec<String> = kernel.model().agents.keys().cloned().collect();
        agent_ids.sort();
        for agent_id in &agent_ids {
            let Some(agent) = kernel.model().agents.get(agent_id) else {
                continue;
            };
            world.submit_action(RuntimeAction::RegisterAgent {
                agent_id: agent_id.clone(),
                pos: agent.pos,
            });
        }

        if world.pending_actions_len() > 0 {
            world
                .step()
                .map_err(|err| format!("runtime gameplay bridge bootstrap step failed: {err:?}"))?;
        }

        Ok(Self {
            world,
            next_simulator_event_id: kernel.journal().len() as u64 + 1,
        })
    }

    pub(crate) fn execute(
        &mut self,
        tick: u64,
        agent_id: &str,
        action: SimulatorAction,
    ) -> Result<ActionResult, String> {
        let Some(runtime_action) = simulator_gameplay_action_to_runtime(&action) else {
            return Err("runtime gameplay bridge received non-gameplay action".to_string());
        };

        let runtime_action_id = self.world.submit_action(runtime_action);
        let previous_journal_len = self.world.journal().events.len();
        let mut rejection: Option<RuntimeRejectReason> = None;

        if let Err(err) = self.world.step() {
            rejection = Some(RuntimeRejectReason::RuleDenied {
                notes: vec![format!("runtime world step failed: {err:?}")],
            });
        } else {
            for event in self
                .world
                .journal()
                .events
                .iter()
                .skip(previous_journal_len)
            {
                if let RuntimeWorldEventBody::Domain(RuntimeDomainEvent::ActionRejected {
                    action_id,
                    reason,
                }) = &event.body
                {
                    if *action_id == runtime_action_id {
                        rejection = Some(reason.clone());
                        break;
                    }
                }
            }
        }

        let simulator_event = if let Some(reason) = rejection.as_ref() {
            WorldEvent {
                id: self.next_simulator_event_id,
                time: tick,
                kind: WorldEventKind::ActionRejected {
                    reason: runtime_reject_reason_to_simulator(reason),
                },
            }
        } else {
            WorldEvent {
                id: self.next_simulator_event_id,
                time: tick,
                kind: WorldEventKind::ResourceTransferred {
                    from: ResourceOwner::Agent {
                        agent_id: agent_id.to_string(),
                    },
                    to: ResourceOwner::Agent {
                        agent_id: agent_id.to_string(),
                    },
                    kind: ResourceKind::Data,
                    amount: 0,
                },
            }
        };
        self.next_simulator_event_id = self.next_simulator_event_id.saturating_add(1);

        Ok(ActionResult {
            action,
            action_id: runtime_action_id,
            success: rejection.is_none(),
            event: simulator_event,
        })
    }
}
