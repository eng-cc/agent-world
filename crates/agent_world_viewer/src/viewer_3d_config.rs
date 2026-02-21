use bevy::prelude::Resource;

const DEFAULT_CM_TO_UNIT: f32 = 0.00001;
const DEFAULT_PHYSICAL_ENABLED: bool = false;
const DEFAULT_METERS_PER_UNIT: f32 = 1.0;
const DEFAULT_FLOATING_ORIGIN_STEP_M: f64 = 1000.0;
const DEFAULT_CAMERA_NEAR_M: f32 = 0.1;
const DEFAULT_CAMERA_FAR_M: f32 = 25_000.0;
const DEFAULT_STELLAR_DISTANCE_AU: f32 = 2.5;
const DEFAULT_LUMINOUS_EFFICACY_LM_PER_W: f32 = 120.0;
const DEFAULT_EXPOSURE_EV100: f32 = 13.5;
const DEFAULT_REFERENCE_RADIATION_AREA_M2: f32 = 1.0;
const SOLAR_CONSTANT_W_M2_AT_1_AU: f32 = 1361.0;
const BASELINE_EXPOSURE_EV100: f32 = 13.5;
const MIN_LIGHT_ILLUMINANCE_LUX: f32 = 2_500.0;
const MAX_LIGHT_ILLUMINANCE_LUX: f32 = 120_000.0;
const DEFAULT_SHADOWS_ENABLED: bool = false;
const DEFAULT_AMBIENT_BRIGHTNESS: f32 = 110.0;
const DEFAULT_FILL_LIGHT_RATIO: f32 = 0.28;
const DEFAULT_RIM_LIGHT_RATIO: f32 = 0.14;
const DEFAULT_LABEL_FADE_START_DISTANCE: f32 = 55.0;
const DEFAULT_LABEL_FADE_END_DISTANCE: f32 = 140.0;
const DEFAULT_MAX_VISIBLE_LABELS: usize = 48;
const DEFAULT_LABEL_OCCLUSION_CELL_SPAN: f32 = 8.0;
const DEFAULT_LABEL_OCCLUSION_CAP_PER_CELL: usize = 2;
const DEFAULT_OVERLAY_REFRESH_TICKS: u64 = 3;
const DEFAULT_OVERLAY_MAX_HEAT_MARKERS: usize = 64;
const DEFAULT_OVERLAY_MAX_FLOW_SEGMENTS: usize = 80;
const DEFAULT_GRID_LOD_DISTANCE: f32 = 120.0;
const DEFAULT_LOCATION_SHELL_ENABLED: bool = true;
const DEFAULT_MATERIAL_AGENT_ROUGHNESS: f32 = 0.38;
const DEFAULT_MATERIAL_AGENT_METALLIC: f32 = 0.08;
const DEFAULT_MATERIAL_AGENT_EMISSIVE_BOOST: f32 = 0.06;
const DEFAULT_MATERIAL_ASSET_ROUGHNESS: f32 = 0.55;
const DEFAULT_MATERIAL_ASSET_METALLIC: f32 = 0.12;
const DEFAULT_MATERIAL_ASSET_EMISSIVE_BOOST: f32 = 0.02;
const DEFAULT_MATERIAL_FACILITY_ROUGHNESS: f32 = 0.48;
const DEFAULT_MATERIAL_FACILITY_METALLIC: f32 = 0.20;
const DEFAULT_MATERIAL_FACILITY_EMISSIVE_BOOST: f32 = 0.08;
const DEFAULT_FRAGMENT_UNLIT: bool = true;
const DEFAULT_FRAGMENT_ALPHA: f32 = 0.92;
const DEFAULT_FRAGMENT_EMISSIVE_BOOST: f32 = 0.24;
const MIN_FRAGMENT_ALPHA: f32 = 0.05;
const MAX_FRAGMENT_ALPHA: f32 = 1.0;
const DEFAULT_DEBAND_DITHER_ENABLED: bool = true;
const DEFAULT_BLOOM_ENABLED: bool = true;
const DEFAULT_BLOOM_INTENSITY: f32 = 0.15;
const MIN_BLOOM_INTENSITY: f32 = 0.0;
const MAX_BLOOM_INTENSITY: f32 = 2.0;
const DEFAULT_COLOR_GRADING_EXPOSURE: f32 = 0.0;
const MIN_COLOR_GRADING_EXPOSURE: f32 = -8.0;
const MAX_COLOR_GRADING_EXPOSURE: f32 = 8.0;
const DEFAULT_COLOR_GRADING_POST_SATURATION: f32 = 1.0;
const MIN_COLOR_GRADING_POST_SATURATION: f32 = 0.0;
const MAX_COLOR_GRADING_POST_SATURATION: f32 = 2.0;

#[derive(Clone, Copy, Debug, Resource)]
pub(super) struct Viewer3dConfig {
    pub cm_to_unit: f32,
    pub render_profile: ViewerRenderProfile,
    pub show_agents: bool,
    pub show_locations: bool,
    pub highlight_selected: bool,
    pub assets: ViewerAssetConfig,
    pub materials: ViewerMaterialConfig,
    pub post_process: ViewerPostProcessConfig,
    pub label_lod: ViewerLabelLodConfig,
    pub render_budget: ViewerRenderBudgetConfig,
    pub lighting: ViewerLightingConfig,
    pub physical: ViewerPhysicalRenderConfig,
}

impl Viewer3dConfig {
    pub(super) fn effective_cm_to_unit(&self) -> f32 {
        if self.physical.enabled {
            (self.physical.meters_per_unit / 100.0).max(f32::EPSILON)
        } else {
            self.cm_to_unit
        }
    }
}

impl Default for Viewer3dConfig {
    fn default() -> Self {
        Self {
            cm_to_unit: DEFAULT_CM_TO_UNIT,
            render_profile: ViewerRenderProfile::default(),
            show_agents: true,
            show_locations: true,
            highlight_selected: true,
            assets: ViewerAssetConfig::default(),
            materials: ViewerMaterialConfig::default(),
            post_process: ViewerPostProcessConfig::default(),
            label_lod: ViewerLabelLodConfig::default(),
            render_budget: ViewerRenderBudgetConfig::default(),
            lighting: ViewerLightingConfig::default(),
            physical: ViewerPhysicalRenderConfig::default(),
        }
    }
}

impl Viewer3dConfig {
    pub(super) fn apply_render_profile(&mut self, profile: ViewerRenderProfile) {
        self.render_profile = profile;
        match profile {
            ViewerRenderProfile::Debug => {
                self.assets.geometry_tier = ViewerGeometryTier::Debug;
                self.assets.location_shell_enabled = false;
                self.materials.fragment.strategy = ViewerFragmentMaterialStrategy::Readability;
                self.materials.fragment.unlit = true;
                self.materials.fragment.alpha = 0.95;
                self.materials.fragment.emissive_boost = 0.34;
                self.lighting.shadows_enabled = false;
                self.lighting.ambient_brightness = 120.0;
                self.lighting.fill_light_ratio = 0.34;
                self.lighting.rim_light_ratio = 0.06;
                self.post_process.tonemapping = ViewerTonemappingMode::Reinhard;
                self.post_process.deband_dither_enabled = false;
                self.post_process.bloom_enabled = false;
                self.post_process.bloom_intensity = 0.08;
                self.post_process.color_grading_exposure = 0.0;
                self.post_process.color_grading_post_saturation = 1.0;
            }
            ViewerRenderProfile::Balanced => {
                self.assets.geometry_tier = ViewerGeometryTier::Balanced;
                self.assets.location_shell_enabled = true;
                self.materials.fragment.strategy = ViewerFragmentMaterialStrategy::Readability;
                self.materials.fragment.unlit = true;
                self.materials.fragment.alpha = DEFAULT_FRAGMENT_ALPHA;
                self.materials.fragment.emissive_boost = DEFAULT_FRAGMENT_EMISSIVE_BOOST;
                self.lighting.shadows_enabled = DEFAULT_SHADOWS_ENABLED;
                self.lighting.ambient_brightness = DEFAULT_AMBIENT_BRIGHTNESS;
                self.lighting.fill_light_ratio = DEFAULT_FILL_LIGHT_RATIO;
                self.lighting.rim_light_ratio = DEFAULT_RIM_LIGHT_RATIO;
                self.post_process = ViewerPostProcessConfig::default();
            }
            ViewerRenderProfile::Cinematic => {
                self.assets.geometry_tier = ViewerGeometryTier::Cinematic;
                self.assets.location_shell_enabled = true;
                self.materials.fragment.strategy = ViewerFragmentMaterialStrategy::Fidelity;
                self.materials.fragment.unlit = false;
                self.materials.fragment.alpha = 0.82;
                self.materials.fragment.emissive_boost = 0.12;
                self.lighting.shadows_enabled = true;
                self.lighting.ambient_brightness = 96.0;
                self.lighting.fill_light_ratio = 0.22;
                self.lighting.rim_light_ratio = 0.18;
                self.post_process.tonemapping = ViewerTonemappingMode::BlenderFilmic;
                self.post_process.deband_dither_enabled = true;
                self.post_process.bloom_enabled = true;
                self.post_process.bloom_intensity = 0.24;
                self.post_process.color_grading_exposure = 0.35;
                self.post_process.color_grading_post_saturation = 1.08;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ViewerRenderProfile {
    Debug,
    Balanced,
    Cinematic,
}

impl Default for ViewerRenderProfile {
    fn default() -> Self {
        Self::Balanced
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ViewerGeometryTier {
    Debug,
    Balanced,
    Cinematic,
}

impl Default for ViewerGeometryTier {
    fn default() -> Self {
        Self::Balanced
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ViewerAssetConfig {
    pub geometry_tier: ViewerGeometryTier,
    pub location_shell_enabled: bool,
}

impl Default for ViewerAssetConfig {
    fn default() -> Self {
        Self {
            geometry_tier: ViewerGeometryTier::Balanced,
            location_shell_enabled: DEFAULT_LOCATION_SHELL_ENABLED,
        }
    }
}

#[derive(Clone, Debug, Default, Resource)]
pub(super) struct ViewerExternalMeshConfig {
    pub agent_mesh_asset: Option<String>,
    pub location_mesh_asset: Option<String>,
    pub asset_mesh_asset: Option<String>,
    pub power_plant_mesh_asset: Option<String>,
    pub power_storage_mesh_asset: Option<String>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ViewerExternalMaterialSlotConfig {
    pub base_color_srgb: Option<[f32; 3]>,
    pub emissive_color_srgb: Option<[f32; 3]>,
}

#[derive(Clone, Copy, Debug, Default, Resource)]
pub(super) struct ViewerExternalMaterialConfig {
    pub agent: ViewerExternalMaterialSlotConfig,
    pub location: ViewerExternalMaterialSlotConfig,
    pub asset: ViewerExternalMaterialSlotConfig,
    pub power_plant: ViewerExternalMaterialSlotConfig,
    pub power_storage: ViewerExternalMaterialSlotConfig,
}

#[derive(Clone, Debug, Default)]
pub(super) struct ViewerExternalTextureSlotConfig {
    pub base_texture_asset: Option<String>,
    pub normal_texture_asset: Option<String>,
    pub metallic_roughness_texture_asset: Option<String>,
    pub emissive_texture_asset: Option<String>,
}

#[derive(Clone, Debug, Default, Resource)]
pub(super) struct ViewerExternalTextureConfig {
    pub agent: ViewerExternalTextureSlotConfig,
    pub location: ViewerExternalTextureSlotConfig,
    pub asset: ViewerExternalTextureSlotConfig,
    pub power_plant: ViewerExternalTextureSlotConfig,
    pub power_storage: ViewerExternalTextureSlotConfig,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ViewerFragmentMaterialStrategy {
    Readability,
    Fidelity,
}

impl Default for ViewerFragmentMaterialStrategy {
    fn default() -> Self {
        Self::Readability
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ViewerTonemappingMode {
    None,
    Reinhard,
    ReinhardLuminance,
    AcesFitted,
    AgX,
    SomewhatBoringDisplayTransform,
    TonyMcMapface,
    BlenderFilmic,
}

impl Default for ViewerTonemappingMode {
    fn default() -> Self {
        Self::TonyMcMapface
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ViewerPostProcessConfig {
    pub tonemapping: ViewerTonemappingMode,
    pub deband_dither_enabled: bool,
    pub bloom_enabled: bool,
    pub bloom_intensity: f32,
    pub color_grading_exposure: f32,
    pub color_grading_post_saturation: f32,
}

impl Default for ViewerPostProcessConfig {
    fn default() -> Self {
        Self {
            tonemapping: ViewerTonemappingMode::default(),
            deband_dither_enabled: DEFAULT_DEBAND_DITHER_ENABLED,
            bloom_enabled: DEFAULT_BLOOM_ENABLED,
            bloom_intensity: DEFAULT_BLOOM_INTENSITY,
            color_grading_exposure: DEFAULT_COLOR_GRADING_EXPOSURE,
            color_grading_post_saturation: DEFAULT_COLOR_GRADING_POST_SATURATION,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ViewerPbrMaterialConfig {
    pub roughness: f32,
    pub metallic: f32,
    pub emissive_boost: f32,
}

impl Default for ViewerPbrMaterialConfig {
    fn default() -> Self {
        Self {
            roughness: DEFAULT_MATERIAL_AGENT_ROUGHNESS,
            metallic: DEFAULT_MATERIAL_AGENT_METALLIC,
            emissive_boost: DEFAULT_MATERIAL_AGENT_EMISSIVE_BOOST,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ViewerFragmentMaterialConfig {
    pub strategy: ViewerFragmentMaterialStrategy,
    pub unlit: bool,
    pub alpha: f32,
    pub emissive_boost: f32,
}

impl Default for ViewerFragmentMaterialConfig {
    fn default() -> Self {
        Self {
            strategy: ViewerFragmentMaterialStrategy::default(),
            unlit: DEFAULT_FRAGMENT_UNLIT,
            alpha: DEFAULT_FRAGMENT_ALPHA,
            emissive_boost: DEFAULT_FRAGMENT_EMISSIVE_BOOST,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ViewerMaterialConfig {
    pub agent: ViewerPbrMaterialConfig,
    pub asset: ViewerPbrMaterialConfig,
    pub facility: ViewerPbrMaterialConfig,
    pub fragment: ViewerFragmentMaterialConfig,
}

impl Default for ViewerMaterialConfig {
    fn default() -> Self {
        Self {
            agent: ViewerPbrMaterialConfig {
                roughness: DEFAULT_MATERIAL_AGENT_ROUGHNESS,
                metallic: DEFAULT_MATERIAL_AGENT_METALLIC,
                emissive_boost: DEFAULT_MATERIAL_AGENT_EMISSIVE_BOOST,
            },
            asset: ViewerPbrMaterialConfig {
                roughness: DEFAULT_MATERIAL_ASSET_ROUGHNESS,
                metallic: DEFAULT_MATERIAL_ASSET_METALLIC,
                emissive_boost: DEFAULT_MATERIAL_ASSET_EMISSIVE_BOOST,
            },
            facility: ViewerPbrMaterialConfig {
                roughness: DEFAULT_MATERIAL_FACILITY_ROUGHNESS,
                metallic: DEFAULT_MATERIAL_FACILITY_METALLIC,
                emissive_boost: DEFAULT_MATERIAL_FACILITY_EMISSIVE_BOOST,
            },
            fragment: ViewerFragmentMaterialConfig::default(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ViewerLabelLodConfig {
    pub fade_start_distance: f32,
    pub fade_end_distance: f32,
    pub max_visible_labels: usize,
    pub occlusion_cell_span: f32,
    pub occlusion_cap_per_cell: usize,
}

impl Default for ViewerLabelLodConfig {
    fn default() -> Self {
        Self {
            fade_start_distance: DEFAULT_LABEL_FADE_START_DISTANCE,
            fade_end_distance: DEFAULT_LABEL_FADE_END_DISTANCE,
            max_visible_labels: DEFAULT_MAX_VISIBLE_LABELS,
            occlusion_cell_span: DEFAULT_LABEL_OCCLUSION_CELL_SPAN,
            occlusion_cap_per_cell: DEFAULT_LABEL_OCCLUSION_CAP_PER_CELL,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ViewerRenderBudgetConfig {
    pub overlay_refresh_ticks: u64,
    pub overlay_max_heat_markers: usize,
    pub overlay_max_flow_segments: usize,
    pub grid_lod_distance: f32,
}

impl Default for ViewerRenderBudgetConfig {
    fn default() -> Self {
        Self {
            overlay_refresh_ticks: DEFAULT_OVERLAY_REFRESH_TICKS,
            overlay_max_heat_markers: DEFAULT_OVERLAY_MAX_HEAT_MARKERS,
            overlay_max_flow_segments: DEFAULT_OVERLAY_MAX_FLOW_SEGMENTS,
            grid_lod_distance: DEFAULT_GRID_LOD_DISTANCE,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ViewerLightingConfig {
    pub shadows_enabled: bool,
    pub ambient_brightness: f32,
    pub fill_light_ratio: f32,
    pub rim_light_ratio: f32,
}

impl Default for ViewerLightingConfig {
    fn default() -> Self {
        Self {
            shadows_enabled: DEFAULT_SHADOWS_ENABLED,
            ambient_brightness: DEFAULT_AMBIENT_BRIGHTNESS,
            fill_light_ratio: DEFAULT_FILL_LIGHT_RATIO,
            rim_light_ratio: DEFAULT_RIM_LIGHT_RATIO,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ViewerPhysicalRenderConfig {
    pub enabled: bool,
    pub meters_per_unit: f32,
    pub floating_origin_step_m: f64,
    pub camera_near_m: f32,
    pub camera_far_m: f32,
    pub stellar_distance_au: f32,
    pub luminous_efficacy_lm_per_w: f32,
    pub exposure_ev100: f32,
    pub reference_radiation_area_m2: f32,
}

impl ViewerPhysicalRenderConfig {
    pub(super) fn irradiance_w_m2(&self) -> f32 {
        let distance = self.stellar_distance_au.max(0.1);
        SOLAR_CONSTANT_W_M2_AT_1_AU / (distance * distance)
    }

    pub(super) fn directional_illuminance_lux(&self) -> f32 {
        self.irradiance_w_m2() * self.luminous_efficacy_lm_per_w
    }

    pub(super) fn exposure_scale(&self) -> f32 {
        2.0_f32.powf((self.exposure_ev100 - BASELINE_EXPOSURE_EV100).clamp(-4.0, 4.0))
    }

    pub(super) fn exposed_illuminance_lux(&self) -> f32 {
        (self.directional_illuminance_lux() / self.exposure_scale())
            .clamp(MIN_LIGHT_ILLUMINANCE_LUX, MAX_LIGHT_ILLUMINANCE_LUX)
    }
}

impl Default for ViewerPhysicalRenderConfig {
    fn default() -> Self {
        Self {
            enabled: DEFAULT_PHYSICAL_ENABLED,
            meters_per_unit: DEFAULT_METERS_PER_UNIT,
            floating_origin_step_m: DEFAULT_FLOATING_ORIGIN_STEP_M,
            camera_near_m: DEFAULT_CAMERA_NEAR_M,
            camera_far_m: DEFAULT_CAMERA_FAR_M,
            stellar_distance_au: DEFAULT_STELLAR_DISTANCE_AU,
            luminous_efficacy_lm_per_w: DEFAULT_LUMINOUS_EFFICACY_LM_PER_W,
            exposure_ev100: DEFAULT_EXPOSURE_EV100,
            reference_radiation_area_m2: DEFAULT_REFERENCE_RADIATION_AREA_M2,
        }
    }
}

pub(super) fn resolve_viewer_3d_config() -> Viewer3dConfig {
    load_viewer_3d_config_from(|key| std::env::var(key).ok())
}

pub(super) fn resolve_viewer_external_mesh_config() -> ViewerExternalMeshConfig {
    load_viewer_external_mesh_config_from(|key| std::env::var(key).ok())
}

pub(super) fn resolve_viewer_external_material_config() -> ViewerExternalMaterialConfig {
    load_viewer_external_material_config_from(|key| std::env::var(key).ok())
}

pub(super) fn resolve_viewer_external_texture_config() -> ViewerExternalTextureConfig {
    load_viewer_external_texture_config_from(|key| std::env::var(key).ok())
}

fn load_viewer_external_mesh_config_from<F>(lookup: F) -> ViewerExternalMeshConfig
where
    F: Fn(&str) -> Option<String>,
{
    ViewerExternalMeshConfig {
        agent_mesh_asset: parse_non_empty_string(&lookup, "AGENT_WORLD_VIEWER_AGENT_MESH_ASSET"),
        location_mesh_asset: parse_non_empty_string(
            &lookup,
            "AGENT_WORLD_VIEWER_LOCATION_MESH_ASSET",
        ),
        asset_mesh_asset: parse_non_empty_string(&lookup, "AGENT_WORLD_VIEWER_ASSET_MESH_ASSET"),
        power_plant_mesh_asset: parse_non_empty_string(
            &lookup,
            "AGENT_WORLD_VIEWER_POWER_PLANT_MESH_ASSET",
        ),
        power_storage_mesh_asset: parse_non_empty_string(
            &lookup,
            "AGENT_WORLD_VIEWER_POWER_STORAGE_MESH_ASSET",
        ),
    }
}

fn load_viewer_external_material_config_from<F>(lookup: F) -> ViewerExternalMaterialConfig
where
    F: Fn(&str) -> Option<String>,
{
    ViewerExternalMaterialConfig {
        agent: ViewerExternalMaterialSlotConfig {
            base_color_srgb: parse_hex_srgb_color(&lookup, "AGENT_WORLD_VIEWER_AGENT_BASE_COLOR"),
            emissive_color_srgb: parse_hex_srgb_color(
                &lookup,
                "AGENT_WORLD_VIEWER_AGENT_EMISSIVE_COLOR",
            ),
        },
        location: ViewerExternalMaterialSlotConfig {
            base_color_srgb: parse_hex_srgb_color(
                &lookup,
                "AGENT_WORLD_VIEWER_LOCATION_BASE_COLOR",
            ),
            emissive_color_srgb: parse_hex_srgb_color(
                &lookup,
                "AGENT_WORLD_VIEWER_LOCATION_EMISSIVE_COLOR",
            ),
        },
        asset: ViewerExternalMaterialSlotConfig {
            base_color_srgb: parse_hex_srgb_color(&lookup, "AGENT_WORLD_VIEWER_ASSET_BASE_COLOR"),
            emissive_color_srgb: parse_hex_srgb_color(
                &lookup,
                "AGENT_WORLD_VIEWER_ASSET_EMISSIVE_COLOR",
            ),
        },
        power_plant: ViewerExternalMaterialSlotConfig {
            base_color_srgb: parse_hex_srgb_color(
                &lookup,
                "AGENT_WORLD_VIEWER_POWER_PLANT_BASE_COLOR",
            ),
            emissive_color_srgb: parse_hex_srgb_color(
                &lookup,
                "AGENT_WORLD_VIEWER_POWER_PLANT_EMISSIVE_COLOR",
            ),
        },
        power_storage: ViewerExternalMaterialSlotConfig {
            base_color_srgb: parse_hex_srgb_color(
                &lookup,
                "AGENT_WORLD_VIEWER_POWER_STORAGE_BASE_COLOR",
            ),
            emissive_color_srgb: parse_hex_srgb_color(
                &lookup,
                "AGENT_WORLD_VIEWER_POWER_STORAGE_EMISSIVE_COLOR",
            ),
        },
    }
}

fn load_viewer_external_texture_config_from<F>(lookup: F) -> ViewerExternalTextureConfig
where
    F: Fn(&str) -> Option<String>,
{
    ViewerExternalTextureConfig {
        agent: ViewerExternalTextureSlotConfig {
            base_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_AGENT_BASE_TEXTURE_ASSET",
            ),
            normal_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_AGENT_NORMAL_TEXTURE_ASSET",
            ),
            metallic_roughness_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_AGENT_METALLIC_ROUGHNESS_TEXTURE_ASSET",
            ),
            emissive_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_AGENT_EMISSIVE_TEXTURE_ASSET",
            ),
        },
        location: ViewerExternalTextureSlotConfig {
            base_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_LOCATION_BASE_TEXTURE_ASSET",
            ),
            normal_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_LOCATION_NORMAL_TEXTURE_ASSET",
            ),
            metallic_roughness_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_LOCATION_METALLIC_ROUGHNESS_TEXTURE_ASSET",
            ),
            emissive_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_LOCATION_EMISSIVE_TEXTURE_ASSET",
            ),
        },
        asset: ViewerExternalTextureSlotConfig {
            base_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_ASSET_BASE_TEXTURE_ASSET",
            ),
            normal_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_ASSET_NORMAL_TEXTURE_ASSET",
            ),
            metallic_roughness_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_ASSET_METALLIC_ROUGHNESS_TEXTURE_ASSET",
            ),
            emissive_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_ASSET_EMISSIVE_TEXTURE_ASSET",
            ),
        },
        power_plant: ViewerExternalTextureSlotConfig {
            base_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_POWER_PLANT_BASE_TEXTURE_ASSET",
            ),
            normal_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_POWER_PLANT_NORMAL_TEXTURE_ASSET",
            ),
            metallic_roughness_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_POWER_PLANT_METALLIC_ROUGHNESS_TEXTURE_ASSET",
            ),
            emissive_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_POWER_PLANT_EMISSIVE_TEXTURE_ASSET",
            ),
        },
        power_storage: ViewerExternalTextureSlotConfig {
            base_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_POWER_STORAGE_BASE_TEXTURE_ASSET",
            ),
            normal_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_POWER_STORAGE_NORMAL_TEXTURE_ASSET",
            ),
            metallic_roughness_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_POWER_STORAGE_METALLIC_ROUGHNESS_TEXTURE_ASSET",
            ),
            emissive_texture_asset: parse_non_empty_string(
                &lookup,
                "AGENT_WORLD_VIEWER_POWER_STORAGE_EMISSIVE_TEXTURE_ASSET",
            ),
        },
    }
}

fn load_viewer_3d_config_from<F>(lookup: F) -> Viewer3dConfig
where
    F: Fn(&str) -> Option<String>,
{
    let mut config = Viewer3dConfig::default();
    if let Some(profile) = parse_render_profile(&lookup, "AGENT_WORLD_VIEWER_RENDER_PROFILE") {
        config.apply_render_profile(profile);
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_CM_TO_UNIT") {
        if value.is_finite() && value > 0.0 {
            config.cm_to_unit = value;
        }
    }
    if let Some(value) = parse_bool(&lookup, "AGENT_WORLD_VIEWER_SHOW_AGENTS") {
        config.show_agents = value;
    }
    if let Some(value) = parse_bool(&lookup, "AGENT_WORLD_VIEWER_SHOW_LOCATIONS") {
        config.show_locations = value;
    }
    if let Some(value) = parse_bool(&lookup, "AGENT_WORLD_VIEWER_HIGHLIGHT_SELECTED") {
        config.highlight_selected = value;
    }
    if let Some(value) = parse_geometry_tier(&lookup, "AGENT_WORLD_VIEWER_ASSET_GEOMETRY_TIER") {
        config.assets.geometry_tier = value;
    }
    if let Some(value) = parse_bool(&lookup, "AGENT_WORLD_VIEWER_LOCATION_SHELL_ENABLED") {
        config.assets.location_shell_enabled = value;
    }
    if let Some(value) =
        parse_fragment_material_strategy(&lookup, "AGENT_WORLD_VIEWER_FRAGMENT_MATERIAL_STRATEGY")
    {
        config.materials.fragment.strategy = value;
    }
    if let Some(value) = parse_bool(&lookup, "AGENT_WORLD_VIEWER_FRAGMENT_UNLIT") {
        config.materials.fragment.unlit = value;
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_FRAGMENT_ALPHA") {
        if value.is_finite() && (MIN_FRAGMENT_ALPHA..=MAX_FRAGMENT_ALPHA).contains(&value) {
            config.materials.fragment.alpha = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_FRAGMENT_EMISSIVE_BOOST") {
        if value.is_finite() && value >= 0.0 {
            config.materials.fragment.emissive_boost = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_MATERIAL_AGENT_ROUGHNESS") {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            config.materials.agent.roughness = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_MATERIAL_AGENT_METALLIC") {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            config.materials.agent.metallic = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_MATERIAL_AGENT_EMISSIVE_BOOST") {
        if value.is_finite() && value >= 0.0 {
            config.materials.agent.emissive_boost = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_MATERIAL_ASSET_ROUGHNESS") {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            config.materials.asset.roughness = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_MATERIAL_ASSET_METALLIC") {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            config.materials.asset.metallic = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_MATERIAL_ASSET_EMISSIVE_BOOST") {
        if value.is_finite() && value >= 0.0 {
            config.materials.asset.emissive_boost = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_MATERIAL_FACILITY_ROUGHNESS") {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            config.materials.facility.roughness = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_MATERIAL_FACILITY_METALLIC") {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            config.materials.facility.metallic = value;
        }
    }
    if let Some(value) = parse_f32(
        &lookup,
        "AGENT_WORLD_VIEWER_MATERIAL_FACILITY_EMISSIVE_BOOST",
    ) {
        if value.is_finite() && value >= 0.0 {
            config.materials.facility.emissive_boost = value;
        }
    }
    if let Some(value) = parse_tonemapping_mode(&lookup, "AGENT_WORLD_VIEWER_TONEMAPPING") {
        config.post_process.tonemapping = value;
    }
    if let Some(value) = parse_bool(&lookup, "AGENT_WORLD_VIEWER_DEBAND_DITHER_ENABLED") {
        config.post_process.deband_dither_enabled = value;
    }
    if let Some(value) = parse_bool(&lookup, "AGENT_WORLD_VIEWER_BLOOM_ENABLED") {
        config.post_process.bloom_enabled = value;
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_BLOOM_INTENSITY") {
        if value.is_finite() && (MIN_BLOOM_INTENSITY..=MAX_BLOOM_INTENSITY).contains(&value) {
            config.post_process.bloom_intensity = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_COLOR_GRADING_EXPOSURE") {
        if value.is_finite()
            && (MIN_COLOR_GRADING_EXPOSURE..=MAX_COLOR_GRADING_EXPOSURE).contains(&value)
        {
            config.post_process.color_grading_exposure = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_COLOR_GRADING_POST_SATURATION") {
        if value.is_finite()
            && (MIN_COLOR_GRADING_POST_SATURATION..=MAX_COLOR_GRADING_POST_SATURATION)
                .contains(&value)
        {
            config.post_process.color_grading_post_saturation = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_LABEL_FADE_START") {
        if value.is_finite() && value >= 0.0 {
            config.label_lod.fade_start_distance = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_LABEL_FADE_END") {
        if value.is_finite() && value > config.label_lod.fade_start_distance {
            config.label_lod.fade_end_distance = value;
        }
    }
    if let Some(value) = parse_usize(&lookup, "AGENT_WORLD_VIEWER_MAX_VISIBLE_LABELS") {
        if value > 0 {
            config.label_lod.max_visible_labels = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_LABEL_OCCLUSION_CELL_SPAN") {
        if value.is_finite() && value > 0.0 {
            config.label_lod.occlusion_cell_span = value;
        }
    }
    if let Some(value) = parse_usize(&lookup, "AGENT_WORLD_VIEWER_LABEL_OCCLUSION_CAP_PER_CELL") {
        if value > 0 {
            config.label_lod.occlusion_cap_per_cell = value;
        }
    }
    if let Some(value) = parse_u64(&lookup, "AGENT_WORLD_VIEWER_OVERLAY_REFRESH_TICKS") {
        if value > 0 {
            config.render_budget.overlay_refresh_ticks = value;
        }
    }
    if let Some(value) = parse_usize(&lookup, "AGENT_WORLD_VIEWER_OVERLAY_MAX_HEAT") {
        if value > 0 {
            config.render_budget.overlay_max_heat_markers = value;
        }
    }
    if let Some(value) = parse_usize(&lookup, "AGENT_WORLD_VIEWER_OVERLAY_MAX_FLOW") {
        if value > 0 {
            config.render_budget.overlay_max_flow_segments = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_GRID_LOD_DISTANCE") {
        if value.is_finite() && value > 0.0 {
            config.render_budget.grid_lod_distance = value;
        }
    }
    if let Some(value) = parse_bool(&lookup, "AGENT_WORLD_VIEWER_SHADOWS_ENABLED") {
        config.lighting.shadows_enabled = value;
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_AMBIENT_BRIGHTNESS") {
        if value.is_finite() && value > 0.0 {
            config.lighting.ambient_brightness = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_FILL_LIGHT_RATIO") {
        if value.is_finite() && value >= 0.0 {
            config.lighting.fill_light_ratio = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_RIM_LIGHT_RATIO") {
        if value.is_finite() && value >= 0.0 {
            config.lighting.rim_light_ratio = value;
        }
    }

    let mut physical = ViewerPhysicalRenderConfig::default();
    if let Some(value) = parse_bool(&lookup, "AGENT_WORLD_VIEWER_PHYSICAL_RENDER_ENABLED") {
        physical.enabled = value;
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_METERS_PER_UNIT") {
        if value.is_finite() && value > 0.0 {
            physical.meters_per_unit = value;
        }
    }
    if let Some(value) = parse_f64(&lookup, "AGENT_WORLD_VIEWER_FLOATING_ORIGIN_STEP_M") {
        if value.is_finite() && value > 0.0 {
            physical.floating_origin_step_m = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_CAMERA_NEAR_M") {
        if value.is_finite() && value > 0.0 {
            physical.camera_near_m = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_CAMERA_FAR_M") {
        if value.is_finite() && value > 0.0 {
            physical.camera_far_m = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_STELLAR_DISTANCE_AU") {
        if value.is_finite() && value > 0.0 {
            physical.stellar_distance_au = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_LUMINOUS_EFFICACY_LM_PER_W") {
        if value.is_finite() && value > 0.0 {
            physical.luminous_efficacy_lm_per_w = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_EXPOSURE_EV100") {
        if value.is_finite() {
            physical.exposure_ev100 = value;
        }
    }
    if let Some(value) = parse_f32(&lookup, "AGENT_WORLD_VIEWER_REFERENCE_RADIATION_AREA_M2") {
        if value.is_finite() && value > 0.0 {
            physical.reference_radiation_area_m2 = value;
        }
    }

    if physical.camera_far_m <= physical.camera_near_m {
        physical.camera_far_m = (physical.camera_near_m + 1.0).max(DEFAULT_CAMERA_FAR_M);
    }
    if config.label_lod.fade_end_distance <= config.label_lod.fade_start_distance {
        config.label_lod.fade_end_distance = config.label_lod.fade_start_distance + 1.0;
    }

    config.physical = physical;
    config
}

fn parse_bool<F>(lookup: &F, key: &str) -> Option<bool>
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

fn parse_render_profile<F>(lookup: &F, key: &str) -> Option<ViewerRenderProfile>
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

fn parse_geometry_tier<F>(lookup: &F, key: &str) -> Option<ViewerGeometryTier>
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

fn parse_fragment_material_strategy<F>(
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

fn parse_tonemapping_mode<F>(lookup: &F, key: &str) -> Option<ViewerTonemappingMode>
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

fn parse_f32<F>(lookup: &F, key: &str) -> Option<f32>
where
    F: Fn(&str) -> Option<String>,
{
    lookup(key).and_then(|raw| raw.trim().parse::<f32>().ok())
}

fn parse_f64<F>(lookup: &F, key: &str) -> Option<f64>
where
    F: Fn(&str) -> Option<String>,
{
    lookup(key).and_then(|raw| raw.trim().parse::<f64>().ok())
}

fn parse_non_empty_string<F>(lookup: &F, key: &str) -> Option<String>
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

fn parse_hex_srgb_color<F>(lookup: &F, key: &str) -> Option<[f32; 3]>
where
    F: Fn(&str) -> Option<String>,
{
    let raw = lookup(key)?;
    let hex = raw.trim();
    let color_hex = hex.strip_prefix('#')?;
    if color_hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&color_hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&color_hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&color_hex[4..6], 16).ok()?;
    Some([r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0])
}

fn parse_usize<F>(lookup: &F, key: &str) -> Option<usize>
where
    F: Fn(&str) -> Option<String>,
{
    lookup(key).and_then(|raw| raw.trim().parse::<usize>().ok())
}

fn parse_u64<F>(lookup: &F, key: &str) -> Option<u64>
where
    F: Fn(&str) -> Option<String>,
{
    lookup(key).and_then(|raw| raw.trim().parse::<u64>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_rpa1_baseline() {
        let config = Viewer3dConfig::default();
        assert!((config.cm_to_unit - 0.00001).abs() < f32::EPSILON);
        assert_eq!(config.render_profile, ViewerRenderProfile::Balanced);
        assert!(config.show_agents);
        assert!(config.show_locations);
        assert!(config.highlight_selected);
        assert_eq!(config.assets.geometry_tier, ViewerGeometryTier::Balanced);
        assert!(config.assets.location_shell_enabled);
        assert!(
            (config.materials.agent.roughness - DEFAULT_MATERIAL_AGENT_ROUGHNESS).abs()
                < f32::EPSILON
        );
        assert!(
            (config.materials.agent.metallic - DEFAULT_MATERIAL_AGENT_METALLIC).abs()
                < f32::EPSILON
        );
        assert_eq!(
            config.materials.fragment.strategy,
            ViewerFragmentMaterialStrategy::Readability
        );
        assert!(config.materials.fragment.unlit);
        assert!((config.materials.fragment.alpha - DEFAULT_FRAGMENT_ALPHA).abs() < f32::EPSILON);
        assert_eq!(
            config.post_process.tonemapping,
            ViewerTonemappingMode::TonyMcMapface
        );
        assert_eq!(
            config.post_process.deband_dither_enabled,
            DEFAULT_DEBAND_DITHER_ENABLED
        );
        assert_eq!(config.post_process.bloom_enabled, DEFAULT_BLOOM_ENABLED);
        assert!(
            (config.post_process.bloom_intensity - DEFAULT_BLOOM_INTENSITY).abs() < f32::EPSILON
        );
        assert!(
            (config.post_process.color_grading_exposure - DEFAULT_COLOR_GRADING_EXPOSURE).abs()
                < f32::EPSILON
        );
        assert!(
            (config.post_process.color_grading_post_saturation
                - DEFAULT_COLOR_GRADING_POST_SATURATION)
                .abs()
                < f32::EPSILON
        );
        assert!(
            (config.label_lod.fade_start_distance - DEFAULT_LABEL_FADE_START_DISTANCE).abs()
                < f32::EPSILON
        );
        assert!(
            (config.label_lod.fade_end_distance - DEFAULT_LABEL_FADE_END_DISTANCE).abs()
                < f32::EPSILON
        );
        assert_eq!(
            config.label_lod.max_visible_labels,
            DEFAULT_MAX_VISIBLE_LABELS
        );
        assert!(
            (config.label_lod.occlusion_cell_span - DEFAULT_LABEL_OCCLUSION_CELL_SPAN).abs()
                < f32::EPSILON
        );
        assert_eq!(
            config.label_lod.occlusion_cap_per_cell,
            DEFAULT_LABEL_OCCLUSION_CAP_PER_CELL
        );
        assert_eq!(
            config.render_budget.overlay_refresh_ticks,
            DEFAULT_OVERLAY_REFRESH_TICKS
        );
        assert_eq!(
            config.render_budget.overlay_max_heat_markers,
            DEFAULT_OVERLAY_MAX_HEAT_MARKERS
        );
        assert_eq!(
            config.render_budget.overlay_max_flow_segments,
            DEFAULT_OVERLAY_MAX_FLOW_SEGMENTS
        );
        assert!(
            (config.render_budget.grid_lod_distance - DEFAULT_GRID_LOD_DISTANCE).abs()
                < f32::EPSILON
        );
        assert!(!config.lighting.shadows_enabled);
        assert!((config.lighting.ambient_brightness - 110.0).abs() < f32::EPSILON);
        assert!((config.lighting.fill_light_ratio - 0.28).abs() < f32::EPSILON);
        assert!((config.lighting.rim_light_ratio - DEFAULT_RIM_LIGHT_RATIO).abs() < f32::EPSILON);
        assert!(!config.physical.enabled);
        assert!((config.physical.meters_per_unit - 1.0).abs() < f32::EPSILON);
        assert!((config.physical.floating_origin_step_m - 1000.0).abs() < f64::EPSILON);
        assert!((config.physical.camera_near_m - 0.1).abs() < f32::EPSILON);
        assert!((config.physical.camera_far_m - 25_000.0).abs() < f32::EPSILON);
        assert!((config.physical.stellar_distance_au - 2.5).abs() < f32::EPSILON);
        assert!((config.physical.luminous_efficacy_lm_per_w - 120.0).abs() < f32::EPSILON);
        assert!((config.physical.exposure_ev100 - 13.5).abs() < f32::EPSILON);
        assert!((config.physical.reference_radiation_area_m2 - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn effective_scale_and_irradiance_follow_physical_settings() {
        let mut config = Viewer3dConfig::default();
        assert!((config.effective_cm_to_unit() - DEFAULT_CM_TO_UNIT).abs() < f32::EPSILON);

        config.physical.enabled = true;
        config.physical.meters_per_unit = 1.0;
        config.physical.stellar_distance_au = 2.5;

        assert!((config.effective_cm_to_unit() - 0.01).abs() < f32::EPSILON);

        let irradiance = config.physical.irradiance_w_m2();
        assert!((irradiance - 217.76).abs() < 0.5);

        let directional = config.physical.directional_illuminance_lux();
        assert!((directional - 26_131.2).abs() < 80.0);
    }

    #[test]
    fn asteroid_belt_irradiance_is_monotonic_in_2_2_to_3_2_au() {
        let mut config = Viewer3dConfig::default();
        config.physical.enabled = true;

        config.physical.stellar_distance_au = 2.2;
        let near = config.physical.irradiance_w_m2();
        config.physical.stellar_distance_au = 2.5;
        let middle = config.physical.irradiance_w_m2();
        config.physical.stellar_distance_au = 3.2;
        let far = config.physical.irradiance_w_m2();

        assert!(near > middle);
        assert!(middle > far);
    }

    #[test]
    fn exposure_ev100_controls_exposed_illuminance_lux() {
        let mut config = Viewer3dConfig::default();
        config.physical.enabled = true;
        config.physical.stellar_distance_au = 2.5;
        config.physical.luminous_efficacy_lm_per_w = 120.0;

        config.physical.exposure_ev100 = 13.5;
        let baseline = config.physical.exposed_illuminance_lux();
        config.physical.exposure_ev100 = 14.5;
        let darker = config.physical.exposed_illuminance_lux();
        config.physical.exposure_ev100 = 12.5;
        let brighter = config.physical.exposed_illuminance_lux();

        assert!(brighter > baseline);
        assert!(baseline > darker);
        assert!((baseline - 26_131.2).abs() < 80.0);
    }

    #[test]
    fn exposed_illuminance_respects_clamp_range() {
        let mut config = Viewer3dConfig::default();
        config.physical.enabled = true;
        config.physical.stellar_distance_au = 0.1;
        config.physical.exposure_ev100 = 9.5;
        let high = config.physical.exposed_illuminance_lux();
        assert!((high - MAX_LIGHT_ILLUMINANCE_LUX).abs() < f32::EPSILON);

        config.physical.stellar_distance_au = 15.0;
        config.physical.exposure_ev100 = 17.5;
        let low = config.physical.exposed_illuminance_lux();
        assert!((low - MIN_LIGHT_ILLUMINANCE_LUX).abs() < f32::EPSILON);
    }
}

#[cfg(test)]
#[path = "viewer_3d_config_profile_tests.rs"]
mod viewer_3d_config_profile_tests;
