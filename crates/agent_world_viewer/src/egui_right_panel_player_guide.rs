use bevy_egui::egui;
use std::collections::HashMap;

use crate::{RightPanelLayoutState, ViewerSelection};

use super::egui_right_panel_player_experience::PlayerGuideStep;

const PLAYER_CINEMATIC_FADE_IN_TICKS: u64 = 6;
const PLAYER_CINEMATIC_HOLD_END_TICKS: u64 = 28;
const PLAYER_CINEMATIC_FADE_OUT_END_TICKS: u64 = 44;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PlayerGuideProgressSnapshot {
    pub(super) connect_world_done: bool,
    pub(super) open_panel_done: bool,
    pub(super) select_target_done: bool,
    pub(super) explore_ready: bool,
}

impl PlayerGuideProgressSnapshot {
    pub(super) fn completed_steps(self) -> usize {
        let steps = [
            self.connect_world_done,
            self.open_panel_done,
            self.select_target_done,
            self.explore_ready,
        ];
        steps.into_iter().filter(|done| *done).count()
    }

    pub(super) fn is_step_complete(self, step: PlayerGuideStep) -> bool {
        match step {
            PlayerGuideStep::ConnectWorld => self.connect_world_done,
            PlayerGuideStep::OpenPanel => self.open_panel_done,
            PlayerGuideStep::SelectTarget => self.select_target_done,
            PlayerGuideStep::ExploreAction => self.explore_ready,
        }
    }
}

pub(super) fn build_player_guide_progress_snapshot(
    status: &crate::ConnectionStatus,
    layout_state: &RightPanelLayoutState,
    selection: &ViewerSelection,
    action_feedback_seen: bool,
) -> PlayerGuideProgressSnapshot {
    let connect_world_done = matches!(status, crate::ConnectionStatus::Connected);
    let open_panel_done = connect_world_done && !layout_state.panel_hidden;
    let select_target_done = open_panel_done && selection.current.is_some();
    PlayerGuideProgressSnapshot {
        connect_world_done,
        open_panel_done,
        select_target_done,
        explore_ready: select_target_done && action_feedback_seen,
    }
}

pub(super) fn player_goal_title(
    step: PlayerGuideStep,
    locale: crate::i18n::UiLocale,
) -> &'static str {
    match (step, locale.is_zh()) {
        (PlayerGuideStep::ConnectWorld, true) => "等待世界同步",
        (PlayerGuideStep::ConnectWorld, false) => "Waiting For World Sync",
        (PlayerGuideStep::OpenPanel, true) => "展开操作面板",
        (PlayerGuideStep::OpenPanel, false) => "Open Control Panel",
        (PlayerGuideStep::SelectTarget, true) => "选择一个目标",
        (PlayerGuideStep::SelectTarget, false) => "Select A Target",
        (PlayerGuideStep::ExploreAction, true) => "开始推进任务",
        (PlayerGuideStep::ExploreAction, false) => "Advance The Run",
    }
}

pub(super) fn player_goal_detail(
    step: PlayerGuideStep,
    locale: crate::i18n::UiLocale,
) -> &'static str {
    match (step, locale.is_zh()) {
        (PlayerGuideStep::ConnectWorld, true) => "连接建立后，你将看到实时 Tick 与事件流。",
        (PlayerGuideStep::ConnectWorld, false) => {
            "Once connected, live ticks and events will start flowing."
        }
        (PlayerGuideStep::OpenPanel, true) => "按 Tab 或右上角入口按钮，打开面板查看操作入口。",
        (PlayerGuideStep::OpenPanel, false) => {
            "Press Tab or use the top-right toggle to open the panel."
        }
        (PlayerGuideStep::SelectTarget, true) => "点击场景中的 Agent 或地点，查看详情并触发联动。",
        (PlayerGuideStep::SelectTarget, false) => {
            "Click an agent or location in the scene to inspect and interact."
        }
        (PlayerGuideStep::ExploreAction, true) => {
            "点击“直接指挥 Agent”，发送一次移动/采集/建造指令并观察反馈。"
        }
        (PlayerGuideStep::ExploreAction, false) => {
            "Click \"Command Agent\", send one move/harvest/build command, then watch feedback."
        }
    }
}

pub(super) fn player_goal_color(step: PlayerGuideStep) -> egui::Color32 {
    match step {
        PlayerGuideStep::ConnectWorld => egui::Color32::from_rgb(122, 88, 34),
        PlayerGuideStep::OpenPanel => egui::Color32::from_rgb(44, 92, 152),
        PlayerGuideStep::SelectTarget => egui::Color32::from_rgb(30, 112, 88),
        PlayerGuideStep::ExploreAction => egui::Color32::from_rgb(38, 128, 74),
    }
}

pub(super) fn player_goal_badge(locale: crate::i18n::UiLocale) -> &'static str {
    if locale.is_zh() {
        "下一步目标"
    } else {
        "Next Goal"
    }
}

pub(super) fn player_guide_progress_badge(locale: crate::i18n::UiLocale) -> &'static str {
    if locale.is_zh() {
        "引导进度"
    } else {
        "Guide Progress"
    }
}

pub(super) fn player_onboarding_title(locale: crate::i18n::UiLocale) -> &'static str {
    if locale.is_zh() {
        "新手引导"
    } else {
        "Player Guide"
    }
}

pub(super) fn player_onboarding_primary_action(
    step: PlayerGuideStep,
    locale: crate::i18n::UiLocale,
) -> &'static str {
    match (step, locale.is_zh()) {
        (PlayerGuideStep::ConnectWorld, true) => "知道了",
        (PlayerGuideStep::ConnectWorld, false) => "Got it",
        (PlayerGuideStep::OpenPanel, true) => "打开面板",
        (PlayerGuideStep::OpenPanel, false) => "Open panel",
        (PlayerGuideStep::SelectTarget, true) => "我来选择",
        (PlayerGuideStep::SelectTarget, false) => "I'll select",
        (PlayerGuideStep::ExploreAction, true) => "继续探索",
        (PlayerGuideStep::ExploreAction, false) => "Keep playing",
    }
}

pub(super) fn player_onboarding_dismiss(locale: crate::i18n::UiLocale) -> &'static str {
    if locale.is_zh() {
        "关闭当前提示"
    } else {
        "Hide this tip"
    }
}

pub(super) fn render_player_guide_progress_lines(
    ui: &mut egui::Ui,
    locale: crate::i18n::UiLocale,
    progress: PlayerGuideProgressSnapshot,
    step: PlayerGuideStep,
    tone: egui::Color32,
) {
    ui.small(format!(
        "{} {}/4",
        player_guide_progress_badge(locale),
        progress.completed_steps()
    ));
    let steps = [
        PlayerGuideStep::ConnectWorld,
        PlayerGuideStep::OpenPanel,
        PlayerGuideStep::SelectTarget,
        PlayerGuideStep::ExploreAction,
    ];
    for item in steps {
        let marker = if progress.is_step_complete(item) {
            "✓"
        } else if item == step {
            "▶"
        } else {
            "·"
        };
        ui.small(
            egui::RichText::new(format!("{marker} {}", player_goal_title(item, locale))).color(
                if item == step {
                    tone
                } else {
                    egui::Color32::from_gray(178)
                },
            ),
        );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PlayerLayoutPreset {
    Mission,
    Command,
    Intel,
}

fn player_layout_preset_label(
    preset: PlayerLayoutPreset,
    locale: crate::i18n::UiLocale,
) -> &'static str {
    match (preset, locale.is_zh()) {
        (PlayerLayoutPreset::Mission, true) => "任务",
        (PlayerLayoutPreset::Mission, false) => "Mission",
        (PlayerLayoutPreset::Command, true) => "指挥",
        (PlayerLayoutPreset::Command, false) => "Command",
        (PlayerLayoutPreset::Intel, true) => "情报",
        (PlayerLayoutPreset::Intel, false) => "Intel",
    }
}

pub(super) fn resolve_player_layout_preset(
    layout_state: &RightPanelLayoutState,
    module_visibility: &crate::right_panel_module_visibility::RightPanelModuleVisibilityState,
) -> PlayerLayoutPreset {
    if !layout_state.panel_hidden
        && module_visibility.show_chat
        && !module_visibility.show_timeline
        && !module_visibility.show_details
    {
        return PlayerLayoutPreset::Command;
    }

    if module_visibility.show_timeline || module_visibility.show_details {
        return PlayerLayoutPreset::Intel;
    }

    PlayerLayoutPreset::Mission
}

pub(super) fn apply_player_layout_preset(
    layout_state: &mut RightPanelLayoutState,
    module_visibility: &mut crate::right_panel_module_visibility::RightPanelModuleVisibilityState,
    preset: PlayerLayoutPreset,
) {
    layout_state.panel_hidden = false;
    layout_state.top_panel_collapsed = false;
    module_visibility.show_controls = false;
    module_visibility.show_overlay = false;
    module_visibility.show_diagnosis = false;

    match preset {
        PlayerLayoutPreset::Mission => {
            module_visibility.show_overview = true;
            module_visibility.show_chat = false;
            module_visibility.show_event_link = true;
            module_visibility.show_timeline = false;
            module_visibility.show_details = false;
        }
        PlayerLayoutPreset::Command => {
            module_visibility.show_overview = true;
            module_visibility.show_chat = true;
            module_visibility.show_event_link = true;
            module_visibility.show_timeline = false;
            module_visibility.show_details = false;
        }
        PlayerLayoutPreset::Intel => {
            module_visibility.show_overview = true;
            module_visibility.show_chat = false;
            module_visibility.show_event_link = true;
            module_visibility.show_timeline = true;
            module_visibility.show_details = true;
        }
    }
}

pub(super) fn render_player_layout_preset_strip(
    context: &egui::Context,
    layout_state: &mut RightPanelLayoutState,
    module_visibility: &mut crate::right_panel_module_visibility::RightPanelModuleVisibilityState,
    locale: crate::i18n::UiLocale,
    now_secs: f64,
) {
    if !should_show_player_layout_preset_strip(layout_state.panel_hidden) {
        return;
    }
    let active = resolve_player_layout_preset(layout_state, module_visibility);
    let anchor_y = player_layout_preset_strip_anchor_y(layout_state.panel_hidden);
    let pulse = ((now_secs * 1.5).sin() * 0.5 + 0.5) as f32;
    egui::Area::new(egui::Id::new("viewer-player-layout-strip"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, anchor_y))
        .movable(false)
        .interactable(true)
        .show(context, |ui| {
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgba_unmultiplied(16, 24, 37, 214))
                .stroke(egui::Stroke::new(
                    1.0 + 0.4 * pulse,
                    egui::Color32::from_rgb(64, 106, 152),
                ))
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    ui.small(if locale.is_zh() {
                        "布局焦点"
                    } else {
                        "Layout Focus"
                    });
                    ui.horizontal_wrapped(|ui| {
                        for preset in [
                            PlayerLayoutPreset::Mission,
                            PlayerLayoutPreset::Command,
                            PlayerLayoutPreset::Intel,
                        ] {
                            if ui
                                .selectable_label(
                                    active == preset,
                                    player_layout_preset_label(preset, locale),
                                )
                                .clicked()
                            {
                                apply_player_layout_preset(layout_state, module_visibility, preset);
                            }
                        }
                    });
                });
        });
}

pub(super) fn should_show_player_layout_preset_strip(panel_hidden: bool) -> bool {
    !panel_hidden
}

pub(super) fn player_layout_preset_strip_anchor_y(panel_hidden: bool) -> f32 {
    if should_show_player_layout_preset_strip(panel_hidden) {
        58.0
    } else {
        0.0
    }
}

fn player_current_tick(state: &crate::ViewerState) -> u64 {
    state
        .snapshot
        .as_ref()
        .map(|snapshot| snapshot.time)
        .or_else(|| state.metrics.as_ref().map(|metrics| metrics.total_ticks))
        .unwrap_or(0)
}

pub(super) fn player_cinematic_intro_alpha(status: &crate::ConnectionStatus, tick: u64) -> f32 {
    if !matches!(status, crate::ConnectionStatus::Connected)
        || tick > PLAYER_CINEMATIC_FADE_OUT_END_TICKS
    {
        return 0.0;
    }
    if tick <= PLAYER_CINEMATIC_FADE_IN_TICKS {
        ((tick + 1) as f32 / (PLAYER_CINEMATIC_FADE_IN_TICKS + 1) as f32).clamp(0.0, 1.0)
    } else if tick <= PLAYER_CINEMATIC_HOLD_END_TICKS {
        1.0
    } else {
        (1.0 - (tick - PLAYER_CINEMATIC_HOLD_END_TICKS) as f32
            / (PLAYER_CINEMATIC_FADE_OUT_END_TICKS - PLAYER_CINEMATIC_HOLD_END_TICKS) as f32)
            .clamp(0.0, 1.0)
    }
}

fn player_cinematic_subtitle(step: PlayerGuideStep, locale: crate::i18n::UiLocale) -> &'static str {
    match (step, locale.is_zh()) {
        (PlayerGuideStep::ConnectWorld, true) => "世界链路建立中，准备接入前哨视角。",
        (PlayerGuideStep::ConnectWorld, false) => {
            "World link is stabilizing. Preparing outpost feed."
        }
        (PlayerGuideStep::OpenPanel, true) => "先展开指挥面板，领取第一条任务线。",
        (PlayerGuideStep::OpenPanel, false) => {
            "Open the control panel to claim your first mission loop."
        }
        (PlayerGuideStep::SelectTarget, true) => "锁定一个目标，你的行动将立刻改变世界。",
        (PlayerGuideStep::SelectTarget, false) => {
            "Lock a target. Your next action will change this world."
        }
        (PlayerGuideStep::ExploreAction, true) => "保持节奏推进任务，连续反馈会持续强化。",
        (PlayerGuideStep::ExploreAction, false) => {
            "Keep the loop moving. Feedback intensity will ramp up."
        }
    }
}

pub(super) fn render_player_cinematic_intro(
    context: &egui::Context,
    state: &crate::ViewerState,
    step: PlayerGuideStep,
    locale: crate::i18n::UiLocale,
    now_secs: f64,
) {
    let tick = player_current_tick(state);
    let alpha = player_cinematic_intro_alpha(&state.status, tick);
    if alpha <= 0.01 {
        return;
    }
    let pulse = ((now_secs * 1.6).sin() * 0.5 + 0.5) as f32;
    let tone = player_goal_color(step);
    let to_u8 = |value: f32| (value.clamp(0.0, 255.0)) as u8;
    egui::Area::new(egui::Id::new("viewer-player-cinematic-intro"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 56.0))
        .movable(false)
        .interactable(false)
        .show(context, |ui| {
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgba_unmultiplied(
                    8,
                    16,
                    24,
                    to_u8(226.0 * alpha),
                ))
                .stroke(egui::Stroke::new(
                    1.0 + 0.6 * pulse,
                    egui::Color32::from_rgba_unmultiplied(
                        tone.r(),
                        tone.g(),
                        tone.b(),
                        to_u8((136.0 + 92.0 * pulse) * alpha),
                    ),
                ))
                .corner_radius(egui::CornerRadius::same(12))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.set_max_width(560.0);
                    ui.vertical_centered(|ui| {
                        ui.small(
                            egui::RichText::new(if locale.is_zh() {
                                "沉浸开场"
                            } else {
                                "Immersive Intro"
                            })
                            .color(tone),
                        );
                        ui.strong(if locale.is_zh() {
                            "前哨部署完成"
                        } else {
                            "Outpost Deployment Ready"
                        });
                        ui.label(player_cinematic_subtitle(step, locale));
                        ui.small(if locale.is_zh() {
                            "按 Tab 可随时展开控制面板"
                        } else {
                            "Press Tab to open the control panel anytime"
                        });
                    });
                });
        });
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlayerMissionLoopSnapshot {
    pub(super) completed_steps: usize,
    pub(super) title: &'static str,
    pub(super) objective: &'static str,
    pub(super) action_label: &'static str,
    pub(super) action_opens_panel: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlayerRewardFeedbackSnapshot {
    pub(super) badge: &'static str,
    pub(super) title: &'static str,
    pub(super) detail: String,
    pub(super) complete: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PlayerMiniMapPoint {
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) selected: bool,
}

pub(super) fn build_player_mission_loop_snapshot(
    step: PlayerGuideStep,
    progress: PlayerGuideProgressSnapshot,
    locale: crate::i18n::UiLocale,
) -> PlayerMissionLoopSnapshot {
    let (action_label, action_opens_panel) = match (step, locale.is_zh()) {
        (PlayerGuideStep::ConnectWorld, true) => ("等待连接完成", false),
        (PlayerGuideStep::ConnectWorld, false) => ("Await sync", false),
        (PlayerGuideStep::OpenPanel, true) => ("打开操作面板", true),
        (PlayerGuideStep::OpenPanel, false) => ("Open control panel", true),
        (PlayerGuideStep::SelectTarget, true) => ("锁定一个目标", false),
        (PlayerGuideStep::SelectTarget, false) => ("Lock one target", false),
        (PlayerGuideStep::ExploreAction, true) => ("打开指挥并发送 1 条指令", false),
        (PlayerGuideStep::ExploreAction, false) => ("Open command and send 1 order", false),
    };
    PlayerMissionLoopSnapshot {
        completed_steps: progress.completed_steps(),
        title: if locale.is_zh() {
            "主任务：建立行动闭环"
        } else {
            "Mission: Build Action Loop"
        },
        objective: player_goal_title(step, locale),
        action_label,
        action_opens_panel,
    }
}

pub(super) fn build_player_reward_feedback_snapshot(
    progress: PlayerGuideProgressSnapshot,
    locale: crate::i18n::UiLocale,
) -> PlayerRewardFeedbackSnapshot {
    let completed_steps = progress.completed_steps();
    match (completed_steps, locale.is_zh()) {
        (4, true) => PlayerRewardFeedbackSnapshot {
            badge: "任务奖励",
            title: "闭环达成",
            detail: "你已打通完整上手路径，可持续推进行动循环。".to_string(),
            complete: true,
        },
        (4, false) => PlayerRewardFeedbackSnapshot {
            badge: "Reward",
            title: "Loop Completed",
            detail: "You finished the onboarding loop and unlocked the full play rhythm."
                .to_string(),
            complete: true,
        },
        (_, true) => PlayerRewardFeedbackSnapshot {
            badge: "进度奖励",
            title: "节奏提升中",
            detail: format!("已完成 {completed_steps}/4 步，继续推进可触发闭环达成反馈。"),
            complete: false,
        },
        (_, false) => PlayerRewardFeedbackSnapshot {
            badge: "Progress Reward",
            title: "Momentum Building",
            detail: format!(
                "{completed_steps}/4 steps completed. Keep pushing to trigger completion feedback."
            ),
            complete: false,
        },
    }
}

pub(super) fn resolve_selected_location_id_for_minimap(
    selection: &ViewerSelection,
    agent_locations: &HashMap<String, String>,
) -> Option<String> {
    let current = selection.current.as_ref()?;
    match current.kind {
        crate::SelectionKind::Location => Some(current.id.clone()),
        crate::SelectionKind::Agent => agent_locations.get(current.id.as_str()).cloned(),
        _ => None,
    }
}

pub(super) fn build_player_minimap_points(
    raw_points: &[(String, f32, f32)],
    selected_location_id: Option<&str>,
) -> Vec<PlayerMiniMapPoint> {
    if raw_points.is_empty() {
        return Vec::new();
    }

    let mut sorted_points = raw_points.to_vec();
    sorted_points.sort_by(|left, right| left.0.cmp(&right.0));

    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_z = f32::INFINITY;
    let mut max_z = f32::NEG_INFINITY;
    for (_, x, z) in &sorted_points {
        min_x = min_x.min(*x);
        max_x = max_x.max(*x);
        min_z = min_z.min(*z);
        max_z = max_z.max(*z);
    }

    let span_x = (max_x - min_x).max(1.0);
    let span_z = (max_z - min_z).max(1.0);
    sorted_points
        .into_iter()
        .map(|(id, x, z)| PlayerMiniMapPoint {
            x: ((x - min_x) / span_x).clamp(0.0, 1.0),
            y: (1.0 - (z - min_z) / span_z).clamp(0.0, 1.0),
            selected: selected_location_id == Some(id.as_str()),
        })
        .collect()
}

fn build_player_minimap_snapshot(
    state: &crate::ViewerState,
    selection: &ViewerSelection,
) -> Vec<PlayerMiniMapPoint> {
    let Some(snapshot) = state.snapshot.as_ref() else {
        return Vec::new();
    };
    let agent_locations = snapshot
        .model
        .agents
        .iter()
        .map(|(agent_id, agent)| (agent_id.clone(), agent.location_id.clone()))
        .collect::<HashMap<_, _>>();
    let selected_location_id =
        resolve_selected_location_id_for_minimap(selection, &agent_locations);
    let raw_points = snapshot
        .model
        .locations
        .iter()
        .map(|(location_id, location)| {
            (
                location_id.clone(),
                location.pos.x_cm as f32,
                location.pos.z_cm as f32,
            )
        })
        .collect::<Vec<_>>();
    build_player_minimap_points(&raw_points, selected_location_id.as_deref())
}

fn render_player_minimap_card(
    context: &egui::Context,
    points: &[PlayerMiniMapPoint],
    locale: crate::i18n::UiLocale,
    now_secs: f64,
) {
    let pulse = ((now_secs * 2.4).sin() * 0.5 + 0.5) as f32;
    egui::Area::new(egui::Id::new("viewer-player-mini-map"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-14.0, -14.0))
        .movable(false)
        .interactable(false)
        .show(context, |ui| {
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgba_unmultiplied(13, 20, 32, 224))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(56, 96, 146)))
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.set_max_width(230.0);
                    ui.small(if locale.is_zh() {
                        "战术小地图"
                    } else {
                        "Tactical Mini-map"
                    });
                    let map_size = egui::vec2(190.0, 110.0);
                    let (rect, _) = ui.allocate_exact_size(map_size, egui::Sense::hover());
                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 6.0, egui::Color32::from_rgb(20, 30, 46));
                    painter.rect_stroke(
                        rect,
                        6.0,
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(48, 72, 108)),
                        egui::StrokeKind::Outside,
                    );
                    painter.line_segment(
                        [
                            egui::pos2(rect.center().x, rect.top()),
                            egui::pos2(rect.center().x, rect.bottom()),
                        ],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 58, 86)),
                    );
                    painter.line_segment(
                        [
                            egui::pos2(rect.left(), rect.center().y),
                            egui::pos2(rect.right(), rect.center().y),
                        ],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 58, 86)),
                    );

                    let mut selected_count = 0usize;
                    for point in points {
                        let pos = egui::pos2(
                            rect.left() + point.x * rect.width(),
                            rect.top() + point.y * rect.height(),
                        );
                        if point.selected {
                            selected_count = selected_count.saturating_add(1);
                        }
                        let radius = if point.selected {
                            4.2 + 1.4 * pulse
                        } else {
                            2.8
                        };
                        let color = if point.selected {
                            egui::Color32::from_rgb(244, 196, 96)
                        } else {
                            egui::Color32::from_rgb(92, 150, 218)
                        };
                        painter.circle_filled(pos, radius, color);
                    }

                    if points.is_empty() {
                        ui.small(if locale.is_zh() {
                            "等待位置数据..."
                        } else {
                            "Waiting for location data..."
                        });
                    } else {
                        ui.small(format!(
                            "{} {} | {} {}",
                            if locale.is_zh() {
                                "地点"
                            } else {
                                "Locations"
                            },
                            points.len(),
                            if locale.is_zh() { "选中" } else { "Selected" },
                            selected_count
                        ));
                    }
                });
        });
}

pub(super) fn render_player_mission_hud(
    context: &egui::Context,
    state: &crate::ViewerState,
    selection: &ViewerSelection,
    layout_state: &mut RightPanelLayoutState,
    module_visibility: &mut crate::right_panel_module_visibility::RightPanelModuleVisibilityState,
    onboarding_visible: bool,
    step: PlayerGuideStep,
    progress: PlayerGuideProgressSnapshot,
    locale: crate::i18n::UiLocale,
    now_secs: f64,
) {
    let snapshot = build_player_mission_loop_snapshot(step, progress, locale);
    let reward = build_player_reward_feedback_snapshot(progress, locale);
    let tone = player_goal_color(step);
    let reward_tone = if reward.complete {
        egui::Color32::from_rgb(54, 166, 96)
    } else {
        egui::Color32::from_rgb(74, 126, 184)
    };
    let compact_mode = player_mission_hud_compact_mode(layout_state.panel_hidden);
    let mission_anchor_y =
        player_mission_hud_anchor_y(layout_state.panel_hidden, onboarding_visible);
    let show_command_action = player_mission_hud_show_command_action(layout_state.panel_hidden);
    let pulse = ((now_secs * 1.8).sin() * 0.5 + 0.5) as f32;
    let mut action_clicked = false;
    let mut command_clicked = false;
    egui::Area::new(egui::Id::new("viewer-player-mission-hud"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(14.0, mission_anchor_y))
        .movable(false)
        .interactable(true)
        .show(context, |ui| {
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgba_unmultiplied(14, 22, 34, 230))
                .stroke(egui::Stroke::new(
                    1.0 + 0.45 * pulse,
                    egui::Color32::from_rgba_unmultiplied(
                        tone.r(),
                        tone.g(),
                        tone.b(),
                        (150.0 + 86.0 * pulse).round() as u8,
                    ),
                ))
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.set_max_width(if compact_mode { 280.0 } else { 320.0 });
                    ui.small(egui::RichText::new(snapshot.title).color(tone).strong());
                    ui.strong(snapshot.objective);
                    if !compact_mode {
                        ui.small(player_goal_detail(step, locale));
                    }
                    let progress_ratio = (snapshot.completed_steps as f32 / 4.0).clamp(0.0, 1.0);
                    ui.add(
                        egui::ProgressBar::new(progress_ratio)
                            .desired_width(280.0)
                            .text(format!(
                                "{} {}/4",
                                if locale.is_zh() {
                                    "任务进度"
                                } else {
                                    "Mission Progress"
                                },
                                snapshot.completed_steps
                            )),
                    );
                    if compact_mode {
                        ui.small(egui::RichText::new(reward.badge).color(reward_tone));
                    } else {
                        egui::Frame::group(ui.style())
                            .fill(egui::Color32::from_rgba_unmultiplied(
                                reward_tone.r(),
                                reward_tone.g(),
                                reward_tone.b(),
                                if reward.complete { 54 } else { 34 },
                            ))
                            .stroke(egui::Stroke::new(1.0, reward_tone))
                            .corner_radius(egui::CornerRadius::same(8))
                            .inner_margin(egui::Margin::same(8))
                            .show(ui, |ui| {
                                ui.small(egui::RichText::new(reward.badge).color(reward_tone));
                                ui.strong(reward.title);
                                ui.small(reward.detail.as_str());
                            });
                    }
                    ui.horizontal_wrapped(|ui| {
                        action_clicked = ui.button(snapshot.action_label).clicked();
                        if show_command_action {
                            command_clicked = ui
                                .button(if locale.is_zh() {
                                    "直接指挥 Agent"
                                } else {
                                    "Command Agent"
                                })
                                .clicked();
                        }
                    });
                });
        });

    if action_clicked && snapshot.action_opens_panel {
        layout_state.panel_hidden = false;
    }
    if command_clicked {
        apply_player_layout_preset(layout_state, module_visibility, PlayerLayoutPreset::Command);
    }

    let points = build_player_minimap_snapshot(state, selection);
    render_player_minimap_card(context, &points, locale, now_secs);
}

pub(super) fn player_mission_hud_compact_mode(panel_hidden: bool) -> bool {
    !panel_hidden
}

pub(super) fn player_mission_hud_anchor_y(panel_hidden: bool, onboarding_visible: bool) -> f32 {
    if player_mission_hud_compact_mode(panel_hidden) {
        96.0
    } else if onboarding_visible {
        214.0
    } else {
        136.0
    }
}

pub(super) fn player_mission_hud_show_command_action(panel_hidden: bool) -> bool {
    panel_hidden
}
