use bevy::prelude::*;

use crate::ui_text::format_status;
use crate::{HeadlessStatus, ViewerState};

pub(super) fn headless_report(mut status: ResMut<HeadlessStatus>, state: Res<ViewerState>) {
    if status
        .last_status
        .as_ref()
        .map(|last| last != &state.status)
        .unwrap_or(true)
    {
        println!("viewer status: {}", format_status(&state.status));
        status.last_status = Some(state.status.clone());
    }

    if state.events.len() != status.last_events {
        println!("viewer events: {}", state.events.len());
        status.last_events = state.events.len();
    }
}
