use agent_world::*;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn wasm_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

#[test]
fn module_store_roundtrip() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("agent-world-store-{unique}"));

    let store = ModuleStore::new(&dir);
    let wasm_bytes = b"store-bytes";
    let hash = wasm_hash(wasm_bytes);

    store.write_artifact(&hash, wasm_bytes).unwrap();
    let loaded_bytes = store.read_artifact(&hash).unwrap();
    assert_eq!(loaded_bytes, wasm_bytes.to_vec());

    let manifest = ModuleManifest {
        module_id: "m.store".to_string(),
        name: "Store".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Reducer,
        wasm_hash: hash.clone(),
        interface_version: "wasm-1".to_string(),
        exports: vec!["reduce".to_string()],
        subscriptions: Vec::new(),
        required_caps: Vec::new(),
        limits: ModuleLimits::unbounded(),
    };

    store.write_meta(&manifest).unwrap();
    let loaded_manifest = store.read_meta(&hash).unwrap();
    assert_eq!(loaded_manifest, manifest);

    let mut registry = ModuleRegistry::default();
    let key = ModuleRegistry::record_key("m.store", "0.1.0");
    registry.records.insert(
        key,
        ModuleRecord {
            manifest,
            registered_at: 1,
            registered_by: "tester".to_string(),
            audit_event_id: None,
        },
    );
    registry
        .active
        .insert("m.store".to_string(), "0.1.0".to_string());

    store.save_registry(&registry).unwrap();
    let loaded_registry = store.load_registry().unwrap();
    assert_eq!(loaded_registry, registry);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn module_store_rejects_version_mismatch() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("agent-world-store-bad-{unique}"));

    let store = ModuleStore::new(&dir);
    fs::create_dir_all(store.root()).unwrap();
    let bad = json!({
        "version": 2,
        "updated_at": 0,
        "records": {},
        "active": {}
    });
    let data = serde_json::to_vec_pretty(&bad).unwrap();
    fs::write(store.registry_path(), data).unwrap();

    let err = store.load_registry().unwrap_err();
    assert!(matches!(err, WorldError::ModuleStoreVersionMismatch { .. }));

    let _ = fs::remove_dir_all(&dir);
}
