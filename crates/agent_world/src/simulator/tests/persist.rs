use super::*;

#[test]
fn kernel_snapshot_roundtrip() {
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

    let snapshot = kernel.snapshot();
    let journal = kernel.journal_snapshot();
    let restored = WorldKernel::from_snapshot(snapshot, journal).unwrap();
    assert_eq!(restored.time(), kernel.time());
    assert_eq!(restored.model(), kernel.model());
}

#[test]
fn kernel_snapshot_roundtrip_keeps_fragment_profile() {
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
    init.seed = 31;
    init.agents.count = 0;

    let (kernel, _) = initialize_kernel(config, init).expect("kernel init");
    let snapshot = kernel.snapshot();
    let journal = kernel.journal_snapshot();
    let restored = WorldKernel::from_snapshot(snapshot, journal).expect("restore from snapshot");

    let fragment_before = kernel
        .model()
        .locations
        .values()
        .find(|loc| loc.id.starts_with("frag-"))
        .expect("fragment before");
    let profile_before = fragment_before
        .fragment_profile
        .clone()
        .expect("profile before");
    let budget_before = fragment_before
        .fragment_budget
        .clone()
        .expect("budget before");
    let fragment_after = restored
        .model()
        .locations
        .values()
        .find(|loc| loc.id.starts_with("frag-"))
        .expect("fragment after");
    let profile_after = fragment_after
        .fragment_profile
        .clone()
        .expect("profile after");
    let budget_after = fragment_after
        .fragment_budget
        .clone()
        .expect("budget after");

    assert_eq!(profile_after, profile_before);
    assert_eq!(budget_after, budget_before);
    assert_eq!(
        restored.model().chunk_resource_budgets,
        kernel.model().chunk_resource_budgets
    );
}

#[test]
fn kernel_persist_and_restore() {
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

    let tmp_dir = std::env::temp_dir().join("agent-world-kernel-test");
    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir).unwrap();
    }
    kernel.save_to_dir(&tmp_dir).unwrap();

    let loaded = WorldKernel::load_from_dir(&tmp_dir).unwrap();
    assert_eq!(loaded.time(), kernel.time());
    assert_eq!(loaded.model(), kernel.model());

    fs::remove_dir_all(&tmp_dir).unwrap();
}

#[test]
fn restore_rejects_mismatched_journal_len() {
    let mut kernel = WorldKernel::new();
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
    });
    kernel.step_until_empty();

    let mut snapshot = kernel.snapshot();
    let journal = kernel.journal_snapshot();
    snapshot.journal_len = journal.events.len() + 1;

    let err = WorldKernel::from_snapshot(snapshot, journal).unwrap_err();
    assert!(matches!(err, PersistError::SnapshotMismatch { .. }));
}

#[test]
fn snapshot_version_validation_rejects_unknown() {
    let kernel = WorldKernel::new();
    let mut snapshot = kernel.snapshot();
    snapshot.version = SNAPSHOT_VERSION.saturating_add(1);
    let err = snapshot.validate_version().unwrap_err();
    assert!(matches!(
        err,
        PersistError::UnsupportedVersion {
            kind,
            version,
            expected
        } if kind == "snapshot" && version == snapshot.version && expected == SNAPSHOT_VERSION
    ));
}

#[test]
fn journal_version_validation_rejects_unknown() {
    let mut journal = WorldJournal::default();
    journal.version = JOURNAL_VERSION.saturating_add(1);
    let err = journal.validate_version().unwrap_err();
    assert!(matches!(
        err,
        PersistError::UnsupportedVersion {
            kind,
            version,
            expected
        } if kind == "journal" && version == journal.version && expected == JOURNAL_VERSION
    ));
}

#[test]
fn snapshot_version_validation_accepts_legacy_and_defaults_chunk_schema() {
    let kernel = WorldKernel::new();
    let snapshot = kernel.snapshot();

    let mut value: serde_json::Value =
        serde_json::from_str(&snapshot.to_json().expect("snapshot to json"))
            .expect("parse snapshot json");
    value["version"] = serde_json::Value::from(SNAPSHOT_VERSION.saturating_sub(1));
    if let serde_json::Value::Object(map) = &mut value {
        map.remove("chunk_generation_schema_version");
    }

    let migrated = WorldSnapshot::from_json(
        &serde_json::to_string(&value).expect("serialize migrated snapshot"),
    )
    .expect("load legacy snapshot");

    assert_eq!(migrated.version, SNAPSHOT_VERSION.saturating_sub(1));
    assert_eq!(
        migrated.chunk_generation_schema_version,
        CHUNK_GENERATION_SCHEMA_VERSION
    );
}

#[test]
fn journal_version_validation_accepts_legacy() {
    let mut journal = WorldJournal::default();
    journal.version = JOURNAL_VERSION.saturating_sub(1);
    assert!(journal.validate_version().is_ok());
}

#[test]
fn initialize_kernel_records_chunk_generated_init_events() {
    let mut config = WorldConfig::default();
    config.asteroid_fragment.base_density_per_km3 = 0.0;

    let mut init = WorldInitConfig::default();
    init.seed = 41;
    init.agents.count = 1;

    let (kernel, _) = initialize_kernel(config, init).expect("kernel init");
    let init_chunk_events = kernel
        .journal()
        .iter()
        .filter(|event| {
            matches!(
                event.kind,
                WorldEventKind::ChunkGenerated {
                    cause: ChunkGenerationCause::Init,
                    ..
                }
            )
        })
        .count();

    assert!(init_chunk_events > 0);
}

#[test]
fn replay_from_snapshot_rebuilds_and_validates_chunk_generated_events() {
    let mut config = WorldConfig::default();
    config.move_cost_per_km_electricity = 0;
    config.asteroid_fragment.base_density_per_km3 = 0.0;

    let mut init = WorldInitConfig::default();
    init.seed = 97;
    init.agents.count = 1;

    let (mut kernel, _) = initialize_kernel(config.clone(), init).expect("init kernel");
    let snapshot = kernel.snapshot();

    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-far".to_string(),
        name: "far".to_string(),
        pos: GeoPos {
            x_cm: 100_000.0,
            y_cm: 100_000.0,
            z_cm: 0.0,
        },
        profile: LocationProfile::default(),
    });
    kernel.step().expect("register far location");

    kernel.submit_action(Action::MoveAgent {
        agent_id: "agent-0".to_string(),
        to: "loc-far".to_string(),
    });
    kernel.step().expect("move to far location");

    let journal = kernel.journal_snapshot();
    let chunk_event_index = journal
        .events
        .iter()
        .enumerate()
        .skip(snapshot.journal_len)
        .find_map(|(idx, event)| match event.kind {
            WorldEventKind::ChunkGenerated {
                cause: ChunkGenerationCause::Action,
                ..
            } => Some(idx),
            _ => None,
        })
        .expect("action chunk generation event exists");

    let replayed = WorldKernel::replay_from_snapshot(snapshot.clone(), journal.clone())
        .expect("replay with chunk-generated event");
    assert_eq!(replayed.model(), kernel.model());

    let mut tampered = journal;
    if let WorldEventKind::ChunkGenerated { block_count, .. } =
        &mut tampered.events[chunk_event_index].kind
    {
        *block_count = block_count.saturating_add(1);
    }

    let err = WorldKernel::replay_from_snapshot(snapshot, tampered).unwrap_err();
    assert!(matches!(err, PersistError::ReplayConflict { .. }));
}

#[test]
fn kernel_replay_from_snapshot() {
    let config = WorldConfig {
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

    let snapshot = kernel.snapshot();
    let mut journal = kernel.journal_snapshot();

    kernel.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: "loc-2".to_string(),
    });
    let event = kernel.step().unwrap();
    journal.events.push(event);

    let replayed = WorldKernel::replay_from_snapshot(snapshot, journal).unwrap();
    let agent = replayed.model().agents.get("agent-1").unwrap();
    assert_eq!(agent.location_id, "loc-2");
}

#[test]
fn replay_from_snapshot_applies_compound_refined_event() {
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

    let snapshot = kernel.snapshot();

    kernel.submit_action(Action::HarvestRadiation {
        agent_id: "agent-refiner".to_string(),
        max_amount: 50,
    });
    kernel.step().expect("seed electricity");

    kernel.submit_action(Action::RefineCompound {
        owner: ResourceOwner::Agent {
            agent_id: "agent-refiner".to_string(),
        },
        compound_mass_g: 2_500,
    });
    kernel.step().expect("refine");

    let journal = kernel.journal_snapshot();
    let replayed = WorldKernel::replay_from_snapshot(snapshot, journal).expect("replay");

    let agent = replayed
        .model()
        .agents
        .get("agent-refiner")
        .expect("agent exists");
    assert_eq!(agent.resources.get(ResourceKind::Electricity), 41);
    assert_eq!(agent.resources.get(ResourceKind::Hardware), 5);
}

#[test]
fn replay_with_budget_caps_keeps_chunk_generated_consistent() {
    let mut config = WorldConfig::default();
    config.move_cost_per_km_electricity = 0;
    config.asteroid_fragment.base_density_per_km3 = 20.0;
    config.asteroid_fragment.voxel_size_km = 10;
    config.asteroid_fragment.cluster_noise = 0.0;
    config.asteroid_fragment.layer_scale_height_km = 0.0;
    config.asteroid_fragment.min_fragment_spacing_cm = 0;
    config.asteroid_fragment.radius_min_cm = 2_500;
    config.asteroid_fragment.radius_max_cm = 2_500;
    config.asteroid_fragment.max_fragments_per_chunk = 2;
    config.asteroid_fragment.max_blocks_per_fragment = 2;
    config.asteroid_fragment.max_blocks_per_chunk = 3;

    let mut init = WorldInitConfig::default();
    init.seed = 197;
    init.agents.count = 1;

    let (mut kernel, _) = initialize_kernel(config.clone(), init).expect("init kernel");
    let snapshot = kernel.snapshot();

    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-budget".to_string(),
        name: "budget".to_string(),
        pos: GeoPos {
            x_cm: 2_500_000.0,
            y_cm: 2_500_000.0,
            z_cm: 0.0,
        },
        profile: LocationProfile::default(),
    });
    kernel.step().expect("register location");

    kernel.submit_action(Action::MoveAgent {
        agent_id: "agent-0".to_string(),
        to: "loc-budget".to_string(),
    });
    kernel.step().expect("move agent");

    let journal = kernel.journal_snapshot();
    let capped_action_event = journal
        .events
        .iter()
        .find_map(|event| match event.kind {
            WorldEventKind::ChunkGenerated {
                cause: ChunkGenerationCause::Action,
                fragment_count,
                block_count,
                ..
            } => Some((fragment_count, block_count)),
            _ => None,
        })
        .expect("action chunk generated event");

    assert!(capped_action_event.0 <= 2);
    assert!(capped_action_event.1 <= 3);

    let replayed = WorldKernel::replay_from_snapshot(snapshot, journal).expect("replay");
    assert_eq!(replayed.model(), kernel.model());
}
