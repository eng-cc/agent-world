use crate::geometry::space_distance_cm;
use crate::runtime::{
    DomainEvent as RuntimeDomainEvent, RejectReason as RuntimeRejectReason,
    WorldEvent as RuntimeWorldEvent, WorldEventBody as RuntimeWorldEventBody,
};
use crate::simulator::{
    Agent, Location, RejectReason as SimulatorRejectReason, ResourceOwner, WorldConfig, WorldEvent,
    WorldEventKind, WorldModel,
};

use super::control_plane::RuntimeLlmSidecar;
use super::location_id_for_pos;

pub(super) fn runtime_state_to_simulator_model(
    state: &crate::runtime::WorldState,
    sidecar: &RuntimeLlmSidecar,
) -> WorldModel {
    let mut model = WorldModel::default();

    for (agent_id, cell) in &state.agents {
        let location_id = location_id_for_pos(cell.state.pos);
        model
            .locations
            .entry(location_id.clone())
            .or_insert_with(|| {
                Location::new(
                    location_id.clone(),
                    format!("runtime-{location_id}"),
                    cell.state.pos,
                )
            });

        let mut agent = Agent::new(agent_id.clone(), location_id, cell.state.pos);
        agent.body = cell.state.body.clone();
        agent.resources = cell.state.resources.clone();
        model.agents.insert(agent_id.clone(), agent);
    }

    model.agent_prompt_profiles = sidecar.prompt_profiles.clone();
    model.agent_player_bindings = sidecar.agent_player_bindings.clone();
    model.agent_player_public_key_bindings = sidecar.agent_public_key_bindings.clone();
    model.player_auth_last_nonce = sidecar.player_auth_last_nonce.clone();
    model
}

pub(super) fn map_runtime_event(
    runtime_event: &RuntimeWorldEvent,
    config: &WorldConfig,
) -> WorldEvent {
    let kind = match &runtime_event.body {
        RuntimeWorldEventBody::Domain(domain) => map_runtime_domain_event(domain, config)
            .unwrap_or_else(|| runtime_fallback_event_kind(runtime_event)),
        _ => runtime_fallback_event_kind(runtime_event),
    };

    WorldEvent {
        id: runtime_event.id,
        time: runtime_event.time,
        kind,
        runtime_event: Some(runtime_event.clone()),
    }
}

pub(super) fn map_runtime_domain_event(
    event: &RuntimeDomainEvent,
    config: &WorldConfig,
) -> Option<WorldEventKind> {
    match event {
        RuntimeDomainEvent::AgentRegistered { agent_id, pos } => {
            Some(WorldEventKind::AgentRegistered {
                agent_id: agent_id.clone(),
                location_id: location_id_for_pos(*pos),
                pos: *pos,
            })
        }
        RuntimeDomainEvent::AgentMoved { agent_id, from, to } => {
            let distance_cm = space_distance_cm(*from, *to);
            Some(WorldEventKind::AgentMoved {
                agent_id: agent_id.clone(),
                from: location_id_for_pos(*from),
                to: location_id_for_pos(*to),
                distance_cm,
                electricity_cost: config.movement_cost(distance_cm),
            })
        }
        RuntimeDomainEvent::ResourceTransferred {
            from_agent_id,
            to_agent_id,
            kind,
            amount,
        } => Some(WorldEventKind::ResourceTransferred {
            from: ResourceOwner::Agent {
                agent_id: from_agent_id.clone(),
            },
            to: ResourceOwner::Agent {
                agent_id: to_agent_id.clone(),
            },
            kind: *kind,
            amount: *amount,
        }),
        RuntimeDomainEvent::ActionRejected { reason, .. } => Some(WorldEventKind::ActionRejected {
            reason: runtime_reject_reason_to_simulator(reason),
        }),
        RuntimeDomainEvent::ActionAccepted {
            action_id,
            action_kind,
            actor_id,
            eta_ticks,
            ..
        } => Some(runtime_structured_event(
            "runtime.action_accepted",
            format!(
                "action_id={action_id} action_kind={} actor_id={} eta_ticks={eta_ticks}",
                fallback_non_empty(action_kind, "unknown_action"),
                fallback_non_empty(actor_id, "system"),
            ),
        )),
        RuntimeDomainEvent::WarDeclared {
            war_id,
            objective,
            intensity,
            ..
        } => Some(runtime_structured_event(
            "runtime.gameplay.war_declared",
            format!(
                "war_id={} objective={} intensity={intensity}",
                fallback_non_empty(war_id, "unknown_war"),
                fallback_non_empty(objective, "unknown_objective"),
            ),
        )),
        RuntimeDomainEvent::WarConcluded {
            war_id,
            winner_alliance_id,
            summary,
            ..
        } => Some(runtime_structured_event(
            "runtime.gameplay.war_concluded",
            format!(
                "war_id={} winner={} summary={}",
                fallback_non_empty(war_id, "unknown_war"),
                fallback_non_empty(winner_alliance_id, "unknown_winner"),
                fallback_non_empty(summary, "none"),
            ),
        )),
        RuntimeDomainEvent::GovernanceProposalOpened {
            proposal_key,
            title,
            closes_at,
            ..
        } => Some(runtime_structured_event(
            "runtime.gameplay.governance_proposal_opened",
            format!(
                "proposal_key={} title={} closes_at={closes_at}",
                fallback_non_empty(proposal_key, "unknown_proposal"),
                fallback_non_empty(title, "untitled"),
            ),
        )),
        RuntimeDomainEvent::GovernanceVoteCast {
            proposal_key,
            option,
            weight,
            ..
        } => Some(runtime_structured_event(
            "runtime.gameplay.governance_vote_cast",
            format!(
                "proposal_key={} option={} weight={weight}",
                fallback_non_empty(proposal_key, "unknown_proposal"),
                fallback_non_empty(option, "unknown_option"),
            ),
        )),
        RuntimeDomainEvent::GovernanceProposalFinalized {
            proposal_key,
            winning_option,
            passed,
            ..
        } => Some(runtime_structured_event(
            "runtime.gameplay.governance_proposal_finalized",
            format!(
                "proposal_key={} winning_option={} passed={passed}",
                fallback_non_empty(proposal_key, "unknown_proposal"),
                winning_option.as_deref().unwrap_or("none"),
            ),
        )),
        RuntimeDomainEvent::CrisisSpawned {
            crisis_id,
            kind,
            severity,
            expires_at,
        } => Some(runtime_structured_event(
            "runtime.gameplay.crisis_spawned",
            format!(
                "crisis_id={} kind={} severity={severity} expires_at={expires_at}",
                fallback_non_empty(crisis_id, "unknown_crisis"),
                fallback_non_empty(kind, "unknown_kind"),
            ),
        )),
        RuntimeDomainEvent::CrisisResolved {
            crisis_id,
            strategy,
            success,
            impact,
            ..
        } => Some(runtime_structured_event(
            "runtime.gameplay.crisis_resolved",
            format!(
                "crisis_id={} strategy={} success={success} impact={impact}",
                fallback_non_empty(crisis_id, "unknown_crisis"),
                fallback_non_empty(strategy, "unknown_strategy"),
            ),
        )),
        RuntimeDomainEvent::CrisisTimedOut {
            crisis_id,
            penalty_impact,
        } => Some(runtime_structured_event(
            "runtime.gameplay.crisis_timed_out",
            format!(
                "crisis_id={} penalty_impact={penalty_impact}",
                fallback_non_empty(crisis_id, "unknown_crisis"),
            ),
        )),
        RuntimeDomainEvent::EconomicContractOpened {
            contract_id,
            counterparty_agent_id,
            settlement_amount,
            expires_at,
            ..
        } => Some(runtime_structured_event(
            "runtime.gameplay.economic_contract_opened",
            format!(
                "contract_id={} counterparty={} settlement_amount={settlement_amount} expires_at={expires_at}",
                fallback_non_empty(contract_id, "unknown_contract"),
                fallback_non_empty(counterparty_agent_id, "unknown_counterparty"),
            ),
        )),
        RuntimeDomainEvent::EconomicContractAccepted {
            contract_id,
            accepter_agent_id,
        } => Some(runtime_structured_event(
            "runtime.gameplay.economic_contract_accepted",
            format!(
                "contract_id={} accepter={}",
                fallback_non_empty(contract_id, "unknown_contract"),
                fallback_non_empty(accepter_agent_id, "unknown_accepter"),
            ),
        )),
        RuntimeDomainEvent::EconomicContractSettled {
            contract_id,
            success,
            transfer_amount,
            tax_amount,
            ..
        } => Some(runtime_structured_event(
            "runtime.gameplay.economic_contract_settled",
            format!(
                "contract_id={} success={success} transfer_amount={transfer_amount} tax_amount={tax_amount}",
                fallback_non_empty(contract_id, "unknown_contract"),
            ),
        )),
        RuntimeDomainEvent::EconomicContractExpired {
            contract_id,
            creator_agent_id,
            counterparty_agent_id,
            ..
        } => Some(runtime_structured_event(
            "runtime.gameplay.economic_contract_expired",
            format!(
                "contract_id={} creator={} counterparty={}",
                fallback_non_empty(contract_id, "unknown_contract"),
                fallback_non_empty(creator_agent_id, "unknown_creator"),
                fallback_non_empty(counterparty_agent_id, "unknown_counterparty"),
            ),
        )),
        RuntimeDomainEvent::MetaProgressGranted {
            target_agent_id,
            track,
            points,
            achievement_id,
            ..
        } => Some(runtime_structured_event(
            "runtime.gameplay.meta_progress_granted",
            format!(
                "target={} track={} points={points} achievement_id={}",
                fallback_non_empty(target_agent_id, "unknown_target"),
                fallback_non_empty(track, "unknown_track"),
                achievement_id.as_deref().unwrap_or("none"),
            ),
        )),
        _ => None,
    }
}

pub(super) fn runtime_reject_reason_to_simulator(
    reason: &RuntimeRejectReason,
) -> SimulatorRejectReason {
    match reason {
        RuntimeRejectReason::AgentAlreadyExists { agent_id } => {
            SimulatorRejectReason::AgentAlreadyExists {
                agent_id: agent_id.clone(),
            }
        }
        RuntimeRejectReason::AgentNotFound { agent_id } => SimulatorRejectReason::AgentNotFound {
            agent_id: agent_id.clone(),
        },
        RuntimeRejectReason::AgentsNotCoLocated {
            agent_id,
            other_agent_id,
        } => SimulatorRejectReason::AgentsNotCoLocated {
            agent_id: agent_id.clone(),
            other_agent_id: other_agent_id.clone(),
        },
        RuntimeRejectReason::InvalidAmount { amount } => {
            SimulatorRejectReason::InvalidAmount { amount: *amount }
        }
        RuntimeRejectReason::InsufficientResource {
            agent_id,
            kind,
            requested,
            available,
        } => SimulatorRejectReason::InsufficientResource {
            owner: ResourceOwner::Agent {
                agent_id: agent_id.clone(),
            },
            kind: *kind,
            requested: *requested,
            available: *available,
        },
        RuntimeRejectReason::FactoryNotFound { factory_id } => {
            SimulatorRejectReason::FacilityNotFound {
                facility_id: factory_id.clone(),
            }
        }
        RuntimeRejectReason::RuleDenied { notes } => SimulatorRejectReason::RuleDenied {
            notes: notes.clone(),
        },
        other => SimulatorRejectReason::RuleDenied {
            notes: vec![format!("runtime reject: {other:?}")],
        },
    }
}

fn runtime_fallback_event_kind(runtime_event: &RuntimeWorldEvent) -> WorldEventKind {
    let (kind, domain_kind) = runtime_event_kind_label(&runtime_event.body);
    WorldEventKind::RuntimeEvent { kind, domain_kind }
}

fn runtime_structured_event(kind: &str, domain_kind: String) -> WorldEventKind {
    WorldEventKind::RuntimeEvent {
        kind: kind.to_string(),
        domain_kind: Some(domain_kind),
    }
}

fn fallback_non_empty<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback
    } else {
        trimmed
    }
}

fn runtime_event_kind_label(body: &RuntimeWorldEventBody) -> (String, Option<String>) {
    let label = match body {
        RuntimeWorldEventBody::Domain(_) => "domain",
        RuntimeWorldEventBody::EffectQueued(_) => "effect_queued",
        RuntimeWorldEventBody::ReceiptAppended(_) => "receipt_appended",
        RuntimeWorldEventBody::PolicyDecisionRecorded(_) => "policy_decision_recorded",
        RuntimeWorldEventBody::RuleDecisionRecorded(_) => "rule_decision_recorded",
        RuntimeWorldEventBody::ActionOverridden(_) => "action_overridden",
        RuntimeWorldEventBody::Governance(_) => "governance",
        RuntimeWorldEventBody::ModuleEvent(_) => "module_event",
        RuntimeWorldEventBody::ModuleCallFailed(_) => "module_call_failed",
        RuntimeWorldEventBody::ModuleEmitted(_) => "module_emitted",
        RuntimeWorldEventBody::ModuleStateUpdated(_) => "module_state_updated",
        RuntimeWorldEventBody::ModuleRuntimeCharged(_) => "module_runtime_charged",
        RuntimeWorldEventBody::SnapshotCreated(_) => "snapshot_created",
        RuntimeWorldEventBody::ManifestUpdated(_) => "manifest_updated",
        RuntimeWorldEventBody::RollbackApplied(_) => "rollback_applied",
    };
    (label.to_string(), None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::GeoPos;
    use crate::runtime::SnapshotMeta;
    use crate::simulator::WorldScenario;
    use crate::viewer::runtime_live::{ViewerRuntimeLiveServer, ViewerRuntimeLiveServerConfig};

    #[test]
    fn map_runtime_domain_event_agent_registered_uses_runtime_location_id() {
        let event = RuntimeDomainEvent::AgentRegistered {
            agent_id: "a1".to_string(),
            pos: GeoPos::new(12.0, 34.0, 56.0),
        };
        let mapped =
            map_runtime_domain_event(&event, &WorldConfig::default()).expect("mapped event");
        match mapped {
            WorldEventKind::AgentRegistered {
                agent_id,
                location_id,
                pos,
            } => {
                assert_eq!(agent_id, "a1");
                assert_eq!(location_id, "runtime:12:34:56");
                assert_eq!(pos, GeoPos::new(12.0, 34.0, 56.0));
            }
            other => panic!("unexpected mapped event: {other:?}"),
        }
    }

    #[test]
    fn map_runtime_domain_event_agent_moved_sets_distance_and_cost() {
        let config = WorldConfig::default();
        let event = RuntimeDomainEvent::AgentMoved {
            agent_id: "a1".to_string(),
            from: GeoPos::new(0.0, 0.0, 0.0),
            to: GeoPos::new(100_000.0, 0.0, 0.0),
        };
        let mapped = map_runtime_domain_event(&event, &config).expect("mapped event");
        match mapped {
            WorldEventKind::AgentMoved {
                distance_cm,
                electricity_cost,
                ..
            } => {
                assert_eq!(distance_cm, 100_000);
                assert_eq!(electricity_cost, config.movement_cost(distance_cm));
            }
            other => panic!("unexpected mapped event: {other:?}"),
        }
    }

    #[test]
    fn map_runtime_domain_event_action_accepted_emits_structured_runtime_event() {
        let event = RuntimeDomainEvent::ActionAccepted {
            action_id: 7,
            action_kind: "".to_string(),
            actor_id: "".to_string(),
            eta_ticks: 3,
            notes: vec!["accepted".to_string()],
        };
        let mapped =
            map_runtime_domain_event(&event, &WorldConfig::default()).expect("mapped event");
        match mapped {
            WorldEventKind::RuntimeEvent { kind, domain_kind } => {
                assert_eq!(kind, "runtime.action_accepted");
                let summary = domain_kind.expect("domain summary");
                assert!(summary.contains("action_id=7"));
                assert!(summary.contains("action_kind=unknown_action"));
                assert!(summary.contains("actor_id=system"));
                assert!(summary.contains("eta_ticks=3"));
            }
            other => panic!("unexpected mapped event: {other:?}"),
        }
    }

    #[test]
    fn map_runtime_domain_event_governance_finalize_keeps_compat_fallbacks() {
        let event = RuntimeDomainEvent::GovernanceProposalFinalized {
            proposal_key: "proposal.alpha".to_string(),
            winning_option: None,
            winning_weight: 0,
            total_weight: 0,
            passed: false,
        };
        let mapped =
            map_runtime_domain_event(&event, &WorldConfig::default()).expect("mapped event");
        match mapped {
            WorldEventKind::RuntimeEvent { kind, domain_kind } => {
                assert_eq!(kind, "runtime.gameplay.governance_proposal_finalized");
                let summary = domain_kind.expect("domain summary");
                assert!(summary.contains("proposal_key=proposal.alpha"));
                assert!(summary.contains("winning_option=none"));
                assert!(summary.contains("passed=false"));
            }
            other => panic!("unexpected mapped event: {other:?}"),
        }
    }

    #[test]
    fn runtime_reject_reason_maps_agent_not_found() {
        let reason = RuntimeRejectReason::AgentNotFound {
            agent_id: "ghost".to_string(),
        };
        let mapped = runtime_reject_reason_to_simulator(&reason);
        match mapped {
            SimulatorRejectReason::AgentNotFound { agent_id } => {
                assert_eq!(agent_id, "ghost");
            }
            other => panic!("unexpected reject mapping: {other:?}"),
        }
    }

    #[test]
    fn runtime_reject_reason_unmapped_falls_back_to_rule_denied() {
        let reason = RuntimeRejectReason::InsufficientMaterial {
            material_kind: "iron".to_string(),
            requested: 10,
            available: 0,
        };
        let mapped = runtime_reject_reason_to_simulator(&reason);
        match mapped {
            SimulatorRejectReason::RuleDenied { notes } => {
                assert_eq!(notes.len(), 1);
                assert!(notes[0].contains("runtime reject"));
            }
            other => panic!("unexpected reject mapping: {other:?}"),
        }
    }

    #[test]
    fn map_runtime_event_fallback_includes_runtime_payload() {
        let event = RuntimeWorldEvent {
            id: 9,
            time: 42,
            caused_by: None,
            body: RuntimeWorldEventBody::SnapshotCreated(SnapshotMeta { journal_len: 1 }),
        };
        let mapped = map_runtime_event(&event, &WorldConfig::default());
        assert!(matches!(mapped.kind, WorldEventKind::RuntimeEvent { .. }));
        assert!(mapped.runtime_event.is_some());
        assert_eq!(mapped.id, 9);
        assert_eq!(mapped.time, 42);
    }

    #[test]
    fn runtime_live_snapshot_includes_runtime_snapshot_payload() {
        let server = ViewerRuntimeLiveServer::new(ViewerRuntimeLiveServerConfig::new(
            WorldScenario::Minimal,
        ))
        .expect("runtime server");
        let snapshot = server.compat_snapshot();
        assert!(snapshot.runtime_snapshot.is_some());
        assert_eq!(
            snapshot.runtime_snapshot.as_ref().unwrap().journal_len,
            server.world.snapshot().journal_len
        );
    }
}
