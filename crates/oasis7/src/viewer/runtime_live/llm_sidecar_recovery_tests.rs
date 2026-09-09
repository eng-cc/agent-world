use super::*;
use crate::viewer::runtime_live::WorldScenario;
use crate::viewer::{ViewerRuntimeLiveServer, ViewerRuntimeLiveServerConfig};

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
