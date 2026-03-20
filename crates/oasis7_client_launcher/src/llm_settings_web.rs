use eframe::egui;
use serde::{Deserialize, Serialize};

use super::{LaunchConfig, UiLanguage};

const LLM_SETTINGS_STORAGE_KEY: &str = "oasis7_client_launcher.llm_settings.v1";

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
struct LlmSettingsDraft {
    api_key: String,
    base_url: String,
    model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LlmSettingsStatus {
    None,
    Success(String),
    Failed(String),
}

#[derive(Debug)]
pub(crate) struct LlmSettingsPanel {
    draft: LlmSettingsDraft,
    status: LlmSettingsStatus,
    open: bool,
}

impl LlmSettingsPanel {
    pub(crate) fn new(_path: impl Into<std::path::PathBuf>) -> Self {
        let mut panel = Self {
            draft: LlmSettingsDraft::default(),
            status: LlmSettingsStatus::None,
            open: false,
        };
        panel.reload_from_storage();
        panel
    }

    pub(crate) fn default_path() -> &'static str {
        "browser:local_storage"
    }

    pub(crate) fn open(&mut self) {
        self.open = true;
        self.reload_from_storage();
    }

    pub(crate) fn show(
        &mut self,
        ctx: &egui::Context,
        language: UiLanguage,
        config: &mut LaunchConfig,
    ) {
        if !self.open {
            return;
        }

        let mut open = self.open;
        let mut save_clicked = false;
        let mut reload_clicked = false;

        egui::Window::new(tr(language, "设置中心", "Settings Center"))
            .open(&mut open)
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading(tr(language, "启动器配置", "Launcher Configuration"));
                ui.separator();

                ui.collapsing(tr(language, "游戏与显示", "Game & Viewer"), |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(tr(language, "场景", "Scenario"));
                        ui.text_edit_singleline(&mut config.scenario);
                        ui.label(tr(language, "实时服务绑定", "Live Bind"));
                        ui.text_edit_singleline(&mut config.live_bind);
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label(tr(language, "WebSocket 绑定", "Web Bind"));
                        ui.text_edit_singleline(&mut config.web_bind);
                        ui.label(tr(language, "游戏页面主机", "Viewer Host"));
                        ui.text_edit_singleline(&mut config.viewer_host);
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.label(tr(language, "游戏页面端口", "Viewer Port"));
                        ui.text_edit_singleline(&mut config.viewer_port);
                        ui.label(tr(language, "前端静态资源目录", "Viewer Static Directory"));
                        ui.text_edit_singleline(&mut config.viewer_static_dir);
                    });
                    ui.horizontal_wrapped(|ui| {
                        ui.checkbox(
                            &mut config.llm_enabled,
                            tr(language, "启用 LLM", "Enable LLM"),
                        );
                        ui.checkbox(
                            &mut config.auto_open_browser,
                            tr(language, "自动打开浏览器", "Open Browser Automatically"),
                        );
                    });
                });

                ui.separator();
                ui.collapsing(
                    tr(language, "区块链运行时", "Blockchain Runtime"),
                    |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.checkbox(
                                &mut config.chain_enabled,
                                tr(language, "启用链运行时", "Enable Chain Runtime"),
                            );
                            ui.label(tr(language, "链状态服务绑定", "Chain Status Bind"));
                            ui.text_edit_singleline(&mut config.chain_status_bind);
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.label(tr(language, "链节点 ID", "Chain Node ID"));
                            ui.text_edit_singleline(&mut config.chain_node_id);
                            ui.label(tr(language, "链世界 ID", "Chain World ID"));
                            ui.text_edit_singleline(&mut config.chain_world_id);
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.label(tr(language, "链节点角色", "Chain Role"));
                            ui.text_edit_singleline(&mut config.chain_node_role);
                            ui.label(tr(
                                language,
                                "链轮询间隔毫秒",
                                "Chain Worker Poll/Fallback Milliseconds",
                            ));
                            ui.text_edit_singleline(&mut config.chain_node_tick_ms);
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.label(tr(
                                language,
                                "PoS 槽时长毫秒",
                                "PoS Slot Duration Milliseconds",
                            ));
                            ui.text_edit_singleline(&mut config.chain_pos_slot_duration_ms);
                            ui.label(tr(language, "PoS 每槽 Tick 数", "PoS Ticks Per Slot"));
                            ui.text_edit_singleline(&mut config.chain_pos_ticks_per_slot);
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.label(tr(
                                language,
                                "PoS 提案 Tick 相位",
                                "PoS Proposal Tick Phase",
                            ));
                            ui.text_edit_singleline(&mut config.chain_pos_proposal_tick_phase);
                            ui.label(tr(language, "PoS 过旧槽滞后上限", "PoS Max Past Slot Lag"));
                            ui.text_edit_singleline(&mut config.chain_pos_max_past_slot_lag);
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.checkbox(
                                &mut config.chain_pos_adaptive_tick_scheduler_enabled,
                                tr(
                                    language,
                                    "启用 PoS 自适应 Tick 调度",
                                    "Enable PoS Adaptive Tick Scheduler",
                                ),
                            );
                            ui.label(tr(
                                language,
                                "PoS 槽时钟起点 Unix 毫秒（可留空）",
                                "PoS Slot Clock Genesis Unix Ms (optional)",
                            ));
                            ui.text_edit_singleline(
                                &mut config.chain_pos_slot_clock_genesis_unix_ms,
                            );
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.label(tr(language, "链验证者", "Chain Validators"));
                            ui.text_edit_singleline(&mut config.chain_node_validators);
                        });
                    },
                );

                ui.separator();
                ui.heading(tr(language, "LLM 连接配置", "LLM Connection"));
                ui.label(format!(
                    "{}: {}",
                    tr(language, "存储键", "Storage Key"),
                    LLM_SETTINGS_STORAGE_KEY
                ));
                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    ui.label("API Key");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.draft.api_key)
                            .desired_width(480.0)
                            .password(true),
                    );
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label("Base URL");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.draft.base_url).desired_width(480.0),
                    );
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label("Model");
                    ui.add(egui::TextEdit::singleline(&mut self.draft.model).desired_width(480.0));
                });

                ui.horizontal(|ui| {
                    if ui
                        .button(tr(language, "保存到浏览器", "Save to Browser"))
                        .clicked()
                    {
                        save_clicked = true;
                    }
                    if ui
                        .button(tr(language, "从浏览器重载", "Reload from Browser"))
                        .clicked()
                    {
                        reload_clicked = true;
                    }
                });

                match &self.status {
                    LlmSettingsStatus::Success(message) => {
                        ui.small(
                            egui::RichText::new(message.as_str())
                                .color(egui::Color32::from_rgb(62, 152, 92)),
                        );
                    }
                    LlmSettingsStatus::Failed(message) => {
                        ui.small(
                            egui::RichText::new(message.as_str())
                                .color(egui::Color32::from_rgb(196, 84, 84)),
                        );
                    }
                    LlmSettingsStatus::None => {}
                }
            });

        self.open = open;
        if save_clicked {
            self.save_to_storage(language);
        }
        if reload_clicked {
            self.reload_from_storage();
            self.status = LlmSettingsStatus::Success(
                tr(
                    language,
                    "已从浏览器重载 LLM 设置",
                    "Reloaded LLM settings from browser storage",
                )
                .to_string(),
            );
        }
    }

    fn reload_from_storage(&mut self) {
        match load_llm_settings_from_storage() {
            Ok(settings) => {
                self.draft = settings;
            }
            Err(err) => {
                self.status = LlmSettingsStatus::Failed(err);
            }
        }
    }

    fn save_to_storage(&mut self, language: UiLanguage) {
        match save_llm_settings_to_storage(&self.draft) {
            Ok(()) => {
                self.status = LlmSettingsStatus::Success(
                    tr(language, "LLM 配置已保存", "LLM settings saved").to_string(),
                );
            }
            Err(err) => {
                self.status = LlmSettingsStatus::Failed(format!(
                    "{}: {err}",
                    tr(language, "LLM 配置保存失败", "Failed to save LLM settings")
                ));
            }
        }
    }
}

fn tr(language: UiLanguage, zh: &'static str, en: &'static str) -> &'static str {
    match language {
        UiLanguage::ZhCn => zh,
        UiLanguage::EnUs => en,
    }
}

fn load_llm_settings_from_storage() -> Result<LlmSettingsDraft, String> {
    let storage = browser_storage()?;
    let raw = storage
        .get_item(LLM_SETTINGS_STORAGE_KEY)
        .map_err(|err| format!("read browser storage failed: {err:?}"))?;
    let Some(raw) = raw else {
        return Ok(LlmSettingsDraft::default());
    };
    if raw.trim().is_empty() {
        return Ok(LlmSettingsDraft::default());
    }
    serde_json::from_str::<LlmSettingsDraft>(raw.as_str())
        .map_err(|err| format!("parse browser storage value failed: {err}"))
}

fn save_llm_settings_to_storage(draft: &LlmSettingsDraft) -> Result<(), String> {
    let storage = browser_storage()?;
    let content =
        serde_json::to_string(draft).map_err(|err| format!("serialize settings failed: {err}"))?;
    storage
        .set_item(LLM_SETTINGS_STORAGE_KEY, content.as_str())
        .map_err(|err| format!("write browser storage failed: {err:?}"))
}

fn browser_storage() -> Result<web_sys::Storage, String> {
    let window = web_sys::window().ok_or_else(|| "window is unavailable".to_string())?;
    window
        .local_storage()
        .map_err(|err| format!("resolve browser localStorage failed: {err:?}"))?
        .ok_or_else(|| "browser localStorage is unavailable".to_string())
}
