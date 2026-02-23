use bevy_egui::egui;

use crate::i18n::{
    panel_entry_hint_label, panel_toggle_shortcut_hint, right_panel_toggle_label, UiLocale,
};
use crate::right_panel_module_visibility::RightPanelModuleVisibilityState;
use crate::{RightPanelLayoutState, ViewerExperienceMode};

use super::egui_right_panel_player_experience::player_entry_card_style;
use super::egui_right_panel_player_guide::{apply_player_layout_preset, PlayerLayoutPreset};

fn player_edge_drawer_hint(locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        "边缘呼出"
    } else {
        "Edge Drawer"
    }
}

fn player_command_entry_button_label(locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        "直接指挥"
    } else {
        "Command Now"
    }
}

pub(super) fn activate_player_command_entry(
    layout_state: &mut RightPanelLayoutState,
    module_visibility: &mut RightPanelModuleVisibilityState,
) {
    apply_player_layout_preset(layout_state, module_visibility, PlayerLayoutPreset::Command);
}

pub(super) fn should_render_hidden_panel_top_entry(mode: ViewerExperienceMode) -> bool {
    mode != ViewerExperienceMode::Player
}

pub(super) fn render_hidden_panel_entry(
    context: &egui::Context,
    mode: ViewerExperienceMode,
    locale: UiLocale,
    now_secs: f64,
    layout_state: &mut RightPanelLayoutState,
    module_visibility: &mut RightPanelModuleVisibilityState,
) {
    let player_mode_enabled = mode == ViewerExperienceMode::Player;

    if should_render_hidden_panel_top_entry(mode) {
        egui::Area::new(egui::Id::new("viewer-right-panel-show-toggle"))
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-12.0, 12.0))
            .movable(false)
            .interactable(true)
            .show(context, |ui| {
                egui::Frame::group(ui.style())
                    .fill(egui::Color32::from_rgb(15, 19, 29))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(34, 40, 56)))
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::same(10))
                    .show(ui, |ui| {
                        ui.set_max_width(super::PANEL_ENTRY_CARD_MAX_WIDTH);
                        ui.strong(crate::i18n::experience_mode_label(mode, locale));
                        ui.label(panel_entry_hint_label(mode, locale));
                        ui.small(panel_toggle_shortcut_hint(locale));
                        if ui.button(right_panel_toggle_label(false, locale)).clicked() {
                            layout_state.panel_hidden = false;
                        }
                    });
            });
    }

    if !player_mode_enabled || should_render_hidden_panel_top_entry(mode) {
        return;
    }

    let (entry_fill, entry_stroke) = player_entry_card_style(now_secs);
    let pulse = ((now_secs * 1.2).sin() * 0.5 + 0.5) as f32;
    let stroke = egui::Stroke::new(
        1.0 + 0.45 * pulse,
        egui::Color32::from_rgba_unmultiplied(
            (74.0 + 26.0 * pulse).round() as u8,
            (128.0 + 32.0 * pulse).round() as u8,
            (178.0 + 24.0 * pulse).round() as u8,
            210,
        ),
    );

    egui::Area::new(egui::Id::new("viewer-right-panel-edge-drawer"))
        .anchor(egui::Align2::RIGHT_CENTER, egui::vec2(-2.0, 0.0))
        .movable(false)
        .interactable(true)
        .show(context, |ui| {
            egui::Frame::group(ui.style())
                .fill(entry_fill.gamma_multiply(0.9))
                .stroke(stroke)
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::same(6))
                .show(ui, |ui| {
                    ui.small(player_edge_drawer_hint(locale));
                    ui.small(panel_toggle_shortcut_hint(locale));
                    let command_tone =
                        egui::RichText::new(player_command_entry_button_label(locale))
                            .color(entry_stroke.color);
                    if ui
                        .small_button(if locale.is_zh() { "展开" } else { "Peek" })
                        .clicked()
                    {
                        layout_state.panel_hidden = false;
                    }
                    if ui.add(egui::Button::new(command_tone).small()).clicked() {
                        activate_player_command_entry(layout_state, module_visibility);
                    }
                });
        });
}
