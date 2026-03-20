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
