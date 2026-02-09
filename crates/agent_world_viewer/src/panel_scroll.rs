#[cfg(test)]
use bevy::input::mouse::MouseScrollUnit;
use bevy::prelude::*;

#[derive(Component)]
pub(super) struct RightPanelScroll;

#[derive(Component)]
pub(super) struct TopPanelScroll;

#[cfg(test)]
pub(super) fn scroll_delta_px_from_parts(unit: MouseScrollUnit, y: f32) -> f32 {
    let scale = match unit {
        MouseScrollUnit::Line => 32.0,
        MouseScrollUnit::Pixel => 1.0,
    };
    y * scale
}

#[cfg(test)]
fn cursor_in_scroll_node(
    cursor: Vec2,
    node: &bevy::ui::ComputedNode,
    transform: &bevy::ui::UiGlobalTransform,
    inherited_visibility: Option<&InheritedVisibility>,
) -> bool {
    inherited_visibility.is_none_or(|v| v.get())
        && !node.is_empty()
        && node.contains_point(*transform, cursor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_delta_uses_line_and_pixel_units() {
        assert_eq!(scroll_delta_px_from_parts(MouseScrollUnit::Line, 2.0), 64.0);
        assert_eq!(scroll_delta_px_from_parts(MouseScrollUnit::Pixel, 2.0), 2.0);
    }

    #[test]
    fn cursor_in_scroll_node_respects_visibility_and_bounds() {
        let node = bevy::ui::ComputedNode {
            size: Vec2::new(200.0, 100.0),
            ..default()
        };
        let transform = bevy::ui::UiGlobalTransform::from_translation(Vec2::new(100.0, 100.0));

        assert!(cursor_in_scroll_node(
            Vec2::new(100.0, 100.0),
            &node,
            &transform,
            None
        ));

        let hidden = InheritedVisibility::HIDDEN;
        assert!(!cursor_in_scroll_node(
            Vec2::new(100.0, 100.0),
            &node,
            &transform,
            Some(&hidden)
        ));

        assert!(!cursor_in_scroll_node(
            Vec2::new(400.0, 400.0),
            &node,
            &transform,
            None
        ));
    }
}
