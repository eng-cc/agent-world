use super::egui_right_panel_player_entry::activate_player_command_entry;
use crate::right_panel_module_visibility::RightPanelModuleVisibilityState;
use crate::RightPanelLayoutState;

#[test]
fn activate_player_command_entry_opens_panel_and_switches_to_command_layout() {
    let mut layout = RightPanelLayoutState {
        top_panel_collapsed: true,
        panel_hidden: true,
    };
    let mut visibility = RightPanelModuleVisibilityState::default();

    activate_player_command_entry(&mut layout, &mut visibility);

    assert!(!layout.panel_hidden);
    assert!(!layout.top_panel_collapsed);
    assert!(visibility.show_chat);
    assert!(visibility.show_overview);
    assert!(!visibility.show_timeline);
    assert!(!visibility.show_details);
}
