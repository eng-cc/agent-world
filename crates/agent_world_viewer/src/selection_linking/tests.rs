use super::*;
use agent_world::simulator::{Agent, Location, PowerPlant, WorldModel, WorldSnapshot};

#[test]
fn nearest_event_uses_smallest_tick_distance() {
    let events = vec![
        WorldEvent {
            id: 1,
            time: 3,
            kind: WorldEventKind::AgentMoved {
                agent_id: "a1".to_string(),
                from: "l1".to_string(),
                to: "l2".to_string(),
                distance_cm: 1,
                electricity_cost: 1,
            },
        },
        WorldEvent {
            id: 2,
            time: 9,
            kind: WorldEventKind::AgentMoved {
                agent_id: "a1".to_string(),
                from: "l2".to_string(),
                to: "l1".to_string(),
                distance_cm: 1,
                electricity_cost: 1,
            },
        },
    ];

    let nearest = nearest_event_to_tick(&events, 8).expect("nearest");
    assert_eq!(nearest.id, 2);
}

#[test]
fn reject_reason_facility_maps_to_plant_target() {
    let mut model = WorldModel::default();
    model.power_plants.insert(
        "pp-1".to_string(),
        PowerPlant::new(
            "pp-1".to_string(),
            "loc-1".to_string(),
            ResourceOwner::Location {
                location_id: "loc-1".to_string(),
            },
            100,
        ),
    );
    let snapshot = WorldSnapshot {
        version: agent_world::simulator::SNAPSHOT_VERSION,
        chunk_generation_schema_version: agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
        time: 1,
        config: agent_world::simulator::WorldConfig::default(),
        model,
        chunk_runtime: agent_world::simulator::ChunkRuntimeConfig::default(),
        next_event_id: 1,
        next_action_id: 1,
        pending_actions: Vec::new(),
        journal_len: 0,
    };

    let event = WorldEvent {
        id: 9,
        time: 2,
        kind: WorldEventKind::ActionRejected {
            reason: RejectReason::FacilityNotFound {
                facility_id: "pp-1".to_string(),
            },
        },
    };

    let target = event_primary_target(&event, Some(&snapshot)).expect("target");
    assert_eq!(target.kind, SelectionKind::PowerPlant);
    assert_eq!(target.id, "pp-1");
}

#[test]
fn selection_related_ticks_match_agent_events() {
    let mut model = WorldModel::default();
    model.locations.insert(
        "loc-1".to_string(),
        Location::new(
            "loc-1",
            "L1",
            agent_world::geometry::GeoPos {
                x_cm: 0.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
        ),
    );
    model.agents.insert(
        "agent-1".to_string(),
        Agent::new(
            "agent-1",
            "loc-1",
            agent_world::geometry::GeoPos {
                x_cm: 0.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
        ),
    );
    let snapshot = WorldSnapshot {
        version: agent_world::simulator::SNAPSHOT_VERSION,
        chunk_generation_schema_version: agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
        time: 1,
        config: agent_world::simulator::WorldConfig::default(),
        model,
        chunk_runtime: agent_world::simulator::ChunkRuntimeConfig::default(),
        next_event_id: 1,
        next_action_id: 1,
        pending_actions: Vec::new(),
        journal_len: 0,
    };

    let events = vec![
        WorldEvent {
            id: 1,
            time: 5,
            kind: WorldEventKind::AgentMoved {
                agent_id: "agent-1".to_string(),
                from: "loc-1".to_string(),
                to: "loc-1".to_string(),
                distance_cm: 1,
                electricity_cost: 1,
            },
        },
        WorldEvent {
            id: 2,
            time: 7,
            kind: WorldEventKind::Power(PowerEvent::PowerConsumed {
                agent_id: "agent-1".to_string(),
                amount: 3,
                reason: agent_world::simulator::ConsumeReason::Decision,
                remaining: 9,
            }),
        },
        WorldEvent {
            id: 3,
            time: 11,
            kind: WorldEventKind::LocationRegistered {
                location_id: "loc-2".to_string(),
                name: "L2".to_string(),
                pos: agent_world::geometry::GeoPos {
                    x_cm: 1.0,
                    y_cm: 1.0,
                    z_cm: 1.0,
                },
                profile: agent_world::simulator::LocationProfile::default(),
            },
        },
    ];

    let selection = SelectionInfo {
        entity: Entity::from_bits(1),
        kind: SelectionKind::Agent,
        id: "agent-1".to_string(),
        name: None,
    };

    let ticks = selection_related_ticks(&selection, &events, Some(&snapshot));
    assert_eq!(ticks, vec![5, 7]);
}

#[test]
fn locate_focus_event_button_selects_target_and_updates_timeline() {
    let mut app = App::new();
    app.add_systems(Update, handle_locate_focus_event_button);

    let selected_entity = app
        .world_mut()
        .spawn((Transform::default(), BaseScale(Vec3::ONE)))
        .id();

    let mut scene = Viewer3dScene::default();
    scene
        .agent_entities
        .insert("agent-1".to_string(), selected_entity);

    let state = ViewerState {
        status: ConnectionStatus::Connected,
        snapshot: None,
        events: vec![WorldEvent {
            id: 1,
            time: 5,
            kind: WorldEventKind::AgentMoved {
                agent_id: "agent-1".to_string(),
                from: "loc-a".to_string(),
                to: "loc-b".to_string(),
                distance_cm: 100,
                electricity_cost: 1,
            },
        }],
        decision_traces: Vec::new(),
        metrics: None,
    };

    app.world_mut().insert_resource(state);
    app.world_mut().insert_resource(scene);
    app.world_mut().insert_resource(Viewer3dConfig::default());
    app.world_mut().insert_resource(ViewerSelection::default());
    app.world_mut()
        .insert_resource(EventObjectLinkState::default());
    app.world_mut().insert_resource(TimelineUiState::default());

    app.world_mut()
        .spawn((Button, Interaction::Pressed, LocateFocusEventButton));

    app.update();

    let selection = app.world().resource::<ViewerSelection>();
    let current = selection.current.as_ref().expect("selection");
    assert_eq!(current.kind, SelectionKind::Agent);
    assert_eq!(current.id, "agent-1");

    let timeline = app.world().resource::<TimelineUiState>();
    assert_eq!(timeline.target_tick, 5);
    assert!(timeline.manual_override);

    let link = app.world().resource::<EventObjectLinkState>();
    assert!(link.message.contains("event #1"));
}

#[test]
fn jump_selection_events_button_moves_timeline_target() {
    let mut app = App::new();
    app.add_systems(Update, handle_jump_selection_events_button);

    let state = ViewerState {
        status: ConnectionStatus::Connected,
        snapshot: None,
        events: vec![
            WorldEvent {
                id: 1,
                time: 3,
                kind: WorldEventKind::AgentMoved {
                    agent_id: "agent-1".to_string(),
                    from: "loc-a".to_string(),
                    to: "loc-b".to_string(),
                    distance_cm: 100,
                    electricity_cost: 1,
                },
            },
            WorldEvent {
                id: 2,
                time: 9,
                kind: WorldEventKind::Power(PowerEvent::PowerConsumed {
                    agent_id: "agent-1".to_string(),
                    amount: 3,
                    reason: agent_world::simulator::ConsumeReason::Decision,
                    remaining: 5,
                }),
            },
        ],
        decision_traces: Vec::new(),
        metrics: None,
    };

    app.world_mut().insert_resource(state);
    app.world_mut().insert_resource(ViewerSelection {
        current: Some(SelectionInfo {
            entity: Entity::from_bits(1),
            kind: SelectionKind::Agent,
            id: "agent-1".to_string(),
            name: None,
        }),
    });
    app.world_mut()
        .insert_resource(EventObjectLinkState::default());
    app.world_mut().insert_resource(TimelineUiState {
        target_tick: 3,
        max_tick_seen: 12,
        manual_override: true,
        drag_active: false,
    });

    app.world_mut()
        .spawn((Button, Interaction::Pressed, JumpSelectionEventsButton));

    app.update();

    let timeline = app.world().resource::<TimelineUiState>();
    assert_eq!(timeline.target_tick, 9);
    assert!(timeline.manual_override);

    let link = app.world().resource::<EventObjectLinkState>();
    assert!(link.message.contains("-> t9"));
}

#[test]
fn event_object_link_controls_use_wrapping_layout() {
    let mut app = App::new();
    app.add_systems(Startup, |mut commands: Commands| {
        let root = commands.spawn(Node::default()).id();
        commands.entity(root).with_children(|parent| {
            spawn_event_object_link_controls(
                parent,
                Handle::<Font>::default(),
                crate::i18n::UiLocale::EnUs,
            );
        });
    });

    app.update();

    let world = app.world_mut();

    let mut wrapping_row_count = 0usize;
    let mut row_query = world.query::<&Node>();
    for node in row_query.iter(world) {
        if node.flex_wrap == FlexWrap::Wrap && node.min_height == Val::Px(24.0) {
            wrapping_row_count += 1;
        }
    }
    assert!(wrapping_row_count >= 1, "expected wrapping controls row");

    let mut locate_query = world.query::<(&Node, &LocateFocusEventButton)>();
    let (locate_button, _) = locate_query.single(world).expect("locate button");
    assert_eq!(locate_button.min_width, Val::Px(120.0));
    assert_eq!(locate_button.flex_grow, 1.0);

    let mut jump_query = world.query::<(&Node, &JumpSelectionEventsButton)>();
    let (jump_button, _) = jump_query.single(world).expect("jump button");
    assert_eq!(jump_button.min_width, Val::Px(120.0));
    assert_eq!(jump_button.flex_grow, 1.0);
}
