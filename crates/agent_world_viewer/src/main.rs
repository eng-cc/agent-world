use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::thread;

use agent_world::geometry::GeoPos;
use agent_world::simulator::{
    AgentDecisionTrace, RunnerMetrics, SpaceConfig, WorldEvent, WorldSnapshot,
};
use agent_world::viewer::{
    ViewerControl, ViewerRequest, ViewerResponse, ViewerStream, VIEWER_PROTOCOL_VERSION,
};
use bevy::prelude::*;

const DEFAULT_ADDR: &str = "127.0.0.1:5010";
const DEFAULT_MAX_EVENTS: usize = 100;
const AGENT_BODY_MESH_RADIUS: f32 = 0.5;
const AGENT_BODY_MESH_LENGTH: f32 = 1.0;
const DEFAULT_2D_CAMERA_RADIUS: f32 = 90.0;
const DEFAULT_3D_CAMERA_RADIUS: f32 = 48.0;
const ORBIT_ROTATE_SENSITIVITY: f32 = 0.005;
const ORBIT_PAN_SENSITIVITY: f32 = 0.002;
const ORBIT_ZOOM_SENSITIVITY: f32 = 0.2;
const ORBIT_MIN_RADIUS: f32 = 4.0;
const ORBIT_MAX_RADIUS: f32 = 300.0;
const PICK_MAX_DISTANCE: f32 = 1.0;
const LABEL_FONT_SIZE: f32 = 18.0;
const LOCATION_LABEL_OFFSET: f32 = 0.8;
const AGENT_LABEL_OFFSET: f32 = 0.6;
const LABEL_SCALE: f32 = 0.03;
const UI_PANEL_WIDTH: f32 = 380.0;
mod app_bootstrap;
mod auto_focus;
mod button_feedback;
mod camera_controls;
mod copyable_text;
mod diagnosis;
mod egui_right_panel;
mod event_click_list;
mod floating_origin;
mod headless;
mod i18n;
mod internal_capture;
mod location_fragment_render;
mod material_library;
mod panel_layout;
mod panel_scroll;
mod right_panel_module_visibility;
mod scene_helpers;
mod selection_linking;
mod timeline_controls;
mod ui_locale_text;
mod ui_text;
mod viewer_3d_config;
mod world_overlay;

use app_bootstrap::{resolve_addr, resolve_offline, run_headless, run_ui};
use auto_focus::{
    apply_startup_auto_focus, auto_focus_config_from_env, handle_focus_selection_hotkey,
    AutoFocusState,
};
use button_feedback::{track_step_loading_state, StepControlLoadingState};
use camera_controls::{
    camera_orbit_preset, camera_projection_for_mode, orbit_camera_controls, sync_camera_mode,
    sync_world_background_visibility, OrbitDragState,
};
use copyable_text::CopyableTextPanelState;
use diagnosis::{spawn_diagnosis_panel, update_diagnosis_panel, DiagnosisState};
use egui_right_panel::render_right_side_panel_egui;
use event_click_list::{
    handle_event_click_buttons, spawn_event_click_list, update_event_click_list_ui,
};
use floating_origin::update_floating_origin;
use headless::headless_report;
use i18n::{control_button_label, locale_or_default, UiI18n};
use internal_capture::{
    internal_capture_config_from_env, trigger_internal_capture, InternalCaptureState,
};
use material_library::{
    build_fragment_element_material_handles, build_location_material_handles,
    FragmentElementMaterialHandles, LocationMaterialHandles,
};
use panel_layout::{spawn_top_panel_toggle, RightPanelLayoutState, TopPanelContainer};
use panel_scroll::{RightPanelScroll, TopPanelScroll};
use right_panel_module_visibility::{
    persist_right_panel_module_visibility, resolve_right_panel_module_visibility_resources,
};
use scene_helpers::*;
use selection_linking::{
    handle_jump_selection_events_button, handle_locate_focus_event_button, pick_3d_selection,
    spawn_event_object_link_controls, update_event_object_link_text, EventObjectLinkState,
};
use timeline_controls::{
    handle_control_buttons, handle_timeline_adjust_buttons, handle_timeline_bar_drag,
    handle_timeline_mark_filter_buttons, handle_timeline_mark_jump_buttons,
    handle_timeline_seek_submit, spawn_timeline_controls, sync_timeline_state_from_world,
    update_timeline_ui, TimelineMarkFilterState, TimelineUiState,
};
use ui_locale_text::{
    agents_activity_no_snapshot, details_click_to_inspect, events_empty, selection_line,
    status_line, summary_no_snapshot,
};
use ui_text::{agent_activity_summary, events_summary, selection_details_summary, world_summary};
use viewer_3d_config::{resolve_viewer_3d_config, Viewer3dConfig};
use world_overlay::{
    handle_world_overlay_toggle_buttons, spawn_world_overlay_controls,
    update_world_overlay_status_text, update_world_overlays_3d, world_overlay_config_from_env,
    WorldOverlayConfig, WorldOverlayUiState,
};

const WORLD_MIN_AXIS: f32 = 0.1;
const WORLD_FLOOR_THICKNESS: f32 = 0.03;
const WORLD_GRID_LINE_THICKNESS_2D: f32 = 0.008;
const WORLD_GRID_LINE_THICKNESS_3D: f32 = 0.014;
const CHUNK_GRID_LINE_THICKNESS_2D: f32 = 0.012;
const CHUNK_GRID_LINE_THICKNESS_3D: f32 = 0.022;

fn main() {
    let addr = resolve_addr();
    let headless = std::env::var("AGENT_WORLD_VIEWER_HEADLESS").is_ok();
    let offline = resolve_offline(headless);

    if headless {
        run_headless(addr, offline);
    } else {
        run_ui(addr, offline);
    }
}

#[derive(Resource)]
struct ViewerConfig {
    addr: String,
    max_events: usize,
}

#[derive(Resource, Default)]
struct OfflineConfig {
    offline: bool,
}

#[derive(Resource)]
struct ViewerClient {
    tx: Sender<ViewerRequest>,
    rx: Mutex<Receiver<ViewerResponse>>,
}

#[derive(Resource)]
struct ViewerState {
    status: ConnectionStatus,
    snapshot: Option<WorldSnapshot>,
    events: Vec<WorldEvent>,
    decision_traces: Vec<AgentDecisionTrace>,
    metrics: Option<RunnerMetrics>,
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            status: ConnectionStatus::Connecting,
            snapshot: None,
            events: Vec::new(),
            decision_traces: Vec::new(),
            metrics: None,
        }
    }
}

#[derive(Resource, Default)]
struct Viewer3dScene {
    root_entity: Option<Entity>,
    floating_origin_offset: Vec3,
    origin: Option<GeoPos>,
    space: Option<SpaceConfig>,
    last_snapshot_time: Option<u64>,
    last_event_id: Option<u64>,
    fragment_elements_visible: bool,
    agent_entities: HashMap<String, Entity>,
    agent_positions: HashMap<String, GeoPos>,
    agent_heights_cm: HashMap<String, i64>,
    agent_location_ids: HashMap<String, String>,
    agent_module_counts: HashMap<String, usize>,
    location_entities: HashMap<String, Entity>,
    asset_entities: HashMap<String, Entity>,
    module_visual_entities: HashMap<String, Entity>,
    power_plant_entities: HashMap<String, Entity>,
    power_storage_entities: HashMap<String, Entity>,
    chunk_entities: HashMap<String, Entity>,
    chunk_line_entities: HashMap<String, Vec<Entity>>,
    location_positions: HashMap<String, GeoPos>,
    location_radii_cm: HashMap<String, i64>,
    background_entities: Vec<Entity>,
    heat_overlay_entities: Vec<Entity>,
    flow_overlay_entities: Vec<Entity>,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
enum ViewerCameraMode {
    TwoD,
    ThreeD,
}

impl Default for ViewerCameraMode {
    fn default() -> Self {
        Self::TwoD
    }
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
enum ViewerPanelMode {
    Observe,
    PromptOps,
}

impl Default for ViewerPanelMode {
    fn default() -> Self {
        Self::Observe
    }
}

#[derive(Resource)]
struct Viewer3dAssets {
    agent_mesh: Handle<Mesh>,
    agent_material: Handle<StandardMaterial>,
    agent_module_marker_mesh: Handle<Mesh>,
    agent_module_marker_material: Handle<StandardMaterial>,
    location_mesh: Handle<Mesh>,
    location_material_library: LocationMaterialHandles,
    fragment_element_material_library: FragmentElementMaterialHandles,
    asset_mesh: Handle<Mesh>,
    asset_material: Handle<StandardMaterial>,
    power_plant_mesh: Handle<Mesh>,
    power_plant_material: Handle<StandardMaterial>,
    power_storage_mesh: Handle<Mesh>,
    power_storage_material: Handle<StandardMaterial>,
    chunk_unexplored_material: Handle<StandardMaterial>,
    chunk_generated_material: Handle<StandardMaterial>,
    chunk_exhausted_material: Handle<StandardMaterial>,
    world_box_mesh: Handle<Mesh>,
    world_floor_material: Handle<StandardMaterial>,
    world_bounds_material: Handle<StandardMaterial>,
    world_grid_material: Handle<StandardMaterial>,
    heat_low_material: Handle<StandardMaterial>,
    heat_mid_material: Handle<StandardMaterial>,
    heat_high_material: Handle<StandardMaterial>,
    flow_power_material: Handle<StandardMaterial>,
    flow_trade_material: Handle<StandardMaterial>,
    label_font: Handle<Font>,
}

#[derive(Resource, Default)]
struct ViewerSelection {
    current: Option<SelectionInfo>,
}

#[derive(Clone)]
struct SelectionInfo {
    entity: Entity,
    kind: SelectionKind,
    id: String,
    name: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectionKind {
    Agent,
    Location,
    Asset,
    PowerPlant,
    PowerStorage,
    Chunk,
}

impl ViewerSelection {
    fn clear(&mut self) {
        self.current = None;
    }
}

#[derive(Component)]
struct Viewer3dCamera;

#[derive(Component)]
struct Viewer3dSceneRoot;

#[derive(Component)]
struct WorldFloorSurface;

#[derive(Component)]
struct WorldBoundsSurface;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum GridLineKind {
    World,
    Chunk,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum GridLineAxis {
    AlongX,
    AlongZ,
}

#[derive(Component, Clone, Copy, Debug)]
struct GridLineVisual {
    kind: GridLineKind,
    axis: GridLineAxis,
    span: f32,
}

#[derive(Component)]
struct OrbitCamera {
    focus: Vec3,
    radius: f32,
    yaw: f32,
    pitch: f32,
}

impl OrbitCamera {
    fn apply_to_transform(&self, transform: &mut Transform) {
        let rotation =
            Quat::from_axis_angle(Vec3::Y, self.yaw) * Quat::from_axis_angle(Vec3::X, self.pitch);
        let offset = rotation * Vec3::new(0.0, 0.0, self.radius);
        transform.translation = self.focus + offset;
        transform.look_at(self.focus, Vec3::Y);
    }
}

fn grid_line_thickness(kind: GridLineKind, mode: ViewerCameraMode) -> f32 {
    match (kind, mode) {
        (GridLineKind::World, ViewerCameraMode::TwoD) => WORLD_GRID_LINE_THICKNESS_2D,
        (GridLineKind::World, ViewerCameraMode::ThreeD) => WORLD_GRID_LINE_THICKNESS_3D,
        (GridLineKind::Chunk, ViewerCameraMode::TwoD) => CHUNK_GRID_LINE_THICKNESS_2D,
        (GridLineKind::Chunk, ViewerCameraMode::ThreeD) => CHUNK_GRID_LINE_THICKNESS_3D,
    }
}

fn grid_line_scale(axis: GridLineAxis, span: f32, thickness: f32) -> Vec3 {
    match axis {
        GridLineAxis::AlongX => Vec3::new(span.max(thickness), thickness, thickness),
        GridLineAxis::AlongZ => Vec3::new(thickness, thickness, span.max(thickness)),
    }
}

#[derive(Component, Copy, Clone)]
struct BaseScale(Vec3);

#[derive(Debug, Clone, PartialEq, Eq)]
enum ConnectionStatus {
    Connecting,
    Connected,
    Error(String),
}

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct SummaryText;

#[derive(Component)]
struct EventsText;

#[derive(Component)]
struct SelectionText;

#[derive(Component)]
struct AgentActivityText;

#[derive(Component)]
struct SelectionDetailsText;

#[derive(Component, Clone)]
struct ControlButton {
    control: ViewerControl,
}

#[derive(Resource, Default)]
struct HeadlessStatus {
    last_status: Option<ConnectionStatus>,
    last_events: usize,
}

#[derive(Resource, Clone, Copy, Debug)]
struct RightPanelWidthState {
    width_px: f32,
}

impl Default for RightPanelWidthState {
    fn default() -> Self {
        Self {
            width_px: UI_PANEL_WIDTH,
        }
    }
}

fn setup_connection(mut commands: Commands, config: Res<ViewerConfig>) {
    let (tx, rx) = spawn_viewer_client(config.addr.clone());
    commands.insert_resource(ViewerClient {
        tx,
        rx: Mutex::new(rx),
    });
    commands.insert_resource(ViewerState::default());
}

fn setup_startup_state(commands: Commands, config: Res<OfflineConfig>, viewer: Res<ViewerConfig>) {
    if config.offline {
        setup_offline_state(commands);
    } else {
        setup_connection(commands, viewer);
    }
}

fn setup_offline_state(mut commands: Commands) {
    commands.insert_resource(ViewerState {
        status: ConnectionStatus::Error("offline mode".to_string()),
        ..ViewerState::default()
    });
}

fn spawn_viewer_client(addr: String) -> (Sender<ViewerRequest>, Receiver<ViewerResponse>) {
    let (tx_out, rx_out) = mpsc::channel::<ViewerRequest>();
    let (tx_in, rx_in) = mpsc::channel::<ViewerResponse>();

    thread::spawn(move || match TcpStream::connect(&addr) {
        Ok(stream) => {
            if let Err(err) = run_connection(stream, rx_out, tx_in.clone()) {
                let _ = tx_in.send(ViewerResponse::Error { message: err });
            }
        }
        Err(err) => {
            let _ = tx_in.send(ViewerResponse::Error {
                message: err.to_string(),
            });
        }
    });

    (tx_out, rx_in)
}

fn run_connection(
    stream: TcpStream,
    rx_out: Receiver<ViewerRequest>,
    tx_in: Sender<ViewerResponse>,
) -> Result<(), String> {
    stream.set_nodelay(true).map_err(|err| err.to_string())?;
    let reader_stream = stream.try_clone().map_err(|err| err.to_string())?;
    let mut writer = std::io::BufWriter::new(stream);

    send_request(
        &mut writer,
        &ViewerRequest::Hello {
            client: "bevy_viewer".to_string(),
            version: VIEWER_PROTOCOL_VERSION,
        },
    )?;
    send_request(
        &mut writer,
        &ViewerRequest::Subscribe {
            streams: vec![
                ViewerStream::Snapshot,
                ViewerStream::Events,
                ViewerStream::Metrics,
            ],
            event_kinds: Vec::new(),
        },
    )?;
    send_request(&mut writer, &ViewerRequest::RequestSnapshot)?;

    let reader_tx = tx_in.clone();
    thread::spawn(move || read_responses(reader_stream, reader_tx));

    for request in rx_out {
        send_request(&mut writer, &request)?;
    }

    Ok(())
}

fn read_responses(stream: TcpStream, tx_in: Sender<ViewerResponse>) {
    let mut reader = std::io::BufReader::new(stream);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match serde_json::from_str::<ViewerResponse>(trimmed) {
                    Ok(response) => {
                        let _ = tx_in.send(response);
                    }
                    Err(err) => {
                        let _ = tx_in.send(ViewerResponse::Error {
                            message: format!("decode error: {err}"),
                        });
                    }
                }
            }
            Err(err) => {
                let _ = tx_in.send(ViewerResponse::Error {
                    message: err.to_string(),
                });
                break;
            }
        }
    }
}

fn send_request(
    writer: &mut std::io::BufWriter<TcpStream>,
    request: &ViewerRequest,
) -> Result<(), String> {
    serde_json::to_writer(&mut *writer, request).map_err(|err| err.to_string())?;
    writer.write_all(b"\n").map_err(|err| err.to_string())?;
    writer.flush().map_err(|err| err.to_string())?;
    Ok(())
}

fn setup_3d_scene(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    camera_mode: Res<ViewerCameraMode>,
    mut scene: ResMut<Viewer3dScene>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let root_entity = commands
        .spawn((
            Transform::default(),
            GlobalTransform::default(),
            Viewer3dSceneRoot,
        ))
        .id();
    scene.root_entity = Some(root_entity);

    let label_font = asset_server.load("fonts/ms-yahei.ttf");
    let agent_mesh = meshes.add(Capsule3d::new(
        AGENT_BODY_MESH_RADIUS,
        AGENT_BODY_MESH_LENGTH,
    ));
    let agent_module_marker_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let location_mesh = meshes.add(Sphere::new(1.0));
    let asset_mesh = meshes.add(Cuboid::new(0.45, 0.45, 0.45));
    let power_plant_mesh = meshes.add(Cuboid::new(0.95, 0.7, 0.95));
    let power_storage_mesh = meshes.add(Cuboid::new(0.7, 1.0, 0.7));
    let world_box_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let agent_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.42, 0.22),
        perceptual_roughness: 0.38,
        metallic: 0.08,
        ..default()
    });
    let agent_module_marker_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.16, 0.92, 0.98),
        unlit: true,
        ..default()
    });
    let location_material_library = build_location_material_handles(&mut materials);
    let fragment_element_material_library = build_fragment_element_material_handles(&mut materials);
    let asset_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.82, 0.76, 0.34),
        perceptual_roughness: 0.55,
        ..default()
    });
    let power_plant_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.42, 0.2),
        perceptual_roughness: 0.5,
        ..default()
    });
    let power_storage_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.86, 0.48),
        perceptual_roughness: 0.45,
        ..default()
    });
    let chunk_unexplored_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.30, 0.42, 0.66, 0.22),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let chunk_generated_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.24, 0.78, 0.44, 0.30),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let chunk_exhausted_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.62, 0.40, 0.28, 0.30),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let world_floor_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.09, 0.11),
        unlit: true,
        ..default()
    });
    let world_bounds_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.22, 0.48, 0.65, 0.10),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let world_grid_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.29, 0.36, 0.48, 0.42),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let heat_low_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.24, 0.52, 0.88, 0.42),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let heat_mid_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.92, 0.62, 0.18, 0.48),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let heat_high_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.95, 0.26, 0.20, 0.55),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let flow_power_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.22, 0.72, 0.98, 0.62),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let flow_trade_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.92, 0.80, 0.26, 0.58),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.insert_resource(Viewer3dAssets {
        agent_mesh,
        agent_material,
        agent_module_marker_mesh,
        agent_module_marker_material,
        location_mesh,
        location_material_library,
        fragment_element_material_library,
        asset_mesh,
        asset_material,
        power_plant_mesh,
        power_plant_material,
        power_storage_mesh,
        power_storage_material,
        chunk_unexplored_material,
        chunk_generated_material,
        chunk_exhausted_material,
        world_box_mesh,
        world_floor_material,
        world_bounds_material,
        world_grid_material,
        heat_low_material,
        heat_mid_material,
        heat_high_material,
        flow_power_material,
        flow_trade_material,
        label_font,
    });

    let mode = *camera_mode;
    let orbit = camera_orbit_preset(mode, None, config.effective_cm_to_unit());
    let mut transform = Transform::default();
    orbit.apply_to_transform(&mut transform);
    let projection = camera_projection_for_mode(mode, &config);
    commands.spawn((
        Camera3d::default(),
        projection,
        Camera {
            order: 0,
            ..default()
        },
        transform,
        Viewer3dCamera,
        orbit,
    ));

    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.94, 0.97, 1.0),
        brightness: config.lighting.ambient_brightness,
        affects_lightmapped_meshes: true,
    });

    let illuminance = config.physical.exposed_illuminance_lux();

    commands.spawn((
        DirectionalLight {
            illuminance,
            shadows_enabled: config.lighting.shadows_enabled,
            ..default()
        },
        Transform::from_xyz(24.0, 36.0, 22.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: (illuminance * config.lighting.fill_light_ratio.max(0.0)).max(800.0),
            color: Color::srgb(0.74, 0.82, 0.92),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-18.0, 20.0, -28.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

#[allow(dead_code)]
fn setup_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    copyable_panel_state: Option<Res<CopyableTextPanelState>>,
) {
    let font = asset_server.load("fonts/ms-yahei.ttf");
    let i18n = UiI18n::default();
    let locale = i18n.locale;
    let copyable_panel_visible = copyable_panel_state
        .as_ref()
        .map(|state| state.visible)
        .unwrap_or(true);

    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
        IsDefaultUiCamera,
    ));

    commands
        .spawn((
            Node {
                width: Val::Px(UI_PANEL_WIDTH),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(8.0),
                flex_direction: FlexDirection::Column,
                border: UiRect::left(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.07, 0.08, 0.1)),
            BorderColor::all(Color::srgb(0.18, 0.2, 0.24)),
        ))
        .with_children(|root| {
            spawn_top_panel_toggle(root, font.clone(), locale, copyable_panel_visible);

            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(372.0),
                    min_height: Val::Px(260.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    overflow: Overflow::scroll_y(),
                    flex_shrink: 0.0,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.1, 0.11, 0.14)),
                BorderColor::all(Color::srgb(0.2, 0.22, 0.26)),
                ScrollPosition::default(),
                TopPanelContainer,
                TopPanelScroll,
            ))
            .with_children(|bar| {
                bar.spawn(Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(32.0),
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(6.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|controls| {
                    for (label, control) in [
                        (ViewerControl::Play, ViewerControl::Play),
                        (ViewerControl::Pause, ViewerControl::Pause),
                        (
                            ViewerControl::Step { count: 1 },
                            ViewerControl::Step { count: 1 },
                        ),
                        (
                            ViewerControl::Seek { tick: 0 },
                            ViewerControl::Seek { tick: 0 },
                        ),
                    ] {
                        controls
                            .spawn((
                                Button,
                                Node {
                                    padding: UiRect::horizontal(Val::Px(10.0)),
                                    height: Val::Px(28.0),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.2, 0.2, 0.24)),
                                ControlButton { control },
                            ))
                            .with_children(|button| {
                                button.spawn((
                                    Text::new(control_button_label(&label, locale)),
                                    TextFont {
                                        font: font.clone(),
                                        font_size: 14.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                });

                bar.spawn((
                    Text::new(status_line(&ConnectionStatus::Connecting, locale)),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    StatusText,
                ));

                bar.spawn((
                    Text::new(selection_line(&ViewerSelection::default(), locale)),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    SelectionText,
                ));

                spawn_world_overlay_controls(bar, font.clone(), locale);
                spawn_diagnosis_panel(bar, font.clone(), locale);
                spawn_event_object_link_controls(bar, font.clone(), locale);
                spawn_timeline_controls(bar, font.clone(), locale);
            });

            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    overflow: Overflow::scroll_y(),
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.08, 0.09, 0.11)),
                BorderColor::all(Color::srgb(0.2, 0.22, 0.26)),
                ScrollPosition::default(),
                RightPanelScroll,
            ))
            .with_children(|content| {
                content
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(140.0),
                            padding: UiRect::all(Val::Px(14.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.12, 0.13, 0.16)),
                        BorderColor::all(Color::srgb(0.24, 0.26, 0.32)),
                    ))
                    .with_children(|summary| {
                        summary.spawn((
                            Text::new(summary_no_snapshot(locale)),
                            TextFont {
                                font: font.clone(),
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                            SummaryText,
                        ));
                    });

                content
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(170.0),
                            padding: UiRect::all(Val::Px(14.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.1, 0.11, 0.14)),
                        BorderColor::all(Color::srgb(0.21, 0.23, 0.29)),
                    ))
                    .with_children(|activity| {
                        activity.spawn((
                            Text::new(agents_activity_no_snapshot(locale)),
                            TextFont {
                                font: font.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.86, 0.88, 0.92)),
                            AgentActivityText,
                        ));
                    });

                content
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(240.0),
                            padding: UiRect::all(Val::Px(14.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.09, 0.1, 0.13)),
                        BorderColor::all(Color::srgb(0.2, 0.22, 0.28)),
                    ))
                    .with_children(|details| {
                        details.spawn((
                            Text::new(details_click_to_inspect(locale)),
                            TextFont {
                                font: font.clone(),
                                font_size: 13.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.88, 0.9, 0.94)),
                            SelectionDetailsText,
                        ));
                    });

                content
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            min_height: Val::Px(260.0),
                            padding: UiRect::all(Val::Px(14.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.09, 0.1, 0.12)),
                        BorderColor::all(Color::srgb(0.2, 0.22, 0.28)),
                    ))
                    .with_children(|events| {
                        events.spawn((
                            Text::new(events_empty(locale)),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.85, 0.85, 0.85)),
                            EventsText,
                        ));

                        spawn_event_click_list(events, font.clone(), locale);
                    });
            });
        });
}

fn poll_viewer_messages(
    mut state: ResMut<ViewerState>,
    config: Res<ViewerConfig>,
    client: Option<Res<ViewerClient>>,
) {
    let Some(client) = client else {
        return;
    };
    let receiver = match client.rx.lock() {
        Ok(receiver) => receiver,
        Err(_) => {
            state.status = ConnectionStatus::Error("viewer receiver poisoned".to_string());
            return;
        }
    };

    loop {
        match receiver.try_recv() {
            Ok(message) => match message {
                ViewerResponse::HelloAck { .. } => {
                    state.status = ConnectionStatus::Connected;
                }
                ViewerResponse::Snapshot { snapshot } => {
                    state.snapshot = Some(snapshot);
                }
                ViewerResponse::Event { event } => {
                    state.events.push(event);
                    if state.events.len() > config.max_events {
                        let overflow = state.events.len() - config.max_events;
                        state.events.drain(0..overflow);
                    }
                }
                ViewerResponse::DecisionTrace { trace } => {
                    state.decision_traces.push(trace);
                    if state.decision_traces.len() > config.max_events {
                        let overflow = state.decision_traces.len() - config.max_events;
                        state.decision_traces.drain(0..overflow);
                    }
                }
                ViewerResponse::Metrics { metrics, .. } => {
                    state.metrics = Some(metrics);
                }
                ViewerResponse::Error { message } => {
                    state.status = ConnectionStatus::Error(message);
                }
            },
            Err(mpsc::TryRecvError::Empty) => break,
            Err(mpsc::TryRecvError::Disconnected) => {
                if !matches!(state.status, ConnectionStatus::Error(_)) {
                    state.status = ConnectionStatus::Error("disconnected".to_string());
                }
                break;
            }
        }
    }
}

fn update_3d_scene(
    mut commands: Commands,
    config: Res<Viewer3dConfig>,
    assets: Res<Viewer3dAssets>,
    mut scene: ResMut<Viewer3dScene>,
    mut selection: ResMut<ViewerSelection>,
    mut transforms: Query<(&mut Transform, Option<&BaseScale>)>,
    overlay_config: Res<WorldOverlayConfig>,
    state: Res<ViewerState>,
) {
    let Some(snapshot) = state.snapshot.as_ref() else {
        return;
    };

    let snapshot_time = snapshot.time;
    let snapshot_changed = scene.last_snapshot_time != Some(snapshot_time);
    let fragment_visibility_changed =
        scene.fragment_elements_visible != overlay_config.show_fragment_elements;
    if snapshot_changed || fragment_visibility_changed {
        rebuild_scene_from_snapshot(
            &mut commands,
            &config,
            &assets,
            &mut scene,
            snapshot,
            overlay_config.show_fragment_elements,
        );
        scene.last_snapshot_time = Some(snapshot_time);
        scene.last_event_id = None;
        scene.fragment_elements_visible = overlay_config.show_fragment_elements;
        selection.clear();
    }

    apply_events_to_scene(
        &mut commands,
        &config,
        &assets,
        &mut scene,
        snapshot_time,
        &state.events,
    );

    if config.highlight_selected {
        if let Some(current) = selection.current.as_ref() {
            apply_entity_highlight(&mut transforms, current.entity);
        }
    } else if let Some(current) = selection.current.as_ref() {
        reset_entity_scale(&mut transforms, current.entity);
    }
}

#[allow(dead_code)]
fn update_ui(
    state: Res<ViewerState>,
    selection: Res<ViewerSelection>,
    viewer_3d_config: Option<Res<Viewer3dConfig>>,
    i18n: Option<Res<UiI18n>>,
    timeline: Option<Res<TimelineUiState>>,
    mut queries: ParamSet<(
        Query<&mut Text, With<StatusText>>,
        Query<&mut Text, With<SummaryText>>,
        Query<&mut Text, With<EventsText>>,
        Query<&mut Text, With<SelectionText>>,
        Query<&mut Text, With<AgentActivityText>>,
        Query<&mut Text, With<SelectionDetailsText>>,
    )>,
) {
    let timeline_changed = timeline
        .as_ref()
        .map(|timeline| timeline.is_changed())
        .unwrap_or(false);
    let locale_changed = i18n
        .as_ref()
        .map(|locale| locale.is_changed())
        .unwrap_or(false);
    let physical_config_changed = viewer_3d_config
        .as_ref()
        .map(|config| config.is_changed())
        .unwrap_or(false);
    if !state.is_changed()
        && !selection.is_changed()
        && !timeline_changed
        && !locale_changed
        && !physical_config_changed
    {
        return;
    }

    let locale = locale_or_default(i18n.as_deref());

    if let Ok(mut text) = queries.p0().single_mut() {
        text.0 = status_line(&state.status, locale);
    }

    if let Ok(mut text) = queries.p1().single_mut() {
        text.0 = ui_locale_text::localize_world_summary_block(
            world_summary(
                state.snapshot.as_ref(),
                state.metrics.as_ref(),
                viewer_3d_config.as_deref().map(|cfg| &cfg.physical),
            ),
            locale,
        );
    }

    let focus_tick = timeline.as_ref().and_then(|timeline| {
        if timeline.manual_override || timeline.drag_active {
            Some(timeline.target_tick)
        } else {
            None
        }
    });

    if let Ok(mut text) = queries.p2().single_mut() {
        text.0 = ui_locale_text::localize_events_summary_block(
            events_summary(&state.events, focus_tick),
            locale,
        );
    }

    if let Ok(mut text) = queries.p3().single_mut() {
        text.0 = selection_line(&selection, locale);
    }

    if let Ok(mut text) = queries.p4().single_mut() {
        text.0 = ui_locale_text::localize_agent_activity_block(
            agent_activity_summary(state.snapshot.as_ref(), &state.events),
            locale,
        );
    }

    if let Ok(mut text) = queries.p5().single_mut() {
        let reference_radiation_area_m2 = viewer_3d_config
            .as_deref()
            .map(|config| config.physical.reference_radiation_area_m2)
            .unwrap_or(1.0);
        text.0 = ui_locale_text::localize_details_block(
            selection_details_summary(
                &selection,
                state.snapshot.as_ref(),
                &state.events,
                &state.decision_traces,
                reference_radiation_area_m2,
            ),
            locale,
        );
    }
}
fn update_3d_viewport(mut cameras: Query<&mut Camera, With<Viewer3dCamera>>) {
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };

    camera.viewport = None;
}

#[cfg(test)]
mod camera_mode_tests {
    use super::*;

    #[test]
    fn default_camera_mode_is_2d() {
        assert_eq!(ViewerCameraMode::default(), ViewerCameraMode::TwoD);
    }

    #[test]
    fn default_panel_mode_is_observe() {
        assert_eq!(ViewerPanelMode::default(), ViewerPanelMode::Observe);
    }
}
#[cfg(test)]
mod tests;
