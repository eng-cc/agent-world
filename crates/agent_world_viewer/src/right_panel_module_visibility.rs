use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

const MODULE_VISIBILITY_PATH_ENV: &str = "AGENT_WORLD_VIEWER_MODULE_VISIBILITY_PATH";
const MODULE_VISIBILITY_DIR: &str = ".agent_world_viewer";
const MODULE_VISIBILITY_FILE: &str = "right_panel_modules.json";
const MODULE_VISIBILITY_VERSION: u32 = 1;

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RightPanelModuleVisibilityState {
    pub show_controls: bool,
    pub show_overview: bool,
    pub show_overlay: bool,
    pub show_diagnosis: bool,
    pub show_event_link: bool,
    pub show_timeline: bool,
    pub show_details: bool,
}

impl Default for RightPanelModuleVisibilityState {
    fn default() -> Self {
        Self {
            show_controls: true,
            show_overview: true,
            show_overlay: true,
            show_diagnosis: true,
            show_event_link: true,
            show_timeline: true,
            show_details: true,
        }
    }
}

#[derive(Resource, Clone, Debug, PartialEq, Eq)]
pub(super) struct RightPanelModuleVisibilityPath {
    pub path: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct PersistedRightPanelModuleVisibility {
    #[serde(default = "persisted_version")]
    version: u32,
    #[serde(default = "default_visible")]
    show_controls: bool,
    #[serde(default = "default_visible")]
    show_overview: bool,
    #[serde(default = "default_visible")]
    show_overlay: bool,
    #[serde(default = "default_visible")]
    show_diagnosis: bool,
    #[serde(default = "default_visible")]
    show_event_link: bool,
    #[serde(default = "default_visible")]
    show_timeline: bool,
    #[serde(default = "default_visible")]
    show_details: bool,
}

impl Default for PersistedRightPanelModuleVisibility {
    fn default() -> Self {
        Self {
            version: MODULE_VISIBILITY_VERSION,
            show_controls: true,
            show_overview: true,
            show_overlay: true,
            show_diagnosis: true,
            show_event_link: true,
            show_timeline: true,
            show_details: true,
        }
    }
}

impl From<PersistedRightPanelModuleVisibility> for RightPanelModuleVisibilityState {
    fn from(value: PersistedRightPanelModuleVisibility) -> Self {
        Self {
            show_controls: value.show_controls,
            show_overview: value.show_overview,
            show_overlay: value.show_overlay,
            show_diagnosis: value.show_diagnosis,
            show_event_link: value.show_event_link,
            show_timeline: value.show_timeline,
            show_details: value.show_details,
        }
    }
}

impl From<RightPanelModuleVisibilityState> for PersistedRightPanelModuleVisibility {
    fn from(value: RightPanelModuleVisibilityState) -> Self {
        Self {
            show_controls: value.show_controls,
            show_overview: value.show_overview,
            show_overlay: value.show_overlay,
            show_diagnosis: value.show_diagnosis,
            show_event_link: value.show_event_link,
            show_timeline: value.show_timeline,
            show_details: value.show_details,
            ..Default::default()
        }
    }
}

fn default_visible() -> bool {
    true
}

fn persisted_version() -> u32 {
    MODULE_VISIBILITY_VERSION
}

pub(super) fn resolve_right_panel_module_visibility_resources() -> (
    RightPanelModuleVisibilityState,
    RightPanelModuleVisibilityPath,
) {
    let path = resolve_visibility_path_from(
        std::env::var(MODULE_VISIBILITY_PATH_ENV).ok(),
        std::env::var("HOME").ok(),
    );

    let state = load_right_panel_module_visibility(path.as_path()).unwrap_or_default();
    (state, RightPanelModuleVisibilityPath { path })
}

pub(super) fn persist_right_panel_module_visibility(
    state: Res<RightPanelModuleVisibilityState>,
    path: Res<RightPanelModuleVisibilityPath>,
    mut last_persisted: Local<Option<RightPanelModuleVisibilityState>>,
) {
    let current_state = *state.as_ref();

    if !state.is_changed() {
        *last_persisted = Some(current_state);
        return;
    }

    if last_persisted.is_none() {
        *last_persisted = Some(current_state);
        return;
    }

    let previous_state = last_persisted.expect("state exists");
    if previous_state == current_state {
        *last_persisted = Some(current_state);
        return;
    }

    let _ = persist_right_panel_module_visibility_to_file(state.as_ref(), path.path.as_path());
    *last_persisted = Some(current_state);
}

fn resolve_visibility_path_from(path_value: Option<String>, home_value: Option<String>) -> PathBuf {
    if let Some(path) = path_value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return PathBuf::from(path);
    }

    if let Some(home) = home_value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return PathBuf::from(home)
            .join(MODULE_VISIBILITY_DIR)
            .join(MODULE_VISIBILITY_FILE);
    }

    PathBuf::from(MODULE_VISIBILITY_DIR).join(MODULE_VISIBILITY_FILE)
}

fn load_right_panel_module_visibility(path: &Path) -> Option<RightPanelModuleVisibilityState> {
    let content = fs::read_to_string(path).ok()?;
    parse_right_panel_module_visibility(content.as_str())
}

fn parse_right_panel_module_visibility(content: &str) -> Option<RightPanelModuleVisibilityState> {
    if content.trim().is_empty() {
        return None;
    }

    let value = serde_json::from_str::<PersistedRightPanelModuleVisibility>(content).ok()?;
    Some(value.into())
}

fn persist_right_panel_module_visibility_to_file(
    state: &RightPanelModuleVisibilityState,
    path: &Path,
) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    let payload = serde_json::to_string_pretty(&PersistedRightPanelModuleVisibility::from(*state))
        .map_err(|error| io::Error::other(format!("serialize module visibility: {error}")))?;
    fs::write(path, payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn visibility_defaults_to_show_all_sections() {
        let state = RightPanelModuleVisibilityState::default();
        assert!(state.show_controls);
        assert!(state.show_overview);
        assert!(state.show_overlay);
        assert!(state.show_diagnosis);
        assert!(state.show_event_link);
        assert!(state.show_timeline);
        assert!(state.show_details);
    }

    #[test]
    fn resolve_visibility_path_prefers_env_path() {
        let path = resolve_visibility_path_from(
            Some(" .tmp/custom-panel-state.json ".to_string()),
            Some("/Users/tester".to_string()),
        );
        assert_eq!(path, PathBuf::from(".tmp/custom-panel-state.json"));
    }

    #[test]
    fn resolve_visibility_path_uses_home_when_env_not_set() {
        let path = resolve_visibility_path_from(None, Some("/Users/tester".to_string()));
        assert_eq!(
            path,
            PathBuf::from("/Users/tester")
                .join(MODULE_VISIBILITY_DIR)
                .join(MODULE_VISIBILITY_FILE)
        );
    }

    #[test]
    fn parse_visibility_defaults_missing_fields_to_visible() {
        let state = parse_right_panel_module_visibility(
            r#"{
  "version": 1,
  "show_controls": false
}"#,
        )
        .expect("parse visibility");

        assert!(!state.show_controls);
        assert!(state.show_overview);
        assert!(state.show_overlay);
        assert!(state.show_diagnosis);
        assert!(state.show_event_link);
        assert!(state.show_timeline);
        assert!(state.show_details);
    }

    #[test]
    fn persist_and_load_visibility_round_trip() {
        let base = unique_temp_dir("right_panel_modules");
        let path = base.join("nested").join("right_panel_modules.json");
        let state = RightPanelModuleVisibilityState {
            show_controls: true,
            show_overview: false,
            show_overlay: true,
            show_diagnosis: false,
            show_event_link: true,
            show_timeline: false,
            show_details: true,
        };

        persist_right_panel_module_visibility_to_file(&state, path.as_path())
            .expect("persist visibility state");
        let loaded = load_right_panel_module_visibility(path.as_path()).expect("load visibility");
        assert_eq!(loaded, state);

        fs::remove_dir_all(base).ok();
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("current time")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}_{}_{}", process::id(), ts))
    }
}
