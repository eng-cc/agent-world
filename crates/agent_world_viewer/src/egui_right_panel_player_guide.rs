use bevy_egui::egui;

use crate::{RightPanelLayoutState, ViewerSelection};

use super::egui_right_panel_player_experience::PlayerGuideStep;

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
