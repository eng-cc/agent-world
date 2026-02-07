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

#[derive(Clone, Copy, Debug, Resource)]
pub(super) struct Viewer3dConfig {
    pub cm_to_unit: f32,
    pub show_agents: bool,
    pub show_locations: bool,
    pub highlight_selected: bool,
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
            physical: ViewerPhysicalRenderConfig::default(),
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
        1361.0 / (distance * distance)
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
    }
}
