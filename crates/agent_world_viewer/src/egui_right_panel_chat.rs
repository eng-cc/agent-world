use std::collections::BTreeSet;

use agent_world::simulator::{LlmChatMessageTrace, LlmChatRole};
use bevy_egui::egui;

use crate::{ViewerClient, ViewerState};

const CHAT_MESSAGE_LIMIT: usize = 80;

#[derive(Default)]
pub(crate) struct AgentChatDraftState {
    selected_agent_id: Option<String>,
    input_message: String,
    status_message: String,
    input_focused: bool,
}

pub(super) fn render_chat_section(
    ui: &mut egui::Ui,
    locale: crate::i18n::UiLocale,
    state: &ViewerState,
    client: Option<&ViewerClient>,
    draft: &mut AgentChatDraftState,
) -> bool {
    ui.strong(if locale.is_zh() {
        "玩家 / Agent 对话"
    } else {
        "Player / Agent Chat"
    });

    let agent_ids = collect_chat_agent_ids(state);
    if agent_ids.is_empty() {
        ui.label(if locale.is_zh() {
            "暂无可用 Agent（等待 snapshot/trace）"
        } else {
            "No available agent yet (waiting for snapshot/trace)"
        });
        draft.input_focused = false;
        return false;
    }

    let selected_valid = draft
        .selected_agent_id
        .as_ref()
        .is_some_and(|current| agent_ids.iter().any(|id| id == current));
    if !selected_valid {
        draft.selected_agent_id = Some(agent_ids[0].clone());
    }
    let selected_agent_id = draft
        .selected_agent_id
        .clone()
        .unwrap_or_else(|| agent_ids[0].clone());

    egui::ComboBox::from_label(if locale.is_zh() {
        "目标 Agent"
    } else {
        "Target Agent"
    })
    .selected_text(selected_agent_id.as_str())
    .show_ui(ui, |ui| {
        for agent_id in &agent_ids {
            if ui
                .selectable_label(selected_agent_id == *agent_id, agent_id.as_str())
                .clicked()
            {
                draft.selected_agent_id = Some(agent_id.clone());
            }
        }
    });

    let input_response = ui.add(
        egui::TextEdit::multiline(&mut draft.input_message)
            .id_source(crate::EGUI_CHAT_INPUT_WIDGET_ID)
            .desired_rows(2)
            .hint_text(if locale.is_zh() {
                "输入玩家消息后发送给 Agent"
            } else {
                "Type player message and send to agent"
            }),
    );
    if input_response.gained_focus() || input_response.clicked() {
        draft.input_focused = true;
    }
    if input_response.lost_focus() {
        draft.input_focused = false;
    }
    let input_active = draft.input_focused || input_response.has_focus();

    ui.horizontal_wrapped(|ui| {
        let can_send = !draft.input_message.trim().is_empty();
        if ui
            .add_enabled(
                can_send,
                egui::Button::new(if locale.is_zh() { "发送" } else { "Send" }),
            )
            .clicked()
        {
            let message = draft.input_message.trim().to_string();
            if let Some(client) = client {
                let request = agent_world::viewer::ViewerRequest::AgentChat {
                    request: agent_world::viewer::AgentChatRequest {
                        agent_id: selected_agent_id.clone(),
                        message,
                        player_id: Some("viewer-player".to_string()),
                    },
                };
                match client.tx.send(request) {
                    Ok(()) => {
                        draft.status_message = if locale.is_zh() {
                            "消息已发送（等待 Agent 下一轮决策回显）".to_string()
                        } else {
                            "Message sent (waiting for next agent decision trace)".to_string()
                        };
                        draft.input_message.clear();
                    }
                    Err(err) => {
                        draft.status_message = if locale.is_zh() {
                            format!("发送失败: {err}")
                        } else {
                            format!("Send failed: {err}")
                        };
                    }
                }
            } else {
                draft.status_message = if locale.is_zh() {
                    "当前未连接 viewer client".to_string()
                } else {
                    "Viewer client unavailable".to_string()
                };
            }
        }
    });

    if !draft.status_message.is_empty() {
        ui.add(
            egui::Label::new(draft.status_message.as_str())
                .wrap()
                .selectable(true),
        );
    }

    let messages =
        collect_chat_messages_for_agent(state, selected_agent_id.as_str(), CHAT_MESSAGE_LIMIT);
    ui.separator();
    if messages.is_empty() {
        ui.label(if locale.is_zh() {
            "暂无对话消息。"
        } else {
            "No chat messages yet."
        });
        return input_active;
    }

    egui::ScrollArea::vertical()
        .max_height(260.0)
        .show(ui, |ui| {
            for message in messages {
                let role = chat_role_label(message.role, locale);
                let line = format!("[T{}][{}] {}", message.time, role, message.content);
                ui.add(egui::Label::new(line).wrap().selectable(true));
            }
        });

    input_active
}

fn collect_chat_agent_ids(state: &ViewerState) -> Vec<String> {
    let mut ids = BTreeSet::new();

    if let Some(snapshot) = state.snapshot.as_ref() {
        for agent_id in snapshot.model.agents.keys() {
            ids.insert(agent_id.clone());
        }
    }
    for trace in &state.decision_traces {
        ids.insert(trace.agent_id.clone());
    }

    ids.into_iter().collect()
}

fn collect_chat_messages_for_agent(
    state: &ViewerState,
    agent_id: &str,
    limit: usize,
) -> Vec<LlmChatMessageTrace> {
    let mut messages = state
        .decision_traces
        .iter()
        .flat_map(|trace| trace.llm_chat_messages.iter())
        .filter(|message| message.agent_id == agent_id)
        .cloned()
        .collect::<Vec<_>>();
    messages.sort_by_key(|message| message.time);
    if messages.len() > limit {
        messages.drain(0..(messages.len() - limit));
    }
    messages
}

fn chat_role_label(role: LlmChatRole, locale: crate::i18n::UiLocale) -> &'static str {
    match (locale.is_zh(), role) {
        (true, LlmChatRole::Player) => "玩家",
        (true, LlmChatRole::Agent) => "Agent",
        (true, LlmChatRole::Tool) => "工具",
        (true, LlmChatRole::System) => "系统",
        (false, LlmChatRole::Player) => "Player",
        (false, LlmChatRole::Agent) => "Agent",
        (false, LlmChatRole::Tool) => "Tool",
        (false, LlmChatRole::System) => "System",
    }
}
