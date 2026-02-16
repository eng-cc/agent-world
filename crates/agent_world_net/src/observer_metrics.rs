use super::observer::{
    HeadSyncModeReport, HeadSyncModeWithDhtReport, HeadSyncSourceMode, HeadSyncSourceModeWithDht,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ObserverModeCounters {
    pub total: u64,
    pub applied: u64,
    pub fallback: u64,
}

impl ObserverModeCounters {
    fn record(&mut self, applied: bool, fallback_used: bool) {
        self.total += 1;
        if applied {
            self.applied += 1;
        }
        if fallback_used {
            self.fallback += 1;
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObserverModeRuntimeMetricsSnapshot {
    pub network_only: ObserverModeCounters,
    pub path_index_only: ObserverModeCounters,
    pub network_then_path_index: ObserverModeCounters,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObserverModeWithDhtRuntimeMetricsSnapshot {
    pub network_with_dht_only: ObserverModeCounters,
    pub path_index_only: ObserverModeCounters,
    pub network_with_dht_then_path_index: ObserverModeCounters,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObserverRuntimeMetricsSnapshot {
    pub mode: ObserverModeRuntimeMetricsSnapshot,
    pub mode_with_dht: ObserverModeWithDhtRuntimeMetricsSnapshot,
}

#[derive(Debug, Clone, Default)]
pub struct ObserverRuntimeMetrics {
    snapshot: ObserverRuntimeMetricsSnapshot,
}

impl ObserverRuntimeMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_mode_report(&mut self, report: &HeadSyncModeReport) {
        let counters = self.select_mode_counters_mut(report.mode);
        counters.record(report.report.applied.is_some(), report.fallback_used);
    }

    pub fn record_mode_with_dht_report(&mut self, report: &HeadSyncModeWithDhtReport) {
        let counters = self.select_mode_with_dht_counters_mut(report.mode);
        counters.record(report.report.applied.is_some(), report.fallback_used);
    }

    pub fn snapshot(&self) -> ObserverRuntimeMetricsSnapshot {
        self.snapshot.clone()
    }

    pub fn reset(&mut self) {
        self.snapshot = ObserverRuntimeMetricsSnapshot::default();
    }

    fn select_mode_counters_mut(&mut self, mode: HeadSyncSourceMode) -> &mut ObserverModeCounters {
        match mode {
            HeadSyncSourceMode::NetworkOnly => &mut self.snapshot.mode.network_only,
            HeadSyncSourceMode::PathIndexOnly => &mut self.snapshot.mode.path_index_only,
            HeadSyncSourceMode::NetworkThenPathIndex => {
                &mut self.snapshot.mode.network_then_path_index
            }
        }
    }

    fn select_mode_with_dht_counters_mut(
        &mut self,
        mode: HeadSyncSourceModeWithDht,
    ) -> &mut ObserverModeCounters {
        match mode {
            HeadSyncSourceModeWithDht::NetworkWithDhtOnly => {
                &mut self.snapshot.mode_with_dht.network_with_dht_only
            }
            HeadSyncSourceModeWithDht::PathIndexOnly => {
                &mut self.snapshot.mode_with_dht.path_index_only
            }
            HeadSyncSourceModeWithDht::NetworkWithDhtThenPathIndex => {
                &mut self.snapshot.mode_with_dht.network_with_dht_then_path_index
            }
        }
    }
}
