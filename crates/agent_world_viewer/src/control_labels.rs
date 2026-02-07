use agent_world::viewer::ViewerControl;
use bevy::prelude::*;

use crate::button_feedback::StepButtonLabel;
use crate::i18n::{control_button_label, UiI18n};

#[derive(Component, Clone)]
pub(super) struct ControlButtonLabel {
    pub control: ViewerControl,
}

pub(super) fn update_control_button_labels(
    i18n: Option<Res<UiI18n>>,
    mut query: Query<
        (&ControlButtonLabel, &mut Text),
        (With<ControlButtonLabel>, Without<StepButtonLabel>),
    >,
) {
    let Some(i18n) = i18n else {
        return;
    };
    if !i18n.is_changed() {
        return;
    }

    for (label, mut text) in &mut query {
        text.0 = control_button_label(&label.control, i18n.locale).to_string();
    }
}
