use std::collections::HashSet;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::geometry::space_distance_cm;
use crate::simulator::{
    initialize_kernel, Action, AgentDecisionTrace, AgentPromptProfile, AgentRunner,
    LlmAgentBehavior, LlmAgentBuildError, OpenAiChatCompletionClient, PromptUpdateOperation,
    ResourceKind, ResourceOwner, RunnerMetrics, WorldConfig, WorldEvent, WorldInitConfig,
    WorldInitError, WorldKernel, WorldScenario, WorldSnapshot,
};
use sha2::{Digest, Sha256};

use super::protocol::{
    PromptControlAck, PromptControlApplyRequest, PromptControlCommand, PromptControlError,
    PromptControlOperation, PromptControlRollbackRequest, ViewerControl, ViewerEventKind,
    ViewerRequest, ViewerResponse, ViewerStream, VIEWER_PROTOCOL_VERSION,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewerLiveDecisionMode {
    Script,
    Llm,
}

#[derive(Debug, Clone)]
pub struct ViewerLiveServerConfig {
    pub bind_addr: String,
    pub tick_interval: Duration,
    pub scenario: WorldScenario,
    pub world_id: String,
    pub decision_mode: ViewerLiveDecisionMode,
}

impl ViewerLiveServerConfig {
    pub fn new(scenario: WorldScenario) -> Self {
        Self {
            bind_addr: "127.0.0.1:5010".to_string(),
            tick_interval: Duration::from_millis(200),
            world_id: format!("live-{}", scenario.as_str()),
            scenario,
            decision_mode: ViewerLiveDecisionMode::Script,
        }
    }

    pub fn with_bind_addr(mut self, addr: impl Into<String>) -> Self {
        self.bind_addr = addr.into();
        self
    }

    pub fn with_tick_interval(mut self, interval: Duration) -> Self {
        self.tick_interval = interval;
        self
    }

    pub fn with_world_id(mut self, world_id: impl Into<String>) -> Self {
        self.world_id = world_id.into();
        self
    }

    pub fn with_decision_mode(mut self, mode: ViewerLiveDecisionMode) -> Self {
        self.decision_mode = mode;
        self
    }

    pub fn with_llm_mode(mut self, enabled: bool) -> Self {
        self.decision_mode = if enabled {
            ViewerLiveDecisionMode::Llm
        } else {
            ViewerLiveDecisionMode::Script
        };
        self
    }
}

#[derive(Debug)]
pub enum ViewerLiveServerError {
    Io(io::Error),
    Serde(String),
    Init(WorldInitError),
    LlmBuild(LlmAgentBuildError),
}

impl From<io::Error> for ViewerLiveServerError {
    fn from(err: io::Error) -> Self {
        ViewerLiveServerError::Io(err)
    }
}

impl From<WorldInitError> for ViewerLiveServerError {
    fn from(err: WorldInitError) -> Self {
        ViewerLiveServerError::Init(err)
    }
}

impl From<LlmAgentBuildError> for ViewerLiveServerError {
    fn from(err: LlmAgentBuildError) -> Self {
        ViewerLiveServerError::LlmBuild(err)
    }
}

impl ViewerLiveServerError {
    fn is_disconnect(&self) -> bool {
        match self {
            ViewerLiveServerError::Io(err) => is_disconnect_error(err),
            _ => false,
        }
    }
}

pub struct ViewerLiveServer {
    config: ViewerLiveServerConfig,
    world: LiveWorld,
}

impl ViewerLiveServer {
    pub fn new(config: ViewerLiveServerConfig) -> Result<Self, ViewerLiveServerError> {
        let init = WorldInitConfig::from_scenario(config.scenario, &WorldConfig::default());
        let world = LiveWorld::new(WorldConfig::default(), init, config.decision_mode)?;
        Ok(Self { config, world })
    }

    pub fn run(&mut self) -> Result<(), ViewerLiveServerError> {
        let listener = TcpListener::bind(&self.config.bind_addr)?;
        for incoming in listener.incoming() {
            let stream = incoming?;
            if let Err(err) = self.serve_stream(stream) {
                eprintln!("viewer live server error: {err:?}");
            }
        }
        Ok(())
    }

    pub fn run_once(&mut self) -> Result<(), ViewerLiveServerError> {
        let listener = TcpListener::bind(&self.config.bind_addr)?;
        let (stream, _) = listener.accept()?;
        self.serve_stream(stream)?;
        Ok(())
    }

    fn serve_stream(&mut self, stream: TcpStream) -> Result<(), ViewerLiveServerError> {
        stream.set_nodelay(true)?;
        let reader_stream = stream.try_clone()?;
        let mut writer = BufWriter::new(stream);
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || read_requests(reader_stream, tx));

        let mut session = ViewerLiveSession::new(self.config.tick_interval);
        let mut last_tick = Instant::now();

        loop {
            match rx.recv_timeout(self.config.tick_interval) {
                Ok(command) => {
                    match session.handle_request(
                        command,
                        &mut writer,
                        &mut self.world,
                        &self.config.world_id,
                    ) {
                        Ok(continue_running) => {
                            if !continue_running {
                                break;
                            }
                        }
                        Err(err) => {
                            if err.is_disconnect() {
                                break;
                            }
                            return Err(err);
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }

            if session.should_emit_event() && last_tick.elapsed() >= session.tick_interval {
                let step = self.world.step()?;

                if let Some(trace) = step.decision_trace {
                    if session.subscribed.contains(&ViewerStream::Events) {
                        if let Err(err) =
                            send_response(&mut writer, &ViewerResponse::DecisionTrace { trace })
                        {
                            if err.is_disconnect() {
                                break;
                            }
                            return Err(err);
                        }
                    }
                }

                if let Some(event) = step.event {
                    if session.event_allowed(&event)
                        && session.subscribed.contains(&ViewerStream::Events)
                    {
                        if let Err(err) =
                            send_response(&mut writer, &ViewerResponse::Event { event })
                        {
                            if err.is_disconnect() {
                                break;
                            }
                            return Err(err);
                        }
                    }
                    if session.subscribed.contains(&ViewerStream::Snapshot) {
                        let snapshot = self.world.snapshot();
                        if let Err(err) =
                            send_response(&mut writer, &ViewerResponse::Snapshot { snapshot })
                        {
                            if err.is_disconnect() {
                                break;
                            }
                            return Err(err);
                        }
                    }
                    session.update_metrics_from_kernel(self.world.kernel());
                    if let Err(err) = session.emit_metrics(&mut writer) {
                        if err.is_disconnect() {
                            break;
                        }
                        return Err(err);
                    }
                }

                last_tick = Instant::now();
            }
        }
        Ok(())
    }
}

struct LiveWorld {
    config: WorldConfig,
    init: WorldInitConfig,
    kernel: WorldKernel,
    decision_mode: ViewerLiveDecisionMode,
    driver: LiveDriver,
}

enum LiveDriver {
    Script(LiveScript),
    Llm(AgentRunner<LlmAgentBehavior<OpenAiChatCompletionClient>>),
}

const SEEK_STALL_LIMIT: u64 = 128;
const PROMPT_UPDATED_BY_DEFAULT: &str = "viewer_prompt_ops";

struct LiveStepResult {
    event: Option<WorldEvent>,
    decision_trace: Option<AgentDecisionTrace>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LiveSeekResult {
    reached: bool,
    current_tick: u64,
}

impl LiveWorld {
    fn new(
        config: WorldConfig,
        init: WorldInitConfig,
        decision_mode: ViewerLiveDecisionMode,
    ) -> Result<Self, ViewerLiveServerError> {
        let (kernel, _) = initialize_kernel(config.clone(), init.clone())?;
        let driver = build_driver(&kernel, decision_mode)?;
        Ok(Self {
            config,
            init,
            kernel,
            decision_mode,
            driver,
        })
    }

    fn kernel(&self) -> &WorldKernel {
        &self.kernel
    }

    fn snapshot(&self) -> WorldSnapshot {
        self.kernel.snapshot()
    }

    fn reset(&mut self) -> Result<(), ViewerLiveServerError> {
        let (kernel, _) = initialize_kernel(self.config.clone(), self.init.clone())?;
        self.kernel = kernel;
        self.driver = build_driver(&self.kernel, self.decision_mode)?;
        Ok(())
    }

    fn step(&mut self) -> Result<LiveStepResult, ViewerLiveServerError> {
        match &mut self.driver {
            LiveDriver::Script(script) => {
                if let Some(action) = script.next_action(&self.kernel) {
                    self.kernel.submit_action(action);
                }
                Ok(LiveStepResult {
                    event: self.kernel.step(),
                    decision_trace: None,
                })
            }
            LiveDriver::Llm(runner) => {
                let tick_result = runner.tick(&mut self.kernel);
                let event = tick_result
                    .as_ref()
                    .and_then(|result| result.action_result.as_ref())
                    .map(|action| action.event.clone());
                let decision_trace = tick_result.and_then(|result| result.decision_trace);
                Ok(LiveStepResult {
                    event,
                    decision_trace,
                })
            }
        }
    }

    fn seek_to_tick(&mut self, target_tick: u64) -> Result<LiveSeekResult, ViewerLiveServerError> {
        let current_tick = self.kernel.time();
        if target_tick == current_tick {
            return Ok(LiveSeekResult {
                reached: true,
                current_tick,
            });
        }

        if target_tick < current_tick {
            self.reset()?;
        }

        let mut stalled_steps = 0_u64;
        while self.kernel.time() < target_tick {
            let tick_before = self.kernel.time();
            let _ = self.step()?;
            let tick_after = self.kernel.time();

            if tick_after == tick_before {
                stalled_steps = stalled_steps.saturating_add(1);
                if stalled_steps >= SEEK_STALL_LIMIT {
                    return Ok(LiveSeekResult {
                        reached: false,
                        current_tick: tick_after,
                    });
                }
            } else {
                stalled_steps = 0;
            }
        }

        Ok(LiveSeekResult {
            reached: true,
            current_tick: self.kernel.time(),
        })
    }

    fn prompt_control_preview(
        &self,
        request: PromptControlApplyRequest,
    ) -> Result<PromptControlAck, PromptControlError> {
        let current = self.current_prompt_profile(request.agent_id.as_str())?;
        ensure_expected_prompt_version(
            request.agent_id.as_str(),
            current.version,
            request.expected_version,
        )?;

        let mut candidate = current.clone();
        apply_prompt_patch(&mut candidate, &request);
        let applied_fields = changed_prompt_fields(&current, &candidate);
        let preview_version = if applied_fields.is_empty() {
            current.version
        } else {
            current.version.saturating_add(1)
        };

        Ok(PromptControlAck {
            agent_id: request.agent_id,
            operation: PromptControlOperation::Apply,
            preview: true,
            version: preview_version,
            updated_at_tick: self.kernel.time(),
            applied_fields,
            digest: prompt_profile_digest(&candidate),
            rolled_back_to_version: None,
        })
    }

    fn prompt_control_apply(
        &mut self,
        request: PromptControlApplyRequest,
    ) -> Result<PromptControlAck, PromptControlError> {
        let current = self.current_prompt_profile(request.agent_id.as_str())?;
        ensure_expected_prompt_version(
            request.agent_id.as_str(),
            current.version,
            request.expected_version,
        )?;

        let mut candidate = current.clone();
        apply_prompt_patch(&mut candidate, &request);
        let applied_fields = changed_prompt_fields(&current, &candidate);
        let digest = prompt_profile_digest(&candidate);

        if applied_fields.is_empty() {
            return Ok(PromptControlAck {
                agent_id: request.agent_id,
                operation: PromptControlOperation::Apply,
                preview: false,
                version: current.version,
                updated_at_tick: current.updated_at_tick,
                applied_fields,
                digest,
                rolled_back_to_version: None,
            });
        }

        candidate.version = current.version.saturating_add(1);
        candidate.updated_at_tick = self.kernel.time();
        candidate.updated_by = normalize_updated_by(request.updated_by.as_deref());

        self.apply_prompt_profile_to_driver(&candidate)?;
        let digest = prompt_profile_digest(&candidate);
        self.kernel.apply_agent_prompt_profile_update(
            candidate.clone(),
            PromptUpdateOperation::Apply,
            applied_fields.clone(),
            digest.clone(),
            None,
        );

        Ok(PromptControlAck {
            agent_id: request.agent_id,
            operation: PromptControlOperation::Apply,
            preview: false,
            version: candidate.version,
            updated_at_tick: candidate.updated_at_tick,
            applied_fields,
            digest,
            rolled_back_to_version: None,
        })
    }

    fn prompt_control_rollback(
        &mut self,
        request: PromptControlRollbackRequest,
    ) -> Result<PromptControlAck, PromptControlError> {
        let current = self.current_prompt_profile(request.agent_id.as_str())?;
        ensure_expected_prompt_version(
            request.agent_id.as_str(),
            current.version,
            request.expected_version,
        )?;

        let target = if request.to_version == 0 {
            AgentPromptProfile::for_agent(request.agent_id.clone())
        } else {
            self.lookup_prompt_profile_version(request.agent_id.as_str(), request.to_version)
                .ok_or_else(|| PromptControlError {
                    code: "target_version_not_found".to_string(),
                    message: format!(
                        "prompt profile version {} not found for {}",
                        request.to_version, request.agent_id
                    ),
                    agent_id: Some(request.agent_id.clone()),
                    current_version: Some(current.version),
                })?
        };

        let mut candidate = current.clone();
        candidate.system_prompt_override = target.system_prompt_override;
        candidate.short_term_goal_override = target.short_term_goal_override;
        candidate.long_term_goal_override = target.long_term_goal_override;
        let applied_fields = changed_prompt_fields(&current, &candidate);
        if applied_fields.is_empty() {
            return Err(PromptControlError {
                code: "rollback_noop".to_string(),
                message: format!(
                    "rollback target version {} yields no prompt changes for {}",
                    request.to_version, request.agent_id
                ),
                agent_id: Some(request.agent_id),
                current_version: Some(current.version),
            });
        }

        candidate.version = current.version.saturating_add(1);
        candidate.updated_at_tick = self.kernel.time();
        candidate.updated_by = normalize_updated_by(request.updated_by.as_deref());

        self.apply_prompt_profile_to_driver(&candidate)?;
        let digest = prompt_profile_digest(&candidate);
        self.kernel.apply_agent_prompt_profile_update(
            candidate.clone(),
            PromptUpdateOperation::Rollback,
            applied_fields.clone(),
            digest.clone(),
            Some(request.to_version),
        );

        Ok(PromptControlAck {
            agent_id: request.agent_id,
            operation: PromptControlOperation::Rollback,
            preview: false,
            version: candidate.version,
            updated_at_tick: candidate.updated_at_tick,
            applied_fields,
            digest,
            rolled_back_to_version: Some(request.to_version),
        })
    }

    fn current_prompt_profile(
        &self,
        agent_id: &str,
    ) -> Result<AgentPromptProfile, PromptControlError> {
        if !self.kernel.model().agents.contains_key(agent_id) {
            return Err(PromptControlError {
                code: "agent_not_found".to_string(),
                message: format!("agent not found: {agent_id}"),
                agent_id: Some(agent_id.to_string()),
                current_version: None,
            });
        }
        Ok(self
            .kernel
            .model()
            .agent_prompt_profiles
            .get(agent_id)
            .cloned()
            .unwrap_or_else(|| AgentPromptProfile::for_agent(agent_id.to_string())))
    }

    fn lookup_prompt_profile_version(
        &self,
        agent_id: &str,
        version: u64,
    ) -> Option<AgentPromptProfile> {
        if let Some(profile) = self.kernel.model().agent_prompt_profiles.get(agent_id) {
            if profile.version == version {
                return Some(profile.clone());
            }
        }
        self.kernel.journal().iter().rev().find_map(|event| {
            let crate::simulator::WorldEventKind::AgentPromptUpdated { profile, .. } = &event.kind
            else {
                return None;
            };
            if profile.agent_id == agent_id && profile.version == version {
                Some(profile.clone())
            } else {
                None
            }
        })
    }

    fn apply_prompt_profile_to_driver(
        &mut self,
        profile: &AgentPromptProfile,
    ) -> Result<(), PromptControlError> {
        match &mut self.driver {
            LiveDriver::Script(_) => Err(PromptControlError {
                code: "llm_mode_required".to_string(),
                message: "prompt_control requires live server running with --llm".to_string(),
                agent_id: Some(profile.agent_id.clone()),
                current_version: self
                    .kernel
                    .model()
                    .agent_prompt_profiles
                    .get(&profile.agent_id)
                    .map(|entry| entry.version),
            }),
            LiveDriver::Llm(runner) => {
                let Some(agent) = runner.get_mut(profile.agent_id.as_str()) else {
                    return Err(PromptControlError {
                        code: "agent_not_registered".to_string(),
                        message: format!(
                            "agent {} is not registered in llm runner",
                            profile.agent_id
                        ),
                        agent_id: Some(profile.agent_id.clone()),
                        current_version: self
                            .kernel
                            .model()
                            .agent_prompt_profiles
                            .get(&profile.agent_id)
                            .map(|entry| entry.version),
                    });
                };
                agent.behavior.apply_prompt_overrides(
                    profile.system_prompt_override.clone(),
                    profile.short_term_goal_override.clone(),
                    profile.long_term_goal_override.clone(),
                );
                Ok(())
            }
        }
    }
}

fn normalize_updated_by(value: Option<&str>) -> String {
    value
        .map(|raw| raw.trim())
        .filter(|raw| !raw.is_empty())
        .unwrap_or(PROMPT_UPDATED_BY_DEFAULT)
        .to_string()
}

fn sanitize_patch_string(value: Option<String>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty())
}

fn apply_prompt_patch(profile: &mut AgentPromptProfile, request: &PromptControlApplyRequest) {
    if let Some(next) = &request.system_prompt_override {
        profile.system_prompt_override = sanitize_patch_string(next.clone());
    }
    if let Some(next) = &request.short_term_goal_override {
        profile.short_term_goal_override = sanitize_patch_string(next.clone());
    }
    if let Some(next) = &request.long_term_goal_override {
        profile.long_term_goal_override = sanitize_patch_string(next.clone());
    }
}

fn changed_prompt_fields(
    current: &AgentPromptProfile,
    candidate: &AgentPromptProfile,
) -> Vec<String> {
    let mut fields = Vec::new();
    if current.system_prompt_override != candidate.system_prompt_override {
        fields.push("system_prompt_override".to_string());
    }
    if current.short_term_goal_override != candidate.short_term_goal_override {
        fields.push("short_term_goal_override".to_string());
    }
    if current.long_term_goal_override != candidate.long_term_goal_override {
        fields.push("long_term_goal_override".to_string());
    }
    fields
}

fn prompt_profile_digest(profile: &AgentPromptProfile) -> String {
    let payload = serde_json::json!({
        "agent_id": profile.agent_id,
        "system_prompt_override": profile.system_prompt_override,
        "short_term_goal_override": profile.short_term_goal_override,
        "long_term_goal_override": profile.long_term_goal_override,
    });
    let bytes = serde_json::to_vec(&payload).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn ensure_expected_prompt_version(
    agent_id: &str,
    current_version: u64,
    expected_version: Option<u64>,
) -> Result<(), PromptControlError> {
    if let Some(expected) = expected_version {
        if expected != current_version {
            return Err(PromptControlError {
                code: "version_conflict".to_string(),
                message: format!(
                    "prompt profile version conflict for {}: expected {}, current {}",
                    agent_id, expected, current_version
                ),
                agent_id: Some(agent_id.to_string()),
                current_version: Some(current_version),
            });
        }
    }
    Ok(())
}

fn build_driver(
    kernel: &WorldKernel,
    decision_mode: ViewerLiveDecisionMode,
) -> Result<LiveDriver, ViewerLiveServerError> {
    match decision_mode {
        ViewerLiveDecisionMode::Script => Ok(LiveDriver::Script(LiveScript::new(kernel))),
        ViewerLiveDecisionMode::Llm => {
            let mut runner = AgentRunner::new();
            let mut agent_ids: Vec<String> = kernel.model().agents.keys().cloned().collect();
            agent_ids.sort();
            for agent_id in agent_ids {
                let mut behavior = LlmAgentBehavior::from_env(agent_id.clone())?;
                if let Some(profile) = kernel.model().agent_prompt_profiles.get(&agent_id) {
                    behavior.apply_prompt_overrides(
                        profile.system_prompt_override.clone(),
                        profile.short_term_goal_override.clone(),
                        profile.long_term_goal_override.clone(),
                    );
                }
                runner.register(behavior);
            }
            Ok(LiveDriver::Llm(runner))
        }
    }
}

#[derive(Debug, Clone)]
struct LiveScript {
    agent_id: Option<String>,
    locations: Vec<String>,
    target_index: usize,
}

impl LiveScript {
    fn new(kernel: &WorldKernel) -> Self {
        let mut agent_ids: Vec<_> = kernel.model().agents.keys().cloned().collect();
        agent_ids.sort();
        let agent_id = agent_ids.first().cloned();

        let mut locations: Vec<_> = kernel.model().locations.keys().cloned().collect();
        locations.sort();

        let target_index = if locations.len() > 1 { 1 } else { 0 };

        Self {
            agent_id,
            locations,
            target_index,
        }
    }

    fn next_action(&mut self, kernel: &WorldKernel) -> Option<Action> {
        let agent_id = self.agent_id.clone()?;
        let model = kernel.model();
        let agent = model.agents.get(&agent_id)?;
        if self.locations.is_empty() {
            return None;
        }

        let current_location_id = agent.location_id.clone();
        let current_location = model.locations.get(&current_location_id)?;

        if self.locations.len() == 1 {
            return Some(single_location_transfer(
                &agent_id,
                &current_location_id,
                agent.resources.get(ResourceKind::Electricity),
                current_location.resources.get(ResourceKind::Electricity),
            ));
        }

        if !self.locations.iter().any(|id| id == &current_location_id) {
            self.locations.push(current_location_id.clone());
            self.locations.sort();
        }

        if self.target_index >= self.locations.len() {
            self.target_index = 0;
        }

        if self.locations[self.target_index] == current_location_id {
            self.target_index = (self.target_index + 1) % self.locations.len();
        }

        let target_id = self.locations[self.target_index].clone();
        let target_location = model.locations.get(&target_id)?;
        let distance_cm = space_distance_cm(agent.pos, target_location.pos);
        let move_cost = kernel.config().movement_cost(distance_cm);
        let agent_power = agent.resources.get(ResourceKind::Electricity);

        if move_cost > 0 && agent_power < move_cost {
            let needed = move_cost - agent_power;
            let available = current_location.resources.get(ResourceKind::Electricity);
            let transfer_amount = if available > 0 {
                needed.min(available).max(1)
            } else {
                1
            };
            return Some(Action::TransferResource {
                from: ResourceOwner::Location {
                    location_id: current_location_id,
                },
                to: ResourceOwner::Agent { agent_id },
                kind: ResourceKind::Electricity,
                amount: transfer_amount,
            });
        }

        Some(Action::MoveAgent {
            agent_id,
            to: target_id,
        })
    }
}

fn single_location_transfer(
    agent_id: &str,
    location_id: &str,
    agent_power: i64,
    location_power: i64,
) -> Action {
    if location_power > 0 {
        return Action::TransferResource {
            from: ResourceOwner::Location {
                location_id: location_id.to_string(),
            },
            to: ResourceOwner::Agent {
                agent_id: agent_id.to_string(),
            },
            kind: ResourceKind::Electricity,
            amount: location_power.min(5),
        };
    }

    if agent_power > 0 {
        return Action::TransferResource {
            from: ResourceOwner::Agent {
                agent_id: agent_id.to_string(),
            },
            to: ResourceOwner::Location {
                location_id: location_id.to_string(),
            },
            kind: ResourceKind::Electricity,
            amount: agent_power.min(5),
        };
    }

    Action::TransferResource {
        from: ResourceOwner::Location {
            location_id: location_id.to_string(),
        },
        to: ResourceOwner::Agent {
            agent_id: agent_id.to_string(),
        },
        kind: ResourceKind::Electricity,
        amount: 1,
    }
}

struct ViewerLiveSession {
    subscribed: HashSet<ViewerStream>,
    event_filters: Option<HashSet<ViewerEventKind>>,
    playing: bool,
    tick_interval: Duration,
    metrics: RunnerMetrics,
}

impl ViewerLiveSession {
    fn new(tick_interval: Duration) -> Self {
        Self {
            subscribed: HashSet::new(),
            event_filters: None,
            playing: false,
            tick_interval,
            metrics: RunnerMetrics::default(),
        }
    }

    fn handle_request(
        &mut self,
        request: ViewerRequest,
        writer: &mut BufWriter<TcpStream>,
        world: &mut LiveWorld,
        world_id: &str,
    ) -> Result<bool, ViewerLiveServerError> {
        match request {
            ViewerRequest::Hello { .. } => {
                let response = ViewerResponse::HelloAck {
                    server: "agent_world".to_string(),
                    version: VIEWER_PROTOCOL_VERSION,
                    world_id: world_id.to_string(),
                };
                send_response(writer, &response)?;
            }
            ViewerRequest::Subscribe {
                streams,
                event_kinds,
            } => {
                self.subscribed = streams.into_iter().collect();
                self.event_filters = if event_kinds.is_empty() {
                    None
                } else {
                    Some(event_kinds.into_iter().collect())
                };
            }
            ViewerRequest::RequestSnapshot => {
                if self.subscribed.is_empty() || self.subscribed.contains(&ViewerStream::Snapshot) {
                    send_response(
                        writer,
                        &ViewerResponse::Snapshot {
                            snapshot: world.snapshot(),
                        },
                    )?;
                }
                if self.subscribed.contains(&ViewerStream::Metrics) {
                    self.update_metrics_from_kernel(world.kernel());
                    send_response(
                        writer,
                        &ViewerResponse::Metrics {
                            time: Some(world.kernel().time()),
                            metrics: self.metrics.clone(),
                        },
                    )?;
                }
            }
            ViewerRequest::PromptControl { command } => {
                let result = match command {
                    PromptControlCommand::Preview { request } => {
                        world.prompt_control_preview(request)
                    }
                    PromptControlCommand::Apply { request } => world.prompt_control_apply(request),
                    PromptControlCommand::Rollback { request } => {
                        world.prompt_control_rollback(request)
                    }
                };
                match result {
                    Ok(ack) => {
                        send_response(writer, &ViewerResponse::PromptControlAck { ack })?;
                    }
                    Err(error) => {
                        send_response(writer, &ViewerResponse::PromptControlError { error })?;
                    }
                }
            }
            ViewerRequest::Control { mode } => match mode {
                ViewerControl::Pause => {
                    self.playing = false;
                }
                ViewerControl::Play => {
                    self.playing = true;
                }
                ViewerControl::Step { count } => {
                    let steps = count.max(1);
                    for _ in 0..steps {
                        let step = world.step()?;

                        if let Some(trace) = step.decision_trace {
                            if self.subscribed.contains(&ViewerStream::Events) {
                                send_response(writer, &ViewerResponse::DecisionTrace { trace })?;
                            }
                        }

                        if let Some(event) = step.event {
                            if self.event_allowed(&event)
                                && self.subscribed.contains(&ViewerStream::Events)
                            {
                                send_response(writer, &ViewerResponse::Event { event })?;
                            }
                            if self.subscribed.contains(&ViewerStream::Snapshot) {
                                send_response(
                                    writer,
                                    &ViewerResponse::Snapshot {
                                        snapshot: world.snapshot(),
                                    },
                                )?;
                            }
                            self.update_metrics_from_kernel(world.kernel());
                            self.emit_metrics(writer)?;
                        }
                    }
                    self.playing = false;
                }
                ViewerControl::Seek { tick } => {
                    self.playing = false;
                    let seek_result = world.seek_to_tick(tick)?;
                    if self.subscribed.contains(&ViewerStream::Snapshot) {
                        send_response(
                            writer,
                            &ViewerResponse::Snapshot {
                                snapshot: world.snapshot(),
                            },
                        )?;
                    }
                    self.update_metrics_from_kernel(world.kernel());
                    self.emit_metrics(writer)?;
                    if !seek_result.reached {
                        send_response(
                            writer,
                            &ViewerResponse::Error {
                                message: format!(
                                    "live seek stalled at tick {} before target {}",
                                    seek_result.current_tick, tick
                                ),
                            },
                        )?;
                    }
                }
            },
        }
        Ok(true)
    }

    fn should_emit_event(&self) -> bool {
        self.playing && self.subscribed.contains(&ViewerStream::Events)
    }

    fn event_allowed(&self, event: &crate::simulator::WorldEvent) -> bool {
        match &self.event_filters {
            Some(filters) => filters.iter().any(|filter| filter.matches(&event.kind)),
            None => true,
        }
    }

    fn update_metrics_from_kernel(&mut self, kernel: &WorldKernel) {
        self.metrics = metrics_from_kernel(kernel);
    }

    fn emit_metrics(&self, writer: &mut BufWriter<TcpStream>) -> Result<(), ViewerLiveServerError> {
        if self.subscribed.contains(&ViewerStream::Metrics) {
            send_response(
                writer,
                &ViewerResponse::Metrics {
                    time: Some(self.metrics.total_ticks),
                    metrics: self.metrics.clone(),
                },
            )?;
        }
        Ok(())
    }
}

fn metrics_from_kernel(kernel: &WorldKernel) -> RunnerMetrics {
    let total_ticks = kernel.time();
    let total_actions = kernel.journal().len() as u64;
    let actions_per_tick = if total_ticks > 0 {
        total_actions as f64 / total_ticks as f64
    } else {
        0.0
    };
    RunnerMetrics {
        total_ticks,
        total_agents: kernel.model().agents.len(),
        agents_active: kernel.model().agents.len(),
        agents_quota_exhausted: 0,
        total_actions,
        total_decisions: 0,
        actions_per_tick,
        decisions_per_tick: 0.0,
        success_rate: 0.0,
    }
}

fn read_requests(stream: TcpStream, tx: mpsc::Sender<ViewerRequest>) {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match serde_json::from_str::<ViewerRequest>(trimmed) {
                    Ok(request) => {
                        if tx.send(request).is_err() {
                            break;
                        }
                    }
                    Err(_) => {}
                }
            }
            Err(_) => break,
        }
    }
}

fn send_response(
    writer: &mut BufWriter<TcpStream>,
    response: &ViewerResponse,
) -> Result<(), ViewerLiveServerError> {
    let payload = serde_json::to_string(response)
        .map_err(|err| ViewerLiveServerError::Serde(err.to_string()))?;
    writer.write_all(payload.as_bytes())?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

fn is_disconnect_error(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::BrokenPipe
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::NotConnected
    )
}

#[cfg(test)]
mod tests;
