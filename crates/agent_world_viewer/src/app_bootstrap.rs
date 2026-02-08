use super::*;

pub(super) fn run_ui(addr: String, offline: bool) {
    let viewer_3d_config = resolve_viewer_3d_config();

    App::new()
        .insert_resource(ViewerConfig {
            addr,
            max_events: DEFAULT_MAX_EVENTS,
        })
        .insert_resource(viewer_3d_config)
        .insert_resource(Viewer3dScene::default())
        .insert_resource(ViewerSelection::default())
        .insert_resource(WorldOverlayConfig::default())
        .insert_resource(WorldOverlayUiState::default())
        .insert_resource(DiagnosisState::default())
        .insert_resource(EventObjectLinkState::default())
        .insert_resource(TimelineUiState::default())
        .insert_resource(TimelineMarkFilterState::default())
        .insert_resource(OrbitDragState::default())
        .insert_resource(UiI18n::default())
        .insert_resource(internal_capture_config_from_env())
        .insert_resource(InternalCaptureState::default())
        .insert_resource(RightPanelLayoutState::default())
        .insert_resource(StepControlLoadingState::default())
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
                trigger_internal_capture,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                update_control_button_labels,
                update_event_object_link_button_labels,
                update_world_overlay_toggle_labels,
            ),
        )
        .add_systems(Update, attach_step_button_markers)
        .add_systems(
            Update,
            (
                handle_top_panel_toggle_button,
                handle_language_toggle_button,
            ),
        )
        .add_systems(Update, init_button_visual_base)
        .add_systems(
            Update,
            (
                track_step_loading_state,
                update_step_button_loading_ui,
                update_button_hover_visuals,
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                update_3d_scene,
                update_world_overlays_3d.after(update_3d_scene),
                orbit_camera_controls,
                update_floating_origin.after(orbit_camera_controls),
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

pub(super) fn run_headless(addr: String, offline: bool) {
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
