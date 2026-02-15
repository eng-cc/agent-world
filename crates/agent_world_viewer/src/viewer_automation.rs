use bevy::prelude::*;

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
    step_index: usize,
    wait_until_secs: Option<f64>,
    completed: bool,
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
    FirstAgent,
    FirstLocation,
    Agent(String),
    Location(String),
}

enum StepResult {
    Applied,
    AppliedYield,
    Pending,
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
    if !config.enabled || state.completed {
        return;
    }

    let now = time.elapsed_secs_f64();
    if let Some(wait_until_secs) = state.wait_until_secs {
        if now < wait_until_secs {
            return;
        }
        state.wait_until_secs = None;
    }

    loop {
        let Some(step) = config.steps.get(state.step_index).cloned() else {
            state.completed = true;
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
                state.step_index += 1;
                continue;
            }
            StepResult::AppliedYield => {
                state.step_index += 1;
                return;
            }
            StepResult::Pending => return,
        }
    }
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

            orbit.radius =
                (orbit.radius * factor.max(0.01)).clamp(ORBIT_MIN_RADIUS, ORBIT_MAX_RADIUS);
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
        ViewerAutomationTarget::FirstAgent => {
            first_sorted_id(&scene.agent_entities).and_then(|id| {
                scene
                    .agent_entities
                    .get(id.as_str())
                    .copied()
                    .map(|entity| (entity, SelectionKind::Agent, id))
            })
        }
        ViewerAutomationTarget::FirstLocation => first_sorted_id(&scene.location_entities)
            .and_then(|id| {
                scene
                    .location_entities
                    .get(id.as_str())
                    .copied()
                    .map(|entity| (entity, SelectionKind::Location, id))
            }),
        ViewerAutomationTarget::Agent(agent_id) => scene
            .agent_entities
            .get(agent_id.as_str())
            .copied()
            .map(|entity| (entity, SelectionKind::Agent, agent_id.clone())),
        ViewerAutomationTarget::Location(location_id) => scene
            .location_entities
            .get(location_id.as_str())
            .copied()
            .map(|entity| (entity, SelectionKind::Location, location_id.clone())),
    }
}

fn first_sorted_id(items: &std::collections::HashMap<String, Entity>) -> Option<String> {
    let mut ids: Vec<_> = items.keys().cloned().collect();
    ids.sort();
    ids.into_iter().next()
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
    match value.to_ascii_lowercase().as_str() {
        "first_agent" => Some(ViewerAutomationTarget::FirstAgent),
        "first_location" => Some(ViewerAutomationTarget::FirstLocation),
        _ => {
            let (kind, id) = value.split_once(':')?;
            let id = id.trim();
            if id.is_empty() {
                return None;
            }
            match kind.trim().to_ascii_lowercase().as_str() {
                "agent" => Some(ViewerAutomationTarget::Agent(id.to_string())),
                "location" => Some(ViewerAutomationTarget::Location(id.to_string())),
                _ => None,
            }
        }
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
            Some(ViewerAutomationTarget::FirstAgent)
        );
        assert_eq!(
            parse_target("first_location"),
            Some(ViewerAutomationTarget::FirstLocation)
        );
        assert_eq!(
            parse_target("agent:agent-1"),
            Some(ViewerAutomationTarget::Agent("agent-1".to_string()))
        );
        assert_eq!(
            parse_target("location:loc-2"),
            Some(ViewerAutomationTarget::Location("loc-2".to_string()))
        );
        assert_eq!(parse_target("asset:a1"), None);
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
                ViewerAutomationStep::Focus(ViewerAutomationTarget::Agent("agent-0".to_string())),
                ViewerAutomationStep::Pan(Vec3::new(1.0, 0.0, -2.0)),
                ViewerAutomationStep::ZoomFactor(0.8),
                ViewerAutomationStep::OrbitDeg {
                    yaw: 10.0,
                    pitch: -4.0
                },
                ViewerAutomationStep::Select(ViewerAutomationTarget::Agent("agent-0".to_string())),
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
            vec![ViewerAutomationStep::Select(ViewerAutomationTarget::Agent(
                "agent-2".to_string()
            ))]
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
                ViewerAutomationStep::Select(ViewerAutomationTarget::FirstAgent),
            ]
        );
    }
}
