use super::*;
use std::collections::HashMap;

fn assert_rgb_approx_eq(actual: Option<[f32; 3]>, expected: [f32; 3]) {
    let actual = actual.expect("expected rgb color to be present");
    assert!((actual[0] - expected[0]).abs() < 1e-6);
    assert!((actual[1] - expected[1]).abs() < 1e-6);
    assert!((actual[2] - expected[2]).abs() < 1e-6);
}

#[test]
fn load_viewer_3d_config_applies_env_overrides() {
    let env = HashMap::from([
        ("OASIS7_VIEWER_CM_TO_UNIT", "0.0002"),
        ("OASIS7_VIEWER_RENDER_PROFILE", "debug"),
        ("OASIS7_VIEWER_SHOW_AGENTS", "false"),
        ("OASIS7_VIEWER_SHOW_LOCATIONS", "0"),
        ("OASIS7_VIEWER_HIGHLIGHT_SELECTED", "no"),
        ("OASIS7_VIEWER_VISUAL_EFFECTS", "enhanced"),
        (
            "OASIS7_VIEWER_AGENT_VARIANT_PALETTE",
            "#112233,#445566,#778899,#AABBCC",
        ),
        ("OASIS7_VIEWER_AGENT_DIRECTION_INDICATOR", "0"),
        ("OASIS7_VIEWER_AGENT_SPEED_EFFECT", "1"),
        ("OASIS7_VIEWER_AGENT_TRAIL_ENABLED", "1"),
        ("OASIS7_VIEWER_LOCATION_RADIATION_GLOW", "0"),
        ("OASIS7_VIEWER_LOCATION_DAMAGE_VISUAL", "1"),
        ("OASIS7_VIEWER_ASSET_QUANTITY_VISUAL", "1"),
        ("OASIS7_VIEWER_ASSET_TYPE_COLOR", "0"),
        (
            "OASIS7_VIEWER_AGENT_MESH_ASSET",
            "models/agents/worker.glb#Mesh0/Primitive0",
        ),
        (
            "OASIS7_VIEWER_LOCATION_MESH_ASSET",
            "models/world/location.glb#Mesh0/Primitive0",
        ),
        (
            "OASIS7_VIEWER_ASSET_MESH_ASSET",
            "models/world/asset.glb#Mesh0/Primitive0",
        ),
        (
            "OASIS7_VIEWER_POWER_PLANT_MESH_ASSET",
            "models/facility/power_plant.glb#Mesh0/Primitive0",
        ),
        (
            "OASIS7_VIEWER_AGENT_BASE_TEXTURE_ASSET",
            "textures/agents/worker_albedo.png",
        ),
        (
            "OASIS7_VIEWER_AGENT_NORMAL_TEXTURE_ASSET",
            "textures/agents/worker_normal.png",
        ),
        (
            "OASIS7_VIEWER_AGENT_METALLIC_ROUGHNESS_TEXTURE_ASSET",
            "textures/agents/worker_metal_rough.png",
        ),
        (
            "OASIS7_VIEWER_AGENT_EMISSIVE_TEXTURE_ASSET",
            "textures/agents/worker_emissive.png",
        ),
        (
            "OASIS7_VIEWER_LOCATION_BASE_TEXTURE_ASSET",
            "textures/world/location_albedo.png",
        ),
        (
            "OASIS7_VIEWER_LOCATION_NORMAL_TEXTURE_ASSET",
            "textures/world/location_normal.png",
        ),
        (
            "OASIS7_VIEWER_LOCATION_METALLIC_ROUGHNESS_TEXTURE_ASSET",
            "textures/world/location_metal_rough.png",
        ),
        (
            "OASIS7_VIEWER_LOCATION_EMISSIVE_TEXTURE_ASSET",
            "textures/world/location_emissive.png",
        ),
        (
            "OASIS7_VIEWER_ASSET_BASE_TEXTURE_ASSET",
            "textures/world/asset_albedo.png",
        ),
        (
            "OASIS7_VIEWER_ASSET_NORMAL_TEXTURE_ASSET",
            "textures/world/asset_normal.png",
        ),
        (
            "OASIS7_VIEWER_ASSET_METALLIC_ROUGHNESS_TEXTURE_ASSET",
            "textures/world/asset_metal_rough.png",
        ),
        (
            "OASIS7_VIEWER_ASSET_EMISSIVE_TEXTURE_ASSET",
            "textures/world/asset_emissive.png",
        ),
        (
            "OASIS7_VIEWER_POWER_PLANT_BASE_TEXTURE_ASSET",
            "textures/facility/power_plant_albedo.png",
        ),
        (
            "OASIS7_VIEWER_POWER_PLANT_NORMAL_TEXTURE_ASSET",
            "textures/facility/power_plant_normal.png",
        ),
        (
            "OASIS7_VIEWER_POWER_PLANT_METALLIC_ROUGHNESS_TEXTURE_ASSET",
            "textures/facility/power_plant_metal_rough.png",
        ),
        (
            "OASIS7_VIEWER_POWER_PLANT_EMISSIVE_TEXTURE_ASSET",
            "textures/facility/power_plant_emissive.png",
        ),
        ("OASIS7_VIEWER_AGENT_BASE_COLOR", "#FF6A38"),
        ("OASIS7_VIEWER_AGENT_EMISSIVE_COLOR", "#E66230"),
        ("OASIS7_VIEWER_LOCATION_BASE_COLOR", "#4B88D9"),
        ("OASIS7_VIEWER_LOCATION_EMISSIVE_COLOR", "#B8D8FF"),
        ("OASIS7_VIEWER_ASSET_BASE_COLOR", "#D1C35A"),
        ("OASIS7_VIEWER_ASSET_EMISSIVE_COLOR", "#FFD45A"),
        ("OASIS7_VIEWER_POWER_PLANT_BASE_COLOR", "#F36934"),
        ("OASIS7_VIEWER_POWER_PLANT_EMISSIVE_COLOR", "#FF7F4A"),
        ("OASIS7_VIEWER_ASSET_GEOMETRY_TIER", "cinematic"),
        ("OASIS7_VIEWER_LOCATION_SHELL_ENABLED", "true"),
        ("OASIS7_VIEWER_FRAGMENT_MATERIAL_STRATEGY", "fidelity"),
        ("OASIS7_VIEWER_FRAGMENT_UNLIT", "false"),
        ("OASIS7_VIEWER_FRAGMENT_ALPHA", "0.78"),
        ("OASIS7_VIEWER_FRAGMENT_EMISSIVE_BOOST", "0.40"),
        ("OASIS7_VIEWER_MATERIAL_AGENT_ROUGHNESS", "0.44"),
        ("OASIS7_VIEWER_MATERIAL_AGENT_METALLIC", "0.22"),
        ("OASIS7_VIEWER_MATERIAL_ASSET_ROUGHNESS", "0.61"),
        ("OASIS7_VIEWER_MATERIAL_ASSET_METALLIC", "0.33"),
        ("OASIS7_VIEWER_MATERIAL_FACILITY_ROUGHNESS", "0.53"),
        ("OASIS7_VIEWER_MATERIAL_FACILITY_METALLIC", "0.47"),
        ("OASIS7_VIEWER_MATERIAL_POWER_PLANT_ROUGHNESS", "0.29"),
        ("OASIS7_VIEWER_MATERIAL_POWER_PLANT_METALLIC", "0.74"),
        ("OASIS7_VIEWER_MATERIAL_POWER_PLANT_EMISSIVE_BOOST", "0.12"),
        ("OASIS7_VIEWER_TONEMAPPING", "aces"),
        ("OASIS7_VIEWER_DEBAND_DITHER_ENABLED", "true"),
        ("OASIS7_VIEWER_BLOOM_ENABLED", "false"),
        ("OASIS7_VIEWER_BLOOM_INTENSITY", "0.42"),
        ("OASIS7_VIEWER_COLOR_GRADING_EXPOSURE", "-0.65"),
        ("OASIS7_VIEWER_COLOR_GRADING_POST_SATURATION", "1.24"),
        ("OASIS7_VIEWER_LABEL_FADE_START", "44"),
        ("OASIS7_VIEWER_LABEL_FADE_END", "110"),
        ("OASIS7_VIEWER_MAX_VISIBLE_LABELS", "32"),
        ("OASIS7_VIEWER_LABEL_OCCLUSION_CELL_SPAN", "9"),
        ("OASIS7_VIEWER_LABEL_OCCLUSION_CAP_PER_CELL", "3"),
        ("OASIS7_VIEWER_OVERLAY_REFRESH_TICKS", "5"),
        ("OASIS7_VIEWER_OVERLAY_MAX_HEAT", "72"),
        ("OASIS7_VIEWER_OVERLAY_MAX_FLOW", "96"),
        ("OASIS7_VIEWER_GRID_LOD_DISTANCE", "180"),
        ("OASIS7_VIEWER_SHADOWS_ENABLED", "1"),
        ("OASIS7_VIEWER_AMBIENT_BRIGHTNESS", "145"),
        ("OASIS7_VIEWER_FILL_LIGHT_RATIO", "0.42"),
        ("OASIS7_VIEWER_RIM_LIGHT_RATIO", "0.20"),
        ("OASIS7_VIEWER_PHYSICAL_RENDER_ENABLED", "true"),
        ("OASIS7_VIEWER_METERS_PER_UNIT", "2.5"),
        ("OASIS7_VIEWER_FLOATING_ORIGIN_STEP_M", "1500"),
        ("OASIS7_VIEWER_CAMERA_NEAR_M", "0.2"),
        ("OASIS7_VIEWER_CAMERA_FAR_M", "60000"),
        ("OASIS7_VIEWER_STELLAR_DISTANCE_AU", "2.8"),
        ("OASIS7_VIEWER_LUMINOUS_EFFICACY_LM_PER_W", "130"),
        ("OASIS7_VIEWER_EXPOSURE_EV100", "12.8"),
        ("OASIS7_VIEWER_REFERENCE_RADIATION_AREA_M2", "1.2"),
    ]);

    let config = load_viewer_3d_config_from(|key| env.get(key).map(|v| v.to_string()));

    assert!((config.cm_to_unit - 0.0002).abs() < f32::EPSILON);
    assert_eq!(config.render_profile, ViewerRenderProfile::Debug);
    assert!(!config.show_agents);
    assert!(!config.show_locations);
    assert!(!config.highlight_selected);
    assert_eq!(config.visual.level, ViewerVisualEffectsLevel::Enhanced);
    let palette = config
        .visual
        .agent_variant_palette
        .expect("expected variant palette");
    assert_rgb_approx_eq(Some(palette[0]), [0.06666667, 0.13333334, 0.2]);
    assert_rgb_approx_eq(Some(palette[1]), [0.26666668, 0.33333334, 0.4]);
    assert_rgb_approx_eq(Some(palette[2]), [0.46666667, 0.53333336, 0.6]);
    assert_rgb_approx_eq(Some(palette[3]), [0.6666667, 0.73333335, 0.8]);
    assert!(!config.visual.agent_direction_indicator);
    assert!(config.visual.agent_speed_effect);
    assert!(config.visual.agent_trail_enabled);
    assert!(!config.visual.location_radiation_glow);
    assert!(config.visual.location_damage_visual);
    assert!(config.visual.asset_quantity_visual);
    assert!(!config.visual.asset_type_color);
    assert_eq!(config.assets.geometry_tier, ViewerGeometryTier::Cinematic);
    assert!(config.assets.location_shell_enabled);
    assert_eq!(
        config.materials.fragment.strategy,
        ViewerFragmentMaterialStrategy::Fidelity
    );
    assert!(!config.materials.fragment.unlit);
    assert!((config.materials.fragment.alpha - 0.78).abs() < f32::EPSILON);
    assert!((config.materials.fragment.emissive_boost - 0.40).abs() < f32::EPSILON);
    assert!((config.materials.agent.roughness - 0.44).abs() < f32::EPSILON);
    assert!((config.materials.agent.metallic - 0.22).abs() < f32::EPSILON);
    assert!((config.materials.asset.roughness - 0.61).abs() < f32::EPSILON);
    assert!((config.materials.asset.metallic - 0.33).abs() < f32::EPSILON);
    assert!((config.materials.facility.roughness - 0.53).abs() < f32::EPSILON);
    assert!((config.materials.facility.metallic - 0.47).abs() < f32::EPSILON);
    assert!((config.materials.power_plant.roughness - 0.29).abs() < f32::EPSILON);
    assert!((config.materials.power_plant.metallic - 0.74).abs() < f32::EPSILON);
    assert!((config.materials.power_plant.emissive_boost - 0.12).abs() < f32::EPSILON);
    assert_eq!(
        config.post_process.tonemapping,
        ViewerTonemappingMode::AcesFitted
    );
    assert!(config.post_process.deband_dither_enabled);
    assert!(!config.post_process.bloom_enabled);
    assert!((config.post_process.bloom_intensity - 0.42).abs() < f32::EPSILON);
    assert!((config.post_process.color_grading_exposure + 0.65).abs() < f32::EPSILON);
    assert!((config.post_process.color_grading_post_saturation - 1.24).abs() < f32::EPSILON);
    assert!((config.label_lod.fade_start_distance - 44.0).abs() < f32::EPSILON);
    assert!((config.label_lod.fade_end_distance - 110.0).abs() < f32::EPSILON);
    assert_eq!(config.label_lod.max_visible_labels, 32);
    assert!((config.label_lod.occlusion_cell_span - 9.0).abs() < f32::EPSILON);
    assert_eq!(config.label_lod.occlusion_cap_per_cell, 3);
    assert_eq!(config.render_budget.overlay_refresh_ticks, 5);
    assert_eq!(config.render_budget.overlay_max_heat_markers, 72);
    assert_eq!(config.render_budget.overlay_max_flow_segments, 96);
    assert!((config.render_budget.grid_lod_distance - 180.0).abs() < f32::EPSILON);
    assert!(config.lighting.shadows_enabled);
    assert!((config.lighting.ambient_brightness - 145.0).abs() < f32::EPSILON);
    assert!((config.lighting.fill_light_ratio - 0.42).abs() < f32::EPSILON);
    assert!((config.lighting.rim_light_ratio - 0.20).abs() < f32::EPSILON);
    assert!(config.physical.enabled);
    assert!((config.physical.meters_per_unit - 2.5).abs() < f32::EPSILON);
    assert!((config.physical.floating_origin_step_m - 1500.0).abs() < f64::EPSILON);
    assert!((config.physical.camera_near_m - 0.2).abs() < f32::EPSILON);
    assert!((config.physical.camera_far_m - 60_000.0).abs() < f32::EPSILON);
    assert!((config.physical.stellar_distance_au - 2.8).abs() < f32::EPSILON);
    assert!((config.physical.luminous_efficacy_lm_per_w - 130.0).abs() < f32::EPSILON);
    assert!((config.physical.exposure_ev100 - 12.8).abs() < f32::EPSILON);
    assert!((config.physical.reference_radiation_area_m2 - 1.2).abs() < f32::EPSILON);

    let external_mesh =
        load_viewer_external_mesh_config_from(|key| env.get(key).map(|v| v.to_string()));
    assert_eq!(
        external_mesh.agent_mesh_asset.as_deref(),
        Some("models/agents/worker.glb#Mesh0/Primitive0")
    );
    assert_eq!(
        external_mesh.location_mesh_asset.as_deref(),
        Some("models/world/location.glb#Mesh0/Primitive0")
    );
    assert_eq!(
        external_mesh.asset_mesh_asset.as_deref(),
        Some("models/world/asset.glb#Mesh0/Primitive0")
    );
    assert_eq!(
        external_mesh.power_plant_mesh_asset.as_deref(),
        Some("models/facility/power_plant.glb#Mesh0/Primitive0")
    );

    let external_texture =
        load_viewer_external_texture_config_from(|key| env.get(key).map(|v| v.to_string()));
    assert_eq!(
        external_texture.agent.base_texture_asset.as_deref(),
        Some("textures/agents/worker_albedo.png")
    );
    assert_eq!(
        external_texture.agent.normal_texture_asset.as_deref(),
        Some("textures/agents/worker_normal.png")
    );
    assert_eq!(
        external_texture
            .agent
            .metallic_roughness_texture_asset
            .as_deref(),
        Some("textures/agents/worker_metal_rough.png")
    );
    assert_eq!(
        external_texture.agent.emissive_texture_asset.as_deref(),
        Some("textures/agents/worker_emissive.png")
    );
    assert_eq!(
        external_texture.location.base_texture_asset.as_deref(),
        Some("textures/world/location_albedo.png")
    );
    assert_eq!(
        external_texture.location.normal_texture_asset.as_deref(),
        Some("textures/world/location_normal.png")
    );
    assert_eq!(
        external_texture
            .location
            .metallic_roughness_texture_asset
            .as_deref(),
        Some("textures/world/location_metal_rough.png")
    );
    assert_eq!(
        external_texture.location.emissive_texture_asset.as_deref(),
        Some("textures/world/location_emissive.png")
    );
    assert_eq!(
        external_texture.asset.base_texture_asset.as_deref(),
        Some("textures/world/asset_albedo.png")
    );
    assert_eq!(
        external_texture.asset.normal_texture_asset.as_deref(),
        Some("textures/world/asset_normal.png")
    );
    assert_eq!(
        external_texture
            .asset
            .metallic_roughness_texture_asset
            .as_deref(),
        Some("textures/world/asset_metal_rough.png")
    );
    assert_eq!(
        external_texture.asset.emissive_texture_asset.as_deref(),
        Some("textures/world/asset_emissive.png")
    );
    assert_eq!(
        external_texture.power_plant.base_texture_asset.as_deref(),
        Some("textures/facility/power_plant_albedo.png")
    );
    assert_eq!(
        external_texture.power_plant.normal_texture_asset.as_deref(),
        Some("textures/facility/power_plant_normal.png")
    );
    assert_eq!(
        external_texture
            .power_plant
            .metallic_roughness_texture_asset
            .as_deref(),
        Some("textures/facility/power_plant_metal_rough.png")
    );
    assert_eq!(
        external_texture
            .power_plant
            .emissive_texture_asset
            .as_deref(),
        Some("textures/facility/power_plant_emissive.png")
    );

    let external_material =
        load_viewer_external_material_config_from(|key| env.get(key).map(|v| v.to_string()));
    assert_rgb_approx_eq(
        external_material.agent.base_color_srgb,
        [1.0, 0.41568628, 0.21960784],
    );
    assert_rgb_approx_eq(
        external_material.agent.emissive_color_srgb,
        [0.9019608, 0.38431373, 0.1882353],
    );
    assert_rgb_approx_eq(
        external_material.location.base_color_srgb,
        [0.29411766, 0.53333336, 0.8509804],
    );
    assert_rgb_approx_eq(
        external_material.location.emissive_color_srgb,
        [0.72156864, 0.84705883, 1.0],
    );
    assert_rgb_approx_eq(
        external_material.asset.base_color_srgb,
        [0.81960785, 0.7647059, 0.3529412],
    );
    assert_rgb_approx_eq(
        external_material.asset.emissive_color_srgb,
        [1.0, 0.83137256, 0.3529412],
    );
    assert_rgb_approx_eq(
        external_material.power_plant.base_color_srgb,
        [0.9529412, 0.4117647, 0.20392157],
    );
    assert_rgb_approx_eq(
        external_material.power_plant.emissive_color_srgb,
        [1.0, 0.49803922, 0.2901961],
    );
}

#[test]
fn load_viewer_3d_config_rejects_removed_old_brand_key() {
    let env = HashMap::from([("AGENT_WORLD_VIEWER_SHOW_AGENTS", "true")]);

    let config = load_viewer_3d_config_from(|key| env.get(key).map(|v| v.to_string()));
    assert_eq!(config.show_agents, Viewer3dConfig::default().show_agents);
}

#[test]
fn load_viewer_3d_config_ignores_invalid_values() {
    let env = HashMap::from([
        ("OASIS7_VIEWER_CM_TO_UNIT", "0"),
        ("OASIS7_VIEWER_RENDER_PROFILE", "invalid"),
        ("OASIS7_VIEWER_SHOW_AGENTS", "invalid"),
        ("OASIS7_VIEWER_VISUAL_EFFECTS", "hyper"),
        (
            "OASIS7_VIEWER_AGENT_VARIANT_PALETTE",
            "#112233,#445566,#778899",
        ),
        ("OASIS7_VIEWER_AGENT_DIRECTION_INDICATOR", "invalid"),
        ("OASIS7_VIEWER_AGENT_SPEED_EFFECT", "invalid"),
        ("OASIS7_VIEWER_AGENT_TRAIL_ENABLED", "2"),
        ("OASIS7_VIEWER_LOCATION_RADIATION_GLOW", "maybe"),
        ("OASIS7_VIEWER_LOCATION_DAMAGE_VISUAL", "??"),
        ("OASIS7_VIEWER_ASSET_QUANTITY_VISUAL", "on-please"),
        ("OASIS7_VIEWER_ASSET_TYPE_COLOR", "nah"),
        ("OASIS7_VIEWER_AGENT_NORMAL_TEXTURE_ASSET", " "),
        ("OASIS7_VIEWER_AGENT_BASE_COLOR", "#12345"),
        ("OASIS7_VIEWER_AGENT_EMISSIVE_COLOR", "123456"),
        ("OASIS7_VIEWER_ASSET_GEOMETRY_TIER", "ultra"),
        ("OASIS7_VIEWER_LOCATION_SHELL_ENABLED", "maybe"),
        ("OASIS7_VIEWER_FRAGMENT_MATERIAL_STRATEGY", "hyper"),
        ("OASIS7_VIEWER_FRAGMENT_UNLIT", "idk"),
        ("OASIS7_VIEWER_FRAGMENT_ALPHA", "1.5"),
        ("OASIS7_VIEWER_FRAGMENT_EMISSIVE_BOOST", "-1"),
        ("OASIS7_VIEWER_MATERIAL_AGENT_ROUGHNESS", "4"),
        ("OASIS7_VIEWER_MATERIAL_AGENT_METALLIC", "-3"),
        ("OASIS7_VIEWER_MATERIAL_POWER_PLANT_ROUGHNESS", "-0.2"),
        ("OASIS7_VIEWER_MATERIAL_POWER_PLANT_METALLIC", "2"),
        ("OASIS7_VIEWER_MATERIAL_POWER_PLANT_EMISSIVE_BOOST", "-1"),
        ("OASIS7_VIEWER_TONEMAPPING", "ultra-hdr"),
        ("OASIS7_VIEWER_DEBAND_DITHER_ENABLED", "???"),
        ("OASIS7_VIEWER_BLOOM_ENABLED", "???"),
        ("OASIS7_VIEWER_BLOOM_INTENSITY", "99"),
        ("OASIS7_VIEWER_COLOR_GRADING_EXPOSURE", "inf"),
        ("OASIS7_VIEWER_COLOR_GRADING_POST_SATURATION", "-1"),
        ("OASIS7_VIEWER_LABEL_FADE_START", "-5"),
        ("OASIS7_VIEWER_LABEL_FADE_END", "2"),
        ("OASIS7_VIEWER_MAX_VISIBLE_LABELS", "0"),
        ("OASIS7_VIEWER_LABEL_OCCLUSION_CELL_SPAN", "0"),
        ("OASIS7_VIEWER_LABEL_OCCLUSION_CAP_PER_CELL", "0"),
        ("OASIS7_VIEWER_OVERLAY_REFRESH_TICKS", "0"),
        ("OASIS7_VIEWER_OVERLAY_MAX_HEAT", "0"),
        ("OASIS7_VIEWER_OVERLAY_MAX_FLOW", "0"),
        ("OASIS7_VIEWER_GRID_LOD_DISTANCE", "0"),
        ("OASIS7_VIEWER_SHADOWS_ENABLED", "invalid"),
        ("OASIS7_VIEWER_AMBIENT_BRIGHTNESS", "0"),
        ("OASIS7_VIEWER_FILL_LIGHT_RATIO", "-1"),
        ("OASIS7_VIEWER_RIM_LIGHT_RATIO", "-1"),
        ("OASIS7_VIEWER_PHYSICAL_RENDER_ENABLED", "1"),
        ("OASIS7_VIEWER_METERS_PER_UNIT", "-1"),
        ("OASIS7_VIEWER_FLOATING_ORIGIN_STEP_M", "nan"),
        ("OASIS7_VIEWER_CAMERA_NEAR_M", "10"),
        ("OASIS7_VIEWER_CAMERA_FAR_M", "2"),
        ("OASIS7_VIEWER_STELLAR_DISTANCE_AU", "-2"),
        ("OASIS7_VIEWER_LUMINOUS_EFFICACY_LM_PER_W", "0"),
        ("OASIS7_VIEWER_REFERENCE_RADIATION_AREA_M2", "0"),
    ]);

    let config = load_viewer_3d_config_from(|key| env.get(key).map(|v| v.to_string()));

    assert!((config.cm_to_unit - DEFAULT_CM_TO_UNIT).abs() < f32::EPSILON);
    assert_eq!(config.render_profile, ViewerRenderProfile::Balanced);
    assert!(config.show_agents);
    assert_eq!(config.visual.level, ViewerVisualEffectsLevel::Standard);
    assert_eq!(config.visual.agent_variant_palette, None);
    assert_eq!(
        config.visual.agent_direction_indicator,
        DEFAULT_AGENT_DIRECTION_INDICATOR
    );
    assert_eq!(config.visual.agent_speed_effect, DEFAULT_AGENT_SPEED_EFFECT);
    assert_eq!(
        config.visual.agent_trail_enabled,
        DEFAULT_AGENT_TRAIL_ENABLED
    );
    assert_eq!(
        config.visual.location_radiation_glow,
        DEFAULT_LOCATION_RADIATION_GLOW
    );
    assert_eq!(
        config.visual.location_damage_visual,
        DEFAULT_LOCATION_DAMAGE_VISUAL
    );
    assert_eq!(
        config.visual.asset_quantity_visual,
        DEFAULT_ASSET_QUANTITY_VISUAL
    );
    assert_eq!(config.visual.asset_type_color, DEFAULT_ASSET_TYPE_COLOR);
    assert_eq!(config.assets.geometry_tier, ViewerGeometryTier::Balanced);
    assert_eq!(
        config.assets.location_shell_enabled,
        DEFAULT_LOCATION_SHELL_ENABLED
    );
    assert_eq!(
        config.materials.fragment.strategy,
        ViewerFragmentMaterialStrategy::Readability
    );
    assert_eq!(config.materials.fragment.unlit, DEFAULT_FRAGMENT_UNLIT);
    assert!((config.materials.fragment.alpha - DEFAULT_FRAGMENT_ALPHA).abs() < f32::EPSILON);
    assert!(
        (config.materials.agent.roughness - DEFAULT_MATERIAL_AGENT_ROUGHNESS).abs() < f32::EPSILON
    );
    assert!(
        (config.materials.agent.metallic - DEFAULT_MATERIAL_AGENT_METALLIC).abs() < f32::EPSILON
    );
    assert!(
        (config.materials.power_plant.roughness - DEFAULT_MATERIAL_POWER_PLANT_ROUGHNESS).abs()
            < f32::EPSILON
    );
    assert!(
        (config.materials.power_plant.metallic - DEFAULT_MATERIAL_POWER_PLANT_METALLIC).abs()
            < f32::EPSILON
    );
    assert!(
        (config.materials.power_plant.emissive_boost - DEFAULT_MATERIAL_POWER_PLANT_EMISSIVE_BOOST)
            .abs()
            < f32::EPSILON
    );
    assert_eq!(
        config.post_process.tonemapping,
        ViewerTonemappingMode::TonyMcMapface
    );
    assert_eq!(
        config.post_process.deband_dither_enabled,
        DEFAULT_DEBAND_DITHER_ENABLED
    );
    assert_eq!(config.post_process.bloom_enabled, DEFAULT_BLOOM_ENABLED);
    assert!((config.post_process.bloom_intensity - DEFAULT_BLOOM_INTENSITY).abs() < f32::EPSILON);
    assert!(
        (config.post_process.color_grading_exposure - DEFAULT_COLOR_GRADING_EXPOSURE).abs()
            < f32::EPSILON
    );
    assert!(
        (config.post_process.color_grading_post_saturation - DEFAULT_COLOR_GRADING_POST_SATURATION)
            .abs()
            < f32::EPSILON
    );
    assert!(
        (config.label_lod.fade_start_distance - DEFAULT_LABEL_FADE_START_DISTANCE).abs()
            < f32::EPSILON
    );
    assert!(
        (config.label_lod.fade_end_distance - DEFAULT_LABEL_FADE_END_DISTANCE).abs() < f32::EPSILON
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
        (config.render_budget.grid_lod_distance - DEFAULT_GRID_LOD_DISTANCE).abs() < f32::EPSILON
    );
    assert_eq!(config.lighting.shadows_enabled, DEFAULT_SHADOWS_ENABLED);
    assert!((config.lighting.ambient_brightness - DEFAULT_AMBIENT_BRIGHTNESS).abs() < f32::EPSILON);
    assert!((config.lighting.fill_light_ratio - DEFAULT_FILL_LIGHT_RATIO).abs() < f32::EPSILON);
    assert!((config.lighting.rim_light_ratio - DEFAULT_RIM_LIGHT_RATIO).abs() < f32::EPSILON);
    assert!(config.physical.enabled);
    assert!((config.physical.meters_per_unit - DEFAULT_METERS_PER_UNIT).abs() < f32::EPSILON);
    assert!(config.physical.floating_origin_step_m.is_finite());
    assert!(config.physical.camera_far_m > config.physical.camera_near_m);
    assert!(
        (config.physical.stellar_distance_au - DEFAULT_STELLAR_DISTANCE_AU).abs() < f32::EPSILON
    );
    assert!(
        (config.physical.luminous_efficacy_lm_per_w - DEFAULT_LUMINOUS_EFFICACY_LM_PER_W).abs()
            < f32::EPSILON
    );
    assert!(
        (config.physical.reference_radiation_area_m2 - DEFAULT_REFERENCE_RADIATION_AREA_M2).abs()
            < f32::EPSILON
    );

    let external_material =
        load_viewer_external_material_config_from(|key| env.get(key).map(|v| v.to_string()));
    assert_eq!(external_material.agent.base_color_srgb, None);
    assert_eq!(external_material.agent.emissive_color_srgb, None);

    let external_texture =
        load_viewer_external_texture_config_from(|key| env.get(key).map(|v| v.to_string()));
    assert_eq!(external_texture.agent.base_texture_asset, None);
    assert_eq!(external_texture.agent.normal_texture_asset, None);
    assert_eq!(
        external_texture.agent.metallic_roughness_texture_asset,
        None
    );
    assert_eq!(external_texture.agent.emissive_texture_asset, None);
}

#[test]
fn render_profile_sets_asset_defaults_and_allows_explicit_override() {
    let debug_profile_env = HashMap::from([("OASIS7_VIEWER_RENDER_PROFILE", "debug")]);
    let debug_config =
        load_viewer_3d_config_from(|key| debug_profile_env.get(key).map(|v| v.to_string()));
    assert_eq!(debug_config.render_profile, ViewerRenderProfile::Debug);
    assert_eq!(debug_config.assets.geometry_tier, ViewerGeometryTier::Debug);
    assert!(!debug_config.assets.location_shell_enabled);
    assert_eq!(
        debug_config.materials.fragment.strategy,
        ViewerFragmentMaterialStrategy::Readability
    );
    assert!(debug_config.materials.fragment.unlit);
    assert!(!debug_config.lighting.shadows_enabled);
    assert_eq!(
        debug_config.post_process.tonemapping,
        ViewerTonemappingMode::Reinhard
    );
    assert!(!debug_config.post_process.deband_dither_enabled);
    assert!(!debug_config.post_process.bloom_enabled);

    let cinematic_with_override_env = HashMap::from([
        ("OASIS7_VIEWER_RENDER_PROFILE", "cinematic"),
        ("OASIS7_VIEWER_LOCATION_SHELL_ENABLED", "false"),
        ("OASIS7_VIEWER_BLOOM_ENABLED", "false"),
    ]);
    let cinematic_config = load_viewer_3d_config_from(|key| {
        cinematic_with_override_env.get(key).map(|v| v.to_string())
    });
    assert_eq!(
        cinematic_config.render_profile,
        ViewerRenderProfile::Cinematic
    );
    assert_eq!(
        cinematic_config.assets.geometry_tier,
        ViewerGeometryTier::Cinematic
    );
    assert!(!cinematic_config.assets.location_shell_enabled);
    assert_eq!(
        cinematic_config.materials.fragment.strategy,
        ViewerFragmentMaterialStrategy::Fidelity
    );
    assert!(!cinematic_config.materials.fragment.unlit);
    assert!(cinematic_config.lighting.shadows_enabled);
    assert_eq!(
        cinematic_config.post_process.tonemapping,
        ViewerTonemappingMode::BlenderFilmic
    );
    assert!(!cinematic_config.post_process.bloom_enabled);
}

#[test]
fn visual_effects_level_sets_baseline_and_allows_explicit_toggle_override() {
    let minimal_env = HashMap::from([("OASIS7_VIEWER_VISUAL_EFFECTS", "minimal")]);
    let minimal_config =
        load_viewer_3d_config_from(|key| minimal_env.get(key).map(|v| v.to_string()));
    assert_eq!(
        minimal_config.visual.level,
        ViewerVisualEffectsLevel::Minimal
    );
    assert!(!minimal_config.visual.agent_direction_indicator);
    assert!(!minimal_config.visual.agent_speed_effect);
    assert!(!minimal_config.visual.agent_trail_enabled);
    assert!(!minimal_config.visual.location_radiation_glow);
    assert!(!minimal_config.visual.location_damage_visual);
    assert!(!minimal_config.visual.asset_quantity_visual);
    assert!(!minimal_config.visual.asset_type_color);

    let minimal_override_env = HashMap::from([
        ("OASIS7_VIEWER_VISUAL_EFFECTS", "minimal"),
        ("OASIS7_VIEWER_AGENT_SPEED_EFFECT", "1"),
        ("OASIS7_VIEWER_LOCATION_RADIATION_GLOW", "true"),
    ]);
    let minimal_override_config =
        load_viewer_3d_config_from(|key| minimal_override_env.get(key).map(|v| v.to_string()));
    assert_eq!(
        minimal_override_config.visual.level,
        ViewerVisualEffectsLevel::Minimal
    );
    assert!(!minimal_override_config.visual.agent_direction_indicator);
    assert!(minimal_override_config.visual.agent_speed_effect);
    assert!(!minimal_override_config.visual.agent_trail_enabled);
    assert!(minimal_override_config.visual.location_radiation_glow);

    let enhanced_env = HashMap::from([("OASIS7_VIEWER_VISUAL_EFFECTS", "enhanced")]);
    let enhanced_config =
        load_viewer_3d_config_from(|key| enhanced_env.get(key).map(|v| v.to_string()));
    assert_eq!(
        enhanced_config.visual.level,
        ViewerVisualEffectsLevel::Enhanced
    );
    assert!(enhanced_config.visual.agent_direction_indicator);
    assert!(enhanced_config.visual.agent_speed_effect);
    assert!(enhanced_config.visual.agent_trail_enabled);
    assert!(enhanced_config.visual.location_radiation_glow);
    assert!(enhanced_config.visual.location_damage_visual);
    assert!(enhanced_config.visual.asset_quantity_visual);
    assert!(enhanced_config.visual.asset_type_color);
}

#[test]
fn load_viewer_external_mesh_config_ignores_empty_values() {
    let env = HashMap::from([
        ("OASIS7_VIEWER_AGENT_MESH_ASSET", "  "),
        ("OASIS7_VIEWER_LOCATION_MESH_ASSET", ""),
        (
            "OASIS7_VIEWER_ASSET_MESH_ASSET",
            " models/world/asset.glb#Mesh0/Primitive0 ",
        ),
    ]);

    let external_mesh =
        load_viewer_external_mesh_config_from(|key| env.get(key).map(|v| v.to_string()));
    assert_eq!(external_mesh.agent_mesh_asset, None);
    assert_eq!(external_mesh.location_mesh_asset, None);
    assert_eq!(
        external_mesh.asset_mesh_asset.as_deref(),
        Some("models/world/asset.glb#Mesh0/Primitive0")
    );
    assert_eq!(external_mesh.power_plant_mesh_asset, None);
}

#[test]
fn load_viewer_external_material_config_ignores_empty_or_invalid_values() {
    let env = HashMap::from([
        ("OASIS7_VIEWER_AGENT_BASE_COLOR", "   "),
        ("OASIS7_VIEWER_AGENT_EMISSIVE_COLOR", "#GG2233"),
        ("OASIS7_VIEWER_ASSET_BASE_COLOR", " #D1C35A "),
    ]);

    let external_material =
        load_viewer_external_material_config_from(|key| env.get(key).map(|v| v.to_string()));
    assert_eq!(external_material.agent.base_color_srgb, None);
    assert_eq!(external_material.agent.emissive_color_srgb, None);
    assert_rgb_approx_eq(
        external_material.asset.base_color_srgb,
        [0.81960785, 0.7647059, 0.3529412],
    );
    assert_eq!(external_material.location.base_color_srgb, None);
    assert_eq!(external_material.power_plant.base_color_srgb, None);
}

#[test]
fn load_viewer_external_texture_config_ignores_empty_values() {
    let env = HashMap::from([
        ("OASIS7_VIEWER_AGENT_BASE_TEXTURE_ASSET", " "),
        ("OASIS7_VIEWER_AGENT_NORMAL_TEXTURE_ASSET", " "),
        (
            "OASIS7_VIEWER_LOCATION_BASE_TEXTURE_ASSET",
            " textures/world/location_albedo.png ",
        ),
        (
            "OASIS7_VIEWER_LOCATION_NORMAL_TEXTURE_ASSET",
            " textures/world/location_normal.png ",
        ),
        ("OASIS7_VIEWER_ASSET_BASE_TEXTURE_ASSET", ""),
        ("OASIS7_VIEWER_ASSET_EMISSIVE_TEXTURE_ASSET", ""),
    ]);

    let external_texture =
        load_viewer_external_texture_config_from(|key| env.get(key).map(|v| v.to_string()));
    assert_eq!(external_texture.agent.base_texture_asset, None);
    assert_eq!(external_texture.agent.normal_texture_asset, None);
    assert_eq!(
        external_texture.location.base_texture_asset.as_deref(),
        Some("textures/world/location_albedo.png")
    );
    assert_eq!(
        external_texture.location.normal_texture_asset.as_deref(),
        Some("textures/world/location_normal.png")
    );
    assert_eq!(external_texture.asset.base_texture_asset, None);
    assert_eq!(external_texture.asset.emissive_texture_asset, None);
    assert_eq!(external_texture.power_plant.base_texture_asset, None);
}
