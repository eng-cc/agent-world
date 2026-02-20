use super::*;
use std::collections::HashMap;

#[test]
fn load_viewer_3d_config_applies_env_overrides() {
    let env = HashMap::from([
        ("AGENT_WORLD_VIEWER_CM_TO_UNIT", "0.0002"),
        ("AGENT_WORLD_VIEWER_RENDER_PROFILE", "debug"),
        ("AGENT_WORLD_VIEWER_SHOW_AGENTS", "false"),
        ("AGENT_WORLD_VIEWER_SHOW_LOCATIONS", "0"),
        ("AGENT_WORLD_VIEWER_HIGHLIGHT_SELECTED", "no"),
        (
            "AGENT_WORLD_VIEWER_AGENT_MESH_ASSET",
            "models/agents/worker.glb#Mesh0/Primitive0",
        ),
        (
            "AGENT_WORLD_VIEWER_LOCATION_MESH_ASSET",
            "models/world/location.glb#Mesh0/Primitive0",
        ),
        (
            "AGENT_WORLD_VIEWER_ASSET_MESH_ASSET",
            "models/world/asset.glb#Mesh0/Primitive0",
        ),
        (
            "AGENT_WORLD_VIEWER_POWER_PLANT_MESH_ASSET",
            "models/facility/power_plant.glb#Mesh0/Primitive0",
        ),
        (
            "AGENT_WORLD_VIEWER_POWER_STORAGE_MESH_ASSET",
            "models/facility/power_storage.glb#Mesh0/Primitive0",
        ),
        ("AGENT_WORLD_VIEWER_ASSET_GEOMETRY_TIER", "cinematic"),
        ("AGENT_WORLD_VIEWER_LOCATION_SHELL_ENABLED", "true"),
        ("AGENT_WORLD_VIEWER_FRAGMENT_MATERIAL_STRATEGY", "fidelity"),
        ("AGENT_WORLD_VIEWER_FRAGMENT_UNLIT", "false"),
        ("AGENT_WORLD_VIEWER_FRAGMENT_ALPHA", "0.78"),
        ("AGENT_WORLD_VIEWER_FRAGMENT_EMISSIVE_BOOST", "0.40"),
        ("AGENT_WORLD_VIEWER_MATERIAL_AGENT_ROUGHNESS", "0.44"),
        ("AGENT_WORLD_VIEWER_MATERIAL_AGENT_METALLIC", "0.22"),
        ("AGENT_WORLD_VIEWER_MATERIAL_ASSET_ROUGHNESS", "0.61"),
        ("AGENT_WORLD_VIEWER_MATERIAL_ASSET_METALLIC", "0.33"),
        ("AGENT_WORLD_VIEWER_MATERIAL_FACILITY_ROUGHNESS", "0.53"),
        ("AGENT_WORLD_VIEWER_MATERIAL_FACILITY_METALLIC", "0.47"),
        ("AGENT_WORLD_VIEWER_TONEMAPPING", "aces"),
        ("AGENT_WORLD_VIEWER_DEBAND_DITHER_ENABLED", "true"),
        ("AGENT_WORLD_VIEWER_BLOOM_ENABLED", "false"),
        ("AGENT_WORLD_VIEWER_BLOOM_INTENSITY", "0.42"),
        ("AGENT_WORLD_VIEWER_COLOR_GRADING_EXPOSURE", "-0.65"),
        ("AGENT_WORLD_VIEWER_COLOR_GRADING_POST_SATURATION", "1.24"),
        ("AGENT_WORLD_VIEWER_LABEL_FADE_START", "44"),
        ("AGENT_WORLD_VIEWER_LABEL_FADE_END", "110"),
        ("AGENT_WORLD_VIEWER_MAX_VISIBLE_LABELS", "32"),
        ("AGENT_WORLD_VIEWER_LABEL_OCCLUSION_CELL_SPAN", "9"),
        ("AGENT_WORLD_VIEWER_LABEL_OCCLUSION_CAP_PER_CELL", "3"),
        ("AGENT_WORLD_VIEWER_OVERLAY_REFRESH_TICKS", "5"),
        ("AGENT_WORLD_VIEWER_OVERLAY_MAX_HEAT", "72"),
        ("AGENT_WORLD_VIEWER_OVERLAY_MAX_FLOW", "96"),
        ("AGENT_WORLD_VIEWER_GRID_LOD_DISTANCE", "180"),
        ("AGENT_WORLD_VIEWER_SHADOWS_ENABLED", "1"),
        ("AGENT_WORLD_VIEWER_AMBIENT_BRIGHTNESS", "145"),
        ("AGENT_WORLD_VIEWER_FILL_LIGHT_RATIO", "0.42"),
        ("AGENT_WORLD_VIEWER_RIM_LIGHT_RATIO", "0.20"),
        ("AGENT_WORLD_VIEWER_PHYSICAL_RENDER_ENABLED", "true"),
        ("AGENT_WORLD_VIEWER_METERS_PER_UNIT", "2.5"),
        ("AGENT_WORLD_VIEWER_FLOATING_ORIGIN_STEP_M", "1500"),
        ("AGENT_WORLD_VIEWER_CAMERA_NEAR_M", "0.2"),
        ("AGENT_WORLD_VIEWER_CAMERA_FAR_M", "60000"),
        ("AGENT_WORLD_VIEWER_STELLAR_DISTANCE_AU", "2.8"),
        ("AGENT_WORLD_VIEWER_LUMINOUS_EFFICACY_LM_PER_W", "130"),
        ("AGENT_WORLD_VIEWER_EXPOSURE_EV100", "12.8"),
        ("AGENT_WORLD_VIEWER_REFERENCE_RADIATION_AREA_M2", "1.2"),
    ]);

    let config = load_viewer_3d_config_from(|key| env.get(key).map(|v| v.to_string()));

    assert!((config.cm_to_unit - 0.0002).abs() < f32::EPSILON);
    assert_eq!(config.render_profile, ViewerRenderProfile::Debug);
    assert!(!config.show_agents);
    assert!(!config.show_locations);
    assert!(!config.highlight_selected);
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
    assert_eq!(
        external_mesh.power_storage_mesh_asset.as_deref(),
        Some("models/facility/power_storage.glb#Mesh0/Primitive0")
    );
}

#[test]
fn load_viewer_3d_config_ignores_invalid_values() {
    let env = HashMap::from([
        ("AGENT_WORLD_VIEWER_CM_TO_UNIT", "0"),
        ("AGENT_WORLD_VIEWER_RENDER_PROFILE", "invalid"),
        ("AGENT_WORLD_VIEWER_SHOW_AGENTS", "invalid"),
        ("AGENT_WORLD_VIEWER_ASSET_GEOMETRY_TIER", "ultra"),
        ("AGENT_WORLD_VIEWER_LOCATION_SHELL_ENABLED", "maybe"),
        ("AGENT_WORLD_VIEWER_FRAGMENT_MATERIAL_STRATEGY", "hyper"),
        ("AGENT_WORLD_VIEWER_FRAGMENT_UNLIT", "idk"),
        ("AGENT_WORLD_VIEWER_FRAGMENT_ALPHA", "1.5"),
        ("AGENT_WORLD_VIEWER_FRAGMENT_EMISSIVE_BOOST", "-1"),
        ("AGENT_WORLD_VIEWER_MATERIAL_AGENT_ROUGHNESS", "4"),
        ("AGENT_WORLD_VIEWER_MATERIAL_AGENT_METALLIC", "-3"),
        ("AGENT_WORLD_VIEWER_TONEMAPPING", "ultra-hdr"),
        ("AGENT_WORLD_VIEWER_DEBAND_DITHER_ENABLED", "???"),
        ("AGENT_WORLD_VIEWER_BLOOM_ENABLED", "???"),
        ("AGENT_WORLD_VIEWER_BLOOM_INTENSITY", "99"),
        ("AGENT_WORLD_VIEWER_COLOR_GRADING_EXPOSURE", "inf"),
        ("AGENT_WORLD_VIEWER_COLOR_GRADING_POST_SATURATION", "-1"),
        ("AGENT_WORLD_VIEWER_LABEL_FADE_START", "-5"),
        ("AGENT_WORLD_VIEWER_LABEL_FADE_END", "2"),
        ("AGENT_WORLD_VIEWER_MAX_VISIBLE_LABELS", "0"),
        ("AGENT_WORLD_VIEWER_LABEL_OCCLUSION_CELL_SPAN", "0"),
        ("AGENT_WORLD_VIEWER_LABEL_OCCLUSION_CAP_PER_CELL", "0"),
        ("AGENT_WORLD_VIEWER_OVERLAY_REFRESH_TICKS", "0"),
        ("AGENT_WORLD_VIEWER_OVERLAY_MAX_HEAT", "0"),
        ("AGENT_WORLD_VIEWER_OVERLAY_MAX_FLOW", "0"),
        ("AGENT_WORLD_VIEWER_GRID_LOD_DISTANCE", "0"),
        ("AGENT_WORLD_VIEWER_SHADOWS_ENABLED", "invalid"),
        ("AGENT_WORLD_VIEWER_AMBIENT_BRIGHTNESS", "0"),
        ("AGENT_WORLD_VIEWER_FILL_LIGHT_RATIO", "-1"),
        ("AGENT_WORLD_VIEWER_RIM_LIGHT_RATIO", "-1"),
        ("AGENT_WORLD_VIEWER_PHYSICAL_RENDER_ENABLED", "1"),
        ("AGENT_WORLD_VIEWER_METERS_PER_UNIT", "-1"),
        ("AGENT_WORLD_VIEWER_FLOATING_ORIGIN_STEP_M", "nan"),
        ("AGENT_WORLD_VIEWER_CAMERA_NEAR_M", "10"),
        ("AGENT_WORLD_VIEWER_CAMERA_FAR_M", "2"),
        ("AGENT_WORLD_VIEWER_STELLAR_DISTANCE_AU", "-2"),
        ("AGENT_WORLD_VIEWER_LUMINOUS_EFFICACY_LM_PER_W", "0"),
        ("AGENT_WORLD_VIEWER_REFERENCE_RADIATION_AREA_M2", "0"),
    ]);

    let config = load_viewer_3d_config_from(|key| env.get(key).map(|v| v.to_string()));

    assert!((config.cm_to_unit - DEFAULT_CM_TO_UNIT).abs() < f32::EPSILON);
    assert_eq!(config.render_profile, ViewerRenderProfile::Balanced);
    assert!(config.show_agents);
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
}

#[test]
fn render_profile_sets_asset_defaults_and_allows_explicit_override() {
    let debug_profile_env = HashMap::from([("AGENT_WORLD_VIEWER_RENDER_PROFILE", "debug")]);
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
        ("AGENT_WORLD_VIEWER_RENDER_PROFILE", "cinematic"),
        ("AGENT_WORLD_VIEWER_LOCATION_SHELL_ENABLED", "false"),
        ("AGENT_WORLD_VIEWER_BLOOM_ENABLED", "false"),
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
fn load_viewer_external_mesh_config_ignores_empty_values() {
    let env = HashMap::from([
        ("AGENT_WORLD_VIEWER_AGENT_MESH_ASSET", "  "),
        ("AGENT_WORLD_VIEWER_LOCATION_MESH_ASSET", ""),
        (
            "AGENT_WORLD_VIEWER_ASSET_MESH_ASSET",
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
    assert_eq!(external_mesh.power_storage_mesh_asset, None);
}
