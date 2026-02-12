use serde::{Deserialize, Serialize};

use crate::membership_reconciliation::{
    MembershipRevocationAnomalyAlert, MembershipRevocationScheduledRunReport,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipRevocationAlertDeliveryMetrics {
    pub attempted: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub deferred: usize,
    pub buffered: usize,
    pub dropped_capacity: usize,
    pub dropped_retry_limit: usize,
    pub dead_lettered: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipRevocationAlertRecoveryReport {
    pub recovered: usize,
    pub emitted_new: usize,
    pub buffered: usize,
    pub deferred: usize,
    pub dropped_capacity: usize,
    pub dropped_retry_limit: usize,
    pub delivery_metrics: MembershipRevocationAlertDeliveryMetrics,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipRevocationCoordinatedRecoveryRunReport {
    pub acquired: bool,
    pub recovered_alerts: usize,
    pub emitted_alerts: usize,
    pub buffered_alerts: usize,
    pub deferred_alerts: usize,
    pub dropped_alerts_capacity: usize,
    pub dropped_alerts_retry_limit: usize,
    pub delivery_metrics: MembershipRevocationAlertDeliveryMetrics,
    pub run_report: Option<MembershipRevocationScheduledRunReport>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MembershipRevocationAlertDeadLetterReason {
    RetryLimitExceeded,
    CapacityEvicted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipRevocationAlertDeadLetterRecord {
    pub world_id: String,
    pub node_id: String,
    pub dropped_at_ms: i64,
    pub reason: MembershipRevocationAlertDeadLetterReason,
    pub pending_alert: MembershipRevocationPendingAlert,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipRevocationCoordinatorLeaseState {
    pub holder_node_id: String,
    pub expires_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MembershipRevocationPendingAlert {
    pub alert: MembershipRevocationAnomalyAlert,
    #[serde(default)]
    pub attempt: usize,
    #[serde(default)]
    pub next_retry_at_ms: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

impl MembershipRevocationPendingAlert {
    pub(crate) fn new(alert: MembershipRevocationAnomalyAlert, now_ms: i64) -> Self {
        Self {
            alert,
            attempt: 0,
            next_retry_at_ms: now_ms,
            last_error: None,
        }
    }

    pub(crate) fn from_legacy(alert: MembershipRevocationAnomalyAlert) -> Self {
        Self {
            alert,
            attempt: 0,
            next_retry_at_ms: 0,
            last_error: None,
        }
    }

    pub(crate) fn with_retry_failure(
        mut self,
        now_ms: i64,
        retry_backoff_ms: i64,
        error: String,
    ) -> Self {
        self.attempt = self.attempt.saturating_add(1);
        self.next_retry_at_ms = now_ms.saturating_add(retry_backoff_ms);
        self.last_error = Some(error);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembershipRevocationAlertAckRetryPolicy {
    pub max_pending_alerts: usize,
    pub max_retry_attempts: usize,
    pub retry_backoff_ms: i64,
}

impl Default for MembershipRevocationAlertAckRetryPolicy {
    fn default() -> Self {
        Self {
            max_pending_alerts: 256,
            max_retry_attempts: 5,
            retry_backoff_ms: 5_000,
        }
    }
}

impl MembershipRevocationAlertAckRetryPolicy {
    pub(crate) fn legacy_compatible() -> Self {
        Self {
            max_pending_alerts: usize::MAX,
            max_retry_attempts: usize::MAX,
            retry_backoff_ms: 0,
        }
    }
}
