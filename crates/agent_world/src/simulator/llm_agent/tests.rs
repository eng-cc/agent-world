use super::*;
use crate::geometry::GeoPos;
use crate::simulator::{
    Action, Observation, ObservedAgent, ObservedLocation, RejectReason, ResourceKind,
    ResourceOwner, WorldEvent, WorldEventKind,
};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
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

#[derive(Debug, Clone)]
struct StressMockClient {
    calls: Arc<AtomicUsize>,
}

impl StressMockClient {
    fn new(calls: Arc<AtomicUsize>) -> Self {
        Self { calls }
    }
}

impl LlmCompletionClient for StressMockClient {
    fn complete(
        &self,
        _request: &LlmCompletionRequest,
    ) -> Result<LlmCompletionResult, LlmClientError> {
        let call_index = self.calls.fetch_add(1, Ordering::SeqCst);
        let output = match call_index % 5 {
            0 => r#"{"type":"plan","missing":["memory"],"next":"module_call"}"#,
            1 => r#"{"type":"module_call","module":"memory.short_term.recent","args":{"limit":4}}"#,
            2 => r#"{"type":"decision_draft","decision":{"decision":"move_agent","to":"loc-2"},"confidence":0.64,"need_verify":true}"#,
            3 => "not-json",
            _ => r#"{"decision":"move_agent","to":"loc-2"}"#,
        }
        .to_string();

        Ok(LlmCompletionResult {
            output,
            model: Some("gpt-stress".to_string()),
            prompt_tokens: Some(16),
            completion_tokens: Some(6),
            total_tokens: Some(22),
        })
    }
}

#[derive(Debug, Clone)]
struct CountingSequenceMockClient {
    outputs: RefCell<VecDeque<String>>,
    calls: Arc<AtomicUsize>,
}

impl CountingSequenceMockClient {
    fn new(outputs: Vec<String>, calls: Arc<AtomicUsize>) -> Self {
        Self {
            outputs: RefCell::new(outputs.into()),
            calls,
        }
    }
}

impl LlmCompletionClient for CountingSequenceMockClient {
    fn complete(
        &self,
        _request: &LlmCompletionRequest,
    ) -> Result<LlmCompletionResult, LlmClientError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let output = self
            .outputs
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(|| "{\"decision\":\"wait\"}".to_string());
        Ok(LlmCompletionResult {
            output,
            model: Some("gpt-count".to_string()),
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

fn make_dense_observation(time: u64, extra: usize) -> Observation {
    let mut observation = make_observation();
    observation.time = time;

    for index in 0..extra {
        observation.visible_agents.push(ObservedAgent {
            agent_id: format!("agent-extra-{index}"),
            location_id: format!("loc-extra-{index}"),
            pos: GeoPos {
                x_cm: 100.0 + index as f64,
                y_cm: (index as f64) * 0.5,
                z_cm: 0.0,
            },
            distance_cm: 100 + index as i64,
        });
        observation.visible_locations.push(ObservedLocation {
            location_id: format!("loc-extra-{index}"),
            name: format!("outpost-extra-{index}"),
            pos: GeoPos {
                x_cm: 100.0 + index as f64,
                y_cm: (index as f64) * 0.5,
                z_cm: 0.0,
            },
            profile: Default::default(),
            distance_cm: 100 + index as i64,
        });
    }

    observation
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
        max_decision_steps: 4,
        max_repair_rounds: 1,
        prompt_max_history_items: 4,
        prompt_profile: LlmPromptProfile::Balanced,
        force_replan_after_same_action: DEFAULT_LLM_FORCE_REPLAN_AFTER_SAME_ACTION,
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
    assert_eq!(config.max_decision_steps, DEFAULT_LLM_MAX_DECISION_STEPS);
    assert_eq!(config.max_repair_rounds, DEFAULT_LLM_MAX_REPAIR_ROUNDS);
    assert_eq!(
        config.prompt_max_history_items,
        DEFAULT_LLM_PROMPT_MAX_HISTORY_ITEMS
    );
    assert_eq!(config.prompt_profile, DEFAULT_LLM_PROMPT_PROFILE);
    assert_eq!(
        config.force_replan_after_same_action,
        DEFAULT_LLM_FORCE_REPLAN_AFTER_SAME_ACTION
    );
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
fn llm_config_reads_multistep_and_prompt_fields_from_env() {
    let mut vars = BTreeMap::new();
    vars.insert(ENV_LLM_MODEL.to_string(), "gpt-4o-mini".to_string());
    vars.insert(
        ENV_LLM_BASE_URL.to_string(),
        "https://api.example.com/v1".to_string(),
    );
    vars.insert(ENV_LLM_API_KEY.to_string(), "secret".to_string());
    vars.insert(ENV_LLM_MAX_DECISION_STEPS.to_string(), "6".to_string());
    vars.insert(ENV_LLM_MAX_REPAIR_ROUNDS.to_string(), "2".to_string());
    vars.insert(
        ENV_LLM_PROMPT_MAX_HISTORY_ITEMS.to_string(),
        "7".to_string(),
    );
    vars.insert(ENV_LLM_PROMPT_PROFILE.to_string(), "compact".to_string());
    vars.insert(
        ENV_LLM_FORCE_REPLAN_AFTER_SAME_ACTION.to_string(),
        "9".to_string(),
    );

    let config = LlmAgentConfig::from_env_with(|key| vars.get(key).cloned(), "").unwrap();
    assert_eq!(config.max_decision_steps, 6);
    assert_eq!(config.max_repair_rounds, 2);
    assert_eq!(config.prompt_max_history_items, 7);
    assert_eq!(config.prompt_profile, LlmPromptProfile::Compact);
    assert_eq!(config.force_replan_after_same_action, 9);
}

#[test]
fn llm_config_rejects_invalid_prompt_profile() {
    let mut vars = BTreeMap::new();
    vars.insert(ENV_LLM_MODEL.to_string(), "gpt-4o-mini".to_string());
    vars.insert(
        ENV_LLM_BASE_URL.to_string(),
        "https://api.example.com/v1".to_string(),
    );
    vars.insert(ENV_LLM_API_KEY.to_string(), "secret".to_string());
    vars.insert(
        ENV_LLM_PROMPT_PROFILE.to_string(),
        "invalid-profile".to_string(),
    );

    let err = LlmAgentConfig::from_env_with(|key| vars.get(key).cloned(), "").unwrap_err();
    assert!(matches!(err, LlmConfigError::InvalidPromptProfile { .. }));
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
    assert_eq!(config.max_decision_steps, DEFAULT_LLM_MAX_DECISION_STEPS);
    assert_eq!(config.max_repair_rounds, DEFAULT_LLM_MAX_REPAIR_ROUNDS);
    assert_eq!(
        config.prompt_max_history_items,
        DEFAULT_LLM_PROMPT_MAX_HISTORY_ITEMS
    );
    assert_eq!(config.prompt_profile, DEFAULT_LLM_PROMPT_PROFILE);
    assert_eq!(
        config.force_replan_after_same_action,
        DEFAULT_LLM_FORCE_REPLAN_AFTER_SAME_ACTION
    );
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
    let request_bodies = Arc::new(Mutex::new(Vec::new()));
    let request_bodies_clone = Arc::clone(&request_bodies);

    let response_body = r#"{"model":"gpt-test","usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2},"choices":[{"message":{"content":null,"tool_calls":[{"id":"call_1","type":"function","function":{"name":"agent_modules_list","arguments":"{}"}}]}}]}"#;
    let handle = thread::spawn(move || {
        for incoming in listener.incoming().take(2) {
            let mut stream = match incoming {
                Ok(stream) => stream,
                Err(_) => break,
            };
            let request_index = accepted_clone.fetch_add(1, Ordering::SeqCst);

            let mut request_buf = [0_u8; 4096];
            let read = stream.read(&mut request_buf).unwrap_or(0);
            let request_text = String::from_utf8_lossy(&request_buf[..read]).to_string();
            request_bodies_clone
                .lock()
                .expect("request bodies lock")
                .push(request_text);

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
        max_decision_steps: 4,
        max_repair_rounds: 1,
        prompt_max_history_items: 4,
        prompt_profile: LlmPromptProfile::Balanced,
        force_replan_after_same_action: DEFAULT_LLM_FORCE_REPLAN_AFTER_SAME_ACTION,
    };
    let client = OpenAiChatCompletionClient::from_config(&config).expect("client");

    let completion = client
        .complete(&sample_completion_request())
        .expect("retry should recover timeout");

    let output: serde_json::Value =
        serde_json::from_str(completion.output.as_str()).expect("output json");
    assert_eq!(
        output.get("type").and_then(|value| value.as_str()),
        Some("module_call")
    );
    assert_eq!(
        output.get("module").and_then(|value| value.as_str()),
        Some("agent.modules.list")
    );
    assert_eq!(accepted.load(Ordering::SeqCst), 2);

    let request_bodies = request_bodies.lock().expect("request bodies lock");
    assert!(!request_bodies.is_empty());
    let latest_request = &request_bodies[request_bodies.len() - 1];
    assert!(latest_request.contains("\"tools\""));
    assert!(latest_request.contains("\"tool_choice\":\"auto\""));
    assert!(latest_request.contains("agent_modules_list"));

    handle.join().expect("server thread");
}

#[test]
fn openai_client_parses_legacy_function_call_payload() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let addr = listener.local_addr().expect("local addr");

    let response_body = r#"{"model":"gpt-test","choices":[{"message":{"content":null,"function_call":{"name":"memory_short_term_recent","arguments":"{\"limit\":5}"}}}]}"#;
    let handle = thread::spawn(move || {
        if let Some(Ok(mut stream)) = listener.incoming().next() {
            let mut request_buf = [0_u8; 1024];
            let _ = stream.read(&mut request_buf);
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
        timeout_ms: 1000,
        system_prompt: "prompt".to_string(),
        short_term_goal: "short-goal".to_string(),
        long_term_goal: "long-goal".to_string(),
        max_module_calls: 3,
        max_decision_steps: 4,
        max_repair_rounds: 1,
        prompt_max_history_items: 4,
        prompt_profile: LlmPromptProfile::Balanced,
        force_replan_after_same_action: DEFAULT_LLM_FORCE_REPLAN_AFTER_SAME_ACTION,
    };
    let client = OpenAiChatCompletionClient::from_config(&config).expect("client");

    let completion = client
        .complete(&sample_completion_request())
        .expect("legacy function_call should parse");

    let output: serde_json::Value =
        serde_json::from_str(completion.output.as_str()).expect("output json");
    assert_eq!(
        output.get("module").and_then(|value| value.as_str()),
        Some("memory.short_term.recent")
    );
    assert_eq!(
        output
            .get("args")
            .and_then(|args| args.get("limit"))
            .and_then(|value| value.as_i64()),
        Some(5)
    );

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
fn llm_agent_supports_plan_module_draft_finalize_flow() {
    let client = SequenceMockClient::new(vec![
        r#"{"type":"plan","missing":["memory"],"next":"module_call"}"#.to_string(),
        r#"{"type":"module_call","module":"agent.modules.list","args":{}}"#.to_string(),
        r#"{"type":"decision_draft","decision":{"decision":"move_agent","to":"loc-2"},"confidence":0.78,"need_verify":false}"#.to_string(),
        r#"{"decision":"move_agent","to":"loc-2"}"#.to_string(),
    ]);
    let mut config = base_config();
    config.max_decision_steps = 6;
    let mut behavior = LlmAgentBehavior::new("agent-1", config, client);

    let decision = behavior.decide(&make_observation());
    assert!(matches!(
        decision,
        AgentDecision::Act(Action::MoveAgent { .. })
    ));

    let trace = behavior.take_decision_trace().expect("trace exists");
    assert!(trace.parse_error.is_none());
    assert_eq!(trace.llm_effect_intents.len(), 1);
    assert_eq!(trace.llm_effect_receipts.len(), 1);
    assert!(!trace.llm_step_trace.is_empty());
    assert!(trace
        .llm_step_trace
        .iter()
        .any(|step| step.step_type == "plan"));
    assert!(trace
        .llm_step_trace
        .iter()
        .any(|step| step.step_type == "module_call"));
    assert!(trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("decision_draft")));
    assert!(!trace.llm_prompt_section_trace.is_empty());
}

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
        ));

        let trace = behavior.take_decision_trace().expect("trace exists");
        assert!(trace.parse_error.is_none());
        assert_eq!(trace.llm_effect_intents.len(), 1);
        assert_eq!(trace.llm_effect_receipts.len(), 1);
        assert!(trace
            .llm_step_trace
            .iter()
            .any(|step| step.step_type == "repair"));
        assert!(!trace.llm_prompt_section_trace.is_empty());
        let input_len = trace.llm_input.unwrap_or_default().len();
        assert!(input_len < 120_000, "llm_input too large: {input_len}");
        assert_eq!(
            trace
                .llm_diagnostics
                .as_ref()
                .map(|diagnostics| diagnostics.retry_count),
            Some(1)
        );
    }

    let total_calls = calls.load(Ordering::SeqCst);
    assert!(total_calls >= TICKS * 5);
    assert!(total_calls <= TICKS * 6);
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
fn llm_agent_user_prompt_contains_step_context_metadata() {
    let behavior = LlmAgentBehavior::new("agent-1", base_config(), MockClient::default());
    let prompt = behavior.user_prompt(&make_observation(), &[], 2, 5);
    assert!(prompt.contains("step_index: 2"));
    assert!(prompt.contains("max_steps: 5"));
    assert!(prompt.contains("module_calls_used: 0"));
    assert!(prompt.contains("module_calls_max: 3"));
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
}

#[test]
fn llm_parse_execute_until_accepts_event_any_of() {
    let parsed = super::decision_flow::parse_llm_turn_response(
        r#"{"decision":"execute_until","action":{"decision":"harvest_radiation","max_amount":3},"until":{"event_any_of":["new_visible_agent","new_visible_location"]},"max_ticks":5}"#,
        "agent-1",
    );

    match parsed {
        super::decision_flow::ParsedLlmTurn::ExecuteUntil(directive) => {
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
fn llm_parse_execute_until_accepts_threshold_event_with_value_lte() {
    let parsed = super::decision_flow::parse_llm_turn_response(
        r#"{"decision":"execute_until","action":{"decision":"harvest_radiation","max_amount":3},"until":{"event":"harvest_yield_below","value_lte":2},"max_ticks":5}"#,
        "agent-1",
    );

    match parsed {
        super::decision_flow::ParsedLlmTurn::ExecuteUntil(directive) => {
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
    let parsed = super::decision_flow::parse_llm_turn_response(
        r#"{"decision":"execute_until","action":{"decision":"harvest_radiation","max_amount":3},"until":{"event":"harvest_available_below"},"max_ticks":5}"#,
        "agent-1",
    );

    match parsed {
        super::decision_flow::ParsedLlmTurn::Invalid(err) => {
            assert!(err.contains("requires until.value_lte"));
        }
        other => panic!("expected invalid execute_until, got {other:?}"),
    }
}

#[test]
fn llm_parse_decision_draft_accepts_shorthand_decision_payload() {
    let parsed = super::decision_flow::parse_llm_turn_response(
        r#"{"type":"decision_draft","decision":"harvest_radiation","max_amount":7,"need_verify":false}"#,
        "agent-1",
    );

    match parsed {
        super::decision_flow::ParsedLlmTurn::DecisionDraft(draft) => {
            assert!(matches!(
                draft.decision,
                AgentDecision::Act(Action::HarvestRadiation { max_amount: 7, .. })
            ));
            assert!(!draft.need_verify);
        }
        other => panic!("expected decision_draft, got {other:?}"),
    }
}
