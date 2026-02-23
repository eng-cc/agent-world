use super::egui_right_panel_player_experience::PlayerGuideStep;
use super::egui_right_panel_player_guide::{
    build_player_mission_loop_snapshot, player_mission_hud_anchor_y,
    player_mission_hud_compact_mode, PlayerGuideProgressSnapshot,
};

#[test]
fn build_player_mission_loop_snapshot_open_panel_requires_open_action() {
    let progress = PlayerGuideProgressSnapshot {
        connect_world_done: true,
        open_panel_done: false,
        select_target_done: false,
        explore_ready: false,
    };
    let snapshot = build_player_mission_loop_snapshot(
        PlayerGuideStep::OpenPanel,
        progress,
        crate::i18n::UiLocale::EnUs,
    );

    assert_eq!(snapshot.completed_steps, 1);
    assert_eq!(snapshot.objective, "Open Control Panel");
    assert_eq!(snapshot.action_label, "Open control panel");
    assert!(snapshot.action_opens_panel);
}

#[test]
fn build_player_mission_loop_snapshot_reports_progress_and_objective() {
    let progress = PlayerGuideProgressSnapshot {
        connect_world_done: true,
        open_panel_done: true,
        select_target_done: true,
        explore_ready: true,
    };
    let snapshot = build_player_mission_loop_snapshot(
        PlayerGuideStep::ExploreAction,
        progress,
        crate::i18n::UiLocale::EnUs,
    );

    assert_eq!(snapshot.completed_steps, 4);
    assert_eq!(snapshot.objective, "Advance The Run");
    assert_eq!(snapshot.action_label, "Run one key action");
    assert!(!snapshot.action_opens_panel);
}

#[test]
fn player_mission_hud_compact_mode_tracks_panel_visibility() {
    assert!(player_mission_hud_compact_mode(false));
    assert!(!player_mission_hud_compact_mode(true));
}

#[test]
fn player_mission_hud_anchor_avoids_onboarding_overlap() {
    assert_eq!(player_mission_hud_anchor_y(false, false), 96.0);
    assert_eq!(player_mission_hud_anchor_y(true, false), 136.0);
    assert_eq!(player_mission_hud_anchor_y(true, true), 214.0);
}
