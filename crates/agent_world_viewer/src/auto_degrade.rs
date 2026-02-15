use bevy::prelude::*;

use super::render_perf_summary::RenderPerfSummary;
use super::viewer_3d_config::Viewer3dConfig;
use super::world_overlay::WorldOverlayConfig;

const AUTO_DEGRADE_ENV: &str = "AGENT_WORLD_VIEWER_AUTO_DEGRADE";
const HIGH_P95_FRAME_MS: f32 = 33.0;
const LOW_P95_FRAME_MS: f32 = 20.0;
const ESCALATE_STREAK: u32 = 8;
const RECOVER_STREAK: u32 = 24;
const MAX_DEGRADE_LEVEL: u8 = 3;

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AutoDegradeConfig {
    pub enabled: bool,
}

impl Default for AutoDegradeConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AutoDegradeState {
    level: u8,
    high_pressure_streak: u32,
    low_pressure_streak: u32,
    baseline_max_visible_labels: usize,
    baseline_show_heat_overlay: bool,
    baseline_show_flow_overlay: bool,
}

impl Default for AutoDegradeState {
    fn default() -> Self {
        Self {
            level: 0,
            high_pressure_streak: 0,
            low_pressure_streak: 0,
            baseline_max_visible_labels: 1,
            baseline_show_heat_overlay: true,
            baseline_show_flow_overlay: true,
        }
    }
}

impl AutoDegradeState {
    pub(super) fn level(&self) -> u8 {
        self.level
    }
}

pub(super) fn auto_degrade_config_from_env() -> AutoDegradeConfig {
    let mut config = AutoDegradeConfig::default();
    if let Some(enabled) = parse_bool_env(AUTO_DEGRADE_ENV) {
        config.enabled = enabled;
    }
    config
}

pub(super) fn update_auto_degrade_policy(
    config: Res<AutoDegradeConfig>,
    mut summary: ResMut<RenderPerfSummary>,
    mut viewer_3d: ResMut<Viewer3dConfig>,
    mut overlay: ResMut<WorldOverlayConfig>,
    mut state: ResMut<AutoDegradeState>,
) {
    if !config.enabled {
        state.level = 0;
        state.high_pressure_streak = 0;
        state.low_pressure_streak = 0;
        summary.auto_degrade_active = false;
        return;
    }

    run_auto_degrade_policy(
        &summary,
        &mut viewer_3d,
        &mut overlay,
        &mut state,
        AutoDegradeTuning::default(),
    );
    summary.auto_degrade_active = summary.auto_degrade_active || state.level() > 0;
}

#[derive(Clone, Copy)]
struct AutoDegradeTuning {
    high_p95_frame_ms: f32,
    low_p95_frame_ms: f32,
    escalate_streak: u32,
    recover_streak: u32,
}

impl Default for AutoDegradeTuning {
    fn default() -> Self {
        Self {
            high_p95_frame_ms: HIGH_P95_FRAME_MS,
            low_p95_frame_ms: LOW_P95_FRAME_MS,
            escalate_streak: ESCALATE_STREAK,
            recover_streak: RECOVER_STREAK,
        }
    }
}

fn run_auto_degrade_policy(
    summary: &RenderPerfSummary,
    viewer_3d: &mut Viewer3dConfig,
    overlay: &mut WorldOverlayConfig,
    state: &mut AutoDegradeState,
    tuning: AutoDegradeTuning,
) {
    if state.level == 0 {
        state.baseline_max_visible_labels = viewer_3d.label_lod.max_visible_labels.max(1);
        state.baseline_show_heat_overlay = overlay.show_resource_heatmap;
        state.baseline_show_flow_overlay = overlay.show_flow_overlay;
    }

    let high_pressure =
        summary.frame_ms_p95 >= tuning.high_p95_frame_ms || summary.auto_degrade_active;
    let low_pressure =
        summary.frame_ms_p95 <= tuning.low_p95_frame_ms && !summary.auto_degrade_active;

    if high_pressure {
        state.high_pressure_streak = state.high_pressure_streak.saturating_add(1);
        state.low_pressure_streak = 0;
    } else if low_pressure {
        state.low_pressure_streak = state.low_pressure_streak.saturating_add(1);
        state.high_pressure_streak = 0;
    } else {
        state.high_pressure_streak = 0;
        state.low_pressure_streak = 0;
    }

    let mut next_level = state.level;
    if state.high_pressure_streak >= tuning.escalate_streak {
        next_level = next_level.saturating_add(1).min(MAX_DEGRADE_LEVEL);
        state.high_pressure_streak = 0;
    }
    if state.low_pressure_streak >= tuning.recover_streak && next_level > 0 {
        next_level -= 1;
        state.low_pressure_streak = 0;
    }

    if next_level != state.level {
        state.level = next_level;
    }
    apply_degrade_level(viewer_3d, overlay, state);
}

fn apply_degrade_level(
    viewer_3d: &mut Viewer3dConfig,
    overlay: &mut WorldOverlayConfig,
    state: &AutoDegradeState,
) {
    let baseline_labels = state.baseline_max_visible_labels.max(1);
    let (label_cap, show_heat, show_flow) = match state.level {
        0 => (
            baseline_labels,
            state.baseline_show_heat_overlay,
            state.baseline_show_flow_overlay,
        ),
        1 => (
            reduced_label_cap(baseline_labels, 2, 12),
            state.baseline_show_heat_overlay,
            state.baseline_show_flow_overlay,
        ),
        2 => (
            reduced_label_cap(baseline_labels, 3, 8),
            false,
            state.baseline_show_flow_overlay,
        ),
        _ => (reduced_label_cap(baseline_labels, 4, 6), false, false),
    };

    viewer_3d.label_lod.max_visible_labels = label_cap;
    overlay.show_resource_heatmap = show_heat;
    overlay.show_flow_overlay = show_flow;
}

fn reduced_label_cap(baseline: usize, divisor: usize, floor: usize) -> usize {
    (baseline.max(1) / divisor.max(1)).max(floor.max(1))
}

fn parse_bool_env(key: &str) -> Option<bool> {
    std::env::var(key).ok().and_then(|raw| {
        let normalized = raw.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hot_summary() -> RenderPerfSummary {
        RenderPerfSummary {
            frame_ms_avg: 36.0,
            frame_ms_p95: 38.0,
            world_entities: 900,
            visible_labels: 80,
            overlay_entities: 120,
            event_window_size: 100,
            auto_degrade_active: true,
        }
    }

    fn cool_summary() -> RenderPerfSummary {
        RenderPerfSummary {
            frame_ms_avg: 14.0,
            frame_ms_p95: 16.0,
            world_entities: 200,
            visible_labels: 16,
            overlay_entities: 20,
            event_window_size: 24,
            auto_degrade_active: false,
        }
    }

    #[test]
    fn auto_degrade_escalates_then_recovers_with_hysteresis() {
        let mut viewer_3d = Viewer3dConfig::default();
        let mut overlay = WorldOverlayConfig::default();
        let mut state = AutoDegradeState::default();
        let tuning = AutoDegradeTuning {
            escalate_streak: 2,
            recover_streak: 3,
            ..AutoDegradeTuning::default()
        };

        run_auto_degrade_policy(
            &hot_summary(),
            &mut viewer_3d,
            &mut overlay,
            &mut state,
            tuning,
        );
        run_auto_degrade_policy(
            &hot_summary(),
            &mut viewer_3d,
            &mut overlay,
            &mut state,
            tuning,
        );
        assert_eq!(state.level(), 1);
        assert!(viewer_3d.label_lod.max_visible_labels <= 24);

        run_auto_degrade_policy(
            &hot_summary(),
            &mut viewer_3d,
            &mut overlay,
            &mut state,
            tuning,
        );
        run_auto_degrade_policy(
            &hot_summary(),
            &mut viewer_3d,
            &mut overlay,
            &mut state,
            tuning,
        );
        assert_eq!(state.level(), 2);
        assert!(!overlay.show_resource_heatmap);
        assert!(overlay.show_flow_overlay);

        for _ in 0..6 {
            run_auto_degrade_policy(
                &cool_summary(),
                &mut viewer_3d,
                &mut overlay,
                &mut state,
                tuning,
            );
        }
        assert!(state.level() <= 1);
    }

    #[test]
    fn auto_degrade_restores_baseline_at_level_zero() {
        let mut viewer_3d = Viewer3dConfig::default();
        viewer_3d.label_lod.max_visible_labels = 64;
        let mut overlay = WorldOverlayConfig {
            show_chunk_overlay: true,
            show_resource_heatmap: false,
            show_flow_overlay: true,
        };
        let mut state = AutoDegradeState::default();
        let tuning = AutoDegradeTuning {
            escalate_streak: 1,
            recover_streak: 1,
            ..AutoDegradeTuning::default()
        };

        run_auto_degrade_policy(
            &hot_summary(),
            &mut viewer_3d,
            &mut overlay,
            &mut state,
            tuning,
        );
        assert_eq!(state.level(), 1);

        run_auto_degrade_policy(
            &cool_summary(),
            &mut viewer_3d,
            &mut overlay,
            &mut state,
            tuning,
        );
        assert_eq!(state.level(), 0);
        assert_eq!(viewer_3d.label_lod.max_visible_labels, 64);
        assert!(!overlay.show_resource_heatmap);
        assert!(overlay.show_flow_overlay);
    }
}
