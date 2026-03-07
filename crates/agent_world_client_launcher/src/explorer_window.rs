use super::*;

const EXPLORER_POLL_INTERVAL_MS: u64 = 1_000;
const EXPLORER_DEFAULT_LIMIT: usize = 50;

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WebExplorerOverviewResponse {
    pub(super) ok: bool,
    pub(super) observed_at_unix_ms: i64,
    pub(super) node_id: String,
    pub(super) world_id: String,
    pub(super) latest_height: u64,
    pub(super) committed_height: u64,
    pub(super) network_committed_height: u64,
    pub(super) last_block_hash: Option<String>,
    pub(super) last_execution_block_hash: Option<String>,
    pub(super) tracked_records: usize,
    pub(super) transfer_total: usize,
    pub(super) transfer_accepted: usize,
    pub(super) transfer_pending: usize,
    pub(super) transfer_confirmed: usize,
    pub(super) transfer_failed: usize,
    pub(super) transfer_timeout: usize,
    pub(super) error_code: Option<String>,
    pub(super) error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WebExplorerTransactionsResponse {
    pub(super) ok: bool,
    pub(super) observed_at_unix_ms: i64,
    pub(super) account_filter: Option<String>,
    pub(super) status_filter: Option<transfer_window::WebTransferLifecycleStatus>,
    pub(super) action_filter: Option<u64>,
    pub(super) limit: usize,
    pub(super) total: usize,
    #[serde(default)]
    pub(super) items: Vec<transfer_window::WebTransferHistoryItem>,
    pub(super) error_code: Option<String>,
    pub(super) error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WebExplorerTransactionResponse {
    pub(super) ok: bool,
    pub(super) observed_at_unix_ms: i64,
    pub(super) action_id: u64,
    pub(super) status: Option<transfer_window::WebTransferHistoryItem>,
    pub(super) error_code: Option<String>,
    pub(super) error: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) enum ExplorerQueryResponse {
    Overview(WebExplorerOverviewResponse),
    Transactions(WebExplorerTransactionsResponse),
    Transaction(WebExplorerTransactionResponse),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ExplorerStatusFilter {
    All,
    Accepted,
    Pending,
    Confirmed,
    Failed,
    Timeout,
}

impl Default for ExplorerStatusFilter {
    fn default() -> Self {
        Self::All
    }
}

impl ExplorerStatusFilter {
    fn query_value(self) -> Option<&'static str> {
        match self {
            Self::All => None,
            Self::Accepted => Some("accepted"),
            Self::Pending => Some("pending"),
            Self::Confirmed => Some("confirmed"),
            Self::Failed => Some("failed"),
            Self::Timeout => Some("timeout"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(super) struct ExplorerPanelState {
    pub(super) overview: Option<WebExplorerOverviewResponse>,
    pub(super) transactions: Vec<transfer_window::WebTransferHistoryItem>,
    pub(super) selected_transaction: Option<transfer_window::WebTransferHistoryItem>,
    pub(super) account_filter: String,
    pub(super) action_input: String,
    pub(super) status_filter: ExplorerStatusFilter,
    pub(super) pending_overview_refresh: bool,
    pub(super) pending_transactions_refresh: bool,
    pub(super) pending_transaction_refresh: bool,
    pub(super) pending_action_id: Option<u64>,
    pub(super) last_poll_at: Option<Instant>,
}

fn parse_positive_action_id(raw: &str) -> Option<u64> {
    raw.trim().parse::<u64>().ok().filter(|value| *value > 0)
}

impl ClientLauncherApp {
    fn explorer_status_filter_text(&self, filter: ExplorerStatusFilter) -> &'static str {
        match (filter, self.ui_language) {
            (ExplorerStatusFilter::All, UiLanguage::ZhCn) => "全部",
            (ExplorerStatusFilter::All, UiLanguage::EnUs) => "All",
            (ExplorerStatusFilter::Accepted, UiLanguage::ZhCn) => "已受理",
            (ExplorerStatusFilter::Accepted, UiLanguage::EnUs) => "Accepted",
            (ExplorerStatusFilter::Pending, UiLanguage::ZhCn) => "待确认",
            (ExplorerStatusFilter::Pending, UiLanguage::EnUs) => "Pending",
            (ExplorerStatusFilter::Confirmed, UiLanguage::ZhCn) => "已确认",
            (ExplorerStatusFilter::Confirmed, UiLanguage::EnUs) => "Confirmed",
            (ExplorerStatusFilter::Failed, UiLanguage::ZhCn) => "失败",
            (ExplorerStatusFilter::Failed, UiLanguage::EnUs) => "Failed",
            (ExplorerStatusFilter::Timeout, UiLanguage::ZhCn) => "超时",
            (ExplorerStatusFilter::Timeout, UiLanguage::EnUs) => "Timeout",
        }
    }

    fn explorer_lifecycle_text(
        &self,
        status: transfer_window::WebTransferLifecycleStatus,
    ) -> &'static str {
        match (status, self.ui_language) {
            (transfer_window::WebTransferLifecycleStatus::Accepted, UiLanguage::ZhCn) => "已受理",
            (transfer_window::WebTransferLifecycleStatus::Accepted, UiLanguage::EnUs) => "Accepted",
            (transfer_window::WebTransferLifecycleStatus::Pending, UiLanguage::ZhCn) => "待确认",
            (transfer_window::WebTransferLifecycleStatus::Pending, UiLanguage::EnUs) => "Pending",
            (transfer_window::WebTransferLifecycleStatus::Confirmed, UiLanguage::ZhCn) => "已确认",
            (transfer_window::WebTransferLifecycleStatus::Confirmed, UiLanguage::EnUs) => {
                "Confirmed"
            }
            (transfer_window::WebTransferLifecycleStatus::Failed, UiLanguage::ZhCn) => "失败",
            (transfer_window::WebTransferLifecycleStatus::Failed, UiLanguage::EnUs) => "Failed",
            (transfer_window::WebTransferLifecycleStatus::Timeout, UiLanguage::ZhCn) => "超时",
            (transfer_window::WebTransferLifecycleStatus::Timeout, UiLanguage::EnUs) => "Timeout",
        }
    }

    fn maybe_request_explorer_panel_data(&mut self) {
        if !self.explorer_window_open || self.web_request_inflight {
            return;
        }

        let now = Instant::now();
        let should_poll = self.explorer_panel_state.last_poll_at.is_none_or(|last| {
            now.duration_since(last) >= Duration::from_millis(EXPLORER_POLL_INTERVAL_MS)
        });

        if self.explorer_panel_state.pending_overview_refresh || should_poll {
            self.explorer_panel_state.pending_overview_refresh = false;
            if should_poll {
                self.explorer_panel_state.last_poll_at = Some(now);
                self.explorer_panel_state.pending_transactions_refresh = true;
            }
            self.request_web_chain_explorer_overview();
            return;
        }

        if self.explorer_panel_state.pending_transactions_refresh {
            self.explorer_panel_state.pending_transactions_refresh = false;
            self.request_web_chain_explorer_transactions(
                self.explorer_panel_state.account_filter.clone(),
                self.explorer_panel_state
                    .status_filter
                    .query_value()
                    .map(str::to_string),
                EXPLORER_DEFAULT_LIMIT,
            );
            return;
        }

        if self.explorer_panel_state.pending_transaction_refresh {
            if let Some(action_id) = self.explorer_panel_state.pending_action_id {
                self.explorer_panel_state.pending_transaction_refresh = false;
                self.request_web_chain_explorer_transaction(action_id);
            }
        }
    }

    pub(super) fn apply_web_explorer_query_result(
        &mut self,
        result: Result<ExplorerQueryResponse, String>,
    ) {
        match result {
            Ok(ExplorerQueryResponse::Overview(response)) => {
                if response.ok {
                    self.explorer_panel_state.overview = Some(response);
                } else {
                    let error_text = response
                        .error
                        .unwrap_or_else(|| self.tr("未知错误", "Unknown error").to_string());
                    let error_code = response
                        .error_code
                        .map(|code| format!(" ({code})"))
                        .unwrap_or_default();
                    self.append_log(format!(
                        "{}{}: {error_text}",
                        self.tr("浏览器概览查询失败", "Explorer overview query failed"),
                        error_code
                    ));
                }
            }
            Ok(ExplorerQueryResponse::Transactions(response)) => {
                if response.ok {
                    let selected_action_id = self
                        .explorer_panel_state
                        .selected_transaction
                        .as_ref()
                        .map(|item| item.action_id);
                    self.explorer_panel_state.transactions = response.items;
                    if let Some(selected_action_id) = selected_action_id {
                        self.explorer_panel_state.selected_transaction = self
                            .explorer_panel_state
                            .transactions
                            .iter()
                            .find(|item| item.action_id == selected_action_id)
                            .cloned();
                    }
                } else {
                    let error_text = response
                        .error
                        .unwrap_or_else(|| self.tr("未知错误", "Unknown error").to_string());
                    let error_code = response
                        .error_code
                        .map(|code| format!(" ({code})"))
                        .unwrap_or_default();
                    self.append_log(format!(
                        "{}{}: {error_text}",
                        self.tr(
                            "浏览器交易列表查询失败",
                            "Explorer transactions query failed"
                        ),
                        error_code
                    ));
                }
            }
            Ok(ExplorerQueryResponse::Transaction(response)) => {
                if response.ok {
                    self.explorer_panel_state.pending_action_id = Some(response.action_id);
                    self.explorer_panel_state.selected_transaction = response.status;
                } else {
                    let error_text = response
                        .error
                        .unwrap_or_else(|| self.tr("未知错误", "Unknown error").to_string());
                    let error_code = response
                        .error_code
                        .map(|code| format!(" ({code})"))
                        .unwrap_or_default();
                    self.append_log(format!(
                        "{}{}: {error_text}",
                        self.tr(
                            "浏览器交易详情查询失败",
                            "Explorer transaction query failed"
                        ),
                        error_code
                    ));
                }
            }
            Err(err) => {
                self.append_log(format!(
                    "{}: {err}",
                    self.tr("浏览器查询失败", "Explorer query failed")
                ));
            }
        }
    }

    pub(super) fn show_explorer_window(&mut self, ctx: &egui::Context) {
        if !self.explorer_window_open {
            return;
        }

        if self.explorer_panel_state.overview.is_none() {
            self.explorer_panel_state.pending_overview_refresh = true;
        }
        if self.explorer_panel_state.transactions.is_empty() {
            self.explorer_panel_state.pending_transactions_refresh = true;
        }
        self.maybe_request_explorer_panel_data();

        let title = self.tr("区块链浏览器", "Blockchain Explorer").to_string();
        let mut window_open = self.explorer_window_open;
        egui::Window::new(title)
            .open(&mut window_open)
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .button(self.tr("刷新概览与列表", "Refresh Overview/Transactions"))
                        .clicked()
                    {
                        self.explorer_panel_state.pending_overview_refresh = true;
                        self.explorer_panel_state.pending_transactions_refresh = true;
                        self.explorer_panel_state.last_poll_at = Some(Instant::now());
                    }
                    if self.web_request_inflight {
                        ui.small(
                            egui::RichText::new(
                                self.tr("请求处理中，请稍候…", "Request in flight, please wait..."),
                            )
                            .color(egui::Color32::from_rgb(201, 146, 44)),
                        );
                    }
                });

                if !self.is_feedback_available() {
                    ui.small(
                        egui::RichText::new(self.tr(
                            "区块链未就绪，浏览器查询不可用",
                            "Blockchain is not ready; explorer queries are unavailable",
                        ))
                        .color(egui::Color32::from_rgb(196, 84, 84)),
                    );
                }

                ui.separator();
                ui.label(self.tr("链概览", "Overview"));
                if let Some(overview) = self.explorer_panel_state.overview.as_ref() {
                    ui.small(format!(
                        "node={} | world={} | observed_at={}",
                        overview.node_id, overview.world_id, overview.observed_at_unix_ms
                    ));
                    ui.small(format!(
                        "height latest={} committed={} network_committed={}",
                        overview.latest_height,
                        overview.committed_height,
                        overview.network_committed_height
                    ));
                    ui.small(format!(
                        "last_block_hash={} | last_execution_block_hash={}",
                        overview.last_block_hash.as_deref().unwrap_or("n/a"),
                        overview
                            .last_execution_block_hash
                            .as_deref()
                            .unwrap_or("n/a")
                    ));
                    ui.small(format!(
                        "tracked={} | total={} | accepted={} | pending={} | confirmed={} | failed={} | timeout={}",
                        overview.tracked_records,
                        overview.transfer_total,
                        overview.transfer_accepted,
                        overview.transfer_pending,
                        overview.transfer_confirmed,
                        overview.transfer_failed,
                        overview.transfer_timeout,
                    ));
                } else {
                    ui.small(self.tr("暂无概览数据", "No overview data"));
                }

                ui.separator();
                ui.label(self.tr("交易过滤", "Transaction Filters"));
                ui.horizontal_wrapped(|ui| {
                    ui.label(self.tr("账户", "Account"));
                    ui.text_edit_singleline(&mut self.explorer_panel_state.account_filter);
                    ui.label(self.tr("状态", "Status"));
                    let all_label = self.explorer_status_filter_text(ExplorerStatusFilter::All);
                    let accepted_label =
                        self.explorer_status_filter_text(ExplorerStatusFilter::Accepted);
                    let pending_label =
                        self.explorer_status_filter_text(ExplorerStatusFilter::Pending);
                    let confirmed_label =
                        self.explorer_status_filter_text(ExplorerStatusFilter::Confirmed);
                    let failed_label =
                        self.explorer_status_filter_text(ExplorerStatusFilter::Failed);
                    let timeout_label =
                        self.explorer_status_filter_text(ExplorerStatusFilter::Timeout);
                    egui::ComboBox::from_id_salt("explorer_status_filter")
                        .selected_text(self.explorer_status_filter_text(
                            self.explorer_panel_state.status_filter,
                        ))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.explorer_panel_state.status_filter,
                                ExplorerStatusFilter::All,
                                all_label,
                            );
                            ui.selectable_value(
                                &mut self.explorer_panel_state.status_filter,
                                ExplorerStatusFilter::Accepted,
                                accepted_label,
                            );
                            ui.selectable_value(
                                &mut self.explorer_panel_state.status_filter,
                                ExplorerStatusFilter::Pending,
                                pending_label,
                            );
                            ui.selectable_value(
                                &mut self.explorer_panel_state.status_filter,
                                ExplorerStatusFilter::Confirmed,
                                confirmed_label,
                            );
                            ui.selectable_value(
                                &mut self.explorer_panel_state.status_filter,
                                ExplorerStatusFilter::Failed,
                                failed_label,
                            );
                            ui.selectable_value(
                                &mut self.explorer_panel_state.status_filter,
                                ExplorerStatusFilter::Timeout,
                                timeout_label,
                            );
                        });
                    if ui
                        .button(self.tr("应用过滤", "Apply Filter"))
                        .clicked()
                    {
                        self.explorer_panel_state.pending_transactions_refresh = true;
                    }
                });

                ui.horizontal_wrapped(|ui| {
                    ui.label("action_id");
                    ui.text_edit_singleline(&mut self.explorer_panel_state.action_input);
                    if ui
                        .button(self.tr("查询详情", "Query Detail"))
                        .clicked()
                    {
                        if let Some(action_id) =
                            parse_positive_action_id(self.explorer_panel_state.action_input.as_str())
                        {
                            self.explorer_panel_state.pending_action_id = Some(action_id);
                            self.explorer_panel_state.pending_transaction_refresh = true;
                        } else {
                            self.append_log(self.tr(
                                "浏览器查询失败：action_id 必须是正整数",
                                "Explorer query failed: action_id must be a positive integer",
                            ));
                        }
                    }
                });

                ui.separator();
                ui.label(self.tr("交易列表", "Transactions"));
                let mut clicked_action_id = None;
                egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                    for item in &self.explorer_panel_state.transactions {
                        let line = format!(
                            "#{:>4} | {} | {} -> {} | amount={} | nonce={} | submitted_at={}",
                            item.action_id,
                            self.explorer_lifecycle_text(item.status),
                            item.from_account_id,
                            item.to_account_id,
                            item.amount,
                            item.nonce,
                            item.submitted_at_unix_ms,
                        );
                        if ui.selectable_label(false, line).clicked() {
                            clicked_action_id = Some(item.action_id);
                        }
                    }
                    if self.explorer_panel_state.transactions.is_empty() {
                        ui.small(self.tr("暂无交易记录", "No transactions"));
                    }
                });
                if let Some(action_id) = clicked_action_id {
                    self.explorer_panel_state.action_input = action_id.to_string();
                    self.explorer_panel_state.pending_action_id = Some(action_id);
                    self.explorer_panel_state.pending_transaction_refresh = true;
                }

                ui.separator();
                ui.label(self.tr("交易详情", "Transaction Detail"));
                if let Some(item) = self.explorer_panel_state.selected_transaction.as_ref() {
                    ui.small(format!(
                        "action_id={} | {} | {} -> {} | amount={} | nonce={} | submitted_at={} | updated_at={}",
                        item.action_id,
                        self.explorer_lifecycle_text(item.status),
                        item.from_account_id,
                        item.to_account_id,
                        item.amount,
                        item.nonce,
                        item.submitted_at_unix_ms,
                        item.updated_at_unix_ms,
                    ));
                    if let Some(error) = item.error.as_deref() {
                        let error_code = item.error_code.as_deref().unwrap_or("unknown");
                        ui.small(
                            egui::RichText::new(format!("error ({error_code}): {error}"))
                                .color(egui::Color32::from_rgb(196, 84, 84)),
                        );
                    }
                } else {
                    ui.small(self.tr("未选择交易", "No transaction selected"));
                }
            });

        self.explorer_window_open = window_open;
    }
}
