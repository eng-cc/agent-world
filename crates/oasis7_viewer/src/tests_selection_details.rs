use super::tests_ui_text::{build_selection_details_text, default_locale};
use super::*;

#[test]
fn update_ui_populates_agent_selection_details_with_llm_trace() {
    let selection = ViewerSelection {
        current: Some(SelectionInfo {
            entity: Entity::from_raw_u32(1).expect("entity"),
            kind: SelectionKind::Agent,
            id: "agent-1".to_string(),
            name: None,
        }),
    };

    let mut model = oasis7::simulator::WorldModel::default();
    model.locations.insert(
        "loc-1".to_string(),
        oasis7::simulator::Location::new(
            "loc-1",
            "Alpha",
            oasis7::geometry::GeoPos::new(0.0, 0.0, 0.0),
        ),
    );
    model.agents.insert(
        "agent-1".to_string(),
        oasis7::simulator::Agent::new(
            "agent-1",
            "loc-1",
            oasis7::geometry::GeoPos::new(1.0, 2.0, 3.0),
        ),
    );

    let snapshot = oasis7::simulator::WorldSnapshot {
        version: oasis7::simulator::SNAPSHOT_VERSION,
        chunk_generation_schema_version: oasis7::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
        time: 11,
        config: oasis7::simulator::WorldConfig::default(),
        model,
        chunk_runtime: oasis7::simulator::ChunkRuntimeConfig::default(),
        next_event_id: 3,
        next_action_id: 2,
        pending_actions: Vec::new(),
        journal_len: 0,
        runtime_snapshot: None,
        player_gameplay: None,
    };

    let events = vec![WorldEvent {
        id: 2,
        time: 10,
        kind: oasis7::simulator::WorldEventKind::AgentMoved {
            agent_id: "agent-1".to_string(),
            from: "loc-0".to_string(),
            to: "loc-1".to_string(),
            distance_cm: 100,
            electricity_cost: 2,
        },
        runtime_event: None,
    }];

    let decision_traces = vec![oasis7::simulator::AgentDecisionTrace {
        agent_id: "agent-1".to_string(),
        time: 10,
        decision: oasis7::simulator::AgentDecision::Wait,
        llm_input: Some("prompt content".to_string()),
        llm_output: Some("{\"decision\":\"wait\"}".to_string()),
        llm_error: None,
        parse_error: None,
        llm_diagnostics: Some(oasis7::simulator::LlmDecisionDiagnostics {
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
        llm_chat_messages: Vec::new(),
    }];

    let state = ViewerState {
        status: ConnectionStatus::Connected,
        snapshot: Some(snapshot),
        events,
        decision_traces,
        metrics: None,
    };
    let locale = default_locale();
    let viewer_config = Viewer3dConfig::default();
    let details_text =
        build_selection_details_text(&selection, &state, Some(&viewer_config), locale);

    assert!(details_text.contains("Details: agent agent-1"));
    assert!(details_text.contains("Body Size: data_height=1.00m (100cm)"));
    assert!(details_text.contains("Location Radius: 100cm (1.00m)"));
    assert!(details_text.contains("Scale Ratio: height/location_radius=1.000"));
    assert!(details_text.contains("Thermal Visual: ratio=0.00 color=heat_low"));
    assert!(details_text.contains("Recent LLM I/O"));
    assert!(details_text.contains("input:"));
    assert!(details_text.contains("output:"));
    assert!(details_text.contains("model: gpt-4o-mini"));
    assert!(details_text.contains("latency_ms: 123"));
    assert!(details_text.contains("tokens: prompt=77 completion=9 total=86"));
    assert!(details_text.contains("retries: 1"));
}

#[test]
fn provider_debug_summary_filters_openclaw_and_errors() {
    let state = ViewerState {
        status: ConnectionStatus::Connected,
        snapshot: None,
        events: Vec::new(),
        decision_traces: vec![
            oasis7::simulator::AgentDecisionTrace {
                agent_id: "agent-1".to_string(),
                time: 12,
                decision: oasis7::simulator::AgentDecision::Act(
                    oasis7::simulator::Action::MoveAgent {
                        agent_id: "agent-1".to_string(),
                        to: "loc-2".to_string(),
                    },
                ),
                llm_input: Some("provider request".to_string()),
                llm_output: Some("move to loc-2".to_string()),
                llm_error: None,
                parse_error: None,
                llm_diagnostics: Some(oasis7::simulator::LlmDecisionDiagnostics {
                    model: Some("openclaw-local".to_string()),
                    latency_ms: Some(87),
                    prompt_tokens: None,
                    completion_tokens: None,
                    total_tokens: None,
                    retry_count: 0,
                }),
                llm_effect_intents: Vec::new(),
                llm_effect_receipts: Vec::new(),
                llm_step_trace: Vec::new(),
                llm_prompt_section_trace: Vec::new(),
                llm_chat_messages: Vec::new(),
            },
            oasis7::simulator::AgentDecisionTrace {
                agent_id: "agent-2".to_string(),
                time: 13,
                decision: oasis7::simulator::AgentDecision::Wait,
                llm_input: Some("builtin request".to_string()),
                llm_output: None,
                llm_error: Some("provider timeout".to_string()),
                parse_error: None,
                llm_diagnostics: Some(oasis7::simulator::LlmDecisionDiagnostics {
                    model: Some("builtin-llm".to_string()),
                    latency_ms: Some(3010),
                    prompt_tokens: None,
                    completion_tokens: None,
                    total_tokens: None,
                    retry_count: 1,
                }),
                llm_effect_intents: Vec::new(),
                llm_effect_receipts: Vec::new(),
                llm_step_trace: Vec::new(),
                llm_prompt_section_trace: Vec::new(),
                llm_chat_messages: Vec::new(),
            },
        ],
        metrics: None,
    };

    let openclaw_text = super::tests_ui_text::build_provider_debug_text(
        &state,
        crate::ui_text::ProviderDebugFilter::OpenClawOnly,
    );
    assert!(openclaw_text.contains("filter=openclaw_only"));
    assert!(openclaw_text.contains("provider=openclaw-local"));
    assert!(openclaw_text.contains("move_agent -> loc-2"));
    assert!(openclaw_text.contains("Recent Latency: t12=87ms"));

    let error_text = super::tests_ui_text::build_provider_debug_text(
        &state,
        crate::ui_text::ProviderDebugFilter::ErrorsOnly,
    );
    assert!(error_text.contains("filter=errors_only"));
    assert!(error_text.contains("Last Error: provider timeout"));
    assert!(error_text.contains("provider=builtin-llm"));
}

#[test]
fn update_ui_populates_location_selection_details() {
    let mut viewer_config = Viewer3dConfig::default();
    viewer_config.physical.reference_radiation_area_m2 = 2.0;
    let selection = ViewerSelection {
        current: Some(SelectionInfo {
            entity: Entity::from_raw_u32(2).expect("entity"),
            kind: SelectionKind::Location,
            id: "loc-1".to_string(),
            name: Some("Alpha".to_string()),
        }),
    };

    let mut model = oasis7::simulator::WorldModel::default();
    let mut location = oasis7::simulator::Location::new_with_profile(
        "loc-1",
        "Alpha",
        oasis7::geometry::GeoPos::new(0.0, 0.0, 0.0),
        oasis7::simulator::LocationProfile {
            material: oasis7::simulator::MaterialKind::Silicate,
            radius_cm: 320,
            radiation_emission_per_tick: 9,
        },
    );
    let mut fragment_budget = oasis7::simulator::FragmentResourceBudget::default();
    fragment_budget
        .total_by_element_g
        .insert(oasis7::simulator::FragmentElementKind::Iron, 1_000);
    fragment_budget
        .remaining_by_element_g
        .insert(oasis7::simulator::FragmentElementKind::Iron, 125);
    location.fragment_budget = Some(fragment_budget);

    model.locations.insert("loc-1".to_string(), location);

    let snapshot = oasis7::simulator::WorldSnapshot {
        version: oasis7::simulator::SNAPSHOT_VERSION,
        chunk_generation_schema_version: oasis7::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
        time: 3,
        config: oasis7::simulator::WorldConfig::default(),
        model,
        chunk_runtime: oasis7::simulator::ChunkRuntimeConfig::default(),
        next_event_id: 1,
        next_action_id: 1,
        pending_actions: Vec::new(),
        journal_len: 0,
        runtime_snapshot: None,
        player_gameplay: None,
    };

    let state = ViewerState {
        status: ConnectionStatus::Connected,
        snapshot: Some(snapshot),
        events: Vec::new(),
        decision_traces: Vec::new(),
        metrics: None,
    };
    let locale = default_locale();
    let details_text =
        build_selection_details_text(&selection, &state, Some(&viewer_config), locale);

    assert!(details_text.contains("Details: location loc-1"));
    assert!(details_text.contains("radiation/tick=9"));
    assert!(details_text.contains("Radiation Visual: power=900.00W flux=450.00W/m2 area=2.00m2"));
    assert!(details_text.contains("Fragment Depletion: mined=87.5% remaining=125g/1000g"));
}
