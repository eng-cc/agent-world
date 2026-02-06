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

    let before = kernel
        .model()
        .locations
        .values()
        .find(|loc| loc.id.starts_with("frag-"))
        .and_then(|loc| loc.fragment_profile.clone())
        .expect("profile before");
    let after = restored
        .model()
        .locations
        .values()
        .find(|loc| loc.id.starts_with("frag-"))
        .and_then(|loc| loc.fragment_profile.clone())
        .expect("profile after");

    assert_eq!(after, before);
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
