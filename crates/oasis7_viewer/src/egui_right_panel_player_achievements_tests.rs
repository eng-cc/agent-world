use super::egui_right_panel_player_experience::sync_player_achievements;
use super::*;

#[test]
fn sync_player_achievements_unlocks_milestones_without_reentry() {
    let mut achievements = PlayerAchievementState::default();
    let mut state = sample_viewer_state(crate::ConnectionStatus::Connecting, Vec::new());
    let mut layout = RightPanelLayoutState {
        top_panel_collapsed: false,
        panel_hidden: true,
    };
    let mut selection = crate::ViewerSelection::default();

    sync_player_achievements(&mut achievements, &state, &selection, &layout, 1.0);
    assert_eq!(player_achievement_popup_len(&achievements), 0);

    state.status = crate::ConnectionStatus::Connected;
    sync_player_achievements(&mut achievements, &state, &selection, &layout, 2.0);
    assert!(player_achievement_is_unlocked(
        &achievements,
        PlayerAchievementMilestone::WorldConnected
    ));
    assert_eq!(
        player_achievement_popup_milestones(&achievements),
        vec![PlayerAchievementMilestone::WorldConnected]
    );

    sync_player_achievements(&mut achievements, &state, &selection, &layout, 2.1);
    assert_eq!(
        player_achievement_popup_milestones(&achievements),
        vec![PlayerAchievementMilestone::WorldConnected]
    );

    layout.panel_hidden = false;
    sync_player_achievements(&mut achievements, &state, &selection, &layout, 3.0);
    assert!(player_achievement_is_unlocked(
        &achievements,
        PlayerAchievementMilestone::PanelOpened
    ));

    selection = sample_selected_viewer_selection();
    sync_player_achievements(&mut achievements, &state, &selection, &layout, 4.0);
    assert!(player_achievement_is_unlocked(
        &achievements,
        PlayerAchievementMilestone::FirstSelection
    ));

    state.events.push(sample_agent_moved_event(10, 10));
    sync_player_achievements(&mut achievements, &state, &selection, &layout, 5.0);
    assert!(player_achievement_is_unlocked(
        &achievements,
        PlayerAchievementMilestone::FirstEventSeen
    ));
    assert_eq!(
        player_achievement_popup_len(&achievements),
        player_achievement_popup_cap()
    );
}

#[test]
fn sync_player_achievements_clamps_popup_queue_and_expires() {
    let mut achievements = PlayerAchievementState::default();
    let mut state = sample_viewer_state(
        crate::ConnectionStatus::Connected,
        vec![sample_agent_moved_event(10, 10)],
    );
    let layout = RightPanelLayoutState {
        top_panel_collapsed: false,
        panel_hidden: false,
    };
    let selection = sample_selected_viewer_selection();

    sync_player_achievements(&mut achievements, &state, &selection, &layout, 1.0);
    sync_player_achievements(&mut achievements, &state, &selection, &layout, 2.0);
    sync_player_achievements(&mut achievements, &state, &selection, &layout, 3.0);
    sync_player_achievements(&mut achievements, &state, &selection, &layout, 4.0);

    assert_eq!(
        player_achievement_popup_milestones(&achievements),
        vec![
            PlayerAchievementMilestone::PanelOpened,
            PlayerAchievementMilestone::FirstSelection,
            PlayerAchievementMilestone::FirstEventSeen
        ]
    );
    assert_eq!(
        player_achievement_popup_len(&achievements),
        player_achievement_popup_cap()
    );

    state.events.push(sample_rejected_event(11, 11));
    sync_player_achievements(&mut achievements, &state, &selection, &layout, 20.0);
    assert_eq!(player_achievement_popup_len(&achievements), 0);
}
