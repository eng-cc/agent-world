use super::*;

#[test]
fn kernel_snapshot_roundtrip() {
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

    let snapshot = kernel.snapshot();
    let journal = kernel.journal_snapshot();
    let restored = WorldKernel::from_snapshot(snapshot, journal).unwrap();
    assert_eq!(restored.time(), kernel.time());
    assert_eq!(restored.model(), kernel.model());
}

#[test]
fn kernel_persist_and_restore() {
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
