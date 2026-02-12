use agent_world::viewer::{PromptControlAck, PromptControlError};
use bevy::prelude::*;

#[derive(Resource, Default, Debug, Clone)]
pub(super) struct PromptControlUiState {
    response_seq: u64,
    last_ack: Option<PromptControlAck>,
    last_error: Option<PromptControlError>,
}

impl PromptControlUiState {
    pub(super) fn response_seq(&self) -> u64 {
        self.response_seq
    }

    pub(super) fn last_ack(&self) -> Option<&PromptControlAck> {
        self.last_ack.as_ref()
    }

    pub(super) fn last_error(&self) -> Option<&PromptControlError> {
        self.last_error.as_ref()
    }

    pub(super) fn record_ack(&mut self, ack: PromptControlAck) {
        self.response_seq = self.response_seq.saturating_add(1);
        self.last_ack = Some(ack);
        self.last_error = None;
    }

    pub(super) fn record_error(&mut self, error: PromptControlError) {
        self.response_seq = self.response_seq.saturating_add(1);
        self.last_error = Some(error);
        self.last_ack = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_control_ui_state_tracks_latest_feedback() {
        let mut state = PromptControlUiState::default();
        state.record_ack(PromptControlAck {
            agent_id: "agent-0".to_string(),
            operation: agent_world::viewer::PromptControlOperation::Apply,
            preview: true,
            version: 2,
            updated_at_tick: 11,
            applied_fields: vec!["system_prompt_override".to_string()],
            digest: "abc".to_string(),
            rolled_back_to_version: None,
        });

        assert_eq!(state.response_seq(), 1);
        assert_eq!(state.last_ack().map(|ack| ack.version), Some(2));
        assert!(state.last_error().is_none());

        state.record_error(PromptControlError {
            code: "version_conflict".to_string(),
            message: "conflict".to_string(),
            agent_id: Some("agent-0".to_string()),
            current_version: Some(3),
        });

        assert_eq!(state.response_seq(), 2);
        assert_eq!(
            state.last_error().map(|err| err.code.as_str()),
            Some("version_conflict")
        );
        assert!(state.last_ack().is_none());
    }
}
