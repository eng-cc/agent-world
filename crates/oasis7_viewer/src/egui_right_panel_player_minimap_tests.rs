use super::egui_right_panel_player_guide::{
    build_player_minimap_points, resolve_selected_location_id_for_minimap,
};
use bevy::prelude::Entity;
use std::collections::HashMap;

#[test]
fn build_player_minimap_points_normalizes_and_marks_selected() {
    let raw_points = vec![
        ("loc-b".to_string(), 50.0, 100.0),
        ("loc-a".to_string(), -50.0, 0.0),
    ];

    let points = build_player_minimap_points(&raw_points, Some("loc-b"));

    assert_eq!(points.len(), 2);
    assert_eq!(points.iter().filter(|point| point.selected).count(), 1);
    assert!(points
        .iter()
        .all(|point| (0.0..=1.0).contains(&point.x) && (0.0..=1.0).contains(&point.y)));
    assert!((points[0].x - 0.0).abs() < f32::EPSILON);
    assert!((points[0].y - 1.0).abs() < f32::EPSILON);
    assert!((points[1].x - 1.0).abs() < f32::EPSILON);
    assert!((points[1].y - 0.0).abs() < f32::EPSILON);
    assert!(points[1].selected);
}

#[test]
fn resolve_selected_location_id_for_minimap_follows_selection_kind() {
    let mut agent_locations = HashMap::new();
    agent_locations.insert("agent-7".to_string(), "loc-z".to_string());

    let location_selection = crate::ViewerSelection {
        current: Some(crate::SelectionInfo {
            entity: Entity::from_bits(1),
            kind: crate::SelectionKind::Location,
            id: "loc-x".to_string(),
            name: None,
        }),
    };
    let agent_selection = crate::ViewerSelection {
        current: Some(crate::SelectionInfo {
            entity: Entity::from_bits(2),
            kind: crate::SelectionKind::Agent,
            id: "agent-7".to_string(),
            name: None,
        }),
    };

    assert_eq!(
        resolve_selected_location_id_for_minimap(&location_selection, &agent_locations),
        Some("loc-x".to_string())
    );
    assert_eq!(
        resolve_selected_location_id_for_minimap(&agent_selection, &agent_locations),
        Some("loc-z".to_string())
    );
}
