use agent_world::{build_world_model, WorldInitConfig, WorldScenario};
use agent_world::simulator::WorldConfig;

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

    println!("scenario: {}", scenario.as_str());
    println!("seed: {}", report.seed);
    println!("locations: {}", report.locations);
    println!("agents: {}", report.agents);
    println!("power_plants: {}", model.power_plants.len());
    println!("power_storages: {}", model.power_storages.len());
}
