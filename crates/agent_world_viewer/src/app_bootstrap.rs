use super::*;
use bevy_egui::{EguiPlugin, EguiPrimaryContextPass};

pub(super) fn run_ui(addr: String, offline: bool) {
    let viewer_3d_config = resolve_viewer_3d_config();
    let auto_degrade_config = auto_degrade_config_from_env();
    let perf_probe_config = perf_probe::perf_probe_config_from_env();
    let auto_focus_config = auto_focus_config_from_env();
    let viewer_automation_config = viewer_automation_config_from_env();
    let event_window = event_window_policy_from_env(DEFAULT_MAX_EVENTS);
    let panel_mode = resolve_panel_mode_from_env();
    let (module_visibility_state, module_visibility_path) =
        resolve_right_panel_module_visibility_resources();

    App::new()
        .insert_resource(ViewerConfig {
            addr,
            max_events: event_window.max_events,
            event_window,
        })
        .insert_resource(viewer_3d_config)
        .insert_resource(Viewer3dScene::default())
        .insert_resource(ViewerCameraMode::default())
        .insert_resource(panel_mode)
        .insert_resource(ViewerSelection::default())
        .insert_resource(world_overlay_config_from_env())
        .insert_resource(WorldOverlayUiState::default())
        .insert_resource(OverlayRenderRuntime::default())
        .insert_resource(DiagnosisState::default())
        .insert_resource(EventObjectLinkState::default())
        .insert_resource(TimelineUiState::default())
        .insert_resource(TimelineMarkFilterState::default())
        .insert_resource(CopyableTextPanelState::default())
        .insert_resource(OrbitDragState::default())
        .insert_resource(UiI18n::default())
        .insert_resource(auto_degrade_config)
        .insert_resource(AutoDegradeState::default())
        .insert_resource(perf_probe_config)
        .insert_resource(perf_probe::PerfProbeState::default())
        .insert_resource(auto_focus_config)
        .insert_resource(AutoFocusState::default())
        .insert_resource(viewer_automation_config)
        .insert_resource(ViewerAutomationState::default())
        .insert_resource(SelectionEmphasisState::default())
        .insert_resource(internal_capture_config_from_env())
        .insert_resource(InternalCaptureState::default())
        .insert_resource(RightPanelLayoutState::default())
        .insert_resource(RightPanelWidthState::default())
        .insert_resource(RenderPerfSummary::default())
        .insert_resource(RenderPerfHistory::default())
        .insert_resource(LabelLodStats::default())
        .insert_resource(module_visibility_state)
        .insert_resource(module_visibility_path)
        .insert_resource(StepControlLoadingState::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Agent World Viewer".to_string(),
                resolution: (1200, 800).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .insert_resource(OfflineConfig { offline })
        .add_systems(Startup, (setup_startup_state, setup_3d_scene))
        .add_systems(
            Update,
            (
                poll_viewer_messages,
                headless_auto_play_once,
                sync_timeline_state_from_world,
                handle_timeline_adjust_buttons,
                handle_timeline_mark_filter_buttons,
                handle_timeline_bar_drag,
                handle_timeline_mark_jump_buttons,
                handle_timeline_seek_submit,
                handle_world_overlay_toggle_buttons,
                handle_event_click_buttons,
                handle_locate_focus_event_button,
                selection_linking::handle_quick_locate_agent_button,
                handle_jump_selection_events_button,
                update_event_object_link_text,
                update_world_overlay_status_text,
                update_diagnosis_panel,
                update_event_click_list_ui,
                update_timeline_ui,
                trigger_internal_capture,
                persist_right_panel_module_visibility,
            )
                .chain(),
        )
        .add_systems(Update, track_step_loading_state)
        .add_systems(
            Update,
            (
                update_3d_scene,
                update_selection_emphasis.after(update_3d_scene),
                apply_startup_auto_focus.after(update_3d_scene),
                update_world_overlays_3d.after(update_3d_scene),
                orbit_camera_controls,
                handle_focus_selection_hotkey.after(orbit_camera_controls),
                run_viewer_automation
                    .after(update_3d_scene)
                    .after(apply_startup_auto_focus)
                    .after(orbit_camera_controls)
                    .before(sync_camera_mode),
                sync_camera_mode
                    .after(orbit_camera_controls)
                    .after(handle_focus_selection_hotkey),
                camera_controls::sync_two_d_map_marker_visibility.after(sync_camera_mode),
                update_grid_line_lod_visibility.after(sync_camera_mode),
                sync_world_background_visibility.after(sync_camera_mode),
                update_floating_origin.after(orbit_camera_controls),
                sample_render_perf_summary.after(update_grid_line_lod_visibility),
                update_auto_degrade_policy.after(sample_render_perf_summary),
                perf_probe::update_perf_probe.after(update_auto_degrade_policy),
                update_3d_viewport,
                handle_control_buttons,
            ),
        )
        .add_systems(
            PostUpdate,
            (
                pick_3d_selection.after(TransformSystems::Propagate),
                update_label_lod.after(pick_3d_selection),
            ),
        )
        .add_systems(EguiPrimaryContextPass, render_right_side_panel_egui)
        .run();
}

fn resolve_panel_mode_from_env() -> ViewerPanelMode {
    let Some(raw) = std::env::var("AGENT_WORLD_VIEWER_PANEL_MODE").ok() else {
        return ViewerPanelMode::default();
    };

    match raw.trim().to_ascii_lowercase().as_str() {
        "observe" | "obs" | "default" => ViewerPanelMode::Observe,
        "prompt_ops" | "prompt-ops" | "promptops" | "ops" => ViewerPanelMode::PromptOps,
        _ => ViewerPanelMode::default(),
    }
}

pub(super) fn run_headless(addr: String, offline: bool) {
    let event_window = event_window_policy_from_env(DEFAULT_MAX_EVENTS);
    App::new()
        .insert_resource(ViewerConfig {
            addr,
            max_events: event_window.max_events,
            event_window,
        })
        .insert_resource(HeadlessStatus::default())
        .insert_resource(OfflineConfig { offline })
        .add_plugins(MinimalPlugins)
        .add_systems(Startup, setup_startup_state)
        .add_systems(
            Update,
            (
                poll_viewer_messages,
                headless_auto_play_once,
                headless_report,
            )
                .chain(),
        )
        .run();
}

pub(super) fn resolve_addr() -> String {
    std::env::var("AGENT_WORLD_VIEWER_ADDR")
        .ok()
        .or_else(|| std::env::args().nth(1))
        .unwrap_or_else(|| DEFAULT_ADDR.to_string())
}

pub(super) fn resolve_offline(headless: bool) -> bool {
    let offline_env = std::env::var("AGENT_WORLD_VIEWER_OFFLINE").is_ok();
    let force_online = std::env::var("AGENT_WORLD_VIEWER_FORCE_ONLINE").is_ok();
    decide_offline(headless, offline_env, force_online)
}

pub(super) fn decide_offline(headless: bool, offline_env: bool, force_online: bool) -> bool {
    if force_online {
        return false;
    }
    if offline_env {
        return true;
    }
    headless
}
