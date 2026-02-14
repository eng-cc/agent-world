use agent_world_proto::viewer as proto;

use crate::simulator::{
    AgentDecisionTrace, RunnerMetrics, WorldEvent, WorldEventKind, WorldSnapshot, WorldTime,
};

pub use proto::{
    PromptControlApplyRequest, PromptControlCommand, PromptControlError, PromptControlOperation,
    PromptControlRollbackRequest, ViewerControl, ViewerEventKind, ViewerRequest, ViewerStream,
    VIEWER_PROTOCOL_VERSION,
};

pub type ViewerResponse =
    proto::ViewerResponse<WorldSnapshot, WorldEvent, AgentDecisionTrace, RunnerMetrics, WorldTime>;
pub type PromptControlAck = proto::PromptControlAck<WorldTime>;

pub fn viewer_event_kind_matches(filter: &ViewerEventKind, kind: &WorldEventKind) -> bool {
    match (filter, kind) {
        (ViewerEventKind::LocationRegistered, WorldEventKind::LocationRegistered { .. }) => true,
        (ViewerEventKind::AgentRegistered, WorldEventKind::AgentRegistered { .. }) => true,
        (ViewerEventKind::AgentMoved, WorldEventKind::AgentMoved { .. }) => true,
        (ViewerEventKind::ResourceTransferred, WorldEventKind::ResourceTransferred { .. }) => true,
        (ViewerEventKind::RadiationHarvested, WorldEventKind::RadiationHarvested { .. }) => true,
        (ViewerEventKind::ActionRejected, WorldEventKind::ActionRejected { .. }) => true,
        (ViewerEventKind::Power, WorldEventKind::Power(_)) => true,
        (ViewerEventKind::PromptUpdated, WorldEventKind::AgentPromptUpdated { .. }) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn viewer_event_kind_matches_world_event_kind() {
        assert!(viewer_event_kind_matches(
            &ViewerEventKind::AgentMoved,
            &WorldEventKind::AgentMoved {
                agent_id: "a1".to_string(),
                from: "loc-a".to_string(),
                to: "loc-b".to_string(),
                distance_cm: 100,
                electricity_cost: 1,
            },
        ));
        assert!(!viewer_event_kind_matches(
            &ViewerEventKind::PromptUpdated,
            &WorldEventKind::AgentMoved {
                agent_id: "a1".to_string(),
                from: "loc-a".to_string(),
                to: "loc-b".to_string(),
                distance_cm: 100,
                electricity_cost: 1,
            },
        ));
    }
}
