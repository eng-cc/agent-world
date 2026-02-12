use super::super::*;
use super::pos;
use crate::models::{CargoEntityEntry, CargoEntityKind};

fn default_module_sandbox() -> BuiltinModuleSandbox {
    BuiltinModuleSandbox::with_preferred_fallback(Box::new(WasmExecutor::new(
        WasmExecutorConfig::default(),
    )))
    .register_builtin(M1_SENSOR_MODULE_ID, M1SensorModule::default())
    .register_builtin(M1_MOBILITY_MODULE_ID, M1MobilityModule::default())
    .register_builtin(M1_MEMORY_MODULE_ID, M1MemoryModule::default())
    .register_builtin(M1_STORAGE_CARGO_MODULE_ID, M1StorageCargoModule)
}

fn scenario_module_sandbox() -> BuiltinModuleSandbox {
    BuiltinModuleSandbox::with_preferred_fallback(Box::new(WasmExecutor::new(
        WasmExecutorConfig::default(),
    )))
    .register_builtin(
        M1_RADIATION_POWER_MODULE_ID,
        M1RadiationPowerModule::default(),
    )
    .register_builtin(M1_STORAGE_POWER_MODULE_ID, M1StoragePowerModule::default())
    .register_builtin(M1_SENSOR_MODULE_ID, M1SensorModule::default())
    .register_builtin(M1_MOBILITY_MODULE_ID, M1MobilityModule::default())
    .register_builtin(M1_MEMORY_MODULE_ID, M1MemoryModule::default())
    .register_builtin(M1_STORAGE_CARGO_MODULE_ID, M1StorageCargoModule)
}

fn setup_world_with_default_modules() -> (World, BuiltinModuleSandbox) {
    let mut world = World::new();
    world
        .install_m1_agent_default_modules("bootstrap")
        .expect("install default modules");

    (world, default_module_sandbox())
}

fn setup_world_with_scenario_modules(
    config: M1ScenarioBootstrapConfig,
) -> (World, BuiltinModuleSandbox) {
    let mut world = World::new();
    world
        .install_m1_scenario_bootstrap_modules("bootstrap", config)
        .expect("install scenario bootstrap modules");

    (world, scenario_module_sandbox())
}

fn last_module_state(world: &World, module_id: &str) -> Option<Vec<u8>> {
    world
        .journal()
        .events
        .iter()
        .rev()
        .find_map(|event| match &event.body {
            WorldEventBody::ModuleStateUpdated(update) if update.module_id == module_id => {
                Some(update.state.clone())
            }
            _ => None,
        })
}

fn last_domain_event<'a>(world: &'a World) -> Option<&'a DomainEvent> {
    world
        .journal()
        .events
        .iter()
        .rev()
        .find_map(|event| match &event.body {
            WorldEventBody::Domain(domain) => Some(domain),
            _ => None,
        })
}

#[test]
fn default_sensor_module_emits_observation() {
    let (mut world, mut sandbox) = setup_world_with_default_modules();

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world
        .step_with_modules(&mut sandbox)
        .expect("register agent step");

    world.submit_action(Action::QueryObservation {
        agent_id: "agent-1".to_string(),
    });
    world
        .step_with_modules(&mut sandbox)
        .expect("query observation step");

    let last = last_domain_event(&world).expect("last domain event");
    match last {
        DomainEvent::Observation { observation } => {
            assert_eq!(observation.agent_id, "agent-1");
        }
        other => panic!("unexpected domain event: {other:?}"),
    }
}

#[test]
fn default_mobility_module_rejects_zero_distance_move() {
    let (mut world, mut sandbox) = setup_world_with_default_modules();

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world
        .step_with_modules(&mut sandbox)
        .expect("register agent step");

    world.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: pos(0.0, 0.0),
    });
    world.step_with_modules(&mut sandbox).expect("move step");

    let last = last_domain_event(&world).expect("last domain event");
    match last {
        DomainEvent::ActionRejected {
            reason: RejectReason::RuleDenied { notes },
            ..
        } => {
            assert!(notes
                .iter()
                .any(|note| note.contains("equals current position")));
        }
        other => panic!("unexpected domain event: {other:?}"),
    }
}

#[test]
fn default_memory_module_records_domain_events() {
    let (mut world, mut sandbox) = setup_world_with_default_modules();

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world
        .step_with_modules(&mut sandbox)
        .expect("register agent step");

    world.submit_action(Action::QueryObservation {
        agent_id: "agent-1".to_string(),
    });
    world
        .step_with_modules(&mut sandbox)
        .expect("observation step");

    let state = last_module_state(&world, M1_MEMORY_MODULE_ID).expect("memory state update");
    let decoded: serde_json::Value =
        serde_cbor::from_slice(&state).expect("decode memory state as cbor value");
    let entries = decoded
        .get("entries")
        .and_then(|value| value.as_array())
        .expect("memory entries array");
    assert!(entries.iter().any(|entry| {
        entry.get("kind")
            == Some(&serde_json::Value::String(
                "domain.agent_registered".to_string(),
            ))
    }));
    assert!(entries.iter().any(|entry| {
        entry.get("kind") == Some(&serde_json::Value::String("domain.observation".to_string()))
    }));
}

#[test]
fn default_storage_cargo_module_tracks_expand_events() {
    let (mut world, mut sandbox) = setup_world_with_default_modules();

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world
        .step_with_modules(&mut sandbox)
        .expect("register agent step");

    world
        .add_agent_cargo_entity(
            "agent-1",
            CargoEntityEntry {
                entity_id: "iface-kit-1".to_string(),
                entity_kind: CargoEntityKind::InterfaceModuleItem,
                quantity: 1,
                size_per_unit: 1,
            },
        )
        .expect("seed cargo entry");

    world.submit_action(Action::ExpandBodyInterface {
        agent_id: "agent-1".to_string(),
        interface_module_item_id: "iface-kit-1".to_string(),
    });
    world
        .step_with_modules(&mut sandbox)
        .expect("expand body interface step");

    let state =
        last_module_state(&world, M1_STORAGE_CARGO_MODULE_ID).expect("cargo module state update");
    let decoded: serde_json::Value =
        serde_cbor::from_slice(&state).expect("decode cargo state as cbor value");
    let consumed = decoded
        .get("consumed_interface_items")
        .and_then(|value| value.as_object())
        .expect("consumed item map");
    assert_eq!(
        consumed.get("iface-kit-1"),
        Some(&serde_json::Value::Number(1_u64.into()))
    );
}

#[test]
fn scenario_modules_limit_mobility_before_sensor_when_power_low() {
    let (mut world, mut sandbox) =
        setup_world_with_scenario_modules(M1ScenarioBootstrapConfig::default());

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world
        .step_with_modules(&mut sandbox)
        .expect("register agent step");

    let mut rejected_by_storage = false;
    for idx in 0..8 {
        world.submit_action(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: pos((idx as f64 + 1.0) * 100_000.0, 0.0),
        });
        world
            .step_with_modules(&mut sandbox)
            .expect("move step with scenario modules");

        let Some(DomainEvent::ActionRejected {
            reason: RejectReason::RuleDenied { notes },
            ..
        }) = last_domain_event(&world)
        else {
            continue;
        };
        if notes
            .iter()
            .any(|note| note.contains("storage insufficient"))
        {
            rejected_by_storage = true;
            break;
        }
    }
    assert!(rejected_by_storage);

    world.submit_action(Action::QueryObservation {
        agent_id: "agent-1".to_string(),
    });
    world
        .step_with_modules(&mut sandbox)
        .expect("query observation after low power move rejection");

    let last = last_domain_event(&world).expect("last domain event");
    assert!(matches!(last, DomainEvent::Observation { .. }));
}

#[test]
fn scenario_modules_replay_keeps_state_consistent() {
    let (mut world, mut sandbox) =
        setup_world_with_scenario_modules(M1ScenarioBootstrapConfig::default());

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world
        .step_with_modules(&mut sandbox)
        .expect("register agent step");

    world
        .add_agent_cargo_entity(
            "agent-1",
            CargoEntityEntry {
                entity_id: "iface-kit-1".to_string(),
                entity_kind: CargoEntityKind::InterfaceModuleItem,
                quantity: 1,
                size_per_unit: 1,
            },
        )
        .expect("seed cargo entry");

    world.submit_action(Action::ExpandBodyInterface {
        agent_id: "agent-1".to_string(),
        interface_module_item_id: "iface-kit-1".to_string(),
    });
    world
        .step_with_modules(&mut sandbox)
        .expect("expand body interface step");

    world.submit_action(Action::QueryObservation {
        agent_id: "agent-1".to_string(),
    });
    world
        .step_with_modules(&mut sandbox)
        .expect("query observation step");

    let snapshot = world.snapshot();
    let journal = world.journal().clone();
    let restored = World::from_snapshot(snapshot, journal).expect("restore world");

    assert_eq!(restored.state(), world.state());
    assert_eq!(restored.module_registry(), world.module_registry());

    assert_eq!(last_domain_event(&restored), last_domain_event(&world));
}
