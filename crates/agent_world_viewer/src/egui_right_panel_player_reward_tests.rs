use super::egui_right_panel_player_guide::{
    build_player_reward_feedback_snapshot, PlayerGuideProgressSnapshot,
};

#[test]
fn build_player_reward_feedback_snapshot_marks_completion_on_four_steps() {
    let progress = PlayerGuideProgressSnapshot {
        connect_world_done: true,
        open_panel_done: true,
        select_target_done: true,
        explore_ready: true,
    };

    let reward = build_player_reward_feedback_snapshot(progress, crate::i18n::UiLocale::EnUs);

    assert_eq!(reward.badge, "Reward");
    assert_eq!(reward.title, "Loop Completed");
    assert!(reward.complete);
}

#[test]
fn build_player_reward_feedback_snapshot_shows_momentum_before_completion() {
    let progress = PlayerGuideProgressSnapshot {
        connect_world_done: true,
        open_panel_done: true,
        select_target_done: false,
        explore_ready: false,
    };

    let reward = build_player_reward_feedback_snapshot(progress, crate::i18n::UiLocale::EnUs);

    assert_eq!(reward.badge, "Progress Reward");
    assert_eq!(reward.title, "Momentum Building");
    assert_eq!(
        reward.detail,
        "2/4 steps completed. Keep pushing to trigger completion feedback."
    );
    assert!(!reward.complete);
}
