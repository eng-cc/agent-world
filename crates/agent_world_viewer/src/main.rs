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
use bevy::camera::Viewport;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

const DEFAULT_ADDR: &str = "127.0.0.1:5010";
const DEFAULT_MAX_EVENTS: usize = 100;
const DEFAULT_CM_TO_UNIT: f32 = 0.00001;
const DEFAULT_AGENT_RADIUS: f32 = 0.35;
const DEFAULT_LOCATION_SIZE: f32 = 1.2;
const ORBIT_ROTATE_SENSITIVITY: f32 = 0.005;
const ORBIT_PAN_SENSITIVITY: f32 = 0.002;
const ORBIT_ZOOM_SENSITIVITY: f32 = 0.2;
const ORBIT_MIN_RADIUS: f32 = 4.0;
const ORBIT_MAX_RADIUS: f32 = 300.0;
const PICK_MAX_DISTANCE: f32 = 1.0;
const CHUNK_PICK_MAX_DISTANCE: f32 = 1.2;
const LABEL_FONT_SIZE: f32 = 18.0;
const LOCATION_LABEL_OFFSET: f32 = 0.8;
const AGENT_LABEL_OFFSET: f32 = 0.6;
const LABEL_SCALE: f32 = 0.03;
const UI_PANEL_WIDTH: f32 = 380.0;
mod camera_controls;
mod diagnosis;
mod event_click_list;
mod headless;
mod scene_helpers;
mod selection_linking;
mod timeline_controls;
mod ui_text;
mod world_overlay;

use camera_controls::{orbit_camera_controls, OrbitDragState};
use diagnosis::{spawn_diagnosis_panel, update_diagnosis_panel, DiagnosisState};
use event_click_list::{
    handle_event_click_buttons, spawn_event_click_list, update_event_click_list_ui,
};
use headless::headless_report;
use scene_helpers::*;
use selection_linking::{
    handle_jump_selection_events_button, handle_locate_focus_event_button, pick_3d_selection,
    spawn_event_object_link_controls, update_event_object_link_text, EventObjectLinkState,
};
use timeline_controls::{
    handle_control_buttons, handle_timeline_adjust_buttons, handle_timeline_bar_drag,
    handle_timeline_mark_filter_buttons, handle_timeline_mark_jump_buttons,
    handle_timeline_seek_submit, spawn_timeline_controls, sync_timeline_state_from_world,
    update_timeline_mark_filter_ui, update_timeline_ui, TimelineMarkFilterState, TimelineUiState,
};
use ui_text::{
    agent_activity_summary, events_summary, format_status, selection_details_summary, world_summary,
};
use world_overlay::{
    handle_world_overlay_toggle_buttons, spawn_world_overlay_controls,
    update_world_overlay_status_text, update_world_overlays_3d, WorldOverlayConfig,
    WorldOverlayUiState,
};

const WORLD_MIN_AXIS: f32 = 0.1;
const WORLD_FLOOR_THICKNESS: f32 = 0.03;
const WORLD_GRID_LINE_THICKNESS: f32 = 0.01;
const WORLD_GRID_LINES_PER_AXIS: usize = 8;

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

fn run_ui(addr: String, offline: bool) {
    App::new()
        .insert_resource(ViewerConfig {
            addr,
            max_events: DEFAULT_MAX_EVENTS,
        })
        .insert_resource(Viewer3dConfig::default())
        .insert_resource(Viewer3dScene::default())
        .insert_resource(ViewerSelection::default())
        .insert_resource(WorldOverlayConfig::default())
        .insert_resource(WorldOverlayUiState::default())
        .insert_resource(DiagnosisState::default())
        .insert_resource(EventObjectLinkState::default())
        .insert_resource(TimelineUiState::default())
        .insert_resource(TimelineMarkFilterState::default())
        .insert_resource(OrbitDragState::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Agent World Viewer".to_string(),
                resolution: (1200, 800).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(OfflineConfig { offline })
        .add_systems(Startup, (setup_startup_state, setup_3d_scene, setup_ui))
        .add_systems(
            Update,
            (
                poll_viewer_messages,
                sync_timeline_state_from_world,
                handle_timeline_adjust_buttons,
                handle_timeline_mark_filter_buttons,
                update_timeline_mark_filter_ui,
                handle_timeline_bar_drag,
                handle_timeline_mark_jump_buttons,
                handle_timeline_seek_submit,
                handle_world_overlay_toggle_buttons,
                handle_event_click_buttons,
                handle_locate_focus_event_button,
                handle_jump_selection_events_button,
                update_event_object_link_text,
                update_world_overlay_status_text,
                update_diagnosis_panel,
                update_event_click_list_ui,
                update_timeline_ui,
                scroll_right_panel,
                update_ui,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                update_3d_scene,
                update_world_overlays_3d.after(update_3d_scene),
                orbit_camera_controls,
                update_3d_viewport,
                handle_control_buttons,
            ),
        )
        .add_systems(
            PostUpdate,
            pick_3d_selection.after(TransformSystems::Propagate),
        )
        .run();
}

fn run_headless(addr: String, offline: bool) {
    App::new()
        .insert_resource(ViewerConfig {
            addr,
            max_events: DEFAULT_MAX_EVENTS,
        })
        .insert_resource(HeadlessStatus::default())
        .insert_resource(OfflineConfig { offline })
        .add_plugins(MinimalPlugins)
        .add_systems(Startup, setup_startup_state)
        .add_systems(Update, (poll_viewer_messages, headless_report))
        .run();
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

#[derive(Resource)]
struct Viewer3dConfig {
    cm_to_unit: f32,
    show_agents: bool,
    show_locations: bool,
    highlight_selected: bool,
}

impl Default for Viewer3dConfig {
    fn default() -> Self {
        Self {
            cm_to_unit: DEFAULT_CM_TO_UNIT,
            show_agents: true,
            show_locations: true,
            highlight_selected: true,
        }
    }
}

#[derive(Resource, Default)]
struct Viewer3dScene {
    origin: Option<GeoPos>,
    space: Option<SpaceConfig>,
    last_snapshot_time: Option<u64>,
    last_event_id: Option<u64>,
    agent_entities: HashMap<String, Entity>,
    agent_positions: HashMap<String, GeoPos>,
    location_entities: HashMap<String, Entity>,
    asset_entities: HashMap<String, Entity>,
    module_visual_entities: HashMap<String, Entity>,
    power_plant_entities: HashMap<String, Entity>,
    power_storage_entities: HashMap<String, Entity>,
    chunk_entities: HashMap<String, Entity>,
    location_positions: HashMap<String, GeoPos>,
    background_entities: Vec<Entity>,
    heat_overlay_entities: Vec<Entity>,
    flow_overlay_entities: Vec<Entity>,
}

#[derive(Resource)]
struct Viewer3dAssets {
    agent_mesh: Handle<Mesh>,
    agent_material: Handle<StandardMaterial>,
    location_mesh: Handle<Mesh>,
    location_material: Handle<StandardMaterial>,
    asset_mesh: Handle<Mesh>,
    asset_material: Handle<StandardMaterial>,
    power_plant_mesh: Handle<Mesh>,
    power_plant_material: Handle<StandardMaterial>,
    power_storage_mesh: Handle<Mesh>,
    power_storage_material: Handle<StandardMaterial>,
    chunk_mesh: Handle<Mesh>,
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

    fn label(&self) -> String {
        match &self.current {
            Some(info) => match info.kind {
                SelectionKind::Agent => format!("Selection: agent {}", info.id),
                SelectionKind::Location => match &info.name {
                    Some(name) => format!("Selection: location {} ({})", info.id, name),
                    None => format!("Selection: location {}", info.id),
                },
                SelectionKind::Asset => format!("Selection: asset {}", info.id),
                SelectionKind::PowerPlant => format!("Selection: power_plant {}", info.id),
                SelectionKind::PowerStorage => format!("Selection: power_storage {}", info.id),
                SelectionKind::Chunk => format!("Selection: chunk {}", info.id),
            },
            None => "Selection: (none)".to_string(),
        }
    }
}

#[derive(Component)]
struct Viewer3dCamera;

#[derive(Component)]
struct OrbitCamera {
    focus: Vec3,
    radius: f32,
    yaw: f32,
    pitch: f32,
}

impl OrbitCamera {
    fn from_transform(transform: &Transform, focus: Vec3) -> Self {
        let offset = transform.translation - focus;
        let radius = offset.length().max(0.1);
        let yaw = offset.x.atan2(offset.z);
        let pitch = offset
            .y
            .atan2((offset.x * offset.x + offset.z * offset.z).sqrt());
        Self {
            focus,
            radius,
            yaw,
            pitch,
        }
    }

    fn apply_to_transform(&self, transform: &mut Transform) {
        let rotation =
            Quat::from_axis_angle(Vec3::Y, self.yaw) * Quat::from_axis_angle(Vec3::X, self.pitch);
        let offset = rotation * Vec3::new(0.0, 0.0, self.radius);
        transform.translation = self.focus + offset;
        transform.look_at(self.focus, Vec3::Y);
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

#[derive(Component)]
struct RightPanelScroll;

#[derive(Component, Clone)]
struct ControlButton {
    control: ViewerControl,
}

#[derive(Resource, Default)]
struct HeadlessStatus {
    last_status: Option<ConnectionStatus>,
    last_events: usize,
}

fn resolve_addr() -> String {
    std::env::var("AGENT_WORLD_VIEWER_ADDR")
        .ok()
        .or_else(|| std::env::args().nth(1))
        .unwrap_or_else(|| DEFAULT_ADDR.to_string())
}

fn resolve_offline(headless: bool) -> bool {
    let offline_env = std::env::var("AGENT_WORLD_VIEWER_OFFLINE").is_ok();
    let force_online = std::env::var("AGENT_WORLD_VIEWER_FORCE_ONLINE").is_ok();
    decide_offline(headless, offline_env, force_online)
}

fn decide_offline(headless: bool, offline_env: bool, force_online: bool) -> bool {
    if force_online {
        return false;
    }
    if offline_env {
        return true;
    }
    headless
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let label_font = asset_server.load("fonts/DejaVuSans.ttf");
    let agent_mesh = meshes.add(Sphere::new(DEFAULT_AGENT_RADIUS));
    let location_mesh = meshes.add(Cuboid::new(
        DEFAULT_LOCATION_SIZE,
        DEFAULT_LOCATION_SIZE * 0.2,
        DEFAULT_LOCATION_SIZE,
    ));
    let asset_mesh = meshes.add(Cuboid::new(0.45, 0.45, 0.45));
    let power_plant_mesh = meshes.add(Cuboid::new(0.95, 0.7, 0.95));
    let power_storage_mesh = meshes.add(Cuboid::new(0.7, 1.0, 0.7));
    let chunk_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let world_box_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let agent_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.7, 1.0),
        perceptual_roughness: 0.6,
        ..default()
    });
    let location_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.35, 0.4),
        perceptual_roughness: 0.8,
        ..default()
    });
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
        base_color: Color::srgba(0.30, 0.42, 0.66, 0.12),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let chunk_generated_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.24, 0.78, 0.44, 0.18),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let chunk_exhausted_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.62, 0.40, 0.28, 0.18),
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
        base_color: Color::srgba(0.30, 0.34, 0.38, 0.55),
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
        location_mesh,
        location_material,
        asset_mesh,
        asset_material,
        power_plant_mesh,
        power_plant_material,
        power_storage_mesh,
        power_storage_material,
        chunk_mesh,
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

    let focus = Vec3::ZERO;
    let transform = Transform::from_xyz(-30.0, 24.0, 30.0).looking_at(focus, Vec3::Y);
    let orbit = OrbitCamera::from_transform(&transform, focus);
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 0,
            ..default()
        },
        transform,
        Viewer3dCamera,
        orbit,
    ));

    commands.spawn((
        PointLight {
            intensity: 6000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(20.0, 30.0, 20.0),
    ));
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/DejaVuSans.ttf");

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
                        ("Play", ViewerControl::Play),
                        ("Pause", ViewerControl::Pause),
                        ("Step", ViewerControl::Step { count: 1 }),
                        ("Seek 0", ViewerControl::Seek { tick: 0 }),
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
                                    Text::new(label),
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
                    Text::new("Status: connecting"),
                    TextFont {
                        font: font.clone(),
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    StatusText,
                ));

                bar.spawn((
                    Text::new("Selection: (none)"),
                    TextFont {
                        font: font.clone(),
                        font_size: 13.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    SelectionText,
                ));

                spawn_world_overlay_controls(bar, font.clone());
                spawn_diagnosis_panel(bar, font.clone());
                spawn_event_object_link_controls(bar, font.clone());
                spawn_timeline_controls(bar, font.clone());
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
                            Text::new("World: (no snapshot)"),
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
                            Text::new(
                                "Agents Activity:
(no snapshot)",
                            ),
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
                            Text::new(
                                "Details:
(click agent/location to inspect)",
                            ),
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
                            Text::new("Events:\n(no events)"),
                            TextFont {
                                font: font.clone(),
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.85, 0.85, 0.85)),
                            EventsText,
                        ));

                        spawn_event_click_list(events, font.clone());
                    });
            });
        });
}

fn scroll_delta_px_from_parts(unit: MouseScrollUnit, y: f32) -> f32 {
    let scale = match unit {
        MouseScrollUnit::Line => 32.0,
        MouseScrollUnit::Pixel => 1.0,
    };
    y * scale
}

fn scroll_delta_px(event: &MouseWheel) -> f32 {
    scroll_delta_px_from_parts(event.unit, event.y)
}

fn cursor_in_right_panel(window_width: f32, cursor_x: f32) -> bool {
    cursor_x >= (window_width - UI_PANEL_WIDTH).max(0.0)
}

fn scroll_right_panel(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut wheel_events: MessageReader<MouseWheel>,
    mut scroll_query: Query<&mut ScrollPosition, With<RightPanelScroll>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    if !cursor_in_right_panel(window.width(), cursor.x) {
        return;
    }

    let Ok(mut scroll) = scroll_query.single_mut() else {
        return;
    };
    for event in wheel_events.read() {
        let delta = scroll_delta_px(event);
        if delta.abs() < f32::EPSILON {
            continue;
        }
        scroll.y = (scroll.y - delta).max(0.0);
    }
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
    state: Res<ViewerState>,
) {
    let Some(snapshot) = state.snapshot.as_ref() else {
        return;
    };

    let snapshot_time = snapshot.time;
    let snapshot_changed = scene.last_snapshot_time != Some(snapshot_time);
    if snapshot_changed {
        rebuild_scene_from_snapshot(&mut commands, &config, &assets, &mut scene, snapshot);
        scene.last_snapshot_time = Some(snapshot_time);
        scene.last_event_id = None;
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

fn update_ui(
    state: Res<ViewerState>,
    selection: Res<ViewerSelection>,
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
    if !state.is_changed() && !selection.is_changed() && !timeline_changed {
        return;
    }

    if let Ok(mut text) = queries.p0().single_mut() {
        text.0 = format!("Status: {}", format_status(&state.status));
    }

    if let Ok(mut text) = queries.p1().single_mut() {
        text.0 = world_summary(state.snapshot.as_ref(), state.metrics.as_ref());
    }

    let focus_tick = timeline.as_ref().and_then(|timeline| {
        if timeline.manual_override || timeline.drag_active {
            Some(timeline.target_tick)
        } else {
            None
        }
    });

    if let Ok(mut text) = queries.p2().single_mut() {
        text.0 = events_summary(&state.events, focus_tick);
    }

    if let Ok(mut text) = queries.p3().single_mut() {
        text.0 = selection.label();
    }

    if let Ok(mut text) = queries.p4().single_mut() {
        text.0 = agent_activity_summary(state.snapshot.as_ref(), &state.events);
    }

    if let Ok(mut text) = queries.p5().single_mut() {
        text.0 = selection_details_summary(
            &selection,
            state.snapshot.as_ref(),
            &state.events,
            &state.decision_traces,
        );
    }
}

fn update_3d_viewport(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<&mut Camera, With<Viewer3dCamera>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok(mut camera) = cameras.single_mut() else {
        return;
    };

    let panel_width_physical = (UI_PANEL_WIDTH * window.scale_factor()).round() as u32;
    let window_width = window.physical_width();
    let window_height = window.physical_height().max(1);
    let render_width = window_width.saturating_sub(panel_width_physical).max(1);

    camera.viewport = Some(Viewport {
        physical_position: UVec2::ZERO,
        physical_size: UVec2::new(render_width, window_height),
        depth: 0.0..1.0,
    });
}

#[cfg(test)]
mod scroll_tests;
#[cfg(test)]
mod tests;
