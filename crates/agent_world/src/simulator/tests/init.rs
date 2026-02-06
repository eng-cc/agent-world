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

#[test]
fn init_seeds_power_facilities() {
    let config = WorldConfig::default();
    let mut init = WorldInitConfig::default();
    init.dust.enabled = false;
    init.agents.count = 1;

    let plant_seed = PowerPlantSeedConfig {
        facility_id: "plant-1".to_string(),
        location_id: "origin".to_string(),
        owner: ResourceOwner::Agent {
            agent_id: "agent-0".to_string(),
        },
        capacity_per_tick: 5,
        fuel_cost_per_pu: 1,
        maintenance_cost: 1,
        efficiency: 1.0,
        degradation: 0.0,
    };
    init.power_plants.push(plant_seed);

    let storage_seed = PowerStorageSeedConfig {
        facility_id: "storage-1".to_string(),
        location_id: "origin".to_string(),
        owner: ResourceOwner::Location {
            location_id: "origin".to_string(),
        },
        capacity: 10,
        current_level: 3,
        charge_efficiency: 1.0,
        discharge_efficiency: 1.0,
        max_charge_rate: 4,
        max_discharge_rate: 4,
    };
    init.power_storages.push(storage_seed);

    let (model, _) = build_world_model(&config, &init).expect("init should succeed");
    assert!(model.power_plants.contains_key("plant-1"));
    assert!(model.power_storages.contains_key("storage-1"));
}

#[test]
fn init_rejects_facility_with_missing_owner() {
    let config = WorldConfig::default();
    let mut init = WorldInitConfig::default();
    init.dust.enabled = false;
    init.agents.count = 0;

    let plant_seed = PowerPlantSeedConfig {
        facility_id: "plant-1".to_string(),
        location_id: "origin".to_string(),
        owner: ResourceOwner::Agent {
            agent_id: "missing-agent".to_string(),
        },
        capacity_per_tick: 5,
        fuel_cost_per_pu: 1,
        maintenance_cost: 1,
        efficiency: 1.0,
        degradation: 0.0,
    };
    init.power_plants.push(plant_seed);

    let err = build_world_model(&config, &init).expect_err("should fail");
    assert!(matches!(err, WorldInitError::FacilityOwnerNotFound { .. }));
}

#[test]
fn scenario_templates_build_models() {
    let config = WorldConfig::default();
    let scenarios = [
        WorldScenario::Minimal,
        WorldScenario::TwoBases,
        WorldScenario::PowerBootstrap,
        WorldScenario::ResourceBootstrap,
        WorldScenario::TwinRegionBootstrap,
        WorldScenario::TriadRegionBootstrap,
        WorldScenario::TriadP2pBootstrap,
        WorldScenario::DustyBootstrap,
        WorldScenario::DustyTwinRegionBootstrap,
        WorldScenario::DustyTriadRegionBootstrap,
    ];

    for scenario in scenarios {
        let init = WorldInitConfig::from_scenario(scenario, &config);
        let (model, _) = build_world_model(&config, &init).expect("scenario init");
        assert!(!model.locations.is_empty());
    }
}

#[test]
fn resource_bootstrap_seeds_stock() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::ResourceBootstrap, &config);
    let (model, _) = build_world_model(&config, &init).expect("scenario init");
    let origin = model.locations.get("origin").expect("origin exists");
    let agent = model.agents.get("agent-0").expect("agent exists");

    assert_eq!(origin.resources.get(ResourceKind::Electricity), 100);
    assert_eq!(origin.resources.get(ResourceKind::Hardware), 20);
    assert_eq!(agent.resources.get(ResourceKind::Data), 10);
    assert_eq!(agent.resources.get(ResourceKind::Electricity), 25);
}

#[test]
fn twin_region_bootstrap_seeds_regions() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::TwinRegionBootstrap, &config);
    let (model, _) = build_world_model(&config, &init).expect("scenario init");

    assert!(model.locations.contains_key("region-a"));
    assert!(model.locations.contains_key("region-b"));
    assert!(model.power_plants.contains_key("plant-a"));
    assert!(model.power_plants.contains_key("plant-b"));
    assert!(model.power_storages.contains_key("storage-a"));
    assert!(model.agents.contains_key("agent-0"));
    assert!(model.agents.contains_key("agent-1"));
}

#[test]
fn triad_region_bootstrap_seeds_regions() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::TriadRegionBootstrap, &config);
    let (model, _) = build_world_model(&config, &init).expect("scenario init");

    assert!(model.locations.contains_key("region-a"));
    assert!(model.locations.contains_key("region-b"));
    assert!(model.locations.contains_key("region-c"));
    assert!(model.power_plants.contains_key("plant-a"));
    assert!(model.power_plants.contains_key("plant-b"));
    assert!(model.power_storages.contains_key("storage-a"));
    assert!(model.power_storages.contains_key("storage-c"));
    assert!(model.agents.contains_key("agent-0"));
    assert!(model.agents.contains_key("agent-1"));
    assert!(model.agents.contains_key("agent-2"));
}

#[test]
fn triad_p2p_bootstrap_seeds_nodes_and_agents() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::TriadP2pBootstrap, &config);
    let (model, _) = build_world_model(&config, &init).expect("scenario init");

    assert!(model.locations.contains_key("node-a"));
    assert!(model.locations.contains_key("node-b"));
    assert!(model.locations.contains_key("node-c"));

    let agent_a = model.agents.get("agent-0").expect("agent-0 exists");
    let agent_b = model.agents.get("agent-1").expect("agent-1 exists");
    let agent_c = model.agents.get("agent-2").expect("agent-2 exists");

    assert_eq!(agent_a.location_id, "node-a");
    assert_eq!(agent_b.location_id, "node-b");
    assert_eq!(agent_c.location_id, "node-c");
}

#[test]
fn dusty_bootstrap_seeds_dust_and_storage() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::DustyBootstrap, &config);
    let (model, report) = build_world_model(&config, &init).expect("scenario init");

    assert!(report.dust_seed.is_some());
    assert!(model.locations.len() >= 1);
    assert!(model.power_storages.contains_key("storage-1"));
    assert!(model.agents.contains_key("agent-0"));
}

#[test]
fn dusty_twin_region_bootstrap_seeds_dust_and_regions() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::DustyTwinRegionBootstrap, &config);
    let (model, report) = build_world_model(&config, &init).expect("scenario init");

    assert!(report.dust_seed.is_some());
    assert!(model.locations.contains_key("region-a"));
    assert!(model.locations.contains_key("region-b"));
    assert!(model.power_plants.contains_key("plant-a"));
    assert!(model.power_plants.contains_key("plant-b"));
    assert!(model.power_storages.contains_key("storage-a"));
    assert!(model.agents.contains_key("agent-0"));
    assert!(model.agents.contains_key("agent-1"));
}

#[test]
fn dusty_triad_region_bootstrap_seeds_dust_and_regions() {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(WorldScenario::DustyTriadRegionBootstrap, &config);
    let (model, report) = build_world_model(&config, &init).expect("scenario init");

    assert!(report.dust_seed.is_some());
    assert!(model.locations.contains_key("region-a"));
    assert!(model.locations.contains_key("region-b"));
    assert!(model.locations.contains_key("region-c"));
    assert!(model.power_plants.contains_key("plant-a"));
    assert!(model.power_plants.contains_key("plant-b"));
    assert!(model.power_storages.contains_key("storage-a"));
    assert!(model.power_storages.contains_key("storage-c"));
    assert!(model.agents.contains_key("agent-0"));
    assert!(model.agents.contains_key("agent-1"));
    assert!(model.agents.contains_key("agent-2"));
}

#[test]
fn scenario_aliases_parse() {
    let cases = [
        ("two-bases", WorldScenario::TwoBases),
        ("bootstrap", WorldScenario::PowerBootstrap),
        ("resources", WorldScenario::ResourceBootstrap),
        ("twin-regions", WorldScenario::TwinRegionBootstrap),
        ("triad-regions", WorldScenario::TriadRegionBootstrap),
        ("p2p-triad", WorldScenario::TriadP2pBootstrap),
        ("dusty", WorldScenario::DustyBootstrap),
        ("dusty-regions", WorldScenario::DustyTwinRegionBootstrap),
        ("dusty-triad", WorldScenario::DustyTriadRegionBootstrap),
    ];

    for (input, expected) in cases {
        let parsed = WorldScenario::parse(input).expect("parse scenario");
        assert_eq!(parsed, expected);
    }
}

#[test]
fn scenarios_are_stable() {
    struct ScenarioExpectation<'a> {
        scenario: WorldScenario,
        expected_agents: usize,
        expect_origin: bool,
        required_locations: &'a [&'a str],
        required_plants: &'a [&'a str],
        required_storages: &'a [&'a str],
        expect_dust: bool,
    }

    let expectations = [
        ScenarioExpectation {
            scenario: WorldScenario::Minimal,
            expected_agents: 1,
            expect_origin: true,
            required_locations: &["origin"],
            required_plants: &[],
            required_storages: &[],
            expect_dust: false,
        },
        ScenarioExpectation {
            scenario: WorldScenario::TwoBases,
            expected_agents: 2,
            expect_origin: true,
            required_locations: &["origin", "base-a", "base-b"],
            required_plants: &[],
            required_storages: &[],
            expect_dust: false,
        },
        ScenarioExpectation {
            scenario: WorldScenario::PowerBootstrap,
            expected_agents: 1,
            expect_origin: true,
            required_locations: &["origin"],
            required_plants: &["plant-1"],
            required_storages: &["storage-1"],
            expect_dust: false,
        },
        ScenarioExpectation {
            scenario: WorldScenario::ResourceBootstrap,
            expected_agents: 1,
            expect_origin: true,
            required_locations: &["origin"],
            required_plants: &[],
            required_storages: &[],
            expect_dust: false,
        },
        ScenarioExpectation {
            scenario: WorldScenario::TwinRegionBootstrap,
            expected_agents: 2,
            expect_origin: false,
            required_locations: &["region-a", "region-b"],
            required_plants: &["plant-a", "plant-b"],
            required_storages: &["storage-a"],
            expect_dust: false,
        },
        ScenarioExpectation {
            scenario: WorldScenario::TriadRegionBootstrap,
            expected_agents: 3,
            expect_origin: false,
            required_locations: &["region-a", "region-b", "region-c"],
            required_plants: &["plant-a", "plant-b"],
            required_storages: &["storage-a", "storage-c"],
            expect_dust: false,
        },
        ScenarioExpectation {
            scenario: WorldScenario::TriadP2pBootstrap,
            expected_agents: 3,
            expect_origin: false,
            required_locations: &["node-a", "node-b", "node-c"],
            required_plants: &[],
            required_storages: &[],
            expect_dust: false,
        },
        ScenarioExpectation {
            scenario: WorldScenario::DustyBootstrap,
            expected_agents: 1,
            expect_origin: true,
            required_locations: &["origin"],
            required_plants: &[],
            required_storages: &["storage-1"],
            expect_dust: true,
        },
        ScenarioExpectation {
            scenario: WorldScenario::DustyTwinRegionBootstrap,
            expected_agents: 2,
            expect_origin: false,
            required_locations: &["region-a", "region-b"],
            required_plants: &["plant-a", "plant-b"],
            required_storages: &["storage-a"],
            expect_dust: true,
        },
        ScenarioExpectation {
            scenario: WorldScenario::DustyTriadRegionBootstrap,
            expected_agents: 3,
            expect_origin: false,
            required_locations: &["region-a", "region-b", "region-c"],
            required_plants: &["plant-a", "plant-b"],
            required_storages: &["storage-a", "storage-c"],
            expect_dust: true,
        },
    ];

    let config = WorldConfig::default();
    for expectation in expectations {
        let init = WorldInitConfig::from_scenario(expectation.scenario, &config);
        let (model, report) = build_world_model(&config, &init).expect("scenario init");

        assert_eq!(report.agents, expectation.expected_agents);
        assert_eq!(model.agents.len(), expectation.expected_agents);
        assert_eq!(model.locations.contains_key("origin"), expectation.expect_origin);

        for location_id in expectation.required_locations {
            assert!(model.locations.contains_key(*location_id));
        }
        for plant_id in expectation.required_plants {
            assert!(model.power_plants.contains_key(*plant_id));
        }
        for storage_id in expectation.required_storages {
            assert!(model.power_storages.contains_key(*storage_id));
        }

        assert_eq!(report.dust_seed.is_some(), expectation.expect_dust);
    }
}
