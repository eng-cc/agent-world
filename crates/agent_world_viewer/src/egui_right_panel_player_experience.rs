use agent_world::simulator::{WorldEvent, WorldEventKind};
use bevy_egui::egui;

use crate::event_click_list::event_row_label;
use crate::selection_linking::selection_kind_label;
use crate::{RightPanelLayoutState, ViewerSelection, ViewerState};

const FEEDBACK_TOAST_MAX: usize = 3;
const FEEDBACK_TOAST_TTL_SECS: f64 = 4.2;
const FEEDBACK_TOAST_FADE_SECS: f64 = 0.8;
const PLAYER_GOAL_HINT_MAX_WIDTH: f32 = 320.0;
const PLAYER_ONBOARDING_MAX_WIDTH: f32 = 360.0;
const PLAYER_HUD_MAX_WIDTH: f32 = 760.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FeedbackTone {
    Positive,
    Warning,
    Info,
}

#[derive(Clone, Debug)]
struct FeedbackToast {
    id: u64,
    title: &'static str,
    detail: String,
    tone: FeedbackTone,
    expires_at_secs: f64,
}

#[derive(Default)]
pub(crate) struct FeedbackToastState {
    toasts: Vec<FeedbackToast>,
    last_seen_event_id: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PlayerGuideStep {
    ConnectWorld,
    OpenPanel,
    SelectTarget,
    ExploreAction,
}

#[derive(Default)]
pub(crate) struct PlayerOnboardingState {
    dismissed_step: Option<PlayerGuideStep>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlayerHudSnapshot {
    pub connection: String,
    pub tick: u64,
    pub events: usize,
    pub selection: String,
    pub objective: &'static str,
}

pub(super) fn feedback_tone_for_event(event: &WorldEventKind) -> FeedbackTone {
    match event {
        WorldEventKind::ActionRejected { .. } => FeedbackTone::Warning,
        WorldEventKind::FactoryBuilt { .. }
        | WorldEventKind::RecipeScheduled { .. }
        | WorldEventKind::CompoundMined { .. }
        | WorldEventKind::CompoundRefined { .. }
        | WorldEventKind::RadiationHarvested { .. }
        | WorldEventKind::AgentMoved { .. }
        | WorldEventKind::ModuleArtifactSaleCompleted { .. } => FeedbackTone::Positive,
        _ => FeedbackTone::Info,
    }
}

fn feedback_title_for_event(tone: FeedbackTone, locale: crate::i18n::UiLocale) -> &'static str {
    match (tone, locale.is_zh()) {
        (FeedbackTone::Positive, true) => "进展达成",
        (FeedbackTone::Positive, false) => "Progress",
        (FeedbackTone::Warning, true) => "操作受阻",
        (FeedbackTone::Warning, false) => "Action Blocked",
        (FeedbackTone::Info, true) => "世界更新",
        (FeedbackTone::Info, false) => "World Update",
    }
}

pub(super) fn push_feedback_toast(
    feedback: &mut FeedbackToastState,
    event: &WorldEvent,
    now_secs: f64,
    locale: crate::i18n::UiLocale,
) {
    let tone = feedback_tone_for_event(&event.kind);
    let detail = super::truncate_observe_text(&event_row_label(event, false, locale), 64);
    feedback.toasts.push(FeedbackToast {
        id: event.id,
        title: feedback_title_for_event(tone, locale),
        detail,
        tone,
        expires_at_secs: now_secs + FEEDBACK_TOAST_TTL_SECS,
    });
    while feedback.toasts.len() > FEEDBACK_TOAST_MAX {
        feedback.toasts.remove(0);
    }
}

pub(super) fn sync_feedback_toasts(
    feedback: &mut FeedbackToastState,
    state: &ViewerState,
    now_secs: f64,
    locale: crate::i18n::UiLocale,
) {
    feedback
        .toasts
        .retain(|toast| toast.expires_at_secs > now_secs);

    let newest_event_id = state.events.last().map(|event| event.id);
    let Some(newest_event_id) = newest_event_id else {
        return;
    };

    let Some(last_seen) = feedback.last_seen_event_id else {
        feedback.last_seen_event_id = Some(newest_event_id);
        return;
    };

    if newest_event_id <= last_seen {
        return;
    }

    let mut seen_max = last_seen;
    for event in state.events.iter().filter(|event| event.id > last_seen) {
        push_feedback_toast(feedback, event, now_secs, locale);
        seen_max = seen_max.max(event.id);
    }
    feedback.last_seen_event_id = Some(seen_max);
}

fn feedback_fill_color(tone: FeedbackTone, alpha: f32) -> egui::Color32 {
    let alpha = alpha.clamp(0.0, 1.0);
    let to_u8 = |value: f32| (value.clamp(0.0, 255.0)) as u8;
    match tone {
        FeedbackTone::Positive => {
            egui::Color32::from_rgba_unmultiplied(22, 69, 50, to_u8(232.0 * alpha))
        }
        FeedbackTone::Warning => {
            egui::Color32::from_rgba_unmultiplied(99, 44, 32, to_u8(236.0 * alpha))
        }
        FeedbackTone::Info => {
            egui::Color32::from_rgba_unmultiplied(24, 42, 66, to_u8(224.0 * alpha))
        }
    }
}

pub(super) fn render_feedback_toasts(
    context: &egui::Context,
    feedback: &FeedbackToastState,
    now_secs: f64,
) {
    let mut vertical_offset = 12.0;
    for toast in feedback.toasts.iter().rev() {
        let remaining = (toast.expires_at_secs - now_secs).max(0.0);
        let alpha = if remaining < FEEDBACK_TOAST_FADE_SECS {
            (remaining / FEEDBACK_TOAST_FADE_SECS) as f32
        } else {
            1.0
        };

        egui::Area::new(egui::Id::new(("viewer-feedback-toast", toast.id)))
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, vertical_offset))
            .movable(false)
            .interactable(false)
            .show(context, |ui| {
                egui::Frame::group(ui.style())
                    .fill(feedback_fill_color(toast.tone, alpha))
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::same(9))
                    .show(ui, |ui| {
                        ui.set_max_width(360.0);
                        ui.strong(toast.title);
                        ui.small(toast.detail.as_str());
                    });
            });
        vertical_offset += 68.0;
    }
}

pub(super) fn resolve_player_guide_step(
    status: &crate::ConnectionStatus,
    layout_state: &RightPanelLayoutState,
    selection: &ViewerSelection,
) -> PlayerGuideStep {
    if !matches!(status, crate::ConnectionStatus::Connected) {
        PlayerGuideStep::ConnectWorld
    } else if layout_state.panel_hidden {
        PlayerGuideStep::OpenPanel
    } else if selection.current.is_none() {
        PlayerGuideStep::SelectTarget
    } else {
        PlayerGuideStep::ExploreAction
    }
}

fn player_goal_title(step: PlayerGuideStep, locale: crate::i18n::UiLocale) -> &'static str {
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

fn player_goal_detail(step: PlayerGuideStep, locale: crate::i18n::UiLocale) -> &'static str {
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

fn player_goal_color(step: PlayerGuideStep) -> egui::Color32 {
    match step {
        PlayerGuideStep::ConnectWorld => egui::Color32::from_rgb(122, 88, 34),
        PlayerGuideStep::OpenPanel => egui::Color32::from_rgb(44, 92, 152),
        PlayerGuideStep::SelectTarget => egui::Color32::from_rgb(30, 112, 88),
        PlayerGuideStep::ExploreAction => egui::Color32::from_rgb(38, 128, 74),
    }
}

fn player_goal_badge(locale: crate::i18n::UiLocale) -> &'static str {
    if locale.is_zh() {
        "下一步目标"
    } else {
        "Next Goal"
    }
}

fn player_onboarding_title(locale: crate::i18n::UiLocale) -> &'static str {
    if locale.is_zh() {
        "新手引导"
    } else {
        "Player Guide"
    }
}

fn player_onboarding_primary_action(
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

fn player_onboarding_dismiss(locale: crate::i18n::UiLocale) -> &'static str {
    if locale.is_zh() {
        "关闭当前提示"
    } else {
        "Hide this tip"
    }
}

fn player_connection_color(status: &crate::ConnectionStatus) -> egui::Color32 {
    match status {
        crate::ConnectionStatus::Connected => egui::Color32::from_rgb(36, 130, 72),
        crate::ConnectionStatus::Connecting => egui::Color32::from_rgb(160, 116, 40),
        crate::ConnectionStatus::Error(_) => egui::Color32::from_rgb(170, 58, 58),
    }
}

fn player_connection_text(
    status: &crate::ConnectionStatus,
    locale: crate::i18n::UiLocale,
) -> &'static str {
    match (status, locale.is_zh()) {
        (crate::ConnectionStatus::Connected, true) => "已连接",
        (crate::ConnectionStatus::Connected, false) => "Connected",
        (crate::ConnectionStatus::Connecting, true) => "连接中",
        (crate::ConnectionStatus::Connecting, false) => "Connecting",
        (crate::ConnectionStatus::Error(_), true) => "连接异常",
        (crate::ConnectionStatus::Error(_), false) => "Connection Error",
    }
}

fn player_selection_text(selection: &ViewerSelection, locale: crate::i18n::UiLocale) -> String {
    let Some(current) = selection.current.as_ref() else {
        return if locale.is_zh() {
            "未选择".to_string()
        } else {
            "None".to_string()
        };
    };
    let id = super::truncate_observe_text(&current.id, 16);
    format!("{} {id}", selection_kind_label(current.kind))
}

fn player_current_tick(state: &ViewerState) -> u64 {
    state
        .snapshot
        .as_ref()
        .map(|snapshot| snapshot.time)
        .or_else(|| state.metrics.as_ref().map(|metrics| metrics.total_ticks))
        .unwrap_or(0)
}

pub(super) fn build_player_hud_snapshot(
    state: &ViewerState,
    selection: &ViewerSelection,
    step: PlayerGuideStep,
    locale: crate::i18n::UiLocale,
) -> PlayerHudSnapshot {
    PlayerHudSnapshot {
        connection: player_connection_text(&state.status, locale).to_string(),
        tick: player_current_tick(state),
        events: state.events.len(),
        selection: player_selection_text(selection, locale),
        objective: player_goal_title(step, locale),
    }
}

fn render_hud_chip(
    ui: &mut egui::Ui,
    label: &str,
    value: &str,
    tone: egui::Color32,
    emphasized: bool,
) {
    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgb(22, 31, 45))
        .stroke(egui::Stroke::new(1.0, tone))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(7))
        .show(ui, |ui| {
            ui.small(egui::RichText::new(label).color(tone));
            if emphasized {
                ui.strong(value);
            } else {
                ui.small(value);
            }
        });
}

pub(super) fn player_entry_card_style(now_secs: f64) -> (egui::Color32, egui::Stroke) {
    let pulse = ((now_secs * 2.0).sin() * 0.5 + 0.5) as f32;
    let fill = egui::Color32::from_rgb(
        18,
        (28.0 + pulse * 8.0).round() as u8,
        (40.0 + pulse * 12.0).round() as u8,
    );
    let stroke = egui::Stroke::new(
        1.0,
        egui::Color32::from_rgb(
            (58.0 + pulse * 32.0).round() as u8,
            (106.0 + pulse * 28.0).round() as u8,
            (152.0 + pulse * 40.0).round() as u8,
        ),
    );
    (fill, stroke)
}

pub(super) fn render_player_compact_hud(
    context: &egui::Context,
    state: &ViewerState,
    selection: &ViewerSelection,
    step: PlayerGuideStep,
    locale: crate::i18n::UiLocale,
    now_secs: f64,
) {
    let snapshot = build_player_hud_snapshot(state, selection, step, locale);
    let objective_color = player_goal_color(step);
    let pulse = ((now_secs * 1.6).sin() * 0.5 + 0.5) as f32;
    let accent = egui::Color32::from_rgba_unmultiplied(
        objective_color.r(),
        objective_color.g(),
        objective_color.b(),
        (136.0 + 72.0 * pulse) as u8,
    );

    egui::Area::new(egui::Id::new("viewer-player-compact-hud"))
        .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 10.0))
        .movable(false)
        .interactable(false)
        .show(context, |ui| {
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgb(12, 20, 30))
                .stroke(egui::Stroke::new(1.0, accent))
                .corner_radius(egui::CornerRadius::same(12))
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.set_max_width(PLAYER_HUD_MAX_WIDTH);
                    ui.horizontal_wrapped(|ui| {
                        render_hud_chip(
                            ui,
                            if locale.is_zh() { "连接" } else { "Conn" },
                            snapshot.connection.as_str(),
                            player_connection_color(&state.status),
                            false,
                        );
                        render_hud_chip(
                            ui,
                            if locale.is_zh() { "Tick" } else { "Tick" },
                            snapshot.tick.to_string().as_str(),
                            egui::Color32::from_rgb(112, 160, 224),
                            true,
                        );
                        render_hud_chip(
                            ui,
                            if locale.is_zh() { "事件" } else { "Events" },
                            snapshot.events.to_string().as_str(),
                            egui::Color32::from_rgb(114, 188, 166),
                            false,
                        );
                        render_hud_chip(
                            ui,
                            if locale.is_zh() { "目标" } else { "Target" },
                            snapshot.selection.as_str(),
                            egui::Color32::from_rgb(152, 178, 232),
                            false,
                        );
                        render_hud_chip(
                            ui,
                            if locale.is_zh() {
                                "当前目标"
                            } else {
                                "Objective"
                            },
                            snapshot.objective,
                            objective_color,
                            false,
                        );
                    });
                });
        });
}

pub(super) fn should_show_player_onboarding_card(
    onboarding: &PlayerOnboardingState,
    step: PlayerGuideStep,
) -> bool {
    onboarding.dismissed_step != Some(step)
}

pub(super) fn dismiss_player_onboarding_step(
    onboarding: &mut PlayerOnboardingState,
    step: PlayerGuideStep,
) {
    onboarding.dismissed_step = Some(step);
}

#[cfg(test)]
pub(super) fn feedback_toast_cap() -> usize {
    FEEDBACK_TOAST_MAX
}

#[cfg(test)]
pub(super) fn feedback_toast_len(feedback: &FeedbackToastState) -> usize {
    feedback.toasts.len()
}

#[cfg(test)]
pub(super) fn feedback_toast_ids(feedback: &FeedbackToastState) -> Vec<u64> {
    feedback.toasts.iter().map(|toast| toast.id).collect()
}

#[cfg(test)]
pub(super) fn feedback_last_seen_event_id(feedback: &FeedbackToastState) -> Option<u64> {
    feedback.last_seen_event_id
}

#[cfg(test)]
pub(super) fn feedback_toast_snapshot(
    feedback: &FeedbackToastState,
    index: usize,
) -> Option<(u64, FeedbackTone, &'static str)> {
    feedback
        .toasts
        .get(index)
        .map(|toast| (toast.id, toast.tone, toast.title))
}

pub(super) fn render_player_goal_hint(
    context: &egui::Context,
    step: PlayerGuideStep,
    locale: crate::i18n::UiLocale,
) {
    let tone = player_goal_color(step);
    egui::Area::new(egui::Id::new("viewer-player-next-goal"))
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(14.0, -14.0))
        .movable(false)
        .interactable(false)
        .show(context, |ui| {
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgb(15, 20, 30))
                .stroke(egui::Stroke::new(1.0, tone))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::same(9))
                .show(ui, |ui| {
                    ui.set_max_width(PLAYER_GOAL_HINT_MAX_WIDTH);
                    ui.small(egui::RichText::new(player_goal_badge(locale)).color(tone));
                    ui.strong(player_goal_title(step, locale));
                    ui.small(player_goal_detail(step, locale));
                });
        });
}

pub(super) fn render_player_onboarding_card(
    context: &egui::Context,
    onboarding: &mut PlayerOnboardingState,
    step: PlayerGuideStep,
    layout_state: &mut RightPanelLayoutState,
    locale: crate::i18n::UiLocale,
) {
    if !should_show_player_onboarding_card(onboarding, step) {
        return;
    }

    let tone = player_goal_color(step);
    let mut primary_clicked = false;
    let mut dismiss_clicked = false;
    egui::Area::new(egui::Id::new("viewer-player-onboarding"))
        .anchor(egui::Align2::LEFT_TOP, egui::vec2(14.0, 14.0))
        .movable(false)
        .interactable(true)
        .show(context, |ui| {
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgb(19, 26, 38))
                .stroke(egui::Stroke::new(1.0, tone))
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.set_max_width(PLAYER_ONBOARDING_MAX_WIDTH);
                    ui.small(
                        egui::RichText::new(player_onboarding_title(locale))
                            .strong()
                            .color(tone),
                    );
                    ui.strong(player_goal_title(step, locale));
                    ui.label(player_goal_detail(step, locale));
                    ui.horizontal_wrapped(|ui| {
                        primary_clicked = ui
                            .button(player_onboarding_primary_action(step, locale))
                            .clicked();
                        dismiss_clicked = ui.button(player_onboarding_dismiss(locale)).clicked();
                    });
                });
        });

    if primary_clicked && step == PlayerGuideStep::OpenPanel {
        layout_state.panel_hidden = false;
    }

    if primary_clicked || dismiss_clicked {
        dismiss_player_onboarding_step(onboarding, step);
    }
}
