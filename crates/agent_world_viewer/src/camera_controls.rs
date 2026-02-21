use bevy::ecs::message::MessageReader;
use bevy::input::gestures::PinchGesture;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

use super::{
    grid_line_scale, grid_line_thickness, AutoFocusState, BaseScale, DetailZoomEntity,
    GridLineVisual, OrbitCamera, RightPanelWidthState, TwoDMapMarker, Viewer3dCamera,
    Viewer3dConfig, ViewerCameraMode, WorldBoundsSurface, WorldFloorSurface, WorldOverlayConfig,
    DEFAULT_2D_CAMERA_RADIUS, DEFAULT_3D_CAMERA_RADIUS, ORBIT_MAX_RADIUS, ORBIT_MIN_RADIUS,
    ORBIT_PAN_SENSITIVITY, ORBIT_ROTATE_SENSITIVITY, ORBIT_ZOOM_SENSITIVITY, ORTHO_MAX_SCALE,
    ORTHO_MIN_SCALE, UI_PANEL_WIDTH,
};

const TWO_D_DEFAULT_DETAIL_RADIUS_RATIO: f32 = 0.0035;
const TWO_D_DETAIL_MIN_RADIUS_MULTIPLIER: f32 = 16.0;
const TWO_D_OVERVIEW_ENTER_DETAIL_MULTIPLIER: f32 = 14.0;
const TWO_D_OVERVIEW_EXIT_DETAIL_MULTIPLIER: f32 = 9.0;
const TWO_D_MIN_RADIUS_RATIO: f32 = 0.0001;
const TWO_D_OVERVIEW_MARKER_MIN_BOOST: f32 = 8.0;
const TWO_D_OVERVIEW_MARKER_MAX_BOOST: f32 = 4096.0;
const ORBIT_KEYBOARD_PAN_SPEED_MULTIPLIER: f32 = 12.0;
const ORBIT_KEYBOARD_PAN_BOOST_MULTIPLIER: f32 = 2.0;
const ORBIT_KEYBOARD_PAN_MIN_SPEED: f32 = 0.2;

#[derive(Resource, Default)]
pub(super) struct OrbitDragState {
    last_cursor_position: Option<Vec2>,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(super) enum TwoDZoomTier {
    #[default]
    Detail,
    Overview,
}

pub(super) fn orbit_camera_controls(
    windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    camera_mode: Res<ViewerCameraMode>,
    config: Res<Viewer3dConfig>,
    panel_width: Option<Res<RightPanelWidthState>>,
    mut egui_contexts: EguiContexts,
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
    let keyboard_captured_by_ui = egui_contexts
        .ctx_mut()
        .ok()
        .map(|ctx| ctx.wants_keyboard_input())
        .unwrap_or(false);
    let keyboard_axis = if cursor_in_3d && !keyboard_captured_by_ui {
        wasd_axis(&keys)
    } else {
        Vec2::ZERO
    };

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

    if delta == Vec2::ZERO && scroll == 0.0 && keyboard_axis == Vec2::ZERO {
        return;
    }

    let Ok((mut orbit, mut transform, mut projection)) = query.single_mut() else {
        return;
    };

    let cm_to_unit = config.effective_cm_to_unit();
    let min_radius = orbit_min_radius(cm_to_unit);
    let changed = apply_orbit_input(
        &mut orbit,
        delta,
        scroll,
        rotate_drag && dragging,
        pan_drag && dragging,
        *camera_mode,
        min_radius,
        ORBIT_MAX_RADIUS,
    );
    let keyboard_changed =
        apply_keyboard_pan(&mut orbit, keyboard_axis, time.delta_secs(), shift_pressed);

    if changed || keyboard_changed {
        orbit.apply_to_transform(&mut transform);
    }

    if scroll != 0.0 && *camera_mode == ViewerCameraMode::TwoD {
        sync_2d_zoom_projection(&mut projection, orbit.radius, cm_to_unit);
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

fn wasd_axis(keys: &ButtonInput<KeyCode>) -> Vec2 {
    Vec2::new(
        key_axis(keys, KeyCode::KeyD, KeyCode::KeyA),
        key_axis(keys, KeyCode::KeyW, KeyCode::KeyS),
    )
}

fn key_axis(keys: &ButtonInput<KeyCode>, positive: KeyCode, negative: KeyCode) -> f32 {
    let positive = if keys.pressed(positive) { 1.0 } else { 0.0 };
    let negative = if keys.pressed(negative) { 1.0 } else { 0.0 };
    positive - negative
}

fn flatten_xz(direction: Vec3) -> Vec3 {
    Vec3::new(direction.x, 0.0, direction.z)
}

fn apply_keyboard_pan(
    orbit: &mut OrbitCamera,
    axis: Vec2,
    delta_secs: f32,
    speed_boost: bool,
) -> bool {
    if axis == Vec2::ZERO || !delta_secs.is_finite() || delta_secs <= 0.0 {
        return false;
    }

    let rotation =
        Quat::from_axis_angle(Vec3::Y, orbit.yaw) * Quat::from_axis_angle(Vec3::X, orbit.pitch);
    let right = flatten_xz(rotation * Vec3::X);
    let forward = flatten_xz(rotation * -Vec3::Z);
    let Some(right) = right.try_normalize() else {
        return false;
    };
    let Some(forward) = forward.try_normalize() else {
        return false;
    };

    let movement_direction = axis.x * right + axis.y * forward;
    let Some(movement_direction) = movement_direction.try_normalize() else {
        return false;
    };

    let mut speed = (orbit.radius * ORBIT_PAN_SENSITIVITY * ORBIT_KEYBOARD_PAN_SPEED_MULTIPLIER)
        .max(ORBIT_KEYBOARD_PAN_MIN_SPEED);
    if speed_boost {
        speed *= ORBIT_KEYBOARD_PAN_BOOST_MULTIPLIER;
    }
    orbit.focus += movement_direction * speed * delta_secs;
    true
}

pub(super) fn sync_2d_zoom_projection(
    projection: &mut Projection,
    orbit_radius: f32,
    cm_to_unit: f32,
) {
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
    min_radius: f32,
    max_radius: f32,
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
        orbit.radius =
            (orbit.radius * (1.0 - scroll * ORBIT_ZOOM_SENSITIVITY)).clamp(min_radius, max_radius);
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
    let min_radius = orbit_min_radius(cm_to_unit);
    match mode {
        ViewerCameraMode::TwoD => {
            let detail_radius = two_d_detail_default_radius(cm_to_unit);
            OrbitCamera {
                focus,
                radius: detail_radius.max(min_radius),
                yaw: 0.0,
                pitch: -1.53,
            }
        }
        ViewerCameraMode::ThreeD => OrbitCamera {
            focus,
            radius: DEFAULT_3D_CAMERA_RADIUS.clamp(min_radius, ORBIT_MAX_RADIUS),
            yaw: -0.7,
            pitch: 0.55,
        },
    }
}

pub(super) fn camera_projection_for_mode(
    mode: ViewerCameraMode,
    config: &Viewer3dConfig,
) -> Projection {
    let cm_to_unit = config.effective_cm_to_unit();
    let (near, far) = camera_clip_planes(cm_to_unit, config);
    match mode {
        ViewerCameraMode::TwoD => {
            let scale = world_view_ortho_scale(cm_to_unit);
            Projection::Orthographic(OrthographicProjection {
                near,
                far,
                scale,
                ..OrthographicProjection::default_3d()
            })
        }
        ViewerCameraMode::ThreeD => Projection::Perspective(PerspectiveProjection {
            near,
            far,
            ..default()
        }),
    }
}

fn world_units_per_meter(cm_to_unit: f32) -> f32 {
    cm_to_unit.max(f32::EPSILON) * 100.0
}

pub(super) fn orbit_min_radius(cm_to_unit: f32) -> f32 {
    (ORBIT_MIN_RADIUS * world_units_per_meter(cm_to_unit)).clamp(0.0001, ORBIT_MAX_RADIUS)
}

fn camera_clip_planes(cm_to_unit: f32, config: &Viewer3dConfig) -> (f32, f32) {
    let units_per_meter = world_units_per_meter(cm_to_unit);
    let near = (config.physical.camera_near_m * units_per_meter).max(0.00001);
    let scaled_far = config.physical.camera_far_m * units_per_meter;
    let fallback_far = world_view_radius(cm_to_unit).max(DEFAULT_3D_CAMERA_RADIUS) * 4.0;
    let far = scaled_far.max(fallback_far).max(near + 0.01);
    (near, far)
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
        .max(orbit_min_radius(cm_to_unit))
}

fn two_d_ortho_scale_for_radius(radius: f32, cm_to_unit: f32) -> f32 {
    let base_scale = world_view_ortho_scale(cm_to_unit);
    let reference_radius = two_d_reference_radius(cm_to_unit);
    let ratio = (radius / reference_radius).max(TWO_D_MIN_RADIUS_RATIO);
    (base_scale * ratio).clamp(ORTHO_MIN_SCALE, ORTHO_MAX_SCALE)
}

fn two_d_detail_default_radius(cm_to_unit: f32) -> f32 {
    let min_radius = orbit_min_radius(cm_to_unit);
    (world_view_radius(cm_to_unit) * TWO_D_DEFAULT_DETAIL_RADIUS_RATIO).clamp(
        min_radius * TWO_D_DETAIL_MIN_RADIUS_MULTIPLIER,
        ORBIT_MAX_RADIUS,
    )
}

fn two_d_overview_thresholds(cm_to_unit: f32) -> (f32, f32) {
    let min_radius = orbit_min_radius(cm_to_unit);
    let detail_radius = two_d_detail_default_radius(cm_to_unit);
    let enter = (detail_radius * TWO_D_OVERVIEW_ENTER_DETAIL_MULTIPLIER)
        .clamp(min_radius * 8.0, ORBIT_MAX_RADIUS);
    let mut exit = (detail_radius * TWO_D_OVERVIEW_EXIT_DETAIL_MULTIPLIER)
        .clamp(min_radius * 6.0, ORBIT_MAX_RADIUS);
    if exit >= enter {
        exit = (enter * 0.82).max(min_radius);
    }
    (exit, enter)
}

fn two_d_zoom_tier_for_radius(radius: f32, cm_to_unit: f32, current: TwoDZoomTier) -> TwoDZoomTier {
    let (exit, enter) = two_d_overview_thresholds(cm_to_unit);
    match current {
        TwoDZoomTier::Detail if radius >= enter => TwoDZoomTier::Overview,
        TwoDZoomTier::Overview if radius <= exit => TwoDZoomTier::Detail,
        _ => current,
    }
}

pub(super) fn sync_two_d_zoom_tier(
    camera_mode: Res<ViewerCameraMode>,
    config: Res<Viewer3dConfig>,
    cameras: Query<&OrbitCamera, With<Viewer3dCamera>>,
    mut zoom_tier: ResMut<TwoDZoomTier>,
) {
    let next = match *camera_mode {
        ViewerCameraMode::TwoD => {
            let Ok(orbit) = cameras.single() else {
                return;
            };
            two_d_zoom_tier_for_radius(orbit.radius, config.effective_cm_to_unit(), *zoom_tier)
        }
        ViewerCameraMode::ThreeD => TwoDZoomTier::Detail,
    };

    if *zoom_tier != next {
        *zoom_tier = next;
    }
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

    let projection_matches_mode = match (*camera_mode, &*projection) {
        (ViewerCameraMode::TwoD, Projection::Orthographic(_)) => true,
        (ViewerCameraMode::ThreeD, Projection::Perspective(_)) => true,
        _ => false,
    };
    if projection_matches_mode {
        return;
    }

    let next_orbit = camera_orbit_preset(
        *camera_mode,
        Some(orbit.focus),
        config.effective_cm_to_unit(),
    );
    *orbit = next_orbit;
    orbit.apply_to_transform(&mut transform);
    *projection = camera_projection_for_mode(*camera_mode, &config);
    if *camera_mode == ViewerCameraMode::TwoD {
        sync_2d_zoom_projection(&mut projection, orbit.radius, config.effective_cm_to_unit());
    }

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
    zoom_tier: Res<TwoDZoomTier>,
    mut query: Query<&mut Visibility, With<TwoDMapMarker>>,
) {
    let next_visibility = match (*camera_mode, *zoom_tier) {
        (ViewerCameraMode::TwoD, TwoDZoomTier::Overview) => Visibility::Visible,
        _ => Visibility::Hidden,
    };
    for mut visibility in &mut query {
        *visibility = next_visibility;
    }
}

pub(super) fn sync_two_d_map_marker_scale(
    camera_mode: Res<ViewerCameraMode>,
    zoom_tier: Res<TwoDZoomTier>,
    config: Res<Viewer3dConfig>,
    cameras: Query<&OrbitCamera, With<Viewer3dCamera>>,
    mut query: Query<(&mut Transform, &BaseScale), With<TwoDMapMarker>>,
) {
    let cm_to_unit = config.effective_cm_to_unit();
    let boost = if matches!(
        (*camera_mode, *zoom_tier),
        (ViewerCameraMode::TwoD, TwoDZoomTier::Overview)
    ) {
        let Ok(orbit) = cameras.single() else {
            return;
        };
        let detail_radius =
            two_d_detail_default_radius(cm_to_unit).max(orbit_min_radius(cm_to_unit));
        (orbit.radius / detail_radius)
            .clamp(1.0, TWO_D_OVERVIEW_MARKER_MAX_BOOST)
            .max(TWO_D_OVERVIEW_MARKER_MIN_BOOST)
    } else {
        1.0
    };

    for (mut transform, base_scale) in &mut query {
        transform.scale = base_scale.0 * boost;
    }
}

pub(super) fn sync_detail_zoom_visibility(
    camera_mode: Res<ViewerCameraMode>,
    zoom_tier: Res<TwoDZoomTier>,
    mut query: Query<&mut Visibility, With<DetailZoomEntity>>,
) {
    let detail_visible = !matches!(
        (*camera_mode, *zoom_tier),
        (ViewerCameraMode::TwoD, TwoDZoomTier::Overview)
    );
    let next_visibility = if detail_visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
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
    fn wasd_axis_maps_pressed_keys() {
        let mut keys = ButtonInput::<KeyCode>::default();
        assert_eq!(wasd_axis(&keys), Vec2::ZERO);

        keys.press(KeyCode::KeyW);
        keys.press(KeyCode::KeyD);
        assert_eq!(wasd_axis(&keys), Vec2::new(1.0, 1.0));

        keys.release(KeyCode::KeyW);
        keys.press(KeyCode::KeyS);
        assert_eq!(wasd_axis(&keys), Vec2::new(1.0, -1.0));

        keys.press(KeyCode::KeyA);
        keys.release(KeyCode::KeyD);
        assert_eq!(wasd_axis(&keys), Vec2::new(-1.0, -1.0));
    }

    #[test]
    fn cursor_in_3d_view_respects_right_panel_bound() {
        let mut window = Window::default();
        window.resolution.set(1200.0, 800.0);

        assert!(cursor_in_3d_view(&window, Vec2::new(879.5, 100.0), 320.0));
        assert!(!cursor_in_3d_view(&window, Vec2::new(880.5, 100.0), 320.0));
    }

    #[test]
    fn two_d_ortho_scale_decreases_when_radius_decreases() {
        let cm_to_unit = Viewer3dConfig::default().effective_cm_to_unit();
        let reference = two_d_reference_radius(cm_to_unit);
        let zoom_in_scale = two_d_ortho_scale_for_radius(
            (reference * 0.5).max(orbit_min_radius(cm_to_unit)),
            cm_to_unit,
        );
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

        let zoom_in_radius =
            (two_d_reference_radius(cm_to_unit) * 0.6).max(orbit_min_radius(cm_to_unit));
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
    fn two_d_detail_default_zoom_is_deep_enough_for_agent_readability() {
        let cm_to_unit = Viewer3dConfig::default().effective_cm_to_unit();
        let detail_radius = two_d_detail_default_radius(cm_to_unit);
        let detail_scale = two_d_ortho_scale_for_radius(detail_radius, cm_to_unit);
        assert!(detail_scale < 0.001);
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
            ORBIT_MIN_RADIUS,
            ORBIT_MAX_RADIUS,
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
            ORBIT_MIN_RADIUS,
            ORBIT_MAX_RADIUS,
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
            ORBIT_MIN_RADIUS,
            ORBIT_MAX_RADIUS,
        );

        assert!(!changed);
        assert!((orbit.yaw - 1.0).abs() < f32::EPSILON);
        assert!((orbit.pitch + 1.53).abs() < f32::EPSILON);
    }

    #[test]
    fn apply_keyboard_pan_two_d_moves_focus_on_horizontal_plane() {
        let mut orbit = OrbitCamera {
            focus: Vec3::new(0.0, 3.0, 0.0),
            radius: 90.0,
            yaw: 0.0,
            pitch: -1.53,
        };

        let changed = apply_keyboard_pan(&mut orbit, Vec2::new(0.0, 1.0), 1.0, false);
        assert!(changed);
        assert!((orbit.focus.y - 3.0).abs() < 1e-6);
        assert!(orbit.focus.z < 0.0);
    }

    #[test]
    fn apply_keyboard_pan_three_d_follows_camera_heading() {
        let mut orbit = OrbitCamera {
            focus: Vec3::ZERO,
            radius: 48.0,
            yaw: std::f32::consts::FRAC_PI_2,
            pitch: 0.55,
        };

        let changed = apply_keyboard_pan(&mut orbit, Vec2::new(0.0, 1.0), 1.0, false);
        assert!(changed);
        assert!(orbit.focus.x < 0.0);
        assert!(orbit.focus.z.abs() < orbit.focus.x.abs());
    }

    #[test]
    fn apply_keyboard_pan_shift_boost_moves_faster() {
        let mut normal = OrbitCamera {
            focus: Vec3::ZERO,
            radius: 64.0,
            yaw: -0.7,
            pitch: 0.55,
        };
        let mut boosted = OrbitCamera {
            focus: Vec3::ZERO,
            radius: 64.0,
            yaw: -0.7,
            pitch: 0.55,
        };

        let normal_changed = apply_keyboard_pan(&mut normal, Vec2::new(1.0, 0.0), 1.0, false);
        let boosted_changed = apply_keyboard_pan(&mut boosted, Vec2::new(1.0, 0.0), 1.0, true);
        assert!(normal_changed && boosted_changed);
        assert!(boosted.focus.distance(Vec3::ZERO) > normal.focus.distance(Vec3::ZERO));
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
    fn camera_projection_scales_near_and_keeps_far_covering_world() {
        let config = Viewer3dConfig::default();
        let units_per_meter = config.effective_cm_to_unit() * 100.0;
        let expected_near = config.physical.camera_near_m * units_per_meter;

        let projection = camera_projection_for_mode(ViewerCameraMode::ThreeD, &config);
        let Projection::Perspective(perspective) = projection else {
            panic!("expected perspective projection");
        };
        assert!((perspective.near - expected_near).abs() < 1e-6);
        assert!(perspective.far >= world_view_radius(config.effective_cm_to_unit()));
    }

    #[test]
    fn orbit_min_radius_scales_with_world_units() {
        let config = Viewer3dConfig::default();
        let min_radius = orbit_min_radius(config.effective_cm_to_unit());
        assert!((min_radius - 0.004).abs() < 1e-6);
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
        let cm_to_unit = Viewer3dConfig::default().effective_cm_to_unit();
        let orbit = camera_orbit_preset(ViewerCameraMode::TwoD, None, cm_to_unit);
        assert!(orbit.pitch < -1.5);
        assert!(orbit.radius >= orbit_min_radius(cm_to_unit));
        assert!(orbit.radius < world_view_radius(cm_to_unit));
    }

    #[test]
    fn sync_camera_mode_two_d_projection_matches_orbit_radius() {
        let mut app = App::new();
        app.add_systems(Update, sync_camera_mode);
        app.insert_resource(Viewer3dConfig::default());
        app.insert_resource(ViewerCameraMode::ThreeD);
        app.insert_resource(AutoFocusState::default());

        let config = *app.world().resource::<Viewer3dConfig>();
        let mut transform = Transform::default();
        let orbit = camera_orbit_preset(
            ViewerCameraMode::ThreeD,
            Some(Vec3::ZERO),
            config.effective_cm_to_unit(),
        );
        orbit.apply_to_transform(&mut transform);
        app.world_mut().spawn((
            Viewer3dCamera,
            orbit,
            transform,
            camera_projection_for_mode(ViewerCameraMode::ThreeD, &config),
        ));

        app.world_mut().insert_resource(ViewerCameraMode::TwoD);
        app.update();

        let mut query = app.world_mut().query::<(&OrbitCamera, &Projection)>();
        let (orbit, projection) = query
            .single(app.world())
            .expect("camera query should contain one entity");
        let Projection::Orthographic(ortho) = projection else {
            panic!("expected orthographic projection");
        };
        let expected = two_d_ortho_scale_for_radius(orbit.radius, config.effective_cm_to_unit());
        assert!((ortho.scale - expected).abs() < 1e-6);
    }

    #[test]
    fn sync_camera_mode_ignores_redundant_three_d_assignment() {
        let mut app = App::new();
        app.add_systems(Update, sync_camera_mode);
        app.insert_resource(Viewer3dConfig::default());
        app.insert_resource(ViewerCameraMode::ThreeD);
        app.insert_resource(AutoFocusState::default());

        let config = *app.world().resource::<Viewer3dConfig>();
        let custom_radius = 13.44;
        let mut transform = Transform::default();
        let orbit = OrbitCamera {
            focus: Vec3::new(10.0, 2.0, -8.0),
            radius: custom_radius,
            yaw: 0.35,
            pitch: -0.8,
        };
        orbit.apply_to_transform(&mut transform);
        app.world_mut().spawn((
            Viewer3dCamera,
            orbit,
            transform,
            camera_projection_for_mode(ViewerCameraMode::ThreeD, &config),
        ));

        app.world_mut().insert_resource(ViewerCameraMode::ThreeD);
        app.update();

        let mut query = app.world_mut().query::<&OrbitCamera>();
        let orbit = query
            .single(app.world())
            .expect("camera query should contain one entity");
        assert!((orbit.radius - custom_radius).abs() < 1e-6);
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
    fn two_d_zoom_tier_switches_with_hysteresis() {
        let cm_to_unit = Viewer3dConfig::default().effective_cm_to_unit();
        let (exit, enter) = two_d_overview_thresholds(cm_to_unit);

        let to_overview =
            two_d_zoom_tier_for_radius(enter + 0.01, cm_to_unit, TwoDZoomTier::Detail);
        assert_eq!(to_overview, TwoDZoomTier::Overview);

        let stay_overview =
            two_d_zoom_tier_for_radius((enter + exit) * 0.5, cm_to_unit, TwoDZoomTier::Overview);
        assert_eq!(stay_overview, TwoDZoomTier::Overview);

        let back_to_detail =
            two_d_zoom_tier_for_radius(exit - 0.01, cm_to_unit, TwoDZoomTier::Overview);
        assert_eq!(back_to_detail, TwoDZoomTier::Detail);
    }

    #[test]
    fn two_d_map_marker_visibility_follows_zoom_tier() {
        let mut app = App::new();
        app.add_systems(Update, sync_two_d_map_marker_visibility);
        app.insert_resource(ViewerCameraMode::TwoD);
        app.insert_resource(TwoDZoomTier::Detail);

        let marker = app
            .world_mut()
            .spawn((TwoDMapMarker, Visibility::Hidden))
            .id();

        app.update();
        let visibility = app
            .world()
            .get::<Visibility>(marker)
            .expect("marker visibility");
        assert_eq!(*visibility, Visibility::Hidden);

        *app.world_mut().resource_mut::<TwoDZoomTier>() = TwoDZoomTier::Overview;
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
    fn two_d_map_marker_scale_boosts_in_overview_and_resets_in_detail() {
        let mut app = App::new();
        app.add_systems(Update, sync_two_d_map_marker_scale);
        app.insert_resource(ViewerCameraMode::TwoD);
        app.insert_resource(TwoDZoomTier::Overview);
        app.insert_resource(Viewer3dConfig::default());

        let config = *app.world().resource::<Viewer3dConfig>();
        let cm_to_unit = config.effective_cm_to_unit();
        let mut camera_transform = Transform::default();
        let orbit = OrbitCamera {
            focus: Vec3::ZERO,
            radius: two_d_detail_default_radius(cm_to_unit) * 24.0,
            yaw: 0.0,
            pitch: -1.53,
        };
        orbit.apply_to_transform(&mut camera_transform);
        app.world_mut()
            .spawn((Viewer3dCamera, orbit, camera_transform));

        let base = Vec3::new(0.002, 0.0002, 0.002);
        let marker = app
            .world_mut()
            .spawn((
                TwoDMapMarker,
                BaseScale(base),
                Transform::from_scale(base),
                Visibility::Visible,
            ))
            .id();

        app.update();
        let boosted = app
            .world()
            .get::<Transform>(marker)
            .expect("marker transform in overview")
            .scale;
        assert!(boosted.x > base.x);

        *app.world_mut().resource_mut::<TwoDZoomTier>() = TwoDZoomTier::Detail;
        app.update();
        let reset = app
            .world()
            .get::<Transform>(marker)
            .expect("marker transform in detail")
            .scale;
        assert!((reset.x - base.x).abs() < 1e-6);
        assert!((reset.y - base.y).abs() < 1e-6);
        assert!((reset.z - base.z).abs() < 1e-6);
    }

    #[test]
    fn detail_zoom_visibility_hides_detail_entities_in_overview() {
        let mut app = App::new();
        app.add_systems(Update, sync_detail_zoom_visibility);
        app.insert_resource(ViewerCameraMode::TwoD);
        app.insert_resource(TwoDZoomTier::Detail);

        let detail_entity = app
            .world_mut()
            .spawn((DetailZoomEntity, Visibility::Visible))
            .id();

        app.update();
        let visibility = app
            .world()
            .get::<Visibility>(detail_entity)
            .expect("detail visibility");
        assert_eq!(*visibility, Visibility::Visible);

        *app.world_mut().resource_mut::<TwoDZoomTier>() = TwoDZoomTier::Overview;
        app.update();
        let visibility = app
            .world()
            .get::<Visibility>(detail_entity)
            .expect("detail visibility");
        assert_eq!(*visibility, Visibility::Hidden);

        *app.world_mut().resource_mut::<ViewerCameraMode>() = ViewerCameraMode::ThreeD;
        app.update();
        let visibility = app
            .world()
            .get::<Visibility>(detail_entity)
            .expect("detail visibility");
        assert_eq!(*visibility, Visibility::Visible);
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
