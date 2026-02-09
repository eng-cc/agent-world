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
    overlay_loading, seek_button_label, selection_line, status_line, timeline_insights,
    timeline_jump_label, timeline_mode_label, timeline_status_line,
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

const DEFAULT_PANEL_WIDTH: f32 = 380.0;
const MIN_PANEL_WIDTH: f32 = 320.0;
const MAX_PANEL_WIDTH: f32 = 620.0;
const EVENT_ROW_LIMIT: usize = 10;
const MAX_TICK_LABELS: usize = 4;

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

    let panel_response = egui::SidePanel::right("viewer-right-side-panel")
        .resizable(true)
        .default_width(DEFAULT_PANEL_WIDTH)
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
            ui.add(egui::Label::new(status_line(&state.status, locale)).selectable(true));
            ui.add(egui::Label::new(selection_line(&selection, locale)).selectable(true));

            ui.separator();
            render_overlay_section(
                ui,
                locale,
                &state,
                &viewer_3d_config,
                overlay_config.as_mut(),
            );

            ui.separator();
            ui.collapsing(
                if locale.is_zh() {
                    "诊断"
                } else {
                    "Diagnosis"
                },
                |ui| {
                    ui.add(
                        egui::Label::new(diagnosis_state.text.as_str())
                            .wrap()
                            .selectable(true),
                    );
                },
            );

            ui.collapsing(
                if locale.is_zh() {
                    "事件联动"
                } else {
                    "Event Link"
                },
                |ui| {
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
                },
            );

            ui.collapsing(
                if locale.is_zh() {
                    "时间轴"
                } else {
                    "Timeline"
                },
                |ui| {
                    render_timeline_section(
                        ui,
                        locale,
                        &state,
                        timeline.as_mut(),
                        timeline_filters.as_mut(),
                        client.as_deref(),
                    );
                },
            );

            if copyable_panel_state.visible {
                ui.separator();
                ui.heading(copy_panel_title(locale));
                ui.label(copy_panel_hint(locale));

                egui::ScrollArea::vertical().show(ui, |ui| {
                    render_text_sections(
                        ui,
                        locale,
                        &state,
                        &selection,
                        timeline.as_ref(),
                        &viewer_3d_config,
                        &diagnosis_state,
                        &link_state,
                    );

                    ui.collapsing(
                        if locale.is_zh() {
                            "事件行"
                        } else {
                            "Event Rows"
                        },
                        |ui| {
                            let focus = focus_tick(&state, Some(timeline.as_ref()));
                            let (rows, focused_event_id) =
                                event_window(&state.events, focus, EVENT_ROW_LIMIT);

                            if rows.is_empty() {
                                ui.label(crate::ui_locale_text::event_links_empty(locale));
                                return;
                            }

                            for event in rows {
                                let line = event_row_label(
                                    event,
                                    focused_event_id == Some(event.id),
                                    locale,
                                );
                                if ui.add(egui::Button::new(line.as_str())).clicked() {
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
                                        link_state.message =
                                            "Link: viewer config unavailable".to_string();
                                    }
                                }
                            }
                        },
                    );
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
    diagnosis_state: &DiagnosisState,
    link_state: &EventObjectLinkState,
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
            if locale.is_zh() { "状态" } else { "Status" },
            status_line(&state.status, locale),
        ),
        (
            if locale.is_zh() {
                "当前选择"
            } else {
                "Selection"
            },
            selection_line(selection, locale),
        ),
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
        (
            if locale.is_zh() {
                "诊断"
            } else {
                "Diagnosis"
            },
            diagnosis_state.text.clone(),
        ),
        (
            if locale.is_zh() {
                "事件联动"
            } else {
                "Event Link"
            },
            map_link_message_for_locale(&link_state.message, locale),
        ),
    ];

    for (title, content) in sections {
        ui.collapsing(title, |ui| {
            ui.add(egui::Label::new(content).wrap().selectable(true));
        });
    }

    if let Some(current) = selection.current.as_ref() {
        ui.collapsing(
            if locale.is_zh() {
                "选中类型"
            } else {
                "Selection Kind"
            },
            |ui| {
                ui.label(format!(
                    "{} {}",
                    selection_kind_label(current.kind),
                    current.id
                ));
            },
        );
    }

    ui.collapsing(
        if locale.is_zh() {
            "Tick 标签"
        } else {
            "Tick Labels"
        },
        |ui| {
            let ticks = vec![timeline.target_tick, focus.unwrap_or(timeline.target_tick)];
            let shown: Vec<String> = ticks
                .into_iter()
                .take(MAX_TICK_LABELS)
                .map(|tick| tick.to_string())
                .collect();
            ui.label(shown.join(","));
        },
    );
}
