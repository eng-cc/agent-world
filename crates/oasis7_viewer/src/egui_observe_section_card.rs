use bevy_egui::egui;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ObserveSectionTone {
    World,
    Activity,
    Industrial,
    Economy,
    Ops,
    Details,
    Events,
    Default,
}

pub(super) fn section_tone(title: &str) -> ObserveSectionTone {
    let lowered = title.to_ascii_lowercase();
    if lowered.contains("world") || title.contains("世界") {
        ObserveSectionTone::World
    } else if lowered.contains("activity") || title.contains("活动") {
        ObserveSectionTone::Activity
    } else if lowered.contains("industrial") || title.contains("工业") {
        ObserveSectionTone::Industrial
    } else if lowered.contains("economy") || title.contains("经营") {
        ObserveSectionTone::Economy
    } else if lowered.contains("ops") || title.contains("导航") {
        ObserveSectionTone::Ops
    } else if lowered.contains("detail") || title.contains("详情") {
        ObserveSectionTone::Details
    } else if lowered.contains("event") || title.contains("事件") {
        ObserveSectionTone::Events
    } else {
        ObserveSectionTone::Default
    }
}

fn section_tone_fill(tone: ObserveSectionTone) -> egui::Color32 {
    match tone {
        ObserveSectionTone::World => egui::Color32::from_rgb(22, 34, 46),
        ObserveSectionTone::Activity => egui::Color32::from_rgb(22, 42, 34),
        ObserveSectionTone::Industrial => egui::Color32::from_rgb(45, 35, 22),
        ObserveSectionTone::Economy => egui::Color32::from_rgb(36, 28, 18),
        ObserveSectionTone::Ops => egui::Color32::from_rgb(33, 24, 42),
        ObserveSectionTone::Details => egui::Color32::from_rgb(30, 30, 30),
        ObserveSectionTone::Events => egui::Color32::from_rgb(34, 30, 26),
        ObserveSectionTone::Default => egui::Color32::from_rgb(30, 30, 30),
    }
}

fn section_tone_accent(tone: ObserveSectionTone) -> egui::Color32 {
    match tone {
        ObserveSectionTone::World => egui::Color32::from_rgb(120, 188, 255),
        ObserveSectionTone::Activity => egui::Color32::from_rgb(118, 210, 156),
        ObserveSectionTone::Industrial => egui::Color32::from_rgb(236, 183, 92),
        ObserveSectionTone::Economy => egui::Color32::from_rgb(230, 165, 87),
        ObserveSectionTone::Ops => egui::Color32::from_rgb(185, 150, 236),
        ObserveSectionTone::Details => egui::Color32::from_rgb(198, 198, 198),
        ObserveSectionTone::Events => egui::Color32::from_rgb(216, 171, 132),
        ObserveSectionTone::Default => egui::Color32::from_rgb(198, 198, 198),
    }
}

fn supports_motion(tone: ObserveSectionTone) -> bool {
    matches!(
        tone,
        ObserveSectionTone::Industrial
            | ObserveSectionTone::Economy
            | ObserveSectionTone::Ops
            | ObserveSectionTone::Events
    )
}

fn motion_amplitude(ui: &egui::Ui, tone: ObserveSectionTone, motion_enabled: bool) -> f32 {
    if !motion_enabled || !supports_motion(tone) {
        return 0.0;
    }
    let time = ui.ctx().input(|input| input.time) as f32;
    ((time * 2.4).sin() * 0.5 + 0.5).clamp(0.0, 1.0)
}

pub(super) fn render_observe_section_card(
    ui: &mut egui::Ui,
    title: &str,
    content: &str,
    product_style: bool,
    motion_enabled: bool,
) {
    if !product_style {
        ui.group(|ui| {
            ui.strong(title);
            ui.add(egui::Label::new(content).wrap().selectable(true));
        });
        return;
    }

    let tone = section_tone(title);
    let fill = section_tone_fill(tone);
    let motion = motion_amplitude(ui, tone, motion_enabled);
    if motion_enabled && motion > 0.0 {
        ui.ctx().request_repaint_after(Duration::from_millis(33));
    }

    let accent = section_tone_accent(tone).gamma_multiply(0.82 + motion * 0.18);
    let stroke_width = 1.0 + motion * 0.6;
    egui::Frame::group(ui.style())
        .fill(fill)
        .stroke(egui::Stroke::new(stroke_width, accent.gamma_multiply(0.72)))
        .corner_radius(egui::CornerRadius::same(8))
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(egui::RichText::new("●").color(accent));
                ui.strong(egui::RichText::new(title).color(accent).size(13.5));
            });
            ui.add_space(2.0);
            ui.add(egui::Label::new(content).wrap().selectable(true));
        });
}
