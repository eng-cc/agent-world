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
        .add(ResourceKind::Data, 24)
        .expect("add test hardware");
    observation
        .self_resources
        .add(ResourceKind::Electricity, 100)
        .expect("add test electricity");
    observation
        .self_resources
        .add(ResourceKind::Electricity, 100)
        .expect("add test electricity");

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
    assert!(!trace.llm_step_trace.is_empty());
}

#[test]
fn llm_agent_rewrites_wait_ticks_to_sustained_schedule_after_full_recipe_coverage() {
    let client = MockClient {
        output: Some(r#"{"decision":"wait_ticks","ticks":3}"#.to_string()),
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
        action_id: 900,
        success: true,
        event: WorldEvent {
            id: 901,
            time: 180,
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

    let coverage_events = [
        (
            "recipe.assembler.control_chip",
            6_i64,
            2_i64,
            "product.component.control_chip",
        ),
        (
            "recipe.assembler.motor_mk1",
            12_i64,
            4_i64,
            "product.component.motor_mk1",
        ),
        (
            "recipe.assembler.logistics_drone",
            24_i64,
            8_i64,
            "product.component.logistics_drone",
        ),
    ];

    for (offset, (recipe_id, electricity_cost, hardware_cost, finished_product_id)) in
        coverage_events.into_iter().enumerate()
    {
        behavior.on_action_result(&ActionResult {
            action: Action::ScheduleRecipe {
                owner: ResourceOwner::Agent {
                    agent_id: "agent-1".to_string(),
                },
                factory_id: "factory.alpha".to_string(),
                recipe_id: recipe_id.to_string(),
                batches: 1,
            },
            action_id: 910 + offset as u64,
            success: true,
            event: WorldEvent {
                id: 920 + offset as u64,
                time: 181 + offset as u64,
                kind: WorldEventKind::RecipeScheduled {
                    owner: ResourceOwner::Agent {
                        agent_id: "agent-1".to_string(),
                    },
                    factory_id: "factory.alpha".to_string(),
                    recipe_id: recipe_id.to_string(),
                    batches: 1,
                    electricity_cost,
                    hardware_cost,
                    data_output: 1,
                    finished_product_id: finished_product_id.to_string(),
                    finished_product_units: 1,
                },
            },
        });
    }

    let mut observation = make_observation();
    observation.time = 190;
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
        .add(ResourceKind::Data, 16)
        .expect("add test hardware");
    observation
        .self_resources
        .add(ResourceKind::Electricity, 90)
        .expect("add test electricity");

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
        .contains("wait_ticks(3) rewritten to sustained production")));
    assert!(trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("decision_rewrite={")));
}

#[test]
fn llm_agent_rewrites_wait_to_recovery_action_after_full_recipe_coverage() {
    let client = MockClient {
        output: Some(r#"{"decision":"wait"}"#.to_string()),
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
        action_id: 930,
        success: true,
        event: WorldEvent {
            id: 931,
            time: 200,
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

    let covered_recipe_ids = [
        "recipe.assembler.control_chip",
        "recipe.assembler.motor_mk1",
        "recipe.assembler.logistics_drone",
    ];
    for (offset, recipe_id) in covered_recipe_ids.into_iter().enumerate() {
        behavior.on_action_result(&ActionResult {
            action: Action::ScheduleRecipe {
                owner: ResourceOwner::Agent {
                    agent_id: "agent-1".to_string(),
                },
                factory_id: "factory.alpha".to_string(),
                recipe_id: recipe_id.to_string(),
                batches: 1,
            },
            action_id: 940 + offset as u64,
            success: true,
            event: WorldEvent {
                id: 950 + offset as u64,
                time: 201 + offset as u64,
                kind: WorldEventKind::RecipeScheduled {
                    owner: ResourceOwner::Agent {
                        agent_id: "agent-1".to_string(),
                    },
                    factory_id: "factory.alpha".to_string(),
                    recipe_id: recipe_id.to_string(),
                    batches: 1,
                    electricity_cost: 6,
                    hardware_cost: 2,
                    data_output: 1,
                    finished_product_id: format!("product.{recipe_id}"),
                    finished_product_units: 1,
                },
            },
        });
    }

    let mut observation = make_observation();
    observation.time = 210;
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
        .add(ResourceKind::Data, 4)
        .expect("add hardware");
    observation
        .self_resources
        .remove(ResourceKind::Electricity, 28)
        .expect("drain electricity");

    let decision = behavior.decide(&observation);
    assert_eq!(
        decision,
        AgentDecision::Act(Action::HarvestRadiation {
            agent_id: "agent-1".to_string(),
            max_amount: DEFAULT_LLM_HARVEST_MAX_AMOUNT_CAP,
        })
    );

    let trace = behavior.take_decision_trace().expect("trace");
    assert!(trace.llm_step_trace.iter().any(|step| step
        .output_summary
        .contains("wait rewritten to sustained production")));
    assert!(trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("decision_rewrite={")));
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
    assert!(
        prompt.contains("\"recipe.assembler.control_chip\"") || prompt.contains("...(truncated)")
    );
    assert!(prompt.contains("\"recipe.assembler.motor_mk1\"") || prompt.contains("...(truncated)"));
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
    seed_known_factory(
        &mut behavior,
        "factory.alpha",
        "factory.assembler.mk1",
        "loc-home",
    );
    let mut observation = make_observation();
    observation
        .self_resources
        .add(ResourceKind::Data, 7)
        .expect("add test hardware");
    observation
        .self_resources
        .add(ResourceKind::Data, 1_000)
        .expect("add test compound");

    let decision = behavior.decide(&observation);
    assert_eq!(
        decision,
        AgentDecision::Act(Action::ScheduleRecipe {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            factory_id: "factory.alpha".to_string(),
            recipe_id: "recipe.assembler.logistics_drone".to_string(),
            batches: 1,
        })
    );

    let trace = behavior.take_decision_trace().expect("trace");
    assert!(trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("batches clamped")));
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
    seed_known_factory(
        &mut behavior,
        "factory.alpha",
        "factory.assembler.mk1",
        "loc-home",
    );
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
        AgentDecision::Act(Action::ScheduleRecipe {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            factory_id: "factory.alpha".to_string(),
            recipe_id: "recipe.assembler.logistics_drone".to_string(),
            batches: 1,
        })
    );
}

#[test]
fn llm_agent_clamps_mine_compound_mass_by_known_location_availability() {
    let client = MockClient {
        output: Some(
            r#"{"decision":"mine_compound","owner":"self","location_id":"loc-home","compound_mass_g":4000}"#
                .to_string(),
        ),
        err: None,
    };
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);
    behavior.on_action_result(&ActionResult {
        action: Action::MineCompound {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            location_id: "loc-home".to_string(),
            compound_mass_g: 4_000,
        },
        action_id: 612,
        success: false,
        event: WorldEvent {
            id: 712,
            time: 160,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::InsufficientResource {
                    owner: ResourceOwner::Location {
                        location_id: "loc-home".to_string(),
                    },
                    kind: ResourceKind::Data,
                    requested: 4_000,
                    available: 1_000,
                },
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
    observation.time = 161;

    let decision = behavior.decide(&observation);
    assert_eq!(
        decision,
        AgentDecision::Act(Action::MineCompound {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            location_id: "loc-home".to_string(),
            compound_mass_g: 1_000,
        })
    );

    let trace = behavior.take_decision_trace().expect("trace");
    assert!(trace.llm_step_trace.iter().any(|step| step
        .output_summary
        .contains("clamped by known_location_compound_available")));
}

#[test]
fn llm_agent_reroutes_mine_compound_from_depleted_location_to_alternative_location() {
    let client = MockClient {
        output: Some(
            r#"{"decision":"mine_compound","owner":"self","location_id":"loc-home","compound_mass_g":3000}"#
                .to_string(),
        ),
        err: None,
    };
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);
    behavior.on_action_result(&ActionResult {
        action: Action::MineCompound {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            location_id: "loc-home".to_string(),
            compound_mass_g: 3_000,
        },
        action_id: 613,
        success: false,
        event: WorldEvent {
            id: 713,
            time: 162,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::InsufficientResource {
                    owner: ResourceOwner::Location {
                        location_id: "loc-home".to_string(),
                    },
                    kind: ResourceKind::Data,
                    requested: 3_000,
                    available: 0,
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
            location_id: "loc-alt".to_string(),
            name: "alt".to_string(),
            pos: GeoPos {
                x_cm: 700_000.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
            profile: Default::default(),
            distance_cm: 700_000,
        },
    ];
    observation.time = 163;

    let decision = behavior.decide(&observation);
    assert_eq!(
        decision,
        AgentDecision::Act(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-alt".to_string(),
        })
    );

    let trace = behavior.take_decision_trace().expect("trace");
    assert!(trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("rerouted to move_agent")));
}

#[test]
fn llm_agent_skips_depleted_location_during_cooldown_window() {
    let client = MockClient {
        output: Some(
            r#"{"decision":"mine_compound","owner":"self","location_id":"loc-home","compound_mass_g":3000}"#
                .to_string(),
        ),
        err: None,
    };
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);

    behavior.on_action_result(&ActionResult {
        action: Action::MineCompound {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            location_id: "loc-home".to_string(),
            compound_mass_g: 3_000,
        },
        action_id: 1_000,
        success: false,
        event: WorldEvent {
            id: 1_001,
            time: 220,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::InsufficientResource {
                    owner: ResourceOwner::Location {
                        location_id: "loc-home".to_string(),
                    },
                    kind: ResourceKind::Data,
                    requested: 3_000,
                    available: 0,
                },
            },
        },
    });

    let mut observation = make_observation();
    observation.time = 221;
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
            location_id: "loc-alt".to_string(),
            name: "alt".to_string(),
            pos: GeoPos {
                x_cm: 700_000.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
            profile: Default::default(),
            distance_cm: 700_000,
        },
    ];

    let decision = behavior.decide(&observation);
    assert_eq!(
        decision,
        AgentDecision::Act(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-alt".to_string(),
        })
    );

    let trace = behavior.take_decision_trace().expect("trace");
    assert!(trace.llm_step_trace.iter().any(|step| step
        .output_summary
        .contains("cooldown guardrail rerouted to move_agent")));
}

#[test]
fn llm_agent_allows_retry_depleted_location_after_cooldown_expires() {
    let client = MockClient {
        output: Some(
            r#"{"decision":"mine_compound","owner":"self","location_id":"loc-home","compound_mass_g":3000}"#
                .to_string(),
        ),
        err: None,
    };
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);

    behavior.on_action_result(&ActionResult {
        action: Action::MineCompound {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            location_id: "loc-home".to_string(),
            compound_mass_g: 3_000,
        },
        action_id: 1_002,
        success: false,
        event: WorldEvent {
            id: 1_003,
            time: 230,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::InsufficientResource {
                    owner: ResourceOwner::Location {
                        location_id: "loc-home".to_string(),
                    },
                    kind: ResourceKind::Data,
                    requested: 3_000,
                    available: 0,
                },
            },
        },
    });

    let mut observation = make_observation();
    observation.time = 240;
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
            compound_mass_g: 3_000,
        })
    );

    let trace = behavior.take_decision_trace().expect("trace");
    assert!(!trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("cooldown guardrail rerouted")));
}
