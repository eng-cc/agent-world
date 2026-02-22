use super::*;
use agent_world_node::{
    NodeConfig, NodeExecutionCommitContext, NodeExecutionCommitResult, NodeExecutionHook, NodeRole,
    NodeRuntime,
};
use ed25519_dalek::SigningKey;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn set_test_llm_env() {
    std::env::set_var(crate::simulator::ENV_LLM_MODEL, "gpt-4o-mini");
    std::env::set_var(
        crate::simulator::ENV_LLM_BASE_URL,
        "https://api.openai.com/v1",
    );
    std::env::set_var(crate::simulator::ENV_LLM_API_KEY, "test-api-key");
}

fn test_signer(seed: u8) -> (String, String) {
    let private_key = [seed; 32];
    let signing_key = SigningKey::from_bytes(&private_key);
    (
        hex::encode(signing_key.verifying_key().to_bytes()),
        hex::encode(private_key),
    )
}

fn signed_prompt_control_apply_request(
    mut request: PromptControlApplyRequest,
    intent: PromptControlAuthIntent,
    nonce: u64,
    public_key_hex: &str,
    private_key_hex: &str,
) -> PromptControlApplyRequest {
    request.public_key = Some(public_key_hex.to_string());
    let proof = crate::viewer::sign_prompt_control_apply_auth_proof(
        intent,
        &request,
        nonce,
        public_key_hex,
        private_key_hex,
    )
    .expect("sign prompt_control apply auth");
    request.auth = Some(proof);
    request
}

fn signed_agent_chat_request(
    mut request: AgentChatRequest,
    nonce: u64,
    public_key_hex: &str,
    private_key_hex: &str,
) -> AgentChatRequest {
    request.public_key = Some(public_key_hex.to_string());
    let proof =
        crate::viewer::sign_agent_chat_auth_proof(&request, nonce, public_key_hex, private_key_hex)
            .expect("sign agent_chat auth");
    request.auth = Some(proof);
    request
}

#[derive(Default)]
struct TestNoopExecutionHook;

impl NodeExecutionHook for TestNoopExecutionHook {
    fn on_commit(
        &mut self,
        context: NodeExecutionCommitContext,
    ) -> Result<NodeExecutionCommitResult, String> {
        Ok(NodeExecutionCommitResult {
            execution_height: context.height,
            execution_block_hash: format!("viewer-test-exec-block-{}", context.height),
            execution_state_root: format!("viewer-test-exec-state-{}", context.height),
        })
    }
}

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
    assert!(config.consensus_gate_max_tick.is_none());
    assert!(config.consensus_runtime.is_none());

    let llm_config = config.clone().with_llm_mode(true);
    assert_eq!(llm_config.decision_mode, ViewerLiveDecisionMode::Llm);

    let script_config = llm_config.with_decision_mode(ViewerLiveDecisionMode::Script);
    assert_eq!(script_config.decision_mode, ViewerLiveDecisionMode::Script);
}

#[test]
fn live_world_consensus_gate_limits_step_budget() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let gate = Arc::new(AtomicU64::new(0));
    let mut world = LiveWorld::new_with_consensus_gate(
        config,
        init,
        ViewerLiveDecisionMode::Script,
        Some(Arc::clone(&gate)),
        None,
    )
    .expect("init ok");

    assert!(!world.can_step_for_consensus());
    gate.store(1, Ordering::SeqCst);
    assert!(world.can_step_for_consensus());

    let _ = world.step().expect("step");
    assert_eq!(world.kernel.time(), 1);
    assert!(!world.can_step_for_consensus());
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
fn live_world_llm_bootstrap_script_mode_advances_tick() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::LlmBootstrap, &config);
    let mut world = LiveWorld::new(config, init, ViewerLiveDecisionMode::Script).expect("init ok");

    for _ in 0..24 {
        let _ = world.step().expect("step");
    }

    assert!(world.kernel.time() > 0);
}

#[test]
fn prompt_control_preview_reports_fields_and_next_version() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let mut world = LiveWorld::new(config, init, ViewerLiveDecisionMode::Script).expect("init ok");
    let (public_key, private_key) = test_signer(11);

    let ack = world
        .prompt_control_preview(signed_prompt_control_apply_request(
            PromptControlApplyRequest {
                agent_id: "agent-0".to_string(),
                player_id: "player-a".to_string(),
                public_key: None,
                auth: None,
                expected_version: Some(0),
                updated_by: None,
                system_prompt_override: Some(Some("系统提示".to_string())),
                short_term_goal_override: None,
                long_term_goal_override: None,
            },
            PromptControlAuthIntent::Preview,
            1,
            public_key.as_str(),
            private_key.as_str(),
        ))
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
    let (public_key, private_key) = test_signer(12);

    let err = world
        .prompt_control_apply(signed_prompt_control_apply_request(
            PromptControlApplyRequest {
                agent_id: "agent-0".to_string(),
                player_id: "player-a".to_string(),
                public_key: None,
                auth: None,
                expected_version: Some(0),
                updated_by: None,
                system_prompt_override: Some(Some("system".to_string())),
                short_term_goal_override: None,
                long_term_goal_override: None,
            },
            PromptControlAuthIntent::Apply,
            2,
            public_key.as_str(),
            private_key.as_str(),
        ))
        .expect_err("script mode should reject apply");

    assert_eq!(err.code, "llm_mode_required");
    assert!(world.kernel.model().agent_prompt_profiles.is_empty());
    assert!(world.kernel.model().agent_player_bindings.is_empty());
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

#[test]
fn prompt_control_preview_requires_non_empty_player_id() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let mut world = LiveWorld::new(config, init, ViewerLiveDecisionMode::Script).expect("init ok");

    let err = world
        .prompt_control_preview(PromptControlApplyRequest {
            agent_id: "agent-0".to_string(),
            player_id: "   ".to_string(),
            public_key: None,
            auth: None,
            expected_version: Some(0),
            updated_by: None,
            system_prompt_override: Some(Some("system".to_string())),
            short_term_goal_override: None,
            long_term_goal_override: None,
        })
        .expect_err("empty player id should be rejected");

    assert_eq!(err.code, "player_id_required");
}

#[test]
fn prompt_control_preview_requires_auth_proof() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let mut world = LiveWorld::new(config, init, ViewerLiveDecisionMode::Script).expect("init ok");
    let (public_key, _) = test_signer(21);

    let err = world
        .prompt_control_preview(PromptControlApplyRequest {
            agent_id: "agent-0".to_string(),
            player_id: "player-a".to_string(),
            public_key: Some(public_key),
            auth: None,
            expected_version: Some(0),
            updated_by: None,
            system_prompt_override: Some(Some("system".to_string())),
            short_term_goal_override: None,
            long_term_goal_override: None,
        })
        .expect_err("missing proof should be rejected");

    assert_eq!(err.code, "auth_proof_required");
}

#[test]
fn prompt_control_preview_rejects_tampered_auth_signature() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let mut world = LiveWorld::new(config, init, ViewerLiveDecisionMode::Script).expect("init ok");
    let (public_key, private_key) = test_signer(22);

    let request = signed_prompt_control_apply_request(
        PromptControlApplyRequest {
            agent_id: "agent-0".to_string(),
            player_id: "player-a".to_string(),
            public_key: None,
            auth: None,
            expected_version: Some(0),
            updated_by: None,
            system_prompt_override: Some(Some("system".to_string())),
            short_term_goal_override: None,
            long_term_goal_override: None,
        },
        PromptControlAuthIntent::Preview,
        7,
        public_key.as_str(),
        private_key.as_str(),
    );
    let mut tampered = request.clone();
    tampered.system_prompt_override = Some(Some("tampered".to_string()));

    let err = world
        .prompt_control_preview(tampered)
        .expect_err("tampered payload should be rejected");
    assert_eq!(err.code, "auth_signature_invalid");
}

#[test]
fn prompt_control_preview_rejects_replayed_nonce() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let mut world = LiveWorld::new(config, init, ViewerLiveDecisionMode::Script).expect("init ok");
    let (public_key, private_key) = test_signer(23);
    let request = signed_prompt_control_apply_request(
        PromptControlApplyRequest {
            agent_id: "agent-0".to_string(),
            player_id: "player-a".to_string(),
            public_key: None,
            auth: None,
            expected_version: Some(0),
            updated_by: None,
            system_prompt_override: Some(Some("system".to_string())),
            short_term_goal_override: None,
            long_term_goal_override: None,
        },
        PromptControlAuthIntent::Preview,
        8,
        public_key.as_str(),
        private_key.as_str(),
    );

    let first = world
        .prompt_control_preview(request.clone())
        .expect("first request accepted");
    assert!(first.preview);

    let replay = world
        .prompt_control_preview(request)
        .expect_err("replay request should be rejected");
    assert_eq!(replay.code, "auth_nonce_replay");
}

#[test]
fn prompt_control_preview_rejects_unbound_player_when_agent_already_bound() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let mut world = LiveWorld::new(config, init, ViewerLiveDecisionMode::Script).expect("init ok");
    let (public_key, private_key) = test_signer(13);

    let bind_event = world
        .kernel
        .bind_agent_player("agent-0", "player-a", None)
        .expect("bind ok");
    assert!(bind_event.is_some());

    let err = world
        .prompt_control_preview(signed_prompt_control_apply_request(
            PromptControlApplyRequest {
                agent_id: "agent-0".to_string(),
                player_id: "player-b".to_string(),
                public_key: None,
                auth: None,
                expected_version: Some(0),
                updated_by: None,
                system_prompt_override: Some(Some("system".to_string())),
                short_term_goal_override: None,
                long_term_goal_override: None,
            },
            PromptControlAuthIntent::Preview,
            3,
            public_key.as_str(),
            private_key.as_str(),
        ))
        .expect_err("mismatched player should be rejected");

    assert_eq!(err.code, "agent_control_forbidden");
    assert_eq!(
        world.kernel.player_binding_for_agent("agent-0"),
        Some("player-a")
    );
}

#[test]
fn prompt_control_preview_requires_matching_public_key_when_agent_is_key_bound() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let mut world = LiveWorld::new(config, init, ViewerLiveDecisionMode::Script).expect("init ok");
    let (bound_public_key, bound_private_key) = test_signer(14);
    let (wrong_public_key, wrong_private_key) = test_signer(15);

    let bind_event = world
        .kernel
        .bind_agent_player("agent-0", "player-a", Some(bound_public_key.as_str()))
        .expect("bind ok");
    assert!(bind_event.is_some());

    let missing_key = world
        .prompt_control_preview(PromptControlApplyRequest {
            agent_id: "agent-0".to_string(),
            player_id: "player-a".to_string(),
            public_key: None,
            auth: None,
            expected_version: Some(0),
            updated_by: None,
            system_prompt_override: Some(Some("system".to_string())),
            short_term_goal_override: None,
            long_term_goal_override: None,
        })
        .expect_err("missing public key should be rejected");
    assert_eq!(missing_key.code, "auth_proof_required");

    let wrong_key = world
        .prompt_control_preview(signed_prompt_control_apply_request(
            PromptControlApplyRequest {
                agent_id: "agent-0".to_string(),
                player_id: "player-a".to_string(),
                public_key: Some(wrong_public_key.clone()),
                auth: None,
                expected_version: Some(0),
                updated_by: None,
                system_prompt_override: Some(Some("system".to_string())),
                short_term_goal_override: None,
                long_term_goal_override: None,
            },
            PromptControlAuthIntent::Preview,
            4,
            wrong_public_key.as_str(),
            wrong_private_key.as_str(),
        ))
        .expect_err("mismatched public key should be rejected");
    assert_eq!(wrong_key.code, "agent_control_forbidden");

    let ack = world
        .prompt_control_preview(signed_prompt_control_apply_request(
            PromptControlApplyRequest {
                agent_id: "agent-0".to_string(),
                player_id: "player-a".to_string(),
                public_key: Some(bound_public_key.clone()),
                auth: None,
                expected_version: Some(0),
                updated_by: None,
                system_prompt_override: Some(Some("system".to_string())),
                short_term_goal_override: None,
                long_term_goal_override: None,
            },
            PromptControlAuthIntent::Preview,
            5,
            bound_public_key.as_str(),
            bound_private_key.as_str(),
        ))
        .expect("matching public key should pass");
    assert!(ack.preview);
}

#[test]
fn agent_chat_requires_player_id() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let mut world = LiveWorld::new(config, init, ViewerLiveDecisionMode::Script).expect("init ok");

    let err = world
        .agent_chat(AgentChatRequest {
            agent_id: "agent-0".to_string(),
            message: "hello".to_string(),
            player_id: None,
            public_key: None,
            auth: None,
        })
        .expect_err("missing player_id should be rejected");

    assert_eq!(err.code, "player_id_required");
}

#[test]
fn agent_chat_rejects_replayed_nonce() {
    set_test_llm_env();
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let mut world = LiveWorld::new(config, init, ViewerLiveDecisionMode::Llm).expect("init ok");
    let (public_key, private_key) = test_signer(24);
    let request = signed_agent_chat_request(
        AgentChatRequest {
            agent_id: "agent-0".to_string(),
            message: "hello".to_string(),
            player_id: Some("player-a".to_string()),
            public_key: None,
            auth: None,
        },
        9,
        public_key.as_str(),
        private_key.as_str(),
    );

    let first = world
        .agent_chat(request.clone())
        .expect("first request accepted");
    assert_eq!(first.player_id.as_deref(), Some("player-a"));

    let replay = world
        .agent_chat(request)
        .expect_err("replay request should be rejected");
    assert_eq!(replay.code, "auth_nonce_replay");
}

#[test]
fn agent_chat_upgrades_legacy_player_binding_with_public_key() {
    set_test_llm_env();
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let mut world = LiveWorld::new(config, init, ViewerLiveDecisionMode::Llm).expect("init ok");
    let (public_key, private_key) = test_signer(16);

    let bind_event = world
        .kernel
        .bind_agent_player("agent-0", "player-a", None)
        .expect("legacy bind ok");
    assert!(bind_event.is_some());
    assert_eq!(world.kernel.public_key_binding_for_agent("agent-0"), None);

    let ack = world
        .agent_chat(signed_agent_chat_request(
            AgentChatRequest {
                agent_id: "agent-0".to_string(),
                message: "hello".to_string(),
                player_id: Some("player-a".to_string()),
                public_key: Some(public_key.clone()),
                auth: None,
            },
            6,
            public_key.as_str(),
            private_key.as_str(),
        ))
        .expect("chat should be accepted");

    assert_eq!(ack.player_id.as_deref(), Some("player-a"));
    assert_eq!(
        world.kernel.public_key_binding_for_agent("agent-0"),
        Some(public_key.as_str())
    );
}

#[test]
fn restore_behavior_long_term_memory_from_model_applies_persisted_entries() {
    set_test_llm_env();
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let (mut kernel, _) = initialize_kernel(config, init).expect("init ok");

    let persisted = crate::simulator::LongTermMemoryEntry::new("mem-3", 7, "persisted insight")
        .with_tag("persisted");
    kernel
        .set_agent_long_term_memory("agent-0", vec![persisted.clone()])
        .expect("set persisted memory");

    let mut behavior = LlmAgentBehavior::from_env("agent-0").expect("build llm behavior");
    assert!(behavior.export_long_term_memory_entries().is_empty());

    restore_behavior_long_term_memory_from_model(&mut behavior, &kernel, "agent-0");
    let restored = behavior.export_long_term_memory_entries();
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].id, persisted.id);
    assert_eq!(restored[0].content, persisted.content);
}

#[test]
fn sync_llm_runner_long_term_memory_writes_back_to_world_model() {
    set_test_llm_env();
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let (mut kernel, _) = initialize_kernel(config, init).expect("init ok");

    let mut behavior = LlmAgentBehavior::from_env("agent-0").expect("build llm behavior");
    let runtime_entry = crate::simulator::LongTermMemoryEntry::new("mem-9", 15, "runtime memory")
        .with_tag("runtime");
    behavior.restore_long_term_memory_entries(&[runtime_entry.clone()]);

    let mut runner = AgentRunner::new();
    runner.register(behavior);

    sync_llm_runner_long_term_memory(&mut kernel, &runner);
    let restored = kernel
        .long_term_memory_for_agent("agent-0")
        .expect("agent memory exists");
    assert_eq!(restored.len(), 1);
    assert_eq!(restored[0].id, runtime_entry.id);
    assert_eq!(restored[0].content, runtime_entry.content);
}

#[test]
fn live_world_consensus_bridge_applies_only_committed_actions() {
    let node_config = NodeConfig::new("node-live-bridge", "live-minimal", NodeRole::Sequencer)
        .expect("node config")
        .with_tick_interval(Duration::from_millis(10))
        .expect("node tick interval");
    let mut node_runtime = NodeRuntime::new(node_config).with_execution_hook(TestNoopExecutionHook);
    node_runtime.start().expect("start node runtime");
    let shared_runtime = Arc::new(Mutex::new(node_runtime));

    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
    let mut world = LiveWorld::new_with_consensus_gate(
        config,
        init,
        ViewerLiveDecisionMode::Script,
        None,
        Some(Arc::clone(&shared_runtime)),
    )
    .expect("init world");

    assert_eq!(world.kernel.time(), 0);
    let first = world.step().expect("submit step");
    assert!(first.event.is_none());
    assert_eq!(world.kernel.time(), 0);

    let mut observed_commit_event = false;
    for _ in 0..40 {
        thread::sleep(Duration::from_millis(20));
        let step = world.step().expect("consensus replay step");
        if step.event.is_some() {
            observed_commit_event = true;
            break;
        }
    }

    assert!(observed_commit_event);
    assert!(world.kernel.time() > 0);

    let mut locked = shared_runtime.lock().expect("lock node runtime");
    locked.stop().expect("stop node runtime");
}
