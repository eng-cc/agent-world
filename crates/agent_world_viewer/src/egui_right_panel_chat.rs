use std::collections::BTreeSet;

use agent_world::simulator::{LlmChatMessageTrace, LlmChatRole};
use bevy_egui::egui;

use crate::{ViewerClient, ViewerState};

const CHAT_MESSAGE_LIMIT: usize = 96;
const CHAT_THREAD_LIMIT: usize = 64;
const CHAT_THREAD_SCAN_MESSAGE_LIMIT: usize = 320;
const CHAT_PREVIEW_CHARS: usize = 42;
const CHAT_BUBBLE_MAX_WIDTH: f32 = 380.0;
const TOOL_CALL_PREVIEW_CHARS: usize = 180;
const TOOL_CALL_CARD_MAX_WIDTH: f32 = 380.0;

#[derive(Clone, Debug, PartialEq, Eq)]
struct ToolCallView {
    module: String,
    status: String,
    args_preview: String,
    result_preview: String,
    raw_preview: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ChatThread {
    id: String,
    agent_id: String,
    title: String,
    started_at: u64,
    updated_at: u64,
    messages: Vec<LlmChatMessageTrace>,
}

#[derive(Debug)]
pub(crate) struct AgentChatDraftState {
    selected_agent_id: Option<String>,
    selected_thread_id: Option<String>,
    input_message: String,
    status_message: String,
    input_focused: bool,
    follow_latest_thread: bool,
}

impl Default for AgentChatDraftState {
    fn default() -> Self {
        Self {
            selected_agent_id: None,
            selected_thread_id: None,
            input_message: String::new(),
            status_message: String::new(),
            input_focused: false,
            follow_latest_thread: true,
        }
    }
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

    let chat_threads = collect_chat_threads(state, CHAT_THREAD_LIMIT, CHAT_MESSAGE_LIMIT);
    sync_chat_selection(draft, &chat_threads, &agent_ids);

    let mut selected_agent_id = draft
        .selected_agent_id
        .clone()
        .unwrap_or_else(|| agent_ids[0].clone());

    let active_thread = draft.selected_thread_id.as_ref().and_then(|thread_id| {
        chat_threads
            .iter()
            .find(|thread| &thread.id == thread_id)
            .cloned()
    });

    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.strong(if locale.is_zh() {
            "聊天记录"
        } else {
            "Chat Records"
        });
        if let Some(thread) = active_thread.as_ref() {
            ui.horizontal_wrapped(|ui| {
                ui.label(if locale.is_zh() {
                    "当前会话"
                } else {
                    "Current Thread"
                });
                ui.label(
                    egui::RichText::new(thread.title.as_str())
                        .color(egui::Color32::from_gray(220))
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(format!("T{}", thread.updated_at))
                        .size(10.5)
                        .color(egui::Color32::from_gray(150)),
                );
            });
        }

        let active_messages = active_thread
            .as_ref()
            .map(|thread| thread.messages.clone())
            .unwrap_or_default();
        if active_messages.is_empty() {
            ui.label(if locale.is_zh() {
                "暂无对话消息。"
            } else {
                "No chat messages yet."
            });
        } else {
            render_info_stream(ui, &active_messages, locale);
            ui.add_space(6.0);
            render_tool_call_stream(ui, &active_messages, locale);
        }
    });

    ui.add_space(6.0);

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
                selected_agent_id = agent_id.clone();
                draft.selected_agent_id = Some(agent_id.clone());
                draft.follow_latest_thread = true;
            }
        }
    });

    let input_response = ui.add(
        egui::TextEdit::multiline(&mut draft.input_message)
            .id_source(crate::EGUI_CHAT_INPUT_WIDGET_ID)
            .desired_rows(3)
            .hint_text(if locale.is_zh() {
                "输入玩家消息后发送给 Agent（Enter 发送，Shift+Enter 换行）"
            } else {
                "Type player message and send to agent (Enter to send, Shift+Enter for newline)"
            }),
    );
    if input_response.gained_focus() || input_response.clicked() {
        draft.input_focused = true;
    }
    if input_response.lost_focus() {
        draft.input_focused = false;
    }
    let input_has_focus = input_response.has_focus();
    let input_active = draft.input_focused || input_has_focus;
    let submit_by_enter = ui.input(|input| {
        should_submit_chat_on_enter(
            input_active,
            input.key_pressed(egui::Key::Enter),
            input.modifiers,
        )
    });

    ui.horizontal_wrapped(|ui| {
        let can_send = !draft.input_message.trim().is_empty();
        let submit_by_button = ui
            .add_enabled(
                can_send,
                egui::Button::new(if locale.is_zh() { "发送" } else { "Send" }),
            )
            .clicked();
        if can_send && (submit_by_button || submit_by_enter) {
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
                        draft.follow_latest_thread = true;
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

    input_active
}

fn render_info_stream(
    ui: &mut egui::Ui,
    messages: &[LlmChatMessageTrace],
    locale: crate::i18n::UiLocale,
) {
    ui.strong(if locale.is_zh() {
        "信息流"
    } else {
        "Info Stream"
    });
    let info_messages = messages
        .iter()
        .filter(|message| !matches!(message.role, LlmChatRole::Tool))
        .collect::<Vec<_>>();
    if info_messages.is_empty() {
        ui.label(if locale.is_zh() {
            "暂无信息消息。"
        } else {
            "No info messages."
        });
        return;
    }

    ui.push_id("chat-info-scroll", |ui| {
        egui::ScrollArea::vertical()
            .max_height(220.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (index, message) in info_messages.into_iter().enumerate() {
                    ui.push_id(("info", index, message.time), |ui| {
                        render_chat_message_bubble(ui, message, locale);
                        ui.add_space(2.0);
                    });
                }
            });
    });
}

fn render_tool_call_stream(
    ui: &mut egui::Ui,
    messages: &[LlmChatMessageTrace],
    locale: crate::i18n::UiLocale,
) {
    ui.strong(if locale.is_zh() {
        "工具调用"
    } else {
        "Tool Calls"
    });
    let tool_messages = messages
        .iter()
        .filter(|message| matches!(message.role, LlmChatRole::Tool))
        .collect::<Vec<_>>();
    if tool_messages.is_empty() {
        ui.label(if locale.is_zh() {
            "暂无工具调用。"
        } else {
            "No tool calls."
        });
        return;
    }

    ui.push_id("chat-tool-scroll", |ui| {
        egui::ScrollArea::vertical()
            .max_height(160.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for (index, message) in tool_messages.into_iter().enumerate() {
                    ui.push_id(("tool", index, message.time), |ui| {
                        render_tool_call_card(ui, message, locale);
                        ui.add_space(4.0);
                    });
                }
            });
    });
}

fn render_tool_call_card(
    ui: &mut egui::Ui,
    message: &LlmChatMessageTrace,
    locale: crate::i18n::UiLocale,
) {
    let tool_call = parse_tool_call_view(message);
    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgb(62, 58, 43))
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::same(8))
        .show(ui, |ui| {
            ui.set_max_width(TOOL_CALL_CARD_MAX_WIDTH);
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    egui::RichText::new(if locale.is_zh() {
                        format!("模块: {}", tool_call.module)
                    } else {
                        format!("Module: {}", tool_call.module)
                    })
                    .color(egui::Color32::from_gray(235))
                    .strong(),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(if locale.is_zh() {
                        format!("状态: {}", tool_call.status)
                    } else {
                        format!("Status: {}", tool_call.status)
                    })
                    .color(tool_status_color(tool_call.status.as_str())),
                );
            });

            ui.label(
                egui::RichText::new(if locale.is_zh() {
                    format!("参数: {}", tool_call.args_preview)
                } else {
                    format!("Args: {}", tool_call.args_preview)
                })
                .color(egui::Color32::from_gray(220)),
            );
            ui.add(
                egui::Label::new(
                    egui::RichText::new(if locale.is_zh() {
                        format!("结果: {}", tool_call.result_preview)
                    } else {
                        format!("Result: {}", tool_call.result_preview)
                    })
                    .color(egui::Color32::from_gray(236)),
                )
                .wrap()
                .selectable(true),
            );
            ui.add(
                egui::Label::new(
                    egui::RichText::new(if locale.is_zh() {
                        format!("原始: {}", tool_call.raw_preview)
                    } else {
                        format!("Raw: {}", tool_call.raw_preview)
                    })
                    .size(10.5)
                    .color(egui::Color32::from_gray(186)),
                )
                .wrap()
                .selectable(true),
            );
            ui.label(
                egui::RichText::new(format!("T{}", message.time))
                    .size(10.0)
                    .color(egui::Color32::from_gray(205)),
            );
        });
}

fn parse_tool_call_view(message: &LlmChatMessageTrace) -> ToolCallView {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&message.content) {
        if value
            .get("type")
            .and_then(|value| value.as_str())
            .is_some_and(|value| value == "module_call_result")
        {
            return ToolCallView {
                module: value
                    .get("module")
                    .and_then(|value| value.as_str())
                    .unwrap_or("-")
                    .to_string(),
                status: value
                    .get("status")
                    .and_then(|value| value.as_str())
                    .unwrap_or("-")
                    .to_string(),
                args_preview: compact_json_preview(value.get("args")),
                result_preview: compact_json_preview(value.get("result")),
                raw_preview: truncate_text(message.content.as_str(), TOOL_CALL_PREVIEW_CHARS),
            };
        }
    }

    parse_legacy_tool_call_view(message.content.as_str()).unwrap_or_else(|| ToolCallView {
        module: "-".to_string(),
        status: "-".to_string(),
        args_preview: "-".to_string(),
        result_preview: truncate_text(message.content.as_str(), TOOL_CALL_PREVIEW_CHARS),
        raw_preview: truncate_text(message.content.as_str(), TOOL_CALL_PREVIEW_CHARS),
    })
}

fn parse_legacy_tool_call_view(content: &str) -> Option<ToolCallView> {
    let module = extract_legacy_field(content, "module")?;
    let status = extract_legacy_field(content, "status").unwrap_or_else(|| "-".to_string());
    let result_preview = content
        .split_once("result=")
        .map(|(_, result)| truncate_text(result, TOOL_CALL_PREVIEW_CHARS))
        .unwrap_or_else(|| truncate_text(content, TOOL_CALL_PREVIEW_CHARS));

    Some(ToolCallView {
        module,
        status,
        args_preview: "-".to_string(),
        result_preview,
        raw_preview: truncate_text(content, TOOL_CALL_PREVIEW_CHARS),
    })
}

fn extract_legacy_field(content: &str, key: &str) -> Option<String> {
    let marker = format!("{key}=");
    let start = content.find(marker.as_str())?;
    let value = &content[start + marker.len()..];
    let token = value.split_whitespace().next().unwrap_or_default().trim();
    if token.is_empty() {
        return None;
    }
    Some(token.to_string())
}

fn compact_json_preview(value: Option<&serde_json::Value>) -> String {
    let Some(value) = value else {
        return "-".to_string();
    };
    let json = serde_json::to_string(value).unwrap_or_else(|_| "\"<serialize_error>\"".to_string());
    truncate_text(json.as_str(), TOOL_CALL_PREVIEW_CHARS)
}

fn tool_status_color(status: &str) -> egui::Color32 {
    match status.trim().to_ascii_lowercase().as_str() {
        "ok" | "success" => egui::Color32::from_rgb(104, 211, 145),
        "error" | "failed" => egui::Color32::from_rgb(244, 114, 114),
        _ => egui::Color32::from_gray(214),
    }
}

fn should_submit_chat_on_enter(
    input_has_focus: bool,
    enter_pressed: bool,
    modifiers: egui::Modifiers,
) -> bool {
    input_has_focus && enter_pressed && modifiers.is_none()
}

fn render_chat_message_bubble(
    ui: &mut egui::Ui,
    message: &LlmChatMessageTrace,
    locale: crate::i18n::UiLocale,
) {
    let (role_label, align_right, fill_color) = match message.role {
        LlmChatRole::Player => (
            if locale.is_zh() { "玩家" } else { "Player" },
            true,
            egui::Color32::from_rgb(37, 91, 167),
        ),
        LlmChatRole::Agent => (
            if locale.is_zh() { "Agent" } else { "Agent" },
            false,
            egui::Color32::from_rgb(54, 56, 66),
        ),
        LlmChatRole::Tool => (
            if locale.is_zh() { "工具" } else { "Tool" },
            false,
            egui::Color32::from_rgb(74, 72, 50),
        ),
        LlmChatRole::System => (
            if locale.is_zh() { "系统" } else { "System" },
            false,
            egui::Color32::from_rgb(70, 48, 52),
        ),
    };

    ui.horizontal(|ui| {
        let layout = if align_right {
            egui::Layout::right_to_left(egui::Align::TOP)
        } else {
            egui::Layout::left_to_right(egui::Align::TOP)
        };

        ui.with_layout(layout, |ui| {
            egui::Frame::group(ui.style())
                .fill(fill_color)
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::same(8))
                .show(ui, |ui| {
                    ui.set_max_width(CHAT_BUBBLE_MAX_WIDTH);
                    ui.label(
                        egui::RichText::new(role_label)
                            .size(10.5)
                            .color(egui::Color32::from_gray(214)),
                    );
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(message.content.as_str())
                                .color(egui::Color32::WHITE),
                        )
                        .wrap()
                        .selectable(true),
                    );
                    ui.label(
                        egui::RichText::new(format!("T{}", message.time))
                            .size(10.0)
                            .color(egui::Color32::from_gray(205)),
                    );
                });
        });
    });
}

fn sync_chat_selection(
    draft: &mut AgentChatDraftState,
    threads: &[ChatThread],
    agent_ids: &[String],
) {
    if !agent_ids.is_empty() {
        let selected_agent_valid = draft
            .selected_agent_id
            .as_ref()
            .is_some_and(|current| agent_ids.iter().any(|agent_id| agent_id == current));
        if !selected_agent_valid {
            draft.selected_agent_id = Some(agent_ids[0].clone());
            draft.follow_latest_thread = true;
        }
    } else {
        draft.selected_agent_id = None;
    }

    if threads.is_empty() {
        draft.selected_thread_id = None;
        return;
    }

    if draft.follow_latest_thread {
        let latest_for_agent = draft.selected_agent_id.as_ref().and_then(|agent_id| {
            threads
                .iter()
                .find(|thread| &thread.agent_id == agent_id)
                .map(|thread| thread.id.clone())
        });
        draft.selected_thread_id = latest_for_agent.or_else(|| Some(threads[0].id.clone()));
    } else {
        let selected_thread_valid = draft
            .selected_thread_id
            .as_ref()
            .is_some_and(|thread_id| threads.iter().any(|thread| &thread.id == thread_id));
        if !selected_thread_valid {
            draft.selected_thread_id = Some(threads[0].id.clone());
            draft.follow_latest_thread = true;
        }
    }

    if let Some(selected_thread_id) = draft.selected_thread_id.as_ref() {
        if let Some(selected_thread) = threads
            .iter()
            .find(|thread| &thread.id == selected_thread_id)
        {
            draft.selected_agent_id = Some(selected_thread.agent_id.clone());
        }
    }
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

fn collect_chat_threads(
    state: &ViewerState,
    thread_limit: usize,
    message_limit: usize,
) -> Vec<ChatThread> {
    let mut threads = Vec::new();

    for agent_id in collect_chat_agent_ids(state) {
        let messages = collect_chat_messages_for_agent(
            state,
            agent_id.as_str(),
            CHAT_THREAD_SCAN_MESSAGE_LIMIT,
        );
        if messages.is_empty() {
            continue;
        }

        let mut sequence = 0usize;
        let mut current_thread: Option<ChatThread> = None;

        for message in messages {
            let starts_new_thread =
                matches!(message.role, LlmChatRole::Player) || current_thread.is_none();

            if starts_new_thread {
                if let Some(mut thread) = current_thread.take() {
                    trim_messages_for_thread(&mut thread, message_limit);
                    threads.push(thread);
                }
                current_thread = Some(ChatThread {
                    id: format!("{agent_id}:{}:{sequence}", message.time),
                    agent_id: agent_id.clone(),
                    title: chat_thread_title(message.content.as_str(), message.time),
                    started_at: message.time,
                    updated_at: message.time,
                    messages: vec![message],
                });
                sequence += 1;
                continue;
            }

            if let Some(thread) = current_thread.as_mut() {
                thread.updated_at = message.time;
                thread.messages.push(message);
            }
        }

        if let Some(mut thread) = current_thread.take() {
            trim_messages_for_thread(&mut thread, message_limit);
            threads.push(thread);
        }
    }

    threads.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| right.started_at.cmp(&left.started_at))
    });
    if threads.len() > thread_limit {
        threads.truncate(thread_limit);
    }

    threads
}

fn trim_messages_for_thread(thread: &mut ChatThread, message_limit: usize) {
    if thread.messages.len() > message_limit {
        let overflow = thread.messages.len() - message_limit;
        thread.messages.drain(0..overflow);
    }
}

fn chat_thread_title(content: &str, time: u64) -> String {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return format!("Chat @ T{time}");
    }
    truncate_text(trimmed, CHAT_PREVIEW_CHARS)
}

fn truncate_text(content: &str, max_chars: usize) -> String {
    let mut chars = content.trim().chars();
    let mut preview = String::new();
    for _ in 0..max_chars {
        let Some(ch) = chars.next() else {
            return preview;
        };
        preview.push(ch);
    }

    if chars.next().is_some() {
        preview.push('…');
    }
    preview
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

#[cfg(test)]
mod tests {
    use super::*;
    use agent_world::simulator::{AgentDecision, AgentDecisionTrace};

    fn message(agent_id: &str, time: u64, role: LlmChatRole, content: &str) -> LlmChatMessageTrace {
        LlmChatMessageTrace {
            time,
            agent_id: agent_id.to_string(),
            role,
            content: content.to_string(),
        }
    }

    fn trace(agent_id: &str, time: u64, messages: Vec<LlmChatMessageTrace>) -> AgentDecisionTrace {
        AgentDecisionTrace {
            agent_id: agent_id.to_string(),
            time,
            decision: AgentDecision::Wait,
            llm_input: None,
            llm_output: None,
            llm_error: None,
            parse_error: None,
            llm_diagnostics: None,
            llm_effect_intents: Vec::new(),
            llm_effect_receipts: Vec::new(),
            llm_step_trace: Vec::new(),
            llm_prompt_section_trace: Vec::new(),
            llm_chat_messages: messages,
        }
    }

    fn viewer_state_with_traces(traces: Vec<AgentDecisionTrace>) -> ViewerState {
        ViewerState {
            status: crate::ConnectionStatus::Connected,
            snapshot: None,
            events: Vec::new(),
            decision_traces: traces,
            metrics: None,
        }
    }

    #[test]
    fn collect_chat_threads_splits_by_player_message() {
        let state = viewer_state_with_traces(vec![trace(
            "agent-a",
            4,
            vec![
                message("agent-a", 1, LlmChatRole::Player, "hello"),
                message("agent-a", 2, LlmChatRole::Agent, "ack"),
                message("agent-a", 3, LlmChatRole::Player, "next topic"),
                message("agent-a", 4, LlmChatRole::Agent, "done"),
            ],
        )]);

        let threads = collect_chat_threads(&state, 16, 16);
        assert_eq!(threads.len(), 2);
        assert_eq!(threads[0].messages.len(), 2);
        assert_eq!(threads[1].messages.len(), 2);
        assert_eq!(threads[0].messages[0].content, "next topic");
    }

    #[test]
    fn collect_chat_threads_orders_latest_first_across_agents() {
        let state = viewer_state_with_traces(vec![
            trace(
                "agent-a",
                4,
                vec![
                    message("agent-a", 1, LlmChatRole::Player, "a1"),
                    message("agent-a", 2, LlmChatRole::Agent, "a2"),
                ],
            ),
            trace(
                "agent-b",
                8,
                vec![
                    message("agent-b", 5, LlmChatRole::Player, "b1"),
                    message("agent-b", 8, LlmChatRole::Agent, "b2"),
                ],
            ),
        ]);

        let threads = collect_chat_threads(&state, 16, 16);
        assert_eq!(threads.len(), 2);
        assert_eq!(threads[0].agent_id, "agent-b");
        assert_eq!(threads[1].agent_id, "agent-a");
    }

    #[test]
    fn truncate_text_marks_ellipsis_when_exceeding_limit() {
        assert_eq!(truncate_text("abcdef", 3), "abc…");
        assert_eq!(truncate_text("abc", 3), "abc");
    }

    #[test]
    fn should_submit_chat_on_enter_requires_focus_and_no_modifiers() {
        assert!(should_submit_chat_on_enter(
            true,
            true,
            egui::Modifiers::default()
        ));
        assert!(!should_submit_chat_on_enter(
            false,
            true,
            egui::Modifiers::default()
        ));
        assert!(!should_submit_chat_on_enter(
            true,
            false,
            egui::Modifiers::default()
        ));

        let mut shift_mod = egui::Modifiers::default();
        shift_mod.shift = true;
        assert!(!should_submit_chat_on_enter(true, true, shift_mod));
    }

    #[test]
    fn parse_tool_call_view_reads_structured_payload() {
        let tool_message = message(
            "agent-a",
            10,
            LlmChatRole::Tool,
            r#"{"type":"module_call_result","module":"environment.current_observation","status":"ok","args":{"limit":3},"result":{"ok":true,"module":"environment.current_observation"}}"#,
        );

        let parsed = parse_tool_call_view(&tool_message);
        assert_eq!(parsed.module, "environment.current_observation");
        assert_eq!(parsed.status, "ok");
        assert!(parsed.args_preview.contains("\"limit\":3"));
        assert!(parsed.result_preview.contains("\"ok\":true"));
    }

    #[test]
    fn parse_tool_call_view_falls_back_to_legacy_text_format() {
        let tool_message = message(
            "agent-a",
            10,
            LlmChatRole::Tool,
            "module=agent.modules.list status=ok result={\"ok\":true}",
        );

        let parsed = parse_tool_call_view(&tool_message);
        assert_eq!(parsed.module, "agent.modules.list");
        assert_eq!(parsed.status, "ok");
        assert_eq!(parsed.args_preview, "-");
        assert!(parsed.result_preview.contains("\"ok\":true"));
    }
}
