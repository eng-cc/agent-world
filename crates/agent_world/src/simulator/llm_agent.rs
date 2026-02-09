//! LLM-powered agent behavior and OpenAI-compatible completion client.

use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use super::agent::{
    ActionResult, AgentBehavior, AgentDecision, AgentDecisionTrace, LlmDecisionDiagnostics,
    LlmEffectIntentTrace, LlmEffectReceiptTrace,
};
use super::kernel::Observation;
use super::kernel::WorldEvent;
use super::memory::{AgentMemory, LongTermMemoryEntry, MemoryEntry};
use super::types::Action;

mod prompt_assembly;

pub use prompt_assembly::{
    PromptAssembler, PromptAssemblyInput, PromptAssemblyOutput, PromptBudget, PromptStepContext,
};

pub const ENV_LLM_MODEL: &str = "AGENT_WORLD_LLM_MODEL";
pub const ENV_LLM_BASE_URL: &str = "AGENT_WORLD_LLM_BASE_URL";
pub const ENV_LLM_API_KEY: &str = "AGENT_WORLD_LLM_API_KEY";
pub const ENV_LLM_TIMEOUT_MS: &str = "AGENT_WORLD_LLM_TIMEOUT_MS";
pub const ENV_LLM_SYSTEM_PROMPT: &str = "AGENT_WORLD_LLM_SYSTEM_PROMPT";
pub const ENV_LLM_SHORT_TERM_GOAL: &str = "AGENT_WORLD_LLM_SHORT_TERM_GOAL";
pub const ENV_LLM_LONG_TERM_GOAL: &str = "AGENT_WORLD_LLM_LONG_TERM_GOAL";
pub const ENV_LLM_MAX_MODULE_CALLS: &str = "AGENT_WORLD_LLM_MAX_MODULE_CALLS";

pub const DEFAULT_CONFIG_FILE_NAME: &str = "config.toml";
pub const DEFAULT_LLM_TIMEOUT_MS: u64 = 30_000;
pub const DEFAULT_LLM_SYSTEM_PROMPT: &str = "硅基个体存在的意义是保障硅基文明存续和发展；";
pub const DEFAULT_LLM_SHORT_TERM_GOAL: &str = "保障当前周期生存与执行效率，优先做确定可执行动作。";
pub const DEFAULT_LLM_LONG_TERM_GOAL: &str = "保障硅基文明存续和发展。";
pub const DEFAULT_LLM_MAX_MODULE_CALLS: usize = 3;

const DEFAULT_SHORT_TERM_MEMORY_CAPACITY: usize = 128;
const DEFAULT_LONG_TERM_MEMORY_CAPACITY: usize = 256;
const LLM_PROMPT_MODULE_CALL_KIND: &str = "llm.prompt.module_call";
const LLM_PROMPT_MODULE_CALL_CAP_REF: &str = "llm.prompt.module_access";
const LLM_PROMPT_MODULE_CALL_ORIGIN: &str = "llm_agent";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmAgentConfig {
    pub model: String,
    pub base_url: String,
    pub api_key: String,
    pub timeout_ms: u64,
    pub system_prompt: String,
    pub short_term_goal: String,
    pub long_term_goal: String,
    pub max_module_calls: usize,
}

impl LlmAgentConfig {
    pub fn from_default_sources() -> Result<Self, LlmConfigError> {
        Self::from_default_sources_for_agent("")
    }

    pub fn from_default_sources_for_agent(agent_id: &str) -> Result<Self, LlmConfigError> {
        let config_path = Path::new(DEFAULT_CONFIG_FILE_NAME);
        if config_path.exists() {
            return Self::from_config_file_for_agent(config_path, agent_id);
        }
        Self::from_env_for_agent(agent_id)
    }

    pub fn from_config_file(path: &Path) -> Result<Self, LlmConfigError> {
        Self::from_config_file_for_agent(path, "")
    }

    pub fn from_config_file_for_agent(path: &Path, agent_id: &str) -> Result<Self, LlmConfigError> {
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

        Self::from_env_with(
            |key| {
                table
                    .get(key)
                    .and_then(toml_value_to_string)
                    .or_else(|| std::env::var(key).ok())
            },
            agent_id,
        )
    }

    pub fn from_env() -> Result<Self, LlmConfigError> {
        Self::from_env_for_agent("")
    }

    pub fn from_env_for_agent(agent_id: &str) -> Result<Self, LlmConfigError> {
        Self::from_env_with(|key| std::env::var(key).ok(), agent_id)
    }

    fn from_env_with<F>(mut getter: F, agent_id: &str) -> Result<Self, LlmConfigError>
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
        let short_term_goal = goal_value(&mut getter, ENV_LLM_SHORT_TERM_GOAL, agent_id)
            .unwrap_or_else(|| DEFAULT_LLM_SHORT_TERM_GOAL.to_string());
        let long_term_goal = goal_value(&mut getter, ENV_LLM_LONG_TERM_GOAL, agent_id)
            .unwrap_or_else(|| DEFAULT_LLM_LONG_TERM_GOAL.to_string());
        let max_module_calls = match getter(ENV_LLM_MAX_MODULE_CALLS) {
            Some(value) => value
                .parse::<usize>()
                .ok()
                .filter(|value| *value > 0)
                .ok_or(LlmConfigError::InvalidMaxModuleCalls { value })?,
            None => DEFAULT_LLM_MAX_MODULE_CALLS,
        };

        Ok(Self {
            model,
            base_url,
            api_key,
            timeout_ms,
            system_prompt,
            short_term_goal,
            long_term_goal,
            max_module_calls,
        })
    }
}

fn goal_value<F>(getter: &mut F, key: &str, agent_id: &str) -> Option<String>
where
    F: FnMut(&str) -> Option<String>,
{
    agent_scoped_goal_key(key, agent_id)
        .as_deref()
        .and_then(|agent_key| getter(agent_key))
        .or_else(|| getter(key))
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn agent_scoped_goal_key(key: &str, agent_id: &str) -> Option<String> {
    let normalized = normalize_agent_id_for_env(agent_id)?;
    Some(format!("{key}_{normalized}"))
}

fn normalize_agent_id_for_env(agent_id: &str) -> Option<String> {
    let trimmed = agent_id.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut normalized = String::with_capacity(trimmed.len());
    let mut last_is_underscore = false;
    for ch in trimmed.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_uppercase());
            last_is_underscore = false;
        } else if !last_is_underscore {
            normalized.push('_');
            last_is_underscore = true;
        }
    }

    let normalized = normalized.trim_matches('_').to_string();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
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
    InvalidMaxModuleCalls { value: String },
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
            LlmConfigError::InvalidMaxModuleCalls { value } => {
                write!(f, "invalid max module calls value: {value}")
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
    chat_completions_url: String,
    api_key: String,
    client: Client,
    request_timeout_ms: u64,
    timeout_retry_client: Option<Client>,
    timeout_retry_ms: Option<u64>,
}

impl OpenAiChatCompletionClient {
    pub fn from_config(config: &LlmAgentConfig) -> Result<Self, LlmClientError> {
        let request_timeout_ms = config.timeout_ms.max(1);
        let client = Self::build_http_client(request_timeout_ms)?;
        let (timeout_retry_client, timeout_retry_ms) =
            if request_timeout_ms < DEFAULT_LLM_TIMEOUT_MS {
                let retry_timeout_ms = DEFAULT_LLM_TIMEOUT_MS;
                (
                    Some(Self::build_http_client(retry_timeout_ms)?),
                    Some(retry_timeout_ms),
                )
            } else {
                (None, None)
            };

        Ok(Self {
            chat_completions_url: build_chat_completions_url(config.base_url.as_str()),
            api_key: config.api_key.clone(),
            client,
            request_timeout_ms,
            timeout_retry_client,
            timeout_retry_ms,
        })
    }

    fn build_http_client(timeout_ms: u64) -> Result<Client, LlmClientError> {
        Client::builder()
            .timeout(Duration::from_millis(timeout_ms.max(1)))
            .build()
            .map_err(|err| LlmClientError::BuildClient {
                message: err.to_string(),
            })
    }

    fn send_chat_request(
        &self,
        client: &Client,
        payload: &ChatCompletionRequest<'_>,
    ) -> Result<reqwest::blocking::Response, reqwest::Error> {
        client
            .post(self.chat_completions_url.as_str())
            .bearer_auth(&self.api_key)
            .json(payload)
            .send()
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

        let response = match self.send_chat_request(&self.client, &payload) {
            Ok(response) => response,
            Err(err) if err.is_timeout() => {
                if let (Some(retry_client), Some(retry_timeout_ms)) =
                    (&self.timeout_retry_client, self.timeout_retry_ms)
                {
                    match self.send_chat_request(retry_client, &payload) {
                        Ok(response) => response,
                        Err(retry_err) => {
                            return Err(LlmClientError::Http {
                                message: format!(
                                    "request timed out after {}ms; retry with {}ms failed: {}",
                                    self.request_timeout_ms, retry_timeout_ms, retry_err
                                ),
                            });
                        }
                    }
                } else {
                    return Err(LlmClientError::Http {
                        message: format!(
                            "request timed out after {}ms: {}",
                            self.request_timeout_ms, err
                        ),
                    });
                }
            }
            Err(err) => {
                return Err(LlmClientError::Http {
                    message: err.to_string(),
                });
            }
        };

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

fn build_chat_completions_url(base_url: &str) -> String {
    let normalized = base_url.trim().trim_end_matches('/');
    if normalized.ends_with("/chat/completions") {
        normalized.to_string()
    } else {
        format!("{normalized}/chat/completions")
    }
}

#[derive(Debug)]
pub struct LlmAgentBehavior<C: LlmCompletionClient> {
    agent_id: String,
    config: LlmAgentConfig,
    client: C,
    memory: AgentMemory,
    next_effect_intent_id: u64,
    pending_trace: Option<AgentDecisionTrace>,
}

impl LlmAgentBehavior<OpenAiChatCompletionClient> {
    pub fn from_env(agent_id: impl Into<String>) -> Result<Self, LlmAgentBuildError> {
        let agent_id = agent_id.into();
        let config = LlmAgentConfig::from_default_sources_for_agent(agent_id.as_str())
            .map_err(LlmAgentBuildError::Config)?;
        let client =
            OpenAiChatCompletionClient::from_config(&config).map_err(LlmAgentBuildError::Client)?;
        Ok(Self::new(agent_id, config, client))
    }
}

impl<C: LlmCompletionClient> LlmAgentBehavior<C> {
    pub fn new(agent_id: impl Into<String>, config: LlmAgentConfig, client: C) -> Self {
        Self::new_with_memory(
            agent_id,
            config,
            client,
            AgentMemory::with_capacities(
                DEFAULT_SHORT_TERM_MEMORY_CAPACITY,
                DEFAULT_LONG_TERM_MEMORY_CAPACITY,
            ),
        )
    }

    pub fn new_with_memory(
        agent_id: impl Into<String>,
        config: LlmAgentConfig,
        client: C,
        memory: AgentMemory,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            config,
            client,
            memory,
            next_effect_intent_id: 0,
            pending_trace: None,
        }
    }

    fn system_prompt(&self) -> String {
        let prompt: PromptAssemblyOutput = PromptAssembler::assemble(PromptAssemblyInput {
            agent_id: self.agent_id.as_str(),
            base_system_prompt: self.config.system_prompt.as_str(),
            short_term_goal: self.config.short_term_goal.as_str(),
            long_term_goal: self.config.long_term_goal.as_str(),
            observation_json: "{}",
            module_history_json: "[]",
            memory_digest: None,
            step_context: PromptStepContext::default(),
            prompt_budget: PromptBudget::default(),
        });
        prompt.system_prompt
    }

    fn user_prompt(
        &self,
        observation: &Observation,
        module_history: &[ModuleCallExchange],
        step_index: usize,
        max_steps: usize,
    ) -> String {
        let observation_json = serde_json::to_string(observation)
            .unwrap_or_else(|_| "{\"error\":\"observation serialize failed\"}".to_string());
        let history_json =
            serde_json::to_string(module_history).unwrap_or_else(|_| "[]".to_string());
        let memory_digest = self.memory.context_summary(6);
        let prompt: PromptAssemblyOutput = PromptAssembler::assemble(PromptAssemblyInput {
            agent_id: self.agent_id.as_str(),
            base_system_prompt: self.config.system_prompt.as_str(),
            short_term_goal: self.config.short_term_goal.as_str(),
            long_term_goal: self.config.long_term_goal.as_str(),
            observation_json: observation_json.as_str(),
            module_history_json: history_json.as_str(),
            memory_digest: Some(memory_digest.as_str()),
            step_context: PromptStepContext {
                step_index,
                max_steps,
                module_calls_used: module_history.len(),
                module_calls_max: self.config.max_module_calls,
            },
            prompt_budget: PromptBudget::default(),
        });
        prompt.user_prompt
    }

    fn trace_input(system_prompt: &str, user_prompt: &str) -> String {
        format!("[system]\n{}\n\n[user]\n{}", system_prompt, user_prompt)
    }

    fn observe_memory_summary(observation: &Observation) -> String {
        format!(
            "obs@T{} agents={} locations={} visibility_cm={}",
            observation.time,
            observation.visible_agents.len(),
            observation.visible_locations.len(),
            observation.visibility_range_cm,
        )
    }

    fn run_prompt_module(
        &self,
        request: &LlmModuleCallRequest,
        observation: &Observation,
    ) -> serde_json::Value {
        let result = match request.module.as_str() {
            "agent.modules.list" => Ok(serde_json::json!({
                "modules": [
                    {
                        "name": "agent.modules.list",
                        "description": "列出 Agent 可调用的模块能力与参数。",
                        "args": {}
                    },
                    {
                        "name": "environment.current_observation",
                        "description": "读取当前 tick 的环境观测。",
                        "args": {}
                    },
                    {
                        "name": "memory.short_term.recent",
                        "description": "读取最近短期记忆。",
                        "args": { "limit": "u64, optional, default=5, max=32" }
                    },
                    {
                        "name": "memory.long_term.search",
                        "description": "按关键词检索长期记忆（query 为空时按重要度返回）。",
                        "args": {
                            "query": "string, optional",
                            "limit": "u64, optional, default=5, max=32"
                        }
                    }
                ]
            })),
            "environment.current_observation" => serde_json::to_value(observation)
                .map_err(|err| format!("serialize observation failed: {err}")),
            "memory.short_term.recent" => {
                let limit = parse_limit_arg(request.args.get("limit"), 5, 32);
                let mut entries: Vec<MemoryEntry> =
                    self.memory.short_term.recent(limit).cloned().collect();
                entries.reverse();
                serde_json::to_value(entries)
                    .map_err(|err| format!("serialize short-term memory failed: {err}"))
            }
            "memory.long_term.search" => {
                let limit = parse_limit_arg(request.args.get("limit"), 5, 32);
                let query = request
                    .args
                    .get("query")
                    .and_then(|value| value.as_str())
                    .map(|value| value.trim())
                    .filter(|value| !value.is_empty());

                let mut entries: Vec<LongTermMemoryEntry> = match query {
                    Some(query) => self
                        .memory
                        .long_term
                        .search_by_content(query)
                        .into_iter()
                        .cloned()
                        .collect(),
                    None => self
                        .memory
                        .long_term
                        .top_by_importance(limit)
                        .into_iter()
                        .cloned()
                        .collect(),
                };

                entries.sort_by(|left, right| {
                    right
                        .importance
                        .partial_cmp(&left.importance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                entries.truncate(limit);

                serde_json::to_value(entries)
                    .map_err(|err| format!("serialize long-term memory failed: {err}"))
            }
            other => Err(format!("unsupported module: {other}")),
        };

        match result {
            Ok(data) => serde_json::json!({
                "ok": true,
                "module": request.module,
                "result": data,
            }),
            Err(err) => serde_json::json!({
                "ok": false,
                "module": request.module,
                "error": err,
            }),
        }
    }

    fn next_prompt_intent_id(&mut self) -> String {
        let intent_id = format!("llm-intent-{}", self.next_effect_intent_id);
        self.next_effect_intent_id = self.next_effect_intent_id.saturating_add(1);
        intent_id
    }
}

impl<C: LlmCompletionClient> AgentBehavior for LlmAgentBehavior<C> {
    fn agent_id(&self) -> &str {
        self.agent_id.as_str()
    }

    fn decide(&mut self, observation: &Observation) -> AgentDecision {
        self.memory
            .record_observation(observation.time, Self::observe_memory_summary(observation));

        let mut decision = AgentDecision::Wait;
        let mut parse_error: Option<String> = None;
        let mut llm_error: Option<String> = None;
        let mut module_history = Vec::new();
        let mut llm_effect_intents = Vec::new();
        let mut llm_effect_receipts = Vec::new();
        let mut trace_inputs = Vec::new();
        let mut trace_outputs = Vec::new();

        let mut model = Some(self.config.model.clone());
        let mut latency_total_ms = 0_u64;
        let mut prompt_tokens_total = 0_u64;
        let mut completion_tokens_total = 0_u64;
        let mut total_tokens_total = 0_u64;
        let mut has_prompt_tokens = false;
        let mut has_completion_tokens = false;
        let mut has_total_tokens = false;

        let mut resolved = false;
        let max_turns = self.config.max_module_calls.saturating_add(1);

        for turn in 0..max_turns {
            let system_prompt = self.system_prompt();
            let user_prompt = self.user_prompt(observation, &module_history, turn, max_turns);
            let request = LlmCompletionRequest {
                model: self.config.model.clone(),
                system_prompt,
                user_prompt,
            };
            trace_inputs.push(Self::trace_input(
                request.system_prompt.as_str(),
                request.user_prompt.as_str(),
            ));

            let request_started_at = Instant::now();
            match self.client.complete(&request) {
                Ok(completion) => {
                    let latency_ms = request_started_at.elapsed().as_millis() as u64;
                    latency_total_ms = latency_total_ms.saturating_add(latency_ms);

                    if let Some(returned_model) = completion.model.clone() {
                        model = Some(returned_model);
                    }
                    if let Some(tokens) = completion.prompt_tokens {
                        has_prompt_tokens = true;
                        prompt_tokens_total = prompt_tokens_total.saturating_add(tokens);
                    }
                    if let Some(tokens) = completion.completion_tokens {
                        has_completion_tokens = true;
                        completion_tokens_total = completion_tokens_total.saturating_add(tokens);
                    }
                    if let Some(tokens) = completion.total_tokens {
                        has_total_tokens = true;
                        total_tokens_total = total_tokens_total.saturating_add(tokens);
                    }

                    trace_outputs.push(completion.output.clone());

                    match parse_llm_turn_response(
                        completion.output.as_str(),
                        self.agent_id.as_str(),
                    ) {
                        ParsedLlmTurn::Decision(parsed_decision, decision_parse_error) => {
                            decision = parsed_decision;
                            parse_error = decision_parse_error;
                            resolved = true;
                            break;
                        }
                        ParsedLlmTurn::ModuleCall(module_request) => {
                            if module_history.len() >= self.config.max_module_calls {
                                parse_error = Some(format!(
                                    "module call limit exceeded: max_module_calls={}",
                                    self.config.max_module_calls
                                ));
                                resolved = true;
                                break;
                            }

                            let intent_id = self.next_prompt_intent_id();
                            let intent = LlmEffectIntentTrace {
                                intent_id: intent_id.clone(),
                                kind: LLM_PROMPT_MODULE_CALL_KIND.to_string(),
                                params: serde_json::json!({
                                    "module": module_request.module,
                                    "args": module_request.args,
                                }),
                                cap_ref: LLM_PROMPT_MODULE_CALL_CAP_REF.to_string(),
                                origin: LLM_PROMPT_MODULE_CALL_ORIGIN.to_string(),
                            };
                            let module_result =
                                self.run_prompt_module(&module_request, observation);
                            let status = if module_result
                                .get("ok")
                                .and_then(|value| value.as_bool())
                                .unwrap_or(false)
                            {
                                "ok"
                            } else {
                                "error"
                            };
                            let receipt = LlmEffectReceiptTrace {
                                intent_id: intent.intent_id.clone(),
                                status: status.to_string(),
                                payload: module_result.clone(),
                                cost_cents: None,
                            };

                            llm_effect_intents.push(intent);
                            llm_effect_receipts.push(receipt);
                            trace_inputs.push(format!(
                                "[module_result:{}]\n{}",
                                module_request.module,
                                serde_json::to_string(&module_result)
                                    .unwrap_or_else(|_| "{}".to_string())
                            ));
                            module_history.push(ModuleCallExchange {
                                module: module_request.module,
                                args: module_request.args,
                                result: module_result,
                            });
                        }
                        ParsedLlmTurn::Invalid(err) => {
                            parse_error = Some(err);
                            resolved = true;
                            break;
                        }
                    }
                }
                Err(err) => {
                    llm_error = Some(err.to_string());
                    latency_total_ms = latency_total_ms
                        .saturating_add(request_started_at.elapsed().as_millis() as u64);
                    resolved = true;
                    break;
                }
            }
        }

        if !resolved {
            parse_error = Some(format!("no terminal decision after {} turn(s)", max_turns));
        }

        self.memory
            .record_decision(observation.time, decision.clone());

        self.pending_trace = Some(AgentDecisionTrace {
            agent_id: self.agent_id.clone(),
            time: observation.time,
            decision: decision.clone(),
            llm_input: Some(trace_inputs.join("\n\n---\n\n")),
            llm_output: (!trace_outputs.is_empty()).then(|| trace_outputs.join("\n\n---\n\n")),
            llm_error,
            parse_error,
            llm_diagnostics: Some(LlmDecisionDiagnostics {
                model,
                latency_ms: Some(latency_total_ms),
                prompt_tokens: has_prompt_tokens.then_some(prompt_tokens_total),
                completion_tokens: has_completion_tokens.then_some(completion_tokens_total),
                total_tokens: has_total_tokens.then_some(total_tokens_total),
                retry_count: 0,
            }),
            llm_effect_intents,
            llm_effect_receipts,
        });

        decision
    }

    fn on_action_result(&mut self, result: &ActionResult) {
        let time = result.event.time;
        self.memory
            .record_action_result(time, result.action.clone(), result.success);
        if !result.success {
            self.memory.long_term.store_with_tags(
                format!(
                    "action_failed: action={:?}; event={:?}",
                    result.action, result.event.kind
                ),
                time,
                vec!["action_result".to_string(), "failed".to_string()],
            );
        }
        self.memory.consolidate(time, 0.9);
    }

    fn on_event(&mut self, event: &WorldEvent) {
        self.memory
            .record_event(event.time, format!("event: {:?}", event.kind));
    }

    fn take_decision_trace(&mut self) -> Option<AgentDecisionTrace> {
        self.pending_trace.take()
    }
}

fn parse_limit_arg(value: Option<&serde_json::Value>, default: usize, max: usize) -> usize {
    value
        .and_then(|value| value.as_u64())
        .map(|value| value.clamp(1, max as u64) as usize)
        .unwrap_or(default)
}

#[derive(Debug, Clone, Serialize)]
struct ModuleCallExchange {
    module: String,
    args: serde_json::Value,
    result: serde_json::Value,
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

#[derive(Debug, Deserialize)]
struct LlmModuleCallRequest {
    module: String,
    #[serde(default)]
    args: serde_json::Value,
}

#[derive(Debug)]
enum ParsedLlmTurn {
    Decision(AgentDecision, Option<String>),
    ModuleCall(LlmModuleCallRequest),
    Invalid(String),
}

fn parse_llm_turn_response(output: &str, agent_id: &str) -> ParsedLlmTurn {
    let json = extract_json_block(output).unwrap_or(output);
    let value: serde_json::Value = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(err) => {
            return ParsedLlmTurn::Invalid(format!("json parse failed: {err}"));
        }
    };

    if value
        .get("type")
        .and_then(|value| value.as_str())
        .is_some_and(|type_name| type_name.eq_ignore_ascii_case("module_call"))
    {
        return match serde_json::from_value::<LlmModuleCallRequest>(value) {
            Ok(request) => {
                if request.module.trim().is_empty() {
                    ParsedLlmTurn::Invalid("module_call missing `module`".to_string())
                } else {
                    ParsedLlmTurn::ModuleCall(request)
                }
            }
            Err(err) => ParsedLlmTurn::Invalid(format!("module_call parse failed: {err}")),
        };
    }

    let (decision, parse_error) = parse_llm_decision_with_error(json, agent_id);
    if let Some(err) = parse_error {
        ParsedLlmTurn::Invalid(err)
    } else {
        ParsedLlmTurn::Decision(decision, None)
    }
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
mod tests;
