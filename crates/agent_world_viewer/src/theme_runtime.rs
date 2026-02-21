use std::collections::HashMap;

use bevy::prelude::*;

const THEME_PRESET_ENV: &str = "AGENT_WORLD_VIEWER_THEME_PRESET";
const THEME_PRESET_FILE_ENV: &str = "AGENT_WORLD_VIEWER_THEME_PRESET_FILE";
const THEME_HOT_RELOAD_ENV: &str = "AGENT_WORLD_VIEWER_THEME_HOT_RELOAD";

const INDUSTRIAL_V2_DEFAULT_PRESET_PATH: &str =
    "crates/agent_world_viewer/assets/themes/industrial_v2/presets/industrial_v2_default.env";
const INDUSTRIAL_V2_MATTE_PRESET_PATH: &str =
    "crates/agent_world_viewer/assets/themes/industrial_v2/presets/industrial_v2_matte.env";
const INDUSTRIAL_V2_GLOSSY_PRESET_PATH: &str =
    "crates/agent_world_viewer/assets/themes/industrial_v2/presets/industrial_v2_glossy.env";

const INDUSTRIAL_V2_DEFAULT_PRESET_EMBEDDED: &str =
    include_str!("../assets/themes/industrial_v2/presets/industrial_v2_default.env");
const INDUSTRIAL_V2_MATTE_PRESET_EMBEDDED: &str =
    include_str!("../assets/themes/industrial_v2/presets/industrial_v2_matte.env");
const INDUSTRIAL_V2_GLOSSY_PRESET_EMBEDDED: &str =
    include_str!("../assets/themes/industrial_v2/presets/industrial_v2_glossy.env");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ThemePresetSelection {
    None,
    IndustrialV2Default,
    IndustrialV2Matte,
    IndustrialV2Glossy,
    Custom,
}

impl ThemePresetSelection {
    pub(crate) const ORDERED: [Self; 5] = [
        Self::None,
        Self::IndustrialV2Default,
        Self::IndustrialV2Matte,
        Self::IndustrialV2Glossy,
        Self::Custom,
    ];

    pub(crate) fn from_env(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "none" | "off" | "disabled" => Some(Self::None),
            "industrial_v2_default" | "industrial-v2-default" | "v2_default" | "default" => {
                Some(Self::IndustrialV2Default)
            }
            "industrial_v2_matte" | "industrial-v2-matte" | "v2_matte" | "matte" => {
                Some(Self::IndustrialV2Matte)
            }
            "industrial_v2_glossy" | "industrial-v2-glossy" | "v2_glossy" | "glossy" => {
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
            Self::IndustrialV2Default => Some(INDUSTRIAL_V2_DEFAULT_PRESET_PATH),
            Self::IndustrialV2Matte => Some(INDUSTRIAL_V2_MATTE_PRESET_PATH),
            Self::IndustrialV2Glossy => Some(INDUSTRIAL_V2_GLOSSY_PRESET_PATH),
            Self::None | Self::Custom => None,
        }
    }

    fn builtin_embedded(self) -> Option<&'static str> {
        match self {
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
    let geometry_tier = config.assets.geometry_tier;
    assets.agent_mesh = crate::resolve_mesh_handle(
        asset_server,
        meshes,
        external_mesh.agent_mesh_asset.as_deref(),
        || Capsule3d::new(crate::AGENT_BODY_MESH_RADIUS, crate::AGENT_BODY_MESH_LENGTH).into(),
    );
    assets.location_mesh = crate::resolve_mesh_handle(
        asset_server,
        meshes,
        external_mesh.location_mesh_asset.as_deref(),
        || crate::location_mesh_for_geometry_tier(geometry_tier),
    );
    assets.asset_mesh = crate::resolve_mesh_handle(
        asset_server,
        meshes,
        external_mesh.asset_mesh_asset.as_deref(),
        || crate::asset_mesh_for_geometry_tier(geometry_tier),
    );
    assets.power_plant_mesh = crate::resolve_mesh_handle(
        asset_server,
        meshes,
        external_mesh.power_plant_mesh_asset.as_deref(),
        || crate::power_plant_mesh_for_geometry_tier(geometry_tier),
    );
    assets.power_storage_mesh = crate::resolve_mesh_handle(
        asset_server,
        meshes,
        external_mesh.power_storage_mesh_asset.as_deref(),
        || crate::power_storage_mesh_for_geometry_tier(geometry_tier),
    );

    let agent_texture = crate::resolve_texture_slot(asset_server, &external_texture.agent);
    let location_texture = crate::resolve_texture_slot(asset_server, &external_texture.location);
    let asset_texture = crate::resolve_texture_slot(asset_server, &external_texture.asset);
    let power_plant_texture =
        crate::resolve_texture_slot(asset_server, &external_texture.power_plant);
    let power_storage_texture =
        crate::resolve_texture_slot(asset_server, &external_texture.power_storage);

    let scalars = crate::material_variant_scalars(variant_preview.active);
    let agent_roughness = crate::apply_material_variant_scalar(
        config.materials.agent.roughness,
        scalars.roughness_scale,
    );
    let agent_metallic = crate::apply_material_variant_scalar(
        config.materials.agent.metallic,
        scalars.metallic_scale,
    );
    let asset_roughness = crate::apply_material_variant_scalar(
        config.materials.asset.roughness,
        scalars.roughness_scale,
    );
    let asset_metallic = crate::apply_material_variant_scalar(
        config.materials.asset.metallic,
        scalars.metallic_scale,
    );
    let facility_roughness = crate::apply_material_variant_scalar(
        config.materials.facility.roughness,
        scalars.roughness_scale,
    );
    let facility_metallic = crate::apply_material_variant_scalar(
        config.materials.facility.metallic,
        scalars.metallic_scale,
    );

    let agent_base_color =
        crate::resolve_srgb_slot_color([1.0, 0.42, 0.22], external_material.agent.base_color_srgb);
    let agent_emissive_color = crate::resolve_srgb_slot_color(
        [0.90, 0.38, 0.20],
        external_material.agent.emissive_color_srgb,
    );
    write_material(
        materials,
        &assets.agent_material,
        StandardMaterial {
            base_color: crate::color_from_srgb(agent_base_color),
            base_color_texture: agent_texture.base_color_texture,
            normal_map_texture: agent_texture.normal_map_texture,
            metallic_roughness_texture: agent_texture.metallic_roughness_texture,
            emissive_texture: agent_texture.emissive_texture,
            perceptual_roughness: agent_roughness,
            metallic: agent_metallic,
            emissive: crate::emissive_from_srgb_with_boost(
                agent_emissive_color,
                config.materials.agent.emissive_boost,
            ),
            ..default()
        },
    );

    let asset_base_color =
        crate::resolve_srgb_slot_color([0.82, 0.76, 0.34], external_material.asset.base_color_srgb);
    let asset_emissive_color = crate::resolve_srgb_slot_color(
        [0.82, 0.76, 0.34],
        external_material.asset.emissive_color_srgb,
    );
    write_material(
        materials,
        &assets.asset_material,
        StandardMaterial {
            base_color: crate::color_from_srgb(asset_base_color),
            base_color_texture: asset_texture.base_color_texture,
            normal_map_texture: asset_texture.normal_map_texture,
            metallic_roughness_texture: asset_texture.metallic_roughness_texture,
            emissive_texture: asset_texture.emissive_texture,
            perceptual_roughness: asset_roughness,
            metallic: asset_metallic,
            emissive: crate::emissive_from_srgb_with_boost(
                asset_emissive_color,
                config.materials.asset.emissive_boost,
            ),
            ..default()
        },
    );

    let power_plant_base_color = crate::resolve_srgb_slot_color(
        [0.95, 0.42, 0.20],
        external_material.power_plant.base_color_srgb,
    );
    let power_plant_emissive_color = crate::resolve_srgb_slot_color(
        [0.95, 0.42, 0.20],
        external_material.power_plant.emissive_color_srgb,
    );
    write_material(
        materials,
        &assets.power_plant_material,
        StandardMaterial {
            base_color: crate::color_from_srgb(power_plant_base_color),
            base_color_texture: power_plant_texture.base_color_texture,
            normal_map_texture: power_plant_texture.normal_map_texture,
            metallic_roughness_texture: power_plant_texture.metallic_roughness_texture,
            emissive_texture: power_plant_texture.emissive_texture,
            perceptual_roughness: facility_roughness,
            metallic: facility_metallic,
            emissive: crate::emissive_from_srgb_with_boost(
                power_plant_emissive_color,
                config.materials.facility.emissive_boost,
            ),
            ..default()
        },
    );

    let power_storage_base_color = crate::resolve_srgb_slot_color(
        [0.20, 0.86, 0.48],
        external_material.power_storage.base_color_srgb,
    );
    let power_storage_emissive_color = crate::resolve_srgb_slot_color(
        [0.20, 0.86, 0.48],
        external_material.power_storage.emissive_color_srgb,
    );
    write_material(
        materials,
        &assets.power_storage_material,
        StandardMaterial {
            base_color: crate::color_from_srgb(power_storage_base_color),
            base_color_texture: power_storage_texture.base_color_texture,
            normal_map_texture: power_storage_texture.normal_map_texture,
            metallic_roughness_texture: power_storage_texture.metallic_roughness_texture,
            emissive_texture: power_storage_texture.emissive_texture,
            perceptual_roughness: facility_roughness,
            metallic: facility_metallic,
            emissive: crate::emissive_from_srgb_with_boost(
                power_storage_emissive_color,
                config.materials.facility.emissive_boost,
            ),
            ..default()
        },
    );

    if crate::location_style_override_enabled(
        external_material.location,
        &external_texture.location,
    ) {
        let location_base_color = crate::resolve_srgb_slot_color(
            [0.30, 0.42, 0.66],
            external_material.location.base_color_srgb,
        );
        let location_emissive_color = crate::resolve_srgb_slot_color(
            location_base_color,
            external_material.location.emissive_color_srgb,
        );
        let location_core = |alpha: f32| StandardMaterial {
            base_color: crate::color_from_srgb_with_alpha(location_base_color, alpha),
            base_color_texture: location_texture.base_color_texture.clone(),
            normal_map_texture: location_texture.normal_map_texture.clone(),
            metallic_roughness_texture: location_texture.metallic_roughness_texture.clone(),
            emissive_texture: location_texture.emissive_texture.clone(),
            perceptual_roughness: facility_roughness,
            metallic: facility_metallic,
            emissive: crate::color_from_srgb(location_emissive_color).into(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        };
        assets.location_core_silicate_material = materials.add(location_core(0.22));
        assets.location_core_metal_material = materials.add(location_core(0.30));
        assets.location_core_ice_material = materials.add(location_core(0.30));
        assets.location_halo_material = materials.add(StandardMaterial {
            base_color: crate::color_from_srgb_with_alpha(location_base_color, 0.10),
            base_color_texture: location_texture.base_color_texture,
            normal_map_texture: location_texture.normal_map_texture,
            metallic_roughness_texture: location_texture.metallic_roughness_texture,
            emissive_texture: location_texture.emissive_texture,
            emissive: crate::color_from_srgb(location_emissive_color).into(),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
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
    crate::ViewerExternalMeshConfig {
        agent_mesh_asset: value_non_empty(vars, "AGENT_WORLD_VIEWER_AGENT_MESH_ASSET"),
        location_mesh_asset: value_non_empty(vars, "AGENT_WORLD_VIEWER_LOCATION_MESH_ASSET"),
        asset_mesh_asset: value_non_empty(vars, "AGENT_WORLD_VIEWER_ASSET_MESH_ASSET"),
        power_plant_mesh_asset: value_non_empty(vars, "AGENT_WORLD_VIEWER_POWER_PLANT_MESH_ASSET"),
        power_storage_mesh_asset: value_non_empty(
            vars,
            "AGENT_WORLD_VIEWER_POWER_STORAGE_MESH_ASSET",
        ),
    }
}

fn parse_external_material_config(
    vars: &HashMap<String, String>,
) -> crate::ViewerExternalMaterialConfig {
    crate::ViewerExternalMaterialConfig {
        agent: crate::ViewerExternalMaterialSlotConfig {
            base_color_srgb: value_hex_color(vars, "AGENT_WORLD_VIEWER_AGENT_BASE_COLOR"),
            emissive_color_srgb: value_hex_color(vars, "AGENT_WORLD_VIEWER_AGENT_EMISSIVE_COLOR"),
        },
        location: crate::ViewerExternalMaterialSlotConfig {
            base_color_srgb: value_hex_color(vars, "AGENT_WORLD_VIEWER_LOCATION_BASE_COLOR"),
            emissive_color_srgb: value_hex_color(
                vars,
                "AGENT_WORLD_VIEWER_LOCATION_EMISSIVE_COLOR",
            ),
        },
        asset: crate::ViewerExternalMaterialSlotConfig {
            base_color_srgb: value_hex_color(vars, "AGENT_WORLD_VIEWER_ASSET_BASE_COLOR"),
            emissive_color_srgb: value_hex_color(vars, "AGENT_WORLD_VIEWER_ASSET_EMISSIVE_COLOR"),
        },
        power_plant: crate::ViewerExternalMaterialSlotConfig {
            base_color_srgb: value_hex_color(vars, "AGENT_WORLD_VIEWER_POWER_PLANT_BASE_COLOR"),
            emissive_color_srgb: value_hex_color(
                vars,
                "AGENT_WORLD_VIEWER_POWER_PLANT_EMISSIVE_COLOR",
            ),
        },
        power_storage: crate::ViewerExternalMaterialSlotConfig {
            base_color_srgb: value_hex_color(vars, "AGENT_WORLD_VIEWER_POWER_STORAGE_BASE_COLOR"),
            emissive_color_srgb: value_hex_color(
                vars,
                "AGENT_WORLD_VIEWER_POWER_STORAGE_EMISSIVE_COLOR",
            ),
        },
    }
}

fn parse_external_texture_config(
    vars: &HashMap<String, String>,
) -> crate::ViewerExternalTextureConfig {
    crate::ViewerExternalTextureConfig {
        agent: parse_external_texture_slot(vars, "AGENT"),
        location: parse_external_texture_slot(vars, "LOCATION"),
        asset: parse_external_texture_slot(vars, "ASSET"),
        power_plant: parse_external_texture_slot(vars, "POWER_PLANT"),
        power_storage: parse_external_texture_slot(vars, "POWER_STORAGE"),
    }
}

fn parse_external_texture_slot(
    vars: &HashMap<String, String>,
    entity: &str,
) -> crate::ViewerExternalTextureSlotConfig {
    let key = |suffix: &str| format!("AGENT_WORLD_VIEWER_{entity}_{suffix}");
    crate::ViewerExternalTextureSlotConfig {
        base_texture_asset: value_non_empty(vars, key("BASE_TEXTURE_ASSET").as_str()),
        normal_texture_asset: value_non_empty(vars, key("NORMAL_TEXTURE_ASSET").as_str()),
        metallic_roughness_texture_asset: value_non_empty(
            vars,
            key("METALLIC_ROUGHNESS_TEXTURE_ASSET").as_str(),
        ),
        emissive_texture_asset: value_non_empty(vars, key("EMISSIVE_TEXTURE_ASSET").as_str()),
    }
}

fn value_non_empty(vars: &HashMap<String, String>, key: &str) -> Option<String> {
    vars.get(key).and_then(|raw| {
        let value = raw.trim();
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    })
}

fn value_hex_color(vars: &HashMap<String, String>, key: &str) -> Option<[f32; 3]> {
    let raw = vars.get(key)?;
    let raw = raw.trim();
    let hex = raw.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0])
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
    fn parse_external_texture_slot_reads_all_channels() {
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
        let slot = parse_external_texture_slot(&vars, "AGENT");
        assert_eq!(slot.base_texture_asset.as_deref(), Some("base.png"));
        assert_eq!(slot.normal_texture_asset.as_deref(), Some("normal.png"));
        assert_eq!(
            slot.metallic_roughness_texture_asset.as_deref(),
            Some("mr.png")
        );
        assert_eq!(slot.emissive_texture_asset.as_deref(), Some("emissive.png"));
    }
}
