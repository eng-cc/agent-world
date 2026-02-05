use super::*;

#[test]
fn init_defaults_create_origin_and_agents() {
    let config = WorldConfig::default();
    let mut init = WorldInitConfig::default();
    init.dust.enabled = false;
    init.agents.count = 2;

    let (model, report) = build_world_model(&config, &init).expect("init should succeed");
    let origin = model.locations.get("origin").expect("origin exists");

    let center_x = config.space.width_cm as f64 / 2.0;
    let center_y = config.space.depth_cm as f64 / 2.0;
    let center_z = config.space.height_cm as f64 / 2.0;

    assert_eq!(origin.pos.x_cm, center_x);
    assert_eq!(origin.pos.y_cm, center_y);
    assert_eq!(origin.pos.z_cm, center_z);
    assert_eq!(report.locations, 1);
    assert_eq!(report.agents, 2);
    assert!(model.agents.contains_key("agent-0"));
    assert!(model.agents.contains_key("agent-1"));
}

#[test]
fn init_is_deterministic_with_seed() {
    let mut config = WorldConfig::default();
    config.dust.base_density_per_km3 = 0.5;

    let mut init = WorldInitConfig::default();
    init.seed = 42;
    init.agents.count = 0;

    let (model_a, report_a) = build_world_model(&config, &init).expect("init A");
    let (model_b, report_b) = build_world_model(&config, &init).expect("init B");

    assert_eq!(model_a, model_b);
    assert_eq!(report_a, report_b);
}

#[test]
fn init_requires_spawn_location() {
    let config = WorldConfig::default();
    let mut init = WorldInitConfig::default();
    init.origin.enabled = false;
    init.agents.count = 1;
    init.agents.location_id = None;

    let err = build_world_model(&config, &init).expect_err("should fail");
    assert!(matches!(err, WorldInitError::SpawnLocationMissing));
}

#[test]
fn init_seeds_locations_and_resources() {
    let config = WorldConfig::default();
    let mut init = WorldInitConfig::default();
    init.origin.enabled = false;
    init.dust.enabled = false;
    init.agents.count = 1;
    init.agents.location_id = Some("base".to_string());
    init.agents
        .resources
        .add(ResourceKind::Data, 5)
        .expect("seed agent resources");

    let mut location_seed = LocationSeedConfig::default();
    location_seed.location_id = "base".to_string();
    location_seed.name = "Base".to_string();
    location_seed.pos = Some(pos(10.0, 10.0));
    location_seed
        .resources
        .add(ResourceKind::Hardware, 3)
        .expect("seed location resources");
    init.locations.push(location_seed);

    let (model, _) = build_world_model(&config, &init).expect("init should succeed");
    let base = model.locations.get("base").expect("base exists");
    assert_eq!(base.resources.get(ResourceKind::Hardware), 3);
    let agent = model.agents.get("agent-0").expect("agent exists");
    assert_eq!(agent.resources.get(ResourceKind::Data), 5);
}

#[test]
fn init_rejects_negative_resources() {
    let config = WorldConfig::default();
    let mut init = WorldInitConfig::default();
    init.dust.enabled = false;
    init.agents.count = 0;
    init.origin
        .resources
        .amounts
        .insert(ResourceKind::Electricity, -5);

    let err = build_world_model(&config, &init).expect_err("should fail");
    assert!(matches!(
        err,
        WorldInitError::InvalidResourceAmount {
            kind: ResourceKind::Electricity,
            amount: -5
        }
    ));
}
