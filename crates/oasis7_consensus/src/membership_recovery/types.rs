use serde::{Deserialize, Serialize};

use crate::error::WorldError;

use super::super::membership_reconciliation::{
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
    ) -> Result<Self, WorldError> {
        self.attempt =
            self.attempt
                .checked_add(1)
                .ok_or_else(|| WorldError::DistributedValidationFailed {
                    reason: format!(
                        "membership revocation pending alert attempt overflow: attempt={}",
                        self.attempt
                    ),
                })?;
        self.next_retry_at_ms = now_ms.checked_add(retry_backoff_ms).ok_or_else(|| {
            WorldError::DistributedValidationFailed {
                reason: format!(
                    "membership revocation pending alert retry timestamp overflow: now_ms={now_ms}, retry_backoff_ms={retry_backoff_ms}"
                ),
            }
        })?;
        self.last_error = Some(error);
        Ok(self)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_alert() -> MembershipRevocationAnomalyAlert {
        MembershipRevocationAnomalyAlert {
            world_id: "w1".to_string(),
            node_id: "node-a".to_string(),
            detected_at_ms: 1000,
            severity: crate::membership_reconciliation::MembershipRevocationAlertSeverity::Warn,
            code: "reconcile_diverged".to_string(),
            message: "membership revocation reconcile diverged".to_string(),
            drained: 1,
            diverged: 1,
            rejected: 0,
        }
    }

    #[test]
    fn with_retry_failure_rejects_attempt_overflow() {
        let pending = MembershipRevocationPendingAlert {
            alert: sample_alert(),
            attempt: usize::MAX,
            next_retry_at_ms: 1000,
            last_error: None,
        };
        let err = pending
            .with_retry_failure(1000, 1, "transport failed".to_string())
            .expect_err("attempt overflow should fail");
        assert!(matches!(
            err,
            WorldError::DistributedValidationFailed { reason }
                if reason.contains("attempt overflow")
        ));
    }

    #[test]
    fn with_retry_failure_rejects_retry_timestamp_overflow() {
        let pending = MembershipRevocationPendingAlert::new(sample_alert(), 1000);
        let err = pending
            .with_retry_failure(i64::MAX, 1, "transport failed".to_string())
            .expect_err("retry timestamp overflow should fail");
        assert!(matches!(
            err,
            WorldError::DistributedValidationFailed { reason }
                if reason.contains("retry timestamp overflow")
        ));
    }
}
