//! Asteroid fragment belt generation utilities.

use crate::geometry::GeoPos;

use super::types::{LocationProfile, MaterialKind};
use super::world_model::{AsteroidFragmentConfig, Location, SpaceConfig};

const MAX_PLACEMENT_ATTEMPTS: usize = 8;

pub fn generate_fragments(
    seed: u64,
    space: &SpaceConfig,
    config: &AsteroidFragmentConfig,
) -> Vec<Location> {
    let mut rng = Lcg::new(seed);
    let voxel_cm = (config.voxel_size_km as i64).max(1) * 100_000;
    let voxels_x = ((space.width_cm + voxel_cm - 1) / voxel_cm).max(1);
    let voxels_y = ((space.depth_cm + voxel_cm - 1) / voxel_cm).max(1);
    let voxels_z = ((space.height_cm + voxel_cm - 1) / voxel_cm).max(1);

    let voxel_volume_km3 = (config.voxel_size_km as f64).powi(3).max(1e-6);
    let total_weights = config.material_weights.total().max(1);

    let mut locations = Vec::new();
    let mut placements = Vec::new();
    let mut idx = 0usize;
    let mid_z_cm = space.height_cm as f64 / 2.0;
    let min_spacing_cm = config.min_fragment_spacing_cm.max(0) as f64;
    let enforce_spacing = min_spacing_cm > 0.0;

    for ix in 0..voxels_x {
        for iy in 0..voxels_y {
            for iz in 0..voxels_z {
                let z_cm = (iz as i64 * voxel_cm) as f64 + (voxel_cm as f64 / 2.0);
                let z_km = ((z_cm - mid_z_cm).abs() / 100_000.0).max(0.0);
                let layer = if config.layer_scale_height_km > 0.0 {
                    (-z_km / config.layer_scale_height_km).exp()
                } else {
                    1.0
                };
                let noise = (rng.next_f64() * 2.0 - 1.0) * config.cluster_noise;
                let density = (config.base_density_per_km3 * (1.0 + noise).max(0.0)) * layer;
                let lambda = density * voxel_volume_km3;
                let count = sample_poisson(&mut rng, lambda);

                for _ in 0..count {
                    let mut placed = false;
                    for _ in 0..MAX_PLACEMENT_ATTEMPTS {
                        let x = (ix as i64 * voxel_cm) as f64 + rng.next_f64() * voxel_cm as f64;
                        let y = (iy as i64 * voxel_cm) as f64 + rng.next_f64() * voxel_cm as f64;
                        let z = (iz as i64 * voxel_cm) as f64 + rng.next_f64() * voxel_cm as f64;
                        let radius_cm = sample_power_law(
                            &mut rng,
                            config.radius_min_cm.max(1) as f64,
                            config.radius_max_cm.max(1) as f64,
                            config.size_powerlaw_q.max(1.0),
                        );
                        let radius_cm = radius_cm.round().max(1.0) as i64;
                        let pos = GeoPos {
                            x_cm: x,
                            y_cm: y,
                            z_cm: z,
                        };
                        if enforce_spacing
                            && !spacing_allows(&pos, radius_cm, &placements, min_spacing_cm)
                        {
                            continue;
                        }

                        let roll = rng.next_u32() % total_weights;
                        let material = config.material_weights.pick(roll);
                        let emission = estimate_radiation_emission(radius_cm as f64, material);
                        let profile = LocationProfile {
                            material,
                            radius_cm,
                            radiation_emission_per_tick: emission,
                        };
                        let location_id = format!("frag-{idx}");
                        let name = location_id.clone();
                        locations.push(Location::new_with_profile(location_id, name, pos, profile));
                        placements.push((pos, radius_cm));
                        idx += 1;
                        placed = true;
                        break;
                    }
                    if !placed {
                        continue;
                    }
                }
            }
        }
    }

    locations
}

fn spacing_allows(
    pos: &GeoPos,
    radius_cm: i64,
    existing: &[(GeoPos, i64)],
    min_spacing_cm: f64,
) -> bool {
    if min_spacing_cm <= 0.0 {
        return true;
    }
    let radius_cm = radius_cm as f64;
    for (other_pos, other_radius_cm) in existing {
        let dx = pos.x_cm - other_pos.x_cm;
        let dy = pos.y_cm - other_pos.y_cm;
        let dz = pos.z_cm - other_pos.z_cm;
        let min_dist = radius_cm + (*other_radius_cm as f64) + min_spacing_cm;
        if (dx * dx + dy * dy + dz * dz) < (min_dist * min_dist) {
            return false;
        }
    }
    true
}

fn estimate_radiation_emission(radius_cm: f64, material: MaterialKind) -> i64 {
    let factor = match material {
        MaterialKind::Silicate => 1.0,
        MaterialKind::Metal => 1.2,
        MaterialKind::Ice => 0.8,
        MaterialKind::Carbon => 0.9,
        MaterialKind::Composite => 1.1,
    };
    let base = (radius_cm / 100.0).max(1.0) * factor;
    base.round().max(1.0) as i64
}

fn sample_power_law(rng: &mut Lcg, r_min: f64, r_max: f64, q: f64) -> f64 {
    if (q - 1.0).abs() < f64::EPSILON {
        let u = rng.next_f64().max(1e-9);
        return (r_min.ln() + u * (r_max.ln() - r_min.ln())).exp();
    }
    let u = rng.next_f64().max(1e-9);
    let one_minus_q = 1.0 - q;
    let min_term = r_min.powf(one_minus_q);
    let max_term = r_max.powf(one_minus_q);
    let value = min_term + u * (max_term - min_term);
    value.max(0.0).powf(1.0 / one_minus_q)
}

fn sample_poisson(rng: &mut Lcg, lambda: f64) -> usize {
    if lambda <= 0.0 {
        return 0;
    }
    let l = (-lambda).exp();
    let mut k = 0usize;
    let mut p = 1.0;
    while p > l {
        k += 1;
        p *= rng.next_f64();
    }
    k.saturating_sub(1)
}

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }

    fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    fn next_f64(&mut self) -> f64 {
        let val = (self.next_u64() >> 11) as f64;
        val / ((1u64 << 53) as f64)
    }
}
