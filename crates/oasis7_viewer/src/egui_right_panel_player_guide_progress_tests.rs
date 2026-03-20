use super::egui_right_panel_player_experience::PlayerGuideStep;
use super::egui_right_panel_player_guide::build_player_guide_progress_snapshot;
use super::{sample_selected_viewer_selection, RightPanelLayoutState};

#[test]
fn build_player_guide_progress_snapshot_tracks_step_completion() {
    let hidden_layout = RightPanelLayoutState {
        top_panel_collapsed: false,
        panel_hidden: true,
    };
    let open_layout = RightPanelLayoutState {
        top_panel_collapsed: false,
        panel_hidden: false,
    };
    let empty_selection = crate::ViewerSelection::default();
    let selected = sample_selected_viewer_selection();

    let connecting = build_player_guide_progress_snapshot(
        &crate::ConnectionStatus::Connecting,
        &hidden_layout,
        &empty_selection,
        false,
    );
    assert_eq!(connecting.completed_steps(), 0);
    assert!(!connecting.is_step_complete(PlayerGuideStep::ConnectWorld));

    let connected_panel_hidden = build_player_guide_progress_snapshot(
        &crate::ConnectionStatus::Connected,
        &hidden_layout,
        &empty_selection,
        false,
    );
    assert_eq!(connected_panel_hidden.completed_steps(), 1);
    assert!(connected_panel_hidden.is_step_complete(PlayerGuideStep::ConnectWorld));
    assert!(!connected_panel_hidden.is_step_complete(PlayerGuideStep::OpenPanel));

    let connected_panel_open = build_player_guide_progress_snapshot(
        &crate::ConnectionStatus::Connected,
        &open_layout,
        &empty_selection,
        false,
    );
    assert_eq!(connected_panel_open.completed_steps(), 2);
    assert!(connected_panel_open.is_step_complete(PlayerGuideStep::OpenPanel));

    let selected_without_feedback = build_player_guide_progress_snapshot(
        &crate::ConnectionStatus::Connected,
        &open_layout,
        &selected,
        false,
    );
    assert_eq!(selected_without_feedback.completed_steps(), 3);
    assert!(selected_without_feedback.is_step_complete(PlayerGuideStep::SelectTarget));
    assert!(!selected_without_feedback.is_step_complete(PlayerGuideStep::ExploreAction));

    let explore_ready = build_player_guide_progress_snapshot(
        &crate::ConnectionStatus::Connected,
        &open_layout,
        &selected,
        true,
    );
    assert_eq!(explore_ready.completed_steps(), 4);
    assert!(explore_ready.is_step_complete(PlayerGuideStep::SelectTarget));
    assert!(explore_ready.is_step_complete(PlayerGuideStep::ExploreAction));
}
