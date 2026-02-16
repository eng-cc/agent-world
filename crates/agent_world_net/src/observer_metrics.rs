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

#[cfg(all(test, feature = "self_tests"))]
mod tests {
    use agent_world::runtime::World;

    use super::super::distributed::WorldHeadAnnounce;
    use super::super::observer::HeadSyncReport;
    use super::*;

    fn sample_head(height: u64) -> WorldHeadAnnounce {
        WorldHeadAnnounce {
            world_id: "w1".to_string(),
            height,
            block_hash: format!("block-{height}"),
            state_root: format!("state-{height}"),
            timestamp_ms: 1,
            signature: "sig".to_string(),
        }
    }

    fn build_mode_report(
        mode: HeadSyncSourceMode,
        applied: bool,
        fallback_used: bool,
    ) -> HeadSyncModeReport {
        let report = HeadSyncReport {
            drained: 1,
            applied: applied.then(|| super::super::observer::HeadSyncResult {
                head: sample_head(1),
                world: World::new(),
            }),
        };
        HeadSyncModeReport {
            mode,
            report,
            fallback_used,
        }
    }

    fn build_mode_with_dht_report(
        mode: HeadSyncSourceModeWithDht,
        applied: bool,
        fallback_used: bool,
    ) -> HeadSyncModeWithDhtReport {
        let report = HeadSyncReport {
            drained: 1,
            applied: applied.then(|| super::super::observer::HeadSyncResult {
                head: sample_head(2),
                world: World::new(),
            }),
        };
        HeadSyncModeWithDhtReport {
            mode,
            report,
            fallback_used,
        }
    }

    #[test]
    fn observer_runtime_metrics_records_mode_report_counters() {
        let mut metrics = ObserverRuntimeMetrics::new();

        metrics.record_mode_report(&build_mode_report(
            HeadSyncSourceMode::NetworkOnly,
            false,
            false,
        ));
        metrics.record_mode_report(&build_mode_report(
            HeadSyncSourceMode::PathIndexOnly,
            true,
            false,
        ));
        metrics.record_mode_report(&build_mode_report(
            HeadSyncSourceMode::NetworkThenPathIndex,
            true,
            true,
        ));

        let snapshot = metrics.snapshot();
        assert_eq!(
            snapshot.mode.network_only,
            ObserverModeCounters {
                total: 1,
                applied: 0,
                fallback: 0,
            }
        );
        assert_eq!(
            snapshot.mode.path_index_only,
            ObserverModeCounters {
                total: 1,
                applied: 1,
                fallback: 0,
            }
        );
        assert_eq!(
            snapshot.mode.network_then_path_index,
            ObserverModeCounters {
                total: 1,
                applied: 1,
                fallback: 1,
            }
        );
    }

    #[test]
    fn observer_runtime_metrics_records_dht_mode_counters_and_supports_reset() {
        let mut metrics = ObserverRuntimeMetrics::new();

        metrics.record_mode_with_dht_report(&build_mode_with_dht_report(
            HeadSyncSourceModeWithDht::NetworkWithDhtOnly,
            true,
            false,
        ));
        metrics.record_mode_with_dht_report(&build_mode_with_dht_report(
            HeadSyncSourceModeWithDht::PathIndexOnly,
            false,
            false,
        ));
        metrics.record_mode_with_dht_report(&build_mode_with_dht_report(
            HeadSyncSourceModeWithDht::NetworkWithDhtThenPathIndex,
            true,
            true,
        ));
        metrics.record_mode_with_dht_report(&build_mode_with_dht_report(
            HeadSyncSourceModeWithDht::NetworkWithDhtThenPathIndex,
            false,
            false,
        ));

        let snapshot = metrics.snapshot();
        assert_eq!(
            snapshot.mode_with_dht.network_with_dht_only,
            ObserverModeCounters {
                total: 1,
                applied: 1,
                fallback: 0,
            }
        );
        assert_eq!(
            snapshot.mode_with_dht.path_index_only,
            ObserverModeCounters {
                total: 1,
                applied: 0,
                fallback: 0,
            }
        );
        assert_eq!(
            snapshot.mode_with_dht.network_with_dht_then_path_index,
            ObserverModeCounters {
                total: 2,
                applied: 1,
                fallback: 1,
            }
        );

        metrics.reset();
        assert_eq!(
            metrics.snapshot(),
            ObserverRuntimeMetricsSnapshot::default()
        );
    }
}
