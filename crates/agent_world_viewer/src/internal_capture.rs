use std::path::PathBuf;

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};

use super::ViewerState;

const CAPTURE_PATH_ENV: &str = "AGENT_WORLD_VIEWER_CAPTURE_PATH";
const CAPTURE_DELAY_SECS_ENV: &str = "AGENT_WORLD_VIEWER_CAPTURE_DELAY_SECS";
const CAPTURE_MAX_WAIT_SECS_ENV: &str = "AGENT_WORLD_VIEWER_CAPTURE_MAX_WAIT_SECS";
const DEFAULT_CAPTURE_DELAY_SECS: f64 = 2.0;
const DEFAULT_CAPTURE_MAX_WAIT_SECS: f64 = 15.0;

#[derive(Resource, Clone, Debug, PartialEq)]
pub(super) struct InternalCaptureConfig {
    pub path: Option<PathBuf>,
    pub delay_secs: f64,
    pub max_wait_secs: f64,
}

impl Default for InternalCaptureConfig {
    fn default() -> Self {
        Self {
            path: None,
            delay_secs: DEFAULT_CAPTURE_DELAY_SECS,
            max_wait_secs: DEFAULT_CAPTURE_MAX_WAIT_SECS,
        }
    }
}

#[derive(Resource, Default, Clone, Debug)]
pub(super) struct InternalCaptureState {
    start_elapsed_secs: Option<f64>,
    requested: bool,
}

pub(super) fn internal_capture_config_from_env() -> InternalCaptureConfig {
    config_from_values(
        std::env::var(CAPTURE_PATH_ENV).ok(),
        std::env::var(CAPTURE_DELAY_SECS_ENV).ok(),
        std::env::var(CAPTURE_MAX_WAIT_SECS_ENV).ok(),
    )
}

pub(super) fn trigger_internal_capture(
    mut commands: Commands,
    time: Res<Time>,
    viewer_state: Res<ViewerState>,
    config: Res<InternalCaptureConfig>,
    mut capture_state: ResMut<InternalCaptureState>,
) {
    let Some(output_path) = config.path.as_ref().cloned() else {
        return;
    };

    let start_elapsed_secs = capture_state
        .start_elapsed_secs
        .get_or_insert(time.elapsed_secs_f64());
    let elapsed_secs = (time.elapsed_secs_f64() - *start_elapsed_secs).max(0.0);
    let snapshot_ready = viewer_state.snapshot.is_some();

    if !should_request_capture(
        &config,
        elapsed_secs,
        snapshot_ready,
        capture_state.requested,
    ) {
        return;
    }

    capture_state.requested = true;
    commands.spawn(Screenshot::primary_window()).observe(
        move |captured: On<ScreenshotCaptured>, mut app_exit: MessageWriter<AppExit>| {
            save_to_disk(output_path.clone())(captured);
            app_exit.write(AppExit::Success);
        },
    );
}

fn config_from_values(
    path_value: Option<String>,
    delay_secs_value: Option<String>,
    max_wait_secs_value: Option<String>,
) -> InternalCaptureConfig {
    let path = path_value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let delay_secs = parse_seconds(delay_secs_value, DEFAULT_CAPTURE_DELAY_SECS);
    let max_wait_secs =
        parse_seconds(max_wait_secs_value, DEFAULT_CAPTURE_MAX_WAIT_SECS).max(delay_secs);

    InternalCaptureConfig {
        path,
        delay_secs,
        max_wait_secs,
    }
}

fn parse_seconds(value: Option<String>, default_value: f64) -> f64 {
    value
        .and_then(|raw| raw.trim().parse::<f64>().ok())
        .filter(|parsed| parsed.is_finite() && *parsed >= 0.0)
        .unwrap_or(default_value)
}

fn should_request_capture(
    config: &InternalCaptureConfig,
    elapsed_secs: f64,
    snapshot_ready: bool,
    already_requested: bool,
) -> bool {
    if already_requested || config.path.is_none() {
        return false;
    }
    if elapsed_secs < config.delay_secs {
        return false;
    }
    snapshot_ready || elapsed_secs >= config.max_wait_secs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_from_values_parses_and_normalizes_thresholds() {
        let config = config_from_values(
            Some("  .tmp/screens/window.png ".to_string()),
            Some("3.5".to_string()),
            Some("2".to_string()),
        );

        assert_eq!(config.path, Some(PathBuf::from(".tmp/screens/window.png")));
        assert!((config.delay_secs - 3.5).abs() < f64::EPSILON);
        assert!((config.max_wait_secs - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn config_from_values_falls_back_on_invalid_values() {
        let config = config_from_values(
            Some("   ".to_string()),
            Some("-1".to_string()),
            Some("abc".to_string()),
        );

        assert_eq!(config.path, None);
        assert!((config.delay_secs - DEFAULT_CAPTURE_DELAY_SECS).abs() < f64::EPSILON);
        assert!((config.max_wait_secs - DEFAULT_CAPTURE_MAX_WAIT_SECS).abs() < f64::EPSILON);
    }

    #[test]
    fn should_request_capture_requires_delay_and_snapshot_or_timeout() {
        let config = InternalCaptureConfig {
            path: Some(PathBuf::from("shot.png")),
            delay_secs: 2.0,
            max_wait_secs: 10.0,
        };

        assert!(!should_request_capture(&config, 1.9, true, false));
        assert!(!should_request_capture(&config, 2.1, false, false));
        assert!(should_request_capture(&config, 2.1, true, false));
        assert!(should_request_capture(&config, 10.0, false, false));
        assert!(!should_request_capture(&config, 10.0, true, true));
    }
}
