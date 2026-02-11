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
const DEFAULT_LABEL_FADE_START_DISTANCE: f32 = 55.0;
const DEFAULT_LABEL_FADE_END_DISTANCE: f32 = 140.0;
const DEFAULT_MAX_VISIBLE_LABELS: usize = 48;
const DEFAULT_LABEL_OCCLUSION_CELL_SPAN: f32 = 8.0;
const DEFAULT_LABEL_OCCLUSION_CAP_PER_CELL: usize = 2;

#[derive(Clone, Copy, Debug, Resource)]
pub(super) struct Viewer3dConfig {
    pub cm_to_unit: f32,
    pub show_agents: bool,
    pub show_locations: bool,
    pub highlight_selected: bool,
    pub label_lod: ViewerLabelLodConfig,
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
            show_agents: true,
            show_locations: true,
            highlight_selected: true,
            label_lod: ViewerLabelLodConfig::default(),
            lighting: ViewerLightingConfig::default(),
            physical: ViewerPhysicalRenderConfig::default(),
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
pub(super) struct ViewerLightingConfig {
    pub shadows_enabled: bool,
    pub ambient_brightness: f32,
    pub fill_light_ratio: f32,
}

impl Default for ViewerLightingConfig {
    fn default() -> Self {
        Self {
            shadows_enabled: DEFAULT_SHADOWS_ENABLED,
            ambient_brightness: DEFAULT_AMBIENT_BRIGHTNESS,
            fill_light_ratio: DEFAULT_FILL_LIGHT_RATIO,
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

fn load_viewer_3d_config_from<F>(lookup: F) -> Viewer3dConfig
where
    F: Fn(&str) -> Option<String>,
{
    let mut config = Viewer3dConfig::default();
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

fn parse_usize<F>(lookup: &F, key: &str) -> Option<usize>
where
    F: Fn(&str) -> Option<String>,
{
    lookup(key).and_then(|raw| raw.trim().parse::<usize>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn defaults_match_rpa1_baseline() {
        let config = Viewer3dConfig::default();
        assert!((config.cm_to_unit - 0.00001).abs() < f32::EPSILON);
        assert!(config.show_agents);
        assert!(config.show_locations);
        assert!(config.highlight_selected);
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
        assert!(!config.lighting.shadows_enabled);
        assert!((config.lighting.ambient_brightness - 110.0).abs() < f32::EPSILON);
        assert!((config.lighting.fill_light_ratio - 0.28).abs() < f32::EPSILON);
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
    fn load_viewer_3d_config_applies_env_overrides() {
        let env = HashMap::from([
            ("AGENT_WORLD_VIEWER_CM_TO_UNIT", "0.0002"),
            ("AGENT_WORLD_VIEWER_SHOW_AGENTS", "false"),
            ("AGENT_WORLD_VIEWER_SHOW_LOCATIONS", "0"),
            ("AGENT_WORLD_VIEWER_HIGHLIGHT_SELECTED", "no"),
            ("AGENT_WORLD_VIEWER_LABEL_FADE_START", "44"),
            ("AGENT_WORLD_VIEWER_LABEL_FADE_END", "110"),
            ("AGENT_WORLD_VIEWER_MAX_VISIBLE_LABELS", "32"),
            ("AGENT_WORLD_VIEWER_LABEL_OCCLUSION_CELL_SPAN", "9"),
            ("AGENT_WORLD_VIEWER_LABEL_OCCLUSION_CAP_PER_CELL", "3"),
            ("AGENT_WORLD_VIEWER_SHADOWS_ENABLED", "1"),
            ("AGENT_WORLD_VIEWER_AMBIENT_BRIGHTNESS", "145"),
            ("AGENT_WORLD_VIEWER_FILL_LIGHT_RATIO", "0.42"),
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
        assert!(!config.show_agents);
        assert!(!config.show_locations);
        assert!(!config.highlight_selected);
        assert!((config.label_lod.fade_start_distance - 44.0).abs() < f32::EPSILON);
        assert!((config.label_lod.fade_end_distance - 110.0).abs() < f32::EPSILON);
        assert_eq!(config.label_lod.max_visible_labels, 32);
        assert!((config.label_lod.occlusion_cell_span - 9.0).abs() < f32::EPSILON);
        assert_eq!(config.label_lod.occlusion_cap_per_cell, 3);
        assert!(config.lighting.shadows_enabled);
        assert!((config.lighting.ambient_brightness - 145.0).abs() < f32::EPSILON);
        assert!((config.lighting.fill_light_ratio - 0.42).abs() < f32::EPSILON);
        assert!(config.physical.enabled);
        assert!((config.physical.meters_per_unit - 2.5).abs() < f32::EPSILON);
        assert!((config.physical.floating_origin_step_m - 1500.0).abs() < f64::EPSILON);
        assert!((config.physical.camera_near_m - 0.2).abs() < f32::EPSILON);
        assert!((config.physical.camera_far_m - 60_000.0).abs() < f32::EPSILON);
        assert!((config.physical.stellar_distance_au - 2.8).abs() < f32::EPSILON);
        assert!((config.physical.luminous_efficacy_lm_per_w - 130.0).abs() < f32::EPSILON);
        assert!((config.physical.exposure_ev100 - 12.8).abs() < f32::EPSILON);
        assert!((config.physical.reference_radiation_area_m2 - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn load_viewer_3d_config_ignores_invalid_values() {
        let env = HashMap::from([
            ("AGENT_WORLD_VIEWER_CM_TO_UNIT", "0"),
            ("AGENT_WORLD_VIEWER_SHOW_AGENTS", "invalid"),
            ("AGENT_WORLD_VIEWER_LABEL_FADE_START", "-5"),
            ("AGENT_WORLD_VIEWER_LABEL_FADE_END", "2"),
            ("AGENT_WORLD_VIEWER_MAX_VISIBLE_LABELS", "0"),
            ("AGENT_WORLD_VIEWER_LABEL_OCCLUSION_CELL_SPAN", "0"),
            ("AGENT_WORLD_VIEWER_LABEL_OCCLUSION_CAP_PER_CELL", "0"),
            ("AGENT_WORLD_VIEWER_SHADOWS_ENABLED", "invalid"),
            ("AGENT_WORLD_VIEWER_AMBIENT_BRIGHTNESS", "0"),
            ("AGENT_WORLD_VIEWER_FILL_LIGHT_RATIO", "-1"),
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
        assert!(config.show_agents);
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
        assert_eq!(config.lighting.shadows_enabled, DEFAULT_SHADOWS_ENABLED);
        assert!(
            (config.lighting.ambient_brightness - DEFAULT_AMBIENT_BRIGHTNESS).abs() < f32::EPSILON
        );
        assert!((config.lighting.fill_light_ratio - DEFAULT_FILL_LIGHT_RATIO).abs() < f32::EPSILON);
        assert!(config.physical.enabled);
        assert!((config.physical.meters_per_unit - DEFAULT_METERS_PER_UNIT).abs() < f32::EPSILON);
        assert!(config.physical.floating_origin_step_m.is_finite());
        assert!(config.physical.camera_far_m > config.physical.camera_near_m);
        assert!(
            (config.physical.stellar_distance_au - DEFAULT_STELLAR_DISTANCE_AU).abs()
                < f32::EPSILON
        );
        assert!(
            (config.physical.luminous_efficacy_lm_per_w - DEFAULT_LUMINOUS_EFFICACY_LM_PER_W).abs()
                < f32::EPSILON
        );
        assert!(
            (config.physical.reference_radiation_area_m2 - DEFAULT_REFERENCE_RADIATION_AREA_M2)
                .abs()
                < f32::EPSILON
        );
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
