use super::egui_right_panel_player_experience::PlayerGuideStep;
use super::egui_right_panel_player_guide::{
    build_player_mission_loop_snapshot, build_player_mission_remaining_hint,
    player_control_stage_label, player_mission_hud_anchor_y, player_mission_hud_compact_mode,
    player_mission_hud_minimap_reserved_bottom, player_mission_hud_show_command_action,
    player_mission_hud_show_minimap, PlayerGuideProgressSnapshot,
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
    assert_eq!(
        snapshot.objective,
        "Open the right control panel to unlock actions"
    );
    assert_eq!(
        snapshot.completion_condition,
        "Completion: right panel is visible"
    );
    assert_eq!(snapshot.eta, "ETA: about 5s");
    assert_eq!(snapshot.short_goals[0].label, "Open control panel");
    assert!(!snapshot.short_goals[0].complete);
    assert_eq!(snapshot.short_goals[1].label, "Lock one target");
    assert!(!snapshot.short_goals[1].complete);
    assert_eq!(snapshot.action_label, "Do next step: Open panel");
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
    assert_eq!(
        snapshot.objective,
        "Send one command and confirm new world feedback"
    );
    assert_eq!(
        snapshot.completion_condition,
        "Completion: at least one new world feedback appears"
    );
    assert_eq!(snapshot.eta, "ETA: about 20s");
    assert_eq!(snapshot.short_goals[0].label, "Send first order");
    assert!(snapshot.short_goals[0].complete);
    assert_eq!(snapshot.short_goals[1].label, "Confirm world feedback");
    assert!(snapshot.short_goals[1].complete);
    assert_eq!(snapshot.action_label, "Do next step: Open command and play");
    assert!(!snapshot.action_opens_panel);
}

#[test]
fn build_player_mission_remaining_hint_reports_tick_gap_after_feedback() {
    let progress = PlayerGuideProgressSnapshot {
        connect_world_done: true,
        open_panel_done: true,
        select_target_done: true,
        explore_ready: true,
    };
    let mut state = super::sample_viewer_state(crate::ConnectionStatus::Connected, Vec::new());
    state.metrics = Some(agent_world::simulator::RunnerMetrics {
        total_ticks: 12,
        ..agent_world::simulator::RunnerMetrics::default()
    });
    let hint = build_player_mission_remaining_hint(
        PlayerGuideStep::ExploreAction,
        progress,
        &state,
        crate::i18n::UiLocale::EnUs,
    );
    assert_eq!(hint, "Remaining: advance about 8 more ticks (goal tick=20)");
}

#[test]
fn build_player_mission_remaining_hint_reports_connection_waiting_message() {
    let progress = PlayerGuideProgressSnapshot {
        connect_world_done: false,
        open_panel_done: false,
        select_target_done: false,
        explore_ready: false,
    };
    let state = super::sample_viewer_state(crate::ConnectionStatus::Connecting, Vec::new());
    let hint = build_player_mission_remaining_hint(
        PlayerGuideStep::ConnectWorld,
        progress,
        &state,
        crate::i18n::UiLocale::EnUs,
    );
    assert_eq!(
        hint,
        "Remaining: wait until the status chip shows Connected"
    );
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

#[test]
fn player_mission_hud_command_action_only_visible_when_hidden() {
    assert!(player_mission_hud_show_command_action(true));
    assert!(!player_mission_hud_show_command_action(false));
}

#[test]
fn player_mission_hud_minimap_is_visible_only_in_world_first_mode() {
    assert!(player_mission_hud_show_minimap(true));
    assert!(!player_mission_hud_show_minimap(false));
}

#[test]
fn player_mission_hud_minimap_reserves_chatter_space() {
    assert_eq!(player_mission_hud_minimap_reserved_bottom(true), 188.0);
    assert_eq!(player_mission_hud_minimap_reserved_bottom(false), 0.0);
}

#[test]
fn player_control_stage_label_maps_core_states() {
    assert_eq!(
        player_control_stage_label("received", crate::i18n::UiLocale::EnUs),
        "Received"
    );
    assert_eq!(
        player_control_stage_label("blocked", crate::i18n::UiLocale::EnUs),
        "Blocked"
    );
    assert_eq!(
        player_control_stage_label("applied", crate::i18n::UiLocale::ZhCn),
        "已生效"
    );
}
