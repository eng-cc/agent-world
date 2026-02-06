//! Persistence utilities: WorldSnapshot, WorldJournal, and error types.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

use super::kernel::WorldEvent;
use super::kernel::ChunkRuntimeConfig;
use super::types::{ActionEnvelope, ActionId, WorldEventId, WorldTime, JOURNAL_VERSION, SNAPSHOT_VERSION};
use super::world_model::{WorldConfig, WorldModel};

// ============================================================================
// Snapshot
// ============================================================================

fn default_snapshot_version() -> u32 {
    SNAPSHOT_VERSION
}

fn default_journal_version() -> u32 {
    JOURNAL_VERSION
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldSnapshot {
    #[serde(default = "default_snapshot_version")]
    pub version: u32,
    pub time: WorldTime,
    pub config: WorldConfig,
    pub model: WorldModel,
    #[serde(default)]
    pub chunk_runtime: ChunkRuntimeConfig,
    pub next_event_id: WorldEventId,
    pub next_action_id: ActionId,
    pub pending_actions: Vec<ActionEnvelope>,
    pub journal_len: usize,
}

impl WorldSnapshot {
    pub fn to_json(&self) -> Result<String, PersistError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn from_json(input: &str) -> Result<Self, PersistError> {
        let snapshot: Self = serde_json::from_str(input)?;
        snapshot.validate_version()?;
        Ok(snapshot)
    }

    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<(), PersistError> {
        write_json_to_path(self, path.as_ref())
    }

    pub fn load_json(path: impl AsRef<Path>) -> Result<Self, PersistError> {
        let snapshot: Self = read_json_from_path(path.as_ref())?;
        snapshot.validate_version()?;
        Ok(snapshot)
    }

    pub(crate) fn validate_version(&self) -> Result<(), PersistError> {
        if self.version == SNAPSHOT_VERSION {
            Ok(())
        } else {
            Err(PersistError::UnsupportedVersion {
                kind: "snapshot".to_string(),
                version: self.version,
                expected: SNAPSHOT_VERSION,
            })
        }
    }
}

// ============================================================================
// Journal
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorldJournal {
    #[serde(default = "default_journal_version")]
    pub version: u32,
    pub events: Vec<WorldEvent>,
}

impl WorldJournal {
    pub fn new() -> Self {
        Self {
            version: JOURNAL_VERSION,
            events: Vec::new(),
        }
    }

    pub fn to_json(&self) -> Result<String, PersistError> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    pub fn from_json(input: &str) -> Result<Self, PersistError> {
        let journal: Self = serde_json::from_str(input)?;
        journal.validate_version()?;
        Ok(journal)
    }

    pub fn save_json(&self, path: impl AsRef<Path>) -> Result<(), PersistError> {
        write_json_to_path(self, path.as_ref())
    }

    pub fn load_json(path: impl AsRef<Path>) -> Result<Self, PersistError> {
        let journal: Self = read_json_from_path(path.as_ref())?;
        journal.validate_version()?;
        Ok(journal)
    }

    pub(crate) fn validate_version(&self) -> Result<(), PersistError> {
        if self.version == JOURNAL_VERSION {
            Ok(())
        } else {
            Err(PersistError::UnsupportedVersion {
                kind: "journal".to_string(),
                version: self.version,
                expected: JOURNAL_VERSION,
            })
        }
    }
}

impl Default for WorldJournal {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Errors
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistError {
    Io(String),
    Serde(String),
    SnapshotMismatch { expected: usize, actual: usize },
    ReplayConflict { message: String },
    UnsupportedVersion {
        kind: String,
        version: u32,
        expected: u32,
    },
}

impl From<io::Error> for PersistError {
    fn from(err: io::Error) -> Self {
        PersistError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for PersistError {
    fn from(err: serde_json::Error) -> Self {
        PersistError::Serde(err.to_string())
    }
}

// ============================================================================
// Helper functions
// ============================================================================

pub(crate) fn write_json_to_path<T: Serialize>(value: &T, path: &Path) -> Result<(), PersistError> {
    let data = serde_json::to_vec_pretty(value)?;
    fs::write(path, data)?;
    Ok(())
}

pub(crate) fn read_json_from_path<T: DeserializeOwned>(path: &Path) -> Result<T, PersistError> {
    let data = fs::read(path)?;
    Ok(serde_json::from_slice(&data)?)
}
