use super::*;
use crate::transfer_entry::{submit_transfer_remote, validate_transfer_draft, TransferDraftIssue};

impl ClientLauncherApp {
    pub(super) fn transfer_issue_text(&self, issue: TransferDraftIssue) -> &'static str {
        match (issue, self.ui_language) {
            (TransferDraftIssue::FromAccountRequired, UiLanguage::ZhCn) => "转出账户不能为空",
            (TransferDraftIssue::FromAccountRequired, UiLanguage::EnUs) => {
                "From account cannot be empty"
            }
            (TransferDraftIssue::ToAccountRequired, UiLanguage::ZhCn) => "转入账户不能为空",
            (TransferDraftIssue::ToAccountRequired, UiLanguage::EnUs) => {
                "To account cannot be empty"
            }
            (TransferDraftIssue::SameAccount, UiLanguage::ZhCn) => "转出与转入账户不能相同",
            (TransferDraftIssue::SameAccount, UiLanguage::EnUs) => {
                "From and to accounts cannot be the same"
            }
            (TransferDraftIssue::AmountInvalid, UiLanguage::ZhCn) => "金额必须是大于 0 的整数",
            (TransferDraftIssue::AmountInvalid, UiLanguage::EnUs) => {
                "Amount must be a positive integer"
            }
            (TransferDraftIssue::NonceInvalid, UiLanguage::ZhCn) => "Nonce 必须是大于 0 的整数",
            (TransferDraftIssue::NonceInvalid, UiLanguage::EnUs) => {
                "Nonce must be a positive integer"
            }
        }
    }

    pub(super) fn submit_transfer(&mut self) {
        let issues = validate_transfer_draft(&self.transfer_draft);
        if !issues.is_empty() {
            for issue in issues {
                self.append_log(format!(
                    "transfer validation failed: {}",
                    self.transfer_issue_text(issue)
                ));
            }
            self.transfer_submit_state = TransferSubmitState::Failed(
                self.tr(
                    "转账提交失败：请先修复输入项",
                    "Transfer submit failed: fix required input fields first",
                )
                .to_string(),
            );
            return;
        }
        if !self.config.chain_enabled {
            let message = self
                .tr(
                    "转账提交失败：链运行时未启用",
                    "Transfer submit failed: chain runtime is disabled",
                )
                .to_string();
            self.append_log(message.clone());
            self.transfer_submit_state = TransferSubmitState::Failed(message);
            return;
        }

        match submit_transfer_remote(&self.transfer_draft, self.config.chain_status_bind.as_str()) {
            Ok(response) => {
                if response.ok {
                    let action_id_text = response
                        .action_id
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "n/a".to_string());
                    let message = format!(
                        "{}: action_id={}",
                        self.tr("转账请求已提交", "Transfer request submitted"),
                        action_id_text
                    );
                    self.append_log(message.clone());
                    self.transfer_submit_state = TransferSubmitState::Success(message);
                } else {
                    let error_text = response
                        .error
                        .unwrap_or_else(|| self.tr("未知错误", "Unknown error").to_string());
                    let error_code = response
                        .error_code
                        .map(|code| format!(" ({code})"))
                        .unwrap_or_default();
                    let message = format!(
                        "{}{}: {}",
                        self.tr("转账提交被拒绝", "Transfer submit rejected"),
                        error_code,
                        error_text
                    );
                    self.append_log(message.clone());
                    self.transfer_submit_state = TransferSubmitState::Failed(message);
                }
            }
            Err(err) => {
                let message = format!(
                    "{}: {err}",
                    self.tr("转账提交失败", "Transfer submit failed")
                );
                self.append_log(message.clone());
                self.transfer_submit_state = TransferSubmitState::Failed(message);
            }
        }
    }

    pub(super) fn show_transfer_window(&mut self, ctx: &egui::Context) {
        if !self.transfer_window_open {
            return;
        }

        let title = self.tr("链上转账", "On-Chain Transfer").to_string();
        let mut window_open = self.transfer_window_open;
        egui::Window::new(title)
            .open(&mut window_open)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(self.tr("转出账户", "From Account"));
                    ui.text_edit_singleline(&mut self.transfer_draft.from_account_id);
                    ui.label(self.tr("转入账户", "To Account"));
                    ui.text_edit_singleline(&mut self.transfer_draft.to_account_id);
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(self.tr("金额", "Amount"));
                    ui.text_edit_singleline(&mut self.transfer_draft.amount);
                    ui.label(self.tr("Nonce", "Nonce"));
                    ui.text_edit_singleline(&mut self.transfer_draft.nonce);
                    if ui.button(self.tr("提交转账", "Submit Transfer")).clicked() {
                        self.submit_transfer();
                    }
                });

                let issues = validate_transfer_draft(&self.transfer_draft);
                if !issues.is_empty() {
                    ui.small(
                        egui::RichText::new(self.tr(
                            "提交前请完善必填项：",
                            "Please complete required fields before submit:",
                        ))
                        .color(egui::Color32::from_rgb(196, 84, 84)),
                    );
                    for issue in issues {
                        ui.small(
                            egui::RichText::new(format!("- {}", self.transfer_issue_text(issue)))
                                .color(egui::Color32::from_rgb(196, 84, 84)),
                        );
                    }
                }
                match &self.transfer_submit_state {
                    TransferSubmitState::Success(message) => {
                        ui.small(
                            egui::RichText::new(message.as_str())
                                .color(egui::Color32::from_rgb(62, 152, 92)),
                        );
                    }
                    TransferSubmitState::Failed(message) => {
                        ui.small(
                            egui::RichText::new(message.as_str())
                                .color(egui::Color32::from_rgb(196, 84, 84)),
                        );
                    }
                    TransferSubmitState::None => {}
                }
            });

        self.transfer_window_open = window_open;
    }
}
