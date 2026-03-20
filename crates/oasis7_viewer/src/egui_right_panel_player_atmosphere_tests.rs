use super::egui_right_panel_player_experience::build_player_atmosphere_snapshot;

#[test]
fn build_player_atmosphere_snapshot_clamps_expected_ranges() {
    let snapshot = build_player_atmosphere_snapshot(0.0);
    assert!((0.0..=0.28).contains(&snapshot.top_alpha));
    assert!((0.0..=0.25).contains(&snapshot.bottom_alpha));
    assert!((0.66..=0.82).contains(&snapshot.orb_x_factor));
    assert!((0.17..=0.27).contains(&snapshot.orb_y_factor));
    assert!((120.0..=162.0).contains(&snapshot.orb_radius));
}

#[test]
fn build_player_atmosphere_snapshot_changes_over_time() {
    let a = build_player_atmosphere_snapshot(0.0);
    let b = build_player_atmosphere_snapshot(9.5);
    assert_ne!(a, b);
}
