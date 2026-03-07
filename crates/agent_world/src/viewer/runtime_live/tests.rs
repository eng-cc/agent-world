use super::*;
use crate::simulator::ResourceOwner;
use ed25519_dalek::SigningKey;
use std::time::Duration;

fn test_signer(seed: u8) -> (String, String) {
    let private_key = [seed; 32];
    let signing_key = SigningKey::from_bytes(&private_key);
    (
        hex::encode(signing_key.verifying_key().to_bytes()),
        hex::encode(private_key),
    )
}

fn set_test_llm_env() {
    std::env::set_var(crate::simulator::ENV_LLM_MODEL, "gpt-4o-mini");
    std::env::set_var(
        crate::simulator::ENV_LLM_BASE_URL,
        "https://api.openai.com/v1",
    );
    std::env::set_var(crate::simulator::ENV_LLM_API_KEY, "test-api-key");
}

fn signed_prompt_control_apply_request(
    mut request: crate::viewer::PromptControlApplyRequest,
    intent: crate::viewer::PromptControlAuthIntent,
    nonce: u64,
    public_key_hex: &str,
    private_key_hex: &str,
) -> crate::viewer::PromptControlApplyRequest {
    request.public_key = Some(public_key_hex.to_string());
    let proof = crate::viewer::sign_prompt_control_apply_auth_proof(
        intent,
        &request,
        nonce,
        public_key_hex,
        private_key_hex,
    )
    .expect("sign prompt auth");
    request.auth = Some(proof);
    request
}

fn signed_agent_chat_request(
    mut request: crate::viewer::AgentChatRequest,
    nonce: u64,
    public_key_hex: &str,
    private_key_hex: &str,
) -> crate::viewer::AgentChatRequest {
    request.public_key = Some(public_key_hex.to_string());
    if request.intent_seq.is_none() {
        request.intent_seq = Some(nonce);
    }
    let proof =
        crate::viewer::sign_agent_chat_auth_proof(&request, nonce, public_key_hex, private_key_hex)
            .expect("sign agent chat auth");
    request.auth = Some(proof);
    request
}

#[test]
fn runtime_simulator_action_mapping_equivalence_covers_core_gameplay_and_economy() {
    let server =
        ViewerRuntimeLiveServer::new(ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal))
            .expect("runtime server");
    let assert_mapped = |action: crate::simulator::Action, expected: RuntimeAction| {
        let mapped = control_plane::simulator_action_to_runtime(&action, &server.world)
            .expect("action should map to runtime");
        assert_eq!(mapped, expected);
    };

    let move_target = GeoPos::new(10.0, 20.0, 30.0);
    assert_mapped(
        crate::simulator::Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: location_id_for_pos(move_target),
        },
        RuntimeAction::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: move_target,
        },
    );
    assert_mapped(
        crate::simulator::Action::TransferResource {
            from: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            to: ResourceOwner::Agent {
                agent_id: "agent-2".to_string(),
            },
            kind: ResourceKind::Electricity,
            amount: 3,
        },
        RuntimeAction::TransferResource {
            from_agent_id: "agent-1".to_string(),
            to_agent_id: "agent-2".to_string(),
            kind: ResourceKind::Electricity,
            amount: 3,
        },
    );
    assert_mapped(
        crate::simulator::Action::DeclareWar {
            initiator_agent_id: "agent-1".to_string(),
            war_id: "war.alpha".to_string(),
            aggressor_alliance_id: "alliance.a".to_string(),
            defender_alliance_id: "alliance.b".to_string(),
            objective: "expand".to_string(),
            intensity: 2,
        },
        RuntimeAction::DeclareWar {
            initiator_agent_id: "agent-1".to_string(),
            war_id: "war.alpha".to_string(),
            aggressor_alliance_id: "alliance.a".to_string(),
            defender_alliance_id: "alliance.b".to_string(),
            objective: "expand".to_string(),
            intensity: 2,
        },
    );
    assert_mapped(
        crate::simulator::Action::OpenEconomicContract {
            creator_agent_id: "agent-1".to_string(),
            contract_id: "contract.alpha".to_string(),
            counterparty_agent_id: "agent-2".to_string(),
            settlement_kind: ResourceKind::Data,
            settlement_amount: 5,
            reputation_stake: 7,
            expires_at: 99,
            description: "trade".to_string(),
        },
        RuntimeAction::OpenEconomicContract {
            creator_agent_id: "agent-1".to_string(),
            contract_id: "contract.alpha".to_string(),
            counterparty_agent_id: "agent-2".to_string(),
            settlement_kind: ResourceKind::Data,
            settlement_amount: 5,
            reputation_stake: 7,
            expires_at: 99,
            description: "trade".to_string(),
        },
    );
}

#[test]
fn runtime_live_server_config_play_interval_defaults_and_clamps() {
    let config = ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal);
    assert_eq!(config.play_step_interval, Duration::from_millis(800));

    let clamped = config.with_play_step_interval(Duration::from_millis(10));
    assert_eq!(clamped.play_step_interval, Duration::from_millis(50));
}

#[test]
fn runtime_live_session_play_step_is_interval_gated() {
    let mut session = RuntimeLiveSession::new();
    session.playing = true;

    assert!(session.should_advance_play_step(Duration::from_millis(40)));
    assert!(!session.should_advance_play_step(Duration::from_millis(40)));
    std::thread::sleep(Duration::from_millis(50));
    assert!(session.should_advance_play_step(Duration::from_millis(40)));
}

#[test]
fn runtime_simulator_action_mapping_covers_module_artifact_actions() {
    let server =
        ViewerRuntimeLiveServer::new(ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal))
            .expect("runtime server");
    let mut source_files = std::collections::BTreeMap::new();
    source_files.insert("module.toml".to_string(), b"manifest".to_vec());
    source_files.insert("src/lib.rs".to_string(), b"pub fn run() {}".to_vec());

    let compile = crate::simulator::Action::CompileModuleArtifactFromSource {
        publisher_agent_id: "agent-1".to_string(),
        module_id: "module.alpha".to_string(),
        manifest_path: "module.toml".to_string(),
        source_files: source_files.clone(),
    };
    let compile_mapped = control_plane::simulator_action_to_runtime(&compile, &server.world)
        .expect("compile action should map");
    assert_eq!(
        compile_mapped,
        RuntimeAction::CompileModuleArtifactFromSource {
            publisher_agent_id: "agent-1".to_string(),
            module_id: "module.alpha".to_string(),
            source_package: crate::runtime::ModuleSourcePackage {
                manifest_path: "module.toml".to_string(),
                files: source_files,
            },
        }
    );

    let deploy = crate::simulator::Action::DeployModuleArtifact {
        publisher_agent_id: "agent-1".to_string(),
        wasm_hash: "hash.alpha".to_string(),
        wasm_bytes: vec![0xAA, 0xBB],
        module_id_hint: Some("module.alpha".to_string()),
    };
    let deploy_mapped = control_plane::simulator_action_to_runtime(&deploy, &server.world)
        .expect("deploy action should map");
    assert_eq!(
        deploy_mapped,
        RuntimeAction::DeployModuleArtifact {
            publisher_agent_id: "agent-1".to_string(),
            wasm_hash: "hash.alpha".to_string(),
            wasm_bytes: vec![0xAA, 0xBB],
        }
    );

    let list = crate::simulator::Action::ListModuleArtifactForSale {
        seller_agent_id: "agent-1".to_string(),
        wasm_hash: "hash.alpha".to_string(),
        price_kind: ResourceKind::Data,
        price_amount: 9,
    };
    let list_mapped = control_plane::simulator_action_to_runtime(&list, &server.world)
        .expect("list action should map");
    assert_eq!(
        list_mapped,
        RuntimeAction::ListModuleArtifactForSale {
            seller_agent_id: "agent-1".to_string(),
            wasm_hash: "hash.alpha".to_string(),
            price_kind: ResourceKind::Data,
            price_amount: 9,
        }
    );

    let buy = crate::simulator::Action::BuyModuleArtifact {
        buyer_agent_id: "agent-2".to_string(),
        wasm_hash: "hash.alpha".to_string(),
    };
    let buy_mapped = control_plane::simulator_action_to_runtime(&buy, &server.world)
        .expect("buy action should map");
    assert_eq!(
        buy_mapped,
        RuntimeAction::BuyModuleArtifact {
            buyer_agent_id: "agent-2".to_string(),
            wasm_hash: "hash.alpha".to_string(),
        }
    );
}

#[test]
fn runtime_simulator_action_mapping_keeps_unmapped_actions_as_none() {
    let server =
        ViewerRuntimeLiveServer::new(ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal))
            .expect("runtime server");

    let build_factory = crate::simulator::Action::BuildFactory {
        owner: ResourceOwner::Agent {
            agent_id: "agent-1".to_string(),
        },
        location_id: "loc-1".to_string(),
        factory_id: "factory-1".to_string(),
        factory_kind: "smelter".to_string(),
    };
    assert!(control_plane::simulator_action_to_runtime(&build_factory, &server.world).is_none());

    let transfer_to_location = crate::simulator::Action::TransferResource {
        from: ResourceOwner::Agent {
            agent_id: "agent-1".to_string(),
        },
        to: ResourceOwner::Location {
            location_id: "loc-1".to_string(),
        },
        kind: ResourceKind::Electricity,
        amount: 1,
    };
    assert!(
        control_plane::simulator_action_to_runtime(&transfer_to_location, &server.world).is_none()
    );
}

#[test]
fn runtime_prompt_control_script_mode_requires_llm_mode() {
    let mut server = ViewerRuntimeLiveServer::new(
        ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal)
            .with_decision_mode(ViewerLiveDecisionMode::Script),
    )
    .expect("runtime server");
    let agent_id = server
        .world
        .state()
        .agents
        .keys()
        .next()
        .cloned()
        .expect("seed agent");
    let (public_key, private_key) = test_signer(11);
    let request = signed_prompt_control_apply_request(
        crate::viewer::PromptControlApplyRequest {
            agent_id: agent_id.clone(),
            player_id: "player-a".to_string(),
            public_key: None,
            auth: None,
            expected_version: Some(0),
            updated_by: None,
            system_prompt_override: Some(Some("system".to_string())),
            short_term_goal_override: None,
            long_term_goal_override: None,
        },
        crate::viewer::PromptControlAuthIntent::Apply,
        1,
        public_key.as_str(),
        private_key.as_str(),
    );
    let err = server
        .handle_prompt_control(crate::viewer::PromptControlCommand::Apply { request })
        .expect_err("script mode should reject prompt control");
    assert_eq!(err.code, "llm_mode_required");
    assert!(server.llm_sidecar.prompt_profiles.is_empty());
}

#[test]
fn runtime_prompt_control_apply_updates_snapshot_and_bindings() {
    let mut server = ViewerRuntimeLiveServer::new(
        ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal)
            .with_decision_mode(ViewerLiveDecisionMode::Llm),
    )
    .expect("runtime server");
    let agent_id = server
        .world
        .state()
        .agents
        .keys()
        .next()
        .cloned()
        .expect("seed agent");
    let (public_key, private_key) = test_signer(12);
    let request = signed_prompt_control_apply_request(
        crate::viewer::PromptControlApplyRequest {
            agent_id: agent_id.clone(),
            player_id: "player-a".to_string(),
            public_key: None,
            auth: None,
            expected_version: Some(0),
            updated_by: None,
            system_prompt_override: Some(Some("system".to_string())),
            short_term_goal_override: None,
            long_term_goal_override: None,
        },
        crate::viewer::PromptControlAuthIntent::Apply,
        2,
        public_key.as_str(),
        private_key.as_str(),
    );

    let ack = server
        .handle_prompt_control(crate::viewer::PromptControlCommand::Apply { request })
        .expect("llm mode apply");
    assert_eq!(ack.version, 1);
    let snapshot = server.compat_snapshot();
    let profile = snapshot
        .model
        .agent_prompt_profiles
        .get(agent_id.as_str())
        .expect("profile in snapshot");
    assert_eq!(profile.version, 1);
    assert_eq!(
        snapshot
            .model
            .agent_player_bindings
            .get(agent_id.as_str())
            .map(String::as_str),
        Some("player-a")
    );
    assert_eq!(
        snapshot
            .model
            .player_auth_last_nonce
            .get("player-a")
            .copied(),
        Some(2)
    );
}

#[test]
fn runtime_agent_chat_script_mode_requires_llm_mode() {
    let mut server = ViewerRuntimeLiveServer::new(
        ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal)
            .with_decision_mode(ViewerLiveDecisionMode::Script),
    )
    .expect("runtime server");
    let agent_id = server
        .world
        .state()
        .agents
        .keys()
        .next()
        .cloned()
        .expect("seed agent");
    let err = server
        .handle_agent_chat(crate::viewer::AgentChatRequest {
            agent_id,
            player_id: Some("player-a".to_string()),
            public_key: None,
            auth: None,
            message: "hello".to_string(),
            intent_tick: None,
            intent_seq: None,
        })
        .expect_err("script mode should reject chat");
    assert_eq!(err.code, "llm_mode_required");
}

#[test]
fn runtime_agent_chat_replay_returns_idempotent_ack() {
    set_test_llm_env();
    let mut server = ViewerRuntimeLiveServer::new(
        ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal)
            .with_decision_mode(ViewerLiveDecisionMode::Llm),
    )
    .expect("runtime server");
    let agent_id = server
        .world
        .state()
        .agents
        .keys()
        .next()
        .cloned()
        .expect("seed agent");
    let (public_key, private_key) = test_signer(21);
    let request = signed_agent_chat_request(
        crate::viewer::AgentChatRequest {
            agent_id: agent_id.clone(),
            player_id: Some("player-a".to_string()),
            public_key: None,
            auth: None,
            message: "hello".to_string(),
            intent_tick: Some(7),
            intent_seq: Some(5),
        },
        5,
        public_key.as_str(),
        private_key.as_str(),
    );

    let first = server
        .handle_agent_chat(request.clone())
        .expect("first request accepted");
    assert_eq!(first.intent_tick, Some(7));
    assert_eq!(first.intent_seq, Some(5));
    assert!(!first.idempotent_replay);

    let replay = server
        .handle_agent_chat(request)
        .expect("replay request accepted");
    assert_eq!(replay.agent_id, first.agent_id);
    assert_eq!(replay.accepted_at_tick, first.accepted_at_tick);
    assert_eq!(replay.message_len, first.message_len);
    assert_eq!(replay.player_id, first.player_id);
    assert_eq!(replay.intent_tick, first.intent_tick);
    assert_eq!(replay.intent_seq, first.intent_seq);
    assert!(replay.idempotent_replay);
    assert_eq!(
        server
            .llm_sidecar
            .player_auth_last_nonce
            .get("player-a")
            .copied(),
        Some(5)
    );
}

#[test]
fn runtime_agent_chat_rejects_intent_seq_conflict_on_payload_change() {
    set_test_llm_env();
    let mut server = ViewerRuntimeLiveServer::new(
        ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal)
            .with_decision_mode(ViewerLiveDecisionMode::Llm),
    )
    .expect("runtime server");
    let agent_id = server
        .world
        .state()
        .agents
        .keys()
        .next()
        .cloned()
        .expect("seed agent");
    let (public_key, private_key) = test_signer(22);
    let first = signed_agent_chat_request(
        crate::viewer::AgentChatRequest {
            agent_id: agent_id.clone(),
            player_id: Some("player-a".to_string()),
            public_key: None,
            auth: None,
            message: "hello".to_string(),
            intent_tick: Some(10),
            intent_seq: Some(6),
        },
        6,
        public_key.as_str(),
        private_key.as_str(),
    );
    server
        .handle_agent_chat(first)
        .expect("first request accepted");

    let conflict = signed_agent_chat_request(
        crate::viewer::AgentChatRequest {
            agent_id,
            player_id: Some("player-a".to_string()),
            public_key: None,
            auth: None,
            message: "changed".to_string(),
            intent_tick: Some(10),
            intent_seq: Some(6),
        },
        6,
        public_key.as_str(),
        private_key.as_str(),
    );
    let err = server
        .handle_agent_chat(conflict)
        .expect_err("same seq with different payload must fail");
    assert_eq!(err.code, "intent_seq_conflict");
}

#[test]
fn runtime_agent_chat_rejects_intent_seq_nonce_mismatch() {
    let mut server = ViewerRuntimeLiveServer::new(
        ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal)
            .with_decision_mode(ViewerLiveDecisionMode::Llm),
    )
    .expect("runtime server");
    let agent_id = server
        .world
        .state()
        .agents
        .keys()
        .next()
        .cloned()
        .expect("seed agent");
    let (public_key, private_key) = test_signer(23);
    let request = signed_agent_chat_request(
        crate::viewer::AgentChatRequest {
            agent_id,
            player_id: Some("player-a".to_string()),
            public_key: None,
            auth: None,
            message: "hello".to_string(),
            intent_tick: Some(3),
            intent_seq: Some(8),
        },
        9,
        public_key.as_str(),
        private_key.as_str(),
    );
    let err = server
        .handle_agent_chat(request)
        .expect_err("intent seq mismatch should fail");
    assert_eq!(err.code, "intent_seq_invalid");
}

fn commit_single_authoritative_batch(
    server: &mut ViewerRuntimeLiveServer,
) -> AuthoritativeBatchFinality {
    let journal_start = server.world.journal().events.len();
    server.script.enqueue(&mut server.world);
    server.world.step().expect("runtime step");

    let mut mapped_events = Vec::new();
    for runtime_event in &server.world.journal().events[journal_start..] {
        mapped_events.push(map_runtime_event(runtime_event, &server.snapshot_config));
    }
    mapped_events.extend(server.pending_virtual_events.drain(..));

    server
        .register_authoritative_batch(mapped_events.as_slice())
        .expect("register authoritative batch")
}

#[test]
fn runtime_authoritative_batch_commit_records_required_roots() {
    let mut server =
        ViewerRuntimeLiveServer::new(ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal))
            .expect("runtime server");

    let batch = commit_single_authoritative_batch(&mut server);
    assert_eq!(batch.finality_state, AuthoritativeFinalityState::Pending);
    assert!(!batch.batch_id.is_empty());
    assert!(is_valid_root_hash(batch.state_root.as_str()));
    assert!(is_valid_root_hash(batch.data_root.as_str()));
    assert_eq!(server.authoritative_batches.len(), 1);
}

#[test]
fn runtime_authoritative_batch_finality_is_monotonic_and_final_only_gates_settlement() {
    let mut server =
        ViewerRuntimeLiveServer::new(ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal))
            .expect("runtime server");

    let pending = commit_single_authoritative_batch(&mut server);
    assert!(!pending.settlement_ready);
    assert!(!pending.ranking_ready);

    let confirmed_updates = server
        .advance_authoritative_batch_finality(pending.confirm_height)
        .expect("advance to confirmed");
    assert_eq!(confirmed_updates.len(), 1);
    let confirmed = &confirmed_updates[0];
    assert_eq!(
        confirmed.finality_state,
        AuthoritativeFinalityState::Confirmed
    );
    assert!(!confirmed.settlement_ready);
    assert!(!confirmed.ranking_ready);

    let final_updates = server
        .advance_authoritative_batch_finality(pending.final_height)
        .expect("advance to final");
    assert_eq!(final_updates.len(), 1);
    let final_update = &final_updates[0];
    assert_eq!(
        final_update.finality_state,
        AuthoritativeFinalityState::Final
    );
    assert!(final_update.settlement_ready);
    assert!(final_update.ranking_ready);

    let no_regression = server
        .advance_authoritative_batch_finality(pending.confirm_height)
        .expect("finality should be monotonic");
    assert!(no_regression.is_empty());
    let stored = server.authoritative_batches.back().expect("stored batch");
    assert_eq!(stored.finality_state, AuthoritativeFinalityState::Final);
}

#[test]
fn runtime_authoritative_batch_data_root_mismatch_blocks_confirmation() {
    let mut server =
        ViewerRuntimeLiveServer::new(ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal))
            .expect("runtime server");

    let pending = commit_single_authoritative_batch(&mut server);
    let tampered_root = "f".repeat(64);
    let batch = server
        .authoritative_batches
        .back_mut()
        .expect("stored batch for tamper");
    batch.data_root = tampered_root;

    let updates = server
        .advance_authoritative_batch_finality(pending.final_height.saturating_add(10))
        .expect("advance finality");
    assert!(updates.is_empty());

    let stored = server.authoritative_batches.back().expect("stored batch");
    assert_eq!(stored.finality_state, AuthoritativeFinalityState::Pending);
    let wire = stored.as_wire(&server.settlement_ranking_gate);
    assert!(!wire.settlement_ready);
    assert!(!wire.ranking_ready);
}
