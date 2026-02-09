use super::*;

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
    app.world_mut().insert_resource(Viewer3dConfig::default());

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
        llm_diagnostics: Some(agent_world::simulator::LlmDecisionDiagnostics {
            model: Some("gpt-4o-mini".to_string()),
            latency_ms: Some(123),
            prompt_tokens: Some(77),
            completion_tokens: Some(9),
            total_tokens: Some(86),
            retry_count: 1,
        }),
        llm_effect_intents: Vec::new(),
        llm_effect_receipts: Vec::new(),
        llm_step_trace: Vec::new(),
        llm_prompt_section_trace: Vec::new(),
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
    assert!(details_text
        .0
        .contains("Thermal Visual: ratio=0.00 color=heat_low"));
    assert!(details_text.0.contains("Recent LLM I/O"));
    assert!(details_text.0.contains("input:"));
    assert!(details_text.0.contains("output:"));
    assert!(details_text.0.contains("model: gpt-4o-mini"));
    assert!(details_text.0.contains("latency_ms: 123"));
    assert!(details_text
        .0
        .contains("tokens: prompt=77 completion=9 total=86"));
    assert!(details_text.0.contains("retries: 1"));
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
    let mut viewer_config = Viewer3dConfig::default();
    viewer_config.physical.reference_radiation_area_m2 = 2.0;
    app.world_mut().insert_resource(viewer_config);

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
    assert!(details_text
        .0
        .contains("Radiation Visual: power=900.00W flux=450.00W/m2 area=2.00m2"));
}
