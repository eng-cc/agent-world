use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process;

use agent_world::simulator::{
    initialize_kernel, Action as SimulatorAction, ActionResult, AgentDecision, AgentDecisionTrace,
    AgentRunner, LlmAgentBehavior, RejectReason, WorldConfig, WorldInitConfig, WorldScenario,
};
use serde::{Deserialize, Serialize};

#[path = "world_llm_agent_demo/llm_io.rs"]
mod llm_io;
#[path = "world_llm_agent_demo/runtime_bridge.rs"]
mod runtime_bridge;

use llm_io::print_llm_io_trace;
#[cfg(test)]
use llm_io::truncate_for_llm_io_log;
use runtime_bridge::{
    advance_kernel_time_with_noop_move, execute_action_in_kernel, is_bridgeable_action,
    RuntimeGameplayBridge,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PromptSwitchSpec {
    tick: u64,
    #[serde(default, alias = "system_prompt")]
    llm_system_prompt: Option<String>,
    #[serde(default, alias = "short_term_goal")]
    llm_short_term_goal: Option<String>,
    #[serde(default, alias = "long_term_goal")]
    llm_long_term_goal: Option<String>,
}

impl PromptSwitchSpec {
    fn has_override(&self) -> bool {
        self.llm_system_prompt.is_some()
            || self.llm_short_term_goal.is_some()
            || self.llm_long_term_goal.is_some()
    }
}

#[derive(Debug, Clone, PartialEq)]
struct CliOptions {
    scenario: WorldScenario,
    ticks: u64,
    runtime_gameplay_bridge: bool,
    load_state_dir: Option<String>,
    save_state_dir: Option<String>,
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
    prompt_switches_json: Option<String>,
    prompt_switches: Vec<PromptSwitchSpec>,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            scenario: WorldScenario::LlmBootstrap,
            ticks: 20,
            runtime_gameplay_bridge: true,
            load_state_dir: None,
            save_state_dir: None,
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
            prompt_switches_json: None,
            prompt_switches: Vec::new(),
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
    runtime_bridge_actions: u64,
    runtime_bridge_action_success: u64,
    runtime_bridge_action_failure: u64,
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
            runtime_bridge_actions: 0,
            runtime_bridge_action_success: 0,
            runtime_bridge_action_failure: 0,
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

    fn observe_runtime_bridge_result(&mut self, action_result: &ActionResult) {
        self.runtime_bridge_actions += 1;
        if action_result.success {
            self.runtime_bridge_action_success += 1;
        } else {
            self.runtime_bridge_action_failure += 1;
        }
    }

    fn finalize(&mut self) {
        if self.trace_counts.traces > 0 {
            self.trace_counts.llm_input_chars_avg =
                self.trace_counts.llm_input_chars_total / self.trace_counts.traces;
        }
    }
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

fn action_metric_key(action: &SimulatorAction) -> String {
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
    let (mut kernel, seed_label) = if let Some(state_dir) = options.load_state_dir.as_ref() {
        match agent_world::simulator::WorldKernel::load_from_dir(state_dir) {
            Ok(kernel) => (kernel, "loaded".to_string()),
            Err(err) => {
                eprintln!("failed to load world state from {}: {err:?}", state_dir);
                process::exit(1);
            }
        }
    } else {
        let init = WorldInitConfig::from_scenario(options.scenario, &config);
        match initialize_kernel(config, init) {
            Ok((kernel, report)) => (kernel, report.seed.to_string()),
            Err(err) => {
                eprintln!("failed to initialize world: {err:?}");
                process::exit(1);
            }
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
    println!("seed: {}", seed_label);
    println!("agents: {}", agent_ids.len());
    if let Some(path) = options.load_state_dir.as_ref() {
        println!("state_dir_loaded: {path}");
    }
    println!("ticks: {}", options.ticks);
    println!(
        "runtime_gameplay_bridge: {}",
        if options.runtime_gameplay_bridge {
            1
        } else {
            0
        }
    );

    let mut runtime_gameplay_bridge = if options.runtime_gameplay_bridge {
        match RuntimeGameplayBridge::from_kernel(&kernel) {
            Ok(bridge) => Some(bridge),
            Err(err) => {
                eprintln!("failed to initialize runtime gameplay bridge: {err}");
                process::exit(1);
            }
        }
    } else {
        None
    };

    let mut run_report = DemoRunReport::new(options.scenario.as_str().to_string(), options.ticks);
    let mut next_prompt_switch_idx = 0usize;

    for idx in 0..options.ticks {
        let tick = idx + 1;
        while next_prompt_switch_idx < options.prompt_switches.len()
            && tick >= options.prompt_switches[next_prompt_switch_idx].tick
        {
            let switch = options.prompt_switches[next_prompt_switch_idx].clone();
            for agent_id in runner.agent_ids() {
                if let Some(agent) = runner.get_mut(agent_id.as_str()) {
                    let current = agent.behavior.prompt_overrides();
                    agent.behavior.apply_prompt_overrides(
                        switch.llm_system_prompt.clone().or(current.system_prompt),
                        switch
                            .llm_short_term_goal
                            .clone()
                            .or(current.short_term_goal),
                        switch.llm_long_term_goal.clone().or(current.long_term_goal),
                    );
                }
            }
            println!(
                "tick={} prompt_switch_applied=true switch_index={} switch_tick={}",
                tick,
                next_prompt_switch_idx + 1,
                switch.tick
            );
            next_prompt_switch_idx += 1;
        }

        match runner.tick_decide_only(&mut kernel) {
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

                let action_result = if let AgentDecision::Act(action) = &result.decision {
                    let mut used_runtime_bridge = false;
                    let executed = if let Some(bridge) = runtime_gameplay_bridge.as_mut() {
                        if is_bridgeable_action(action) {
                            match bridge.execute(tick, result.agent_id.as_str(), action.clone()) {
                                Ok(bridged) => {
                                    used_runtime_bridge = true;
                                    run_report.observe_runtime_bridge_result(&bridged);
                                    bridged
                                }
                                Err(err) => {
                                    eprintln!(
                                        "runtime gameplay bridge execute failed at tick {} agent {}: {}",
                                        tick, result.agent_id, err
                                    );
                                    execute_action_in_kernel(
                                        &mut kernel,
                                        result.agent_id.as_str(),
                                        action.clone(),
                                    )
                                }
                            }
                        } else {
                            execute_action_in_kernel(
                                &mut kernel,
                                result.agent_id.as_str(),
                                action.clone(),
                            )
                        }
                    } else {
                        execute_action_in_kernel(
                            &mut kernel,
                            result.agent_id.as_str(),
                            action.clone(),
                        )
                    };

                    let _ = runner.notify_action_result(result.agent_id.as_str(), &executed);
                    if used_runtime_bridge {
                        advance_kernel_time_with_noop_move(&mut kernel, result.agent_id.as_str());
                    }
                    Some(executed)
                } else {
                    None
                };

                if let Some(action_result) = action_result.as_ref() {
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
    if let Some(path) = options.save_state_dir.as_ref() {
        if let Err(err) = kernel.save_to_dir(path) {
            eprintln!("failed to save world state to {}: {err:?}", path);
            process::exit(1);
        }
        println!("state_dir_saved: {path}");
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
    println!(
        "runtime_bridge_actions: {}",
        run_report.runtime_bridge_actions
    );
    println!(
        "runtime_bridge_action_success: {}",
        run_report.runtime_bridge_action_success
    );
    println!(
        "runtime_bridge_action_failure: {}",
        run_report.runtime_bridge_action_failure
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

fn normalize_prompt_switches(
    mut switches: Vec<PromptSwitchSpec>,
    source_hint: &str,
) -> Result<Vec<PromptSwitchSpec>, String> {
    if switches.is_empty() {
        return Err(format!("{source_hint} requires at least one switch entry"));
    }

    switches.sort_by_key(|entry| entry.tick);
    let mut previous_tick: Option<u64> = None;
    for entry in &switches {
        if entry.tick == 0 {
            return Err(format!("{source_hint} tick must be a positive integer"));
        }
        if !entry.has_override() {
            return Err(format!(
                "{source_hint} tick={} requires at least one llm_* override field",
                entry.tick
            ));
        }
        if previous_tick == Some(entry.tick) {
            return Err(format!(
                "{source_hint} contains duplicated tick={}",
                entry.tick
            ));
        }
        previous_tick = Some(entry.tick);
    }
    Ok(switches)
}

fn parse_prompt_switches_json(raw: &str) -> Result<Vec<PromptSwitchSpec>, String> {
    let parsed: Vec<PromptSwitchSpec> = serde_json::from_str(raw)
        .map_err(|err| format!("invalid --prompt-switches-json: {err}"))?;
    normalize_prompt_switches(parsed, "--prompt-switches-json")
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
            "--runtime-gameplay-bridge" => {
                options.runtime_gameplay_bridge = true;
            }
            "--no-runtime-gameplay-bridge" => {
                options.runtime_gameplay_bridge = false;
            }
            "--load-state-dir" => {
                options.load_state_dir = Some(
                    iter.next()
                        .ok_or_else(|| "--load-state-dir requires a directory path".to_string())?
                        .to_string(),
                );
            }
            "--save-state-dir" => {
                options.save_state_dir = Some(
                    iter.next()
                        .ok_or_else(|| "--save-state-dir requires a directory path".to_string())?
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
            "--prompt-switches-json" => {
                options.prompt_switches_json = Some(
                    iter.next()
                        .ok_or_else(|| "--prompt-switches-json requires a JSON string".to_string())?
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

    if options.prompt_switches_json.is_some()
        && (options.prompt_switch_tick.is_some() || options.has_switch_prompt_override())
    {
        return Err(
            "cannot combine --prompt-switches-json with --prompt-switch-tick/--switch-llm-*"
                .to_string(),
        );
    }

    if let Some(raw_json) = options.prompt_switches_json.as_ref() {
        options.prompt_switches = parse_prompt_switches_json(raw_json)?;
    } else {
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
        if let Some(tick) = options.prompt_switch_tick {
            options.prompt_switches = normalize_prompt_switches(
                vec![PromptSwitchSpec {
                    tick,
                    llm_system_prompt: options.switch_llm_system_prompt.clone(),
                    llm_short_term_goal: options.switch_llm_short_term_goal.clone(),
                    llm_long_term_goal: options.switch_llm_long_term_goal.clone(),
                }],
                "legacy --prompt-switch-tick",
            )?;
        }
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
    println!("  --load-state-dir <path>  Load simulator state from directory");
    println!("  --save-state-dir <path>  Save simulator state to directory after run");
    println!(
        "  --runtime-gameplay-bridge / --no-runtime-gameplay-bridge  Enable or disable runtime bridge for gameplay/economic actions (default: enabled)"
    );
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
        "  --prompt-switches-json <json>  Multi-stage switch plan (array of {{\"tick\":n,\"llm_*\":...}}); cannot be mixed with legacy --prompt-switch-* options"
    );
    println!(
        "Available scenarios: {}",
        WorldScenario::variants().join(", ")
    );
}

#[cfg(test)]
#[path = "world_llm_agent_demo/tests.rs"]
mod tests;
