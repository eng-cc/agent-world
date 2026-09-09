use super::*;
use crate::viewer::runtime_live::WorldScenario;
use crate::viewer::{ViewerRuntimeLiveServer, ViewerRuntimeLiveServerConfig};

#[test]
fn provider_terminal_marker_requires_complete_identity() {
    let agent_id = "agent-terminal-identity";
    let mut context = test_provider_context(
        agent_id,
        "turn-terminal-identity",
        "request-terminal-identity",
        1,
    );
    let identity_digest =
        crate::simulator::h_v1("oasis7.test.terminal-identity.v1", &"terminal-identity");
    context.request_context.request_digest = identity_digest.clone();
    context.turn_context.request_digest = identity_digest;
    let mut sidecar = RuntimeLlmSidecar::new(ViewerLiveDecisionMode::Llm);
    sidecar
        .provider_contexts
        .insert(agent_id.to_string(), context.clone());
    sidecar.record_provider_terminal_state(
        agent_id,
        &context,
        "failed",
        Some("failed_provider".to_string()),
        Some("feedback-terminal-identity".to_string()),
    );
    let decision = async_support::RuntimeLlmDecision {
        agent_id: agent_id.to_string(),
        decision: AgentDecision::Wait,
        decision_trace: None,
        cognition: None,
        memory_write_intents: Vec::new(),
        continuation_admitted: false,
    };
    assert!(sidecar.provider_decision_is_terminalized(&decision));

    let mut later_context = context;
    later_context.request_context.agent_session_id = "session-terminal-identity-later".to_string();
    later_context.turn_context.agent_session_id = "session-terminal-identity-later".to_string();
    later_context.request_context.request_digest =
        crate::simulator::h_v1("oasis7.test.terminal-identity-later.v1", &agent_id);
    later_context.turn_context.request_digest =
        later_context.request_context.request_digest.clone();
    sidecar
        .provider_contexts
        .insert(agent_id.to_string(), later_context);
    assert!(
        !sidecar.provider_decision_is_terminalized(&decision),
        "same turn/request values from a later session or digest must not match a terminal marker"
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn assert_terminal_marker_suppresses_queued_provider_decision(decision: AgentDecision) {
    let decision_kind = match &decision {
        AgentDecision::Act(_) => "action",
        AgentDecision::Wait => "wait",
        _ => "other",
    };
    let path = std::env::temp_dir().join(format!(
        "oasis7-viewer-provider-lineage-terminal-queue-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos(),
        decision_kind
    ));
    let agent_id = "agent-terminal-queued";
    let mut context = test_provider_context(
        agent_id,
        "turn-terminal-queued",
        "request-terminal-queued",
        1,
    );
    let terminal_digest =
        crate::simulator::h_v1("oasis7.test.terminal-queued.v1", &"terminal-queued");
    context.request_context.request_digest = terminal_digest.clone();
    context.turn_context.request_digest = terminal_digest;
    let queued = async_support::RuntimeLlmDecision {
        agent_id: agent_id.to_string(),
        decision,
        decision_trace: None,
        cognition: None,
        memory_write_intents: Vec::new(),
        continuation_admitted: false,
    };
    let mut first = RuntimeLlmSidecar::new(ViewerLiveDecisionMode::Llm);
    first.configure_provider_lineage_store(path.clone());
    first.provider_agent_ids.insert(agent_id.to_string());
    first
        .provider_contexts
        .insert(agent_id.to_string(), context.clone());
    first.provider_terminal_states.insert(
        agent_id.to_string(),
        lineage_persistence::ProviderTerminalState {
            agent_id: agent_id.to_string(),
            agent_session_id: "session-test".to_string(),
            agent_turn_id: "turn-terminal-queued".to_string(),
            decision_request_id: "request-terminal-queued".to_string(),
            request_digest: context.request_context.request_digest.to_string(),
            status: "committed".to_string(),
            reject_reason: None,
            feedback_id: Some("feedback-terminal-queued".to_string()),
        },
    );
    first.provider_completed_decisions.push_back(queued.clone());
    first
        .provider_held_decisions
        .insert(agent_id.to_string(), queued);
    first
        .persist_provider_lineage()
        .expect("persist terminal queued provider decision");

    let mut restored = RuntimeLlmSidecar::new(ViewerLiveDecisionMode::Llm);
    restored.configure_provider_lineage_store(path.clone());
    restored
        .restore_provider_lineage(&RuntimeWorld::default())
        .expect("restore terminal queued provider decision");
    assert!(
        restored.provider_completed_decisions.is_empty(),
        "terminal marker must dominate queued provider decisions after restore"
    );
    assert!(
        restored.provider_held_decisions.is_empty(),
        "terminal marker must clear held provider decisions after restore"
    );
    assert!(
        restored.provider_contexts.is_empty(),
        "terminal marker cleanup must release the exact old identity"
    );
    let _ = std::fs::remove_file(path);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn provider_lineage_restore_terminal_marker_suppresses_queued_action_replay() {
    assert_terminal_marker_suppresses_queued_provider_decision(AgentDecision::Act(
        crate::simulator::Action::MoveAgent {
            agent_id: "agent-terminal-queued".to_string(),
            to: "loc-terminal-queued".to_string(),
        },
    ));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn provider_lineage_restore_terminal_marker_suppresses_queued_wait_replay() {
    assert_terminal_marker_suppresses_queued_provider_decision(AgentDecision::Wait);
}

#[test]
fn provider_transport_exhaustion_closes_recovered_runtime_prefix_before_release() {
    let mut world = RuntimeWorld::new();
    world
        .bind_cognition_runtime("world", "recovery-branch", 0, None, "pending", 0)
        .expect("Runtime cognition binding");
    world.submit_action(RuntimeAction::RegisterAgent {
        agent_id: "agent-recovery".to_string(),
        pos: GeoPos::new(0, 0, 0),
    });
    world.step().expect("register Runtime agent");
    let mut context =
        test_provider_context("agent-recovery", "turn-recovery", "request-recovery", 1);
    let digest = crate::simulator::h_v1("oasis7.test.recovery-digest.v1", &"recovery");
    context.request_context.observation_digest = digest.clone();
    context.request_context.capability_catalog_digest = digest.clone();
    context.request_context.capability_invocation_context_digest = digest.clone();
    context.request_context.memory_snapshot_digest = digest.clone();
    context.request_context.goal_snapshot_digest = digest.clone();
    context.request_context.continuation_digest = digest.clone();
    context.request_context.runtime_binding.base_world_hash = digest.clone();
    context
        .request_context
        .runtime_binding
        .runtime_manifest_hash = digest.clone();
    context.request_context.request_digest = digest.clone();
    context.turn_context.request_digest = digest;
    super::async_support::runtime_provider_prefix(&mut world, &context)
        .expect("persist recovered provider Runtime prefix");

    let mut server = ViewerRuntimeLiveServer::new(
        ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal)
            .with_decision_mode(ViewerLiveDecisionMode::Llm),
    )
    .expect("Runtime live server");
    server.world = world;
    server
        .llm_sidecar
        .provider_agent_ids
        .insert("agent-recovery".to_string());
    server
        .llm_sidecar
        .provider_contexts
        .insert("agent-recovery".to_string(), context.clone());
    server
        .llm_sidecar
        .provider_active_turns
        .insert("agent-recovery".to_string(), context);
    server
        .llm_sidecar
        .provider_transport_exhausted
        .insert("agent-recovery".to_string());

    let _trace = server
        .enqueue_llm_action_from_sidecar()
        .expect_err("transport exhaustion must return a typed terminal trace");
    let events = server.world.cognition()["cognition_journal"]["events"]
        .as_array()
        .expect("Runtime cognition journal events");
    assert!(
        events.iter().any(|event| {
            event["event_kind"] == "CognitionTurnFailed"
                && event["agent_id"] == "agent-recovery"
                && event["agent_turn_id"] == "turn-recovery"
        }),
        "recovery must close the Runtime turn before sidecar release: {events:?}"
    );
    assert!(
        events.iter().any(|event| {
            event["event_kind"] == "CognitionTurnCompleted"
                && event["status"] == "failed"
                && event["agent_id"] == "agent-recovery"
                && event["agent_turn_id"] == "turn-recovery"
        }),
        "recovery must append a failed Runtime completion: {events:?}"
    );
    assert!(
        server.world.cognition_in_flight_wakes().unwrap().is_empty(),
        "a recovered provider failure must not leave a Runtime wake in flight"
    );
}

#[test]
fn restored_mismatched_active_identity_terminalizes_runtime_before_release() {
    let path = std::env::temp_dir().join(format!(
        "oasis7-provider-quarantine-terminal-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let agent_id = "agent-mismatch-recovery";
    let mut runtime_context = test_provider_context(
        agent_id,
        "turn-runtime-recovery",
        "request-runtime-recovery",
        1,
    );
    let digest = crate::simulator::h_v1(
        "oasis7.test.quarantine-terminal-digest.v1",
        &"quarantine-terminal",
    );
    runtime_context.request_context.observation_digest = digest.clone();
    runtime_context.request_context.capability_catalog_digest = digest.clone();
    runtime_context
        .request_context
        .capability_invocation_context_digest = digest.clone();
    runtime_context.request_context.memory_snapshot_digest = digest.clone();
    runtime_context.request_context.goal_snapshot_digest = digest.clone();
    runtime_context.request_context.continuation_digest = digest.clone();
    runtime_context
        .request_context
        .runtime_binding
        .base_world_hash = digest.clone();
    runtime_context
        .request_context
        .runtime_binding
        .runtime_manifest_hash = digest.clone();
    runtime_context.request_context.request_digest = digest.clone();
    runtime_context.turn_context.request_digest = digest;
    let mut active_marker = runtime_context.clone();
    active_marker.request_context.agent_turn_id = "turn-marker-recovery".to_string();
    active_marker.turn_context.agent_turn_id = "turn-marker-recovery".to_string();
    active_marker.request_context.agent_session_id = "session-marker-recovery".to_string();
    active_marker.turn_context.agent_session_id = "session-marker-recovery".to_string();
    let marker_digest = crate::simulator::h_v1(
        "oasis7.test.quarantine-terminal-marker-digest.v1",
        &"quarantine-terminal-marker",
    );
    active_marker.request_context.request_digest = marker_digest.clone();
    active_marker.turn_context.request_digest = marker_digest;

    let mut world = RuntimeWorld::new();
    world
        .bind_cognition_runtime("world", "quarantine-branch", 0, None, "pending", 0)
        .expect("Runtime cognition binding");
    world.submit_action(RuntimeAction::RegisterAgent {
        agent_id: agent_id.to_string(),
        pos: GeoPos::new(0, 0, 0),
    });
    world.step().expect("register Runtime agent");
    super::async_support::runtime_provider_prefix(&mut world, &active_marker)
        .expect("persist Runtime prefix for mismatched recovery identity");

    let mut first = RuntimeLlmSidecar::new(ViewerLiveDecisionMode::Llm);
    first.configure_provider_lineage_store(path.clone());
    first.provider_agent_ids.insert(agent_id.to_string());
    first
        .provider_active_turns
        .insert(agent_id.to_string(), active_marker.clone());
    first
        .provider_contexts
        .insert(agent_id.to_string(), runtime_context);
    first
        .persist_provider_lineage()
        .expect("persist mismatched active identity");

    let mut server = ViewerRuntimeLiveServer::new(
        ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal)
            .with_decision_mode(ViewerLiveDecisionMode::Llm)
            .with_provider_lineage_store(path.clone()),
    )
    .expect("Runtime live server");
    server.world = world;
    server
        .llm_sidecar
        .restore_provider_lineage(&server.world)
        .expect("restore mismatched active identity");
    assert!(
        server
            .llm_sidecar
            .provider_transport_exhausted
            .contains(agent_id),
        "restore must route the quarantine into terminalization"
    );
    let _trace = server
        .enqueue_llm_action_from_sidecar()
        .expect_err("quarantine must emit a terminal recovery trace");
    let events = server.world.cognition()["cognition_journal"]["events"]
        .as_array()
        .expect("Runtime cognition journal events");
    assert!(
        events.iter().any(|event| {
            event["event_kind"] == "CognitionTurnFailed"
                && event["agent_id"] == agent_id
                && event["agent_turn_id"] == "turn-marker-recovery"
        }),
        "restored mismatch must terminalize the correlated Runtime turn: {events:?}"
    );
    assert!(
        events.iter().any(|event| {
            event["event_kind"] == "CognitionTurnCompleted"
                && event["status"] == "failed"
                && event["agent_id"] == agent_id
                && event["agent_turn_id"] == "turn-marker-recovery"
        }),
        "restored mismatch must append failed Runtime completion: {events:?}"
    );
    let feedback = server
        .world
        .runtime_feedback_outbox()
        .expect("Runtime feedback outbox");
    assert!(
        feedback.iter().any(|record| {
            record.agent_subject == agent_id
                && record.agent_session_id == "session-marker-recovery"
                && record.agent_turn_id == "turn-marker-recovery"
                && record.decision_request_id == "request-runtime-recovery"
                && record.request_digest == active_marker.request_context.request_digest.to_string()
        }),
        "quarantine feedback must bind the Runtime-terminalized active marker identity: {feedback:?}"
    );
    assert!(
        feedback.iter().all(|record| {
            record.agent_session_id != "session-test"
                && record.agent_turn_id != "turn-runtime-recovery"
        }),
        "conflicting provider context must not supply feedback identity: {feedback:?}"
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn restored_missing_active_identity_terminalizes_runtime_and_feedback() {
    let path = std::env::temp_dir().join(format!(
        "oasis7-provider-missing-terminal-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let agent_id = "agent-missing-terminal";
    let mut active_marker = test_provider_context(
        agent_id,
        "turn-missing-terminal",
        "request-missing-terminal",
        1,
    );
    active_marker.request_context.agent_session_id = "session-missing-terminal".to_string();
    active_marker.turn_context.agent_session_id = "session-missing-terminal".to_string();
    let marker_digest = crate::simulator::h_v1(
        "oasis7.test.missing-terminal-digest.v1",
        &"missing-terminal",
    );
    active_marker.request_context.observation_digest = marker_digest.clone();
    active_marker.request_context.capability_catalog_digest = marker_digest.clone();
    active_marker
        .request_context
        .capability_invocation_context_digest = marker_digest.clone();
    active_marker.request_context.memory_snapshot_digest = marker_digest.clone();
    active_marker.request_context.goal_snapshot_digest = marker_digest.clone();
    active_marker.request_context.continuation_digest = marker_digest.clone();
    active_marker
        .request_context
        .runtime_binding
        .base_world_hash = marker_digest.clone();
    active_marker
        .request_context
        .runtime_binding
        .runtime_manifest_hash = marker_digest.clone();
    active_marker.request_context.request_digest = marker_digest.clone();
    active_marker.turn_context.request_digest = marker_digest;

    let mut world = RuntimeWorld::new();
    world
        .bind_cognition_runtime("world", "missing-terminal-branch", 0, None, "pending", 0)
        .expect("Runtime cognition binding");
    world.submit_action(RuntimeAction::RegisterAgent {
        agent_id: agent_id.to_string(),
        pos: GeoPos::new(0, 0, 0),
    });
    world.step().expect("register Runtime agent");
    super::async_support::runtime_provider_prefix(&mut world, &active_marker)
        .expect("persist Runtime prefix for missing identity");

    let mut first = RuntimeLlmSidecar::new(ViewerLiveDecisionMode::Llm);
    first.configure_provider_lineage_store(path.clone());
    first.provider_agent_ids.insert(agent_id.to_string());
    first
        .provider_active_turns
        .insert(agent_id.to_string(), active_marker.clone());
    // Deliberately omit provider_contexts: restore has only the Runtime active
    // marker and must carry it through terminalization and feedback.
    first
        .persist_provider_lineage()
        .expect("persist missing active identity");

    let mut server = ViewerRuntimeLiveServer::new(
        ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal)
            .with_decision_mode(ViewerLiveDecisionMode::Llm)
            .with_provider_lineage_store(path.clone()),
    )
    .expect("Runtime live server");
    server.world = world;
    server
        .llm_sidecar
        .restore_provider_lineage(&server.world)
        .expect("restore missing active identity");
    assert!(
        server
            .llm_sidecar
            .provider_recovery_context(agent_id)
            .is_some(),
        "missing active identity must remain recoverable before the control pass"
    );
    let _trace = server
        .enqueue_llm_action_from_sidecar()
        .expect_err("missing identity must emit a terminal recovery trace");
    let events = server.world.cognition()["cognition_journal"]["events"]
        .as_array()
        .expect("Runtime cognition journal events");
    assert!(
        events.iter().any(|event| {
            event["event_kind"] == "CognitionTurnFailed"
                && event["agent_id"] == agent_id
                && event["agent_session_id"] == "session-missing-terminal"
                && event["agent_turn_id"] == "turn-missing-terminal"
                && event["decision_request_id"] == "request-missing-terminal"
                && event["request_digest"]
                    == active_marker.request_context.request_digest.to_string()
        }),
        "missing identity must produce Runtime terminalization evidence: {events:?}"
    );
    assert!(
        events.iter().any(|event| {
            event["event_kind"] == "CognitionTurnCompleted"
                && event["status"] == "failed"
                && event["agent_id"] == agent_id
                && event["agent_session_id"] == "session-missing-terminal"
                && event["agent_turn_id"] == "turn-missing-terminal"
                && event["decision_request_id"] == "request-missing-terminal"
                && event["request_digest"]
                    == active_marker.request_context.request_digest.to_string()
        }),
        "missing identity must produce a Runtime failed completion: {events:?}"
    );
    let feedback = server
        .world
        .runtime_feedback_outbox()
        .expect("Runtime feedback outbox");
    assert!(
        feedback.iter().any(|record| {
            record.agent_subject == agent_id
                && record.agent_session_id == "session-missing-terminal"
                && record.agent_turn_id == "turn-missing-terminal"
                && record.decision_request_id == "request-missing-terminal"
                && record.request_digest == active_marker.request_context.request_digest.to_string()
        }),
        "missing identity feedback must preserve the Runtime marker identity: {feedback:?}"
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn provider_wake_handoff_failure_retains_terminal_identity_for_recovery() {
    let lineage_path = std::env::temp_dir().join(format!(
        "oasis7-provider-wake-recovery-{}-{}.json",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let mut world = RuntimeWorld::new();
    world
        .bind_cognition_runtime("world", "recovery-wake-branch", 0, None, "pending", 0)
        .expect("Runtime cognition binding");
    world.submit_action(RuntimeAction::RegisterAgent {
        agent_id: "agent-wake-recovery".to_string(),
        pos: GeoPos::new(0, 0, 0),
    });
    world.step().expect("register Runtime agent");
    let mut context = test_provider_context(
        "agent-wake-recovery",
        "turn-wake-recovery",
        "request-wake-recovery",
        1,
    );
    let digest = crate::simulator::h_v1("oasis7.test.wake-recovery-digest.v1", &"wake-recovery");
    context.request_context.observation_digest = digest.clone();
    context.request_context.capability_catalog_digest = digest.clone();
    context.request_context.capability_invocation_context_digest = digest.clone();
    context.request_context.memory_snapshot_digest = digest.clone();
    context.request_context.goal_snapshot_digest = digest.clone();
    context.request_context.continuation_digest = digest.clone();
    context.request_context.runtime_binding.base_world_hash = digest.clone();
    context
        .request_context
        .runtime_binding
        .runtime_manifest_hash = digest.clone();
    context.request_context.request_digest = digest.clone();
    context.turn_context.request_digest = digest;
    super::async_support::runtime_provider_prefix(&mut world, &context)
        .expect("persist provider Runtime prefix");

    let mut server = ViewerRuntimeLiveServer::new(
        ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal)
            .with_decision_mode(ViewerLiveDecisionMode::Llm)
            .with_provider_lineage_store(lineage_path.clone()),
    )
    .expect("Runtime live server");
    server.world = world;
    server
        .llm_sidecar
        .provider_agent_ids
        .insert("agent-wake-recovery".to_string());
    server
        .llm_sidecar
        .provider_contexts
        .insert("agent-wake-recovery".to_string(), context.clone());
    server
        .llm_sidecar
        .provider_active_turns
        .insert("agent-wake-recovery".to_string(), context);
    server
        .llm_sidecar
        .provider_transport_exhausted
        .insert("agent-wake-recovery".to_string());
    // The sidecar mirrors a wake that Runtime no longer has. The resulting
    // consume error is the real authority failure path, rather than a
    // synthetic post-success branch.
    let wake: crate::runtime::SchedulerWakeV1 = serde_json::from_value(serde_json::json!({
        "schema_version": "scheduler-wake.v1",
        "wake_id": "wake-missing-in-runtime",
        "continuation_id": "continuation-wake-recovery",
        "world_id": "world",
        "branch_id": "recovery-wake-branch",
        "finality_epoch": 0,
        "finality_block_hash": null,
        "finality_status": "pending",
        "reorg_epoch": 0,
        "runtime_manifest_hash": "manifest-wake-recovery",
        "agent_id": "agent-wake-recovery",
        "agent_session_id": "session-test",
        "agent_turn_id": "turn-wake-recovery",
        "decision_request_id": "request-wake-recovery",
        "next_wake_tick": 0,
        "eligible_since_tick": 0,
        "starvation_deadline_tick": 1,
        "initial_priority": 0,
        "wake_seq": 1,
        "retry_seq": 0,
        "status": "pending",
        "pending_reason": "capacity_available"
    }))
    .expect("wake fixture");
    server
        .llm_sidecar
        .pending_runtime_wakes
        .insert(wake.wake_id.clone(), wake);
    // The fixture has already restored the durable sidecar state before the
    // control pass; avoid reloading the pre-fault checkpoint over the
    // in-memory wake/error transition while exercising sibling progress.
    server.llm_sidecar.provider_lineage_hydrated = true;

    let trace = server
        .enqueue_llm_action_from_sidecar()
        .expect_err("wake authority failure must return a terminal trace");
    assert!(
        trace
            .llm_error
            .as_deref()
            .is_some_and(|error| error.contains("Runtime wake handoff failed")),
        "trace must expose the real Runtime wake return error: {trace:?}"
    );
    assert!(
        server
            .llm_sidecar
            .provider_recovery_context("agent-wake-recovery")
            .is_some(),
        "wake handoff failure must retain the terminal identity for retry"
    );
    assert!(
        server
            .llm_sidecar
            .provider_wake_recovery_pending("agent-wake-recovery")
            .is_some(),
        "wake handoff failure must leave a durable wake recovery marker"
    );
    let checkpoint: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&lineage_path).expect("read wake recovery checkpoint"),
    )
    .expect("decode wake recovery checkpoint");
    assert!(
        checkpoint["provider_wake_recovery_pending"]
            .as_object()
            .is_some_and(|values| values.contains_key("agent-wake-recovery")),
        "wake recovery marker must be persisted for restart: {checkpoint:?}"
    );

    // An unresolved wake belongs only to its own Agent. A completed sibling
    // response in the queue must still make progress in the next control
    // pass while the wake retry continues to fail.
    server.world.submit_action(RuntimeAction::RegisterAgent {
        agent_id: "agent-wake-sibling".to_string(),
        pos: GeoPos::new(0, 0, 0),
    });
    server.world.step().expect("register wake recovery sibling");
    let fenced_provider = crate::simulator::MockDecisionProvider::new("wake-recovery-fenced");
    let fenced_behavior = crate::simulator::ProviderBackedAgentBehavior::new_legacy_compatibility(
        "agent-wake-recovery",
        fenced_provider,
        vec![crate::simulator::ActionCatalogEntry::new("wait", "wait")],
    );
    let sibling_provider = crate::simulator::MockDecisionProvider::new("wake-recovery-sibling");
    let sibling_behavior = crate::simulator::ProviderBackedAgentBehavior::new_legacy_compatibility(
        "agent-wake-sibling",
        sibling_provider,
        vec![crate::simulator::ActionCatalogEntry::new("wait", "wait")],
    );
    let mut sibling_runner = crate::simulator::AsyncAgentRunner::with_default_capacity();
    sibling_runner
        .register(fenced_behavior)
        .expect("register fenced wake recovery provider actor");
    sibling_runner
        .register(sibling_behavior)
        .expect("register wake recovery sibling provider actor");
    server.llm_sidecar.runner = Some(RuntimeDecisionRunner::ProviderBacked(sibling_runner));
    server
        .llm_sidecar
        .provider_agent_ids
        .insert("agent-wake-sibling".to_string());
    server
        .llm_sidecar
        .provider_completed_decisions
        .push_back(async_support::RuntimeLlmDecision {
            agent_id: "agent-wake-sibling".to_string(),
            decision: AgentDecision::Wait,
            decision_trace: None,
            cognition: None,
            memory_write_intents: Vec::new(),
            continuation_admitted: false,
        });
    server.llm_sidecar.request_decision();
    let sibling_result = server.enqueue_llm_action_from_sidecar();
    assert!(
        sibling_result.is_ok(),
        "an unresolved Agent A wake must not block sibling B: {sibling_result:?}"
    );
    assert!(
        server
            .llm_sidecar
            .provider_held_decisions
            .contains_key("agent-wake-sibling"),
        "healthy sibling B must be delivered while Agent A wake recovery remains pending"
    );

    let mut restarted = ViewerRuntimeLiveServer::new(
        ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal)
            .with_decision_mode(ViewerLiveDecisionMode::Llm)
            .with_provider_lineage_store(lineage_path.clone()),
    )
    .expect("restarted Runtime live server");
    restarted.world = server.world.clone();
    restarted
        .llm_sidecar
        .restore_provider_lineage(&restarted.world)
        .expect("restore wake recovery marker");
    assert!(
        restarted
            .llm_sidecar
            .provider_wake_recovery_pending("agent-wake-recovery")
            .is_some(),
        "restart must retain the exact wake recovery identity"
    );
    assert!(
        restarted.retry_provider_wake_recovery().is_err(),
        "retry must expose the same Runtime wake authority error"
    );
    assert!(
        restarted
            .llm_sidecar
            .provider_recovery_context("agent-wake-recovery")
            .is_some(),
        "failed retry must retain identity rather than redispatching"
    );
    let _ = std::fs::remove_file(lineage_path);
}
