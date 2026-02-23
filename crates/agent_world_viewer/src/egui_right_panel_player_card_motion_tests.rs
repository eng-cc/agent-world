use super::egui_right_panel_player_card_motion::{
    build_player_card_transition_snapshot, sync_player_guide_transition, PlayerGuideTransitionState,
};
use super::egui_right_panel_player_experience::PlayerGuideStep;

#[test]
fn player_card_transition_fades_in_and_settles_slide_distance() {
    let mut transition = PlayerGuideTransitionState::default();
    sync_player_guide_transition(&mut transition, PlayerGuideStep::ConnectWorld, 12.0);

    let enter = build_player_card_transition_snapshot(
        &transition,
        PlayerGuideStep::ConnectWorld,
        12.0,
        1.0,
    );
    let settled = build_player_card_transition_snapshot(
        &transition,
        PlayerGuideStep::ConnectWorld,
        13.0,
        1.0,
    );

    assert!(enter.alpha < settled.alpha);
    assert!(enter.slide_px > settled.slide_px);
    assert!(settled.slide_px <= 0.05);
}

#[test]
fn player_card_transition_restarts_entry_when_guide_step_changes() {
    let mut transition = PlayerGuideTransitionState::default();
    sync_player_guide_transition(&mut transition, PlayerGuideStep::ConnectWorld, 3.0);
    let before =
        build_player_card_transition_snapshot(&transition, PlayerGuideStep::ConnectWorld, 4.2, 1.0);
    sync_player_guide_transition(&mut transition, PlayerGuideStep::OpenPanel, 4.2);
    let restarted =
        build_player_card_transition_snapshot(&transition, PlayerGuideStep::OpenPanel, 4.2, 1.0);

    assert!(before.slide_px < 1.0);
    assert!(restarted.slide_px > 8.0);
    assert!(restarted.alpha < before.alpha);
}
