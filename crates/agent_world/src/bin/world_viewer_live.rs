use std::env;
use std::process;
use std::thread;
use std::time::Duration;

use agent_world::simulator::WorldScenario;
use agent_world::viewer::{
    ViewerLiveDecisionMode, ViewerLiveServer, ViewerLiveServerConfig, ViewerWebBridge,
    ViewerWebBridgeConfig,
};
use node::{NodeConfig, NodeRole, NodeRuntime};

#[derive(Debug, Clone, PartialEq)]
struct CliOptions {
    scenario: WorldScenario,
    bind_addr: String,
    web_bind_addr: Option<String>,
    tick_ms: u64,
    llm_mode: bool,
    node_enabled: bool,
    node_id: String,
    node_role: NodeRole,
    node_tick_ms: u64,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            scenario: WorldScenario::TwinRegionBootstrap,
            bind_addr: "127.0.0.1:5010".to_string(),
            web_bind_addr: None,
            tick_ms: 200,
            llm_mode: false,
            node_enabled: true,
            node_id: "viewer-live-node".to_string(),
            node_role: NodeRole::Observer,
            node_tick_ms: 200,
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let options = match parse_options(args.iter().skip(1).map(|arg| arg.as_str())) {
        Ok(options) => options,
        Err(err) => {
            eprintln!("{err}");
            print_help();
            process::exit(1);
        }
    };

    let mut node_runtime = match start_live_node(&options) {
        Ok(runtime) => runtime,
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    if let Some(web_bind_addr) = options.web_bind_addr.clone() {
        let upstream_addr = options.bind_addr.clone();
        thread::spawn(move || {
            let bridge = ViewerWebBridge::new(ViewerWebBridgeConfig::new(
                web_bind_addr.clone(),
                upstream_addr,
            ));
            if let Err(err) = bridge.run() {
                eprintln!("viewer web bridge failed on {}: {err:?}", web_bind_addr);
            }
        });
    }

    let config = ViewerLiveServerConfig::new(options.scenario)
        .with_bind_addr(options.bind_addr)
        .with_tick_interval(Duration::from_millis(options.tick_ms))
        .with_decision_mode(if options.llm_mode {
            ViewerLiveDecisionMode::Llm
        } else {
            ViewerLiveDecisionMode::Script
        });

    let mut server = match ViewerLiveServer::new(config) {
        Ok(server) => server,
        Err(err) => {
            eprintln!("failed to start live viewer server: {err:?}");
            if let Some(runtime) = node_runtime.as_mut() {
                if let Err(stop_err) = runtime.stop() {
                    eprintln!("failed to stop node runtime: {stop_err:?}");
                }
            }
            process::exit(1);
        }
    };

    let run_result = server.run();
    if let Some(runtime) = node_runtime.as_mut() {
        if let Err(stop_err) = runtime.stop() {
            eprintln!("failed to stop node runtime: {stop_err:?}");
        }
    }

    if let Err(err) = run_result {
        eprintln!("live viewer server failed: {err:?}");
        process::exit(1);
    }
}

fn start_live_node(options: &CliOptions) -> Result<Option<NodeRuntime>, String> {
    if !options.node_enabled {
        return Ok(None);
    }

    let world_id = format!("live-{}", options.scenario.as_str());
    let config = NodeConfig::new(options.node_id.clone(), world_id, options.node_role)
        .and_then(|config| config.with_tick_interval(Duration::from_millis(options.node_tick_ms)))
        .map_err(|err| format!("failed to build node config: {err:?}"))?;

    let mut runtime = NodeRuntime::new(config);
    runtime
        .start()
        .map_err(|err| format!("failed to start node runtime: {err:?}"))?;
    Ok(Some(runtime))
}

fn parse_options<'a>(args: impl Iterator<Item = &'a str>) -> Result<CliOptions, String> {
    let mut options = CliOptions::default();
    let mut scenario_arg: Option<&str> = None;
    let mut iter = args.peekable();

    while let Some(arg) = iter.next() {
        match arg {
            "--help" | "-h" => {
                print_help();
                process::exit(0);
            }
            "--bind" => {
                options.bind_addr = iter
                    .next()
                    .ok_or_else(|| "--bind requires an address".to_string())?
                    .to_string();
            }
            "--tick-ms" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| "--tick-ms requires a positive integer".to_string())?;
                options.tick_ms = raw
                    .parse::<u64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or_else(|| "--tick-ms requires a positive integer".to_string())?;
            }
            "--web-bind" => {
                options.web_bind_addr = Some(
                    iter.next()
                        .ok_or_else(|| "--web-bind requires an address".to_string())?
                        .to_string(),
                );
            }
            "--scenario" => {
                scenario_arg = Some(
                    iter.next()
                        .ok_or_else(|| "--scenario requires a name".to_string())?,
                );
            }
            "--llm" => {
                options.llm_mode = true;
            }
            "--no-node" => {
                options.node_enabled = false;
            }
            "--node-id" => {
                options.node_id = iter
                    .next()
                    .ok_or_else(|| "--node-id requires a value".to_string())?
                    .to_string();
            }
            "--node-role" => {
                let role = iter
                    .next()
                    .ok_or_else(|| "--node-role requires a value".to_string())?;
                options.node_role = role.parse::<NodeRole>().map_err(|_| {
                    "--node-role must be one of: sequencer, storage, observer".to_string()
                })?;
            }
            "--node-tick-ms" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| "--node-tick-ms requires a positive integer".to_string())?;
                options.node_tick_ms = raw
                    .parse::<u64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or_else(|| "--node-tick-ms requires a positive integer".to_string())?;
            }
            _ => {
                if scenario_arg.is_none() {
                    scenario_arg = Some(arg);
                } else {
                    return Err(format!("unexpected argument: {arg}"));
                }
            }
        }
    }

    if let Some(name) = scenario_arg {
        options.scenario = WorldScenario::parse(name).ok_or_else(|| {
            format!(
                "Unknown scenario: {name}. available: {}",
                WorldScenario::variants().join(", ")
            )
        })?;
    }

    Ok(options)
}

fn print_help() {
    println!(
        "Usage: world_viewer_live [scenario] [--bind <addr>] [--web-bind <addr>] [--tick-ms <ms>] [--llm] [--no-node]"
    );
    println!("Options:");
    println!("  --bind <addr>     Bind address (default: 127.0.0.1:5010)");
    println!("  --web-bind <addr> WebSocket bridge bind address (optional)");
    println!("  --tick-ms <ms>    Tick interval in milliseconds (default: 200)");
    println!("  --scenario <name> Scenario name (default: twin_region_bootstrap)");
    println!("  --llm             Use LLM decisions instead of built-in script");
    println!("  --no-node         Disable embedded node runtime startup");
    println!("  --node-id <id>    Node identifier (default: viewer-live-node)");
    println!("  --node-role <r>   Node role: sequencer|storage|observer (default: observer)");
    println!("  --node-tick-ms <ms> Node runtime tick interval (default: 200)");
    println!(
        "Available scenarios: {}",
        WorldScenario::variants().join(", ")
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_options_defaults() {
        let options = parse_options([].into_iter()).expect("defaults");
        assert_eq!(options.scenario, WorldScenario::TwinRegionBootstrap);
        assert_eq!(options.bind_addr, "127.0.0.1:5010");
        assert!(options.web_bind_addr.is_none());
        assert_eq!(options.tick_ms, 200);
        assert!(!options.llm_mode);
        assert!(options.node_enabled);
        assert_eq!(options.node_id, "viewer-live-node");
        assert_eq!(options.node_role, NodeRole::Observer);
        assert_eq!(options.node_tick_ms, 200);
    }

    #[test]
    fn parse_options_enables_llm_mode() {
        let options = parse_options(["--llm"].into_iter()).expect("llm mode");
        assert!(options.llm_mode);
    }

    #[test]
    fn parse_options_reads_custom_values() {
        let options = parse_options(
            [
                "llm_bootstrap",
                "--bind",
                "127.0.0.1:9001",
                "--web-bind",
                "127.0.0.1:9002",
                "--tick-ms",
                "50",
                "--node-id",
                "viewer-live-1",
                "--node-role",
                "storage",
                "--node-tick-ms",
                "30",
            ]
            .into_iter(),
        )
        .expect("custom");
        assert_eq!(options.scenario, WorldScenario::LlmBootstrap);
        assert_eq!(options.bind_addr, "127.0.0.1:9001");
        assert_eq!(options.web_bind_addr.as_deref(), Some("127.0.0.1:9002"));
        assert_eq!(options.tick_ms, 50);
        assert_eq!(options.node_id, "viewer-live-1");
        assert_eq!(options.node_role, NodeRole::Storage);
        assert_eq!(options.node_tick_ms, 30);
    }

    #[test]
    fn parse_options_rejects_zero_tick_ms() {
        let err = parse_options(["--tick-ms", "0"].into_iter()).expect_err("reject zero");
        assert!(err.contains("positive integer"));
    }

    #[test]
    fn parse_options_disables_node() {
        let options = parse_options(["--no-node"].into_iter()).expect("parse");
        assert!(!options.node_enabled);
    }

    #[test]
    fn parse_options_rejects_invalid_node_role() {
        let err = parse_options(["--node-role", "unknown"].into_iter())
            .expect_err("invalid node role");
        assert!(err.contains("--node-role"));
    }
}
