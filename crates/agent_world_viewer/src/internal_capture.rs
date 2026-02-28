use std::path::PathBuf;

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};

use super::{
    ConnectionStatus, OrbitCamera, SelectionKind, Viewer3dCamera, Viewer3dScene, ViewerCameraMode,
    ViewerSelection, ViewerState,
};

const CAPTURE_PATH_ENV: &str = "AGENT_WORLD_VIEWER_CAPTURE_PATH";
const CAPTURE_STATUS_PATH_ENV: &str = "AGENT_WORLD_VIEWER_CAPTURE_STATUS_PATH";
const CAPTURE_DELAY_SECS_ENV: &str = "AGENT_WORLD_VIEWER_CAPTURE_DELAY_SECS";
const CAPTURE_MAX_WAIT_SECS_ENV: &str = "AGENT_WORLD_VIEWER_CAPTURE_MAX_WAIT_SECS";
const DEFAULT_CAPTURE_DELAY_SECS: f64 = 2.0;
const DEFAULT_CAPTURE_MAX_WAIT_SECS: f64 = 15.0;

#[derive(Resource, Clone, Debug, PartialEq)]
pub(super) struct InternalCaptureConfig {
    pub path: Option<PathBuf>,
    pub status_path: Option<PathBuf>,
    pub delay_secs: f64,
    pub max_wait_secs: f64,
}

impl Default for InternalCaptureConfig {
    fn default() -> Self {
        Self {
            path: None,
            status_path: None,
            delay_secs: DEFAULT_CAPTURE_DELAY_SECS,
            max_wait_secs: DEFAULT_CAPTURE_MAX_WAIT_SECS,
        }
    }
}

#[derive(Resource, Default, Clone, Debug)]
pub(super) struct InternalCaptureState {
    start_elapsed_secs: Option<f64>,
    requested: bool,
    last_status_dump: Option<String>,
}

pub(super) fn internal_capture_config_from_env() -> InternalCaptureConfig {
    config_from_values(
        std::env::var(CAPTURE_PATH_ENV).ok(),
        std::env::var(CAPTURE_STATUS_PATH_ENV).ok(),
        std::env::var(CAPTURE_DELAY_SECS_ENV).ok(),
        std::env::var(CAPTURE_MAX_WAIT_SECS_ENV).ok(),
    )
}

pub(super) fn trigger_internal_capture(
    mut commands: Commands,
    time: Res<Time>,
    viewer_state: Res<ViewerState>,
    viewer_selection: Res<ViewerSelection>,
    camera_mode: Res<ViewerCameraMode>,
    scene: Res<Viewer3dScene>,
    camera_query: Query<&OrbitCamera, With<Viewer3dCamera>>,
    config: Res<InternalCaptureConfig>,
    mut capture_state: ResMut<InternalCaptureState>,
) {
    persist_capture_status(
        &config,
        &viewer_state,
        &viewer_selection,
        *camera_mode,
        &scene,
        &camera_query,
        &mut capture_state,
    );

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
    status_path_value: Option<String>,
    delay_secs_value: Option<String>,
    max_wait_secs_value: Option<String>,
) -> InternalCaptureConfig {
    let path = parse_optional_path(path_value);
    let status_path = parse_optional_path(status_path_value);
    let delay_secs = parse_seconds(delay_secs_value, DEFAULT_CAPTURE_DELAY_SECS);
    let max_wait_secs =
        parse_seconds(max_wait_secs_value, DEFAULT_CAPTURE_MAX_WAIT_SECS).max(delay_secs);

    InternalCaptureConfig {
        path,
        status_path,
        delay_secs,
        max_wait_secs,
    }
}

fn parse_optional_path(path_value: Option<String>) -> Option<PathBuf> {
    path_value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
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

fn persist_capture_status(
    config: &InternalCaptureConfig,
    viewer_state: &ViewerState,
    viewer_selection: &ViewerSelection,
    camera_mode: ViewerCameraMode,
    scene: &Viewer3dScene,
    camera_query: &Query<&OrbitCamera, With<Viewer3dCamera>>,
    capture_state: &mut InternalCaptureState,
) {
    let Some(status_path) = config.status_path.as_ref() else {
        return;
    };

    let status_dump = render_status_dump(
        viewer_state,
        viewer_selection,
        camera_mode,
        scene,
        camera_query,
    );
    if capture_state.last_status_dump.as_deref() == Some(status_dump.as_str()) {
        return;
    }

    if let Some(parent) = status_path.parent() {
        if let Err(err) = std::fs::create_dir_all(parent) {
            eprintln!(
                "viewer internal capture status mkdir failed: path={} err={err}",
                status_path.display()
            );
            return;
        }
    }

    if let Err(err) = std::fs::write(status_path, status_dump.as_bytes()) {
        eprintln!(
            "viewer internal capture status write failed: path={} err={err}",
            status_path.display()
        );
        return;
    }
    capture_state.last_status_dump = Some(status_dump);
}

fn render_status_dump(
    viewer_state: &ViewerState,
    viewer_selection: &ViewerSelection,
    camera_mode: ViewerCameraMode,
    scene: &Viewer3dScene,
    camera_query: &Query<&OrbitCamera, With<Viewer3dCamera>>,
) -> String {
    let (connection_status, last_error) = match &viewer_state.status {
        ConnectionStatus::Connected => ("connected", String::new()),
        ConnectionStatus::Connecting => ("connecting", String::new()),
        ConnectionStatus::Error(message) => ("error", sanitize_status_text(message)),
    };
    let snapshot_ready = if viewer_state.snapshot.is_some() {
        1
    } else {
        0
    };
    let (selection_kind, selection_id) = if let Some(selection) = viewer_selection.current.as_ref()
    {
        (
            selection_kind_label(selection.kind),
            sanitize_status_text(selection.id.as_str()),
        )
    } else {
        ("none", String::new())
    };
    let camera_mode_label = match camera_mode {
        ViewerCameraMode::TwoD => "2d",
        ViewerCameraMode::ThreeD => "3d",
    };
    let orbit_radius = camera_query
        .single()
        .map(|orbit| orbit.radius)
        .unwrap_or(-1.0);

    format!(
        "connection_status={connection_status}\nlast_error={last_error}\nsnapshot_ready={snapshot_ready}\nevent_count={}\ndecision_trace_count={}\nselection_kind={selection_kind}\nselection_id={selection_id}\ncamera_mode={camera_mode_label}\norbit_radius={orbit_radius:.6}\nscene_power_plant_count={}\nscene_power_storage_count={}\n",
        viewer_state.events.len(),
        viewer_state.decision_traces.len(),
        scene.power_plant_entities.len(),
        scene.power_storage_entities.len(),
    )
}

fn selection_kind_label(kind: SelectionKind) -> &'static str {
    match kind {
        SelectionKind::Agent => "agent",
        SelectionKind::Location => "location",
        SelectionKind::Fragment => "fragment",
        SelectionKind::Asset => "asset",
        SelectionKind::PowerPlant => "power_plant",
        SelectionKind::PowerStorage => "power_storage",
        SelectionKind::Chunk => "chunk",
    }
}

fn sanitize_status_text(raw: &str) -> String {
    raw.chars()
        .map(|ch| match ch {
            '\n' | '\r' => ' ',
            _ => ch,
        })
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_from_values_parses_and_normalizes_thresholds() {
        let config = config_from_values(
            Some("  .tmp/screens/window.png ".to_string()),
            Some(" .tmp/screens/capture_status.txt ".to_string()),
            Some("3.5".to_string()),
            Some("2".to_string()),
        );

        assert_eq!(config.path, Some(PathBuf::from(".tmp/screens/window.png")));
        assert_eq!(
            config.status_path,
            Some(PathBuf::from(".tmp/screens/capture_status.txt"))
        );
        assert!((config.delay_secs - 3.5).abs() < f64::EPSILON);
        assert!((config.max_wait_secs - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn config_from_values_falls_back_on_invalid_values() {
        let config = config_from_values(
            Some("   ".to_string()),
            Some(" ".to_string()),
            Some("-1".to_string()),
            Some("abc".to_string()),
        );

        assert_eq!(config.path, None);
        assert_eq!(config.status_path, None);
        assert!((config.delay_secs - DEFAULT_CAPTURE_DELAY_SECS).abs() < f64::EPSILON);
        assert!((config.max_wait_secs - DEFAULT_CAPTURE_MAX_WAIT_SECS).abs() < f64::EPSILON);
    }

    #[test]
    fn should_request_capture_requires_delay_and_snapshot_or_timeout() {
        let config = InternalCaptureConfig {
            path: Some(PathBuf::from("shot.png")),
            status_path: None,
            delay_secs: 2.0,
            max_wait_secs: 10.0,
        };

        assert!(!should_request_capture(&config, 1.9, true, false));
        assert!(!should_request_capture(&config, 2.1, false, false));
        assert!(should_request_capture(&config, 2.1, true, false));
        assert!(should_request_capture(&config, 10.0, false, false));
        assert!(!should_request_capture(&config, 10.0, true, true));
    }

    #[test]
    fn sanitize_status_text_replaces_newlines() {
        assert_eq!(
            sanitize_status_text("line1\nline2\rline3"),
            "line1 line2 line3"
        );
    }

    #[test]
    fn render_status_dump_includes_selection_and_scene_semantics() {
        let viewer_state = ViewerState::default();
        let viewer_selection = ViewerSelection {
            current: Some(super::super::SelectionInfo {
                entity: Entity::from_bits(7),
                kind: SelectionKind::PowerPlant,
                id: "plant-1".to_string(),
                name: None,
            }),
        };
        let camera_mode = ViewerCameraMode::ThreeD;
        let mut scene = Viewer3dScene::default();
        scene
            .power_plant_entities
            .insert("plant-1".to_string(), Entity::from_bits(11));
        scene
            .power_storage_entities
            .insert("storage-1".to_string(), Entity::from_bits(12));

        let mut app = App::new();
        app.world_mut().spawn((
            OrbitCamera {
                focus: Vec3::ZERO,
                radius: 23.5,
                yaw: 0.0,
                pitch: 0.0,
            },
            Viewer3dCamera,
        ));
        let mut camera_query_state = app
            .world_mut()
            .query_filtered::<&OrbitCamera, With<Viewer3dCamera>>();
        let camera_query = camera_query_state.query(app.world());

        let dump = render_status_dump(
            &viewer_state,
            &viewer_selection,
            camera_mode,
            &scene,
            &camera_query,
        );

        assert!(dump.contains("selection_kind=power_plant"));
        assert!(dump.contains("selection_id=plant-1"));
        assert!(dump.contains("camera_mode=3d"));
        assert!(dump.contains("orbit_radius=23.500000"));
        assert!(dump.contains("scene_power_plant_count=1"));
        assert!(dump.contains("scene_power_storage_count=1"));
    }
}
