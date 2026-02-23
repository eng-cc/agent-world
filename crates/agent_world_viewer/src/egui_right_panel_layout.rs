use bevy_egui::egui;

use crate::RightPanelLayoutState;

pub(super) fn sanitize_available_width(available_width: f32, fallback: f32) -> f32 {
    if available_width.is_finite() && available_width > 0.0 {
        available_width
    } else {
        fallback
    }
}

pub(super) fn adaptive_panel_max_width(available_width: f32) -> f32 {
    let width = sanitize_available_width(available_width, super::MAIN_PANEL_DEFAULT_WIDTH);
    (width * super::MAIN_PANEL_MAX_WIDTH_RATIO).max(super::MAIN_PANEL_MIN_WIDTH)
}

pub(super) fn adaptive_panel_default_width(available_width: f32) -> f32 {
    let width = sanitize_available_width(available_width, super::MAIN_PANEL_DEFAULT_WIDTH);
    (width * 0.22).clamp(super::MAIN_PANEL_MIN_WIDTH, adaptive_panel_max_width(width))
}

pub(super) fn adaptive_chat_panel_max_width(available_width: f32) -> f32 {
    let width = sanitize_available_width(available_width, super::CHAT_PANEL_DEFAULT_WIDTH);
    (width * super::CHAT_PANEL_MAX_WIDTH_RATIO).max(super::CHAT_PANEL_MIN_WIDTH)
}

pub(super) fn adaptive_chat_panel_default_width(available_width: f32) -> f32 {
    let width = sanitize_available_width(available_width, super::CHAT_PANEL_DEFAULT_WIDTH);
    (width * 0.25).clamp(
        super::CHAT_PANEL_MIN_WIDTH,
        adaptive_chat_panel_max_width(width),
    )
}

pub(super) fn is_compact_chat_layout(available_width: f32) -> bool {
    let width = sanitize_available_width(available_width, super::MAIN_PANEL_DEFAULT_WIDTH);
    width < super::CHAT_SIDE_PANEL_COMPACT_BREAKPOINT
}

pub(super) fn adaptive_main_panel_min_width(available_width: f32) -> f32 {
    if is_compact_chat_layout(available_width) {
        super::MAIN_PANEL_COMPACT_MIN_WIDTH
    } else {
        super::MAIN_PANEL_MIN_WIDTH
    }
}

fn max_total_right_panel_width_budget(available_width: f32) -> f32 {
    let width = sanitize_available_width(available_width, super::MAIN_PANEL_DEFAULT_WIDTH);
    (width - super::MIN_INTERACTION_VIEWPORT_WIDTH).max(0.0)
}

pub(super) fn adaptive_chat_panel_max_width_for_side_layout(available_width: f32) -> f32 {
    let layout_budget = max_total_right_panel_width_budget(available_width);
    let chat_budget = (layout_budget - super::MAIN_PANEL_MIN_WIDTH).max(0.0);
    adaptive_chat_panel_max_width(available_width).min(chat_budget)
}

pub(super) fn adaptive_main_panel_max_width_for_layout(
    available_width: f32,
    chat_panel_width: f32,
) -> f32 {
    let layout_budget = max_total_right_panel_width_budget(available_width);
    let panel_budget = (layout_budget - chat_panel_width).max(0.0);
    let panel_min_width = adaptive_main_panel_min_width(available_width);
    adaptive_panel_max_width(available_width).min(panel_budget.max(panel_min_width))
}

pub(super) fn player_main_panel_max_width_for_layout(
    available_width: f32,
    chat_panel_width: f32,
) -> f32 {
    let panel_min_width = adaptive_main_panel_min_width(available_width);
    let panel_budget_max =
        adaptive_main_panel_max_width_for_layout(available_width, chat_panel_width);
    let immersive_cap =
        (sanitize_available_width(available_width, super::MAIN_PANEL_DEFAULT_WIDTH) * 0.34)
            .max(panel_min_width);
    panel_budget_max.min(immersive_cap.max(panel_min_width))
}

pub(super) fn should_show_chat_panel(
    layout_state: &RightPanelLayoutState,
    show_chat: bool,
) -> bool {
    !layout_state.top_panel_collapsed && !layout_state.panel_hidden && show_chat
}

pub(super) fn total_right_panel_width(main_panel_width: f32, chat_panel_width: f32) -> f32 {
    main_panel_width.max(0.0) + chat_panel_width.max(0.0)
}

pub(super) fn panel_toggle_shortcut_pressed(context: &egui::Context) -> bool {
    let tab_pressed =
        context.input(|input| input.key_pressed(egui::Key::Tab) && !input.modifiers.any());
    if !tab_pressed {
        return false;
    }

    let chat_input_focused =
        context.memory(|memory| memory.has_focus(egui::Id::new(crate::EGUI_CHAT_INPUT_WIDGET_ID)));
    !chat_input_focused
}
