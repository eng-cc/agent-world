use super::*;

impl ClientLauncherApp {
    pub(super) fn tr<'a>(&self, zh: &'a str, en: &'a str) -> &'a str {
        match self.ui_language {
            UiLanguage::ZhCn => zh,
            UiLanguage::EnUs => en,
        }
    }

    pub(super) fn glossary_term_text(&self, term: GlossaryTerm) -> &'static str {
        match term {
            GlossaryTerm::Nonce => "nonce",
            GlossaryTerm::Slot => "slot",
            GlossaryTerm::Mempool => "mempool",
            GlossaryTerm::ActionId => "action_id",
        }
    }

    pub(super) fn glossary_term_definition(&self, term: GlossaryTerm) -> &'static str {
        match (term, self.ui_language) {
            (GlossaryTerm::Nonce, UiLanguage::ZhCn) => {
                "每个账户的递增序号，用于防重放；通常使用 next_nonce_hint。"
            }
            (GlossaryTerm::Nonce, UiLanguage::EnUs) => {
                "Per-account increasing sequence to prevent replay; usually use next_nonce_hint."
            }
            (GlossaryTerm::Slot, UiLanguage::ZhCn) => {
                "链出块时间片编号；多个 tick 组成一个 slot，用于排序区块时间。"
            }
            (GlossaryTerm::Slot, UiLanguage::EnUs) => {
                "Block time window index; multiple ticks form one slot for chain ordering."
            }
            (GlossaryTerm::Mempool, UiLanguage::ZhCn) => {
                "待打包交易池，包含 accepted/pending 状态的交易。"
            }
            (GlossaryTerm::Mempool, UiLanguage::EnUs) => {
                "Queue of transactions waiting to be packed, including accepted/pending states."
            }
            (GlossaryTerm::ActionId, UiLanguage::ZhCn) => {
                "链内动作编号，可用于精确追踪单笔转账状态与查询。"
            }
            (GlossaryTerm::ActionId, UiLanguage::EnUs) => {
                "On-chain action identifier for tracking one transfer lifecycle and queries."
            }
        }
    }

    pub(super) fn render_glossary_term_chip(&self, ui: &mut egui::Ui, term: GlossaryTerm) {
        ui.label(
            egui::RichText::new(self.glossary_term_text(term))
                .underline()
                .color(egui::Color32::from_rgb(74, 116, 168)),
        )
        .on_hover_text(self.glossary_term_definition(term));
    }

    pub(super) fn append_log<S: Into<String>>(&mut self, line: S) {
        self.logs.push_back(line.into());
        while self.logs.len() > MAX_LOG_LINES {
            self.logs.pop_front();
        }
    }

    pub(super) fn web_request_inflight_for(&self, domain: WebRequestDomain) -> bool {
        match domain {
            WebRequestDomain::StatePoll => self.web_request_inflight.state_poll,
            WebRequestDomain::ControlAction => self.web_request_inflight.control_action,
            WebRequestDomain::FeedbackSubmit => self.web_request_inflight.feedback_submit,
            WebRequestDomain::TransferSubmit => self.web_request_inflight.transfer_submit,
            WebRequestDomain::TransferQuery => self.web_request_inflight.transfer_query,
            WebRequestDomain::ExplorerQuery => self.web_request_inflight.explorer_query,
        }
    }

    pub(super) fn set_web_request_inflight(&mut self, domain: WebRequestDomain, inflight: bool) {
        match domain {
            WebRequestDomain::StatePoll => self.web_request_inflight.state_poll = inflight,
            WebRequestDomain::ControlAction => self.web_request_inflight.control_action = inflight,
            WebRequestDomain::FeedbackSubmit => {
                self.web_request_inflight.feedback_submit = inflight;
            }
            WebRequestDomain::TransferSubmit => {
                self.web_request_inflight.transfer_submit = inflight;
            }
            WebRequestDomain::TransferQuery => self.web_request_inflight.transfer_query = inflight,
            WebRequestDomain::ExplorerQuery => self.web_request_inflight.explorer_query = inflight,
        }
    }

    #[cfg(test)]
    pub(super) fn any_web_request_inflight(&self) -> bool {
        self.web_request_inflight.any()
    }

    pub(super) fn any_transfer_request_inflight(&self) -> bool {
        self.web_request_inflight.transfer_any()
    }

    #[cfg(target_arch = "wasm32")]
    pub(super) fn apply_web_feedback_submit_result(
        &mut self,
        result: Result<WebFeedbackSubmitResponse, String>,
    ) {
        match result {
            Ok(response) => {
                if response.ok {
                    let feedback_id = response.feedback_id.unwrap_or_else(|| "n/a".to_string());
                    let event_id = response.event_id.unwrap_or_else(|| "n/a".to_string());
                    let message = format!(
                        "{}: feedback_id={feedback_id}, event_id={event_id}",
                        self.tr(
                            "反馈已提交到分布式网络",
                            "Feedback submitted to distributed network"
                        )
                    );
                    self.append_log(message.clone());
                    self.feedback_submit_state = FeedbackSubmitState::Success(message);
                } else {
                    let error_text = response
                        .error
                        .unwrap_or_else(|| self.tr("未知错误", "Unknown error").to_string());
                    let message = format!(
                        "{}: {error_text}",
                        self.tr("反馈提交被拒绝", "Feedback submit rejected")
                    );
                    self.append_log(message.clone());
                    self.feedback_submit_state = FeedbackSubmitState::Failed(message);
                }
            }
            Err(err) => {
                let message = format!(
                    "{}: {err}",
                    self.tr("反馈提交失败", "Feedback submit failed")
                );
                self.append_log(message.clone());
                self.feedback_submit_state = FeedbackSubmitState::Failed(message);
            }
        }
    }

    pub(super) fn feedback_unavailable_hint(&self) -> Option<String> {
        if self.is_feedback_available() {
            return None;
        }
        let message = match (&self.chain_runtime_status, self.ui_language) {
            (ChainRuntimeStatus::Disabled, UiLanguage::ZhCn) => {
                "反馈/转账/浏览器功能已禁用：区块链功能关闭".to_string()
            }
            (ChainRuntimeStatus::Disabled, UiLanguage::EnUs) => {
                "Feedback/Transfer/Explorer are disabled because blockchain is disabled".to_string()
            }
            (ChainRuntimeStatus::NotStarted, UiLanguage::ZhCn) => {
                "反馈/转账/浏览器功能暂不可用：区块链未启动".to_string()
            }
            (ChainRuntimeStatus::NotStarted, UiLanguage::EnUs) => {
                "Feedback/Transfer/Explorer are unavailable because blockchain is not started"
                    .to_string()
            }
            (ChainRuntimeStatus::Starting, UiLanguage::ZhCn) => {
                "反馈/转账/浏览器功能暂不可用：区块链启动中".to_string()
            }
            (ChainRuntimeStatus::Starting, UiLanguage::EnUs) => {
                "Feedback/Transfer/Explorer are unavailable while blockchain is starting"
                    .to_string()
            }
            (ChainRuntimeStatus::StaleExecutionWorld(detail), UiLanguage::ZhCn) => {
                format!("反馈/转账/浏览器功能暂不可用：检测到旧执行世界冲突（{detail}）")
            }
            (ChainRuntimeStatus::StaleExecutionWorld(detail), UiLanguage::EnUs) => {
                format!(
                    "Feedback/Transfer/Explorer are unavailable: stale execution world detected ({detail})"
                )
            }
            (ChainRuntimeStatus::Unreachable(detail), UiLanguage::ZhCn) => {
                format!("反馈/转账/浏览器功能暂不可用：区块链不可达（{detail}）")
            }
            (ChainRuntimeStatus::Unreachable(detail), UiLanguage::EnUs) => {
                format!(
                    "Feedback/Transfer/Explorer are unavailable: blockchain unreachable ({detail})"
                )
            }
            (ChainRuntimeStatus::ConfigError(detail), UiLanguage::ZhCn) => {
                format!("反馈/转账/浏览器功能暂不可用：区块链配置错误（{detail}）")
            }
            (ChainRuntimeStatus::ConfigError(detail), UiLanguage::EnUs) => {
                format!("Feedback/Transfer/Explorer are unavailable: blockchain config error ({detail})")
            }
            (ChainRuntimeStatus::Ready, UiLanguage::ZhCn) => {
                "反馈/转账/浏览器功能暂不可用：区块链功能关闭".to_string()
            }
            (ChainRuntimeStatus::Ready, UiLanguage::EnUs) => {
                "Feedback/Transfer/Explorer are unavailable: blockchain is disabled".to_string()
            }
        };
        Some(message)
    }
}
