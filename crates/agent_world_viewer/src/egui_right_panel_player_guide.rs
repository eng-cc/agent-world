use crate::web_test_api::WebTestApiControlFeedbackSnapshot;
use crate::{RightPanelLayoutState, ViewerSelection, ViewerState};
use agent_world::simulator::{
    PlayerGameplayGoalKind, PlayerGameplaySnapshot, PlayerGameplayStageId,
    PlayerGameplayStageStatus, WorldEventKind,
};
use bevy_egui::egui;
use std::collections::HashMap;

use super::egui_right_panel_player_experience::PlayerGuideStep;
use super::egui_right_panel_player_micro_loop::{
    build_player_micro_loop_snapshot, format_due_timer_line, PlayerMicroLoopSnapshot,
    PlayerMicroLoopTone, PlayerNoProgressDiagnosis,
};

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
    panel_hidden
}

pub(super) fn player_layout_preset_strip_anchor_y(panel_hidden: bool) -> f32 {
    if should_show_player_layout_preset_strip(panel_hidden) {
        74.0
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

pub(super) fn player_control_stage_label(
    stage: &str,
    locale: crate::i18n::UiLocale,
) -> &'static str {
    match (stage, locale.is_zh()) {
        ("received", true) => "已接收",
        ("received", false) => "Received",
        ("executing", true) => "执行中",
        ("executing", false) => "Executing",
        ("completed_advanced", true) | ("applied", true) => "已完成（有推进）",
        ("completed_advanced", false) | ("applied", false) => "Completed (advanced)",
        ("completed_no_progress", true) => "已完成（无推进）",
        ("completed_no_progress", false) => "Completed (no progress)",
        ("blocked", true) => "已阻断",
        ("blocked", false) => "Blocked",
        (_, true) => "处理中",
        (_, false) => "Pending",
    }
}

pub(super) fn player_control_stage_color(stage: &str) -> egui::Color32 {
    match stage {
        "completed_advanced" | "applied" => egui::Color32::from_rgb(78, 182, 108),
        "completed_no_progress" => egui::Color32::from_rgb(224, 176, 92),
        "blocked" => egui::Color32::from_rgb(226, 128, 98),
        "executing" | "received" => egui::Color32::from_rgb(118, 168, 236),
        _ => egui::Color32::from_rgb(186, 206, 238),
    }
}

pub(super) fn player_control_stage_shows_recovery_actions(stage: &str) -> bool {
    matches!(stage, "completed_no_progress")
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
    pub(super) completion_condition: &'static str,
    pub(super) eta: &'static str,
    pub(super) short_goals: [PlayerShortGoalSnapshot; 2],
    pub(super) action_label: &'static str,
    pub(super) action_opens_panel: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PlayerShortGoalSnapshot {
    pub(super) label: &'static str,
    pub(super) complete: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlayerRewardFeedbackSnapshot {
    pub(super) badge: &'static str,
    pub(super) title: &'static str,
    pub(super) detail: String,
    pub(super) complete: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PlayerPostOnboardingStatus {
    Active,
    Blocked,
    BranchReady,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlayerPostOnboardingSnapshot {
    pub(super) status: PlayerPostOnboardingStatus,
    pub(super) title: &'static str,
    pub(super) objective: String,
    pub(super) progress_detail: String,
    pub(super) progress_percent: u8,
    pub(super) blocker_detail: Option<String>,
    pub(super) next_step: String,
    pub(super) branch_hint: Option<String>,
    pub(super) action_label: &'static str,
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
        (PlayerGuideStep::ConnectWorld, true) => ("执行下一步：确认连接状态", false),
        (PlayerGuideStep::ConnectWorld, false) => ("Do next step: Check connection", false),
        (PlayerGuideStep::OpenPanel, true) => ("执行下一步：打开面板", true),
        (PlayerGuideStep::OpenPanel, false) => ("Do next step: Open panel", true),
        (PlayerGuideStep::SelectTarget, true) => ("执行下一步：切换任务视图并选目标", false),
        (PlayerGuideStep::SelectTarget, false) => {
            ("Do next step: Switch to mission view and select", false)
        }
        (PlayerGuideStep::ExploreAction, true) => ("执行下一步：打开指挥并开始推进", false),
        (PlayerGuideStep::ExploreAction, false) => ("Do next step: Open command and play", false),
    };
    let short_goals = build_player_short_goals(step, progress, locale);
    PlayerMissionLoopSnapshot {
        completed_steps: progress.completed_steps(),
        title: if locale.is_zh() {
            "主任务：建立行动闭环"
        } else {
            "Mission: Build Action Loop"
        },
        objective: player_goal_action_sentence(step, locale),
        completion_condition: player_goal_completion_condition(step, locale),
        eta: player_goal_eta(step, locale),
        short_goals,
        action_label,
        action_opens_panel,
    }
}

fn build_player_short_goals(
    step: PlayerGuideStep,
    progress: PlayerGuideProgressSnapshot,
    locale: crate::i18n::UiLocale,
) -> [PlayerShortGoalSnapshot; 2] {
    let (labels, done) = match (step, locale.is_zh()) {
        (PlayerGuideStep::ConnectWorld, true) => (
            ["建立世界连接", "展开操作面板"],
            [progress.connect_world_done, progress.open_panel_done],
        ),
        (PlayerGuideStep::ConnectWorld, false) => (
            ["Connect to world", "Open control panel"],
            [progress.connect_world_done, progress.open_panel_done],
        ),
        (PlayerGuideStep::OpenPanel, true) => (
            ["展开操作面板", "锁定一个目标"],
            [progress.open_panel_done, progress.select_target_done],
        ),
        (PlayerGuideStep::OpenPanel, false) => (
            ["Open control panel", "Lock one target"],
            [progress.open_panel_done, progress.select_target_done],
        ),
        (PlayerGuideStep::SelectTarget, true) => (
            ["锁定一个目标", "发送首条指令"],
            [progress.select_target_done, progress.explore_ready],
        ),
        (PlayerGuideStep::SelectTarget, false) => (
            ["Lock one target", "Send first order"],
            [progress.select_target_done, progress.explore_ready],
        ),
        (PlayerGuideStep::ExploreAction, true) => (
            ["发送首条指令", "确认世界反馈"],
            [progress.explore_ready, progress.explore_ready],
        ),
        (PlayerGuideStep::ExploreAction, false) => (
            ["Send first order", "Confirm world feedback"],
            [progress.explore_ready, progress.explore_ready],
        ),
    };

    [
        PlayerShortGoalSnapshot {
            label: labels[0],
            complete: done[0],
        },
        PlayerShortGoalSnapshot {
            label: labels[1],
            complete: done[1],
        },
    ]
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

pub(super) fn build_player_post_onboarding_snapshot(
    state: &ViewerState,
    control_feedback: Option<&WebTestApiControlFeedbackSnapshot>,
    locale: crate::i18n::UiLocale,
) -> PlayerPostOnboardingSnapshot {
    if let Some(gameplay) = state
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.player_gameplay.as_ref())
        .filter(|gameplay| gameplay.stage_id == PlayerGameplayStageId::PostOnboarding)
    {
        return build_player_post_onboarding_snapshot_from_gameplay(gameplay, locale);
    }

    build_player_post_onboarding_snapshot_from_events(state, control_feedback, locale)
}

fn build_player_post_onboarding_snapshot_from_events(
    state: &ViewerState,
    control_feedback: Option<&WebTestApiControlFeedbackSnapshot>,
    locale: crate::i18n::UiLocale,
) -> PlayerPostOnboardingSnapshot {
    let mut has_material_flow = false;
    let mut has_factory_ready = false;
    let mut has_recipe_running = false;
    let mut has_first_output = false;
    let mut latest_blocker = None::<(String, String)>;

    for event in &state.events {
        match &event.kind {
            WorldEventKind::RadiationHarvested { .. } | WorldEventKind::CompoundMined { .. } => {
                has_material_flow = true;
            }
            WorldEventKind::FactoryBuilt { .. } => {
                has_factory_ready = true;
            }
            WorldEventKind::RecipeScheduled { .. } => {
                has_recipe_running = true;
            }
            WorldEventKind::CompoundRefined { .. } => {
                has_material_flow = true;
                has_first_output = true;
            }
            WorldEventKind::RuntimeEvent { kind, domain_kind } => match kind.as_str() {
                "runtime.economy.factory_built" => {
                    has_factory_ready = true;
                }
                "runtime.economy.recipe_started" => {
                    has_recipe_running = true;
                }
                "runtime.economy.recipe_completed" => {
                    has_recipe_running = true;
                    has_first_output = true;
                }
                "runtime.economy.factory_production_blocked" => {
                    has_recipe_running = true;
                    let summary = domain_kind.as_deref().unwrap_or_default();
                    let reason = post_onboarding_summary_value(summary, "reason")
                        .unwrap_or("unknown")
                        .to_string();
                    let detail = post_onboarding_summary_value(summary, "detail")
                        .unwrap_or_default()
                        .to_string();
                    latest_blocker = Some((reason, detail));
                }
                "runtime.economy.factory_production_resumed" => {
                    has_recipe_running = true;
                    latest_blocker = None;
                }
                _ => {}
            },
            _ => {}
        }
    }

    let blocked_feedback = control_feedback.and_then(|feedback| {
        matches!(
            feedback.stage.as_str(),
            "blocked" | "completed_no_progress"
        )
        .then(|| {
            (
                feedback.reason.clone().unwrap_or_else(|| {
                    if locale.is_zh() {
                        "当前行动未形成有效推进".to_string()
                    } else {
                        "the latest command did not create useful forward progress".to_string()
                    }
                }),
                feedback.hint.clone().unwrap_or_default(),
            )
        })
    });

    if has_first_output {
        return PlayerPostOnboardingSnapshot {
            status: PlayerPostOnboardingStatus::BranchReady,
            title: if locale.is_zh() {
                "下一阶段：选择中循环方向"
            } else {
                "Next Stage: Choose Your Mid-loop Path"
            },
            objective: if locale.is_zh() {
                "第一项持续工业能力已建立，开始把它扩张成稳定组织能力。".to_string()
            } else {
                "Your first sustainable industrial capability is online. Turn it into stable organizational momentum.".to_string()
            },
            progress_detail: if locale.is_zh() {
                "阶段进展：已完成首个可见产出/稳定产线里程碑。".to_string()
            } else {
                "Stage progress: your first visible output or stable line milestone is complete."
                    .to_string()
            },
            progress_percent: 100,
            blocker_detail: None,
            next_step: if locale.is_zh() {
                "下一步：保持 Command 视图，继续扩产、推进治理提案，或为关键节点补防护。"
                    .to_string()
            } else {
                "Next: stay in Command view and either expand production, push governance, or secure a critical node."
                    .to_string()
            },
            branch_hint: Some(if locale.is_zh() {
                "已解锁分支：生产扩张 / 治理影响 / 冲突安全".to_string()
            } else {
                "Branches unlocked: Production Expansion / Governance Influence / Conflict Security"
                    .to_string()
            }),
            action_label: if locale.is_zh() {
                "进入指挥并推进 1 步"
            } else {
                "Open command and advance 1 step"
            },
        };
    }

    if let Some((reason, detail)) = latest_blocker.or(blocked_feedback) {
        return PlayerPostOnboardingSnapshot {
            status: PlayerPostOnboardingStatus::Blocked,
            title: if locale.is_zh() {
                "PostOnboarding：恢复持续能力"
            } else {
                "PostOnboarding: Recover Sustainable Capability"
            },
            objective: if locale.is_zh() {
                "优先恢复被阻塞的产线或能力链，而不是重复单次动作。".to_string()
            } else {
                "Recover the blocked line or capability chain instead of repeating one-off actions."
                    .to_string()
            },
            progress_detail: if locale.is_zh() {
                "阶段进展：你已经进入经营阶段，但当前主线被阻塞。".to_string()
            } else {
                "Stage progress: you are in the management phase, but the primary line is blocked."
                    .to_string()
            },
            progress_percent: 68,
            blocker_detail: Some(post_onboarding_blocker_detail(
                reason.as_str(),
                detail.as_str(),
                locale,
            )),
            next_step: post_onboarding_blocker_next_step(reason.as_str(), detail.as_str(), locale),
            branch_hint: None,
            action_label: if locale.is_zh() {
                "进入指挥并推进 1 步"
            } else {
                "Open command and advance 1 step"
            },
        };
    }

    if has_recipe_running {
        PlayerPostOnboardingSnapshot {
            status: PlayerPostOnboardingStatus::Active,
            title: if locale.is_zh() {
                "PostOnboarding：稳定第一条产线"
            } else {
                "PostOnboarding: Stabilize Your First Line"
            },
            objective: if locale.is_zh() {
                "让第一条生产线连续推进，直到出现稳定产出或明确阻塞原因。".to_string()
            } else {
                "Keep your first production line moving until it produces stable output or exposes a clear blocker."
                    .to_string()
            },
            progress_detail: if locale.is_zh() {
                "阶段进展：首条产线已启动，接下来重点看输出与停机原因。".to_string()
            } else {
                "Stage progress: the first line is running; now watch for output and stoppage reasons."
                    .to_string()
            },
            progress_percent: 72,
            blocker_detail: None,
            next_step: if locale.is_zh() {
                "下一步：保持 Command 视图，再推进 1~2 次，并观察是否出现产出、恢复或阻塞反馈。"
                    .to_string()
            } else {
                "Next: stay in Command view, advance 1-2 more times, and watch for output, recovery, or blocker feedback."
                    .to_string()
            },
            branch_hint: None,
            action_label: if locale.is_zh() {
                "进入指挥并推进 1 步"
            } else {
                "Open command and advance 1 step"
            },
        }
    } else if has_factory_ready {
        PlayerPostOnboardingSnapshot {
            status: PlayerPostOnboardingStatus::Active,
            title: if locale.is_zh() {
                "PostOnboarding：启动第一座工厂"
            } else {
                "PostOnboarding: Start Your First Factory Run"
            },
            objective: if locale.is_zh() {
                "把已建成的工厂推进成真正运转的持续能力。".to_string()
            } else {
                "Turn the factory you built into a running, repeatable capability."
                    .to_string()
            },
            progress_detail: if locale.is_zh() {
                "阶段进展：工厂已就绪，还差一次可见的生产推进。".to_string()
            } else {
                "Stage progress: the factory is ready; one visible production push remains."
                    .to_string()
            },
            progress_percent: 54,
            blocker_detail: None,
            next_step: if locale.is_zh() {
                "下一步：切到 Command 视图并继续推进，直到工厂启动配方、产出结果或返回阻塞原因。"
                    .to_string()
            } else {
                "Next: switch to Command view and keep advancing until the factory starts a recipe, yields output, or returns a blocker."
                    .to_string()
            },
            branch_hint: None,
            action_label: if locale.is_zh() {
                "进入指挥并推进 1 步"
            } else {
                "Open command and advance 1 step"
            },
        }
    } else if has_material_flow {
        PlayerPostOnboardingSnapshot {
            status: PlayerPostOnboardingStatus::Active,
            title: if locale.is_zh() {
                "PostOnboarding：把资源流变成产出"
            } else {
                "PostOnboarding: Turn Material Flow Into Output"
            },
            objective: if locale.is_zh() {
                "不要停留在一次性采集，继续把资源推进到可见产出。".to_string()
            } else {
                "Do not stop at one-off harvesting; push the resource flow into visible output."
                    .to_string()
            },
            progress_detail: if locale.is_zh() {
                "阶段进展：基础资源已经动起来，接下来要形成第一项持续能力。".to_string()
            } else {
                "Stage progress: base resources are moving; now convert them into the first sustainable capability."
                    .to_string()
            },
            progress_percent: 38,
            blocker_detail: None,
            next_step: if locale.is_zh() {
                "下一步：继续在 Command 视图推进采集、精炼、建厂或首个配方，直到出现稳定产出。"
                    .to_string()
            } else {
                "Next: keep using Command view to harvest, refine, build, or start the first recipe until stable output appears."
                    .to_string()
            },
            branch_hint: None,
            action_label: if locale.is_zh() {
                "进入指挥并推进 1 步"
            } else {
                "Open command and advance 1 step"
            },
        }
    } else {
        PlayerPostOnboardingSnapshot {
            status: PlayerPostOnboardingStatus::Active,
            title: if locale.is_zh() {
                "PostOnboarding：建立第一项持续能力"
            } else {
                "PostOnboarding: Establish Your First Sustainable Capability"
            },
            objective: if locale.is_zh() {
                "首局行动闭环已完成，下一步不是重复教程，而是做出第一项持续工业成果。".to_string()
            } else {
                "The first-session action loop is complete. The next step is not to repeat the tutorial, but to create your first sustainable industrial result."
                    .to_string()
            },
            progress_detail: if locale.is_zh() {
                "阶段进展：你已从“会操作”进入“会经营”的起点。".to_string()
            } else {
                "Stage progress: you have moved from 'can operate' into the start of 'can manage'."
                    .to_string()
            },
            progress_percent: 20,
            blocker_detail: None,
            next_step: if locale.is_zh() {
                "下一步：保持 Command 视图，再推进 2~3 次，优先追首个工业产出、首条稳定产线或一次明确的恢复反馈。"
                    .to_string()
            } else {
                "Next: stay in Command view and advance 2-3 more times, prioritizing the first industrial output, the first stable line, or one clear recovery signal."
                    .to_string()
            },
            branch_hint: None,
            action_label: if locale.is_zh() {
                "进入指挥并推进 1 步"
            } else {
                "Open command and advance 1 step"
            },
        }
    }
}

fn build_player_post_onboarding_snapshot_from_gameplay(
    gameplay: &PlayerGameplaySnapshot,
    locale: crate::i18n::UiLocale,
) -> PlayerPostOnboardingSnapshot {
    let status = match gameplay.stage_status {
        PlayerGameplayStageStatus::Active => PlayerPostOnboardingStatus::Active,
        PlayerGameplayStageStatus::Blocked => PlayerPostOnboardingStatus::Blocked,
        PlayerGameplayStageStatus::BranchReady => PlayerPostOnboardingStatus::BranchReady,
    };
    let blocker_reason = gameplay
        .blocker_kind
        .as_deref()
        .or(gameplay.blocker_detail.as_deref())
        .unwrap_or("unknown");
    let blocker_detail = matches!(status, PlayerPostOnboardingStatus::Blocked).then(|| {
        post_onboarding_blocker_detail(
            blocker_reason,
            gameplay.blocker_detail.as_deref().unwrap_or_default(),
            locale,
        )
    });
    let next_step = if matches!(status, PlayerPostOnboardingStatus::Blocked) {
        post_onboarding_blocker_next_step(
            blocker_reason,
            gameplay.blocker_detail.as_deref().unwrap_or_default(),
            locale,
        )
    } else if locale.is_zh() {
        localized_post_onboarding_next_step_for_goal(gameplay.goal_kind, locale)
    } else {
        gameplay.next_step_hint.clone()
    };
    let branch_hint = if locale.is_zh() {
        gameplay
            .branch_hint
            .as_ref()
            .map(|_| "已解锁分支：生产扩张 / 治理影响 / 冲突安全".to_string())
    } else {
        gameplay.branch_hint.clone()
    };

    PlayerPostOnboardingSnapshot {
        status,
        title: localized_post_onboarding_title_for_goal(gameplay.goal_kind, status, locale),
        objective: if locale.is_zh() {
            localized_post_onboarding_objective_for_goal(gameplay.goal_kind, status, locale)
        } else {
            gameplay.objective.clone()
        },
        progress_detail: if locale.is_zh() {
            localized_post_onboarding_progress_detail_for_goal(
                gameplay.goal_kind,
                status,
                locale,
            )
        } else {
            gameplay.progress_detail.clone()
        },
        progress_percent: gameplay.progress_percent,
        blocker_detail,
        next_step,
        branch_hint,
        action_label: if locale.is_zh() {
            "进入指挥并推进 1 步"
        } else {
            "Open command and advance 1 step"
        },
    }
}

fn player_goal_action_sentence(
    step: PlayerGuideStep,
    locale: crate::i18n::UiLocale,
) -> &'static str {
    match (step, locale.is_zh()) {
        (PlayerGuideStep::ConnectWorld, true) => "等待连接状态变为“已连接”",
        (PlayerGuideStep::ConnectWorld, false) => "Wait until connection status becomes Connected",
        (PlayerGuideStep::OpenPanel, true) => "打开右侧操作面板，进入可操作状态",
        (PlayerGuideStep::OpenPanel, false) => "Open the right control panel to unlock actions",
        (PlayerGuideStep::SelectTarget, true) => "在场景中选择 1 个 Agent 或地点",
        (PlayerGuideStep::SelectTarget, false) => "Select one agent or location in the scene",
        (PlayerGuideStep::ExploreAction, true) => "发送 1 次指令并确认世界出现新反馈",
        (PlayerGuideStep::ExploreAction, false) => {
            "Send one command and confirm new world feedback"
        }
    }
}

fn post_onboarding_summary_value<'a>(summary: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("{key}=");
    let start = summary.find(needle.as_str())?;
    let value_start = start + needle.len();
    let rest = &summary[value_start..];
    let value_end = rest.find(' ').unwrap_or(rest.len());
    let value = rest[..value_end].trim();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn localized_post_onboarding_title_for_goal(
    goal_kind: PlayerGameplayGoalKind,
    status: PlayerPostOnboardingStatus,
    locale: crate::i18n::UiLocale,
) -> &'static str {
    match (goal_kind, status, locale.is_zh()) {
        (PlayerGameplayGoalKind::ChooseMidLoopPath, _, true) => "下一阶段：选择中循环方向",
        (PlayerGameplayGoalKind::ChooseMidLoopPath, _, false) => {
            "Next Stage: Choose Your Mid-loop Path"
        }
        (PlayerGameplayGoalKind::RecoverCapability, _, true) => "PostOnboarding：恢复持续能力",
        (PlayerGameplayGoalKind::RecoverCapability, _, false) => {
            "PostOnboarding: Recover Sustainable Capability"
        }
        (PlayerGameplayGoalKind::StabilizeFirstLine, _, true) => "PostOnboarding：稳定第一条产线",
        (PlayerGameplayGoalKind::StabilizeFirstLine, _, false) => {
            "PostOnboarding: Stabilize Your First Line"
        }
        (PlayerGameplayGoalKind::StartFactoryRun, _, true) => "PostOnboarding：启动第一座工厂",
        (PlayerGameplayGoalKind::StartFactoryRun, _, false) => {
            "PostOnboarding: Start Your First Factory Run"
        }
        (PlayerGameplayGoalKind::TurnMaterialFlowIntoOutput, _, true) => {
            "PostOnboarding：把资源流变成产出"
        }
        (PlayerGameplayGoalKind::TurnMaterialFlowIntoOutput, _, false) => {
            "PostOnboarding: Turn Material Flow Into Output"
        }
        (PlayerGameplayGoalKind::EstablishFirstCapability, _, true) => {
            "PostOnboarding：建立第一项持续能力"
        }
        (PlayerGameplayGoalKind::EstablishFirstCapability, _, false) => {
            "PostOnboarding: Establish Your First Sustainable Capability"
        }
        (_, PlayerPostOnboardingStatus::BranchReady, true) => "下一阶段：选择中循环方向",
        (_, PlayerPostOnboardingStatus::BranchReady, false) => {
            "Next Stage: Choose Your Mid-loop Path"
        }
        (_, PlayerPostOnboardingStatus::Blocked, true) => "PostOnboarding：恢复持续能力",
        (_, PlayerPostOnboardingStatus::Blocked, false) => {
            "PostOnboarding: Recover Sustainable Capability"
        }
        (_, _, true) => "PostOnboarding：建立第一项持续能力",
        (_, _, false) => "PostOnboarding: Establish Your First Sustainable Capability",
    }
}

fn localized_post_onboarding_objective_for_goal(
    goal_kind: PlayerGameplayGoalKind,
    status: PlayerPostOnboardingStatus,
    locale: crate::i18n::UiLocale,
) -> String {
    match (goal_kind, status, locale.is_zh()) {
        (PlayerGameplayGoalKind::ChooseMidLoopPath, _, true) => {
            "第一项持续工业能力已建立，开始把它扩张成稳定组织能力。".to_string()
        }
        (PlayerGameplayGoalKind::ChooseMidLoopPath, _, false) => {
            "Your first sustainable industrial capability is online. Turn it into stable organizational momentum.".to_string()
        }
        (PlayerGameplayGoalKind::RecoverCapability, _, true) => {
            "优先恢复被阻塞的产线或能力链，而不是重复单次动作。".to_string()
        }
        (PlayerGameplayGoalKind::RecoverCapability, _, false) => {
            "Recover the blocked line or capability chain instead of repeating one-off actions."
                .to_string()
        }
        (PlayerGameplayGoalKind::StabilizeFirstLine, _, true) => {
            "让第一条生产线连续推进，直到出现稳定产出或明确阻塞原因。".to_string()
        }
        (PlayerGameplayGoalKind::StabilizeFirstLine, _, false) => {
            "Keep your first production line moving until it produces stable output or exposes a clear blocker."
                .to_string()
        }
        (PlayerGameplayGoalKind::StartFactoryRun, _, true) => {
            "把已建成的工厂推进成真正运转的持续能力。".to_string()
        }
        (PlayerGameplayGoalKind::StartFactoryRun, _, false) => {
            "Turn the factory you built into a running, repeatable capability.".to_string()
        }
        (PlayerGameplayGoalKind::TurnMaterialFlowIntoOutput, _, true) => {
            "不要停留在一次性采集，继续把资源推进到可见产出。".to_string()
        }
        (PlayerGameplayGoalKind::TurnMaterialFlowIntoOutput, _, false) => {
            "Do not stop at one-off harvesting; push the resource flow into visible output."
                .to_string()
        }
        (_, _, true) => {
            "首局行动闭环已完成，下一步不是重复教程，而是做出第一项持续工业成果。"
                .to_string()
        }
        (_, _, false) => {
            "The first-session action loop is complete. The next step is not to repeat the tutorial, but to create your first sustainable industrial result."
                .to_string()
        }
    }
}

fn localized_post_onboarding_progress_detail_for_goal(
    goal_kind: PlayerGameplayGoalKind,
    status: PlayerPostOnboardingStatus,
    locale: crate::i18n::UiLocale,
) -> String {
    match (goal_kind, status, locale.is_zh()) {
        (PlayerGameplayGoalKind::ChooseMidLoopPath, _, true) => {
            "阶段进展：已完成首个可见产出/稳定产线里程碑。".to_string()
        }
        (PlayerGameplayGoalKind::ChooseMidLoopPath, _, false) => {
            "Stage progress: your first visible output or stable line milestone is complete."
                .to_string()
        }
        (PlayerGameplayGoalKind::RecoverCapability, _, true) => {
            "阶段进展：你已经进入经营阶段，但当前主线被阻塞。".to_string()
        }
        (PlayerGameplayGoalKind::RecoverCapability, _, false) => {
            "Stage progress: you are in the management phase, but the primary line is blocked."
                .to_string()
        }
        (PlayerGameplayGoalKind::StabilizeFirstLine, _, true) => {
            "阶段进展：首条产线已启动，接下来重点看输出与停机原因。".to_string()
        }
        (PlayerGameplayGoalKind::StabilizeFirstLine, _, false) => {
            "Stage progress: the first line is running; now watch for output and stoppage reasons."
                .to_string()
        }
        (PlayerGameplayGoalKind::StartFactoryRun, _, true) => {
            "阶段进展：工厂已就绪，还差一次可见的生产推进。".to_string()
        }
        (PlayerGameplayGoalKind::StartFactoryRun, _, false) => {
            "Stage progress: the factory is ready; one visible production push remains."
                .to_string()
        }
        (PlayerGameplayGoalKind::TurnMaterialFlowIntoOutput, _, true) => {
            "阶段进展：基础资源已经动起来，接下来要形成第一项持续能力。".to_string()
        }
        (PlayerGameplayGoalKind::TurnMaterialFlowIntoOutput, _, false) => {
            "Stage progress: base resources are moving; now convert them into the first sustainable capability."
                .to_string()
        }
        (_, PlayerPostOnboardingStatus::Blocked, true) => {
            "阶段进展：你已经进入经营阶段，但当前主线被阻塞。".to_string()
        }
        (_, PlayerPostOnboardingStatus::Blocked, false) => {
            "Stage progress: you are in the management phase, but the primary line is blocked."
                .to_string()
        }
        (_, _, true) => "阶段进展：你已从“会操作”进入“会经营”的起点。".to_string(),
        (_, _, false) => {
            "Stage progress: you have moved from 'can operate' into the start of 'can manage'."
                .to_string()
        }
    }
}

fn localized_post_onboarding_next_step_for_goal(
    goal_kind: PlayerGameplayGoalKind,
    locale: crate::i18n::UiLocale,
) -> String {
    match (goal_kind, locale.is_zh()) {
        (PlayerGameplayGoalKind::ChooseMidLoopPath, true) => {
            "下一步：保持 Command 视图，继续扩产、推进治理提案，或为关键节点补防护。"
                .to_string()
        }
        (PlayerGameplayGoalKind::ChooseMidLoopPath, false) => {
            "Next: stay in Command view and either expand production, push governance, or secure a critical node."
                .to_string()
        }
        (PlayerGameplayGoalKind::StabilizeFirstLine, true) => {
            "下一步：保持 Command 视图，再推进 1~2 次，并观察是否出现产出、恢复或阻塞反馈。"
                .to_string()
        }
        (PlayerGameplayGoalKind::StabilizeFirstLine, false) => {
            "Next: stay in Command view, advance 1-2 more times, and watch for output, recovery, or blocker feedback."
                .to_string()
        }
        (PlayerGameplayGoalKind::StartFactoryRun, true) => {
            "下一步：切到 Command 视图并继续推进，直到工厂启动配方、产出结果或返回阻塞原因。"
                .to_string()
        }
        (PlayerGameplayGoalKind::StartFactoryRun, false) => {
            "Next: switch to Command view and keep advancing until the factory starts a recipe, yields output, or returns a blocker."
                .to_string()
        }
        (PlayerGameplayGoalKind::TurnMaterialFlowIntoOutput, true) => {
            "下一步：继续在 Command 视图推进采集、精炼、建厂或首个配方，直到出现稳定产出。"
                .to_string()
        }
        (PlayerGameplayGoalKind::TurnMaterialFlowIntoOutput, false) => {
            "Next: keep using Command view to harvest, refine, build, or start the first recipe until stable output appears."
                .to_string()
        }
        (_, true) => {
            "下一步：保持 Command 视图，再推进 2~3 次，优先追首个工业产出、首条稳定产线或一次明确的恢复反馈。"
                .to_string()
        }
        (_, false) => {
            "Next: stay in Command view and advance 2-3 more times, prioritizing the first industrial output, the first stable line, or one clear recovery signal."
                .to_string()
        }
    }
}

fn post_onboarding_blocker_detail(
    reason: &str,
    detail: &str,
    locale: crate::i18n::UiLocale,
) -> String {
    let normalized = format!("{reason} {detail}");
    if normalized.contains("material_shortage") || normalized.contains("missing_input") {
        if locale.is_zh() {
            "主阻塞：缺料，当前产线拿不到继续推进所需的输入。".to_string()
        } else {
            "Primary blocker: missing materials. The current line cannot get the inputs it needs."
                .to_string()
        }
    } else if normalized.contains("electricity")
        || normalized.contains("power")
        || normalized.contains("energy")
    {
        if locale.is_zh() {
            "主阻塞：缺电/能源不足，当前能力链无法持续运转。".to_string()
        } else {
            "Primary blocker: insufficient power or energy. The capability chain cannot keep running."
                .to_string()
        }
    } else if normalized.contains("logistics") {
        if locale.is_zh() {
            "主阻塞：物流阻塞，资源没能按节奏流到目标节点。".to_string()
        } else {
            "Primary blocker: logistics jam. Resources are not reaching the target node in time."
                .to_string()
        }
    } else if normalized.contains("governance") {
        if locale.is_zh() {
            "主阻塞：治理限制，当前行为被制度或权限约束挡住。".to_string()
        } else {
            "Primary blocker: governance restriction. Rules or permissions are blocking progress."
                .to_string()
        }
    } else if normalized.contains("war") || normalized.contains("crisis") {
        if locale.is_zh() {
            "主阻塞：危机/冲突压力，当前应先保全与恢复。".to_string()
        } else {
            "Primary blocker: crisis or conflict pressure. Stabilization must come before expansion."
                .to_string()
        }
    } else if locale.is_zh() {
        format!("主阻塞：{reason}")
    } else {
        format!("Primary blocker: {reason}")
    }
}

fn post_onboarding_blocker_next_step(
    reason: &str,
    detail: &str,
    locale: crate::i18n::UiLocale,
) -> String {
    let normalized = format!("{reason} {detail}");
    if normalized.contains("material_shortage") || normalized.contains("missing_input") {
        if locale.is_zh() {
            "建议下一步：补齐上游原料或继续推进采集/精炼，再观察产线是否恢复。".to_string()
        } else {
            "Next: replenish upstream materials or keep harvesting/refining, then check whether the line resumes."
                .to_string()
        }
    } else if normalized.contains("electricity")
        || normalized.contains("power")
        || normalized.contains("energy")
    {
        if locale.is_zh() {
            "建议下一步：先补能源，再继续推进工厂或配方。".to_string()
        } else {
            "Next: restore energy first, then continue advancing the factory or recipe."
                .to_string()
        }
    } else if normalized.contains("logistics") {
        if locale.is_zh() {
            "建议下一步：重新推进运输/位置相关操作，先打通物流路径。".to_string()
        } else {
            "Next: advance movement or transport-related actions and reopen the logistics path."
                .to_string()
        }
    } else if normalized.contains("governance") {
        if locale.is_zh() {
            "建议下一步：切换到治理/规则相关面板，确认限制来源后再继续推进。".to_string()
        } else {
            "Next: inspect governance or rules-related panels, identify the restriction, and then continue."
                .to_string()
        }
    } else if normalized.contains("war") || normalized.contains("crisis") {
        if locale.is_zh() {
            "建议下一步：优先保全节点、处理危机，再回到扩张主线。".to_string()
        } else {
            "Next: secure the node and handle the crisis first, then return to expansion."
                .to_string()
        }
    } else if locale.is_zh() {
        "建议下一步：继续在 Command 视图推进 1 步，并观察新的阻塞或恢复反馈。".to_string()
    } else {
        "Next: advance one more step in Command view and watch for new blocker or recovery feedback."
            .to_string()
    }
}

pub(crate) fn player_post_onboarding_status_color(
    status: PlayerPostOnboardingStatus,
) -> egui::Color32 {
    match status {
        PlayerPostOnboardingStatus::Active => egui::Color32::from_rgb(86, 144, 214),
        PlayerPostOnboardingStatus::Blocked => egui::Color32::from_rgb(224, 148, 92),
        PlayerPostOnboardingStatus::BranchReady => egui::Color32::from_rgb(74, 176, 108),
    }
}

pub(crate) fn player_post_onboarding_status_label(
    status: PlayerPostOnboardingStatus,
    locale: crate::i18n::UiLocale,
) -> &'static str {
    match (status, locale.is_zh()) {
        (PlayerPostOnboardingStatus::Active, true) => "阶段推进中",
        (PlayerPostOnboardingStatus::Active, false) => "Stage Active",
        (PlayerPostOnboardingStatus::Blocked, true) => "阶段受阻",
        (PlayerPostOnboardingStatus::Blocked, false) => "Stage Blocked",
        (PlayerPostOnboardingStatus::BranchReady, true) => "分支已解锁",
        (PlayerPostOnboardingStatus::BranchReady, false) => "Branch Ready",
    }
}

fn player_goal_completion_condition(
    step: PlayerGuideStep,
    locale: crate::i18n::UiLocale,
) -> &'static str {
    match (step, locale.is_zh()) {
        (PlayerGuideStep::ConnectWorld, true) => "完成条件：状态栏显示“已连接”",
        (PlayerGuideStep::ConnectWorld, false) => "Completion: connection chip shows Connected",
        (PlayerGuideStep::OpenPanel, true) => "完成条件：右侧面板可见",
        (PlayerGuideStep::OpenPanel, false) => "Completion: right panel is visible",
        (PlayerGuideStep::SelectTarget, true) => "完成条件：目标栏出现选中对象",
        (PlayerGuideStep::SelectTarget, false) => "Completion: target chip shows a selected object",
        (PlayerGuideStep::ExploreAction, true) => "完成条件：你的操作后新增至少 1 条世界反馈",
        (PlayerGuideStep::ExploreAction, false) => {
            "Completion: at least one new world feedback appears"
        }
    }
}

fn player_goal_eta(step: PlayerGuideStep, locale: crate::i18n::UiLocale) -> &'static str {
    match (step, locale.is_zh()) {
        (PlayerGuideStep::ConnectWorld, true) => "预计耗时：约 10 秒",
        (PlayerGuideStep::ConnectWorld, false) => "ETA: about 10s",
        (PlayerGuideStep::OpenPanel, true) => "预计耗时：约 5 秒",
        (PlayerGuideStep::OpenPanel, false) => "ETA: about 5s",
        (PlayerGuideStep::SelectTarget, true) => "预计耗时：约 10 秒",
        (PlayerGuideStep::SelectTarget, false) => "ETA: about 10s",
        (PlayerGuideStep::ExploreAction, true) => "预计耗时：约 20 秒",
        (PlayerGuideStep::ExploreAction, false) => "ETA: about 20s",
    }
}

pub(super) fn build_player_mission_remaining_hint(
    step: PlayerGuideStep,
    progress: PlayerGuideProgressSnapshot,
    state: &crate::ViewerState,
    locale: crate::i18n::UiLocale,
) -> String {
    let current_tick = player_current_tick(state);
    match (step, locale.is_zh()) {
        (PlayerGuideStep::ConnectWorld, true) => {
            if progress.connect_world_done {
                "剩余：已完成连接，可进入下一步".to_string()
            } else {
                "剩余：等待状态栏出现“已连接”".to_string()
            }
        }
        (PlayerGuideStep::ConnectWorld, false) => {
            if progress.connect_world_done {
                "Remaining: connection done, proceed to next step".to_string()
            } else {
                "Remaining: wait until the status chip shows Connected".to_string()
            }
        }
        (PlayerGuideStep::OpenPanel, true) => {
            if progress.open_panel_done {
                "剩余：面板已展开，继续锁定目标".to_string()
            } else {
                "剩余：展开右侧面板".to_string()
            }
        }
        (PlayerGuideStep::OpenPanel, false) => {
            if progress.open_panel_done {
                "Remaining: panel opened, proceed to target selection".to_string()
            } else {
                "Remaining: open the right panel".to_string()
            }
        }
        (PlayerGuideStep::SelectTarget, true) => {
            if progress.select_target_done {
                "剩余：目标已锁定，继续发出首条指令".to_string()
            } else {
                "剩余：在场景里选中 1 个 Agent 或地点".to_string()
            }
        }
        (PlayerGuideStep::SelectTarget, false) => {
            if progress.select_target_done {
                "Remaining: target locked, send your first command".to_string()
            } else {
                "Remaining: select one agent or location in the scene".to_string()
            }
        }
        (PlayerGuideStep::ExploreAction, true) => {
            let remaining_tick = 20_u64.saturating_sub(current_tick);
            if !progress.explore_ready {
                "剩余：发送指令后至少出现 1 条新的世界反馈".to_string()
            } else if remaining_tick > 0 {
                format!("剩余：再推进约 {remaining_tick} tick（目标 tick=20）")
            } else {
                "剩余：首局主循环目标已达成".to_string()
            }
        }
        (PlayerGuideStep::ExploreAction, false) => {
            let remaining_tick = 20_u64.saturating_sub(current_tick);
            if !progress.explore_ready {
                "Remaining: trigger at least one new world feedback after your command".to_string()
            } else if remaining_tick > 0 {
                format!("Remaining: advance about {remaining_tick} more ticks (goal tick=20)")
            } else {
                "Remaining: first-session loop target reached".to_string()
            }
        }
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

fn player_micro_loop_tone_color(tone: PlayerMicroLoopTone) -> egui::Color32 {
    match tone {
        PlayerMicroLoopTone::Positive => egui::Color32::from_rgb(92, 188, 126),
        PlayerMicroLoopTone::Warning => egui::Color32::from_rgb(230, 148, 96),
        PlayerMicroLoopTone::Info => egui::Color32::from_rgb(116, 174, 236),
    }
}

fn render_player_micro_loop_summary(
    ui: &mut egui::Ui,
    snapshot: &PlayerMicroLoopSnapshot,
    locale: crate::i18n::UiLocale,
) {
    let tone = player_micro_loop_tone_color(snapshot.action_status.tone);
    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgba_unmultiplied(26, 34, 48, 144))
        .stroke(egui::Stroke::new(1.0, tone))
        .corner_radius(egui::CornerRadius::same(7))
        .inner_margin(egui::Margin::same(7))
        .show(ui, |ui| {
            ui.small(if locale.is_zh() {
                "微循环反馈"
            } else {
                "Micro-loop Feedback"
            });
            ui.small(egui::RichText::new(snapshot.action_status.headline.as_str()).color(tone));
            ui.small(snapshot.action_status.detail.as_str());
            if let Some(pending_eta_ticks) = snapshot.action_status.pending_eta_ticks {
                ui.small(if locale.is_zh() {
                    format!("动作 ETA: 约 {} tick", pending_eta_ticks)
                } else {
                    format!("Action ETA: about {} ticks", pending_eta_ticks)
                });
            }
            if snapshot.due_timers.is_empty() {
                ui.small(if locale.is_zh() {
                    "关键计时器：暂无激活项"
                } else {
                    "Key timers: none active"
                });
            } else {
                ui.small(if locale.is_zh() {
                    "关键计时器（战争/治理/危机/合约）"
                } else {
                    "Key timers (war/governance/crisis/contract)"
                });
                for timer in snapshot.due_timers.iter().take(4) {
                    ui.small(
                        egui::RichText::new(format_due_timer_line(timer, locale)).color(
                            if timer.overdue_ticks > 0 {
                                egui::Color32::from_rgb(238, 168, 108)
                            } else {
                                egui::Color32::from_rgb(186, 206, 238)
                            },
                        ),
                    );
                }
            }
        });
}

fn render_player_control_result_strip(
    ui: &mut egui::Ui,
    feedback: &WebTestApiControlFeedbackSnapshot,
    locale: crate::i18n::UiLocale,
    pulse: f32,
) {
    let stage_color = player_control_stage_color(feedback.stage.as_str());
    let stroke_alpha = (172.0 + 58.0 * pulse).round() as u8;
    let fill_alpha = if feedback.stage == "blocked" { 62 } else { 44 };
    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgba_unmultiplied(
            stage_color.r(),
            stage_color.g(),
            stage_color.b(),
            fill_alpha,
        ))
        .stroke(egui::Stroke::new(
            1.1,
            egui::Color32::from_rgba_unmultiplied(
                stage_color.r(),
                stage_color.g(),
                stage_color.b(),
                stroke_alpha,
            ),
        ))
        .corner_radius(egui::CornerRadius::same(7))
        .inner_margin(egui::Margin::same(7))
        .show(ui, |ui| {
            ui.small(if locale.is_zh() {
                "控制结果"
            } else {
                "Control Result"
            });
            ui.small(
                egui::RichText::new(format!(
                    "{} · {}",
                    feedback.action,
                    player_control_stage_label(feedback.stage.as_str(), locale)
                ))
                .color(stage_color)
                .strong(),
            );
            ui.small(if locale.is_zh() {
                format!(
                    "增量: tick +{} · event +{} · trace +{}",
                    feedback.delta_logical_time,
                    feedback.delta_event_seq,
                    feedback.delta_trace_count
                )
            } else {
                format!(
                    "Delta: tick +{} · event +{} · trace +{}",
                    feedback.delta_logical_time,
                    feedback.delta_event_seq,
                    feedback.delta_trace_count
                )
            });
            ui.small(feedback.effect.as_str());
        });
}

pub(super) fn render_player_mission_hud(
    context: &egui::Context,
    state: &crate::ViewerState,
    selection: &ViewerSelection,
    client: Option<&crate::ViewerClient>,
    control_feedback: Option<&WebTestApiControlFeedbackSnapshot>,
    control_profile: Option<&crate::ViewerControlProfileState>,
    layout_state: &mut RightPanelLayoutState,
    module_visibility: &mut crate::right_panel_module_visibility::RightPanelModuleVisibilityState,
    onboarding_visible: bool,
    step: PlayerGuideStep,
    progress: PlayerGuideProgressSnapshot,
    stuck_hint: Option<&str>,
    stuck_diagnosis: Option<&PlayerNoProgressDiagnosis>,
    locale: crate::i18n::UiLocale,
    now_secs: f64,
) {
    let snapshot = build_player_mission_loop_snapshot(step, progress, locale);
    let remaining_hint = build_player_mission_remaining_hint(step, progress, state, locale);
    let reward = build_player_reward_feedback_snapshot(progress, locale);
    let post_onboarding = progress
        .explore_ready
        .then(|| build_player_post_onboarding_snapshot(state, control_feedback, locale));
    let tone = post_onboarding
        .as_ref()
        .map(|snapshot| player_post_onboarding_status_color(snapshot.status))
        .unwrap_or_else(|| player_goal_color(step));
    let reward_tone = if let Some(post_onboarding) = post_onboarding.as_ref() {
        player_post_onboarding_status_color(post_onboarding.status)
    } else if reward.complete {
        egui::Color32::from_rgb(54, 166, 96)
    } else {
        egui::Color32::from_rgb(74, 126, 184)
    };
    let compact_mode = player_mission_hud_compact_mode(layout_state.panel_hidden);
    let mission_anchor_y = player_mission_hud_anchor_y(
        layout_state.panel_hidden,
        onboarding_visible,
        stuck_hint.is_some(),
    );
    let pulse = ((now_secs * 1.8).sin() * 0.5 + 0.5) as f32;
    let mut action_clicked = false;
    let mut command_clicked = false;
    let (mut recover_play_clicked, mut recover_step_clicked) = (false, false);
    let micro_loop_snapshot = build_player_micro_loop_snapshot(state, locale);
    let control_feedback_needs_recovery = control_feedback.as_ref().is_some_and(|feedback| {
        player_control_stage_shows_recovery_actions(feedback.stage.as_str())
    });
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
                    if let Some(post_onboarding) = post_onboarding.as_ref() {
                        ui.small(
                            egui::RichText::new(player_post_onboarding_status_label(
                                post_onboarding.status,
                                locale,
                            ))
                            .color(tone)
                            .strong(),
                        );
                        ui.small(if locale.is_zh() {
                            "阶段目标"
                        } else {
                            "Stage Goal"
                        });
                        ui.strong(post_onboarding.title);
                        ui.label(post_onboarding.objective.as_str());
                        ui.small(
                            egui::RichText::new(post_onboarding.progress_detail.as_str())
                                .color(egui::Color32::from_rgb(186, 206, 238)),
                        );
                    } else {
                        ui.small(egui::RichText::new(snapshot.title).color(tone).strong());
                        ui.small(if locale.is_zh() {
                            "主目标"
                        } else {
                            "Main Goal"
                        });
                        ui.strong(snapshot.objective);
                        ui.small(snapshot.completion_condition);
                        ui.small(snapshot.eta);
                        ui.small(
                            egui::RichText::new(remaining_hint.as_str())
                                .color(egui::Color32::from_rgb(186, 206, 238)),
                        );
                    }
                    if let Some(feedback) = control_feedback.as_ref() {
                        render_player_control_result_strip(ui, feedback, locale, pulse);
                    }
                    render_player_micro_loop_summary(ui, &micro_loop_snapshot, locale);
                    if let Some(post_onboarding) = post_onboarding.as_ref() {
                        if let Some(blocker_detail) = post_onboarding.blocker_detail.as_ref() {
                            egui::Frame::group(ui.style())
                                .fill(egui::Color32::from_rgba_unmultiplied(92, 48, 28, 132))
                                .stroke(egui::Stroke::new(1.0, reward_tone))
                                .corner_radius(egui::CornerRadius::same(6))
                                .inner_margin(egui::Margin::same(6))
                                .show(ui, |ui| {
                                    ui.small(if locale.is_zh() {
                                        "当前阻塞"
                                    } else {
                                        "Current Blocker"
                                    });
                                    ui.small(
                                        egui::RichText::new(blocker_detail.as_str())
                                            .color(egui::Color32::from_rgb(248, 214, 186)),
                                    );
                                });
                        }
                        ui.small(post_onboarding.next_step.as_str());
                        if let Some(branch_hint) = post_onboarding.branch_hint.as_ref() {
                            egui::Frame::group(ui.style())
                                .fill(egui::Color32::from_rgba_unmultiplied(
                                    reward_tone.r(),
                                    reward_tone.g(),
                                    reward_tone.b(),
                                    28,
                                ))
                                .stroke(egui::Stroke::new(1.0, reward_tone))
                                .corner_radius(egui::CornerRadius::same(8))
                                .inner_margin(egui::Margin::same(8))
                                .show(ui, |ui| {
                                    ui.small(if locale.is_zh() {
                                        "下一批方向"
                                    } else {
                                        "Next Branches"
                                    });
                                    ui.strong(branch_hint.as_str());
                                });
                        }
                    } else {
                        egui::CollapsingHeader::new(if locale.is_zh() {
                            "展开短目标"
                        } else {
                            "Expand short goals"
                        })
                        .default_open(false)
                        .show(ui, |ui| {
                            for goal in snapshot.short_goals {
                                let marker = if goal.complete { "✓" } else { "□" };
                                let color = if goal.complete {
                                    tone
                                } else {
                                    egui::Color32::from_gray(182)
                                };
                                ui.small(
                                    egui::RichText::new(format!("{marker} {}", goal.label))
                                        .color(color),
                                );
                            }
                        });
                        if !compact_mode {
                            ui.small(player_goal_detail(step, locale));
                        }
                    }
                    if let Some(stuck_hint) = stuck_hint {
                        egui::Frame::group(ui.style())
                            .fill(egui::Color32::from_rgba_unmultiplied(84, 42, 28, 132))
                            .stroke(egui::Stroke::new(
                                1.0,
                                egui::Color32::from_rgb(224, 146, 92),
                            ))
                            .corner_radius(egui::CornerRadius::same(6))
                            .inner_margin(egui::Margin::same(6))
                            .show(ui, |ui| {
                                ui.small(
                                    egui::RichText::new(stuck_hint)
                                        .color(egui::Color32::from_rgb(248, 210, 180)),
                                );
                                if let Some(diagnosis) = stuck_diagnosis {
                                    ui.small(
                                        egui::RichText::new(if locale.is_zh() {
                                            format!("原因：{}", diagnosis.reason)
                                        } else {
                                            format!("Cause: {}", diagnosis.reason)
                                        })
                                        .color(egui::Color32::from_rgb(244, 188, 152)),
                                    );
                                    ui.small(
                                        egui::RichText::new(if locale.is_zh() {
                                            format!("建议：{}", diagnosis.suggestion)
                                        } else {
                                            format!("Next: {}", diagnosis.suggestion)
                                        })
                                        .color(egui::Color32::from_rgb(204, 226, 244)),
                                    );
                                }
                                if client.is_some() && !control_feedback_needs_recovery {
                                    ui.horizontal_wrapped(|ui| {
                                        recover_play_clicked = ui
                                            .button(if locale.is_zh() {
                                                "恢复：step x1"
                                            } else {
                                                "Recover: step x1"
                                            })
                                            .clicked();
                                        recover_step_clicked = ui
                                            .button(if locale.is_zh() {
                                                "恢复：step x8"
                                            } else {
                                                "Recover: step x8"
                                            })
                                            .clicked();
                                    });
                                }
                            });
                    }
                    if let Some(feedback) = control_feedback.as_ref() {
                        let stage_color = player_control_stage_color(feedback.stage.as_str());
                        let show_detail_card =
                            player_control_stage_shows_recovery_actions(feedback.stage.as_str())
                                || feedback.reason.is_some()
                                || feedback.hint.is_some();
                        if show_detail_card {
                            egui::Frame::group(ui.style())
                                .fill(egui::Color32::from_rgba_unmultiplied(28, 36, 52, 156))
                                .stroke(egui::Stroke::new(1.0, stage_color))
                                .corner_radius(egui::CornerRadius::same(6))
                                .inner_margin(egui::Margin::same(6))
                                .show(ui, |ui| {
                                    ui.small(if locale.is_zh() {
                                        "反馈细节"
                                    } else {
                                        "Feedback Details"
                                    });
                                    if let Some(reason) = feedback.reason.as_ref() {
                                        ui.small(
                                            egui::RichText::new(reason.as_str())
                                                .color(egui::Color32::from_rgb(226, 164, 136)),
                                        );
                                    }
                                    if let Some(hint) = feedback.hint.as_ref() {
                                        ui.small(
                                            egui::RichText::new(hint.as_str())
                                                .color(egui::Color32::from_rgb(186, 206, 238)),
                                        );
                                    }
                                    if player_control_stage_shows_recovery_actions(
                                        feedback.stage.as_str(),
                                    ) && client.is_some()
                                    {
                                        ui.horizontal_wrapped(|ui| {
                                            recover_play_clicked = ui
                                                .button(if locale.is_zh() {
                                                    "恢复：step x1"
                                                } else {
                                                    "Recover: step x1"
                                                })
                                                .clicked();
                                            recover_step_clicked = ui
                                                .button(if locale.is_zh() {
                                                    "重试：step x8"
                                                } else {
                                                    "Retry: step x8"
                                                })
                                                .clicked();
                                        });
                                    }
                                });
                        }
                    }
                    let progress_ratio = post_onboarding
                        .as_ref()
                        .map(|snapshot| snapshot.progress_percent as f32 / 100.0)
                        .unwrap_or_else(|| (snapshot.completed_steps as f32 / 4.0).clamp(0.0, 1.0));
                    ui.add(
                        egui::ProgressBar::new(progress_ratio)
                            .desired_width(280.0)
                            .text(format!(
                                "{} {}",
                                if locale.is_zh() {
                                    if post_onboarding.is_some() {
                                        "阶段进度"
                                    } else {
                                        "任务进度"
                                    }
                                } else if post_onboarding.is_some() {
                                    "Stage Progress"
                                } else {
                                    "Mission Progress"
                                },
                                if let Some(post_onboarding) = post_onboarding.as_ref() {
                                    format!("{}%", post_onboarding.progress_percent)
                                } else {
                                    format!("{}/4", snapshot.completed_steps)
                                }
                            )),
                    );
                    if compact_mode && post_onboarding.is_none() {
                        ui.small(egui::RichText::new(reward.badge).color(reward_tone));
                    } else if let Some(post_onboarding) = post_onboarding.as_ref() {
                        egui::Frame::group(ui.style())
                            .fill(egui::Color32::from_rgba_unmultiplied(
                                reward_tone.r(),
                                reward_tone.g(),
                                reward_tone.b(),
                                if matches!(
                                    post_onboarding.status,
                                    PlayerPostOnboardingStatus::BranchReady
                                ) {
                                    54
                                } else {
                                    28
                                },
                            ))
                            .stroke(egui::Stroke::new(1.0, reward_tone))
                            .corner_radius(egui::CornerRadius::same(8))
                            .inner_margin(egui::Margin::same(8))
                            .show(ui, |ui| {
                                ui.small(
                                    egui::RichText::new(player_post_onboarding_status_label(
                                        post_onboarding.status,
                                        locale,
                                    ))
                                    .color(reward_tone),
                                );
                                ui.strong(post_onboarding.title);
                                ui.small(post_onboarding.next_step.as_str());
                            });
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
                        action_clicked = ui
                            .button(
                                post_onboarding
                                    .as_ref()
                                    .map(|snapshot| snapshot.action_label)
                                    .unwrap_or(snapshot.action_label),
                            )
                            .clicked();
                        if player_mission_hud_show_command_action(layout_state.panel_hidden) {
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
    if action_clicked && post_onboarding.is_some() {
        apply_player_layout_preset(layout_state, module_visibility, PlayerLayoutPreset::Command);
        if let Some(client) = client {
            let _ = crate::dispatch_viewer_control(
                client,
                control_profile,
                agent_world::viewer::ViewerControl::Step { count: 1 },
                None,
            );
        }
    } else if action_clicked && step == PlayerGuideStep::ExploreAction {
        apply_player_layout_preset(layout_state, module_visibility, PlayerLayoutPreset::Command);
        if let Some(client) = client {
            let _ = crate::dispatch_viewer_control(
                client,
                control_profile,
                agent_world::viewer::ViewerControl::Step { count: 1 },
                None,
            );
        }
    }
    if command_clicked {
        apply_player_layout_preset(layout_state, module_visibility, PlayerLayoutPreset::Command);
    }
    if let Some(client) = client {
        if recover_play_clicked {
            let _ = crate::dispatch_viewer_control(
                client,
                control_profile,
                agent_world::viewer::ViewerControl::Step { count: 1 },
                None,
            );
        }
        if recover_step_clicked {
            let _ = crate::dispatch_viewer_control(
                client,
                control_profile,
                agent_world::viewer::ViewerControl::Step { count: 8 },
                None,
            );
        }
    }

    if player_mission_hud_show_minimap(layout_state.panel_hidden) {
        let points = build_player_minimap_snapshot(state, selection);
        render_player_minimap_card(context, &points, locale, now_secs);
    }
}

pub(super) fn player_mission_hud_compact_mode(panel_hidden: bool) -> bool {
    !panel_hidden
}

pub(super) fn player_mission_hud_anchor_y(
    panel_hidden: bool,
    onboarding_visible: bool,
    stuck_hint_visible: bool,
) -> f32 {
    if player_mission_hud_compact_mode(panel_hidden) {
        96.0
    } else if onboarding_visible {
        if stuck_hint_visible {
            298.0
        } else {
            214.0
        }
    } else {
        136.0
    }
}

pub(super) fn player_mission_hud_show_command_action(panel_hidden: bool) -> bool {
    panel_hidden
}

pub(super) fn player_mission_hud_show_minimap(panel_hidden: bool) -> bool {
    panel_hidden
}

pub(super) fn player_mission_hud_minimap_reserved_bottom(panel_hidden: bool) -> f32 {
    if player_mission_hud_show_minimap(panel_hidden) {
        188.0
    } else {
        0.0
    }
}
