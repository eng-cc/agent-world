//! LLM-powered agent behavior and OpenAI-compatible completion client.

use async_openai::config::OpenAIConfig;
use async_openai::error::OpenAIError;
use async_openai::types::responses::{
    CreateResponse, CreateResponseArgs, FunctionTool, InputParam, OutputItem, Response, Tool,
    ToolChoiceOptions, ToolChoiceParam,
};
use async_openai::Client as AsyncOpenAiClient;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;

use super::agent::{
    ActionResult, AgentBehavior, AgentDecision, AgentDecisionTrace, LlmChatMessageTrace,
    LlmChatRole, LlmDecisionDiagnostics, LlmEffectIntentTrace, LlmEffectReceiptTrace,
    LlmPromptSectionTrace, LlmStepTrace,
};
use super::kernel::{Observation, RejectReason, WorldEvent, WorldEventKind};
use super::memory::{AgentMemory, LongTermMemoryEntry, MemoryEntry};
use super::types::{Action, ResourceKind, ResourceOwner};

mod behavior_loop;
mod config_helpers;
mod decision_flow;
mod execution_controls;
mod memory_selector;
mod openai_payload;
mod prompt_assembly;

pub use memory_selector::{MemorySelector, MemorySelectorConfig};
pub use prompt_assembly::{
    PromptAssembler, PromptAssemblyInput, PromptAssemblyOutput, PromptBudget, PromptStepContext,
};

use decision_flow::{
    parse_limit_arg, parse_llm_turn_payloads_with_debug_mode, prompt_section_kind_name,
    prompt_section_priority_name, summarize_trace_text, DecisionRewriteReceipt,
    ExecuteUntilDirective, LlmModuleCallRequest, ModuleCallExchange, ParsedLlmTurn,
};
use execution_controls::{ActionReplanGuardState, ActiveExecuteUntil};

use config_helpers::{
    goal_value, parse_non_negative_usize, parse_positive_i64, parse_positive_usize, required_env,
    toml_value_to_string,
};
use openai_payload::{
    build_responses_request_payload, completion_result_from_sdk_response,
    normalize_openai_api_base_url,
};
#[cfg(test)]
use openai_payload::{
    output_item_to_completion_turn, responses_tools, responses_tools_with_debug_mode,
};

pub const ENV_LLM_MODEL: &str = "AGENT_WORLD_LLM_MODEL";
pub const ENV_LLM_BASE_URL: &str = "AGENT_WORLD_LLM_BASE_URL";
pub const ENV_LLM_API_KEY: &str = "AGENT_WORLD_LLM_API_KEY";
pub const ENV_LLM_TIMEOUT_MS: &str = "AGENT_WORLD_LLM_TIMEOUT_MS";
pub const ENV_LLM_SYSTEM_PROMPT: &str = "AGENT_WORLD_LLM_SYSTEM_PROMPT";
pub const ENV_LLM_SHORT_TERM_GOAL: &str = "AGENT_WORLD_LLM_SHORT_TERM_GOAL";
pub const ENV_LLM_LONG_TERM_GOAL: &str = "AGENT_WORLD_LLM_LONG_TERM_GOAL";
pub const ENV_LLM_MAX_MODULE_CALLS: &str = "AGENT_WORLD_LLM_MAX_MODULE_CALLS";
pub const ENV_LLM_MAX_DECISION_STEPS: &str = "AGENT_WORLD_LLM_MAX_DECISION_STEPS";
pub const ENV_LLM_MAX_REPAIR_ROUNDS: &str = "AGENT_WORLD_LLM_MAX_REPAIR_ROUNDS";
pub const ENV_LLM_PROMPT_MAX_HISTORY_ITEMS: &str = "AGENT_WORLD_LLM_PROMPT_MAX_HISTORY_ITEMS";
pub const ENV_LLM_PROMPT_PROFILE: &str = "AGENT_WORLD_LLM_PROMPT_PROFILE";
pub const ENV_LLM_FORCE_REPLAN_AFTER_SAME_ACTION: &str =
    "AGENT_WORLD_LLM_FORCE_REPLAN_AFTER_SAME_ACTION";
pub const ENV_LLM_HARVEST_MAX_AMOUNT_CAP: &str = "AGENT_WORLD_LLM_HARVEST_MAX_AMOUNT_CAP";
pub const ENV_LLM_EXECUTE_UNTIL_AUTO_REENTER_TICKS: &str =
    "AGENT_WORLD_LLM_EXECUTE_UNTIL_AUTO_REENTER_TICKS";
pub const ENV_LLM_DEBUG_MODE: &str = "AGENT_WORLD_LLM_DEBUG_MODE";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmPromptProfile {
    Compact,
    Balanced,
}

impl LlmPromptProfile {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "compact" => Some(Self::Compact),
            "balanced" => Some(Self::Balanced),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Balanced => "balanced",
        }
    }

    pub fn prompt_budget(self) -> PromptBudget {
        match self {
            Self::Compact => PromptBudget {
                context_window_tokens: 4_096,
                reserved_output_tokens: 768,
                safety_margin_tokens: 352,
            },
            Self::Balanced => PromptBudget {
                context_window_tokens: 4_608,
                reserved_output_tokens: 896,
                safety_margin_tokens: 480,
            },
        }
    }

    pub fn memory_selector_config(self) -> MemorySelectorConfig {
        match self {
            Self::Compact => MemorySelectorConfig {
                short_term_candidate_limit: 8,
                long_term_candidate_limit: 12,
                short_term_top_k: 3,
                long_term_top_k: 4,
            },
            Self::Balanced => MemorySelectorConfig {
                short_term_candidate_limit: 8,
                long_term_candidate_limit: 12,
                short_term_top_k: 2,
                long_term_top_k: 3,
            },
        }
    }
}

pub const DEFAULT_CONFIG_FILE_NAME: &str = "config.toml";
pub const DEFAULT_LLM_TIMEOUT_MS: u64 = 180_000;
pub const DEFAULT_LLM_SYSTEM_PROMPT: &str = "硅基个体存在的意义是保障硅基文明存续和发展；";
pub const DEFAULT_LLM_SHORT_TERM_GOAL: &str = "保障当前周期生存与执行效率，优先做确定可执行动作。";
pub const DEFAULT_LLM_LONG_TERM_GOAL: &str = "保障硅基文明存续和发展。";
pub const DEFAULT_LLM_MAX_MODULE_CALLS: usize = 3;
pub const DEFAULT_LLM_MAX_DECISION_STEPS: usize = 4;
pub const DEFAULT_LLM_MAX_REPAIR_ROUNDS: usize = 1;
pub const DEFAULT_LLM_PROMPT_MAX_HISTORY_ITEMS: usize = 4;
pub const DEFAULT_LLM_PROMPT_PROFILE: LlmPromptProfile = LlmPromptProfile::Balanced;
pub const DEFAULT_LLM_FORCE_REPLAN_AFTER_SAME_ACTION: usize = 4;
pub const DEFAULT_LLM_HARVEST_MAX_AMOUNT_CAP: i64 = 100;
pub const DEFAULT_LLM_EXECUTE_UNTIL_AUTO_REENTER_TICKS: usize = 4;
pub const DEFAULT_LLM_DEBUG_MODE: bool = false;
pub const DEFAULT_LLM_HARVEST_EXECUTE_UNTIL_MAX_TICKS: u64 = 3;
const DEFAULT_RECIPE_HARDWARE_COST_PER_BATCH: i64 = 2;
const DEFAULT_REFINE_RECOVERY_MASS_G_PER_HARDWARE: i64 = 1_000;
const DEFAULT_REFINE_ELECTRICITY_COST_PER_KG: i64 = 2;
const DEFAULT_MINE_COMPOUND_MAX_PER_ACTION_G: i64 = 5_000;
const DEFAULT_MINE_ELECTRICITY_COST_PER_KG: i64 = 1;
const DEFAULT_MAX_MOVE_DISTANCE_CM_PER_TICK: i64 = 1_000_000;
const TRACKED_RECIPE_IDS: [&str; 3] = [
    "recipe.assembler.control_chip",
    "recipe.assembler.motor_mk1",
    "recipe.assembler.logistics_drone",
];

const DEFAULT_SHORT_TERM_MEMORY_CAPACITY: usize = 128;
const DEFAULT_LONG_TERM_MEMORY_CAPACITY: usize = 256;
const LLM_PROMPT_MODULE_CALL_KIND: &str = "llm.prompt.module_call";
const LLM_PROMPT_MODULE_CALL_CAP_REF: &str = "llm.prompt.module_access";
const LLM_PROMPT_MODULE_CALL_ORIGIN: &str = "llm_agent";
const PROMPT_MODULE_RESULT_MAX_CHARS: usize = 520;
const PROMPT_MODULE_ARGS_MAX_CHARS: usize = 192;
const PROMPT_MEMORY_DIGEST_MAX_CHARS: usize = 360;
const PROMPT_CONVERSATION_ITEM_MAX_CHARS: usize = 320;
const PROMPT_CONVERSATION_MAX_ITEMS: usize = 12;
const PROMPT_OBSERVATION_VISIBLE_AGENTS_MAX: usize = 5;
const PROMPT_OBSERVATION_VISIBLE_LOCATIONS_MAX: usize = 5;
const CONVERSATION_HISTORY_MAX_ITEMS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
struct PromptLastActionSummary {
    kind: String,
    success: bool,
    reject_reason: Option<String>,
    decision_rewrite: Option<DecisionRewriteReceipt>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct RecipeCoverageProgress {
    completed: BTreeSet<String>,
}

impl RecipeCoverageProgress {
    fn is_tracked(recipe_id: &str) -> bool {
        TRACKED_RECIPE_IDS
            .iter()
            .any(|candidate| candidate == &recipe_id.trim())
    }

    fn mark_completed(&mut self, recipe_id: &str) {
        let normalized = recipe_id.trim();
        if Self::is_tracked(normalized) {
            self.completed.insert(normalized.to_string());
        }
    }

    fn is_completed(&self, recipe_id: &str) -> bool {
        self.completed.contains(recipe_id.trim())
    }

    fn missing_recipe_ids(&self) -> Vec<String> {
        TRACKED_RECIPE_IDS
            .iter()
            .filter(|recipe_id| !self.completed.contains(**recipe_id))
            .map(|recipe_id| (*recipe_id).to_string())
            .collect()
    }

    fn next_uncovered_recipe_excluding(&self, current_recipe_id: &str) -> Option<String> {
        let current_recipe_id = current_recipe_id.trim();
        self.missing_recipe_ids()
            .into_iter()
            .find(|recipe_id| recipe_id.as_str() != current_recipe_id)
    }

    fn summary_json(&self) -> serde_json::Value {
        let completed = TRACKED_RECIPE_IDS
            .iter()
            .filter(|recipe_id| self.completed.contains(**recipe_id))
            .map(|recipe_id| (*recipe_id).to_string())
            .collect::<Vec<_>>();
        let missing = self.missing_recipe_ids();
        serde_json::json!({
            "tracked_total": TRACKED_RECIPE_IDS.len(),
            "completed": completed,
            "missing": missing,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LlmPromptOverrides {
    pub system_prompt: Option<String>,
    pub short_term_goal: Option<String>,
    pub long_term_goal: Option<String>,
}

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
    pub max_decision_steps: usize,
    pub max_repair_rounds: usize,
    pub prompt_max_history_items: usize,
    pub prompt_profile: LlmPromptProfile,
    pub force_replan_after_same_action: usize,
    pub harvest_max_amount_cap: i64,
    pub execute_until_auto_reenter_ticks: usize,
    pub llm_debug_mode: bool,
}

fn parse_debug_mode_flag(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
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
        let max_module_calls = parse_positive_usize(
            &mut getter,
            ENV_LLM_MAX_MODULE_CALLS,
            DEFAULT_LLM_MAX_MODULE_CALLS,
            |value| LlmConfigError::InvalidMaxModuleCalls { value },
        )?;
        let max_decision_steps = parse_positive_usize(
            &mut getter,
            ENV_LLM_MAX_DECISION_STEPS,
            DEFAULT_LLM_MAX_DECISION_STEPS,
            |value| LlmConfigError::InvalidMaxDecisionSteps { value },
        )?;
        let max_repair_rounds = parse_positive_usize(
            &mut getter,
            ENV_LLM_MAX_REPAIR_ROUNDS,
            DEFAULT_LLM_MAX_REPAIR_ROUNDS,
            |value| LlmConfigError::InvalidMaxRepairRounds { value },
        )?;
        let prompt_max_history_items = parse_positive_usize(
            &mut getter,
            ENV_LLM_PROMPT_MAX_HISTORY_ITEMS,
            DEFAULT_LLM_PROMPT_MAX_HISTORY_ITEMS,
            |value| LlmConfigError::InvalidPromptMaxHistoryItems { value },
        )?;
        let prompt_profile = match getter(ENV_LLM_PROMPT_PROFILE) {
            Some(value) => LlmPromptProfile::parse(value.as_str())
                .ok_or(LlmConfigError::InvalidPromptProfile { value })?,
            None => DEFAULT_LLM_PROMPT_PROFILE,
        };
        let force_replan_after_same_action = parse_non_negative_usize(
            &mut getter,
            ENV_LLM_FORCE_REPLAN_AFTER_SAME_ACTION,
            DEFAULT_LLM_FORCE_REPLAN_AFTER_SAME_ACTION,
            |value| LlmConfigError::InvalidForceReplanAfterSameAction { value },
        )?;
        let harvest_max_amount_cap = parse_positive_i64(
            &mut getter,
            ENV_LLM_HARVEST_MAX_AMOUNT_CAP,
            DEFAULT_LLM_HARVEST_MAX_AMOUNT_CAP,
            |value| LlmConfigError::InvalidHarvestMaxAmountCap { value },
        )?;
        let execute_until_auto_reenter_ticks = parse_non_negative_usize(
            &mut getter,
            ENV_LLM_EXECUTE_UNTIL_AUTO_REENTER_TICKS,
            DEFAULT_LLM_EXECUTE_UNTIL_AUTO_REENTER_TICKS,
            |value| LlmConfigError::InvalidExecuteUntilAutoReenterTicks { value },
        )?;
        let llm_debug_mode = match getter(ENV_LLM_DEBUG_MODE) {
            Some(value) => parse_debug_mode_flag(value.as_str())
                .ok_or(LlmConfigError::InvalidDebugMode { value })?,
            None => DEFAULT_LLM_DEBUG_MODE,
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
            max_decision_steps,
            max_repair_rounds,
            prompt_max_history_items,
            prompt_profile,
            force_replan_after_same_action,
            harvest_max_amount_cap,
            execute_until_auto_reenter_ticks,
            llm_debug_mode,
        })
    }

    fn prompt_budget(&self) -> PromptBudget {
        self.prompt_profile.prompt_budget()
    }

    fn memory_selector_config(&self) -> MemorySelectorConfig {
        self.prompt_profile.memory_selector_config()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmConfigError {
    MissingEnv { key: &'static str },
    EmptyEnv { key: &'static str },
    InvalidTimeout { value: String },
    InvalidMaxModuleCalls { value: String },
    InvalidMaxDecisionSteps { value: String },
    InvalidMaxRepairRounds { value: String },
    InvalidPromptMaxHistoryItems { value: String },
    InvalidPromptProfile { value: String },
    InvalidForceReplanAfterSameAction { value: String },
    InvalidHarvestMaxAmountCap { value: String },
    InvalidExecuteUntilAutoReenterTicks { value: String },
    InvalidDebugMode { value: String },
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
            LlmConfigError::InvalidMaxDecisionSteps { value } => {
                write!(f, "invalid max decision steps value: {value}")
            }
            LlmConfigError::InvalidMaxRepairRounds { value } => {
                write!(f, "invalid max repair rounds value: {value}")
            }
            LlmConfigError::InvalidPromptMaxHistoryItems { value } => {
                write!(f, "invalid prompt max history items value: {value}")
            }
            LlmConfigError::InvalidPromptProfile { value } => {
                write!(f, "invalid prompt profile value: {value}")
            }
            LlmConfigError::InvalidForceReplanAfterSameAction { value } => {
                write!(f, "invalid force replan after same action value: {value}")
            }
            LlmConfigError::InvalidHarvestMaxAmountCap { value } => {
                write!(f, "invalid harvest max amount cap value: {value}")
            }
            LlmConfigError::InvalidExecuteUntilAutoReenterTicks { value } => {
                write!(f, "invalid execute_until auto reenter ticks value: {value}")
            }
            LlmConfigError::InvalidDebugMode { value } => {
                write!(f, "invalid debug mode value: {value}")
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
    pub debug_mode: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LlmCompletionTurn {
    Decision {
        payload: serde_json::Value,
    },
    ModuleCall {
        module: String,
        args: serde_json::Value,
    },
}

pub trait LlmCompletionClient {
    fn complete(
        &self,
        request: &LlmCompletionRequest,
    ) -> Result<LlmCompletionResult, LlmClientError>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LlmCompletionResult {
    pub turns: Vec<LlmCompletionTurn>,
    pub output: String,
    pub model: Option<String>,
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct OpenAiChatCompletionClient {
    client: AsyncOpenAiClient<OpenAIConfig>,
    request_timeout_ms: u64,
    timeout_retry_client: Option<AsyncOpenAiClient<OpenAIConfig>>,
    timeout_retry_ms: Option<u64>,
}

impl OpenAiChatCompletionClient {
    pub fn from_config(config: &LlmAgentConfig) -> Result<Self, LlmClientError> {
        let request_timeout_ms = config.timeout_ms.max(1);
        let api_base = normalize_openai_api_base_url(config.base_url.as_str());
        let client = Self::build_client(
            api_base.as_str(),
            config.api_key.as_str(),
            request_timeout_ms,
        )?;
        let (timeout_retry_client, timeout_retry_ms) =
            if request_timeout_ms < DEFAULT_LLM_TIMEOUT_MS {
                let retry_timeout_ms = DEFAULT_LLM_TIMEOUT_MS;
                (
                    Some(Self::build_client(
                        api_base.as_str(),
                        config.api_key.as_str(),
                        retry_timeout_ms,
                    )?),
                    Some(retry_timeout_ms),
                )
            } else {
                (None, None)
            };

        Ok(Self {
            client,
            request_timeout_ms,
            timeout_retry_client,
            timeout_retry_ms,
        })
    }

    fn build_http_client(timeout_ms: u64) -> Result<reqwest::Client, LlmClientError> {
        #[cfg(target_arch = "wasm32")]
        let builder = {
            let _ = timeout_ms;
            reqwest::Client::builder()
        };

        #[cfg(not(target_arch = "wasm32"))]
        let builder =
            reqwest::Client::builder().timeout(std::time::Duration::from_millis(timeout_ms.max(1)));

        builder.build().map_err(|err| LlmClientError::BuildClient {
            message: err.to_string(),
        })
    }

    fn build_client(
        api_base: &str,
        api_key: &str,
        timeout_ms: u64,
    ) -> Result<AsyncOpenAiClient<OpenAIConfig>, LlmClientError> {
        let config = OpenAIConfig::new()
            .with_api_base(api_base.to_string())
            .with_api_key(api_key.to_string());

        let http_client = Self::build_http_client(timeout_ms)?;
        Ok(AsyncOpenAiClient::with_config(config).with_http_client(http_client))
    }

    fn send_responses_request(
        &self,
        client: &AsyncOpenAiClient<OpenAIConfig>,
        payload: CreateResponse,
    ) -> Result<Response, OpenAiRequestError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| OpenAiRequestError::Other(err.to_string()))?;

        runtime
            .block_on(client.responses().create(payload))
            .map_err(OpenAiRequestError::from)
    }
}

#[derive(Debug)]
enum OpenAiRequestError {
    Timeout(String),
    ParseBody(String),
    Other(String),
}

impl From<OpenAIError> for OpenAiRequestError {
    fn from(value: OpenAIError) -> Self {
        match value {
            OpenAIError::Reqwest(err) if err.is_timeout() => Self::Timeout(err.to_string()),
            OpenAIError::JSONDeserialize(_, raw_body) => Self::ParseBody(raw_body),
            other => Self::Other(other.to_string()),
        }
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

const OPENAI_TOOL_AGENT_MODULES_LIST: &str = "agent_modules_list";
const OPENAI_TOOL_ENVIRONMENT_CURRENT_OBSERVATION: &str = "environment_current_observation";
const OPENAI_TOOL_MEMORY_SHORT_TERM_RECENT: &str = "memory_short_term_recent";
const OPENAI_TOOL_MEMORY_LONG_TERM_SEARCH: &str = "memory_long_term_search";
const OPENAI_TOOL_AGENT_SUBMIT_DECISION: &str = "agent_submit_decision";
const OPENAI_TOOL_AGENT_DEBUG_GRANT_RESOURCE: &str = "agent_debug_grant_resource";

fn sanitize_prompt_override(value: Option<String>) -> Option<String> {
    let Some(value) = value else {
        return None;
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

impl LlmCompletionClient for OpenAiChatCompletionClient {
    fn complete(
        &self,
        request: &LlmCompletionRequest,
    ) -> Result<LlmCompletionResult, LlmClientError> {
        let payload = build_responses_request_payload(request)?;

        match self.send_responses_request(&self.client, payload.clone()) {
            Ok(response) => return completion_result_from_sdk_response(response),
            Err(OpenAiRequestError::ParseBody(raw_body)) => {
                return Err(LlmClientError::DecodeResponse {
                    message: format!(
                        "responses sdk decode failed (primary request): {}",
                        summarize_trace_text(raw_body.as_str(), 320)
                    ),
                });
            }
            Err(OpenAiRequestError::Timeout(err)) => {
                if let (Some(retry_client), Some(retry_timeout_ms)) =
                    (&self.timeout_retry_client, self.timeout_retry_ms)
                {
                    match self.send_responses_request(retry_client, payload) {
                        Ok(response) => return completion_result_from_sdk_response(response),
                        Err(OpenAiRequestError::ParseBody(raw_body)) => {
                            return Err(LlmClientError::DecodeResponse {
                                message: format!(
                                    "responses sdk decode failed (retry request): {}",
                                    summarize_trace_text(raw_body.as_str(), 320)
                                ),
                            });
                        }
                        Err(retry_err) => {
                            let retry_message = match retry_err {
                                OpenAiRequestError::Timeout(message) => message,
                                OpenAiRequestError::ParseBody(message) => message,
                                OpenAiRequestError::Other(message) => message,
                            };
                            return Err(LlmClientError::Http {
                                message: format!(
                                    "request timed out after {}ms; retry with {}ms failed: {}",
                                    self.request_timeout_ms, retry_timeout_ms, retry_message
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
            Err(OpenAiRequestError::Other(err)) => {
                return Err(LlmClientError::Http { message: err });
            }
        }
    }
}

#[derive(Debug)]
pub struct LlmAgentBehavior<C: LlmCompletionClient> {
    agent_id: String,
    config: LlmAgentConfig,
    prompt_overrides: LlmPromptOverrides,
    client: C,
    memory: AgentMemory,
    next_effect_intent_id: u64,
    pending_trace: Option<AgentDecisionTrace>,
    replan_guard_state: ActionReplanGuardState,
    active_execute_until: Option<ActiveExecuteUntil>,
    conversation_history: Vec<LlmChatMessageTrace>,
    conversation_trace_cursor: usize,
    last_action_summary: Option<PromptLastActionSummary>,
    pending_decision_rewrite: Option<DecisionRewriteReceipt>,
    known_factory_locations: BTreeMap<String, String>,
    known_factory_kind_aliases: BTreeMap<String, String>,
    recipe_coverage: RecipeCoverageProgress,
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
            prompt_overrides: LlmPromptOverrides::default(),
            client,
            memory,
            next_effect_intent_id: 0,
            pending_trace: None,
            replan_guard_state: ActionReplanGuardState::default(),
            active_execute_until: None,
            conversation_history: Vec::new(),
            conversation_trace_cursor: 0,
            last_action_summary: None,
            pending_decision_rewrite: None,
            known_factory_locations: BTreeMap::new(),
            known_factory_kind_aliases: BTreeMap::new(),
            recipe_coverage: RecipeCoverageProgress::default(),
        }
    }

    pub fn apply_prompt_overrides(
        &mut self,
        system_prompt: Option<String>,
        short_term_goal: Option<String>,
        long_term_goal: Option<String>,
    ) {
        self.prompt_overrides.system_prompt = sanitize_prompt_override(system_prompt);
        self.prompt_overrides.short_term_goal = sanitize_prompt_override(short_term_goal);
        self.prompt_overrides.long_term_goal = sanitize_prompt_override(long_term_goal);
    }

    pub fn prompt_overrides(&self) -> LlmPromptOverrides {
        self.prompt_overrides.clone()
    }

    pub fn push_player_message(&mut self, time: u64, message: impl AsRef<str>) -> bool {
        self.append_conversation_message(time, LlmChatRole::Player, message.as_ref())
            .is_some()
    }

    fn effective_system_prompt(&self) -> &str {
        self.prompt_overrides
            .system_prompt
            .as_deref()
            .unwrap_or(self.config.system_prompt.as_str())
    }

    fn effective_short_term_goal(&self) -> &str {
        self.prompt_overrides
            .short_term_goal
            .as_deref()
            .unwrap_or(self.config.short_term_goal.as_str())
    }

    fn effective_long_term_goal(&self) -> &str {
        self.prompt_overrides
            .long_term_goal
            .as_deref()
            .unwrap_or(self.config.long_term_goal.as_str())
    }

    #[cfg(test)]
    fn system_prompt(&self) -> String {
        let prompt_budget = self.config.prompt_budget();
        let prompt: PromptAssemblyOutput = PromptAssembler::assemble(PromptAssemblyInput {
            agent_id: self.agent_id.as_str(),
            base_system_prompt: self.effective_system_prompt(),
            short_term_goal: self.effective_short_term_goal(),
            long_term_goal: self.effective_long_term_goal(),
            observation_json: "{}",
            module_history_json: "[]",
            conversation_history_json: "[]",
            memory_digest: None,
            step_context: PromptStepContext::default(),
            harvest_max_amount_cap: self.config.harvest_max_amount_cap,
            prompt_budget,
        });
        prompt.system_prompt
    }

    #[cfg(test)]
    fn user_prompt(
        &self,
        observation: &Observation,
        module_history: &[ModuleCallExchange],
        step_index: usize,
        max_steps: usize,
    ) -> String {
        self.assemble_prompt_output(observation, module_history, step_index, max_steps)
            .user_prompt
    }

    fn assemble_prompt_output(
        &self,
        observation: &Observation,
        module_history: &[ModuleCallExchange],
        step_index: usize,
        max_steps: usize,
    ) -> PromptAssemblyOutput {
        let observation_json = self.observation_json_for_prompt(observation);
        let history_start = module_history
            .len()
            .saturating_sub(self.config.prompt_max_history_items);
        let history_slice = &module_history[history_start..];
        let history_json = Self::module_history_json_for_prompt(history_slice);
        let memory_selector_config = self.config.memory_selector_config();
        let memory_selection =
            MemorySelector::select(&self.memory, observation.time, &memory_selector_config);
        let memory_digest = Self::memory_digest_for_prompt(memory_selection.digest.as_str());
        let conversation_json = self.conversation_history_json_for_prompt();
        let prompt_budget = self.config.prompt_budget();
        PromptAssembler::assemble(PromptAssemblyInput {
            agent_id: self.agent_id.as_str(),
            base_system_prompt: self.effective_system_prompt(),
            short_term_goal: self.effective_short_term_goal(),
            long_term_goal: self.effective_long_term_goal(),
            observation_json: observation_json.as_str(),
            module_history_json: history_json.as_str(),
            conversation_history_json: conversation_json.as_str(),
            memory_digest: Some(memory_digest.as_str()),
            step_context: PromptStepContext {
                step_index,
                max_steps,
                module_calls_used: module_history.len(),
                module_calls_max: self.config.max_module_calls,
            },
            harvest_max_amount_cap: self.config.harvest_max_amount_cap,
            prompt_budget,
        })
    }

    fn observation_json_for_prompt(&self, observation: &Observation) -> String {
        let mut visible_agents = observation.visible_agents.iter().collect::<Vec<_>>();
        visible_agents.sort_by_key(|agent| agent.distance_cm);
        let visible_agents = visible_agents
            .into_iter()
            .take(PROMPT_OBSERVATION_VISIBLE_AGENTS_MAX)
            .map(|agent| {
                serde_json::json!({
                    "agent_id": agent.agent_id,
                    "distance_cm": agent.distance_cm,
                })
            })
            .collect::<Vec<_>>();

        let mut visible_locations = observation.visible_locations.iter().collect::<Vec<_>>();
        visible_locations.sort_by_key(|location| location.distance_cm);
        let visible_locations = visible_locations
            .into_iter()
            .take(PROMPT_OBSERVATION_VISIBLE_LOCATIONS_MAX)
            .map(|location| {
                serde_json::json!({
                    "location_id": location.location_id,
                    "distance_cm": location.distance_cm,
                })
            })
            .collect::<Vec<_>>();

        let last_action = self
            .last_action_summary
            .as_ref()
            .map(|summary| {
                serde_json::json!({
                    "kind": summary.kind,
                    "success": summary.success,
                    "reject_reason": summary.reject_reason,
                    "decision_rewrite": summary.decision_rewrite,
                })
            })
            .unwrap_or(serde_json::Value::Null);
        let recipe_coverage = self.recipe_coverage.summary_json();

        serde_json::to_string(&serde_json::json!({
            "time": observation.time,
            "agent_id": observation.agent_id,
            "pos": observation.pos,
            "self_resources": observation.self_resources,
            "last_action": last_action,
            "recipe_coverage": recipe_coverage,
            "visibility_range_cm": observation.visibility_range_cm,
            "visible_agents_total": observation.visible_agents.len(),
            "visible_agents_omitted": observation
                .visible_agents
                .len()
                .saturating_sub(visible_agents.len()),
            "visible_agents": visible_agents,
            "visible_locations_total": observation.visible_locations.len(),
            "visible_locations_omitted": observation
                .visible_locations
                .len()
                .saturating_sub(visible_locations.len()),
            "visible_locations": visible_locations,
        }))
        .unwrap_or_else(|_| "{\"error\":\"observation serialize failed\"}".to_string())
    }

    fn action_kind_name_for_prompt(action: &Action) -> &'static str {
        match action {
            Action::RegisterLocation { .. } => "register_location",
            Action::RegisterAgent { .. } => "register_agent",
            Action::RegisterPowerPlant { .. } => "register_power_plant",
            Action::RegisterPowerStorage { .. } => "register_power_storage",
            Action::UpsertModuleVisualEntity { .. } => "upsert_module_visual_entity",
            Action::RemoveModuleVisualEntity { .. } => "remove_module_visual_entity",
            Action::DrawPower { .. } => "draw_power",
            Action::StorePower { .. } => "store_power",
            Action::MoveAgent { .. } => "move_agent",
            Action::HarvestRadiation { .. } => "harvest_radiation",
            Action::BuyPower { .. } => "buy_power",
            Action::SellPower { .. } => "sell_power",
            Action::TransferResource { .. } => "transfer_resource",
            Action::DebugGrantResource { .. } => "debug_grant_resource",
            Action::MineCompound { .. } => "mine_compound",
            Action::RefineCompound { .. } => "refine_compound",
            Action::BuildFactory { .. } => "build_factory",
            Action::ScheduleRecipe { .. } => "schedule_recipe",
        }
    }

    fn reject_reason_code_for_prompt(reason: &RejectReason) -> String {
        match reason {
            RejectReason::InsufficientResource { kind, .. } => {
                let kind = match kind {
                    ResourceKind::Electricity => "electricity",
                    ResourceKind::Compound => "compound",
                    ResourceKind::Hardware => "hardware",
                    ResourceKind::Data => "data",
                };
                format!("insufficient_resource.{kind}")
            }
            RejectReason::FacilityNotFound { .. } => "factory_not_found".to_string(),
            RejectReason::FacilityAlreadyExists { .. } => "facility_already_exists".to_string(),
            RejectReason::AgentAlreadyAtLocation { .. } => "agent_already_at_location".to_string(),
            RejectReason::AgentNotAtLocation { .. } => "agent_not_at_location".to_string(),
            RejectReason::ThermalOverload { .. } => "thermal_overload".to_string(),
            RejectReason::RadiationUnavailable { .. } => "radiation_unavailable".to_string(),
            _ => "other".to_string(),
        }
    }

    fn summarize_action_result_for_prompt(
        result: &ActionResult,
        decision_rewrite: Option<DecisionRewriteReceipt>,
    ) -> PromptLastActionSummary {
        PromptLastActionSummary {
            kind: Self::action_kind_name_for_prompt(&result.action).to_string(),
            success: result.success,
            reject_reason: result
                .reject_reason()
                .map(Self::reject_reason_code_for_prompt),
            decision_rewrite,
        }
    }

    fn decision_label_for_rewrite(decision: &AgentDecision) -> String {
        match decision {
            AgentDecision::Act(action) => Self::action_label_for_rewrite(action),
            AgentDecision::Wait => "wait".to_string(),
            AgentDecision::WaitTicks(_) => "wait_ticks".to_string(),
        }
    }

    fn action_label_for_rewrite(action: &Action) -> String {
        Self::action_kind_name_for_prompt(action).to_string()
    }

    fn decision_rewrite_receipt(
        from: &AgentDecision,
        to: &AgentDecision,
        reason: Option<&str>,
    ) -> Option<DecisionRewriteReceipt> {
        let from_label = Self::decision_label_for_rewrite(from);
        let to_label = Self::decision_label_for_rewrite(to);
        if from_label == to_label {
            return None;
        }
        Some(DecisionRewriteReceipt {
            from: from_label,
            to: to_label,
            reason: reason
                .unwrap_or("decision rewritten by guardrail")
                .trim()
                .to_string(),
        })
    }

    fn action_rewrite_receipt(
        from: &Action,
        to: &Action,
        reason: Option<&str>,
    ) -> Option<DecisionRewriteReceipt> {
        let from_label = Self::action_label_for_rewrite(from);
        let to_label = Self::action_label_for_rewrite(to);
        if from_label == to_label {
            return None;
        }
        Some(DecisionRewriteReceipt {
            from: from_label,
            to: to_label,
            reason: reason
                .unwrap_or("action rewritten by guardrail")
                .trim()
                .to_string(),
        })
    }

    fn decision_rewrite_receipt_json(receipt: &DecisionRewriteReceipt) -> String {
        serde_json::to_string(receipt).unwrap_or_else(|_| {
            format!(
                r#"{{"from":"{}","to":"{}","reason":"{}"}}"#,
                receipt.from, receipt.to, receipt.reason
            )
        })
    }

    fn record_decision_rewrite_receipt(
        &mut self,
        time: u64,
        receipt: &DecisionRewriteReceipt,
        turn_output_summary: &mut String,
    ) {
        let receipt_json = Self::decision_rewrite_receipt_json(receipt);
        let note = format!("decision_rewrite: {receipt_json}");
        self.memory.record_note(time, note.clone());
        let _ = self.append_conversation_message(time, LlmChatRole::System, note.as_str());
        *turn_output_summary = format!(
            "{}; decision_rewrite={}",
            turn_output_summary,
            summarize_trace_text(receipt_json.as_str(), 200)
        );
    }

    fn memory_digest_for_prompt(digest: &str) -> String {
        summarize_trace_text(digest, PROMPT_MEMORY_DIGEST_MAX_CHARS)
    }

    fn conversation_history_json_for_prompt(&self) -> String {
        let start = self
            .conversation_history
            .len()
            .saturating_sub(PROMPT_CONVERSATION_MAX_ITEMS);
        let compact = self.conversation_history[start..]
            .iter()
            .map(|entry| {
                serde_json::json!({
                    "time": entry.time,
                    "role": match entry.role {
                        LlmChatRole::Player => "player",
                        LlmChatRole::Agent => "agent",
                        LlmChatRole::Tool => "tool",
                        LlmChatRole::System => "system",
                    },
                    "content": summarize_trace_text(
                        entry.content.as_str(),
                        PROMPT_CONVERSATION_ITEM_MAX_CHARS,
                    ),
                })
            })
            .collect::<Vec<_>>();
        serde_json::to_string(&compact).unwrap_or_else(|_| "[]".to_string())
    }

    fn module_history_json_for_prompt(module_history: &[ModuleCallExchange]) -> String {
        let compact_history = module_history
            .iter()
            .map(|exchange| {
                serde_json::json!({
                    "module": exchange.module,
                    "args": Self::compact_json_value_for_prompt(
                        &exchange.args,
                        PROMPT_MODULE_ARGS_MAX_CHARS,
                    ),
                    "result": Self::module_result_for_prompt(&exchange.result),
                })
            })
            .collect::<Vec<_>>();

        serde_json::to_string(&compact_history).unwrap_or_else(|_| "[]".to_string())
    }

    fn compact_json_value_for_prompt(
        value: &serde_json::Value,
        max_chars: usize,
    ) -> serde_json::Value {
        let serialized = serde_json::to_string(value).unwrap_or_else(|_| "null".to_string());
        let total_chars = serialized.chars().count();
        if total_chars <= max_chars {
            return value.clone();
        }

        serde_json::json!({
            "truncated": true,
            "original_chars": total_chars,
            "preview": summarize_trace_text(serialized.as_str(), max_chars),
        })
    }

    fn module_result_for_prompt(result: &serde_json::Value) -> serde_json::Value {
        Self::compact_json_value_for_prompt(result, PROMPT_MODULE_RESULT_MAX_CHARS)
    }

    fn trace_input(system_prompt: &str, user_prompt: &str) -> String {
        format!("[system]\n{}\n\n[user]\n{}", system_prompt, user_prompt)
    }

    fn apply_decision_guardrails(
        &self,
        decision: AgentDecision,
        observation: &Observation,
    ) -> (AgentDecision, Option<String>) {
        match decision {
            AgentDecision::Act(action) => {
                let (guarded_action, note) =
                    self.apply_action_guardrails(action, Some(observation));
                (AgentDecision::Act(guarded_action), note)
            }
            other => (other, None),
        }
    }

    fn apply_action_guardrails(
        &self,
        action: Action,
        observation: Option<&Observation>,
    ) -> (Action, Option<String>) {
        match action {
            Action::MoveAgent { agent_id, to } if agent_id == self.agent_id => {
                let Some(observation) = observation else {
                    return (Action::MoveAgent { agent_id, to }, None);
                };
                self.guarded_move_to_location(to.as_str(), observation)
            }
            Action::HarvestRadiation {
                agent_id,
                max_amount,
            } if max_amount > self.config.harvest_max_amount_cap => {
                let capped = self.config.harvest_max_amount_cap;
                (
                    Action::HarvestRadiation {
                        agent_id,
                        max_amount: capped,
                    },
                    Some(format!(
                        "harvest_radiation.max_amount clamped: {} -> {}",
                        max_amount, capped
                    )),
                )
            }
            Action::BuildFactory {
                owner,
                location_id,
                factory_id,
                factory_kind,
            } => {
                let Some(observation) = observation else {
                    return (
                        Action::BuildFactory {
                            owner,
                            location_id,
                            factory_id,
                            factory_kind,
                        },
                        None,
                    );
                };
                let owner_is_self = matches!(
                    &owner,
                    ResourceOwner::Agent { agent_id } if agent_id == self.agent_id.as_str()
                );
                if !owner_is_self {
                    return (
                        Action::BuildFactory {
                            owner,
                            location_id,
                            factory_id,
                            factory_kind,
                        },
                        None,
                    );
                }

                if let Some(existing_factory_id) = self.resolve_existing_factory_id_for_build(
                    factory_id.as_str(),
                    factory_kind.as_str(),
                ) {
                    let recipe_id = self.next_recovery_recipe_id_for_existing_factory();
                    let (guarded_schedule_action, schedule_note) = self.apply_action_guardrails(
                        Action::ScheduleRecipe {
                            owner: owner.clone(),
                            factory_id: existing_factory_id.clone(),
                            recipe_id: recipe_id.clone(),
                            batches: 1,
                        },
                        Some(observation),
                    );
                    let mut notes = vec![format!(
                        "build_factory dedup guardrail rerouted to schedule_recipe: requested_factory_id={} factory_kind={} existing_factory_id={} recipe_id={}",
                        factory_id, factory_kind, existing_factory_id, recipe_id
                    )];
                    if let Some(schedule_note) = schedule_note {
                        notes.push(schedule_note);
                    }
                    return (guarded_schedule_action, Some(notes.join("; ")));
                }

                (
                    Action::BuildFactory {
                        owner,
                        location_id,
                        factory_id,
                        factory_kind,
                    },
                    None,
                )
            }
            Action::ScheduleRecipe {
                owner,
                factory_id,
                recipe_id,
                batches,
            } => {
                let Some(observation) = observation else {
                    return (
                        Action::ScheduleRecipe {
                            owner,
                            factory_id,
                            recipe_id,
                            batches,
                        },
                        None,
                    );
                };
                let Some(mut cost_per_batch) =
                    Self::default_recipe_hardware_cost_per_batch(recipe_id.as_str())
                else {
                    return (
                        Action::ScheduleRecipe {
                            owner,
                            factory_id,
                            recipe_id,
                            batches,
                        },
                        None,
                    );
                };
                if cost_per_batch <= 0 {
                    return (
                        Action::ScheduleRecipe {
                            owner,
                            factory_id,
                            recipe_id,
                            batches,
                        },
                        None,
                    );
                }

                let owner_is_self = matches!(
                    &owner,
                    ResourceOwner::Agent { agent_id } if agent_id == self.agent_id.as_str()
                );
                if !owner_is_self {
                    return (
                        Action::ScheduleRecipe {
                            owner,
                            factory_id,
                            recipe_id,
                            batches,
                        },
                        None,
                    );
                }

                let mut factory_id = factory_id;
                let mut recipe_id = recipe_id;
                let mut schedule_notes = Vec::new();
                if let Some(canonical_factory_id) =
                    self.normalize_schedule_factory_id(factory_id.as_str())
                {
                    schedule_notes.push(format!(
                        "schedule_recipe factory_id normalized by guardrail: requested_factory_id={} -> canonical_factory_id={}",
                        factory_id, canonical_factory_id
                    ));
                    factory_id = canonical_factory_id;
                }

                if let Some(factory_location_id) =
                    self.known_factory_locations.get(factory_id.as_str())
                {
                    if let Some(current_location_id) =
                        Self::current_location_id_from_observation(observation)
                    {
                        if current_location_id != factory_location_id {
                            let (move_action, move_note) = self.guarded_move_to_location(
                                factory_location_id.as_str(),
                                observation,
                            );
                            let mut notes = schedule_notes.clone();
                            notes.push(format!(
                                "schedule_recipe factory location precheck rerouted to move_agent: current_location={} factory_location={}",
                                current_location_id, factory_location_id
                            ));
                            if let Some(move_note) = move_note {
                                notes.push(move_note);
                            }
                            return (move_action, Some(notes.join("; ")));
                        }
                    }
                }
                if self.recipe_coverage.is_completed(recipe_id.as_str()) {
                    if let Some(next_recipe_id) = self
                        .recipe_coverage
                        .next_uncovered_recipe_excluding(recipe_id.as_str())
                    {
                        schedule_notes.push(format!(
                            "schedule_recipe coverage hard-switch applied: completed_recipe={} -> next_uncovered_recipe={}",
                            recipe_id, next_recipe_id
                        ));
                        recipe_id = next_recipe_id;
                        if let Some(next_cost_per_batch) =
                            Self::default_recipe_hardware_cost_per_batch(recipe_id.as_str())
                        {
                            cost_per_batch = next_cost_per_batch;
                        }
                    }
                }

                let available_hardware = observation.self_resources.get(ResourceKind::Hardware);
                let available_electricity =
                    observation.self_resources.get(ResourceKind::Electricity);
                let available_compound = observation.self_resources.get(ResourceKind::Compound);
                if available_hardware < cost_per_batch {
                    let hardware_shortfall = cost_per_batch.saturating_sub(available_hardware);
                    let target_recovery_mass_g = hardware_shortfall
                        .saturating_mul(DEFAULT_REFINE_RECOVERY_MASS_G_PER_HARDWARE)
                        .max(DEFAULT_REFINE_RECOVERY_MASS_G_PER_HARDWARE);
                    let recovery_mass_g = target_recovery_mass_g
                        .min(DEFAULT_MINE_COMPOUND_MAX_PER_ACTION_G)
                        .max(DEFAULT_REFINE_RECOVERY_MASS_G_PER_HARDWARE);
                    let capped_from = (target_recovery_mass_g > recovery_mass_g)
                        .then_some(target_recovery_mass_g);
                    let missing_compound_g = recovery_mass_g.saturating_sub(available_compound);
                    if missing_compound_g > 0 {
                        let mine_mass_g = missing_compound_g
                            .min(DEFAULT_MINE_COMPOUND_MAX_PER_ACTION_G)
                            .max(1);
                        let mine_required_electricity = ((mine_mass_g + 999) / 1000)
                            .saturating_mul(DEFAULT_MINE_ELECTRICITY_COST_PER_KG);
                        if available_electricity >= mine_required_electricity {
                            if let Some(current_location_id) =
                                Self::current_location_id_from_observation(observation)
                            {
                                return (
                                    Action::MineCompound {
                                        owner,
                                        location_id: current_location_id.to_string(),
                                        compound_mass_g: mine_mass_g,
                                    },
                                    Some({
                                        let mut notes = schedule_notes.clone();
                                        notes.push(format!(
                                            "schedule_recipe guardrail rerouted to mine_compound before refine: available_hardware={} < recipe_hardware_cost_per_batch={}; hardware_shortfall={}; recovery_mass_g={}{}; available_compound={}; mine_mass_g={}",
                                            available_hardware,
                                            cost_per_batch,
                                            hardware_shortfall,
                                            recovery_mass_g,
                                            capped_from
                                                .map(|from| format!(" (capped_from={} by mine_max_per_action_g={})", from, DEFAULT_MINE_COMPOUND_MAX_PER_ACTION_G))
                                                .unwrap_or_default(),
                                            available_compound,
                                            mine_mass_g
                                        ));
                                        notes.join("; ")
                                    }),
                                );
                            }
                            return (
                                Action::HarvestRadiation {
                                    agent_id: self.agent_id.clone(),
                                    max_amount: self.config.harvest_max_amount_cap,
                                },
                                Some({
                                    let mut notes = schedule_notes.clone();
                                    notes.push(format!(
                                        "schedule_recipe guardrail rerouted to harvest_radiation: available_hardware={} < recipe_hardware_cost_per_batch={} and available_compound={} < recovery_mass_g={} but current_location_id is unknown for mine_compound",
                                        available_hardware,
                                        cost_per_batch,
                                        available_compound,
                                        recovery_mass_g
                                    ));
                                    notes.join("; ")
                                }),
                            );
                        } else {
                            return (
                                Action::HarvestRadiation {
                                    agent_id: self.agent_id.clone(),
                                    max_amount: self.config.harvest_max_amount_cap,
                                },
                                Some({
                                    let mut notes = schedule_notes.clone();
                                    notes.push(format!(
                                        "schedule_recipe guardrail rerouted to harvest_radiation: available_hardware={} < recipe_hardware_cost_per_batch={} and available_compound={} < recovery_mass_g={} with available_electricity={} < mine_required_electricity={}",
                                        available_hardware,
                                        cost_per_batch,
                                        available_compound,
                                        recovery_mass_g,
                                        available_electricity,
                                        mine_required_electricity
                                    ));
                                    notes.join("; ")
                                }),
                            );
                        }
                    }

                    let required_refine_electricity = ((recovery_mass_g + 999) / 1000)
                        .saturating_mul(DEFAULT_REFINE_ELECTRICITY_COST_PER_KG);
                    if available_electricity >= required_refine_electricity {
                        return (
                            Action::RefineCompound {
                                owner,
                                compound_mass_g: recovery_mass_g,
                            },
                            Some({
                                let mut notes = schedule_notes.clone();
                                notes.push(format!(
                                    "schedule_recipe guardrail rerouted to refine_compound: available_hardware={} < recipe_hardware_cost_per_batch={}; hardware_shortfall={}; recovery_mass_g={}{}",
                                    available_hardware,
                                    cost_per_batch,
                                    hardware_shortfall,
                                    recovery_mass_g,
                                    capped_from
                                        .map(|from| format!(" (capped_from={} by mine_max_per_action_g={})", from, DEFAULT_MINE_COMPOUND_MAX_PER_ACTION_G))
                                        .unwrap_or_default()
                                ));
                                notes.join("; ")
                            }),
                        );
                    }
                    return (
                        Action::HarvestRadiation {
                            agent_id: self.agent_id.clone(),
                            max_amount: self.config.harvest_max_amount_cap,
                        },
                        Some({
                            let mut notes = schedule_notes.clone();
                            notes.push(format!(
                                "schedule_recipe guardrail rerouted to harvest_radiation: available_hardware={} < recipe_hardware_cost_per_batch={} and available_electricity={} < refine_required_electricity={} (recovery_mass_g={})",
                                available_hardware, cost_per_batch, available_electricity, required_refine_electricity, recovery_mass_g
                            ));
                            notes.join("; ")
                        }),
                    );
                }

                let max_batches = available_hardware / cost_per_batch;
                if batches > max_batches {
                    (
                        Action::ScheduleRecipe {
                            owner,
                            factory_id,
                            recipe_id,
                            batches: max_batches,
                        },
                        Some({
                            let mut notes = schedule_notes;
                            notes.push(format!(
                                "schedule_recipe.batches clamped by hardware guardrail: {} -> {} (available_hardware={}, recipe_hardware_cost_per_batch={})",
                                batches, max_batches, available_hardware, cost_per_batch
                            ));
                            notes.join("; ")
                        }),
                    )
                } else {
                    (
                        Action::ScheduleRecipe {
                            owner,
                            factory_id,
                            recipe_id,
                            batches,
                        },
                        (!schedule_notes.is_empty()).then_some(schedule_notes.join("; ")),
                    )
                }
            }
            other => (other, None),
        }
    }

    fn apply_execute_until_guardrails(
        &self,
        mut directive: ExecuteUntilDirective,
        observation: &Observation,
    ) -> (ExecuteUntilDirective, Option<String>) {
        let mut notes = Vec::new();
        let (guarded_action, action_note) =
            self.apply_action_guardrails(directive.action, Some(observation));
        directive.action = guarded_action;
        if let Some(action_note) = action_note {
            notes.push(action_note);
        }

        if matches!(directive.action, Action::HarvestRadiation { .. })
            && directive.max_ticks > DEFAULT_LLM_HARVEST_EXECUTE_UNTIL_MAX_TICKS
        {
            let original = directive.max_ticks;
            directive.max_ticks = DEFAULT_LLM_HARVEST_EXECUTE_UNTIL_MAX_TICKS;
            notes.push(format!(
                "execute_until.max_ticks clamped for harvest_radiation: {} -> {}; force replan sooner to avoid long harvest tail",
                original, directive.max_ticks
            ));
        }

        let note = if notes.is_empty() {
            None
        } else {
            Some(notes.join("; "))
        };
        (directive, note)
    }

    fn default_recipe_hardware_cost_per_batch(recipe_id: &str) -> Option<i64> {
        match recipe_id.trim() {
            "recipe.assembler.control_chip" => Some(DEFAULT_RECIPE_HARDWARE_COST_PER_BATCH),
            "recipe.assembler.motor_mk1" => Some(DEFAULT_RECIPE_HARDWARE_COST_PER_BATCH * 2),
            "recipe.assembler.logistics_drone" => Some(DEFAULT_RECIPE_HARDWARE_COST_PER_BATCH * 4),
            _ => None,
        }
    }

    fn current_location_id_from_observation(observation: &Observation) -> Option<&str> {
        observation
            .visible_locations
            .iter()
            .find(|location| location.distance_cm == 0)
            .map(|location| location.location_id.as_str())
    }

    fn normalize_schedule_factory_id(&self, factory_id: &str) -> Option<String> {
        let requested_factory_id = factory_id.trim();
        if requested_factory_id.is_empty()
            || self
                .known_factory_locations
                .contains_key(requested_factory_id)
        {
            return None;
        }
        self.known_factory_kind_aliases
            .get(requested_factory_id)
            .filter(|canonical_factory_id| {
                self.known_factory_locations
                    .contains_key(canonical_factory_id.as_str())
            })
            .cloned()
    }

    fn resolve_existing_factory_id_for_build(
        &self,
        factory_id: &str,
        factory_kind: &str,
    ) -> Option<String> {
        let requested_factory_id = factory_id.trim();
        if !requested_factory_id.is_empty()
            && self
                .known_factory_locations
                .contains_key(requested_factory_id)
        {
            return Some(requested_factory_id.to_string());
        }
        let requested_factory_kind = factory_kind.trim();
        if requested_factory_kind.is_empty() {
            return None;
        }
        self.known_factory_kind_aliases
            .get(requested_factory_kind)
            .cloned()
    }

    fn next_recovery_recipe_id_for_existing_factory(&self) -> String {
        self.recipe_coverage
            .missing_recipe_ids()
            .into_iter()
            .next()
            .unwrap_or_else(|| TRACKED_RECIPE_IDS[0].to_string())
    }

    fn find_reachable_move_relay(
        &self,
        to: &str,
        observation: &Observation,
    ) -> Option<(String, i64, i64, i64)> {
        if DEFAULT_MAX_MOVE_DISTANCE_CM_PER_TICK <= 0 {
            return None;
        }
        let target_location = observation
            .visible_locations
            .iter()
            .find(|location| location.location_id == to)?;
        if target_location.distance_cm <= DEFAULT_MAX_MOVE_DISTANCE_CM_PER_TICK {
            return None;
        }

        let mut best: Option<(String, i64, i64)> = None;
        for candidate in &observation.visible_locations {
            if candidate.location_id == target_location.location_id {
                continue;
            }
            if candidate.distance_cm <= 0
                || candidate.distance_cm > DEFAULT_MAX_MOVE_DISTANCE_CM_PER_TICK
            {
                continue;
            }

            let candidate_to_target =
                crate::geometry::space_distance_cm(candidate.pos, target_location.pos);
            if candidate_to_target >= target_location.distance_cm {
                continue;
            }

            let should_replace = match &best {
                None => true,
                Some((_, best_candidate_to_target, best_distance_from_self)) => {
                    candidate_to_target < *best_candidate_to_target
                        || (candidate_to_target == *best_candidate_to_target
                            && candidate.distance_cm < *best_distance_from_self)
                }
            };

            if should_replace {
                best = Some((
                    candidate.location_id.clone(),
                    candidate_to_target,
                    candidate.distance_cm,
                ));
            }
        }

        best.map(
            |(relay_location_id, relay_to_target_distance, relay_distance_from_self)| {
                (
                    relay_location_id,
                    target_location.distance_cm,
                    relay_distance_from_self,
                    relay_to_target_distance,
                )
            },
        )
    }

    fn guarded_move_to_location(
        &self,
        to: &str,
        observation: &Observation,
    ) -> (Action, Option<String>) {
        if let Some((
            relay_location_id,
            target_distance,
            relay_distance_from_self,
            relay_to_target_distance,
        )) = self.find_reachable_move_relay(to, observation)
        {
            return (
                Action::MoveAgent {
                    agent_id: self.agent_id.clone(),
                    to: relay_location_id.clone(),
                },
                Some(format!(
                    "move_agent segmented by distance guardrail: target={} distance_cm={} exceeds max_distance_cm={}; rerouted_via={} relay_distance_cm={} relay_to_target_cm={}",
                    to,
                    target_distance,
                    DEFAULT_MAX_MOVE_DISTANCE_CM_PER_TICK,
                    relay_location_id,
                    relay_distance_from_self,
                    relay_to_target_distance
                )),
            );
        }

        (
            Action::MoveAgent {
                agent_id: self.agent_id.clone(),
                to: to.to_string(),
            },
            None,
        )
    }

    fn remember_factory_location_hint(
        &mut self,
        factory_id: &str,
        location_id: &str,
        factory_kind: Option<&str>,
    ) {
        let factory_id = factory_id.trim();
        let location_id = location_id.trim();
        if factory_id.is_empty() || location_id.is_empty() {
            return;
        }
        self.known_factory_locations
            .insert(factory_id.to_string(), location_id.to_string());
        if let Some(factory_kind) = factory_kind
            .map(str::trim)
            .filter(|factory_kind| !factory_kind.is_empty())
        {
            self.known_factory_kind_aliases
                .insert(factory_kind.to_string(), factory_id.to_string());
        }
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
                        "args": { "limit": "u64, optional, default=3, max=8" }
                    },
                    {
                        "name": "memory.long_term.search",
                        "description": "按关键词检索长期记忆（query 为空时按重要度返回）。",
                        "args": {
                            "query": "string, optional",
                            "limit": "u64, optional, default=3, max=8"
                        }
                    }
                ]
            })),
            "environment.current_observation" => serde_json::to_value(observation)
                .map_err(|err| format!("serialize observation failed: {err}")),
            "memory.short_term.recent" => {
                let limit = parse_limit_arg(request.args.get("limit"), 3, 8);
                let mut entries: Vec<MemoryEntry> =
                    self.memory.short_term.recent(limit).cloned().collect();
                entries.reverse();
                serde_json::to_value(entries)
                    .map_err(|err| format!("serialize short-term memory failed: {err}"))
            }
            "memory.long_term.search" => {
                let limit = parse_limit_arg(request.args.get("limit"), 3, 8);
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

    fn append_conversation_message(
        &mut self,
        time: u64,
        role: LlmChatRole,
        content: &str,
    ) -> Option<LlmChatMessageTrace> {
        let normalized = content.trim();
        if normalized.is_empty() {
            return None;
        }
        let trace = LlmChatMessageTrace {
            time,
            agent_id: self.agent_id.clone(),
            role,
            content: summarize_trace_text(normalized, PROMPT_CONVERSATION_ITEM_MAX_CHARS * 2),
        };
        self.conversation_history.push(trace.clone());
        if self.conversation_history.len() > CONVERSATION_HISTORY_MAX_ITEMS {
            let overflow = self.conversation_history.len() - CONVERSATION_HISTORY_MAX_ITEMS;
            self.conversation_history.drain(0..overflow);
            self.conversation_trace_cursor =
                self.conversation_trace_cursor.saturating_sub(overflow);
        }
        Some(trace)
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

#[cfg(test)]
mod tests;
