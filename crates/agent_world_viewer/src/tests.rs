use super::*;
use agent_world::simulator::{ResourceKind, WorldEventKind};

#[test]
fn update_ui_sets_status_and_events() {
    let mut app = App::new();
    app.add_systems(Update, update_ui);

    app.world_mut().spawn((Text::new(""), StatusText));
    app.world_mut().spawn((Text::new(""), SummaryText));
    app.world_mut().spawn((Text::new(""), EventsText));
    app.world_mut().spawn((Text::new(""), SelectionText));
    app.world_mut().spawn((Text::new(""), AgentActivityText));
    app.world_mut().insert_resource(ViewerSelection::default());

    let event = WorldEvent {
        id: 1,
        time: 7,
        kind: agent_world::simulator::WorldEventKind::ActionRejected {
            reason: agent_world::simulator::RejectReason::InvalidAmount { amount: 1 },
        },
    };

    let state = ViewerState {
        status: ConnectionStatus::Error("oops".to_string()),
        snapshot: None,
        events: vec![event.clone()],
        decision_traces: Vec::new(),
        metrics: None,
    };
    app.world_mut().insert_resource(state);

    app.update();

    let world = app.world_mut();

    let status_text = {
        let mut query = world.query::<(&Text, &StatusText)>();
        query.single(world).expect("status text").0.clone()
    };
    assert_eq!(status_text.0, "Status: error: oops");

    let events_text = {
        let mut query = world.query::<(&Text, &EventsText)>();
        query.single(world).expect("events text").0.clone()
    };
    assert_eq!(events_text.0, events_summary(&[event]));
}

#[test]
fn update_ui_populates_world_summary_and_metrics() {
    let mut app = App::new();
    app.add_systems(Update, update_ui);

    app.world_mut().spawn((Text::new(""), SummaryText));
    app.world_mut().spawn((Text::new(""), StatusText));
    app.world_mut().spawn((Text::new(""), EventsText));
    app.world_mut().spawn((Text::new(""), SelectionText));
    app.world_mut().spawn((Text::new(""), AgentActivityText));
    app.world_mut().insert_resource(ViewerSelection::default());

    let mut model = agent_world::simulator::WorldModel::default();
    model.locations.insert(
        "loc-1".to_string(),
        agent_world::simulator::Location::new(
            "loc-1",
            "Alpha",
            agent_world::geometry::GeoPos {
                x_cm: 0.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
        ),
    );
    model.locations.insert(
        "loc-2".to_string(),
        agent_world::simulator::Location::new(
            "loc-2",
            "Beta",
            agent_world::geometry::GeoPos {
                x_cm: 1.0,
                y_cm: 1.0,
                z_cm: 0.0,
            },
        ),
    );
    model.agents.insert(
        "agent-1".to_string(),
        agent_world::simulator::Agent::new(
            "agent-1",
            "loc-1",
            agent_world::geometry::GeoPos {
                x_cm: 0.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
        ),
    );

    let snapshot = agent_world::simulator::WorldSnapshot {
        version: agent_world::simulator::SNAPSHOT_VERSION,
        chunk_generation_schema_version: agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
        time: 42,
        config: agent_world::simulator::WorldConfig::default(),
        model,
        chunk_runtime: agent_world::simulator::ChunkRuntimeConfig::default(),
        next_event_id: 1,
        next_action_id: 1,
        pending_actions: Vec::new(),
        journal_len: 0,
    };

    let metrics = RunnerMetrics {
        total_ticks: 42,
        total_actions: 7,
        total_decisions: 4,
        ..RunnerMetrics::default()
    };

    let state = ViewerState {
        status: ConnectionStatus::Connected,
        snapshot: Some(snapshot),
        events: Vec::new(),
        decision_traces: Vec::new(),
        metrics: Some(metrics),
    };
    app.world_mut().insert_resource(state);

    app.update();

    let world = app.world_mut();
    let summary_text = {
        let mut query = world.query::<(&Text, &SummaryText)>();
        query.single(world).expect("summary text").0.clone()
    };

    assert!(summary_text.0.contains("Time: 42"));
    assert!(summary_text.0.contains("Locations: 2"));
    assert!(summary_text.0.contains("Agents: 1"));
    assert!(summary_text.0.contains("Ticks: 42"));
    assert!(summary_text.0.contains("Actions: 7"));
    assert!(summary_text.0.contains("Decisions: 4"));
}

#[test]
fn update_ui_reflects_filtered_events() {
    let mut app = App::new();
    app.add_systems(Update, update_ui);

    app.world_mut().spawn((Text::new(""), EventsText));
    app.world_mut().spawn((Text::new(""), SummaryText));
    app.world_mut().spawn((Text::new(""), StatusText));
    app.world_mut().spawn((Text::new(""), SelectionText));
    app.world_mut().spawn((Text::new(""), AgentActivityText));
    app.world_mut().insert_resource(ViewerSelection::default());

    let event = WorldEvent {
        id: 9,
        time: 5,
        kind: agent_world::simulator::WorldEventKind::Power(
            agent_world::simulator::PowerEvent::PowerConsumed {
                agent_id: "agent-1".to_string(),
                amount: 3,
                reason: agent_world::simulator::ConsumeReason::Decision,
                remaining: 7,
            },
        ),
    };

    let state = ViewerState {
        status: ConnectionStatus::Connected,
        snapshot: None,
        events: vec![event.clone()],
        decision_traces: Vec::new(),
        metrics: None,
    };
    app.world_mut().insert_resource(state);

    app.update();

    let world = app.world_mut();
    let events_text = {
        let mut query = world.query::<(&Text, &EventsText)>();
        query.single(world).expect("events text").0.clone()
    };
    assert!(events_text.0.contains("Power"));
}

#[test]
fn update_ui_populates_agent_activity_panel() {
    let mut app = App::new();
    app.add_systems(Update, update_ui);

    app.world_mut().spawn((Text::new(""), EventsText));
    app.world_mut().spawn((Text::new(""), SummaryText));
    app.world_mut().spawn((Text::new(""), StatusText));
    app.world_mut().spawn((Text::new(""), SelectionText));
    app.world_mut().spawn((Text::new(""), AgentActivityText));
    app.world_mut().insert_resource(ViewerSelection::default());

    let mut model = agent_world::simulator::WorldModel::default();
    model.locations.insert(
        "loc-1".to_string(),
        agent_world::simulator::Location::new(
            "loc-1",
            "Alpha",
            agent_world::geometry::GeoPos::new(0.0, 0.0, 0.0),
        ),
    );
    model.locations.insert(
        "loc-2".to_string(),
        agent_world::simulator::Location::new(
            "loc-2",
            "Beta",
            agent_world::geometry::GeoPos::new(1.0, 1.0, 0.0),
        ),
    );

    let mut agent =
        agent_world::simulator::Agent::new("agent-1", "loc-2", GeoPos::new(1.0, 1.0, 0.0));
    agent
        .resources
        .set(ResourceKind::Electricity, 42)
        .expect("set electricity");
    model.agents.insert("agent-1".to_string(), agent);

    let snapshot = agent_world::simulator::WorldSnapshot {
        version: agent_world::simulator::SNAPSHOT_VERSION,
        chunk_generation_schema_version: agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
        time: 9,
        config: agent_world::simulator::WorldConfig::default(),
        model,
        chunk_runtime: agent_world::simulator::ChunkRuntimeConfig::default(),
        next_event_id: 1,
        next_action_id: 1,
        pending_actions: Vec::new(),
        journal_len: 0,
    };

    let events = vec![WorldEvent {
        id: 7,
        time: 8,
        kind: WorldEventKind::RadiationHarvested {
            agent_id: "agent-1".to_string(),
            location_id: "loc-2".to_string(),
            amount: 6,
            available: 12,
        },
    }];

    app.world_mut().insert_resource(ViewerState {
        status: ConnectionStatus::Connected,
        snapshot: Some(snapshot),
        events,
        decision_traces: Vec::new(),
        metrics: None,
    });

    app.update();

    let world = app.world_mut();
    let activity_text = {
        let mut query = world.query::<(&Text, &AgentActivityText)>();
        query.single(world).expect("activity text").0.clone()
    };
    assert!(activity_text.0.contains("agent-1 @ loc-2"));
    assert!(activity_text.0.contains("E=42"));
    assert!(activity_text.0.contains("harvest +6"));
}

#[test]
fn spawn_world_background_adds_bounds_and_grid() {
    let mut app = App::new();
    app.add_systems(Update, spawn_background_test_system);
    app.insert_resource(Viewer3dConfig::default());
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(Viewer3dAssets {
        agent_mesh: Handle::default(),
        agent_material: Handle::default(),
        location_mesh: Handle::default(),
        location_material: Handle::default(),
        world_box_mesh: Handle::default(),
        world_floor_material: Handle::default(),
        world_bounds_material: Handle::default(),
        world_grid_material: Handle::default(),
        label_font: Handle::default(),
    });

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&Name>();
    let names: Vec<String> = query.iter(world).map(|name| name.to_string()).collect();
    assert!(names.iter().any(|name| name == "world:bounds"));
    assert!(names.iter().any(|name| name == "world:floor"));
    assert!(names.iter().any(|name| name.starts_with("world:grid:x:")));
    assert!(names.iter().any(|name| name.starts_with("world:grid:z:")));
}

#[test]
fn handle_control_buttons_sends_request() {
    let mut app = App::new();
    app.add_systems(Update, handle_control_buttons);

    let (tx, rx) = mpsc::channel::<ViewerRequest>();
    app.world_mut().insert_resource(ViewerClient {
        tx,
        rx: Mutex::new(mpsc::channel::<ViewerResponse>().1),
    });

    app.world_mut().spawn((
        Button,
        Interaction::Pressed,
        ControlButton {
            control: ViewerControl::Step { count: 2 },
        },
    ));

    app.update();

    let request = rx.try_recv().expect("request sent");
    assert_eq!(
        request,
        ViewerRequest::Control {
            mode: ViewerControl::Step { count: 2 }
        }
    );
}

#[test]
fn control_buttons_send_expected_requests() {
    let mut app = App::new();
    app.add_systems(Update, handle_control_buttons);

    let (tx, rx) = mpsc::channel::<ViewerRequest>();
    app.world_mut().insert_resource(ViewerClient {
        tx,
        rx: Mutex::new(mpsc::channel::<ViewerResponse>().1),
    });

    for control in [
        ViewerControl::Play,
        ViewerControl::Pause,
        ViewerControl::Step { count: 1 },
        ViewerControl::Seek { tick: 0 },
    ] {
        app.world_mut().spawn((
            Button,
            Interaction::Pressed,
            ControlButton {
                control: control.clone(),
            },
        ));
    }

    app.update();

    let mut seen = Vec::new();
    while let Ok(request) = rx.try_recv() {
        seen.push(request);
    }

    assert!(seen.contains(&ViewerRequest::Control {
        mode: ViewerControl::Play
    }));
    assert!(seen.contains(&ViewerRequest::Control {
        mode: ViewerControl::Pause
    }));
    assert!(seen.contains(&ViewerRequest::Control {
        mode: ViewerControl::Step { count: 1 }
    }));
    assert!(seen.contains(&ViewerRequest::Control {
        mode: ViewerControl::Seek { tick: 0 }
    }));
}

#[test]
fn poll_viewer_messages_collects_decision_traces() {
    let mut app = App::new();
    app.add_systems(Update, poll_viewer_messages);

    app.world_mut().insert_resource(ViewerConfig {
        addr: "127.0.0.1:0".to_string(),
        max_events: 2,
    });

    let (tx, rx) = mpsc::channel::<ViewerResponse>();
    app.world_mut().insert_resource(ViewerClient {
        tx: mpsc::channel::<ViewerRequest>().0,
        rx: Mutex::new(rx),
    });
    app.world_mut().insert_resource(ViewerState::default());

    tx.send(ViewerResponse::DecisionTrace {
        trace: agent_world::simulator::AgentDecisionTrace {
            agent_id: "agent-1".to_string(),
            time: 1,
            decision: agent_world::simulator::AgentDecision::Wait,
            llm_input: Some("p1".to_string()),
            llm_output: Some("o1".to_string()),
            llm_error: None,
            parse_error: None,
        },
    })
    .expect("send trace1");
    tx.send(ViewerResponse::DecisionTrace {
        trace: agent_world::simulator::AgentDecisionTrace {
            agent_id: "agent-1".to_string(),
            time: 2,
            decision: agent_world::simulator::AgentDecision::Wait,
            llm_input: Some("p2".to_string()),
            llm_output: Some("o2".to_string()),
            llm_error: None,
            parse_error: None,
        },
    })
    .expect("send trace2");
    tx.send(ViewerResponse::DecisionTrace {
        trace: agent_world::simulator::AgentDecisionTrace {
            agent_id: "agent-1".to_string(),
            time: 3,
            decision: agent_world::simulator::AgentDecision::Wait,
            llm_input: Some("p3".to_string()),
            llm_output: Some("o3".to_string()),
            llm_error: None,
            parse_error: None,
        },
    })
    .expect("send trace3");

    app.update();

    let state = app.world_mut().resource::<ViewerState>();
    assert_eq!(state.decision_traces.len(), 2);
    assert_eq!(state.decision_traces[0].time, 2);
    assert_eq!(state.decision_traces[1].time, 3);
}

#[test]
fn headless_report_tracks_status_and_event_count() {
    let mut app = App::new();
    app.add_systems(Update, headless_report);
    app.world_mut().insert_resource(HeadlessStatus::default());

    app.world_mut().insert_resource(ViewerState {
        status: ConnectionStatus::Connecting,
        snapshot: None,
        events: Vec::new(),
        decision_traces: Vec::new(),
        metrics: None,
    });

    app.update();

    let status = app.world_mut().resource::<HeadlessStatus>();
    assert_eq!(status.last_status, Some(ConnectionStatus::Connecting));
    assert_eq!(status.last_events, 0);

    app.world_mut().insert_resource(ViewerState {
        status: ConnectionStatus::Connected,
        snapshot: None,
        events: vec![WorldEvent {
            id: 1,
            time: 1,
            kind: agent_world::simulator::WorldEventKind::ActionRejected {
                reason: agent_world::simulator::RejectReason::InvalidAmount { amount: 1 },
            },
        }],
        decision_traces: Vec::new(),
        metrics: None,
    });

    app.update();

    let status = app.world_mut().resource::<HeadlessStatus>();
    assert_eq!(status.last_status, Some(ConnectionStatus::Connected));
    assert_eq!(status.last_events, 1);
}

#[test]
fn decide_offline_defaults_headless_and_respects_overrides() {
    assert!(decide_offline(true, false, false));
    assert!(!decide_offline(false, false, false));
    assert!(decide_offline(false, true, false));
    assert!(!decide_offline(true, true, true));
    assert!(!decide_offline(true, false, true));
}

#[test]
fn space_origin_is_center_of_bounds() {
    let space = SpaceConfig {
        width_cm: 100,
        depth_cm: 200,
        height_cm: 300,
    };
    let origin = space_origin(&space);
    assert_eq!(origin.x_cm, 50.0);
    assert_eq!(origin.y_cm, 100.0);
    assert_eq!(origin.z_cm, 150.0);
}

#[test]
fn geo_to_vec3_scales_and_swaps_axes() {
    let origin = GeoPos::new(100.0, 200.0, 300.0);
    let pos = GeoPos::new(110.0, 220.0, 330.0);
    let vec = geo_to_vec3(pos, origin, 0.01);
    assert!((vec.x - 0.1).abs() < 1e-6);
    assert!((vec.y - 0.3).abs() < 1e-6);
    assert!((vec.z - 0.2).abs() < 1e-6);
}

#[test]
fn ray_point_distance_returns_expected_distance() {
    let ray = Ray3d {
        origin: Vec3::ZERO,
        direction: Dir3::new(Vec3::X).expect("direction"),
    };
    let point = Vec3::new(2.0, 1.0, 0.0);
    let distance = ray_point_distance(ray, point).expect("distance");
    assert!((distance - 1.0).abs() < 1e-6);
    assert!(ray_point_distance(ray, Vec3::new(-1.0, 0.0, 0.0)).is_none());
}

#[test]
fn spawn_location_entity_adds_label_text() {
    let mut app = App::new();
    app.add_systems(Update, spawn_label_test_system);
    app.insert_resource(Viewer3dConfig::default());
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(Viewer3dAssets {
        agent_mesh: Handle::default(),
        agent_material: Handle::default(),
        location_mesh: Handle::default(),
        location_material: Handle::default(),
        world_box_mesh: Handle::default(),
        world_floor_material: Handle::default(),
        world_bounds_material: Handle::default(),
        world_grid_material: Handle::default(),
        label_font: Handle::default(),
    });

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&Text2d>();
    assert!(query.iter(world).next().is_some());
}

#[test]
fn update_ui_populates_agent_selection_details_with_llm_trace() {
    let mut app = App::new();
    app.add_systems(Update, update_ui);

    app.world_mut().spawn((Text::new(""), StatusText));
    app.world_mut().spawn((Text::new(""), SummaryText));
    app.world_mut().spawn((Text::new(""), EventsText));
    app.world_mut().spawn((Text::new(""), SelectionText));
    app.world_mut().spawn((Text::new(""), AgentActivityText));
    app.world_mut().spawn((Text::new(""), SelectionDetailsText));

    let entity = app.world_mut().spawn_empty().id();
    app.world_mut().insert_resource(ViewerSelection {
        current: Some(SelectionInfo {
            entity,
            kind: SelectionKind::Agent,
            id: "agent-1".to_string(),
            name: None,
        }),
    });

    let mut model = agent_world::simulator::WorldModel::default();
    model.locations.insert(
        "loc-1".to_string(),
        agent_world::simulator::Location::new(
            "loc-1",
            "Alpha",
            agent_world::geometry::GeoPos::new(0.0, 0.0, 0.0),
        ),
    );
    model.agents.insert(
        "agent-1".to_string(),
        agent_world::simulator::Agent::new(
            "agent-1",
            "loc-1",
            agent_world::geometry::GeoPos::new(1.0, 2.0, 3.0),
        ),
    );

    let snapshot = agent_world::simulator::WorldSnapshot {
        version: agent_world::simulator::SNAPSHOT_VERSION,
        chunk_generation_schema_version: agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
        time: 11,
        config: agent_world::simulator::WorldConfig::default(),
        model,
        chunk_runtime: agent_world::simulator::ChunkRuntimeConfig::default(),
        next_event_id: 3,
        next_action_id: 2,
        pending_actions: Vec::new(),
        journal_len: 0,
    };

    let events = vec![WorldEvent {
        id: 2,
        time: 10,
        kind: agent_world::simulator::WorldEventKind::AgentMoved {
            agent_id: "agent-1".to_string(),
            from: "loc-0".to_string(),
            to: "loc-1".to_string(),
            distance_cm: 100,
            electricity_cost: 2,
        },
    }];

    let decision_traces = vec![agent_world::simulator::AgentDecisionTrace {
        agent_id: "agent-1".to_string(),
        time: 10,
        decision: agent_world::simulator::AgentDecision::Wait,
        llm_input: Some("prompt content".to_string()),
        llm_output: Some("{\"decision\":\"wait\"}".to_string()),
        llm_error: None,
        parse_error: None,
    }];

    app.world_mut().insert_resource(ViewerState {
        status: ConnectionStatus::Connected,
        snapshot: Some(snapshot),
        events,
        decision_traces,
        metrics: None,
    });

    app.update();

    let world = app.world_mut();
    let details_text = {
        let mut query = world.query::<(&Text, &SelectionDetailsText)>();
        query.single(world).expect("details text").0.clone()
    };

    assert!(details_text.0.contains("Details: agent agent-1"));
    assert!(details_text.0.contains("Recent LLM I/O"));
    assert!(details_text.0.contains("input:"));
    assert!(details_text.0.contains("output:"));
}

#[test]
fn update_ui_populates_location_selection_details() {
    let mut app = App::new();
    app.add_systems(Update, update_ui);

    app.world_mut().spawn((Text::new(""), StatusText));
    app.world_mut().spawn((Text::new(""), SummaryText));
    app.world_mut().spawn((Text::new(""), EventsText));
    app.world_mut().spawn((Text::new(""), SelectionText));
    app.world_mut().spawn((Text::new(""), AgentActivityText));
    app.world_mut().spawn((Text::new(""), SelectionDetailsText));

    let entity = app.world_mut().spawn_empty().id();
    app.world_mut().insert_resource(ViewerSelection {
        current: Some(SelectionInfo {
            entity,
            kind: SelectionKind::Location,
            id: "loc-1".to_string(),
            name: Some("Alpha".to_string()),
        }),
    });

    let mut model = agent_world::simulator::WorldModel::default();
    model.locations.insert(
        "loc-1".to_string(),
        agent_world::simulator::Location::new_with_profile(
            "loc-1",
            "Alpha",
            agent_world::geometry::GeoPos::new(0.0, 0.0, 0.0),
            agent_world::simulator::LocationProfile {
                material: agent_world::simulator::MaterialKind::Silicate,
                radius_cm: 320,
                radiation_emission_per_tick: 9,
            },
        ),
    );

    let snapshot = agent_world::simulator::WorldSnapshot {
        version: agent_world::simulator::SNAPSHOT_VERSION,
        chunk_generation_schema_version: agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
        time: 3,
        config: agent_world::simulator::WorldConfig::default(),
        model,
        chunk_runtime: agent_world::simulator::ChunkRuntimeConfig::default(),
        next_event_id: 1,
        next_action_id: 1,
        pending_actions: Vec::new(),
        journal_len: 0,
    };

    app.world_mut().insert_resource(ViewerState {
        status: ConnectionStatus::Connected,
        snapshot: Some(snapshot),
        events: Vec::new(),
        decision_traces: Vec::new(),
        metrics: None,
    });

    app.update();

    let world = app.world_mut();
    let details_text = {
        let mut query = world.query::<(&Text, &SelectionDetailsText)>();
        query.single(world).expect("details text").0.clone()
    };

    assert!(details_text.0.contains("Details: location loc-1"));
    assert!(details_text.0.contains("radiation/tick=9"));
}

fn spawn_background_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    spawn_world_background(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        &sample_snapshot(),
    );
}

fn sample_snapshot() -> WorldSnapshot {
    let mut model = agent_world::simulator::WorldModel::default();
    model.locations.insert(
        "loc-1".to_string(),
        agent_world::simulator::Location::new(
            "loc-1",
            "Alpha",
            agent_world::geometry::GeoPos::new(0.0, 0.0, 0.0),
        ),
    );
    WorldSnapshot {
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
    }
}

fn spawn_label_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let origin = GeoPos::new(0.0, 0.0, 0.0);
    spawn_location_entity(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "loc-1",
        "Alpha",
        GeoPos::new(0.0, 0.0, 0.0),
    );
}
