use super::*;
use crate::geometry::{GeoPos, DEFAULT_CLOUD_WIDTH_CM};
use std::sync::{Arc, Mutex};

#[test]
fn kernel_registers_and_moves_agent() {
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
        pos: pos(1.0, 1.0),
        profile: LocationProfile::default(),
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
        profile: LocationProfile::default(),
    });
    kernel2.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(1.0, 1.0),
        profile: LocationProfile::default(),
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
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(1.0, 1.0),
        profile: LocationProfile::default(),
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
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(1.0, 0.0),
        profile: LocationProfile::default(),
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
fn register_location_rejects_out_of_bounds() {
    let mut kernel = WorldKernel::new();
    let out_of_bounds = GeoPos {
        x_cm: (DEFAULT_CLOUD_WIDTH_CM + 1) as f64,
        y_cm: 0.0,
        z_cm: 0.0,
    };
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-oob".to_string(),
        name: "void".to_string(),
        pos: out_of_bounds,
        profile: LocationProfile::default(),
    });
    let event = kernel.step().unwrap();
    assert!(matches!(
        event.kind,
        WorldEventKind::ActionRejected {
            reason: RejectReason::PositionOutOfBounds { .. }
        }
    ));
}

#[test]
fn harvest_radiation_adds_electricity() {
    let mut kernel = WorldKernel::new();
    let mut profile = LocationProfile::default();
    profile.radiation_emission_per_tick = 50;
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-rad".to_string(),
        name: "rad".to_string(),
        pos: pos(0.0, 0.0),
        profile,
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-rad".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::HarvestRadiation {
        agent_id: "agent-1".to_string(),
        max_amount: 20,
    });
    let event = kernel.step().unwrap();
    match event.kind {
        WorldEventKind::RadiationHarvested {
            amount, available, ..
        } => {
            assert_eq!(amount, 20);
            assert_eq!(available, 51);
        }
        other => panic!("unexpected event: {other:?}"),
    }

    let agent = kernel.model().agents.get("agent-1").unwrap();
    assert_eq!(agent.resources.get(ResourceKind::Electricity), 20);
}

#[test]
fn harvest_radiation_respects_max_per_tick() {
    let mut config = WorldConfig::default();
    config.physics.max_harvest_per_tick = 5;
    let mut kernel = WorldKernel::with_config(config);

    let mut profile = LocationProfile::default();
    profile.radiation_emission_per_tick = 50;
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-rad".to_string(),
        name: "rad".to_string(),
        pos: pos(0.0, 0.0),
        profile,
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-rad".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::HarvestRadiation {
        agent_id: "agent-1".to_string(),
        max_amount: 20,
    });
    let event = kernel.step().unwrap();
    match event.kind {
        WorldEventKind::RadiationHarvested { amount, .. } => {
            assert_eq!(amount, 5);
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn harvest_radiation_applies_thermal_penalty() {
    let mut config = WorldConfig::default();
    config.physics.thermal_capacity = 5;
    config.physics.heat_factor = 1;
    config.physics.max_harvest_per_tick = 100;
    let mut kernel = WorldKernel::with_config(config);

    let mut profile = LocationProfile::default();
    profile.radiation_emission_per_tick = 50;
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-rad".to_string(),
        name: "rad".to_string(),
        pos: pos(0.0, 0.0),
        profile,
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-rad".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::HarvestRadiation {
        agent_id: "agent-1".to_string(),
        max_amount: 10,
    });
    let _ = kernel.step().unwrap();

    kernel.submit_action(Action::HarvestRadiation {
        agent_id: "agent-1".to_string(),
        max_amount: 10,
    });
    let event = kernel.step().unwrap();
    match event.kind {
        WorldEventKind::RadiationHarvested { amount, .. } => {
            assert!(amount < 10);
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn harvest_radiation_includes_nearby_sources_and_distance_decay() {
    let mut config = WorldConfig::default();
    config.physics.radiation_floor = 0;
    config.physics.radiation_decay_k = 0.0;
    config.physics.max_harvest_per_tick = 10_000;
    let mut kernel = WorldKernel::with_config(config);

    let source_near = LocationProfile {
        material: MaterialKind::Metal,
        radius_cm: 100,
        radiation_emission_per_tick: 90,
    };
    let source_far = LocationProfile {
        material: MaterialKind::Metal,
        radius_cm: 100,
        radiation_emission_per_tick: 90,
    };

    kernel.submit_action(Action::RegisterLocation {
        location_id: "harvest-site".to_string(),
        name: "harvest-site".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "source-near".to_string(),
        name: "source-near".to_string(),
        pos: pos(100.0, 0.0),
        profile: source_near,
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "source-far".to_string(),
        name: "source-far".to_string(),
        pos: pos(2_000.0, 0.0),
        profile: source_far,
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "harvest-site".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::HarvestRadiation {
        agent_id: "agent-1".to_string(),
        max_amount: 10_000,
    });

    let event = kernel.step().unwrap();
    let available = match event.kind {
        WorldEventKind::RadiationHarvested { available, .. } => available,
        other => panic!("unexpected event: {other:?}"),
    };

    assert!(available > 0);
    assert!(available < 180);
}

#[test]
fn harvest_radiation_uses_background_floor_when_no_source() {
    let mut config = WorldConfig::default();
    config.physics.radiation_floor = 3;
    config.physics.max_harvest_per_tick = 10;
    let mut kernel = WorldKernel::with_config(config);

    kernel.submit_action(Action::RegisterLocation {
        location_id: "site".to_string(),
        name: "site".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "site".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::HarvestRadiation {
        agent_id: "agent-1".to_string(),
        max_amount: 10,
    });

    let event = kernel.step().unwrap();
    match event.kind {
        WorldEventKind::RadiationHarvested {
            amount, available, ..
        } => {
            assert_eq!(available, 3);
            assert_eq!(amount, 3);
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn harvest_radiation_caps_background_floor_when_no_source() {
    let mut config = WorldConfig::default();
    config.physics.radiation_floor = 20;
    config.physics.radiation_floor_cap_per_tick = 4;
    config.physics.max_harvest_per_tick = 10;
    let mut kernel = WorldKernel::with_config(config);

    kernel.submit_action(Action::RegisterLocation {
        location_id: "site".to_string(),
        name: "site".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "site".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::HarvestRadiation {
        agent_id: "agent-1".to_string(),
        max_amount: 10,
    });

    let event = kernel.step().unwrap();
    match event.kind {
        WorldEventKind::RadiationHarvested {
            amount, available, ..
        } => {
            assert_eq!(available, 4);
            assert_eq!(amount, 4);
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn kernel_rejects_move_beyond_max_distance_per_tick() {
    let mut config = WorldConfig::default();
    config.physics.max_move_distance_cm_per_tick = 100;
    config.physics.max_move_speed_cm_per_s = i64::MAX;
    let mut kernel = WorldKernel::with_config(config);
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "far".to_string(),
        pos: pos(101.0, 0.0),
        profile: LocationProfile::default(),
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
            assert!(matches!(
                reason,
                RejectReason::MoveDistanceExceeded {
                    distance_cm: 101,
                    max_distance_cm: 100,
                }
            ));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn kernel_rejects_move_beyond_max_speed() {
    let mut config = WorldConfig::default();
    config.physics.time_step_s = 1;
    config.physics.max_move_distance_cm_per_tick = i64::MAX;
    config.physics.max_move_speed_cm_per_s = 100;
    let mut kernel = WorldKernel::with_config(config);
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "fast".to_string(),
        pos: pos(101.0, 0.0),
        profile: LocationProfile::default(),
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
            assert!(matches!(
                reason,
                RejectReason::MoveSpeedExceeded {
                    required_speed_cm_per_s: 101,
                    max_speed_cm_per_s: 100,
                    time_step_s: 1,
                }
            ));
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
        profile: LocationProfile::default(),
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
            assert!(matches!(
                reason,
                RejectReason::AgentAlreadyAtLocation { .. }
            ));
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
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "far".to_string(),
        pos: pos(DEFAULT_VISIBILITY_RANGE_CM as f64 + 1.0, 0.0),
        profile: LocationProfile::default(),
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
    assert!(obs
        .visible_locations
        .iter()
        .any(|loc| loc.location_id == "loc-1"));
    assert!(!obs
        .visible_locations
        .iter()
        .any(|loc| loc.location_id == "loc-2"));
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
fn movement_cost_scales_with_time_step_and_power_unit() {
    let mut config = WorldConfig::default();
    config.move_cost_per_km_electricity = 2;

    assert_eq!(config.movement_cost(CM_PER_KM), 2);

    config.physics.time_step_s = 20;
    assert_eq!(config.movement_cost(CM_PER_KM), 4);

    config.physics.power_unit_j = 2_000;
    assert_eq!(config.movement_cost(CM_PER_KM), 2);

    config.physics.power_unit_j = 500;
    assert_eq!(config.movement_cost(CM_PER_KM), 8);
}

#[test]
fn movement_cost_uses_calibrated_per_km_in_move_action() {
    let mut config = WorldConfig::default();
    config.move_cost_per_km_electricity = 2;
    config.physics.time_step_s = 20;
    config.physics.power_unit_j = 2_000;
    config.physics.max_move_distance_cm_per_tick = i64::MAX;
    config.physics.max_move_speed_cm_per_s = i64::MAX;
    config.physics.max_harvest_per_tick = 50;
    let mut kernel = WorldKernel::with_config(config);

    let mut source_profile = LocationProfile::default();
    source_profile.radiation_emission_per_tick = 100;
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
        profile: source_profile,
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "outpost".to_string(),
        pos: pos(CM_PER_KM as f64, 0.0),
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::HarvestRadiation {
        agent_id: "agent-1".to_string(),
        max_amount: 10,
    });
    let _ = kernel.step().unwrap();

    kernel.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: "loc-2".to_string(),
    });

    let event = kernel.step().unwrap();
    match event.kind {
        WorldEventKind::AgentMoved {
            electricity_cost, ..
        } => {
            assert_eq!(electricity_cost, 2);
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn kernel_transfer_requires_colocation() {
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
        pos: pos(10.0, 10.0),
        profile: LocationProfile::default(),
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
fn observe_triggers_chunk_generation_for_agent_chunk() {
    let mut config = WorldConfig::default();
    config.asteroid_fragment.base_density_per_km3 = 2.0;
    config.asteroid_fragment.voxel_size_km = 1;
    config.asteroid_fragment.cluster_noise = 0.0;
    config.asteroid_fragment.layer_scale_height_km = 0.0;
    config.asteroid_fragment.radius_min_cm = 10;
    config.asteroid_fragment.radius_max_cm = 10;

    let mut init = WorldInitConfig::default();
    init.seed = 7;
    init.agents.count = 1;

    let (mut kernel, _) = initialize_kernel(config, init).expect("init kernel");
    let before = kernel.model().locations.len();

    let _ = kernel.observe("agent-0").expect("observe");

    let after = kernel.model().locations.len();
    assert!(after >= before);
}

#[test]
fn observe_records_chunk_generated_event_with_observe_cause() {
    let mut config = WorldConfig::default();
    config.asteroid_fragment.base_density_per_km3 = 0.0;

    let mut model = WorldModel::default();
    for coord in chunk_coords(&config.space) {
        model.chunks.insert(coord, ChunkState::Unexplored);
    }

    let location_pos = pos(100_000.0, 100_000.0);
    model.locations.insert(
        "origin".to_string(),
        Location::new_with_profile(
            "origin".to_string(),
            "Origin".to_string(),
            location_pos,
            LocationProfile::default(),
        ),
    );
    model.agents.insert(
        "agent-0".to_string(),
        Agent::new_with_power("agent-0", "origin", location_pos, &config.power),
    );

    let chunk_runtime = ChunkRuntimeConfig {
        world_seed: 9,
        asteroid_fragment_enabled: true,
        asteroid_fragment_seed_offset: 1,
        min_fragment_spacing_cm: None,
    };
    let mut kernel = WorldKernel::with_model_and_chunk_runtime(config, model, chunk_runtime);

    let before = kernel.journal().len();
    let _ = kernel.observe("agent-0").expect("observe");
    assert!(kernel.journal().len() > before);
    assert!(kernel.journal().iter().any(|event| {
        matches!(
            event.kind,
            WorldEventKind::ChunkGenerated {
                cause: ChunkGenerationCause::Observe,
                ..
            }
        )
    }));
}

#[test]
fn action_chunk_generation_consumes_boundary_reservations() {
    let mut config = WorldConfig::default();
    config.move_cost_per_km_electricity = 0;
    config.space = SpaceConfig {
        width_cm: 4_000_000,
        depth_cm: 2_000_000,
        height_cm: 1_000_000,
    };
    config.asteroid_fragment.base_density_per_km3 = 0.005;
    config.asteroid_fragment.voxel_size_km = 20;
    config.asteroid_fragment.cluster_noise = 0.0;
    config.asteroid_fragment.layer_scale_height_km = 0.0;
    config.asteroid_fragment.radius_min_cm = 1_000;
    config.asteroid_fragment.radius_max_cm = 1_000;
    config.physics.max_move_distance_cm_per_tick = i64::MAX;
    config.physics.max_move_speed_cm_per_s = i64::MAX;

    let mut init = WorldInitConfig::default();
    init.seed = 1337;
    init.origin.enabled = false;
    init.agents.count = 0;
    init.asteroid_fragment.min_fragment_spacing_cm = Some(2_000_000);
    init.asteroid_fragment.bootstrap_chunks = vec![ChunkCoord { x: 0, y: 0, z: 0 }];

    let (mut kernel, _) = initialize_kernel(config, init).expect("init kernel");
    let right_coord = ChunkCoord { x: 1, y: 0, z: 0 };
    assert!(kernel
        .model()
        .chunk_boundary_reservations
        .get(&right_coord)
        .is_some_and(|entries| !entries.is_empty()));

    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-left".to_string(),
        name: "left".to_string(),
        pos: GeoPos {
            x_cm: 100_000.0,
            y_cm: 1_000_000.0,
            z_cm: 500_000.0,
        },
        profile: LocationProfile::default(),
    });
    kernel.step().expect("register left location");

    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-right".to_string(),
        name: "right".to_string(),
        pos: GeoPos {
            x_cm: 3_000_000.0,
            y_cm: 1_000_000.0,
            z_cm: 500_000.0,
        },
        profile: LocationProfile::default(),
    });
    kernel.step().expect("register right location");

    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-0".to_string(),
        location_id: "loc-left".to_string(),
    });
    kernel.step().expect("register agent");

    kernel.submit_action(Action::MoveAgent {
        agent_id: "agent-0".to_string(),
        to: "loc-right".to_string(),
    });
    let event = kernel.step().expect("move action");
    assert!(matches!(event.kind, WorldEventKind::AgentMoved { .. }));

    assert!(kernel
        .model()
        .chunks
        .get(&right_coord)
        .is_some_and(|state| matches!(state, ChunkState::Generated | ChunkState::Exhausted)));
    assert!(!kernel
        .model()
        .chunk_boundary_reservations
        .contains_key(&right_coord));
    assert!(kernel.journal().iter().any(|entry| {
        matches!(
            entry.kind,
            WorldEventKind::ChunkGenerated {
                cause: ChunkGenerationCause::Action,
                coord,
                ..
            } if coord == right_coord
        )
    }));
}

#[test]
fn step_replenishes_fragments_every_hundred_ticks_at_one_percent() {
    let mut config = WorldConfig::default();
    config.asteroid_fragment.base_density_per_km3 = 0.0;
    config.asteroid_fragment.min_fragments_per_chunk = 0;
    config.asteroid_fragment.max_fragments_per_chunk = 100;
    config.asteroid_fragment.replenish_interval_ticks = 100;
    config.asteroid_fragment.replenish_percent_ppm = 10_000;

    let target_chunk = ChunkCoord { x: 0, y: 0, z: 0 };
    let mut init = WorldInitConfig::default();
    init.seed = 123;
    init.agents.count = 0;
    init.asteroid_fragment.bootstrap_chunks = vec![target_chunk];

    let (mut kernel, _) = initialize_kernel(config.clone(), init).expect("init kernel");
    let before_count = kernel
        .model()
        .locations
        .values()
        .filter(|location| {
            location.id.starts_with("frag-")
                && chunk_coord_of(location.pos, &config.space)
                    .is_some_and(|coord| coord == target_chunk)
        })
        .count();
    assert_eq!(before_count, 0);

    for i in 0..100 {
        kernel.submit_action(Action::RegisterLocation {
            location_id: format!("tick-loc-{i}"),
            name: format!("tick-loc-{i}"),
            pos: GeoPos {
                x_cm: 1000.0 + i as f64,
                y_cm: 2000.0 + i as f64,
                z_cm: 3000.0,
            },
            profile: LocationProfile::default(),
        });
        kernel.step().expect("step");
    }

    let replenish_events: Vec<&WorldEventKind> = kernel
        .journal()
        .iter()
        .filter_map(|event| match &event.kind {
            WorldEventKind::FragmentsReplenished { .. } => Some(&event.kind),
            _ => None,
        })
        .collect();
    assert_eq!(replenish_events.len(), 1);

    let after_count = kernel
        .model()
        .locations
        .values()
        .filter(|location| {
            location.id.starts_with("frag-")
                && chunk_coord_of(location.pos, &config.space)
                    .is_some_and(|coord| coord == target_chunk)
        })
        .count();
    assert_eq!(after_count, 1);
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
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "lab".to_string(),
        pos: loc2_pos,
        profile: LocationProfile::default(),
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

#[test]
fn kernel_consume_fragment_resource_keeps_chunk_budget_in_sync() {
    let mut config = WorldConfig::default();
    config.space = SpaceConfig {
        width_cm: 200_000,
        depth_cm: 200_000,
        height_cm: 200_000,
    };
    config.asteroid_fragment.base_density_per_km3 = 5.0;
    config.asteroid_fragment.voxel_size_km = 1;
    config.asteroid_fragment.cluster_noise = 0.0;
    config.asteroid_fragment.layer_scale_height_km = 0.0;
    config.asteroid_fragment.radius_min_cm = 120;
    config.asteroid_fragment.radius_max_cm = 120;

    let mut init = WorldInitConfig::default();
    init.seed = 77;
    init.agents.count = 0;

    let (mut kernel, _) = initialize_kernel(config.clone(), init).expect("init kernel");
    let fragment = kernel
        .model()
        .locations
        .values()
        .find(|loc| loc.id.starts_with("frag-"))
        .cloned()
        .expect("fragment exists");
    let coord = chunk_coord_of(fragment.pos, &config.space).expect("fragment chunk coord");
    let element = fragment
        .fragment_budget
        .as_ref()
        .and_then(|budget| budget.remaining_by_element_g.keys().next().copied())
        .expect("fragment element");

    let before_fragment = fragment
        .fragment_budget
        .as_ref()
        .expect("fragment budget")
        .get_remaining(element);
    let before_chunk = kernel
        .model()
        .chunk_resource_budgets
        .get(&coord)
        .expect("chunk budget")
        .get_remaining(element);
    let amount = before_fragment.min(30).max(1);

    kernel
        .consume_fragment_resource(&fragment.id, element, amount)
        .expect("consume by kernel api");

    let after_fragment = kernel
        .model()
        .locations
        .get(&fragment.id)
        .and_then(|loc| loc.fragment_budget.as_ref())
        .expect("fragment budget after")
        .get_remaining(element);
    let after_chunk = kernel
        .model()
        .chunk_resource_budgets
        .get(&coord)
        .expect("chunk budget after")
        .get_remaining(element);

    assert_eq!(after_fragment, before_fragment - amount);
    assert_eq!(after_chunk, before_chunk - amount);
}

#[test]
fn mine_compound_consumes_fragment_budget_and_awards_owner_compound() {
    let mut config = WorldConfig::default();
    config.economy.mine_electricity_cost_per_kg = 2;
    config.economy.mine_compound_max_per_action_g = 2_000;
    config.economy.mine_compound_max_per_location_g = 10_000;
    config.space = SpaceConfig {
        width_cm: 200_000,
        depth_cm: 200_000,
        height_cm: 200_000,
    };
    config.asteroid_fragment.base_density_per_km3 = 5.0;
    config.asteroid_fragment.voxel_size_km = 1;
    config.asteroid_fragment.cluster_noise = 0.0;
    config.asteroid_fragment.layer_scale_height_km = 0.0;
    config.asteroid_fragment.radius_min_cm = 120;
    config.asteroid_fragment.radius_max_cm = 120;

    let mut init = WorldInitConfig::default();
    init.seed = 91;
    init.agents.count = 0;

    let (mut kernel, _) = initialize_kernel(config.clone(), init).expect("init kernel");
    let fragment = kernel
        .model()
        .locations
        .values()
        .find(|loc| loc.id.starts_with("frag-"))
        .cloned()
        .expect("fragment exists");
    let location_id = fragment.id.clone();
    let before_remaining_total = fragment
        .fragment_budget
        .as_ref()
        .expect("fragment budget")
        .remaining_by_element_g
        .values()
        .copied()
        .sum::<i64>();

    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-miner".to_string(),
        location_id: location_id.clone(),
    });
    kernel.step().expect("register miner");
    seed_owner_resource(
        &mut kernel,
        ResourceOwner::Agent {
            agent_id: "agent-miner".to_string(),
        },
        ResourceKind::Electricity,
        50,
    );

    kernel.submit_action(Action::MineCompound {
        owner: ResourceOwner::Agent {
            agent_id: "agent-miner".to_string(),
        },
        location_id: location_id.clone(),
        compound_mass_g: 2_000,
    });
    let event = kernel.step().expect("mine compound");
    match event.kind {
        WorldEventKind::CompoundMined {
            owner,
            location_id: mined_location,
            compound_mass_g,
            electricity_cost,
            extracted_elements,
        } => {
            assert_eq!(
                owner,
                ResourceOwner::Agent {
                    agent_id: "agent-miner".to_string()
                }
            );
            assert_eq!(mined_location, location_id);
            assert_eq!(compound_mass_g, 2_000);
            assert_eq!(electricity_cost, 4);
            assert_eq!(
                extracted_elements.values().copied().sum::<i64>(),
                compound_mass_g
            );
            assert!(!extracted_elements.is_empty());
        }
        other => panic!("unexpected event: {other:?}"),
    }

    let agent = kernel
        .model()
        .agents
        .get("agent-miner")
        .expect("agent exists");
    assert_eq!(agent.resources.get(ResourceKind::Compound), 2_000);
    assert_eq!(agent.resources.get(ResourceKind::Electricity), 46);

    let after_fragment = kernel
        .model()
        .locations
        .get(&location_id)
        .and_then(|location| location.fragment_budget.as_ref())
        .expect("fragment budget after");
    let after_remaining_total = after_fragment
        .remaining_by_element_g
        .values()
        .copied()
        .sum::<i64>();
    assert_eq!(after_remaining_total, before_remaining_total - 2_000);
}

#[test]
fn mine_compound_enforces_location_cap() {
    let mut config = WorldConfig::default();
    config.economy.mine_electricity_cost_per_kg = 1;
    config.economy.mine_compound_max_per_action_g = 1_000;
    config.economy.mine_compound_max_per_location_g = 1_500;
    config.space = SpaceConfig {
        width_cm: 200_000,
        depth_cm: 200_000,
        height_cm: 200_000,
    };
    config.asteroid_fragment.base_density_per_km3 = 5.0;
    config.asteroid_fragment.voxel_size_km = 1;
    config.asteroid_fragment.cluster_noise = 0.0;
    config.asteroid_fragment.layer_scale_height_km = 0.0;
    config.asteroid_fragment.radius_min_cm = 120;
    config.asteroid_fragment.radius_max_cm = 120;

    let mut init = WorldInitConfig::default();
    init.seed = 92;
    init.agents.count = 0;

    let (mut kernel, _) = initialize_kernel(config.clone(), init).expect("init kernel");
    let location_id = kernel
        .model()
        .locations
        .values()
        .find(|loc| loc.id.starts_with("frag-"))
        .map(|loc| loc.id.clone())
        .expect("fragment exists");

    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-miner".to_string(),
        location_id: location_id.clone(),
    });
    kernel.step().expect("register miner");
    seed_owner_resource(
        &mut kernel,
        ResourceOwner::Agent {
            agent_id: "agent-miner".to_string(),
        },
        ResourceKind::Electricity,
        20,
    );

    kernel.submit_action(Action::MineCompound {
        owner: ResourceOwner::Agent {
            agent_id: "agent-miner".to_string(),
        },
        location_id: location_id.clone(),
        compound_mass_g: 1_000,
    });
    let first = kernel.step().expect("first mining");
    assert!(matches!(first.kind, WorldEventKind::CompoundMined { .. }));

    kernel.submit_action(Action::MineCompound {
        owner: ResourceOwner::Agent {
            agent_id: "agent-miner".to_string(),
        },
        location_id: location_id.clone(),
        compound_mass_g: 600,
    });
    let second = kernel.step().expect("second mining");
    match second.kind {
        WorldEventKind::ActionRejected { reason } => {
            assert!(matches!(
                reason,
                RejectReason::InsufficientResource {
                    owner: ResourceOwner::Location { .. },
                    kind: ResourceKind::Compound,
                    requested: 600,
                    available: 500,
                }
            ));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn refine_compound_consumes_electricity_and_outputs_hardware() {
    let mut config = WorldConfig::default();
    config.economy.refine_electricity_cost_per_kg = 3;
    config.economy.refine_hardware_yield_ppm = 2_000;

    let mut kernel = WorldKernel::with_config(config);
    let mut profile = LocationProfile::default();
    profile.radiation_emission_per_tick = 120;
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-refine".to_string(),
        name: "refine".to_string(),
        pos: pos(0.0, 0.0),
        profile,
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-refiner".to_string(),
        location_id: "loc-refine".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::HarvestRadiation {
        agent_id: "agent-refiner".to_string(),
        max_amount: 50,
    });
    kernel.step().expect("seed electricity");
    seed_owner_resource(
        &mut kernel,
        ResourceOwner::Agent {
            agent_id: "agent-refiner".to_string(),
        },
        ResourceKind::Compound,
        2_500,
    );

    kernel.submit_action(Action::RefineCompound {
        owner: ResourceOwner::Agent {
            agent_id: "agent-refiner".to_string(),
        },
        compound_mass_g: 2_500,
    });

    let event = kernel.step().expect("refine action");
    match event.kind {
        WorldEventKind::CompoundRefined {
            owner,
            compound_mass_g,
            electricity_cost,
            hardware_output,
        } => {
            assert_eq!(
                owner,
                ResourceOwner::Agent {
                    agent_id: "agent-refiner".to_string()
                }
            );
            assert_eq!(compound_mass_g, 2_500);
            assert_eq!(electricity_cost, 9);
            assert_eq!(hardware_output, 5);
        }
        other => panic!("unexpected event: {other:?}"),
    }

    let agent = kernel
        .model()
        .agents
        .get("agent-refiner")
        .expect("agent exists");
    assert_eq!(agent.resources.get(ResourceKind::Electricity), 41);
    assert_eq!(agent.resources.get(ResourceKind::Hardware), 5);
}

#[test]
fn refine_compound_rejects_when_compound_insufficient() {
    let mut config = WorldConfig::default();
    config.economy.refine_electricity_cost_per_kg = 4;
    config.economy.refine_hardware_yield_ppm = 1_000;

    let mut kernel = WorldKernel::with_config(config);
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-refine".to_string(),
        name: "refine".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-refiner".to_string(),
        location_id: "loc-refine".to_string(),
    });
    kernel.step_until_empty();

    seed_owner_resource(
        &mut kernel,
        ResourceOwner::Agent {
            agent_id: "agent-refiner".to_string(),
        },
        ResourceKind::Electricity,
        100,
    );

    kernel.submit_action(Action::RefineCompound {
        owner: ResourceOwner::Agent {
            agent_id: "agent-refiner".to_string(),
        },
        compound_mass_g: 1_500,
    });

    let event = kernel.step().expect("refine rejected");
    match event.kind {
        WorldEventKind::ActionRejected { reason } => {
            assert!(matches!(
                reason,
                RejectReason::InsufficientResource {
                    kind: ResourceKind::Compound,
                    requested: 1_500,
                    available: 0,
                    ..
                }
            ));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn refine_compound_rejects_when_electricity_insufficient() {
    let mut config = WorldConfig::default();
    config.economy.refine_electricity_cost_per_kg = 4;
    config.economy.refine_hardware_yield_ppm = 1_000;

    let mut kernel = WorldKernel::with_config(config);
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-refine".to_string(),
        name: "refine".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-refiner".to_string(),
        location_id: "loc-refine".to_string(),
    });
    kernel.step_until_empty();
    seed_owner_resource(
        &mut kernel,
        ResourceOwner::Agent {
            agent_id: "agent-refiner".to_string(),
        },
        ResourceKind::Compound,
        1_500,
    );

    kernel.submit_action(Action::RefineCompound {
        owner: ResourceOwner::Agent {
            agent_id: "agent-refiner".to_string(),
        },
        compound_mass_g: 1_500,
    });

    let event = kernel.step().expect("refine rejected");
    match event.kind {
        WorldEventKind::ActionRejected { reason } => {
            assert!(matches!(
                reason,
                RejectReason::InsufficientResource {
                    kind: ResourceKind::Electricity,
                    requested: 8,
                    available: 0,
                    ..
                }
            ));
        }
        other => panic!("unexpected event: {other:?}"),
    }
}

#[test]
fn build_factory_consumes_resources_and_persists_factory_state() {
    let mut config = WorldConfig::default();
    config.economy.factory_build_electricity_cost = 7;
    config.economy.factory_build_hardware_cost = 3;
    config.economy.refine_electricity_cost_per_kg = 1;
    config.economy.refine_hardware_yield_ppm = 1_000;

    let mut kernel = WorldKernel::with_config(config);
    let mut profile = LocationProfile::default();
    profile.radiation_emission_per_tick = 120;
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-factory".to_string(),
        name: "factory-site".to_string(),
        pos: pos(0.0, 0.0),
        profile,
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-builder".to_string(),
        location_id: "loc-factory".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::HarvestRadiation {
        agent_id: "agent-builder".to_string(),
        max_amount: 30,
    });
    kernel.step().expect("harvest for factory build");
    seed_owner_resource(
        &mut kernel,
        ResourceOwner::Agent {
            agent_id: "agent-builder".to_string(),
        },
        ResourceKind::Compound,
        3_000,
    );

    kernel.submit_action(Action::RefineCompound {
        owner: ResourceOwner::Agent {
            agent_id: "agent-builder".to_string(),
        },
        compound_mass_g: 3_000,
    });
    kernel.step().expect("refine hardware for factory build");

    kernel.submit_action(Action::BuildFactory {
        owner: ResourceOwner::Agent {
            agent_id: "agent-builder".to_string(),
        },
        location_id: "loc-factory".to_string(),
        factory_id: "factory.alpha".to_string(),
        factory_kind: "factory.assembler.mk1".to_string(),
    });

    let event = kernel.step().expect("build factory");
    match event.kind {
        WorldEventKind::FactoryBuilt {
            owner,
            location_id,
            factory_id,
            factory_kind,
            electricity_cost,
            hardware_cost,
        } => {
            assert_eq!(
                owner,
                ResourceOwner::Agent {
                    agent_id: "agent-builder".to_string()
                }
            );
            assert_eq!(location_id, "loc-factory");
            assert_eq!(factory_id, "factory.alpha");
            assert_eq!(factory_kind, "factory.assembler.mk1");
            assert_eq!(electricity_cost, 7);
            assert_eq!(hardware_cost, 3);
        }
        other => panic!("unexpected event: {other:?}"),
    }

    let agent = kernel
        .model()
        .agents
        .get("agent-builder")
        .expect("agent exists");
    assert_eq!(agent.resources.get(ResourceKind::Electricity), 20);
    assert_eq!(agent.resources.get(ResourceKind::Hardware), 0);
    assert!(kernel.model().factories.contains_key("factory.alpha"));
}

#[test]
fn schedule_recipe_consumes_inputs_and_outputs_data() {
    let mut config = WorldConfig::default();
    config.economy.factory_build_electricity_cost = 0;
    config.economy.factory_build_hardware_cost = 0;
    config.economy.refine_electricity_cost_per_kg = 1;
    config.economy.refine_hardware_yield_ppm = 1_000;
    config.economy.recipe_electricity_cost_per_batch = 5;
    config.economy.recipe_hardware_cost_per_batch = 2;
    config.economy.recipe_data_output_per_batch = 3;
    config.physics.max_harvest_per_tick = 100;

    let mut kernel = WorldKernel::with_config(config);
    let mut profile = LocationProfile::default();
    profile.radiation_emission_per_tick = 200;
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-factory".to_string(),
        name: "factory-site".to_string(),
        pos: pos(0.0, 0.0),
        profile,
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-builder".to_string(),
        location_id: "loc-factory".to_string(),
    });
    kernel.step_until_empty();

    kernel.submit_action(Action::HarvestRadiation {
        agent_id: "agent-builder".to_string(),
        max_amount: 80,
    });
    kernel.step().expect("harvest for recipe");
    seed_owner_resource(
        &mut kernel,
        ResourceOwner::Agent {
            agent_id: "agent-builder".to_string(),
        },
        ResourceKind::Compound,
        16_000,
    );

    kernel.submit_action(Action::RefineCompound {
        owner: ResourceOwner::Agent {
            agent_id: "agent-builder".to_string(),
        },
        compound_mass_g: 16_000,
    });
    kernel.step().expect("refine hardware for recipe");

    kernel.submit_action(Action::BuildFactory {
        owner: ResourceOwner::Agent {
            agent_id: "agent-builder".to_string(),
        },
        location_id: "loc-factory".to_string(),
        factory_id: "factory.alpha".to_string(),
        factory_kind: "factory.assembler.mk1".to_string(),
    });
    kernel.step().expect("build factory");

    kernel.submit_action(Action::ScheduleRecipe {
        owner: ResourceOwner::Agent {
            agent_id: "agent-builder".to_string(),
        },
        factory_id: "factory.alpha".to_string(),
        recipe_id: "recipe.assembler.logistics_drone".to_string(),
        batches: 2,
    });
    let event = kernel.step().expect("schedule recipe");
    match event.kind {
        WorldEventKind::RecipeScheduled {
            owner,
            factory_id,
            recipe_id,
            batches,
            electricity_cost,
            hardware_cost,
            data_output,
            finished_product_id,
            finished_product_units,
        } => {
            assert_eq!(
                owner,
                ResourceOwner::Agent {
                    agent_id: "agent-builder".to_string()
                }
            );
            assert_eq!(factory_id, "factory.alpha");
            assert_eq!(recipe_id, "recipe.assembler.logistics_drone");
            assert_eq!(batches, 2);
            assert_eq!(electricity_cost, 40);
            assert_eq!(hardware_cost, 16);
            assert_eq!(data_output, 24);
            assert_eq!(finished_product_id, "logistics_drone");
            assert_eq!(finished_product_units, 2);
        }
        other => panic!("unexpected event: {other:?}"),
    }

    let agent = kernel
        .model()
        .agents
        .get("agent-builder")
        .expect("agent exists");
    assert_eq!(agent.resources.get(ResourceKind::Electricity), 24);
    assert_eq!(agent.resources.get(ResourceKind::Hardware), 0);
    assert_eq!(agent.resources.get(ResourceKind::Data), 24);
}

fn collect_basic_action_sequence(kernel: &mut WorldKernel) -> Vec<WorldEventKind> {
    let mut kinds = Vec::new();

    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-seq".to_string(),
        name: "seq".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
    });
    kinds.push(kernel.step().expect("register location").kind);

    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-seq".to_string(),
        location_id: "loc-seq".to_string(),
    });
    kinds.push(kernel.step().expect("register agent").kind);

    kernel.submit_action(Action::RegisterAgent {
        agent_id: "agent-seq".to_string(),
        location_id: "loc-seq".to_string(),
    });
    kinds.push(kernel.step().expect("reject duplicate agent").kind);

    kinds
}

#[test]
fn kernel_rule_hooks_default_path_keeps_action_behavior() {
    let mut baseline = WorldKernel::new();
    let baseline_kinds = collect_basic_action_sequence(&mut baseline);

    let mut with_noop_hooks = WorldKernel::new();
    with_noop_hooks
        .add_pre_action_rule_hook(|action_id, _, _| KernelRuleDecision::allow(action_id));
    with_noop_hooks.add_post_action_rule_hook(|_, _, _| {});
    let hook_kinds = collect_basic_action_sequence(&mut with_noop_hooks);

    assert_eq!(baseline_kinds, hook_kinds);
}

#[test]
fn kernel_rule_hooks_run_in_registration_order() {
    let mut kernel = WorldKernel::new();
    let trace = Arc::new(Mutex::new(Vec::new()));

    let trace_pre_1 = Arc::clone(&trace);
    kernel.add_pre_action_rule_hook(move |action_id, _, _| {
        trace_pre_1.lock().expect("lock trace").push("pre-1");
        KernelRuleDecision::allow(action_id)
    });

    let trace_pre_2 = Arc::clone(&trace);
    kernel.add_pre_action_rule_hook(move |action_id, _, _| {
        trace_pre_2.lock().expect("lock trace").push("pre-2");
        KernelRuleDecision::allow(action_id)
    });

    let trace_post_1 = Arc::clone(&trace);
    kernel.add_post_action_rule_hook(move |_, _, _| {
        trace_post_1.lock().expect("lock trace").push("post-1");
    });

    let trace_post_2 = Arc::clone(&trace);
    kernel.add_post_action_rule_hook(move |_, _, _| {
        trace_post_2.lock().expect("lock trace").push("post-2");
    });

    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-hook-order".to_string(),
        name: "hook-order".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
    });
    kernel.step().expect("step with hooks");

    let trace = trace.lock().expect("lock trace");
    assert_eq!(*trace, vec!["pre-1", "pre-2", "post-1", "post-2"]);
}

#[test]
fn kernel_post_action_hook_receives_emitted_event() {
    let mut kernel = WorldKernel::new();
    let captured = Arc::new(Mutex::new(None::<(ActionId, Action, WorldEvent)>));
    let captured_hook = Arc::clone(&captured);

    kernel.add_post_action_rule_hook(move |action_id, action, event| {
        *captured_hook.lock().expect("lock captured") =
            Some((action_id, action.clone(), event.clone()));
    });

    let action = Action::RegisterLocation {
        location_id: "loc-hook-post".to_string(),
        name: "hook-post".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
    };
    let submitted_action_id = kernel.submit_action(action.clone());
    let emitted_event = kernel.step().expect("step with post hook");

    let captured = captured.lock().expect("lock captured");
    let (hook_action_id, hook_action, hook_event) = captured.clone().expect("captured event");
    assert_eq!(hook_action_id, submitted_action_id);
    assert_eq!(hook_action, action);
    assert_eq!(hook_event, emitted_event);
}
