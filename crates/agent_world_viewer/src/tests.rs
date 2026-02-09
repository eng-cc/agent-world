use super::*;
use crate::app_bootstrap::decide_offline;
use crate::button_feedback::StepControlLoadingState;
use crate::timeline_controls::{
    normalized_x_to_tick, TimelineAdjustButton, TimelineBar, TimelineBarFill,
    TimelineSeekSubmitButton, TimelineStatusText,
};
use crate::viewer_3d_config::Viewer3dConfig;
use agent_world::simulator::{MaterialKind, ResourceKind, WorldEventKind};

#[path = "tests_selection_details.rs"]
mod tests_selection_details;

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
    app.world_mut().insert_resource(Viewer3dConfig::default());

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
    assert_eq!(events_text.0, events_summary(&[event], None));
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
    app.world_mut().insert_resource(Viewer3dConfig::default());

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
    assert!(summary_text.0.contains("Render Physical: off"));
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
    app.world_mut().insert_resource(Viewer3dConfig::default());

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
        location_material_library: LocationMaterialHandles::default(),
        asset_mesh: Handle::default(),
        asset_material: Handle::default(),
        power_plant_mesh: Handle::default(),
        power_plant_material: Handle::default(),
        power_storage_mesh: Handle::default(),
        power_storage_material: Handle::default(),
        chunk_mesh: Handle::default(),
        chunk_unexplored_material: Handle::default(),
        chunk_generated_material: Handle::default(),
        chunk_exhausted_material: Handle::default(),
        world_box_mesh: Handle::default(),
        world_floor_material: Handle::default(),
        world_bounds_material: Handle::default(),
        world_grid_material: Handle::default(),
        heat_low_material: Handle::default(),
        heat_mid_material: Handle::default(),
        heat_high_material: Handle::default(),
        flow_power_material: Handle::default(),
        flow_trade_material: Handle::default(),
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
    app.world_mut().insert_resource(ViewerState::default());
    app.world_mut()
        .insert_resource(StepControlLoadingState::default());

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
    app.world_mut().insert_resource(ViewerState::default());
    app.world_mut()
        .insert_resource(StepControlLoadingState::default());

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
fn timeline_adjust_and_submit_sends_seek_request() {
    let mut app = App::new();
    app.add_systems(
        Update,
        (handle_timeline_adjust_buttons, handle_timeline_seek_submit).chain(),
    );

    let (tx, rx) = mpsc::channel::<ViewerRequest>();
    app.world_mut().insert_resource(ViewerClient {
        tx,
        rx: Mutex::new(mpsc::channel::<ViewerResponse>().1),
    });
    app.world_mut().insert_resource(TimelineUiState {
        target_tick: 10,
        max_tick_seen: 100,
        manual_override: false,
        drag_active: false,
    });

    app.world_mut().spawn((
        Button,
        Interaction::Pressed,
        TimelineAdjustButton { delta: 15 },
    ));

    app.update();

    app.world_mut()
        .spawn((Button, Interaction::Pressed, TimelineSeekSubmitButton));

    app.update();

    let request = rx.try_recv().expect("seek request");
    assert_eq!(
        request,
        ViewerRequest::Control {
            mode: ViewerControl::Seek { tick: 25 }
        }
    );
}

#[test]
fn timeline_drag_updates_target_tick() {
    let mut app = App::new();
    app.add_systems(Update, handle_timeline_bar_drag);

    app.world_mut().insert_resource(ViewerState::default());
    app.world_mut().insert_resource(TimelineUiState {
        target_tick: 0,
        max_tick_seen: 100,
        manual_override: false,
        drag_active: false,
    });

    app.world_mut().spawn((
        Button,
        Interaction::Pressed,
        bevy::ui::RelativeCursorPosition {
            cursor_over: true,
            normalized: Some(Vec2::new(0.25, 0.0)),
        },
        TimelineBar,
    ));

    app.update();

    let timeline = app.world().resource::<TimelineUiState>();
    assert_eq!(timeline.target_tick, 75);
    assert!(timeline.manual_override);
    assert!(timeline.drag_active);
}

#[test]
fn update_timeline_ui_renders_text_and_fill() {
    let mut app = App::new();
    app.add_systems(Update, update_timeline_ui);

    app.world_mut().spawn((Text::new(""), TimelineStatusText));
    app.world_mut().spawn((
        Node {
            width: Val::Px(0.0),
            height: Val::Px(8.0),
            ..default()
        },
        TimelineBarFill,
    ));

    let mut state = ViewerState::default();
    state.metrics = Some(RunnerMetrics {
        total_ticks: 40,
        ..RunnerMetrics::default()
    });
    app.world_mut().insert_resource(state);
    app.world_mut().insert_resource(TimelineUiState {
        target_tick: 50,
        max_tick_seen: 100,
        manual_override: true,
        drag_active: false,
    });

    app.update();

    let world = app.world_mut();
    let timeline_text = {
        let mut query = world.query::<(&Text, &TimelineStatusText)>();
        query.single(world).expect("timeline text").0.clone()
    };
    assert!(timeline_text.0.contains("now=40"));
    assert!(timeline_text.0.contains("target=50"));
    assert!(timeline_text.0.contains("max=100"));
    assert!(timeline_text.0.contains("mode=manual"));

    let fill_width = {
        let mut query = world.query::<(&Node, &TimelineBarFill)>();
        query.single(world).expect("timeline fill").0.width
    };
    assert_eq!(fill_width, Val::Percent(50.0));
}

#[test]
fn normalized_x_to_tick_maps_centered_range() {
    assert_eq!(normalized_x_to_tick(-0.5, 100), 0);
    assert_eq!(normalized_x_to_tick(0.0, 100), 50);
    assert_eq!(normalized_x_to_tick(0.5, 100), 100);
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
            llm_diagnostics: None,
            llm_effect_intents: Vec::new(),
            llm_effect_receipts: Vec::new(),
            llm_step_trace: Vec::new(),
            llm_prompt_section_trace: Vec::new(),
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
            llm_diagnostics: None,
            llm_effect_intents: Vec::new(),
            llm_effect_receipts: Vec::new(),
            llm_step_trace: Vec::new(),
            llm_prompt_section_trace: Vec::new(),
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
            llm_diagnostics: None,
            llm_effect_intents: Vec::new(),
            llm_effect_receipts: Vec::new(),
            llm_step_trace: Vec::new(),
            llm_prompt_section_trace: Vec::new(),
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
        location_material_library: LocationMaterialHandles::default(),
        asset_mesh: Handle::default(),
        asset_material: Handle::default(),
        power_plant_mesh: Handle::default(),
        power_plant_material: Handle::default(),
        power_storage_mesh: Handle::default(),
        power_storage_material: Handle::default(),
        chunk_mesh: Handle::default(),
        chunk_unexplored_material: Handle::default(),
        chunk_generated_material: Handle::default(),
        chunk_exhausted_material: Handle::default(),
        world_box_mesh: Handle::default(),
        world_floor_material: Handle::default(),
        world_bounds_material: Handle::default(),
        world_grid_material: Handle::default(),
        heat_low_material: Handle::default(),
        heat_mid_material: Handle::default(),
        heat_high_material: Handle::default(),
        flow_power_material: Handle::default(),
        flow_trade_material: Handle::default(),
        label_font: Handle::default(),
    });

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&Text2d>();
    assert!(query.iter(world).next().is_some());
}

#[test]
fn spawn_location_entity_uses_physical_radius_scale() {
    let mut app = App::new();
    app.add_systems(Update, spawn_location_scale_test_system);
    app.insert_resource(Viewer3dConfig::default());
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(Viewer3dAssets {
        agent_mesh: Handle::default(),
        agent_material: Handle::default(),
        location_mesh: Handle::default(),
        location_material_library: LocationMaterialHandles::default(),
        asset_mesh: Handle::default(),
        asset_material: Handle::default(),
        power_plant_mesh: Handle::default(),
        power_plant_material: Handle::default(),
        power_storage_mesh: Handle::default(),
        power_storage_material: Handle::default(),
        chunk_mesh: Handle::default(),
        chunk_unexplored_material: Handle::default(),
        chunk_generated_material: Handle::default(),
        chunk_exhausted_material: Handle::default(),
        world_box_mesh: Handle::default(),
        world_floor_material: Handle::default(),
        world_bounds_material: Handle::default(),
        world_grid_material: Handle::default(),
        heat_low_material: Handle::default(),
        heat_mid_material: Handle::default(),
        heat_high_material: Handle::default(),
        flow_power_material: Handle::default(),
        flow_trade_material: Handle::default(),
        label_font: Handle::default(),
    });

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&LocationMarker, &BaseScale)>();
    let (marker, base) = query
        .iter(world)
        .find(|(marker, _)| marker.id == "loc-scale")
        .expect("location marker exists");
    assert!((base.0.x - 200.0).abs() < 1e-3);
    assert_eq!(marker.material, MaterialKind::Silicate);
}

#[test]
fn spawn_agent_entity_uses_body_height_scale() {
    let mut app = App::new();
    app.add_systems(Update, spawn_agent_scale_test_system);
    app.insert_resource(Viewer3dConfig::default());
    app.insert_resource(Viewer3dScene::default());
    app.insert_resource(Viewer3dAssets {
        agent_mesh: Handle::default(),
        agent_material: Handle::default(),
        location_mesh: Handle::default(),
        location_material_library: LocationMaterialHandles::default(),
        asset_mesh: Handle::default(),
        asset_material: Handle::default(),
        power_plant_mesh: Handle::default(),
        power_plant_material: Handle::default(),
        power_storage_mesh: Handle::default(),
        power_storage_material: Handle::default(),
        chunk_mesh: Handle::default(),
        chunk_unexplored_material: Handle::default(),
        chunk_generated_material: Handle::default(),
        chunk_exhausted_material: Handle::default(),
        world_box_mesh: Handle::default(),
        world_floor_material: Handle::default(),
        world_bounds_material: Handle::default(),
        world_grid_material: Handle::default(),
        heat_low_material: Handle::default(),
        heat_mid_material: Handle::default(),
        heat_high_material: Handle::default(),
        flow_power_material: Handle::default(),
        flow_trade_material: Handle::default(),
        label_font: Handle::default(),
    });

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<(&AgentMarker, &BaseScale)>();
    let (_, base) = query
        .iter(world)
        .find(|(marker, _)| marker.id == "agent-scale")
        .expect("agent marker exists");
    assert!((base.0.x - 0.7).abs() < 1e-3);
}

#[test]
fn update_ui_populates_asset_selection_details() {
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
            kind: SelectionKind::Asset,
            id: "asset-1".to_string(),
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
    model.assets.insert(
        "asset-1".to_string(),
        agent_world::simulator::Asset {
            id: "asset-1".to_string(),
            owner: agent_world::simulator::ResourceOwner::Location {
                location_id: "loc-1".to_string(),
            },
            kind: agent_world::simulator::AssetKind::Resource {
                kind: agent_world::simulator::ResourceKind::Electricity,
            },
            quantity: 25,
        },
    );

    let snapshot = agent_world::simulator::WorldSnapshot {
        version: agent_world::simulator::SNAPSHOT_VERSION,
        chunk_generation_schema_version: agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
        time: 8,
        config: agent_world::simulator::WorldConfig::default(),
        model,
        chunk_runtime: agent_world::simulator::ChunkRuntimeConfig::default(),
        next_event_id: 2,
        next_action_id: 1,
        pending_actions: Vec::new(),
        journal_len: 0,
    };

    let events = vec![WorldEvent {
        id: 1,
        time: 8,
        kind: agent_world::simulator::WorldEventKind::ResourceTransferred {
            from: agent_world::simulator::ResourceOwner::Location {
                location_id: "loc-1".to_string(),
            },
            to: agent_world::simulator::ResourceOwner::Location {
                location_id: "loc-1".to_string(),
            },
            kind: agent_world::simulator::ResourceKind::Electricity,
            amount: 3,
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
    let details_text = {
        let mut query = world.query::<(&Text, &SelectionDetailsText)>();
        query.single(world).expect("details text").0.clone()
    };

    assert!(details_text.0.contains("Details: asset asset-1"));
    assert!(details_text.0.contains("Owner: location::loc-1"));
    assert!(details_text.0.contains("Recent Owner Events"));
}

#[test]
fn update_ui_populates_power_plant_selection_details() {
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
            kind: SelectionKind::PowerPlant,
            id: "plant-1".to_string(),
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
    model.power_plants.insert(
        "plant-1".to_string(),
        agent_world::simulator::PowerPlant {
            id: "plant-1".to_string(),
            location_id: "loc-1".to_string(),
            owner: agent_world::simulator::ResourceOwner::Location {
                location_id: "loc-1".to_string(),
            },
            capacity_per_tick: 30,
            current_output: 12,
            fuel_cost_per_pu: 2,
            maintenance_cost: 1,
            status: agent_world::simulator::PlantStatus::Running,
            efficiency: 0.9,
            degradation: 0.1,
        },
    );

    let snapshot = agent_world::simulator::WorldSnapshot {
        version: agent_world::simulator::SNAPSHOT_VERSION,
        chunk_generation_schema_version: agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
        time: 9,
        config: agent_world::simulator::WorldConfig::default(),
        model,
        chunk_runtime: agent_world::simulator::ChunkRuntimeConfig::default(),
        next_event_id: 3,
        next_action_id: 1,
        pending_actions: Vec::new(),
        journal_len: 0,
    };

    let events = vec![WorldEvent {
        id: 2,
        time: 9,
        kind: agent_world::simulator::WorldEventKind::Power(
            agent_world::simulator::PowerEvent::PowerGenerated {
                plant_id: "plant-1".to_string(),
                location_id: "loc-1".to_string(),
                amount: 7,
            },
        ),
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
    let details_text = {
        let mut query = world.query::<(&Text, &SelectionDetailsText)>();
        query.single(world).expect("details text").0.clone()
    };

    assert!(details_text.0.contains("Details: power_plant plant-1"));
    assert!(details_text.0.contains("Output: current=12"));
    assert!(details_text.0.contains("generated 7"));
}

#[test]
fn update_ui_populates_chunk_selection_details() {
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
            kind: SelectionKind::Chunk,
            id: "0,0,0".to_string(),
            name: Some("generated".to_string()),
        }),
    });

    let mut model = agent_world::simulator::WorldModel::default();
    model.chunks.insert(
        agent_world::simulator::ChunkCoord { x: 0, y: 0, z: 0 },
        agent_world::simulator::ChunkState::Generated,
    );

    let mut budget = agent_world::simulator::ChunkResourceBudget::default();
    budget
        .total_by_element_g
        .insert(agent_world::simulator::FragmentElementKind::Iron, 120);
    budget
        .remaining_by_element_g
        .insert(agent_world::simulator::FragmentElementKind::Iron, 90);
    model.chunk_resource_budgets.insert(
        agent_world::simulator::ChunkCoord { x: 0, y: 0, z: 0 },
        budget,
    );

    let snapshot = agent_world::simulator::WorldSnapshot {
        version: agent_world::simulator::SNAPSHOT_VERSION,
        chunk_generation_schema_version: agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
        time: 12,
        config: agent_world::simulator::WorldConfig::default(),
        model,
        chunk_runtime: agent_world::simulator::ChunkRuntimeConfig::default(),
        next_event_id: 3,
        next_action_id: 1,
        pending_actions: Vec::new(),
        journal_len: 0,
    };

    let events = vec![WorldEvent {
        id: 2,
        time: 12,
        kind: agent_world::simulator::WorldEventKind::ChunkGenerated {
            coord: agent_world::simulator::ChunkCoord { x: 0, y: 0, z: 0 },
            seed: 11,
            fragment_count: 4,
            block_count: 18,
            chunk_budget: agent_world::simulator::ChunkResourceBudget::default(),
            cause: agent_world::simulator::ChunkGenerationCause::Action,
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
    let details_text = {
        let mut query = world.query::<(&Text, &SelectionDetailsText)>();
        query.single(world).expect("details text").0.clone()
    };

    assert!(details_text.0.contains("Details: chunk 0,0,0"));
    assert!(details_text.0.contains("State: generated"));
    assert!(details_text.0.contains("Budget (remaining top):"));
    assert!(details_text.0.contains("generated fragments=4 blocks=18"));
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
        MaterialKind::Silicate,
        100,
    );
}

fn spawn_location_scale_test_system(
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
        "loc-scale",
        "Scale",
        GeoPos::new(0.0, 0.0, 0.0),
        MaterialKind::Silicate,
        20_000,
    );
}

fn spawn_agent_scale_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let origin = GeoPos::new(0.0, 0.0, 0.0);
    spawn_agent_entity(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "agent-scale",
        GeoPos::new(0.0, 0.0, 0.0),
        200,
    );
}
