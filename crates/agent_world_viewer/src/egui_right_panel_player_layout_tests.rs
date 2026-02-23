use super::egui_right_panel_player_guide::{
    apply_player_layout_preset, resolve_player_layout_preset, PlayerLayoutPreset,
};
use crate::right_panel_module_visibility::RightPanelModuleVisibilityState;
use crate::RightPanelLayoutState;

#[test]
fn apply_player_layout_preset_command_opens_panel_and_enables_chat() {
    let mut layout = RightPanelLayoutState {
        top_panel_collapsed: true,
        panel_hidden: true,
    };
    let mut visibility = RightPanelModuleVisibilityState::default();

    apply_player_layout_preset(&mut layout, &mut visibility, PlayerLayoutPreset::Command);

    assert!(!layout.panel_hidden);
    assert!(!layout.top_panel_collapsed);
    assert!(visibility.show_chat);
    assert!(visibility.show_overview);
    assert!(!visibility.show_timeline);
    assert!(!visibility.show_details);
}

#[test]
fn apply_player_layout_preset_mission_reduces_non_essential_sections() {
    let mut layout = RightPanelLayoutState::default();
    let mut visibility = RightPanelModuleVisibilityState {
        show_controls: true,
        show_overview: true,
        show_chat: true,
        show_overlay: true,
        show_diagnosis: true,
        show_event_link: true,
        show_timeline: true,
        show_details: true,
    };

    apply_player_layout_preset(&mut layout, &mut visibility, PlayerLayoutPreset::Mission);

    assert!(!layout.panel_hidden);
    assert!(visibility.show_overview);
    assert!(!visibility.show_chat);
    assert!(!visibility.show_timeline);
    assert!(!visibility.show_details);
    assert!(!visibility.show_overlay);
    assert!(!visibility.show_diagnosis);
}

#[test]
fn resolve_player_layout_preset_tracks_command_and_intel_state() {
    let layout = RightPanelLayoutState {
        top_panel_collapsed: false,
        panel_hidden: false,
    };
    let command_visibility = RightPanelModuleVisibilityState {
        show_controls: false,
        show_overview: true,
        show_chat: true,
        show_overlay: false,
        show_diagnosis: false,
        show_event_link: true,
        show_timeline: false,
        show_details: false,
    };
    assert_eq!(
        resolve_player_layout_preset(&layout, &command_visibility),
        PlayerLayoutPreset::Command
    );

    let intel_visibility = RightPanelModuleVisibilityState {
        show_controls: false,
        show_overview: true,
        show_chat: false,
        show_overlay: false,
        show_diagnosis: false,
        show_event_link: true,
        show_timeline: true,
        show_details: true,
    };
    assert_eq!(
        resolve_player_layout_preset(&layout, &intel_visibility),
        PlayerLayoutPreset::Intel
    );
}
