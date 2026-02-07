use std::env;
use std::process;

use agent_world::simulator::{
    initialize_kernel, AgentRunner, LlmAgentBehavior, WorldConfig, WorldInitConfig, WorldScenario,
};

#[derive(Debug, Clone, PartialEq)]
struct CliOptions {
    scenario: WorldScenario,
    ticks: u64,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            scenario: WorldScenario::LlmBootstrap,
            ticks: 20,
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

    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(options.scenario, &config);
    let (mut kernel, report) = match initialize_kernel(config, init) {
        Ok(result) => result,
        Err(err) => {
            eprintln!("failed to initialize world: {err:?}");
            process::exit(1);
        }
    };

    let mut runner: AgentRunner<LlmAgentBehavior<_>> = AgentRunner::new();
    let mut agent_ids: Vec<String> = kernel.model().agents.keys().cloned().collect();
    agent_ids.sort();

    if agent_ids.is_empty() {
        eprintln!("no agents in scenario {}", options.scenario.as_str());
        process::exit(1);
    }

    for agent_id in &agent_ids {
        let behavior = match LlmAgentBehavior::from_env(agent_id.clone()) {
            Ok(behavior) => behavior,
            Err(err) => {
                eprintln!("failed to create llm behavior for {agent_id}: {err}");
                process::exit(1);
            }
        };
        runner.register(behavior);
    }

    println!("scenario: {}", options.scenario.as_str());
    println!("seed: {}", report.seed);
    println!("agents: {}", report.agents);
    println!("ticks: {}", options.ticks);

    let mut active_ticks = 0u64;
    for idx in 0..options.ticks {
        match runner.tick(&mut kernel) {
            Some(result) => {
                active_ticks += 1;
                if let Some(action_result) = result.action_result {
                    println!(
                        "tick={} agent={} success={} action={:?}",
                        idx + 1,
                        result.agent_id,
                        action_result.success,
                        action_result.action
                    );
                } else {
                    println!(
                        "tick={} agent={} decision={:?}",
                        idx + 1,
                        result.agent_id,
                        result.decision
                    );
                }
            }
            None => {
                println!("tick={} idle", idx + 1);
                break;
            }
        }
    }

    let metrics = runner.metrics();
    println!("active_ticks: {}", active_ticks);
    println!("total_actions: {}", metrics.total_actions);
    println!("total_decisions: {}", metrics.total_decisions);
    println!("world_time: {}", kernel.time());
    println!("journal_events: {}", kernel.journal().len());
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
            "--ticks" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| "--ticks requires a positive integer".to_string())?;
                options.ticks = raw
                    .parse::<u64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .ok_or_else(|| "--ticks requires a positive integer".to_string())?;
            }
            "--scenario" => {
                scenario_arg = Some(
                    iter.next()
                        .ok_or_else(|| "--scenario requires a scenario name".to_string())?,
                );
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
                "unknown scenario: {name}. available: {}",
                WorldScenario::variants().join(", ")
            )
        })?;
    }

    Ok(options)
}

fn print_help() {
    println!("Usage: world_llm_agent_demo [scenario] [--ticks <n>]");
    println!("Options:");
    println!("  --scenario <name>  Scenario name (default: llm_bootstrap)");
    println!("  --ticks <n>        Max runner ticks (default: 20)");
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
        assert_eq!(options.scenario, WorldScenario::LlmBootstrap);
        assert_eq!(options.ticks, 20);
    }

    #[test]
    fn parse_options_accepts_alias_scenario() {
        let options = parse_options(["llm"].into_iter()).expect("scenario alias");
        assert_eq!(options.scenario, WorldScenario::LlmBootstrap);
    }

    #[test]
    fn parse_options_accepts_ticks() {
        let options = parse_options(["--ticks", "12"].into_iter()).expect("ticks");
        assert_eq!(options.ticks, 12);
    }

    #[test]
    fn parse_options_rejects_zero_ticks() {
        let err = parse_options(["--ticks", "0"].into_iter()).expect_err("reject zero");
        assert!(err.contains("positive integer"));
    }
}
