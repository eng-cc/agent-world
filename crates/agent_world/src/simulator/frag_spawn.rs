use crate::geometry::GeoPos;

use super::world_model::{Location, SpaceConfig};

pub(super) const FRAGMENT_LOCATION_PREFIX: &str = "frag-";

const FRAGMENT_SPAWN_STANDOFF_CM: f64 = 5_000.0;
const FRAGMENT_SPAWN_MIN_STANDOFF_CM: f64 = 2_000.0;
const FRAGMENT_SPAWN_UPWARD_BIAS: f64 = 0.82;

pub(super) fn fragment_spawn_pos(
    location: &Location,
    space: &SpaceConfig,
    spawn_roll: u64,
) -> GeoPos {
    let angle = (splitmix64(spawn_roll.wrapping_add(0x9E37_79B9_7F4A_7C15)) as f64
        / u64::MAX as f64)
        * std::f64::consts::TAU;
    let planar = (1.0 - FRAGMENT_SPAWN_UPWARD_BIAS * FRAGMENT_SPAWN_UPWARD_BIAS).sqrt();
    let radius_cm = location.profile.radius_cm.max(1) as f64;
    let spawn_at = |standoff_cm: f64| {
        let distance_cm = radius_cm + standoff_cm;
        GeoPos::new(
            location.pos.x_cm + angle.cos() * planar * distance_cm,
            location.pos.y_cm + angle.sin() * planar * distance_cm,
            location.pos.z_cm + FRAGMENT_SPAWN_UPWARD_BIAS * distance_cm,
        )
    };

    let primary = spawn_at(FRAGMENT_SPAWN_STANDOFF_CM);
    if space.contains(primary) {
        return primary;
    }

    let fallback = spawn_at(FRAGMENT_SPAWN_MIN_STANDOFF_CM);
    if space.contains(fallback) {
        return fallback;
    }

    location.pos
}

fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
