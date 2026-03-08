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
        artifact_identity: Some(super::signed_test_artifact_identity(wasm_hash.as_str())),
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
    assert_eq!(
        restored.tick_consensus_records(),
        world.tick_consensus_records()
    );
    restored
        .verify_tick_consensus_chain()
        .expect("verify persisted tick consensus chain");

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
fn persist_writes_sidecar_generation_index_and_pinset() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-sidecar".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().expect("step");

    let dir = temp_dir("persist-sidecar-generation-index");
    world
        .save_to_dir(&dir)
        .expect("save world with sidecar generation index");

    let index_path = dir.join(".distfs-state/sidecar-generations/index.json");
    let index: serde_json::Value = serde_json::from_slice(
        &fs::read(index_path.as_path()).expect("read sidecar generation index"),
    )
    .expect("decode sidecar generation index");
    let latest_generation = index
        .get("latest_generation")
        .and_then(|value| value.as_str())
        .expect("latest generation");
    assert!(index.get("rollback_safe_generation").is_none());
    assert!(dir
        .join(".distfs-state/sidecar-generations/generation.tmp")
        .exists());
    assert!(dir
        .join(format!(
            ".distfs-state/sidecar-generations/generations/{latest_generation}.json"
        ))
        .exists());

    let generation = index
        .get("generations")
        .and_then(|value| value.get(latest_generation))
        .expect("latest generation entry");
    let pinned_blob_hashes = generation
        .get("pinned_blob_hashes")
        .and_then(|value| value.as_array())
        .expect("pinned blob hashes");
    assert!(!pinned_blob_hashes.is_empty());
    let manifest: agent_world_proto::distributed::SnapshotManifest = serde_json::from_slice(
        &fs::read(dir.join("snapshot.manifest.json")).expect("read snapshot manifest"),
    )
    .expect("decode snapshot manifest");
    let journal_segments: Vec<agent_world_proto::distributed_storage::JournalSegmentRef> =
        serde_json::from_slice(
            &fs::read(dir.join("journal.segments.json")).expect("read journal segments"),
        )
        .expect("decode journal segments");
    let expected_pins = manifest
        .chunks
        .iter()
        .map(|chunk| chunk.content_hash.clone())
        .chain(
            journal_segments
                .iter()
                .map(|segment| segment.content_hash.clone()),
        )
        .collect::<std::collections::BTreeSet<_>>();
    let actual_pins = pinned_blob_hashes
        .iter()
        .map(|value| value.as_str().expect("pin string").to_string())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(actual_pins, expected_pins);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn persist_sidecar_generation_record_points_to_generation_local_payloads() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-sidecar-local-payload".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().expect("step");

    let dir = temp_dir("persist-sidecar-generation-local-payload");
    world.save_to_dir(&dir).expect("save world");

    let index: serde_json::Value = serde_json::from_slice(
        &fs::read(dir.join(".distfs-state/sidecar-generations/index.json"))
            .expect("read sidecar generation index"),
    )
    .expect("decode sidecar generation index");
    let latest_generation = index
        .get("latest_generation")
        .and_then(|value| value.as_str())
        .expect("latest generation");
    let generation_record: serde_json::Value = serde_json::from_slice(
        &fs::read(dir.join(format!(
            ".distfs-state/sidecar-generations/generations/{latest_generation}.json"
        )))
        .expect("read sidecar generation record"),
    )
    .expect("decode sidecar generation record");
    let snapshot_manifest_path = generation_record
        .get("snapshot_manifest_path")
        .and_then(|value| value.as_str())
        .expect("snapshot manifest path");
    let journal_segments_path = generation_record
        .get("journal_segments_path")
        .and_then(|value| value.as_str())
        .expect("journal segments path");
    assert!(snapshot_manifest_path.contains(&format!("payloads/{latest_generation}/")));
    assert!(journal_segments_path.contains(&format!("payloads/{latest_generation}/")));
    assert!(dir
        .join(".distfs-state")
        .join(snapshot_manifest_path)
        .exists());
    assert!(dir
        .join(".distfs-state")
        .join(journal_segments_path)
        .exists());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn persist_sidecar_generation_switch_keeps_latest_and_rollback_safe_only() {
    let mut world = World::new();
    let dir = temp_dir("persist-sidecar-generation-keep-two");

    for step_index in 0..3 {
        world.submit_action(Action::RegisterAgent {
            agent_id: format!("agent-sidecar-keep-{step_index}"),
            pos: pos(step_index as f64, step_index as f64),
        });
        world.step().expect("step before save");
        world.save_to_dir(&dir).expect("save world");
    }

    let index: serde_json::Value = serde_json::from_slice(
        &fs::read(dir.join(".distfs-state/sidecar-generations/index.json"))
            .expect("read sidecar generation index"),
    )
    .expect("decode sidecar generation index");
    let latest_generation = index
        .get("latest_generation")
        .and_then(|value| value.as_str())
        .expect("latest generation")
        .to_string();
    let rollback_safe_generation = index
        .get("rollback_safe_generation")
        .and_then(|value| value.as_str())
        .expect("rollback safe generation")
        .to_string();
    let generations = index
        .get("generations")
        .and_then(|value| value.as_object())
        .expect("generation map");
    assert_eq!(generations.len(), 2);
    assert!(generations.contains_key(latest_generation.as_str()));
    assert!(generations.contains_key(rollback_safe_generation.as_str()));
    assert!(dir
        .join(format!(
            ".distfs-state/sidecar-generations/payloads/{latest_generation}"
        ))
        .exists());
    assert!(dir
        .join(format!(
            ".distfs-state/sidecar-generations/payloads/{rollback_safe_generation}"
        ))
        .exists());
    let staging_entries =
        fs::read_dir(dir.join(".distfs-state/sidecar-generations/generation.tmp"))
            .expect("read staging dir")
            .count();
    assert_eq!(staging_entries, 0);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn persist_sidecar_generation_sweep_keeps_only_retained_blobs() {
    let mut world = World::new();
    let dir = temp_dir("persist-sidecar-generation-sweep");

    for step_index in 0..3 {
        world.submit_action(Action::RegisterAgent {
            agent_id: format!("agent-sidecar-sweep-{step_index}"),
            pos: pos(step_index as f64, step_index as f64),
        });
        world.step().expect("step before save");
        world.save_to_dir(&dir).expect("save world");
    }

    let index: serde_json::Value = serde_json::from_slice(
        &fs::read(dir.join(".distfs-state/sidecar-generations/index.json"))
            .expect("read sidecar generation index"),
    )
    .expect("decode sidecar generation index");
    let latest_generation = index
        .get("latest_generation")
        .and_then(|value| value.as_str())
        .expect("latest generation")
        .to_string();
    let rollback_safe_generation = index
        .get("rollback_safe_generation")
        .and_then(|value| value.as_str())
        .expect("rollback safe generation")
        .to_string();
    let generations = index
        .get("generations")
        .and_then(|value| value.as_object())
        .expect("generation map");
    let retained_blob_hashes = [
        latest_generation.as_str(),
        rollback_safe_generation.as_str(),
    ]
    .into_iter()
    .flat_map(|generation_id| {
        generations
            .get(generation_id)
            .and_then(|value| value.get("pinned_blob_hashes"))
            .and_then(|value| value.as_array())
            .expect("generation pinned blob hashes")
            .iter()
            .map(|value| value.as_str().expect("pin string").to_string())
            .collect::<Vec<_>>()
    })
    .collect::<std::collections::BTreeSet<_>>();
    let actual_blob_hashes = LocalCasStore::new(dir.join(".distfs-state"))
        .list_blob_hashes()
        .expect("list blob hashes")
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(actual_blob_hashes, retained_blob_hashes);
    assert_eq!(
        index
            .get("last_gc_result")
            .and_then(|value| value.get("status"))
            .and_then(|value| value.as_str()),
        Some("success")
    );
    assert!(index
        .get("last_gc_result")
        .and_then(|value| value.get("error"))
        .is_none());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn persist_sidecar_generation_gc_failure_preserves_latest_and_rollback_blobs() {
    let mut world = World::new();
    let dir = temp_dir("persist-sidecar-generation-gc-failure");

    for step_index in 0..2 {
        world.submit_action(Action::RegisterAgent {
            agent_id: format!("agent-sidecar-gc-failure-{step_index}"),
            pos: pos(step_index as f64, step_index as f64),
        });
        world.step().expect("step before save");
        world.save_to_dir(&dir).expect("save world");
    }

    let second_index: serde_json::Value = serde_json::from_slice(
        &fs::read(dir.join(".distfs-state/sidecar-generations/index.json"))
            .expect("read second sidecar generation index"),
    )
    .expect("decode second sidecar generation index");
    let second_latest_generation = second_index
        .get("latest_generation")
        .and_then(|value| value.as_str())
        .expect("second latest generation")
        .to_string();
    fs::write(
        dir.join(format!(
            ".distfs-state/sidecar-generations/payloads/{second_latest_generation}/journal.segments.json"
        )),
        b"not-json",
    )
    .expect("corrupt rollback-safe generation payload");

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-sidecar-gc-failure-2".to_string(),
        pos: pos(2.0, 2.0),
    });
    world.step().expect("third step before save");
    world
        .save_to_dir(&dir)
        .expect("save world should degrade instead of failing");

    let third_index: serde_json::Value = serde_json::from_slice(
        &fs::read(dir.join(".distfs-state/sidecar-generations/index.json"))
            .expect("read third sidecar generation index"),
    )
    .expect("decode third sidecar generation index");
    let latest_generation = third_index
        .get("latest_generation")
        .and_then(|value| value.as_str())
        .expect("latest generation")
        .to_string();
    let rollback_safe_generation = third_index
        .get("rollback_safe_generation")
        .and_then(|value| value.as_str())
        .expect("rollback safe generation")
        .to_string();
    assert_eq!(rollback_safe_generation, second_latest_generation);
    assert_eq!(
        third_index
            .get("last_gc_result")
            .and_then(|value| value.get("status"))
            .and_then(|value| value.as_str()),
        Some("failed")
    );
    assert!(third_index
        .get("last_gc_result")
        .and_then(|value| value.get("error"))
        .and_then(|value| value.as_str())
        .is_some());

    let generations = third_index
        .get("generations")
        .and_then(|value| value.as_object())
        .expect("generation map");
    let retained_blob_hashes = [
        latest_generation.as_str(),
        rollback_safe_generation.as_str(),
    ]
    .into_iter()
    .flat_map(|generation_id| {
        generations
            .get(generation_id)
            .and_then(|value| value.get("pinned_blob_hashes"))
            .and_then(|value| value.as_array())
            .expect("generation pinned blob hashes")
            .iter()
            .map(|value| value.as_str().expect("pin string").to_string())
            .collect::<Vec<_>>()
    })
    .collect::<std::collections::BTreeSet<_>>();
    let actual_blob_hashes = LocalCasStore::new(dir.join(".distfs-state"))
        .list_blob_hashes()
        .expect("list blob hashes")
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    assert!(retained_blob_hashes.is_subset(&actual_blob_hashes));

    let restored =
        World::load_from_dir(&dir).expect("restore from latest generation after gc failure");
    assert_eq!(restored.state(), world.state());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn persist_sidecar_generation_interrupted_save_rolls_back_and_retry_cleans_staging() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-sidecar-interrupt-0".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().expect("first step");

    let dir = temp_dir("persist-sidecar-generation-interrupt");
    world.save_to_dir(&dir).expect("first save");
    let first_restored = World::load_from_dir(&dir).expect("load first saved world");
    let first_state = first_restored.state().clone();
    let first_index: serde_json::Value = serde_json::from_slice(
        &fs::read(dir.join(".distfs-state/sidecar-generations/index.json"))
            .expect("read first sidecar generation index"),
    )
    .expect("decode first sidecar generation index");
    let first_latest_generation = first_index
        .get("latest_generation")
        .and_then(|value| value.as_str())
        .expect("first latest generation")
        .to_string();

    fs::write(
        dir.join(".distfs-state/sidecar-generations/.test-fail-after-stage"),
        b"1",
    )
    .expect("install sidecar failpoint");
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-sidecar-interrupt-1".to_string(),
        pos: pos(1.0, 1.0),
    });
    world.step().expect("second step");
    let err = world
        .save_to_dir(&dir)
        .expect_err("interrupted save should fail after staging");
    assert!(matches!(
        err,
        WorldError::DistributedValidationFailed { .. }
    ));
    assert!(
        fs::read_dir(dir.join(".distfs-state/sidecar-generations/generation.tmp"))
            .expect("read staging dir after interrupted save")
            .count()
            > 0
    );

    let rolled_back = World::load_from_dir(&dir).expect("load after interrupted save");
    assert_eq!(rolled_back.state(), &first_state);
    let interrupted_index: serde_json::Value = serde_json::from_slice(
        &fs::read(dir.join(".distfs-state/sidecar-generations/index.json"))
            .expect("read interrupted sidecar generation index"),
    )
    .expect("decode interrupted sidecar generation index");
    assert_eq!(
        interrupted_index
            .get("latest_generation")
            .and_then(|value| value.as_str()),
        Some(first_latest_generation.as_str())
    );

    fs::remove_file(dir.join(".distfs-state/sidecar-generations/.test-fail-after-stage"))
        .expect("remove sidecar failpoint");
    world
        .save_to_dir(&dir)
        .expect("retry save after interruption");

    let staging_entries =
        fs::read_dir(dir.join(".distfs-state/sidecar-generations/generation.tmp"))
            .expect("read staging dir after retry")
            .count();
    assert_eq!(staging_entries, 0);
    let restored = World::load_from_dir(&dir).expect("load after retry save");
    assert_eq!(restored.state(), world.state());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn persist_sidecar_generation_retry_cleans_partial_staging_and_orphan_blob() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-sidecar-partial-0".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().expect("first step");

    let dir = temp_dir("persist-sidecar-generation-partial-staging");
    world.save_to_dir(&dir).expect("first save");

    let partial_staging_dir =
        dir.join(".distfs-state/sidecar-generations/generation.tmp/interrupted-partial");
    fs::create_dir_all(partial_staging_dir.as_path()).expect("create partial staging dir");
    fs::write(
        partial_staging_dir.join("snapshot.manifest.json"),
        br#"{"partial""#,
    )
    .expect("write partial snapshot manifest");
    fs::write(
        partial_staging_dir.join("journal.segments.json"),
        br#"[{"partial""#,
    )
    .expect("write partial journal segments");
    fs::write(partial_staging_dir.join("generation.json"), b"not-json")
        .expect("write partial generation record");

    let orphan_bytes = b"sidecar-orphan-after-interrupt";
    let orphan_hash = agent_world_distfs::blake3_hex(orphan_bytes);
    let orphan_blob_path = dir.join(format!(".distfs-state/blobs/{orphan_hash}.blob"));
    fs::write(orphan_blob_path.as_path(), orphan_bytes).expect("write orphan blob");

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-sidecar-partial-1".to_string(),
        pos: pos(1.0, 1.0),
    });
    world.step().expect("second step");
    world
        .save_to_dir(&dir)
        .expect("retry save after partial staging");

    let staging_entries =
        fs::read_dir(dir.join(".distfs-state/sidecar-generations/generation.tmp"))
            .expect("read staging dir after cleanup")
            .count();
    assert_eq!(staging_entries, 0);
    assert!(!orphan_blob_path.exists());
    let restored = World::load_from_dir(&dir).expect("load after cleanup save");
    assert_eq!(restored.state(), world.state());

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn persist_updates_sidecar_generation_index_with_rollback_safe_generation() {
    let mut world = World::new();
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-sidecar-2".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().expect("first step");

    let dir = temp_dir("persist-sidecar-generation-rollback-safe");
    world.save_to_dir(&dir).expect("first save");

    let first_index: serde_json::Value = serde_json::from_slice(
        &fs::read(dir.join(".distfs-state/sidecar-generations/index.json"))
            .expect("read first sidecar generation index"),
    )
    .expect("decode first sidecar generation index");
    let first_latest = first_index
        .get("latest_generation")
        .and_then(|value| value.as_str())
        .expect("first latest generation")
        .to_string();

    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-sidecar-3".to_string(),
        pos: pos(1.0, 1.0),
    });
    world.step().expect("second step");
    world.save_to_dir(&dir).expect("second save");

    let second_index: serde_json::Value = serde_json::from_slice(
        &fs::read(dir.join(".distfs-state/sidecar-generations/index.json"))
            .expect("read second sidecar generation index"),
    )
    .expect("decode second sidecar generation index");
    let latest_generation = second_index
        .get("latest_generation")
        .and_then(|value| value.as_str())
        .expect("latest generation");
    let rollback_safe_generation = second_index
        .get("rollback_safe_generation")
        .and_then(|value| value.as_str())
        .expect("rollback safe generation");
    assert_ne!(latest_generation, rollback_safe_generation);
    assert_eq!(rollback_safe_generation, first_latest);
    assert!(second_index
        .get("generations")
        .and_then(|value| value.get(latest_generation))
        .is_some());
    assert!(second_index
        .get("generations")
        .and_then(|value| value.get(rollback_safe_generation))
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
fn rollback_with_reconciliation_recovers_from_detected_tick_consensus_drift() {
    let mut world = World::new();
    world
        .bind_node_identity("relay.node.1", "relay-public-key-1")
        .expect("bind relay identity");
    world.submit_action(Action::RegisterAgent {
        agent_id: "agent-1".to_string(),
        pos: pos(0.0, 0.0),
    });
    world.step().expect("step");

    let stable_snapshot = world.snapshot();
    let stable_journal = world.journal().clone();

    world
        .record_tick_consensus_propagation_for_tick(0, "relay.node.1")
        .expect("inject propagation record that breaks parent ordering");
    let drift = world
        .first_tick_consensus_drift()
        .expect("drift report should be present");
    assert_eq!(drift.tick, 0);
    assert!(
        drift.reason.contains("parent hash mismatch"),
        "unexpected drift reason: {}",
        drift.reason
    );
    world
        .verify_tick_consensus_chain()
        .expect_err("drifted chain should fail verification");

    world
        .rollback_to_snapshot_with_reconciliation(
            stable_snapshot,
            stable_journal,
            "reconcile-after-drift",
        )
        .expect("rollback with reconciliation");

    assert!(
        world.first_tick_consensus_drift().is_none(),
        "drift should be fully reconciled after rollback"
    );
    world
        .verify_tick_consensus_chain()
        .expect("reconciled chain should verify");
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
