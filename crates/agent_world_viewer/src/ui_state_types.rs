use bevy::prelude::*;

use super::UI_PANEL_WIDTH;
use agent_world::viewer::ViewerControl;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ConnectionStatus {
    Connecting,
    Connected,
    Error(String),
}

#[derive(Component)]
pub(super) struct StatusText;

#[derive(Component)]
pub(super) struct SummaryText;

#[derive(Component)]
pub(super) struct EventsText;

#[derive(Component)]
pub(super) struct SelectionText;

#[derive(Component)]
pub(super) struct AgentActivityText;

#[derive(Component)]
pub(super) struct SelectionDetailsText;

#[derive(Component, Clone)]
pub(super) struct ControlButton {
    pub(super) control: ViewerControl,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource, Default)]
pub(super) struct HeadlessStatus {
    pub(super) last_status: Option<ConnectionStatus>,
    pub(super) last_events: usize,
}

#[derive(Resource, Clone, Copy, Debug)]
pub(super) struct RightPanelWidthState {
    pub(super) width_px: f32,
}

impl Default for RightPanelWidthState {
    fn default() -> Self {
        Self {
            width_px: UI_PANEL_WIDTH,
        }
    }
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub(super) struct ChatHistoryPanelWidthState {
    pub(super) width_px: f32,
}
