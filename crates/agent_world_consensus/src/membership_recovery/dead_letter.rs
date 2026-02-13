use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use super::super::error::WorldError;

use super::types::{
    MembershipRevocationAlertDeadLetterRecord, MembershipRevocationAlertDeliveryMetrics,
};

pub trait MembershipRevocationAlertDeadLetterStore {
    fn append(&self, record: &MembershipRevocationAlertDeadLetterRecord) -> Result<(), WorldError>;

    fn list(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<Vec<MembershipRevocationAlertDeadLetterRecord>, WorldError>;

    fn replace(
        &self,
        world_id: &str,
        node_id: &str,
        records: &[MembershipRevocationAlertDeadLetterRecord],
    ) -> Result<(), WorldError>;

    fn append_delivery_metrics(
        &self,
        world_id: &str,
        node_id: &str,
        exported_at_ms: i64,
        metrics: &MembershipRevocationAlertDeliveryMetrics,
    ) -> Result<(), WorldError>;

    fn list_delivery_metrics(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<Vec<(i64, MembershipRevocationAlertDeliveryMetrics)>, WorldError>;
}

#[derive(Debug, Clone, Default)]
pub struct NoopMembershipRevocationAlertDeadLetterStore;

impl MembershipRevocationAlertDeadLetterStore for NoopMembershipRevocationAlertDeadLetterStore {
    fn append(
        &self,
        _record: &MembershipRevocationAlertDeadLetterRecord,
    ) -> Result<(), WorldError> {
        Ok(())
    }

    fn list(
        &self,
        _world_id: &str,
        _node_id: &str,
    ) -> Result<Vec<MembershipRevocationAlertDeadLetterRecord>, WorldError> {
        Ok(Vec::new())
    }

    fn replace(
        &self,
        _world_id: &str,
        _node_id: &str,
        _records: &[MembershipRevocationAlertDeadLetterRecord],
    ) -> Result<(), WorldError> {
        Ok(())
    }

    fn append_delivery_metrics(
        &self,
        _world_id: &str,
        _node_id: &str,
        _exported_at_ms: i64,
        _metrics: &MembershipRevocationAlertDeliveryMetrics,
    ) -> Result<(), WorldError> {
        Ok(())
    }

    fn list_delivery_metrics(
        &self,
        _world_id: &str,
        _node_id: &str,
    ) -> Result<Vec<(i64, MembershipRevocationAlertDeliveryMetrics)>, WorldError> {
        Ok(Vec::new())
    }
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryMembershipRevocationAlertDeadLetterStore {
    records: Arc<Mutex<BTreeMap<(String, String), Vec<MembershipRevocationAlertDeadLetterRecord>>>>,
    delivery_metrics: Arc<
        Mutex<BTreeMap<(String, String), Vec<(i64, MembershipRevocationAlertDeliveryMetrics)>>>,
    >,
}

impl InMemoryMembershipRevocationAlertDeadLetterStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn list(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<Vec<MembershipRevocationAlertDeadLetterRecord>, WorldError> {
        let key = super::normalized_schedule_key(world_id, node_id)?;
        let guard = self.records.lock().map_err(|_| {
            WorldError::Io("membership revocation dead-letter store lock poisoned".into())
        })?;
        Ok(guard.get(&key).cloned().unwrap_or_default())
    }

    pub fn list_delivery_metrics(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<Vec<(i64, MembershipRevocationAlertDeliveryMetrics)>, WorldError> {
        let key = super::normalized_schedule_key(world_id, node_id)?;
        let guard = self.delivery_metrics.lock().map_err(|_| {
            WorldError::Io("membership revocation dead-letter metrics lock poisoned".into())
        })?;
        Ok(guard.get(&key).cloned().unwrap_or_default())
    }
}

impl MembershipRevocationAlertDeadLetterStore for InMemoryMembershipRevocationAlertDeadLetterStore {
    fn append(&self, record: &MembershipRevocationAlertDeadLetterRecord) -> Result<(), WorldError> {
        let key = super::normalized_schedule_key(&record.world_id, &record.node_id)?;
        let mut guard = self.records.lock().map_err(|_| {
            WorldError::Io("membership revocation dead-letter store lock poisoned".into())
        })?;
        guard.entry(key).or_default().push(record.clone());
        Ok(())
    }

    fn list(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<Vec<MembershipRevocationAlertDeadLetterRecord>, WorldError> {
        InMemoryMembershipRevocationAlertDeadLetterStore::list(self, world_id, node_id)
    }

    fn replace(
        &self,
        world_id: &str,
        node_id: &str,
        records: &[MembershipRevocationAlertDeadLetterRecord],
    ) -> Result<(), WorldError> {
        let key = super::normalized_schedule_key(world_id, node_id)?;
        let mut guard = self.records.lock().map_err(|_| {
            WorldError::Io("membership revocation dead-letter store lock poisoned".into())
        })?;
        if records.is_empty() {
            guard.remove(&key);
        } else {
            guard.insert(key, records.to_vec());
        }
        Ok(())
    }

    fn append_delivery_metrics(
        &self,
        world_id: &str,
        node_id: &str,
        exported_at_ms: i64,
        metrics: &MembershipRevocationAlertDeliveryMetrics,
    ) -> Result<(), WorldError> {
        let key = super::normalized_schedule_key(world_id, node_id)?;
        let mut guard = self.delivery_metrics.lock().map_err(|_| {
            WorldError::Io("membership revocation dead-letter metrics lock poisoned".into())
        })?;
        guard
            .entry(key)
            .or_default()
            .push((exported_at_ms, metrics.clone()));
        Ok(())
    }

    fn list_delivery_metrics(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<Vec<(i64, MembershipRevocationAlertDeliveryMetrics)>, WorldError> {
        InMemoryMembershipRevocationAlertDeadLetterStore::list_delivery_metrics(
            self, world_id, node_id,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct FileMembershipRevocationAlertDeliveryMetricsLine {
    exported_at_ms: i64,
    metrics: MembershipRevocationAlertDeliveryMetrics,
}

#[derive(Debug, Clone)]
pub struct FileMembershipRevocationAlertDeadLetterStore {
    root_dir: PathBuf,
}

impl FileMembershipRevocationAlertDeadLetterStore {
    pub fn new(root_dir: impl Into<PathBuf>) -> Result<Self, WorldError> {
        let root_dir = root_dir.into();
        fs::create_dir_all(&root_dir)?;
        Ok(Self { root_dir })
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    fn dead_letter_path(&self, world_id: &str, node_id: &str) -> Result<PathBuf, WorldError> {
        let (world_id, node_id) = super::normalized_schedule_key(world_id, node_id)?;
        Ok(self.root_dir.join(format!(
            "{world_id}.{node_id}.revocation-alert-dead-letter.jsonl"
        )))
    }

    fn delivery_metrics_path(&self, world_id: &str, node_id: &str) -> Result<PathBuf, WorldError> {
        let (world_id, node_id) = super::normalized_schedule_key(world_id, node_id)?;
        Ok(self.root_dir.join(format!(
            "{world_id}.{node_id}.revocation-alert-delivery-metrics.jsonl"
        )))
    }

    pub fn list(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<Vec<MembershipRevocationAlertDeadLetterRecord>, WorldError> {
        let path = self.dead_letter_path(world_id, node_id)?;
        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            records.push(serde_json::from_str(&line)?);
        }
        Ok(records)
    }

    pub fn list_delivery_metrics(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<Vec<(i64, MembershipRevocationAlertDeliveryMetrics)>, WorldError> {
        let path = self.delivery_metrics_path(world_id, node_id)?;
        if !path.exists() {
            return Ok(Vec::new());
        }

        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(file);
        let mut lines = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let parsed: FileMembershipRevocationAlertDeliveryMetricsLine =
                serde_json::from_str(&line)?;
            lines.push((parsed.exported_at_ms, parsed.metrics));
        }
        Ok(lines)
    }
}

impl MembershipRevocationAlertDeadLetterStore for FileMembershipRevocationAlertDeadLetterStore {
    fn append(&self, record: &MembershipRevocationAlertDeadLetterRecord) -> Result<(), WorldError> {
        let path = self.dead_letter_path(&record.world_id, &record.node_id)?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let line = serde_json::to_string(record)?;
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    fn list(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<Vec<MembershipRevocationAlertDeadLetterRecord>, WorldError> {
        FileMembershipRevocationAlertDeadLetterStore::list(self, world_id, node_id)
    }

    fn replace(
        &self,
        world_id: &str,
        node_id: &str,
        records: &[MembershipRevocationAlertDeadLetterRecord],
    ) -> Result<(), WorldError> {
        let path = self.dead_letter_path(world_id, node_id)?;
        if records.is_empty() {
            if path.exists() {
                fs::remove_file(path)?;
            }
            return Ok(());
        }

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let mut payload = Vec::new();
        for record in records {
            payload.push(serde_json::to_string(record)?);
        }
        fs::write(path, payload.join("\n") + "\n")?;
        Ok(())
    }

    fn append_delivery_metrics(
        &self,
        world_id: &str,
        node_id: &str,
        exported_at_ms: i64,
        metrics: &MembershipRevocationAlertDeliveryMetrics,
    ) -> Result<(), WorldError> {
        let path = self.delivery_metrics_path(world_id, node_id)?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let line = serde_json::to_string(&FileMembershipRevocationAlertDeliveryMetricsLine {
            exported_at_ms,
            metrics: metrics.clone(),
        })?;
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    fn list_delivery_metrics(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<Vec<(i64, MembershipRevocationAlertDeliveryMetrics)>, WorldError> {
        FileMembershipRevocationAlertDeadLetterStore::list_delivery_metrics(self, world_id, node_id)
    }
}
