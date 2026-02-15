use std::collections::HashMap;

use bevy::prelude::*;

use super::camera_controls::{orbit_min_radius, sync_2d_zoom_projection};
use super::{
    camera_orbit_preset, camera_projection_for_mode, OrbitCamera, SelectionInfo, SelectionKind,
    Viewer3dCamera, Viewer3dConfig, Viewer3dScene, ViewerCameraMode, ViewerSelection,
    ORBIT_MAX_RADIUS,
};

const AUTO_FOCUS_ENV: &str = "AGENT_WORLD_VIEWER_AUTO_FOCUS";
const AUTO_FOCUS_TARGET_ENV: &str = "AGENT_WORLD_VIEWER_AUTO_FOCUS_TARGET";
const AUTO_FOCUS_FORCE_3D_ENV: &str = "AGENT_WORLD_VIEWER_AUTO_FOCUS_FORCE_3D";
const AUTO_FOCUS_RADIUS_ENV: &str = "AGENT_WORLD_VIEWER_AUTO_FOCUS_RADIUS";

const DEFAULT_AUTO_FOCUS_FORCE_3D: bool = true;
const DEFAULT_MANUAL_FOCUS_RADIUS_M: f32 = 14.0;
const MIN_TWO_D_FOCUS_RADIUS_M: f32 = 12.0;
const MIN_LOCATION_FOCUS_RADIUS_M: f32 = 6.0;
const MIN_AGENT_FOCUS_RADIUS_M: f32 = 5.0;
const MAX_AGENT_FOCUS_RADIUS_M: f32 = 32.0;

#[derive(Resource, Clone, Debug, PartialEq)]
pub(super) struct AutoFocusConfig {
    pub enabled: bool,
    pub target: AutoFocusTarget,
    pub force_3d: bool,
    pub radius_override: Option<f32>,
}

impl Default for AutoFocusConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            target: AutoFocusTarget::FirstFragment,
            force_3d: DEFAULT_AUTO_FOCUS_FORCE_3D,
            radius_override: None,
        }
    }
}

#[derive(Resource, Default, Clone, Debug)]
pub(super) struct AutoFocusState {
    pub startup_applied: bool,
    pub skip_next_mode_sync: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum AutoFocusTarget {
    FirstFragment,
    FirstLocation,
    FirstAgent,
    Location(String),
    Agent(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ResolvedFocus {
    focus: Vec3,
    radius: f32,
}

enum ResolvedTarget {
    Location { id: String, entity: Entity },
    Agent { id: String, entity: Entity },
}

pub(super) fn auto_focus_config_from_env() -> AutoFocusConfig {
    config_from_values(
        std::env::var(AUTO_FOCUS_ENV).ok(),
        std::env::var(AUTO_FOCUS_TARGET_ENV).ok(),
        std::env::var(AUTO_FOCUS_FORCE_3D_ENV).ok(),
        std::env::var(AUTO_FOCUS_RADIUS_ENV).ok(),
    )
}

pub(super) fn apply_startup_auto_focus(
    auto_focus_config: Res<AutoFocusConfig>,
    mut auto_focus_state: ResMut<AutoFocusState>,
    scene: Res<Viewer3dScene>,
    config: Res<Viewer3dConfig>,
    mut camera_mode: ResMut<ViewerCameraMode>,
    transforms: Query<&Transform, Without<Viewer3dCamera>>,
    mut camera_query: Query<
        (&mut OrbitCamera, &mut Transform, &mut Projection),
        With<Viewer3dCamera>,
    >,
) {
    if auto_focus_state.startup_applied || !auto_focus_config.enabled {
        return;
    }

    let Some(target) = resolve_target(&auto_focus_config.target, &scene) else {
        return;
    };
    let Some(resolved_focus) = resolve_focus(
        target,
        &scene,
        &transforms,
        config.effective_cm_to_unit(),
        focus_radius_units(DEFAULT_MANUAL_FOCUS_RADIUS_M, config.effective_cm_to_unit()),
    ) else {
        return;
    };

    let Ok((mut orbit, mut camera_transform, mut projection)) = camera_query.single_mut() else {
        return;
    };

    apply_focus_to_camera(
        resolved_focus,
        auto_focus_config.radius_override,
        auto_focus_config.force_3d,
        &config,
        &mut camera_mode,
        &mut orbit,
        &mut camera_transform,
        &mut projection,
        Some(&mut auto_focus_state),
    );
    auto_focus_state.startup_applied = true;
}

pub(super) fn handle_focus_selection_hotkey(
    keys: Res<ButtonInput<KeyCode>>,
    selection: Res<ViewerSelection>,
    scene: Res<Viewer3dScene>,
    config: Res<Viewer3dConfig>,
    mut camera_mode: ResMut<ViewerCameraMode>,
    transforms: Query<&Transform, Without<Viewer3dCamera>>,
    mut camera_query: Query<
        (&mut OrbitCamera, &mut Transform, &mut Projection),
        With<Viewer3dCamera>,
    >,
) {
    if !keys.just_pressed(KeyCode::KeyF) {
        return;
    }
    let Some(current) = selection.current.as_ref() else {
        return;
    };

    let Some(resolved_focus) =
        resolve_focus_from_selection(current, &scene, &transforms, config.effective_cm_to_unit())
    else {
        return;
    };

    let Ok((mut orbit, mut camera_transform, mut projection)) = camera_query.single_mut() else {
        return;
    };

    apply_focus_to_camera(
        resolved_focus,
        None,
        false,
        &config,
        &mut camera_mode,
        &mut orbit,
        &mut camera_transform,
        &mut projection,
        None,
    );
}

fn apply_focus_to_camera(
    resolved_focus: ResolvedFocus,
    radius_override: Option<f32>,
    force_3d: bool,
    config: &Viewer3dConfig,
    camera_mode: &mut ViewerCameraMode,
    orbit: &mut OrbitCamera,
    camera_transform: &mut Transform,
    projection: &mut Projection,
    auto_focus_state: Option<&mut AutoFocusState>,
) {
    let cm_to_unit = config.effective_cm_to_unit();
    if force_3d && *camera_mode != ViewerCameraMode::ThreeD {
        *camera_mode = ViewerCameraMode::ThreeD;
        let preset = camera_orbit_preset(
            ViewerCameraMode::ThreeD,
            Some(resolved_focus.focus),
            cm_to_unit,
        );
        orbit.yaw = preset.yaw;
        orbit.pitch = preset.pitch;
        *projection = camera_projection_for_mode(ViewerCameraMode::ThreeD, config);
        if let Some(state) = auto_focus_state {
            state.skip_next_mode_sync = true;
        }
    }

    orbit.focus = resolved_focus.focus;
    let min_radius = if matches!(*camera_mode, ViewerCameraMode::TwoD) && !force_3d {
        focus_radius_units(MIN_TWO_D_FOCUS_RADIUS_M, cm_to_unit).max(orbit_min_radius(cm_to_unit))
    } else {
        orbit_min_radius(cm_to_unit)
    };
    orbit.radius = radius_override
        .unwrap_or(resolved_focus.radius)
        .clamp(min_radius, ORBIT_MAX_RADIUS);

    if matches!(*camera_mode, ViewerCameraMode::TwoD) {
        if !matches!(*projection, Projection::Orthographic(_)) {
            *projection = camera_projection_for_mode(ViewerCameraMode::TwoD, config);
        }
        sync_2d_zoom_projection(projection, orbit.radius, config.effective_cm_to_unit());
    }

    orbit.apply_to_transform(camera_transform);
}

fn resolve_focus_from_selection(
    selection: &SelectionInfo,
    scene: &Viewer3dScene,
    transforms: &Query<&Transform, Without<Viewer3dCamera>>,
    cm_to_unit: f32,
) -> Option<ResolvedFocus> {
    let entity = selection.entity;
    let focus = transforms.get(entity).ok()?.translation;
    let radius = match selection.kind {
        SelectionKind::Location => scene
            .location_radii_cm
            .get(selection.id.as_str())
            .copied()
            .map(|radius_cm| location_focus_radius(radius_cm, cm_to_unit))
            .unwrap_or(focus_radius_units(
                DEFAULT_MANUAL_FOCUS_RADIUS_M,
                cm_to_unit,
            )),
        SelectionKind::Agent => scene
            .agent_heights_cm
            .get(selection.id.as_str())
            .copied()
            .map(|height_cm| agent_focus_radius(height_cm, cm_to_unit))
            .unwrap_or(focus_radius_units(
                DEFAULT_MANUAL_FOCUS_RADIUS_M,
                cm_to_unit,
            )),
        _ => focus_radius_units(DEFAULT_MANUAL_FOCUS_RADIUS_M, cm_to_unit),
    };

    Some(ResolvedFocus { focus, radius })
}

fn resolve_target(target: &AutoFocusTarget, scene: &Viewer3dScene) -> Option<ResolvedTarget> {
    match target {
        AutoFocusTarget::FirstFragment => {
            first_sorted_matching(&scene.location_entities, |id| id.starts_with("frag-"))
                .or_else(|| first_sorted_matching(&scene.location_entities, |_| true))
                .and_then(|id| {
                    scene
                        .location_entities
                        .get(id.as_str())
                        .copied()
                        .map(|entity| ResolvedTarget::Location { id, entity })
                })
        }
        AutoFocusTarget::FirstLocation => first_sorted_matching(&scene.location_entities, |_| true)
            .and_then(|id| {
                scene
                    .location_entities
                    .get(id.as_str())
                    .copied()
                    .map(|entity| ResolvedTarget::Location { id, entity })
            }),
        AutoFocusTarget::FirstAgent => first_sorted_matching(&scene.agent_entities, |_| true)
            .and_then(|id| {
                scene
                    .agent_entities
                    .get(id.as_str())
                    .copied()
                    .map(|entity| ResolvedTarget::Agent { id, entity })
            }),
        AutoFocusTarget::Location(location_id) => scene
            .location_entities
            .get(location_id.as_str())
            .copied()
            .map(|entity| ResolvedTarget::Location {
                id: location_id.clone(),
                entity,
            }),
        AutoFocusTarget::Agent(agent_id) => scene
            .agent_entities
            .get(agent_id.as_str())
            .copied()
            .map(|entity| ResolvedTarget::Agent {
                id: agent_id.clone(),
                entity,
            }),
    }
}

fn first_sorted_matching<F>(items: &HashMap<String, Entity>, predicate: F) -> Option<String>
where
    F: Fn(&str) -> bool,
{
    let mut ids = items
        .keys()
        .filter(|id| predicate(id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids.into_iter().next()
}

fn resolve_focus(
    target: ResolvedTarget,
    scene: &Viewer3dScene,
    transforms: &Query<&Transform, Without<Viewer3dCamera>>,
    cm_to_unit: f32,
    fallback_radius: f32,
) -> Option<ResolvedFocus> {
    match target {
        ResolvedTarget::Location { id, entity } => {
            let focus = transforms.get(entity).ok()?.translation;
            let radius = scene
                .location_radii_cm
                .get(id.as_str())
                .copied()
                .map(|radius_cm| location_focus_radius(radius_cm, cm_to_unit))
                .unwrap_or(fallback_radius);
            Some(ResolvedFocus { focus, radius })
        }
        ResolvedTarget::Agent { id, entity } => {
            let focus = transforms.get(entity).ok()?.translation;
            let radius = scene
                .agent_heights_cm
                .get(id.as_str())
                .copied()
                .map(|height_cm| agent_focus_radius(height_cm, cm_to_unit))
                .unwrap_or(fallback_radius);
            Some(ResolvedFocus { focus, radius })
        }
    }
}

fn location_focus_radius(radius_cm: i64, cm_to_unit: f32) -> f32 {
    let location_radius = (radius_cm.max(1) as f32 * cm_to_unit).max(0.01);
    (location_radius * 3.2).clamp(
        focus_radius_units(MIN_LOCATION_FOCUS_RADIUS_M, cm_to_unit),
        ORBIT_MAX_RADIUS,
    )
}

fn agent_focus_radius(height_cm: i64, cm_to_unit: f32) -> f32 {
    let height = (height_cm.max(1) as f32 * cm_to_unit).max(0.005);
    (height * 18.0).clamp(
        focus_radius_units(MIN_AGENT_FOCUS_RADIUS_M, cm_to_unit),
        focus_radius_units(MAX_AGENT_FOCUS_RADIUS_M, cm_to_unit),
    )
}

fn focus_radius_units(radius_m: f32, cm_to_unit: f32) -> f32 {
    (radius_m.max(0.0) * cm_to_unit.max(f32::EPSILON) * 100.0).max(0.0001)
}

fn config_from_values(
    enabled_value: Option<String>,
    target_value: Option<String>,
    force_3d_value: Option<String>,
    radius_value: Option<String>,
) -> AutoFocusConfig {
    let parsed_target = target_value
        .as_deref()
        .and_then(parse_auto_focus_target_from);
    let target = parsed_target
        .clone()
        .unwrap_or(AutoFocusTarget::FirstFragment);

    let enabled = parse_bool(enabled_value.as_deref()).unwrap_or(false) || parsed_target.is_some();
    let force_3d = parse_bool(force_3d_value.as_deref()).unwrap_or(DEFAULT_AUTO_FOCUS_FORCE_3D);
    let radius_override = radius_value
        .as_deref()
        .and_then(|raw| raw.trim().parse::<f32>().ok())
        .filter(|radius| radius.is_finite() && *radius > 0.0);

    AutoFocusConfig {
        enabled,
        target,
        force_3d,
        radius_override,
    }
}

fn parse_bool(raw: Option<&str>) -> Option<bool> {
    raw.map(str::trim).and_then(|value| {
        if value.eq_ignore_ascii_case("1")
            || value.eq_ignore_ascii_case("true")
            || value.eq_ignore_ascii_case("yes")
            || value.eq_ignore_ascii_case("on")
        {
            return Some(true);
        }
        if value.eq_ignore_ascii_case("0")
            || value.eq_ignore_ascii_case("false")
            || value.eq_ignore_ascii_case("no")
            || value.eq_ignore_ascii_case("off")
        {
            return Some(false);
        }
        None
    })
}

fn parse_auto_focus_target_from(raw: &str) -> Option<AutoFocusTarget> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let normalized = trimmed.to_ascii_lowercase();
    match normalized.as_str() {
        "first_fragment" | "fragment" => Some(AutoFocusTarget::FirstFragment),
        "first_location" | "location" => Some(AutoFocusTarget::FirstLocation),
        "first_agent" | "agent" => Some(AutoFocusTarget::FirstAgent),
        _ => {
            if let Some((prefix, id)) = trimmed.split_once(':') {
                let id = id.trim();
                if id.is_empty() {
                    return None;
                }
                if prefix.eq_ignore_ascii_case("location") {
                    return Some(AutoFocusTarget::Location(id.to_string()));
                }
                if prefix.eq_ignore_ascii_case("agent") {
                    return Some(AutoFocusTarget::Agent(id.to_string()));
                }
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_from_values_enables_when_target_is_valid() {
        let config = config_from_values(
            Some("false".to_string()),
            Some("location:frag-2".to_string()),
            Some("0".to_string()),
            Some("18.5".to_string()),
        );

        assert!(config.enabled);
        assert_eq!(
            config.target,
            AutoFocusTarget::Location("frag-2".to_string())
        );
        assert!(!config.force_3d);
        assert_eq!(config.radius_override, Some(18.5));
    }

    #[test]
    fn config_from_values_falls_back_for_invalid_target() {
        let config = config_from_values(
            Some("0".to_string()),
            Some("bad_target".to_string()),
            Some("on".to_string()),
            Some("-5".to_string()),
        );

        assert!(!config.enabled);
        assert_eq!(config.target, AutoFocusTarget::FirstFragment);
        assert!(config.force_3d);
        assert_eq!(config.radius_override, None);
    }

    #[test]
    fn parse_auto_focus_target_variants() {
        assert_eq!(
            parse_auto_focus_target_from("first_fragment"),
            Some(AutoFocusTarget::FirstFragment)
        );
        assert_eq!(
            parse_auto_focus_target_from("location:loc-1"),
            Some(AutoFocusTarget::Location("loc-1".to_string()))
        );
        assert_eq!(
            parse_auto_focus_target_from("agent:agent-0"),
            Some(AutoFocusTarget::Agent("agent-0".to_string()))
        );
        assert_eq!(parse_auto_focus_target_from(""), None);
    }

    #[test]
    fn resolve_target_prefers_first_fragment_then_location() {
        let mut scene = Viewer3dScene::default();
        scene
            .location_entities
            .insert("region-1".to_string(), Entity::from_bits(2));
        scene
            .location_entities
            .insert("frag-2".to_string(), Entity::from_bits(1));
        scene
            .location_entities
            .insert("frag-1".to_string(), Entity::from_bits(3));

        let target =
            resolve_target(&AutoFocusTarget::FirstFragment, &scene).expect("target should resolve");
        match target {
            ResolvedTarget::Location { id, entity } => {
                assert_eq!(id, "frag-1");
                assert_eq!(entity, Entity::from_bits(3));
            }
            ResolvedTarget::Agent { .. } => panic!("expected location target"),
        }
    }

    #[test]
    fn apply_focus_to_camera_syncs_two_d_projection_scale_with_radius() {
        let config = Viewer3dConfig::default();
        let mut camera_mode = ViewerCameraMode::TwoD;
        let mut orbit = OrbitCamera {
            focus: Vec3::ZERO,
            radius: 12.0,
            yaw: 0.0,
            pitch: -1.53,
        };
        let mut camera_transform = Transform::default();
        let mut projection = camera_projection_for_mode(ViewerCameraMode::TwoD, &config);
        let before_scale = match &projection {
            Projection::Orthographic(ortho) => ortho.scale,
            _ => panic!("expected orthographic projection"),
        };

        apply_focus_to_camera(
            ResolvedFocus {
                focus: Vec3::new(4.0, 2.0, -3.0),
                radius: 320.0,
            },
            None,
            false,
            &config,
            &mut camera_mode,
            &mut orbit,
            &mut camera_transform,
            &mut projection,
            None,
        );

        let after_scale = match &projection {
            Projection::Orthographic(ortho) => ortho.scale,
            _ => panic!("expected orthographic projection"),
        };
        assert!(after_scale > before_scale);
    }
}
