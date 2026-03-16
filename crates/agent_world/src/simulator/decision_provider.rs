use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use serde::{Deserialize, Serialize};

use super::{
    Action, ActionId, ActionResult, AgentBehavior, AgentDecision, AgentDecisionTrace,
    LlmChatMessageTrace, LlmChatRole, LlmDecisionDiagnostics, LlmPromptSectionTrace, LlmStepTrace,
    Observation, ObservedAgent, ObservedLocation, ResourceKind, ResourceStock, WorldEvent,
    WorldTime,
};
use crate::geometry::GeoPos;

const DEFAULT_PROVIDER_TIMEOUT_BUDGET_MS: u64 = 3_000;
const MAX_RECENT_EVENT_SUMMARIES: usize = 8;
pub const DEFAULT_PROVIDER_OBSERVATION_SCHEMA_VERSION: &str = "oc_dual_obs_v1";
pub const DEFAULT_PROVIDER_ACTION_SCHEMA_VERSION: &str = "oc_dual_act_v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProviderExecutionMode {
    PlayerParity,
    #[default]
    HeadlessAgent,
}

impl ProviderExecutionMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PlayerParity => "player_parity",
            Self::HeadlessAgent => "headless_agent",
        }
    }
}

fn default_provider_execution_mode() -> ProviderExecutionMode {
    ProviderExecutionMode::HeadlessAgent
}

fn default_observation_schema_version() -> String {
    DEFAULT_PROVIDER_OBSERVATION_SCHEMA_VERSION.to_string()
}

fn default_action_schema_version() -> String {
    DEFAULT_PROVIDER_ACTION_SCHEMA_VERSION.to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionCatalogEntry {
    pub action_ref: String,
    pub summary: String,
}

impl ActionCatalogEntry {
    pub fn new(action_ref: impl Into<String>, summary: impl Into<String>) -> Self {
        Self {
            action_ref: action_ref.into(),
            summary: summary.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservationEnvelope {
    pub agent_id: String,
    pub world_time: WorldTime,
    #[serde(default = "default_provider_execution_mode")]
    pub mode: ProviderExecutionMode,
    #[serde(default = "default_observation_schema_version")]
    pub observation_schema_version: String,
    #[serde(default = "default_action_schema_version")]
    pub action_schema_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment_class: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<String>,
    pub observation: Observation,
    #[serde(default)]
    pub recent_event_summary: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_summary: Option<String>,
    #[serde(default)]
    pub action_catalog: Vec<ActionCatalogEntry>,
    pub timeout_budget_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionRequest {
    pub observation: ObservationEnvelope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_config_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_profile: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixture_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replay_id: Option<String>,
    pub timeout_budget_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum ProviderDecision {
    Wait,
    WaitTicks { ticks: u64 },
    Act { action_ref: String, action: Action },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderErrorEnvelope {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub retryable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProviderTokenUsage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_tokens: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderTranscriptEntry {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ProviderTraceEnvelope {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_summary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(default)]
    pub transcript: Vec<ProviderTranscriptEntry>,
    #[serde(default)]
    pub tool_trace: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<ProviderTokenUsage>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cost_cents: Option<u64>,
    #[serde(default)]
    pub schema_repair_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProviderDiagnostics {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
    #[serde(default)]
    pub retry_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryWriteIntent {
    pub scope: String,
    pub summary: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionResponse {
    pub decision: ProviderDecision,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_error: Option<ProviderErrorEnvelope>,
    #[serde(default)]
    pub diagnostics: ProviderDiagnostics,
    #[serde(default)]
    pub trace_payload: ProviderTraceEnvelope,
    #[serde(default)]
    pub memory_write_intents: Vec<MemoryWriteIntent>,
}

impl DecisionResponse {
    pub fn wait(provider_id: impl Into<String>) -> Self {
        let provider_id = provider_id.into();
        Self {
            decision: ProviderDecision::Wait,
            provider_error: None,
            diagnostics: ProviderDiagnostics {
                provider_id: Some(provider_id.clone()),
                ..ProviderDiagnostics::default()
            },
            trace_payload: ProviderTraceEnvelope {
                provider_id: Some(provider_id),
                output_summary: Some("decision=wait".to_string()),
                ..ProviderTraceEnvelope::default()
            },
            memory_write_intents: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedbackEnvelope {
    pub action_id: ActionId,
    pub success: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reject_reason: Option<String>,
    #[serde(default)]
    pub emitted_events: Vec<WorldEvent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub world_delta_summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GoldenDecisionFixture {
    pub fixture_id: String,
    pub request: DecisionRequest,
    pub expected_decision: ProviderDecision,
}

pub fn golden_decision_provider_fixtures() -> Vec<GoldenDecisionFixture> {
    let observation = Observation {
        time: 7,
        agent_id: "agent-1".to_string(),
        pos: GeoPos {
            x_cm: 0.0,
            y_cm: 0.0,
            z_cm: 0.0,
        },
        self_resources: {
            let mut stock = ResourceStock::default();
            let _ = stock.add(ResourceKind::Electricity, 24);
            let _ = stock.add(ResourceKind::Data, 8);
            stock
        },
        visibility_range_cm: 1_000,
        visible_agents: vec![ObservedAgent {
            agent_id: "agent-2".to_string(),
            location_id: "loc-2".to_string(),
            pos: GeoPos {
                x_cm: 100.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
            distance_cm: 100,
        }],
        visible_locations: vec![
            ObservedLocation {
                location_id: "loc-1".to_string(),
                name: "base".to_string(),
                pos: GeoPos {
                    x_cm: 0.0,
                    y_cm: 0.0,
                    z_cm: 0.0,
                },
                profile: Default::default(),
                distance_cm: 0,
            },
            ObservedLocation {
                location_id: "loc-2".to_string(),
                name: "neighbor".to_string(),
                pos: GeoPos {
                    x_cm: 100.0,
                    y_cm: 0.0,
                    z_cm: 0.0,
                },
                profile: Default::default(),
                distance_cm: 100,
            },
        ],
        module_lifecycle: Default::default(),
        module_market: Default::default(),
        power_market: Default::default(),
        social_state: Default::default(),
    };
    let action_catalog = vec![
        ActionCatalogEntry::new("wait", "Skip current tick without mutating world state"),
        ActionCatalogEntry::new(
            "move_agent",
            "Move the acting agent to a visible location via runtime validation",
        ),
    ];
    vec![GoldenDecisionFixture {
        fixture_id: "golden.move.visible_location.v1".to_string(),
        request: DecisionRequest {
            observation: ObservationEnvelope {
                agent_id: "agent-1".to_string(),
                world_time: observation.time,
                mode: ProviderExecutionMode::HeadlessAgent,
                observation_schema_version: DEFAULT_PROVIDER_OBSERVATION_SCHEMA_VERSION.to_string(),
                action_schema_version: DEFAULT_PROVIDER_ACTION_SCHEMA_VERSION.to_string(),
                environment_class: Some("golden_fixture".to_string()),
                fallback_reason: None,
                observation,
                recent_event_summary: vec![
                    "event: AgentRegistered(agent-1 @ loc-1)".to_string(),
                    "event: AgentRegistered(agent-2 @ loc-2)".to_string(),
                ],
                memory_summary: Some(
                    "goal=巡游移动; recent_failure=none; location_hint=loc-2 visible".to_string(),
                ),
                action_catalog,
                timeout_budget_ms: DEFAULT_PROVIDER_TIMEOUT_BUDGET_MS,
            },
            provider_config_ref: Some("golden/mock-provider".to_string()),
            agent_profile: None,
            fixture_id: Some("golden.move.visible_location.v1".to_string()),
            replay_id: None,
            timeout_budget_ms: DEFAULT_PROVIDER_TIMEOUT_BUDGET_MS,
        },
        expected_decision: ProviderDecision::Act {
            action_ref: "move_agent".to_string(),
            action: Action::MoveAgent {
                agent_id: "agent-1".to_string(),
                to: "loc-2".to_string(),
            },
        },
    }]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionProviderError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

impl DecisionProviderError {
    pub fn new(code: impl Into<String>, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            retryable,
        }
    }

    fn as_trace_message(&self) -> String {
        format!("{}: {}", self.code, self.message)
    }
}

impl fmt::Display for DecisionProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl Error for DecisionProviderError {}

pub trait DecisionProvider {
    fn provider_id(&self) -> &str;

    fn decide(
        &mut self,
        request: &DecisionRequest,
    ) -> Result<DecisionResponse, DecisionProviderError>;

    fn push_feedback(&mut self, _feedback: &FeedbackEnvelope) -> Result<(), DecisionProviderError> {
        Ok(())
    }

    fn on_world_event(&mut self, _event: &WorldEvent) -> Result<(), DecisionProviderError> {
        Ok(())
    }
}

pub struct ProviderBackedAgentBehavior<P: DecisionProvider> {
    agent_id: String,
    provider: P,
    action_catalog: Vec<ActionCatalogEntry>,
    provider_config_ref: Option<String>,
    agent_profile: Option<String>,
    execution_mode: ProviderExecutionMode,
    observation_schema_version: String,
    action_schema_version: String,
    environment_class: Option<String>,
    fallback_reason: Option<String>,
    fixture_id: Option<String>,
    replay_id: Option<String>,
    timeout_budget_ms: u64,
    memory_summary: Option<String>,
    recent_event_summary: VecDeque<String>,
    pending_trace: Option<AgentDecisionTrace>,
}

impl<P: DecisionProvider> ProviderBackedAgentBehavior<P> {
    pub fn new(
        agent_id: impl Into<String>,
        provider: P,
        action_catalog: Vec<ActionCatalogEntry>,
    ) -> Self {
        Self {
            agent_id: agent_id.into(),
            provider,
            action_catalog,
            provider_config_ref: None,
            agent_profile: None,
            execution_mode: ProviderExecutionMode::HeadlessAgent,
            observation_schema_version: DEFAULT_PROVIDER_OBSERVATION_SCHEMA_VERSION.to_string(),
            action_schema_version: DEFAULT_PROVIDER_ACTION_SCHEMA_VERSION.to_string(),
            environment_class: None,
            fallback_reason: None,
            fixture_id: None,
            replay_id: None,
            timeout_budget_ms: DEFAULT_PROVIDER_TIMEOUT_BUDGET_MS,
            memory_summary: None,
            recent_event_summary: VecDeque::new(),
            pending_trace: None,
        }
    }

    pub fn with_provider_config_ref(mut self, provider_config_ref: impl Into<String>) -> Self {
        self.provider_config_ref = Some(provider_config_ref.into());
        self
    }

    pub fn with_agent_profile(mut self, agent_profile: impl Into<String>) -> Self {
        self.agent_profile = Some(agent_profile.into());
        self
    }

    pub fn with_execution_mode(mut self, execution_mode: ProviderExecutionMode) -> Self {
        self.execution_mode = execution_mode;
        self
    }

    pub fn with_observation_schema_version(
        mut self,
        observation_schema_version: impl Into<String>,
    ) -> Self {
        self.observation_schema_version = observation_schema_version.into();
        self
    }

    pub fn with_action_schema_version(mut self, action_schema_version: impl Into<String>) -> Self {
        self.action_schema_version = action_schema_version.into();
        self
    }

    pub fn with_environment_class(mut self, environment_class: impl Into<String>) -> Self {
        self.environment_class = Some(environment_class.into());
        self
    }

    pub fn with_fallback_reason(mut self, fallback_reason: impl Into<String>) -> Self {
        self.fallback_reason = Some(fallback_reason.into());
        self
    }

    pub fn with_fixture_id(mut self, fixture_id: impl Into<String>) -> Self {
        self.fixture_id = Some(fixture_id.into());
        self
    }

    pub fn with_replay_id(mut self, replay_id: impl Into<String>) -> Self {
        self.replay_id = Some(replay_id.into());
        self
    }

    pub fn with_timeout_budget_ms(mut self, timeout_budget_ms: u64) -> Self {
        self.timeout_budget_ms = timeout_budget_ms.max(1);
        self
    }

    pub fn with_memory_summary(mut self, memory_summary: impl Into<String>) -> Self {
        self.memory_summary = Some(memory_summary.into());
        self
    }

    fn push_recent_event_summary(&mut self, summary: String) {
        if self.recent_event_summary.len() >= MAX_RECENT_EVENT_SUMMARIES {
            self.recent_event_summary.pop_front();
        }
        self.recent_event_summary.push_back(summary);
    }

    fn build_request(&self, observation: &Observation) -> DecisionRequest {
        DecisionRequest {
            observation: ObservationEnvelope {
                agent_id: self.agent_id.clone(),
                world_time: observation.time,
                mode: self.execution_mode,
                observation_schema_version: self.observation_schema_version.clone(),
                action_schema_version: self.action_schema_version.clone(),
                environment_class: self.environment_class.clone(),
                fallback_reason: self.fallback_reason.clone(),
                observation: observation.clone(),
                recent_event_summary: self.recent_event_summary.iter().cloned().collect(),
                memory_summary: self.memory_summary.clone(),
                action_catalog: self.action_catalog.clone(),
                timeout_budget_ms: self.timeout_budget_ms,
            },
            provider_config_ref: self.provider_config_ref.clone(),
            agent_profile: self.agent_profile.clone(),
            fixture_id: self.fixture_id.clone(),
            replay_id: self.replay_id.clone(),
            timeout_budget_ms: self.timeout_budget_ms,
        }
    }

    fn provider_error_to_trace(error: &DecisionProviderError) -> AgentDecisionTrace {
        AgentDecisionTrace {
            agent_id: String::new(),
            time: 0,
            decision: AgentDecision::Wait,
            llm_input: None,
            llm_output: None,
            llm_error: Some(error.as_trace_message()),
            parse_error: None,
            llm_diagnostics: Some(LlmDecisionDiagnostics {
                model: None,
                latency_ms: None,
                prompt_tokens: None,
                completion_tokens: None,
                total_tokens: None,
                retry_count: 0,
            }),
            llm_effect_intents: vec![],
            llm_effect_receipts: vec![],
            llm_step_trace: vec![],
            llm_prompt_section_trace: vec![],
            llm_chat_messages: vec![],
        }
    }

    fn response_to_trace(
        &self,
        observation: &Observation,
        request: &DecisionRequest,
        response: &DecisionResponse,
        decision: &AgentDecision,
        parse_error: Option<String>,
        provider_error: Option<String>,
    ) -> AgentDecisionTrace {
        let input_summary = response
            .trace_payload
            .input_summary
            .clone()
            .or_else(|| serde_json::to_string(request).ok());
        let output_summary = response
            .trace_payload
            .output_summary
            .clone()
            .or_else(|| serde_json::to_string(response).ok());
        let transcript = response
            .trace_payload
            .transcript
            .iter()
            .map(|entry| LlmChatMessageTrace {
                time: observation.time,
                agent_id: self.agent_id.clone(),
                role: match entry.role.as_str() {
                    "player" => LlmChatRole::Player,
                    "tool" => LlmChatRole::Tool,
                    "system" => LlmChatRole::System,
                    _ => LlmChatRole::Agent,
                },
                content: entry.content.clone(),
            })
            .collect();
        let step_trace = response
            .trace_payload
            .tool_trace
            .iter()
            .enumerate()
            .map(|(index, summary)| LlmStepTrace {
                step_index: index,
                step_type: "provider_tool_trace".to_string(),
                input_summary: summary.clone(),
                output_summary: summary.clone(),
                status: "ok".to_string(),
            })
            .collect();
        AgentDecisionTrace {
            agent_id: self.agent_id.clone(),
            time: observation.time,
            decision: decision.clone(),
            llm_input: input_summary,
            llm_output: output_summary,
            llm_error: provider_error,
            parse_error,
            llm_diagnostics: Some(LlmDecisionDiagnostics {
                model: response
                    .diagnostics
                    .provider_id
                    .clone()
                    .or_else(|| response.trace_payload.provider_id.clone()),
                latency_ms: response
                    .diagnostics
                    .latency_ms
                    .or(response.trace_payload.latency_ms),
                prompt_tokens: response
                    .trace_payload
                    .token_usage
                    .as_ref()
                    .and_then(|usage| usage.prompt_tokens),
                completion_tokens: response
                    .trace_payload
                    .token_usage
                    .as_ref()
                    .and_then(|usage| usage.completion_tokens),
                total_tokens: response
                    .trace_payload
                    .token_usage
                    .as_ref()
                    .and_then(|usage| usage.total_tokens),
                retry_count: response.diagnostics.retry_count,
            }),
            llm_effect_intents: vec![],
            llm_effect_receipts: vec![],
            llm_step_trace: step_trace,
            llm_prompt_section_trace: Vec::<LlmPromptSectionTrace>::new(),
            llm_chat_messages: transcript,
        }
    }

    fn feedback_from_action_result(result: &ActionResult) -> FeedbackEnvelope {
        FeedbackEnvelope {
            action_id: result.action_id,
            success: result.success,
            reject_reason: result.reject_reason().map(|reason| format!("{reason:?}")),
            emitted_events: vec![result.event.clone()],
            world_delta_summary: Some(format!(
                "action={:?}; success={}; event={:?}",
                result.action, result.success, result.event.kind
            )),
        }
    }
}

impl<P: DecisionProvider> AgentBehavior for ProviderBackedAgentBehavior<P> {
    fn agent_id(&self) -> &str {
        self.agent_id.as_str()
    }

    fn decide(&mut self, observation: &Observation) -> AgentDecision {
        let request = self.build_request(observation);
        let started_at = Instant::now();
        let response = match self.provider.decide(&request) {
            Ok(response) => response,
            Err(error) => {
                let mut trace = Self::provider_error_to_trace(&error);
                trace.agent_id = self.agent_id.clone();
                trace.time = observation.time;
                self.pending_trace = Some(trace);
                return AgentDecision::Wait;
            }
        };
        let latency_ms = started_at.elapsed().as_millis() as u64;
        let mut provider_error = response
            .provider_error
            .as_ref()
            .map(|error| format!("{}: {}", error.code, error.message));
        let (decision, parse_error) = match &response.decision {
            ProviderDecision::Wait => (AgentDecision::Wait, None),
            ProviderDecision::WaitTicks { ticks } => (AgentDecision::WaitTicks(*ticks), None),
            ProviderDecision::Act { action_ref, action } => {
                if self
                    .action_catalog
                    .iter()
                    .any(|entry| entry.action_ref == *action_ref)
                {
                    (AgentDecision::Act(action.clone()), None)
                } else {
                    provider_error = provider_error
                        .or_else(|| Some(format!("provider_invalid_action_ref: {action_ref}")));
                    (
                        AgentDecision::Wait,
                        Some(format!(
                            "unknown action_ref returned by provider: {action_ref}"
                        )),
                    )
                }
            }
        };
        let mut response = response;
        if response.diagnostics.latency_ms.is_none() {
            response.diagnostics.latency_ms = Some(latency_ms);
        }
        if response.trace_payload.latency_ms.is_none() {
            response.trace_payload.latency_ms = Some(latency_ms);
        }
        let trace = self.response_to_trace(
            observation,
            &request,
            &response,
            &decision,
            parse_error,
            provider_error,
        );
        self.pending_trace = Some(trace);
        decision
    }

    fn on_action_result(&mut self, result: &ActionResult) {
        let feedback = Self::feedback_from_action_result(result);
        let summary = feedback
            .world_delta_summary
            .clone()
            .unwrap_or_else(|| "action_feedback=unknown".to_string());
        self.push_recent_event_summary(summary.clone());
        self.memory_summary = Some(summary);
        let _ = self.provider.push_feedback(&feedback);
    }

    fn on_event(&mut self, event: &WorldEvent) {
        self.push_recent_event_summary(format!("event: {:?}", event.kind));
        let _ = self.provider.on_world_event(event);
    }

    fn take_decision_trace(&mut self) -> Option<AgentDecisionTrace> {
        self.pending_trace.take()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MockDecisionProviderState {
    pub recorded_requests: Vec<DecisionRequest>,
    pub recorded_feedback: Vec<FeedbackEnvelope>,
    pub recorded_events: Vec<WorldEvent>,
}

#[derive(Debug)]
pub struct MockDecisionProvider {
    provider_id: String,
    scripted_responses: VecDeque<Result<DecisionResponse, DecisionProviderError>>,
    shared_state: Arc<Mutex<MockDecisionProviderState>>,
}

impl MockDecisionProvider {
    pub fn new(provider_id: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
            scripted_responses: VecDeque::new(),
            shared_state: Arc::new(Mutex::new(MockDecisionProviderState::default())),
        }
    }

    pub fn with_scripted_responses(
        provider_id: impl Into<String>,
        scripted_responses: Vec<Result<DecisionResponse, DecisionProviderError>>,
    ) -> Self {
        Self {
            provider_id: provider_id.into(),
            scripted_responses: scripted_responses.into(),
            shared_state: Arc::new(Mutex::new(MockDecisionProviderState::default())),
        }
    }

    pub fn shared_state(&self) -> Arc<Mutex<MockDecisionProviderState>> {
        Arc::clone(&self.shared_state)
    }

    pub fn enqueue_response(&mut self, response: DecisionResponse) {
        self.scripted_responses.push_back(Ok(response));
    }

    pub fn enqueue_error(&mut self, error: DecisionProviderError) {
        self.scripted_responses.push_back(Err(error));
    }
}

impl DecisionProvider for MockDecisionProvider {
    fn provider_id(&self) -> &str {
        self.provider_id.as_str()
    }

    fn decide(
        &mut self,
        request: &DecisionRequest,
    ) -> Result<DecisionResponse, DecisionProviderError> {
        self.shared_state
            .lock()
            .expect("mock state lock")
            .recorded_requests
            .push(request.clone());
        match self.scripted_responses.pop_front() {
            Some(result) => result,
            None => Ok(DecisionResponse::wait(self.provider_id.clone())),
        }
    }

    fn push_feedback(&mut self, feedback: &FeedbackEnvelope) -> Result<(), DecisionProviderError> {
        self.shared_state
            .lock()
            .expect("mock state lock")
            .recorded_feedback
            .push(feedback.clone());
        Ok(())
    }

    fn on_world_event(&mut self, event: &WorldEvent) -> Result<(), DecisionProviderError> {
        self.shared_state
            .lock()
            .expect("mock state lock")
            .recorded_events
            .push(event.clone());
        Ok(())
    }
}
