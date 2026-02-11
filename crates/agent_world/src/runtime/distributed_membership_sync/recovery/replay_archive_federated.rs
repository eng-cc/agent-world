use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use super::super::super::error::WorldError;
use super::super::{MembershipRevocationAlertSeverity, MembershipRevocationAlertSink};
use super::replay_archive::{
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionPolicy,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillSchedulePolicy,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleStateStore,
};
use super::replay_archive_tiered::{
    MembershipRevocationDeadLetterReplayRollbackGovernanceArchiveTieredOffloadDrillAlertRunReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditTieredOffloadPolicy,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertPolicy,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertRunReport,
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertStateStore,
};
use super::replay_audit::{
    MembershipRevocationDeadLetterReplayRollbackAlertStateStore,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord,
    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore,
    MembershipRevocationDeadLetterReplayRollbackGovernanceLevel,
    MembershipRevocationDeadLetterReplayRollbackGovernanceStateStore,
};
use super::{normalized_schedule_key, MembershipSyncClient};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipRevocationDeadLetterReplayRollbackGovernanceAuditArchiveTier {
    Hot,
    Cold,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryPolicy {
    pub include_hot: bool,
    pub include_cold: bool,
    pub max_records: usize,
    pub min_audited_at_ms: Option<i64>,
    pub levels: Vec<MembershipRevocationDeadLetterReplayRollbackGovernanceLevel>,
}

impl Default for MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryPolicy {
    fn default() -> Self {
        Self {
            include_hot: true,
            include_cold: true,
            max_records: 200,
            min_audited_at_ms: None,
            levels: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryRecord {
    pub world_id: String,
    pub node_id: String,
    pub tier: MembershipRevocationDeadLetterReplayRollbackGovernanceAuditArchiveTier,
    pub audit: MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryReport {
    pub world_id: String,
    pub queried_node_count: usize,
    pub scanned_hot: usize,
    pub scanned_cold: usize,
    pub returned: usize,
    pub records:
        Vec<MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome {
    Emitted,
    SuppressedCooldown,
    SuppressedNoAnomaly,
    SkippedNoDrill,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent {
    pub world_id: String,
    pub node_id: String,
    pub event_at_ms: i64,
    pub outcome:
        MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<MembershipRevocationAlertSeverity>,
}

pub trait MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus {
    fn publish(
        &self,
        world_id: &str,
        node_id: &str,
        event: &MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent,
    ) -> Result<(), WorldError>;

    fn list(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<
        Vec<MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent>,
        WorldError,
    >;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus
{
    events: Arc<
        Mutex<
            BTreeMap<
                (String, String),
                Vec<MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent>,
            >,
        >,
    >,
}

impl InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus
    for InMemoryMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus
{
    fn publish(
        &self,
        world_id: &str,
        node_id: &str,
        event: &MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent,
    ) -> Result<(), WorldError> {
        let key = normalized_schedule_key(world_id, node_id)?;
        let mut guard = self.events.lock().map_err(|_| {
            WorldError::Io(
                "membership revocation dead-letter replay rollback governance recovery drill alert event bus lock poisoned"
                    .into(),
            )
        })?;
        guard.entry(key).or_default().push(event.clone());
        Ok(())
    }

    fn list(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<
        Vec<MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent>,
        WorldError,
    > {
        let key = normalized_schedule_key(world_id, node_id)?;
        let guard = self.events.lock().map_err(|_| {
            WorldError::Io(
                "membership revocation dead-letter replay rollback governance recovery drill alert event bus lock poisoned"
                    .into(),
            )
        })?;
        Ok(guard.get(&key).cloned().unwrap_or_default())
    }
}

#[derive(Debug, Clone)]
pub struct FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus {
    root_dir: PathBuf,
}

impl FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus {
    pub fn new(root_dir: impl Into<PathBuf>) -> Result<Self, WorldError> {
        let root_dir = root_dir.into();
        fs::create_dir_all(&root_dir)?;
        Ok(Self { root_dir })
    }

    fn event_path(&self, world_id: &str, node_id: &str) -> Result<PathBuf, WorldError> {
        let (world_id, node_id) = normalized_schedule_key(world_id, node_id)?;
        Ok(self.root_dir.join(format!(
            "{world_id}.{node_id}.revocation-dead-letter-replay-rollback-governance-recovery-drill-alert-event.jsonl"
        )))
    }
}

impl MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus
    for FileMembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus
{
    fn publish(
        &self,
        world_id: &str,
        node_id: &str,
        event: &MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent,
    ) -> Result<(), WorldError> {
        let path = self.event_path(world_id, node_id)?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let line = serde_json::to_string(event)?;
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    fn list(
        &self,
        world_id: &str,
        node_id: &str,
    ) -> Result<
        Vec<MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent>,
        WorldError,
    > {
        let path = self.event_path(world_id, node_id)?;
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(file);
        let mut events = Vec::new();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            events.push(serde_json::from_str(&line)?);
        }
        Ok(events)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipRevocationDeadLetterReplayRollbackGovernanceArchiveTieredOffloadDrillAlertEventBusRunReport
{
    pub run_report:
        MembershipRevocationDeadLetterReplayRollbackGovernanceArchiveTieredOffloadDrillAlertRunReport,
    pub alert_event: MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent,
}

impl MembershipSyncClient {
    pub fn query_revocation_dead_letter_replay_rollback_governance_audit_archive_aggregated(
        &self,
        world_id: &str,
        node_ids: &[String],
        policy: &MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryPolicy,
        hot_archive_store: &(dyn MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore
              + Send
              + Sync),
        cold_archive_store: &(dyn MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore
              + Send
              + Sync),
    ) -> Result<
        MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryReport,
        WorldError,
    > {
        validate_governance_audit_aggregate_query_policy(policy)?;
        if node_ids.is_empty() {
            return Err(WorldError::DistributedValidationFailed {
                reason: "membership revocation dead-letter rollback governance audit aggregate query requires at least one node_id".to_string(),
            });
        }
        let first_node_id = node_ids.first().ok_or_else(|| WorldError::DistributedValidationFailed {
            reason: "membership revocation dead-letter rollback governance audit aggregate query requires at least one node_id".to_string(),
        })?;
        let (normalized_world_id, _) = normalized_schedule_key(world_id, first_node_id)?;
        let mut queried_nodes = BTreeSet::new();
        for node_id in node_ids {
            let (_, node_id) = normalized_schedule_key(&normalized_world_id, node_id)?;
            queried_nodes.insert(node_id);
        }
        let queried_node_count = queried_nodes.len();

        let mut scanned_hot = 0usize;
        let mut scanned_cold = 0usize;
        let mut records = Vec::new();
        for node_id in queried_nodes {
            if policy.include_hot {
                let hot = hot_archive_store.list(&normalized_world_id, &node_id)?;
                scanned_hot = scanned_hot.saturating_add(hot.len());
                append_aggregate_records(
                    &mut records,
                    &normalized_world_id,
                    &node_id,
                    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditArchiveTier::Hot,
                    hot,
                    policy,
                );
            }
            if policy.include_cold {
                let cold = cold_archive_store.list(&normalized_world_id, &node_id)?;
                scanned_cold = scanned_cold.saturating_add(cold.len());
                append_aggregate_records(
                    &mut records,
                    &normalized_world_id,
                    &node_id,
                    MembershipRevocationDeadLetterReplayRollbackGovernanceAuditArchiveTier::Cold,
                    cold,
                    policy,
                );
            }
        }

        records.sort_by(|left, right| {
            right
                .audit
                .audited_at_ms
                .cmp(&left.audit.audited_at_ms)
                .then_with(|| left.node_id.cmp(&right.node_id))
                .then_with(|| left.tier.cmp(&right.tier))
        });
        if records.len() > policy.max_records {
            records.truncate(policy.max_records);
        }

        Ok(
            MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryReport {
                world_id: normalized_world_id,
                queried_node_count,
                scanned_hot,
                scanned_cold,
                returned: records.len(),
                records,
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn run_revocation_dead_letter_replay_rollback_governance_archive_tiered_offload_with_drill_schedule_alert_and_event_bus(
        &self,
        world_id: &str,
        node_id: &str,
        scheduled_at_ms: i64,
        retention_policy: &MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionPolicy,
        offload_policy: &MembershipRevocationDeadLetterReplayRollbackGovernanceAuditTieredOffloadPolicy,
        drill_schedule_policy: &MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillSchedulePolicy,
        drill_alert_policy: &MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertPolicy,
        hot_archive_store: &(dyn MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore
              + Send
              + Sync),
        cold_archive_store: &(dyn MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRetentionStore
              + Send
              + Sync),
        drill_schedule_state_store: &(dyn MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillScheduleStateStore
              + Send
              + Sync),
        drill_alert_state_store: &(dyn MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertStateStore
              + Send
              + Sync),
        rollback_alert_state_store: &(dyn MembershipRevocationDeadLetterReplayRollbackAlertStateStore
              + Send
              + Sync),
        rollback_governance_state_store: &(dyn MembershipRevocationDeadLetterReplayRollbackGovernanceStateStore
              + Send
              + Sync),
        rollback_governance_audit_store: &(dyn MembershipRevocationDeadLetterReplayRollbackGovernanceAuditStore
              + Send
              + Sync),
        alert_sink: &(dyn MembershipRevocationAlertSink + Send + Sync),
        event_bus: &(dyn MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventBus
              + Send
              + Sync),
    ) -> Result<
        MembershipRevocationDeadLetterReplayRollbackGovernanceArchiveTieredOffloadDrillAlertEventBusRunReport,
        WorldError,
    >{
        let run_report = self
            .run_revocation_dead_letter_replay_rollback_governance_archive_tiered_offload_with_drill_schedule_and_alert(
                world_id,
                node_id,
                scheduled_at_ms,
                retention_policy,
                offload_policy,
                drill_schedule_policy,
                drill_alert_policy,
                hot_archive_store,
                cold_archive_store,
                drill_schedule_state_store,
                drill_alert_state_store,
                rollback_alert_state_store,
                rollback_governance_state_store,
                rollback_governance_audit_store,
                alert_sink,
            )?;
        let alert_event =
            alert_event_from_run_report(scheduled_at_ms, &run_report.drill_alert_report);
        event_bus.publish(world_id, node_id, &alert_event)?;
        Ok(
            MembershipRevocationDeadLetterReplayRollbackGovernanceArchiveTieredOffloadDrillAlertEventBusRunReport {
                run_report,
                alert_event,
            },
        )
    }
}

fn validate_governance_audit_aggregate_query_policy(
    policy: &MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryPolicy,
) -> Result<(), WorldError> {
    if !policy.include_hot && !policy.include_cold {
        return Err(WorldError::DistributedValidationFailed {
            reason: "membership revocation dead-letter rollback governance audit aggregate query requires include_hot or include_cold".to_string(),
        });
    }
    if policy.max_records == 0 {
        return Err(WorldError::DistributedValidationFailed {
            reason: "membership revocation dead-letter rollback governance audit aggregate query max_records must be positive".to_string(),
        });
    }
    Ok(())
}

fn append_aggregate_records(
    records: &mut Vec<
        MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryRecord,
    >,
    world_id: &str,
    node_id: &str,
    tier: MembershipRevocationDeadLetterReplayRollbackGovernanceAuditArchiveTier,
    source: Vec<MembershipRevocationDeadLetterReplayRollbackGovernanceAuditRecord>,
    policy: &MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryPolicy,
) {
    for audit in source {
        if let Some(min_audited_at_ms) = policy.min_audited_at_ms {
            if audit.audited_at_ms < min_audited_at_ms {
                continue;
            }
        }
        if !policy.levels.is_empty() && !policy.levels.contains(&audit.governance_level) {
            continue;
        }
        records.push(
            MembershipRevocationDeadLetterReplayRollbackGovernanceAuditAggregateQueryRecord {
                world_id: world_id.to_string(),
                node_id: node_id.to_string(),
                tier,
                audit,
            },
        );
    }
}

fn alert_event_from_run_report(
    event_at_ms: i64,
    run_report: &MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertRunReport,
) -> MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent {
    let outcome = if !run_report.drill_executed {
        MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::SkippedNoDrill
    } else if !run_report.anomaly_detected {
        MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::SuppressedNoAnomaly
    } else if run_report.alert_emitted {
        MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::Emitted
    } else if run_report.cooldown_blocked {
        MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::SuppressedCooldown
    } else {
        MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEventOutcome::SuppressedNoAnomaly
    };
    let severity = if !run_report.anomaly_detected {
        None
    } else if run_report
        .reasons
        .iter()
        .any(|reason| reason == "emergency_history_detected")
    {
        Some(MembershipRevocationAlertSeverity::Critical)
    } else {
        Some(MembershipRevocationAlertSeverity::Warn)
    };
    MembershipRevocationDeadLetterReplayRollbackGovernanceRecoveryDrillAlertEvent {
        world_id: run_report.world_id.clone(),
        node_id: run_report.node_id.clone(),
        event_at_ms,
        outcome,
        reasons: run_report.reasons.clone(),
        severity,
    }
}
