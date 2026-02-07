use bevy::ecs::message::MessageReader;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::{
    OrbitCamera, Viewer3dCamera, ORBIT_MAX_RADIUS, ORBIT_MIN_RADIUS, ORBIT_PAN_SENSITIVITY,
    ORBIT_ROTATE_SENSITIVITY, ORBIT_ZOOM_SENSITIVITY, UI_PANEL_WIDTH,
};

#[derive(Resource, Default)]
pub(super) struct OrbitDragState {
    last_cursor_position: Option<Vec2>,
}

pub(super) fn orbit_camera_controls(
    windows: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut drag_state: ResMut<OrbitDragState>,
    mut query: Query<(&mut OrbitCamera, &mut Transform), With<Viewer3dCamera>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    let cursor_position = window.cursor_position();
    let cursor_in_3d = cursor_position
        .map(|cursor| cursor_in_3d_view(window, cursor))
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
    );

    if changed {
        orbit.apply_to_transform(&mut transform);
    }
}

fn cursor_in_3d_view(window: &Window, cursor: Vec2) -> bool {
    let viewport_width = (window.width() - UI_PANEL_WIDTH).max(0.0);
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
) -> bool {
    let mut changed = false;

    if rotate_drag && delta != Vec2::ZERO {
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

        let changed = apply_orbit_input(&mut orbit, Vec2::new(6.0, -4.0), 1.0, false, true);
        assert!(changed);
        assert_ne!(orbit.focus, Vec3::ZERO);
        assert!(orbit.radius < 20.0);
    }
}
