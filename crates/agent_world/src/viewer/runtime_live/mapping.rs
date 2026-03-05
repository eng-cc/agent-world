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
