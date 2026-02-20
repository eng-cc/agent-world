use super::*;
use agent_world::simulator::FragmentElementKind;

#[test]
fn location_visual_radius_cm_keeps_base_when_budget_missing() {
    assert_eq!(location_visual_radius_cm(900, None), 900);
}

#[test]
fn location_visual_radius_cm_tracks_remaining_mass_ratio() {
    let mut budget = FragmentResourceBudget::default();
    budget
        .total_by_element_g
        .insert(FragmentElementKind::Iron, 1_000);
    budget
        .remaining_by_element_g
        .insert(FragmentElementKind::Iron, 125);

    assert_eq!(location_visual_radius_cm(800, Some(&budget)), 400);

    budget
        .remaining_by_element_g
        .insert(FragmentElementKind::Iron, 0);
    let min_radius = location_visual_radius_cm(800, Some(&budget));
    assert_eq!(min_radius, 192);
}

#[test]
fn location_render_radius_units_scales_by_world_units_without_clamp() {
    let mapped = location_render_radius_units(500_000, 0.00001);
    assert!((mapped - 5.0).abs() < f32::EPSILON);

    let tiny = location_render_radius_units(100, 0.00001);
    assert!((tiny - 0.001).abs() < f32::EPSILON);

    let large = location_render_radius_units(10_000_000, 0.00001);
    assert!((large - 100.0).abs() < f32::EPSILON);
}

#[test]
fn should_apply_scale_highlight_skips_fragment() {
    assert!(should_apply_scale_highlight(SelectionKind::Agent));
    assert!(should_apply_scale_highlight(SelectionKind::Location));
    assert!(!should_apply_scale_highlight(SelectionKind::Fragment));
}
