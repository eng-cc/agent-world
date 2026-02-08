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
