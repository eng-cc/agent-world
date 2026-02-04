use super::*;

#[test]
fn kernel_registers_and_moves_agent() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(1.0, 1.0),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step().unwrap();
    let starting_energy = 500;
    kernel.model().agents.get("agent-1").unwrap();
    let mut kernel2 = WorldKernel::new();
    kernel2.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel2.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(1.0, 1.0),
    });
    kernel2.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel2.step_until_empty();

    kernel2.submit_action(Action::TransferResource {
        from: ResourceOwner::Location {
            location_id: "loc-1".to_string(),
        },
        to: ResourceOwner::Agent {
            agent_id: "agent-1".to_string(),
        },
        kind: ResourceKind::Electricity,
        amount: starting_energy,
    });
    let config = WorldConfig {
        visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
        move_cost_per_km_electricity: 0,
        ..Default::default()
    };
    let mut kernel = WorldKernel::with_config(config);
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(1.0, 1.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: "loc-2".to_string(),
    });
    let event = kernel.step().unwrap();
    match event.kind {
        WorldEventKind::AgentMoved {
            agent_id,
            from,
            to,
            distance_cm,
            electricity_cost,
        } => {
            assert_eq!(agent_id, "agent-1");
            assert_eq!(from, "loc-1");
            assert_eq!(to, "loc-2");
            assert!(distance_cm > 0);
            assert_eq!(electricity_cost, 0);
        }
        other => panic!("unexpected event: {other:?}"),
    }

    let agent = kernel.model().agents.get("agent-1").unwrap();
    assert_eq!(agent.location_id, "loc-2");
    assert_eq!(agent.pos, pos(1.0, 1.0));
}

#[test]
fn kernel_move_requires_energy() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(1.0, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: "loc-2".to_string(),
    });
    let event = kernel.step().unwrap();
    match event.kind {
        WorldEventKind::ActionRejected { reason } => {
            assert!(matches!(reason, RejectReason::InsufficientResource { .. }));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn kernel_rejects_move_to_same_location() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: "loc-1".to_string(),
    });
    let event = kernel.step().unwrap();
    match event.kind {
        WorldEventKind::ActionRejected { reason } => {
            assert!(matches!(reason, RejectReason::AgentAlreadyAtLocation { .. }));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn kernel_observe_visibility_range() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "far".to_string(),
        pos: pos(10.0, 10.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-2".to_string(),
        location_id: "loc-2".to_string(),
    });
    kernel.step_until_empty();

    let obs = kernel.observe("agent-1").unwrap();
    assert!(obs.visible_agents.is_empty());
    assert!(obs.visible_locations.iter().any(|loc| loc.location_id == "loc-1"));
    assert!(!obs.visible_locations.iter().any(|loc| loc.location_id == "loc-2"));
}

#[test]
fn kernel_config_overrides_defaults() {
    let config = WorldConfig {
        visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM * 2,
        move_cost_per_km_electricity: DEFAULT_MOVE_COST_PER_KM_ELECTRICITY * 2,
        ..Default::default()
    };
    let kernel = WorldKernel::with_config(config);
    assert_eq!(
        kernel.config().visibility_range_cm,
        DEFAULT_VISIBILITY_RANGE_CM * 2
    );
    assert_eq!(
        kernel.config().move_cost_per_km_electricity,
        DEFAULT_MOVE_COST_PER_KM_ELECTRICITY * 2
    );
}

#[test]
fn kernel_transfer_requires_colocation() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(10.0, 10.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-2".to_string(),
        location_id: "loc-2".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::TransferResource {
        from: ResourceOwner::Agent {
            agent_id: "agent-1".to_string(),
        },
        to: ResourceOwner::Agent {
            agent_id: "agent-2".to_string(),
        },
        kind: ResourceKind::Electricity,
        amount: 10,
    });
    let event = kernel.step().unwrap();
    match event.kind {
        WorldEventKind::ActionRejected { reason } => {
            assert!(matches!(reason, RejectReason::AgentsNotCoLocated { .. }));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn kernel_closed_loop_example() {
    let config = WorldConfig {
        visibility_range_cm: DEFAULT_VISIBILITY_RANGE_CM,
        move_cost_per_km_electricity: 0,
        ..Default::default()
    };
    let mut kernel = WorldKernel::with_config(config);
    let loc1_pos = pos(0.0, 0.0);
    let loc2_pos = pos(2.0, 2.0);
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "plant".to_string(),
        pos: loc1_pos,
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "lab".to_string(),
        pos: loc2_pos,
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-2".to_string(),
        location_id: "loc-2".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: "loc-2".to_string(),
    });
    kernel.step().unwrap();

    let agent = kernel.model().agents.get("agent-1").unwrap();
    assert_eq!(agent.location_id, "loc-2");
}
