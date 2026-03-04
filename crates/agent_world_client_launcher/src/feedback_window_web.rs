use super::*;

impl ClientLauncherApp {
    pub(super) fn submit_feedback(&mut self) {
        let message = self.tr(
            "Web 模式暂不支持反馈提交",
            "Feedback submit is not supported in web mode yet",
        );
        self.feedback_submit_state = FeedbackSubmitState::Failed(message.to_string());
        self.append_log(message.to_string());
    }

    pub(super) fn show_feedback_window(&mut self, _ctx: &egui::Context) {
        self.feedback_window_open = false;
    }
}
