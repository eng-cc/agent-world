use super::*;
use crate::geometry::GeoPos;
use crate::simulator::{
    Action, LlmChatRole, Observation, ObservedAgent, ObservedLocation, PowerOrderSide,
    RejectReason, ResourceKind, ResourceOwner, ResourceStock, SocialAdjudicationDecision,
    SocialStake, WorldEvent, WorldEventKind,
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

fn completion_turn_from_value(value: serde_json::Value) -> LlmCompletionTurn {
    if value
        .get("type")
        .and_then(|value| value.as_str())
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("module_call"))
    {
        return LlmCompletionTurn::ModuleCall {
            module: value
                .get("module")
                .and_then(|value| value.as_str())
                .unwrap_or_default()
                .to_string(),
            args: value
                .get("args")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({})),
        };
    }

    LlmCompletionTurn::Decision { payload: value }
}

fn next_json_start(raw: &str, from: usize) -> Option<usize> {
    raw.get(from..)?
        .char_indices()
        .find_map(|(offset, ch)| match ch {
            '{' | '[' => Some(from + offset),
            _ => None,
        })
}

fn extract_json_block_from(raw: &str, start: usize) -> Option<(usize, usize)> {
    let open_char = raw.get(start..)?.chars().next()?;
    if open_char != '{' && open_char != '[' {
        return None;
    }
    let close_char = if open_char == '{' { '}' } else { ']' };

    let mut depth: u32 = 0;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, ch) in raw[start..].char_indices() {
        let index = start + offset;
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            c if c == open_char => depth = depth.saturating_add(1),
            c if c == close_char => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some((start, index));
                }
            }
            _ => {}
        }
    }

    None
}

fn extract_json_blocks(raw: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut cursor = 0_usize;

    while let Some(start) = next_json_start(raw, cursor) {
        let Some((_, end)) = extract_json_block_from(raw, start) else {
            break;
        };
        if let Some(block) = raw.get(start..=end) {
            blocks.push(block);
        }
        cursor = end.saturating_add(1);
    }

    blocks
}

pub(super) fn completion_turns_from_output(output: &str) -> Vec<LlmCompletionTurn> {
    let blocks = extract_json_blocks(output);
    if blocks.is_empty() {
        return Vec::new();
    }
    blocks
        .into_iter()
        .filter_map(|block| serde_json::from_str::<serde_json::Value>(block).ok())
        .map(completion_turn_from_value)
        .collect()
}

impl LlmCompletionClient for MockClient {
    fn complete(
        &self,
        request: &LlmCompletionRequest,
    ) -> Result<LlmCompletionResult, LlmClientError> {
        if let Some(err) = &self.err {
            return Err(err.clone());
        }
        let output = self
            .output
            .clone()
            .unwrap_or_else(|| "{\"decision\":\"wait\"}".to_string());
        Ok(LlmCompletionResult {
            turns: completion_turns_from_output(output.as_str()),
            output,
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
            turns: completion_turns_from_output(output.as_str()),
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
            turns: completion_turns_from_output(output.as_str()),
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
            turns: completion_turns_from_output(output.as_str()),
            output,
            model: Some("gpt-count".to_string()),
            prompt_tokens: Some(12),
            completion_tokens: Some(4),
            total_tokens: Some(16),
        })
    }
}

fn make_observation() -> Observation {
    let mut self_resources = ResourceStock::default();
    self_resources
        .add(ResourceKind::Electricity, 30)
        .expect("seed electricity");
    self_resources
        .add(ResourceKind::Data, 0)
        .expect("seed data");
    self_resources
        .add(ResourceKind::Data, 12)
        .expect("seed data");

    Observation {
        time: 7,
        agent_id: "agent-1".to_string(),
        pos: GeoPos {
            x_cm: 0.0,
            y_cm: 0.0,
            z_cm: 0.0,
        },
        self_resources,
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
        module_lifecycle: Default::default(),
        module_market: Default::default(),
        power_market: Default::default(),
        social_state: Default::default(),
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
        llm_debug_mode: DEFAULT_LLM_DEBUG_MODE,
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
    assert_eq!(config.llm_debug_mode, DEFAULT_LLM_DEBUG_MODE);
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
fn llm_config_reads_debug_mode_from_env() {
    let mut vars = BTreeMap::new();
    vars.insert(ENV_LLM_MODEL.to_string(), "gpt-4o-mini".to_string());
    vars.insert(
        ENV_LLM_BASE_URL.to_string(),
        "https://api.example.com/v1".to_string(),
    );
    vars.insert(ENV_LLM_API_KEY.to_string(), "secret".to_string());
    vars.insert(ENV_LLM_DEBUG_MODE.to_string(), "true".to_string());

    let config = LlmAgentConfig::from_env_with(|key| vars.get(key).cloned(), "").unwrap();
    assert!(config.llm_debug_mode);
}

#[test]
fn llm_config_rejects_invalid_debug_mode() {
    let mut vars = BTreeMap::new();
    vars.insert(ENV_LLM_MODEL.to_string(), "gpt-4o-mini".to_string());
    vars.insert(
        ENV_LLM_BASE_URL.to_string(),
        "https://api.example.com/v1".to_string(),
    );
    vars.insert(ENV_LLM_API_KEY.to_string(), "secret".to_string());
    vars.insert(ENV_LLM_DEBUG_MODE.to_string(), "maybe".to_string());

    let err = LlmAgentConfig::from_env_with(|key| vars.get(key).cloned(), "").unwrap_err();
    assert!(matches!(err, LlmConfigError::InvalidDebugMode { .. }));
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
    assert_eq!(config.llm_debug_mode, DEFAULT_LLM_DEBUG_MODE);
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
    assert_eq!(tools.len(), 10);

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
            OPENAI_TOOL_WORLD_RULES_GUIDE.to_string(),
            OPENAI_TOOL_MODULE_LIFECYCLE_STATUS.to_string(),
            OPENAI_TOOL_POWER_ORDER_BOOK_STATUS.to_string(),
            OPENAI_TOOL_MODULE_MARKET_STATUS.to_string(),
            OPENAI_TOOL_SOCIAL_STATE_STATUS.to_string(),
            OPENAI_TOOL_AGENT_SUBMIT_DECISION.to_string(),
        ]
    );
}

#[test]
fn responses_tools_register_debug_grant_tool_in_debug_mode_only() {
    let normal = responses_tools_with_debug_mode(false);
    assert_eq!(normal.len(), 10);
    let normal_names = normal
        .into_iter()
        .filter_map(|tool| match tool {
            Tool::Function(function_tool) => Some(function_tool.name),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(!normal_names.contains(&OPENAI_TOOL_AGENT_DEBUG_GRANT_RESOURCE.to_string()));

    let debug = responses_tools_with_debug_mode(true);
    assert_eq!(debug.len(), 11);
    let debug_names = debug
        .into_iter()
        .filter_map(|tool| match tool {
            Tool::Function(function_tool) => Some(function_tool.name),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(debug_names.contains(&OPENAI_TOOL_AGENT_DEBUG_GRANT_RESOURCE.to_string()));
}

#[test]
fn response_function_call_maps_to_typed_module_call_turn() {
    let output_item = OutputItem::FunctionCall(async_openai::types::responses::FunctionToolCall {
        arguments: "{\"limit\":5}".to_string(),
        call_id: "call_1".to_string(),
        name: OPENAI_TOOL_MEMORY_SHORT_TERM_RECENT.to_string(),
        id: None,
        status: None,
    });

    let turn = output_item_to_completion_turn(&output_item).expect("module_call turn");
    match turn {
        LlmCompletionTurn::ModuleCall { module, args } => {
            assert_eq!(module, "memory.short_term.recent");
            assert_eq!(args.get("limit").and_then(|v| v.as_i64()), Some(5));
        }
        other => panic!("expected module_call turn, got {other:?}"),
    }
}

#[test]
fn response_function_call_maps_module_lifecycle_status_tool_name() {
    let output_item = OutputItem::FunctionCall(async_openai::types::responses::FunctionToolCall {
        arguments: "{}".to_string(),
        call_id: "call_lifecycle".to_string(),
        name: OPENAI_TOOL_MODULE_LIFECYCLE_STATUS.to_string(),
        id: None,
        status: None,
    });

    let turn = output_item_to_completion_turn(&output_item).expect("module_call turn");
    match turn {
        LlmCompletionTurn::ModuleCall { module, args } => {
            assert_eq!(module, "module.lifecycle.status");
            assert_eq!(args, serde_json::json!({}));
        }
        other => panic!("expected module_call turn, got {other:?}"),
    }
}

#[test]
fn response_function_call_maps_world_rules_guide_tool_name() {
    let output_item = OutputItem::FunctionCall(async_openai::types::responses::FunctionToolCall {
        arguments: "{\"topic\":\"industry\"}".to_string(),
        call_id: "call_world_rules".to_string(),
        name: OPENAI_TOOL_WORLD_RULES_GUIDE.to_string(),
        id: None,
        status: None,
    });

    let turn = output_item_to_completion_turn(&output_item).expect("module_call turn");
    match turn {
        LlmCompletionTurn::ModuleCall { module, args } => {
            assert_eq!(module, "world.rules.guide");
            assert_eq!(
                args.get("topic").and_then(|value| value.as_str()),
                Some("industry")
            );
        }
        other => panic!("expected module_call turn, got {other:?}"),
    }
}

#[test]
fn response_function_call_maps_power_order_book_status_tool_name() {
    let output_item = OutputItem::FunctionCall(async_openai::types::responses::FunctionToolCall {
        arguments: "{\"limit_orders\":12}".to_string(),
        call_id: "call_order_book".to_string(),
        name: OPENAI_TOOL_POWER_ORDER_BOOK_STATUS.to_string(),
        id: None,
        status: None,
    });

    let turn = output_item_to_completion_turn(&output_item).expect("module_call turn");
    match turn {
        LlmCompletionTurn::ModuleCall { module, args } => {
            assert_eq!(module, "power.order_book.status");
            assert_eq!(
                args.get("limit_orders").and_then(|value| value.as_i64()),
                Some(12)
            );
        }
        other => panic!("expected module_call turn, got {other:?}"),
    }
}

#[test]
fn response_function_call_maps_module_market_status_tool_name() {
    let output_item = OutputItem::FunctionCall(async_openai::types::responses::FunctionToolCall {
        arguments: "{\"wasm_hash\":\"hash-1\",\"limit_listings\":6}".to_string(),
        call_id: "call_module_market".to_string(),
        name: OPENAI_TOOL_MODULE_MARKET_STATUS.to_string(),
        id: None,
        status: None,
    });

    let turn = output_item_to_completion_turn(&output_item).expect("module_call turn");
    match turn {
        LlmCompletionTurn::ModuleCall { module, args } => {
            assert_eq!(module, "module.market.status");
            assert_eq!(
                args.get("wasm_hash").and_then(|value| value.as_str()),
                Some("hash-1")
            );
            assert_eq!(
                args.get("limit_listings").and_then(|value| value.as_i64()),
                Some(6)
            );
        }
        other => panic!("expected module_call turn, got {other:?}"),
    }
}

#[test]
fn response_function_call_maps_social_state_status_tool_name() {
    let output_item = OutputItem::FunctionCall(async_openai::types::responses::FunctionToolCall {
        arguments: "{\"include_inactive\":false,\"limit_facts\":5}".to_string(),
        call_id: "call_social_state".to_string(),
        name: OPENAI_TOOL_SOCIAL_STATE_STATUS.to_string(),
        id: None,
        status: None,
    });

    let turn = output_item_to_completion_turn(&output_item).expect("module_call turn");
    match turn {
        LlmCompletionTurn::ModuleCall { module, args } => {
            assert_eq!(module, "social.state.status");
            assert_eq!(
                args.get("include_inactive")
                    .and_then(|value| value.as_bool()),
                Some(false)
            );
            assert_eq!(
                args.get("limit_facts").and_then(|value| value.as_i64()),
                Some(5)
            );
        }
        other => panic!("expected module_call turn, got {other:?}"),
    }
}

#[test]
fn responses_tools_module_lifecycle_schema_declares_filter_and_limits() {
    let lifecycle = responses_tools()
        .into_iter()
        .find_map(|tool| match tool {
            Tool::Function(function_tool)
                if function_tool.name == OPENAI_TOOL_MODULE_LIFECYCLE_STATUS =>
            {
                Some(function_tool)
            }
            _ => None,
        })
        .expect("module lifecycle tool exists");
    let parameters = lifecycle.parameters.expect("module lifecycle parameters");
    let properties = parameters
        .get("properties")
        .and_then(|value| value.as_object())
        .expect("module lifecycle properties");
    assert!(properties.contains_key("module_id"));
    assert!(properties.contains_key("limit_artifacts"));
    assert!(properties.contains_key("limit_installed"));
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

    let turn = output_item_to_completion_turn(&output_item).expect("module_call turn");
    match turn {
        LlmCompletionTurn::ModuleCall { args, .. } => {
            assert_eq!(
                args.get("_raw").and_then(|value| value.as_str()),
                Some("not-json")
            );
        }
        other => panic!("expected module_call turn, got {other:?}"),
    }
}

#[test]
fn response_function_call_maps_decision_tool_to_typed_decision_turn() {
    let output_item = OutputItem::FunctionCall(async_openai::types::responses::FunctionToolCall {
        arguments: "{\"decision\":\"wait_ticks\",\"ticks\":2}".to_string(),
        call_id: "call_decision".to_string(),
        name: OPENAI_TOOL_AGENT_SUBMIT_DECISION.to_string(),
        id: None,
        status: None,
    });

    let turn = output_item_to_completion_turn(&output_item).expect("decision turn");
    match turn {
        LlmCompletionTurn::Decision { payload } => {
            assert_eq!(
                payload.get("decision").and_then(|value| value.as_str()),
                Some("wait_ticks")
            );
            assert_eq!(
                payload.get("ticks").and_then(|value| value.as_i64()),
                Some(2)
            );
        }
        other => panic!("expected decision turn, got {other:?}"),
    }
}

#[test]
fn response_function_call_maps_debug_grant_tool_to_typed_decision_turn() {
    let output_item = OutputItem::FunctionCall(async_openai::types::responses::FunctionToolCall {
        arguments: "{\"owner\":\"self\",\"kind\":\"data\",\"amount\":3}".to_string(),
        call_id: "call_debug".to_string(),
        name: OPENAI_TOOL_AGENT_DEBUG_GRANT_RESOURCE.to_string(),
        id: None,
        status: None,
    });

    let turn = output_item_to_completion_turn(&output_item).expect("decision turn");
    match turn {
        LlmCompletionTurn::Decision { payload } => {
            assert_eq!(
                payload.get("decision").and_then(|value| value.as_str()),
                Some("debug_grant_resource")
            );
            assert_eq!(
                payload.get("kind").and_then(|value| value.as_str()),
                Some("data")
            );
            assert_eq!(
                payload.get("amount").and_then(|value| value.as_i64()),
                Some(3)
            );
        }
        other => panic!("expected decision turn, got {other:?}"),
    }
}
