use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

use super::camera_controls::orbit_min_radius;
use super::selection_linking::apply_selection;
use super::*;

const AUTOMATION_STEPS_ENV: &str = "AGENT_WORLD_VIEWER_AUTOMATION_STEPS";
const AUTO_SELECT_ENV: &str = "AGENT_WORLD_VIEWER_AUTO_SELECT";
const AUTO_SELECT_TARGET_ENV: &str = "AGENT_WORLD_VIEWER_AUTO_SELECT_TARGET";

#[derive(Resource, Clone, Debug, PartialEq)]
pub(super) struct ViewerAutomationConfig {
    pub enabled: bool,
    pub steps: Vec<ViewerAutomationStep>,
}

impl Default for ViewerAutomationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            steps: Vec::new(),
        }
    }
}

#[derive(Resource, Default, Clone, Debug)]
pub(super) struct ViewerAutomationState {
    startup_step_index: usize,
    wait_until_secs: Option<f64>,
    runtime_steps: VecDeque<ViewerAutomationStep>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum ViewerAutomationStep {
    WaitSeconds(f64),
    SetMode(ViewerCameraMode),
    Focus(ViewerAutomationTarget),
    Pan(Vec3),
    ZoomFactor(f32),
    OrbitDeg { yaw: f32, pitch: f32 },
    Select(ViewerAutomationTarget),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ViewerAutomationTarget {
    FirstKind(&'static str),
    KindId { kind: &'static str, id: String },
}

const TARGET_KIND_AGENT: &str = "agent";
const TARGET_KIND_LOCATION: &str = "location";
const TARGET_KIND_ASSET: &str = "asset";
const TARGET_KIND_MODULE_VISUAL: &str = "module_visual";
const TARGET_KIND_POWER_PLANT: &str = "power_plant";
const TARGET_KIND_POWER_STORAGE: &str = "power_storage";
const TARGET_KIND_CHUNK: &str = "chunk";
const TARGET_KIND_FRAGMENT: &str = "fragment";

struct TargetKindSpec<'a> {
    selection_kind: SelectionKind,
    entities: &'a HashMap<String, Entity>,
    first_filter: fn(&str) -> bool,
}

enum StepResult {
    Applied,
    AppliedYield,
    Pending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StepSource {
    StartupConfig,
    RuntimeQueue,
}

pub(super) fn viewer_automation_config_from_env() -> ViewerAutomationConfig {
    config_from_values(
        std::env::var(AUTO_SELECT_ENV).ok(),
        std::env::var(AUTO_SELECT_TARGET_ENV).ok(),
        std::env::var(AUTOMATION_STEPS_ENV).ok(),
    )
}

pub(super) fn run_viewer_automation(
    time: Res<Time>,
    config: Res<ViewerAutomationConfig>,
    mut state: ResMut<ViewerAutomationState>,
    mut camera_mode: ResMut<ViewerCameraMode>,
    viewer_config: Res<Viewer3dConfig>,
    scene: Res<Viewer3dScene>,
    mut selection: ResMut<ViewerSelection>,
    mut queries: ParamSet<(
        Query<(&mut OrbitCamera, &mut Transform, &mut Projection), With<Viewer3dCamera>>,
        Query<(&mut Transform, Option<&BaseScale>)>,
        Query<&Transform>,
    )>,
    mut location_markers: Query<&LocationMarker>,
) {
    let now = time.elapsed_secs_f64();
    if let Some(wait_until_secs) = state.wait_until_secs {
        if now < wait_until_secs {
            return;
        }
        state.wait_until_secs = None;
    }

    loop {
        let Some((source, step)) = next_step(&config, &state) else {
            return;
        };

        let result = apply_step(
            step,
            now,
            &scene,
            &viewer_config,
            &mut camera_mode,
            &mut selection,
            &mut queries,
            &mut location_markers,
            &mut state,
        );
        match result {
            StepResult::Applied => {
                advance_step(&mut state, source);
                continue;
            }
            StepResult::AppliedYield => {
                advance_step(&mut state, source);
                return;
            }
            StepResult::Pending => return,
        }
    }
}

fn next_step(
    config: &ViewerAutomationConfig,
    state: &ViewerAutomationState,
) -> Option<(StepSource, ViewerAutomationStep)> {
    if let Some(step) = state.runtime_steps.front().cloned() {
        return Some((StepSource::RuntimeQueue, step));
    }
    if !config.enabled {
        return None;
    }
    config
        .steps
        .get(state.startup_step_index)
        .cloned()
        .map(|step| (StepSource::StartupConfig, step))
}

fn advance_step(state: &mut ViewerAutomationState, source: StepSource) {
    match source {
        StepSource::StartupConfig => {
            state.startup_step_index += 1;
        }
        StepSource::RuntimeQueue => {
            let _ = state.runtime_steps.pop_front();
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub(super) fn enqueue_runtime_steps(
    state: &mut ViewerAutomationState,
    steps: impl IntoIterator<Item = ViewerAutomationStep>,
) {
    state.runtime_steps.extend(steps);
}

#[cfg(target_arch = "wasm32")]
pub(super) fn parse_automation_steps(raw: &str) -> Vec<ViewerAutomationStep> {
    parse_steps(Some(raw))
}

#[cfg(target_arch = "wasm32")]
pub(super) fn parse_automation_mode(raw: &str) -> Option<ViewerCameraMode> {
    parse_mode(raw)
}

#[cfg(target_arch = "wasm32")]
pub(super) fn parse_automation_target(raw: &str) -> Option<ViewerAutomationTarget> {
    parse_target(raw)
}

fn apply_step(
    step: ViewerAutomationStep,
    now: f64,
    scene: &Viewer3dScene,
    viewer_config: &Viewer3dConfig,
    camera_mode: &mut ViewerCameraMode,
    selection: &mut ViewerSelection,
    queries: &mut ParamSet<(
        Query<(&mut OrbitCamera, &mut Transform, &mut Projection), With<Viewer3dCamera>>,
        Query<(&mut Transform, Option<&BaseScale>)>,
        Query<&Transform>,
    )>,
    location_markers: &mut Query<&LocationMarker>,
    state: &mut ViewerAutomationState,
) -> StepResult {
    match step {
        ViewerAutomationStep::WaitSeconds(seconds) => {
            state.wait_until_secs = Some(now + seconds.max(0.0));
            StepResult::AppliedYield
        }
        ViewerAutomationStep::SetMode(mode) => {
            if *camera_mode != mode {
                *camera_mode = mode;
                StepResult::AppliedYield
            } else {
                StepResult::Applied
            }
        }
        ViewerAutomationStep::Focus(target) => {
            let Some((entity, _, _)) = resolve_target_entity(scene, &target) else {
                return StepResult::Pending;
            };

            let target_translation = {
                let transform_query = queries.p2();
                let Ok(target_transform) = transform_query.get(entity) else {
                    return StepResult::Pending;
                };
                target_transform.translation
            };

            let mut camera_query = queries.p0();
            let Ok((mut orbit, mut camera_transform, _)) = camera_query.single_mut() else {
                return StepResult::Pending;
            };
            orbit.focus = target_translation;
            orbit.apply_to_transform(&mut camera_transform);
            StepResult::Applied
        }
        ViewerAutomationStep::Pan(delta) => {
            let mut camera_query = queries.p0();
            let Ok((mut orbit, mut camera_transform, _)) = camera_query.single_mut() else {
                return StepResult::Pending;
            };
            orbit.focus += delta;
            orbit.apply_to_transform(&mut camera_transform);
            StepResult::Applied
        }
        ViewerAutomationStep::ZoomFactor(factor) => {
            let mut camera_query = queries.p0();
            let Ok((mut orbit, mut camera_transform, mut projection)) = camera_query.single_mut()
            else {
                return StepResult::Pending;
            };

            let min_radius = orbit_min_radius(viewer_config.effective_cm_to_unit());
            orbit.radius = (orbit.radius * factor.max(0.01)).clamp(min_radius, ORBIT_MAX_RADIUS);
            if *camera_mode == ViewerCameraMode::TwoD {
                if let Projection::Orthographic(ortho) = &mut *projection {
                    ortho.scale =
                        (ortho.scale * factor.max(0.01)).clamp(ORTHO_MIN_SCALE, ORTHO_MAX_SCALE);
                } else {
                    *projection = camera_projection_for_mode(ViewerCameraMode::TwoD, viewer_config);
                }
            }
            orbit.apply_to_transform(&mut camera_transform);
            StepResult::Applied
        }
        ViewerAutomationStep::OrbitDeg { yaw, pitch } => {
            if *camera_mode != ViewerCameraMode::ThreeD {
                return StepResult::Applied;
            }
            let mut camera_query = queries.p0();
            let Ok((mut orbit, mut camera_transform, _)) = camera_query.single_mut() else {
                return StepResult::Pending;
            };
            orbit.yaw += yaw.to_radians();
            orbit.pitch = (orbit.pitch + pitch.to_radians()).clamp(-1.54, 1.54);
            orbit.apply_to_transform(&mut camera_transform);
            StepResult::Applied
        }
        ViewerAutomationStep::Select(target) => {
            let Some((entity, kind, id)) = resolve_target_entity(scene, &target) else {
                return StepResult::Pending;
            };
            let name = if kind == SelectionKind::Location {
                location_markers
                    .get(entity)
                    .ok()
                    .map(|marker| marker.name.clone())
            } else {
                None
            };

            apply_selection(
                selection,
                &mut queries.p1(),
                viewer_config,
                entity,
                kind,
                id,
                name,
            );
            StepResult::Applied
        }
    }
}

fn resolve_target_entity(
    scene: &Viewer3dScene,
    target: &ViewerAutomationTarget,
) -> Option<(Entity, SelectionKind, String)> {
    match target {
        ViewerAutomationTarget::FirstKind(kind) => {
            let spec = target_kind_spec(scene, kind)?;
            let id = first_sorted_matching(spec.entities, spec.first_filter)
                .or_else(|| first_sorted_id(spec.entities))?;
            let entity = spec.entities.get(id.as_str()).copied()?;
            Some((entity, spec.selection_kind, id))
        }
        ViewerAutomationTarget::KindId { kind, id } => {
            let spec = target_kind_spec(scene, kind)?;
            spec.entities
                .get(id.as_str())
                .copied()
                .map(|entity| (entity, spec.selection_kind, id.clone()))
        }
    }
}

fn target_kind_spec<'a>(scene: &'a Viewer3dScene, kind: &str) -> Option<TargetKindSpec<'a>> {
    match kind {
        TARGET_KIND_AGENT => Some(TargetKindSpec {
            selection_kind: SelectionKind::Agent,
            entities: &scene.agent_entities,
            first_filter: always_true,
        }),
        TARGET_KIND_LOCATION => Some(TargetKindSpec {
            selection_kind: SelectionKind::Location,
            entities: &scene.location_entities,
            first_filter: always_true,
        }),
        TARGET_KIND_FRAGMENT => Some(TargetKindSpec {
            selection_kind: SelectionKind::Fragment,
            entities: &scene.location_entities,
            first_filter: is_fragment_id,
        }),
        TARGET_KIND_ASSET => Some(TargetKindSpec {
            selection_kind: SelectionKind::Asset,
            entities: &scene.asset_entities,
            first_filter: always_true,
        }),
        TARGET_KIND_MODULE_VISUAL => Some(TargetKindSpec {
            selection_kind: SelectionKind::Asset,
            entities: &scene.module_visual_entities,
            first_filter: always_true,
        }),
        TARGET_KIND_POWER_PLANT => Some(TargetKindSpec {
            selection_kind: SelectionKind::PowerPlant,
            entities: &scene.power_plant_entities,
            first_filter: always_true,
        }),
        TARGET_KIND_POWER_STORAGE => Some(TargetKindSpec {
            selection_kind: SelectionKind::PowerStorage,
            entities: &scene.power_storage_entities,
            first_filter: always_true,
        }),
        TARGET_KIND_CHUNK => Some(TargetKindSpec {
            selection_kind: SelectionKind::Chunk,
            entities: &scene.chunk_entities,
            first_filter: always_true,
        }),
        _ => None,
    }
}

fn first_sorted_id(items: &HashMap<String, Entity>) -> Option<String> {
    let mut ids: Vec<_> = items.keys().cloned().collect();
    ids.sort();
    ids.into_iter().next()
}

fn first_sorted_matching(
    items: &HashMap<String, Entity>,
    predicate: fn(&str) -> bool,
) -> Option<String> {
    let mut ids: Vec<_> = items
        .keys()
        .filter(|id| predicate(id.as_str()))
        .cloned()
        .collect();
    ids.sort();
    ids.into_iter().next()
}

fn always_true(_id: &str) -> bool {
    true
}

fn is_fragment_id(id: &str) -> bool {
    id.starts_with("frag-")
}

fn config_from_values(
    auto_select: Option<String>,
    auto_select_target: Option<String>,
    automation_steps: Option<String>,
) -> ViewerAutomationConfig {
    let steps = parse_steps(automation_steps.as_deref());
    if !steps.is_empty() {
        return ViewerAutomationConfig {
            enabled: true,
            steps,
        };
    }

    let target = auto_select_target
        .as_deref()
        .and_then(parse_target)
        .or_else(|| auto_select.as_deref().and_then(parse_target));
    let auto_select_enabled = auto_select
        .as_deref()
        .map(parse_truthy)
        .unwrap_or(auto_select_target.is_some());
    if auto_select_enabled {
        if let Some(target) = target {
            return ViewerAutomationConfig {
                enabled: true,
                steps: vec![ViewerAutomationStep::Select(target)],
            };
        }
    }

    ViewerAutomationConfig::default()
}

fn parse_steps(raw: Option<&str>) -> Vec<ViewerAutomationStep> {
    let mut steps = Vec::new();
    let Some(raw) = raw else {
        return steps;
    };

    for segment in raw.split(';') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        let Some((key, value)) = segment.split_once('=') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        let parsed = match key.as_str() {
            "wait" => value
                .parse::<f64>()
                .ok()
                .map(ViewerAutomationStep::WaitSeconds),
            "mode" => parse_mode(value).map(ViewerAutomationStep::SetMode),
            "focus" => parse_target(value).map(ViewerAutomationStep::Focus),
            "pan" => parse_vec3(value).map(ViewerAutomationStep::Pan),
            "zoom" => value
                .parse::<f32>()
                .ok()
                .map(ViewerAutomationStep::ZoomFactor),
            "orbit" => parse_orbit(value),
            "select" => parse_target(value).map(ViewerAutomationStep::Select),
            _ => None,
        };
        if let Some(step) = parsed {
            steps.push(step);
        }
    }
    steps
}

fn parse_mode(raw: &str) -> Option<ViewerCameraMode> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "2d" | "two_d" | "twod" => Some(ViewerCameraMode::TwoD),
        "3d" | "three_d" | "threed" => Some(ViewerCameraMode::ThreeD),
        _ => None,
    }
}

fn parse_target(raw: &str) -> Option<ViewerAutomationTarget> {
    let value = raw.trim();
    if value.is_empty() {
        return None;
    }

    let normalized = value.to_ascii_lowercase();
    if let Some(kind_token) = normalized.strip_prefix("first_") {
        let kind = canonical_target_kind(kind_token)?;
        return Some(ViewerAutomationTarget::FirstKind(kind));
    }
    if let Some(kind_token) = normalized.strip_prefix("first:") {
        let kind = canonical_target_kind(kind_token)?;
        return Some(ViewerAutomationTarget::FirstKind(kind));
    }

    let (kind_token, id) = value.split_once(':')?;
    let kind = canonical_target_kind(kind_token)?;
    let id = id.trim();
    if id.is_empty() {
        return None;
    }
    Some(ViewerAutomationTarget::KindId {
        kind,
        id: id.to_string(),
    })
}

fn canonical_target_kind(raw: &str) -> Option<&'static str> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "agent" => Some(TARGET_KIND_AGENT),
        "location" | "loc" => Some(TARGET_KIND_LOCATION),
        "fragment" | "frag" => Some(TARGET_KIND_FRAGMENT),
        "asset" => Some(TARGET_KIND_ASSET),
        "module_visual" | "module-visual" | "modulevisual" => Some(TARGET_KIND_MODULE_VISUAL),
        "power_plant" | "power-plant" | "powerplant" => Some(TARGET_KIND_POWER_PLANT),
        "power_storage" | "power-storage" | "powerstorage" => Some(TARGET_KIND_POWER_STORAGE),
        "chunk" => Some(TARGET_KIND_CHUNK),
        _ => None,
    }
}

fn parse_vec3(raw: &str) -> Option<Vec3> {
    let values: Vec<_> = raw
        .split(',')
        .map(|value| value.trim().parse::<f32>().ok())
        .collect();
    match values.as_slice() {
        [Some(x), Some(y), Some(z)] => Some(Vec3::new(*x, *y, *z)),
        _ => None,
    }
}

fn parse_orbit(raw: &str) -> Option<ViewerAutomationStep> {
    let values: Vec<_> = raw
        .split(',')
        .map(|value| value.trim().parse::<f32>().ok())
        .collect();
    match values.as_slice() {
        [Some(yaw), Some(pitch)] => Some(ViewerAutomationStep::OrbitDeg {
            yaw: *yaw,
            pitch: *pitch,
        }),
        _ => None,
    }
}

fn parse_truthy(raw: &str) -> bool {
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_target_supports_first_and_explicit_variants() {
        assert_eq!(
            parse_target("first_agent"),
            Some(ViewerAutomationTarget::FirstKind(TARGET_KIND_AGENT))
        );
        assert_eq!(
            parse_target("first_location"),
            Some(ViewerAutomationTarget::FirstKind(TARGET_KIND_LOCATION))
        );
        assert_eq!(
            parse_target("first:power_plant"),
            Some(ViewerAutomationTarget::FirstKind(TARGET_KIND_POWER_PLANT))
        );
        assert_eq!(
            parse_target("agent:agent-1"),
            Some(ViewerAutomationTarget::KindId {
                kind: TARGET_KIND_AGENT,
                id: "agent-1".to_string(),
            })
        );
        assert_eq!(
            parse_target("location:loc-2"),
            Some(ViewerAutomationTarget::KindId {
                kind: TARGET_KIND_LOCATION,
                id: "loc-2".to_string(),
            })
        );
        assert_eq!(
            parse_target("power-plant:plant-1"),
            Some(ViewerAutomationTarget::KindId {
                kind: TARGET_KIND_POWER_PLANT,
                id: "plant-1".to_string(),
            })
        );
        assert_eq!(
            parse_target("modulevisual:mv-1"),
            Some(ViewerAutomationTarget::KindId {
                kind: TARGET_KIND_MODULE_VISUAL,
                id: "mv-1".to_string(),
            })
        );
        assert_eq!(
            parse_target("fragment:frag-2"),
            Some(ViewerAutomationTarget::KindId {
                kind: TARGET_KIND_FRAGMENT,
                id: "frag-2".to_string(),
            })
        );
        assert_eq!(
            parse_target("asset:a1"),
            Some(ViewerAutomationTarget::KindId {
                kind: TARGET_KIND_ASSET,
                id: "a1".to_string(),
            })
        );
        assert_eq!(parse_target("unknown:x"), None);
        assert_eq!(parse_target(""), None);
    }

    #[test]
    fn parse_steps_supports_camera_and_selection_actions() {
        let steps = parse_steps(Some(
            "mode=3d;wait=0.6;focus=agent:agent-0;pan=1,0,-2;zoom=0.8;orbit=10,-4;select=agent:agent-0",
        ));
        assert_eq!(
            steps,
            vec![
                ViewerAutomationStep::SetMode(ViewerCameraMode::ThreeD),
                ViewerAutomationStep::WaitSeconds(0.6),
                ViewerAutomationStep::Focus(ViewerAutomationTarget::KindId {
                    kind: TARGET_KIND_AGENT,
                    id: "agent-0".to_string(),
                }),
                ViewerAutomationStep::Pan(Vec3::new(1.0, 0.0, -2.0)),
                ViewerAutomationStep::ZoomFactor(0.8),
                ViewerAutomationStep::OrbitDeg {
                    yaw: 10.0,
                    pitch: -4.0
                },
                ViewerAutomationStep::Select(ViewerAutomationTarget::KindId {
                    kind: TARGET_KIND_AGENT,
                    id: "agent-0".to_string(),
                }),
            ]
        );
    }

    #[test]
    fn config_from_values_uses_auto_select_when_steps_absent() {
        let config = config_from_values(
            Some("1".to_string()),
            Some("agent:agent-2".to_string()),
            None,
        );
        assert!(config.enabled);
        assert_eq!(
            config.steps,
            vec![ViewerAutomationStep::Select(
                ViewerAutomationTarget::KindId {
                    kind: TARGET_KIND_AGENT,
                    id: "agent-2".to_string(),
                }
            )]
        );
    }

    #[test]
    fn config_from_values_prioritizes_explicit_steps() {
        let config = config_from_values(
            Some("1".to_string()),
            Some("agent:agent-2".to_string()),
            Some("wait=0.2;select=first_agent".to_string()),
        );
        assert!(config.enabled);
        assert_eq!(
            config.steps,
            vec![
                ViewerAutomationStep::WaitSeconds(0.2),
                ViewerAutomationStep::Select(ViewerAutomationTarget::FirstKind(TARGET_KIND_AGENT)),
            ]
        );
    }

    #[test]
    fn resolve_target_entity_supports_extended_scene_kinds() {
        let mut scene = Viewer3dScene::default();
        scene
            .agent_entities
            .insert("agent-1".to_string(), Entity::from_bits(1));
        scene
            .location_entities
            .insert("loc-1".to_string(), Entity::from_bits(2));
        scene
            .location_entities
            .insert("frag-2".to_string(), Entity::from_bits(3));
        scene
            .asset_entities
            .insert("asset-1".to_string(), Entity::from_bits(4));
        scene
            .module_visual_entities
            .insert("mv-1".to_string(), Entity::from_bits(5));
        scene
            .power_plant_entities
            .insert("plant-1".to_string(), Entity::from_bits(6));
        scene
            .power_storage_entities
            .insert("storage-1".to_string(), Entity::from_bits(7));
        scene
            .chunk_entities
            .insert("chunk-1".to_string(), Entity::from_bits(8));

        let fragment_target = ViewerAutomationTarget::FirstKind(TARGET_KIND_FRAGMENT);
        let Some((fragment_entity, fragment_kind, fragment_id)) =
            resolve_target_entity(&scene, &fragment_target)
        else {
            panic!("fragment target should resolve");
        };
        assert_eq!(fragment_entity, Entity::from_bits(3));
        assert_eq!(fragment_kind, SelectionKind::Fragment);
        assert_eq!(fragment_id, "frag-2");

        let module_target = ViewerAutomationTarget::KindId {
            kind: TARGET_KIND_MODULE_VISUAL,
            id: "mv-1".to_string(),
        };
        let Some((module_entity, module_kind, module_id)) =
            resolve_target_entity(&scene, &module_target)
        else {
            panic!("module_visual target should resolve");
        };
        assert_eq!(module_entity, Entity::from_bits(5));
        assert_eq!(module_kind, SelectionKind::Asset);
        assert_eq!(module_id, "mv-1");

        let chunk_target = ViewerAutomationTarget::KindId {
            kind: TARGET_KIND_CHUNK,
            id: "chunk-1".to_string(),
        };
        let Some((chunk_entity, chunk_kind, chunk_id)) =
            resolve_target_entity(&scene, &chunk_target)
        else {
            panic!("chunk target should resolve");
        };
        assert_eq!(chunk_entity, Entity::from_bits(8));
        assert_eq!(chunk_kind, SelectionKind::Chunk);
        assert_eq!(chunk_id, "chunk-1");
    }
}
