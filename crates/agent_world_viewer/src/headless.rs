use bevy::prelude::*;

use crate::ui_text::format_status;
use crate::{
    ConnectionStatus, HeadlessStatus, ViewerClient, ViewerControl, ViewerRequest, ViewerState,
};

const HEADLESS_AUTO_PLAY_ENV: &str = "AGENT_WORLD_VIEWER_HEADLESS_AUTO_PLAY";
const AUTO_PLAY_ENV: &str = "AGENT_WORLD_VIEWER_AUTO_PLAY";

pub(super) fn headless_auto_play_once(
    client: Option<Res<ViewerClient>>,
    state: Res<ViewerState>,
    mut sent: Local<bool>,
) {
    if *sent || !headless_auto_play_enabled() {
        return;
    }
    if !matches!(state.status, ConnectionStatus::Connected) {
        return;
    }
    let Some(client) = client else {
        return;
    };
    let _ = client.tx.send(ViewerRequest::Control {
        mode: ViewerControl::Play,
    });
    *sent = true;
}

pub(super) fn headless_report(mut status: ResMut<HeadlessStatus>, state: Res<ViewerState>) {
    if status
        .last_status
        .as_ref()
        .map(|last| last != &state.status)
        .unwrap_or(true)
    {
        eprintln!("viewer status: {}", format_status(&state.status));
        status.last_status = Some(state.status.clone());
    }

    if state.events.len() != status.last_events {
        eprintln!("viewer events: {}", state.events.len());
        status.last_events = state.events.len();
    }
}

fn headless_auto_play_enabled() -> bool {
    if let Some(value) = parse_bool_env(AUTO_PLAY_ENV) {
        return value;
    }
    if std::env::var("AGENT_WORLD_VIEWER_HEADLESS").is_ok() {
        return parse_bool_env(HEADLESS_AUTO_PLAY_ENV).unwrap_or(true);
    }
    false
}

fn parse_bool_env(key: &str) -> Option<bool> {
    let raw = std::env::var(key).ok()?;
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}
