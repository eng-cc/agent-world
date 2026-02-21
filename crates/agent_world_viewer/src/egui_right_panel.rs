use agent_world::simulator::WorldEventKind;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::app_bootstrap::ThemeRuntimeState;
use crate::button_feedback::StepControlLoadingState;
use crate::copyable_text::{copy_panel_hint, copy_panel_title, ensure_egui_cjk_font};
use crate::event_click_list::{
    apply_event_click_action, event_row_label, event_window, focus_tick,
};
use crate::i18n::{
    camera_mode_button_label, camera_mode_section_label, copyable_panel_toggle_label,
    language_toggle_label, locale_or_default, module_switches_title, top_controls_label,
    top_panel_toggle_label, UiI18n,
};
use crate::right_panel_module_visibility::RightPanelModuleVisibilityState;
use crate::selection_linking::{
    jump_selection_events_action, locate_focus_event_action, quick_locate_agent_action,
    selection_kind_label,
};
use crate::timeline_controls::{
    timeline_axis_max_public, timeline_mark_filter_label_public, timeline_mark_jump_action,
    timeline_seek_action, TimelineMarkKindPublic, TimelineUiState,
};
use crate::ui_locale_text::{
    localize_agent_activity_block, localize_details_block, localize_economy_dashboard_block,
    localize_events_summary_block, localize_industrial_ops_block, localize_ops_navigation_block,
    localize_world_summary_block, map_link_message_for_locale, overlay_button_label,
    overlay_chunk_legend_label, overlay_chunk_legend_title, overlay_grid_line_width_hint,
    overlay_loading, seek_button_label, status_line, timeline_insights, timeline_jump_label,
    timeline_mode_label, timeline_status_line,
};
use crate::ui_text::{
    agent_activity_summary, economy_dashboard_summary, events_summary, industrial_ops_summary,
    ops_navigation_alert_summary, selection_details_summary, world_summary,
};
use crate::world_overlay::overlay_status_text_public;
use crate::{
    grid_line_thickness, CopyableTextPanelState, DiagnosisState, EventObjectLinkState,
    GridLineKind, RenderPerfSummary, RightPanelLayoutState, RightPanelWidthState,
    TimelineMarkFilterState, Viewer3dConfig, ViewerCameraMode, ViewerClient, ViewerSelection,
    ViewerState, WorldOverlayConfig,
};

#[path = "egui_observe_section_card.rs"]
mod egui_observe_section_card;
#[path = "egui_right_panel_chat.rs"]
mod egui_right_panel_chat;
#[path = "egui_right_panel_controls.rs"]
mod egui_right_panel_controls;
#[path = "egui_right_panel_theme_runtime.rs"]
mod egui_right_panel_theme_runtime;

use egui_observe_section_card::render_observe_section_card;
use egui_right_panel_chat::{render_chat_section, AgentChatDraftState};
#[cfg(test)]
use egui_right_panel_controls::send_control_request;
use egui_right_panel_controls::{
    render_control_buttons, render_module_toggle_button, ControlPanelUiState,
};
use egui_right_panel_theme_runtime::render_theme_runtime_section;

const MAIN_PANEL_DEFAULT_WIDTH: f32 = 320.0;
const MAIN_PANEL_MIN_WIDTH: f32 = 240.0;
const MAIN_PANEL_MAX_WIDTH: f32 = 420.0;
const CHAT_PANEL_DEFAULT_WIDTH: f32 = 360.0;
const CHAT_PANEL_MIN_WIDTH: f32 = 280.0;
const CHAT_PANEL_MAX_WIDTH: f32 = 480.0;
const EVENT_ROW_LIMIT: usize = 10;
const MAX_TICK_LABELS: usize = 4;
const EVENT_ROW_LABEL_MAX_CHARS: usize = 72;
const OPS_NAV_PANEL_ENV: &str = "AGENT_WORLD_VIEWER_SHOW_OPS_NAV";
const PRODUCT_STYLE_ENV: &str = "AGENT_WORLD_VIEWER_PRODUCT_STYLE";
const PRODUCT_STYLE_MOTION_ENV: &str = "AGENT_WORLD_VIEWER_PRODUCT_STYLE_MOTION";

fn env_toggle_enabled(raw: Option<&str>) -> bool {
    raw.map(|value| value.trim().to_ascii_lowercase())
        .map(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

fn is_ops_nav_panel_enabled() -> bool {
    env_toggle_enabled(std::env::var(OPS_NAV_PANEL_ENV).ok().as_deref())
}

fn is_product_style_enabled() -> bool {
    env_toggle_enabled(std::env::var(PRODUCT_STYLE_ENV).ok().as_deref())
}

fn is_product_style_motion_enabled() -> bool {
    env_toggle_enabled(std::env::var(PRODUCT_STYLE_MOTION_ENV).ok().as_deref())
}

fn adaptive_panel_default_width(available_width: f32) -> f32 {
    let width = if available_width.is_finite() {
        available_width
    } else {
        MAIN_PANEL_DEFAULT_WIDTH
    };
    (width * 0.22).clamp(MAIN_PANEL_MIN_WIDTH, MAIN_PANEL_MAX_WIDTH)
}

fn adaptive_chat_panel_default_width(available_width: f32) -> f32 {
    let width = if available_width.is_finite() {
        available_width
    } else {
        CHAT_PANEL_DEFAULT_WIDTH
    };
    (width * 0.25).clamp(CHAT_PANEL_MIN_WIDTH, CHAT_PANEL_MAX_WIDTH)
}

fn should_show_chat_panel(layout_state: &RightPanelLayoutState, show_chat: bool) -> bool {
    !layout_state.top_panel_collapsed && show_chat
}

fn total_right_panel_width(main_panel_width: f32, chat_panel_width: f32) -> f32 {
    main_panel_width.max(0.0) + chat_panel_width.max(0.0)
}

#[derive(SystemParam)]
pub(super) struct RightPanelParams<'w, 's> {
    panel_width: ResMut<'w, RightPanelWidthState>,
    layout_state: ResMut<'w, RightPanelLayoutState>,
    camera_mode: ResMut<'w, ViewerCameraMode>,
    i18n: Option<ResMut<'w, UiI18n>>,
    copyable_panel_state: ResMut<'w, CopyableTextPanelState>,
    module_visibility: ResMut<'w, RightPanelModuleVisibilityState>,
    overlay_config: ResMut<'w, WorldOverlayConfig>,
    state: Res<'w, ViewerState>,
    selection: ResMut<'w, ViewerSelection>,
    render_perf: Option<Res<'w, RenderPerfSummary>>,
    viewer_3d_config: Option<Res<'w, Viewer3dConfig>>,
    loading: ResMut<'w, StepControlLoadingState>,
    client: Option<Res<'w, ViewerClient>>,
    chat_focus_signal: ResMut<'w, crate::ChatInputFocusSignal>,
    timeline: ResMut<'w, TimelineUiState>,
    timeline_filters: ResMut<'w, TimelineMarkFilterState>,
    theme_runtime: ResMut<'w, ThemeRuntimeState>,
    diagnosis_state: Res<'w, DiagnosisState>,
    link_state: ResMut<'w, EventObjectLinkState>,
    scene: Res<'w, crate::Viewer3dScene>,
    transforms: Query<'w, 's, (&'static mut Transform, Option<&'static crate::BaseScale>)>,
}

pub(super) fn render_right_side_panel_egui(
    mut contexts: EguiContexts,
    mut cjk_font_initialized: Local<bool>,
    mut chat_draft: Local<AgentChatDraftState>,
    mut control_panel: Local<ControlPanelUiState>,
    params: RightPanelParams,
) {
    let RightPanelParams {
        mut panel_width,
        mut layout_state,
        mut camera_mode,
        mut i18n,
        mut copyable_panel_state,
        mut module_visibility,
        mut overlay_config,
        state,
        mut selection,
        render_perf,
        viewer_3d_config,
        mut loading,
        client,
        mut chat_focus_signal,
        mut timeline,
        mut timeline_filters,
        mut theme_runtime,
        diagnosis_state,
        mut link_state,
        scene,
        mut transforms,
    } = params;

    let locale = locale_or_default(i18n.as_deref());

    let Ok(context) = contexts.ctx_mut() else {
        return;
    };
    ensure_egui_cjk_font(context, &mut cjk_font_initialized);

    if copyable_panel_state.visible != module_visibility.show_details {
        copyable_panel_state.visible = module_visibility.show_details;
    }
    chat_focus_signal.wants_ime_focus = false;

    let show_chat_panel =
        should_show_chat_panel(layout_state.as_ref(), module_visibility.show_chat);
    let chat_panel_width = if show_chat_panel {
        let default_chat_width =
            adaptive_chat_panel_default_width(context.available_rect().width());
        let chat_response = egui::SidePanel::right("viewer-chat-side-panel")
            .resizable(true)
            .default_width(default_chat_width)
            .width_range(CHAT_PANEL_MIN_WIDTH..=CHAT_PANEL_MAX_WIDTH)
            .show(context, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);
                ui.heading(if locale.is_zh() { "对话" } else { "Chat" });
                chat_focus_signal.wants_ime_focus =
                    render_chat_section(ui, locale, &state, client.as_deref(), &mut chat_draft);
            });
        chat_response.response.rect.width()
    } else {
        0.0
    };

    let default_panel_width = adaptive_panel_default_width(context.available_rect().width());
    let panel_response = egui::SidePanel::right("viewer-right-side-panel")
        .resizable(true)
        .default_width(default_panel_width)
        .width_range(MAIN_PANEL_MIN_WIDTH..=MAIN_PANEL_MAX_WIDTH)
        .show(context, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(6.0, 6.0);

            ui.horizontal_wrapped(|ui| {
                if ui
                    .button(top_panel_toggle_label(
                        layout_state.top_panel_collapsed,
                        locale,
                    ))
                    .clicked()
                {
                    layout_state.top_panel_collapsed = !layout_state.top_panel_collapsed;
                }

                if ui.button(language_toggle_label(locale)).clicked() {
                    if let Some(i18n) = i18n.as_deref_mut() {
                        i18n.locale = i18n.locale.toggled();
                    }
                }

                ui.separator();
                ui.label(camera_mode_section_label(locale));

                let is_two_d = *camera_mode == ViewerCameraMode::TwoD;
                if ui
                    .selectable_label(
                        is_two_d,
                        camera_mode_button_label(ViewerCameraMode::TwoD, locale),
                    )
                    .clicked()
                {
                    *camera_mode = ViewerCameraMode::TwoD;
                }

                if ui
                    .selectable_label(
                        !is_two_d,
                        camera_mode_button_label(ViewerCameraMode::ThreeD, locale),
                    )
                    .clicked()
                {
                    *camera_mode = ViewerCameraMode::ThreeD;
                }

                if ui
                    .button(copyable_panel_toggle_label(
                        copyable_panel_state.visible,
                        locale,
                    ))
                    .clicked()
                {
                    copyable_panel_state.visible = !copyable_panel_state.visible;
                    module_visibility.show_details = copyable_panel_state.visible;
                }

                ui.label(top_controls_label(locale));
            });

            if layout_state.top_panel_collapsed {
                return;
            }

            ui.separator();
            ui.strong(module_switches_title(locale));
            ui.horizontal_wrapped(|ui| {
                render_module_toggle_button(
                    ui,
                    "controls",
                    &mut module_visibility.show_controls,
                    locale,
                );
                render_module_toggle_button(
                    ui,
                    "overview",
                    &mut module_visibility.show_overview,
                    locale,
                );
                render_module_toggle_button(ui, "chat", &mut module_visibility.show_chat, locale);
                render_module_toggle_button(
                    ui,
                    "overlay",
                    &mut module_visibility.show_overlay,
                    locale,
                );
                render_module_toggle_button(
                    ui,
                    "diagnosis",
                    &mut module_visibility.show_diagnosis,
                    locale,
                );
                render_module_toggle_button(
                    ui,
                    "event_link",
                    &mut module_visibility.show_event_link,
                    locale,
                );
                render_module_toggle_button(
                    ui,
                    "timeline",
                    &mut module_visibility.show_timeline,
                    locale,
                );

                let mut details_visible = module_visibility.show_details;
                render_module_toggle_button(ui, "details", &mut details_visible, locale);
                module_visibility.show_details = details_visible;
                copyable_panel_state.visible = details_visible;
            });

            if module_visibility.show_controls {
                ui.separator();
                render_control_buttons(
                    ui,
                    locale,
                    &state,
                    loading.as_mut(),
                    &mut control_panel,
                    client.as_deref(),
                );
                render_theme_runtime_section(ui, locale, theme_runtime.as_mut());
            }

            if module_visibility.show_overview {
                ui.separator();
                render_overview_section(
                    ui,
                    locale,
                    &state,
                    &selection,
                    timeline.as_ref(),
                    render_perf.as_deref(),
                );
            }

            if module_visibility.show_overlay {
                ui.separator();
                render_overlay_section(
                    ui,
                    locale,
                    *camera_mode,
                    &state,
                    &viewer_3d_config,
                    overlay_config.as_mut(),
                );
            }

            if module_visibility.show_diagnosis {
                ui.separator();
                ui.strong(if locale.is_zh() {
                    "诊断"
                } else {
                    "Diagnosis"
                });
                ui.add(
                    egui::Label::new(diagnosis_state.text.as_str())
                        .wrap()
                        .selectable(true),
                );
            }

            if module_visibility.show_event_link {
                ui.separator();
                ui.strong(if locale.is_zh() {
                    "事件联动"
                } else {
                    "Event Link"
                });
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .button(crate::ui_locale_text::quick_locate_agent_label(locale))
                        .clicked()
                    {
                        if let Some(config) = viewer_3d_config.as_deref() {
                            quick_locate_agent_action(
                                &scene,
                                config,
                                selection.as_mut(),
                                link_state.as_mut(),
                                &mut transforms,
                            );
                        } else {
                            link_state.message = "Link: viewer config unavailable".to_string();
                        }
                    }

                    if ui
                        .button(crate::ui_locale_text::locate_focus_label(locale))
                        .clicked()
                    {
                        if let Some(config) = viewer_3d_config.as_deref() {
                            locate_focus_event_action(
                                &state,
                                &scene,
                                config,
                                selection.as_mut(),
                                link_state.as_mut(),
                                &mut transforms,
                                Some(timeline.as_mut()),
                            );
                        } else {
                            link_state.message = "Link: viewer config unavailable".to_string();
                        }
                    }

                    if ui
                        .button(crate::ui_locale_text::jump_selection_label(locale))
                        .clicked()
                    {
                        jump_selection_events_action(
                            &state,
                            &selection,
                            link_state.as_mut(),
                            Some(timeline.as_mut()),
                        );
                    }
                });
                ui.add(
                    egui::Label::new(map_link_message_for_locale(&link_state.message, locale))
                        .wrap()
                        .selectable(true),
                );
            }

            if module_visibility.show_timeline {
                ui.separator();
                ui.strong(if locale.is_zh() {
                    "时间轴"
                } else {
                    "Timeline"
                });
                render_timeline_section(
                    ui,
                    locale,
                    &state,
                    timeline.as_mut(),
                    timeline_filters.as_mut(),
                    client.as_deref(),
                );
            }

            if module_visibility.show_details {
                ui.separator();
                ui.heading(copy_panel_title(locale));
                ui.add(egui::Label::new(copy_panel_hint(locale)).wrap());

                egui::ScrollArea::vertical().show(ui, |ui| {
                    render_text_sections(
                        ui,
                        locale,
                        &state,
                        &selection,
                        timeline.as_ref(),
                        &viewer_3d_config,
                    );

                    ui.separator();
                    ui.strong(if locale.is_zh() {
                        "事件行"
                    } else {
                        "Event Rows"
                    });

                    let focus = focus_tick(&state, Some(timeline.as_ref()));
                    let (rows, focused_event_id) =
                        event_window(&state.events, focus, EVENT_ROW_LIMIT);

                    if rows.is_empty() {
                        ui.label(crate::ui_locale_text::event_links_empty(locale));
                        return;
                    }

                    for event in rows {
                        let line =
                            event_row_label(event, focused_event_id == Some(event.id), locale);
                        let line_preview = truncate_observe_text(&line, EVENT_ROW_LABEL_MAX_CHARS);
                        let mut response = ui.add(egui::Button::new(line_preview.as_str()));
                        if line_preview != line {
                            response = response.on_hover_text(line.as_str());
                        }
                        if response.clicked() {
                            if let Some(config) = viewer_3d_config.as_deref() {
                                apply_event_click_action(
                                    event.id,
                                    &state,
                                    &scene,
                                    config,
                                    selection.as_mut(),
                                    &mut transforms,
                                    link_state.as_mut(),
                                    Some(timeline.as_mut()),
                                );
                            } else {
                                link_state.message = "Link: viewer config unavailable".to_string();
                            }
                        }
                    }
                });
            }
        });

    panel_width.width_px =
        total_right_panel_width(panel_response.response.rect.width(), chat_panel_width);
}

fn render_overview_section(
    ui: &mut egui::Ui,
    locale: crate::i18n::UiLocale,
    state: &ViewerState,
    selection: &ViewerSelection,
    timeline: &TimelineUiState,
    perf_summary: Option<&RenderPerfSummary>,
) {
    let current_tick = state
        .snapshot
        .as_ref()
        .map(|snapshot| snapshot.time)
        .or_else(|| state.metrics.as_ref().map(|metrics| metrics.total_ticks))
        .unwrap_or(0);

    let selection_value = selection
        .current
        .as_ref()
        .map(|current| format!("{} {}", selection_kind_label(current.kind), current.id))
        .unwrap_or_else(|| {
            if locale.is_zh() {
                "(无)".to_string()
            } else {
                "(none)".to_string()
            }
        });

    let rejected_events = rejection_event_count(&state.events);
    let (connection_text, connection_color) = connection_signal(&state.status, locale);
    let (health_text, health_color) = health_signal(rejected_events, locale);
    let (mode_text, mode_color) = mode_signal(timeline, locale);

    ui.horizontal_wrapped(|ui| {
        render_status_badge(ui, &connection_text, connection_color);
        render_status_badge(ui, &health_text, health_color);
        render_status_badge(ui, &mode_text, mode_color);
    });

    let chips = [
        (
            if locale.is_zh() { "Tick" } else { "Tick" },
            current_tick.to_string(),
        ),
        (
            if locale.is_zh() { "事件" } else { "Events" },
            state.events.len().to_string(),
        ),
        (
            if locale.is_zh() { "轨迹" } else { "Traces" },
            state.decision_traces.len().to_string(),
        ),
        (
            if locale.is_zh() {
                "选择"
            } else {
                "Selection"
            },
            truncate_observe_text(&selection_value, 18),
        ),
    ];

    ui.horizontal_wrapped(|ui| {
        for (label, value) in chips {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.small(label);
                ui.label(value);
            });
        }
    });

    ui.add(egui::Label::new(status_line(&state.status, locale)).selectable(true));

    if let Some(perf) = perf_summary {
        let frame_line = if locale.is_zh() {
            format!(
                "渲染: avg/p95 {:.1}/{:.1} ms",
                perf.frame_ms_avg, perf.frame_ms_p95
            )
        } else {
            format!(
                "Render: avg/p95 {:.1}/{:.1} ms",
                perf.frame_ms_avg, perf.frame_ms_p95
            )
        };
        let entity_line = if locale.is_zh() {
            format!(
                "对象:{} 标签:{} 覆盖层:{} 事件窗:{}",
                perf.world_entities,
                perf.visible_labels,
                perf.overlay_entities,
                perf.event_window_size
            )
        } else {
            format!(
                "Entities:{} Labels:{} Overlays:{} EventWindow:{}",
                perf.world_entities,
                perf.visible_labels,
                perf.overlay_entities,
                perf.event_window_size
            )
        };
        let budget_line = if locale.is_zh() {
            if perf.auto_degrade_active {
                "预算状态: 自动降级触发".to_string()
            } else {
                "预算状态: 稳定".to_string()
            }
        } else if perf.auto_degrade_active {
            "Budget: auto degrade active".to_string()
        } else {
            "Budget: stable".to_string()
        };

        ui.add(egui::Label::new(frame_line).selectable(true));
        ui.add(egui::Label::new(entity_line).selectable(true));
        ui.add(egui::Label::new(budget_line).selectable(true));
    }
}

fn render_status_badge(ui: &mut egui::Ui, text: &str, fill: egui::Color32) {
    ui.add(egui::Label::new(
        egui::RichText::new(format!("  {text}  "))
            .color(egui::Color32::WHITE)
            .background_color(fill),
    ));
}

fn connection_signal(
    status: &crate::ConnectionStatus,
    locale: crate::i18n::UiLocale,
) -> (String, egui::Color32) {
    match status {
        crate::ConnectionStatus::Connected => (
            if locale.is_zh() {
                "连接正常"
            } else {
                "Conn OK"
            }
            .to_string(),
            egui::Color32::from_rgb(36, 130, 72),
        ),
        crate::ConnectionStatus::Connecting => (
            if locale.is_zh() {
                "连接中"
            } else {
                "Connecting"
            }
            .to_string(),
            egui::Color32::from_rgb(144, 108, 36),
        ),
        crate::ConnectionStatus::Error(_) => (
            if locale.is_zh() {
                "连接异常"
            } else {
                "Conn Error"
            }
            .to_string(),
            egui::Color32::from_rgb(160, 52, 52),
        ),
    }
}

fn health_signal(rejected_events: usize, locale: crate::i18n::UiLocale) -> (String, egui::Color32) {
    if rejected_events == 0 {
        (
            if locale.is_zh() {
                "健康:正常"
            } else {
                "Health: OK"
            }
            .to_string(),
            egui::Color32::from_rgb(32, 112, 64),
        )
    } else if rejected_events <= 2 {
        (
            if locale.is_zh() {
                format!("健康:告警{}", rejected_events)
            } else {
                format!("Health: Warn {rejected_events}")
            },
            egui::Color32::from_rgb(150, 110, 32),
        )
    } else {
        (
            if locale.is_zh() {
                format!("健康:高风险{}", rejected_events)
            } else {
                format!("Health: High {rejected_events}")
            },
            egui::Color32::from_rgb(154, 48, 48),
        )
    }
}

fn mode_signal(
    timeline: &TimelineUiState,
    locale: crate::i18n::UiLocale,
) -> (String, egui::Color32) {
    if timeline.manual_override || timeline.drag_active {
        (
            if locale.is_zh() {
                "观察:手动"
            } else {
                "View: Manual"
            }
            .to_string(),
            egui::Color32::from_rgb(125, 96, 28),
        )
    } else {
        (
            if locale.is_zh() {
                "观察:实时"
            } else {
                "View: Live"
            }
            .to_string(),
            egui::Color32::from_rgb(38, 94, 148),
        )
    }
}

fn rejection_event_count(events: &[agent_world::simulator::WorldEvent]) -> usize {
    events
        .iter()
        .filter(|event| matches!(event.kind, WorldEventKind::ActionRejected { .. }))
        .count()
}

fn truncate_observe_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut out = String::new();
    for ch in text.chars().take(max_chars.saturating_sub(1)) {
        out.push(ch);
    }
    out.push('…');
    out
}

fn render_overlay_section(
    ui: &mut egui::Ui,
    locale: crate::i18n::UiLocale,
    camera_mode: ViewerCameraMode,
    state: &ViewerState,
    viewer_3d_config: &Option<Res<Viewer3dConfig>>,
    overlay_config: &mut WorldOverlayConfig,
) {
    ui.horizontal_wrapped(|ui| {
        if ui.button(overlay_button_label("chunk", locale)).clicked() {
            overlay_config.show_chunk_overlay = !overlay_config.show_chunk_overlay;
        }
        if ui.button(overlay_button_label("heat", locale)).clicked() {
            overlay_config.show_resource_heatmap = !overlay_config.show_resource_heatmap;
        }
        if ui.button(overlay_button_label("flow", locale)).clicked() {
            overlay_config.show_flow_overlay = !overlay_config.show_flow_overlay;
        }
    });

    let text = if let Some(config) = viewer_3d_config.as_deref() {
        overlay_status_text_public(
            state.snapshot.as_ref(),
            &state.events,
            *overlay_config,
            config.effective_cm_to_unit(),
            locale,
        )
    } else {
        overlay_loading(locale).to_string()
    };

    ui.add(egui::Label::new(text).wrap().selectable(true));

    ui.add_space(4.0);
    ui.strong(overlay_chunk_legend_title(locale));
    ui.horizontal_wrapped(|ui| {
        ui.colored_label(
            egui::Color32::from_rgba_premultiplied(76, 107, 168, 180),
            format!("● {}", overlay_chunk_legend_label("unexplored", locale)),
        );
        ui.colored_label(
            egui::Color32::from_rgba_premultiplied(61, 199, 112, 196),
            format!("● {}", overlay_chunk_legend_label("generated", locale)),
        );
        ui.colored_label(
            egui::Color32::from_rgba_premultiplied(158, 102, 71, 196),
            format!("● {}", overlay_chunk_legend_label("exhausted", locale)),
        );
        ui.colored_label(
            egui::Color32::from_rgba_premultiplied(77, 87, 97, 140),
            format!("● {}", overlay_chunk_legend_label("world_grid", locale)),
        );
    });

    let world_thickness = grid_line_thickness(GridLineKind::World, camera_mode);
    let chunk_thickness = grid_line_thickness(GridLineKind::Chunk, camera_mode);
    ui.add(
        egui::Label::new(overlay_grid_line_width_hint(
            locale,
            camera_mode,
            world_thickness,
            chunk_thickness,
        ))
        .wrap()
        .selectable(true),
    );
}

fn render_timeline_section(
    ui: &mut egui::Ui,
    locale: crate::i18n::UiLocale,
    state: &ViewerState,
    timeline: &mut TimelineUiState,
    filters: &mut TimelineMarkFilterState,
    client: Option<&ViewerClient>,
) {
    let current_tick = state
        .snapshot
        .as_ref()
        .map(|snapshot| snapshot.time)
        .or_else(|| state.metrics.as_ref().map(|metrics| metrics.total_ticks))
        .unwrap_or(0);

    let axis_max = timeline_axis_max_public(timeline, current_tick);
    let mode = timeline_mode_label(timeline.drag_active, timeline.manual_override, locale);
    ui.add(
        egui::Label::new(timeline_status_line(
            current_tick,
            timeline.target_tick,
            axis_max,
            mode,
            locale,
        ))
        .wrap()
        .selectable(true),
    );

    ui.horizontal_wrapped(|ui| {
        if ui
            .button(timeline_mark_filter_label_public(
                TimelineMarkKindPublic::Error,
                filters.show_error,
                locale,
            ))
            .clicked()
        {
            filters.show_error = !filters.show_error;
        }

        if ui
            .button(timeline_mark_filter_label_public(
                TimelineMarkKindPublic::Llm,
                filters.show_llm,
                locale,
            ))
            .clicked()
        {
            filters.show_llm = !filters.show_llm;
        }

        if ui
            .button(timeline_mark_filter_label_public(
                TimelineMarkKindPublic::Peak,
                filters.show_peak,
                locale,
            ))
            .clicked()
        {
            filters.show_peak = !filters.show_peak;
        }
    });

    ui.horizontal_wrapped(|ui| {
        if ui.button(timeline_jump_label("err", locale)).clicked() {
            timeline_mark_jump_action(
                state,
                timeline,
                Some(filters),
                TimelineMarkKindPublic::Error,
            );
        }
        if ui.button(timeline_jump_label("llm", locale)).clicked() {
            timeline_mark_jump_action(state, timeline, Some(filters), TimelineMarkKindPublic::Llm);
        }
        if ui.button(timeline_jump_label("peak", locale)).clicked() {
            timeline_mark_jump_action(state, timeline, Some(filters), TimelineMarkKindPublic::Peak);
        }
    });

    let slider_response =
        ui.add(egui::Slider::new(&mut timeline.target_tick, 0..=axis_max.max(1)).text("tick"));
    if slider_response.changed() {
        timeline.manual_override = true;
    }

    ui.horizontal_wrapped(|ui| {
        if ui.button("-10").clicked() {
            timeline.target_tick = timeline.target_tick.saturating_sub(10);
            timeline.manual_override = true;
        }
        if ui.button("-1").clicked() {
            timeline.target_tick = timeline.target_tick.saturating_sub(1);
            timeline.manual_override = true;
        }
        if ui.button("+1").clicked() {
            timeline.target_tick = timeline.target_tick.saturating_add(1);
            timeline.manual_override = true;
        }
        if ui.button("+10").clicked() {
            timeline.target_tick = timeline.target_tick.saturating_add(10);
            timeline.manual_override = true;
        }
        if ui.button(seek_button_label(locale)).clicked() {
            timeline_seek_action(timeline, client);
        }
    });

    let insights = timeline_insights(
        0,
        0,
        0,
        "-".to_string(),
        "-".to_string(),
        "-".to_string(),
        filters.show_error,
        filters.show_llm,
        filters.show_peak,
        "················",
        locale,
    );
    ui.add(egui::Label::new(insights).wrap().selectable(true));
}

fn render_text_sections(
    ui: &mut egui::Ui,
    locale: crate::i18n::UiLocale,
    state: &ViewerState,
    selection: &ViewerSelection,
    timeline: &TimelineUiState,
    viewer_3d_config: &Option<Res<Viewer3dConfig>>,
) {
    let focus = if timeline.manual_override || timeline.drag_active {
        Some(timeline.target_tick)
    } else {
        None
    };

    let reference_radiation_area_m2 = viewer_3d_config
        .as_deref()
        .map(|cfg| cfg.physical.reference_radiation_area_m2)
        .unwrap_or(1.0);

    let summary = localize_world_summary_block(
        world_summary(
            state.snapshot.as_ref(),
            state.metrics.as_ref(),
            viewer_3d_config.as_deref().map(|cfg| &cfg.physical),
        ),
        locale,
    );
    let activity = localize_agent_activity_block(
        agent_activity_summary(state.snapshot.as_ref(), &state.events),
        locale,
    );
    let industrial = industrial_ops_summary(state.snapshot.as_ref(), &state.events)
        .map(|text| localize_industrial_ops_block(text, locale));
    let economy = economy_dashboard_summary(state.snapshot.as_ref(), &state.events)
        .map(|text| localize_economy_dashboard_block(text, locale));
    let ops_navigation = if is_ops_nav_panel_enabled() {
        ops_navigation_alert_summary(state.snapshot.as_ref(), &state.events)
            .map(|text| localize_ops_navigation_block(text, locale))
    } else {
        None
    };
    let details = localize_details_block(
        selection_details_summary(
            selection,
            state.snapshot.as_ref(),
            &state.events,
            &state.decision_traces,
            reference_radiation_area_m2,
        ),
        locale,
    );
    let events = localize_events_summary_block(events_summary(&state.events, focus), locale);

    let mut sections: Vec<(&str, String)> = vec![
        (
            if locale.is_zh() {
                "世界摘要"
            } else {
                "World Summary"
            },
            summary,
        ),
        (
            if locale.is_zh() {
                "Agent 活动"
            } else {
                "Agent Activity"
            },
            activity,
        ),
    ];
    if let Some(industrial) = industrial {
        sections.push((
            if locale.is_zh() {
                "工业链路"
            } else {
                "Industrial Ops"
            },
            industrial,
        ));
    }
    if let Some(economy) = economy {
        sections.push((
            if locale.is_zh() {
                "经营看板"
            } else {
                "Economy Dashboard"
            },
            economy,
        ));
    }
    if let Some(ops_navigation) = ops_navigation {
        sections.push((
            if locale.is_zh() {
                "运营导航"
            } else {
                "Ops Navigator"
            },
            ops_navigation,
        ));
    }
    sections.push((
        if locale.is_zh() {
            "选中详情"
        } else {
            "Selection Details"
        },
        details,
    ));
    sections.push((if locale.is_zh() { "事件" } else { "Events" }, events));

    let product_style = is_product_style_enabled();
    let product_style_motion = product_style && is_product_style_motion_enabled();
    for (title, content) in sections {
        render_observe_section_card(
            ui,
            title,
            content.as_str(),
            product_style,
            product_style_motion,
        );
    }

    if let Some(current) = selection.current.as_ref() {
        ui.add(
            egui::Label::new(format!(
                "{} {} {}",
                if locale.is_zh() {
                    "选中类型:"
                } else {
                    "Selection kind:"
                },
                selection_kind_label(current.kind),
                current.id
            ))
            .wrap()
            .selectable(true),
        );
    }

    let ticks = vec![timeline.target_tick, focus.unwrap_or(timeline.target_tick)];
    let shown: Vec<String> = ticks
        .into_iter()
        .take(MAX_TICK_LABELS)
        .map(|tick| tick.to_string())
        .collect();

    ui.add(
        egui::Label::new(format!(
            "{} {}",
            if locale.is_zh() {
                "Tick 标签:"
            } else {
                "Tick labels:"
            },
            shown.join(", ")
        ))
        .wrap()
        .selectable(true),
    );
}

#[cfg(test)]
#[path = "egui_right_panel_tests.rs"]
mod tests;
