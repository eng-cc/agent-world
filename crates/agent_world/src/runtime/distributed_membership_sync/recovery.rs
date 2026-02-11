use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use super::super::error::WorldError;
use super::reconciliation::{
    MembershipRevocationAlertDedupPolicy, MembershipRevocationAlertDedupState,
    MembershipRevocationAlertPolicy, MembershipRevocationAlertSink,
    MembershipRevocationAnomalyAlert, MembershipRevocationReconcilePolicy,
    MembershipRevocationReconcileSchedulePolicy, MembershipRevocationScheduleCoordinator,
    MembershipRevocationScheduleStateStore, MembershipRevocationScheduledRunReport,
};
use super::{
    logic, MembershipDirectorySignerKeyring, MembershipSyncClient, MembershipSyncSubscription,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipRevocationAlertRecoveryReport {
    pub recovered: usize,
    pub emitted_new: usize,
    pub buffered: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipRevocationCoordinatedRecoveryRunReport {
    pub acquired: bool,
    pub recovered_alerts: usize,
    pub emitted_alerts: usize,
    pub buffered_alerts: usize,
    pub run_report: Option<MembershipRevocationScheduledRunReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipRevocationCoordinatorLeaseState {
    pub holder_node_id: String,
    pub expires_at_ms: i64,
}

pub trait MembershipRevocationCoordinatorStateStore {
    fn load(
        &self,
        world_id: &str,
    ) -> Result<Option<MembershipRevocationCoordinatorLeaseState>, WorldError>;

    fn save(
        &self,
        world_id: &str,
        state: &MembershipRevocationCoordinatorLeaseState,
    ) -> Result<(), WorldError>;

    fn clear(&self, world_id: &str) -> Result<(), WorldError>;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryMembershipRevocationCoordinatorStateStore {
    leases: Arc<Mutex<BTreeMap<String, MembershipRevocationCoordinatorLeaseState>>>,
}

impl InMemoryMembershipRevocationCoordinatorStateStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MembershipRevocationCoordinatorStateStore
    for InMemoryMembershipRevocationCoordinatorStateStore
{
    fn load(
        &self,
        world_id: &str,
    ) -> Result<Option<MembershipRevocationCoordinatorLeaseState>, WorldError> {
        let world_id = logic::normalized_world_id(world_id)?;
        let guard = self.leases.lock().map_err(|_| {
            WorldError::Io("membership revocation coordinator state lock poisoned".into())
        })?;
        Ok(guard.get(&world_id).cloned())
    }

    fn save(
        &self,
        world_id: &str,
        state: &MembershipRevocationCoordinatorLeaseState,
    ) -> Result<(), WorldError> {
        let world_id = logic::normalized_world_id(world_id)?;
        let mut guard = self.leases.lock().map_err(|_| {
            WorldError::Io("membership revocation coordinator state lock poisoned".into())
        })?;
        guard.insert(world_id, state.clone());
        Ok(())
    }

    fn clear(&self, world_id: &str) -> Result<(), WorldError> {
        let world_id = logic::normalized_world_id(world_id)?;
        let mut guard = self.leases.lock().map_err(|_| {
            WorldError::Io("membership revocation coordinator state lock poisoned".into())
        })?;
        guard.remove(&world_id);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileMembershipRevocationCoordinatorStateStore {
    root_dir: PathBuf,
}

impl FileMembershipRevocationCoordinatorStateStore {
    pub fn new(root_dir: impl Into<PathBuf>) -> Result<Self, WorldError> {
        let root_dir = root_dir.into();
        fs::create_dir_all(&root_dir)?;
        Ok(Self { root_dir })
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    fn lease_path(&self, world_id: &str) -> Result<PathBuf, WorldError> {
        let world_id = logic::normalized_world_id(world_id)?;
        Ok(self
            .root_dir
            .join(format!("{world_id}.revocation-coordinator-lease.json")))
    }
}

impl MembershipRevocationCoordinatorStateStore for FileMembershipRevocationCoordinatorStateStore {
    fn load(
        &self,
        world_id: &str,
    ) -> Result<Option<MembershipRevocationCoordinatorLeaseState>, WorldError> {
        let path = self.lease_path(world_id)?;
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(path)?;
        let state = serde_json::from_slice(&bytes)?;
        Ok(Some(state))
    }

    fn save(
        &self,
        world_id: &str,
        state: &MembershipRevocationCoordinatorLeaseState,
    ) -> Result<(), WorldError> {
        let path = self.lease_path(world_id)?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let bytes = serde_json::to_vec(state)?;
        fs::write(path, bytes)?;
        Ok(())
    }

    fn clear(&self, world_id: &str) -> Result<(), WorldError> {
        let path = self.lease_path(world_id)?;
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}

pub trait MembershipRevocationAlertRecoveryStore {
    fn load_pending(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<Vec<MembershipRevocationAnomalyAlert>, WorldError>;

    fn save_pending(
        &self,
        world_id: &str,
        node_id: &str,
        alerts: &[MembershipRevocationAnomalyAlert],
    ) -> Result<(), WorldError>;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryMembershipRevocationAlertRecoveryStore {
    pending: Arc<Mutex<BTreeMap<(String, String), Vec<MembershipRevocationAnomalyAlert>>>>,
}

impl InMemoryMembershipRevocationAlertRecoveryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MembershipRevocationAlertRecoveryStore for InMemoryMembershipRevocationAlertRecoveryStore {
    fn load_pending(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<Vec<MembershipRevocationAnomalyAlert>, WorldError> {
        let key = normalized_schedule_key(world_id, node_id)?;
        let guard = self.pending.lock().map_err(|_| {
            WorldError::Io("membership revocation recovery store lock poisoned".into())
        })?;
        Ok(guard.get(&key).cloned().unwrap_or_default())
    }

    fn save_pending(
        &self,
        world_id: &str,
        node_id: &str,
        alerts: &[MembershipRevocationAnomalyAlert],
    ) -> Result<(), WorldError> {
        let key = normalized_schedule_key(world_id, node_id)?;
        let mut guard = self.pending.lock().map_err(|_| {
            WorldError::Io("membership revocation recovery store lock poisoned".into())
        })?;
        if alerts.is_empty() {
            guard.remove(&key);
        } else {
            guard.insert(key, alerts.to_vec());
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileMembershipRevocationAlertRecoveryStore {
    root_dir: PathBuf,
}

impl FileMembershipRevocationAlertRecoveryStore {
    pub fn new(root_dir: impl Into<PathBuf>) -> Result<Self, WorldError> {
        let root_dir = root_dir.into();
        fs::create_dir_all(&root_dir)?;
        Ok(Self { root_dir })
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    fn pending_path(&self, world_id: &str, node_id: &str) -> Result<PathBuf, WorldError> {
        let (world_id, node_id) = normalized_schedule_key(world_id, node_id)?;
        Ok(self.root_dir.join(format!(
            "{world_id}.{node_id}.revocation-alert-pending.json"
        )))
    }
}

impl MembershipRevocationAlertRecoveryStore for FileMembershipRevocationAlertRecoveryStore {
    fn load_pending(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<Vec<MembershipRevocationAnomalyAlert>, WorldError> {
        let path = self.pending_path(world_id, node_id)?;
        if !path.exists() {
            return Ok(Vec::new());
        }
        let bytes = fs::read(path)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    fn save_pending(
        &self,
        world_id: &str,
        node_id: &str,
        alerts: &[MembershipRevocationAnomalyAlert],
    ) -> Result<(), WorldError> {
        let path = self.pending_path(world_id, node_id)?;
        if alerts.is_empty() {
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

        let bytes = serde_json::to_vec(alerts)?;
        fs::write(path, bytes)?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct StoreBackedMembershipRevocationScheduleCoordinator {
    store: Arc<dyn MembershipRevocationCoordinatorStateStore + Send + Sync>,
}

impl StoreBackedMembershipRevocationScheduleCoordinator {
    pub fn new(store: Arc<dyn MembershipRevocationCoordinatorStateStore + Send + Sync>) -> Self {
        Self { store }
    }
}

impl MembershipRevocationScheduleCoordinator
    for StoreBackedMembershipRevocationScheduleCoordinator
{
    fn acquire(
        &self,
        world_id: &str,
        node_id: &str,
        now_ms: i64,
        lease_ttl_ms: i64,
    ) -> Result<bool, WorldError> {
        validate_coordinator_lease_ttl_ms(lease_ttl_ms)?;
        let world_id = logic::normalized_world_id(world_id)?;
        let node_id = normalized_node_id(node_id)?;

        if let Some(existing) = self.store.load(&world_id)? {
            let lease_active = now_ms < existing.expires_at_ms;
            if lease_active && existing.holder_node_id != node_id {
                return Ok(false);
            }
        }

        self.store.save(
            &world_id,
            &MembershipRevocationCoordinatorLeaseState {
                holder_node_id: node_id,
                expires_at_ms: now_ms.saturating_add(lease_ttl_ms),
            },
        )?;
        Ok(true)
    }

    fn release(&self, world_id: &str, node_id: &str) -> Result<(), WorldError> {
        let world_id = logic::normalized_world_id(world_id)?;
        let node_id = normalized_node_id(node_id)?;

        let should_clear = self
            .store
            .load(&world_id)?
            .map(|lease| lease.holder_node_id == node_id)
            .unwrap_or(false);
        if should_clear {
            self.store.clear(&world_id)?;
        }
        Ok(())
    }
}

impl MembershipSyncClient {
    pub fn emit_revocation_reconcile_alerts_with_recovery(
        &self,
        world_id: &str,
        node_id: &str,
        sink: &(dyn MembershipRevocationAlertSink + Send + Sync),
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        new_alerts: Vec<MembershipRevocationAnomalyAlert>,
    ) -> Result<MembershipRevocationAlertRecoveryReport, WorldError> {
        let mut pending = recovery_store.load_pending(world_id, node_id)?;
        let recovered_total = pending.len();
        pending.extend(new_alerts);

        let mut report = MembershipRevocationAlertRecoveryReport {
            recovered: 0,
            emitted_new: 0,
            buffered: 0,
        };

        for (index, alert) in pending.iter().enumerate() {
            if sink.emit(alert).is_ok() {
                if index < recovered_total {
                    report.recovered = report.recovered.saturating_add(1);
                } else {
                    report.emitted_new = report.emitted_new.saturating_add(1);
                }
                continue;
            }

            let failed = pending[index..].to_vec();
            report.buffered = failed.len();
            recovery_store.save_pending(world_id, node_id, &failed)?;
            return Ok(report);
        }

        recovery_store.save_pending(world_id, node_id, &[])?;
        Ok(report)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run_revocation_reconcile_coordinated_with_recovery(
        &self,
        world_id: &str,
        node_id: &str,
        now_ms: i64,
        subscription: &MembershipSyncSubscription,
        keyring: &mut MembershipDirectorySignerKeyring,
        reconcile_policy: &MembershipRevocationReconcilePolicy,
        schedule_policy: &MembershipRevocationReconcileSchedulePolicy,
        alert_policy: &MembershipRevocationAlertPolicy,
        dedup_policy: Option<&MembershipRevocationAlertDedupPolicy>,
        mut dedup_state: Option<&mut MembershipRevocationAlertDedupState>,
        schedule_store: &(dyn MembershipRevocationScheduleStateStore + Send + Sync),
        alert_sink: &(dyn MembershipRevocationAlertSink + Send + Sync),
        recovery_store: &(dyn MembershipRevocationAlertRecoveryStore + Send + Sync),
        coordinator: &(dyn MembershipRevocationScheduleCoordinator + Send + Sync),
        coordinator_lease_ttl_ms: i64,
    ) -> Result<MembershipRevocationCoordinatedRecoveryRunReport, WorldError> {
        if !coordinator.acquire(world_id, node_id, now_ms, coordinator_lease_ttl_ms)? {
            return Ok(MembershipRevocationCoordinatedRecoveryRunReport {
                acquired: false,
                recovered_alerts: 0,
                emitted_alerts: 0,
                buffered_alerts: 0,
                run_report: None,
            });
        }

        let run_outcome = (|| {
            let mut schedule_state = schedule_store.load(world_id, node_id)?;
            let run_report = self.run_revocation_reconcile_schedule(
                world_id,
                node_id,
                now_ms,
                subscription,
                keyring,
                reconcile_policy,
                schedule_policy,
                &mut schedule_state,
            )?;
            schedule_store.save(world_id, node_id, &schedule_state)?;

            let mut alerts = Vec::new();
            if let Some(reconcile_report) = run_report.reconcile_report.as_ref() {
                alerts = self.evaluate_revocation_reconcile_alerts(
                    world_id,
                    node_id,
                    now_ms,
                    reconcile_report,
                    alert_policy,
                )?;
                if let Some(dedup_policy) = dedup_policy {
                    let state = dedup_state.as_deref_mut().ok_or_else(|| {
                        WorldError::DistributedValidationFailed {
                            reason: "membership revocation dedup_state is required when dedup_policy is configured"
                                .to_string(),
                        }
                    })?;
                    alerts =
                        self.deduplicate_revocation_alerts(alerts, now_ms, dedup_policy, state)?;
                }
            }

            let recovery_report = self.emit_revocation_reconcile_alerts_with_recovery(
                world_id,
                node_id,
                alert_sink,
                recovery_store,
                alerts,
            )?;

            Ok(MembershipRevocationCoordinatedRecoveryRunReport {
                acquired: true,
                recovered_alerts: recovery_report.recovered,
                emitted_alerts: recovery_report.emitted_new,
                buffered_alerts: recovery_report.buffered,
                run_report: Some(run_report),
            })
        })();

        let release_outcome = coordinator.release(world_id, node_id);
        match (run_outcome, release_outcome) {
            (Ok(report), Ok(())) => Ok(report),
            (Err(err), Ok(())) => Err(err),
            (Ok(_), Err(release_err)) => Err(release_err),
            (Err(err), Err(_)) => Err(err),
        }
    }
}

fn validate_coordinator_lease_ttl_ms(lease_ttl_ms: i64) -> Result<(), WorldError> {
    if lease_ttl_ms <= 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!(
                "membership revocation coordinator lease_ttl_ms must be positive, got {}",
                lease_ttl_ms
            ),
        });
    }
    Ok(())
}

fn normalized_node_id(raw: &str) -> Result<String, WorldError> {
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Err(WorldError::DistributedValidationFailed {
            reason: "membership revocation checkpoint node_id cannot be empty".to_string(),
        });
    }
    if normalized.contains('/') || normalized.contains('\\') || normalized.contains("..") {
        return Err(WorldError::DistributedValidationFailed {
            reason: format!("membership revocation checkpoint node_id is invalid: {normalized}"),
        });
    }
    Ok(normalized.to_string())
}

fn normalized_schedule_key(world_id: &str, node_id: &str) -> Result<(String, String), WorldError> {
    Ok((
        logic::normalized_world_id(world_id)?,
        normalized_node_id(node_id)?,
    ))
}
