use serde::{Deserialize, Serialize};

use crate::simulator::{
    AgentDecisionTrace, RunnerMetrics, WorldEvent, WorldEventKind, WorldSnapshot, WorldTime,
};

pub const VIEWER_PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ViewerRequest {
    Hello {
        client: String,
        version: u32,
    },
    Subscribe {
        streams: Vec<ViewerStream>,
        #[serde(default)]
        event_kinds: Vec<ViewerEventKind>,
    },
    RequestSnapshot,
    Control {
        mode: ViewerControl,
    },
    PromptControl {
        command: PromptControlCommand,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum PromptControlCommand {
    Preview {
        request: PromptControlApplyRequest,
    },
    Apply {
        request: PromptControlApplyRequest,
    },
    Rollback {
        request: PromptControlRollbackRequest,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptControlApplyRequest {
    pub agent_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_version: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_override_field"
    )]
    pub system_prompt_override: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_override_field"
    )]
    pub short_term_goal_override: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_override_field"
    )]
    pub long_term_goal_override: Option<Option<String>>,
}

fn deserialize_override_field<'de, D>(deserializer: D) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    Ok(Some(value))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptControlRollbackRequest {
    pub agent_id: String,
    pub to_version: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_version: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewerStream {
    Snapshot,
    Events,
    Metrics,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewerEventKind {
    LocationRegistered,
    AgentRegistered,
    AgentMoved,
    ResourceTransferred,
    RadiationHarvested,
    ActionRejected,
    Power,
    PromptUpdated,
}

impl ViewerEventKind {
    pub fn matches(&self, kind: &WorldEventKind) -> bool {
        match (self, kind) {
            (ViewerEventKind::LocationRegistered, WorldEventKind::LocationRegistered { .. }) => {
                true
            }
            (ViewerEventKind::AgentRegistered, WorldEventKind::AgentRegistered { .. }) => true,
            (ViewerEventKind::AgentMoved, WorldEventKind::AgentMoved { .. }) => true,
            (ViewerEventKind::ResourceTransferred, WorldEventKind::ResourceTransferred { .. }) => {
                true
            }
            (ViewerEventKind::RadiationHarvested, WorldEventKind::RadiationHarvested { .. }) => {
                true
            }
            (ViewerEventKind::ActionRejected, WorldEventKind::ActionRejected { .. }) => true,
            (ViewerEventKind::Power, WorldEventKind::Power(_)) => true,
            (ViewerEventKind::PromptUpdated, WorldEventKind::AgentPromptUpdated { .. }) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum ViewerControl {
    Pause,
    Play,
    Step { count: usize },
    Seek { tick: WorldTime },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ViewerResponse {
    HelloAck {
        server: String,
        version: u32,
        world_id: String,
    },
    Snapshot {
        snapshot: WorldSnapshot,
    },
    Event {
        event: WorldEvent,
    },
    DecisionTrace {
        trace: AgentDecisionTrace,
    },
    Metrics {
        time: Option<WorldTime>,
        metrics: RunnerMetrics,
    },
    PromptControlAck {
        ack: PromptControlAck,
    },
    PromptControlError {
        error: PromptControlError,
    },
    Error {
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptControlOperation {
    Apply,
    Rollback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptControlAck {
    pub agent_id: String,
    pub operation: PromptControlOperation,
    pub preview: bool,
    pub version: u64,
    pub updated_at_tick: WorldTime,
    pub applied_fields: Vec<String>,
    pub digest: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rolled_back_to_version: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptControlError {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_version: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulator::{AgentDecision, LlmEffectIntentTrace, LlmEffectReceiptTrace};

    #[test]
    fn viewer_request_round_trip() {
        let request = ViewerRequest::Control {
            mode: ViewerControl::Step { count: 2 },
        };
        let json = serde_json::to_string(&request).expect("serialize request");
        let parsed: ViewerRequest = serde_json::from_str(&json).expect("deserialize request");
        assert_eq!(parsed, request);
    }

    #[test]
    fn viewer_subscribe_round_trip_with_filters() {
        let request = ViewerRequest::Subscribe {
            streams: vec![ViewerStream::Events],
            event_kinds: vec![ViewerEventKind::AgentMoved, ViewerEventKind::Power],
        };
        let json = serde_json::to_string(&request).expect("serialize subscribe");
        let parsed: ViewerRequest = serde_json::from_str(&json).expect("deserialize subscribe");
        assert_eq!(parsed, request);
    }

    #[test]
    fn viewer_response_round_trip_error() {
        let response = ViewerResponse::Error {
            message: "boom".to_string(),
        };
        let json = serde_json::to_string(&response).expect("serialize response");
        let parsed: ViewerResponse = serde_json::from_str(&json).expect("deserialize response");
        assert_eq!(parsed, response);
    }

    #[test]
    fn viewer_prompt_control_request_round_trip() {
        let request = ViewerRequest::PromptControl {
            command: PromptControlCommand::Apply {
                request: PromptControlApplyRequest {
                    agent_id: "agent-0".to_string(),
                    expected_version: Some(3),
                    updated_by: Some("tester".to_string()),
                    system_prompt_override: Some(Some("system".to_string())),
                    short_term_goal_override: Some(None),
                    long_term_goal_override: None,
                },
            },
        };
        let json = serde_json::to_string(&request).expect("serialize request");
        let parsed: ViewerRequest = serde_json::from_str(&json).expect("deserialize request");
        assert_eq!(parsed, request);
    }

    #[test]
    fn viewer_prompt_control_response_round_trip() {
        let response = ViewerResponse::PromptControlAck {
            ack: PromptControlAck {
                agent_id: "agent-0".to_string(),
                operation: PromptControlOperation::Rollback,
                preview: false,
                version: 7,
                updated_at_tick: 42,
                applied_fields: vec![
                    "system_prompt_override".to_string(),
                    "short_term_goal_override".to_string(),
                ],
                digest: "abc".to_string(),
                rolled_back_to_version: Some(5),
            },
        };
        let json = serde_json::to_string(&response).expect("serialize response");
        let parsed: ViewerResponse = serde_json::from_str(&json).expect("deserialize response");
        assert_eq!(parsed, response);
    }

    #[test]
    fn viewer_response_round_trip_decision_trace() {
        let response = ViewerResponse::DecisionTrace {
            trace: AgentDecisionTrace {
                agent_id: "agent-0".to_string(),
                time: 12,
                decision: AgentDecision::Wait,
                llm_input: Some("prompt".to_string()),
                llm_output: Some("{\"decision\":\"wait\"}".to_string()),
                llm_error: None,
                parse_error: None,
                llm_diagnostics: None,
                llm_effect_intents: vec![LlmEffectIntentTrace {
                    intent_id: "llm-intent-0".to_string(),
                    kind: "llm.prompt.module_call".to_string(),
                    params: serde_json::json!({
                        "module": "agent.modules.list",
                        "args": {},
                    }),
                    cap_ref: "llm.prompt.module_access".to_string(),
                    origin: "llm_agent".to_string(),
                }],
                llm_effect_receipts: vec![LlmEffectReceiptTrace {
                    intent_id: "llm-intent-0".to_string(),
                    status: "ok".to_string(),
                    payload: serde_json::json!({
                        "ok": true,
                    }),
                    cost_cents: None,
                }],
                llm_step_trace: vec![],
                llm_prompt_section_trace: vec![],
            },
        };
        let json = serde_json::to_string(&response).expect("serialize response");
        let parsed: ViewerResponse = serde_json::from_str(&json).expect("deserialize response");
        assert_eq!(parsed, response);
    }
}
