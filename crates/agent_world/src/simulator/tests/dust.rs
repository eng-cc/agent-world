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
