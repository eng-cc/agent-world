use std::env;
use std::process;
use std::time::Duration;

use agent_world::viewer::{ViewerLiveServer, ViewerLiveServerConfig};
use agent_world::WorldScenario;

fn main() {
    let mut args = env::args().skip(1);
    let mut scenario: Option<String> = None;
    let mut bind_addr = "127.0.0.1:5010".to_string();
    let mut tick_ms: u64 = 200;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                return;
            }
            "--bind" => {
                bind_addr = match args.next() {
                    Some(addr) => addr,
                    None => {
                        eprintln!("--bind requires an address");
                        process::exit(1);
                    }
                };
            }
            "--tick-ms" => {
                tick_ms = match args.next().and_then(|v| v.parse::<u64>().ok()) {
                    Some(value) => value.max(1),
                    None => {
                        eprintln!("--tick-ms requires a positive integer");
                        process::exit(1);
                    }
                };
            }
            "--scenario" => {
                scenario = match args.next() {
                    Some(name) => Some(name),
                    None => {
                        eprintln!("--scenario requires a name");
                        process::exit(1);
                    }
                };
            }
            _ => {
                if scenario.is_none() {
                    scenario = Some(arg);
                }
            }
        }
    }

    let scenario = match scenario {
        Some(name) => match WorldScenario::parse(&name) {
            Some(scenario) => scenario,
            None => {
                eprintln!("Unknown scenario: {name}");
                eprintln!("Available scenarios: {}", WorldScenario::variants().join(", "));
                process::exit(1);
            }
        },
        None => WorldScenario::TwinRegionBootstrap,
    };

    let config = ViewerLiveServerConfig::new(scenario)
        .with_bind_addr(bind_addr)
        .with_tick_interval(Duration::from_millis(tick_ms));

    let mut server = match ViewerLiveServer::new(config) {
        Ok(server) => server,
        Err(err) => {
            eprintln!("failed to start live viewer server: {err:?}");
            process::exit(1);
        }
    };

    if let Err(err) = server.run() {
        eprintln!("live viewer server failed: {err:?}");
        process::exit(1);
    }
}

fn print_help() {
    println!("Usage: world_viewer_live [scenario] [--bind <addr>] [--tick-ms <ms>]");
    println!("Options:");
    println!("  --bind <addr>     Bind address (default: 127.0.0.1:5010)");
    println!("  --tick-ms <ms>    Tick interval in milliseconds (default: 200)");
    println!("  --scenario <name> Scenario name (default: twin_region_bootstrap)");
    println!("Available scenarios: {}", WorldScenario::variants().join(", "));
}
