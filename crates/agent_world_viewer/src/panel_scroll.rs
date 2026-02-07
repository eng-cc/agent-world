use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use bevy::ui::{ComputedNode, UiGlobalTransform};

#[derive(Component)]
pub(super) struct RightPanelScroll;

#[derive(Component)]
pub(super) struct TopPanelScroll;

pub(super) fn scroll_delta_px_from_parts(unit: MouseScrollUnit, y: f32) -> f32 {
    let scale = match unit {
        MouseScrollUnit::Line => 32.0,
        MouseScrollUnit::Pixel => 1.0,
    };
    y * scale
}

fn scroll_delta_px(event: &MouseWheel) -> f32 {
    scroll_delta_px_from_parts(event.unit, event.y)
}

fn cursor_in_scroll_node(
    cursor: Vec2,
    node: &ComputedNode,
    transform: &UiGlobalTransform,
    inherited_visibility: Option<&InheritedVisibility>,
) -> bool {
    inherited_visibility.is_none_or(|v| v.get())
        && !node.is_empty()
        && node.contains_point(*transform, cursor)
}

pub(super) fn scroll_right_panel(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut wheel_events: MessageReader<MouseWheel>,
    mut top_scroll_query: Query<
        (
            &mut ScrollPosition,
            &ComputedNode,
            &UiGlobalTransform,
            Option<&InheritedVisibility>,
        ),
        (With<TopPanelScroll>, Without<RightPanelScroll>),
    >,
    mut bottom_scroll_query: Query<
        (
            &mut ScrollPosition,
            &ComputedNode,
            &UiGlobalTransform,
            Option<&InheritedVisibility>,
        ),
        (With<RightPanelScroll>, Without<TopPanelScroll>),
    >,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.physical_cursor_position().or_else(|| {
        window
            .cursor_position()
            .map(|pos| pos * window.scale_factor())
    }) else {
        return;
    };

    let mut top_scroll = top_scroll_query.single_mut().ok();
    let mut bottom_scroll = bottom_scroll_query.single_mut().ok();

    let use_top_scroll = top_scroll
        .as_ref()
        .is_some_and(|(_, node, transform, visibility)| {
            cursor_in_scroll_node(cursor, node, transform, *visibility)
        });
    let use_bottom_scroll =
        bottom_scroll
            .as_ref()
            .is_some_and(|(_, node, transform, visibility)| {
                cursor_in_scroll_node(cursor, node, transform, *visibility)
            });

    if !use_top_scroll && !use_bottom_scroll {
        return;
    }

    for event in wheel_events.read() {
        let delta = scroll_delta_px(event);
        if delta.abs() < f32::EPSILON {
            continue;
        }

        if use_top_scroll {
            if let Some((ref mut scroll, ..)) = top_scroll.as_mut() {
                scroll.y = (scroll.y - delta).max(0.0);
                continue;
            }
        }

        if use_bottom_scroll {
            if let Some((ref mut scroll, ..)) = bottom_scroll.as_mut() {
                scroll.y = (scroll.y - delta).max(0.0);
            }
        }
    }
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
        let node = ComputedNode {
            size: Vec2::new(200.0, 100.0),
            ..default()
        };
        let transform = UiGlobalTransform::from_translation(Vec2::new(100.0, 100.0));

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
