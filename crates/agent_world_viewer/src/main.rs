#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
use std::collections::HashMap;
#[cfg(target_arch = "wasm32")]
use std::collections::VecDeque;
#[cfg(not(target_arch = "wasm32"))]
use std::io::{BufRead, Write};
#[cfg(not(target_arch = "wasm32"))]
use std::net::TcpStream;
use std::sync::mpsc;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc::{Receiver, Sender};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Mutex;
#[cfg(not(target_arch = "wasm32"))]
use std::thread;

use agent_world::geometry::GeoPos;
use agent_world::simulator::{
    AgentDecisionTrace, RunnerMetrics, SpaceConfig, WorldEvent, WorldSnapshot,
};
use agent_world::viewer::{
    ViewerControl, ViewerRequest, ViewerResponse, ViewerStream, VIEWER_PROTOCOL_VERSION,
};
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::view::{ColorGrading, ColorGradingGlobal};
#[cfg(target_arch = "wasm32")]
use gloo_timers::callback::Interval;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::{CloseEvent, ErrorEvent, Event, MessageEvent, UrlSearchParams, WebSocket};

#[cfg(not(target_arch = "wasm32"))]
const DEFAULT_ADDR: &str = "127.0.0.1:5010";
#[cfg(target_arch = "wasm32")]
const DEFAULT_WEB_WS_ADDR: &str = "ws://127.0.0.1:5011";
const DEFAULT_MAX_EVENTS: usize = 100;
const AGENT_BODY_MESH_RADIUS: f32 = 0.5;
const AGENT_BODY_MESH_LENGTH: f32 = 1.0;
const DEFAULT_2D_CAMERA_RADIUS: f32 = 90.0;
const DEFAULT_3D_CAMERA_RADIUS: f32 = 48.0;
const ORBIT_ROTATE_SENSITIVITY: f32 = 0.005;
const ORBIT_PAN_SENSITIVITY: f32 = 0.002;
const ORBIT_ZOOM_SENSITIVITY: f32 = 0.2;
const ORBIT_MIN_RADIUS: f32 = 4.0;
const ORBIT_MAX_RADIUS: f32 = 5_000.0;
const ORTHO_MIN_SCALE: f32 = 0.00005;
const ORTHO_MAX_SCALE: f32 = 128.0;
const PICK_MAX_DISTANCE: f32 = 1.0;
const LABEL_FONT_SIZE: f32 = 18.0;
const LOCATION_LABEL_OFFSET: f32 = 0.8;
const AGENT_LABEL_OFFSET: f32 = 0.6;
const LABEL_SCALE: f32 = 0.03;
const UI_PANEL_WIDTH: f32 = 380.0;
pub(crate) const EGUI_CHAT_INPUT_WIDGET_ID: &str = "viewer-chat-input-message";
mod app_bootstrap;
mod auto_degrade;
mod auto_focus;
mod button_feedback;
mod camera_controls;
mod copyable_text;
mod diagnosis;
mod egui_right_panel;
mod event_click_list;
mod event_window;
mod floating_origin;
mod headless;
mod i18n;
mod internal_capture;
mod label_lod;
mod location_fragment_render;
mod material_library;
mod panel_layout;
mod panel_scroll;
mod perf_probe;
mod render_perf_summary;
mod right_panel_module_visibility;
mod scene_dirty_refresh;
mod scene_helpers;
mod selection_emphasis;
mod selection_linking;
mod timeline_controls;
mod ui_locale_text;
mod ui_state_types;
mod ui_text;
mod viewer_3d_config;
mod viewer_automation;
#[cfg(target_arch = "wasm32")]
mod wasm_egui_input_bridge;
mod web_test_api;
mod world_overlay;

use app_bootstrap::run_ui;
#[cfg(not(target_arch = "wasm32"))]
use app_bootstrap::{resolve_addr, resolve_offline, run_headless};
use auto_degrade::{auto_degrade_config_from_env, update_auto_degrade_policy, AutoDegradeState};
use auto_focus::{
    apply_startup_auto_focus, auto_focus_config_from_env, handle_focus_selection_hotkey,
    AutoFocusState,
};
use button_feedback::{track_step_loading_state, StepControlLoadingState};
use camera_controls::{
    camera_orbit_preset, camera_projection_for_mode, orbit_camera_controls,
    sync_2d_zoom_projection, sync_camera_mode, sync_world_background_visibility,
    update_grid_line_lod_visibility, OrbitDragState, TwoDZoomTier,
};
use copyable_text::{load_embedded_cjk_font, CopyableTextPanelState};
use diagnosis::{spawn_diagnosis_panel, update_diagnosis_panel, DiagnosisState};
use egui_right_panel::render_right_side_panel_egui;
use event_click_list::{
    handle_event_click_buttons, spawn_event_click_list, update_event_click_list_ui,
};
use event_window::{event_window_policy_from_env, push_event_with_window, EventWindowPolicy};
use floating_origin::update_floating_origin;
use headless::headless_auto_play_once;
#[cfg(not(target_arch = "wasm32"))]
use headless::headless_report;
use i18n::{control_button_label, locale_or_default, UiI18n};
use internal_capture::{
    internal_capture_config_from_env, trigger_internal_capture, InternalCaptureState,
};
use label_lod::{update_label_lod, LabelLodStats};
use material_library::{build_fragment_element_material_handles, FragmentElementMaterialHandles};
use panel_layout::{spawn_top_panel_toggle, RightPanelLayoutState, TopPanelContainer};
use panel_scroll::{RightPanelScroll, TopPanelScroll};
use render_perf_summary::{sample_render_perf_summary, RenderPerfHistory, RenderPerfSummary};
use right_panel_module_visibility::{
    persist_right_panel_module_visibility, resolve_right_panel_module_visibility_resources,
};
use scene_dirty_refresh::{refresh_scene_dirty_objects, scene_requires_full_rebuild};
use scene_helpers::*;
use selection_emphasis::{update_selection_emphasis, SelectionEmphasisState};
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
use ui_state_types::*;
use ui_text::{agent_activity_summary, events_summary, selection_details_summary, world_summary};
use viewer_3d_config::{
    resolve_viewer_3d_config, resolve_viewer_external_material_config,
    resolve_viewer_external_mesh_config, resolve_viewer_external_texture_config, Viewer3dConfig,
    ViewerExternalMaterialConfig, ViewerExternalMaterialSlotConfig, ViewerExternalMeshConfig,
    ViewerExternalTextureConfig, ViewerExternalTextureSlotConfig, ViewerGeometryTier,
    ViewerTonemappingMode,
};
use viewer_automation::{
    run_viewer_automation, viewer_automation_config_from_env, ViewerAutomationState,
};
#[cfg(target_arch = "wasm32")]
use wasm_egui_input_bridge::{
    pump_wasm_egui_input_bridge_events, setup_wasm_egui_input_bridge,
    sync_wasm_egui_input_bridge_focus,
};
use world_overlay::{
    handle_world_overlay_toggle_buttons, spawn_world_overlay_controls,
    update_world_overlay_status_text, update_world_overlays_3d, world_overlay_config_from_env,
    OverlayRenderRuntime, WorldOverlayConfig, WorldOverlayUiState,
};

#[cfg(not(target_arch = "wasm32"))]
fn setup_wasm_egui_input_bridge() {}

#[cfg(not(target_arch = "wasm32"))]
fn sync_wasm_egui_input_bridge_focus() {}

#[cfg(not(target_arch = "wasm32"))]
fn pump_wasm_egui_input_bridge_events() {}

const WORLD_MIN_AXIS: f32 = 0.1;
const WORLD_FLOOR_THICKNESS: f32 = 0.03;
const WORLD_GRID_LINE_THICKNESS_2D: f32 = 0.008;
const WORLD_GRID_LINE_THICKNESS_3D: f32 = 0.014;
const CHUNK_GRID_LINE_THICKNESS_2D: f32 = 0.012;
const CHUNK_GRID_LINE_THICKNESS_3D: f32 = 0.022;
const MAX_EMISSIVE_COLOR_COMPONENT: f32 = 4.0;
const RECONNECT_BACKOFF_BASE_SECS: f64 = 0.8;
const RECONNECT_BACKOFF_MAX_SECS: f64 = 12.0;

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
fn main() {
    run_ui(resolve_web_addr(), false);
}

#[cfg(target_arch = "wasm32")]
struct WasmWsRuntime {
    _socket: WebSocket,
    _sender_loop: Interval,
    _on_open: Closure<dyn FnMut(Event)>,
    _on_message: Closure<dyn FnMut(MessageEvent)>,
    _on_error: Closure<dyn FnMut(Event)>,
    _on_close: Closure<dyn FnMut(CloseEvent)>,
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static WASM_WS_RUNTIME: RefCell<Option<WasmWsRuntime>> = RefCell::new(None);
    static WASM_WS_REQUEST_QUEUE: RefCell<VecDeque<ViewerRequest>> = RefCell::new(VecDeque::new());
    static WASM_WS_RESPONSE_QUEUE: RefCell<VecDeque<ViewerResponse>> = RefCell::new(VecDeque::new());
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Copy)]
struct WasmQueueSendError;

#[cfg(target_arch = "wasm32")]
impl std::fmt::Display for WasmQueueSendError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("wasm queue unavailable")
    }
}

#[cfg(target_arch = "wasm32")]
impl std::error::Error for WasmQueueSendError {}

#[derive(Default)]
struct ViewerReconnectRuntime {
    attempt: u32,
    next_retry_at_secs: Option<f64>,
    last_error_signature: Option<String>,
}

impl ViewerReconnectRuntime {
    fn reset(&mut self) {
        self.attempt = 0;
        self.next_retry_at_secs = None;
        self.last_error_signature = None;
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Default)]
struct WasmViewerRequestTx;

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Default)]
struct WasmViewerResponseTx;

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Default)]
struct WasmViewerResponseRx;

#[cfg(target_arch = "wasm32")]
impl WasmViewerRequestTx {
    fn send(&self, request: ViewerRequest) -> Result<(), WasmQueueSendError> {
        WASM_WS_REQUEST_QUEUE.with(|queue| {
            queue.borrow_mut().push_back(request);
        });
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl WasmViewerResponseTx {
    fn send(&self, response: ViewerResponse) -> Result<(), WasmQueueSendError> {
        WASM_WS_RESPONSE_QUEUE.with(|queue| {
            queue.borrow_mut().push_back(response);
        });
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
impl WasmViewerResponseRx {
    fn try_recv(&self) -> Result<ViewerResponse, mpsc::TryRecvError> {
        WASM_WS_RESPONSE_QUEUE.with(|queue| match queue.borrow_mut().pop_front() {
            Some(value) => Ok(value),
            None => Err(mpsc::TryRecvError::Empty),
        })
    }
}

#[cfg(target_arch = "wasm32")]
fn wasm_reset_ws_queues() {
    WASM_WS_REQUEST_QUEUE.with(|queue| queue.borrow_mut().clear());
    WASM_WS_RESPONSE_QUEUE.with(|queue| queue.borrow_mut().clear());
}

#[cfg(target_arch = "wasm32")]
fn wasm_try_recv_request() -> Result<ViewerRequest, mpsc::TryRecvError> {
    WASM_WS_REQUEST_QUEUE.with(|queue| match queue.borrow_mut().pop_front() {
        Some(request) => Ok(request),
        None => Err(mpsc::TryRecvError::Empty),
    })
}

#[cfg(target_arch = "wasm32")]
fn resolve_web_addr() -> String {
    let default_addr = DEFAULT_WEB_WS_ADDR.to_string();
    let Some(window) = web_sys::window() else {
        return default_addr;
    };

    let search = match window.location().search() {
        Ok(search) => search,
        Err(_) => return default_addr,
    };

    let params = match UrlSearchParams::new_with_str(&search) {
        Ok(params) => params,
        Err(_) => return default_addr,
    };

    if let Some(ws) = params.get("ws") {
        return normalize_ws_addr(ws.trim());
    }
    if let Some(addr) = params.get("addr") {
        return normalize_ws_addr(addr.trim());
    }

    default_addr
}

#[cfg(target_arch = "wasm32")]
fn normalize_ws_addr(raw: &str) -> String {
    if raw.starts_with("ws://") || raw.starts_with("wss://") {
        return raw.to_string();
    }
    if let Some(stripped) = raw.strip_prefix("http://") {
        return format!("ws://{stripped}");
    }
    if let Some(stripped) = raw.strip_prefix("https://") {
        return format!("wss://{stripped}");
    }
    format!("ws://{raw}")
}

#[derive(Resource)]
struct ViewerConfig {
    addr: String,
    max_events: usize,
    event_window: EventWindowPolicy,
}

#[derive(Resource, Default)]
struct OfflineConfig {
    offline: bool,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource)]
struct ViewerClient {
    tx: Sender<ViewerRequest>,
    rx: Mutex<Receiver<ViewerResponse>>,
}

#[cfg(target_arch = "wasm32")]
#[derive(Resource, Clone)]
struct ViewerClient {
    tx: WasmViewerRequestTx,
    rx: WasmViewerResponseRx,
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
}

impl Default for ViewerPanelMode {
    fn default() -> Self {
        Self::Observe
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ViewerMaterialVariantPreset {
    Default,
    Matte,
    Glossy,
}

impl Default for ViewerMaterialVariantPreset {
    fn default() -> Self {
        Self::Default
    }
}

impl ViewerMaterialVariantPreset {
    fn next(self) -> Self {
        match self {
            Self::Default => Self::Matte,
            Self::Matte => Self::Glossy,
            Self::Glossy => Self::Default,
        }
    }
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
struct MaterialVariantPreviewState {
    active: ViewerMaterialVariantPreset,
}

#[derive(Resource)]
struct Viewer3dAssets {
    agent_mesh: Handle<Mesh>,
    agent_material: Handle<StandardMaterial>,
    agent_module_marker_mesh: Handle<Mesh>,
    agent_module_marker_material: Handle<StandardMaterial>,
    location_mesh: Handle<Mesh>,
    fragment_element_material_library: FragmentElementMaterialHandles,
    asset_mesh: Handle<Mesh>,
    asset_material: Handle<StandardMaterial>,
    power_plant_mesh: Handle<Mesh>,
    power_plant_material: Handle<StandardMaterial>,
    power_storage_mesh: Handle<Mesh>,
    power_storage_material: Handle<StandardMaterial>,
    location_core_silicate_material: Handle<StandardMaterial>,
    location_core_metal_material: Handle<StandardMaterial>,
    location_core_ice_material: Handle<StandardMaterial>,
    location_halo_material: Handle<StandardMaterial>,
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

#[derive(Resource, Default)]
struct ChatInputFocusSignal {
    wants_ime_focus: bool,
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
    Fragment,
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
enum ViewerLightRigRole {
    Key,
    Fill,
    Rim,
}

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

fn location_mesh_for_geometry_tier(tier: ViewerGeometryTier) -> Mesh {
    let subdivisions = match tier {
        ViewerGeometryTier::Debug => 2,
        ViewerGeometryTier::Balanced => 4,
        ViewerGeometryTier::Cinematic => 6,
    };
    Sphere::new(1.0)
        .mesh()
        .ico(subdivisions)
        .unwrap_or_else(|_| Sphere::new(1.0).into())
}

fn asset_mesh_for_geometry_tier(tier: ViewerGeometryTier) -> Mesh {
    match tier {
        ViewerGeometryTier::Debug => Cuboid::new(0.40, 0.40, 0.40).into(),
        ViewerGeometryTier::Balanced => Cuboid::new(0.45, 0.45, 0.45).into(),
        ViewerGeometryTier::Cinematic => Cuboid::new(0.50, 0.46, 0.50).into(),
    }
}

fn power_plant_mesh_for_geometry_tier(tier: ViewerGeometryTier) -> Mesh {
    match tier {
        ViewerGeometryTier::Debug => Cuboid::new(0.85, 0.62, 0.85).into(),
        ViewerGeometryTier::Balanced => Cuboid::new(0.95, 0.7, 0.95).into(),
        ViewerGeometryTier::Cinematic => Cuboid::new(1.05, 0.78, 1.05).into(),
    }
}

fn power_storage_mesh_for_geometry_tier(tier: ViewerGeometryTier) -> Mesh {
    match tier {
        ViewerGeometryTier::Debug => Cuboid::new(0.62, 0.92, 0.62).into(),
        ViewerGeometryTier::Balanced => Cuboid::new(0.7, 1.0, 0.7).into(),
        ViewerGeometryTier::Cinematic => Cuboid::new(0.78, 1.08, 0.78).into(),
    }
}

fn resolve_mesh_handle<F>(
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    override_path: Option<&str>,
    fallback: F,
) -> Handle<Mesh>
where
    F: FnOnce() -> Mesh,
{
    if let Some(path) = override_path {
        asset_server.load(path.to_string())
    } else {
        meshes.add(fallback())
    }
}

fn resolve_srgb_slot_color(default: [f32; 3], override_color: Option<[f32; 3]>) -> [f32; 3] {
    override_color.unwrap_or(default)
}

#[derive(Clone, Debug, Default)]
struct ResolvedTextureSlot {
    base_color_texture: Option<Handle<Image>>,
    normal_map_texture: Option<Handle<Image>>,
    metallic_roughness_texture: Option<Handle<Image>>,
    emissive_texture: Option<Handle<Image>>,
}

fn resolve_texture_slot(
    asset_server: &AssetServer,
    slot: &ViewerExternalTextureSlotConfig,
) -> ResolvedTextureSlot {
    ResolvedTextureSlot {
        base_color_texture: slot
            .base_texture_asset
            .as_ref()
            .map(|path| asset_server.load(path.to_string())),
        normal_map_texture: slot
            .normal_texture_asset
            .as_ref()
            .map(|path| asset_server.load(path.to_string())),
        metallic_roughness_texture: slot
            .metallic_roughness_texture_asset
            .as_ref()
            .map(|path| asset_server.load(path.to_string())),
        emissive_texture: slot
            .emissive_texture_asset
            .as_ref()
            .map(|path| asset_server.load(path.to_string())),
    }
}

fn texture_slot_override_enabled(slot: &ViewerExternalTextureSlotConfig) -> bool {
    slot.base_texture_asset.is_some()
        || slot.normal_texture_asset.is_some()
        || slot.metallic_roughness_texture_asset.is_some()
        || slot.emissive_texture_asset.is_some()
}

#[derive(Clone, Copy, Debug)]
struct MaterialVariantScalars {
    roughness_scale: f32,
    metallic_scale: f32,
}

fn parse_material_variant_preset(raw: &str) -> Option<ViewerMaterialVariantPreset> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "default" | "base" | "balanced" => Some(ViewerMaterialVariantPreset::Default),
        "matte" | "flat" => Some(ViewerMaterialVariantPreset::Matte),
        "glossy" | "shine" => Some(ViewerMaterialVariantPreset::Glossy),
        _ => None,
    }
}

fn resolve_material_variant_preview_state() -> MaterialVariantPreviewState {
    resolve_material_variant_preview_state_from(|key| std::env::var(key).ok())
}

fn resolve_material_variant_preview_state_from<F>(lookup: F) -> MaterialVariantPreviewState
where
    F: Fn(&str) -> Option<String>,
{
    let active = lookup("AGENT_WORLD_VIEWER_MATERIAL_VARIANT_PRESET")
        .as_deref()
        .and_then(parse_material_variant_preset)
        .unwrap_or_default();
    MaterialVariantPreviewState { active }
}

fn material_variant_scalars(preset: ViewerMaterialVariantPreset) -> MaterialVariantScalars {
    match preset {
        ViewerMaterialVariantPreset::Default => MaterialVariantScalars {
            roughness_scale: 1.0,
            metallic_scale: 1.0,
        },
        ViewerMaterialVariantPreset::Matte => MaterialVariantScalars {
            roughness_scale: 1.35,
            metallic_scale: 0.65,
        },
        ViewerMaterialVariantPreset::Glossy => MaterialVariantScalars {
            roughness_scale: 0.65,
            metallic_scale: 1.35,
        },
    }
}

fn apply_material_variant_scalar(base: f32, scale: f32) -> f32 {
    (base * scale).clamp(0.0, 1.0)
}

fn apply_material_variant_to_material(
    materials: &mut Assets<StandardMaterial>,
    handle: &Handle<StandardMaterial>,
    base_roughness: f32,
    base_metallic: f32,
    preset: ViewerMaterialVariantPreset,
) {
    let scalars = material_variant_scalars(preset);
    let Some(material) = materials.get_mut(handle) else {
        return;
    };
    material.perceptual_roughness =
        apply_material_variant_scalar(base_roughness, scalars.roughness_scale);
    material.metallic = apply_material_variant_scalar(base_metallic, scalars.metallic_scale);
}

fn apply_material_variant_to_scene_materials(
    materials: &mut Assets<StandardMaterial>,
    assets: &Viewer3dAssets,
    config: &Viewer3dConfig,
    preset: ViewerMaterialVariantPreset,
) {
    apply_material_variant_to_material(
        materials,
        &assets.agent_material,
        config.materials.agent.roughness,
        config.materials.agent.metallic,
        preset,
    );
    apply_material_variant_to_material(
        materials,
        &assets.asset_material,
        config.materials.asset.roughness,
        config.materials.asset.metallic,
        preset,
    );
    apply_material_variant_to_material(
        materials,
        &assets.power_plant_material,
        config.materials.facility.roughness,
        config.materials.facility.metallic,
        preset,
    );
    apply_material_variant_to_material(
        materials,
        &assets.power_storage_material,
        config.materials.facility.roughness,
        config.materials.facility.metallic,
        preset,
    );
}

fn color_from_srgb(rgb: [f32; 3]) -> Color {
    Color::srgb(rgb[0], rgb[1], rgb[2])
}

fn color_from_srgb_with_alpha(rgb: [f32; 3], alpha: f32) -> Color {
    Color::srgba(rgb[0], rgb[1], rgb[2], alpha)
}

fn emissive_from_srgb_with_boost(rgb: [f32; 3], boost: f32) -> LinearRgba {
    Color::srgb(
        (rgb[0] * boost).clamp(0.0, MAX_EMISSIVE_COLOR_COMPONENT),
        (rgb[1] * boost).clamp(0.0, MAX_EMISSIVE_COLOR_COMPONENT),
        (rgb[2] * boost).clamp(0.0, MAX_EMISSIVE_COLOR_COMPONENT),
    )
    .into()
}

fn location_material_override_enabled(slot: ViewerExternalMaterialSlotConfig) -> bool {
    slot.base_color_srgb.is_some() || slot.emissive_color_srgb.is_some()
}

fn location_style_override_enabled(
    material_slot: ViewerExternalMaterialSlotConfig,
    texture_slot: &ViewerExternalTextureSlotConfig,
) -> bool {
    location_material_override_enabled(material_slot) || texture_slot_override_enabled(texture_slot)
}

fn lighting_illuminance_triplet(config: &Viewer3dConfig) -> (f32, f32, f32) {
    let key = config.physical.exposed_illuminance_lux();
    let fill = (key * config.lighting.fill_light_ratio.max(0.0)).max(800.0);
    let rim = (key * config.lighting.rim_light_ratio.max(0.0)).max(450.0);
    (key, fill, rim)
}

fn resolve_tonemapping(mode: ViewerTonemappingMode) -> Tonemapping {
    match mode {
        ViewerTonemappingMode::None => Tonemapping::None,
        ViewerTonemappingMode::Reinhard => Tonemapping::Reinhard,
        ViewerTonemappingMode::ReinhardLuminance => Tonemapping::ReinhardLuminance,
        ViewerTonemappingMode::AcesFitted => Tonemapping::AcesFitted,
        ViewerTonemappingMode::AgX => Tonemapping::AgX,
        ViewerTonemappingMode::SomewhatBoringDisplayTransform => {
            Tonemapping::SomewhatBoringDisplayTransform
        }
        ViewerTonemappingMode::TonyMcMapface => Tonemapping::TonyMcMapface,
        ViewerTonemappingMode::BlenderFilmic => Tonemapping::BlenderFilmic,
    }
}

fn build_color_grading(config: &Viewer3dConfig) -> ColorGrading {
    let mut grading = ColorGrading::default();
    grading.global = ColorGradingGlobal {
        exposure: config.post_process.color_grading_exposure,
        post_saturation: config.post_process.color_grading_post_saturation,
        ..default()
    };
    grading
}

fn build_bloom(config: &Viewer3dConfig) -> Option<Bloom> {
    if !config.post_process.bloom_enabled {
        return None;
    }
    let mut bloom = Bloom::NATURAL;
    bloom.intensity = config.post_process.bloom_intensity.max(0.0);
    Some(bloom)
}

fn camera_post_process_components(
    config: &Viewer3dConfig,
) -> (Tonemapping, DebandDither, ColorGrading, Option<Bloom>) {
    let tonemapping = resolve_tonemapping(config.post_process.tonemapping);
    let deband_dither = if config.post_process.deband_dither_enabled {
        DebandDither::Enabled
    } else {
        DebandDither::Disabled
    };
    let color_grading = build_color_grading(config);
    let bloom = build_bloom(config);
    (tonemapping, deband_dither, color_grading, bloom)
}

#[derive(Component, Copy, Clone)]
struct BaseScale(Vec3);

fn reconnect_backoff_secs(attempt: u32) -> f64 {
    let exponential = 2_f64.powi(attempt.saturating_sub(1).min(4) as i32);
    (RECONNECT_BACKOFF_BASE_SECS * exponential).min(RECONNECT_BACKOFF_MAX_SECS)
}

fn reconnectable_error_signature(message: &str) -> Option<String> {
    let normalized = message.trim().to_ascii_lowercase();
    if normalized.is_empty()
        || normalized == "offline mode"
        || normalized.starts_with("agent chat error:")
    {
        return None;
    }

    if normalized.contains("websocket") {
        return Some("websocket".to_string());
    }
    if normalized.contains("connection refused") {
        return Some("connection_refused".to_string());
    }
    if normalized.contains("timed out") {
        return Some("timed_out".to_string());
    }
    if normalized.contains("connection reset") {
        return Some("connection_reset".to_string());
    }
    if normalized.contains("broken pipe") {
        return Some("broken_pipe".to_string());
    }
    if normalized.contains("disconnected") {
        return Some("disconnected".to_string());
    }
    if normalized.contains("viewer receiver poisoned") {
        return Some("receiver_poisoned".to_string());
    }

    None
}

fn websocket_close_code(message: &str) -> Option<u16> {
    let marker = "code=";
    let start = message.find(marker)? + marker.len();
    let digits = message[start..]
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u16>().ok()
}

fn friendly_connection_error(message: &str) -> String {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return "connection error".to_string();
    }

    let lowered = trimmed.to_ascii_lowercase();
    if lowered == "offline mode" || lowered.starts_with("agent chat error:") {
        return trimmed.to_string();
    }
    if lowered.starts_with("websocket closed:") {
        if let Some(code) = websocket_close_code(trimmed) {
            return format!("connection closed (code {code}), retrying...");
        }
        return "connection closed, retrying...".to_string();
    }
    if lowered.contains("connection refused") || lowered.contains("err_connection_refused") {
        return "viewer server unreachable, retrying...".to_string();
    }
    if lowered.contains("timed out") {
        return "connection timed out, retrying...".to_string();
    }
    if lowered.contains("connection reset") || lowered.contains("broken pipe") {
        return "connection interrupted, retrying...".to_string();
    }
    if lowered.contains("disconnected") {
        return "viewer disconnected, retrying...".to_string();
    }
    if lowered.contains("websocket error") {
        return "network error, retrying...".to_string();
    }
    if lowered.contains("viewer receiver poisoned") {
        return "viewer channel unavailable, retrying...".to_string();
    }

    trimmed.to_string()
}

fn viewer_client_from_addr(addr: String) -> ViewerClient {
    let (tx, rx) = spawn_viewer_client(addr);
    #[cfg(not(target_arch = "wasm32"))]
    {
        ViewerClient {
            tx,
            rx: Mutex::new(rx),
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        ViewerClient { tx, rx }
    }
}

fn setup_connection(mut commands: Commands, config: Res<ViewerConfig>) {
    commands.insert_resource(viewer_client_from_addr(config.addr.clone()));
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

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
fn spawn_viewer_client(addr: String) -> (WasmViewerRequestTx, WasmViewerResponseRx) {
    wasm_reset_ws_queues();
    let tx_out = WasmViewerRequestTx;
    let tx_in = WasmViewerResponseTx;
    let rx_in = WasmViewerResponseRx;

    let ws_url = normalize_ws_addr(&addr);
    let socket = match WebSocket::new(&ws_url) {
        Ok(socket) => socket,
        Err(err) => {
            let _ = tx_in.send(ViewerResponse::Error {
                message: format!("websocket open failed: {err:?}"),
            });
            return (tx_out, rx_in);
        }
    };

    let open_socket = socket.clone();
    let open_tx = tx_in.clone();
    let on_open = Closure::wrap(Box::new(move |_event: Event| {
        send_request_ws(
            &open_socket,
            &ViewerRequest::Hello {
                client: "bevy_viewer_web".to_string(),
                version: VIEWER_PROTOCOL_VERSION,
            },
            &open_tx,
        );
        send_request_ws(
            &open_socket,
            &ViewerRequest::Subscribe {
                streams: vec![
                    ViewerStream::Snapshot,
                    ViewerStream::Events,
                    ViewerStream::Metrics,
                ],
                event_kinds: Vec::new(),
            },
            &open_tx,
        );
        send_request_ws(&open_socket, &ViewerRequest::RequestSnapshot, &open_tx);
    }) as Box<dyn FnMut(_)>);
    socket.set_onopen(Some(on_open.as_ref().unchecked_ref()));

    let message_tx = tx_in.clone();
    let on_message = Closure::wrap(Box::new(move |event: MessageEvent| {
        let Some(text) = event.data().as_string() else {
            let _ = message_tx.send(ViewerResponse::Error {
                message: "websocket message decode failed: non-text payload".to_string(),
            });
            return;
        };
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return;
        }
        match serde_json::from_str::<ViewerResponse>(trimmed) {
            Ok(response) => {
                let _ = message_tx.send(response);
            }
            Err(err) => {
                let _ = message_tx.send(ViewerResponse::Error {
                    message: format!("decode error: {err}"),
                });
            }
        }
    }) as Box<dyn FnMut(_)>);
    socket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));

    let error_tx = tx_in.clone();
    let on_error = Closure::wrap(Box::new(move |event: Event| {
        let detail = event
            .dyn_ref::<ErrorEvent>()
            .map(|error| error.message())
            .filter(|message| !message.trim().is_empty())
            .unwrap_or_else(|| "network error".to_string());
        let _ = error_tx.send(ViewerResponse::Error {
            message: format!("websocket error: {detail}"),
        });
    }) as Box<dyn FnMut(_)>);
    socket.set_onerror(Some(on_error.as_ref().unchecked_ref()));

    let close_tx = tx_in.clone();
    let on_close = Closure::wrap(Box::new(move |event: CloseEvent| {
        let _ = close_tx.send(ViewerResponse::Error {
            message: format!(
                "websocket closed: code={} reason={}",
                event.code(),
                event.reason()
            ),
        });
    }) as Box<dyn FnMut(_)>);
    socket.set_onclose(Some(on_close.as_ref().unchecked_ref()));

    let sender_socket = socket.clone();
    let sender_tx = tx_in.clone();
    let sender_loop = Interval::new(16, move || {
        while let Ok(request) = wasm_try_recv_request() {
            send_request_ws(&sender_socket, &request, &sender_tx);
        }
    });

    WASM_WS_RUNTIME.with(|runtime| {
        *runtime.borrow_mut() = Some(WasmWsRuntime {
            _socket: socket,
            _sender_loop: sender_loop,
            _on_open: on_open,
            _on_message: on_message,
            _on_error: on_error,
            _on_close: on_close,
        });
    });

    (tx_out, rx_in)
}

#[cfg(target_arch = "wasm32")]
fn send_request_ws(socket: &WebSocket, request: &ViewerRequest, tx_in: &WasmViewerResponseTx) {
    match serde_json::to_string(request) {
        Ok(payload) => {
            if let Err(err) = socket.send_with_str(&payload) {
                let _ = tx_in.send(ViewerResponse::Error {
                    message: format!("websocket send failed: {err:?}"),
                });
            }
        }
        Err(err) => {
            let _ = tx_in.send(ViewerResponse::Error {
                message: format!("request encode failed: {err}"),
            });
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
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
    external_mesh: Res<ViewerExternalMeshConfig>,
    external_material: Res<ViewerExternalMaterialConfig>,
    external_texture: Res<ViewerExternalTextureConfig>,
    variant_preview: Res<MaterialVariantPreviewState>,
    camera_mode: Res<ViewerCameraMode>,
    mut scene: ResMut<Viewer3dScene>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut fonts: ResMut<Assets<Font>>,
    asset_server: Res<AssetServer>,
) {
    let geometry_tier = config.assets.geometry_tier;
    let root_entity = commands
        .spawn((
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Visible,
            Viewer3dSceneRoot,
        ))
        .id();
    scene.root_entity = Some(root_entity);

    let label_font = load_embedded_cjk_font(&mut fonts);
    let agent_mesh = resolve_mesh_handle(
        &asset_server,
        &mut meshes,
        external_mesh.agent_mesh_asset.as_deref(),
        || Capsule3d::new(AGENT_BODY_MESH_RADIUS, AGENT_BODY_MESH_LENGTH).into(),
    );
    let agent_module_marker_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let location_mesh = resolve_mesh_handle(
        &asset_server,
        &mut meshes,
        external_mesh.location_mesh_asset.as_deref(),
        || location_mesh_for_geometry_tier(geometry_tier),
    );
    let asset_mesh = resolve_mesh_handle(
        &asset_server,
        &mut meshes,
        external_mesh.asset_mesh_asset.as_deref(),
        || asset_mesh_for_geometry_tier(geometry_tier),
    );
    let power_plant_mesh = resolve_mesh_handle(
        &asset_server,
        &mut meshes,
        external_mesh.power_plant_mesh_asset.as_deref(),
        || power_plant_mesh_for_geometry_tier(geometry_tier),
    );
    let power_storage_mesh = resolve_mesh_handle(
        &asset_server,
        &mut meshes,
        external_mesh.power_storage_mesh_asset.as_deref(),
        || power_storage_mesh_for_geometry_tier(geometry_tier),
    );
    let world_box_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let agent_texture = resolve_texture_slot(&asset_server, &external_texture.agent);
    let location_texture = resolve_texture_slot(&asset_server, &external_texture.location);
    let asset_texture = resolve_texture_slot(&asset_server, &external_texture.asset);
    let power_plant_texture = resolve_texture_slot(&asset_server, &external_texture.power_plant);
    let power_storage_texture =
        resolve_texture_slot(&asset_server, &external_texture.power_storage);
    let variant_scalars = material_variant_scalars(variant_preview.active);
    let agent_roughness = apply_material_variant_scalar(
        config.materials.agent.roughness,
        variant_scalars.roughness_scale,
    );
    let agent_metallic = apply_material_variant_scalar(
        config.materials.agent.metallic,
        variant_scalars.metallic_scale,
    );
    let asset_roughness = apply_material_variant_scalar(
        config.materials.asset.roughness,
        variant_scalars.roughness_scale,
    );
    let asset_metallic = apply_material_variant_scalar(
        config.materials.asset.metallic,
        variant_scalars.metallic_scale,
    );
    let facility_roughness = apply_material_variant_scalar(
        config.materials.facility.roughness,
        variant_scalars.roughness_scale,
    );
    let facility_metallic = apply_material_variant_scalar(
        config.materials.facility.metallic,
        variant_scalars.metallic_scale,
    );
    let agent_base_color =
        resolve_srgb_slot_color([1.0, 0.42, 0.22], external_material.agent.base_color_srgb);
    let agent_emissive_color = resolve_srgb_slot_color(
        [0.90, 0.38, 0.20],
        external_material.agent.emissive_color_srgb,
    );
    let agent_material = materials.add(StandardMaterial {
        base_color: color_from_srgb(agent_base_color),
        base_color_texture: agent_texture.base_color_texture,
        normal_map_texture: agent_texture.normal_map_texture,
        metallic_roughness_texture: agent_texture.metallic_roughness_texture,
        emissive_texture: agent_texture.emissive_texture,
        perceptual_roughness: agent_roughness,
        metallic: agent_metallic,
        emissive: emissive_from_srgb_with_boost(
            agent_emissive_color,
            config.materials.agent.emissive_boost,
        ),
        ..default()
    });
    let agent_module_marker_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.16, 0.92, 0.98),
        unlit: true,
        ..default()
    });
    let fragment_element_material_library =
        build_fragment_element_material_handles(&mut materials, config.materials.fragment);
    let asset_base_color =
        resolve_srgb_slot_color([0.82, 0.76, 0.34], external_material.asset.base_color_srgb);
    let asset_emissive_color = resolve_srgb_slot_color(
        [0.82, 0.76, 0.34],
        external_material.asset.emissive_color_srgb,
    );
    let asset_material = materials.add(StandardMaterial {
        base_color: color_from_srgb(asset_base_color),
        base_color_texture: asset_texture.base_color_texture,
        normal_map_texture: asset_texture.normal_map_texture,
        metallic_roughness_texture: asset_texture.metallic_roughness_texture,
        emissive_texture: asset_texture.emissive_texture,
        perceptual_roughness: asset_roughness,
        metallic: asset_metallic,
        emissive: emissive_from_srgb_with_boost(
            asset_emissive_color,
            config.materials.asset.emissive_boost,
        ),
        ..default()
    });
    let power_plant_base_color = resolve_srgb_slot_color(
        [0.95, 0.42, 0.20],
        external_material.power_plant.base_color_srgb,
    );
    let power_plant_emissive_color = resolve_srgb_slot_color(
        [0.95, 0.42, 0.20],
        external_material.power_plant.emissive_color_srgb,
    );
    let power_plant_material = materials.add(StandardMaterial {
        base_color: color_from_srgb(power_plant_base_color),
        base_color_texture: power_plant_texture.base_color_texture,
        normal_map_texture: power_plant_texture.normal_map_texture,
        metallic_roughness_texture: power_plant_texture.metallic_roughness_texture,
        emissive_texture: power_plant_texture.emissive_texture,
        perceptual_roughness: facility_roughness,
        metallic: facility_metallic,
        emissive: emissive_from_srgb_with_boost(
            power_plant_emissive_color,
            config.materials.facility.emissive_boost,
        ),
        ..default()
    });
    let power_storage_base_color = resolve_srgb_slot_color(
        [0.20, 0.86, 0.48],
        external_material.power_storage.base_color_srgb,
    );
    let power_storage_emissive_color = resolve_srgb_slot_color(
        [0.20, 0.86, 0.48],
        external_material.power_storage.emissive_color_srgb,
    );
    let power_storage_material = materials.add(StandardMaterial {
        base_color: color_from_srgb(power_storage_base_color),
        base_color_texture: power_storage_texture.base_color_texture,
        normal_map_texture: power_storage_texture.normal_map_texture,
        metallic_roughness_texture: power_storage_texture.metallic_roughness_texture,
        emissive_texture: power_storage_texture.emissive_texture,
        perceptual_roughness: facility_roughness,
        metallic: facility_metallic,
        emissive: emissive_from_srgb_with_boost(
            power_storage_emissive_color,
            config.materials.facility.emissive_boost,
        ),
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
    let (
        location_core_silicate_material,
        location_core_metal_material,
        location_core_ice_material,
        location_halo_material,
    ) = if location_style_override_enabled(external_material.location, &external_texture.location) {
        let location_base_color = resolve_srgb_slot_color(
            [0.30, 0.42, 0.66],
            external_material.location.base_color_srgb,
        );
        let location_emissive_color = resolve_srgb_slot_color(
            location_base_color,
            external_material.location.emissive_color_srgb,
        );
        let location_texture = location_texture.clone();
        let location_core_material = |alpha: f32| StandardMaterial {
            base_color: color_from_srgb_with_alpha(location_base_color, alpha),
            base_color_texture: location_texture.base_color_texture.clone(),
            normal_map_texture: location_texture.normal_map_texture.clone(),
            metallic_roughness_texture: location_texture.metallic_roughness_texture.clone(),
            emissive_texture: location_texture.emissive_texture.clone(),
            perceptual_roughness: facility_roughness,
            metallic: facility_metallic,
            emissive: color_from_srgb(location_emissive_color).into(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        };

        (
            materials.add(location_core_material(0.22)),
            materials.add(location_core_material(0.30)),
            materials.add(location_core_material(0.30)),
            materials.add(StandardMaterial {
                base_color: color_from_srgb_with_alpha(location_base_color, 0.10),
                base_color_texture: location_texture.base_color_texture,
                normal_map_texture: location_texture.normal_map_texture,
                metallic_roughness_texture: location_texture.metallic_roughness_texture,
                emissive_texture: location_texture.emissive_texture,
                emissive: color_from_srgb(location_emissive_color).into(),
                alpha_mode: AlphaMode::Blend,
                ..default()
            }),
        )
    } else {
        (
            chunk_unexplored_material.clone(),
            chunk_generated_material.clone(),
            chunk_exhausted_material.clone(),
            world_bounds_material.clone(),
        )
    };
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
        fragment_element_material_library,
        asset_mesh,
        asset_material,
        power_plant_mesh,
        power_plant_material,
        power_storage_mesh,
        power_storage_material,
        location_core_silicate_material,
        location_core_metal_material,
        location_core_ice_material,
        location_halo_material,
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
    let mut projection = camera_projection_for_mode(mode, &config);
    if mode == ViewerCameraMode::TwoD {
        sync_2d_zoom_projection(&mut projection, orbit.radius, config.effective_cm_to_unit());
    }
    let (tonemapping, deband_dither, color_grading, bloom) =
        camera_post_process_components(&config);
    let mut camera_entity = commands.spawn((
        Camera3d::default(),
        projection,
        Camera {
            order: 0,
            ..default()
        },
        transform,
        Viewer3dCamera,
        orbit,
        tonemapping,
        deband_dither,
        color_grading,
    ));
    if let Some(settings) = bloom {
        camera_entity.insert(settings);
    }

    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.94, 0.97, 1.0),
        brightness: config.lighting.ambient_brightness,
        affects_lightmapped_meshes: true,
    });

    let (key_illuminance, fill_illuminance, rim_illuminance) =
        lighting_illuminance_triplet(&config);

    commands.spawn((
        DirectionalLight {
            illuminance: key_illuminance,
            shadows_enabled: config.lighting.shadows_enabled,
            ..default()
        },
        Transform::from_xyz(24.0, 36.0, 22.0).looking_at(Vec3::ZERO, Vec3::Y),
        ViewerLightRigRole::Key,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: fill_illuminance,
            color: Color::srgb(0.74, 0.82, 0.92),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(-18.0, 20.0, -28.0).looking_at(Vec3::ZERO, Vec3::Y),
        ViewerLightRigRole::Fill,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: rim_illuminance,
            color: Color::srgb(0.96, 0.88, 0.78),
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(28.0, 16.0, -24.0).looking_at(Vec3::ZERO, Vec3::Y),
        ViewerLightRigRole::Rim,
    ));
}

#[allow(dead_code)]
fn setup_ui(
    mut commands: Commands,
    mut fonts: ResMut<Assets<Font>>,
    _asset_server: Res<AssetServer>,
    copyable_panel_state: Option<Res<CopyableTextPanelState>>,
) {
    let font = load_embedded_cjk_font(&mut fonts);
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
    #[cfg(not(target_arch = "wasm32"))]
    let receiver = match client.rx.lock() {
        Ok(receiver) => receiver,
        Err(_) => {
            state.status =
                ConnectionStatus::Error(friendly_connection_error("viewer receiver poisoned"));
            return;
        }
    };
    #[cfg(target_arch = "wasm32")]
    let receiver = &client.rx;

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
                    push_event_with_window(&mut state.events, event, config.event_window);
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
                    state.status = ConnectionStatus::Error(friendly_connection_error(&message));
                }
                ViewerResponse::PromptControlAck { .. } => {}
                ViewerResponse::PromptControlError { .. } => {}
                ViewerResponse::AgentChatAck { .. } => {}
                ViewerResponse::AgentChatError { error } => {
                    state.status = ConnectionStatus::Error(format!(
                        "agent chat error: {} ({})",
                        error.message, error.code
                    ));
                }
            },
            Err(mpsc::TryRecvError::Empty) => break,
            Err(mpsc::TryRecvError::Disconnected) => {
                if !matches!(state.status, ConnectionStatus::Error(_)) {
                    state.status =
                        ConnectionStatus::Error(friendly_connection_error("disconnected"));
                }
                break;
            }
        }
    }
}

fn attempt_viewer_reconnect(
    mut commands: Commands,
    config: Res<ViewerConfig>,
    offline: Option<Res<OfflineConfig>>,
    time: Option<Res<Time>>,
    state: Option<ResMut<ViewerState>>,
    mut reconnect: Local<ViewerReconnectRuntime>,
) {
    if offline.as_deref().is_some_and(|cfg| cfg.offline) {
        reconnect.reset();
        return;
    }

    let Some(mut state) = state else {
        reconnect.reset();
        return;
    };

    let ConnectionStatus::Error(message) = &state.status else {
        reconnect.reset();
        return;
    };

    let Some(signature) = reconnectable_error_signature(message) else {
        reconnect.reset();
        return;
    };

    let now = time
        .as_deref()
        .map(Time::elapsed_secs_f64)
        .unwrap_or_default();
    let is_new_error = reconnect.last_error_signature.as_deref() != Some(signature.as_str());
    if is_new_error {
        reconnect.attempt = 0;
        reconnect.next_retry_at_secs = Some(now);
        reconnect.last_error_signature = Some(signature);
    }

    let should_retry = reconnect
        .next_retry_at_secs
        .map(|next| now >= next)
        .unwrap_or(true);
    if !should_retry {
        return;
    }

    commands.insert_resource(viewer_client_from_addr(config.addr.clone()));
    state.status = ConnectionStatus::Connecting;
    reconnect.attempt = reconnect.attempt.saturating_add(1);
    reconnect.next_retry_at_secs = Some(now + reconnect_backoff_secs(reconnect.attempt));
}

fn handle_material_variant_preview_hotkey(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut preview_state: ResMut<MaterialVariantPreviewState>,
    config: Res<Viewer3dConfig>,
    assets: Option<Res<Viewer3dAssets>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !keyboard.just_pressed(KeyCode::F8) {
        return;
    }

    preview_state.active = preview_state.active.next();
    let Some(assets) = assets else {
        return;
    };
    apply_material_variant_to_scene_materials(
        &mut materials,
        &assets,
        &config,
        preview_state.active,
    );
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
    if scene_requires_full_rebuild(&scene, snapshot) {
        rebuild_scene_from_snapshot(&mut commands, &config, &assets, &mut scene, snapshot);
        scene.last_event_id = None;
        selection.clear();
    } else {
        refresh_scene_dirty_objects(&mut commands, &config, &assets, &mut scene, snapshot);
    }
    scene.last_snapshot_time = Some(snapshot_time);

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
            if should_apply_scale_highlight(current.kind) {
                apply_entity_highlight(&mut transforms, current.entity);
            } else {
                reset_entity_scale(&mut transforms, current.entity);
            }
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
#[path = "tests_camera_mode.rs"]
mod camera_mode_tests;
#[cfg(test)]
mod tests;
