use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WebFeedbackDraftIssue {
    TitleRequired,
    DescriptionRequired,
}

fn validate_web_feedback_draft(draft: &FeedbackDraft) -> Vec<WebFeedbackDraftIssue> {
    let mut issues = Vec::new();
    if draft.title.trim().is_empty() {
        issues.push(WebFeedbackDraftIssue::TitleRequired);
    }
    if draft.description.trim().is_empty() {
        issues.push(WebFeedbackDraftIssue::DescriptionRequired);
    }
    issues
}

impl ClientLauncherApp {
    fn web_feedback_kind_label(&self, kind: FeedbackKind) -> &'static str {
        match (kind, self.ui_language) {
            (FeedbackKind::Bug, UiLanguage::ZhCn) => "Bug",
            (FeedbackKind::Bug, UiLanguage::EnUs) => "Bug",
            (FeedbackKind::Suggestion, UiLanguage::ZhCn) => "建议",
            (FeedbackKind::Suggestion, UiLanguage::EnUs) => "Suggestion",
        }
    }

    fn web_feedback_issue_text(&self, issue: WebFeedbackDraftIssue) -> &'static str {
        match (issue, self.ui_language) {
            (WebFeedbackDraftIssue::TitleRequired, UiLanguage::ZhCn) => "反馈标题不能为空",
            (WebFeedbackDraftIssue::TitleRequired, UiLanguage::EnUs) => {
                "Feedback title cannot be empty"
            }
            (WebFeedbackDraftIssue::DescriptionRequired, UiLanguage::ZhCn) => "反馈描述不能为空",
            (WebFeedbackDraftIssue::DescriptionRequired, UiLanguage::EnUs) => {
                "Feedback description cannot be empty"
            }
        }
    }

    pub(super) fn submit_feedback(&mut self) {
        if !self.is_feedback_available() {
            let message = self
                .tr(
                    "反馈提交失败：区块链未就绪",
                    "Feedback submit failed: blockchain is not ready",
                )
                .to_string();
            self.append_log(message.clone());
            self.feedback_submit_state = FeedbackSubmitState::Failed(message);
            return;
        }

        let issues = validate_web_feedback_draft(&self.feedback_draft);
        if !issues.is_empty() {
            for issue in issues {
                self.append_log(format!(
                    "feedback validation failed: {}",
                    self.web_feedback_issue_text(issue)
                ));
            }
            self.feedback_submit_state = FeedbackSubmitState::Failed(
                self.tr(
                    "反馈提交失败：请先修复表单必填项",
                    "Feedback submit failed: fix required form fields first",
                )
                .to_string(),
            );
            return;
        }

        if self.web_request_inflight {
            let message = self
                .tr(
                    "反馈提交失败：已有请求处理中",
                    "Feedback submit failed: another request is in flight",
                )
                .to_string();
            self.append_log(message.clone());
            self.feedback_submit_state = FeedbackSubmitState::Failed(message);
            return;
        }

        let request = WebFeedbackSubmitRequest {
            category: self.feedback_draft.kind.slug().to_string(),
            title: self.feedback_draft.title.trim().to_string(),
            description: self.feedback_draft.description.trim().to_string(),
            platform: "client_launcher_web".to_string(),
            game_version: "unknown".to_string(),
        };
        self.feedback_submit_state = FeedbackSubmitState::None;
        self.request_web_chain_feedback(request);
    }

    pub(super) fn show_feedback_window(&mut self, ctx: &egui::Context) {
        if !self.feedback_window_open {
            return;
        }

        let issues = validate_web_feedback_draft(&self.feedback_draft);
        let submit_enabled =
            issues.is_empty() && self.is_feedback_available() && !self.web_request_inflight;

        let title = self
            .tr("反馈（Bug / 建议）", "Feedback (Bug / Suggestion)")
            .to_string();
        let kind_bug_label = self.web_feedback_kind_label(FeedbackKind::Bug).to_string();
        let kind_suggestion_label = self
            .web_feedback_kind_label(FeedbackKind::Suggestion)
            .to_string();
        let kind_selected_label = self
            .web_feedback_kind_label(self.feedback_draft.kind)
            .to_string();
        let description_hint = self
            .tr(
                "请写复现步骤、预期结果、实际结果",
                "Describe steps, expected result, and actual result",
            )
            .to_string();
        let mut window_open = self.feedback_window_open;
        egui::Window::new(title)
            .open(&mut window_open)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(self.tr("类型", "Type"));
                    egui::ComboBox::from_id_salt("feedback_kind_window_web")
                        .selected_text(kind_selected_label.as_str())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.feedback_draft.kind,
                                FeedbackKind::Bug,
                                kind_bug_label.as_str(),
                            );
                            ui.selectable_value(
                                &mut self.feedback_draft.kind,
                                FeedbackKind::Suggestion,
                                kind_suggestion_label.as_str(),
                            );
                        });
                    ui.label(self.tr("标题", "Title"));
                    ui.text_edit_singleline(&mut self.feedback_draft.title);
                });
                ui.label(self.tr("描述", "Description"));
                ui.add(
                    egui::TextEdit::multiline(&mut self.feedback_draft.description)
                        .desired_rows(4)
                        .hint_text(description_hint.as_str()),
                );
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .add_enabled(
                            submit_enabled,
                            egui::Button::new(self.tr("提交反馈", "Submit Feedback")),
                        )
                        .clicked()
                    {
                        self.submit_feedback();
                    }
                });

                if self.web_request_inflight {
                    ui.small(
                        egui::RichText::new(
                            self.tr("请求处理中，请稍候…", "Request in flight, please wait..."),
                        )
                        .color(egui::Color32::from_rgb(201, 146, 44)),
                    );
                }
                if !self.is_feedback_available() {
                    ui.small(
                        egui::RichText::new(self.tr(
                            "区块链未就绪时不可提交反馈",
                            "Feedback submit is unavailable until blockchain is ready",
                        ))
                        .color(egui::Color32::from_rgb(196, 84, 84)),
                    );
                }
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
                            egui::RichText::new(format!(
                                "- {}",
                                self.web_feedback_issue_text(issue)
                            ))
                            .color(egui::Color32::from_rgb(196, 84, 84)),
                        );
                    }
                }
                match &self.feedback_submit_state {
                    FeedbackSubmitState::Success(message) => {
                        ui.small(
                            egui::RichText::new(message.as_str())
                                .color(egui::Color32::from_rgb(62, 152, 92)),
                        );
                    }
                    FeedbackSubmitState::Failed(message) => {
                        ui.small(
                            egui::RichText::new(message.as_str())
                                .color(egui::Color32::from_rgb(196, 84, 84)),
                        );
                    }
                    FeedbackSubmitState::None => {}
                }
            });

        self.feedback_window_open = window_open;
    }
}
