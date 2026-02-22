pub(super) fn env_toggle_enabled(raw: Option<&str>) -> bool {
    raw.map(|value| value.trim().to_ascii_lowercase())
        .map(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

pub(super) fn is_ops_nav_panel_enabled() -> bool {
    env_toggle_enabled(std::env::var(super::OPS_NAV_PANEL_ENV).ok().as_deref())
}

pub(super) fn is_product_style_enabled() -> bool {
    env_toggle_enabled(std::env::var(super::PRODUCT_STYLE_ENV).ok().as_deref())
}

pub(super) fn is_product_style_motion_enabled() -> bool {
    env_toggle_enabled(
        std::env::var(super::PRODUCT_STYLE_MOTION_ENV)
            .ok()
            .as_deref(),
    )
}
