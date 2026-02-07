//! LLM-powered agent behavior and OpenAI-compatible completion client.

use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use super::agent::{AgentBehavior, AgentDecision, AgentDecisionTrace, LlmDecisionDiagnostics};
use super::kernel::Observation;
use super::types::Action;

pub const ENV_LLM_MODEL: &str = "AGENT_WORLD_LLM_MODEL";
pub const ENV_LLM_BASE_URL: &str = "AGENT_WORLD_LLM_BASE_URL";
pub const ENV_LLM_API_KEY: &str = "AGENT_WORLD_LLM_API_KEY";
pub const ENV_LLM_TIMEOUT_MS: &str = "AGENT_WORLD_LLM_TIMEOUT_MS";
pub const ENV_LLM_SYSTEM_PROMPT: &str = "AGENT_WORLD_LLM_SYSTEM_PROMPT";

pub const DEFAULT_CONFIG_FILE_NAME: &str = "config.toml";
pub const DEFAULT_LLM_TIMEOUT_MS: u64 = 30_000;
pub const DEFAULT_LLM_SYSTEM_PROMPT: &str = "硅基个体存在的意义是保障硅基文明存续和发展；";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmAgentConfig {
    pub model: String,
    pub base_url: String,
    pub api_key: String,
    pub timeout_ms: u64,
    pub system_prompt: String,
}

impl LlmAgentConfig {
    pub fn from_default_sources() -> Result<Self, LlmConfigError> {
        let config_path = Path::new(DEFAULT_CONFIG_FILE_NAME);
        if config_path.exists() {
            return Self::from_config_file(config_path);
        }
        Self::from_env()
    }

    pub fn from_config_file(path: &Path) -> Result<Self, LlmConfigError> {
        let content = fs::read_to_string(path).map_err(|err| LlmConfigError::ReadConfigFile {
            path: path.display().to_string(),
            message: err.to_string(),
        })?;
        let value: toml::Value =
            toml::from_str(&content).map_err(|err| LlmConfigError::ParseConfigFile {
                path: path.display().to_string(),
                message: err.to_string(),
            })?;
        let table = value
            .as_table()
            .ok_or_else(|| LlmConfigError::ParseConfigFile {
                path: path.display().to_string(),
                message: "root is not a TOML table".to_string(),
            })?;

        Self::from_env_with(|key| {
            table
                .get(key)
                .and_then(toml_value_to_string)
                .or_else(|| std::env::var(key).ok())
        })
    }

    pub fn from_env() -> Result<Self, LlmConfigError> {
        Self::from_env_with(|key| std::env::var(key).ok())
    }

    fn from_env_with<F>(mut getter: F) -> Result<Self, LlmConfigError>
    where
        F: FnMut(&str) -> Option<String>,
    {
        let model = required_env(&mut getter, ENV_LLM_MODEL)?;
        let base_url = required_env(&mut getter, ENV_LLM_BASE_URL)?;
        let api_key = required_env(&mut getter, ENV_LLM_API_KEY)?;
        let timeout_ms = match getter(ENV_LLM_TIMEOUT_MS) {
            Some(value) => value
                .parse::<u64>()
                .map_err(|_| LlmConfigError::InvalidTimeout { value })?,
            None => DEFAULT_LLM_TIMEOUT_MS,
        };
        let system_prompt = getter(ENV_LLM_SYSTEM_PROMPT)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_LLM_SYSTEM_PROMPT.to_string());

        Ok(Self {
            model,
            base_url,
            api_key,
            timeout_ms,
            system_prompt,
        })
    }
}

fn toml_value_to_string(value: &toml::Value) -> Option<String> {
    match value {
        toml::Value::String(value) => Some(value.clone()),
        toml::Value::Integer(value) => Some(value.to_string()),
        toml::Value::Float(value) => Some(value.to_string()),
        toml::Value::Boolean(value) => Some(value.to_string()),
        _ => None,
    }
}

fn required_env<F>(getter: &mut F, key: &'static str) -> Result<String, LlmConfigError>
where
    F: FnMut(&str) -> Option<String>,
{
    let value = getter(key).ok_or(LlmConfigError::MissingEnv { key })?;
    if value.trim().is_empty() {
        return Err(LlmConfigError::EmptyEnv { key });
    }
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmConfigError {
    MissingEnv { key: &'static str },
    EmptyEnv { key: &'static str },
    InvalidTimeout { value: String },
    ReadConfigFile { path: String, message: String },
    ParseConfigFile { path: String, message: String },
}

impl fmt::Display for LlmConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmConfigError::MissingEnv { key } => write!(f, "missing env variable: {key}"),
            LlmConfigError::EmptyEnv { key } => write!(f, "empty env variable: {key}"),
            LlmConfigError::InvalidTimeout { value } => {
                write!(f, "invalid timeout value: {value}")
            }
            LlmConfigError::ReadConfigFile { path, message } => {
                write!(f, "read config file failed ({path}): {message}")
            }
            LlmConfigError::ParseConfigFile { path, message } => {
                write!(f, "parse config file failed ({path}): {message}")
            }
        }
    }
}

impl Error for LlmConfigError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmCompletionRequest {
    pub model: String,
    pub system_prompt: String,
    pub user_prompt: String,
}

pub trait LlmCompletionClient {
    fn complete(
        &self,
        request: &LlmCompletionRequest,
    ) -> Result<LlmCompletionResult, LlmClientError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmCompletionResult {
    pub output: String,
    pub model: Option<String>,
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct OpenAiChatCompletionClient {
    base_url: String,
    api_key: String,
    client: Client,
}

impl OpenAiChatCompletionClient {
    pub fn from_config(config: &LlmAgentConfig) -> Result<Self, LlmClientError> {
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms.max(1)))
            .build()
            .map_err(|err| LlmClientError::BuildClient {
                message: err.to_string(),
            })?;

        Ok(Self {
            base_url: config.base_url.trim_end_matches('/').to_string(),
            api_key: config.api_key.clone(),
            client,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmClientError {
    BuildClient { message: String },
    Http { message: String },
    HttpStatus { code: u16, message: String },
    DecodeResponse { message: String },
    EmptyChoice,
}

impl fmt::Display for LlmClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmClientError::BuildClient { message } => write!(f, "client build failed: {message}"),
            LlmClientError::Http { message } => write!(f, "http request failed: {message}"),
            LlmClientError::HttpStatus { code, message } => {
                write!(f, "http status {code}: {message}")
            }
            LlmClientError::DecodeResponse { message } => {
                write!(f, "decode response failed: {message}")
            }
            LlmClientError::EmptyChoice => write!(f, "empty completion choice"),
        }
    }
}

impl Error for LlmClientError {}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: [ChatMessage<'a>; 2],
}

#[derive(Debug, Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    usage: Option<ChatUsage>,
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChatUsage {
    #[serde(default)]
    prompt_tokens: Option<u64>,
    #[serde(default)]
    completion_tokens: Option<u64>,
    #[serde(default)]
    total_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChatChoiceMessage {
    content: String,
}

impl LlmCompletionClient for OpenAiChatCompletionClient {
    fn complete(
        &self,
        request: &LlmCompletionRequest,
    ) -> Result<LlmCompletionResult, LlmClientError> {
        let url = format!("{}/chat/completions", self.base_url);
        let payload = ChatCompletionRequest {
            model: request.model.as_str(),
            messages: [
                ChatMessage {
                    role: "system",
                    content: request.system_prompt.as_str(),
                },
                ChatMessage {
                    role: "user",
                    content: request.user_prompt.as_str(),
                },
            ],
        };

        let response = self
            .client
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .map_err(|err| LlmClientError::Http {
                message: err.to_string(),
            })?;

        let status = response.status();
        if status != StatusCode::OK {
            let message = response.text().unwrap_or_else(|_| "<no body>".to_string());
            return Err(LlmClientError::HttpStatus {
                code: status.as_u16(),
                message,
            });
        }

        let response: ChatCompletionResponse =
            response
                .json()
                .map_err(|err| LlmClientError::DecodeResponse {
                    message: err.to_string(),
                })?;

        let model = response.model;
        let usage = response.usage;
        let first = response
            .choices
            .into_iter()
            .next()
            .ok_or(LlmClientError::EmptyChoice)?;

        Ok(LlmCompletionResult {
            output: first.message.content,
            model,
            prompt_tokens: usage.as_ref().and_then(|usage| usage.prompt_tokens),
            completion_tokens: usage.as_ref().and_then(|usage| usage.completion_tokens),
            total_tokens: usage.as_ref().and_then(|usage| usage.total_tokens),
        })
    }
}

#[derive(Debug)]
pub struct LlmAgentBehavior<C: LlmCompletionClient> {
    agent_id: String,
    config: LlmAgentConfig,
    client: C,
    pending_trace: Option<AgentDecisionTrace>,
}

impl LlmAgentBehavior<OpenAiChatCompletionClient> {
    pub fn from_env(agent_id: impl Into<String>) -> Result<Self, LlmAgentBuildError> {
        let config = LlmAgentConfig::from_default_sources().map_err(LlmAgentBuildError::Config)?;
        let client =
            OpenAiChatCompletionClient::from_config(&config).map_err(LlmAgentBuildError::Client)?;
        Ok(Self::new(agent_id, config, client))
    }
}

impl<C: LlmCompletionClient> LlmAgentBehavior<C> {
    pub fn new(agent_id: impl Into<String>, config: LlmAgentConfig, client: C) -> Self {
        Self {
            agent_id: agent_id.into(),
            config,
            client,
            pending_trace: None,
        }
    }

    fn user_prompt(&self, observation: &Observation) -> String {
        let observation_json = serde_json::to_string(observation)
            .unwrap_or_else(|_| "{\"error\":\"observation serialize failed\"}".to_string());
        format!(
            "你是一个硅基文明 Agent。请根据观测，严格只输出 JSON，不要输出额外文字。\n\
支持格式：\n\
{{\"decision\":\"wait\"}}\n\
{{\"decision\":\"wait_ticks\",\"ticks\":<u64>}}\n\
{{\"decision\":\"move_agent\",\"to\":\"<location_id>\"}}\n\
{{\"decision\":\"harvest_radiation\",\"max_amount\":<i64>}}\n\
当前 agent_id: {}\n\
观测(JSON): {}",
            self.agent_id, observation_json
        )
    }

    fn trace_input(system_prompt: &str, user_prompt: &str) -> String {
        format!("[system]\n{}\n\n[user]\n{}", system_prompt, user_prompt)
    }
}

impl<C: LlmCompletionClient> AgentBehavior for LlmAgentBehavior<C> {
    fn agent_id(&self) -> &str {
        self.agent_id.as_str()
    }

    fn decide(&mut self, observation: &Observation) -> AgentDecision {
        let user_prompt = self.user_prompt(observation);
        let request = LlmCompletionRequest {
            model: self.config.model.clone(),
            system_prompt: self.config.system_prompt.clone(),
            user_prompt,
        };
        let trace_input = Self::trace_input(&request.system_prompt, &request.user_prompt);

        let request_started_at = Instant::now();
        match self.client.complete(&request) {
            Ok(completion) => {
                let latency_ms = request_started_at.elapsed().as_millis() as u64;
                let (decision, parse_error) = parse_llm_decision_with_error(
                    completion.output.as_str(),
                    self.agent_id.as_str(),
                );
                self.pending_trace = Some(AgentDecisionTrace {
                    agent_id: self.agent_id.clone(),
                    time: observation.time,
                    decision: decision.clone(),
                    llm_input: Some(trace_input),
                    llm_output: Some(completion.output),
                    llm_error: None,
                    parse_error,
                    llm_diagnostics: Some(LlmDecisionDiagnostics {
                        model: completion.model.or(Some(request.model.clone())),
                        latency_ms: Some(latency_ms),
                        prompt_tokens: completion.prompt_tokens,
                        completion_tokens: completion.completion_tokens,
                        total_tokens: completion.total_tokens,
                        retry_count: 0,
                    }),
                });
                decision
            }
            Err(err) => {
                let latency_ms = request_started_at.elapsed().as_millis() as u64;
                self.pending_trace = Some(AgentDecisionTrace {
                    agent_id: self.agent_id.clone(),
                    time: observation.time,
                    decision: AgentDecision::Wait,
                    llm_input: Some(trace_input),
                    llm_output: None,
                    llm_error: Some(err.to_string()),
                    parse_error: None,
                    llm_diagnostics: Some(LlmDecisionDiagnostics {
                        model: Some(request.model.clone()),
                        latency_ms: Some(latency_ms),
                        prompt_tokens: None,
                        completion_tokens: None,
                        total_tokens: None,
                        retry_count: 0,
                    }),
                });
                AgentDecision::Wait
            }
        }
    }

    fn take_decision_trace(&mut self) -> Option<AgentDecisionTrace> {
        self.pending_trace.take()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmAgentBuildError {
    Config(LlmConfigError),
    Client(LlmClientError),
}

impl fmt::Display for LlmAgentBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlmAgentBuildError::Config(err) => write!(f, "llm config error: {err}"),
            LlmAgentBuildError::Client(err) => write!(f, "llm client error: {err}"),
        }
    }
}

impl Error for LlmAgentBuildError {}

#[derive(Debug, Deserialize)]
struct LlmDecisionPayload {
    decision: String,
    ticks: Option<u64>,
    to: Option<String>,
    max_amount: Option<i64>,
}

fn parse_llm_decision_with_error(output: &str, agent_id: &str) -> (AgentDecision, Option<String>) {
    let json = extract_json_block(output).unwrap_or(output);
    let parsed = match serde_json::from_str::<LlmDecisionPayload>(json) {
        Ok(value) => value,
        Err(err) => {
            return (
                AgentDecision::Wait,
                Some(format!("json parse failed: {err}")),
            )
        }
    };

    let decision = match parsed.decision.trim().to_ascii_lowercase().as_str() {
        "wait" => AgentDecision::Wait,
        "wait_ticks" => AgentDecision::WaitTicks(parsed.ticks.unwrap_or(1).max(1)),
        "move_agent" => {
            let to = parsed.to.unwrap_or_default();
            if to.trim().is_empty() {
                return (
                    AgentDecision::Wait,
                    Some("move_agent missing `to`".to_string()),
                );
            }
            AgentDecision::Act(Action::MoveAgent {
                agent_id: agent_id.to_string(),
                to,
            })
        }
        "harvest_radiation" => {
            let max_amount = parsed.max_amount.unwrap_or(1);
            if max_amount <= 0 {
                return (
                    AgentDecision::Wait,
                    Some("harvest_radiation requires positive max_amount".to_string()),
                );
            }
            AgentDecision::Act(Action::HarvestRadiation {
                agent_id: agent_id.to_string(),
                max_amount,
            })
        }
        other => {
            return (
                AgentDecision::Wait,
                Some(format!("unsupported decision: {other}")),
            )
        }
    };

    (decision, None)
}

fn extract_json_block(raw: &str) -> Option<&str> {
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    if end < start {
        return None;
    }
    raw.get(start..=end)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::GeoPos;
    use crate::simulator::{Observation, ObservedAgent, ObservedLocation};
    use std::collections::BTreeMap;
    use std::path::Path;
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

        let config = LlmAgentConfig::from_env_with(|key| vars.get(key).cloned()).unwrap();
        assert_eq!(config.system_prompt, DEFAULT_LLM_SYSTEM_PROMPT);
        assert_eq!(config.timeout_ms, DEFAULT_LLM_TIMEOUT_MS);
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

        let config = LlmAgentConfig::from_env_with(|key| vars.get(key).cloned()).unwrap();
        assert_eq!(config.system_prompt, "自定义 system prompt");
        assert_eq!(config.timeout_ms, 2000);
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
}
