use super::*;

#[test]
fn llm_agent_repair_round_can_recover_invalid_output() {
    let client = SequenceMockClient::new(vec![
        "not-json".to_string(),
        r#"{"decision":"wait_ticks","ticks":2}"#.to_string(),
    ]);
    let mut config = base_config();
    config.max_repair_rounds = 1;
    config.max_decision_steps = 4;
    let mut behavior = LlmAgentBehavior::new("agent-1", config, client);

    let decision = behavior.decide(&make_observation());
    assert_eq!(decision, AgentDecision::WaitTicks(2));

    let trace = behavior.take_decision_trace().expect("trace exists");
    let diagnostics = trace.llm_diagnostics.expect("diagnostics");
    assert_eq!(diagnostics.retry_count, 1);
    assert!(trace
        .llm_step_trace
        .iter()
        .any(|step| step.step_type == "repair"));
}

#[test]
fn llm_agent_long_run_stress_keeps_pipeline_stable() {
    const TICKS: usize = 240;
    let calls = Arc::new(AtomicUsize::new(0));
    let client = StressMockClient::new(Arc::clone(&calls));

    let mut config = base_config();
    config.max_decision_steps = 6;
    config.max_module_calls = 2;
    config.max_repair_rounds = 1;
    config.prompt_max_history_items = 2;
    config.prompt_profile = LlmPromptProfile::Compact;
    config.execute_until_auto_reenter_ticks = 0;

    let mut behavior = LlmAgentBehavior::new("agent-1", config, client);

    for tick in 0..TICKS {
        let time = 10_000 + tick as u64;
        behavior
            .memory
            .record_note(time, format!("stress-note-{tick}-{}", "x".repeat(180)));
        let observation = make_dense_observation(time, 8);

        let decision = behavior.decide(&observation);
        assert!(matches!(
            decision,
            AgentDecision::Act(Action::MoveAgent { .. })
                | AgentDecision::Wait
                | AgentDecision::WaitTicks(_)
        ));

        let trace = behavior.take_decision_trace().expect("trace exists");
        assert!(trace.llm_error.is_none());
        if let Some(parse_error) = trace.parse_error.as_deref() {
            assert!(
                parse_error.contains("deprecated in dialogue mode")
                    || parse_error.contains("no terminal decision")
                    || parse_error.contains("no actionable")
                    || parse_error.contains("replan guard requires"),
                "unexpected parse_error: {parse_error}"
            );
        }
        assert!(trace.llm_effect_intents.len() <= 1);
        assert_eq!(
            trace.llm_effect_receipts.len(),
            trace.llm_effect_intents.len()
        );
        assert!(trace
            .llm_step_trace
            .iter()
            .any(|step| step.step_type == "dialogue_turn" || step.step_type == "repair"));
        assert!(!trace.llm_prompt_section_trace.is_empty());
        let input_len = trace.llm_input.unwrap_or_default().len();
        assert!(input_len < 120_000, "llm_input too large: {input_len}");
        assert!(
            trace
                .llm_diagnostics
                .as_ref()
                .map(|diagnostics| diagnostics.retry_count)
                .unwrap_or_default()
                <= 1
        );
    }

    let total_calls = calls.load(Ordering::SeqCst);
    assert!(total_calls >= TICKS * 2);
    assert!(total_calls <= TICKS * 4);
}

#[test]
fn llm_agent_limits_module_call_rounds() {
    let client = SequenceMockClient::new(vec![
        "{\"type\":\"module_call\",\"module\":\"agent.modules.list\",\"args\":{}}".to_string(),
        "{\"type\":\"module_call\",\"module\":\"agent.modules.list\",\"args\":{}}".to_string(),
    ]);

    let mut config = base_config();
    config.max_module_calls = 1;
    let mut behavior = LlmAgentBehavior::new("agent-1", config, client);

    let decision = behavior.decide(&make_observation());
    assert_eq!(decision, AgentDecision::Wait);

    let trace = behavior.take_decision_trace().expect("trace exists");
    assert_eq!(trace.llm_effect_intents.len(), 1);
    assert_eq!(trace.llm_effect_receipts.len(), 1);
}

#[test]
fn llm_agent_system_prompt_contains_configured_goals() {
    let behavior = LlmAgentBehavior::new("agent-1", base_config(), MockClient::default());
    let system_prompt = behavior.system_prompt();
    assert!(system_prompt.contains("short-goal"));
    assert!(system_prompt.contains("long-goal"));
    assert!(system_prompt.contains("agent_submit_decision"));
}

#[test]
fn llm_agent_runtime_prompt_overrides_take_effect() {
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), MockClient::default());
    behavior.apply_prompt_overrides(
        Some("runtime-system".to_string()),
        Some("runtime-short".to_string()),
        Some("runtime-long".to_string()),
    );

    let system_prompt = behavior.system_prompt();
    assert!(system_prompt.contains("runtime-system"));
    assert!(system_prompt.contains("runtime-short"));
    assert!(system_prompt.contains("runtime-long"));
    assert!(!system_prompt.contains("short-goal"));
    assert!(!system_prompt.contains("long-goal"));
}

#[test]
fn llm_agent_user_prompt_omits_step_context_metadata() {
    let behavior = LlmAgentBehavior::new("agent-1", base_config(), MockClient::default());
    let prompt = behavior.user_prompt(&make_observation(), &[], 2, 5);
    assert!(!prompt.contains("step_index"));
    assert!(!prompt.contains("max_steps"));
    assert!(!prompt.contains("module_calls_used"));
    assert!(!prompt.contains("module_calls_max"));
    assert!(prompt.contains("[Conversation]"));
    assert!(prompt.contains("harvest_radiation"));
    assert!(prompt.contains("max_amount"));
    assert!(prompt.contains(format!("不超过 {}", DEFAULT_LLM_HARVEST_MAX_AMOUNT_CAP).as_str()));
}

#[test]
fn llm_agent_user_prompt_contains_failure_recovery_policy() {
    let behavior = LlmAgentBehavior::new("agent-1", base_config(), MockClient::default());
    let prompt = behavior.user_prompt(&make_observation(), &[], 0, 4);
    assert!(prompt.contains("[Failure Recovery Policy]"));
    assert!(prompt.contains("insufficient_resource.hardware -> refine_compound"));
    assert!(prompt.contains("insufficient_resource.electricity -> harvest_radiation"));
    assert!(prompt.contains("factory_not_found -> build_factory"));
}

#[test]
fn llm_agent_user_prompt_includes_last_action_summary_after_feedback() {
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), MockClient::default());
    let action_result = ActionResult {
        action: Action::BuildFactory {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            location_id: "loc-1".to_string(),
            factory_id: "factory.alpha".to_string(),
            factory_kind: "factory.assembler.mk1".to_string(),
        },
        action_id: 9,
        success: false,
        event: WorldEvent {
            id: 9,
            time: 11,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::InsufficientResource {
                    owner: ResourceOwner::Agent {
                        agent_id: "agent-1".to_string(),
                    },
                    kind: ResourceKind::Hardware,
                    requested: 10,
                    available: 0,
                },
            },
        },
    };

    behavior.on_action_result(&action_result);
    let prompt = behavior.user_prompt(&make_observation(), &[], 0, 4);
    assert!(prompt.contains("\"last_action\""));
    assert!(prompt.contains("\"kind\":\"build_factory\""));
    assert!(prompt.contains("\"success\":false"));
    assert!(prompt.contains("\"reject_reason\":\"insufficient_resource.hardware\""));
}

#[test]
fn llm_agent_user_prompt_preserves_facility_already_exists_reject_reason() {
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), MockClient::default());
    let action_result = ActionResult {
        action: Action::BuildFactory {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            location_id: "loc-1".to_string(),
            factory_id: "factory.alpha".to_string(),
            factory_kind: "factory.assembler.mk1".to_string(),
        },
        action_id: 10,
        success: false,
        event: WorldEvent {
            id: 10,
            time: 12,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::FacilityAlreadyExists {
                    facility_id: "factory.alpha".to_string(),
                },
            },
        },
    };

    behavior.on_action_result(&action_result);
    let prompt = behavior.user_prompt(&make_observation(), &[], 0, 4);
    assert!(prompt.contains("\"last_action\""));
    assert!(prompt.contains("\"kind\":\"build_factory\""));
    assert!(prompt.contains("\"success\":false"));
    assert!(prompt.contains("\"reject_reason\":\"facility_already_exists\""));
    assert!(!prompt.contains("\"reject_reason\":\"other\""));
}

#[test]
fn llm_agent_user_prompt_contains_memory_digest_section() {
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), MockClient::default());
    behavior
        .memory
        .record_note(7, "recent-memory-note-for-prompt");

    let prompt = behavior.user_prompt(&make_observation(), &[], 0, 4);
    assert!(prompt.contains("[Memory Digest]"));
    assert!(prompt.contains("recent-memory-note-for-prompt"));
}

#[test]
fn llm_agent_user_prompt_respects_history_item_cap() {
    let mut config = base_config();
    config.prompt_max_history_items = 2;
    let behavior = LlmAgentBehavior::new("agent-1", config, MockClient::default());

    let history = vec![
        ModuleCallExchange {
            module: "mod-a".to_string(),
            args: serde_json::json!({}),
            result: serde_json::json!({"ok": true}),
        },
        ModuleCallExchange {
            module: "mod-b".to_string(),
            args: serde_json::json!({}),
            result: serde_json::json!({"ok": true}),
        },
        ModuleCallExchange {
            module: "mod-c".to_string(),
            args: serde_json::json!({}),
            result: serde_json::json!({"ok": true}),
        },
    ];

    let prompt = behavior.user_prompt(&make_observation(), &history, 0, 4);
    assert!(!prompt.contains("mod-a"));
    assert!(prompt.contains("mod-b"));
    assert!(prompt.contains("mod-c"));
}

#[test]
fn llm_agent_compacts_large_module_result_payload_for_prompt_history() {
    let giant_payload = format!("payload-{}", "x".repeat(6000));
    let compact = LlmAgentBehavior::<MockClient>::module_result_for_prompt(&serde_json::json!({
        "ok": true,
        "module": "memory.short_term.recent",
        "result": [giant_payload.clone()],
    }));

    let compact_json = serde_json::to_string(&compact).expect("serialize compact result");
    assert!(compact_json.contains("\"truncated\":true"));
    assert!(compact_json.contains("\"original_chars\":"));
    assert!(!compact_json.contains(giant_payload.as_str()));
}

#[test]
fn llm_agent_compacts_dense_observation_for_prompt_context() {
    let behavior = LlmAgentBehavior::new("agent-1", base_config(), MockClient::default());
    let observation = make_dense_observation(42, 40);

    let prompt = behavior.user_prompt(&observation, &[], 0, 4);
    assert!(prompt.contains("\"visible_agents_total\":41"));
    assert!(prompt.contains("\"visible_agents_omitted\":"));
    assert!(prompt.contains("\"visible_locations_total\":41"));
    assert!(prompt.contains("\"visible_locations_omitted\":"));
    assert!(prompt.contains("\"self_resources\""));
    assert!(prompt.contains("\"electricity\":30"));
    assert!(!prompt.contains("agent-extra-39"));
    assert!(!prompt.contains("loc-extra-39"));
}

#[test]
fn llm_agent_compacts_large_module_args_payload_for_prompt_history() {
    let giant_query = format!("query-{}", "x".repeat(4_000));
    let history = vec![ModuleCallExchange {
        module: "memory.long_term.search".to_string(),
        args: serde_json::json!({"query": giant_query.clone()}),
        result: serde_json::json!({"ok": true}),
    }];

    let history_json = LlmAgentBehavior::<MockClient>::module_history_json_for_prompt(&history);
    assert!(history_json.contains("\"truncated\":true"));
    assert!(!history_json.contains(giant_query.as_str()));
}

#[test]
fn llm_agent_records_failed_action_into_long_term_memory() {
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), MockClient::default());
    let result = ActionResult {
        action: Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-x".to_string(),
        },
        action_id: 11,
        success: false,
        event: WorldEvent {
            id: 3,
            time: 9,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::AgentNotFound {
                    agent_id: "agent-1".to_string(),
                },
            },
        },
    };

    behavior.on_action_result(&result);

    assert!(!behavior.memory.long_term.is_empty());
    let failed = behavior.memory.long_term.search_by_tag("failed");
    assert!(!failed.is_empty());
}

#[test]
fn llm_agent_emits_parse_error_in_trace() {
    let client = MockClient {
        output: Some("not json".to_string()),
        err: None,
    };
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);

    let decision = behavior.decide(&make_observation());
    assert_eq!(decision, AgentDecision::Wait);

    let trace = behavior.take_decision_trace().expect("trace should exist");
    assert!(trace.parse_error.is_some());
    assert!(trace
        .llm_output
        .as_deref()
        .unwrap_or_default()
        .contains("not json"));
    let diagnostics = trace.llm_diagnostics.as_ref().expect("diagnostics");
    assert_eq!(diagnostics.model.as_deref(), Some("gpt-test"));
    assert_eq!(diagnostics.retry_count, 1);
}

#[test]
fn llm_agent_force_replan_after_repeated_actions() {
    let client = SequenceMockClient::new(vec![
        "{\"decision\":\"harvest_radiation\",\"max_amount\":5}".to_string(),
        "{\"decision\":\"harvest_radiation\",\"max_amount\":5}".to_string(),
        "{\"decision\":\"harvest_radiation\",\"max_amount\":5}".to_string(),
        "{\"type\":\"module_call\",\"module\":\"agent.modules.list\",\"args\":{}}".to_string(),
        "{\"decision\":\"move_agent\",\"to\":\"loc-2\"}".to_string(),
    ]);

    let mut config = base_config();
    config.max_decision_steps = 4;
    config.max_repair_rounds = 1;
    config.force_replan_after_same_action = 2;

    let mut behavior = LlmAgentBehavior::new("agent-1", config, client);

    let mut observation = make_observation();
    observation.time = 10;
    let decision_1 = behavior.decide(&observation);
    assert!(matches!(
        decision_1,
        AgentDecision::Act(Action::HarvestRadiation { .. })
    ));

    observation.time = 11;
    let decision_2 = behavior.decide(&observation);
    assert!(matches!(
        decision_2,
        AgentDecision::Act(Action::HarvestRadiation { .. })
    ));

    observation.time = 12;
    let decision_3 = behavior.decide(&observation);
    assert!(matches!(
        decision_3,
        AgentDecision::Act(Action::MoveAgent { .. })
    ));

    let trace = behavior.take_decision_trace().expect("trace exists");
    let llm_input = trace.llm_input.unwrap_or_default();
    assert!(llm_input.contains("[Anti-Repetition Guard]"));
    assert!(trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("replan guard requires")));
    assert!(trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("module_call")));
}

#[test]
fn llm_agent_force_replan_allows_switch_to_new_terminal_action_without_module_call() {
    let client = SequenceMockClient::new(vec![
        r#"{"decision":"harvest_radiation","max_amount":5}"#.to_string(),
        r#"{"decision":"harvest_radiation","max_amount":5}"#.to_string(),
        r#"{"decision":"move_agent","to":"loc-2"}"#.to_string(),
    ]);

    let mut config = base_config();
    config.max_decision_steps = 4;
    config.max_repair_rounds = 1;
    config.force_replan_after_same_action = 2;
    config.execute_until_auto_reenter_ticks = 0;

    let mut behavior = LlmAgentBehavior::new("agent-1", config, client);

    let mut observation = make_observation();
    observation.time = 40;
    let decision_1 = behavior.decide(&observation);
    assert!(matches!(
        decision_1,
        AgentDecision::Act(Action::HarvestRadiation { .. })
    ));

    observation.time = 41;
    let decision_2 = behavior.decide(&observation);
    assert!(matches!(
        decision_2,
        AgentDecision::Act(Action::HarvestRadiation { .. })
    ));

    observation.time = 42;
    let decision_3 = behavior.decide(&observation);
    assert!(matches!(
        decision_3,
        AgentDecision::Act(Action::MoveAgent { .. })
    ));

    let trace = behavior.take_decision_trace().expect("trace exists");
    assert!(trace.parse_error.is_none());
    assert!(trace.llm_effect_intents.is_empty());
    assert!(!trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("replan guard requires")));
}

#[test]
fn llm_agent_force_replan_breaks_repeated_harvest_loop_with_repair() {
    let client = SequenceMockClient::new(vec![
        r#"{"decision":"harvest_radiation","max_amount":5}"#.to_string(),
        r#"{"decision":"harvest_radiation","max_amount":5}"#.to_string(),
        r#"{"decision":"harvest_radiation","max_amount":5}"#.to_string(),
        r#"{"decision":"refine_compound","compound_mass_g":1000}"#.to_string(),
    ]);

    let mut config = base_config();
    config.max_decision_steps = 4;
    config.max_repair_rounds = 1;
    config.force_replan_after_same_action = 2;
    config.execute_until_auto_reenter_ticks = 0;

    let mut behavior = LlmAgentBehavior::new("agent-1", config, client);
    let mut observation = make_observation();

    observation.time = 70;
    let first = behavior.decide(&observation);
    assert!(matches!(
        first,
        AgentDecision::Act(Action::HarvestRadiation { max_amount: 5, .. })
    ));
    behavior.on_action_result(&ActionResult {
        action: Action::HarvestRadiation {
            agent_id: "agent-1".to_string(),
            max_amount: 5,
        },
        action_id: 701,
        success: true,
        event: WorldEvent {
            id: 801,
            time: 70,
            kind: WorldEventKind::RadiationHarvested {
                agent_id: "agent-1".to_string(),
                location_id: "loc-2".to_string(),
                amount: 5,
                available: 90,
            },
        },
    });

    observation.time = 71;
    let second = behavior.decide(&observation);
    assert!(matches!(
        second,
        AgentDecision::Act(Action::HarvestRadiation { max_amount: 5, .. })
    ));
    behavior.on_action_result(&ActionResult {
        action: Action::HarvestRadiation {
            agent_id: "agent-1".to_string(),
            max_amount: 5,
        },
        action_id: 702,
        success: true,
        event: WorldEvent {
            id: 802,
            time: 71,
            kind: WorldEventKind::RadiationHarvested {
                agent_id: "agent-1".to_string(),
                location_id: "loc-2".to_string(),
                amount: 4,
                available: 86,
            },
        },
    });

    observation.time = 72;
    let third = behavior.decide(&observation);
    assert!(matches!(
        third,
        AgentDecision::Act(Action::RefineCompound {
            compound_mass_g: 1000,
            ..
        })
    ));

    let trace = behavior.take_decision_trace().expect("trace exists");
    assert!(trace.parse_error.is_none());
    assert!(trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("replan guard requires")));
}

#[test]
fn llm_agent_mock_sequence_recovers_and_completes_factory_recipe_chain() {
    let world_config = crate::simulator::WorldConfig::default();
    let world_init = crate::simulator::WorldInitConfig::from_scenario(
        crate::simulator::WorldScenario::LlmBootstrap,
        &world_config,
    );
    let (mut kernel, _) =
        crate::simulator::initialize_kernel(world_config, world_init).expect("init kernel");
    let start_location_id = kernel
        .model()
        .agents
        .get("agent-0")
        .expect("agent exists")
        .location_id
        .clone();

    let client = SequenceMockClient::new(vec![
        format!(
            r#"{{"decision":"build_factory","owner":"self","location_id":"{}","factory_id":"factory.alpha","factory_kind":"factory.assembler.mk1"}}"#,
            start_location_id
        ),
        r#"{"decision":"refine_compound","owner":"self","compound_mass_g":7000}"#.to_string(),
        format!(
            r#"{{"decision":"build_factory","owner":"self","location_id":"{}","factory_id":"factory.alpha","factory_kind":"factory.assembler.mk1"}}"#,
            start_location_id
        ),
        r#"{"decision":"schedule_recipe","owner":"self","factory_id":"factory.alpha","recipe_id":"recipe.assembler.control_chip","batches":1}"#.to_string(),
    ]);

    let mut config = base_config();
    config.max_decision_steps = 1;
    config.max_repair_rounds = 0;
    config.execute_until_auto_reenter_ticks = 0;
    config.force_replan_after_same_action = 0;

    let behavior = LlmAgentBehavior::new("agent-0", config, client);
    let mut runner: crate::simulator::AgentRunner<LlmAgentBehavior<SequenceMockClient>> =
        crate::simulator::AgentRunner::new();
    runner.register(behavior);

    let tick1 = runner.tick(&mut kernel).expect("tick1");
    let action1 = tick1.action_result.expect("tick1 action");
    assert!(!action1.success);
    assert!(matches!(
        action1.reject_reason(),
        Some(RejectReason::InsufficientResource {
            kind: ResourceKind::Hardware,
            ..
        })
    ));
    let mut seeded_snapshot = kernel.snapshot();
    seeded_snapshot
        .model
        .agents
        .get_mut("agent-0")
        .expect("agent exists")
        .resources
        .add(ResourceKind::Compound, 7_000)
        .expect("seed compound for refine");
    seeded_snapshot
        .model
        .agents
        .get_mut("agent-0")
        .expect("agent exists")
        .resources
        .add(ResourceKind::Electricity, 20)
        .expect("seed electricity for schedule");
    let seeded_journal = kernel.journal_snapshot();
    kernel = crate::simulator::WorldKernel::from_snapshot(seeded_snapshot, seeded_journal)
        .expect("restore seeded kernel");

    let tick2 = runner.tick(&mut kernel).expect("tick2");
    let action2 = tick2.action_result.expect("tick2 action");
    assert!(action2.success);
    assert!(matches!(
        action2.event.kind,
        WorldEventKind::CompoundRefined {
            hardware_output: 7,
            ..
        }
    ));

    let tick3 = runner.tick(&mut kernel).expect("tick3");
    let action3 = tick3.action_result.expect("tick3 action");
    assert!(action3.success);
    assert!(matches!(
        action3.event.kind,
        WorldEventKind::FactoryBuilt { .. }
    ));

    let tick4 = runner.tick(&mut kernel).expect("tick4");
    let action4 = tick4.action_result.expect("tick4 action");
    assert!(action4.success);
    assert!(matches!(
        action4.event.kind,
        WorldEventKind::RecipeScheduled { .. }
    ));

    let factory = kernel
        .model()
        .factories
        .get("factory.alpha")
        .expect("factory exists");
    assert_eq!(factory.kind, "factory.assembler.mk1");

    let agent = kernel.model().agents.get("agent-0").expect("agent exists");
    assert_eq!(agent.resources.get(ResourceKind::Data), 9);
}

#[test]
fn llm_agent_execute_until_continues_without_llm_until_event() {
    let calls = Arc::new(AtomicUsize::new(0));
    let client = CountingSequenceMockClient::new(
        vec![
            "{\"decision\":\"execute_until\",\"action\":{\"decision\":\"harvest_radiation\",\"max_amount\":9},\"until\":{\"event\":\"new_visible_agent|new_visible_location\"},\"max_ticks\":3}".to_string(),
            "{\"decision\":\"move_agent\",\"to\":\"loc-2\"}".to_string(),
        ],
        Arc::clone(&calls),
    );

    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);

    let mut observation = make_observation();
    observation.time = 20;

    let first = behavior.decide(&observation);
    assert!(matches!(
        first,
        AgentDecision::Act(Action::HarvestRadiation { .. })
    ));

    observation.time = 21;
    let second = behavior.decide(&observation);
    assert!(matches!(
        second,
        AgentDecision::Act(Action::HarvestRadiation { .. })
    ));
    let second_trace = behavior.take_decision_trace().expect("second trace");
    assert!(second_trace.llm_input.is_none());
    assert!(second_trace
        .llm_output
        .unwrap_or_default()
        .contains("execute_until continue"));

    observation.time = 22;
    observation.visible_agents.push(ObservedAgent {
        agent_id: "agent-new".to_string(),
        location_id: "loc-new".to_string(),
        pos: GeoPos {
            x_cm: 5.0,
            y_cm: 1.0,
            z_cm: 0.0,
        },
        distance_cm: 5,
    });

    let third = behavior.decide(&observation);
    assert!(matches!(
        third,
        AgentDecision::Act(Action::MoveAgent { .. })
    ));

    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn llm_agent_auto_reentry_arms_execute_until_for_repeated_actions() {
    let calls = Arc::new(AtomicUsize::new(0));
    let client = CountingSequenceMockClient::new(
        vec![
            r#"{"decision":"harvest_radiation","max_amount":9}"#.to_string(),
            r#"{"decision":"harvest_radiation","max_amount":9}"#.to_string(),
            r#"{"decision":"move_agent","to":"loc-2"}"#.to_string(),
        ],
        Arc::clone(&calls),
    );

    let mut config = base_config();
    config.execute_until_auto_reenter_ticks = 3;
    config.force_replan_after_same_action = 6;
    let mut behavior = LlmAgentBehavior::new("agent-1", config, client);

    let mut observation = make_observation();
    observation.time = 26;
    let first = behavior.decide(&observation);
    assert!(matches!(
        first,
        AgentDecision::Act(Action::HarvestRadiation { max_amount: 9, .. })
    ));

    behavior.on_action_result(&ActionResult {
        action: Action::HarvestRadiation {
            agent_id: "agent-1".to_string(),
            max_amount: 9,
        },
        action_id: 301,
        success: true,
        event: WorldEvent {
            id: 401,
            time: 26,
            kind: WorldEventKind::RadiationHarvested {
                agent_id: "agent-1".to_string(),
                location_id: "loc-2".to_string(),
                amount: 9,
                available: 90,
            },
        },
    });

    observation.time = 27;
    let second = behavior.decide(&observation);
    assert!(matches!(
        second,
        AgentDecision::Act(Action::HarvestRadiation { max_amount: 9, .. })
    ));
    let second_trace = behavior.take_decision_trace().expect("second trace");
    assert!(second_trace
        .llm_step_trace
        .iter()
        .any(|step| step.step_type == "execute_until_auto_reentry"));

    behavior.on_action_result(&ActionResult {
        action: Action::HarvestRadiation {
            agent_id: "agent-1".to_string(),
            max_amount: 9,
        },
        action_id: 302,
        success: true,
        event: WorldEvent {
            id: 402,
            time: 27,
            kind: WorldEventKind::RadiationHarvested {
                agent_id: "agent-1".to_string(),
                location_id: "loc-2".to_string(),
                amount: 8,
                available: 82,
            },
        },
    });

    observation.time = 28;
    let third = behavior.decide(&observation);
    assert!(matches!(
        third,
        AgentDecision::Act(Action::HarvestRadiation { max_amount: 9, .. })
    ));
    let third_trace = behavior.take_decision_trace().expect("third trace");
    assert!(third_trace.llm_input.is_none());
    assert!(third_trace
        .llm_step_trace
        .iter()
        .any(|step| step.step_type == "execute_until_continue"));

    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn llm_agent_auto_reentry_can_be_disabled() {
    let calls = Arc::new(AtomicUsize::new(0));
    let client = CountingSequenceMockClient::new(
        vec![
            r#"{"decision":"harvest_radiation","max_amount":9}"#.to_string(),
            r#"{"decision":"harvest_radiation","max_amount":9}"#.to_string(),
            r#"{"decision":"move_agent","to":"loc-2"}"#.to_string(),
        ],
        Arc::clone(&calls),
    );

    let mut config = base_config();
    config.execute_until_auto_reenter_ticks = 0;
    config.force_replan_after_same_action = 6;
    let mut behavior = LlmAgentBehavior::new("agent-1", config, client);

    let mut observation = make_observation();
    observation.time = 29;
    let first = behavior.decide(&observation);
    assert!(matches!(
        first,
        AgentDecision::Act(Action::HarvestRadiation { max_amount: 9, .. })
    ));

    behavior.on_action_result(&ActionResult {
        action: Action::HarvestRadiation {
            agent_id: "agent-1".to_string(),
            max_amount: 9,
        },
        action_id: 303,
        success: true,
        event: WorldEvent {
            id: 403,
            time: 29,
            kind: WorldEventKind::RadiationHarvested {
                agent_id: "agent-1".to_string(),
                location_id: "loc-2".to_string(),
                amount: 9,
                available: 90,
            },
        },
    });

    observation.time = 30;
    let second = behavior.decide(&observation);
    assert!(matches!(
        second,
        AgentDecision::Act(Action::HarvestRadiation { max_amount: 9, .. })
    ));

    behavior.on_action_result(&ActionResult {
        action: Action::HarvestRadiation {
            agent_id: "agent-1".to_string(),
            max_amount: 9,
        },
        action_id: 304,
        success: true,
        event: WorldEvent {
            id: 404,
            time: 30,
            kind: WorldEventKind::RadiationHarvested {
                agent_id: "agent-1".to_string(),
                location_id: "loc-2".to_string(),
                amount: 8,
                available: 82,
            },
        },
    });

    observation.time = 31;
    let third = behavior.decide(&observation);
    assert!(matches!(
        third,
        AgentDecision::Act(Action::MoveAgent { .. })
    ));

    assert_eq!(calls.load(Ordering::SeqCst), 3);
}

#[test]
fn llm_agent_execute_until_stops_on_insufficient_electricity_reject_reason() {
    let calls = Arc::new(AtomicUsize::new(0));
    let client = CountingSequenceMockClient::new(
        vec![
            r#"{"decision":"execute_until","action":{"decision":"harvest_radiation","max_amount":6},"until":{"event":"insufficient_electricity"},"max_ticks":4}"#.to_string(),
            r#"{"decision":"move_agent","to":"loc-2"}"#.to_string(),
        ],
        Arc::clone(&calls),
    );
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);

    let mut observation = make_observation();
    observation.time = 30;
    let first = behavior.decide(&observation);
    assert!(matches!(
        first,
        AgentDecision::Act(Action::HarvestRadiation { max_amount: 6, .. })
    ));

    behavior.on_action_result(&ActionResult {
        action: Action::HarvestRadiation {
            agent_id: "agent-1".to_string(),
            max_amount: 6,
        },
        action_id: 101,
        success: false,
        event: WorldEvent {
            id: 201,
            time: 30,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::InsufficientResource {
                    owner: ResourceOwner::Agent {
                        agent_id: "agent-1".to_string(),
                    },
                    kind: ResourceKind::Electricity,
                    requested: 8,
                    available: 1,
                },
            },
        },
    });

    observation.time = 31;
    let second = behavior.decide(&observation);
    assert!(matches!(
        second,
        AgentDecision::Act(Action::MoveAgent { .. })
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn llm_agent_execute_until_stops_on_thermal_overload_reject_reason() {
    let calls = Arc::new(AtomicUsize::new(0));
    let client = CountingSequenceMockClient::new(
        vec![
            r#"{"decision":"execute_until","action":{"decision":"harvest_radiation","max_amount":7},"until":{"event":"thermal_overload"},"max_ticks":4}"#.to_string(),
            r#"{"decision":"move_agent","to":"loc-2"}"#.to_string(),
        ],
        Arc::clone(&calls),
    );
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);

    let mut observation = make_observation();
    observation.time = 40;
    let first = behavior.decide(&observation);
    assert!(matches!(
        first,
        AgentDecision::Act(Action::HarvestRadiation { max_amount: 7, .. })
    ));

    behavior.on_action_result(&ActionResult {
        action: Action::HarvestRadiation {
            agent_id: "agent-1".to_string(),
            max_amount: 7,
        },
        action_id: 102,
        success: false,
        event: WorldEvent {
            id: 202,
            time: 40,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::ThermalOverload {
                    heat: 130,
                    capacity: 100,
                },
            },
        },
    });

    observation.time = 41;
    let second = behavior.decide(&observation);
    assert!(matches!(
        second,
        AgentDecision::Act(Action::MoveAgent { .. })
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn llm_agent_execute_until_stops_on_harvest_yield_threshold() {
    let calls = Arc::new(AtomicUsize::new(0));
    let client = CountingSequenceMockClient::new(
        vec![
            r#"{"decision":"execute_until","action":{"decision":"harvest_radiation","max_amount":9},"until":{"event":"harvest_yield_below","value_lte":2},"max_ticks":4}"#.to_string(),
            r#"{"decision":"move_agent","to":"loc-2"}"#.to_string(),
        ],
        Arc::clone(&calls),
    );
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);

    let mut observation = make_observation();
    observation.time = 50;
    let first = behavior.decide(&observation);
    assert!(matches!(
        first,
        AgentDecision::Act(Action::HarvestRadiation { max_amount: 9, .. })
    ));

    behavior.on_action_result(&ActionResult {
        action: Action::HarvestRadiation {
            agent_id: "agent-1".to_string(),
            max_amount: 9,
        },
        action_id: 103,
        success: true,
        event: WorldEvent {
            id: 203,
            time: 50,
            kind: WorldEventKind::RadiationHarvested {
                agent_id: "agent-1".to_string(),
                location_id: "loc-2".to_string(),
                amount: 2,
                available: 8,
            },
        },
    });

    observation.time = 51;
    let second = behavior.decide(&observation);
    assert!(matches!(
        second,
        AgentDecision::Act(Action::MoveAgent { .. })
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn llm_agent_clamps_harvest_max_amount_to_configured_cap() {
    let client = MockClient {
        output: Some(r#"{"decision":"harvest_radiation","max_amount":1000000}"#.to_string()),
        err: None,
    };
    let mut config = base_config();
    config.harvest_max_amount_cap = 42;
    let mut behavior = LlmAgentBehavior::new("agent-1", config, client);

    let decision = behavior.decide(&make_observation());
    assert_eq!(
        decision,
        AgentDecision::Act(Action::HarvestRadiation {
            agent_id: "agent-1".to_string(),
            max_amount: 42,
        })
    );

    let trace = behavior.take_decision_trace().expect("trace");
    assert!(trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("max_amount clamped")));
}

#[test]
fn llm_agent_prechecks_schedule_recipe_location_and_reroutes_to_move_agent() {
    let client = MockClient {
        output: Some(
            r#"{"decision":"schedule_recipe","owner":"self","factory_id":"factory.alpha","recipe_id":"recipe.assembler.control_chip","batches":1}"#.to_string(),
        ),
        err: None,
    };
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);

    behavior.on_action_result(&ActionResult {
        action: Action::ScheduleRecipe {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            factory_id: "factory.alpha".to_string(),
            recipe_id: "recipe.assembler.control_chip".to_string(),
            batches: 1,
        },
        action_id: 410,
        success: false,
        event: WorldEvent {
            id: 510,
            time: 90,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::AgentNotAtLocation {
                    agent_id: "agent-1".to_string(),
                    location_id: "loc-factory".to_string(),
                },
            },
        },
    });

    let mut observation = make_observation();
    observation.visible_locations = vec![
        ObservedLocation {
            location_id: "loc-home".to_string(),
            name: "home".to_string(),
            pos: GeoPos {
                x_cm: 0.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
            profile: Default::default(),
            distance_cm: 0,
        },
        ObservedLocation {
            location_id: "loc-factory".to_string(),
            name: "factory".to_string(),
            pos: GeoPos {
                x_cm: 900_000.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
            profile: Default::default(),
            distance_cm: 900_000,
        },
    ];

    let decision = behavior.decide(&observation);
    assert_eq!(
        decision,
        AgentDecision::Act(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-factory".to_string(),
        })
    );

    let trace = behavior.take_decision_trace().expect("trace");
    assert!(trace.llm_step_trace.iter().any(|step| step
        .output_summary
        .contains("factory location precheck rerouted")));
}

#[test]
fn llm_agent_normalizes_schedule_factory_id_from_kind_alias() {
    let client = MockClient {
        output: Some(
            r#"{"decision":"schedule_recipe","owner":"self","factory_id":"factory.assembler.mk1","recipe_id":"recipe.assembler.control_chip","batches":1}"#
                .to_string(),
        ),
        err: None,
    };
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);
    behavior.on_action_result(&ActionResult {
        action: Action::BuildFactory {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            location_id: "loc-home".to_string(),
            factory_id: "factory.alpha".to_string(),
            factory_kind: "factory.assembler.mk1".to_string(),
        },
        action_id: 531,
        success: true,
        event: WorldEvent {
            id: 631,
            time: 140,
            kind: WorldEventKind::FactoryBuilt {
                owner: ResourceOwner::Agent {
                    agent_id: "agent-1".to_string(),
                },
                location_id: "loc-home".to_string(),
                factory_id: "factory.alpha".to_string(),
                factory_kind: "factory.assembler.mk1".to_string(),
                electricity_cost: 10,
                hardware_cost: 5,
            },
        },
    });

    let mut observation = make_observation();
    observation.visible_locations = vec![ObservedLocation {
        location_id: "loc-home".to_string(),
        name: "home".to_string(),
        pos: GeoPos {
            x_cm: 0.0,
            y_cm: 0.0,
            z_cm: 0.0,
        },
        profile: Default::default(),
        distance_cm: 0,
    }];
    observation
        .self_resources
        .add(ResourceKind::Hardware, 10)
        .expect("add test hardware");

    let decision = behavior.decide(&observation);
    assert_eq!(
        decision,
        AgentDecision::Act(Action::ScheduleRecipe {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            factory_id: "factory.alpha".to_string(),
            recipe_id: "recipe.assembler.control_chip".to_string(),
            batches: 1,
        })
    );

    let trace = behavior.take_decision_trace().expect("trace");
    assert!(trace.llm_step_trace.iter().any(|step| step
        .output_summary
        .contains("factory_id normalized by guardrail")));
}

#[test]
fn llm_agent_reroutes_duplicate_build_factory_to_schedule_on_known_factory() {
    let client = MockClient {
        output: Some(
            r#"{"decision":"build_factory","owner":"self","location_id":"loc-home","factory_id":"factory.assembler.mk1","factory_kind":"factory.assembler.mk1"}"#
                .to_string(),
        ),
        err: None,
    };
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);
    behavior.on_action_result(&ActionResult {
        action: Action::BuildFactory {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            location_id: "loc-home".to_string(),
            factory_id: "factory.alpha".to_string(),
            factory_kind: "factory.assembler.mk1".to_string(),
        },
        action_id: 532,
        success: true,
        event: WorldEvent {
            id: 632,
            time: 141,
            kind: WorldEventKind::FactoryBuilt {
                owner: ResourceOwner::Agent {
                    agent_id: "agent-1".to_string(),
                },
                location_id: "loc-home".to_string(),
                factory_id: "factory.alpha".to_string(),
                factory_kind: "factory.assembler.mk1".to_string(),
                electricity_cost: 10,
                hardware_cost: 5,
            },
        },
    });

    let mut observation = make_observation();
    observation.visible_locations = vec![ObservedLocation {
        location_id: "loc-home".to_string(),
        name: "home".to_string(),
        pos: GeoPos {
            x_cm: 0.0,
            y_cm: 0.0,
            z_cm: 0.0,
        },
        profile: Default::default(),
        distance_cm: 0,
    }];
    observation
        .self_resources
        .add(ResourceKind::Hardware, 16)
        .expect("add test hardware");

    let decision = behavior.decide(&observation);
    assert_eq!(
        decision,
        AgentDecision::Act(Action::ScheduleRecipe {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            factory_id: "factory.alpha".to_string(),
            recipe_id: "recipe.assembler.control_chip".to_string(),
            batches: 1,
        })
    );

    let trace = behavior.take_decision_trace().expect("trace");
    assert!(trace.llm_step_trace.iter().any(|step| step
        .output_summary
        .contains("build_factory dedup guardrail rerouted to schedule_recipe")));
}

#[test]
fn llm_agent_segments_move_agent_when_target_distance_exceeds_limit() {
    let client = MockClient {
        output: Some(r#"{"decision":"move_agent","to":"loc-factory"}"#.to_string()),
        err: None,
    };
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);

    let mut observation = make_observation();
    observation.visible_locations = vec![
        ObservedLocation {
            location_id: "loc-home".to_string(),
            name: "home".to_string(),
            pos: GeoPos {
                x_cm: 0.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
            profile: Default::default(),
            distance_cm: 0,
        },
        ObservedLocation {
            location_id: "loc-relay".to_string(),
            name: "relay".to_string(),
            pos: GeoPos {
                x_cm: 900_000.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
            profile: Default::default(),
            distance_cm: 900_000,
        },
        ObservedLocation {
            location_id: "loc-factory".to_string(),
            name: "factory".to_string(),
            pos: GeoPos {
                x_cm: 2_500_000.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
            profile: Default::default(),
            distance_cm: 2_500_000,
        },
    ];

    let decision = behavior.decide(&observation);
    assert_eq!(
        decision,
        AgentDecision::Act(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-relay".to_string(),
        })
    );

    let trace = behavior.take_decision_trace().expect("trace");
    assert!(trace.llm_step_trace.iter().any(|step| step
        .output_summary
        .contains("segmented by distance guardrail")));
}

#[test]
fn llm_agent_uses_relay_fallback_after_move_distance_exceeded_history() {
    let client = SequenceMockClient::new(vec![
        r#"{"decision":"move_agent","to":"loc-far"}"#.to_string(),
        r#"{"decision":"move_agent","to":"loc-far"}"#.to_string(),
    ]);
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);

    let mut observation = make_observation();
    observation.visible_locations = vec![
        ObservedLocation {
            location_id: "loc-home".to_string(),
            name: "home".to_string(),
            pos: GeoPos {
                x_cm: 0.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
            profile: Default::default(),
            distance_cm: 0,
        },
        ObservedLocation {
            location_id: "loc-relay".to_string(),
            name: "relay".to_string(),
            pos: GeoPos {
                x_cm: 900_000.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
            profile: Default::default(),
            distance_cm: 900_000,
        },
    ];

    let first = behavior.decide(&observation);
    assert_eq!(
        first,
        AgentDecision::Act(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-far".to_string(),
        })
    );
    let _ = behavior.take_decision_trace();

    behavior.on_action_result(&ActionResult {
        action: Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-far".to_string(),
        },
        action_id: 611,
        success: false,
        event: WorldEvent {
            id: 711,
            time: 150,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::MoveDistanceExceeded {
                    distance_cm: 1_800_000,
                    max_distance_cm: 1_000_000,
                },
            },
        },
    });

    observation.time = 151;
    let second = behavior.decide(&observation);
    assert_eq!(
        second,
        AgentDecision::Act(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-relay".to_string(),
        })
    );

    let trace = behavior.take_decision_trace().expect("trace");
    assert!(trace.llm_step_trace.iter().any(|step| step
        .output_summary
        .contains("fallback relay after move_distance_exceeded")));
}

#[test]
fn llm_agent_hard_switches_schedule_recipe_to_next_uncovered_recipe() {
    let client = MockClient {
        output: Some(
            r#"{"decision":"schedule_recipe","owner":"self","factory_id":"factory.alpha","recipe_id":"recipe.assembler.control_chip","batches":1}"#.to_string(),
        ),
        err: None,
    };
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);

    behavior.on_action_result(&ActionResult {
        action: Action::ScheduleRecipe {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            factory_id: "factory.alpha".to_string(),
            recipe_id: "recipe.assembler.control_chip".to_string(),
            batches: 1,
        },
        action_id: 520,
        success: true,
        event: WorldEvent {
            id: 620,
            time: 120,
            kind: WorldEventKind::RecipeScheduled {
                owner: ResourceOwner::Agent {
                    agent_id: "agent-1".to_string(),
                },
                factory_id: "factory.alpha".to_string(),
                recipe_id: "recipe.assembler.control_chip".to_string(),
                batches: 1,
                electricity_cost: 6,
                hardware_cost: 2,
                data_output: 1,
                finished_product_id: "product.component.control_chip".to_string(),
                finished_product_units: 1,
            },
        },
    });

    let mut observation = make_observation();
    observation
        .self_resources
        .add(ResourceKind::Hardware, 24)
        .expect("add test hardware");

    let decision = behavior.decide(&observation);
    assert_eq!(
        decision,
        AgentDecision::Act(Action::ScheduleRecipe {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            factory_id: "factory.alpha".to_string(),
            recipe_id: "recipe.assembler.motor_mk1".to_string(),
            batches: 1,
        })
    );

    let trace = behavior.take_decision_trace().expect("trace");
    assert!(trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("coverage hard-switch applied")));
}

#[test]
fn llm_agent_user_prompt_includes_recipe_coverage_summary() {
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), MockClient::default());
    behavior.on_action_result(&ActionResult {
        action: Action::ScheduleRecipe {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            factory_id: "factory.alpha".to_string(),
            recipe_id: "recipe.assembler.control_chip".to_string(),
            batches: 1,
        },
        action_id: 521,
        success: true,
        event: WorldEvent {
            id: 621,
            time: 121,
            kind: WorldEventKind::RecipeScheduled {
                owner: ResourceOwner::Agent {
                    agent_id: "agent-1".to_string(),
                },
                factory_id: "factory.alpha".to_string(),
                recipe_id: "recipe.assembler.control_chip".to_string(),
                batches: 1,
                electricity_cost: 6,
                hardware_cost: 2,
                data_output: 1,
                finished_product_id: "product.component.control_chip".to_string(),
                finished_product_units: 1,
            },
        },
    });

    let prompt = behavior.user_prompt(&make_observation(), &[], 0, 4);
    assert!(prompt.contains("\"recipe_coverage\""));
    assert!(prompt.contains("\"recipe.assembler.control_chip\""));
    assert!(prompt.contains("\"recipe.assembler.motor_mk1\""));
}

#[test]
fn llm_agent_reroutes_schedule_recipe_when_hardware_cannot_cover_one_batch() {
    let client = MockClient {
        output: Some(
            r#"{"decision":"schedule_recipe","owner":"self","factory_id":"factory.alpha","recipe_id":"recipe.assembler.logistics_drone","batches":5}"#.to_string(),
        ),
        err: None,
    };
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);
    let mut observation = make_observation();
    observation
        .self_resources
        .add(ResourceKind::Hardware, 7)
        .expect("add test hardware");
    observation
        .self_resources
        .add(ResourceKind::Compound, 1_000)
        .expect("add test compound");

    let decision = behavior.decide(&observation);
    assert_eq!(
        decision,
        AgentDecision::Act(Action::RefineCompound {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            compound_mass_g: 1_000,
        })
    );

    let trace = behavior.take_decision_trace().expect("trace");
    assert!(trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("rerouted to refine_compound")));
    assert!(trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("decision_rewrite={")));

    behavior.on_action_result(&ActionResult {
        action: Action::RefineCompound {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            compound_mass_g: 1_000,
        },
        action_id: 300,
        success: true,
        event: WorldEvent {
            id: 400,
            time: 99,
            kind: WorldEventKind::CompoundRefined {
                owner: ResourceOwner::Agent {
                    agent_id: "agent-1".to_string(),
                },
                compound_mass_g: 1_000,
                electricity_cost: 2,
                hardware_output: 1,
            },
        },
    });
    observation.time = 100;
    let prompt = behavior.user_prompt(&observation, &[], 0, 4);
    assert!(prompt.contains("\"decision_rewrite\":"));
    assert!(prompt.contains("\"from\":\"schedule_recipe\""));
    assert!(prompt.contains("\"to\":\"refine_compound\""));
}

#[test]
fn llm_agent_reroutes_schedule_recipe_to_mine_when_compound_missing_and_caps_mass() {
    let client = MockClient {
        output: Some(
            r#"{"decision":"schedule_recipe","owner":"self","factory_id":"factory.alpha","recipe_id":"recipe.assembler.logistics_drone","batches":1}"#.to_string(),
        ),
        err: None,
    };
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);
    let mut observation = make_observation();
    observation.visible_locations = vec![ObservedLocation {
        location_id: "loc-home".to_string(),
        name: "home".to_string(),
        pos: GeoPos {
            x_cm: 0.0,
            y_cm: 0.0,
            z_cm: 0.0,
        },
        profile: Default::default(),
        distance_cm: 0,
    }];

    let decision = behavior.decide(&observation);
    assert_eq!(
        decision,
        AgentDecision::Act(Action::MineCompound {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            location_id: "loc-home".to_string(),
            compound_mass_g: 5_000,
        })
    );

    let trace = behavior.take_decision_trace().expect("trace");
    assert!(trace.llm_step_trace.iter().any(|step| step
        .output_summary
        .contains("rerouted to mine_compound before refine")));
}

#[test]
fn llm_agent_clamps_schedule_recipe_batches_by_available_hardware() {
    let client = MockClient {
        output: Some(
            r#"{"decision":"schedule_recipe","owner":"self","factory_id":"factory.alpha","recipe_id":"recipe.assembler.logistics_drone","batches":5}"#.to_string(),
        ),
        err: None,
    };
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);
    let mut observation = make_observation();
    observation
        .self_resources
        .add(ResourceKind::Hardware, 24)
        .expect("add test hardware");

    let decision = behavior.decide(&observation);
    assert_eq!(
        decision,
        AgentDecision::Act(Action::ScheduleRecipe {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            factory_id: "factory.alpha".to_string(),
            recipe_id: "recipe.assembler.logistics_drone".to_string(),
            batches: 3,
        })
    );

    let trace = behavior.take_decision_trace().expect("trace");
    assert!(trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("batches clamped")));
}

#[test]
fn llm_agent_rewrites_execute_until_wait_action_to_actionable_harvest() {
    let client = MockClient {
        output: Some(
            r#"{"decision":"execute_until","action":{"decision":"wait"},"until":{"event":"new_visible_agent"},"max_ticks":4}"#
                .to_string(),
        ),
        err: None,
    };
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);
    let mut observation = make_observation();

    let decision = behavior.decide(&observation);
    assert_eq!(
        decision,
        AgentDecision::Act(Action::HarvestRadiation {
            agent_id: "agent-1".to_string(),
            max_amount: 1,
        })
    );

    let trace = behavior.take_decision_trace().expect("trace");
    assert!(trace.parse_error.is_none());
    assert!(trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("decision_rewrite={")));

    behavior.on_action_result(&ActionResult {
        action: Action::HarvestRadiation {
            agent_id: "agent-1".to_string(),
            max_amount: 1,
        },
        action_id: 301,
        success: true,
        event: WorldEvent {
            id: 401,
            time: 101,
            kind: WorldEventKind::RadiationHarvested {
                agent_id: "agent-1".to_string(),
                location_id: "loc-1".to_string(),
                amount: 1,
                available: 12,
            },
        },
    });
    observation.time = 102;
    let prompt = behavior.user_prompt(&observation, &[], 0, 4);
    assert!(prompt.contains("\"decision_rewrite\":"));
    assert!(prompt.contains("\"from\":\"wait\""));
    assert!(prompt.contains("\"to\":\"harvest_radiation\""));
}

#[test]
fn llm_agent_clamps_execute_until_harvest_action_to_configured_cap() {
    let client = MockClient {
        output: Some(
            r#"{"decision":"execute_until","action":{"decision":"harvest_radiation","max_amount":1000},"until":{"event":"new_visible_agent"},"max_ticks":4}"#.to_string(),
        ),
        err: None,
    };
    let mut config = base_config();
    config.harvest_max_amount_cap = 25;
    let mut behavior = LlmAgentBehavior::new("agent-1", config, client);

    let decision = behavior.decide(&make_observation());
    assert_eq!(
        decision,
        AgentDecision::Act(Action::HarvestRadiation {
            agent_id: "agent-1".to_string(),
            max_amount: 25,
        })
    );

    let trace = behavior.take_decision_trace().expect("trace");
    assert!(trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("max_amount clamped")));
}

#[test]
fn llm_agent_clamps_execute_until_harvest_max_ticks_to_short_cap() {
    let calls = Arc::new(AtomicUsize::new(0));
    let client = CountingSequenceMockClient::new(
        vec![
            r#"{"decision":"execute_until","action":{"decision":"harvest_radiation","max_amount":8},"until":{"event":"new_visible_agent"},"max_ticks":8}"#.to_string(),
            r#"{"decision":"move_agent","to":"loc-2"}"#.to_string(),
        ],
        Arc::clone(&calls),
    );
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);

    let mut observation = make_observation();
    observation.time = 70;
    let first = behavior.decide(&observation);
    assert!(matches!(
        first,
        AgentDecision::Act(Action::HarvestRadiation { max_amount: 8, .. })
    ));
    let first_trace = behavior.take_decision_trace().expect("first trace");
    assert!(first_trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("max_ticks=3")));
    assert!(first_trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("max_ticks clamped")));

    for offset in 0..4_u64 {
        behavior.on_action_result(&ActionResult {
            action: Action::HarvestRadiation {
                agent_id: "agent-1".to_string(),
                max_amount: 8,
            },
            action_id: 200 + offset,
            success: true,
            event: WorldEvent {
                id: 300 + offset,
                time: 70 + offset,
                kind: WorldEventKind::RadiationHarvested {
                    agent_id: "agent-1".to_string(),
                    location_id: "loc-2".to_string(),
                    amount: 6,
                    available: 80,
                },
            },
        });

        observation.time = 71 + offset;
        let decision = behavior.decide(&observation);
        if offset < 3 {
            assert!(matches!(
                decision,
                AgentDecision::Act(Action::HarvestRadiation { .. })
            ));
        } else {
            assert!(matches!(
                decision,
                AgentDecision::Act(Action::MoveAgent { .. })
            ));
        }
        let _ = behavior.take_decision_trace();
    }

    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn llm_agent_execute_until_stops_on_harvest_available_threshold() {
    let calls = Arc::new(AtomicUsize::new(0));
    let client = CountingSequenceMockClient::new(
        vec![
            r#"{"decision":"execute_until","action":{"decision":"harvest_radiation","max_amount":9},"until":{"event":"harvest_available_below","value_lte":1},"max_ticks":4}"#.to_string(),
            r#"{"decision":"move_agent","to":"loc-2"}"#.to_string(),
        ],
        Arc::clone(&calls),
    );
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);

    let mut observation = make_observation();
    observation.time = 60;
    let first = behavior.decide(&observation);
    assert!(matches!(
        first,
        AgentDecision::Act(Action::HarvestRadiation { max_amount: 9, .. })
    ));

    behavior.on_action_result(&ActionResult {
        action: Action::HarvestRadiation {
            agent_id: "agent-1".to_string(),
            max_amount: 9,
        },
        action_id: 104,
        success: true,
        event: WorldEvent {
            id: 204,
            time: 60,
            kind: WorldEventKind::RadiationHarvested {
                agent_id: "agent-1".to_string(),
                location_id: "loc-2".to_string(),
                amount: 5,
                available: 1,
            },
        },
    });

    observation.time = 61;
    let second = behavior.decide(&observation);
    assert!(matches!(
        second,
        AgentDecision::Act(Action::MoveAgent { .. })
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[test]
fn llm_agent_prompt_contains_execute_until_and_exploration_guidance() {
    let behavior = LlmAgentBehavior::new("agent-1", base_config(), MockClient::default());
    let system_prompt = behavior.system_prompt();
    let user_prompt = behavior.user_prompt(&make_observation(), &[], 0, 4);

    assert!(system_prompt.contains("anti_stagnation"));
    assert!(system_prompt.contains("exploration_bias"));
    assert!(system_prompt.contains("execute_until"));
    assert!(user_prompt.contains("execute_until"));
    assert!(user_prompt.contains("transfer_resource"));
    assert!(user_prompt.contains("refine_compound"));
    assert!(user_prompt.contains("build_factory"));
    assert!(user_prompt.contains("schedule_recipe"));
    assert!(user_prompt.contains("observation.recipe_coverage.missing"));
    assert!(user_prompt.contains("move_agent.to 不能是当前所在位置"));
}

#[test]
fn llm_parse_turn_responses_extracts_multiple_json_blocks() {
    let turns = completion_turns_from_output(
        r#"{"type":"module_call","module":"agent.modules.list","args":{}}

---

{"type":"decision_draft","decision":{"decision":"wait"},"need_verify":false}

---

{"decision":"wait"}"#,
    );
    let parsed = super::decision_flow::parse_llm_turn_payloads(turns.as_slice(), "agent-1");

    assert_eq!(parsed.len(), 3);
    assert!(matches!(
        parsed[0],
        super::decision_flow::ParsedLlmTurn::ModuleCall { .. }
    ));
    assert!(matches!(
        parsed[1],
        super::decision_flow::ParsedLlmTurn::DecisionDraft { .. }
    ));
    assert!(matches!(
        parsed[2],
        super::decision_flow::ParsedLlmTurn::Decision {
            decision: AgentDecision::Wait,
            ..
        }
    ));
}

#[test]
fn llm_parse_execute_until_accepts_event_any_of() {
    let turns = completion_turns_from_output(
        r#"{"decision":"execute_until","action":{"decision":"harvest_radiation","max_amount":3},"until":{"event_any_of":["new_visible_agent","new_visible_location"]},"max_ticks":5}"#,
    );
    let parsed = super::decision_flow::parse_llm_turn_payloads(turns.as_slice(), "agent-1")
        .into_iter()
        .next()
        .expect("single parsed turn");

    match parsed {
        super::decision_flow::ParsedLlmTurn::ExecuteUntil { directive, .. } => {
            assert_eq!(directive.until_conditions.len(), 2);
            assert_eq!(
                directive.until_conditions[0],
                super::decision_flow::ExecuteUntilCondition {
                    kind: super::decision_flow::ExecuteUntilEventKind::NewVisibleAgent,
                    value_lte: None,
                }
            );
            assert_eq!(
                directive.until_conditions[1],
                super::decision_flow::ExecuteUntilCondition {
                    kind: super::decision_flow::ExecuteUntilEventKind::NewVisibleLocation,
                    value_lte: None,
                }
            );
        }
        other => panic!("expected execute_until, got {other:?}"),
    }
}

#[test]
fn llm_parse_execute_until_rewrites_wait_action_to_minimal_harvest() {
    let turns = completion_turns_from_output(
        r#"{"decision":"execute_until","action":{"decision":"wait"},"until":{"event":"new_visible_agent"},"max_ticks":5}"#,
    );
    let parsed = super::decision_flow::parse_llm_turn_payloads(turns.as_slice(), "agent-1")
        .into_iter()
        .next()
        .expect("single parsed turn");

    match parsed {
        super::decision_flow::ParsedLlmTurn::ExecuteUntil {
            directive,
            rewrite_receipt,
            ..
        } => {
            assert!(matches!(
                directive.action,
                Action::HarvestRadiation { max_amount: 1, .. }
            ));
            let rewrite_receipt = rewrite_receipt.expect("rewrite receipt");
            assert_eq!(rewrite_receipt.from, "wait");
            assert_eq!(rewrite_receipt.to, "harvest_radiation");
            assert!(rewrite_receipt.reason.contains("non-actionable"));
        }
        other => panic!("expected execute_until, got {other:?}"),
    }
}

#[test]
fn llm_parse_execute_until_accepts_threshold_event_with_value_lte() {
    let turns = completion_turns_from_output(
        r#"{"decision":"execute_until","action":{"decision":"harvest_radiation","max_amount":3},"until":{"event":"harvest_yield_below","value_lte":2},"max_ticks":5}"#,
    );
    let parsed = super::decision_flow::parse_llm_turn_payloads(turns.as_slice(), "agent-1")
        .into_iter()
        .next()
        .expect("single parsed turn");

    match parsed {
        super::decision_flow::ParsedLlmTurn::ExecuteUntil { directive, .. } => {
            assert_eq!(directive.until_conditions.len(), 1);
            assert_eq!(
                directive.until_conditions[0],
                super::decision_flow::ExecuteUntilCondition {
                    kind: super::decision_flow::ExecuteUntilEventKind::HarvestYieldBelow,
                    value_lte: Some(2),
                }
            );
        }
        other => panic!("expected execute_until, got {other:?}"),
    }
}

#[test]
fn llm_parse_execute_until_rejects_threshold_event_without_value_lte() {
    let turns = completion_turns_from_output(
        r#"{"decision":"execute_until","action":{"decision":"harvest_radiation","max_amount":3},"until":{"event":"harvest_available_below"},"max_ticks":5}"#,
    );
    let parsed = super::decision_flow::parse_llm_turn_payloads(turns.as_slice(), "agent-1")
        .into_iter()
        .next()
        .expect("single parsed turn");

    match parsed {
        super::decision_flow::ParsedLlmTurn::Invalid(err) => {
            assert!(err.contains("requires until.value_lte"));
        }
        other => panic!("expected invalid execute_until, got {other:?}"),
    }
}

#[test]
fn llm_parse_decision_draft_accepts_shorthand_decision_payload() {
    let turns = completion_turns_from_output(
        r#"{"type":"decision_draft","decision":"harvest_radiation","max_amount":7,"need_verify":false}"#,
    );
    let parsed = super::decision_flow::parse_llm_turn_payloads(turns.as_slice(), "agent-1")
        .into_iter()
        .next()
        .expect("single parsed turn");

    match parsed {
        super::decision_flow::ParsedLlmTurn::DecisionDraft { draft, .. } => {
            assert!(matches!(
                draft.decision,
                AgentDecision::Act(Action::HarvestRadiation { max_amount: 7, .. })
            ));
            assert!(!draft.need_verify);
        }
        other => panic!("expected decision_draft, got {other:?}"),
    }
}

#[test]
fn llm_parse_turn_response_extracts_message_to_user() {
    let turns = completion_turns_from_output(
        r#"{"decision":"wait","message_to_user":"先暂停一回合观察环境。"}"#,
    );
    let parsed = super::decision_flow::parse_llm_turn_payloads(turns.as_slice(), "agent-1")
        .into_iter()
        .next()
        .expect("single parsed turn");

    match parsed {
        super::decision_flow::ParsedLlmTurn::Decision {
            decision,
            message_to_user,
            ..
        } => {
            assert_eq!(decision, AgentDecision::Wait);
            assert_eq!(message_to_user.as_deref(), Some("先暂停一回合观察环境。"));
        }
        other => panic!("expected decision, got {other:?}"),
    }
}

#[test]
fn llm_parse_turn_response_normalizes_module_alias_name() {
    let turns = completion_turns_from_output(
        r#"{"type":"module_call","module":"agent_modules_list","args":{}}"#,
    );
    let parsed = super::decision_flow::parse_llm_turn_payloads(turns.as_slice(), "agent-1")
        .into_iter()
        .next()
        .expect("single parsed turn");

    match parsed {
        super::decision_flow::ParsedLlmTurn::ModuleCall { request, .. } => {
            assert_eq!(request.module, "agent.modules.list");
        }
        other => panic!("expected module_call, got {other:?}"),
    }
}
