use bevy_egui::egui;

use crate::button_feedback::{mark_step_loading_on_control, StepControlLoadingState};
use crate::i18n::{
    advanced_debug_toggle_label, module_toggle_label, play_pause_toggle_label, step_button_label,
};
use crate::{
    dispatch_viewer_control, ViewerClient, ViewerControl, ViewerControlProfileState, ViewerState,
};

#[derive(Default)]
pub(crate) struct ControlPanelUiState {
    pub(super) playing: bool,
    pub(super) advanced_debug_expanded: bool,
}

pub(super) fn render_module_toggle_button(
    ui: &mut egui::Ui,
    module_key: &str,
    visible: &mut bool,
    locale: crate::i18n::UiLocale,
) {
    if ui
        .button(module_toggle_label(module_key, *visible, locale))
        .clicked()
    {
        *visible = !*visible;
    }
}

pub(super) fn render_control_buttons(
    ui: &mut egui::Ui,
    locale: crate::i18n::UiLocale,
    state: &ViewerState,
    loading: &mut StepControlLoadingState,
    control_ui: &mut ControlPanelUiState,
    client: Option<&ViewerClient>,
    control_profile: Option<&ViewerControlProfileState>,
) {
    ui.horizontal_wrapped(|ui| {
        let play_pause = if control_ui.playing {
            ViewerControl::Pause
        } else {
            ViewerControl::Play
        };
        if ui
            .button(play_pause_toggle_label(control_ui.playing, locale))
            .clicked()
        {
            send_control_request(
                play_pause,
                state,
                loading,
                control_ui,
                client,
                control_profile,
            );
        }

        if ui
            .button(advanced_debug_toggle_label(
                control_ui.advanced_debug_expanded,
                locale,
            ))
            .clicked()
        {
            control_ui.advanced_debug_expanded = !control_ui.advanced_debug_expanded;
        }
    });

    if !control_ui.advanced_debug_expanded {
        return;
    }

    ui.horizontal_wrapped(|ui| {
        let step_control = ViewerControl::Step { count: 1 };
        if ui
            .add_enabled(
                !loading.pending,
                egui::Button::new(step_button_label(locale, loading.pending)),
            )
            .clicked()
        {
            send_control_request(
                step_control,
                state,
                loading,
                control_ui,
                client,
                control_profile,
            );
        }
    });
}

pub(super) fn send_control_request(
    control: ViewerControl,
    state: &ViewerState,
    loading: &mut StepControlLoadingState,
    control_ui: &mut ControlPanelUiState,
    client: Option<&ViewerClient>,
    control_profile: Option<&ViewerControlProfileState>,
) {
    mark_step_loading_on_control(&control, state, loading);
    if let Some(client) = client {
        let _ = dispatch_viewer_control(client, control_profile, control.clone());
    }
    match control {
        ViewerControl::Play => control_ui.playing = true,
        ViewerControl::Pause | ViewerControl::Step { .. } | ViewerControl::Seek { .. } => {
            control_ui.playing = false;
        }
    }
}
