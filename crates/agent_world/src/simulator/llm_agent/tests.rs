use super::*;
use crate::geometry::GeoPos;
use crate::simulator::{
    Observation, ObservedAgent, ObservedLocation, RejectReason, WorldEvent, WorldEventKind,
};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Default, Clone)]
struct MockClient {
    output: Option<String>,
    err: Option<LlmClientError>,
}

impl LlmCompletionClient for MockClient {
    fn complete(
        &self,
        request: &LlmCompletionRequest,
    ) -> Result<LlmCompletionResult, LlmClientError> {
        if let Some(err) = &self.err {
            return Err(err.clone());
        }
        Ok(LlmCompletionResult {
            output: self
                .output
                .clone()
                .unwrap_or_else(|| "{\"decision\":\"wait\"}".to_string()),
            model: Some(request.model.clone()),
            prompt_tokens: Some(12),
            completion_tokens: Some(4),
            total_tokens: Some(16),
        })
    }
}

#[derive(Debug, Clone)]
struct SequenceMockClient {
    outputs: RefCell<VecDeque<String>>,
    model: String,
}

impl SequenceMockClient {
    fn new(outputs: Vec<String>) -> Self {
        Self {
            outputs: RefCell::new(outputs.into()),
            model: "gpt-test".to_string(),
        }
    }
}

impl LlmCompletionClient for SequenceMockClient {
    fn complete(
        &self,
        _request: &LlmCompletionRequest,
    ) -> Result<LlmCompletionResult, LlmClientError> {
        let output = self
            .outputs
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(|| "{\"decision\":\"wait\"}".to_string());
        Ok(LlmCompletionResult {
            output,
            model: Some(self.model.clone()),
            prompt_tokens: Some(12),
            completion_tokens: Some(4),
            total_tokens: Some(16),
        })
    }
}

fn make_observation() -> Observation {
    Observation {
        time: 7,
        agent_id: "agent-1".to_string(),
        pos: GeoPos {
            x_cm: 0.0,
            y_cm: 0.0,
            z_cm: 0.0,
        },
        visibility_range_cm: 100,
        visible_agents: vec![ObservedAgent {
            agent_id: "agent-2".to_string(),
            location_id: "loc-2".to_string(),
            pos: GeoPos {
                x_cm: 1.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
            distance_cm: 1,
        }],
        visible_locations: vec![ObservedLocation {
            location_id: "loc-2".to_string(),
            name: "outpost".to_string(),
            pos: GeoPos {
                x_cm: 1.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
            profile: Default::default(),
            distance_cm: 1,
        }],
    }
}

fn base_config() -> LlmAgentConfig {
    LlmAgentConfig {
        model: "gpt-test".to_string(),
        base_url: "https://example.invalid/v1".to_string(),
        api_key: "test-key".to_string(),
        timeout_ms: 1000,
        system_prompt: "prompt".to_string(),
        short_term_goal: "short-goal".to_string(),
        long_term_goal: "long-goal".to_string(),
        max_module_calls: 3,
    }
}

fn sample_completion_request() -> LlmCompletionRequest {
    LlmCompletionRequest {
        model: "gpt-test".to_string(),
        system_prompt: "system".to_string(),
        user_prompt: "user".to_string(),
    }
}

#[test]
fn llm_config_uses_default_system_prompt() {
    let mut vars = BTreeMap::new();
    vars.insert(ENV_LLM_MODEL.to_string(), "gpt-4o-mini".to_string());
    vars.insert(
        ENV_LLM_BASE_URL.to_string(),
        "https://api.example.com/v1".to_string(),
    );
    vars.insert(ENV_LLM_API_KEY.to_string(), "secret".to_string());

    let config = LlmAgentConfig::from_env_with(|key| vars.get(key).cloned(), "").unwrap();
    assert_eq!(config.system_prompt, DEFAULT_LLM_SYSTEM_PROMPT);
    assert_eq!(config.timeout_ms, DEFAULT_LLM_TIMEOUT_MS);
    assert_eq!(config.short_term_goal, DEFAULT_LLM_SHORT_TERM_GOAL);
    assert_eq!(config.long_term_goal, DEFAULT_LLM_LONG_TERM_GOAL);
    assert_eq!(config.max_module_calls, DEFAULT_LLM_MAX_MODULE_CALLS);
}

#[test]
fn llm_config_reads_system_prompt_from_env() {
    let mut vars = BTreeMap::new();
    vars.insert(ENV_LLM_MODEL.to_string(), "gpt-4o-mini".to_string());
    vars.insert(
        ENV_LLM_BASE_URL.to_string(),
        "https://api.example.com/v1".to_string(),
    );
    vars.insert(ENV_LLM_API_KEY.to_string(), "secret".to_string());
    vars.insert(
        ENV_LLM_SYSTEM_PROMPT.to_string(),
        "自定义 system prompt".to_string(),
    );
    vars.insert(ENV_LLM_TIMEOUT_MS.to_string(), "2000".to_string());

    let config = LlmAgentConfig::from_env_with(|key| vars.get(key).cloned(), "").unwrap();
    assert_eq!(config.system_prompt, "自定义 system prompt");
    assert_eq!(config.timeout_ms, 2000);
}

#[test]
fn llm_config_reads_goal_and_module_limits_from_env() {
    let mut vars = BTreeMap::new();
    vars.insert(ENV_LLM_MODEL.to_string(), "gpt-4o-mini".to_string());
    vars.insert(
        ENV_LLM_BASE_URL.to_string(),
        "https://api.example.com/v1".to_string(),
    );
    vars.insert(ENV_LLM_API_KEY.to_string(), "secret".to_string());
    vars.insert(
        ENV_LLM_SHORT_TERM_GOAL.to_string(),
        "保持本轮高效".to_string(),
    );
    vars.insert(
        ENV_LLM_LONG_TERM_GOAL.to_string(),
        "建立长期资源优势".to_string(),
    );
    vars.insert(ENV_LLM_MAX_MODULE_CALLS.to_string(), "5".to_string());

    let config = LlmAgentConfig::from_env_with(|key| vars.get(key).cloned(), "").unwrap();
    assert_eq!(config.short_term_goal, "保持本轮高效");
    assert_eq!(config.long_term_goal, "建立长期资源优势");
    assert_eq!(config.max_module_calls, 5);
}

#[test]
fn llm_config_agent_scoped_goal_overrides_global_value() {
    let mut vars = BTreeMap::new();
    vars.insert(ENV_LLM_MODEL.to_string(), "gpt-4o-mini".to_string());
    vars.insert(
        ENV_LLM_BASE_URL.to_string(),
        "https://api.example.com/v1".to_string(),
    );
    vars.insert(ENV_LLM_API_KEY.to_string(), "secret".to_string());
    vars.insert(
        ENV_LLM_SHORT_TERM_GOAL.to_string(),
        "global-short".to_string(),
    );
    vars.insert(
        "AGENT_WORLD_LLM_SHORT_TERM_GOAL_AGENT_1".to_string(),
        "agent-short".to_string(),
    );

    let config = LlmAgentConfig::from_env_with(|key| vars.get(key).cloned(), "agent-1").unwrap();
    assert_eq!(config.short_term_goal, "agent-short");
    assert_eq!(config.long_term_goal, DEFAULT_LLM_LONG_TERM_GOAL);
}

#[test]
fn llm_config_reads_from_config_file() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path_buf = std::env::temp_dir().join(format!("agent-world-llm-config-{unique}.toml"));
    let path = Path::new(&path_buf);
    let content = r#"
AGENT_WORLD_LLM_MODEL = "gpt-4o-mini"
AGENT_WORLD_LLM_BASE_URL = "https://api.example.com/v1"
AGENT_WORLD_LLM_API_KEY = "secret"
AGENT_WORLD_LLM_TIMEOUT_MS = 4567
"#;
    std::fs::write(path, content).unwrap();

    let config = LlmAgentConfig::from_config_file(path).unwrap();

    std::fs::remove_file(path).ok();

    assert_eq!(config.model, "gpt-4o-mini");
    assert_eq!(config.base_url, "https://api.example.com/v1");
    assert_eq!(config.api_key, "secret");
    assert_eq!(config.timeout_ms, 4567);
    assert_eq!(config.system_prompt, DEFAULT_LLM_SYSTEM_PROMPT);
}

#[test]
fn build_chat_completions_url_handles_suffix_variants() {
    assert_eq!(
        build_chat_completions_url("https://api.example.com/v1"),
        "https://api.example.com/v1/chat/completions"
    );
    assert_eq!(
        build_chat_completions_url("https://api.example.com/v1/"),
        "https://api.example.com/v1/chat/completions"
    );
    assert_eq!(
        build_chat_completions_url("https://api.example.com/v1/chat/completions"),
        "https://api.example.com/v1/chat/completions"
    );
}

#[test]
fn openai_client_enables_timeout_retry_when_timeout_below_default() {
    let mut config = base_config();
    config.timeout_ms = 8000;
    let client = OpenAiChatCompletionClient::from_config(&config).expect("client");

    assert_eq!(client.request_timeout_ms, 8000);
    assert_eq!(client.timeout_retry_ms, Some(DEFAULT_LLM_TIMEOUT_MS));
    assert!(client.timeout_retry_client.is_some());
}

#[test]
fn openai_client_timeout_retry_can_recover_short_timeout_request() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let addr = listener.local_addr().expect("local addr");
    let accepted = Arc::new(AtomicUsize::new(0));
    let accepted_clone = Arc::clone(&accepted);

    let response_body = r#"{"model":"gpt-test","usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2},"choices":[{"message":{"content":"{\"decision\":\"wait\"}"}}]}"#;
    let handle = thread::spawn(move || {
        for incoming in listener.incoming().take(2) {
            let mut stream = match incoming {
                Ok(stream) => stream,
                Err(_) => break,
            };
            let request_index = accepted_clone.fetch_add(1, Ordering::SeqCst);

            let mut request_buf = [0_u8; 1024];
            let _ = stream.read(&mut request_buf);

            if request_index == 0 {
                thread::sleep(Duration::from_millis(80));
            }

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });

    let config = LlmAgentConfig {
        model: "gpt-test".to_string(),
        base_url: format!("http://{}/v1", addr),
        api_key: "test-key".to_string(),
        timeout_ms: 10,
        system_prompt: "prompt".to_string(),
        short_term_goal: "short-goal".to_string(),
        long_term_goal: "long-goal".to_string(),
        max_module_calls: 3,
    };
    let client = OpenAiChatCompletionClient::from_config(&config).expect("client");

    let completion = client
        .complete(&sample_completion_request())
        .expect("retry should recover timeout");

    assert_eq!(completion.output, "{\"decision\":\"wait\"}");
    assert_eq!(accepted.load(Ordering::SeqCst), 2);

    handle.join().expect("server thread");
}

#[test]
fn llm_agent_parse_move_action() {
    let client = MockClient {
        output: Some("{\"decision\":\"move_agent\",\"to\":\"loc-2\"}".to_string()),
        err: None,
    };
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);
    let decision = behavior.decide(&make_observation());

    assert_eq!(
        decision,
        AgentDecision::Act(Action::MoveAgent {
            agent_id: "agent-1".to_string(),
            to: "loc-2".to_string(),
        })
    );
}

#[test]
fn llm_agent_parse_json_in_markdown_block() {
    let client = MockClient {
        output: Some(
            "```json\n{\"decision\":\"harvest_radiation\",\"max_amount\":5}\n```".to_string(),
        ),
        err: None,
    };
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);
    let decision = behavior.decide(&make_observation());

    assert_eq!(
        decision,
        AgentDecision::Act(Action::HarvestRadiation {
            agent_id: "agent-1".to_string(),
            max_amount: 5,
        })
    );
}

#[test]
fn llm_agent_falls_back_to_wait_when_client_fails() {
    let client = MockClient {
        output: None,
        err: Some(LlmClientError::Http {
            message: "timeout".to_string(),
        }),
    };
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);
    let decision = behavior.decide(&make_observation());
    assert_eq!(decision, AgentDecision::Wait);
}

#[test]
fn llm_agent_falls_back_to_wait_when_output_invalid() {
    let client = MockClient {
        output: Some("not json".to_string()),
        err: None,
    };
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);
    let decision = behavior.decide(&make_observation());
    assert_eq!(decision, AgentDecision::Wait);
}

#[test]
fn llm_agent_emits_decision_trace_with_io() {
    let client = MockClient {
        output: Some("{\"decision\":\"move_agent\",\"to\":\"loc-2\"}".to_string()),
        err: None,
    };
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);

    let decision = behavior.decide(&make_observation());
    assert!(matches!(
        decision,
        AgentDecision::Act(Action::MoveAgent { .. })
    ));

    let trace = behavior.take_decision_trace().expect("trace should exist");
    assert_eq!(trace.agent_id, "agent-1");
    assert!(trace
        .llm_input
        .as_deref()
        .unwrap_or_default()
        .contains("[system]"));
    assert!(trace
        .llm_output
        .as_deref()
        .unwrap_or_default()
        .contains("move_agent"));
    assert_eq!(trace.llm_error, None);
    assert_eq!(trace.parse_error, None);
    let diagnostics = trace.llm_diagnostics.as_ref().expect("diagnostics");
    assert_eq!(diagnostics.model.as_deref(), Some("gpt-test"));
    assert_eq!(diagnostics.prompt_tokens, Some(12));
    assert_eq!(diagnostics.completion_tokens, Some(4));
    assert_eq!(diagnostics.total_tokens, Some(16));
    assert_eq!(diagnostics.retry_count, 0);
    assert!(diagnostics.latency_ms.is_some());
    assert_eq!(behavior.take_decision_trace(), None);
}

#[test]
fn llm_agent_supports_module_call_then_decision() {
    let client = SequenceMockClient::new(vec![
        "{\"type\":\"module_call\",\"module\":\"agent.modules.list\",\"args\":{}}".to_string(),
        "{\"decision\":\"move_agent\",\"to\":\"loc-2\"}".to_string(),
    ]);
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);

    let decision = behavior.decide(&make_observation());
    assert!(matches!(
        decision,
        AgentDecision::Act(Action::MoveAgent { .. })
    ));

    let trace = behavior.take_decision_trace().expect("trace exists");
    assert_eq!(trace.parse_error, None);
    assert_eq!(trace.llm_effect_intents.len(), 1);
    assert_eq!(trace.llm_effect_receipts.len(), 1);
    let intent = &trace.llm_effect_intents[0];
    assert_eq!(intent.kind, "llm.prompt.module_call");
    assert_eq!(intent.cap_ref, "llm.prompt.module_access");
    assert_eq!(intent.origin, "llm_agent");
    assert_eq!(
        intent.params.get("module").and_then(|value| value.as_str()),
        Some("agent.modules.list")
    );
    let receipt = &trace.llm_effect_receipts[0];
    assert_eq!(receipt.intent_id, intent.intent_id);
    assert_eq!(receipt.status, "ok");
    assert_eq!(receipt.cost_cents, None);
    let llm_input = trace.llm_input.unwrap_or_default();
    assert!(llm_input.contains("[module_result:agent.modules.list]"));
    let llm_output = trace.llm_output.unwrap_or_default();
    assert!(llm_output.contains("module_call"));
    assert!(llm_output.contains("move_agent"));
    let diagnostics = trace.llm_diagnostics.expect("diagnostics");
    assert_eq!(diagnostics.prompt_tokens, Some(24));
    assert_eq!(diagnostics.completion_tokens, Some(8));
    assert_eq!(diagnostics.total_tokens, Some(32));
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
    let parse_error = trace.parse_error.unwrap_or_default();
    assert!(parse_error.contains("module call limit exceeded"));
}

#[test]
fn llm_agent_system_prompt_contains_configured_goals() {
    let behavior = LlmAgentBehavior::new("agent-1", base_config(), MockClient::default());
    let system_prompt = behavior.system_prompt();
    assert!(system_prompt.contains("short-goal"));
    assert!(system_prompt.contains("long-goal"));
    assert!(system_prompt.contains("module_call"));
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
    assert_eq!(diagnostics.retry_count, 0);
}
