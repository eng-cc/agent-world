use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process;

use agent_world::simulator::{
    initialize_kernel, ActionResult, AgentDecision, AgentDecisionTrace, AgentRunner,
    LlmAgentBehavior, RejectReason, WorldConfig, WorldInitConfig, WorldScenario,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq)]
struct CliOptions {
    scenario: WorldScenario,
    ticks: u64,
    report_json: Option<String>,
    print_llm_io: bool,
    llm_io_max_chars: Option<usize>,
    llm_system_prompt: Option<String>,
    llm_short_term_goal: Option<String>,
    llm_long_term_goal: Option<String>,
    prompt_switch_tick: Option<u64>,
    switch_llm_system_prompt: Option<String>,
    switch_llm_short_term_goal: Option<String>,
    switch_llm_long_term_goal: Option<String>,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            scenario: WorldScenario::LlmBootstrap,
            ticks: 20,
            report_json: None,
            print_llm_io: false,
            llm_io_max_chars: None,
            llm_system_prompt: None,
            llm_short_term_goal: None,
            llm_long_term_goal: None,
            prompt_switch_tick: None,
            switch_llm_system_prompt: None,
            switch_llm_short_term_goal: None,
            switch_llm_long_term_goal: None,
        }
    }
}

impl CliOptions {
    fn has_initial_prompt_override(&self) -> bool {
        self.llm_system_prompt.is_some()
            || self.llm_short_term_goal.is_some()
            || self.llm_long_term_goal.is_some()
    }

    fn has_switch_prompt_override(&self) -> bool {
        self.switch_llm_system_prompt.is_some()
            || self.switch_llm_short_term_goal.is_some()
            || self.switch_llm_long_term_goal.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
struct DecisionCounts {
    wait: u64,
    wait_ticks: u64,
    act: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
struct TraceCounts {
    traces: u64,
    llm_errors: u64,
    parse_errors: u64,
    repair_rounds_total: u64,
    repair_rounds_max: u32,
    llm_input_chars_total: u64,
    llm_input_chars_avg: u64,
    llm_input_chars_max: usize,
    step_entries: u64,
    prompt_section_entries: u64,
    prompt_section_clipped: u64,
    step_type_counts: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct DemoRunReport {
    scenario: String,
    ticks_requested: u64,
    active_ticks: u64,
    total_actions: u64,
    total_decisions: u64,
    action_success: u64,
    action_failure: u64,
    action_reject_reason_counts: BTreeMap<String, u64>,
    action_kind_counts: BTreeMap<String, u64>,
    action_kind_success_counts: BTreeMap<String, u64>,
    action_kind_failure_counts: BTreeMap<String, u64>,
    first_action_tick: BTreeMap<String, u64>,
    decision_counts: DecisionCounts,
    trace_counts: TraceCounts,
    world_time: u64,
    journal_events: usize,
}

impl DemoRunReport {
    fn new(scenario: String, ticks_requested: u64) -> Self {
        Self {
            scenario,
            ticks_requested,
            active_ticks: 0,
            total_actions: 0,
            total_decisions: 0,
            action_success: 0,
            action_failure: 0,
            action_reject_reason_counts: BTreeMap::new(),
            action_kind_counts: BTreeMap::new(),
            action_kind_success_counts: BTreeMap::new(),
            action_kind_failure_counts: BTreeMap::new(),
            first_action_tick: BTreeMap::new(),
            decision_counts: DecisionCounts::default(),
            trace_counts: TraceCounts::default(),
            world_time: 0,
            journal_events: 0,
        }
    }

    fn observe_decision(&mut self, decision: &AgentDecision) {
        match decision {
            AgentDecision::Wait => {
                self.decision_counts.wait += 1;
            }
            AgentDecision::WaitTicks(_) => {
                self.decision_counts.wait_ticks += 1;
            }
            AgentDecision::Act(_) => {
                self.decision_counts.act += 1;
            }
        }
    }

    fn observe_trace(&mut self, trace: &AgentDecisionTrace) {
        self.trace_counts.traces += 1;

        if trace.llm_error.is_some() {
            self.trace_counts.llm_errors += 1;
        }
        if trace.parse_error.is_some() {
            self.trace_counts.parse_errors += 1;
        }

        let retry_count = trace
            .llm_diagnostics
            .as_ref()
            .map(|diagnostics| diagnostics.retry_count)
            .unwrap_or(0);
        self.trace_counts.repair_rounds_total += retry_count as u64;
        self.trace_counts.repair_rounds_max = self.trace_counts.repair_rounds_max.max(retry_count);

        if let Some(input) = trace.llm_input.as_ref() {
            let chars = input.chars().count();
            self.trace_counts.llm_input_chars_total += chars as u64;
            self.trace_counts.llm_input_chars_max =
                self.trace_counts.llm_input_chars_max.max(chars);
        }

        self.trace_counts.step_entries += trace.llm_step_trace.len() as u64;
        self.trace_counts.prompt_section_entries += trace.llm_prompt_section_trace.len() as u64;

        for step in &trace.llm_step_trace {
            *self
                .trace_counts
                .step_type_counts
                .entry(step.step_type.clone())
                .or_insert(0) += 1;
        }

        for section in &trace.llm_prompt_section_trace {
            if !section.included || section.emitted_tokens < section.estimated_tokens {
                self.trace_counts.prompt_section_clipped += 1;
            }
        }
    }

    fn observe_action_result(&mut self, tick: u64, action_result: &ActionResult) {
        let action_kind = action_metric_key(&action_result.action);
        *self
            .action_kind_counts
            .entry(action_kind.clone())
            .or_insert(0) += 1;
        self.first_action_tick
            .entry(action_kind.clone())
            .or_insert(tick);

        if action_result.success {
            self.action_success += 1;
            *self
                .action_kind_success_counts
                .entry(action_kind)
                .or_insert(0) += 1;
            return;
        }
        self.action_failure += 1;
        *self
            .action_kind_failure_counts
            .entry(action_kind)
            .or_insert(0) += 1;
        if let Some(reason) = action_result.reject_reason() {
            let key = reject_reason_metric_key(reason);
            *self.action_reject_reason_counts.entry(key).or_insert(0) += 1;
        }
    }

    fn finalize(&mut self) {
        if self.trace_counts.traces > 0 {
            self.trace_counts.llm_input_chars_avg =
                self.trace_counts.llm_input_chars_total / self.trace_counts.traces;
        }
    }
}

fn truncate_for_llm_io_log(text: &str, max_chars: Option<usize>) -> String {
    let Some(max_chars) = max_chars else {
        return text.to_string();
    };
    if max_chars == 0 {
        return text.to_string();
    }

    let total_chars = text.chars().count();
    if total_chars <= max_chars {
        return text.to_string();
    }

    let mut truncated = String::new();
    for (index, ch) in text.chars().enumerate() {
        if index >= max_chars {
            break;
        }
        truncated.push(ch);
    }
    truncated.push_str(&format!(
        "\n...(truncated, total_chars={total_chars}, max_chars={max_chars})"
    ));
    truncated
}

fn print_llm_io_trace(
    tick: u64,
    agent_id: &str,
    trace: &AgentDecisionTrace,
    llm_io_max_chars: Option<usize>,
) {
    println!("tick={} agent={} llm_io_begin", tick, agent_id);

    if let Some(input) = trace.llm_input.as_ref() {
        println!("tick={} agent={} llm_input_begin", tick, agent_id);
        println!("{}", truncate_for_llm_io_log(input, llm_io_max_chars));
        println!("tick={} agent={} llm_input_end", tick, agent_id);
    } else {
        println!("tick={} agent={} llm_input=<none>", tick, agent_id);
    }

    if let Some(output) = trace.llm_output.as_ref() {
        println!("tick={} agent={} llm_output_begin", tick, agent_id);
        println!("{}", truncate_for_llm_io_log(output, llm_io_max_chars));
        println!("tick={} agent={} llm_output_end", tick, agent_id);
    } else {
        println!("tick={} agent={} llm_output=<none>", tick, agent_id);
    }

    if let Some(error) = trace.llm_error.as_ref() {
        println!("tick={} agent={} llm_error={}", tick, agent_id, error);
    }
    if let Some(parse_error) = trace.parse_error.as_ref() {
        println!(
            "tick={} agent={} parse_error={}",
            tick, agent_id, parse_error
        );
    }

    println!("tick={} agent={} llm_io_end", tick, agent_id);
}

fn reject_reason_metric_key(reason: &RejectReason) -> String {
    serde_json::to_value(reason)
        .ok()
        .and_then(|value| {
            value
                .get("type")
                .and_then(|inner| inner.as_str())
                .map(normalize_reason_metric_key)
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn action_metric_key(action: &agent_world::simulator::Action) -> String {
    serde_json::to_value(action)
        .ok()
        .and_then(|value| {
            value
                .get("type")
                .and_then(|inner| inner.as_str())
                .map(normalize_reason_metric_key)
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn normalize_reason_metric_key(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "unknown".to_string();
    }

    if trimmed
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    {
        return trimmed.to_string();
    }

    let mut normalized = String::with_capacity(trimmed.len() + 8);
    for (index, ch) in trimmed.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index > 0 {
                normalized.push('_');
            }
            normalized.push(ch.to_ascii_lowercase());
        } else if ch == '-' || ch == ' ' {
            normalized.push('_');
        } else {
            normalized.push(ch.to_ascii_lowercase());
        }
    }
    normalized
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
        let mut behavior = match LlmAgentBehavior::from_env(agent_id.clone()) {
            Ok(behavior) => behavior,
            Err(err) => {
                eprintln!("failed to create llm behavior for {agent_id}: {err}");
                process::exit(1);
            }
        };
        if options.has_initial_prompt_override() {
            behavior.apply_prompt_overrides(
                options.llm_system_prompt.clone(),
                options.llm_short_term_goal.clone(),
                options.llm_long_term_goal.clone(),
            );
        }
        runner.register(behavior);
    }

    println!("scenario: {}", options.scenario.as_str());
    println!("seed: {}", report.seed);
    println!("agents: {}", report.agents);
    println!("ticks: {}", options.ticks);

    let mut run_report = DemoRunReport::new(options.scenario.as_str().to_string(), options.ticks);
    let mut prompt_switch_applied = false;

    for idx in 0..options.ticks {
        let tick = idx + 1;
        if !prompt_switch_applied {
            if let Some(switch_tick) = options.prompt_switch_tick {
                if tick >= switch_tick {
                    for agent_id in runner.agent_ids() {
                        if let Some(agent) = runner.get_mut(agent_id.as_str()) {
                            let current = agent.behavior.prompt_overrides();
                            agent.behavior.apply_prompt_overrides(
                                options
                                    .switch_llm_system_prompt
                                    .clone()
                                    .or(current.system_prompt),
                                options
                                    .switch_llm_short_term_goal
                                    .clone()
                                    .or(current.short_term_goal),
                                options
                                    .switch_llm_long_term_goal
                                    .clone()
                                    .or(current.long_term_goal),
                            );
                        }
                    }
                    prompt_switch_applied = true;
                    println!("tick={} prompt_switch_applied=true", tick);
                }
            }
        }

        match runner.tick(&mut kernel) {
            Some(result) => {
                run_report.active_ticks += 1;
                run_report.observe_decision(&result.decision);

                if let Some(trace) = result.decision_trace.as_ref() {
                    run_report.observe_trace(trace);
                    if options.print_llm_io {
                        print_llm_io_trace(
                            tick,
                            result.agent_id.as_str(),
                            trace,
                            options.llm_io_max_chars,
                        );
                    }
                }

                if let Some(action_result) = result.action_result.as_ref() {
                    run_report.observe_action_result(idx + 1, action_result);
                    println!(
                        "tick={} agent={} success={} action={:?}",
                        tick, result.agent_id, action_result.success, action_result.action
                    );
                    if let Some(reason) = action_result.reject_reason() {
                        println!(
                            "tick={} agent={} reject_reason={:?}",
                            tick, result.agent_id, reason
                        );
                    }
                } else {
                    println!(
                        "tick={} agent={} decision={:?}",
                        tick, result.agent_id, result.decision
                    );
                }
            }
            None => {
                println!("tick={} idle", tick);
                break;
            }
        }
    }

    let metrics = runner.metrics();
    run_report.total_actions = metrics.total_actions;
    run_report.total_decisions = metrics.total_decisions;
    run_report.world_time = kernel.time();
    run_report.journal_events = kernel.journal().len();
    run_report.finalize();

    if let Some(path) = options.report_json.as_ref() {
        if let Err(err) = write_report_json(path, &run_report) {
            eprintln!("failed to write report json: {err}");
            process::exit(1);
        }
        println!("report_json: {path}");
    }

    println!("active_ticks: {}", run_report.active_ticks);
    println!("total_actions: {}", run_report.total_actions);
    println!("total_decisions: {}", run_report.total_decisions);
    println!("world_time: {}", run_report.world_time);
    println!("journal_events: {}", run_report.journal_events);
    println!("action_success: {}", run_report.action_success);
    println!("action_failure: {}", run_report.action_failure);
    if !run_report.action_reject_reason_counts.is_empty() {
        for (reason, count) in &run_report.action_reject_reason_counts {
            println!("action_reject_reason_{}: {}", reason, count);
        }
    }
    if !run_report.action_kind_counts.is_empty() {
        for (kind, count) in &run_report.action_kind_counts {
            println!("action_kind_{}: {}", kind, count);
        }
    }
    if !run_report.first_action_tick.is_empty() {
        for (kind, tick) in &run_report.first_action_tick {
            println!("first_action_tick_{}: {}", kind, tick);
        }
    }
    println!("decision_wait: {}", run_report.decision_counts.wait);
    println!(
        "decision_wait_ticks: {}",
        run_report.decision_counts.wait_ticks
    );
    println!("decision_act: {}", run_report.decision_counts.act);
    println!("trace_count: {}", run_report.trace_counts.traces);
    println!("llm_errors: {}", run_report.trace_counts.llm_errors);
    println!("parse_errors: {}", run_report.trace_counts.parse_errors);
    println!(
        "repair_rounds_total: {}",
        run_report.trace_counts.repair_rounds_total
    );
    println!(
        "repair_rounds_max: {}",
        run_report.trace_counts.repair_rounds_max
    );
    println!(
        "llm_input_chars_avg: {}",
        run_report.trace_counts.llm_input_chars_avg
    );
    println!(
        "llm_input_chars_max: {}",
        run_report.trace_counts.llm_input_chars_max
    );
}

fn write_report_json(path: &str, run_report: &DemoRunReport) -> Result<(), String> {
    let report_path = Path::new(path);
    if let Some(parent) = report_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| {
                format!(
                    "failed to create report directory {}: {err}",
                    parent.display()
                )
            })?;
        }
    }

    let content = serde_json::to_string_pretty(run_report)
        .map_err(|err| format!("failed to serialize report json: {err}"))?;
    fs::write(report_path, format!("{content}\n")).map_err(|err| {
        format!(
            "failed to write report file {}: {err}",
            report_path.display()
        )
    })
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
            "--report-json" => {
                options.report_json = Some(
                    iter.next()
                        .ok_or_else(|| "--report-json requires a file path".to_string())?
                        .to_string(),
                );
            }
            "--print-llm-io" => {
                options.print_llm_io = true;
            }
            "--llm-io-max-chars" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| "--llm-io-max-chars requires a positive integer".to_string())?;
                options.llm_io_max_chars = Some(
                    raw.parse::<usize>()
                        .ok()
                        .filter(|value| *value > 0)
                        .ok_or_else(|| {
                            "--llm-io-max-chars requires a positive integer".to_string()
                        })?,
                );
            }
            "--llm-system-prompt" => {
                options.llm_system_prompt = Some(
                    iter.next()
                        .ok_or_else(|| "--llm-system-prompt requires prompt text".to_string())?
                        .to_string(),
                );
            }
            "--llm-short-term-goal" => {
                options.llm_short_term_goal = Some(
                    iter.next()
                        .ok_or_else(|| "--llm-short-term-goal requires goal text".to_string())?
                        .to_string(),
                );
            }
            "--llm-long-term-goal" => {
                options.llm_long_term_goal = Some(
                    iter.next()
                        .ok_or_else(|| "--llm-long-term-goal requires goal text".to_string())?
                        .to_string(),
                );
            }
            "--prompt-switch-tick" => {
                let raw = iter.next().ok_or_else(|| {
                    "--prompt-switch-tick requires a positive integer".to_string()
                })?;
                options.prompt_switch_tick = Some(
                    raw.parse::<u64>()
                        .ok()
                        .filter(|value| *value > 0)
                        .ok_or_else(|| {
                            "--prompt-switch-tick requires a positive integer".to_string()
                        })?,
                );
            }
            "--switch-llm-system-prompt" => {
                options.switch_llm_system_prompt = Some(
                    iter.next()
                        .ok_or_else(|| {
                            "--switch-llm-system-prompt requires prompt text".to_string()
                        })?
                        .to_string(),
                );
            }
            "--switch-llm-short-term-goal" => {
                options.switch_llm_short_term_goal = Some(
                    iter.next()
                        .ok_or_else(|| {
                            "--switch-llm-short-term-goal requires goal text".to_string()
                        })?
                        .to_string(),
                );
            }
            "--switch-llm-long-term-goal" => {
                options.switch_llm_long_term_goal = Some(
                    iter.next()
                        .ok_or_else(|| {
                            "--switch-llm-long-term-goal requires goal text".to_string()
                        })?
                        .to_string(),
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

    if options.has_switch_prompt_override() && options.prompt_switch_tick.is_none() {
        return Err(
            "--prompt-switch-tick is required when switch prompt overrides are set".to_string(),
        );
    }
    if options.prompt_switch_tick.is_some() && !options.has_switch_prompt_override() {
        return Err(
            "--prompt-switch-tick requires at least one --switch-llm-* override".to_string(),
        );
    }

    Ok(options)
}

fn print_help() {
    println!(
        "Usage: world_llm_agent_demo [scenario] [--ticks <n>] [--report-json <path>] [--print-llm-io] [--llm-io-max-chars <n>] [prompt overrides]"
    );
    println!("Options:");
    println!("  --scenario <name>  Scenario name (default: llm_bootstrap)");
    println!("  --ticks <n>        Max runner ticks (default: 20)");
    println!("  --report-json <path>  Persist run summary as JSON report");
    println!("  --print-llm-io     Print LLM input/output to stdout for each tick");
    println!("  --llm-io-max-chars <n>  Truncate each LLM input/output block to n chars");
    println!("  --llm-system-prompt <text>  Override default system prompt for this run");
    println!("  --llm-short-term-goal <text>  Override default short-term goal for this run");
    println!("  --llm-long-term-goal <text>  Override default long-term goal for this run");
    println!("  --prompt-switch-tick <n>  Apply switch prompt overrides at tick n (1-based)");
    println!("  --switch-llm-system-prompt <text>  System prompt used after --prompt-switch-tick");
    println!(
        "  --switch-llm-short-term-goal <text>  Short-term goal used after --prompt-switch-tick"
    );
    println!(
        "  --switch-llm-long-term-goal <text>  Long-term goal used after --prompt-switch-tick"
    );
    println!(
        "Available scenarios: {}",
        WorldScenario::variants().join(", ")
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_world::simulator::{Action, WorldEvent, WorldEventKind};

    #[test]
    fn parse_options_defaults() {
        let options = parse_options([].into_iter()).expect("defaults");
        assert_eq!(options.scenario, WorldScenario::LlmBootstrap);
        assert_eq!(options.ticks, 20);
        assert!(!options.print_llm_io);
        assert_eq!(options.llm_io_max_chars, None);
        assert_eq!(options.llm_system_prompt, None);
        assert_eq!(options.llm_short_term_goal, None);
        assert_eq!(options.llm_long_term_goal, None);
        assert_eq!(options.prompt_switch_tick, None);
        assert_eq!(options.switch_llm_system_prompt, None);
        assert_eq!(options.switch_llm_short_term_goal, None);
        assert_eq!(options.switch_llm_long_term_goal, None);
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
    fn parse_options_accepts_report_json_path() {
        let options = parse_options(["--report-json", ".tmp/report.json"].into_iter())
            .expect("report json path");
        assert_eq!(options.report_json.as_deref(), Some(".tmp/report.json"));
    }

    #[test]
    fn parse_options_enables_print_llm_io() {
        let options = parse_options(["--print-llm-io"].into_iter()).expect("llm io option");
        assert!(options.print_llm_io);
    }

    #[test]
    fn parse_options_accepts_llm_io_max_chars() {
        let options = parse_options(["--llm-io-max-chars", "256"].into_iter())
            .expect("llm io max chars option");
        assert_eq!(options.llm_io_max_chars, Some(256));
    }

    #[test]
    fn parse_options_accepts_initial_prompt_overrides() {
        let options = parse_options(
            [
                "--llm-system-prompt",
                "sys",
                "--llm-short-term-goal",
                "short",
                "--llm-long-term-goal",
                "long",
            ]
            .into_iter(),
        )
        .expect("prompt overrides");
        assert_eq!(options.llm_system_prompt.as_deref(), Some("sys"));
        assert_eq!(options.llm_short_term_goal.as_deref(), Some("short"));
        assert_eq!(options.llm_long_term_goal.as_deref(), Some("long"));
    }

    #[test]
    fn parse_options_accepts_switch_prompt_overrides() {
        let options = parse_options(
            [
                "--prompt-switch-tick",
                "9",
                "--switch-llm-system-prompt",
                "sys2",
                "--switch-llm-short-term-goal",
                "short2",
                "--switch-llm-long-term-goal",
                "long2",
            ]
            .into_iter(),
        )
        .expect("switch prompt overrides");
        assert_eq!(options.prompt_switch_tick, Some(9));
        assert_eq!(options.switch_llm_system_prompt.as_deref(), Some("sys2"));
        assert_eq!(
            options.switch_llm_short_term_goal.as_deref(),
            Some("short2")
        );
        assert_eq!(options.switch_llm_long_term_goal.as_deref(), Some("long2"));
    }

    #[test]
    fn parse_options_rejects_missing_report_json_path() {
        let err = parse_options(["--report-json"].into_iter()).expect_err("missing report path");
        assert!(err.contains("file path"));
    }

    #[test]
    fn parse_options_rejects_zero_ticks() {
        let err = parse_options(["--ticks", "0"].into_iter()).expect_err("reject zero");
        assert!(err.contains("positive integer"));
    }

    #[test]
    fn parse_options_rejects_invalid_llm_io_max_chars() {
        let err = parse_options(["--llm-io-max-chars", "0"].into_iter())
            .expect_err("reject zero llm io max chars");
        assert!(err.contains("positive integer"));
    }

    #[test]
    fn parse_options_rejects_switch_prompt_without_tick() {
        let err = parse_options(["--switch-llm-system-prompt", "sys2"].into_iter())
            .expect_err("switch prompt without tick");
        assert!(err.contains("--prompt-switch-tick"));
    }

    #[test]
    fn parse_options_rejects_switch_tick_without_switch_prompt() {
        let err = parse_options(["--prompt-switch-tick", "4"].into_iter())
            .expect_err("switch tick without switch prompt");
        assert!(err.contains("--switch-llm-"));
    }

    #[test]
    fn truncate_for_llm_io_log_marks_truncation() {
        let truncated = truncate_for_llm_io_log("abcdef", Some(3));
        assert!(truncated.starts_with("abc"));
        assert!(truncated.contains("truncated"));
    }

    #[test]
    fn reject_reason_metric_key_uses_serde_tag_name() {
        let key = reject_reason_metric_key(&RejectReason::InvalidAmount { amount: 0 });
        assert_eq!(key, "invalid_amount");
    }

    #[test]
    fn action_metric_key_uses_serde_tag_name() {
        let key = action_metric_key(&Action::BuildFactory {
            owner: agent_world::simulator::ResourceOwner::Agent {
                agent_id: "agent-0".to_string(),
            },
            location_id: "loc-0".to_string(),
            factory_id: "factory.alpha".to_string(),
            factory_kind: "factory.assembler.mk1".to_string(),
        });
        assert_eq!(key, "build_factory");
    }

    #[test]
    fn observe_action_result_counts_reject_reason_breakdown() {
        let mut report = DemoRunReport::new("llm_bootstrap".to_string(), 1);
        let action_result = ActionResult {
            action: Action::HarvestRadiation {
                agent_id: "agent-0".to_string(),
                max_amount: 1,
            },
            action_id: 1,
            success: false,
            event: WorldEvent {
                id: 1,
                time: 1,
                kind: WorldEventKind::ActionRejected {
                    reason: RejectReason::InvalidAmount { amount: 0 },
                },
            },
        };

        report.observe_action_result(3, &action_result);

        assert_eq!(report.action_success, 0);
        assert_eq!(report.action_failure, 1);
        assert_eq!(report.action_kind_counts.get("harvest_radiation"), Some(&1));
        assert_eq!(
            report.action_kind_failure_counts.get("harvest_radiation"),
            Some(&1)
        );
        assert_eq!(report.first_action_tick.get("harvest_radiation"), Some(&3));
        assert_eq!(
            report.action_reject_reason_counts.get("invalid_amount"),
            Some(&1)
        );
    }

    #[test]
    fn observe_action_result_counts_success_and_first_tick_per_action_kind() {
        let mut report = DemoRunReport::new("llm_bootstrap".to_string(), 1);
        let success = ActionResult {
            action: Action::BuildFactory {
                owner: agent_world::simulator::ResourceOwner::Agent {
                    agent_id: "agent-0".to_string(),
                },
                location_id: "loc-0".to_string(),
                factory_id: "factory.alpha".to_string(),
                factory_kind: "factory.assembler.mk1".to_string(),
            },
            action_id: 7,
            success: true,
            event: WorldEvent {
                id: 7,
                time: 7,
                kind: WorldEventKind::FactoryBuilt {
                    owner: agent_world::simulator::ResourceOwner::Agent {
                        agent_id: "agent-0".to_string(),
                    },
                    location_id: "loc-0".to_string(),
                    factory_id: "factory.alpha".to_string(),
                    factory_kind: "factory.assembler.mk1".to_string(),
                    electricity_cost: 10,
                    hardware_cost: 2,
                },
            },
        };
        let failure = ActionResult {
            action: Action::ScheduleRecipe {
                owner: agent_world::simulator::ResourceOwner::Agent {
                    agent_id: "agent-0".to_string(),
                },
                factory_id: "factory.alpha".to_string(),
                recipe_id: "recipe.assembler.logistics_drone".to_string(),
                batches: 1,
            },
            action_id: 8,
            success: false,
            event: WorldEvent {
                id: 8,
                time: 8,
                kind: WorldEventKind::ActionRejected {
                    reason: RejectReason::FacilityNotFound {
                        facility_id: "factory.alpha".to_string(),
                    },
                },
            },
        };

        report.observe_action_result(5, &success);
        report.observe_action_result(9, &failure);

        assert_eq!(report.action_kind_counts.get("build_factory"), Some(&1));
        assert_eq!(report.action_kind_counts.get("schedule_recipe"), Some(&1));
        assert_eq!(
            report.action_kind_success_counts.get("build_factory"),
            Some(&1)
        );
        assert_eq!(
            report.action_kind_failure_counts.get("schedule_recipe"),
            Some(&1)
        );
        assert_eq!(report.first_action_tick.get("build_factory"), Some(&5));
        assert_eq!(report.first_action_tick.get("schedule_recipe"), Some(&9));
    }
}
