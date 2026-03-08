use super::*;

const EXPLORER_POLL_INTERVAL_MS: u64 = 1_000;
const EXPLORER_DEFAULT_LIMIT: usize = 50;

#[path = "explorer_window_p1.rs"]
mod explorer_window_p1;

use explorer_window_p1::ExplorerP1State;
pub(super) use explorer_window_p1::{
    WebExplorerAddressResponse, WebExplorerAssetsResponse, WebExplorerContractResponse,
    WebExplorerContractsResponse, WebExplorerMempoolResponse,
};

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
pub(super) struct WebExplorerBlockItem {
    pub(super) height: u64,
    pub(super) slot: u64,
    pub(super) epoch: u64,
    pub(super) block_hash: String,
    pub(super) action_root: String,
    pub(super) action_count: usize,
    pub(super) committed_at_unix_ms: i64,
    #[serde(default)]
    pub(super) tx_hashes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WebExplorerTxItem {
    pub(super) tx_hash: String,
    pub(super) action_id: u64,
    pub(super) from_account_id: String,
    pub(super) to_account_id: String,
    pub(super) amount: u64,
    pub(super) nonce: u64,
    pub(super) status: transfer_window::WebTransferLifecycleStatus,
    pub(super) submitted_at_unix_ms: i64,
    pub(super) updated_at_unix_ms: i64,
    pub(super) block_height: Option<u64>,
    pub(super) block_hash: Option<String>,
    pub(super) error_code: Option<String>,
    pub(super) error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WebExplorerBlocksResponse {
    pub(super) ok: bool,
    pub(super) observed_at_unix_ms: i64,
    pub(super) limit: usize,
    pub(super) cursor: usize,
    pub(super) total: usize,
    pub(super) next_cursor: Option<usize>,
    #[serde(default)]
    pub(super) items: Vec<WebExplorerBlockItem>,
    pub(super) error_code: Option<String>,
    pub(super) error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WebExplorerBlockResponse {
    pub(super) ok: bool,
    pub(super) observed_at_unix_ms: i64,
    pub(super) height: Option<u64>,
    pub(super) block_hash: Option<String>,
    pub(super) block: Option<WebExplorerBlockItem>,
    pub(super) error_code: Option<String>,
    pub(super) error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WebExplorerTxsResponse {
    pub(super) ok: bool,
    pub(super) observed_at_unix_ms: i64,
    pub(super) account_filter: Option<String>,
    pub(super) status_filter: Option<transfer_window::WebTransferLifecycleStatus>,
    pub(super) action_filter: Option<u64>,
    pub(super) limit: usize,
    pub(super) cursor: usize,
    pub(super) total: usize,
    pub(super) next_cursor: Option<usize>,
    #[serde(default)]
    pub(super) items: Vec<WebExplorerTxItem>,
    pub(super) error_code: Option<String>,
    pub(super) error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WebExplorerTxResponse {
    pub(super) ok: bool,
    pub(super) observed_at_unix_ms: i64,
    pub(super) tx_hash: Option<String>,
    pub(super) action_id: Option<u64>,
    pub(super) tx: Option<WebExplorerTxItem>,
    pub(super) error_code: Option<String>,
    pub(super) error: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WebExplorerSearchHit {
    pub(super) item_type: String,
    pub(super) key: String,
    pub(super) summary: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WebExplorerSearchResponse {
    pub(super) ok: bool,
    pub(super) observed_at_unix_ms: i64,
    pub(super) q: String,
    pub(super) total: usize,
    #[serde(default)]
    pub(super) items: Vec<WebExplorerSearchHit>,
    pub(super) error_code: Option<String>,
    pub(super) error: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) enum ExplorerQueryResponse {
    Overview(WebExplorerOverviewResponse),
    Blocks(WebExplorerBlocksResponse),
    Block(WebExplorerBlockResponse),
    Txs(WebExplorerTxsResponse),
    Tx(WebExplorerTxResponse),
    Search(WebExplorerSearchResponse),
    Address(WebExplorerAddressResponse),
    Contracts(WebExplorerContractsResponse),
    Contract(WebExplorerContractResponse),
    Assets(WebExplorerAssetsResponse),
    Mempool(WebExplorerMempoolResponse),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExplorerTab {
    Blocks,
    Txs,
    Search,
    Address,
    Contracts,
    Assets,
    Mempool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ExplorerQuickShortcut {
    LatestBlock,
    RecentTxs,
    MyAccount,
}

impl Default for ExplorerTab {
    fn default() -> Self {
        Self::Blocks
    }
}

#[derive(Debug, Clone)]
pub(super) struct ExplorerPanelState {
    pub(super) overview: Option<WebExplorerOverviewResponse>,
    active_tab: ExplorerTab,
    pub(super) blocks: Vec<WebExplorerBlockItem>,
    pub(super) selected_block: Option<WebExplorerBlockItem>,
    pub(super) block_height_input: String,
    pub(super) block_hash_input: String,
    pub(super) blocks_limit: usize,
    pub(super) blocks_cursor: usize,
    pub(super) blocks_total: usize,
    pub(super) blocks_next_cursor: Option<usize>,
    pub(super) txs: Vec<WebExplorerTxItem>,
    pub(super) selected_tx: Option<WebExplorerTxItem>,
    pub(super) tx_hash_input: String,
    pub(super) tx_action_input: String,
    pub(super) account_filter: String,
    pub(super) action_filter_input: String,
    pub(super) status_filter: ExplorerStatusFilter,
    pub(super) txs_limit: usize,
    pub(super) txs_cursor: usize,
    pub(super) txs_total: usize,
    pub(super) txs_next_cursor: Option<usize>,
    pub(super) search_query: String,
    pub(super) search_results: Vec<WebExplorerSearchHit>,
    pub(super) pending_overview_refresh: bool,
    pub(super) pending_blocks_refresh: bool,
    pub(super) pending_block_refresh: bool,
    pub(super) pending_txs_refresh: bool,
    pub(super) pending_tx_refresh: bool,
    pub(super) pending_search_refresh: bool,
    pub(super) pending_block_height: Option<u64>,
    pub(super) pending_block_hash: Option<String>,
    pub(super) pending_tx_hash: Option<String>,
    pub(super) pending_tx_action_id: Option<u64>,
    pub(super) last_poll_at: Option<Instant>,
    p1: ExplorerP1State,
}

impl Default for ExplorerPanelState {
    fn default() -> Self {
        Self {
            overview: None,
            active_tab: ExplorerTab::default(),
            blocks: Vec::new(),
            selected_block: None,
            block_height_input: String::new(),
            block_hash_input: String::new(),
            blocks_limit: EXPLORER_DEFAULT_LIMIT,
            blocks_cursor: 0,
            blocks_total: 0,
            blocks_next_cursor: None,
            txs: Vec::new(),
            selected_tx: None,
            tx_hash_input: String::new(),
            tx_action_input: String::new(),
            account_filter: String::new(),
            action_filter_input: String::new(),
            status_filter: ExplorerStatusFilter::default(),
            txs_limit: EXPLORER_DEFAULT_LIMIT,
            txs_cursor: 0,
            txs_total: 0,
            txs_next_cursor: None,
            search_query: String::new(),
            search_results: Vec::new(),
            pending_overview_refresh: false,
            pending_blocks_refresh: false,
            pending_block_refresh: false,
            pending_txs_refresh: false,
            pending_tx_refresh: false,
            pending_search_refresh: false,
            pending_block_height: None,
            pending_block_hash: None,
            pending_tx_hash: None,
            pending_tx_action_id: None,
            last_poll_at: None,
            p1: ExplorerP1State::default(),
        }
    }
}

fn parse_positive_u64(raw: &str) -> Option<u64> {
    raw.trim().parse::<u64>().ok().filter(|value| *value > 0)
}

pub(super) fn resolve_explorer_my_account_candidate(
    transfer_from_account: &str,
    tx_account_filter: &str,
    address_account_input: &str,
) -> Option<String> {
    for candidate in [
        transfer_from_account.trim(),
        tx_account_filter.trim(),
        address_account_input.trim(),
    ] {
        if !candidate.is_empty() {
            return Some(candidate.to_string());
        }
    }
    None
}

fn short_hash(raw: &str) -> String {
    if raw.len() <= 18 {
        return raw.to_string();
    }
    format!("{}...{}", &raw[..10], &raw[(raw.len() - 6)..])
}

fn explorer_status_filter_text(
    ui_language: UiLanguage,
    filter: ExplorerStatusFilter,
) -> &'static str {
    match (filter, ui_language) {
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

impl ClientLauncherApp {
    fn explorer_tab_text(&self, tab: ExplorerTab) -> &'static str {
        match (tab, self.ui_language) {
            (ExplorerTab::Blocks, UiLanguage::ZhCn) => "区块",
            (ExplorerTab::Blocks, UiLanguage::EnUs) => "Blocks",
            (ExplorerTab::Txs, UiLanguage::ZhCn) => "交易",
            (ExplorerTab::Txs, UiLanguage::EnUs) => "Txs",
            (ExplorerTab::Search, UiLanguage::ZhCn) => "搜索",
            (ExplorerTab::Search, UiLanguage::EnUs) => "Search",
            (ExplorerTab::Address, UiLanguage::ZhCn) => "地址",
            (ExplorerTab::Address, UiLanguage::EnUs) => "Address",
            (ExplorerTab::Contracts, UiLanguage::ZhCn) => "合约",
            (ExplorerTab::Contracts, UiLanguage::EnUs) => "Contracts",
            (ExplorerTab::Assets, UiLanguage::ZhCn) => "资产",
            (ExplorerTab::Assets, UiLanguage::EnUs) => "Assets",
            (ExplorerTab::Mempool, UiLanguage::ZhCn) => "内存池",
            (ExplorerTab::Mempool, UiLanguage::EnUs) => "Mempool",
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

    fn explorer_lifecycle_color(
        &self,
        status: transfer_window::WebTransferLifecycleStatus,
    ) -> egui::Color32 {
        match status {
            transfer_window::WebTransferLifecycleStatus::Accepted => {
                egui::Color32::from_rgb(66, 132, 214)
            }
            transfer_window::WebTransferLifecycleStatus::Pending => {
                egui::Color32::from_rgb(214, 150, 58)
            }
            transfer_window::WebTransferLifecycleStatus::Confirmed => {
                egui::Color32::from_rgb(56, 150, 86)
            }
            transfer_window::WebTransferLifecycleStatus::Failed => {
                egui::Color32::from_rgb(188, 60, 60)
            }
            transfer_window::WebTransferLifecycleStatus::Timeout => {
                egui::Color32::from_rgb(153, 92, 45)
            }
        }
    }

    fn explorer_shortcut_text(&self, shortcut: ExplorerQuickShortcut) -> &'static str {
        match (shortcut, self.ui_language) {
            (ExplorerQuickShortcut::LatestBlock, UiLanguage::ZhCn) => "最新区块",
            (ExplorerQuickShortcut::LatestBlock, UiLanguage::EnUs) => "Latest Block",
            (ExplorerQuickShortcut::RecentTxs, UiLanguage::ZhCn) => "最近交易",
            (ExplorerQuickShortcut::RecentTxs, UiLanguage::EnUs) => "Recent Txs",
            (ExplorerQuickShortcut::MyAccount, UiLanguage::ZhCn) => "我的账户",
            (ExplorerQuickShortcut::MyAccount, UiLanguage::EnUs) => "My Account",
        }
    }

    fn explorer_my_account_candidate(&self) -> Option<String> {
        resolve_explorer_my_account_candidate(
            self.transfer_draft.from_account_id.as_str(),
            self.explorer_panel_state.account_filter.as_str(),
            self.explorer_panel_state.p1.address_account_input.as_str(),
        )
    }

    pub(super) fn apply_explorer_quick_shortcut(&mut self, shortcut: ExplorerQuickShortcut) {
        match shortcut {
            ExplorerQuickShortcut::LatestBlock => {
                self.explorer_panel_state.active_tab = ExplorerTab::Blocks;
                self.explorer_panel_state.blocks_cursor = 0;
                self.explorer_panel_state.pending_blocks_refresh = true;
                if let Some(latest_height) = self
                    .explorer_panel_state
                    .overview
                    .as_ref()
                    .map(|overview| overview.latest_height)
                {
                    self.explorer_panel_state.block_height_input = latest_height.to_string();
                    self.explorer_panel_state.block_hash_input.clear();
                    self.explorer_panel_state.pending_block_height = Some(latest_height);
                    self.explorer_panel_state.pending_block_hash = None;
                    self.explorer_panel_state.pending_block_refresh = true;
                }
            }
            ExplorerQuickShortcut::RecentTxs => {
                self.explorer_panel_state.active_tab = ExplorerTab::Txs;
                self.explorer_panel_state.txs_cursor = 0;
                self.explorer_panel_state.account_filter.clear();
                self.explorer_panel_state.action_filter_input.clear();
                self.explorer_panel_state.status_filter = ExplorerStatusFilter::All;
                self.explorer_panel_state.pending_txs_refresh = true;
            }
            ExplorerQuickShortcut::MyAccount => {
                let Some(account_id) = self.explorer_my_account_candidate() else {
                    self.append_log(self.tr(
                        "快捷入口“我的账户”不可用：请先在转账或过滤中选择账户。",
                        "My Account shortcut is unavailable: choose an account first.",
                    ));
                    return;
                };
                self.explorer_panel_state.active_tab = ExplorerTab::Address;
                self.explorer_panel_state.p1.address_account_input = account_id;
                self.explorer_panel_state.p1.address_cursor = 0;
                self.explorer_panel_state.p1.pending_address_refresh = true;
            }
        }
        self.explorer_panel_state.pending_overview_refresh = true;
    }
    fn maybe_request_explorer_panel_data(&mut self) {
        if !self.explorer_window_open
            || self.web_request_inflight_for(WebRequestDomain::ExplorerQuery)
        {
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
                match self.explorer_panel_state.active_tab {
                    ExplorerTab::Blocks => self.explorer_panel_state.pending_blocks_refresh = true,
                    ExplorerTab::Txs => self.explorer_panel_state.pending_txs_refresh = true,
                    ExplorerTab::Search => self.explorer_panel_state.pending_search_refresh = true,
                    ExplorerTab::Address
                    | ExplorerTab::Contracts
                    | ExplorerTab::Assets
                    | ExplorerTab::Mempool => {
                        self.schedule_explorer_p1_tab_refresh(self.explorer_panel_state.active_tab)
                    }
                }
            }
            self.request_web_chain_explorer_overview();
            return;
        }

        if self.explorer_panel_state.pending_blocks_refresh {
            self.explorer_panel_state.pending_blocks_refresh = false;
            self.request_web_chain_explorer_blocks(
                self.explorer_panel_state.blocks_cursor,
                self.explorer_panel_state.blocks_limit,
            );
            return;
        }

        if self.explorer_panel_state.pending_block_refresh {
            self.explorer_panel_state.pending_block_refresh = false;
            self.request_web_chain_explorer_block(
                self.explorer_panel_state.pending_block_height,
                self.explorer_panel_state.pending_block_hash.clone(),
            );
            return;
        }

        if self.explorer_panel_state.pending_txs_refresh {
            self.explorer_panel_state.pending_txs_refresh = false;
            self.request_web_chain_explorer_txs(
                self.explorer_panel_state.account_filter.clone(),
                self.explorer_panel_state
                    .status_filter
                    .query_value()
                    .map(str::to_string),
                self.explorer_panel_state.action_filter_input.clone(),
                self.explorer_panel_state.txs_cursor,
                self.explorer_panel_state.txs_limit,
            );
            return;
        }

        if self.explorer_panel_state.pending_tx_refresh {
            self.explorer_panel_state.pending_tx_refresh = false;
            self.request_web_chain_explorer_tx(
                self.explorer_panel_state.pending_tx_hash.clone(),
                self.explorer_panel_state.pending_tx_action_id,
            );
            return;
        }

        if self.explorer_panel_state.pending_search_refresh {
            self.explorer_panel_state.pending_search_refresh = false;
            self.request_web_chain_explorer_search(self.explorer_panel_state.search_query.clone());
            return;
        }

        if self.maybe_request_explorer_p1_data() {
            return;
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
                    self.log_explorer_error(
                        self.tr("浏览器概览查询失败", "Explorer overview query failed"),
                        response.error_code,
                        response.error,
                    );
                }
            }
            Ok(ExplorerQueryResponse::Blocks(response)) => {
                if response.ok {
                    let selected_height = self
                        .explorer_panel_state
                        .selected_block
                        .as_ref()
                        .map(|item| item.height);
                    self.explorer_panel_state.blocks_limit = response.limit;
                    self.explorer_panel_state.blocks_cursor = response.cursor;
                    self.explorer_panel_state.blocks_total = response.total;
                    self.explorer_panel_state.blocks_next_cursor = response.next_cursor;
                    self.explorer_panel_state.blocks = response.items;
                    if let Some(selected_height) = selected_height {
                        self.explorer_panel_state.selected_block = self
                            .explorer_panel_state
                            .blocks
                            .iter()
                            .find(|item| item.height == selected_height)
                            .cloned();
                    }
                } else {
                    self.log_explorer_error(
                        self.tr("区块列表查询失败", "Block list query failed"),
                        response.error_code,
                        response.error,
                    );
                }
            }
            Ok(ExplorerQueryResponse::Block(response)) => {
                if response.ok {
                    self.explorer_panel_state.selected_block = response.block;
                } else {
                    self.log_explorer_error(
                        self.tr("区块详情查询失败", "Block detail query failed"),
                        response.error_code,
                        response.error,
                    );
                }
            }
            Ok(ExplorerQueryResponse::Txs(response)) => {
                if response.ok {
                    let selected_hash = self
                        .explorer_panel_state
                        .selected_tx
                        .as_ref()
                        .map(|item| item.tx_hash.clone());
                    self.explorer_panel_state.txs_limit = response.limit;
                    self.explorer_panel_state.txs_cursor = response.cursor;
                    self.explorer_panel_state.txs_total = response.total;
                    self.explorer_panel_state.txs_next_cursor = response.next_cursor;
                    self.explorer_panel_state.txs = response.items;
                    if let Some(selected_hash) = selected_hash {
                        self.explorer_panel_state.selected_tx = self
                            .explorer_panel_state
                            .txs
                            .iter()
                            .find(|item| item.tx_hash == selected_hash)
                            .cloned();
                    }
                } else {
                    self.log_explorer_error(
                        self.tr("交易列表查询失败", "Tx list query failed"),
                        response.error_code,
                        response.error,
                    );
                }
            }
            Ok(ExplorerQueryResponse::Tx(response)) => {
                if response.ok {
                    if let Some(tx_hash) = response.tx_hash {
                        self.explorer_panel_state.tx_hash_input = tx_hash;
                    }
                    self.explorer_panel_state.selected_tx = response.tx;
                } else {
                    self.log_explorer_error(
                        self.tr("交易详情查询失败", "Tx detail query failed"),
                        response.error_code,
                        response.error,
                    );
                }
            }
            Ok(ExplorerQueryResponse::Search(response)) => {
                if response.ok {
                    self.explorer_panel_state.search_query = response.q;
                    self.explorer_panel_state.search_results = response.items;
                } else {
                    self.log_explorer_error(
                        self.tr("搜索查询失败", "Search query failed"),
                        response.error_code,
                        response.error,
                    );
                }
            }
            Ok(ExplorerQueryResponse::Address(response)) => {
                self.apply_explorer_address_response(response);
            }
            Ok(ExplorerQueryResponse::Contracts(response)) => {
                self.apply_explorer_contracts_response(response);
            }
            Ok(ExplorerQueryResponse::Contract(response)) => {
                self.apply_explorer_contract_response(response);
            }
            Ok(ExplorerQueryResponse::Assets(response)) => {
                self.apply_explorer_assets_response(response);
            }
            Ok(ExplorerQueryResponse::Mempool(response)) => {
                self.apply_explorer_mempool_response(response);
            }
            Err(err) => {
                self.append_log(format!(
                    "{}: {err}",
                    self.tr("浏览器查询失败", "Explorer query failed")
                ));
            }
        }
    }

    fn log_explorer_error(
        &mut self,
        prefix: &str,
        error_code: Option<String>,
        error: Option<String>,
    ) {
        let error_text = error.unwrap_or_else(|| self.tr("未知错误", "Unknown error").to_string());
        let error_code = error_code
            .map(|code| format!(" ({code})"))
            .unwrap_or_default();
        self.append_log(format!("{prefix}{error_code}: {error_text}"));
    }

    pub(super) fn show_explorer_window(&mut self, ctx: &egui::Context) {
        if !self.explorer_window_open {
            return;
        }

        if self.explorer_panel_state.overview.is_none() {
            self.explorer_panel_state.pending_overview_refresh = true;
        }
        if self.explorer_panel_state.blocks.is_empty() {
            self.explorer_panel_state.pending_blocks_refresh = true;
        }
        if self.explorer_panel_state.txs.is_empty() {
            self.explorer_panel_state.pending_txs_refresh = true;
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
                        .button(self.tr("刷新当前视图", "Refresh Current View"))
                        .clicked()
                    {
                        self.explorer_panel_state.pending_overview_refresh = true;
                        match self.explorer_panel_state.active_tab {
                            ExplorerTab::Blocks => {
                                self.explorer_panel_state.pending_blocks_refresh = true
                            }
                            ExplorerTab::Txs => {
                                self.explorer_panel_state.pending_txs_refresh = true
                            }
                            ExplorerTab::Search => {
                                self.explorer_panel_state.pending_search_refresh = true
                            }
                            ExplorerTab::Address
                            | ExplorerTab::Contracts
                            | ExplorerTab::Assets
                            | ExplorerTab::Mempool => self.schedule_explorer_p1_tab_refresh(
                                self.explorer_panel_state.active_tab,
                            ),
                        }
                        self.explorer_panel_state.last_poll_at = Some(Instant::now());
                    }
                    if ui
                        .button(self.tr("重置当前筛选", "Reset Current Filters"))
                        .clicked()
                    {
                        self.reset_explorer_active_tab_state();
                    }
                    if self.web_request_inflight_for(WebRequestDomain::ExplorerQuery) {
                        ui.small(
                            egui::RichText::new(
                                self.tr("请求处理中，请稍候…", "Request in flight, please wait..."),
                            )
                            .color(egui::Color32::from_rgb(201, 146, 44)),
                        );
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(self.tr("快捷入口", "Quick Shortcuts"));
                    if ui
                        .button(self.explorer_shortcut_text(ExplorerQuickShortcut::LatestBlock))
                        .clicked()
                    {
                        self.apply_explorer_quick_shortcut(ExplorerQuickShortcut::LatestBlock);
                    }
                    if ui
                        .button(self.explorer_shortcut_text(ExplorerQuickShortcut::RecentTxs))
                        .clicked()
                    {
                        self.apply_explorer_quick_shortcut(ExplorerQuickShortcut::RecentTxs);
                    }
                    let my_account_available = self.explorer_my_account_candidate().is_some();
                    if ui
                        .add_enabled(
                            my_account_available,
                            egui::Button::new(
                                self.explorer_shortcut_text(ExplorerQuickShortcut::MyAccount),
                            ),
                        )
                        .clicked()
                    {
                        self.apply_explorer_quick_shortcut(ExplorerQuickShortcut::MyAccount);
                    }
                    if !my_account_available {
                        ui.small(self.tr(
                            "提示：先在转账中选择转出账户即可启用“我的账户”。",
                            "Tip: choose a sender in Transfer to enable My Account.",
                        ));
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    ui.small(self.tr("术语", "Glossary"));
                    self.render_glossary_term_chip(ui, GlossaryTerm::Slot);
                    self.render_glossary_term_chip(ui, GlossaryTerm::Mempool);
                    self.render_glossary_term_chip(ui, GlossaryTerm::ActionId);
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
                self.render_overview(ui);

                ui.separator();
                self.render_tabs(ui);

                ui.separator();
                match self.explorer_panel_state.active_tab {
                    ExplorerTab::Blocks => self.render_blocks_tab(ui),
                    ExplorerTab::Txs => self.render_txs_tab(ui),
                    ExplorerTab::Search => self.render_search_tab(ui),
                    ExplorerTab::Address => self.render_address_tab(ui),
                    ExplorerTab::Contracts => self.render_contracts_tab(ui),
                    ExplorerTab::Assets => self.render_assets_tab(ui),
                    ExplorerTab::Mempool => self.render_mempool_tab(ui),
                }
            });

        self.explorer_window_open = window_open;
    }

    fn render_overview(&mut self, ui: &mut egui::Ui) {
        ui.label(self.tr("链概览", "Overview"));
        if let Some(overview) = self.explorer_panel_state.overview.as_ref() {
            ui.group(|ui| {
                ui.small(format!(
                    "node={} | world={} | observed_at={}",
                    overview.node_id, overview.world_id, overview.observed_at_unix_ms
                ));
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new(format!("latest {}", overview.latest_height)).strong(),
                    );
                    ui.label(
                        egui::RichText::new(format!("committed {}", overview.committed_height))
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "network {}",
                            overview.network_committed_height
                        ))
                        .strong(),
                    );
                });
                ui.horizontal_wrapped(|ui| {
                    ui.small(format!(
                        "last_block={}",
                        overview.last_block_hash.as_deref().unwrap_or("n/a")
                    ));
                    ui.small(format!(
                        "last_exec={}",
                        overview
                            .last_execution_block_hash
                            .as_deref()
                            .unwrap_or("n/a")
                    ));
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        egui::RichText::new(format!("tracked {}", overview.tracked_records))
                            .color(egui::Color32::from_rgb(112, 121, 130)),
                    );
                    ui.label(
                        egui::RichText::new(format!("total {}", overview.transfer_total))
                            .color(egui::Color32::from_rgb(81, 104, 132)),
                    );
                    ui.label(
                        egui::RichText::new(format!("accepted {}", overview.transfer_accepted))
                            .color(self.explorer_lifecycle_color(
                                transfer_window::WebTransferLifecycleStatus::Accepted,
                            )),
                    );
                    ui.label(
                        egui::RichText::new(format!("pending {}", overview.transfer_pending))
                            .color(self.explorer_lifecycle_color(
                                transfer_window::WebTransferLifecycleStatus::Pending,
                            )),
                    );
                    ui.label(
                        egui::RichText::new(format!("confirmed {}", overview.transfer_confirmed))
                            .color(self.explorer_lifecycle_color(
                                transfer_window::WebTransferLifecycleStatus::Confirmed,
                            )),
                    );
                    ui.label(
                        egui::RichText::new(format!("failed {}", overview.transfer_failed)).color(
                            self.explorer_lifecycle_color(
                                transfer_window::WebTransferLifecycleStatus::Failed,
                            ),
                        ),
                    );
                    ui.label(
                        egui::RichText::new(format!("timeout {}", overview.transfer_timeout))
                            .color(self.explorer_lifecycle_color(
                                transfer_window::WebTransferLifecycleStatus::Timeout,
                            )),
                    );
                });
            });
        } else {
            ui.small(self.tr("暂无概览数据", "No overview data"));
        }
    }

    fn render_tabs(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            for tab in [
                ExplorerTab::Blocks,
                ExplorerTab::Txs,
                ExplorerTab::Search,
                ExplorerTab::Address,
                ExplorerTab::Contracts,
                ExplorerTab::Assets,
                ExplorerTab::Mempool,
            ] {
                let selected = self.explorer_panel_state.active_tab == tab;
                if ui
                    .selectable_label(selected, self.explorer_tab_text(tab))
                    .clicked()
                {
                    self.explorer_panel_state.active_tab = tab;
                    match tab {
                        ExplorerTab::Blocks => {
                            self.explorer_panel_state.pending_blocks_refresh = true
                        }
                        ExplorerTab::Txs => self.explorer_panel_state.pending_txs_refresh = true,
                        ExplorerTab::Search => {
                            self.explorer_panel_state.pending_search_refresh = true
                        }
                        ExplorerTab::Address
                        | ExplorerTab::Contracts
                        | ExplorerTab::Assets
                        | ExplorerTab::Mempool => self.schedule_explorer_p1_tab_refresh(tab),
                    }
                }
            }
        });
    }

    fn render_blocks_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(self.tr("区块列表", "Blocks"));
        ui.horizontal_wrapped(|ui| {
            let prev_disabled = self.explorer_panel_state.blocks_cursor == 0;
            if ui
                .add_enabled(!prev_disabled, egui::Button::new(self.tr("上一页", "Prev")))
                .clicked()
            {
                self.explorer_panel_state.blocks_cursor = self
                    .explorer_panel_state
                    .blocks_cursor
                    .saturating_sub(self.explorer_panel_state.blocks_limit);
                self.explorer_panel_state.pending_blocks_refresh = true;
            }
            let next_disabled = self.explorer_panel_state.blocks_next_cursor.is_none();
            if ui
                .add_enabled(!next_disabled, egui::Button::new(self.tr("下一页", "Next")))
                .clicked()
            {
                if let Some(next_cursor) = self.explorer_panel_state.blocks_next_cursor {
                    self.explorer_panel_state.blocks_cursor = next_cursor;
                    self.explorer_panel_state.pending_blocks_refresh = true;
                }
            }
            ui.small(format!(
                "cursor={} limit={} total={}",
                self.explorer_panel_state.blocks_cursor,
                self.explorer_panel_state.blocks_limit,
                self.explorer_panel_state.blocks_total,
            ));
        });

        ui.horizontal_wrapped(|ui| {
            ui.label("height");
            ui.text_edit_singleline(&mut self.explorer_panel_state.block_height_input);
            ui.label("hash");
            ui.text_edit_singleline(&mut self.explorer_panel_state.block_hash_input);
            if ui.button(self.tr("查询区块", "Query Block")).clicked() {
                let height =
                    parse_positive_u64(self.explorer_panel_state.block_height_input.as_str());
                let hash = self
                    .explorer_panel_state
                    .block_hash_input
                    .trim()
                    .to_string();
                if height.is_none() && hash.is_empty() {
                    self.append_log(self.tr(
                        "区块查询失败：height 或 hash 至少填写一个",
                        "Block query failed: provide height or hash",
                    ));
                } else {
                    self.explorer_panel_state.pending_block_height = height;
                    self.explorer_panel_state.pending_block_hash =
                        if hash.is_empty() { None } else { Some(hash) };
                    self.explorer_panel_state.pending_block_refresh = true;
                }
            }
        });

        let mut clicked_height = None;
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for block in &self.explorer_panel_state.blocks {
                    let line = format!(
                        "h={} slot={} | txs={} | hash={} | committed_at={}",
                        block.height,
                        block.slot,
                        block.tx_hashes.len(),
                        short_hash(block.block_hash.as_str()),
                        block.committed_at_unix_ms,
                    );
                    if ui.selectable_label(false, line).clicked() {
                        clicked_height = Some(block.height);
                    }
                }
                if self.explorer_panel_state.blocks.is_empty() {
                    ui.small(self.tr("暂无区块记录", "No blocks"));
                }
            });
        if let Some(height) = clicked_height {
            self.explorer_panel_state.block_height_input = height.to_string();
            self.explorer_panel_state.pending_block_height = Some(height);
            self.explorer_panel_state.pending_block_hash = None;
            self.explorer_panel_state.pending_block_refresh = true;
        }

        ui.separator();
        ui.label(self.tr("区块详情", "Block Detail"));
        if let Some(block) = self.explorer_panel_state.selected_block.as_ref() {
            ui.small(format!(
                "height={} slot={} epoch={} action_count={} committed_at={}",
                block.height,
                block.slot,
                block.epoch,
                block.action_count,
                block.committed_at_unix_ms
            ));
            ui.small(format!("block_hash={}", block.block_hash));
            ui.small(format!("action_root={}", block.action_root));
            if !block.tx_hashes.is_empty() {
                ui.small(self.tr("交易哈希", "Tx hashes"));
                let hashes = block.tx_hashes.clone();
                ui.horizontal_wrapped(|ui| {
                    for hash in hashes {
                        if ui.button(short_hash(hash.as_str())).clicked() {
                            self.explorer_panel_state.active_tab = ExplorerTab::Txs;
                            self.explorer_panel_state.tx_hash_input = hash.clone();
                            self.explorer_panel_state.pending_tx_hash = Some(hash);
                            self.explorer_panel_state.pending_tx_action_id = None;
                            self.explorer_panel_state.pending_tx_refresh = true;
                        }
                    }
                });
            }
        } else {
            ui.small(self.tr("未选择区块", "No block selected"));
        }
    }

    fn render_txs_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(self.tr("交易列表", "Txs"));
        ui.horizontal_wrapped(|ui| {
            let ui_language = self.ui_language;
            ui.label(self.tr("账户", "Account"));
            ui.text_edit_singleline(&mut self.explorer_panel_state.account_filter);
            ui.label(self.tr("状态", "Status"));
            egui::ComboBox::from_id_salt("explorer_status_filter")
                .selected_text(explorer_status_filter_text(
                    ui_language,
                    self.explorer_panel_state.status_filter,
                ))
                .show_ui(ui, |ui| {
                    for filter in [
                        ExplorerStatusFilter::All,
                        ExplorerStatusFilter::Accepted,
                        ExplorerStatusFilter::Pending,
                        ExplorerStatusFilter::Confirmed,
                        ExplorerStatusFilter::Failed,
                        ExplorerStatusFilter::Timeout,
                    ] {
                        ui.selectable_value(
                            &mut self.explorer_panel_state.status_filter,
                            filter,
                            explorer_status_filter_text(ui_language, filter),
                        );
                    }
                });
            ui.label("action_id");
            ui.text_edit_singleline(&mut self.explorer_panel_state.action_filter_input);
            if ui.button(self.tr("应用过滤", "Apply Filter")).clicked() {
                self.explorer_panel_state.txs_cursor = 0;
                self.explorer_panel_state.pending_txs_refresh = true;
            }
            if ui.button(self.tr("清空过滤", "Clear Filters")).clicked() {
                self.explorer_panel_state.account_filter.clear();
                self.explorer_panel_state.action_filter_input.clear();
                self.explorer_panel_state.status_filter = ExplorerStatusFilter::All;
                self.explorer_panel_state.txs_cursor = 0;
                self.explorer_panel_state.pending_txs_refresh = true;
            }
        });

        ui.horizontal_wrapped(|ui| {
            let prev_disabled = self.explorer_panel_state.txs_cursor == 0;
            if ui
                .add_enabled(!prev_disabled, egui::Button::new(self.tr("上一页", "Prev")))
                .clicked()
            {
                self.explorer_panel_state.txs_cursor = self
                    .explorer_panel_state
                    .txs_cursor
                    .saturating_sub(self.explorer_panel_state.txs_limit);
                self.explorer_panel_state.pending_txs_refresh = true;
            }
            let next_disabled = self.explorer_panel_state.txs_next_cursor.is_none();
            if ui
                .add_enabled(!next_disabled, egui::Button::new(self.tr("下一页", "Next")))
                .clicked()
            {
                if let Some(next_cursor) = self.explorer_panel_state.txs_next_cursor {
                    self.explorer_panel_state.txs_cursor = next_cursor;
                    self.explorer_panel_state.pending_txs_refresh = true;
                }
            }
            ui.small(format!(
                "cursor={} limit={} total={}",
                self.explorer_panel_state.txs_cursor,
                self.explorer_panel_state.txs_limit,
                self.explorer_panel_state.txs_total,
            ));
        });

        ui.horizontal_wrapped(|ui| {
            ui.label("tx_hash");
            ui.text_edit_singleline(&mut self.explorer_panel_state.tx_hash_input);
            ui.label("action_id");
            ui.text_edit_singleline(&mut self.explorer_panel_state.tx_action_input);
            if ui.button(self.tr("查询交易", "Query Tx")).clicked() {
                let tx_hash = self.explorer_panel_state.tx_hash_input.trim().to_string();
                let action_id =
                    parse_positive_u64(self.explorer_panel_state.tx_action_input.as_str());
                if tx_hash.is_empty() && action_id.is_none() {
                    self.append_log(self.tr(
                        "交易查询失败：tx_hash 或 action_id 至少填写一个",
                        "Tx query failed: provide tx_hash or action_id",
                    ));
                } else {
                    self.explorer_panel_state.pending_tx_hash = if tx_hash.is_empty() {
                        None
                    } else {
                        Some(tx_hash)
                    };
                    self.explorer_panel_state.pending_tx_action_id = action_id;
                    self.explorer_panel_state.pending_tx_refresh = true;
                }
            }
        });

        let mut clicked_hash = None;
        egui::ScrollArea::vertical()
            .max_height(220.0)
            .show(ui, |ui| {
                for tx in &self.explorer_panel_state.txs {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "[{}]",
                                self.explorer_lifecycle_text(tx.status)
                            ))
                            .color(self.explorer_lifecycle_color(tx.status)),
                        );
                        let line = format!(
                            "{} | {} -> {} | amount={} | h={} | {}",
                            short_hash(tx.tx_hash.as_str()),
                            tx.from_account_id,
                            tx.to_account_id,
                            tx.amount,
                            tx.block_height
                                .map(|height| height.to_string())
                                .unwrap_or_else(|| "n/a".to_string()),
                            tx.submitted_at_unix_ms,
                        );
                        if ui.selectable_label(false, line).clicked() {
                            clicked_hash = Some(tx.tx_hash.clone());
                        }
                    });
                }
                if self.explorer_panel_state.txs.is_empty() {
                    ui.small(self.tr("暂无交易记录", "No txs"));
                }
            });
        if let Some(tx_hash) = clicked_hash {
            self.explorer_panel_state.tx_hash_input = tx_hash.clone();
            self.explorer_panel_state.pending_tx_hash = Some(tx_hash);
            self.explorer_panel_state.pending_tx_action_id = None;
            self.explorer_panel_state.pending_tx_refresh = true;
        }

        ui.separator();
        ui.label(self.tr("交易详情", "Tx Detail"));
        if let Some(tx) = self.explorer_panel_state.selected_tx.as_ref() {
            ui.small(format!(
                "tx_hash={} | action_id={} | {}",
                tx.tx_hash,
                tx.action_id,
                self.explorer_lifecycle_text(tx.status)
            ));
            ui.small(
                egui::RichText::new(self.explorer_lifecycle_text(tx.status))
                    .color(self.explorer_lifecycle_color(tx.status)),
            );
            ui.small(format!(
                "{} -> {} | amount={} | nonce={} | submitted_at={} | updated_at={}",
                tx.from_account_id,
                tx.to_account_id,
                tx.amount,
                tx.nonce,
                tx.submitted_at_unix_ms,
                tx.updated_at_unix_ms,
            ));
            ui.small(format!(
                "block_height={} | block_hash={}",
                tx.block_height
                    .map(|height| height.to_string())
                    .unwrap_or_else(|| "n/a".to_string()),
                tx.block_hash.as_deref().unwrap_or("n/a"),
            ));
            if let Some(error) = tx.error.as_deref() {
                ui.small(
                    egui::RichText::new(format!(
                        "error ({}): {}",
                        tx.error_code.as_deref().unwrap_or("unknown"),
                        error
                    ))
                    .color(egui::Color32::from_rgb(196, 84, 84)),
                );
            }
        } else {
            ui.small(self.tr("未选择交易", "No tx selected"));
        }
    }

    fn render_search_tab(&mut self, ui: &mut egui::Ui) {
        ui.label(self.tr("统一搜索", "Unified Search"));
        ui.horizontal_wrapped(|ui| {
            ui.label(self.tr(
                "支持 height/block_hash/tx_hash/action_id/account_id",
                "Supports height/block_hash/tx_hash/action_id/account_id",
            ));
        });
        ui.horizontal_wrapped(|ui| {
            ui.text_edit_singleline(&mut self.explorer_panel_state.search_query);
            if ui.button(self.tr("搜索", "Search")).clicked() {
                if self.explorer_panel_state.search_query.trim().is_empty() {
                    self.append_log(
                        self.tr("搜索失败：请输入关键词", "Search failed: query is empty"),
                    );
                } else {
                    self.explorer_panel_state.pending_search_refresh = true;
                }
            }
            if ui.button(self.tr("清空", "Clear")).clicked() {
                self.explorer_panel_state.search_query.clear();
                self.explorer_panel_state.search_results.clear();
            }
        });

        let mut clicked: Option<(String, String)> = None;
        egui::ScrollArea::vertical()
            .max_height(260.0)
            .show(ui, |ui| {
                for item in &self.explorer_panel_state.search_results {
                    let line = format!("[{}] {} | {}", item.item_type, item.key, item.summary,);
                    if ui.selectable_label(false, line).clicked() {
                        clicked = Some((item.item_type.clone(), item.key.clone()));
                    }
                }
                if self.explorer_panel_state.search_results.is_empty() {
                    ui.small(self.tr("暂无搜索结果", "No search results"));
                }
            });

        if let Some((item_type, key)) = clicked {
            match item_type.as_str() {
                "block" => {
                    self.explorer_panel_state.active_tab = ExplorerTab::Blocks;
                    self.explorer_panel_state.block_height_input = key.clone();
                    self.explorer_panel_state.block_hash_input = key.clone();
                    self.explorer_panel_state.pending_block_height =
                        parse_positive_u64(key.as_str());
                    self.explorer_panel_state.pending_block_hash = Some(key);
                    self.explorer_panel_state.pending_block_refresh = true;
                }
                "tx" => {
                    self.explorer_panel_state.active_tab = ExplorerTab::Txs;
                    self.explorer_panel_state.tx_hash_input = key.clone();
                    self.explorer_panel_state.pending_tx_hash = Some(key);
                    self.explorer_panel_state.pending_tx_action_id = None;
                    self.explorer_panel_state.pending_tx_refresh = true;
                }
                _ => {
                    self.append_log(format!(
                        "{}: {}",
                        self.tr("未支持的搜索类型", "Unsupported search item type"),
                        item_type
                    ));
                }
            }
        }
    }
}
