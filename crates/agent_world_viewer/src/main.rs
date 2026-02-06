use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::thread;

use agent_world::geometry::GeoPos;
use agent_world::simulator::{
    RunnerMetrics, SpaceConfig, WorldEvent, WorldEventKind, WorldSnapshot,
};
use agent_world::viewer::{
    ViewerControl, ViewerRequest, ViewerResponse, ViewerStream, VIEWER_PROTOCOL_VERSION,
};
use bevy::prelude::*;
use bevy::camera::Viewport;
use bevy::ecs::message::MessageReader;
use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::input::mouse::{MouseMotion, MouseWheel};
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
const LABEL_FONT_SIZE: f32 = 18.0;
const LOCATION_LABEL_OFFSET: f32 = 0.8;
const AGENT_LABEL_OFFSET: f32 = 0.6;
const LABEL_SCALE: f32 = 0.03;
const UI_PANEL_WIDTH: f32 = 380.0;

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
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Agent World Viewer".to_string(),
                    resolution: (1200, 800).into(),
                    ..default()
                }),
                ..default()
            }),
        )
        .insert_resource(OfflineConfig { offline })
        .add_systems(Startup, (setup_startup_state, setup_3d_scene, setup_ui))
        .add_systems(
            Update,
            (
                poll_viewer_messages,
                update_ui,
                update_3d_scene,
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
    metrics: Option<RunnerMetrics>,
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            status: ConnectionStatus::Connecting,
            snapshot: None,
            events: Vec::new(),
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
    last_snapshot_time: Option<u64>,
    last_event_id: Option<u64>,
    agent_entities: HashMap<String, Entity>,
    location_entities: HashMap<String, Entity>,
    location_positions: HashMap<String, GeoPos>,
}

#[derive(Resource)]
struct Viewer3dAssets {
    agent_mesh: Handle<Mesh>,
    agent_material: Handle<StandardMaterial>,
    location_mesh: Handle<Mesh>,
    location_material: Handle<StandardMaterial>,
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
        let pitch = offset.y.atan2((offset.x * offset.x + offset.z * offset.z).sqrt());
        Self {
            focus,
            radius,
            yaw,
            pitch,
        }
    }

    fn apply_to_transform(&self, transform: &mut Transform) {
        let rotation = Quat::from_axis_angle(Vec3::Y, self.yaw)
            * Quat::from_axis_angle(Vec3::X, self.pitch);
        let offset = rotation * Vec3::new(0.0, 0.0, self.radius);
        transform.translation = self.focus + offset;
        transform.look_at(self.focus, Vec3::Y);
    }
}

#[derive(Component)]
struct AgentMarker {
    id: String,
}

#[derive(Component)]
struct LocationMarker {
    id: String,
    name: String,
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

    thread::spawn(move || {
        match TcpStream::connect(&addr) {
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
        }
    });

    (tx_out, rx_in)
}

fn run_connection(
    stream: TcpStream,
    rx_out: Receiver<ViewerRequest>,
    tx_in: Sender<ViewerResponse>,
) -> Result<(), String> {
    stream
        .set_nodelay(true)
        .map_err(|err| err.to_string())?;
    let reader_stream = stream
        .try_clone()
        .map_err(|err| err.to_string())?;
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
    writer
        .write_all(b"\n")
        .map_err(|err| err.to_string())?;
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

    commands.insert_resource(Viewer3dAssets {
        agent_mesh,
        agent_material,
        location_mesh,
        location_material,
        label_font,
    });

    let focus = Vec3::ZERO;
    let transform = Transform::from_xyz(-30.0, 24.0, 30.0).looking_at(focus, Vec3::Y);
    let orbit = OrbitCamera::from_transform(&transform, focus);
    commands.spawn((Camera3d::default(), transform, Viewer3dCamera, orbit));

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

    commands.spawn((Camera2d, IsDefaultUiCamera));

    commands
        .spawn((
            Node {
                width: Val::Px(UI_PANEL_WIDTH),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.11, 0.13)),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(92.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.14)),
            ))
            .with_children(|bar| {
                bar.spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(32.0),
                    column_gap: Val::Px(8.0),
                    flex_direction: FlexDirection::Row,
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
                        controls.spawn((
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
            });

            root.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|content| {
                content
                    .spawn((
                        Node {
                            width: Val::Percent(100.0),
                            flex_grow: 1.0,
                            padding: UiRect::all(Val::Px(14.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.11, 0.11, 0.13)),
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
                            flex_grow: 1.4,
                            padding: UiRect::all(Val::Px(14.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.08, 0.08, 0.1)),
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
            Ok(message) => {
                match message {
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
                    ViewerResponse::Metrics { metrics, .. } => {
                        state.metrics = Some(metrics);
                    }
                    ViewerResponse::Error { message } => {
                        state.status = ConnectionStatus::Error(message);
                    }
                }
            }
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
    mut queries: ParamSet<(
        Query<&mut Text, With<StatusText>>,
        Query<&mut Text, With<SummaryText>>,
        Query<&mut Text, With<EventsText>>,
        Query<&mut Text, With<SelectionText>>,
    )>,
) {
    if !state.is_changed() && !selection.is_changed() {
        return;
    }

    if let Ok(mut text) = queries.p0().single_mut() {
        text.0 = format!("Status: {}", format_status(&state.status));
    }

    if let Ok(mut text) = queries.p1().single_mut() {
        text.0 = world_summary(state.snapshot.as_ref(), state.metrics.as_ref());
    }

    if let Ok(mut text) = queries.p2().single_mut() {
        text.0 = events_summary(&state.events);
    }

    if let Ok(mut text) = queries.p3().single_mut() {
        text.0 = selection.label();
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

fn orbit_camera_controls(
    mut mouse_motion: MessageReader<MouseMotion>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut OrbitCamera, &mut Transform), With<Viewer3dCamera>>,
) {
    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        delta += event.delta;
    }

    let mut scroll = 0.0;
    for event in mouse_wheel.read() {
        scroll += event.y;
    }

    if delta == Vec2::ZERO && scroll == 0.0 {
        return;
    }

    let Ok((mut orbit, mut transform)) = query.single_mut() else {
        return;
    };

    let mut changed = false;

    if buttons.pressed(MouseButton::Left) && delta != Vec2::ZERO {
        orbit.yaw -= delta.x * ORBIT_ROTATE_SENSITIVITY;
        orbit.pitch = (orbit.pitch - delta.y * ORBIT_ROTATE_SENSITIVITY).clamp(-1.54, 1.54);
        changed = true;
    } else if buttons.pressed(MouseButton::Right) && delta != Vec2::ZERO {
        let rotation =
            Quat::from_axis_angle(Vec3::Y, orbit.yaw) * Quat::from_axis_angle(Vec3::X, orbit.pitch);
        let right = rotation * Vec3::X;
        let up = rotation * Vec3::Y;
        let pan_scale = orbit.radius * ORBIT_PAN_SENSITIVITY;
        orbit.focus += (-delta.x * pan_scale) * right + (delta.y * pan_scale) * up;
        changed = true;
    }

    if scroll != 0.0 {
        orbit.radius =
            (orbit.radius * (1.0 - scroll * ORBIT_ZOOM_SENSITIVITY)).clamp(
                ORBIT_MIN_RADIUS,
                ORBIT_MAX_RADIUS,
            );
        changed = true;
    }

    if changed {
        orbit.apply_to_transform(&mut transform);
    }
}

fn pick_3d_selection(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Viewer3dCamera>>,
    agents: Query<(Entity, &GlobalTransform, &AgentMarker)>,
    locations: Query<(Entity, &GlobalTransform, &LocationMarker)>,
    config: Res<Viewer3dConfig>,
    mut selection: ResMut<ViewerSelection>,
    mut transforms: Query<(&mut Transform, Option<&BaseScale>)>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };
    if cursor_position.x > (window.width() - UI_PANEL_WIDTH) {
        return;
    }

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let mut best: Option<(Entity, SelectionKind, String, Option<String>, f32)> = None;

    for (entity, transform, marker) in agents.iter() {
        if let Some(distance) = ray_point_distance(ray, transform.translation()) {
            if distance <= PICK_MAX_DISTANCE
                && best
                    .as_ref()
                    .map(|(_, _, _, _, best_dist)| distance < *best_dist)
                    .unwrap_or(true)
            {
                best = Some((
                    entity,
                    SelectionKind::Agent,
                    marker.id.clone(),
                    None,
                    distance,
                ));
            }
        }
    }

    for (entity, transform, marker) in locations.iter() {
        if let Some(distance) = ray_point_distance(ray, transform.translation()) {
            if distance <= PICK_MAX_DISTANCE
                && best
                    .as_ref()
                    .map(|(_, _, _, _, best_dist)| distance < *best_dist)
                    .unwrap_or(true)
            {
                best = Some((
                    entity,
                    SelectionKind::Location,
                    marker.id.clone(),
                    Some(marker.name.clone()),
                    distance,
                ));
            }
        }
    }

    if let Some((entity, kind, id, name, _)) = best {
        if let Some(current) = selection.current.take() {
            reset_entity_scale(&mut transforms, current.entity);
        }
        selection.current = Some(SelectionInfo {
            entity,
            kind,
            id,
            name,
        });
        if config.highlight_selected {
            apply_entity_highlight(&mut transforms, entity);
        }
    } else if selection.current.is_some() {
        if let Some(current) = selection.current.take() {
            reset_entity_scale(&mut transforms, current.entity);
        }
    }
}

fn handle_control_buttons(
    mut interactions: Query<(&Interaction, &ControlButton), (Changed<Interaction>, With<Button>)>,
    client: Res<ViewerClient>,
) {
    for (interaction, button) in &mut interactions {
        if *interaction == Interaction::Pressed {
            let _ = client.tx.send(ViewerRequest::Control {
                mode: button.control.clone(),
            });
        }
    }
}

fn headless_report(mut status: ResMut<HeadlessStatus>, state: Res<ViewerState>) {
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

fn format_status(status: &ConnectionStatus) -> String {
    match status {
        ConnectionStatus::Connecting => "connecting".to_string(),
        ConnectionStatus::Connected => "connected".to_string(),
        ConnectionStatus::Error(message) => format!("error: {message}"),
    }
}

fn world_summary(snapshot: Option<&WorldSnapshot>, metrics: Option<&RunnerMetrics>) -> String {
    let mut lines = Vec::new();
    if let Some(snapshot) = snapshot {
        let model = &snapshot.model;
        lines.push(format!("Time: {}", snapshot.time));
        lines.push(format!("Locations: {}", model.locations.len()));
        lines.push(format!("Agents: {}", model.agents.len()));
        lines.push(format!("Assets: {}", model.assets.len()));
        lines.push(format!("Power Plants: {}", model.power_plants.len()));
        lines.push(format!("Power Storages: {}", model.power_storages.len()));
    } else {
        lines.push("World: (no snapshot)".to_string());
    }

    if let Some(metrics) = metrics {
        lines.push("".to_string());
        lines.push(format!("Ticks: {}", metrics.total_ticks));
        lines.push(format!("Actions: {}", metrics.total_actions));
        lines.push(format!("Decisions: {}", metrics.total_decisions));
    }

    lines.join("\n")
}

fn events_summary(events: &[WorldEvent]) -> String {
    if events.is_empty() {
        return "Events:\n(no events)".to_string();
    }

    let mut lines = Vec::new();
    lines.push("Events:".to_string());
    for event in events.iter().rev().take(20).rev() {
        lines.push(format!(
            "#{} t{} {:?}",
            event.id, event.time, event.kind
        ));
    }
    lines.join("\n")
}

fn rebuild_scene_from_snapshot(
    commands: &mut Commands,
    config: &Viewer3dConfig,
    assets: &Viewer3dAssets,
    scene: &mut Viewer3dScene,
    snapshot: &WorldSnapshot,
) {
    for entity in scene
        .agent_entities
        .values()
        .chain(scene.location_entities.values())
    {
        commands.entity(*entity).despawn();
    }

    scene.agent_entities.clear();
    scene.location_entities.clear();
    scene.location_positions.clear();

    let origin = space_origin(&snapshot.config.space);
    scene.origin = Some(origin);

    for (location_id, location) in snapshot.model.locations.iter() {
        spawn_location_entity(
            commands,
            config,
            assets,
            scene,
            origin,
            location_id,
            &location.name,
            location.pos,
        );
    }

    for (agent_id, agent) in snapshot.model.agents.iter() {
        spawn_agent_entity(commands, config, assets, scene, origin, agent_id, agent.pos);
    }
}

fn apply_events_to_scene(
    commands: &mut Commands,
    config: &Viewer3dConfig,
    assets: &Viewer3dAssets,
    scene: &mut Viewer3dScene,
    snapshot_time: u64,
    events: &[WorldEvent],
) {
    let Some(origin) = scene.origin else {
        return;
    };

    let mut last_event_id = scene.last_event_id;
    let mut processed = false;

    for event in events {
        if event.time <= snapshot_time {
            continue;
        }
        if let Some(last_id) = last_event_id {
            if event.id <= last_id {
                continue;
            }
        }

        match &event.kind {
            WorldEventKind::LocationRegistered {
                location_id,
                name,
                pos,
                ..
            } => {
                spawn_location_entity(
                    commands,
                    config,
                    assets,
                    scene,
                    origin,
                    location_id,
                    name,
                    *pos,
                );
            }
            WorldEventKind::AgentRegistered { agent_id, pos, .. } => {
                spawn_agent_entity(commands, config, assets, scene, origin, agent_id, *pos);
            }
            WorldEventKind::AgentMoved { agent_id, to, .. } => {
                if let Some(pos) = scene.location_positions.get(to) {
                    spawn_agent_entity(commands, config, assets, scene, origin, agent_id, *pos);
                }
            }
            _ => {}
        }

        last_event_id = Some(event.id);
        processed = true;
    }

    if processed {
        scene.last_event_id = last_event_id;
    }
}

fn spawn_location_entity(
    commands: &mut Commands,
    config: &Viewer3dConfig,
    assets: &Viewer3dAssets,
    scene: &mut Viewer3dScene,
    origin: GeoPos,
    location_id: &str,
    name: &str,
    pos: GeoPos,
) {
    scene
        .location_positions
        .insert(location_id.to_string(), pos);

    if !config.show_locations {
        return;
    }

    let translation = geo_to_vec3(pos, origin, config.cm_to_unit);
    if let Some(entity) = scene.location_entities.get(location_id) {
        commands.entity(*entity).insert((
            Transform::from_translation(translation),
            LocationMarker {
                id: location_id.to_string(),
                name: name.to_string(),
            },
        ));
        return;
    }

    let entity = commands
        .spawn((
            Mesh3d(assets.location_mesh.clone()),
            MeshMaterial3d(assets.location_material.clone()),
            Transform::from_translation(translation),
            Name::new(format!("location:{location_id}:{name}")),
            LocationMarker {
                id: location_id.to_string(),
                name: name.to_string(),
            },
            BaseScale(Vec3::ONE),
        ))
        .id();
    commands.entity(entity).with_children(|parent| {
        spawn_label(
            parent,
            assets,
            format!("{name}"),
            LOCATION_LABEL_OFFSET,
            format!("label:location:{location_id}"),
        );
    });
    scene.location_entities.insert(location_id.to_string(), entity);
}

fn spawn_agent_entity(
    commands: &mut Commands,
    config: &Viewer3dConfig,
    assets: &Viewer3dAssets,
    scene: &mut Viewer3dScene,
    origin: GeoPos,
    agent_id: &str,
    pos: GeoPos,
) {
    if !config.show_agents {
        return;
    }

    let translation = geo_to_vec3(pos, origin, config.cm_to_unit);
    if let Some(entity) = scene.agent_entities.get(agent_id) {
        commands
            .entity(*entity)
            .insert(Transform::from_translation(translation));
        return;
    }

    let entity = commands
        .spawn((
            Mesh3d(assets.agent_mesh.clone()),
            MeshMaterial3d(assets.agent_material.clone()),
            Transform::from_translation(translation),
            Name::new(format!("agent:{agent_id}")),
            AgentMarker {
                id: agent_id.to_string(),
            },
            BaseScale(Vec3::ONE),
        ))
        .id();
    commands.entity(entity).with_children(|parent| {
        spawn_label(
            parent,
            assets,
            agent_id.to_string(),
            AGENT_LABEL_OFFSET,
            format!("label:agent:{agent_id}"),
        );
    });
    scene.agent_entities.insert(agent_id.to_string(), entity);
}

fn spawn_label(
    parent: &mut ChildSpawnerCommands,
    assets: &Viewer3dAssets,
    text: String,
    offset_y: f32,
    name: String,
) {
    parent.spawn((
        Text2d::new(text),
        TextFont {
            font: assets.label_font.clone(),
            font_size: LABEL_FONT_SIZE,
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, offset_y, 0.0))
            .with_scale(Vec3::splat(LABEL_SCALE)),
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        Name::new(name),
    ));
}

fn space_origin(space: &SpaceConfig) -> GeoPos {
    GeoPos {
        x_cm: space.width_cm as f64 / 2.0,
        y_cm: space.depth_cm as f64 / 2.0,
        z_cm: space.height_cm as f64 / 2.0,
    }
}

fn geo_to_vec3(pos: GeoPos, origin: GeoPos, cm_to_unit: f32) -> Vec3 {
    let scale = cm_to_unit as f64;
    Vec3::new(
        ((pos.x_cm - origin.x_cm) * scale) as f32,
        ((pos.z_cm - origin.z_cm) * scale) as f32,
        ((pos.y_cm - origin.y_cm) * scale) as f32,
    )
}

fn ray_point_distance(ray: Ray3d, point: Vec3) -> Option<f32> {
    let direction = ray.direction.as_vec3();
    let to_point = point - ray.origin;
    let t = direction.dot(to_point);
    if t < 0.0 {
        return None;
    }
    let closest = ray.origin + direction * t;
    Some(closest.distance(point))
}

fn apply_entity_highlight(
    transforms: &mut Query<(&mut Transform, Option<&BaseScale>)>,
    entity: Entity,
) {
    if let Ok((mut transform, base)) = transforms.get_mut(entity) {
        let base_scale = base.map(|scale| scale.0).unwrap_or(Vec3::ONE);
        transform.scale = base_scale * 1.6;
    }
}

fn reset_entity_scale(
    transforms: &mut Query<(&mut Transform, Option<&BaseScale>)>,
    entity: Entity,
) {
    if let Ok((mut transform, base)) = transforms.get_mut(entity) {
        let base_scale = base.map(|scale| scale.0).unwrap_or(Vec3::ONE);
        transform.scale = base_scale;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_ui_sets_status_and_events() {
        let mut app = App::new();
        app.add_systems(Update, update_ui);

        app.world_mut().spawn((Text::new(""), StatusText));
        app.world_mut().spawn((Text::new(""), SummaryText));
        app.world_mut().spawn((Text::new(""), EventsText));
        app.world_mut().spawn((Text::new(""), SelectionText));
        app.world_mut().insert_resource(ViewerSelection::default());

        let event = WorldEvent {
            id: 1,
            time: 7,
            kind: agent_world::simulator::WorldEventKind::ActionRejected {
                reason: agent_world::simulator::RejectReason::InvalidAmount { amount: 1 },
            },
        };

        let state = ViewerState {
            status: ConnectionStatus::Error("oops".to_string()),
            snapshot: None,
            events: vec![event.clone()],
            metrics: None,
        };
        app.world_mut().insert_resource(state);

        app.update();

        let world = app.world_mut();

        let status_text = {
            let mut query = world.query::<(&Text, &StatusText)>();
            query.single(world).expect("status text").0.clone()
        };
        assert_eq!(status_text.0, "Status: error: oops");

        let events_text = {
            let mut query = world.query::<(&Text, &EventsText)>();
            query.single(world).expect("events text").0.clone()
        };
        assert_eq!(events_text.0, events_summary(&[event]));
    }

    #[test]
    fn update_ui_populates_world_summary_and_metrics() {
        let mut app = App::new();
        app.add_systems(Update, update_ui);

        app.world_mut().spawn((Text::new(""), SummaryText));
        app.world_mut().spawn((Text::new(""), StatusText));
        app.world_mut().spawn((Text::new(""), EventsText));
        app.world_mut().spawn((Text::new(""), SelectionText));
        app.world_mut().insert_resource(ViewerSelection::default());

        let mut model = agent_world::simulator::WorldModel::default();
        model.locations.insert(
            "loc-1".to_string(),
            agent_world::simulator::Location::new(
                "loc-1",
                "Alpha",
                agent_world::geometry::GeoPos {
                    x_cm: 0.0,
                    y_cm: 0.0,
                    z_cm: 0.0,
                },
            ),
        );
        model.locations.insert(
            "loc-2".to_string(),
            agent_world::simulator::Location::new(
                "loc-2",
                "Beta",
                agent_world::geometry::GeoPos {
                    x_cm: 1.0,
                    y_cm: 1.0,
                    z_cm: 0.0,
                },
            ),
        );
        model.agents.insert(
            "agent-1".to_string(),
            agent_world::simulator::Agent::new(
                "agent-1",
                "loc-1",
            agent_world::geometry::GeoPos {
                x_cm: 0.0,
                y_cm: 0.0,
                z_cm: 0.0,
            },
            ),
        );

        let snapshot = agent_world::simulator::WorldSnapshot {
            version: agent_world::simulator::SNAPSHOT_VERSION,
            chunk_generation_schema_version: agent_world::simulator::CHUNK_GENERATION_SCHEMA_VERSION,
            time: 42,
            config: agent_world::simulator::WorldConfig::default(),
            model,
            chunk_runtime: agent_world::simulator::ChunkRuntimeConfig::default(),
            next_event_id: 1,
            next_action_id: 1,
            pending_actions: Vec::new(),
            journal_len: 0,
        };

        let metrics = RunnerMetrics {
            total_ticks: 42,
            total_actions: 7,
            total_decisions: 4,
            ..RunnerMetrics::default()
        };

        let state = ViewerState {
            status: ConnectionStatus::Connected,
            snapshot: Some(snapshot),
            events: Vec::new(),
            metrics: Some(metrics),
        };
        app.world_mut().insert_resource(state);

        app.update();

        let world = app.world_mut();
        let summary_text = {
            let mut query = world.query::<(&Text, &SummaryText)>();
            query.single(world).expect("summary text").0.clone()
        };

        assert!(summary_text.0.contains("Time: 42"));
        assert!(summary_text.0.contains("Locations: 2"));
        assert!(summary_text.0.contains("Agents: 1"));
        assert!(summary_text.0.contains("Ticks: 42"));
        assert!(summary_text.0.contains("Actions: 7"));
        assert!(summary_text.0.contains("Decisions: 4"));
    }

    #[test]
    fn update_ui_reflects_filtered_events() {
        let mut app = App::new();
        app.add_systems(Update, update_ui);

        app.world_mut().spawn((Text::new(""), EventsText));
        app.world_mut().spawn((Text::new(""), SummaryText));
        app.world_mut().spawn((Text::new(""), StatusText));
        app.world_mut().spawn((Text::new(""), SelectionText));
        app.world_mut().insert_resource(ViewerSelection::default());

        let event = WorldEvent {
            id: 9,
            time: 5,
            kind: agent_world::simulator::WorldEventKind::Power(
                agent_world::simulator::PowerEvent::PowerConsumed {
                    agent_id: "agent-1".to_string(),
                    amount: 3,
                    reason: agent_world::simulator::ConsumeReason::Decision,
                    remaining: 7,
                },
            ),
        };

        let state = ViewerState {
            status: ConnectionStatus::Connected,
            snapshot: None,
            events: vec![event.clone()],
            metrics: None,
        };
        app.world_mut().insert_resource(state);

        app.update();

        let world = app.world_mut();
        let events_text = {
            let mut query = world.query::<(&Text, &EventsText)>();
            query.single(world).expect("events text").0.clone()
        };
        assert!(events_text.0.contains("Power"));
    }

    #[test]
    fn handle_control_buttons_sends_request() {
        let mut app = App::new();
        app.add_systems(Update, handle_control_buttons);

        let (tx, rx) = mpsc::channel::<ViewerRequest>();
        app.world_mut().insert_resource(ViewerClient {
            tx,
            rx: Mutex::new(mpsc::channel::<ViewerResponse>().1),
        });

        app.world_mut().spawn((
            Button,
            Interaction::Pressed,
            ControlButton {
                control: ViewerControl::Step { count: 2 },
            },
        ));

        app.update();

        let request = rx.try_recv().expect("request sent");
        assert_eq!(
            request,
            ViewerRequest::Control {
                mode: ViewerControl::Step { count: 2 }
            }
        );
    }

    #[test]
    fn control_buttons_send_expected_requests() {
        let mut app = App::new();
        app.add_systems(Update, handle_control_buttons);

        let (tx, rx) = mpsc::channel::<ViewerRequest>();
        app.world_mut().insert_resource(ViewerClient {
            tx,
            rx: Mutex::new(mpsc::channel::<ViewerResponse>().1),
        });

        for control in [
            ViewerControl::Play,
            ViewerControl::Pause,
            ViewerControl::Step { count: 1 },
            ViewerControl::Seek { tick: 0 },
        ] {
            app.world_mut().spawn((
                Button,
                Interaction::Pressed,
                ControlButton { control: control.clone() },
            ));
        }

        app.update();

        let mut seen = Vec::new();
        while let Ok(request) = rx.try_recv() {
            seen.push(request);
        }

        assert!(seen.contains(&ViewerRequest::Control { mode: ViewerControl::Play }));
        assert!(seen.contains(&ViewerRequest::Control { mode: ViewerControl::Pause }));
        assert!(seen.contains(&ViewerRequest::Control { mode: ViewerControl::Step { count: 1 } }));
        assert!(seen.contains(&ViewerRequest::Control { mode: ViewerControl::Seek { tick: 0 } }));
    }

    #[test]
    fn headless_report_tracks_status_and_event_count() {
        let mut app = App::new();
        app.add_systems(Update, headless_report);
        app.world_mut().insert_resource(HeadlessStatus::default());

        app.world_mut().insert_resource(ViewerState {
            status: ConnectionStatus::Connecting,
            snapshot: None,
            events: Vec::new(),
            metrics: None,
        });

        app.update();

        let status = app.world_mut().resource::<HeadlessStatus>();
        assert_eq!(status.last_status, Some(ConnectionStatus::Connecting));
        assert_eq!(status.last_events, 0);

        app.world_mut().insert_resource(ViewerState {
            status: ConnectionStatus::Connected,
            snapshot: None,
            events: vec![WorldEvent {
                id: 1,
                time: 1,
                kind: agent_world::simulator::WorldEventKind::ActionRejected {
                    reason: agent_world::simulator::RejectReason::InvalidAmount { amount: 1 },
                },
            }],
            metrics: None,
        });

        app.update();

        let status = app.world_mut().resource::<HeadlessStatus>();
        assert_eq!(status.last_status, Some(ConnectionStatus::Connected));
        assert_eq!(status.last_events, 1);
    }

    #[test]
    fn decide_offline_defaults_headless_and_respects_overrides() {
        assert!(decide_offline(true, false, false));
        assert!(!decide_offline(false, false, false));
        assert!(decide_offline(false, true, false));
        assert!(!decide_offline(true, true, true));
        assert!(!decide_offline(true, false, true));
    }

    #[test]
    fn space_origin_is_center_of_bounds() {
        let space = SpaceConfig {
            width_cm: 100,
            depth_cm: 200,
            height_cm: 300,
        };
        let origin = space_origin(&space);
        assert_eq!(origin.x_cm, 50.0);
        assert_eq!(origin.y_cm, 100.0);
        assert_eq!(origin.z_cm, 150.0);
    }

    #[test]
    fn geo_to_vec3_scales_and_swaps_axes() {
        let origin = GeoPos::new(100.0, 200.0, 300.0);
        let pos = GeoPos::new(110.0, 220.0, 330.0);
        let vec = geo_to_vec3(pos, origin, 0.01);
        assert!((vec.x - 0.1).abs() < 1e-6);
        assert!((vec.y - 0.3).abs() < 1e-6);
        assert!((vec.z - 0.2).abs() < 1e-6);
    }

    #[test]
    fn ray_point_distance_returns_expected_distance() {
        let ray = Ray3d {
            origin: Vec3::ZERO,
            direction: Dir3::new(Vec3::X).expect("direction"),
        };
        let point = Vec3::new(2.0, 1.0, 0.0);
        let distance = ray_point_distance(ray, point).expect("distance");
        assert!((distance - 1.0).abs() < 1e-6);
        assert!(ray_point_distance(ray, Vec3::new(-1.0, 0.0, 0.0)).is_none());
    }

    #[test]
    fn spawn_location_entity_adds_label_text() {
        let mut app = App::new();
        app.add_systems(Update, spawn_label_test_system);
        app.insert_resource(Viewer3dConfig::default());
        app.insert_resource(Viewer3dScene::default());
        app.insert_resource(Viewer3dAssets {
            agent_mesh: Handle::default(),
            agent_material: Handle::default(),
            location_mesh: Handle::default(),
            location_material: Handle::default(),
            label_font: Handle::default(),
        });

        app.update();

        let world = app.world_mut();
        let mut query = world.query::<&Text2d>();
        assert!(query.iter(world).next().is_some());
    }

    fn spawn_label_test_system(
        mut commands: Commands,
        config: Res<Viewer3dConfig>,
        assets: Res<Viewer3dAssets>,
        mut scene: ResMut<Viewer3dScene>,
    ) {
        let origin = GeoPos::new(0.0, 0.0, 0.0);
        spawn_location_entity(
            &mut commands,
            &config,
            &assets,
            &mut scene,
            origin,
            "loc-1",
            "Alpha",
            GeoPos::new(0.0, 0.0, 0.0),
        );
    }
}
