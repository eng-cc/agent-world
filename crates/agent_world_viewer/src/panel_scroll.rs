use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::panel_layout::RightPanelLayoutState;
use super::UI_PANEL_WIDTH;

const TOP_SCROLL_HIT_HEIGHT_PX: f32 = 430.0;

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

pub(super) fn cursor_in_right_panel(window_width: f32, cursor_x: f32) -> bool {
    cursor_x >= (window_width - UI_PANEL_WIDTH).max(0.0)
}

fn cursor_in_top_control_area(window_height: f32, cursor_y: f32) -> bool {
    cursor_y >= (window_height - TOP_SCROLL_HIT_HEIGHT_PX).max(0.0)
}

pub(super) fn scroll_right_panel(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut wheel_events: MessageReader<MouseWheel>,
    mut top_scroll_query: Query<
        &mut ScrollPosition,
        (With<TopPanelScroll>, Without<RightPanelScroll>),
    >,
    mut bottom_scroll_query: Query<
        &mut ScrollPosition,
        (With<RightPanelScroll>, Without<TopPanelScroll>),
    >,
    layout_state: Res<RightPanelLayoutState>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    if !cursor_in_right_panel(window.width(), cursor.x) {
        return;
    }

    let use_top_scroll =
        !layout_state.top_panel_collapsed && cursor_in_top_control_area(window.height(), cursor.y);

    for event in wheel_events.read() {
        let delta = scroll_delta_px(event);
        if delta.abs() < f32::EPSILON {
            continue;
        }

        if use_top_scroll {
            if let Ok(mut scroll) = top_scroll_query.single_mut() {
                scroll.y = (scroll.y - delta).max(0.0);
                continue;
            }
        }

        if let Ok(mut scroll) = bottom_scroll_query.single_mut() {
            scroll.y = (scroll.y - delta).max(0.0);
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
    fn cursor_in_right_panel_matches_panel_width() {
        assert!(cursor_in_right_panel(1200.0, 900.0));
        assert!(cursor_in_right_panel(1200.0, 820.0));
        assert!(!cursor_in_right_panel(1200.0, 819.0));
    }

    #[test]
    fn cursor_in_top_control_area_uses_top_band() {
        assert!(cursor_in_top_control_area(800.0, 780.0));
        assert!(cursor_in_top_control_area(800.0, 420.0));
        assert!(!cursor_in_top_control_area(800.0, 360.0));
    }
}
