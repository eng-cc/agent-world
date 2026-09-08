//! Native C0 request budget and retry accounting tests.

use super::agent_cognition_identity::{
    production_observation, production_request_fixture, production_turn_context, request_fixture,
    request_from_value,
};
use crate::simulator::{AgentCognitionStore, AsyncAgentRunner};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn cognition_budget_fields_are_wire_data_bound_to_request_identity() {
    let mut limited = request_fixture(0, 60_000);
    limited["budget_contract"] = json!({
        "max_latency_ms": 60_000,
        "max_repair_attempts": 2,
        "max_model_calls": 2,
        "max_tool_calls": 1
    });
    let limited_context = request_from_value(limited.clone());
    let encoded = serde_json::to_value(&limited_context).expect("encode budget context");
    assert_eq!(encoded["budget_contract"]["max_model_calls"], json!(2));
    assert_eq!(encoded["budget_contract"]["max_tool_calls"], json!(1));

    limited["budget_contract"]["max_model_calls"] = json!(3);
    let changed_context = request_from_value(limited);
    assert_ne!(
        limited_context.request_digest, changed_context.request_digest,
        "changing the request budget must change request identity"
    );

    let mut store = AgentCognitionStore::default();
    store
        .begin_request(limited_context)
        .expect("first budget-bound request accepted");
    let error = store
        .begin_request(changed_context)
        .expect_err("changing budget on the same request key must fail closed");
    assert_eq!(error.code(), "request_identity_collision");
}

#[derive(Clone)]
struct RetryCountingLlmClient {
    calls: Arc<AtomicUsize>,
}

impl crate::simulator::llm_agent::LlmCompletionClient for RetryCountingLlmClient {
    fn complete(
        &self,
        request: &crate::simulator::llm_agent::LlmCompletionRequest,
    ) -> Result<crate::simulator::llm_agent::LlmCompletionResult, crate::simulator::LlmClientError>
    {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(crate::simulator::llm_agent::LlmCompletionResult {
            turns: vec![crate::simulator::llm_agent::LlmCompletionTurn::Decision {
                payload: json!({"decision": "wait"}),
            }],
            output: "{\"decision\":\"wait\"}".to_string(),
            model: Some(request.model.clone()),
            prompt_tokens: Some(1),
            completion_tokens: Some(1),
            total_tokens: Some(2),
        })
    }
}

fn retry_budget_llm_config() -> crate::simulator::LlmAgentConfig {
    crate::simulator::LlmAgentConfig {
        model: "gpt-budget-test".to_string(),
        base_url: "https://example.invalid/v1".to_string(),
        api_key: "test-key".to_string(),
        timeout_ms: 1_000,
        system_prompt: "test".to_string(),
        short_term_goal: "test".to_string(),
        long_term_goal: "test".to_string(),
        max_module_calls: 3,
        max_decision_steps: 4,
        max_repair_rounds: 1,
        prompt_max_history_items: 4,
        prompt_profile: crate::simulator::llm_agent::LlmPromptProfile::Balanced,
        force_replan_after_same_action: 4,
        harvest_max_amount_cap: 100,
        execute_until_auto_reenter_ticks: 4,
        llm_debug_mode: false,
    }
}

#[test]
fn transport_retry_keeps_request_model_budget_consumed() {
    let mut fixture = production_request_fixture(1, 60_000);
    fixture["retry_seq"] = json!(1);
    fixture["budget_contract"]["max_model_calls"] = json!(1);
    fixture["budget_contract"]["max_tool_calls"] = json!(1);
    let request_context = request_from_value(fixture);
    let turn_context = production_turn_context(&request_context);
    let calls = Arc::new(AtomicUsize::new(0));
    let behavior = crate::simulator::LlmAgentBehavior::new(
        "agent-1",
        retry_budget_llm_config(),
        RetryCountingLlmClient {
            calls: Arc::clone(&calls),
        },
    );
    let mut runner = AsyncAgentRunner::new(16).expect("create target actor runner");
    runner.register(behavior).expect("register builtin actor");
    let turn_id = runner
        .start_turn_with_request_context_and_observation(
            "agent-1",
            production_observation("agent-1", 42),
            turn_context.clone(),
            request_context.clone(),
        )
        .expect("open production builtin turn");
    let first = loop {
        if let Some(outcome) = runner
            .poll_completed()
            .expect("poll production builtin turn")
            .into_iter()
            .find(|outcome| outcome.turn_id == turn_id)
        {
            break outcome;
        }
        std::thread::yield_now();
    };
    assert_eq!(
        first.decision,
        Some(crate::simulator::AgentDecision::Wait),
        "first outcome: {first:?}"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    runner
        .retry_awaiting_turn_with_request_context_and_observation(
            "agent-1",
            production_observation("agent-1", 43),
            turn_context,
            request_context,
        )
        .expect("transport retry accepted");
    let retry = loop {
        if let Some(outcome) = runner
            .poll_completed()
            .expect("poll retried builtin turn")
            .into_iter()
            .find(|outcome| outcome.turn_id == turn_id)
        {
            break outcome;
        }
        std::thread::yield_now();
    };
    // The async runner exposes provider failures through `decision_trace` and
    // leaves the top-level decision empty. The native behavior still emits a
    // stable Wait trace before the second provider invocation is attempted.
    assert!(retry.decision.is_none(), "retry outcome: {retry:?}");
    assert_eq!(
        retry
            .decision_trace
            .as_ref()
            .map(|trace| trace.decision.clone()),
        Some(crate::simulator::AgentDecision::Wait),
        "retry trace: {retry:?}"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(
        retry
            .decision_trace
            .and_then(|trace| trace.llm_error)
            .is_some_and(|error| error.starts_with("budget_exhausted:"))
    );
}
