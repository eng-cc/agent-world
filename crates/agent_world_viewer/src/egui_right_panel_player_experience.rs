use super::egui_right_panel_player_card_motion::{
    build_player_card_transition_snapshot, sync_player_guide_transition, PlayerGuideTransitionState,
};
use super::egui_right_panel_player_guide::{
    build_player_guide_progress_snapshot, player_goal_badge, player_goal_color, player_goal_detail,
    player_goal_title, player_onboarding_dismiss, player_onboarding_primary_action,
    player_onboarding_title, render_player_cinematic_intro, render_player_guide_progress_lines,
    render_player_layout_preset_strip, render_player_mission_hud, PlayerGuideProgressSnapshot,
};
use agent_world::simulator::{ResourceOwner, WorldEvent, WorldEventKind};
use bevy_egui::egui;
use std::collections::BTreeSet;

use crate::event_click_list::event_row_label;
use crate::selection_linking::selection_kind_label;
use crate::{RightPanelLayoutState, ViewerSelection, ViewerState};

const FEEDBACK_TOAST_MAX: usize = 3;
const FEEDBACK_TOAST_TTL_SECS: f64 = 4.2;
const FEEDBACK_TOAST_FADE_SECS: f64 = 0.8;
const PLAYER_ACHIEVEMENT_MAX: usize = 3;
const PLAYER_ACHIEVEMENT_TTL_SECS: f64 = 5.2;
const PLAYER_ACHIEVEMENT_FADE_SECS: f64 = 1.0;
const PLAYER_ACHIEVEMENT_MAX_WIDTH: f32 = 320.0;
const PLAYER_ATMOSPHERE_TOP_ALPHA_BASE: f32 = 0.12;
const PLAYER_ATMOSPHERE_BOTTOM_ALPHA_BASE: f32 = 0.08;
const AGENT_CHATTER_MAX: usize = 4;
const AGENT_CHATTER_TTL_SECS: f64 = 5.0;
const AGENT_CHATTER_FADE_SECS: f64 = 0.9;
const AGENT_CHATTER_MAX_WIDTH: f32 = 320.0;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum PlayerAchievementMilestone {
    WorldConnected,
    PanelOpened,
    FirstSelection,
    FirstEventSeen,
}

#[derive(Clone, Debug)]
struct PlayerAchievementToast {
    id: u64,
    milestone: PlayerAchievementMilestone,
    expires_at_secs: f64,
}

#[derive(Clone, Debug)]
struct PlayerAgentChatterBubble {
    id: u64,
    speaker: String,
    line: String,
    tone: FeedbackTone,
    expires_at_secs: f64,
}

#[derive(Default)]
struct PlayerAgentChatterState {
    bubbles: Vec<PlayerAgentChatterBubble>,
    last_seen_event_id: Option<u64>,
}

#[derive(Default)]
pub(crate) struct PlayerAchievementState {
    unlocked: BTreeSet<PlayerAchievementMilestone>,
    toasts: Vec<PlayerAchievementToast>,
    next_toast_id: u64,
    chatter: PlayerAgentChatterState,
}

#[derive(Default)]
pub(crate) struct PlayerOnboardingState {
    dismissed_step: Option<PlayerGuideStep>,
    guide_transition: PlayerGuideTransitionState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlayerHudSnapshot {
    pub connection: String,
    pub tick: u64,
    pub events: usize,
    pub selection: String,
    pub objective: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PlayerAtmosphereSnapshot {
    pub(super) top_alpha: f32,
    pub(super) bottom_alpha: f32,
    pub(super) orb_x_factor: f32,
    pub(super) orb_y_factor: f32,
    pub(super) orb_radius: f32,
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

pub(super) fn build_player_atmosphere_snapshot(now_secs: f64) -> PlayerAtmosphereSnapshot {
    let pulse = ((now_secs * 0.55).sin() * 0.5 + 0.5) as f32;
    let drift = ((now_secs * 0.32).cos() * 0.5 + 0.5) as f32;
    PlayerAtmosphereSnapshot {
        top_alpha: (PLAYER_ATMOSPHERE_TOP_ALPHA_BASE + 0.05 * pulse).clamp(0.0, 0.28),
        bottom_alpha: (PLAYER_ATMOSPHERE_BOTTOM_ALPHA_BASE + 0.06 * drift).clamp(0.0, 0.25),
        orb_x_factor: 0.74 + 0.08 * ((now_secs * 0.24).sin() as f32),
        orb_y_factor: 0.22 + 0.05 * ((now_secs * 0.18).cos() as f32),
        orb_radius: 120.0 + 42.0 * pulse,
    }
}

pub(super) fn render_player_atmosphere(context: &egui::Context, now_secs: f64) {
    let snapshot = build_player_atmosphere_snapshot(now_secs);
    let rect = context.content_rect();
    if rect.width() <= 1.0 || rect.height() <= 1.0 {
        return;
    }
    let layer = egui::LayerId::new(
        egui::Order::Background,
        egui::Id::new("viewer-player-atmosphere"),
    );
    let painter = context.layer_painter(layer);
    let to_u8 = |value: f32| (value.clamp(0.0, 255.0)) as u8;

    let top_rect = egui::Rect::from_min_max(
        rect.min,
        egui::pos2(rect.max.x, rect.min.y + rect.height() * 0.34),
    );
    painter.rect_filled(
        top_rect,
        0.0,
        egui::Color32::from_rgba_unmultiplied(12, 28, 44, to_u8(snapshot.top_alpha * 255.0)),
    );

    let bottom_rect = egui::Rect::from_min_max(
        egui::pos2(rect.min.x, rect.max.y - rect.height() * 0.29),
        rect.max,
    );
    painter.rect_filled(
        bottom_rect,
        0.0,
        egui::Color32::from_rgba_unmultiplied(8, 18, 34, to_u8(snapshot.bottom_alpha * 255.0)),
    );

    let orb_center = egui::pos2(
        rect.min.x + rect.width() * snapshot.orb_x_factor,
        rect.min.y + rect.height() * snapshot.orb_y_factor,
    );
    painter.circle_filled(
        orb_center,
        snapshot.orb_radius,
        egui::Color32::from_rgba_unmultiplied(42, 132, 188, 28),
    );
}

fn player_achievement_badge(locale: crate::i18n::UiLocale) -> &'static str {
    if locale.is_zh() {
        "里程碑解锁"
    } else {
        "Milestone Unlocked"
    }
}

fn player_achievement_title(
    milestone: PlayerAchievementMilestone,
    locale: crate::i18n::UiLocale,
) -> &'static str {
    match (milestone, locale.is_zh()) {
        (PlayerAchievementMilestone::WorldConnected, true) => "世界连接成功",
        (PlayerAchievementMilestone::WorldConnected, false) => "World Link Established",
        (PlayerAchievementMilestone::PanelOpened, true) => "操作面板已展开",
        (PlayerAchievementMilestone::PanelOpened, false) => "Control Panel Online",
        (PlayerAchievementMilestone::FirstSelection, true) => "首次锁定目标",
        (PlayerAchievementMilestone::FirstSelection, false) => "First Target Locked",
        (PlayerAchievementMilestone::FirstEventSeen, true) => "首次收到世界回应",
        (PlayerAchievementMilestone::FirstEventSeen, false) => "First World Response",
    }
}

fn player_achievement_detail(
    milestone: PlayerAchievementMilestone,
    locale: crate::i18n::UiLocale,
) -> &'static str {
    match (milestone, locale.is_zh()) {
        (PlayerAchievementMilestone::WorldConnected, true) => "实时 Tick 与事件流已经开始。",
        (PlayerAchievementMilestone::WorldConnected, false) => {
            "Live ticks and events are now flowing."
        }
        (PlayerAchievementMilestone::PanelOpened, true) => "主操作入口已就绪，可随时查看详情。",
        (PlayerAchievementMilestone::PanelOpened, false) => {
            "Control entry is ready, inspect details anytime."
        }
        (PlayerAchievementMilestone::FirstSelection, true) => "你可以围绕该目标推进下一步行动。",
        (PlayerAchievementMilestone::FirstSelection, false) => {
            "You can now plan actions around this target."
        }
        (PlayerAchievementMilestone::FirstEventSeen, true) => "你的操作已在世界中产生反馈。",
        (PlayerAchievementMilestone::FirstEventSeen, false) => {
            "Your actions are now reflected in the world."
        }
    }
}

fn player_achievement_color(milestone: PlayerAchievementMilestone) -> egui::Color32 {
    match milestone {
        PlayerAchievementMilestone::WorldConnected => egui::Color32::from_rgb(56, 108, 176),
        PlayerAchievementMilestone::PanelOpened => egui::Color32::from_rgb(72, 146, 204),
        PlayerAchievementMilestone::FirstSelection => egui::Color32::from_rgb(58, 152, 102),
        PlayerAchievementMilestone::FirstEventSeen => egui::Color32::from_rgb(194, 142, 62),
    }
}

fn should_unlock_player_achievement(
    milestone: PlayerAchievementMilestone,
    state: &ViewerState,
    selection: &ViewerSelection,
    layout_state: &RightPanelLayoutState,
) -> bool {
    match milestone {
        PlayerAchievementMilestone::WorldConnected => {
            matches!(state.status, crate::ConnectionStatus::Connected)
        }
        PlayerAchievementMilestone::PanelOpened => !layout_state.panel_hidden,
        PlayerAchievementMilestone::FirstSelection => selection.current.is_some(),
        PlayerAchievementMilestone::FirstEventSeen => !state.events.is_empty(),
    }
}

fn unlock_player_achievement(
    achievements: &mut PlayerAchievementState,
    milestone: PlayerAchievementMilestone,
    now_secs: f64,
) -> bool {
    if !achievements.unlocked.insert(milestone) {
        return false;
    }

    achievements.next_toast_id = achievements.next_toast_id.saturating_add(1);
    achievements.toasts.push(PlayerAchievementToast {
        id: achievements.next_toast_id,
        milestone,
        expires_at_secs: now_secs + PLAYER_ACHIEVEMENT_TTL_SECS,
    });
    while achievements.toasts.len() > PLAYER_ACHIEVEMENT_MAX {
        achievements.toasts.remove(0);
    }
    true
}

pub(super) fn sync_player_achievements(
    achievements: &mut PlayerAchievementState,
    state: &ViewerState,
    selection: &ViewerSelection,
    layout_state: &RightPanelLayoutState,
    now_secs: f64,
) {
    achievements
        .toasts
        .retain(|toast| toast.expires_at_secs > now_secs);

    let milestones = [
        PlayerAchievementMilestone::WorldConnected,
        PlayerAchievementMilestone::PanelOpened,
        PlayerAchievementMilestone::FirstSelection,
        PlayerAchievementMilestone::FirstEventSeen,
    ];

    for milestone in milestones {
        if achievements.unlocked.contains(&milestone) {
            continue;
        }
        if should_unlock_player_achievement(milestone, state, selection, layout_state) {
            unlock_player_achievement(achievements, milestone, now_secs);
            break;
        }
    }
}

pub(super) fn render_player_achievement_popups(
    context: &egui::Context,
    achievements: &PlayerAchievementState,
    locale: crate::i18n::UiLocale,
    now_secs: f64,
) {
    let mut vertical_offset = 88.0;
    let to_u8 = |value: f32| (value.clamp(0.0, 255.0)) as u8;
    for toast in achievements.toasts.iter().rev() {
        let remaining = (toast.expires_at_secs - now_secs).max(0.0);
        let alpha = if remaining < PLAYER_ACHIEVEMENT_FADE_SECS {
            (remaining / PLAYER_ACHIEVEMENT_FADE_SECS) as f32
        } else {
            1.0
        };
        let tone = player_achievement_color(toast.milestone);
        let fill = egui::Color32::from_rgba_unmultiplied(16, 25, 20, to_u8(224.0 * alpha));
        let stroke = egui::Stroke::new(
            1.0,
            egui::Color32::from_rgba_unmultiplied(
                tone.r(),
                tone.g(),
                tone.b(),
                to_u8(236.0 * alpha),
            ),
        );

        egui::Area::new(egui::Id::new(("viewer-player-achievement", toast.id)))
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-14.0, vertical_offset))
            .movable(false)
            .interactable(false)
            .show(context, |ui| {
                egui::Frame::group(ui.style())
                    .fill(fill)
                    .stroke(stroke)
                    .corner_radius(egui::CornerRadius::same(9))
                    .inner_margin(egui::Margin::same(10))
                    .show(ui, |ui| {
                        ui.set_max_width(PLAYER_ACHIEVEMENT_MAX_WIDTH);
                        ui.small(egui::RichText::new(player_achievement_badge(locale)).color(tone));
                        ui.strong(player_achievement_title(toast.milestone, locale));
                        ui.small(player_achievement_detail(toast.milestone, locale));
                    });
            });
        vertical_offset += 86.0;
    }
}

fn owner_agent_id(owner: &ResourceOwner) -> Option<&str> {
    match owner {
        ResourceOwner::Agent { agent_id } => Some(agent_id.as_str()),
        ResourceOwner::Location { .. } => None,
    }
}

fn chatter_line_for_event(
    event: &WorldEvent,
    locale: crate::i18n::UiLocale,
) -> Option<(String, String, FeedbackTone)> {
    match &event.kind {
        WorldEventKind::AgentMoved { agent_id, to, .. } => Some((
            super::truncate_observe_text(agent_id, 14),
            if locale.is_zh() {
                format!("已移动至 {}", super::truncate_observe_text(to, 14))
            } else {
                format!("Moved to {}", super::truncate_observe_text(to, 14))
            },
            FeedbackTone::Positive,
        )),
        WorldEventKind::RadiationHarvested {
            agent_id, amount, ..
        } => Some((
            super::truncate_observe_text(agent_id, 14),
            if locale.is_zh() {
                format!("采集辐照 +{amount}")
            } else {
                format!("Harvested radiation +{amount}")
            },
            FeedbackTone::Positive,
        )),
        WorldEventKind::CompoundMined {
            owner,
            compound_mass_g,
            ..
        } => owner_agent_id(owner).map(|agent_id| {
            (
                super::truncate_observe_text(agent_id, 14),
                if locale.is_zh() {
                    format!("开采复合物 {compound_mass_g}g")
                } else {
                    format!("Mined compound {compound_mass_g}g")
                },
                FeedbackTone::Positive,
            )
        }),
        WorldEventKind::CompoundRefined {
            owner,
            hardware_output,
            ..
        } => owner_agent_id(owner).map(|agent_id| {
            (
                super::truncate_observe_text(agent_id, 14),
                if locale.is_zh() {
                    format!("精炼产出硬件 +{hardware_output}")
                } else {
                    format!("Refined hardware +{hardware_output}")
                },
                FeedbackTone::Positive,
            )
        }),
        WorldEventKind::FactoryBuilt {
            owner,
            factory_kind,
            ..
        } => owner_agent_id(owner).map(|agent_id| {
            (
                super::truncate_observe_text(agent_id, 14),
                if locale.is_zh() {
                    format!("建成 {}", super::truncate_observe_text(factory_kind, 18))
                } else {
                    format!("Built {}", super::truncate_observe_text(factory_kind, 18))
                },
                FeedbackTone::Info,
            )
        }),
        WorldEventKind::RecipeScheduled {
            owner, recipe_id, ..
        } => owner_agent_id(owner).map(|agent_id| {
            (
                super::truncate_observe_text(agent_id, 14),
                if locale.is_zh() {
                    format!("启动配方 {}", super::truncate_observe_text(recipe_id, 18))
                } else {
                    format!(
                        "Started recipe {}",
                        super::truncate_observe_text(recipe_id, 18)
                    )
                },
                FeedbackTone::Info,
            )
        }),
        WorldEventKind::ActionRejected { .. } => Some((
            if locale.is_zh() {
                "系统".to_string()
            } else {
                "System".to_string()
            },
            super::truncate_observe_text(&event_row_label(event, false, locale), 58),
            FeedbackTone::Warning,
        )),
        _ => None,
    }
}

fn chatter_stroke_color(tone: FeedbackTone, alpha: f32) -> egui::Color32 {
    let alpha = alpha.clamp(0.0, 1.0);
    let to_u8 = |value: f32| (value.clamp(0.0, 255.0)) as u8;
    match tone {
        FeedbackTone::Positive => {
            egui::Color32::from_rgba_unmultiplied(72, 180, 118, to_u8(236.0 * alpha))
        }
        FeedbackTone::Warning => {
            egui::Color32::from_rgba_unmultiplied(214, 104, 82, to_u8(232.0 * alpha))
        }
        FeedbackTone::Info => {
            egui::Color32::from_rgba_unmultiplied(108, 166, 230, to_u8(224.0 * alpha))
        }
    }
}

fn push_agent_chatter_bubble(
    achievements: &mut PlayerAchievementState,
    event: &WorldEvent,
    speaker: String,
    line: String,
    tone: FeedbackTone,
    now_secs: f64,
) {
    achievements.chatter.bubbles.push(PlayerAgentChatterBubble {
        id: event.id,
        speaker: super::truncate_observe_text(&speaker, 14),
        line: super::truncate_observe_text(&line, 68),
        tone,
        expires_at_secs: now_secs + AGENT_CHATTER_TTL_SECS,
    });
    while achievements.chatter.bubbles.len() > AGENT_CHATTER_MAX {
        achievements.chatter.bubbles.remove(0);
    }
}

pub(super) fn sync_agent_chatter_bubbles(
    achievements: &mut PlayerAchievementState,
    state: &ViewerState,
    now_secs: f64,
    locale: crate::i18n::UiLocale,
) {
    achievements
        .chatter
        .bubbles
        .retain(|bubble| bubble.expires_at_secs > now_secs);

    let newest_event_id = state.events.last().map(|event| event.id);
    let Some(newest_event_id) = newest_event_id else {
        return;
    };

    let Some(last_seen) = achievements.chatter.last_seen_event_id else {
        achievements.chatter.last_seen_event_id = Some(newest_event_id);
        return;
    };

    if newest_event_id <= last_seen {
        return;
    }

    let mut seen_max = last_seen;
    for event in state.events.iter().filter(|event| event.id > last_seen) {
        if let Some((speaker, line, tone)) = chatter_line_for_event(event, locale) {
            push_agent_chatter_bubble(achievements, event, speaker, line, tone, now_secs);
        }
        seen_max = seen_max.max(event.id);
    }
    achievements.chatter.last_seen_event_id = Some(seen_max);
}

pub(super) fn render_agent_chatter_bubbles(
    context: &egui::Context,
    achievements: &PlayerAchievementState,
    now_secs: f64,
) {
    let mut vertical_offset = 14.0;
    for bubble in achievements.chatter.bubbles.iter().rev() {
        let remaining = (bubble.expires_at_secs - now_secs).max(0.0);
        let alpha = if remaining < AGENT_CHATTER_FADE_SECS {
            (remaining / AGENT_CHATTER_FADE_SECS) as f32
        } else {
            1.0
        };
        let accent = chatter_stroke_color(bubble.tone, alpha);

        egui::Area::new(egui::Id::new(("viewer-agent-chatter", bubble.id)))
            .anchor(
                egui::Align2::RIGHT_BOTTOM,
                egui::vec2(-14.0, -vertical_offset),
            )
            .movable(false)
            .interactable(false)
            .show(context, |ui| {
                egui::Frame::group(ui.style())
                    .fill(feedback_fill_color(bubble.tone, 0.82 * alpha))
                    .stroke(egui::Stroke::new(1.0, accent))
                    .corner_radius(egui::CornerRadius::same(9))
                    .inner_margin(egui::Margin::same(9))
                    .show(ui, |ui| {
                        ui.set_max_width(AGENT_CHATTER_MAX_WIDTH);
                        ui.small(egui::RichText::new(bubble.speaker.as_str()).color(accent));
                        ui.label(bubble.line.as_str());
                    });
            });
        vertical_offset += 74.0;
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

pub(super) fn should_show_player_goal_hint(
    onboarding: &PlayerOnboardingState,
    step: PlayerGuideStep,
    layout_state: &RightPanelLayoutState,
) -> bool {
    layout_state.panel_hidden && !should_show_player_onboarding_card(onboarding, step)
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

#[cfg(test)]
pub(super) fn player_achievement_popup_cap() -> usize {
    PLAYER_ACHIEVEMENT_MAX
}

#[cfg(test)]
pub(super) fn player_achievement_popup_len(achievements: &PlayerAchievementState) -> usize {
    achievements.toasts.len()
}

#[cfg(test)]
pub(super) fn player_achievement_popup_milestones(
    achievements: &PlayerAchievementState,
) -> Vec<PlayerAchievementMilestone> {
    achievements
        .toasts
        .iter()
        .map(|toast| toast.milestone)
        .collect()
}

#[cfg(test)]
pub(super) fn player_achievement_is_unlocked(
    achievements: &PlayerAchievementState,
    milestone: PlayerAchievementMilestone,
) -> bool {
    achievements.unlocked.contains(&milestone)
}

#[cfg(test)]
pub(super) fn player_agent_chatter_cap() -> usize {
    AGENT_CHATTER_MAX
}

#[cfg(test)]
pub(super) fn player_agent_chatter_len(achievements: &PlayerAchievementState) -> usize {
    achievements.chatter.bubbles.len()
}

#[cfg(test)]
pub(super) fn player_agent_chatter_last_seen_event_id(
    achievements: &PlayerAchievementState,
) -> Option<u64> {
    achievements.chatter.last_seen_event_id
}

#[cfg(test)]
pub(super) fn player_agent_chatter_ids(achievements: &PlayerAchievementState) -> Vec<u64> {
    achievements
        .chatter
        .bubbles
        .iter()
        .map(|bubble| bubble.id)
        .collect()
}

#[cfg(test)]
pub(super) fn player_agent_chatter_snapshot(
    achievements: &PlayerAchievementState,
    index: usize,
) -> Option<(u64, FeedbackTone, String, String)> {
    achievements.chatter.bubbles.get(index).map(|bubble| {
        (
            bubble.id,
            bubble.tone,
            bubble.speaker.clone(),
            bubble.line.clone(),
        )
    })
}

pub(super) fn render_player_goal_hint(
    context: &egui::Context,
    onboarding: &PlayerOnboardingState,
    step: PlayerGuideStep,
    progress: PlayerGuideProgressSnapshot,
    locale: crate::i18n::UiLocale,
    now_secs: f64,
) {
    let tone = player_goal_color(step);
    let motion =
        build_player_card_transition_snapshot(&onboarding.guide_transition, step, now_secs, 0.8);
    let to_u8 = |value: f32| (value.clamp(0.0, 255.0)) as u8;
    egui::Area::new(egui::Id::new("viewer-player-next-goal"))
        .anchor(
            egui::Align2::LEFT_BOTTOM,
            egui::vec2(14.0, -14.0 + motion.slide_px),
        )
        .movable(false)
        .interactable(false)
        .show(context, |ui| {
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgba_unmultiplied(
                    15,
                    20,
                    30,
                    to_u8(224.0 * motion.alpha),
                ))
                .stroke(egui::Stroke::new(
                    1.0 + 0.4 * motion.pulse,
                    egui::Color32::from_rgba_unmultiplied(
                        tone.r(),
                        tone.g(),
                        tone.b(),
                        to_u8((152.0 + 84.0 * motion.pulse) * motion.alpha),
                    ),
                ))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::same(9))
                .show(ui, |ui| {
                    ui.set_max_width(PLAYER_GOAL_HINT_MAX_WIDTH);
                    ui.small(egui::RichText::new(player_goal_badge(locale)).color(tone));
                    ui.strong(player_goal_title(step, locale));
                    ui.small(player_goal_detail(step, locale));
                    render_player_guide_progress_lines(ui, locale, progress, step, tone);
                });
        });
}

pub(super) fn render_player_onboarding_card(
    context: &egui::Context,
    onboarding: &mut PlayerOnboardingState,
    step: PlayerGuideStep,
    progress: PlayerGuideProgressSnapshot,
    layout_state: &mut RightPanelLayoutState,
    locale: crate::i18n::UiLocale,
    now_secs: f64,
) {
    if !should_show_player_onboarding_card(onboarding, step) {
        return;
    }

    let tone = player_goal_color(step);
    let motion =
        build_player_card_transition_snapshot(&onboarding.guide_transition, step, now_secs, 1.2);
    let to_u8 = |value: f32| (value.clamp(0.0, 255.0)) as u8;
    let mut primary_clicked = false;
    let mut dismiss_clicked = false;
    egui::Area::new(egui::Id::new("viewer-player-onboarding"))
        .anchor(
            egui::Align2::LEFT_TOP,
            egui::vec2(14.0, 14.0 - motion.slide_px),
        )
        .movable(false)
        .interactable(true)
        .show(context, |ui| {
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgba_unmultiplied(
                    19,
                    26,
                    38,
                    to_u8(236.0 * motion.alpha),
                ))
                .stroke(egui::Stroke::new(
                    1.0 + 0.45 * motion.pulse,
                    egui::Color32::from_rgba_unmultiplied(
                        tone.r(),
                        tone.g(),
                        tone.b(),
                        to_u8((150.0 + 90.0 * motion.pulse) * motion.alpha),
                    ),
                ))
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
                    render_player_guide_progress_lines(ui, locale, progress, step, tone);
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

pub(super) fn render_player_experience_layers(
    context: &egui::Context,
    state: &ViewerState,
    selection: &ViewerSelection,
    layout_state: &mut RightPanelLayoutState,
    module_visibility: &mut crate::right_panel_module_visibility::RightPanelModuleVisibilityState,
    onboarding: &mut PlayerOnboardingState,
    achievements: &mut PlayerAchievementState,
    locale: crate::i18n::UiLocale,
    now_secs: f64,
) {
    render_player_atmosphere(context, now_secs);
    render_player_layout_preset_strip(context, layout_state, module_visibility, locale, now_secs);
    sync_player_achievements(achievements, state, selection, layout_state, now_secs);
    sync_agent_chatter_bubbles(achievements, state, now_secs, locale);
    let guide_step = resolve_player_guide_step(&state.status, layout_state, selection);
    let guide_progress =
        build_player_guide_progress_snapshot(&state.status, layout_state, selection);
    let onboarding_visible = should_show_player_onboarding_card(onboarding, guide_step);
    sync_player_guide_transition(&mut onboarding.guide_transition, guide_step, now_secs);
    render_player_cinematic_intro(context, state, guide_step, locale, now_secs);
    render_player_compact_hud(context, state, selection, guide_step, locale, now_secs);
    render_player_mission_hud(
        context,
        state,
        selection,
        layout_state,
        module_visibility,
        onboarding_visible,
        guide_step,
        guide_progress,
        locale,
        now_secs,
    );
    render_player_achievement_popups(context, achievements, locale, now_secs);
    render_agent_chatter_bubbles(context, achievements, now_secs);
    if should_show_player_goal_hint(onboarding, guide_step, layout_state) {
        render_player_goal_hint(
            context,
            onboarding,
            guide_step,
            guide_progress,
            locale,
            now_secs,
        );
    }
    if onboarding_visible {
        render_player_onboarding_card(
            context,
            onboarding,
            guide_step,
            guide_progress,
            layout_state,
            locale,
            now_secs,
        );
    }
}
