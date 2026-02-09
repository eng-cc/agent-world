use bevy::ecs::message::MessageReader;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::{
    OrbitCamera, RightPanelWidthState, Viewer3dCamera, Viewer3dConfig, ViewerCameraMode,
    WorldBoundsSurface, WorldFloorSurface, DEFAULT_2D_CAMERA_RADIUS, DEFAULT_3D_CAMERA_RADIUS,
    ORBIT_MAX_RADIUS, ORBIT_MIN_RADIUS, ORBIT_PAN_SENSITIVITY, ORBIT_ROTATE_SENSITIVITY,
    ORBIT_ZOOM_SENSITIVITY, UI_PANEL_WIDTH,
};

#[derive(Resource, Default)]
pub(super) struct OrbitDragState {
    last_cursor_position: Option<Vec2>,
}

pub(super) fn orbit_camera_controls(
    windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    camera_mode: Res<ViewerCameraMode>,
    panel_width: Option<Res<RightPanelWidthState>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut drag_state: ResMut<OrbitDragState>,
    mut query: Query<(&mut OrbitCamera, &mut Transform), With<Viewer3dCamera>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    let cursor_position = window.cursor_position();
    let panel_width_px = panel_width
        .as_deref()
        .map(|state| state.width_px)
        .unwrap_or(UI_PANEL_WIDTH);
    let cursor_in_3d = cursor_position
        .map(|cursor| cursor_in_3d_view(window, cursor, panel_width_px))
        .unwrap_or(false);

    let shift_pressed = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let rotate_drag = buttons.pressed(MouseButton::Left) && !shift_pressed;
    let pan_drag = buttons.pressed(MouseButton::Right)
        || buttons.pressed(MouseButton::Middle)
        || (buttons.pressed(MouseButton::Left) && shift_pressed);
    let dragging = cursor_in_3d && (rotate_drag || pan_drag);

    let (delta, next_cursor) =
        drag_delta(drag_state.last_cursor_position, cursor_position, dragging);
    drag_state.last_cursor_position = next_cursor;

    let mut scroll = 0.0;
    for event in mouse_wheel.read() {
        if cursor_in_3d {
            scroll += event.y;
        }
    }

    if delta == Vec2::ZERO && scroll == 0.0 {
        return;
    }

    let Ok((mut orbit, mut transform)) = query.single_mut() else {
        return;
    };

    let changed = apply_orbit_input(
        &mut orbit,
        delta,
        scroll,
        rotate_drag && dragging,
        pan_drag && dragging,
        *camera_mode,
    );

    if changed {
        orbit.apply_to_transform(&mut transform);
    }
}

fn cursor_in_3d_view(window: &Window, cursor: Vec2, panel_width_px: f32) -> bool {
    let viewport_width = (window.width() - panel_width_px).max(0.0);
    cursor.x <= viewport_width
}

fn drag_delta(
    previous: Option<Vec2>,
    current: Option<Vec2>,
    dragging: bool,
) -> (Vec2, Option<Vec2>) {
    if !dragging {
        return (Vec2::ZERO, None);
    }

    let Some(cursor) = current else {
        return (Vec2::ZERO, None);
    };

    let delta = previous.map(|last| cursor - last).unwrap_or(Vec2::ZERO);
    (delta, Some(cursor))
}

fn apply_orbit_input(
    orbit: &mut OrbitCamera,
    delta: Vec2,
    scroll: f32,
    rotate_drag: bool,
    pan_drag: bool,
    mode: ViewerCameraMode,
) -> bool {
    let mut changed = false;

    let allow_rotate = matches!(mode, ViewerCameraMode::ThreeD);
    if allow_rotate && rotate_drag && delta != Vec2::ZERO {
        orbit.yaw -= delta.x * ORBIT_ROTATE_SENSITIVITY;
        orbit.pitch = (orbit.pitch - delta.y * ORBIT_ROTATE_SENSITIVITY).clamp(-1.54, 1.54);
        changed = true;
    }

    if pan_drag && delta != Vec2::ZERO {
        let rotation =
            Quat::from_axis_angle(Vec3::Y, orbit.yaw) * Quat::from_axis_angle(Vec3::X, orbit.pitch);
        let right = rotation * Vec3::X;
        let up = rotation * Vec3::Y;
        let pan_scale = orbit.radius * ORBIT_PAN_SENSITIVITY;
        orbit.focus += (-delta.x * pan_scale) * right + (delta.y * pan_scale) * up;
        changed = true;
    }

    if scroll != 0.0 {
        orbit.radius = (orbit.radius * (1.0 - scroll * ORBIT_ZOOM_SENSITIVITY))
            .clamp(ORBIT_MIN_RADIUS, ORBIT_MAX_RADIUS);
        changed = true;
    }

    changed
}

pub(super) fn camera_orbit_preset(
    mode: ViewerCameraMode,
    focus: Option<Vec3>,
    cm_to_unit: f32,
) -> OrbitCamera {
    let focus = focus.unwrap_or(Vec3::ZERO);
    match mode {
        ViewerCameraMode::TwoD => {
            let min_radius = world_view_radius(cm_to_unit).max(ORBIT_MIN_RADIUS);
            OrbitCamera {
                focus,
                radius: min_radius.max(DEFAULT_2D_CAMERA_RADIUS),
                yaw: 0.0,
                pitch: -1.53,
            }
        }
        ViewerCameraMode::ThreeD => OrbitCamera {
            focus,
            radius: DEFAULT_3D_CAMERA_RADIUS,
            yaw: -0.7,
            pitch: 0.55,
        },
    }
}

pub(super) fn camera_projection_for_mode(
    mode: ViewerCameraMode,
    config: &Viewer3dConfig,
) -> Projection {
    match mode {
        ViewerCameraMode::TwoD => {
            let scale = world_view_ortho_scale(config.effective_cm_to_unit());
            Projection::Orthographic(OrthographicProjection {
                near: config.physical.camera_near_m,
                far: config.physical.camera_far_m,
                scale,
                ..OrthographicProjection::default_3d()
            })
        }
        ViewerCameraMode::ThreeD => Projection::Perspective(PerspectiveProjection {
            near: config.physical.camera_near_m,
            far: config.physical.camera_far_m,
            ..default()
        }),
    }
}

fn world_view_radius(cm_to_unit: f32) -> f32 {
    let default_space = agent_world::simulator::SpaceConfig::default();
    let cm_span = default_space
        .width_cm
        .max(default_space.depth_cm)
        .max(default_space.height_cm)
        .max(1) as f32;
    (cm_span * cm_to_unit * 0.55).clamp(12.0, ORBIT_MAX_RADIUS)
}

fn world_view_ortho_scale(cm_to_unit: f32) -> f32 {
    let default_space = agent_world::simulator::SpaceConfig::default();
    let cm_span = default_space
        .width_cm
        .max(default_space.depth_cm)
        .max(default_space.height_cm)
        .max(1) as f32;
    let world_span_units = cm_span * cm_to_unit;
    let reference_viewport_px = 880.0;
    ((world_span_units * 1.15) / reference_viewport_px).clamp(0.03, 4.0)
}

pub(super) fn sync_camera_mode(
    camera_mode: Res<ViewerCameraMode>,
    config: Res<Viewer3dConfig>,
    mut cameras: Query<(&mut OrbitCamera, &mut Transform, &mut Projection), With<Viewer3dCamera>>,
) {
    if !camera_mode.is_changed() {
        return;
    }

    let Ok((mut orbit, mut transform, mut projection)) = cameras.single_mut() else {
        return;
    };

    let next_orbit = camera_orbit_preset(
        *camera_mode,
        Some(orbit.focus),
        config.effective_cm_to_unit(),
    );
    *orbit = next_orbit;
    orbit.apply_to_transform(&mut transform);
    *projection = camera_projection_for_mode(*camera_mode, &config);
}

pub(super) fn sync_world_background_visibility(
    _camera_mode: Res<ViewerCameraMode>,
    mut query: Query<(
        Option<&WorldFloorSurface>,
        Option<&WorldBoundsSurface>,
        &mut Visibility,
    )>,
) {
    for (is_floor, is_bounds, mut visibility) in &mut query {
        if is_floor.is_some() || is_bounds.is_some() {
            *visibility = Visibility::Hidden;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drag_delta_requires_active_dragging() {
        let current = Vec2::new(40.0, 20.0);
        let (delta, next_cursor) = drag_delta(Some(Vec2::new(10.0, 10.0)), Some(current), false);
        assert_eq!(delta, Vec2::ZERO);
        assert_eq!(next_cursor, None);
    }

    #[test]
    fn drag_delta_uses_cursor_position_difference() {
        let previous = Vec2::new(10.0, 10.0);
        let current = Vec2::new(24.0, 30.0);
        let (delta, next_cursor) = drag_delta(Some(previous), Some(current), true);
        assert_eq!(delta, Vec2::new(14.0, 20.0));
        assert_eq!(next_cursor, Some(current));
    }

    #[test]
    fn apply_orbit_input_updates_focus_and_radius() {
        let mut orbit = OrbitCamera {
            focus: Vec3::ZERO,
            radius: 20.0,
            yaw: 0.0,
            pitch: 0.0,
        };

        let changed = apply_orbit_input(
            &mut orbit,
            Vec2::new(6.0, -4.0),
            1.0,
            false,
            true,
            ViewerCameraMode::ThreeD,
        );
        assert!(changed);
        assert_ne!(orbit.focus, Vec3::ZERO);
        assert!(orbit.radius < 20.0);
    }

    #[test]
    fn apply_orbit_input_2d_mode_ignores_rotation_drag() {
        let mut orbit = OrbitCamera {
            focus: Vec3::ZERO,
            radius: 20.0,
            yaw: 1.0,
            pitch: -1.53,
        };

        let changed = apply_orbit_input(
            &mut orbit,
            Vec2::new(8.0, 5.0),
            0.0,
            true,
            false,
            ViewerCameraMode::TwoD,
        );

        assert!(!changed);
        assert!((orbit.yaw - 1.0).abs() < f32::EPSILON);
        assert!((orbit.pitch + 1.53).abs() < f32::EPSILON);
    }

    #[test]
    fn camera_projection_matches_mode() {
        let config = Viewer3dConfig::default();
        let two_d = camera_projection_for_mode(ViewerCameraMode::TwoD, &config);
        match two_d {
            Projection::Orthographic(projection) => {
                assert!(projection.scale > 0.0);
                assert!(projection.scale < 1.0);
            }
            _ => panic!("expected orthographic projection for 2D mode"),
        }

        let three_d = camera_projection_for_mode(ViewerCameraMode::ThreeD, &config);
        assert!(matches!(three_d, Projection::Perspective(_)));
    }

    #[test]
    fn camera_orbit_preset_two_d_has_top_down_pitch() {
        let orbit = camera_orbit_preset(
            ViewerCameraMode::TwoD,
            None,
            Viewer3dConfig::default().effective_cm_to_unit(),
        );
        assert!(orbit.pitch < -1.5);
        assert!(orbit.radius >= DEFAULT_2D_CAMERA_RADIUS);
    }

    #[test]
    fn world_background_surfaces_are_always_hidden() {
        let mut app = App::new();
        app.add_systems(Update, sync_world_background_visibility);
        app.insert_resource(ViewerCameraMode::default());
        let floor = app
            .world_mut()
            .spawn((WorldFloorSurface, Visibility::Visible))
            .id();
        let bounds = app
            .world_mut()
            .spawn((WorldBoundsSurface, Visibility::Visible))
            .id();

        app.world_mut().insert_resource(ViewerCameraMode::TwoD);
        app.update();

        let floor_visibility = app
            .world()
            .get::<Visibility>(floor)
            .expect("floor visibility");
        let bounds_visibility = app
            .world()
            .get::<Visibility>(bounds)
            .expect("bounds visibility");
        assert_eq!(*floor_visibility, Visibility::Hidden);
        assert_eq!(*bounds_visibility, Visibility::Hidden);

        app.world_mut().insert_resource(ViewerCameraMode::ThreeD);
        app.update();
        let floor_visibility = app
            .world()
            .get::<Visibility>(floor)
            .expect("floor visibility");
        let bounds_visibility = app
            .world()
            .get::<Visibility>(bounds)
            .expect("bounds visibility");
        assert_eq!(*floor_visibility, Visibility::Hidden);
        assert_eq!(*bounds_visibility, Visibility::Hidden);
    }
}
