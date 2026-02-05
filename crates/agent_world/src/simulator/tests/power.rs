use super::*;
use crate::geometry::great_circle_distance_cm;

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

#[test]
fn power_store_and_draw_actions() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "hub".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterPowerPlant {
        facility_id: "plant-1".to_string(),
        location_id: "loc-1".to_string(),
        owner: ResourceOwner::Location {
            location_id: "loc-1".to_string(),
        },
        capacity_per_tick: 100,
        fuel_cost_per_pu: 0,
        maintenance_cost: 0,
        efficiency: 1.0,
        degradation: 0.0,
    });
    kernel.submit_action(Action::RegisterPowerStorage {
        facility_id: "storage-1".to_string(),
        location_id: "loc-1".to_string(),
        owner: ResourceOwner::Location {
            location_id: "loc-1".to_string(),
        },
        capacity: 100,
        current_level: 0,
        charge_efficiency: 1.0,
        discharge_efficiency: 1.0,
        max_charge_rate: 100,
        max_discharge_rate: 100,
    });
    kernel.step_until_empty();
    kernel.process_power_generation_tick();

    kernel.submit_action(Action::StorePower {
        storage_id: "storage-1".to_string(),
        amount: 40,
    });
    let event = kernel.step().unwrap();
    match event.kind {
        WorldEventKind::Power(PowerEvent::PowerStored {
            storage_id,
            input,
            stored,
            ..
        }) => {
            assert_eq!(storage_id, "storage-1");
            assert_eq!(input, 40);
            assert_eq!(stored, 40);
        }
        other => panic!("unexpected event: {other:?}"),
    }

    kernel.submit_action(Action::DrawPower {
        storage_id: "storage-1".to_string(),
        amount: 15,
    });
    let event = kernel.step().unwrap();
    match event.kind {
        WorldEventKind::Power(PowerEvent::PowerDischarged {
            storage_id,
            output,
            drawn,
            ..
        }) => {
            assert_eq!(storage_id, "storage-1");
            assert_eq!(output, 15);
            assert_eq!(drawn, 15);
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn power_buy_applies_transfer_loss() {
    let mut config = WorldConfig::default();
    config.power.transfer_loss_per_km_bps = 1000;
    config.power.transfer_max_distance_km = 10;
    let mut kernel = WorldKernel::with_config(config.clone());

    let loc1_pos = pos(0.0, 0.0);
    let loc2_pos = pos(0.0, 0.001);
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "source".to_string(),
        pos: loc1_pos,
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "sink".to_string(),
        pos: loc2_pos,
    });
    kernel.submit_action(Action::RegisterPowerPlant {
        facility_id: "plant-1".to_string(),
        location_id: "loc-1".to_string(),
        owner: ResourceOwner::Location {
            location_id: "loc-1".to_string(),
        },
        capacity_per_tick: 200,
        fuel_cost_per_pu: 0,
        maintenance_cost: 0,
        efficiency: 1.0,
        degradation: 0.0,
    });
    kernel.step_until_empty();
    kernel.process_power_generation_tick();

    let distance_cm = great_circle_distance_cm(loc1_pos, loc2_pos);
    let distance_km = (distance_cm + CM_PER_KM - 1) / CM_PER_KM;
    let amount = 100;
    let expected_loss = (amount as i128)
        .saturating_mul(distance_km as i128)
        .saturating_mul(config.power.transfer_loss_per_km_bps as i128)
        / 10_000;

    kernel.submit_action(Action::BuyPower {
        buyer: ResourceOwner::Location {
            location_id: "loc-2".to_string(),
        },
        seller: ResourceOwner::Location {
            location_id: "loc-1".to_string(),
        },
        amount,
        price_per_pu: 1,
    });
    let event = kernel.step().unwrap();
    match event.kind {
        WorldEventKind::Power(PowerEvent::PowerTransferred {
            from,
            to,
            amount: transferred,
            loss,
            price_per_pu,
        }) => {
            assert_eq!(
                from,
                ResourceOwner::Location {
                    location_id: "loc-1".to_string()
                }
            );
            assert_eq!(
                to,
                ResourceOwner::Location {
                    location_id: "loc-2".to_string()
                }
            );
            assert_eq!(transferred, amount);
            assert_eq!(loss, expected_loss as i64);
            assert_eq!(price_per_pu, 1);
        }
        other => panic!("unexpected event: {other:?}"),
    }

    let source_power = kernel
        .model()
        .locations
        .get("loc-1")
        .unwrap()
        .resources
        .get(ResourceKind::Electricity);
    let sink_power = kernel
        .model()
        .locations
        .get("loc-2")
        .unwrap()
        .resources
        .get(ResourceKind::Electricity);
    assert_eq!(source_power, 200 - amount);
    assert_eq!(sink_power, amount - expected_loss as i64);
}

#[test]
fn power_transfer_rejects_out_of_range() {
    let mut config = WorldConfig::default();
    config.power.transfer_loss_per_km_bps = 10;
    config.power.transfer_max_distance_km = 0;
    let mut kernel = WorldKernel::with_config(config);

    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "source".to_string(),
        pos: pos(0.0, 0.0),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "sink".to_string(),
        pos: pos(0.0, 0.01),
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
    kernel.process_power_generation_tick();

    kernel.submit_action(Action::BuyPower {
        buyer: ResourceOwner::Location {
            location_id: "loc-2".to_string(),
        },
        seller: ResourceOwner::Location {
            location_id: "loc-1".to_string(),
        },
        amount: 10,
        price_per_pu: 1,
    });
    let event = kernel.step().unwrap();
    match event.kind {
        WorldEventKind::ActionRejected { reason } => {
            assert!(matches!(
                reason,
                RejectReason::PowerTransferDistanceExceeded { .. }
            ));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}
