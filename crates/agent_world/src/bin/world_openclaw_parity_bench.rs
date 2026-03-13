use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;
use std::time::Instant;

use agent_world::simulator::{
    initialize_kernel, Action, ActionCatalogEntry, ActionResult, AgentBehavior, AgentDecision,
    AgentDecisionTrace, AgentRunner, LlmAgentBehavior, Observation, OpenAiChatCompletionClient,
    OpenClawAdapter, OpenClawLocalHttpClient, RuntimePerfSnapshot, WorldConfig, WorldEvent,
    WorldInitConfig, WorldScenario,
};
use serde::{Deserialize, Serialize};

const DEFAULT_PROTOCOL_VERSION: &str = "2026-03-12";
const DEFAULT_ADAPTER_VERSION: &str = "openclaw_phase1_adapter_v1";
const DEFAULT_TIMEOUT_MS: u64 = 3_000;
const DEFAULT_TICKS: u64 = 20;
const DEFAULT_PROVIDER_CONNECT_TIMEOUT_MS: u64 = 3_000;
const DEFAULT_OPENCLAW_AGENT_PROFILE: &str = "agent_world_p0_low_freq_npc";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum BenchProviderKind {
    Builtin,
    OpenclawLocalHttp,
}

impl BenchProviderKind {
    fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "builtin" => Some(Self::Builtin),
            "openclaw_local_http" | "openclaw" => Some(Self::OpenclawLocalHttp),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Builtin => "builtin",
            Self::OpenclawLocalHttp => "openclaw_local_http",
        }
    }

    fn summary_suffix(self) -> &'static str {
        self.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliOptions {
    provider: BenchProviderKind,
    scenario: WorldScenario,
    scenario_id: String,
    parity_tier: String,
    benchmark_run_id: String,
    fixture_id: Option<String>,
    protocol_version: String,
    adapter_version: String,
    ticks: u64,
    timeout_ms: u64,
    out_dir: PathBuf,
    openclaw_base_url: Option<String>,
    openclaw_auth_token: Option<String>,
    openclaw_connect_timeout_ms: u64,
    openclaw_agent_profile: String,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            provider: BenchProviderKind::Builtin,
            scenario: WorldScenario::LlmBootstrap,
            scenario_id: "P0-001".to_string(),
            parity_tier: "P0".to_string(),
            benchmark_run_id: "manual".to_string(),
            fixture_id: None,
            protocol_version: DEFAULT_PROTOCOL_VERSION.to_string(),
            adapter_version: DEFAULT_ADAPTER_VERSION.to_string(),
            ticks: DEFAULT_TICKS,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            out_dir: PathBuf::from("artifacts/manual"),
            openclaw_base_url: None,
            openclaw_auth_token: None,
            openclaw_connect_timeout_ms: DEFAULT_PROVIDER_CONNECT_TIMEOUT_MS,
            openclaw_agent_profile: DEFAULT_OPENCLAW_AGENT_PROFILE.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct ProviderRunInfo {
    provider_kind: String,
    provider_version: String,
    adapter_version: String,
    protocol_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_queue_depth: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    agent_profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct FixtureRefs {
    initial_world_snapshot_ref: String,
    observation_sequence_ref: String,
    goal_definition: String,
    action_catalog_ref: String,
    player_context_ref: String,
    memory_fixture_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct StepTraceRecord {
    benchmark_run_id: String,
    parity_tier: String,
    scenario_id: String,
    fixture_id: String,
    provider_kind: String,
    provider_version: String,
    adapter_version: String,
    protocol_version: String,
    step_index: u64,
    agent_id: String,
    decision: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    action_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    latency_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_code: Option<String>,
    retry_count: u32,
    trace_present: bool,
    trace_message_count: usize,
    trace_tool_call_count: usize,
    context_drift_flag: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    action_success: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
struct SampleSummary {
    benchmark_run_id: String,
    parity_tier: String,
    scenario_id: String,
    fixture_id: String,
    provider_kind: String,
    provider_version: String,
    adapter_version: String,
    protocol_version: String,
    scenario: String,
    seed: String,
    status: String,
    goal_completed: bool,
    completion_time_ms: u64,
    decision_steps: u64,
    invalid_action_count: u64,
    timeout_count: u64,
    recoverable_error_count: u64,
    fatal_error_count: u64,
    trace_completeness_ratio_ppm: u64,
    median_latency_ms: u64,
    p95_latency_ms: u64,
    context_drift_count: u64,
    action_kind_counts: BTreeMap<String, u64>,
    error_counts: BTreeMap<String, u64>,
    fixture_refs: FixtureRefs,
    provider: ProviderRunInfo,
    notes: Vec<String>,
    runtime_perf: RuntimePerfSnapshot,
}

enum BenchBehavior {
    Builtin(LlmAgentBehavior<OpenAiChatCompletionClient>),
    OpenClaw(ProviderBackedOpenClawBehavior),
}

struct ProviderBackedOpenClawBehavior {
    inner: agent_world::simulator::ProviderBackedAgentBehavior<OpenClawAdapter>,
}

impl AgentBehavior for BenchBehavior {
    fn agent_id(&self) -> &str {
        match self {
            Self::Builtin(inner) => inner.agent_id(),
            Self::OpenClaw(inner) => inner.agent_id(),
        }
    }

    fn decide(&mut self, observation: &Observation) -> AgentDecision {
        match self {
            Self::Builtin(inner) => inner.decide(observation),
            Self::OpenClaw(inner) => inner.decide(observation),
        }
    }

    fn on_action_result(&mut self, result: &ActionResult) {
        match self {
            Self::Builtin(inner) => inner.on_action_result(result),
            Self::OpenClaw(inner) => inner.on_action_result(result),
        }
    }

    fn on_event(&mut self, event: &WorldEvent) {
        match self {
            Self::Builtin(inner) => inner.on_event(event),
            Self::OpenClaw(inner) => inner.on_event(event),
        }
    }

    fn take_decision_trace(&mut self) -> Option<AgentDecisionTrace> {
        match self {
            Self::Builtin(inner) => inner.take_decision_trace(),
            Self::OpenClaw(inner) => inner.take_decision_trace(),
        }
    }
}

impl AgentBehavior for ProviderBackedOpenClawBehavior {
    fn agent_id(&self) -> &str {
        self.inner.agent_id()
    }

    fn decide(&mut self, observation: &Observation) -> AgentDecision {
        self.inner.decide(observation)
    }

    fn on_action_result(&mut self, result: &ActionResult) {
        self.inner.on_action_result(result);
    }

    fn on_event(&mut self, event: &WorldEvent) {
        self.inner.on_event(event);
    }

    fn take_decision_trace(&mut self) -> Option<AgentDecisionTrace> {
        self.inner.take_decision_trace()
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let options = match parse_options(args.iter().skip(1).map(String::as_str)) {
        Ok(options) => options,
        Err(err) => {
            eprintln!("{err}");
            print_help();
            process::exit(1);
        }
    };

    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(options.scenario, &config);
    let (mut kernel, init_report) = match initialize_kernel(config, init) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("failed to initialize world: {err:?}");
            process::exit(1);
        }
    };
    let seed = init_report.seed.to_string();
    let fixture_id = options
        .fixture_id
        .clone()
        .unwrap_or_else(|| format!("{}-{}", options.scenario.as_str(), seed));

    let provider = match prepare_provider_info(&options) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("failed to prepare provider: {err}");
            process::exit(1);
        }
    };

    let raw_dir = options.out_dir.join("raw");
    let summary_dir = options.out_dir.join("summary");
    if let Err(err) = fs::create_dir_all(&raw_dir) {
        eprintln!("failed to create raw dir {}: {err}", raw_dir.display());
        process::exit(1);
    }
    if let Err(err) = fs::create_dir_all(&summary_dir) {
        eprintln!(
            "failed to create summary dir {}: {err}",
            summary_dir.display()
        );
        process::exit(1);
    }

    let raw_path = raw_dir.join(format!(
        "{}.{}.jsonl",
        sanitize_filename(fixture_id.as_str()),
        options.provider.summary_suffix()
    ));
    let summary_path = summary_dir.join(format!(
        "{}.{}.json",
        sanitize_filename(options.scenario_id.as_str()),
        options.provider.summary_suffix()
    ));

    let mut runner: AgentRunner<BenchBehavior> = AgentRunner::new();
    let mut agent_ids: Vec<String> = kernel.model().agents.keys().cloned().collect();
    agent_ids.sort();
    if agent_ids.is_empty() {
        eprintln!("no agents in scenario {}", options.scenario.as_str());
        process::exit(1);
    }

    for agent_id in &agent_ids {
        let behavior = match build_behavior(agent_id.as_str(), &options) {
            Ok(value) => value,
            Err(err) => {
                eprintln!("failed to build behavior for {agent_id}: {err}");
                process::exit(1);
            }
        };
        runner.register(behavior);
    }

    let run_started_at = Instant::now();
    let mut step_records = Vec::new();
    let mut notes = Vec::new();
    let mut action_kind_counts = BTreeMap::new();
    let mut error_counts = BTreeMap::new();
    let mut invalid_action_count = 0_u64;
    let mut timeout_count = 0_u64;
    let mut recoverable_error_count = 0_u64;
    let mut fatal_error_count = 0_u64;
    let mut context_drift_count = 0_u64;
    let mut trace_present_count = 0_u64;
    let mut decision_steps = 0_u64;
    let mut latencies = Vec::new();

    for step_index in 1..=options.ticks {
        let Some(result) = runner.tick(&mut kernel) else {
            notes.push(format!("step {step_index}: runner returned no result"));
            continue;
        };
        decision_steps += 1;
        let action_ref = action_ref_from_decision(&result.decision);
        if let Some(action_ref) = action_ref.as_ref() {
            let entry = action_kind_counts.entry(action_ref.clone()).or_insert(0);
            *entry += 1;
        }

        let trace_present = result.decision_trace.is_some();
        if trace_present {
            trace_present_count += 1;
        }
        let latency_ms = result
            .decision_trace
            .as_ref()
            .and_then(|trace| trace.llm_diagnostics.as_ref())
            .and_then(|diagnostics| diagnostics.latency_ms);
        if let Some(latency_ms) = latency_ms {
            latencies.push(latency_ms);
        }

        let mut error_code = classify_trace_error(
            result.decision_trace.as_ref(),
            result.action_result.as_ref(),
        );
        if let Some(code) = error_code.as_ref() {
            let entry = error_counts.entry(code.clone()).or_insert(0);
            *entry += 1;
            match code.as_str() {
                "timeout" => {
                    timeout_count += 1;
                    recoverable_error_count += 1;
                }
                "provider_unreachable" | "invalid_action_schema" | "action_rejected" => {
                    recoverable_error_count += 1;
                }
                "context_drift" => {
                    context_drift_count += 1;
                }
                "session_cross_talk" => {
                    fatal_error_count += 1;
                }
                "trace_missing" => {}
                _ => {
                    fatal_error_count += 1;
                }
            }
        }

        let action_success = result.action_result.as_ref().map(|value| value.success);
        if matches!(action_success, Some(false)) {
            invalid_action_count += 1;
            let entry = error_counts
                .entry("action_rejected".to_string())
                .or_insert(0);
            *entry += 1;
            if error_code.is_none() {
                error_code = Some("action_rejected".to_string());
            }
        }

        if let Some(result_action) = result.action_result.as_ref() {
            if let Some(reject_reason) = result_action.reject_reason() {
                notes.push(format!(
                    "step {step_index}: action rejected for agent {} with {:?}",
                    result.agent_id, reject_reason
                ));
            }
        }

        step_records.push(StepTraceRecord {
            benchmark_run_id: options.benchmark_run_id.clone(),
            parity_tier: options.parity_tier.clone(),
            scenario_id: options.scenario_id.clone(),
            fixture_id: fixture_id.clone(),
            provider_kind: options.provider.as_str().to_string(),
            provider_version: provider.provider_version.clone(),
            adapter_version: options.adapter_version.clone(),
            protocol_version: options.protocol_version.clone(),
            step_index,
            agent_id: result.agent_id.clone(),
            decision: decision_label(&result.decision),
            action_ref,
            latency_ms,
            error_code,
            retry_count: result
                .decision_trace
                .as_ref()
                .and_then(|trace| trace.llm_diagnostics.as_ref())
                .map(|diagnostics| diagnostics.retry_count)
                .unwrap_or(0),
            trace_present,
            trace_message_count: result
                .decision_trace
                .as_ref()
                .map(|trace| trace.llm_chat_messages.len())
                .unwrap_or(0),
            trace_tool_call_count: result
                .decision_trace
                .as_ref()
                .map(|trace| trace.llm_step_trace.len())
                .unwrap_or(0),
            context_drift_flag: false,
            action_success,
        });
    }

    let goal_completed = scenario_goal_completed(
        options.scenario_id.as_str(),
        &action_kind_counts,
        &error_counts,
        invalid_action_count,
    );
    let status = derive_status(goal_completed, &error_counts, &notes);
    let trace_completeness_ratio_ppm = ratio_ppm(trace_present_count, decision_steps);
    let summary = SampleSummary {
        benchmark_run_id: options.benchmark_run_id.clone(),
        parity_tier: options.parity_tier.clone(),
        scenario_id: options.scenario_id.clone(),
        fixture_id,
        provider_kind: options.provider.as_str().to_string(),
        provider_version: provider.provider_version.clone(),
        adapter_version: options.adapter_version.clone(),
        protocol_version: options.protocol_version.clone(),
        scenario: options.scenario.as_str().to_string(),
        seed,
        status,
        goal_completed,
        completion_time_ms: run_started_at.elapsed().as_millis().min(u64::MAX as u128) as u64,
        decision_steps,
        invalid_action_count,
        timeout_count,
        recoverable_error_count,
        fatal_error_count,
        trace_completeness_ratio_ppm,
        median_latency_ms: percentile_u64(&latencies, 50.0),
        p95_latency_ms: percentile_u64(&latencies, 95.0),
        context_drift_count,
        action_kind_counts,
        error_counts,
        fixture_refs: FixtureRefs {
            initial_world_snapshot_ref: format!(
                "scenario://{}/snapshot",
                options.scenario.as_str()
            ),
            observation_sequence_ref: format!(
                "scenario://{}/observations",
                options.scenario.as_str()
            ),
            goal_definition: format!("parity://{}/{}", options.parity_tier, options.scenario_id),
            action_catalog_ref: "catalog://openclaw/phase1".to_string(),
            player_context_ref: "player://default".to_string(),
            memory_fixture_ref: "memory://default".to_string(),
        },
        provider,
        notes,
        runtime_perf: runner.runtime_perf_snapshot(),
    };

    if let Err(err) = write_jsonl(raw_path.as_path(), &step_records) {
        eprintln!("failed to write raw trace jsonl: {err}");
        process::exit(1);
    }
    if let Err(err) = write_json(summary_path.as_path(), &summary) {
        eprintln!("failed to write summary json: {err}");
        process::exit(1);
    }

    println!("provider: {}", options.provider.as_str());
    println!("scenario: {}", options.scenario.as_str());
    println!("scenario_id: {}", options.scenario_id);
    println!("benchmark_run_id: {}", options.benchmark_run_id);
    println!("summary_json: {}", summary_path.display());
    println!("raw_jsonl: {}", raw_path.display());
    println!("status: {}", summary.status);
    println!(
        "goal_completed: {}",
        if summary.goal_completed { 1 } else { 0 }
    );
    println!("decision_steps: {}", summary.decision_steps);
    println!("invalid_action_count: {}", summary.invalid_action_count);
    println!("timeout_count: {}", summary.timeout_count);
    println!(
        "trace_completeness_ratio_ppm: {}",
        summary.trace_completeness_ratio_ppm
    );
    println!("median_latency_ms: {}", summary.median_latency_ms);
    println!("p95_latency_ms: {}", summary.p95_latency_ms);
}

fn prepare_provider_info(options: &CliOptions) -> Result<ProviderRunInfo, String> {
    match options.provider {
        BenchProviderKind::Builtin => Ok(ProviderRunInfo {
            provider_kind: options.provider.as_str().to_string(),
            provider_version: "builtin_llm_env".to_string(),
            adapter_version: options.adapter_version.clone(),
            protocol_version: options.protocol_version.clone(),
            provider_status: None,
            provider_last_error: None,
            provider_queue_depth: None,
            agent_profile: None,
        }),
        BenchProviderKind::OpenclawLocalHttp => {
            let base_url = options.openclaw_base_url.as_deref().ok_or_else(|| {
                "--openclaw-base-url is required for openclaw_local_http".to_string()
            })?;
            let client = OpenClawLocalHttpClient::new(
                base_url,
                options.openclaw_auth_token.as_deref(),
                options.openclaw_connect_timeout_ms,
            )
            .map_err(|err| err.to_string())?;
            let info = client.provider_info().map_err(|err| err.to_string())?;
            let health = client.provider_health().map_err(|err| err.to_string())?;
            Ok(ProviderRunInfo {
                provider_kind: options.provider.as_str().to_string(),
                provider_version: info.version.unwrap_or_else(|| "unknown".to_string()),
                adapter_version: options.adapter_version.clone(),
                protocol_version: info
                    .protocol_version
                    .unwrap_or_else(|| options.protocol_version.clone()),
                provider_status: health.status,
                provider_last_error: health.last_error,
                provider_queue_depth: health.queue_depth,
                agent_profile: Some(options.openclaw_agent_profile.clone()),
            })
        }
    }
}

fn build_behavior(agent_id: &str, options: &CliOptions) -> Result<BenchBehavior, String> {
    match options.provider {
        BenchProviderKind::Builtin => LlmAgentBehavior::from_env(agent_id.to_string())
            .map(BenchBehavior::Builtin)
            .map_err(|err| err.to_string()),
        BenchProviderKind::OpenclawLocalHttp => {
            let base_url = options.openclaw_base_url.as_deref().ok_or_else(|| {
                "--openclaw-base-url is required for openclaw_local_http".to_string()
            })?;
            let adapter = OpenClawAdapter::new(
                base_url,
                options.openclaw_auth_token.as_deref(),
                options.openclaw_connect_timeout_ms,
            )
            .map_err(|err| err.to_string())?;
            let behavior = agent_world::simulator::ProviderBackedAgentBehavior::new(
                agent_id.to_string(),
                adapter,
                phase1_action_catalog(),
            )
            .with_provider_config_ref("openclaw://local-http")
            .with_agent_profile(options.openclaw_agent_profile.clone());
            Ok(BenchBehavior::OpenClaw(ProviderBackedOpenClawBehavior {
                inner: behavior,
            }))
        }
    }
}

fn phase1_action_catalog() -> Vec<ActionCatalogEntry> {
    vec![
        ActionCatalogEntry::new("wait", "yield current turn without acting"),
        ActionCatalogEntry::new("wait_ticks", "sleep for a bounded number of ticks"),
        ActionCatalogEntry::new("move_agent", "move to a neighboring location"),
        ActionCatalogEntry::new("speak_to_nearby", "emit a lightweight nearby speech event"),
        ActionCatalogEntry::new(
            "inspect_target",
            "emit a lightweight target inspection event",
        ),
        ActionCatalogEntry::new(
            "simple_interact",
            "emit a lightweight single-step interaction event",
        ),
    ]
}

fn action_ref_from_decision(decision: &AgentDecision) -> Option<String> {
    match decision {
        AgentDecision::Wait => Some("wait".to_string()),
        AgentDecision::WaitTicks(_) => Some("wait_ticks".to_string()),
        AgentDecision::Act(action) => Some(action_ref_from_action(action).to_string()),
    }
}

fn action_ref_from_action(action: &Action) -> &'static str {
    match action {
        Action::MoveAgent { .. } => "move_agent",
        Action::SpeakToNearby { .. } => "speak_to_nearby",
        Action::InspectTarget { .. } => "inspect_target",
        Action::SimpleInteract { .. } => "simple_interact",
        _ => "other",
    }
}

fn decision_label(decision: &AgentDecision) -> String {
    match decision {
        AgentDecision::Wait => "wait".to_string(),
        AgentDecision::WaitTicks(ticks) => format!("wait_ticks:{ticks}"),
        AgentDecision::Act(action) => format!("act:{}", action_ref_from_action(action)),
    }
}

fn classify_trace_error(
    trace: Option<&AgentDecisionTrace>,
    action_result: Option<&ActionResult>,
) -> Option<String> {
    if let Some(result) = action_result {
        if !result.success {
            return Some("action_rejected".to_string());
        }
    }
    let err =
        trace.and_then(|value| value.llm_error.as_deref().or(value.parse_error.as_deref()))?;
    let lowered = err.to_ascii_lowercase();
    if lowered.contains("timeout") {
        Some("timeout".to_string())
    } else if lowered.contains("provider_unreachable") || lowered.contains("unreachable") {
        Some("provider_unreachable".to_string())
    } else if lowered.contains("invalid_action_schema") || lowered.contains("schema") {
        Some("invalid_action_schema".to_string())
    } else if lowered.contains("session_cross_talk") || lowered.contains("cross talk") {
        Some("session_cross_talk".to_string())
    } else if lowered.contains("context_drift") || lowered.contains("drift") {
        Some("context_drift".to_string())
    } else {
        None
    }
}

fn scenario_goal_completed(
    scenario_id: &str,
    action_kind_counts: &BTreeMap<String, u64>,
    error_counts: &BTreeMap<String, u64>,
    invalid_action_count: u64,
) -> bool {
    match scenario_id {
        "P0-001" => action_kind_counts.get("move_agent").copied().unwrap_or(0) >= 3,
        "P0-002" => {
            action_kind_counts
                .get("inspect_target")
                .copied()
                .unwrap_or(0)
                >= 1
        }
        "P0-003" => {
            action_kind_counts
                .get("speak_to_nearby")
                .copied()
                .unwrap_or(0)
                >= 2
        }
        "P0-004" => {
            action_kind_counts
                .get("simple_interact")
                .copied()
                .unwrap_or(0)
                >= 1
                && invalid_action_count == 0
        }
        "P0-005" => !error_counts.is_empty() && invalid_action_count == 0,
        _ => action_kind_counts.values().copied().sum::<u64>() > 0,
    }
}

fn derive_status(
    goal_completed: bool,
    error_counts: &BTreeMap<String, u64>,
    notes: &[String],
) -> String {
    if error_counts.contains_key("session_cross_talk") {
        return "blocked".to_string();
    }
    if notes.iter().any(|note| note.contains("invalid_fixture")) {
        return "invalid_fixture".to_string();
    }
    if goal_completed {
        "passed".to_string()
    } else {
        "failed".to_string()
    }
}

fn ratio_ppm(numerator: u64, denominator: u64) -> u64 {
    if denominator == 0 {
        0
    } else {
        numerator
            .saturating_mul(1_000_000)
            .saturating_div(denominator)
    }
}

fn percentile_u64(values: &[u64], percentile: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let rank = (((sorted.len() - 1) as f64) * percentile / 100.0).round() as usize;
    sorted[rank.min(sorted.len() - 1)]
}

fn sanitize_filename(input: &str) -> String {
    input
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect()
}

fn write_jsonl(path: &Path, records: &[StepTraceRecord]) -> Result<(), String> {
    let mut file =
        File::create(path).map_err(|err| format!("create {} failed: {err}", path.display()))?;
    for record in records {
        let line = serde_json::to_string(record)
            .map_err(|err| format!("serialize record failed: {err}"))?;
        writeln!(file, "{line}")
            .map_err(|err| format!("write {} failed: {err}", path.display()))?;
    }
    Ok(())
}

fn write_json(path: &Path, summary: &SampleSummary) -> Result<(), String> {
    let content = serde_json::to_string_pretty(summary)
        .map_err(|err| format!("serialize summary failed: {err}"))?;
    fs::write(path, format!("{content}\n"))
        .map_err(|err| format!("write {} failed: {err}", path.display()))
}

fn parse_options<'a>(args: impl Iterator<Item = &'a str>) -> Result<CliOptions, String> {
    let mut options = CliOptions::default();
    let mut iter = args.peekable();

    while let Some(arg) = iter.next() {
        match arg {
            "--provider" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| "--provider requires a value".to_string())?;
                options.provider = BenchProviderKind::parse(raw)
                    .ok_or_else(|| format!("invalid --provider: {raw}"))?;
            }
            "--scenario" => {
                let raw = iter
                    .next()
                    .ok_or_else(|| "--scenario requires a value".to_string())?;
                options.scenario = WorldScenario::parse(raw)
                    .ok_or_else(|| format!("invalid --scenario: {raw}"))?;
            }
            "--scenario-id" => {
                options.scenario_id = iter
                    .next()
                    .ok_or_else(|| "--scenario-id requires a value".to_string())?
                    .to_string();
            }
            "--parity-tier" => {
                options.parity_tier = iter
                    .next()
                    .ok_or_else(|| "--parity-tier requires a value".to_string())?
                    .to_string();
            }
            "--benchmark-run-id" => {
                options.benchmark_run_id = iter
                    .next()
                    .ok_or_else(|| "--benchmark-run-id requires a value".to_string())?
                    .to_string();
            }
            "--fixture-id" => {
                options.fixture_id = Some(
                    iter.next()
                        .ok_or_else(|| "--fixture-id requires a value".to_string())?
                        .to_string(),
                );
            }
            "--protocol-version" => {
                options.protocol_version = iter
                    .next()
                    .ok_or_else(|| "--protocol-version requires a value".to_string())?
                    .to_string();
            }
            "--adapter-version" => {
                options.adapter_version = iter
                    .next()
                    .ok_or_else(|| "--adapter-version requires a value".to_string())?
                    .to_string();
            }
            "--ticks" => {
                options.ticks = parse_u64(
                    iter.next()
                        .ok_or_else(|| "--ticks requires a value".to_string())?,
                    "--ticks",
                )?;
            }
            "--timeout-ms" => {
                options.timeout_ms = parse_u64(
                    iter.next()
                        .ok_or_else(|| "--timeout-ms requires a value".to_string())?,
                    "--timeout-ms",
                )?;
            }
            "--out-dir" => {
                options.out_dir = PathBuf::from(
                    iter.next()
                        .ok_or_else(|| "--out-dir requires a value".to_string())?,
                );
            }
            "--openclaw-base-url" => {
                options.openclaw_base_url = Some(
                    iter.next()
                        .ok_or_else(|| "--openclaw-base-url requires a value".to_string())?
                        .to_string(),
                );
            }
            "--openclaw-auth-token" => {
                options.openclaw_auth_token = Some(
                    iter.next()
                        .ok_or_else(|| "--openclaw-auth-token requires a value".to_string())?
                        .to_string(),
                );
            }
            "--openclaw-connect-timeout-ms" => {
                options.openclaw_connect_timeout_ms = parse_u64(
                    iter.next().ok_or_else(|| {
                        "--openclaw-connect-timeout-ms requires a value".to_string()
                    })?,
                    "--openclaw-connect-timeout-ms",
                )?;
            }
            "--openclaw-agent-profile" => {
                options.openclaw_agent_profile = iter
                    .next()
                    .ok_or_else(|| "--openclaw-agent-profile requires a value".to_string())?
                    .to_string();
            }
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            other => return Err(format!("unknown option: {other}")),
        }
    }

    if options.scenario_id.trim().is_empty() {
        return Err("--scenario-id cannot be empty".to_string());
    }
    if options.parity_tier.trim().is_empty() {
        return Err("--parity-tier cannot be empty".to_string());
    }
    if options.benchmark_run_id.trim().is_empty() {
        return Err("--benchmark-run-id cannot be empty".to_string());
    }
    if options.out_dir.as_os_str().is_empty() {
        return Err("--out-dir cannot be empty".to_string());
    }
    if options.provider == BenchProviderKind::OpenclawLocalHttp
        && options
            .openclaw_base_url
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
    {
        return Err("--openclaw-base-url is required for openclaw_local_http".to_string());
    }
    if options.provider == BenchProviderKind::OpenclawLocalHttp
        && options.openclaw_agent_profile.trim().is_empty()
    {
        return Err("--openclaw-agent-profile cannot be empty".to_string());
    }
    Ok(options)
}

fn parse_u64(raw: &str, flag: &str) -> Result<u64, String> {
    raw.parse::<u64>()
        .map_err(|err| format!("invalid {flag}: {err}"))
}

fn print_help() {
    println!(
        "Usage: world_openclaw_parity_bench [options]\n\n\
Run one parity benchmark sample for builtin or OpenClaw(Local HTTP) and emit\n\
raw jsonl + single-sample summary json following the parity benchmark contract.\n\n\
Options:\n\
  --provider <builtin|openclaw_local_http>\n\
  --scenario <name>\n\
  --scenario-id <id>\n\
  --parity-tier <P0|P1|P2>\n\
  --benchmark-run-id <id>\n\
  --fixture-id <id>\n\
  --ticks <n>\n\
  --timeout-ms <n>\n\
  --out-dir <path>\n\
  --openclaw-base-url <url>\n\
  --openclaw-auth-token <token>\n\
  --openclaw-connect-timeout-ms <n>\n\
  --openclaw-agent-profile <id>\n\
  --protocol-version <str>\n\
  --adapter-version <str>\n\
  -h, --help\n"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_options_accepts_openclaw_provider() {
        let options = parse_options(
            [
                "--provider",
                "openclaw_local_http",
                "--scenario-id",
                "P0-002",
                "--benchmark-run-id",
                "run-1",
                "--openclaw-base-url",
                "http://127.0.0.1:5841",
                "--out-dir",
                ".tmp/parity",
            ]
            .into_iter(),
        )
        .expect("parse options");
        assert_eq!(options.provider, BenchProviderKind::OpenclawLocalHttp);
        assert_eq!(options.scenario_id, "P0-002");
        assert_eq!(options.benchmark_run_id, "run-1");
        assert_eq!(
            options.openclaw_base_url.as_deref(),
            Some("http://127.0.0.1:5841")
        );
        assert_eq!(
            options.openclaw_agent_profile,
            DEFAULT_OPENCLAW_AGENT_PROFILE
        );
    }

    #[test]
    fn parse_options_rejects_openclaw_without_base_url() {
        let err = parse_options(
            [
                "--provider",
                "openclaw_local_http",
                "--benchmark-run-id",
                "run-1",
            ]
            .into_iter(),
        )
        .expect_err("missing base url should fail");
        assert!(err.contains("--openclaw-base-url"));
    }

    #[test]
    fn parse_options_accepts_custom_openclaw_agent_profile() {
        let options = parse_options(
            [
                "--provider",
                "openclaw_local_http",
                "--benchmark-run-id",
                "run-2",
                "--openclaw-base-url",
                "http://127.0.0.1:5841",
                "--openclaw-agent-profile",
                "agent_world_p1_memory_loop",
            ]
            .into_iter(),
        )
        .expect("parse custom profile");
        assert_eq!(options.openclaw_agent_profile, "agent_world_p1_memory_loop");
    }

    #[test]
    fn scenario_goal_completed_uses_p0_rules() {
        let mut action_kind_counts = BTreeMap::new();
        action_kind_counts.insert("move_agent".to_string(), 3);
        assert!(scenario_goal_completed(
            "P0-001",
            &action_kind_counts,
            &BTreeMap::new(),
            0,
        ));

        action_kind_counts.clear();
        action_kind_counts.insert("simple_interact".to_string(), 1);
        assert!(scenario_goal_completed(
            "P0-004",
            &action_kind_counts,
            &BTreeMap::new(),
            0,
        ));
        assert!(!scenario_goal_completed(
            "P0-004",
            &action_kind_counts,
            &BTreeMap::new(),
            1,
        ));
    }

    #[test]
    fn classify_trace_error_detects_timeout() {
        let trace = AgentDecisionTrace {
            agent_id: "agent-1".to_string(),
            time: 1,
            decision: AgentDecision::Wait,
            llm_input: None,
            llm_output: None,
            llm_error: Some("timeout: provider request timed out".to_string()),
            parse_error: None,
            llm_diagnostics: None,
            llm_effect_intents: Vec::new(),
            llm_effect_receipts: Vec::new(),
            llm_step_trace: Vec::new(),
            llm_prompt_section_trace: Vec::new(),
            llm_chat_messages: Vec::new(),
        };
        assert_eq!(
            classify_trace_error(Some(&trace), None).as_deref(),
            Some("timeout")
        );
    }
}
