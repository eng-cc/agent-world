use super::*;
#[cfg(not(target_arch = "wasm32"))]
use agent_world_launcher_ui::launcher_ui_fields_for_native;
#[cfg(target_arch = "wasm32")]
use agent_world_launcher_ui::launcher_ui_fields_for_web;
use agent_world_launcher_ui::{LauncherUiField, LauncherUiFieldKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StartupGuideTarget {
    Game,
    Chain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct StartupGuideState {
    pub(super) open: bool,
    pub(super) target: StartupGuideTarget,
    pub(super) first_check_done: bool,
}

impl Default for StartupGuideState {
    fn default() -> Self {
        Self {
            open: false,
            target: StartupGuideTarget::Game,
            first_check_done: false,
        }
    }
}

pub(super) fn issue_field_ids(issue: ConfigIssue) -> &'static [&'static str] {
    match issue {
        ConfigIssue::ScenarioRequired => &["scenario"],
        ConfigIssue::LiveBindInvalid => &["live_bind"],
        ConfigIssue::WebBindInvalid => &["web_bind"],
        ConfigIssue::ViewerHostRequired => &["viewer_host"],
        ConfigIssue::ViewerPortInvalid => &["viewer_port"],
        ConfigIssue::ViewerStaticDirRequired | ConfigIssue::ViewerStaticDirMissing => {
            &["viewer_static_dir"]
        }
        ConfigIssue::LauncherBinRequired | ConfigIssue::LauncherBinMissing => &["launcher_bin"],
        ConfigIssue::ChainRuntimeBinRequired | ConfigIssue::ChainRuntimeBinMissing => {
            &["chain_runtime_bin"]
        }
        ConfigIssue::ChainStatusBindInvalid => &["chain_status_bind"],
        ConfigIssue::ChainNodeIdRequired => &["chain_node_id"],
        ConfigIssue::ChainRoleInvalid => &["chain_node_role"],
        ConfigIssue::ChainTickMsInvalid => &["chain_node_tick_ms"],
        ConfigIssue::ChainPosSlotDurationMsInvalid => &["chain_pos_slot_duration_ms"],
        ConfigIssue::ChainPosTicksPerSlotInvalid => &["chain_pos_ticks_per_slot"],
        ConfigIssue::ChainPosProposalTickPhaseInvalid => &["chain_pos_proposal_tick_phase"],
        ConfigIssue::ChainPosProposalTickPhaseOutOfRange => {
            &["chain_pos_ticks_per_slot", "chain_pos_proposal_tick_phase"]
        }
        ConfigIssue::ChainPosSlotClockGenesisUnixMsInvalid => {
            &["chain_pos_slot_clock_genesis_unix_ms"]
        }
        ConfigIssue::ChainPosMaxPastSlotLagInvalid => &["chain_pos_max_past_slot_lag"],
        ConfigIssue::ChainValidatorsInvalid => &["chain_node_validators"],
    }
}

impl ClientLauncherApp {
    pub(super) fn ui_field_label(&self, field: &LauncherUiField) -> &'static str {
        match self.ui_language {
            UiLanguage::ZhCn => field.label_zh,
            UiLanguage::EnUs => field.label_en,
        }
    }

    pub(super) fn render_config_field(
        &mut self,
        ui: &mut egui::Ui,
        field: &LauncherUiField,
        stack_text_fields: bool,
    ) {
        let label = self.ui_field_label(field);
        match field.kind {
            LauncherUiFieldKind::Text => {
                if let Some(value) = launcher_text_field_mut(&mut self.config, field.id) {
                    if stack_text_fields {
                        ui.vertical(|ui| {
                            ui.label(label);
                            ui.add_sized(
                                [ui.available_width(), 0.0],
                                egui::TextEdit::singleline(value),
                            );
                        });
                    } else {
                        ui.horizontal(|ui| {
                            ui.label(label);
                            ui.text_edit_singleline(value);
                        });
                    }
                }
            }
            LauncherUiFieldKind::Checkbox => {
                if let Some(value) = launcher_checkbox_field_mut(&mut self.config, field.id) {
                    ui.checkbox(value, label);
                }
            }
        }
    }

    pub(super) fn render_config_section(&mut self, ui: &mut egui::Ui, section: &str) {
        let stack_text_fields = ui.available_width() <= 560.0;
        ui.vertical(|ui| {
            #[cfg(not(target_arch = "wasm32"))]
            {
                for field in
                    launcher_ui_fields_for_native().filter(|field| field.section == section)
                {
                    self.render_config_field(ui, field, stack_text_fields);
                }
            }

            #[cfg(target_arch = "wasm32")]
            {
                for field in launcher_ui_fields_for_web().filter(|field| field.section == section) {
                    self.render_config_field(ui, field, stack_text_fields);
                }
            }
        });
    }

    pub(super) fn render_config_validation_summary(
        &mut self,
        ui: &mut egui::Ui,
        game_required_issues: &[ConfigIssue],
        chain_required_issues: &[ConfigIssue],
    ) {
        let chain_issue_count = if self.config.chain_enabled {
            chain_required_issues.len()
        } else {
            0
        };
        let has_issue = !game_required_issues.is_empty() || chain_issue_count > 0;

        ui.horizontal_wrapped(|ui| {
            ui.label(self.tr(
                "低频配置已收口到高级配置弹窗。",
                "Low-frequency settings are grouped in Advanced Config.",
            ));
            if ui.button(self.tr("高级配置", "Advanced Config")).clicked() {
                self.config_window_open = true;
            }
        });

        if !has_issue {
            ui.colored_label(
                egui::Color32::from_rgb(36, 130, 78),
                self.tr(
                    "当前配置校验通过，可直接执行高频操作。",
                    "Configuration checks passed; quick actions are ready.",
                ),
            );
            return;
        }

        let summary = if self.config.chain_enabled {
            match self.ui_language {
                UiLanguage::ZhCn => format!(
                    "存在配置问题：游戏 {} 项，区块链 {} 项",
                    game_required_issues.len(),
                    chain_issue_count
                ),
                UiLanguage::EnUs => format!(
                    "Configuration issues detected: game {}, blockchain {}",
                    game_required_issues.len(),
                    chain_issue_count
                ),
            }
        } else {
            match self.ui_language {
                UiLanguage::ZhCn => format!("存在配置问题：游戏 {} 项", game_required_issues.len()),
                UiLanguage::EnUs => format!(
                    "Configuration issues detected: game {}",
                    game_required_issues.len()
                ),
            }
        };
        ui.colored_label(egui::Color32::from_rgb(188, 60, 60), summary);
        ui.small(self.tr(
            "点击“启动游戏/启动区块链”将自动弹出可编辑的配置引导。",
            "Click Start Game/Start Blockchain to open the editable configuration guide.",
        ));
    }

    pub(super) fn show_config_window(
        &mut self,
        ctx: &egui::Context,
        game_required_issues: &[ConfigIssue],
        chain_required_issues: &[ConfigIssue],
    ) {
        if !self.config_window_open {
            return;
        }

        let mut keep_open = self.config_window_open;
        egui::Window::new(self.tr("高级配置", "Advanced Config"))
            .collapsible(false)
            .resizable(true)
            .default_width(780.0)
            .default_height(640.0)
            .open(&mut keep_open)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for section in NATIVE_UI_SECTIONS {
                            self.render_config_section(ui, section);
                        }
                    });

                ui.separator();

                if game_required_issues.is_empty() {
                    ui.colored_label(
                        egui::Color32::from_rgb(36, 130, 78),
                        self.tr(
                            "必填配置项已通过校验，可启动游戏",
                            "Required configuration check passed; game can start",
                        ),
                    );
                } else {
                    ui.group(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(188, 60, 60),
                            self.tr(
                                "游戏启动前请先修复以下必填配置项：",
                                "Fix the required game configuration issues before starting:",
                            ),
                        );
                        for issue in game_required_issues {
                            ui.label(format!("- {}", issue.text(self.ui_language)));
                        }
                    });
                }

                if self.config.chain_enabled && !chain_required_issues.is_empty() {
                    ui.group(|ui| {
                        ui.colored_label(
                            egui::Color32::from_rgb(188, 60, 60),
                            self.tr(
                                "区块链启动前请先修复以下配置项：",
                                "Fix the blockchain configuration issues before starting:",
                            ),
                        );
                        for issue in chain_required_issues {
                            ui.label(format!("- {}", issue.text(self.ui_language)));
                        }
                    });
                }
            });
        self.config_window_open = keep_open;
    }

    pub(super) fn maybe_open_startup_guide_on_first_check(
        &mut self,
        game_required_issues: &[ConfigIssue],
        chain_required_issues: &[ConfigIssue],
    ) {
        if self.startup_guide_state.first_check_done {
            return;
        }
        self.startup_guide_state.first_check_done = true;

        if !game_required_issues.is_empty() {
            self.open_startup_guide(StartupGuideTarget::Game);
            self.append_log(self.tr(
                "首次检查发现游戏配置缺失，已打开配置引导。",
                "Initial check found missing game configuration; configuration guide opened.",
            ));
            return;
        }

        if self.config.chain_enabled && !chain_required_issues.is_empty() {
            self.open_startup_guide(StartupGuideTarget::Chain);
            self.append_log(self.tr(
                "首次检查发现区块链配置缺失，已打开配置引导。",
                "Initial check found missing blockchain configuration; configuration guide opened.",
            ));
        }
    }

    pub(super) fn handle_start_game_click(&mut self, game_required_issues: &[ConfigIssue]) {
        if game_required_issues.is_empty() {
            self.start_process();
            return;
        }

        self.status = LauncherStatus::InvalidArgs;
        self.append_log(self.tr(
            "游戏启动前校验失败：已打开配置引导，请先补齐字段。",
            "Game preflight validation failed: configuration guide opened, fill required fields first.",
        ));
        for issue in game_required_issues {
            self.append_log(format!("- {}", issue.text(self.ui_language)));
        }
        self.open_startup_guide(StartupGuideTarget::Game);
    }

    pub(super) fn handle_start_chain_click(&mut self, chain_required_issues: &[ConfigIssue]) {
        if chain_required_issues.is_empty() {
            self.start_chain_process();
            return;
        }

        let mut details = Vec::new();
        for issue in chain_required_issues {
            let detail = issue.text(self.ui_language).to_string();
            details.push(detail.clone());
            self.append_log(format!("- {detail}"));
        }
        self.chain_runtime_status = ChainRuntimeStatus::ConfigError(details.join("; "));
        self.append_log(self.tr(
            "区块链启动前校验失败：已打开配置引导，请先补齐字段。",
            "Blockchain preflight validation failed: configuration guide opened, fill required fields first.",
        ));
        self.open_startup_guide(StartupGuideTarget::Chain);
    }

    pub(super) fn show_startup_guide_window(
        &mut self,
        ctx: &egui::Context,
        game_required_issues: &[ConfigIssue],
        chain_required_issues: &[ConfigIssue],
    ) {
        if !self.startup_guide_state.open {
            return;
        }

        let target = self.startup_guide_state.target;
        let issues = match target {
            StartupGuideTarget::Game => game_required_issues,
            StartupGuideTarget::Chain => chain_required_issues,
        };
        let field_ids = self.collect_issue_fields(issues);

        let title = match target {
            StartupGuideTarget::Game => self.tr("启动引导（游戏）", "Startup Guide (Game)"),
            StartupGuideTarget::Chain => {
                self.tr("启动引导（区块链）", "Startup Guide (Blockchain)")
            }
        };
        let intro = match target {
            StartupGuideTarget::Game => self.tr(
                "检测到游戏启动前有必填问题，请直接在此窗口补齐。",
                "Required game settings are missing. Fill them directly in this window.",
            ),
            StartupGuideTarget::Chain => self.tr(
                "检测到区块链启动前有必填问题，请直接在此窗口补齐。",
                "Required blockchain settings are missing. Fill them directly in this window.",
            ),
        };

        let mut keep_open = self.startup_guide_state.open;
        let mut request_close = false;
        egui::Window::new(title)
            .collapsible(false)
            .resizable(true)
            .default_width(760.0)
            .default_height(560.0)
            .open(&mut keep_open)
            .show(ctx, |ui| {
                ui.label(intro);
                ui.separator();

                if issues.is_empty() {
                    ui.colored_label(
                        egui::Color32::from_rgb(36, 130, 78),
                        self.tr(
                            "当前目标已无阻断配置，可关闭窗口后继续启动。",
                            "No blocking configuration remains for this target. Close this window and start again.",
                        ),
                    );
                    if ui.button(self.tr("关闭", "Close")).clicked() {
                        request_close = true;
                    }
                    return;
                }

                ui.colored_label(
                    egui::Color32::from_rgb(188, 60, 60),
                    self.tr("待修复问题：", "Issues to fix:"),
                );
                for issue in issues {
                    ui.label(format!("- {}", issue.text(self.ui_language)));
                }

                ui.separator();
                ui.label(self.tr("直接编辑下列字段：", "Edit fields directly below:"));

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let stack_text_fields = ui.available_width() <= 560.0;
                        for field_id in &field_ids {
                            if !self.render_config_field_by_id(ui, field_id, stack_text_fields) {
                                ui.small(
                                    self.tr(
                                        "存在未映射字段，请通过“高级配置”继续修复。",
                                        "Some fields are not mapped. Use Advanced Config to continue.",
                                    ),
                                );
                            }
                        }
                    });

                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    if ui.button(self.tr("打开高级配置", "Open Advanced Config")).clicked() {
                        self.config_window_open = true;
                    }
                    if ui.button(self.tr("关闭", "Close")).clicked() {
                        request_close = true;
                    }
                });
            });

        if request_close {
            keep_open = false;
        }
        self.startup_guide_state.open = keep_open;
    }

    fn open_startup_guide(&mut self, target: StartupGuideTarget) {
        self.startup_guide_state.target = target;
        self.startup_guide_state.open = true;
    }

    fn collect_issue_fields(&self, issues: &[ConfigIssue]) -> Vec<&'static str> {
        let mut fields = Vec::new();
        for issue in issues {
            for field_id in issue_field_ids(*issue) {
                if !fields.contains(field_id) {
                    fields.push(*field_id);
                }
            }
        }
        fields
    }

    fn render_config_field_by_id(
        &mut self,
        ui: &mut egui::Ui,
        field_id: &str,
        stack_text_fields: bool,
    ) -> bool {
        #[cfg(not(target_arch = "wasm32"))]
        let field = launcher_ui_fields_for_native().find(|field| field.id == field_id);
        #[cfg(target_arch = "wasm32")]
        let field = launcher_ui_fields_for_web().find(|field| field.id == field_id);

        if let Some(field) = field {
            self.render_config_field(ui, field, stack_text_fields);
            true
        } else {
            false
        }
    }
}
