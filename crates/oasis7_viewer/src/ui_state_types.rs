use bevy::prelude::*;

use super::UI_PANEL_WIDTH;
use oasis7::viewer::ViewerControl;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ConnectionStatus {
    Connecting,
    Connected,
    Error(String),
}

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

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RightPanelLayoutState {
    pub top_panel_collapsed: bool,
    pub panel_hidden: bool,
}
