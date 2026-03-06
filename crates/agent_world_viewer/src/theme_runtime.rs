use std::collections::HashMap;

use bevy::prelude::*;

const THEME_PRESET_ENV: &str = "AGENT_WORLD_VIEWER_THEME_PRESET";
const THEME_PRESET_FILE_ENV: &str = "AGENT_WORLD_VIEWER_THEME_PRESET_FILE";
const THEME_HOT_RELOAD_ENV: &str = "AGENT_WORLD_VIEWER_THEME_HOT_RELOAD";

const INDUSTRIAL_V3_DEFAULT_PRESET_PATH: &str =
    "crates/agent_world_viewer/assets/themes/industrial_v3/presets/industrial_v3_default.env";
const INDUSTRIAL_V3_MATTE_PRESET_PATH: &str =
    "crates/agent_world_viewer/assets/themes/industrial_v3/presets/industrial_v3_matte.env";
const INDUSTRIAL_V3_GLOSSY_PRESET_PATH: &str =
    "crates/agent_world_viewer/assets/themes/industrial_v3/presets/industrial_v3_glossy.env";

const INDUSTRIAL_V2_DEFAULT_PRESET_PATH: &str =
    "crates/agent_world_viewer/assets/themes/industrial_v2/presets/industrial_v2_default.env";
const INDUSTRIAL_V2_MATTE_PRESET_PATH: &str =
    "crates/agent_world_viewer/assets/themes/industrial_v2/presets/industrial_v2_matte.env";
const INDUSTRIAL_V2_GLOSSY_PRESET_PATH: &str =
    "crates/agent_world_viewer/assets/themes/industrial_v2/presets/industrial_v2_glossy.env";

const INDUSTRIAL_V3_DEFAULT_PRESET_EMBEDDED: &str =
    include_str!("../assets/themes/industrial_v3/presets/industrial_v3_default.env");
const INDUSTRIAL_V3_MATTE_PRESET_EMBEDDED: &str =
    include_str!("../assets/themes/industrial_v3/presets/industrial_v3_matte.env");
const INDUSTRIAL_V3_GLOSSY_PRESET_EMBEDDED: &str =
    include_str!("../assets/themes/industrial_v3/presets/industrial_v3_glossy.env");

const INDUSTRIAL_V2_DEFAULT_PRESET_EMBEDDED: &str =
    include_str!("../assets/themes/industrial_v2/presets/industrial_v2_default.env");
const INDUSTRIAL_V2_MATTE_PRESET_EMBEDDED: &str =
    include_str!("../assets/themes/industrial_v2/presets/industrial_v2_matte.env");
const INDUSTRIAL_V2_GLOSSY_PRESET_EMBEDDED: &str =
    include_str!("../assets/themes/industrial_v2/presets/industrial_v2_glossy.env");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ThemePresetSelection {
    None,
    IndustrialV3Default,
    IndustrialV3Matte,
    IndustrialV3Glossy,
    IndustrialV2Default,
    IndustrialV2Matte,
    IndustrialV2Glossy,
    Custom,
}

impl ThemePresetSelection {
    pub(crate) const ORDERED: [Self; 8] = [
        Self::None,
        Self::IndustrialV3Default,
        Self::IndustrialV3Matte,
        Self::IndustrialV3Glossy,
        Self::IndustrialV2Default,
        Self::IndustrialV2Matte,
        Self::IndustrialV2Glossy,
        Self::Custom,
    ];

    pub(crate) fn from_env(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "none" | "off" | "disabled" => Some(Self::None),
            "industrial_v3_default" | "industrial-v3-default" | "v3_default" | "default" => {
                Some(Self::IndustrialV3Default)
            }
            "industrial_v3_matte" | "industrial-v3-matte" | "v3_matte" | "matte" => {
                Some(Self::IndustrialV3Matte)
            }
            "industrial_v3_glossy" | "industrial-v3-glossy" | "v3_glossy" | "glossy" => {
                Some(Self::IndustrialV3Glossy)
            }
            "industrial_v2_default" | "industrial-v2-default" | "v2_default" => {
                Some(Self::IndustrialV2Default)
            }
            "industrial_v2_matte" | "industrial-v2-matte" | "v2_matte" => {
                Some(Self::IndustrialV2Matte)
            }
            "industrial_v2_glossy" | "industrial-v2-glossy" | "v2_glossy" => {
                Some(Self::IndustrialV2Glossy)
            }
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }

    pub(crate) fn label(self, locale: crate::i18n::UiLocale) -> &'static str {
        match (self, locale.is_zh()) {
            (Self::None, true) => "关闭",
            (Self::None, false) => "Off",
            (Self::IndustrialV3Default, true) => "industrial_v3 默认",
            (Self::IndustrialV3Default, false) => "industrial_v3 default",
            (Self::IndustrialV3Matte, true) => "industrial_v3 哑光",
            (Self::IndustrialV3Matte, false) => "industrial_v3 matte",
            (Self::IndustrialV3Glossy, true) => "industrial_v3 亮面",
            (Self::IndustrialV3Glossy, false) => "industrial_v3 glossy",
            (Self::IndustrialV2Default, true) => "industrial_v2 默认",
            (Self::IndustrialV2Default, false) => "industrial_v2 default",
            (Self::IndustrialV2Matte, true) => "industrial_v2 哑光",
            (Self::IndustrialV2Matte, false) => "industrial_v2 matte",
            (Self::IndustrialV2Glossy, true) => "industrial_v2 亮面",
            (Self::IndustrialV2Glossy, false) => "industrial_v2 glossy",
            (Self::Custom, true) => "自定义文件",
            (Self::Custom, false) => "Custom file",
        }
    }

    fn builtin_path(self) -> Option<&'static str> {
        match self {
            Self::IndustrialV3Default => Some(INDUSTRIAL_V3_DEFAULT_PRESET_PATH),
            Self::IndustrialV3Matte => Some(INDUSTRIAL_V3_MATTE_PRESET_PATH),
            Self::IndustrialV3Glossy => Some(INDUSTRIAL_V3_GLOSSY_PRESET_PATH),
            Self::IndustrialV2Default => Some(INDUSTRIAL_V2_DEFAULT_PRESET_PATH),
            Self::IndustrialV2Matte => Some(INDUSTRIAL_V2_MATTE_PRESET_PATH),
            Self::IndustrialV2Glossy => Some(INDUSTRIAL_V2_GLOSSY_PRESET_PATH),
            Self::None | Self::Custom => None,
        }
    }

    fn builtin_embedded(self) -> Option<&'static str> {
        match self {
            Self::IndustrialV3Default => Some(INDUSTRIAL_V3_DEFAULT_PRESET_EMBEDDED),
            Self::IndustrialV3Matte => Some(INDUSTRIAL_V3_MATTE_PRESET_EMBEDDED),
            Self::IndustrialV3Glossy => Some(INDUSTRIAL_V3_GLOSSY_PRESET_EMBEDDED),
            Self::IndustrialV2Default => Some(INDUSTRIAL_V2_DEFAULT_PRESET_EMBEDDED),
            Self::IndustrialV2Matte => Some(INDUSTRIAL_V2_MATTE_PRESET_EMBEDDED),
            Self::IndustrialV2Glossy => Some(INDUSTRIAL_V2_GLOSSY_PRESET_EMBEDDED),
            Self::None | Self::Custom => None,
        }
    }
}

#[derive(Resource, Debug)]
pub(crate) struct ThemeRuntimeState {
    pub selection: ThemePresetSelection,
    pub custom_preset_path: String,
    pub hot_reload_enabled: bool,
    pub pending_apply: bool,
    pub status_message: String,
    last_applied_path: Option<String>,
    last_applied_modified_ms: Option<u128>,
}

impl Default for ThemeRuntimeState {
    fn default() -> Self {
        Self {
            selection: ThemePresetSelection::None,
            custom_preset_path: String::new(),
            hot_reload_enabled: false,
            pending_apply: false,
            status_message: "Theme runtime: idle".to_string(),
            last_applied_path: None,
            last_applied_modified_ms: None,
        }
    }
}

impl ThemeRuntimeState {
    fn active_preset_path(&self) -> Option<String> {
        match self.selection {
            ThemePresetSelection::None => None,
            ThemePresetSelection::Custom => {
                let path = self.custom_preset_path.trim();
                if path.is_empty() {
                    None
                } else {
                    Some(path.to_string())
                }
            }
            _ => self.selection.builtin_path().map(str::to_string),
        }
    }
}

pub(crate) fn resolve_theme_runtime_state() -> ThemeRuntimeState {
    let preset_selection = std::env::var(THEME_PRESET_ENV)
        .ok()
        .and_then(|raw| ThemePresetSelection::from_env(raw.as_str()));
    let preset_file = std::env::var(THEME_PRESET_FILE_ENV).unwrap_or_default();
    let hot_reload_enabled = parse_toggle(std::env::var(THEME_HOT_RELOAD_ENV).ok().as_deref());

    let (selection, custom_preset_path, pending_apply) = if !preset_file.trim().is_empty() {
        (ThemePresetSelection::Custom, preset_file, true)
    } else if let Some(selection) = preset_selection {
        (
            selection,
            String::new(),
            selection != ThemePresetSelection::None,
        )
    } else {
        (ThemePresetSelection::None, String::new(), false)
    };

    ThemeRuntimeState {
        selection,
        custom_preset_path,
        hot_reload_enabled,
        pending_apply,
        status_message: "Theme runtime: idle".to_string(),
        last_applied_path: None,
        last_applied_modified_ms: None,
    }
}

pub(crate) fn apply_theme_runtime_updates(
    mut theme_state: ResMut<ThemeRuntimeState>,
    config: Res<crate::Viewer3dConfig>,
    mut external_mesh: ResMut<crate::ViewerExternalMeshConfig>,
    mut external_material: ResMut<crate::ViewerExternalMaterialConfig>,
    mut external_texture: ResMut<crate::ViewerExternalTextureConfig>,
    mut variant_preview: ResMut<crate::MaterialVariantPreviewState>,
    mut scene: ResMut<crate::Viewer3dScene>,
    assets: Option<ResMut<crate::Viewer3dAssets>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let Some(mut assets) = assets else {
        return;
    };

    let preset_path = theme_state.active_preset_path();
    let mut should_apply = theme_state.pending_apply;
    if !should_apply && theme_state.hot_reload_enabled {
        if let Some(path) = preset_path.as_deref() {
            let current_mtime = preset_modified_millis(path);
            let known_path = theme_state.last_applied_path.as_deref();
            let known_mtime = theme_state.last_applied_modified_ms;
            should_apply = known_path != Some(path)
                || match (known_mtime, current_mtime) {
                    (Some(previous), Some(current)) => current > previous,
                    (None, Some(_)) => true,
                    _ => false,
                };
        }
    }

    if !should_apply {
        return;
    }

    theme_state.pending_apply = false;
    let Some(path) = preset_path else {
        theme_state.status_message = "Theme runtime: no preset selected".to_string();
        return;
    };

    let apply_result = apply_theme_from_path(
        path.as_str(),
        &config,
        &mut external_mesh,
        &mut external_material,
        &mut external_texture,
        &mut variant_preview,
        &mut scene,
        &mut assets,
        &mut meshes,
        &mut materials,
        &asset_server,
    );

    match apply_result {
        Ok(summary) => {
            theme_state.status_message = summary;
            theme_state.last_applied_modified_ms = preset_modified_millis(path.as_str());
            theme_state.last_applied_path = Some(path);
        }
        Err(error) => {
            theme_state.status_message = format!("Theme runtime error: {error}");
        }
    }
}

fn apply_theme_from_path(
    path: &str,
    config: &crate::Viewer3dConfig,
    external_mesh: &mut crate::ViewerExternalMeshConfig,
    external_material: &mut crate::ViewerExternalMaterialConfig,
    external_texture: &mut crate::ViewerExternalTextureConfig,
    variant_preview: &mut crate::MaterialVariantPreviewState,
    scene: &mut crate::Viewer3dScene,
    assets: &mut crate::Viewer3dAssets,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
) -> Result<String, String> {
    let preset_vars = load_preset_vars(path)?;
    *external_mesh = parse_external_mesh_config(&preset_vars);
    *external_material = parse_external_material_config(&preset_vars);
    *external_texture = parse_external_texture_config(&preset_vars);

    if let Some(parsed_variant) = preset_vars
        .get("AGENT_WORLD_VIEWER_MATERIAL_VARIANT_PRESET")
        .and_then(|raw| crate::parse_material_variant_preset(raw))
    {
        variant_preview.active = parsed_variant;
    }

    apply_theme_to_assets_and_scene(
        config,
        external_mesh,
        external_material,
        external_texture,
        variant_preview,
        scene,
        assets,
        meshes,
        materials,
        asset_server,
    );

    Ok(format!(
        "Theme runtime: applied {} ({:?})",
        path, variant_preview.active
    ))
}

fn apply_theme_to_assets_and_scene(
    config: &crate::Viewer3dConfig,
    external_mesh: &crate::ViewerExternalMeshConfig,
    external_material: &crate::ViewerExternalMaterialConfig,
    external_texture: &crate::ViewerExternalTextureConfig,
    variant_preview: &crate::MaterialVariantPreviewState,
    scene: &mut crate::Viewer3dScene,
    assets: &mut crate::Viewer3dAssets,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &AssetServer,
) {
    let resolved_theme_assets = crate::resolve_theme_scene_assets(
        config,
        external_mesh,
        external_material,
        external_texture,
        variant_preview,
        meshes,
        asset_server,
    );
    assets.agent_mesh = resolved_theme_assets.agent_mesh;
    assets.location_mesh = resolved_theme_assets.location_mesh;
    assets.asset_mesh = resolved_theme_assets.asset_mesh;
    assets.power_plant_mesh = resolved_theme_assets.power_plant_mesh;

    write_material(
        materials,
        &assets.agent_material,
        resolved_theme_assets.agent_material,
    );
    write_material(
        materials,
        &assets.asset_material,
        resolved_theme_assets.asset_material,
    );
    write_material(
        materials,
        &assets.power_plant_material,
        resolved_theme_assets.power_plant_material,
    );

    if let Some(location_override) = resolved_theme_assets.location_override_materials {
        assets.location_core_silicate_material = materials.add(location_override.core_silicate);
        assets.location_core_metal_material = materials.add(location_override.core_metal);
        assets.location_core_ice_material = materials.add(location_override.core_ice);
        assets.location_halo_material = materials.add(location_override.halo);
    } else {
        assets.location_core_silicate_material = assets.chunk_unexplored_material.clone();
        assets.location_core_metal_material = assets.chunk_generated_material.clone();
        assets.location_core_ice_material = assets.chunk_exhausted_material.clone();
        assets.location_halo_material = assets.world_bounds_material.clone();
    }

    scene.last_snapshot_time = None;
    scene.last_event_id = None;
}

fn write_material(
    materials: &mut Assets<StandardMaterial>,
    handle: &Handle<StandardMaterial>,
    template: StandardMaterial,
) {
    if let Some(material) = materials.get_mut(handle) {
        *material = template;
    }
}

fn parse_external_mesh_config(vars: &HashMap<String, String>) -> crate::ViewerExternalMeshConfig {
    crate::load_viewer_external_mesh_config_from(|key| vars.get(key).cloned())
}

fn parse_external_material_config(
    vars: &HashMap<String, String>,
) -> crate::ViewerExternalMaterialConfig {
    crate::load_viewer_external_material_config_from(|key| vars.get(key).cloned())
}

fn parse_external_texture_config(
    vars: &HashMap<String, String>,
) -> crate::ViewerExternalTextureConfig {
    crate::load_viewer_external_texture_config_from(|key| vars.get(key).cloned())
}

fn parse_toggle(raw: Option<&str>) -> bool {
    raw.map(|value| value.trim().to_ascii_lowercase())
        .map(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

fn load_preset_vars(path: &str) -> Result<HashMap<String, String>, String> {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(preset) = embedded_preset_for_path(path) {
            return Ok(parse_preset_vars_from_text(preset));
        }
        return Err(format!("web runtime cannot read preset file: {path}"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(content) = std::fs::read_to_string(path) {
            return Ok(parse_preset_vars_from_text(content.as_str()));
        }
        if let Some(preset) = embedded_preset_for_path(path) {
            return Ok(parse_preset_vars_from_text(preset));
        }
        Err(format!("failed to read preset file: {path}"))
    }
}

fn parse_preset_vars_from_text(content: &str) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let line = line.strip_prefix("export ").unwrap_or(line);
        if let Some(rest) = line.strip_prefix("unset ") {
            vars.remove(rest.trim());
            continue;
        }

        if let Some((key, raw_value)) = line.split_once('=') {
            let key = key.trim();
            if key.is_empty() {
                continue;
            }
            let value = trim_wrapping_quotes(raw_value.trim()).to_string();
            vars.insert(key.to_string(), value);
        }
    }
    vars
}

fn trim_wrapping_quotes(raw: &str) -> &str {
    if raw.len() >= 2
        && ((raw.starts_with('"') && raw.ends_with('"'))
            || (raw.starts_with('\'') && raw.ends_with('\'')))
    {
        &raw[1..raw.len() - 1]
    } else {
        raw
    }
}

fn embedded_preset_for_path(path: &str) -> Option<&'static str> {
    match path {
        INDUSTRIAL_V3_DEFAULT_PRESET_PATH => {
            ThemePresetSelection::IndustrialV3Default.builtin_embedded()
        }
        INDUSTRIAL_V3_MATTE_PRESET_PATH => {
            ThemePresetSelection::IndustrialV3Matte.builtin_embedded()
        }
        INDUSTRIAL_V3_GLOSSY_PRESET_PATH => {
            ThemePresetSelection::IndustrialV3Glossy.builtin_embedded()
        }
        INDUSTRIAL_V2_DEFAULT_PRESET_PATH => {
            ThemePresetSelection::IndustrialV2Default.builtin_embedded()
        }
        INDUSTRIAL_V2_MATTE_PRESET_PATH => {
            ThemePresetSelection::IndustrialV2Matte.builtin_embedded()
        }
        INDUSTRIAL_V2_GLOSSY_PRESET_PATH => {
            ThemePresetSelection::IndustrialV2Glossy.builtin_embedded()
        }
        _ => None,
    }
}

#[cfg(target_arch = "wasm32")]
fn preset_modified_millis(_path: &str) -> Option<u128> {
    None
}

#[cfg(not(target_arch = "wasm32"))]
fn preset_modified_millis(path: &str) -> Option<u128> {
    let metadata = std::fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let duration = modified.duration_since(std::time::UNIX_EPOCH).ok()?;
    Some(duration.as_millis())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_preset_vars_from_text_reads_exports_and_unset() {
        let vars = parse_preset_vars_from_text(
            r#"
            # comment
            export A=1
            B='hello'
            unset A
            C="value with spaces"
            "#,
        );
        assert_eq!(vars.get("A"), None);
        assert_eq!(vars.get("B").map(String::as_str), Some("hello"));
        assert_eq!(vars.get("C").map(String::as_str), Some("value with spaces"));
    }

    #[test]
    fn resolve_theme_runtime_state_uses_preset_file_when_provided() {
        let state = ThemeRuntimeState {
            selection: ThemePresetSelection::Custom,
            custom_preset_path: "abc.env".to_string(),
            hot_reload_enabled: false,
            pending_apply: true,
            status_message: String::new(),
            last_applied_path: None,
            last_applied_modified_ms: None,
        };
        assert_eq!(state.active_preset_path().as_deref(), Some("abc.env"));
    }

    #[test]
    fn parse_external_texture_config_reads_all_channels() {
        let mut vars = HashMap::new();
        vars.insert(
            "AGENT_WORLD_VIEWER_AGENT_BASE_TEXTURE_ASSET".to_string(),
            "base.png".to_string(),
        );
        vars.insert(
            "AGENT_WORLD_VIEWER_AGENT_NORMAL_TEXTURE_ASSET".to_string(),
            "normal.png".to_string(),
        );
        vars.insert(
            "AGENT_WORLD_VIEWER_AGENT_METALLIC_ROUGHNESS_TEXTURE_ASSET".to_string(),
            "mr.png".to_string(),
        );
        vars.insert(
            "AGENT_WORLD_VIEWER_AGENT_EMISSIVE_TEXTURE_ASSET".to_string(),
            "emissive.png".to_string(),
        );
        let slot = parse_external_texture_config(&vars).agent;
        assert_eq!(slot.base_texture_asset.as_deref(), Some("base.png"));
        assert_eq!(slot.normal_texture_asset.as_deref(), Some("normal.png"));
        assert_eq!(
            slot.metallic_roughness_texture_asset.as_deref(),
            Some("mr.png")
        );
        assert_eq!(slot.emissive_texture_asset.as_deref(), Some("emissive.png"));
    }

    #[test]
    fn theme_preset_from_env_prefers_v3_for_plain_aliases() {
        assert_eq!(
            ThemePresetSelection::from_env("default"),
            Some(ThemePresetSelection::IndustrialV3Default)
        );
        assert_eq!(
            ThemePresetSelection::from_env("matte"),
            Some(ThemePresetSelection::IndustrialV3Matte)
        );
        assert_eq!(
            ThemePresetSelection::from_env("glossy"),
            Some(ThemePresetSelection::IndustrialV3Glossy)
        );
    }

    #[test]
    fn theme_preset_from_env_keeps_v2_compat_aliases() {
        assert_eq!(
            ThemePresetSelection::from_env("industrial_v2_default"),
            Some(ThemePresetSelection::IndustrialV2Default)
        );
        assert_eq!(
            ThemePresetSelection::from_env("v2_matte"),
            Some(ThemePresetSelection::IndustrialV2Matte)
        );
        assert_eq!(
            ThemePresetSelection::from_env("industrial-v2-glossy"),
            Some(ThemePresetSelection::IndustrialV2Glossy)
        );
    }

    #[test]
    fn embedded_preset_for_path_supports_v3_and_v2() {
        assert!(embedded_preset_for_path(INDUSTRIAL_V3_DEFAULT_PRESET_PATH).is_some());
        assert!(embedded_preset_for_path(INDUSTRIAL_V3_MATTE_PRESET_PATH).is_some());
        assert!(embedded_preset_for_path(INDUSTRIAL_V3_GLOSSY_PRESET_PATH).is_some());
        assert!(embedded_preset_for_path(INDUSTRIAL_V2_DEFAULT_PRESET_PATH).is_some());
    }
}
