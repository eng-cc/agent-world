use agent_world::simulator::WorldEventKind;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::button_feedback::{mark_step_loading_on_control, StepControlLoadingState};
use crate::copyable_text::{copy_panel_hint, copy_panel_title, ensure_egui_cjk_font};
use crate::event_click_list::{
    apply_event_click_action, event_row_label, event_window, focus_tick,
};
use crate::i18n::{
    control_button_label, copyable_panel_toggle_label, language_toggle_label, locale_or_default,
    step_button_label, top_controls_label, top_panel_toggle_label, UiI18n,
};
use crate::selection_linking::{
    jump_selection_events_action, locate_focus_event_action, selection_kind_label,
};
use crate::timeline_controls::{
    timeline_axis_max_public, timeline_mark_filter_label_public, timeline_mark_jump_action,
    timeline_seek_action, TimelineMarkKindPublic, TimelineUiState,
};
use crate::ui_locale_text::{
    localize_agent_activity_block, localize_details_block, localize_events_summary_block,
    localize_world_summary_block, map_link_message_for_locale, overlay_button_label,
    overlay_loading, seek_button_label, status_line, timeline_insights, timeline_jump_label,
    timeline_mode_label, timeline_status_line,
};
use crate::ui_text::{
    agent_activity_summary, events_summary, selection_details_summary, world_summary,
};
use crate::world_overlay::overlay_status_text_public;
use crate::{
    CopyableTextPanelState, DiagnosisState, EventObjectLinkState, RightPanelLayoutState,
    RightPanelWidthState, TimelineMarkFilterState, Viewer3dConfig, ViewerClient, ViewerControl,
    ViewerSelection, ViewerState, WorldOverlayConfig,
};

const DEFAULT_PANEL_WIDTH: f32 = 320.0;
const MIN_PANEL_WIDTH: f32 = 240.0;
const MAX_PANEL_WIDTH: f32 = 420.0;
const EVENT_ROW_LIMIT: usize = 10;
const MAX_TICK_LABELS: usize = 4;
const EVENT_ROW_LABEL_MAX_CHARS: usize = 72;

fn adaptive_panel_default_width(available_width: f32) -> f32 {
    let width = if available_width.is_finite() {
        available_width
    } else {
        DEFAULT_PANEL_WIDTH
    };
    (width * 0.22).clamp(MIN_PANEL_WIDTH, MAX_PANEL_WIDTH)
}

#[derive(SystemParam)]
pub(super) struct RightPanelParams<'w, 's> {
    panel_width: ResMut<'w, RightPanelWidthState>,
    layout_state: ResMut<'w, RightPanelLayoutState>,
    i18n: Option<ResMut<'w, UiI18n>>,
    copyable_panel_state: ResMut<'w, CopyableTextPanelState>,
    overlay_config: ResMut<'w, WorldOverlayConfig>,
    state: Res<'w, ViewerState>,
    selection: ResMut<'w, ViewerSelection>,
    viewer_3d_config: Option<Res<'w, Viewer3dConfig>>,
    loading: ResMut<'w, StepControlLoadingState>,
    client: Option<Res<'w, ViewerClient>>,
    timeline: ResMut<'w, TimelineUiState>,
    timeline_filters: ResMut<'w, TimelineMarkFilterState>,
    diagnosis_state: Res<'w, DiagnosisState>,
    link_state: ResMut<'w, EventObjectLinkState>,
    scene: Res<'w, crate::Viewer3dScene>,
    transforms: Query<'w, 's, (&'static mut Transform, Option<&'static crate::BaseScale>)>,
}

pub(super) fn render_right_side_panel_egui(
    mut contexts: EguiContexts,
    mut cjk_font_initialized: Local<bool>,
    params: RightPanelParams,
) {
    let RightPanelParams {
        mut panel_width,
        mut layout_state,
        mut i18n,
        mut copyable_panel_state,
        mut overlay_config,
        state,
        mut selection,
        viewer_3d_config,
        mut loading,
        client,
        mut timeline,
        mut timeline_filters,
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

    let default_panel_width = adaptive_panel_default_width(context.available_rect().width());
    let panel_response = egui::SidePanel::right("viewer-right-side-panel")
        .resizable(true)
        .default_width(default_panel_width)
        .width_range(MIN_PANEL_WIDTH..=MAX_PANEL_WIDTH)
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

                if ui
                    .button(copyable_panel_toggle_label(
                        copyable_panel_state.visible,
                        locale,
                    ))
                    .clicked()
                {
                    copyable_panel_state.visible = !copyable_panel_state.visible;
                }

                ui.label(top_controls_label(locale));
            });

            if layout_state.top_panel_collapsed {
                return;
            }

            ui.separator();
            render_control_buttons(ui, locale, &state, loading.as_mut(), client.as_deref());
            render_overview_section(ui, locale, &state, &selection, timeline.as_ref());

            ui.separator();
            render_overlay_section(
                ui,
                locale,
                &state,
                &viewer_3d_config,
                overlay_config.as_mut(),
            );

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

            ui.separator();
            ui.strong(if locale.is_zh() {
                "事件联动"
            } else {
                "Event Link"
            });
            ui.horizontal_wrapped(|ui| {
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

            if copyable_panel_state.visible {
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

    panel_width.width_px = panel_response.response.rect.width();
}

fn render_control_buttons(
    ui: &mut egui::Ui,
    locale: crate::i18n::UiLocale,
    state: &ViewerState,
    loading: &mut StepControlLoadingState,
    client: Option<&ViewerClient>,
) {
    ui.horizontal_wrapped(|ui| {
        let controls = [
            (
                control_button_label(&ViewerControl::Play, locale),
                ViewerControl::Play,
            ),
            (
                control_button_label(&ViewerControl::Pause, locale),
                ViewerControl::Pause,
            ),
            (
                step_button_label(locale, loading.pending),
                ViewerControl::Step { count: 1 },
            ),
            (
                control_button_label(&ViewerControl::Seek { tick: 0 }, locale),
                ViewerControl::Seek { tick: 0 },
            ),
        ];

        for (label, control) in controls {
            let disabled = matches!(control, ViewerControl::Step { .. }) && loading.pending;
            if ui
                .add_enabled(!disabled, egui::Button::new(label))
                .clicked()
            {
                mark_step_loading_on_control(&control, state, loading);
                if let Some(client) = client {
                    let _ = client.tx.send(agent_world::viewer::ViewerRequest::Control {
                        mode: control.clone(),
                    });
                }
            }
        }
    });
}

fn render_overview_section(
    ui: &mut egui::Ui,
    locale: crate::i18n::UiLocale,
    state: &ViewerState,
    selection: &ViewerSelection,
    timeline: &TimelineUiState,
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

    let sections = [
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
        (
            if locale.is_zh() {
                "选中详情"
            } else {
                "Selection Details"
            },
            details,
        ),
        (if locale.is_zh() { "事件" } else { "Events" }, events),
    ];

    for (title, content) in sections {
        ui.group(|ui| {
            ui.strong(title);
            ui.add(egui::Label::new(content).wrap().selectable(true));
        });
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
mod tests {
    use super::*;

    #[test]
    fn adaptive_panel_width_clamps_to_bounds() {
        assert_eq!(adaptive_panel_default_width(200.0), MIN_PANEL_WIDTH);
        assert_eq!(adaptive_panel_default_width(10_000.0), MAX_PANEL_WIDTH);
        assert_eq!(adaptive_panel_default_width(1200.0), 264.0);
        assert_eq!(adaptive_panel_default_width(1500.0), 330.0);
    }

    #[test]
    fn connection_signal_matches_status() {
        let (text, _) = connection_signal(
            &crate::ConnectionStatus::Connected,
            crate::i18n::UiLocale::ZhCn,
        );
        assert_eq!(text, "连接正常");
        let (text, _) = connection_signal(
            &crate::ConnectionStatus::Connecting,
            crate::i18n::UiLocale::EnUs,
        );
        assert_eq!(text, "Connecting");
        let (text, _) = connection_signal(
            &crate::ConnectionStatus::Error("x".to_string()),
            crate::i18n::UiLocale::EnUs,
        );
        assert_eq!(text, "Conn Error");
    }

    #[test]
    fn rejection_event_count_only_counts_rejected_events() {
        use agent_world::geometry::GeoPos;
        use agent_world::simulator::{RejectReason, WorldEvent, WorldEventKind};

        let events = vec![
            WorldEvent {
                id: 1,
                time: 1,
                kind: WorldEventKind::LocationRegistered {
                    location_id: "loc-1".to_string(),
                    name: "Alpha".to_string(),
                    pos: GeoPos::new(0.0, 0.0, 0.0),
                    profile: Default::default(),
                },
            },
            WorldEvent {
                id: 2,
                time: 2,
                kind: WorldEventKind::ActionRejected {
                    reason: RejectReason::AgentNotFound {
                        agent_id: "a-1".to_string(),
                    },
                },
            },
        ];

        assert_eq!(rejection_event_count(&events), 1);
    }
}
