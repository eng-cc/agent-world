use super::*;

pub(super) fn default_prompt_presets() -> Vec<PromptPresetDraft> {
    vec![
        PromptPresetDraft {
            name: "资源采集计划".to_string(),
            content: "先汇报当前可见资源与电力状态，再给出接下来3步最稳妥的资源采集计划。"
                .to_string(),
        },
        PromptPresetDraft {
            name: "制造优先级".to_string(),
            content: "请评估当前工厂链路瓶颈，并给出制造优先级和原因。".to_string(),
        },
        PromptPresetDraft {
            name: "异常排查".to_string(),
            content: "请检查当前失败动作和最近工具调用，给出最可能的失败根因与修复建议。"
                .to_string(),
        },
    ]
}

pub(super) fn sync_prompt_presets(draft: &mut AgentChatDraftState) {
    if draft.prompt_presets.is_empty() {
        draft.prompt_presets.push(PromptPresetDraft {
            name: "Preset 1".to_string(),
            content: String::new(),
        });
        draft.selected_preset_index = 0;
        return;
    }

    if draft.selected_preset_index >= draft.prompt_presets.len() {
        draft.selected_preset_index = draft.prompt_presets.len().saturating_sub(1);
    }
}

pub(super) fn next_preset_name(locale: crate::i18n::UiLocale, count: usize) -> String {
    if locale.is_zh() {
        format!("预设 {}", count + 1)
    } else {
        format!("Preset {}", count + 1)
    }
}

pub(super) fn selected_preset_label(draft: &AgentChatDraftState) -> String {
    draft
        .prompt_presets
        .get(draft.selected_preset_index)
        .map(|preset| preset.name.trim())
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("Preset {}", draft.selected_preset_index + 1))
}

pub(super) fn apply_selected_preset_to_input(draft: &mut AgentChatDraftState) -> bool {
    let Some(preset) = draft.prompt_presets.get(draft.selected_preset_index) else {
        return false;
    };
    let content = preset.content.trim();
    if content.is_empty() {
        return false;
    }
    draft.input_message = content.to_string();
    true
}

pub(super) fn prompt_preset_scroll_max_height(available_height: f32) -> f32 {
    if !available_height.is_finite() {
        return PROMPT_PRESET_SCROLL_MAX_HEIGHT;
    }
    available_height
        .max(0.0)
        .min(PROMPT_PRESET_SCROLL_MAX_HEIGHT)
}

pub(super) fn render_prompt_preset_editor(
    ui: &mut egui::Ui,
    locale: crate::i18n::UiLocale,
    state: &ViewerState,
    client: Option<&ViewerClient>,
    draft: &mut AgentChatDraftState,
    selected_agent_id: &str,
) {
    sync_prompt_presets(draft);
    let current_profile = current_prompt_profile_for_agent(state, selected_agent_id);
    load_profile_draft_if_needed(draft, selected_agent_id, &current_profile);

    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            let toggle_label = if locale.is_zh() {
                if draft.preset_panel_open {
                    "▼ 预设 Prompt"
                } else {
                    "▶ 预设 Prompt"
                }
            } else if draft.preset_panel_open {
                "▼ Prompt Presets"
            } else {
                "▶ Prompt Presets"
            };
            if ui.small_button(toggle_label).clicked() {
                draft.preset_panel_open = !draft.preset_panel_open;
            }

            ui.label(
                egui::RichText::new(if locale.is_zh() {
                    "可编辑并快速填充到输入框"
                } else {
                    "Edit and quickly fill chat input"
                })
                .size(11.0)
                .color(egui::Color32::from_gray(170)),
            );
        });

        if !draft.preset_panel_open {
            return;
        }

        ui.add_space(4.0);
        let preset_panel_max_height = prompt_preset_scroll_max_height(ui.available_height());
        egui::ScrollArea::vertical()
            .max_height(preset_panel_max_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let mut selected_index = draft.selected_preset_index;
                egui::ComboBox::from_label(if locale.is_zh() {
                    "预设项"
                } else {
                    "Preset"
                })
                .selected_text(selected_preset_label(draft))
                .show_ui(ui, |ui| {
                    for (index, preset) in draft.prompt_presets.iter().enumerate() {
                        let label = if preset.name.trim().is_empty() {
                            if locale.is_zh() {
                                format!("预设 {}", index + 1)
                            } else {
                                format!("Preset {}", index + 1)
                            }
                        } else {
                            preset.name.clone()
                        };
                        if ui
                            .selectable_label(selected_index == index, label.as_str())
                            .clicked()
                        {
                            selected_index = index;
                        }
                    }
                });
                draft.selected_preset_index = selected_index;

                let mut add_preset = false;
                let mut remove_preset = false;
                let mut fill_input = false;
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .small_button(if locale.is_zh() { "新增" } else { "Add" })
                        .clicked()
                    {
                        add_preset = true;
                    }
                    if ui
                        .small_button(if locale.is_zh() { "删除" } else { "Delete" })
                        .clicked()
                    {
                        remove_preset = true;
                    }
                    if ui
                        .small_button(if locale.is_zh() {
                            "填充到输入框"
                        } else {
                            "Fill Input"
                        })
                        .clicked()
                    {
                        fill_input = true;
                    }
                });

                if add_preset {
                    let next_name = next_preset_name(locale, draft.prompt_presets.len());
                    draft.prompt_presets.push(PromptPresetDraft {
                        name: next_name,
                        content: String::new(),
                    });
                    draft.selected_preset_index = draft.prompt_presets.len().saturating_sub(1);
                }
                if remove_preset && !draft.prompt_presets.is_empty() {
                    draft.prompt_presets.remove(draft.selected_preset_index);
                    sync_prompt_presets(draft);
                }
                if fill_input {
                    if apply_selected_preset_to_input(draft) {
                        draft.status_message = if locale.is_zh() {
                            "已将预设填充到输入框，可直接发送或继续修改。".to_string()
                        } else {
                            "Preset filled into input. You can send or keep editing.".to_string()
                        };
                    } else {
                        draft.status_message = if locale.is_zh() {
                            "当前预设内容为空，无法填充。".to_string()
                        } else {
                            "Selected preset is empty.".to_string()
                        };
                    }
                }

                if let Some(preset) = draft.prompt_presets.get_mut(draft.selected_preset_index) {
                    ui.label(if locale.is_zh() {
                        "预设名称"
                    } else {
                        "Preset Name"
                    });
                    ui.text_edit_singleline(&mut preset.name);

                    ui.label(if locale.is_zh() {
                        "预设内容"
                    } else {
                        "Preset Content"
                    });
                    ui.add(
                        egui::TextEdit::multiline(&mut preset.content)
                            .desired_rows(PROMPT_PRESET_DEFAULT_CONTENT_ROWS)
                            .hint_text(if locale.is_zh() {
                                "输入预设 prompt 内容"
                            } else {
                                "Type preset prompt content"
                            }),
                    );
                }
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                ui.horizontal_wrapped(|ui| {
                    ui.strong(if locale.is_zh() {
                        "Agent Prompt 草稿"
                    } else {
                        "Agent Prompt Draft"
                    });
                    ui.label(
                        egui::RichText::new(if locale.is_zh() {
                            format!(
                                "目标: {} 版本: {}",
                                selected_agent_id, current_profile.version
                            )
                        } else {
                            format!(
                                "Target: {} Version: {}",
                                selected_agent_id, current_profile.version
                            )
                        })
                        .size(10.5)
                        .color(egui::Color32::from_gray(170)),
                    );
                });

                let mut reload_profile = false;
                let mut apply_profile = false;
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .small_button(if locale.is_zh() {
                            "加载当前配置"
                        } else {
                            "Load Current"
                        })
                        .clicked()
                    {
                        reload_profile = true;
                    }
                    if ui
                        .small_button(if locale.is_zh() {
                            "应用到 Agent"
                        } else {
                            "Apply to Agent"
                        })
                        .clicked()
                    {
                        apply_profile = true;
                    }
                });

                if reload_profile {
                    load_profile_draft_from_profile(draft, selected_agent_id, &current_profile);
                    draft.status_message = if locale.is_zh() {
                        "已加载当前 Agent Prompt 配置。".to_string()
                    } else {
                        "Loaded current agent prompt profile.".to_string()
                    };
                }

                ui.label(if locale.is_zh() {
                    "System Prompt"
                } else {
                    "System Prompt"
                });
                ui.add(
                    egui::TextEdit::multiline(&mut draft.profile_system_prompt)
                        .desired_rows(PROMPT_PRESET_DEFAULT_CONTENT_ROWS)
                        .hint_text(DEFAULT_LLM_SYSTEM_PROMPT),
                );

                ui.label(if locale.is_zh() {
                    "短期目标"
                } else {
                    "Short-term Goal"
                });
                ui.add(
                    egui::TextEdit::multiline(&mut draft.profile_short_term_goal)
                        .desired_rows(3)
                        .hint_text(DEFAULT_LLM_SHORT_TERM_GOAL),
                );

                ui.label(if locale.is_zh() {
                    "长期目标"
                } else {
                    "Long-term Goal"
                });
                ui.add(
                    egui::TextEdit::multiline(&mut draft.profile_long_term_goal)
                        .desired_rows(3)
                        .hint_text(DEFAULT_LLM_LONG_TERM_GOAL),
                );

                if apply_profile {
                    match send_prompt_profile_apply_command(
                        client,
                        state,
                        selected_agent_id,
                        &current_profile,
                        draft,
                    ) {
                        Ok(()) => {
                            draft.status_message = if locale.is_zh() {
                                "Prompt 配置已提交，等待服务端应用。".to_string()
                            } else {
                                "Prompt profile request sent. Waiting for server apply.".to_string()
                            };
                            draft.profile_loaded_agent_id = Some(selected_agent_id.to_string());
                        }
                        Err(err) => {
                            draft.status_message = if locale.is_zh() {
                                format!("提交失败: {err}")
                            } else {
                                format!("Apply failed: {err}")
                            };
                        }
                    }
                }
            });
    });
}
