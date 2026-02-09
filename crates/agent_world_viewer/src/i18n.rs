use bevy::prelude::*;

use super::{SelectionKind, ViewerControl};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum UiLocale {
    ZhCn,
    EnUs,
}

#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UiI18n {
    pub locale: UiLocale,
}

impl Default for UiI18n {
    fn default() -> Self {
        Self {
            locale: UiLocale::ZhCn,
        }
    }
}

impl UiLocale {
    pub(super) fn toggled(self) -> Self {
        match self {
            UiLocale::ZhCn => UiLocale::EnUs,
            UiLocale::EnUs => UiLocale::ZhCn,
        }
    }

    pub(super) fn is_zh(self) -> bool {
        matches!(self, UiLocale::ZhCn)
    }
}

pub(super) fn locale_or_default(i18n: Option<&UiI18n>) -> UiLocale {
    i18n.map(|value| value.locale).unwrap_or(UiLocale::EnUs)
}

pub(super) fn top_panel_toggle_label(collapsed: bool, locale: UiLocale) -> &'static str {
    match (locale, collapsed) {
        (UiLocale::ZhCn, false) => "隐藏顶部",
        (UiLocale::ZhCn, true) => "显示顶部",
        (UiLocale::EnUs, false) => "Hide Top",
        (UiLocale::EnUs, true) => "Show Top",
    }
}

pub(super) fn top_controls_label(locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        "顶部控制区"
    } else {
        "Top Controls"
    }
}

pub(super) fn language_toggle_label(locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        "语言：中文"
    } else {
        "Language: English"
    }
}

pub(super) fn copyable_panel_toggle_label(visible: bool, locale: UiLocale) -> &'static str {
    match (locale, visible) {
        (UiLocale::ZhCn, true) => "隐藏复制窗",
        (UiLocale::ZhCn, false) => "显示复制窗",
        (UiLocale::EnUs, true) => "Hide Copy Panel",
        (UiLocale::EnUs, false) => "Show Copy Panel",
    }
}

pub(super) fn control_button_label(control: &ViewerControl, locale: UiLocale) -> &'static str {
    match control {
        ViewerControl::Play => {
            if locale.is_zh() {
                "播放"
            } else {
                "Play"
            }
        }
        ViewerControl::Pause => {
            if locale.is_zh() {
                "暂停"
            } else {
                "Pause"
            }
        }
        ViewerControl::Step { .. } => {
            if locale.is_zh() {
                "单步"
            } else {
                "Step"
            }
        }
        ViewerControl::Seek { .. } => {
            if locale.is_zh() {
                "跳转 0"
            } else {
                "Seek 0"
            }
        }
    }
}

pub(super) fn step_button_label(locale: UiLocale, pending: bool) -> &'static str {
    if locale.is_zh() {
        if pending {
            "单步 ..."
        } else {
            "单步"
        }
    } else if pending {
        "Step ..."
    } else {
        "Step"
    }
}

pub(super) fn selection_kind_label(kind: SelectionKind, locale: UiLocale) -> &'static str {
    match kind {
        SelectionKind::Agent => {
            if locale.is_zh() {
                "agent"
            } else {
                "agent"
            }
        }
        SelectionKind::Location => {
            if locale.is_zh() {
                "地点"
            } else {
                "location"
            }
        }
        SelectionKind::Asset => {
            if locale.is_zh() {
                "资产"
            } else {
                "asset"
            }
        }
        SelectionKind::PowerPlant => {
            if locale.is_zh() {
                "电厂"
            } else {
                "power_plant"
            }
        }
        SelectionKind::PowerStorage => {
            if locale.is_zh() {
                "储能"
            } else {
                "power_storage"
            }
        }
        SelectionKind::Chunk => {
            if locale.is_zh() {
                "分块"
            } else {
                "chunk"
            }
        }
    }
}

pub(super) fn on_off_label(enabled: bool, locale: UiLocale) -> &'static str {
    if locale.is_zh() {
        if enabled {
            "开"
        } else {
            "关"
        }
    } else if enabled {
        "on"
    } else {
        "off"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_locale_is_chinese() {
        assert_eq!(UiI18n::default().locale, UiLocale::ZhCn);
    }

    #[test]
    fn locale_toggle_round_trip() {
        assert_eq!(UiLocale::ZhCn.toggled(), UiLocale::EnUs);
        assert_eq!(UiLocale::EnUs.toggled(), UiLocale::ZhCn);
    }

    #[test]
    fn copyable_toggle_label_is_localized() {
        assert_eq!(
            copyable_panel_toggle_label(true, UiLocale::ZhCn),
            "隐藏复制窗"
        );
        assert_eq!(
            copyable_panel_toggle_label(false, UiLocale::ZhCn),
            "显示复制窗"
        );
        assert_eq!(
            copyable_panel_toggle_label(true, UiLocale::EnUs),
            "Hide Copy Panel"
        );
        assert_eq!(
            copyable_panel_toggle_label(false, UiLocale::EnUs),
            "Show Copy Panel"
        );
    }
}
