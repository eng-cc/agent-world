use super::*;

use js_sys::{Array, Reflect};
use serde::{Deserialize, Serialize};
use web_sys::wasm_bindgen::JsCast;
use web_sys::wasm_bindgen::JsValue;

const TEST_QUEUE_KEY: &str = "__OASIS7_LAUNCHER_TEST_QUEUE";
const TEST_STATE_KEY: &str = "__OASIS7_LAUNCHER_TEST_STATE";

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum LauncherTestCommand {
    OpenTransferWindow,
    SetTransferDraft {
        #[serde(default)]
        from_account_id: Option<String>,
        #[serde(default)]
        to_account_id: Option<String>,
        #[serde(default)]
        amount: Option<String>,
        #[serde(default)]
        nonce: Option<String>,
    },
    SetTransferNonceMode {
        mode: String,
    },
    SubmitTransfer,
    RefreshTransferAccounts,
    RefreshTransferHistory,
    PollTransferStatus,
}

#[derive(Debug, Serialize)]
struct LauncherTestStateSnapshot {
    chain_runtime_status: String,
    transfer_window_open: bool,
    transfer_nonce_mode: &'static str,
    transfer_draft: LauncherTestTransferDraftSnapshot,
    transfer_submit_state: Option<LauncherTestSubmitStateSnapshot>,
    tracked_action_id: Option<u64>,
    tracked_status: Option<LauncherTestTrackedStatusSnapshot>,
    account_ids: Vec<String>,
    history_count: usize,
    logs_tail: Vec<String>,
}

#[derive(Debug, Serialize)]
struct LauncherTestTransferDraftSnapshot {
    from_account_id: String,
    to_account_id: String,
    amount: String,
    nonce: String,
}

#[derive(Debug, Serialize)]
struct LauncherTestSubmitStateSnapshot {
    kind: &'static str,
    message: String,
}

#[derive(Debug, Serialize)]
struct LauncherTestTrackedStatusSnapshot {
    action_id: u64,
    status: &'static str,
    error_code: Option<String>,
    error: Option<String>,
}

pub(super) fn sync_launcher_test_hook(app: &mut ClientLauncherApp) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let queue = ensure_test_queue(&window);
    while let Some(command) = pop_test_command(&queue) {
        apply_test_command(app, command);
    }
    publish_test_state(&window, app);
}

fn ensure_test_queue(window: &web_sys::Window) -> Array {
    let key = JsValue::from_str(TEST_QUEUE_KEY);
    if let Ok(existing) = Reflect::get(window.as_ref(), &key) {
        if let Ok(queue) = existing.dyn_into::<Array>() {
            return queue;
        }
    }
    let queue = Array::new();
    let _ = Reflect::set(window.as_ref(), &key, queue.as_ref());
    queue
}

fn pop_test_command(queue: &Array) -> Option<LauncherTestCommand> {
    if queue.length() == 0 {
        return None;
    }
    let value = queue.shift();
    let text = js_sys::JSON::stringify(&value).ok()?.as_string()?;
    serde_json::from_str(text.as_str()).ok()
}

fn apply_test_command(app: &mut ClientLauncherApp, command: LauncherTestCommand) {
    match command {
        LauncherTestCommand::OpenTransferWindow => {
            app.transfer_window_open = true;
        }
        LauncherTestCommand::SetTransferDraft {
            from_account_id,
            to_account_id,
            amount,
            nonce,
        } => {
            if let Some(value) = from_account_id {
                app.transfer_draft.from_account_id = value;
            }
            if let Some(value) = to_account_id {
                app.transfer_draft.to_account_id = value;
            }
            if let Some(value) = amount {
                app.transfer_draft.amount = value;
            }
            if let Some(value) = nonce {
                app.transfer_draft.nonce = value;
            }
        }
        LauncherTestCommand::SetTransferNonceMode { mode } => {
            app.transfer_panel_state.nonce_mode = match mode.trim().to_ascii_lowercase().as_str() {
                "manual" => transfer_window::TransferNonceMode::Manual,
                _ => transfer_window::TransferNonceMode::Auto,
            };
        }
        LauncherTestCommand::SubmitTransfer => {
            app.submit_transfer();
        }
        LauncherTestCommand::RefreshTransferAccounts => {
            app.request_web_chain_transfer_accounts();
        }
        LauncherTestCommand::RefreshTransferHistory => {
            app.request_web_chain_transfer_history(
                app.transfer_panel_state.history_account_filter.clone(),
                app.transfer_panel_state.history_action_filter.clone(),
            );
        }
        LauncherTestCommand::PollTransferStatus => {
            if let Some(action_id) = app.transfer_panel_state.tracked_action_id {
                app.request_web_chain_transfer_status(action_id);
            }
        }
    }
}

fn publish_test_state(window: &web_sys::Window, app: &ClientLauncherApp) {
    let snapshot = LauncherTestStateSnapshot {
        chain_runtime_status: app.chain_runtime_status.text(app.ui_language).to_string(),
        transfer_window_open: app.transfer_window_open,
        transfer_nonce_mode: match app.transfer_panel_state.nonce_mode {
            transfer_window::TransferNonceMode::Auto => "auto",
            transfer_window::TransferNonceMode::Manual => "manual",
        },
        transfer_draft: LauncherTestTransferDraftSnapshot {
            from_account_id: app.transfer_draft.from_account_id.clone(),
            to_account_id: app.transfer_draft.to_account_id.clone(),
            amount: app.transfer_draft.amount.clone(),
            nonce: app.transfer_draft.nonce.clone(),
        },
        transfer_submit_state: match &app.transfer_submit_state {
            TransferSubmitState::None => None,
            TransferSubmitState::Success(message) => Some(LauncherTestSubmitStateSnapshot {
                kind: "success",
                message: message.clone(),
            }),
            TransferSubmitState::Failed(message) => Some(LauncherTestSubmitStateSnapshot {
                kind: "failed",
                message: message.clone(),
            }),
        },
        tracked_action_id: app.transfer_panel_state.tracked_action_id,
        tracked_status: app
            .transfer_panel_state
            .tracked_action_status
            .as_ref()
            .map(|status| LauncherTestTrackedStatusSnapshot {
                action_id: status.action_id,
                status: match status.status {
                    transfer_window::WebTransferLifecycleStatus::Accepted => "accepted",
                    transfer_window::WebTransferLifecycleStatus::Pending => "pending",
                    transfer_window::WebTransferLifecycleStatus::Confirmed => "confirmed",
                    transfer_window::WebTransferLifecycleStatus::Failed => "failed",
                    transfer_window::WebTransferLifecycleStatus::Timeout => "timeout",
                },
                error_code: status.error_code.clone(),
                error: status.error.clone(),
            }),
        account_ids: app
            .transfer_panel_state
            .accounts
            .iter()
            .map(|account| account.account_id.clone())
            .collect(),
        history_count: app.transfer_panel_state.history.len(),
        logs_tail: app.logs.iter().rev().take(12).cloned().collect::<Vec<_>>(),
    };
    let Ok(text) = serde_json::to_string(&snapshot) else {
        return;
    };
    let Ok(value) = js_sys::JSON::parse(text.as_str()) else {
        return;
    };
    let _ = Reflect::set(window.as_ref(), &JsValue::from_str(TEST_STATE_KEY), &value);
}
