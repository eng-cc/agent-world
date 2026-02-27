use super::egui_right_panel_player_experience::{
    build_player_stuck_hint, sync_player_stuck_hint_state, PlayerGuideStep, PlayerOnboardingState,
};

#[test]
fn stuck_hint_triggers_after_five_seconds_without_progress() {
    let mut onboarding = PlayerOnboardingState::default();
    let state = super::sample_viewer_state(crate::ConnectionStatus::Connected, Vec::new());

    assert_eq!(
        sync_player_stuck_hint_state(&mut onboarding, &state, 0.0),
        None
    );
    assert_eq!(
        sync_player_stuck_hint_state(&mut onboarding, &state, 3.0),
        None
    );
    let idle = sync_player_stuck_hint_state(&mut onboarding, &state, 6.0)
        .expect("stuck hint should trigger after threshold");
    assert!(idle >= 5.0);
}

#[test]
fn stuck_hint_clears_after_tick_progress() {
    let mut onboarding = PlayerOnboardingState::default();
    let state = super::sample_viewer_state(crate::ConnectionStatus::Connected, Vec::new());
    let _ = sync_player_stuck_hint_state(&mut onboarding, &state, 0.0);
    let _ = sync_player_stuck_hint_state(&mut onboarding, &state, 6.0);

    let mut progressed_state = state;
    progressed_state.metrics = Some(agent_world::simulator::RunnerMetrics {
        total_ticks: 1,
        ..agent_world::simulator::RunnerMetrics::default()
    });
    assert_eq!(
        sync_player_stuck_hint_state(&mut onboarding, &progressed_state, 7.0),
        None
    );
}

#[test]
fn build_player_stuck_hint_mentions_recovery_actions_for_explore_step() {
    let message = build_player_stuck_hint(
        PlayerGuideStep::ExploreAction,
        crate::i18n::UiLocale::EnUs,
        7.2,
    );
    assert!(message.contains("Do next step"));
    assert!(message.contains("Command Agent"));
}
