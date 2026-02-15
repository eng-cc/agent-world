use agent_world_wasm_abi::{ModuleManifest, ModuleRecord, ModuleRegistry};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const REGISTRY_VERSION: u64 = 1;
const REGISTRY_FILE: &str = "module_registry.json";
const MODULES_DIR: &str = "modules";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleStoreError {
    VersionMismatch { expected: u64, found: u64 },
    Io(String),
    Serde(String),
}

impl From<io::Error> for ModuleStoreError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<serde_json::Error> for ModuleStoreError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serde(error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ModuleRegistryFile {
    pub version: u64,
    pub updated_at: i64,
    pub records: BTreeMap<String, ModuleRecord>,
    pub active: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ModuleStore {
    root: PathBuf,
    registry_path: PathBuf,
    modules_dir: PathBuf,
}

impl ModuleStore {
    pub fn new(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        let registry_path = root.join(REGISTRY_FILE);
        let modules_dir = root.join(MODULES_DIR);
        Self {
            root,
            registry_path,
            modules_dir,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn registry_path(&self) -> &Path {
        &self.registry_path
    }

    pub fn modules_dir(&self) -> &Path {
        &self.modules_dir
    }

    pub fn write_artifact(
        &self,
        wasm_hash: &str,
        bytes: &[u8],
    ) -> Result<PathBuf, ModuleStoreError> {
        self.ensure_dirs()?;
        let path = self.modules_dir.join(format!("{wasm_hash}.wasm"));
        write_bytes_atomic(&path, bytes)?;
        Ok(path)
    }

    pub fn read_artifact(&self, wasm_hash: &str) -> Result<Vec<u8>, ModuleStoreError> {
        let path = self.modules_dir.join(format!("{wasm_hash}.wasm"));
        Ok(fs::read(path)?)
    }

    pub fn write_meta(&self, manifest: &ModuleManifest) -> Result<PathBuf, ModuleStoreError> {
        self.ensure_dirs()?;
        let path = self
            .modules_dir
            .join(format!("{}.meta.json", manifest.wasm_hash));
        write_json_atomic(manifest, &path)?;
        Ok(path)
    }

    pub fn read_meta(&self, wasm_hash: &str) -> Result<ModuleManifest, ModuleStoreError> {
        let path = self.modules_dir.join(format!("{wasm_hash}.meta.json"));
        read_json_from_path(&path)
    }

    pub fn save_registry(&self, registry: &ModuleRegistry) -> Result<(), ModuleStoreError> {
        self.ensure_dirs()?;
        let file = ModuleRegistryFile {
            version: REGISTRY_VERSION,
            updated_at: now_unix(),
            records: registry.records.clone(),
            active: registry.active.clone(),
        };
        write_json_atomic(&file, &self.registry_path)
    }

    pub fn load_registry(&self) -> Result<ModuleRegistry, ModuleStoreError> {
        if !self.registry_path.exists() {
            return Ok(ModuleRegistry::default());
        }
        let file: ModuleRegistryFile = read_json_from_path(&self.registry_path)?;
        if file.version != REGISTRY_VERSION {
            return Err(ModuleStoreError::VersionMismatch {
                expected: REGISTRY_VERSION,
                found: file.version,
            });
        }
        Ok(ModuleRegistry {
            records: file.records,
            active: file.active,
        })
    }

    fn ensure_dirs(&self) -> Result<(), ModuleStoreError> {
        fs::create_dir_all(&self.root)?;
        fs::create_dir_all(&self.modules_dir)?;
        Ok(())
    }
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

fn write_json_atomic<T: Serialize>(value: &T, path: &Path) -> Result<(), ModuleStoreError> {
    let tmp = path.with_extension("tmp");
    write_json_to_path(value, &tmp)?;
    fs::rename(tmp, path)?;
    Ok(())
}

fn write_bytes_atomic(path: &Path, bytes: &[u8]) -> Result<(), ModuleStoreError> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(tmp, path)?;
    Ok(())
}

fn write_json_to_path<T: Serialize>(value: &T, path: &Path) -> Result<(), ModuleStoreError> {
    let data = serde_json::to_vec_pretty(value)?;
    fs::write(path, data)?;
    Ok(())
}

fn read_json_from_path<T: DeserializeOwned>(path: &Path) -> Result<T, ModuleStoreError> {
    let data = fs::read(path)?;
    Ok(serde_json::from_slice(&data)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_world_wasm_abi::{
        ModuleKind, ModuleLimits, ModuleManifest, ModuleRecord, ModuleRole, ModuleSubscription,
    };

    fn temp_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{unique}"))
    }

    fn sample_manifest(hash: &str) -> ModuleManifest {
        ModuleManifest {
            module_id: "m.store".to_string(),
            name: "Store".to_string(),
            version: "0.1.0".to_string(),
            kind: ModuleKind::Reducer,
            role: ModuleRole::Domain,
            wasm_hash: hash.to_string(),
            interface_version: "wasm-1".to_string(),
            exports: vec!["reduce".to_string()],
            subscriptions: Vec::<ModuleSubscription>::new(),
            required_caps: Vec::new(),
            artifact_identity: None,
            limits: ModuleLimits::unbounded(),
        }
    }

    #[test]
    fn roundtrip_artifact_meta_and_registry() {
        let dir = temp_dir("agent-world-wasm-store");
        let store = ModuleStore::new(&dir);

        let wasm_hash = "abc123";
        let bytes = vec![1_u8, 2, 3];
        store
            .write_artifact(wasm_hash, &bytes)
            .expect("write artifact");
        assert_eq!(
            store.read_artifact(wasm_hash).expect("read artifact"),
            bytes
        );

        let manifest = sample_manifest(wasm_hash);
        store.write_meta(&manifest).expect("write meta");
        assert_eq!(store.read_meta(wasm_hash).expect("read meta"), manifest);

        let mut registry = ModuleRegistry::default();
        let key = ModuleRegistry::record_key("m.store", "0.1.0");
        registry.records.insert(
            key,
            ModuleRecord {
                manifest,
                registered_at: 1,
                registered_by: "tester".to_string(),
                audit_event_id: Some(2),
            },
        );
        registry
            .active
            .insert("m.store".to_string(), "0.1.0".to_string());

        store.save_registry(&registry).expect("save registry");
        assert_eq!(store.load_registry().expect("load registry"), registry);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn load_registry_rejects_version_mismatch() {
        let dir = temp_dir("agent-world-wasm-store-version");
        let store = ModuleStore::new(&dir);
        fs::create_dir_all(store.root()).expect("create root");
        let invalid = serde_json::json!({
            "version": 99,
            "updated_at": 0,
            "records": {},
            "active": {}
        });
        fs::write(
            store.registry_path(),
            serde_json::to_vec_pretty(&invalid).expect("encode"),
        )
        .expect("write registry");

        assert!(matches!(
            store.load_registry(),
            Err(ModuleStoreError::VersionMismatch {
                expected: 1,
                found: 99
            })
        ));

        let _ = fs::remove_dir_all(dir);
    }
}
