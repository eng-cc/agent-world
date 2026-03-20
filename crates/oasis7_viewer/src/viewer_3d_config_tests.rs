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
        (config.materials.agent.roughness - DEFAULT_MATERIAL_AGENT_ROUGHNESS).abs() < f32::EPSILON
    );
    assert!(
        (config.materials.agent.metallic - DEFAULT_MATERIAL_AGENT_METALLIC).abs() < f32::EPSILON
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
