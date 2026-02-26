use super::super::*;
use super::pos;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("duration")
        .as_nanos();
    std::env::temp_dir().join(format!("agent-world-{prefix}-{unique}"))
}

fn install_test_module(world: &mut World, module_id: &str, wasm_bytes: &[u8]) -> String {
    world.set_policy(PolicySet::allow_all());
    let wasm_hash = util::sha256_hex(wasm_bytes);
    world
        .register_module_artifact(wasm_hash.clone(), wasm_bytes)
        .expect("register module artifact");

    let module_manifest = ModuleManifest {
        module_id: module_id.to_string(),
        name: "Persistence Module".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        role: ModuleRole::Rule,
        wasm_hash: wasm_hash.clone(),
        interface_version: "wasm-1".to_string(),
        abi_contract: ModuleAbiContract::default(),
        exports: vec!["reduce".to_string()],
        subscriptions: Vec::new(),
        required_caps: Vec::new(),
        artifact_identity: None,
        limits: ModuleLimits::unbounded(),
    };
    let changes = ModuleChangeSet {
        register: vec![module_manifest.clone()],
        activate: vec![ModuleActivation {
            module_id: module_manifest.module_id.clone(),
            version: module_manifest.version.clone(),
        }],
        ..ModuleChangeSet::default()
    };
    let manifest = Manifest {
        version: 2,
        content: json!({
            "module_changes": changes,
        }),
    };

    let proposal_id = world
        .propose_manifest_update(manifest, "alice")
        .expect("propose module manifest");
    world.shadow_proposal(proposal_id).expect("shadow proposal");
    world
        .approve_proposal(proposal_id, "bob", ProposalDecision::Approve)
        .expect("approve proposal");
    world.apply_proposal(proposal_id).expect("apply proposal");
    wasm_hash
}

#[test]
fn persist_and_restore_world() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().unwrap();

    let dir = temp_dir("persist-restore");

    world.save_to_dir(&dir).unwrap();

    let restored = World::load_from_dir(&dir).unwrap();
    assert_eq!(restored.state(), world.state());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn persist_and_restore_world_defaults_to_module_store_roundtrip() {
    let mut world = World::new();
    let wasm_hash = install_test_module(&mut world, "m.persistence.default", b"persist-default");
    let module_record_key = ModuleRegistry::record_key("m.persistence.default", "0.1.0");
    let dir = temp_dir("persist-module-store-default");

    world
        .save_to_dir(&dir)
        .expect("save with default module store");
    assert!(
        dir.join("module_registry.json").exists(),
        "default save should persist module registry"
    );
    assert!(
        dir.join("modules")
            .join(format!("{wasm_hash}.wasm"))
            .exists(),
        "default save should persist module artifact bytes"
    );

    let mut restored = World::load_from_dir(&dir).expect("load with default module store");
    assert!(restored
        .module_registry()
        .records
        .contains_key(&module_record_key));
    let artifact = restored
        .load_module(&wasm_hash)
        .expect("module bytes hydrated from default load");
    assert_eq!(artifact.bytes, b"persist-default".to_vec());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn load_from_dir_rejects_tampered_module_artifact_bytes() {
    let mut world = World::new();
    let wasm_hash = install_test_module(&mut world, "m.persistence.tamper", b"persist-tamper");
    let dir = temp_dir("persist-module-store-tamper");

    world.save_to_dir(&dir).expect("save world");
    fs::write(
        dir.join("modules").join(format!("{wasm_hash}.wasm")),
        b"tampered-bytes",
    )
    .expect("tamper module artifact");

    let err = World::load_from_dir(&dir).expect_err("tampered module artifact should be rejected");
    assert!(matches!(
        err,
        WorldError::ModuleStoreManifestMismatch { .. }
    ));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn load_from_dir_without_module_store_keeps_legacy_compatibility() {
    let mut world = World::new();
    let wasm_hash = install_test_module(&mut world, "m.persistence.legacy", b"persist-legacy");
    let module_record_key = ModuleRegistry::record_key("m.persistence.legacy", "0.1.0");
    let dir = temp_dir("persist-module-store-legacy");

    world.save_to_dir(&dir).expect("save world");
    fs::remove_file(dir.join("module_registry.json")).expect("remove module registry");
    fs::remove_dir_all(dir.join("modules")).expect("remove module store modules dir");

    let mut restored = World::load_from_dir(&dir).expect("legacy load without module store");
    assert!(restored
        .module_registry()
        .records
        .contains_key(&module_record_key));
    let err = restored
        .load_module(&wasm_hash)
        .expect_err("legacy world should load without hydrated module bytes");
    assert!(matches!(err, WorldError::ModuleChangeInvalid { .. }));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn persist_writes_distfs_sidecar_and_restores_without_json_files() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().unwrap();

    let dir = temp_dir("persist-distfs-sidecar");
    world.save_to_dir(&dir).expect("save world with sidecar");

    assert!(dir.join("snapshot.manifest.json").exists());
    assert!(dir.join("journal.segments.json").exists());
    assert!(dir.join(".distfs-state").exists());

    fs::remove_file(dir.join("snapshot.json")).expect("remove legacy snapshot");
    fs::remove_file(dir.join("journal.json")).expect("remove legacy journal");

    let restored = World::load_from_dir(&dir).expect("restore from distfs sidecar");
    assert_eq!(restored.state(), world.state());
    let audit_value: serde_json::Value = serde_json::from_slice(
        &fs::read(dir.join("distfs.recovery.audit.json")).expect("read distfs audit"),
    )
    .expect("decode distfs audit");
    assert_eq!(
        audit_value.get("status").and_then(|value| value.as_str()),
        Some("distfs_restored")
    );
    assert!(audit_value.get("reason").is_none());
    assert!(audit_value
        .get("timestamp_ms")
        .and_then(|value| value.as_i64())
        .is_some());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn load_from_dir_falls_back_to_json_when_distfs_sidecar_is_invalid() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().unwrap();

    let dir = temp_dir("persist-distfs-fallback");
    world.save_to_dir(&dir).expect("save world");
    fs::write(
        dir.join("snapshot.manifest.json"),
        b"{\"manifest\":\"broken\"}",
    )
    .expect("tamper sidecar");

    let restored = World::load_from_dir(&dir).expect("fallback to legacy json");
    assert_eq!(restored.state(), world.state());
    let audit_value: serde_json::Value = serde_json::from_slice(
        &fs::read(dir.join("distfs.recovery.audit.json")).expect("read distfs fallback audit"),
    )
    .expect("decode distfs fallback audit");
    assert_eq!(
        audit_value.get("status").and_then(|value| value.as_str()),
        Some("fallback_json")
    );
    assert!(audit_value
        .get("reason")
        .and_then(|value| value.as_str())
        .map(|reason| reason.contains("distfs_restore_failed"))
        .unwrap_or(false));
    assert!(audit_value
        .get("timestamp_ms")
        .and_then(|value| value.as_i64())
        .is_some());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn snapshot_json_without_era_fields_keeps_backward_compatibility() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-legacy".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().expect("step");

    let snapshot = world.snapshot();
    let mut value = serde_json::to_value(&snapshot).expect("encode snapshot");
    let object = value.as_object_mut().expect("snapshot object");
    object.remove("event_id_era");
    object.remove("action_id_era");
    object.remove("intent_id_era");
    object.remove("proposal_id_era");

    let legacy_json = serde_json::to_string(&value).expect("legacy json");
    let restored = Snapshot::from_json(&legacy_json).expect("decode legacy snapshot");
    assert_eq!(restored.event_id_era, 0);
    assert_eq!(restored.action_id_era, 0);
    assert_eq!(restored.intent_id_era, 0);
    assert_eq!(restored.proposal_id_era, 0);
}

#[test]
fn rollback_to_snapshot_resets_state() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().unwrap();
    let snapshot = world.snapshot();

    world.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: pos(9.0, 9.0),
    });
    world.step().unwrap();
    assert_eq!(
        world.state().agents.get("agent-1").unwrap().state.pos,
        pos(9.0, 9.0)
    );

    let journal = world.journal().clone();
    world
        .rollback_to_snapshot(snapshot.clone(), journal, "test-rollback")
        .unwrap();

    assert_eq!(world.state(), &snapshot.state);
    let last = world.journal().events.last().unwrap();
    assert!(matches!(last.body, WorldEventBody::RollbackApplied(_)));
}

#[test]
fn snapshot_retention_policy_prunes_old_entries() {
    let mut world = World::new();
    world.set_snapshot_retention(SnapshotRetentionPolicy { max_snapshots: 1 });

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().unwrap();
    let snap1 = world.create_snapshot().unwrap();

    world.submit_action(Action::MoveAgent {
        agent_id: "agent-1".to_string(),
        to: pos(3.0, 3.0),
    });
    world.step().unwrap();
    let snap2 = world.create_snapshot().unwrap();

    assert_eq!(world.snapshot_catalog().records.len(), 1);
    let last_record = &world.snapshot_catalog().records[0];
    assert_eq!(last_record.snapshot_hash, util::hash_json(&snap2).unwrap());
    assert_ne!(last_record.snapshot_hash, util::hash_json(&snap1).unwrap());
}

#[test]
fn snapshot_file_pruning_removes_old_files() {
    let mut world = World::new();
    world.set_snapshot_retention(SnapshotRetentionPolicy { max_snapshots: 1 });

    let dir = std::env::temp_dir().join(format!(
        "agent-world-snapshots-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    world.save_snapshot_to_dir(&dir).unwrap();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().unwrap();
    world.save_snapshot_to_dir(&dir).unwrap();

    let snapshots_dir = dir.join("snapshots");
    let file_count = fs::read_dir(&snapshots_dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .count();
    assert_eq!(file_count, 1);

    let _ = fs::remove_dir_all(&dir);
}
