use eframe::egui;

use super::{LaunchConfig, UiLanguage};

#[derive(Debug, Default)]
pub(crate) struct LlmSettingsPanel {
    open: bool,
}

impl LlmSettingsPanel {
    pub(crate) fn new(_path: impl Into<std::path::PathBuf>) -> Self {
        Self { open: false }
    }

    pub(crate) fn default_path() -> &'static str {
        "config.toml"
    }

    pub(crate) fn open(&mut self) {
        self.open = true;
    }

    pub(crate) fn show(
        &mut self,
        _ctx: &egui::Context,
        _language: UiLanguage,
        _config: &mut LaunchConfig,
    ) {
        // Web launcher currently focuses on process control API.
        self.open = false;
    }
}
