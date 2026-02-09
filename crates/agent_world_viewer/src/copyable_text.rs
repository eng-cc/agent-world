use bevy::prelude::*;
use bevy_egui::egui;
use std::sync::Arc;

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CopyableTextPanelState {
    pub visible: bool,
}

const EGUI_CJK_FONT_NAME: &str = "ms-yahei-cjk";
const EGUI_CJK_FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/ms-yahei.ttf");

impl Default for CopyableTextPanelState {
    fn default() -> Self {
        Self { visible: true }
    }
}

pub(super) fn copy_panel_title(locale: crate::i18n::UiLocale) -> &'static str {
    if locale.is_zh() {
        "状态明细"
    } else {
        "State Details"
    }
}

pub(super) fn copy_panel_hint(locale: crate::i18n::UiLocale) -> &'static str {
    if locale.is_zh() {
        "用于界面内观察与对比，避免外部复制分析"
    } else {
        "For in-app observability; avoid external copy-heavy analysis"
    }
}

pub(super) fn ensure_egui_cjk_font(context: &egui::Context, initialized: &mut bool) {
    if *initialized {
        return;
    }

    let mut fonts = egui::FontDefinitions::default();
    install_cjk_font(&mut fonts);
    context.set_fonts(fonts);
    *initialized = true;
}

fn install_cjk_font(fonts: &mut egui::FontDefinitions) {
    fonts.font_data.insert(
        EGUI_CJK_FONT_NAME.to_string(),
        Arc::new(egui::FontData::from_static(EGUI_CJK_FONT_BYTES)),
    );

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, EGUI_CJK_FONT_NAME.to_string());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push(EGUI_CJK_FONT_NAME.to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_cjk_font_registers_font_and_priority() {
        let mut fonts = egui::FontDefinitions::default();
        install_cjk_font(&mut fonts);

        assert!(fonts.font_data.contains_key(EGUI_CJK_FONT_NAME));

        let proportional = fonts
            .families
            .get(&egui::FontFamily::Proportional)
            .expect("proportional family");
        assert_eq!(
            proportional.first().map(String::as_str),
            Some(EGUI_CJK_FONT_NAME)
        );

        let monospace = fonts
            .families
            .get(&egui::FontFamily::Monospace)
            .expect("monospace family");
        assert!(monospace.iter().any(|name| name == EGUI_CJK_FONT_NAME));
    }

    #[test]
    fn copy_panel_title_is_localized() {
        assert_eq!(copy_panel_title(crate::i18n::UiLocale::ZhCn), "状态明细");
        assert_eq!(
            copy_panel_title(crate::i18n::UiLocale::EnUs),
            "State Details"
        );
    }
}
