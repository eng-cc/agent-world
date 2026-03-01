use super::{
    ViewerFragmentMaterialStrategy, ViewerGeometryTier, ViewerRenderProfile, ViewerTonemappingMode,
    ViewerVisualEffectsLevel,
};

pub(super) fn parse_bool<F>(lookup: &F, key: &str) -> Option<bool>
where
    F: Fn(&str) -> Option<String>,
{
    lookup(key).and_then(|raw| {
        let normalized = raw.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        }
    })
}

pub(super) fn parse_render_profile<F>(lookup: &F, key: &str) -> Option<ViewerRenderProfile>
where
    F: Fn(&str) -> Option<String>,
{
    lookup(key).and_then(|raw| match raw.trim().to_ascii_lowercase().as_str() {
        "debug" | "dbg" => Some(ViewerRenderProfile::Debug),
        "balanced" | "balance" | "default" => Some(ViewerRenderProfile::Balanced),
        "cinematic" | "cinema" | "quality" => Some(ViewerRenderProfile::Cinematic),
        _ => None,
    })
}

pub(super) fn parse_geometry_tier<F>(lookup: &F, key: &str) -> Option<ViewerGeometryTier>
where
    F: Fn(&str) -> Option<String>,
{
    lookup(key).and_then(|raw| match raw.trim().to_ascii_lowercase().as_str() {
        "debug" | "low" => Some(ViewerGeometryTier::Debug),
        "balanced" | "medium" => Some(ViewerGeometryTier::Balanced),
        "cinematic" | "high" => Some(ViewerGeometryTier::Cinematic),
        _ => None,
    })
}

pub(super) fn parse_visual_effects_level<F>(lookup: &F, key: &str) -> Option<ViewerVisualEffectsLevel>
where
    F: Fn(&str) -> Option<String>,
{
    lookup(key).and_then(|raw| match raw.trim().to_ascii_lowercase().as_str() {
        "minimal" | "min" | "low" => Some(ViewerVisualEffectsLevel::Minimal),
        "standard" | "std" | "default" => Some(ViewerVisualEffectsLevel::Standard),
        "enhanced" | "enhance" | "high" => Some(ViewerVisualEffectsLevel::Enhanced),
        _ => None,
    })
}

pub(super) fn parse_fragment_material_strategy<F>(
    lookup: &F,
    key: &str,
) -> Option<ViewerFragmentMaterialStrategy>
where
    F: Fn(&str) -> Option<String>,
{
    lookup(key).and_then(|raw| match raw.trim().to_ascii_lowercase().as_str() {
        "readability" | "readable" | "clarity" => Some(ViewerFragmentMaterialStrategy::Readability),
        "fidelity" | "quality" | "realistic" => Some(ViewerFragmentMaterialStrategy::Fidelity),
        _ => None,
    })
}

pub(super) fn parse_tonemapping_mode<F>(lookup: &F, key: &str) -> Option<ViewerTonemappingMode>
where
    F: Fn(&str) -> Option<String>,
{
    lookup(key).and_then(|raw| match raw.trim().to_ascii_lowercase().as_str() {
        "none" | "off" => Some(ViewerTonemappingMode::None),
        "reinhard" => Some(ViewerTonemappingMode::Reinhard),
        "reinhard_luminance" | "reinhard-luminance" | "reinhardluminance" => {
            Some(ViewerTonemappingMode::ReinhardLuminance)
        }
        "aces" | "acesfitted" | "aces_fitted" | "aces-fitted" => {
            Some(ViewerTonemappingMode::AcesFitted)
        }
        "agx" => Some(ViewerTonemappingMode::AgX),
        "somewhat_boring_display_transform"
        | "somewhat-boring-display-transform"
        | "somewhatboringdisplaytransform"
        | "sbdt" => Some(ViewerTonemappingMode::SomewhatBoringDisplayTransform),
        "tony_mc_mapface" | "tony-mc-mapface" | "tonymcmapface" | "tony" | "default" => {
            Some(ViewerTonemappingMode::TonyMcMapface)
        }
        "blender_filmic" | "blender-filmic" | "blenderfilmic" | "filmic" => {
            Some(ViewerTonemappingMode::BlenderFilmic)
        }
        _ => None,
    })
}

pub(super) fn parse_f32<F>(lookup: &F, key: &str) -> Option<f32>
where
    F: Fn(&str) -> Option<String>,
{
    lookup(key).and_then(|raw| raw.trim().parse::<f32>().ok())
}

pub(super) fn parse_f64<F>(lookup: &F, key: &str) -> Option<f64>
where
    F: Fn(&str) -> Option<String>,
{
    lookup(key).and_then(|raw| raw.trim().parse::<f64>().ok())
}

pub(super) fn parse_non_empty_string<F>(lookup: &F, key: &str) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    lookup(key).and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

pub(super) fn parse_hex_srgb_color<F>(lookup: &F, key: &str) -> Option<[f32; 3]>
where
    F: Fn(&str) -> Option<String>,
{
    lookup(key).and_then(|raw| parse_hex_srgb_literal(raw.trim()))
}

fn parse_hex_srgb_literal(raw: &str) -> Option<[f32; 3]> {
    let color_hex = raw.strip_prefix('#')?;
    if color_hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&color_hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&color_hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&color_hex[4..6], 16).ok()?;
    Some([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0])
}

pub(super) fn parse_agent_variant_palette<F>(lookup: &F, key: &str) -> Option<[[f32; 3]; 4]>
where
    F: Fn(&str) -> Option<String>,
{
    let raw = lookup(key)?;
    let mut palette = [[0.0; 3]; 4];
    let mut count = 0usize;
    for color in raw.split(',').map(str::trim) {
        if color.is_empty() || count >= palette.len() {
            return None;
        }
        palette[count] = parse_hex_srgb_literal(color)?;
        count += 1;
    }
    if count == palette.len() {
        Some(palette)
    } else {
        None
    }
}

pub(super) fn parse_usize<F>(lookup: &F, key: &str) -> Option<usize>
where
    F: Fn(&str) -> Option<String>,
{
    lookup(key).and_then(|raw| raw.trim().parse::<usize>().ok())
}

pub(super) fn parse_u64<F>(lookup: &F, key: &str) -> Option<u64>
where
    F: Fn(&str) -> Option<String>,
{
    lookup(key).and_then(|raw| raw.trim().parse::<u64>().ok())
}
