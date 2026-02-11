use super::*;

#[test]
fn live_script_moves_between_locations() {
    let mut config = WorldConfig::default();
    config.physics.max_move_distance_cm_per_tick = i64::MAX;
    config.physics.max_move_speed_cm_per_s = i64::MAX;
    config.move_cost_per_km_electricity = 0;
    let init = WorldInitConfig::from_scenario(WorldScenario::TwinRegionBootstrap, &config);
    let (mut kernel, _) = initialize_kernel(config, init).expect("init ok");

    let mut script = LiveScript::new(&kernel);
    let initial_location = kernel
        .model()
        .agents
        .get("agent-0")
        .expect("agent exists")
        .location_id
        .clone();
    let mut moved = false;
    for _ in 0..2 {
        let action = script.next_action(&kernel).expect("action");
        kernel.submit_action(action);
        kernel.step_until_empty();

        let agent = kernel.model().agents.get("agent-0").expect("agent exists");
        if agent.location_id != initial_location {
            moved = true;
            break;
        }
    }

    assert!(moved);
}

#[test]
fn live_world_reset_rebuilds_kernel() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let mut world = LiveWorld::new(config, init, ViewerLiveDecisionMode::Script).expect("init ok");

    for _ in 0..5 {
        let _ = world.step().expect("step");
        if world.kernel.time() > 0 {
            break;
        }
    }
    assert!(world.kernel.time() > 0);

    world.reset().expect("reset ok");
    assert_eq!(world.kernel.time(), 0);
}

#[test]
fn live_server_config_supports_llm_mode() {
    let config = ViewerLiveServerConfig::new(WorldScenario::Minimal);
    assert_eq!(config.decision_mode, ViewerLiveDecisionMode::Script);

    let llm_config = config.clone().with_llm_mode(true);
    assert_eq!(llm_config.decision_mode, ViewerLiveDecisionMode::Llm);

    let script_config = llm_config.with_decision_mode(ViewerLiveDecisionMode::Script);
    assert_eq!(script_config.decision_mode, ViewerLiveDecisionMode::Script);
}

#[test]
fn live_world_seek_to_future_tick_advances_time() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let mut world = LiveWorld::new(config, init, ViewerLiveDecisionMode::Script).expect("init ok");

    let result = world.seek_to_tick(3).expect("seek ok");
    assert!(result.reached);
    assert_eq!(result.current_tick, 3);
    assert_eq!(world.kernel.time(), 3);
}

#[test]
fn live_world_seek_to_past_tick_resets_and_replays() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let mut world = LiveWorld::new(config, init, ViewerLiveDecisionMode::Script).expect("init ok");

    let forward = world.seek_to_tick(5).expect("seek forward");
    assert!(forward.reached);
    assert_eq!(world.kernel.time(), 5);

    let rewind = world.seek_to_tick(2).expect("seek rewind");
    assert!(rewind.reached);
    assert_eq!(rewind.current_tick, 2);
    assert_eq!(world.kernel.time(), 2);
}

#[test]
fn live_world_seek_reports_stall_when_world_cannot_advance() {
    let config = WorldConfig::default();
    let mut init = WorldInitConfig::default();
    init.agents = crate::simulator::AgentSpawnConfig {
        count: 0,
        ..crate::simulator::AgentSpawnConfig::default()
    };
    let mut world = LiveWorld::new(config, init, ViewerLiveDecisionMode::Script).expect("init ok");

    let result = world.seek_to_tick(2).expect("seek handled");
    assert!(!result.reached);
    assert_eq!(result.current_tick, 0);
    assert_eq!(world.kernel.time(), 0);
}

#[test]
fn prompt_control_preview_reports_fields_and_next_version() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let world = LiveWorld::new(config, init, ViewerLiveDecisionMode::Script).expect("init ok");

    let ack = world
        .prompt_control_preview(PromptControlApplyRequest {
            agent_id: "agent-0".to_string(),
            expected_version: Some(0),
            updated_by: None,
            system_prompt_override: Some(Some("系统提示".to_string())),
            short_term_goal_override: None,
            long_term_goal_override: None,
        })
        .expect("preview ack");

    assert!(ack.preview);
    assert_eq!(ack.version, 1);
    assert_eq!(ack.operation, PromptControlOperation::Apply);
    assert_eq!(
        ack.applied_fields,
        vec!["system_prompt_override".to_string()]
    );
    assert!(!ack.digest.is_empty());
}

#[test]
fn prompt_control_apply_requires_llm_mode() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let mut world = LiveWorld::new(config, init, ViewerLiveDecisionMode::Script).expect("init ok");

    let err = world
        .prompt_control_apply(PromptControlApplyRequest {
            agent_id: "agent-0".to_string(),
            expected_version: Some(0),
            updated_by: None,
            system_prompt_override: Some(Some("system".to_string())),
            short_term_goal_override: None,
            long_term_goal_override: None,
        })
        .expect_err("script mode should reject apply");

    assert_eq!(err.code, "llm_mode_required");
    assert!(world.kernel.model().agent_prompt_profiles.is_empty());
    assert!(!world.kernel.journal().iter().any(|event| matches!(
        event.kind,
        crate::simulator::WorldEventKind::AgentPromptUpdated { .. }
    )));
}

#[test]
fn prompt_profile_version_lookup_reads_from_journal() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let mut world = LiveWorld::new(config, init, ViewerLiveDecisionMode::Script).expect("init ok");

    let mut profile = AgentPromptProfile::for_agent("agent-0");
    profile.system_prompt_override = Some("v1".to_string());
    profile.version = 1;
    profile.updated_at_tick = world.kernel.time();
    profile.updated_by = "test".to_string();
    world.kernel.apply_agent_prompt_profile_update(
        profile.clone(),
        PromptUpdateOperation::Apply,
        vec!["system_prompt_override".to_string()],
        "digest-1".to_string(),
        None,
    );

    let loaded = world
        .lookup_prompt_profile_version("agent-0", 1)
        .expect("profile v1");
    assert_eq!(loaded.system_prompt_override.as_deref(), Some("v1"));
    assert_eq!(loaded.version, 1);
}
