use agent_world::{build_world_model, WorldInitConfig, WorldScenario};
use agent_world::simulator::{ResourceKind, WorldConfig};
use std::collections::BTreeMap;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if matches!(args.get(1).map(|s| s.as_str()), Some("--help") | Some("-h")) {
        println!("Usage: world_init_demo [scenario]");
        println!("Available scenarios: {}", WorldScenario::variants().join(", "));
        return;
    }

    let scenario = if let Some(name) = args.get(1) {
        match WorldScenario::parse(name) {
            Some(scenario) => scenario,
            None => {
                eprintln!("Unknown scenario: {name}");
                eprintln!("Available scenarios: {}", WorldScenario::variants().join(", "));
                std::process::exit(1);
            }
        }
    } else {
        WorldScenario::Minimal
    };

    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(scenario, &config);
    let (model, report) = build_world_model(&config, &init).expect("init should succeed");
    let dust_fragments = model.locations.len().saturating_sub(report.locations);

    println!("scenario: {}", scenario.as_str());
    println!("seed: {}", report.seed);
    println!("locations: {}", report.locations);
    println!("agents: {}", report.agents);
    println!("power_plants: {}", model.power_plants.len());
    println!("power_storages: {}", model.power_storages.len());
    println!("dust_fragments: {}", dust_fragments);
    println!("location_resources:");

    let mut location_ids: Vec<_> = model.locations.keys().collect();
    location_ids.sort();
    for location_id in location_ids {
        let location = &model.locations[location_id];
        let electricity = location.resources.get(ResourceKind::Electricity);
        let hardware = location.resources.get(ResourceKind::Hardware);
        let data = location.resources.get(ResourceKind::Data);
        println!(
            "- {}: electricity={} hardware={} data={}",
            location_id, electricity, hardware, data
        );
    }

    let mut facility_counts: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for plant in model.power_plants.values() {
        let entry = facility_counts
            .entry(plant.location_id.clone())
            .or_insert((0, 0));
        entry.0 += 1;
    }
    for storage in model.power_storages.values() {
        let entry = facility_counts
            .entry(storage.location_id.clone())
            .or_insert((0, 0));
        entry.1 += 1;
    }
    println!("location_facilities:");
    let mut facility_locations: Vec<_> = facility_counts.keys().collect();
    facility_locations.sort();
    for location_id in facility_locations {
        let (plants, storages) = facility_counts[location_id];
        println!(
            "- {}: power_plants={} power_storages={}",
            location_id, plants, storages
        );
    }

    println!("agent_resources:");
    let mut agent_ids: Vec<_> = model.agents.keys().collect();
    agent_ids.sort();
    for agent_id in agent_ids {
        let agent = &model.agents[agent_id];
        let electricity = agent.resources.get(ResourceKind::Electricity);
        let hardware = agent.resources.get(ResourceKind::Hardware);
        let data = agent.resources.get(ResourceKind::Data);
        println!(
            "- {}: electricity={} hardware={} data={}",
            agent_id, electricity, hardware, data
        );
    }
}
