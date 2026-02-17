use super::*;

#[test]
fn power_idle_consumption_depletes_agent() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
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
fn power_tick_dissipates_more_heat_when_hotter() {
    let mut config = WorldConfig::default();
    config.power.idle_cost_per_tick = 0;
    config.physics.radiation_floor = 0;
    config.physics.max_harvest_per_tick = 100;
    config.physics.thermal_capacity = 100;
    config.physics.thermal_dissipation = 10;
    config.physics.thermal_dissipation_gradient_bps = 10_000;
    let mut kernel = WorldKernel::with_config(config);

    let mut low_profile = LocationProfile::default();
    low_profile.radiation_emission_per_tick = 5;
    let mut high_profile = LocationProfile::default();
    high_profile.radiation_emission_per_tick = 40;

    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-low".to_string(),
        name: "low".to_string(),
        pos: pos(0.0, 0.0),
        profile: low_profile,
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-high".to_string(),
        name: "high".to_string(),
        pos: pos(10_000_000.0, 0.0),
        profile: high_profile,
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-low".to_string(),
        location_id: "loc-low".to_string(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-high".to_string(),
        location_id: "loc-high".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::HarvestRadiation {
        agent_id: "agent-low".to_string(),
        max_amount: 100,
    });
    kernel.submit_action(Action::HarvestRadiation {
        agent_id: "agent-high".to_string(),
        max_amount: 100,
    });
    kernel.step_until_empty();

    let low_before = kernel
        .model()
        .agents
        .get("agent-low")
        .expect("agent-low exists")
        .thermal
        .heat;
    let high_before = kernel
        .model()
        .agents
        .get("agent-high")
        .expect("agent-high exists")
        .thermal
        .heat;

    assert!(high_before > low_before);

    kernel.process_power_tick();

    let low_after = kernel
        .model()
        .agents
        .get("agent-low")
        .expect("agent-low exists")
        .thermal
        .heat;
    let high_after = kernel
        .model()
        .agents
        .get("agent-high")
        .expect("agent-high exists")
        .thermal
        .heat;

    let low_drop = low_before.saturating_sub(low_after);
    let high_drop = high_before.saturating_sub(high_after);

    assert!(high_drop > low_drop);
}

#[test]
fn power_tick_thermal_dissipation_never_makes_heat_negative() {
    let mut config = WorldConfig::default();
    config.power.idle_cost_per_tick = 0;
    config.physics.radiation_floor = 0;
    config.physics.max_harvest_per_tick = 100;
    config.physics.thermal_capacity = 100;
    config.physics.thermal_dissipation = 50;
    config.physics.thermal_dissipation_gradient_bps = 20_000;
    let mut kernel = WorldKernel::with_config(config);

    let mut profile = LocationProfile::default();
    profile.radiation_emission_per_tick = 1;
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-hot".to_string(),
        name: "hot".to_string(),
        pos: pos(0.0, 0.0),
        profile,
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-hot".to_string(),
        location_id: "loc-hot".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::HarvestRadiation {
        agent_id: "agent-hot".to_string(),
        max_amount: 100,
    });
    kernel.step_until_empty();

    kernel.process_power_tick();

    let heat = kernel
        .model()
        .agents
        .get("agent-hot")
        .expect("agent-hot exists")
        .thermal
        .heat;
    assert_eq!(heat, 0);
}

#[test]
fn power_shutdown_agent_cannot_move() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(0.0, 1.0),
        profile: LocationProfile::default(),
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
        profile: LocationProfile::default(),
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
        profile: LocationProfile::default(),
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
        profile: LocationProfile::default(),
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
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.submit_action(Action::RegisterPowerPlant {
        facility_id: "plant-1".to_string(),
        location_id: "loc-1".to_string(),
        owner: ResourceOwner::Agent {
            agent_id: "agent-1".to_string(),
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
    let level = kernel
        .model()
        .agents
        .get("agent-1")
        .expect("agent exists")
        .resources
        .get(ResourceKind::Electricity);
    assert_eq!(level, 50);
}

#[test]
fn build_radiation_power_factory_registers_plant_and_generates_to_owner() {
    let mut config = WorldConfig::default();
    config.economy.factory_build_electricity_cost = 0;
    config.economy.factory_build_hardware_cost = 0;
    config.economy.radiation_power_plant_output_per_tick = 12;
    config.physics.radiation_floor = 0;
    config.physics.radiation_floor_cap_per_tick = 0;
    let mut kernel = WorldKernel::with_config(config);

    let mut profile = LocationProfile::default();
    profile.radiation_emission_per_tick = 100;
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "plant".to_string(),
        pos: pos(0.0, 0.0),
        profile,
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::BuildFactory {
        owner: ResourceOwner::Agent {
            agent_id: "agent-1".to_string(),
        },
        location_id: "loc-1".to_string(),
        factory_id: "factory.power.alpha".to_string(),
        factory_kind: "factory.power.radiation.mk1".to_string(),
    });
    let event = kernel.step().expect("build radiation power factory");
    assert!(matches!(event.kind, WorldEventKind::FactoryBuilt { .. }));
    assert!(kernel.model().factories.contains_key("factory.power.alpha"));
    assert!(kernel
        .model()
        .power_plants
        .contains_key("factory.power.alpha"));

    let events = kernel.process_power_generation_tick();
    assert!(!events.is_empty());
    let level = kernel
        .model()
        .agents
        .get("agent-1")
        .expect("agent exists")
        .resources
        .get(ResourceKind::Electricity);
    assert_eq!(level, 12);
    let location_level = kernel
        .model()
        .locations
        .get("loc-1")
        .expect("location exists")
        .resources
        .get(ResourceKind::Electricity);
    assert_eq!(location_level, 0);
}

#[test]
fn power_storage_charge_and_discharge() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "storage".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
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
    assert!(discharge_event.is_none());

    let charge_event = kernel.charge_power_storage(&"storage-1".to_string(), 10);
    assert!(charge_event.is_none());
}

#[test]
fn power_store_and_draw_actions() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "hub".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
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

    kernel.submit_action(Action::StorePower {
        storage_id: "storage-1".to_string(),
        amount: 40,
    });
    let event = kernel.step().unwrap();
    assert!(matches!(
        event.kind,
        WorldEventKind::ActionRejected {
            reason: RejectReason::RuleDenied { .. }
        }
    ));

    kernel.submit_action(Action::DrawPower {
        storage_id: "storage-1".to_string(),
        amount: 15,
    });
    let event = kernel.step().unwrap();
    assert!(matches!(
        event.kind,
        WorldEventKind::ActionRejected {
            reason: RejectReason::RuleDenied { .. }
        }
    ));
}

#[test]
fn power_buy_rejects_when_location_owner_involved() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "source".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "sink".to_string(),
        pos: pos(0.0, CM_PER_KM as f64),
        profile: LocationProfile::default(),
    });
    kernel.step_until_empty();

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
    assert!(matches!(
        event.kind,
        WorldEventKind::ActionRejected {
            reason: RejectReason::RuleDenied { .. }
        }
    ));
}

#[test]
fn power_buy_allows_between_colocated_agents() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "hub".to_string(),
        name: "hub".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "seller".to_string(),
        location_id: "hub".to_string(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "buyer".to_string(),
        location_id: "hub".to_string(),
    });
    kernel.step_until_empty();
    seed_owner_resource(
        &mut kernel,
        ResourceOwner::Agent {
            agent_id: "seller".to_string(),
        },
        ResourceKind::Electricity,
        20,
    );

    kernel.submit_action(Action::BuyPower {
        buyer: ResourceOwner::Agent {
            agent_id: "buyer".to_string(),
        },
        seller: ResourceOwner::Agent {
            agent_id: "seller".to_string(),
        },
        amount: 10,
        price_per_pu: 1,
    });
    let event = kernel.step().unwrap();
    assert!(matches!(
        event.kind,
        WorldEventKind::Power(PowerEvent::PowerTransferred { loss: 0, .. })
    ));

    let seller = kernel.model().agents.get("seller").expect("seller exists");
    let buyer = kernel.model().agents.get("buyer").expect("buyer exists");
    assert_eq!(seller.resources.get(ResourceKind::Electricity), 10);
    assert_eq!(buyer.resources.get(ResourceKind::Electricity), 10);
}

#[test]
fn power_transfer_rejects_when_location_owner_involved() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "source".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "seller".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();
    seed_owner_resource(
        &mut kernel,
        ResourceOwner::Agent {
            agent_id: "seller".to_string(),
        },
        ResourceKind::Electricity,
        20,
    );

    kernel.submit_action(Action::SellPower {
        seller: ResourceOwner::Agent {
            agent_id: "seller".to_string(),
        },
        buyer: ResourceOwner::Location {
            location_id: "loc-1".to_string(),
        },
        amount: 10,
        price_per_pu: 1,
    });
    let event = kernel.step().unwrap();
    assert!(matches!(
        event.kind,
        WorldEventKind::ActionRejected {
            reason: RejectReason::RuleDenied { .. }
        }
    ));
}
