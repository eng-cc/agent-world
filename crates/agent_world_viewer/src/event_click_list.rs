use agent_world::simulator::WorldEvent;
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::prelude::*;

use super::selection_linking::{event_primary_target, target_entity, EventObjectLinkState};
use super::*;

const EVENT_ROW_LIMIT: usize = 10;
const EVENT_LABEL_MAX_CHARS: usize = 110;

#[derive(Component)]
pub(super) struct EventClickListRoot {
    font: Handle<Font>,
}

#[derive(Component)]
pub(super) struct EventClickButton {
    event_id: u64,
}

pub(super) fn spawn_event_click_list(parent: &mut ChildSpawnerCommands, font: Handle<Font>) {
    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                row_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.11, 0.13, 0.17)),
            BorderColor::all(Color::srgb(0.2, 0.24, 0.3)),
            EventClickListRoot { font: font.clone() },
        ))
        .with_children(|list| {
            list.spawn((
                Text::new("Event Links:\n(waiting events)"),
                TextFont {
                    font,
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.78, 0.86)),
            ));
        });
}

pub(super) fn update_event_click_list_ui(
    mut commands: Commands,
    state: Res<ViewerState>,
    timeline: Option<Res<TimelineUiState>>,
    roots: Query<(Entity, &EventClickListRoot, Option<&Children>)>,
) {
    let timeline_changed = timeline
        .as_ref()
        .map(|timeline| timeline.is_changed())
        .unwrap_or(false);
    if !state.is_changed() && !timeline_changed {
        return;
    }

    let focus = focus_tick(&state, timeline.as_deref());

    for (root, marker, children) in &roots {
        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }

        commands.entity(root).with_children(|list| {
            let (window, focused_event_id) = event_window(&state.events, focus, EVENT_ROW_LIMIT);

            if window.is_empty() {
                list.spawn((
                    Text::new("Event Links:\n(no events)"),
                    TextFont {
                        font: marker.font.clone(),
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.72, 0.78, 0.86)),
                ));
                return;
            }

            list.spawn((
                Text::new("Event Links: click row to locate object"),
                TextFont {
                    font: marker.font.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.78, 0.86)),
            ));

            for event in window {
                let focused = focused_event_id == Some(event.id);
                list.spawn((
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        min_height: Val::Px(20.0),
                        padding: UiRect::horizontal(Val::Px(6.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexStart,
                        ..default()
                    },
                    BackgroundColor(if focused {
                        Color::srgb(0.2, 0.26, 0.34)
                    } else {
                        Color::srgb(0.13, 0.14, 0.18)
                    }),
                    EventClickButton { event_id: event.id },
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(event_row_label(event, focused)),
                        TextFont {
                            font: marker.font.clone(),
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(if focused {
                            Color::srgb(0.92, 0.96, 1.0)
                        } else {
                            Color::srgb(0.84, 0.87, 0.92)
                        }),
                    ));
                });
            }
        });
    }
}

pub(super) fn handle_event_click_buttons(
    state: Res<ViewerState>,
    scene: Res<Viewer3dScene>,
    config: Res<Viewer3dConfig>,
    mut selection: ResMut<ViewerSelection>,
    mut transforms: Query<(&mut Transform, Option<&BaseScale>)>,
    mut interactions: Query<
        (&Interaction, &EventClickButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut link_state: ResMut<EventObjectLinkState>,
    mut timeline: Option<ResMut<TimelineUiState>>,
) {
    for (interaction, button) in &mut interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(event) = state
            .events
            .iter()
            .find(|event| event.id == button.event_id)
        else {
            link_state.message = format!("Link: event #{} not found", button.event_id);
            continue;
        };

        if let Some(timeline) = timeline.as_deref_mut() {
            timeline.target_tick = event.time;
            timeline.manual_override = true;
        }

        let Some(target) = event_primary_target(event, state.snapshot.as_ref()) else {
            link_state.message = format!(
                "Link: event #{} at t{} has no mappable target",
                event.id, event.time
            );
            continue;
        };

        let Some(entity) = target_entity(&scene, &target) else {
            link_state.message = format!(
                "Link: event #{} target {} {} not visible",
                event.id,
                selection_kind_name(target.kind),
                target.id
            );
            continue;
        };

        if let Some(current) = selection.current.take() {
            reset_entity_scale(&mut transforms, current.entity);
        }

        selection.current = Some(SelectionInfo {
            entity,
            kind: target.kind,
            id: target.id.clone(),
            name: target.name.clone(),
        });

        if config.highlight_selected {
            apply_entity_highlight(&mut transforms, entity);
        }

        link_state.message = format!(
            "Link: event #{} -> {} {} (t{})",
            event.id,
            selection_kind_name(target.kind),
            target.id,
            event.time
        );
    }
}

fn event_window(
    events: &[WorldEvent],
    focus_tick: Option<u64>,
    limit: usize,
) -> (Vec<&WorldEvent>, Option<u64>) {
    if events.is_empty() || limit == 0 {
        return (Vec::new(), None);
    }

    if focus_tick.is_none() {
        let start = events.len().saturating_sub(limit);
        return (events[start..].iter().collect(), None);
    }

    let focus_tick = focus_tick.unwrap_or(0);
    let mut nearest_idx = 0_usize;
    let mut nearest_dist = u64::MAX;

    for (idx, event) in events.iter().enumerate() {
        let dist = event.time.abs_diff(focus_tick);
        if dist < nearest_dist {
            nearest_dist = dist;
            nearest_idx = idx;
        }
    }

    let half = limit / 2;
    let max_start = events.len().saturating_sub(limit);
    let start = nearest_idx.saturating_sub(half).min(max_start);
    let end = (start + limit).min(events.len());
    (
        events[start..end].iter().collect(),
        Some(events[nearest_idx].id),
    )
}

fn event_row_label(event: &WorldEvent, focused: bool) -> String {
    let mut body = format!("#{:>3} t{:>4} {:?}", event.id, event.time, event.kind);
    truncate_chars(&mut body, EVENT_LABEL_MAX_CHARS);
    if focused {
        format!(">> {body}")
    } else {
        format!("   {body}")
    }
}

fn truncate_chars(text: &mut String, limit: usize) {
    if text.chars().count() <= limit {
        return;
    }
    let mut cut = 0;
    for (idx, _) in text.char_indices().take(limit) {
        cut = idx;
    }
    text.truncate(cut);
    text.push('…');
}

fn focus_tick(state: &ViewerState, timeline: Option<&TimelineUiState>) -> Option<u64> {
    match timeline {
        Some(timeline) if timeline.manual_override || timeline.drag_active => {
            Some(timeline.target_tick)
        }
        _ => state
            .snapshot
            .as_ref()
            .map(|snapshot| snapshot.time)
            .or_else(|| state.metrics.as_ref().map(|metrics| metrics.total_ticks)),
    }
}

fn selection_kind_name(kind: SelectionKind) -> &'static str {
    match kind {
        SelectionKind::Agent => "agent",
        SelectionKind::Location => "location",
        SelectionKind::Asset => "asset",
        SelectionKind::PowerPlant => "power_plant",
        SelectionKind::PowerStorage => "power_storage",
        SelectionKind::Chunk => "chunk",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_world::geometry::GeoPos;
    use agent_world::simulator::{
        ChunkRuntimeConfig, Location, WorldConfig, WorldModel, WorldSnapshot,
    };

    fn sample_snapshot() -> WorldSnapshot {
        let mut model = WorldModel::default();
        model.locations.insert(
            "loc-1".to_string(),
            Location::new("loc-1", "Alpha", GeoPos::new(0.0, 0.0, 0.0)),
        );
        WorldSnapshot {
            version: agent_world::simulator::SNAPSHOT_VERSION,
            chunk_generation_schema_version:
                agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
            time: 7,
            config: WorldConfig::default(),
            model,
            chunk_runtime: ChunkRuntimeConfig::default(),
            next_event_id: 1,
            next_action_id: 1,
            pending_actions: Vec::new(),
            journal_len: 0,
        }
    }

    #[test]
    fn event_window_centers_nearest_focus_tick() {
        let events = vec![
            WorldEvent {
                id: 1,
                time: 1,
                kind: agent_world::simulator::WorldEventKind::LocationRegistered {
                    location_id: "loc-1".to_string(),
                    name: "a".to_string(),
                    pos: GeoPos::new(0.0, 0.0, 0.0),
                    profile: Default::default(),
                },
            },
            WorldEvent {
                id: 2,
                time: 5,
                kind: agent_world::simulator::WorldEventKind::LocationRegistered {
                    location_id: "loc-2".to_string(),
                    name: "b".to_string(),
                    pos: GeoPos::new(0.0, 0.0, 0.0),
                    profile: Default::default(),
                },
            },
            WorldEvent {
                id: 3,
                time: 9,
                kind: agent_world::simulator::WorldEventKind::LocationRegistered {
                    location_id: "loc-3".to_string(),
                    name: "c".to_string(),
                    pos: GeoPos::new(0.0, 0.0, 0.0),
                    profile: Default::default(),
                },
            },
        ];

        let (window, focused) = event_window(&events, Some(6), 2);
        assert_eq!(window.len(), 2);
        assert_eq!(window[0].id, 1);
        assert_eq!(window[1].id, 2);
        assert_eq!(focused, Some(2));
    }

    #[test]
    fn event_click_selects_target_and_updates_timeline() {
        let mut app = App::new();
        app.add_systems(Update, handle_event_click_buttons);
        app.world_mut().insert_resource(Viewer3dConfig::default());
        app.world_mut().insert_resource(ViewerSelection::default());
        app.world_mut()
            .insert_resource(EventObjectLinkState::default());
        app.world_mut().insert_resource(TimelineUiState::default());

        let target_entity = app
            .world_mut()
            .spawn((
                Transform::default(),
                GlobalTransform::default(),
                BaseScale(Vec3::ONE),
            ))
            .id();
        let mut scene = Viewer3dScene::default();
        scene
            .location_entities
            .insert("loc-1".to_string(), target_entity);
        app.world_mut().insert_resource(scene);

        let event = WorldEvent {
            id: 11,
            time: 9,
            kind: agent_world::simulator::WorldEventKind::LocationRegistered {
                location_id: "loc-1".to_string(),
                name: "Alpha".to_string(),
                pos: GeoPos::new(0.0, 0.0, 0.0),
                profile: Default::default(),
            },
        };

        app.world_mut().insert_resource(ViewerState {
            status: ConnectionStatus::Connected,
            snapshot: Some(sample_snapshot()),
            events: vec![event],
            decision_traces: Vec::new(),
            metrics: None,
        });

        app.world_mut().spawn((
            Button,
            Interaction::Pressed,
            EventClickButton { event_id: 11 },
        ));

        app.update();

        let selection = app.world().resource::<ViewerSelection>();
        let current = selection.current.as_ref().expect("selection exists");
        assert_eq!(current.kind, SelectionKind::Location);
        assert_eq!(current.id, "loc-1");

        let timeline = app.world().resource::<TimelineUiState>();
        assert_eq!(timeline.target_tick, 9);
        assert!(timeline.manual_override);

        let link_state = app.world().resource::<EventObjectLinkState>();
        assert!(link_state.message.contains("event #11 -> location loc-1"));
    }

    #[test]
    fn event_click_maps_module_visual_event_to_scene_entity() {
        let mut app = App::new();
        app.add_systems(Update, handle_event_click_buttons);
        app.world_mut().insert_resource(Viewer3dConfig::default());
        app.world_mut().insert_resource(ViewerSelection::default());
        app.world_mut()
            .insert_resource(EventObjectLinkState::default());
        app.world_mut().insert_resource(TimelineUiState::default());

        let visual_entity = app
            .world_mut()
            .spawn((
                Transform::default(),
                GlobalTransform::default(),
                BaseScale(Vec3::ONE),
            ))
            .id();

        let mut scene = Viewer3dScene::default();
        scene
            .module_visual_entities
            .insert("mv-1".to_string(), visual_entity);
        app.world_mut().insert_resource(scene);

        let event = WorldEvent {
            id: 13,
            time: 12,
            kind: agent_world::simulator::WorldEventKind::ModuleVisualEntityUpserted {
                entity: agent_world::simulator::ModuleVisualEntity {
                    entity_id: "mv-1".to_string(),
                    module_id: "m.test".to_string(),
                    kind: "relay".to_string(),
                    label: None,
                    anchor: agent_world::simulator::ModuleVisualAnchor::Absolute {
                        pos: GeoPos::new(0.0, 0.0, 0.0),
                    },
                },
            },
        };

        app.world_mut().insert_resource(ViewerState {
            status: ConnectionStatus::Connected,
            snapshot: Some(sample_snapshot()),
            events: vec![event],
            decision_traces: Vec::new(),
            metrics: None,
        });

        app.world_mut().spawn((
            Button,
            Interaction::Pressed,
            EventClickButton { event_id: 13 },
        ));

        app.update();

        let selection = app.world().resource::<ViewerSelection>();
        let current = selection.current.as_ref().expect("selection exists");
        assert_eq!(current.kind, SelectionKind::Asset);
        assert_eq!(current.id, "mv-1");

        let link_state = app.world().resource::<EventObjectLinkState>();
        assert!(link_state.message.contains("event #13 -> asset mv-1"));
    }

    #[test]
    fn event_click_without_target_only_updates_message() {
        let mut app = App::new();
        app.add_systems(Update, handle_event_click_buttons);
        app.world_mut().insert_resource(Viewer3dConfig::default());
        app.world_mut().insert_resource(Viewer3dScene::default());
        app.world_mut().insert_resource(ViewerSelection::default());
        app.world_mut()
            .insert_resource(EventObjectLinkState::default());
        app.world_mut().insert_resource(TimelineUiState::default());

        let event = WorldEvent {
            id: 12,
            time: 10,
            kind: agent_world::simulator::WorldEventKind::ActionRejected {
                reason: agent_world::simulator::RejectReason::InvalidAmount { amount: 0 },
            },
        };

        app.world_mut().insert_resource(ViewerState {
            status: ConnectionStatus::Connected,
            snapshot: Some(sample_snapshot()),
            events: vec![event],
            decision_traces: Vec::new(),
            metrics: None,
        });

        app.world_mut().spawn((
            Button,
            Interaction::Pressed,
            EventClickButton { event_id: 12 },
        ));

        app.update();

        let selection = app.world().resource::<ViewerSelection>();
        assert!(selection.current.is_none());

        let link_state = app.world().resource::<EventObjectLinkState>();
        assert!(link_state.message.contains("has no mappable target"));

        let timeline = app.world().resource::<TimelineUiState>();
        assert_eq!(timeline.target_tick, 10);
        assert!(timeline.manual_override);
    }
}
