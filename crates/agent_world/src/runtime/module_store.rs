//! Module storage persistence for artifacts and registry.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::error::WorldError;
use super::modules::{ModuleManifest, ModuleRecord, ModuleRegistry};
use super::util::{read_json_from_path, write_json_to_path};

const REGISTRY_VERSION: u64 = 1;
const REGISTRY_FILE: &str = "module_registry.json";
const MODULES_DIR: &str = "modules";

/// On-disk registry representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ModuleRegistryFile {
    pub version: u64,
    pub updated_at: i64,
    pub records: BTreeMap<String, ModuleRecord>,
    pub active: BTreeMap<String, String>,
}

/// File-based module store for artifacts, meta, and registry.
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

    pub fn write_artifact(&self, wasm_hash: &str, bytes: &[u8]) -> Result<PathBuf, WorldError> {
        self.ensure_dirs()?;
        let path = self.modules_dir.join(format!("{wasm_hash}.wasm"));
        write_bytes_atomic(&path, bytes)?;
        Ok(path)
    }

    pub fn read_artifact(&self, wasm_hash: &str) -> Result<Vec<u8>, WorldError> {
        let path = self.modules_dir.join(format!("{wasm_hash}.wasm"));
        Ok(fs::read(path)?)
    }

    pub fn write_meta(&self, manifest: &ModuleManifest) -> Result<PathBuf, WorldError> {
        self.ensure_dirs()?;
        let path = self
            .modules_dir
            .join(format!("{}.meta.json", manifest.wasm_hash));
        write_json_atomic(manifest, &path)?;
        Ok(path)
    }

    pub fn read_meta(&self, wasm_hash: &str) -> Result<ModuleManifest, WorldError> {
        let path = self.modules_dir.join(format!("{wasm_hash}.meta.json"));
        read_json_from_path(&path)
    }

    pub fn save_registry(&self, registry: &ModuleRegistry) -> Result<(), WorldError> {
        self.ensure_dirs()?;
        let file = ModuleRegistryFile {
            version: REGISTRY_VERSION,
            updated_at: now_unix(),
            records: registry.records.clone(),
            active: registry.active.clone(),
        };
        write_json_atomic(&file, &self.registry_path)
    }

    pub fn load_registry(&self) -> Result<ModuleRegistry, WorldError> {
        if !self.registry_path.exists() {
            return Ok(ModuleRegistry::default());
        }
        let file: ModuleRegistryFile = read_json_from_path(&self.registry_path)?;
        if file.version != REGISTRY_VERSION {
            return Err(WorldError::ModuleStoreVersionMismatch {
                expected: REGISTRY_VERSION,
                found: file.version,
            });
        }
        Ok(ModuleRegistry {
            records: file.records,
            active: file.active,
        })
    }

    fn ensure_dirs(&self) -> Result<(), WorldError> {
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

fn write_json_atomic<T: Serialize>(value: &T, path: &Path) -> Result<(), WorldError> {
    let tmp = path.with_extension("tmp");
    write_json_to_path(value, &tmp)?;
    fs::rename(tmp, path)?;
    Ok(())
}

fn write_bytes_atomic(path: &Path, bytes: &[u8]) -> Result<(), WorldError> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, bytes)?;
    fs::rename(tmp, path)?;
    Ok(())
}
