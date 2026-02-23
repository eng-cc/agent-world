use bevy_egui::egui;

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
) -> PlayerGuideProgressSnapshot {
    let connect_world_done = matches!(status, crate::ConnectionStatus::Connected);
    let open_panel_done = connect_world_done && !layout_state.panel_hidden;
    let select_target_done = open_panel_done && selection.current.is_some();
    PlayerGuideProgressSnapshot {
        connect_world_done,
        open_panel_done,
        select_target_done,
        explore_ready: select_target_done,
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
        (PlayerGuideStep::ExploreAction, true) => "保持观察目标状态，按需执行移动、采集或建造。",
        (PlayerGuideStep::ExploreAction, false) => {
            "Track your target and execute move, harvest, or build actions."
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
        (PlayerGuideStep::ExploreAction, true) => ("执行一次关键行动", false),
        (PlayerGuideStep::ExploreAction, false) => ("Run one key action", false),
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

pub(super) fn render_player_mission_hud(
    context: &egui::Context,
    layout_state: &mut RightPanelLayoutState,
    step: PlayerGuideStep,
    progress: PlayerGuideProgressSnapshot,
    locale: crate::i18n::UiLocale,
    now_secs: f64,
) {
    let snapshot = build_player_mission_loop_snapshot(step, progress, locale);
    let tone = player_goal_color(step);
    let pulse = ((now_secs * 1.8).sin() * 0.5 + 0.5) as f32;
    let mut action_clicked = false;
    egui::Area::new(egui::Id::new("viewer-player-mission-hud"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(14.0, 136.0))
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
                    ui.set_max_width(320.0);
                    ui.small(egui::RichText::new(snapshot.title).color(tone).strong());
                    ui.strong(snapshot.objective);
                    ui.small(player_goal_detail(step, locale));
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
                    action_clicked = ui.button(snapshot.action_label).clicked();
                });
        });

    if action_clicked && snapshot.action_opens_panel {
        layout_state.panel_hidden = false;
    }
}
