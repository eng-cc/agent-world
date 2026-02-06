use super::*;

#[test]
fn dust_generator_produces_locations_within_bounds() {
    let space = SpaceConfig {
        width_cm: 100_000,
        depth_cm: 100_000,
        height_cm: 100_000,
    };
    let mut dust = DustConfig::default();
    dust.base_density_per_km3 = 50.0;
    dust.voxel_size_km = 1;
    dust.cluster_noise = 0.0;
    dust.layer_scale_height_km = 0.0;
    dust.radius_min_cm = 10;
    dust.radius_max_cm = 10;

    let fragments = generate_fragments(42, &space, &dust);
    assert!(!fragments.is_empty());

    for frag in fragments {
        assert!(space.contains(frag.pos));
        assert_eq!(frag.profile.radius_cm, 10);
        assert!(frag.profile.radiation_emission_per_tick > 0);
    }
}

#[test]
fn dust_generator_respects_min_fragment_spacing() {
    let space = SpaceConfig {
        width_cm: 200_000,
        depth_cm: 200_000,
        height_cm: 200_000,
    };
    let mut dust = DustConfig::default();
    dust.base_density_per_km3 = 5.0;
    dust.voxel_size_km = 1;
    dust.cluster_noise = 0.0;
    dust.layer_scale_height_km = 0.0;
    dust.radius_min_cm = 10;
    dust.radius_max_cm = 10;
    dust.min_fragment_spacing_cm = 50_000;

    let fragments = generate_fragments(7, &space, &dust);
    assert!(fragments.len() > 1);

    for i in 0..fragments.len() {
        for j in (i + 1)..fragments.len() {
            let a = &fragments[i];
            let b = &fragments[j];
            let dx = a.pos.x_cm - b.pos.x_cm;
            let dy = a.pos.y_cm - b.pos.y_cm;
            let dz = a.pos.z_cm - b.pos.z_cm;
            let min_dist = (a.profile.radius_cm
                + b.profile.radius_cm
                + dust.min_fragment_spacing_cm) as f64;
            assert!((dx * dx + dy * dy + dz * dz) >= (min_dist * min_dist));
        }
    }
}
