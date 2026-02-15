use bevy::ecs::message::MessageReader;
use bevy::input::gestures::PinchGesture;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::{
    grid_line_scale, grid_line_thickness, AutoFocusState, BaseScale, GridLineVisual, OrbitCamera,
    RightPanelWidthState, TwoDMapMarker, Viewer3dCamera, Viewer3dConfig, ViewerCameraMode,
    WorldBoundsSurface, WorldFloorSurface, WorldOverlayConfig, DEFAULT_2D_CAMERA_RADIUS,
    DEFAULT_3D_CAMERA_RADIUS, ORBIT_MAX_RADIUS, ORBIT_MIN_RADIUS, ORBIT_PAN_SENSITIVITY,
    ORBIT_ROTATE_SENSITIVITY, ORBIT_ZOOM_SENSITIVITY, ORTHO_MAX_SCALE, ORTHO_MIN_SCALE,
    UI_PANEL_WIDTH,
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
    config: Res<Viewer3dConfig>,
    panel_width: Option<Res<RightPanelWidthState>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut pinch_gesture: MessageReader<PinchGesture>,
    mut drag_state: ResMut<OrbitDragState>,
    mut query: Query<(&mut OrbitCamera, &mut Transform, &mut Projection), With<Viewer3dCamera>>,
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
            scroll += normalized_mouse_wheel_delta(event.unit, event.y);
        }
    }
    for event in pinch_gesture.read() {
        if cursor_in_3d {
            scroll += pinch_scroll_delta(event.0);
        }
    }

    if delta == Vec2::ZERO && scroll == 0.0 {
        return;
    }

    let Ok((mut orbit, mut transform, mut projection)) = query.single_mut() else {
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

    if scroll != 0.0 && *camera_mode == ViewerCameraMode::TwoD {
        sync_2d_zoom_projection(&mut projection, orbit.radius, config.effective_cm_to_unit());
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

fn normalized_mouse_wheel_delta(unit: MouseScrollUnit, y: f32) -> f32 {
    match unit {
        MouseScrollUnit::Line => y,
        MouseScrollUnit::Pixel => y / MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR,
    }
}

fn pinch_scroll_delta(delta: f32) -> f32 {
    // Pinch magnify deltas are much smaller than line-based wheel deltas.
    delta * 8.0
}

fn sync_2d_zoom_projection(projection: &mut Projection, orbit_radius: f32, cm_to_unit: f32) {
    if let Projection::Orthographic(ortho) = projection {
        ortho.scale = two_d_ortho_scale_for_radius(orbit_radius, cm_to_unit);
    }
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

fn two_d_reference_radius(cm_to_unit: f32) -> f32 {
    world_view_radius(cm_to_unit)
        .max(DEFAULT_2D_CAMERA_RADIUS)
        .max(ORBIT_MIN_RADIUS)
}

fn two_d_ortho_scale_for_radius(radius: f32, cm_to_unit: f32) -> f32 {
    let base_scale = world_view_ortho_scale(cm_to_unit);
    let reference_radius = two_d_reference_radius(cm_to_unit);
    let ratio = (radius / reference_radius).max(0.01);
    (base_scale * ratio).clamp(ORTHO_MIN_SCALE, ORTHO_MAX_SCALE)
}

pub(super) fn sync_camera_mode(
    camera_mode: Res<ViewerCameraMode>,
    config: Res<Viewer3dConfig>,
    auto_focus_state: Option<ResMut<AutoFocusState>>,
    mut cameras: Query<(&mut OrbitCamera, &mut Transform, &mut Projection), With<Viewer3dCamera>>,
    mut grid_lines: Query<
        (&GridLineVisual, &mut Transform, &mut BaseScale),
        Without<Viewer3dCamera>,
    >,
) {
    if let Some(mut state) = auto_focus_state {
        if state.skip_next_mode_sync {
            state.skip_next_mode_sync = false;
            return;
        }
    }

    let mode_changed = camera_mode.is_changed();
    if !mode_changed {
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

    for (visual, mut transform, mut base_scale) in &mut grid_lines {
        let thickness = grid_line_thickness(visual.kind, *camera_mode);
        let scale = grid_line_scale(visual.axis, visual.span, thickness);
        transform.scale = scale;
        base_scale.0 = scale;
    }
}

pub(super) fn update_grid_line_lod_visibility(
    camera_mode: Res<ViewerCameraMode>,
    config: Res<Viewer3dConfig>,
    overlay_config: Res<WorldOverlayConfig>,
    cameras: Query<&Transform, With<Viewer3dCamera>>,
    mut grid_lines: Query<(&GridLineVisual, &GlobalTransform, &mut Visibility)>,
    mut last_camera_position: Local<Option<Vec3>>,
) {
    let Ok(camera_transform) = cameras.single() else {
        return;
    };

    let camera_position = camera_transform.translation;
    let lod_distance = config.render_budget.grid_lod_distance.max(1.0);
    let move_threshold = (lod_distance * 0.04).max(0.5);
    let camera_moved = last_camera_position
        .as_ref()
        .map(|last| last.distance(camera_position) > move_threshold)
        .unwrap_or(true);
    let should_recompute = camera_mode.is_changed()
        || config.is_changed()
        || overlay_config.is_changed()
        || camera_moved;
    if !should_recompute {
        return;
    }
    *last_camera_position = Some(camera_position);

    let mode_factor = match *camera_mode {
        ViewerCameraMode::TwoD => 1.35,
        ViewerCameraMode::ThreeD => 1.0,
    };
    let chunk_cutoff = lod_distance * mode_factor;

    for (visual, line_transform, mut visibility) in &mut grid_lines {
        if visual.kind == crate::GridLineKind::World {
            *visibility = Visibility::Visible;
            continue;
        }
        if !overlay_config.show_chunk_overlay {
            *visibility = Visibility::Hidden;
            continue;
        }

        let distance = camera_position.distance(line_transform.translation());
        *visibility = if distance <= chunk_cutoff {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
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

pub(super) fn sync_two_d_map_marker_visibility(
    camera_mode: Res<ViewerCameraMode>,
    mut query: Query<&mut Visibility, With<TwoDMapMarker>>,
) {
    let next_visibility = match *camera_mode {
        ViewerCameraMode::TwoD => Visibility::Visible,
        ViewerCameraMode::ThreeD => Visibility::Hidden,
    };
    for mut visibility in &mut query {
        *visibility = next_visibility;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GridLineKind;

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
    fn normalized_mouse_wheel_delta_converts_pixel_to_line_scale() {
        let line = normalized_mouse_wheel_delta(MouseScrollUnit::Line, 1.5);
        let pixel = normalized_mouse_wheel_delta(
            MouseScrollUnit::Pixel,
            MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR * 1.5,
        );
        assert!((line - pixel).abs() < f32::EPSILON);
    }

    #[test]
    fn pinch_scroll_delta_expands_small_magnify_values() {
        let zoom_in = pinch_scroll_delta(0.25);
        let zoom_out = pinch_scroll_delta(-0.25);
        assert!(zoom_in > 0.0);
        assert!(zoom_out < 0.0);
        assert!((zoom_in + zoom_out).abs() < f32::EPSILON);
    }

    #[test]
    fn two_d_ortho_scale_decreases_when_radius_decreases() {
        let cm_to_unit = Viewer3dConfig::default().effective_cm_to_unit();
        let reference = two_d_reference_radius(cm_to_unit);
        let zoom_in_scale =
            two_d_ortho_scale_for_radius((reference * 0.5).max(ORBIT_MIN_RADIUS), cm_to_unit);
        let zoom_out_scale =
            two_d_ortho_scale_for_radius((reference * 1.5).min(ORBIT_MAX_RADIUS), cm_to_unit);
        assert!(zoom_in_scale < zoom_out_scale);
    }

    #[test]
    fn sync_2d_zoom_projection_updates_orthographic_scale() {
        let config = Viewer3dConfig::default();
        let cm_to_unit = config.effective_cm_to_unit();
        let mut projection = camera_projection_for_mode(ViewerCameraMode::TwoD, &config);
        let before = match &projection {
            Projection::Orthographic(ortho) => ortho.scale,
            _ => panic!("expected orthographic projection"),
        };

        let zoom_in_radius = (two_d_reference_radius(cm_to_unit) * 0.6).max(ORBIT_MIN_RADIUS);
        sync_2d_zoom_projection(&mut projection, zoom_in_radius, cm_to_unit);
        let after = match &projection {
            Projection::Orthographic(ortho) => ortho.scale,
            _ => panic!("expected orthographic projection"),
        };
        assert!(after < before);
    }

    #[test]
    fn two_d_ortho_scale_supports_large_zoom_out() {
        let cm_to_unit = 0.0002;
        let reference = two_d_reference_radius(cm_to_unit);
        let far_zoom_out_scale =
            two_d_ortho_scale_for_radius((reference * 4.0).min(ORBIT_MAX_RADIUS), cm_to_unit);
        assert!(far_zoom_out_scale > 8.0);
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
    fn apply_orbit_input_zoom_out_clamps_to_expanded_max_radius() {
        let mut orbit = OrbitCamera {
            focus: Vec3::ZERO,
            radius: 100.0,
            yaw: 0.0,
            pitch: 0.0,
        };

        let changed = apply_orbit_input(
            &mut orbit,
            Vec2::ZERO,
            -1_000.0,
            false,
            false,
            ViewerCameraMode::TwoD,
        );
        assert!(changed);
        assert!(ORBIT_MAX_RADIUS > 300.0);
        assert!((orbit.radius - ORBIT_MAX_RADIUS).abs() < f32::EPSILON);
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
    fn grid_line_thickness_uses_mode_and_kind() {
        let world_2d = grid_line_thickness(GridLineKind::World, ViewerCameraMode::TwoD);
        let world_3d = grid_line_thickness(GridLineKind::World, ViewerCameraMode::ThreeD);
        let chunk_2d = grid_line_thickness(GridLineKind::Chunk, ViewerCameraMode::TwoD);
        let chunk_3d = grid_line_thickness(GridLineKind::Chunk, ViewerCameraMode::ThreeD);

        assert!(world_2d < world_3d);
        assert!(chunk_2d < chunk_3d);
        assert!(chunk_2d > world_2d);
        assert!(chunk_3d > world_3d);
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

    #[test]
    fn two_d_map_marker_visibility_follows_camera_mode() {
        let mut app = App::new();
        app.add_systems(Update, sync_two_d_map_marker_visibility);
        app.insert_resource(ViewerCameraMode::TwoD);

        let marker = app
            .world_mut()
            .spawn((TwoDMapMarker, Visibility::Hidden))
            .id();

        app.update();
        let visibility = app
            .world()
            .get::<Visibility>(marker)
            .expect("marker visibility");
        assert_eq!(*visibility, Visibility::Visible);

        *app.world_mut().resource_mut::<ViewerCameraMode>() = ViewerCameraMode::ThreeD;
        app.update();
        let visibility = app
            .world()
            .get::<Visibility>(marker)
            .expect("marker visibility");
        assert_eq!(*visibility, Visibility::Hidden);
    }

    #[test]
    fn grid_line_lod_hides_far_chunk_lines_and_keeps_world_lines() {
        let mut app = App::new();
        app.add_systems(Update, update_grid_line_lod_visibility);
        app.insert_resource(ViewerCameraMode::ThreeD);
        app.insert_resource(Viewer3dConfig::default());
        app.insert_resource(WorldOverlayConfig::default());

        app.world_mut().spawn((
            Viewer3dCamera,
            Transform::from_xyz(0.0, 0.0, 0.0),
            GlobalTransform::default(),
        ));

        let world_line = app
            .world_mut()
            .spawn((
                GridLineVisual {
                    kind: GridLineKind::World,
                    axis: crate::GridLineAxis::AlongX,
                    span: 10.0,
                },
                GlobalTransform::from_translation(Vec3::new(0.0, 0.0, -200.0)),
                Visibility::Visible,
            ))
            .id();

        let chunk_line = app
            .world_mut()
            .spawn((
                GridLineVisual {
                    kind: GridLineKind::Chunk,
                    axis: crate::GridLineAxis::AlongX,
                    span: 10.0,
                },
                GlobalTransform::from_translation(Vec3::new(0.0, 0.0, -220.0)),
                Visibility::Visible,
            ))
            .id();

        app.update();

        let world_visibility = app
            .world()
            .get::<Visibility>(world_line)
            .expect("world visibility");
        let chunk_visibility = app
            .world()
            .get::<Visibility>(chunk_line)
            .expect("chunk visibility");

        assert_eq!(*world_visibility, Visibility::Visible);
        assert_eq!(*chunk_visibility, Visibility::Hidden);
    }
}
