use super::*;

fn setup_kernel_with_provider_agent(agent_id: &str) -> WorldKernel {
    let mut kernel = WorldKernel::with_config(WorldConfig {
        move_cost_per_km_electricity: 0,
        ..Default::default()
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-1".to_string(),
        name: "base".to_string(),
        pos: pos(0.0, 0.0),
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterLocation {
        location_id: "loc-2".to_string(),
        name: "neighbor".to_string(),
        pos: pos(100.0, 0.0),
        profile: LocationProfile::default(),
    });
    kernel.submit_action(Action::RegisterAgent {
        agent_id: agent_id.to_string(),
        location_id: "loc-1".to_string(),
    });
    kernel.step_until_empty();
    kernel
}

fn provider_action_catalog() -> Vec<ActionCatalogEntry> {
    vec![
        ActionCatalogEntry::new("wait", "Skip this tick"),
        ActionCatalogEntry::new("move_agent", "Move the acting agent to a visible location"),
    ]
}

#[test]
fn golden_decision_provider_fixture_round_trips() {
    let fixtures = golden_decision_provider_fixtures();
    assert_eq!(fixtures.len(), 1);
    assert_eq!(fixtures[0].fixture_id, "golden.move.visible_location.v1");
    let encoded = serde_json::to_string_pretty(&fixtures[0]).expect("encode fixture");
    let decoded: GoldenDecisionFixture = serde_json::from_str(encoded.as_str()).expect("decode");
    assert_eq!(decoded, fixtures[0]);
    assert!(decoded
        .request
        .observation
        .action_catalog
        .iter()
        .any(|entry| entry.action_ref == "move_agent"));
    assert_eq!(
        decoded.request.observation.mode,
        ProviderExecutionMode::HeadlessAgent
    );
    assert_eq!(
        decoded.request.observation.observation_schema_version,
        DEFAULT_PROVIDER_OBSERVATION_SCHEMA_VERSION
    );
    assert_eq!(
        decoded.request.observation.action_schema_version,
        DEFAULT_PROVIDER_ACTION_SCHEMA_VERSION
    );
    assert_eq!(
        decoded.request.observation.environment_class.as_deref(),
        Some("golden_fixture")
    );
    assert_eq!(
        decoded.request.fixture_id.as_deref(),
        Some("golden.move.visible_location.v1")
    );
}

#[test]
fn provider_backed_agent_behavior_executes_mocked_move_and_records_feedback() {
    let mut kernel = setup_kernel_with_provider_agent("agent-1");
    let response = DecisionResponse {
        decision: ProviderDecision::Act {
            action_ref: "move_agent".to_string(),
            action: Action::MoveAgent {
                agent_id: "agent-1".to_string(),
                to: "loc-2".to_string(),
            },
        },
        provider_error: None,
        diagnostics: ProviderDiagnostics {
            provider_id: Some("mock-openclaw".to_string()),
            provider_version: Some("v0".to_string()),
            latency_ms: Some(42),
            retry_count: 0,
        },
        trace_payload: ProviderTraceEnvelope {
            provider_id: Some("mock-openclaw".to_string()),
            input_summary: Some("fixture=golden.move.visible_location.v1".to_string()),
            output_summary: Some("decision=move_agent(to=loc-2)".to_string()),
            latency_ms: Some(42),
            transcript: vec![ProviderTranscriptEntry {
                role: "agent".to_string(),
                content: "move to loc-2".to_string(),
            }],
            tool_trace: vec!["selected visible location loc-2".to_string()],
            token_usage: Some(ProviderTokenUsage {
                prompt_tokens: Some(10),
                completion_tokens: Some(5),
                total_tokens: Some(15),
            }),
            cost_cents: Some(1),
            schema_repair_count: 0,
        },
        memory_write_intents: vec![],
    };
    let provider =
        MockDecisionProvider::with_scripted_responses("mock-openclaw", vec![Ok(response)]);
    let shared_state = provider.shared_state();
    let behavior = ProviderBackedAgentBehavior::new("agent-1", provider, provider_action_catalog())
        .with_provider_config_ref("mock://openclaw-local-http")
        .with_agent_profile("oasis7_p0_low_freq_npc")
        .with_execution_mode(ProviderExecutionMode::PlayerParity)
        .with_environment_class("unit_test")
        .with_fallback_reason("parity_probe")
        .with_fixture_id("fixture.agent-1")
        .with_replay_id("replay.agent-1")
        .with_memory_summary("goal=move");

    let mut runner: AgentRunner<ProviderBackedAgentBehavior<MockDecisionProvider>> =
        AgentRunner::new();
    runner.register(behavior);

    let tick = runner.tick(&mut kernel).expect("provider-backed tick");
    assert!(matches!(
        tick.decision,
        AgentDecision::Act(Action::MoveAgent { .. })
    ));
    assert!(tick.is_success());
    let trace = tick.decision_trace.expect("trace emitted");
    assert_eq!(trace.llm_error, None);
    assert_eq!(
        trace.llm_diagnostics.and_then(|value| value.latency_ms),
        Some(42)
    );
    assert!(trace
        .llm_output
        .as_deref()
        .unwrap_or_default()
        .contains("move_agent"));

    let agent = kernel.model().agents.get("agent-1").expect("agent exists");
    assert_eq!(agent.location_id, "loc-2");

    let snapshot = shared_state.lock().expect("mock state lock").clone();
    assert_eq!(snapshot.recorded_requests.len(), 1);
    assert_eq!(snapshot.recorded_feedback.len(), 1);
    assert!(snapshot.recorded_feedback[0].success);
    assert_eq!(
        snapshot.recorded_requests[0].provider_config_ref.as_deref(),
        Some("mock://openclaw-local-http")
    );
    assert_eq!(
        snapshot.recorded_requests[0].agent_profile.as_deref(),
        Some("oasis7_p0_low_freq_npc")
    );
    assert!(snapshot.recorded_requests[0]
        .observation
        .action_catalog
        .iter()
        .any(|entry| entry.action_ref == "move_agent"));
    assert_eq!(
        snapshot.recorded_requests[0].observation.mode,
        ProviderExecutionMode::PlayerParity
    );
    assert_eq!(
        snapshot.recorded_requests[0]
            .observation
            .observation_schema_version,
        DEFAULT_PROVIDER_OBSERVATION_SCHEMA_VERSION
    );
    assert_eq!(
        snapshot.recorded_requests[0]
            .observation
            .action_schema_version,
        DEFAULT_PROVIDER_ACTION_SCHEMA_VERSION
    );
    assert_eq!(
        snapshot.recorded_requests[0]
            .observation
            .environment_class
            .as_deref(),
        Some("unit_test")
    );
    assert_eq!(
        snapshot.recorded_requests[0]
            .observation
            .fallback_reason
            .as_deref(),
        Some("parity_probe")
    );
    assert_eq!(
        snapshot.recorded_requests[0].fixture_id.as_deref(),
        Some("fixture.agent-1")
    );
    assert_eq!(
        snapshot.recorded_requests[0].replay_id.as_deref(),
        Some("replay.agent-1")
    );
}

#[test]
fn provider_backed_agent_behavior_downgrades_provider_error_to_wait() {
    let mut kernel = setup_kernel_with_provider_agent("agent-1");
    let provider = MockDecisionProvider::with_scripted_responses(
        "mock-openclaw",
        vec![Err(DecisionProviderError::new(
            "provider_timeout",
            "request exceeded 3000ms budget",
            true,
        ))],
    );
    let behavior = ProviderBackedAgentBehavior::new("agent-1", provider, provider_action_catalog());
    let mut runner: AgentRunner<ProviderBackedAgentBehavior<MockDecisionProvider>> =
        AgentRunner::new();
    runner.register(behavior);

    let tick = runner.tick(&mut kernel).expect("provider tick");
    assert!(matches!(tick.decision, AgentDecision::Wait));
    assert!(tick.action_result.is_none());
    let trace = tick.decision_trace.expect("error trace emitted");
    assert!(trace
        .llm_error
        .as_deref()
        .unwrap_or_default()
        .contains("provider_timeout"));
}

#[test]
fn provider_backed_agent_behavior_rejects_unknown_action_ref() {
    let mut kernel = setup_kernel_with_provider_agent("agent-1");
    let response = DecisionResponse {
        decision: ProviderDecision::Act {
            action_ref: "unknown_action".to_string(),
            action: Action::MoveAgent {
                agent_id: "agent-1".to_string(),
                to: "loc-2".to_string(),
            },
        },
        provider_error: None,
        diagnostics: ProviderDiagnostics {
            provider_id: Some("mock-openclaw".to_string()),
            provider_version: None,
            latency_ms: Some(10),
            retry_count: 0,
        },
        trace_payload: ProviderTraceEnvelope {
            provider_id: Some("mock-openclaw".to_string()),
            output_summary: Some("decision=unknown_action".to_string()),
            ..ProviderTraceEnvelope::default()
        },
        memory_write_intents: vec![],
    };
    let provider =
        MockDecisionProvider::with_scripted_responses("mock-openclaw", vec![Ok(response)]);
    let behavior = ProviderBackedAgentBehavior::new("agent-1", provider, provider_action_catalog());
    let mut runner: AgentRunner<ProviderBackedAgentBehavior<MockDecisionProvider>> =
        AgentRunner::new();
    runner.register(behavior);

    let tick = runner.tick(&mut kernel).expect("provider tick");
    assert!(matches!(tick.decision, AgentDecision::Wait));
    assert!(tick.action_result.is_none());
    let trace = tick.decision_trace.expect("trace emitted");
    assert!(trace
        .parse_error
        .as_deref()
        .unwrap_or_default()
        .contains("unknown action_ref"));
}
