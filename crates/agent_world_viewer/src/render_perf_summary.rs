use std::collections::VecDeque;

use bevy::prelude::*;

use super::{LabelLodStats, Viewer3dConfig, Viewer3dScene, ViewerState};

const PERF_SAMPLE_WINDOW: usize = 180;

#[derive(Resource, Clone, Debug, Default)]
pub(super) struct RenderPerfSummary {
    pub frame_ms_avg: f32,
    pub frame_ms_p95: f32,
    pub world_entities: usize,
    pub visible_labels: usize,
    pub overlay_entities: usize,
    pub event_window_size: usize,
    pub auto_degrade_active: bool,
}

#[derive(Resource, Default)]
pub(super) struct RenderPerfHistory {
    frame_ms_samples: VecDeque<f32>,
}

pub(super) fn sample_render_perf_summary(
    time: Option<Res<Time>>,
    config: Res<Viewer3dConfig>,
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
    summary.visible_labels = labels
        .iter()
        .filter(|visibility| **visibility == Visibility::Visible)
        .count();

    let label_degraded = label_stats
        .as_deref()
        .map(|stats| (*stats).degraded())
        .unwrap_or(false);
    let label_capacity_hit = summary.visible_labels >= config.label_lod.max_visible_labels;
    let overlay_capacity_hit = scene.heat_overlay_entities.len()
        >= config.render_budget.overlay_max_heat_markers
        || scene.flow_overlay_entities.len() >= config.render_budget.overlay_max_flow_segments;
    summary.auto_degrade_active = label_degraded || label_capacity_hit || overlay_capacity_hit;
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn sample_headless_perf_summary(
    time: Option<Res<Time>>,
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
    summary.auto_degrade_active = false;
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
}
