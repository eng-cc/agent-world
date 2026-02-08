use bevy::prelude::*;

use super::{OrbitCamera, Viewer3dCamera, Viewer3dConfig, Viewer3dScene};

pub(super) fn update_floating_origin(
    config: Res<Viewer3dConfig>,
    mut scene: ResMut<Viewer3dScene>,
    mut root_query: Query<&mut Transform>,
    mut camera_query: Query<(&mut OrbitCamera, &mut Transform), With<Viewer3dCamera>>,
) {
    let Some(root_entity) = scene.root_entity else {
        return;
    };
    let Ok(mut root_transform) = root_query.get_mut(root_entity) else {
        return;
    };
    let Ok((mut orbit, mut camera_transform)) = camera_query.single_mut() else {
        return;
    };

    let step = config.physical.floating_origin_step_m.max(1.0) as f32;
    let Some((next_offset, next_focus, next_camera_translation)) = rebase_if_needed(
        config.physical.enabled,
        step,
        scene.floating_origin_offset,
        orbit.focus,
        camera_transform.translation,
    ) else {
        root_transform.translation = -scene.floating_origin_offset;
        return;
    };

    scene.floating_origin_offset = next_offset;
    orbit.focus = next_focus;
    camera_transform.translation = next_camera_translation;
    root_transform.translation = -next_offset;
}

fn rebase_if_needed(
    enabled: bool,
    step: f32,
    current_offset: Vec3,
    focus: Vec3,
    camera_translation: Vec3,
) -> Option<(Vec3, Vec3, Vec3)> {
    if !enabled {
        if current_offset == Vec3::ZERO {
            return None;
        }
        return Some((
            Vec3::ZERO,
            focus + current_offset,
            camera_translation + current_offset,
        ));
    }

    let distance = (focus - current_offset).length();
    if distance < step {
        return None;
    }

    let snapped = Vec3::new(
        snap(focus.x, step),
        snap(focus.y, step),
        snap(focus.z, step),
    );
    if snapped == current_offset {
        return None;
    }

    let delta = snapped - current_offset;
    Some((snapped, focus - delta, camera_translation - delta))
}

fn snap(value: f32, step: f32) -> f32 {
    (value / step).round() * step
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snap_rounds_to_step_grid() {
        assert!((snap(1499.0, 1000.0) - 1000.0).abs() < f32::EPSILON);
        assert!((snap(1501.0, 1000.0) - 2000.0).abs() < f32::EPSILON);
        assert!((snap(-1499.0, 1000.0) - -1000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn rebase_enabled_snaps_and_shifts_camera_space() {
        let result = rebase_if_needed(
            true,
            1000.0,
            Vec3::ZERO,
            Vec3::new(1700.0, 0.0, -2300.0),
            Vec3::new(1730.0, 20.0, -2280.0),
        )
        .expect("rebase expected");

        assert_eq!(result.0, Vec3::new(2000.0, 0.0, -2000.0));
        assert_eq!(result.1, Vec3::new(-300.0, 0.0, -300.0));
        assert_eq!(result.2, Vec3::new(-270.0, 20.0, -280.0));
    }

    #[test]
    fn rebase_disabled_restores_world_space_camera() {
        let result = rebase_if_needed(
            false,
            1000.0,
            Vec3::new(2000.0, 0.0, 0.0),
            Vec3::new(-200.0, 0.0, 50.0),
            Vec3::new(-170.0, 15.0, 60.0),
        )
        .expect("restore expected");

        assert_eq!(result.0, Vec3::ZERO);
        assert_eq!(result.1, Vec3::new(1800.0, 0.0, 50.0));
        assert_eq!(result.2, Vec3::new(1830.0, 15.0, 60.0));
    }
}
