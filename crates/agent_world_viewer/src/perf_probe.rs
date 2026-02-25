use bevy::prelude::*;

use super::render_perf_summary::{infer_perf_hotspot, RenderPerfSummary};

const PERF_PROBE_ENV: &str = "AGENT_WORLD_VIEWER_PERF_PROBE";
const PERF_PROBE_INTERVAL_ENV: &str = "AGENT_WORLD_VIEWER_PERF_PROBE_INTERVAL_SECS";
const PERF_BUDGET_ENV: &str = "AGENT_WORLD_VIEWER_PERF_BUDGET_MS";

#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub(super) struct PerfProbeConfig {
    pub enabled: bool,
    pub interval_secs: f64,
    pub budget_ms: f32,
}

impl Default for PerfProbeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_secs: 1.0,
            budget_ms: 33.0,
        }
    }
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(super) struct PerfProbeState {
    elapsed_secs: f64,
    next_emit_secs: f64,
    samples: u64,
    over_budget_samples: u64,
}

pub(super) fn perf_probe_config_from_env() -> PerfProbeConfig {
    config_from_values(
        std::env::var(PERF_PROBE_ENV).ok(),
        std::env::var(PERF_PROBE_INTERVAL_ENV).ok(),
        std::env::var(PERF_BUDGET_ENV).ok(),
    )
}

pub(super) fn update_perf_probe(
    time: Option<Res<Time>>,
    config: Res<PerfProbeConfig>,
    summary: Res<RenderPerfSummary>,
    mut state: ResMut<PerfProbeState>,
) {
    if !config.enabled {
        return;
    }
    let Some(time) = time.as_deref() else {
        return;
    };

    let frame_ms = (time.delta_secs_f64() as f32) * 1000.0;
    if !frame_ms.is_finite() || frame_ms <= 0.0 {
        return;
    }

    state.elapsed_secs += time.delta_secs_f64();
    state.samples = state.samples.saturating_add(1);
    if frame_ms > config.budget_ms {
        state.over_budget_samples = state.over_budget_samples.saturating_add(1);
    }

    if state.elapsed_secs < state.next_emit_secs {
        return;
    }
    state.next_emit_secs = state.elapsed_secs + config.interval_secs.max(0.1);

    let over_budget_pct = if state.samples > 0 {
        state.over_budget_samples as f64 / state.samples as f64 * 100.0
    } else {
        0.0
    };
    let hotspot = infer_perf_hotspot(&summary);
    println!(
        "viewer perf_probe t={:.1}s avg={:.2} p95={:.2} over_budget_pct={:.2} event_window={} auto_degrade={} hotspot={} runtime_health={} runtime_bottleneck={} runtime_tick_p95={:.2} runtime_decision_p95={:.2} runtime_action_p95={:.2} runtime_callback_p95={:.2} runtime_llm_api_p95={:.2} label_capacity_hit={} overlay_capacity_hit={} event_backlog_hit={}",
        state.elapsed_secs,
        summary.frame_ms_avg,
        summary.frame_ms_p95,
        over_budget_pct,
        summary.event_window_size,
        summary.auto_degrade_active,
        hotspot.as_str(),
        summary.runtime_health.as_str(),
        summary.runtime_bottleneck.as_str(),
        summary.runtime_tick_p95_ms,
        summary.runtime_decision_p95_ms,
        summary.runtime_action_execution_p95_ms,
        summary.runtime_callback_p95_ms,
        summary.runtime_llm_api_p95_ms,
        summary.label_capacity_hit,
        summary.overlay_capacity_hit,
        summary.event_backlog_hit,
    );
}

fn config_from_values(
    enabled_raw: Option<String>,
    interval_raw: Option<String>,
    budget_raw: Option<String>,
) -> PerfProbeConfig {
    let mut config = PerfProbeConfig::default();
    if let Some(enabled) = parse_bool(enabled_raw.as_deref()) {
        config.enabled = enabled;
    }
    if let Some(interval) = interval_raw.and_then(|raw| raw.trim().parse::<f64>().ok()) {
        if interval.is_finite() && interval > 0.0 {
            config.interval_secs = interval;
        }
    }
    if let Some(budget) = budget_raw.and_then(|raw| raw.trim().parse::<f32>().ok()) {
        if budget.is_finite() && budget > 0.0 {
            config.budget_ms = budget;
        }
    }
    config
}

fn parse_bool(raw: Option<&str>) -> Option<bool> {
    let normalized = raw?.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_from_values_parses_and_defaults() {
        let default_config = config_from_values(None, None, None);
        assert!(!default_config.enabled);
        assert!((default_config.interval_secs - 1.0).abs() < f64::EPSILON);
        assert!((default_config.budget_ms - 33.0).abs() < f32::EPSILON);

        let config = config_from_values(
            Some("true".to_string()),
            Some("0.5".to_string()),
            Some("25.0".to_string()),
        );
        assert!(config.enabled);
        assert!((config.interval_secs - 0.5).abs() < f64::EPSILON);
        assert!((config.budget_ms - 25.0).abs() < f32::EPSILON);
    }

    #[test]
    fn config_from_values_ignores_invalid_values() {
        let config = config_from_values(
            Some("unknown".to_string()),
            Some("-3".to_string()),
            Some("nan".to_string()),
        );
        assert!(!config.enabled);
        assert!((config.interval_secs - 1.0).abs() < f64::EPSILON);
        assert!((config.budget_ms - 33.0).abs() < f32::EPSILON);
    }
}
