use super::*;
use agent_world::simulator::RejectReason;

#[test]
fn events_summary_without_focus_keeps_compact_view() {
    let events = vec![WorldEvent {
        id: 1,
        time: 7,
        kind: WorldEventKind::ActionRejected {
            reason: RejectReason::InvalidAmount { amount: 1 },
        },
    }];

    let text = events_summary(&events, None);
    assert!(text.starts_with("Events:"));
    assert!(text.contains("#1 t7"));
    assert!(!text.contains("Events (focused):"));
}

#[test]
fn events_summary_with_focus_marks_nearest_context() {
    let events = vec![
        WorldEvent {
            id: 1,
            time: 3,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::InvalidAmount { amount: 1 },
            },
        },
        WorldEvent {
            id: 2,
            time: 8,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::InvalidAmount { amount: 2 },
            },
        },
        WorldEvent {
            id: 3,
            time: 11,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::InvalidAmount { amount: 3 },
            },
        },
    ];

    let text = events_summary(&events, Some(9));
    assert!(text.starts_with("Events (focused):"));
    assert!(text.contains("Focus: requested t9 -> nearest t8 (#2), Δt=1"));
    assert!(text.contains(">> #2 t8"));
}

#[test]
fn thermal_ratio_color_follows_design_thresholds() {
    assert_eq!(thermal_ratio_color(0.0), "heat_low");
    assert_eq!(thermal_ratio_color(0.6), "heat_low");
    assert_eq!(thermal_ratio_color(0.61), "heat_mid");
    assert_eq!(thermal_ratio_color(1.0), "heat_mid");
    assert_eq!(thermal_ratio_color(1.01), "heat_high");
}

#[test]
fn radiation_visual_metrics_convert_to_power_and_flux() {
    let (power, flux, area) = radiation_visual_metrics(12, 1_000, 10, 2.0);
    assert!((power - 1_200.0).abs() < f64::EPSILON);
    assert!((flux - 600.0).abs() < f64::EPSILON);
    assert!((area - 2.0).abs() < f64::EPSILON);

    let (_, fallback_flux, fallback_area) = radiation_visual_metrics(3, 500, 0, 0.0);
    assert!((fallback_area - 1.0).abs() < f64::EPSILON);
    assert!((fallback_flux - 1_500.0).abs() < f64::EPSILON);
}

#[test]
fn world_summary_includes_physical_render_block_when_enabled() {
    let mut physical = ViewerPhysicalRenderConfig::default();
    physical.enabled = true;
    physical.meters_per_unit = 1.0;
    physical.stellar_distance_au = 2.5;
    physical.exposure_ev100 = 13.5;
    physical.reference_radiation_area_m2 = 2.0;

    let summary = world_summary(None, None, Some(&physical));
    assert!(summary.contains("World: (no snapshot)"));
    assert!(summary.contains("Render Physical: on"));
    assert!(summary.contains("Unit: 1u=1.00m"));
    assert!(summary.contains("Camera Clip(m): near=0.10 far=25000"));
    assert!(summary.contains("Stellar Distance(AU): 2.50"));
    assert!(summary.contains("Irradiance(W/m²): 217.8"));
    assert!(summary.contains("Exposed Illuminance(lux): 26131"));
    assert!(summary.contains("Exposure(EV100): 13.50"));
    assert!(summary.contains("Radiation Ref Area(m²): 2.00"));
}

#[test]
fn world_summary_displays_physical_flag_when_disabled() {
    let physical = ViewerPhysicalRenderConfig::default();
    let summary = world_summary(None, None, Some(&physical));
    assert!(summary.contains("Render Physical: off"));
    assert!(!summary.contains("Unit: 1u="));
}
