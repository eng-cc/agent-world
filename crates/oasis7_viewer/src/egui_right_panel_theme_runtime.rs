use bevy_egui::egui;

use crate::app_bootstrap::{ThemePresetSelection, ThemeRuntimeState};

pub(super) fn render_theme_runtime_section(
    ui: &mut egui::Ui,
    locale: crate::i18n::UiLocale,
    theme_runtime: &mut ThemeRuntimeState,
) {
    ui.separator();
    ui.strong(if locale.is_zh() {
        "主题热切换"
    } else {
        "Theme Runtime"
    });

    let mut selected = theme_runtime.selection;
    egui::ComboBox::from_label(if locale.is_zh() { "预设" } else { "Preset" })
        .selected_text(selected.label(locale))
        .show_ui(ui, |ui| {
            for option in ThemePresetSelection::ORDERED.iter().copied() {
                ui.selectable_value(&mut selected, option, option.label(locale));
            }
        });
    theme_runtime.selection = selected;

    if theme_runtime.selection == ThemePresetSelection::Custom {
        ui.horizontal_wrapped(|ui| {
            ui.label(if locale.is_zh() {
                "文件路径"
            } else {
                "Preset file"
            });
            ui.text_edit_singleline(&mut theme_runtime.custom_preset_path);
        });
    }

    ui.horizontal_wrapped(|ui| {
        if ui
            .button(if locale.is_zh() {
                "应用主题"
            } else {
                "Apply Theme"
            })
            .clicked()
        {
            theme_runtime.pending_apply = true;
        }
        ui.checkbox(
            &mut theme_runtime.hot_reload_enabled,
            if locale.is_zh() {
                "自动热重载"
            } else {
                "Auto Hot Reload"
            },
        );
    });

    ui.add(
        egui::Label::new(theme_runtime.status_message.as_str())
            .wrap()
            .selectable(true),
    );
}
