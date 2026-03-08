use super::*;

impl ClientLauncherApp {
    pub(super) fn should_show_onboarding_reminder(&self) -> bool {
        !self.is_expert_mode() && !self.onboarding_state.completed && !self.onboarding_state.open
    }

    pub(super) fn dismiss_onboarding_with_reminder(&mut self) {
        self.onboarding_state.completed = false;
        self.onboarding_state.open = false;
        self.onboarding_state.dismissed = true;
        self.ux_state.onboarding_completed = false;
        self.ux_state.onboarding_dismissed = true;
        self.persist_ux_state_or_log(
            "保存引导状态失败（已降级为会话内状态）",
            "Persist onboarding state failed (fallback to session-only)",
        );
        self.increment_guidance_counter(super::self_guided::GuidanceCounter::OnboardingSkipped);
        self.append_log(self.tr(
            "已暂时关闭首次引导，主界面将持续显示继续提醒。",
            "Onboarding dismissed for now; a persistent reminder will remain in the main panel.",
        ));
    }

    pub(super) fn render_onboarding_reminder_banner(&mut self, ui: &mut egui::Ui) {
        if !self.should_show_onboarding_reminder() {
            return;
        }

        ui.group(|ui| {
            ui.horizontal_wrapped(|ui| {
                ui.small(
                    egui::RichText::new(self.tr(
                        "未完成引导：点击“继续引导”可按步骤完成首次启动。",
                        "Onboarding not finished: click Continue to complete first startup.",
                    ))
                    .color(egui::Color32::from_rgb(74, 116, 168)),
                );

                if ui.button(self.tr("继续引导", "Continue Onboarding")).clicked() {
                    self.record_guided_quick_action_click();
                    self.open_onboarding_manual();
                }
                if ui.button(self.tr("重置引导", "Reset Guide")).clicked() {
                    self.record_guided_quick_action_click();
                    self.reset_onboarding();
                }
            });
        });
    }
}
