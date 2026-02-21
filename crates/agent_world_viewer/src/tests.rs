use super::*;
use crate::app_bootstrap::decide_offline;
use crate::button_feedback::StepControlLoadingState;
use crate::timeline_controls::{
    normalized_x_to_tick, TimelineAdjustButton, TimelineBar, TimelineBarFill,
    TimelineSeekSubmitButton, TimelineStatusText,
};
use crate::viewer_3d_config::{
    Viewer3dConfig, ViewerExternalMaterialSlotConfig, ViewerExternalTextureSlotConfig,
    ViewerTonemappingMode,
};
use agent_world::simulator::{MaterialKind, ResourceKind, WorldEventKind};
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};

#[path = "tests_selection_details.rs"]
mod tests_selection_details;

#[path = "tests_scene_grid.rs"]
mod tests_scene_grid;

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
        event_window: EventWindowPolicy::new(2, 2, 1),
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
            llm_chat_messages: Vec::new(),
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
            llm_chat_messages: Vec::new(),
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
            llm_chat_messages: Vec::new(),
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
fn headless_auto_play_sends_play_once_after_connected() {
    struct AutoPlayEnvGuard;
    impl Drop for AutoPlayEnvGuard {
        fn drop(&mut self) {
            unsafe {
                std::env::remove_var("AGENT_WORLD_VIEWER_AUTO_PLAY");
            }
        }
    }
    unsafe {
        std::env::set_var("AGENT_WORLD_VIEWER_AUTO_PLAY", "1");
    }
    let _guard = AutoPlayEnvGuard;

    let mut app = App::new();
    app.add_systems(Update, headless_auto_play_once);

    let (tx_request, rx_request) = mpsc::channel::<ViewerRequest>();
    app.world_mut().insert_resource(ViewerClient {
        tx: tx_request,
        rx: Mutex::new(mpsc::channel::<ViewerResponse>().1),
    });
    app.world_mut().insert_resource(ViewerState {
        status: ConnectionStatus::Connected,
        ..ViewerState::default()
    });

    app.update();
    let first = rx_request.try_recv().expect("first control request");
    assert_eq!(
        first,
        ViewerRequest::Control {
            mode: ViewerControl::Play
        }
    );

    app.update();
    assert!(rx_request.try_recv().is_err());
}

#[test]
fn poll_viewer_messages_applies_event_window_sampling_policy() {
    let mut app = App::new();
    app.add_systems(Update, poll_viewer_messages);

    app.world_mut().insert_resource(ViewerConfig {
        addr: "127.0.0.1:0".to_string(),
        max_events: 16,
        event_window: EventWindowPolicy::new(6, 3, 2),
    });

    let (tx, rx) = mpsc::channel::<ViewerResponse>();
    app.world_mut().insert_resource(ViewerClient {
        tx: mpsc::channel::<ViewerRequest>().0,
        rx: Mutex::new(rx),
    });
    app.world_mut().insert_resource(ViewerState::default());

    for id in 1..=8_u64 {
        tx.send(ViewerResponse::Event {
            event: WorldEvent {
                id,
                time: id,
                kind: agent_world::simulator::WorldEventKind::ActionRejected {
                    reason: agent_world::simulator::RejectReason::InvalidAmount {
                        amount: id as i64,
                    },
                },
            },
        })
        .expect("send event");
    }

    app.update();

    let state = app.world().resource::<ViewerState>();
    let ids: Vec<u64> = state.events.iter().map(|event| event.id).collect();
    assert_eq!(ids, vec![1, 3, 5, 6, 7, 8]);
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
fn lighting_illuminance_triplet_tracks_key_fill_rim_ratios() {
    let mut config = Viewer3dConfig::default();
    config.physical.enabled = true;
    config.physical.stellar_distance_au = 2.5;
    config.physical.luminous_efficacy_lm_per_w = 120.0;
    config.physical.exposure_ev100 = 13.5;
    config.lighting.fill_light_ratio = 0.30;
    config.lighting.rim_light_ratio = 0.10;

    let (key, fill, rim) = lighting_illuminance_triplet(&config);
    assert!(key > fill);
    assert!(fill > rim);
    assert!(((fill / key) - 0.30).abs() < 0.01);
    assert!(((rim / key) - 0.10).abs() < 0.01);
}

#[test]
fn lighting_illuminance_triplet_clamps_low_fill_and_rim() {
    let mut config = Viewer3dConfig::default();
    config.physical.enabled = true;
    config.physical.stellar_distance_au = 2.5;
    config.physical.luminous_efficacy_lm_per_w = 120.0;
    config.physical.exposure_ev100 = 13.5;
    config.lighting.fill_light_ratio = 0.0;
    config.lighting.rim_light_ratio = 0.0;

    let (_key, fill, rim) = lighting_illuminance_triplet(&config);
    assert!((fill - 800.0).abs() < f32::EPSILON);
    assert!((rim - 450.0).abs() < f32::EPSILON);
}

#[test]
fn camera_post_process_components_map_config_values() {
    let mut config = Viewer3dConfig::default();
    config.post_process.tonemapping = ViewerTonemappingMode::AcesFitted;
    config.post_process.deband_dither_enabled = true;
    config.post_process.bloom_enabled = true;
    config.post_process.bloom_intensity = 0.33;
    config.post_process.color_grading_exposure = 0.6;
    config.post_process.color_grading_post_saturation = 1.14;

    let (tonemapping, deband, grading, bloom) = camera_post_process_components(&config);
    assert_eq!(tonemapping, Tonemapping::AcesFitted);
    assert_eq!(deband, DebandDither::Enabled);
    assert!((grading.global.exposure - 0.6).abs() < f32::EPSILON);
    assert!((grading.global.post_saturation - 1.14).abs() < f32::EPSILON);
    assert!((bloom.expect("bloom").intensity - 0.33).abs() < f32::EPSILON);
}

#[test]
fn camera_post_process_components_disable_bloom_and_deband() {
    let mut config = Viewer3dConfig::default();
    config.post_process.tonemapping = ViewerTonemappingMode::None;
    config.post_process.deband_dither_enabled = false;
    config.post_process.bloom_enabled = false;
    config.post_process.color_grading_exposure = -0.35;
    config.post_process.color_grading_post_saturation = 0.82;

    let (tonemapping, deband, grading, bloom) = camera_post_process_components(&config);
    assert_eq!(tonemapping, Tonemapping::None);
    assert_eq!(deband, DebandDither::Disabled);
    assert!((grading.global.exposure + 0.35).abs() < f32::EPSILON);
    assert!((grading.global.post_saturation - 0.82).abs() < f32::EPSILON);
    assert!(bloom.is_none());
}

#[test]
fn resolve_srgb_slot_color_prefers_override() {
    let resolved = resolve_srgb_slot_color([0.11, 0.22, 0.33], Some([0.44, 0.55, 0.66]));
    assert!((resolved[0] - 0.44).abs() < f32::EPSILON);
    assert!((resolved[1] - 0.55).abs() < f32::EPSILON);
    assert!((resolved[2] - 0.66).abs() < f32::EPSILON);
}

#[test]
fn emissive_from_srgb_with_boost_clamps_components() {
    let clamped = emissive_from_srgb_with_boost([1.0, 1.0, 1.0], 6.0);
    let clamped_more = emissive_from_srgb_with_boost([1.0, 1.0, 1.0], 100.0);
    assert!((clamped.red - clamped_more.red).abs() < f32::EPSILON);
    assert!((clamped.green - clamped_more.green).abs() < f32::EPSILON);
    assert!((clamped.blue - clamped_more.blue).abs() < f32::EPSILON);
}

#[test]
fn location_material_override_enabled_detects_any_slot_override() {
    let empty = ViewerExternalMaterialSlotConfig::default();
    assert!(!location_material_override_enabled(empty));

    let base_only = ViewerExternalMaterialSlotConfig {
        base_color_srgb: Some([0.2, 0.3, 0.4]),
        emissive_color_srgb: None,
    };
    assert!(location_material_override_enabled(base_only));

    let emissive_only = ViewerExternalMaterialSlotConfig {
        base_color_srgb: None,
        emissive_color_srgb: Some([0.6, 0.7, 0.8]),
    };
    assert!(location_material_override_enabled(emissive_only));
}

#[test]
fn texture_slot_override_enabled_detects_base_texture_override() {
    let empty = ViewerExternalTextureSlotConfig::default();
    assert!(!texture_slot_override_enabled(&empty));

    let texture_override = ViewerExternalTextureSlotConfig {
        base_texture_asset: Some("textures/world/location_albedo.png".to_string()),
        ..ViewerExternalTextureSlotConfig::default()
    };
    assert!(texture_slot_override_enabled(&texture_override));
}

#[test]
fn location_style_override_enabled_detects_material_or_texture_override() {
    let material_empty = ViewerExternalMaterialSlotConfig::default();
    let texture_empty = ViewerExternalTextureSlotConfig::default();
    assert!(!location_style_override_enabled(
        material_empty,
        &texture_empty
    ));

    let material_only = ViewerExternalMaterialSlotConfig {
        base_color_srgb: Some([0.3, 0.5, 0.7]),
        emissive_color_srgb: None,
    };
    assert!(location_style_override_enabled(
        material_only,
        &texture_empty
    ));

    let texture_only = ViewerExternalTextureSlotConfig {
        base_texture_asset: Some("textures/world/location_albedo.png".to_string()),
        ..ViewerExternalTextureSlotConfig::default()
    };
    assert!(location_style_override_enabled(
        material_empty,
        &texture_only
    ));
}

#[path = "tests_scene_entities.rs"]
mod tests_scene_entities;

#[path = "tests_selection_panels.rs"]
mod tests_selection_panels;

fn spawn_label_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let origin = GeoPos::new(0.0, 0.0, 0.0);
    spawn_location_entity_with_radiation(
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
        0,
    );
}

fn spawn_location_scale_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let origin = GeoPos::new(0.0, 0.0, 0.0);
    spawn_location_entity_with_radiation(
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
        0,
    );
}

fn spawn_location_detail_ring_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let origin = GeoPos::new(0.0, 0.0, 0.0);
    spawn_location_entity_with_radiation(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "loc-detail-ring",
        "DetailRing",
        GeoPos::new(0.0, 0.0, 0.0),
        MaterialKind::Silicate,
        20_000,
        0,
    );
}

fn spawn_location_detail_halo_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let origin = GeoPos::new(0.0, 0.0, 0.0);
    spawn_location_entity_with_radiation(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "loc-detail-halo",
        "DetailHalo",
        GeoPos::new(0.0, 0.0, 0.0),
        MaterialKind::Metal,
        6_000,
        10_000,
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
        None,
        GeoPos::new(0.0, 0.0, 0.0),
        200,
        5,
    );
}

fn spawn_agent_surface_attachment_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let origin = GeoPos::new(0.0, 0.0, 0.0);
    spawn_location_entity_with_radiation(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "loc-surface",
        "Surface",
        GeoPos::new(0.0, 0.0, 0.0),
        MaterialKind::Silicate,
        240,
        0,
    );
    spawn_agent_entity(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "agent-surface",
        Some("loc-surface"),
        GeoPos::new(0.0, 0.0, 0.0),
        100,
        6,
    );
}

fn spawn_agent_surface_standoff_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let origin = GeoPos::new(0.0, 0.0, 0.0);
    spawn_location_entity_with_radiation(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "loc-surface-standoff",
        "SurfaceStandoff",
        GeoPos::new(0.0, 0.0, 0.0),
        MaterialKind::Silicate,
        240,
        0,
    );
    spawn_agent_entity(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        origin,
        "agent-surface-standoff",
        Some("loc-surface-standoff"),
        GeoPos::new(0.0, 0.0, 5_240.0),
        100,
        6,
    );
}

fn spawn_agent_module_marker_count_test_system(
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
        "agent-module-cap",
        None,
        GeoPos::new(0.0, 0.0, 0.0),
        180,
        24,
    );
}

fn spawn_agent_robot_layout_test_system(
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
        "agent-robot-layout",
        None,
        GeoPos::new(0.0, 0.0, 0.0),
        180,
        8,
    );
}

fn rebuild_scene_module_count_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let mut model = agent_world::simulator::WorldModel::default();
    model.agents.insert(
        "agent-modules".to_string(),
        agent_world::simulator::Agent::new("agent-modules", "loc-1", GeoPos::new(0.0, 0.0, 0.0)),
    );
    model.locations.insert(
        "loc-1".to_string(),
        agent_world::simulator::Location::new("loc-1", "Loc", GeoPos::new(0.0, 0.0, 0.0)),
    );
    model.module_visual_entities.insert(
        "mv-1".to_string(),
        agent_world::simulator::ModuleVisualEntity {
            entity_id: "mv-1".to_string(),
            module_id: "m.power".to_string(),
            kind: "artifact".to_string(),
            label: None,
            anchor: agent_world::simulator::ModuleVisualAnchor::Agent {
                agent_id: "agent-modules".to_string(),
            },
        },
    );
    model.module_visual_entities.insert(
        "mv-2".to_string(),
        agent_world::simulator::ModuleVisualEntity {
            entity_id: "mv-2".to_string(),
            module_id: "m.sensor".to_string(),
            kind: "artifact".to_string(),
            label: None,
            anchor: agent_world::simulator::ModuleVisualAnchor::Agent {
                agent_id: "agent-modules".to_string(),
            },
        },
    );

    let snapshot = agent_world::simulator::WorldSnapshot {
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

    rebuild_scene_from_snapshot(&mut commands, &config, &assets, &mut scene, &snapshot);
}

fn rebuild_scene_default_module_count_test_system(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
) {
    let mut model = agent_world::simulator::WorldModel::default();
    model.agents.insert(
        "agent-default-modules".to_string(),
        agent_world::simulator::Agent::new(
            "agent-default-modules",
            "loc-1",
            GeoPos::new(0.0, 0.0, 0.0),
        ),
    );
    model.locations.insert(
        "loc-1".to_string(),
        agent_world::simulator::Location::new("loc-1", "Loc", GeoPos::new(0.0, 0.0, 0.0)),
    );

    let snapshot = agent_world::simulator::WorldSnapshot {
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

    rebuild_scene_from_snapshot(&mut commands, &config, &assets, &mut scene, &snapshot);
}
