use super::*;

#[test]
fn power_idle_consumption_depletes_agent() {
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

    let events = kernel.process_power_tick();
    assert!(!events.is_empty());
}

#[test]
fn power_shutdown_agent_cannot_move() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(0.0, 1.0),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    let ticks = kernel
        .config()
        .power
        .default_power_level
        .saturating_add(1)
        .max(1) as usize;
    for _ in 0..ticks {
        kernel.process_power_tick();
    }

    kernel.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: "loc-2".to_string(),
    });
    let event = kernel.step().unwrap();
    assert!(matches!(
        event.kind,
        WorldEventKind::ActionRejected {
            reason: RejectReason::AgentShutdown { .. }
        }
    ));
}

#[test]
fn power_charge_recovers_agent() {
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

    let ticks = kernel
        .config()
        .power
        .default_power_level
        .saturating_add(1)
        .max(1) as usize;
    for _ in 0..ticks {
        kernel.process_power_tick();
    }

    assert!(kernel.is_agent_shutdown(&"agent-1".to_string()));
    kernel.charge_agent_power(&"agent-1".to_string(), 100);
    assert!(!kernel.is_agent_shutdown(&"agent-1".to_string()));
}

#[test]
fn power_consume_for_decision() {
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

    let event = kernel.consume_agent_power(
        &"agent-1".to_string(),
        kernel.config().power.decision_cost,
        ConsumeReason::Decision,
    );
    assert!(event.is_some());
}

#[test]
fn shutdown_agents_list() {
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

    let ticks = kernel
        .config()
        .power
        .default_power_level
        .saturating_add(1)
        .max(1) as usize;
    for _ in 0..ticks {
        kernel.process_power_tick();
    }

    let shutdown = kernel.shutdown_agents();
    assert!(shutdown.contains(&"agent-1".to_string()));
}

#[test]
fn power_generation_creates_electricity() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "plant".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterPowerPlant {
        facility_id: "plant-1".to_string(),
        location_id: "loc-1".to_string(),
        owner: ResourceOwner::Location {
            location_id: "loc-1".to_string(),
        },
        capacity_per_tick: 50,
        fuel_cost_per_pu: 0,
        maintenance_cost: 0,
        efficiency: 1.0,
        degradation: 0.0,
    });
    kernel.step_until_empty();

    let events = kernel.process_power_generation_tick();
    assert!(!events.is_empty());
}

#[test]
fn power_storage_charge_and_discharge() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "storage".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterPowerStorage {
        facility_id: "storage-1".to_string(),
        location_id: "loc-1".to_string(),
        owner: ResourceOwner::Location {
            location_id: "loc-1".to_string(),
        },
        capacity: 100,
        current_level: 50,
        charge_efficiency: 1.0,
        discharge_efficiency: 1.0,
        max_charge_rate: 50,
        max_discharge_rate: 50,
    });
    kernel.step_until_empty();

    let discharge_event = kernel.discharge_power_storage(&"storage-1".to_string(), 25);
    assert!(discharge_event.is_some());

    let charge_event = kernel.charge_power_storage(&"storage-1".to_string(), 10);
    assert!(charge_event.is_some());
}
