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
    Error {
        message: String,
    },
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
            },
        };
        let json = serde_json::to_string(&response).expect("serialize response");
        let parsed: ViewerResponse = serde_json::from_str(&json).expect("deserialize response");
        assert_eq!(parsed, response);
    }
}
