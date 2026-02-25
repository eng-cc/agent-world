use std::collections::VecDeque;

use agent_world::simulator::{RuntimePerfBottleneck, RuntimePerfHealth};
use bevy::prelude::*;

use super::{LabelLodStats, Viewer3dConfig, Viewer3dScene, ViewerConfig, ViewerState};

const PERF_SAMPLE_WINDOW: usize = 180;
const FRAME_BUDGET_MS: f32 = 33.0;
const EVENT_BACKLOG_PRESSURE_PCT: usize = 90;

#[derive(Resource, Clone, Debug, Default)]
pub(super) struct RenderPerfSummary {
    pub frame_ms_avg: f32,
    pub frame_ms_p95: f32,
    pub world_entities: usize,
    pub visible_labels: usize,
    pub overlay_entities: usize,
    pub event_window_size: usize,
    pub auto_degrade_active: bool,
    pub label_capacity_hit: bool,
    pub overlay_capacity_hit: bool,
    pub event_backlog_hit: bool,
    pub runtime_health: RuntimePerfHealth,
    pub runtime_bottleneck: RuntimePerfBottleneck,
    pub runtime_tick_p95_ms: f64,
    pub runtime_decision_p95_ms: f64,
    pub runtime_action_execution_p95_ms: f64,
    pub runtime_callback_p95_ms: f64,
    pub runtime_llm_api_p95_ms: f64,
    pub runtime_llm_api_budget_ms: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(super) enum PerfHotspot {
    #[default]
    None,
    RuntimeTick,
    RuntimeDecision,
    RuntimeActionExecution,
    RuntimeCallback,
    RuntimeLlmApi,
    OverlayCapacity,
    LabelCapacity,
    EventBacklog,
    RenderFrame,
}

impl PerfHotspot {
    pub(super) fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::RuntimeTick => "runtime_tick",
            Self::RuntimeDecision => "runtime_decision",
            Self::RuntimeActionExecution => "runtime_action_execution",
            Self::RuntimeCallback => "runtime_callback",
            Self::RuntimeLlmApi => "runtime_llm_api",
            Self::OverlayCapacity => "overlay_capacity",
            Self::LabelCapacity => "label_capacity",
            Self::EventBacklog => "event_backlog",
            Self::RenderFrame => "render_frame",
        }
    }
}

#[derive(Resource, Default)]
pub(super) struct RenderPerfHistory {
    frame_ms_samples: VecDeque<f32>,
}

pub(super) fn sample_render_perf_summary(
    time: Option<Res<Time>>,
    viewer_config: Res<ViewerConfig>,
    viewer_3d_config: Res<Viewer3dConfig>,
    scene: Res<Viewer3dScene>,
    state: Res<ViewerState>,
    label_stats: Option<Res<LabelLodStats>>,
    labels: Query<&Visibility, With<Text2d>>,
    mut history: ResMut<RenderPerfHistory>,
    mut summary: ResMut<RenderPerfSummary>,
) {
    sample_frame_time_window(time.as_deref(), &mut history);

    let frame_samples: Vec<f32> = history.frame_ms_samples.iter().copied().collect();
    summary.frame_ms_avg = mean_ms(&frame_samples);
    summary.frame_ms_p95 = percentile_ms(&frame_samples, 0.95);

    summary.world_entities = scene_world_entity_count(&scene);
    summary.overlay_entities =
        scene.heat_overlay_entities.len() + scene.flow_overlay_entities.len();
    summary.event_window_size = state.events.len();
    summary.event_backlog_hit =
        event_backlog_hit(summary.event_window_size, viewer_config.max_events);
    summary.visible_labels = labels
        .iter()
        .filter(|visibility| **visibility == Visibility::Visible)
        .count();
    apply_runtime_perf_summary(&mut summary, state.metrics.as_ref());

    let label_degraded = label_stats
        .as_deref()
        .map(|stats| (*stats).degraded())
        .unwrap_or(false);
    let label_capacity_hit =
        summary.visible_labels >= viewer_3d_config.label_lod.max_visible_labels;
    let overlay_capacity_hit = scene.heat_overlay_entities.len()
        >= viewer_3d_config.render_budget.overlay_max_heat_markers
        || scene.flow_overlay_entities.len()
            >= viewer_3d_config.render_budget.overlay_max_flow_segments;
    summary.label_capacity_hit = label_capacity_hit;
    summary.overlay_capacity_hit = overlay_capacity_hit;
    summary.auto_degrade_active = label_degraded || label_capacity_hit || overlay_capacity_hit;
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn sample_headless_perf_summary(
    time: Option<Res<Time>>,
    viewer_config: Res<ViewerConfig>,
    state: Res<ViewerState>,
    mut history: ResMut<RenderPerfHistory>,
    mut summary: ResMut<RenderPerfSummary>,
) {
    sample_frame_time_window(time.as_deref(), &mut history);
    let frame_samples: Vec<f32> = history.frame_ms_samples.iter().copied().collect();
    summary.frame_ms_avg = mean_ms(&frame_samples);
    summary.frame_ms_p95 = percentile_ms(&frame_samples, 0.95);
    summary.world_entities = state
        .snapshot
        .as_ref()
        .map(snapshot_world_entity_count)
        .unwrap_or(0);
    summary.visible_labels = 0;
    summary.overlay_entities = 0;
    summary.event_window_size = state.events.len();
    summary.label_capacity_hit = false;
    summary.overlay_capacity_hit = false;
    summary.event_backlog_hit =
        event_backlog_hit(summary.event_window_size, viewer_config.max_events);
    apply_runtime_perf_summary(&mut summary, state.metrics.as_ref());
    summary.auto_degrade_active = false;
}

pub(super) fn infer_perf_hotspot(summary: &RenderPerfSummary) -> PerfHotspot {
    if summary.runtime_llm_api_budget_ms > 0.0
        && summary.runtime_llm_api_p95_ms > summary.runtime_llm_api_budget_ms
    {
        return PerfHotspot::RuntimeLlmApi;
    }
    if runtime_health_is_elevated(summary.runtime_health) {
        if let Some(hotspot) = runtime_bottleneck_hotspot(summary.runtime_bottleneck) {
            return hotspot;
        }
    }
    if summary.overlay_capacity_hit {
        return PerfHotspot::OverlayCapacity;
    }
    if summary.label_capacity_hit {
        return PerfHotspot::LabelCapacity;
    }
    if summary.event_backlog_hit {
        return PerfHotspot::EventBacklog;
    }
    if summary.frame_ms_p95 > FRAME_BUDGET_MS || summary.auto_degrade_active {
        return PerfHotspot::RenderFrame;
    }
    PerfHotspot::None
}

fn sample_frame_time_window(time: Option<&Time>, history: &mut RenderPerfHistory) {
    if let Some(time) = time {
        let frame_ms = (time.delta_secs_f64() as f32) * 1000.0;
        if frame_ms.is_finite() && frame_ms > 0.0 {
            history.frame_ms_samples.push_back(frame_ms);
            while history.frame_ms_samples.len() > PERF_SAMPLE_WINDOW {
                history.frame_ms_samples.pop_front();
            }
        }
    }
}

fn mean_ms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.iter().sum::<f32>() / samples.len() as f32
}

fn percentile_ms(samples: &[f32], percentile: f32) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    let mut sorted = samples.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let index = ((sorted.len() - 1) as f32 * percentile.clamp(0.0, 1.0)).round() as usize;
    sorted[index]
}

fn scene_world_entity_count(scene: &Viewer3dScene) -> usize {
    scene.agent_entities.len()
        + scene.location_entities.len()
        + scene.asset_entities.len()
        + scene.module_visual_entities.len()
        + scene.power_plant_entities.len()
        + scene.power_storage_entities.len()
        + scene.chunk_entities.len()
        + scene
            .chunk_line_entities
            .values()
            .map(std::vec::Vec::len)
            .sum::<usize>()
        + scene.background_entities.len()
        + scene.heat_overlay_entities.len()
        + scene.flow_overlay_entities.len()
}

fn apply_runtime_perf_summary(
    summary: &mut RenderPerfSummary,
    metrics: Option<&agent_world::simulator::RunnerMetrics>,
) {
    let Some(metrics) = metrics else {
        summary.runtime_health = RuntimePerfHealth::Unknown;
        summary.runtime_bottleneck = RuntimePerfBottleneck::None;
        summary.runtime_tick_p95_ms = 0.0;
        summary.runtime_decision_p95_ms = 0.0;
        summary.runtime_action_execution_p95_ms = 0.0;
        summary.runtime_callback_p95_ms = 0.0;
        summary.runtime_llm_api_p95_ms = 0.0;
        summary.runtime_llm_api_budget_ms = 0.0;
        return;
    };
    let runtime = &metrics.runtime_perf;
    summary.runtime_health = runtime.health;
    summary.runtime_bottleneck = runtime.bottleneck;
    summary.runtime_tick_p95_ms = runtime.tick.p95_ms;
    summary.runtime_decision_p95_ms = runtime.decision.p95_ms;
    summary.runtime_action_execution_p95_ms = runtime.action_execution.p95_ms;
    summary.runtime_callback_p95_ms = runtime.callback.p95_ms;
    summary.runtime_llm_api_p95_ms = runtime.llm_api.p95_ms;
    summary.runtime_llm_api_budget_ms = runtime.llm_api.budget_ms;
}

fn runtime_health_is_elevated(health: RuntimePerfHealth) -> bool {
    matches!(
        health,
        RuntimePerfHealth::Warn | RuntimePerfHealth::Critical
    )
}

fn runtime_bottleneck_hotspot(bottleneck: RuntimePerfBottleneck) -> Option<PerfHotspot> {
    match bottleneck {
        RuntimePerfBottleneck::None => None,
        RuntimePerfBottleneck::Tick => Some(PerfHotspot::RuntimeTick),
        RuntimePerfBottleneck::Decision => Some(PerfHotspot::RuntimeDecision),
        RuntimePerfBottleneck::ActionExecution => Some(PerfHotspot::RuntimeActionExecution),
        RuntimePerfBottleneck::Callback => Some(PerfHotspot::RuntimeCallback),
    }
}

fn event_backlog_hit(event_window_size: usize, max_events: usize) -> bool {
    if max_events == 0 {
        return false;
    }
    event_window_size.saturating_mul(100) >= max_events.saturating_mul(EVENT_BACKLOG_PRESSURE_PCT)
}

#[cfg(not(target_arch = "wasm32"))]
fn snapshot_world_entity_count(snapshot: &agent_world::simulator::WorldSnapshot) -> usize {
    let model = &snapshot.model;
    model.locations.len()
        + model.agents.len()
        + model.assets.len()
        + model.module_visual_entities.len()
        + model.power_plants.len()
        + model.power_storages.len()
        + model.chunks.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_ms_returns_zero_for_empty_samples() {
        assert_eq!(percentile_ms(&[], 0.95), 0.0);
        assert_eq!(mean_ms(&[]), 0.0);
    }

    #[test]
    fn percentile_ms_uses_sorted_percentile_value() {
        let samples = vec![22.0, 18.0, 30.0, 16.0, 24.0];
        let p95 = percentile_ms(&samples, 0.95);
        assert_eq!(p95, 30.0);
        assert!((mean_ms(&samples) - 22.0).abs() < f32::EPSILON);
    }

    #[test]
    fn scene_world_entity_count_includes_overlay_and_chunk_lines() {
        let mut scene = Viewer3dScene::default();
        scene
            .agent_entities
            .insert("a".to_string(), Entity::from_bits(1));
        scene
            .location_entities
            .insert("l".to_string(), Entity::from_bits(2));
        scene
            .chunk_entities
            .insert("c".to_string(), Entity::from_bits(3));
        scene.chunk_line_entities.insert(
            "c".to_string(),
            vec![Entity::from_bits(4), Entity::from_bits(5)],
        );
        scene.background_entities.push(Entity::from_bits(6));
        scene.heat_overlay_entities.push(Entity::from_bits(7));
        scene.flow_overlay_entities.push(Entity::from_bits(8));

        assert_eq!(scene_world_entity_count(&scene), 8);
    }

    #[test]
    fn infer_perf_hotspot_prefers_runtime_llm_api_over_budget() {
        let summary = RenderPerfSummary {
            runtime_llm_api_p95_ms: 1200.0,
            runtime_llm_api_budget_ms: 1000.0,
            runtime_health: RuntimePerfHealth::Warn,
            runtime_bottleneck: RuntimePerfBottleneck::ActionExecution,
            frame_ms_p95: 55.0,
            ..RenderPerfSummary::default()
        };

        assert_eq!(infer_perf_hotspot(&summary), PerfHotspot::RuntimeLlmApi);
    }

    #[test]
    fn infer_perf_hotspot_prefers_runtime_bottleneck_when_runtime_health_elevated() {
        let summary = RenderPerfSummary {
            runtime_health: RuntimePerfHealth::Critical,
            runtime_bottleneck: RuntimePerfBottleneck::Decision,
            frame_ms_p95: 60.0,
            ..RenderPerfSummary::default()
        };

        assert_eq!(infer_perf_hotspot(&summary), PerfHotspot::RuntimeDecision);
    }

    #[test]
    fn infer_perf_hotspot_uses_capacity_then_frame_pressure() {
        let overlay = RenderPerfSummary {
            overlay_capacity_hit: true,
            frame_ms_p95: 50.0,
            ..RenderPerfSummary::default()
        };
        assert_eq!(infer_perf_hotspot(&overlay), PerfHotspot::OverlayCapacity);

        let label = RenderPerfSummary {
            label_capacity_hit: true,
            frame_ms_p95: 50.0,
            ..RenderPerfSummary::default()
        };
        assert_eq!(infer_perf_hotspot(&label), PerfHotspot::LabelCapacity);

        let frame = RenderPerfSummary {
            frame_ms_p95: 40.0,
            ..RenderPerfSummary::default()
        };
        assert_eq!(infer_perf_hotspot(&frame), PerfHotspot::RenderFrame);
    }

    #[test]
    fn event_backlog_hit_uses_ninety_percent_threshold() {
        assert!(!event_backlog_hit(89, 100));
        assert!(event_backlog_hit(90, 100));
    }
}
