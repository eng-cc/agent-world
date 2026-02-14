use std::env;
use std::process;
use std::time::Duration;

use agent_world::simulator::WorldScenario;
use agent_world::viewer::{ViewerLiveDecisionMode, ViewerLiveServer, ViewerLiveServerConfig};

#[derive(Debug, Clone, PartialEq)]
struct CliOptions {
    scenario: WorldScenario,
    bind_addr: String,
    tick_ms: u64,
    llm_mode: bool,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            scenario: WorldScenario::TwinRegionBootstrap,
            bind_addr: "127.0.0.1:5010".to_string(),
            tick_ms: 200,
            llm_mode: false,
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
            process::exit(1);
        }
    };

    if let Err(err) = server.run() {
        eprintln!("live viewer server failed: {err:?}");
        process::exit(1);
    }
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
            "--scenario" => {
                scenario_arg = Some(
                    iter.next()
                        .ok_or_else(|| "--scenario requires a name".to_string())?,
                );
            }
            "--llm" => {
                options.llm_mode = true;
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
    println!("Usage: world_viewer_live [scenario] [--bind <addr>] [--tick-ms <ms>] [--llm]");
    println!("Options:");
    println!("  --bind <addr>     Bind address (default: 127.0.0.1:5010)");
    println!("  --tick-ms <ms>    Tick interval in milliseconds (default: 200)");
    println!("  --scenario <name> Scenario name (default: twin_region_bootstrap)");
    println!("  --llm             Use LLM decisions instead of built-in script");
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
        assert_eq!(options.tick_ms, 200);
        assert!(!options.llm_mode);
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
                "--tick-ms",
                "50",
            ]
            .into_iter(),
        )
        .expect("custom");
        assert_eq!(options.scenario, WorldScenario::LlmBootstrap);
        assert_eq!(options.bind_addr, "127.0.0.1:9001");
        assert_eq!(options.tick_ms, 50);
    }

    #[test]
    fn parse_options_rejects_zero_tick_ms() {
        let err = parse_options(["--tick-ms", "0"].into_iter()).expect_err("reject zero");
        assert!(err.contains("positive integer"));
    }
}
