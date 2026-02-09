use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::sync::Arc;

use crate::diagnosis::DiagnosisText;
use crate::i18n::{locale_or_default, UiI18n};
use crate::selection_linking::EventObjectLinkText;
use crate::timeline_controls::TimelineStatusText;
use crate::world_overlay::WorldOverlayStatusText;
use crate::{
    AgentActivityText, EventsText, SelectionDetailsText, SelectionText, StatusText, SummaryText,
};

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

pub(super) fn render_copyable_text_panel(
    mut contexts: EguiContexts,
    mut cjk_font_initialized: Local<bool>,
    panel_state: Res<CopyableTextPanelState>,
    i18n: Option<Res<UiI18n>>,
    status_query: Query<&Text, With<StatusText>>,
    selection_query: Query<&Text, With<SelectionText>>,
    summary_query: Query<&Text, With<SummaryText>>,
    activity_query: Query<&Text, With<AgentActivityText>>,
    details_query: Query<&Text, With<SelectionDetailsText>>,
    events_query: Query<&Text, With<EventsText>>,
    diagnosis_query: Query<&Text, With<DiagnosisText>>,
    link_query: Query<&Text, With<EventObjectLinkText>>,
    timeline_status_query: Query<&Text, With<TimelineStatusText>>,
    overlay_status_query: Query<&Text, With<WorldOverlayStatusText>>,
) -> Result {
    if !panel_state.visible {
        return Ok(());
    }

    let context = contexts.ctx_mut()?;
    ensure_egui_cjk_font(context, &mut cjk_font_initialized);

    let locale = locale_or_default(i18n.as_deref());
    let panel_title = copy_panel_title(locale);

    let sections = [
        (
            if locale.is_zh() { "状态" } else { "Status" },
            read_text(status_query.single().ok()),
        ),
        (
            if locale.is_zh() {
                "当前选择"
            } else {
                "Selection"
            },
            read_text(selection_query.single().ok()),
        ),
        (
            if locale.is_zh() {
                "世界摘要"
            } else {
                "World Summary"
            },
            read_text(summary_query.single().ok()),
        ),
        (
            if locale.is_zh() {
                "Agent 活动"
            } else {
                "Agent Activity"
            },
            read_text(activity_query.single().ok()),
        ),
        (
            if locale.is_zh() {
                "选中详情"
            } else {
                "Selection Details"
            },
            read_text(details_query.single().ok()),
        ),
        (
            if locale.is_zh() {
                "事件列表"
            } else {
                "Events"
            },
            read_text(events_query.single().ok()),
        ),
        (
            if locale.is_zh() {
                "诊断"
            } else {
                "Diagnosis"
            },
            read_text(diagnosis_query.single().ok()),
        ),
        (
            if locale.is_zh() {
                "事件联动"
            } else {
                "Event Link"
            },
            read_text(link_query.single().ok()),
        ),
        (
            if locale.is_zh() {
                "时间轴"
            } else {
                "Timeline"
            },
            read_text(timeline_status_query.single().ok()),
        ),
        (
            if locale.is_zh() {
                "覆盖层"
            } else {
                "Overlay"
            },
            read_text(overlay_status_query.single().ok()),
        ),
    ];

    egui::Window::new(panel_title)
        .default_width(560.0)
        .default_height(620.0)
        .resizable(true)
        .show(context, |ui| {
            ui.label(copy_panel_hint(locale));
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (title, text) in sections {
                    if text.is_empty() {
                        continue;
                    }

                    ui.collapsing(title, |ui| {
                        ui.add(
                            egui::Label::new(egui::RichText::new(text))
                                .wrap()
                                .selectable(true),
                        );
                    });
                }
            });
        });

    Ok(())
}

fn read_text(text: Option<&Text>) -> String {
    text.map(|value| value.0.clone()).unwrap_or_default()
}

fn copy_panel_title(locale: crate::i18n::UiLocale) -> &'static str {
    if locale.is_zh() {
        "可复制信息"
    } else {
        "Copyable Text"
    }
}

fn copy_panel_hint(locale: crate::i18n::UiLocale) -> &'static str {
    if locale.is_zh() {
        "可拖选文本，并使用 Cmd/Ctrl+C 复制"
    } else {
        "Select text and use Cmd/Ctrl+C to copy"
    }
}

fn ensure_egui_cjk_font(context: &egui::Context, initialized: &mut bool) {
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
    fn read_text_returns_empty_when_missing() {
        assert_eq!(read_text(None), "");
    }

    #[test]
    fn read_text_clones_content() {
        let text = Text::new("hello");
        assert_eq!(read_text(Some(&text)), "hello");
    }

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
        assert_eq!(copy_panel_title(crate::i18n::UiLocale::ZhCn), "可复制信息");
        assert_eq!(
            copy_panel_title(crate::i18n::UiLocale::EnUs),
            "Copyable Text"
        );
    }
}
