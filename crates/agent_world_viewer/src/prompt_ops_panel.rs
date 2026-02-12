use agent_world::simulator::{AgentPromptProfile, PromptUpdateOperation, WorldEventKind};
use agent_world::viewer::{
    PromptControlAck, PromptControlApplyRequest, PromptControlCommand, PromptControlError,
    PromptControlRollbackRequest, ViewerRequest,
};
use bevy_egui::egui;
use std::collections::BTreeSet;

use crate::{PromptControlUiState, ViewerClient, ViewerState};

const PROMPT_UPDATED_BY_VIEWER: &str = "agent_world_viewer";
const VALUE_PREVIEW_LIMIT: usize = 56;
const AUDIT_ROW_LIMIT: usize = 8;

#[derive(Default)]
pub(super) struct PromptOpsDraftState {
    selected_agent_id: Option<String>,
    loaded_agent_id: Option<String>,
    system_prompt: String,
    short_term_goal: String,
    long_term_goal: String,
    rollback_to_version: String,
    status_message: String,
    last_feedback_seq: u64,
}

pub(super) fn render_prompt_ops_section(
    ui: &mut egui::Ui,
    locale: crate::i18n::UiLocale,
    state: &ViewerState,
    prompt_ops_draft: &mut PromptOpsDraftState,
    prompt_control_state: Option<&PromptControlUiState>,
    client: Option<&ViewerClient>,
) {
    sync_prompt_ops_selected_agent(prompt_ops_draft, state);
    sync_prompt_control_feedback(prompt_ops_draft, prompt_control_state, locale);

    let title = if locale.is_zh() {
        "Prompt 运维"
    } else {
        "Prompt Ops"
    };
    ui.strong(title);

    let scope_note = if locale.is_zh() {
        "约束：仅支持修改 Agent Prompt，不提供玩家动作输入。"
    } else {
        "Constraint: prompt-only control. No direct player action input is allowed."
    };
    ui.add(egui::Label::new(scope_note).wrap().selectable(true));

    let agent_ids = collect_prompt_ops_agent_ids(state);
    let chips = [
        (
            if locale.is_zh() { "Agent" } else { "Agents" },
            agent_ids.len().to_string(),
        ),
        (
            if locale.is_zh() { "轨迹" } else { "Traces" },
            state.decision_traces.len().to_string(),
        ),
        (
            if locale.is_zh() { "连接" } else { "Link" },
            if client.is_some() {
                if locale.is_zh() {
                    "在线".to_string()
                } else {
                    "online".to_string()
                }
            } else if locale.is_zh() {
                "离线".to_string()
            } else {
                "offline".to_string()
            },
        ),
    ];

    ui.horizontal_wrapped(|ui| {
        for (label, value) in chips {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.small(label);
                ui.label(value);
            });
        }
    });

    ui.separator();
    ui.strong(if locale.is_zh() {
        "Agent 选择"
    } else {
        "Agent Target"
    });
    if agent_ids.is_empty() {
        prompt_ops_draft.loaded_agent_id = None;
        ui.label(if locale.is_zh() {
            "暂无可操作 Agent（等待快照或轨迹）。"
        } else {
            "No agent is available yet (waiting for snapshot/trace)."
        });
        return;
    }

    ui.horizontal_wrapped(|ui| {
        for agent_id in &agent_ids {
            let selected = prompt_ops_draft.selected_agent_id.as_deref() == Some(agent_id.as_str());
            if ui.selectable_label(selected, agent_id).clicked() {
                prompt_ops_draft.selected_agent_id = Some(agent_id.clone());
            }
        }
    });

    let selected_agent_id = prompt_ops_draft
        .selected_agent_id
        .clone()
        .unwrap_or_else(|| agent_ids[0].clone());
    if prompt_ops_draft.selected_agent_id.is_none() {
        prompt_ops_draft.selected_agent_id = Some(selected_agent_id.clone());
    }

    let current_profile = current_prompt_profile_for_agent(state, selected_agent_id.as_str());
    load_prompt_draft_if_needed(
        prompt_ops_draft,
        selected_agent_id.as_str(),
        &current_profile,
    );

    let latest_tick = state
        .decision_traces
        .iter()
        .rev()
        .find(|trace| trace.agent_id == selected_agent_id)
        .map(|trace| trace.time);
    let latest_tick_label = latest_tick.map(|tick| tick.to_string()).unwrap_or_else(|| {
        if locale.is_zh() {
            "(无轨迹)".to_string()
        } else {
            "(no trace)".to_string()
        }
    });

    ui.add(
        egui::Label::new(if locale.is_zh() {
            format!(
                "目标: {selected_agent_id} 最近轨迹 tick: {latest_tick_label} 当前版本: {}",
                current_profile.version
            )
        } else {
            format!(
                "Target: {selected_agent_id} latest trace tick: {latest_tick_label} current version: {}",
                current_profile.version
            )
        })
        .selectable(true),
    );

    ui.separator();
    ui.strong(if locale.is_zh() {
        "Prompt 草稿"
    } else {
        "Prompt Draft"
    });

    if ui
        .button(if locale.is_zh() {
            "载入当前"
        } else {
            "Load Current"
        })
        .clicked()
    {
        load_prompt_draft_from_profile(
            prompt_ops_draft,
            selected_agent_id.as_str(),
            &current_profile,
        );
        prompt_ops_draft.status_message = if locale.is_zh() {
            format!(
                "已加载 {selected_agent_id} 当前版本 {}",
                current_profile.version
            )
        } else {
            format!(
                "Loaded current profile for {selected_agent_id} at version {}",
                current_profile.version
            )
        };
    }

    ui.label("system prompt");
    ui.add(
        egui::TextEdit::multiline(&mut prompt_ops_draft.system_prompt)
            .desired_rows(4)
            .hint_text(if locale.is_zh() {
                "输入 system prompt 覆盖草稿"
            } else {
                "Type a system prompt override draft"
            }),
    );

    ui.label("short-term goal");
    ui.add(
        egui::TextEdit::multiline(&mut prompt_ops_draft.short_term_goal)
            .desired_rows(2)
            .hint_text(if locale.is_zh() {
                "输入短期目标草稿"
            } else {
                "Type a short-term goal draft"
            }),
    );

    ui.label("long-term goal");
    ui.add(
        egui::TextEdit::multiline(&mut prompt_ops_draft.long_term_goal)
            .desired_rows(2)
            .hint_text(if locale.is_zh() {
                "输入长期目标草稿"
            } else {
                "Type a long-term goal draft"
            }),
    );

    ui.separator();
    ui.strong(if locale.is_zh() {
        "差异预览"
    } else {
        "Diff Preview"
    });
    let diff_rows = collect_prompt_diff_rows(&current_profile, prompt_ops_draft);
    if diff_rows.is_empty() {
        ui.label(if locale.is_zh() {
            "当前草稿与生效配置一致。"
        } else {
            "Draft matches the active profile."
        });
    } else {
        for row in diff_rows {
            let line = if locale.is_zh() {
                format!(
                    "{}: {} -> {}",
                    prompt_field_label(row.field, locale),
                    prompt_value_preview(row.current.as_deref(), locale),
                    prompt_value_preview(row.next.as_deref(), locale)
                )
            } else {
                format!(
                    "{}: {} -> {}",
                    prompt_field_label(row.field, locale),
                    prompt_value_preview(row.current.as_deref(), locale),
                    prompt_value_preview(row.next.as_deref(), locale)
                )
            };
            ui.add(egui::Label::new(line).wrap().selectable(true));
        }
    }

    ui.horizontal_wrapped(|ui| {
        ui.label(if locale.is_zh() {
            "回滚到版本"
        } else {
            "Rollback to"
        });
        ui.add(
            egui::TextEdit::singleline(&mut prompt_ops_draft.rollback_to_version)
                .desired_width(90.0)
                .hint_text("0"),
        );
    });

    ui.horizontal_wrapped(|ui| {
        if ui
            .button(if locale.is_zh() { "预览" } else { "Preview" })
            .clicked()
        {
            let message = match client {
                Some(client) => {
                    match send_prompt_apply_command(
                        client,
                        selected_agent_id.as_str(),
                        &current_profile,
                        prompt_ops_draft,
                        true,
                    ) {
                        Ok(_) => {
                            if locale.is_zh() {
                                "已发送预览请求，等待服务端回执。".to_string()
                            } else {
                                "Preview request sent, waiting for server ack.".to_string()
                            }
                        }
                        Err(err) => {
                            if locale.is_zh() {
                                format!("发送预览失败: {err}")
                            } else {
                                format!("Failed to send preview request: {err}")
                            }
                        }
                    }
                }
                None => {
                    if locale.is_zh() {
                        "离线模式不可执行预览。".to_string()
                    } else {
                        "Preview is unavailable in offline mode.".to_string()
                    }
                }
            };
            prompt_ops_draft.status_message = message;
        }

        if ui
            .button(if locale.is_zh() { "提交" } else { "Apply" })
            .clicked()
        {
            let message = match client {
                Some(client) => {
                    match send_prompt_apply_command(
                        client,
                        selected_agent_id.as_str(),
                        &current_profile,
                        prompt_ops_draft,
                        false,
                    ) {
                        Ok(_) => {
                            if locale.is_zh() {
                                "已发送提交请求，等待服务端回执。".to_string()
                            } else {
                                "Apply request sent, waiting for server ack.".to_string()
                            }
                        }
                        Err(err) => {
                            if locale.is_zh() {
                                format!("发送提交失败: {err}")
                            } else {
                                format!("Failed to send apply request: {err}")
                            }
                        }
                    }
                }
                None => {
                    if locale.is_zh() {
                        "离线模式不可执行提交。".to_string()
                    } else {
                        "Apply is unavailable in offline mode.".to_string()
                    }
                }
            };
            prompt_ops_draft.status_message = message;
        }

        if ui
            .button(if locale.is_zh() { "回滚" } else { "Rollback" })
            .clicked()
        {
            let message = match client {
                Some(client) => match send_prompt_rollback_command(
                    client,
                    selected_agent_id.as_str(),
                    &current_profile,
                    prompt_ops_draft.rollback_to_version.trim(),
                ) {
                    Ok(_) => {
                        if locale.is_zh() {
                            "已发送回滚请求，等待服务端回执。".to_string()
                        } else {
                            "Rollback request sent, waiting for server ack.".to_string()
                        }
                    }
                    Err(err) => {
                        if locale.is_zh() {
                            format!("发送回滚失败: {err}")
                        } else {
                            format!("Failed to send rollback request: {err}")
                        }
                    }
                },
                None => {
                    if locale.is_zh() {
                        "离线模式不可执行回滚。".to_string()
                    } else {
                        "Rollback is unavailable in offline mode.".to_string()
                    }
                }
            };
            prompt_ops_draft.status_message = message;
        }
    });

    if !prompt_ops_draft.status_message.is_empty() {
        ui.add(
            egui::Label::new(prompt_ops_draft.status_message.as_str())
                .wrap()
                .selectable(true),
        );
    }

    ui.separator();
    ui.strong(if locale.is_zh() {
        "变更审计"
    } else {
        "Audit Trail"
    });
    let audit_entries =
        collect_prompt_audit_entries(state, selected_agent_id.as_str(), AUDIT_ROW_LIMIT);
    if audit_entries.is_empty() {
        ui.label(if locale.is_zh() {
            "暂无 AgentPromptUpdated 事件。"
        } else {
            "No AgentPromptUpdated events yet."
        });
    } else {
        for entry in audit_entries {
            let fields = if entry.applied_fields.is_empty() {
                if locale.is_zh() {
                    "(无字段变化)".to_string()
                } else {
                    "(no fields)".to_string()
                }
            } else {
                entry.applied_fields.join(",")
            };
            let op = match entry.operation {
                PromptUpdateOperation::Apply => {
                    if locale.is_zh() {
                        "apply"
                    } else {
                        "apply"
                    }
                }
                PromptUpdateOperation::Rollback => {
                    if locale.is_zh() {
                        "rollback"
                    } else {
                        "rollback"
                    }
                }
            };
            let rollback_suffix = entry
                .rolled_back_to_version
                .map(|version| {
                    if locale.is_zh() {
                        format!(" to={version}")
                    } else {
                        format!(" to={version}")
                    }
                })
                .unwrap_or_default();
            let line = if locale.is_zh() {
                format!(
                    "tick={} v{} {}{} fields={} digest={}",
                    entry.tick,
                    entry.version,
                    op,
                    rollback_suffix,
                    fields,
                    short_digest(entry.digest.as_str())
                )
            } else {
                format!(
                    "tick={} v{} {}{} fields={} digest={}",
                    entry.tick,
                    entry.version,
                    op,
                    rollback_suffix,
                    fields,
                    short_digest(entry.digest.as_str())
                )
            };
            ui.add(egui::Label::new(line).wrap().selectable(true));
        }
    }
}

pub(super) fn collect_prompt_ops_agent_ids(state: &ViewerState) -> Vec<String> {
    let mut ids = BTreeSet::new();

    if let Some(snapshot) = state.snapshot.as_ref() {
        for agent_id in snapshot.model.agents.keys() {
            ids.insert(agent_id.clone());
        }
    }

    for trace in &state.decision_traces {
        ids.insert(trace.agent_id.clone());
    }

    for event in &state.events {
        if let WorldEventKind::AgentPromptUpdated { profile, .. } = &event.kind {
            ids.insert(profile.agent_id.clone());
        }
    }

    ids.into_iter().collect()
}

fn sync_prompt_ops_selected_agent(prompt_ops_draft: &mut PromptOpsDraftState, state: &ViewerState) {
    let ids = collect_prompt_ops_agent_ids(state);
    if ids.is_empty() {
        prompt_ops_draft.selected_agent_id = None;
        return;
    }

    let already_selected = prompt_ops_draft.selected_agent_id.clone();
    let selected_exists = already_selected
        .as_deref()
        .map(|agent_id| ids.iter().any(|id| id == agent_id))
        .unwrap_or(false);

    if !selected_exists {
        prompt_ops_draft.selected_agent_id = Some(ids[0].clone());
    }
}

fn sync_prompt_control_feedback(
    prompt_ops_draft: &mut PromptOpsDraftState,
    prompt_control_state: Option<&PromptControlUiState>,
    locale: crate::i18n::UiLocale,
) {
    let Some(prompt_control_state) = prompt_control_state else {
        return;
    };

    if prompt_control_state.response_seq() == prompt_ops_draft.last_feedback_seq {
        return;
    }
    prompt_ops_draft.last_feedback_seq = prompt_control_state.response_seq();

    if let Some(error) = prompt_control_state.last_error() {
        prompt_ops_draft.status_message = format_prompt_error_message(error, locale);
        return;
    }
    if let Some(ack) = prompt_control_state.last_ack() {
        prompt_ops_draft.status_message = format_prompt_ack_message(ack, locale);
        if !ack.preview {
            prompt_ops_draft.rollback_to_version = ack.version.saturating_sub(1).to_string();
        }
    }
}

fn format_prompt_ack_message(ack: &PromptControlAck, locale: crate::i18n::UiLocale) -> String {
    let fields = if ack.applied_fields.is_empty() {
        if locale.is_zh() {
            "(无字段变化)".to_string()
        } else {
            "(no fields)".to_string()
        }
    } else {
        ack.applied_fields.join(",")
    };

    if ack.preview {
        if locale.is_zh() {
            format!(
                "预览成功: agent={} next_version={} fields={} digest={}",
                ack.agent_id,
                ack.version,
                fields,
                short_digest(ack.digest.as_str())
            )
        } else {
            format!(
                "Preview OK: agent={} next_version={} fields={} digest={}",
                ack.agent_id,
                ack.version,
                fields,
                short_digest(ack.digest.as_str())
            )
        }
    } else {
        let op = match ack.operation {
            agent_world::viewer::PromptControlOperation::Apply => {
                if locale.is_zh() {
                    "apply"
                } else {
                    "apply"
                }
            }
            agent_world::viewer::PromptControlOperation::Rollback => {
                if locale.is_zh() {
                    "rollback"
                } else {
                    "rollback"
                }
            }
        };
        let rollback_suffix = ack
            .rolled_back_to_version
            .map(|version| format!(" to={version}"))
            .unwrap_or_default();
        if locale.is_zh() {
            format!(
                "提交成功: agent={} op={}{} version={} fields={} digest={}",
                ack.agent_id,
                op,
                rollback_suffix,
                ack.version,
                fields,
                short_digest(ack.digest.as_str())
            )
        } else {
            format!(
                "Apply OK: agent={} op={}{} version={} fields={} digest={}",
                ack.agent_id,
                op,
                rollback_suffix,
                ack.version,
                fields,
                short_digest(ack.digest.as_str())
            )
        }
    }
}

fn format_prompt_error_message(
    error: &PromptControlError,
    locale: crate::i18n::UiLocale,
) -> String {
    let agent = error.agent_id.as_deref().unwrap_or("-");
    let current_version = error
        .current_version
        .map(|version| version.to_string())
        .unwrap_or_else(|| "-".to_string());

    if locale.is_zh() {
        format!(
            "请求失败: code={} agent={} current={} msg={}",
            error.code, agent, current_version, error.message
        )
    } else {
        format!(
            "Request failed: code={} agent={} current={} msg={}",
            error.code, agent, current_version, error.message
        )
    }
}

fn load_prompt_draft_if_needed(
    prompt_ops_draft: &mut PromptOpsDraftState,
    selected_agent_id: &str,
    profile: &AgentPromptProfile,
) {
    if prompt_ops_draft.loaded_agent_id.as_deref() == Some(selected_agent_id) {
        return;
    }

    load_prompt_draft_from_profile(prompt_ops_draft, selected_agent_id, profile);
}

fn load_prompt_draft_from_profile(
    prompt_ops_draft: &mut PromptOpsDraftState,
    selected_agent_id: &str,
    profile: &AgentPromptProfile,
) {
    prompt_ops_draft.system_prompt = profile.system_prompt_override.clone().unwrap_or_default();
    prompt_ops_draft.short_term_goal = profile.short_term_goal_override.clone().unwrap_or_default();
    prompt_ops_draft.long_term_goal = profile.long_term_goal_override.clone().unwrap_or_default();
    prompt_ops_draft.rollback_to_version = profile.version.saturating_sub(1).to_string();
    prompt_ops_draft.loaded_agent_id = Some(selected_agent_id.to_string());
}

fn send_prompt_apply_command(
    client: &ViewerClient,
    selected_agent_id: &str,
    current_profile: &AgentPromptProfile,
    prompt_ops_draft: &PromptOpsDraftState,
    preview: bool,
) -> Result<(), String> {
    let request = build_prompt_apply_request(selected_agent_id, current_profile, prompt_ops_draft);
    let command = if preview {
        PromptControlCommand::Preview { request }
    } else {
        PromptControlCommand::Apply { request }
    };

    client
        .tx
        .send(ViewerRequest::PromptControl { command })
        .map_err(|err| err.to_string())
}

fn send_prompt_rollback_command(
    client: &ViewerClient,
    selected_agent_id: &str,
    current_profile: &AgentPromptProfile,
    rollback_to_version_raw: &str,
) -> Result<(), String> {
    let rollback_to_version = rollback_to_version_raw
        .parse::<u64>()
        .map_err(|_| format!("invalid rollback version: {rollback_to_version_raw}"))?;

    let request = PromptControlRollbackRequest {
        agent_id: selected_agent_id.to_string(),
        to_version: rollback_to_version,
        expected_version: Some(current_profile.version),
        updated_by: Some(PROMPT_UPDATED_BY_VIEWER.to_string()),
    };

    client
        .tx
        .send(ViewerRequest::PromptControl {
            command: PromptControlCommand::Rollback { request },
        })
        .map_err(|err| err.to_string())
}

fn build_prompt_apply_request(
    selected_agent_id: &str,
    current_profile: &AgentPromptProfile,
    prompt_ops_draft: &PromptOpsDraftState,
) -> PromptControlApplyRequest {
    let next_system = normalize_prompt_text(prompt_ops_draft.system_prompt.as_str());
    let next_short = normalize_prompt_text(prompt_ops_draft.short_term_goal.as_str());
    let next_long = normalize_prompt_text(prompt_ops_draft.long_term_goal.as_str());

    PromptControlApplyRequest {
        agent_id: selected_agent_id.to_string(),
        expected_version: Some(current_profile.version),
        updated_by: Some(PROMPT_UPDATED_BY_VIEWER.to_string()),
        system_prompt_override: patch_override(
            current_profile.system_prompt_override.as_ref(),
            next_system.as_ref(),
        ),
        short_term_goal_override: patch_override(
            current_profile.short_term_goal_override.as_ref(),
            next_short.as_ref(),
        ),
        long_term_goal_override: patch_override(
            current_profile.long_term_goal_override.as_ref(),
            next_long.as_ref(),
        ),
    }
}

fn patch_override(current: Option<&String>, next: Option<&String>) -> Option<Option<String>> {
    if current.map(|value| value.as_str()) == next.map(|value| value.as_str()) {
        None
    } else {
        Some(next.cloned())
    }
}

fn normalize_prompt_text(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PromptDiffRow {
    field: &'static str,
    current: Option<String>,
    next: Option<String>,
}

fn collect_prompt_diff_rows(
    current_profile: &AgentPromptProfile,
    prompt_ops_draft: &PromptOpsDraftState,
) -> Vec<PromptDiffRow> {
    let mut rows = Vec::new();

    let next_system = normalize_prompt_text(prompt_ops_draft.system_prompt.as_str());
    if current_profile.system_prompt_override != next_system {
        rows.push(PromptDiffRow {
            field: "system_prompt_override",
            current: current_profile.system_prompt_override.clone(),
            next: next_system,
        });
    }

    let next_short = normalize_prompt_text(prompt_ops_draft.short_term_goal.as_str());
    if current_profile.short_term_goal_override != next_short {
        rows.push(PromptDiffRow {
            field: "short_term_goal_override",
            current: current_profile.short_term_goal_override.clone(),
            next: next_short,
        });
    }

    let next_long = normalize_prompt_text(prompt_ops_draft.long_term_goal.as_str());
    if current_profile.long_term_goal_override != next_long {
        rows.push(PromptDiffRow {
            field: "long_term_goal_override",
            current: current_profile.long_term_goal_override.clone(),
            next: next_long,
        });
    }

    rows
}

fn prompt_field_label(field: &str, locale: crate::i18n::UiLocale) -> &'static str {
    match field {
        "system_prompt_override" => {
            if locale.is_zh() {
                "system"
            } else {
                "system"
            }
        }
        "short_term_goal_override" => {
            if locale.is_zh() {
                "short"
            } else {
                "short"
            }
        }
        "long_term_goal_override" => {
            if locale.is_zh() {
                "long"
            } else {
                "long"
            }
        }
        _ => "unknown",
    }
}

fn prompt_value_preview(value: Option<&str>, locale: crate::i18n::UiLocale) -> String {
    let Some(value) = value else {
        return if locale.is_zh() {
            "(空)".to_string()
        } else {
            "(empty)".to_string()
        };
    };

    let compact = value.replace('\n', " ");
    if compact.chars().count() <= VALUE_PREVIEW_LIMIT {
        compact
    } else {
        let mut output = compact
            .chars()
            .take(VALUE_PREVIEW_LIMIT.saturating_sub(1))
            .collect::<String>();
        output.push('…');
        output
    }
}

fn current_prompt_profile_for_agent(state: &ViewerState, agent_id: &str) -> AgentPromptProfile {
    for event in state.events.iter().rev() {
        let WorldEventKind::AgentPromptUpdated { profile, .. } = &event.kind else {
            continue;
        };
        if profile.agent_id == agent_id {
            return profile.clone();
        }
    }

    state
        .snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.model.agent_prompt_profiles.get(agent_id).cloned())
        .unwrap_or_else(|| AgentPromptProfile::for_agent(agent_id.to_string()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PromptAuditEntry {
    tick: u64,
    version: u64,
    operation: PromptUpdateOperation,
    applied_fields: Vec<String>,
    digest: String,
    rolled_back_to_version: Option<u64>,
}

fn collect_prompt_audit_entries(
    state: &ViewerState,
    selected_agent_id: &str,
    limit: usize,
) -> Vec<PromptAuditEntry> {
    let mut entries = Vec::new();

    for event in state.events.iter().rev() {
        let WorldEventKind::AgentPromptUpdated {
            profile,
            operation,
            applied_fields,
            digest,
            rolled_back_to_version,
        } = &event.kind
        else {
            continue;
        };

        if profile.agent_id != selected_agent_id {
            continue;
        }

        entries.push(PromptAuditEntry {
            tick: event.time,
            version: profile.version,
            operation: *operation,
            applied_fields: applied_fields.clone(),
            digest: digest.clone(),
            rolled_back_to_version: *rolled_back_to_version,
        });

        if entries.len() >= limit {
            break;
        }
    }

    entries
}

fn short_digest(digest: &str) -> String {
    digest.chars().take(12).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConnectionStatus, ViewerState};

    fn sample_prompt_event(
        tick: u64,
        agent_id: &str,
        version: u64,
        operation: PromptUpdateOperation,
    ) -> agent_world::simulator::WorldEvent {
        agent_world::simulator::WorldEvent {
            id: tick,
            time: tick,
            kind: WorldEventKind::AgentPromptUpdated {
                profile: AgentPromptProfile {
                    agent_id: agent_id.to_string(),
                    version,
                    updated_at_tick: tick,
                    updated_by: "tester".to_string(),
                    system_prompt_override: Some(format!("system-{version}")),
                    short_term_goal_override: None,
                    long_term_goal_override: None,
                },
                operation,
                applied_fields: vec!["system_prompt_override".to_string()],
                digest: format!("digest-{tick}"),
                rolled_back_to_version: if operation == PromptUpdateOperation::Rollback {
                    Some(version.saturating_sub(1))
                } else {
                    None
                },
            },
        }
    }

    #[test]
    fn build_prompt_apply_request_respects_tristate_patch() {
        let current = AgentPromptProfile {
            agent_id: "agent-0".to_string(),
            system_prompt_override: Some("keep".to_string()),
            short_term_goal_override: None,
            long_term_goal_override: Some("remove".to_string()),
            version: 9,
            updated_at_tick: 0,
            updated_by: "x".to_string(),
        };

        let draft = PromptOpsDraftState {
            system_prompt: "keep".to_string(),
            short_term_goal: "  new goal ".to_string(),
            long_term_goal: "\n\t".to_string(),
            ..Default::default()
        };

        let request = build_prompt_apply_request("agent-0", &current, &draft);
        assert_eq!(request.expected_version, Some(9));
        assert_eq!(request.system_prompt_override, None);
        assert_eq!(
            request.short_term_goal_override,
            Some(Some("new goal".to_string()))
        );
        assert_eq!(request.long_term_goal_override, Some(None));
    }

    #[test]
    fn collect_prompt_audit_entries_filters_by_agent_and_orders_latest_first() {
        let state = ViewerState {
            status: ConnectionStatus::Connected,
            snapshot: None,
            events: vec![
                sample_prompt_event(1, "agent-1", 1, PromptUpdateOperation::Apply),
                sample_prompt_event(2, "agent-0", 1, PromptUpdateOperation::Apply),
                sample_prompt_event(3, "agent-0", 2, PromptUpdateOperation::Rollback),
            ],
            decision_traces: Vec::new(),
            metrics: None,
        };

        let entries = collect_prompt_audit_entries(&state, "agent-0", 8);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].tick, 3);
        assert_eq!(entries[0].version, 2);
        assert_eq!(entries[1].tick, 2);
        assert_eq!(entries[1].version, 1);
    }

    #[test]
    fn collect_prompt_ops_agent_ids_includes_prompt_event_agents() {
        let state = ViewerState {
            status: ConnectionStatus::Connected,
            snapshot: None,
            events: vec![sample_prompt_event(
                1,
                "agent-9",
                1,
                PromptUpdateOperation::Apply,
            )],
            decision_traces: Vec::new(),
            metrics: None,
        };

        let ids = collect_prompt_ops_agent_ids(&state);
        assert_eq!(ids, vec!["agent-9".to_string()]);
    }
}
