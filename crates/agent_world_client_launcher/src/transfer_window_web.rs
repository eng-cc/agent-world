use super::*;

impl ClientLauncherApp {
    pub(super) fn submit_transfer(&mut self) {
        let message = self.tr(
            "Web 模式暂不支持转账提交",
            "Transfer submit is not supported in web mode yet",
        );
        self.transfer_submit_state = TransferSubmitState::Failed(message.to_string());
        self.append_log(message.to_string());
    }

    pub(super) fn show_transfer_window(&mut self, _ctx: &egui::Context) {
        self.transfer_window_open = false;
    }
}
