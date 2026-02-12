use super::*;
use agent_world::viewer::{
    PromptControlAck, PromptControlError, PromptControlOperation, ViewerRequest, ViewerResponse,
};

#[test]
fn poll_viewer_messages_records_prompt_control_ack() {
    let mut app = App::new();
    app.add_systems(Update, poll_viewer_messages);

    app.world_mut().insert_resource(ViewerConfig {
        addr: "127.0.0.1:0".to_string(),
        max_events: 16,
    });

    let (tx_response, rx_response) = mpsc::channel::<ViewerResponse>();
    let (tx_request, _) = mpsc::channel::<ViewerRequest>();
    app.world_mut().insert_resource(ViewerClient {
        tx: tx_request,
        rx: Mutex::new(rx_response),
    });
    app.world_mut().insert_resource(ViewerState {
        status: ConnectionStatus::Connected,
        ..ViewerState::default()
    });
    app.world_mut()
        .insert_resource(PromptControlUiState::default());

    tx_response
        .send(ViewerResponse::PromptControlAck {
            ack: PromptControlAck {
                agent_id: "agent-0".to_string(),
                operation: PromptControlOperation::Apply,
                preview: false,
                version: 3,
                updated_at_tick: 12,
                applied_fields: vec!["system_prompt_override".to_string()],
                digest: "abc123".to_string(),
                rolled_back_to_version: None,
            },
        })
        .expect("send ack");

    app.update();

    let prompt_state = app.world().resource::<PromptControlUiState>();
    assert_eq!(prompt_state.response_seq(), 1);
    assert_eq!(prompt_state.last_ack().map(|ack| ack.version), Some(3));
    assert!(prompt_state.last_error().is_none());
}

#[test]
fn poll_viewer_messages_records_prompt_control_error_without_connection_drop() {
    let mut app = App::new();
    app.add_systems(Update, poll_viewer_messages);

    app.world_mut().insert_resource(ViewerConfig {
        addr: "127.0.0.1:0".to_string(),
        max_events: 16,
    });

    let (tx_response, rx_response) = mpsc::channel::<ViewerResponse>();
    let (tx_request, _) = mpsc::channel::<ViewerRequest>();
    app.world_mut().insert_resource(ViewerClient {
        tx: tx_request,
        rx: Mutex::new(rx_response),
    });
    app.world_mut().insert_resource(ViewerState {
        status: ConnectionStatus::Connected,
        ..ViewerState::default()
    });
    app.world_mut()
        .insert_resource(PromptControlUiState::default());

    tx_response
        .send(ViewerResponse::PromptControlError {
            error: PromptControlError {
                code: "version_conflict".to_string(),
                message: "conflict".to_string(),
                agent_id: Some("agent-0".to_string()),
                current_version: Some(4),
            },
        })
        .expect("send error");

    app.update();

    let prompt_state = app.world().resource::<PromptControlUiState>();
    assert_eq!(prompt_state.response_seq(), 1);
    assert_eq!(
        prompt_state.last_error().map(|error| error.code.as_str()),
        Some("version_conflict")
    );
    assert!(prompt_state.last_ack().is_none());

    let viewer_state = app.world().resource::<ViewerState>();
    assert_eq!(viewer_state.status, ConnectionStatus::Connected);
}
