use super::*;
use crate::geometry::GeoPos;
use crate::simulator::{
    Action, Observation, ObservedAgent, ObservedLocation, RejectReason, ResourceKind,
    ResourceOwner, WorldEvent, WorldEventKind,
};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

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
        harvest_max_amount_cap: DEFAULT_LLM_HARVEST_MAX_AMOUNT_CAP,
        execute_until_auto_reenter_ticks: DEFAULT_LLM_EXECUTE_UNTIL_AUTO_REENTER_TICKS,
    }
}

#[test]
fn llm_prompt_profile_uses_relaxed_token_budget_for_stability() {
    let compact = LlmPromptProfile::Compact.prompt_budget();
    assert_eq!(compact.context_window_tokens, 4_096);
    assert_eq!(compact.reserved_output_tokens, 768);
    assert_eq!(compact.safety_margin_tokens, 352);
    assert_eq!(compact.effective_input_budget_tokens(), 2_976);

    let balanced = LlmPromptProfile::Balanced.prompt_budget();
    assert_eq!(balanced.context_window_tokens, 4_608);
    assert_eq!(balanced.reserved_output_tokens, 896);
    assert_eq!(balanced.safety_margin_tokens, 480);
    assert_eq!(balanced.effective_input_budget_tokens(), 3_232);
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
    assert_eq!(
        config.harvest_max_amount_cap,
        DEFAULT_LLM_HARVEST_MAX_AMOUNT_CAP
    );
    assert_eq!(
        config.execute_until_auto_reenter_ticks,
        DEFAULT_LLM_EXECUTE_UNTIL_AUTO_REENTER_TICKS
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
    vars.insert(ENV_LLM_HARVEST_MAX_AMOUNT_CAP.to_string(), "88".to_string());
    vars.insert(
        ENV_LLM_EXECUTE_UNTIL_AUTO_REENTER_TICKS.to_string(),
        "5".to_string(),
    );

    let config = LlmAgentConfig::from_env_with(|key| vars.get(key).cloned(), "").unwrap();
    assert_eq!(config.max_decision_steps, 6);
    assert_eq!(config.max_repair_rounds, 2);
    assert_eq!(config.prompt_max_history_items, 7);
    assert_eq!(config.prompt_profile, LlmPromptProfile::Compact);
    assert_eq!(config.force_replan_after_same_action, 9);
    assert_eq!(config.harvest_max_amount_cap, 88);
    assert_eq!(config.execute_until_auto_reenter_ticks, 5);
}

#[test]
fn llm_config_rejects_invalid_harvest_max_amount_cap() {
    let mut vars = BTreeMap::new();
    vars.insert(ENV_LLM_MODEL.to_string(), "gpt-4o-mini".to_string());
    vars.insert(
        ENV_LLM_BASE_URL.to_string(),
        "https://api.example.com/v1".to_string(),
    );
    vars.insert(ENV_LLM_API_KEY.to_string(), "secret".to_string());
    vars.insert(ENV_LLM_HARVEST_MAX_AMOUNT_CAP.to_string(), "0".to_string());

    let err = LlmAgentConfig::from_env_with(|key| vars.get(key).cloned(), "").unwrap_err();
    assert!(matches!(
        err,
        LlmConfigError::InvalidHarvestMaxAmountCap { .. }
    ));
}

#[test]
fn llm_config_rejects_invalid_execute_until_auto_reenter_ticks() {
    let mut vars = BTreeMap::new();
    vars.insert(ENV_LLM_MODEL.to_string(), "gpt-4o-mini".to_string());
    vars.insert(
        ENV_LLM_BASE_URL.to_string(),
        "https://api.example.com/v1".to_string(),
    );
    vars.insert(ENV_LLM_API_KEY.to_string(), "secret".to_string());
    vars.insert(
        ENV_LLM_EXECUTE_UNTIL_AUTO_REENTER_TICKS.to_string(),
        "-1".to_string(),
    );

    let err = LlmAgentConfig::from_env_with(|key| vars.get(key).cloned(), "").unwrap_err();
    assert!(matches!(
        err,
        LlmConfigError::InvalidExecuteUntilAutoReenterTicks { .. }
    ));
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
    assert_eq!(
        config.harvest_max_amount_cap,
        DEFAULT_LLM_HARVEST_MAX_AMOUNT_CAP
    );
    assert_eq!(
        config.execute_until_auto_reenter_ticks,
        DEFAULT_LLM_EXECUTE_UNTIL_AUTO_REENTER_TICKS
    );
}

#[test]
fn normalize_openai_api_base_url_handles_suffix_variants() {
    assert_eq!(
        normalize_openai_api_base_url("https://api.example.com/v1"),
        "https://api.example.com/v1"
    );
    assert_eq!(
        normalize_openai_api_base_url("https://api.example.com/v1/"),
        "https://api.example.com/v1"
    );
    assert_eq!(
        normalize_openai_api_base_url("https://api.example.com/v1/chat/completions"),
        "https://api.example.com/v1"
    );
    assert_eq!(
        normalize_openai_api_base_url("https://api.example.com/v1/responses"),
        "https://api.example.com/v1"
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
fn responses_tools_register_expected_function_names() {
    let tools = responses_tools();
    assert_eq!(tools.len(), 4);

    let names = tools
        .into_iter()
        .filter_map(|tool| match tool {
            Tool::Function(function_tool) => Some(function_tool.name),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        vec![
            OPENAI_TOOL_AGENT_MODULES_LIST.to_string(),
            OPENAI_TOOL_ENVIRONMENT_CURRENT_OBSERVATION.to_string(),
            OPENAI_TOOL_MEMORY_SHORT_TERM_RECENT.to_string(),
            OPENAI_TOOL_MEMORY_LONG_TERM_SEARCH.to_string(),
        ]
    );
}

#[test]
fn response_function_call_maps_to_module_call_json() {
    let output_item = OutputItem::FunctionCall(async_openai::types::responses::FunctionToolCall {
        arguments: "{\"limit\":5}".to_string(),
        call_id: "call_1".to_string(),
        name: OPENAI_TOOL_MEMORY_SHORT_TERM_RECENT.to_string(),
        id: None,
        status: None,
    });

    let output_json = output_item_to_module_call_json(&output_item).expect("module_call json");
    let value: serde_json::Value = serde_json::from_str(output_json.as_str()).expect("json");

    assert_eq!(
        value.get("type").and_then(|v| v.as_str()),
        Some("module_call")
    );
    assert_eq!(
        value.get("module").and_then(|v| v.as_str()),
        Some("memory.short_term.recent")
    );
    assert_eq!(
        value
            .get("args")
            .and_then(|args| args.get("limit"))
            .and_then(|v| v.as_i64()),
        Some(5)
    );
}

#[test]
fn response_function_call_invalid_json_arguments_are_preserved_as_raw() {
    let output_item = OutputItem::FunctionCall(async_openai::types::responses::FunctionToolCall {
        arguments: "not-json".to_string(),
        call_id: "call_2".to_string(),
        name: OPENAI_TOOL_AGENT_MODULES_LIST.to_string(),
        id: None,
        status: None,
    });

    let output_json = output_item_to_module_call_json(&output_item).expect("module_call json");
    let value: serde_json::Value = serde_json::from_str(output_json.as_str()).expect("json");

    assert_eq!(
        value
            .get("args")
            .and_then(|args| args.get("_raw"))
            .and_then(|v| v.as_str()),
        Some("not-json")
    );
}

#[test]
fn build_responses_request_payload_includes_tools_and_auto_choice() {
    let request = LlmCompletionRequest {
        model: "gpt-test".to_string(),
        system_prompt: "system".to_string(),
        user_prompt: "user".to_string(),
    };

    let payload = build_responses_request_payload(&request).expect("payload");
    let payload_json = serde_json::to_value(payload).expect("payload json");

    assert_eq!(
        payload_json.get("instructions").and_then(|v| v.as_str()),
        Some("system")
    );
    assert_eq!(
        payload_json.get("input").and_then(|v| v.as_str()),
        Some("user")
    );

    let tool_choice = payload_json
        .get("tool_choice")
        .expect("tool choice exists")
        .as_str()
        .expect("tool choice string");
    assert_eq!(tool_choice, "auto");

    let tools = payload_json
        .get("tools")
        .and_then(|v| v.as_array())
        .expect("tools array");
    assert_eq!(tools.len(), 4);

    let function_names = tools
        .iter()
        .filter_map(|tool| tool.get("name").and_then(|v| v.as_str()))
        .collect::<Vec<_>>();
    assert_eq!(
        function_names,
        vec![
            OPENAI_TOOL_AGENT_MODULES_LIST,
            OPENAI_TOOL_ENVIRONMENT_CURRENT_OBSERVATION,
            OPENAI_TOOL_MEMORY_SHORT_TERM_RECENT,
            OPENAI_TOOL_MEMORY_LONG_TERM_SEARCH,
        ]
    );
}

#[test]
fn completion_result_from_raw_response_json_parses_function_call_without_annotations() {
    let raw = r#"{
      "model":"gpt-test",
      "output":[
        {
          "type":"function_call",
          "name":"memory_short_term_recent",
          "arguments":"{\"limit\":3}"
        }
      ],
      "usage":{
        "input_tokens":11,
        "output_tokens":7,
        "total_tokens":18
      }
    }"#;

    let result = completion_result_from_raw_response_json(raw).expect("result");
    let output: serde_json::Value = serde_json::from_str(result.output.as_str()).expect("json");

    assert_eq!(result.model.as_deref(), Some("gpt-test"));
    assert_eq!(result.prompt_tokens, Some(11));
    assert_eq!(result.completion_tokens, Some(7));
    assert_eq!(result.total_tokens, Some(18));
    assert_eq!(
        output.get("module").and_then(|value| value.as_str()),
        Some("memory.short_term.recent")
    );
    assert_eq!(
        output
            .get("args")
            .and_then(|value| value.get("limit"))
            .and_then(|value| value.as_i64()),
        Some(3)
    );
}

#[test]
fn completion_result_from_raw_response_json_parses_output_text_without_annotations() {
    let raw = r#"{
      "model":"gpt-test",
      "output":[
        {
          "type":"message",
          "role":"assistant",
          "content":[
            {"type":"output_text","text":"{\"decision\":\"wait\"}"}
          ]
        }
      ]
    }"#;

    let result = completion_result_from_raw_response_json(raw).expect("result");
    assert_eq!(result.model.as_deref(), Some("gpt-test"));
    assert_eq!(result.output, "{\"decision\":\"wait\"}");
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
fn llm_agent_consumes_multi_json_output_in_single_turn() {
    let calls = Arc::new(AtomicUsize::new(0));
    let client = CountingSequenceMockClient::new(
        vec![
            r#"{"type":"module_call","module":"agent.modules.list","args":{}}

---

{"decision":"move_agent","to":"loc-2"}"#
                .to_string(),
        ],
        Arc::clone(&calls),
    );
    let mut behavior = LlmAgentBehavior::new("agent-1", base_config(), client);

    let decision = behavior.decide(&make_observation());
    assert!(matches!(
        decision,
        AgentDecision::Act(Action::MoveAgent { .. })
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    let trace = behavior.take_decision_trace().expect("trace exists");
    assert!(trace.parse_error.is_none());
    assert_eq!(trace.llm_effect_intents.len(), 1);
    assert_eq!(trace.llm_effect_receipts.len(), 1);
}

#[test]
fn llm_agent_skips_extra_module_calls_and_consumes_later_terminal_decision() {
    let calls = Arc::new(AtomicUsize::new(0));
    let client = CountingSequenceMockClient::new(
        vec![
            r#"{"type":"module_call","module":"agent.modules.list","args":{}}

---

{"type":"module_call","module":"environment.current_observation","args":{}}

---

{"type":"module_call","module":"agent.modules.list","args":{}}

---

{"decision":"wait"}"#
                .to_string(),
        ],
        Arc::clone(&calls),
    );

    let mut config = base_config();
    config.max_module_calls = 2;
    let mut behavior = LlmAgentBehavior::new("agent-1", config, client);

    let decision = behavior.decide(&make_observation());
    assert_eq!(decision, AgentDecision::Wait);
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    let trace = behavior.take_decision_trace().expect("trace exists");
    assert!(trace.parse_error.is_none());
    assert_eq!(trace.llm_effect_intents.len(), 2);
    assert_eq!(trace.llm_effect_receipts.len(), 2);
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
fn llm_agent_force_replan_plan_can_finalize_without_module_call_when_missing_is_empty() {
    let client = SequenceMockClient::new(vec![
        r#"{"decision":"harvest_radiation","max_amount":5}"#.to_string(),
        r#"{"decision":"harvest_radiation","max_amount":5}"#.to_string(),
        r#"{"type":"plan","missing":[],"next":"module_call"}"#.to_string(),
        r#"{"decision":"move_agent","to":"loc-2"}"#.to_string(),
    ]);

    let mut config = base_config();
    config.max_decision_steps = 5;
    config.max_repair_rounds = 1;
    config.force_replan_after_same_action = 2;
    config.execute_until_auto_reenter_ticks = 0;

    let mut behavior = LlmAgentBehavior::new("agent-1", config, client);

    let mut observation = make_observation();
    observation.time = 70;
    let first = behavior.decide(&observation);
    assert!(matches!(
        first,
        AgentDecision::Act(Action::HarvestRadiation { .. })
    ));

    observation.time = 71;
    let second = behavior.decide(&observation);
    assert!(matches!(
        second,
        AgentDecision::Act(Action::HarvestRadiation { .. })
    ));

    observation.time = 72;
    let third = behavior.decide(&observation);
    assert!(matches!(
        third,
        AgentDecision::Act(Action::MoveAgent { .. })
    ));

    let trace = behavior.take_decision_trace().expect("trace exists");
    assert!(trace.parse_error.is_none());
    assert!(trace.llm_effect_intents.is_empty());
    assert!(trace
        .llm_step_trace
        .iter()
        .any(|step| step.output_summary.contains("plan missing=-")));
    assert!(!trace
        .llm_step_trace
        .iter()
        .any(|step| step.step_type == "module_call"));
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
    assert_eq!(trace.llm_effect_intents.len(), 1);
    assert_eq!(trace.llm_effect_receipts.len(), 1);
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
fn llm_agent_user_prompt_contains_step_context_metadata() {
    let behavior = LlmAgentBehavior::new("agent-1", base_config(), MockClient::default());
    let prompt = behavior.user_prompt(&make_observation(), &[], 2, 5);
    assert!(prompt.contains("step_index: 2"));
    assert!(prompt.contains("max_steps: 5"));
    assert!(prompt.contains("module_calls_used: 0"));
    assert!(prompt.contains("module_calls_max: 3"));
    assert!(prompt.contains("harvest_radiation"));
    assert!(prompt.contains("max_amount"));
    assert!(prompt.contains(format!("不超过 {}", DEFAULT_LLM_HARVEST_MAX_AMOUNT_CAP).as_str()));
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
    assert!(user_prompt.contains("move_agent.to 不能是当前所在位置"));
}

#[test]
fn llm_parse_turn_responses_extracts_multiple_json_blocks() {
    let parsed = super::decision_flow::parse_llm_turn_responses(
        r#"{"type":"module_call","module":"agent.modules.list","args":{}}

---

{"type":"decision_draft","decision":{"decision":"wait"},"need_verify":false}

---

{"decision":"wait"}"#,
        "agent-1",
    );

    assert_eq!(parsed.len(), 3);
    assert!(matches!(
        parsed[0],
        super::decision_flow::ParsedLlmTurn::ModuleCall(_)
    ));
    assert!(matches!(
        parsed[1],
        super::decision_flow::ParsedLlmTurn::DecisionDraft(_)
    ));
    assert!(matches!(
        parsed[2],
        super::decision_flow::ParsedLlmTurn::Decision(AgentDecision::Wait, _)
    ));
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
