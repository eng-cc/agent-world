use std::collections::HashSet;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use agent_world_node::NodeRuntime;

use crate::geometry::space_distance_cm;
use crate::simulator::{
    initialize_kernel, Action, ActionResult, ActionSubmitter, AgentDecision, AgentDecisionTrace,
    AgentPromptProfile, AgentRunner, LlmAgentBehavior, LlmAgentBuildError,
    OpenAiChatCompletionClient, PromptUpdateOperation, ResourceKind, ResourceOwner, RunnerMetrics,
    WorldConfig, WorldEvent, WorldEventKind, WorldInitConfig, WorldInitError, WorldKernel,
    WorldScenario, WorldSnapshot,
};

#[path = "live/consensus_bridge.rs"]
mod consensus_bridge;
use consensus_bridge::*;
#[path = "live/seek.rs"]
mod seek;

use super::auth::{
    verify_agent_chat_auth_proof, verify_prompt_control_apply_auth_proof,
    verify_prompt_control_rollback_auth_proof, PromptControlAuthIntent,
};
use super::protocol::{
    viewer_event_kind_matches, AgentChatAck, AgentChatError, AgentChatRequest, PromptControlAck,
    PromptControlApplyRequest, PromptControlCommand, PromptControlError, PromptControlOperation,
    PromptControlRollbackRequest, ViewerControl, ViewerEventKind, ViewerRequest, ViewerResponse,
    ViewerStream, VIEWER_PROTOCOL_VERSION,
};
#[path = "live/live_helpers.rs"]
mod live_helpers;
use live_helpers::*;

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
    pub consensus_gate_max_tick: Option<Arc<AtomicU64>>,
    pub consensus_runtime: Option<Arc<Mutex<NodeRuntime>>>,
}

impl ViewerLiveServerConfig {
    pub fn new(scenario: WorldScenario) -> Self {
        Self {
            bind_addr: "127.0.0.1:5010".to_string(),
            tick_interval: Duration::from_millis(200),
            world_id: format!("live-{}", scenario.as_str()),
            scenario,
            decision_mode: ViewerLiveDecisionMode::Script,
            consensus_gate_max_tick: None,
            consensus_runtime: None,
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

    pub fn with_consensus_gate_max_tick(mut self, max_tick: Arc<AtomicU64>) -> Self {
        self.consensus_gate_max_tick = Some(max_tick);
        self
    }

    pub fn with_consensus_runtime(mut self, runtime: Arc<Mutex<NodeRuntime>>) -> Self {
        self.consensus_runtime = Some(runtime);
        self
    }
}

#[derive(Debug)]
pub enum ViewerLiveServerError {
    Io(io::Error),
    Serde(String),
    Init(WorldInitError),
    LlmBuild(LlmAgentBuildError),
    Node(String),
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
        let world = if let Some(max_tick) = config.consensus_gate_max_tick.clone() {
            LiveWorld::new_with_consensus_gate(
                WorldConfig::default(),
                init,
                config.decision_mode,
                Some(max_tick),
                config.consensus_runtime.clone(),
            )?
        } else {
            LiveWorld::new_with_consensus_gate(
                WorldConfig::default(),
                init,
                config.decision_mode,
                None,
                config.consensus_runtime.clone(),
            )?
        };
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
                if !self.world.can_step_for_consensus() {
                    continue;
                }
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
                }

                session.update_metrics_from_kernel(self.world.kernel());
                if let Err(err) = session.emit_metrics(&mut writer) {
                    if err.is_disconnect() {
                        break;
                    }
                    return Err(err);
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
    consensus_gate_max_tick: Option<Arc<AtomicU64>>,
    consensus_bridge: Option<LiveConsensusBridge>,
}

enum LiveDriver {
    Script(LiveScript),
    Llm(AgentRunner<LlmAgentBehavior<OpenAiChatCompletionClient>>),
}

struct LiveStepResult {
    event: Option<WorldEvent>,
    decision_trace: Option<AgentDecisionTrace>,
}

impl LiveWorld {
    #[cfg(test)]
    fn new(
        config: WorldConfig,
        init: WorldInitConfig,
        decision_mode: ViewerLiveDecisionMode,
    ) -> Result<Self, ViewerLiveServerError> {
        Self::new_with_consensus_gate(config, init, decision_mode, None, None)
    }

    fn new_with_consensus_gate(
        config: WorldConfig,
        init: WorldInitConfig,
        decision_mode: ViewerLiveDecisionMode,
        consensus_gate_max_tick: Option<Arc<AtomicU64>>,
        consensus_runtime: Option<Arc<Mutex<NodeRuntime>>>,
    ) -> Result<Self, ViewerLiveServerError> {
        let (kernel, _) = initialize_kernel(config.clone(), init.clone())?;
        let driver = build_driver(&kernel, decision_mode)?;
        Ok(Self {
            config,
            init,
            kernel,
            decision_mode,
            driver,
            consensus_gate_max_tick,
            consensus_bridge: consensus_runtime.map(LiveConsensusBridge::new),
        })
    }

    fn kernel(&self) -> &WorldKernel {
        &self.kernel
    }

    fn snapshot(&self) -> WorldSnapshot {
        self.kernel.snapshot()
    }

    fn can_step_for_consensus(&self) -> bool {
        if self.consensus_bridge.is_some() {
            return true;
        }
        let Some(max_tick) = self.consensus_gate_max_tick.as_ref() else {
            return true;
        };
        self.kernel.time() < max_tick.load(Ordering::SeqCst)
    }

    fn reset(&mut self) -> Result<(), ViewerLiveServerError> {
        let (kernel, _) = initialize_kernel(self.config.clone(), self.init.clone())?;
        self.kernel = kernel;
        self.driver = build_driver(&self.kernel, self.decision_mode)?;
        if let Some(bridge) = self.consensus_bridge.as_mut() {
            bridge.reset_pending();
        }
        Ok(())
    }

    fn step(&mut self) -> Result<LiveStepResult, ViewerLiveServerError> {
        if self.consensus_bridge.is_some() {
            return self.step_via_consensus();
        }
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
                sync_llm_runner_long_term_memory(&mut self.kernel, runner);
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

    fn prompt_control_preview(
        &mut self,
        request: PromptControlApplyRequest,
    ) -> Result<PromptControlAck, PromptControlError> {
        let player_id =
            normalize_required_player_id(request.player_id.as_str(), request.agent_id.as_str())?;
        let public_key = normalize_optional_public_key(request.public_key.as_deref());
        self.verify_and_consume_prompt_control_apply_auth(
            PromptControlAuthIntent::Preview,
            &request,
        )?;
        ensure_agent_player_access(
            self.kernel(),
            request.agent_id.as_str(),
            player_id.as_str(),
            public_key.as_deref(),
        )?;
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
        let player_id =
            normalize_required_player_id(request.player_id.as_str(), request.agent_id.as_str())?;
        let public_key = normalize_optional_public_key(request.public_key.as_deref());
        self.verify_and_consume_prompt_control_apply_auth(
            PromptControlAuthIntent::Apply,
            &request,
        )?;
        ensure_agent_player_access(
            self.kernel(),
            request.agent_id.as_str(),
            player_id.as_str(),
            public_key.as_deref(),
        )?;
        let current = self.current_prompt_profile(request.agent_id.as_str())?;
        ensure_expected_prompt_version(
            request.agent_id.as_str(),
            current.version,
            request.expected_version,
        )?;
        ensure_updated_by_matches_player(
            request.updated_by.as_deref(),
            player_id.as_str(),
            request.agent_id.as_str(),
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
        candidate.updated_by = player_id.clone();

        self.apply_prompt_profile_to_driver(&candidate)?;
        self.bind_agent_player_access(
            request.agent_id.as_str(),
            player_id.as_str(),
            public_key.as_deref(),
        )?;
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
        let player_id =
            normalize_required_player_id(request.player_id.as_str(), request.agent_id.as_str())?;
        let public_key = normalize_optional_public_key(request.public_key.as_deref());
        self.verify_and_consume_prompt_control_rollback_auth(&request)?;
        ensure_agent_player_access(
            self.kernel(),
            request.agent_id.as_str(),
            player_id.as_str(),
            public_key.as_deref(),
        )?;
        let current = self.current_prompt_profile(request.agent_id.as_str())?;
        ensure_expected_prompt_version(
            request.agent_id.as_str(),
            current.version,
            request.expected_version,
        )?;
        ensure_updated_by_matches_player(
            request.updated_by.as_deref(),
            player_id.as_str(),
            request.agent_id.as_str(),
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
        candidate.updated_by = player_id.clone();

        self.apply_prompt_profile_to_driver(&candidate)?;
        self.bind_agent_player_access(
            request.agent_id.as_str(),
            player_id.as_str(),
            public_key.as_deref(),
        )?;
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

    fn agent_chat(&mut self, request: AgentChatRequest) -> Result<AgentChatAck, AgentChatError> {
        let player_id = request
            .player_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);
        let Some(player_id) = player_id else {
            return Err(AgentChatError {
                code: "player_id_required".to_string(),
                message: "agent_chat requires non-empty player_id".to_string(),
                agent_id: Some(request.agent_id),
            });
        };
        let public_key = normalize_optional_public_key(request.public_key.as_deref());
        let message = request.message.trim().to_string();
        if message.is_empty() {
            return Err(AgentChatError {
                code: "empty_message".to_string(),
                message: "chat message cannot be empty".to_string(),
                agent_id: Some(request.agent_id),
            });
        }
        self.verify_and_consume_agent_chat_auth(&request)?;

        if matches!(self.driver, LiveDriver::Script(_)) {
            return Err(AgentChatError {
                code: "llm_mode_required".to_string(),
                message: "agent chat requires live server running with --llm".to_string(),
                agent_id: Some(request.agent_id),
            });
        }

        self.bind_agent_player_access_for_chat(
            request.agent_id.as_str(),
            player_id.as_str(),
            public_key.as_deref(),
        )?;
        let runner = match &mut self.driver {
            LiveDriver::Llm(runner) => runner,
            LiveDriver::Script(_) => unreachable!("script mode handled above"),
        };
        let Some(agent) = runner.get_mut(request.agent_id.as_str()) else {
            return Err(AgentChatError {
                code: "agent_not_registered".to_string(),
                message: format!("agent {} is not registered in llm runner", request.agent_id),
                agent_id: Some(request.agent_id),
            });
        };
        if !agent
            .behavior
            .push_player_message(self.kernel.time(), message.as_str())
        {
            return Err(AgentChatError {
                code: "empty_message".to_string(),
                message: "chat message cannot be empty".to_string(),
                agent_id: Some(request.agent_id),
            });
        }
        Ok(AgentChatAck {
            agent_id: request.agent_id,
            accepted_at_tick: self.kernel.time(),
            message_len: message.chars().count(),
            player_id: Some(player_id),
        })
    }

    fn verify_and_consume_prompt_control_apply_auth(
        &mut self,
        intent: PromptControlAuthIntent,
        request: &PromptControlApplyRequest,
    ) -> Result<(), PromptControlError> {
        let Some(auth) = request.auth.as_ref() else {
            return Err(PromptControlError {
                code: "auth_proof_required".to_string(),
                message: "prompt_control requires auth proof".to_string(),
                agent_id: Some(request.agent_id.clone()),
                current_version: self.current_prompt_version(request.agent_id.as_str()),
            });
        };
        let verified =
            verify_prompt_control_apply_auth_proof(intent, request, auth).map_err(|message| {
                PromptControlError {
                    code: map_auth_verify_error_code(message.as_str()).to_string(),
                    message,
                    agent_id: Some(request.agent_id.clone()),
                    current_version: self.current_prompt_version(request.agent_id.as_str()),
                }
            })?;
        self.kernel
            .consume_player_auth_nonce(verified.player_id.as_str(), verified.nonce)
            .map_err(|message| PromptControlError {
                code: "auth_nonce_replay".to_string(),
                message,
                agent_id: Some(request.agent_id.clone()),
                current_version: self.current_prompt_version(request.agent_id.as_str()),
            })?;
        Ok(())
    }

    fn verify_and_consume_prompt_control_rollback_auth(
        &mut self,
        request: &PromptControlRollbackRequest,
    ) -> Result<(), PromptControlError> {
        let Some(auth) = request.auth.as_ref() else {
            return Err(PromptControlError {
                code: "auth_proof_required".to_string(),
                message: "prompt_control rollback requires auth proof".to_string(),
                agent_id: Some(request.agent_id.clone()),
                current_version: self.current_prompt_version(request.agent_id.as_str()),
            });
        };
        let verified =
            verify_prompt_control_rollback_auth_proof(request, auth).map_err(|message| {
                PromptControlError {
                    code: map_auth_verify_error_code(message.as_str()).to_string(),
                    message,
                    agent_id: Some(request.agent_id.clone()),
                    current_version: self.current_prompt_version(request.agent_id.as_str()),
                }
            })?;
        self.kernel
            .consume_player_auth_nonce(verified.player_id.as_str(), verified.nonce)
            .map_err(|message| PromptControlError {
                code: "auth_nonce_replay".to_string(),
                message,
                agent_id: Some(request.agent_id.clone()),
                current_version: self.current_prompt_version(request.agent_id.as_str()),
            })?;
        Ok(())
    }

    fn verify_and_consume_agent_chat_auth(
        &mut self,
        request: &AgentChatRequest,
    ) -> Result<(), AgentChatError> {
        let Some(auth) = request.auth.as_ref() else {
            return Err(AgentChatError {
                code: "auth_proof_required".to_string(),
                message: "agent_chat requires auth proof".to_string(),
                agent_id: Some(request.agent_id.clone()),
            });
        };
        let verified =
            verify_agent_chat_auth_proof(request, auth).map_err(|message| AgentChatError {
                code: map_auth_verify_error_code(message.as_str()).to_string(),
                message,
                agent_id: Some(request.agent_id.clone()),
            })?;
        self.kernel
            .consume_player_auth_nonce(verified.player_id.as_str(), verified.nonce)
            .map_err(|message| AgentChatError {
                code: "auth_nonce_replay".to_string(),
                message,
                agent_id: Some(request.agent_id.clone()),
            })?;
        Ok(())
    }

    fn current_prompt_version(&self, agent_id: &str) -> Option<u64> {
        self.kernel
            .model()
            .agent_prompt_profiles
            .get(agent_id)
            .map(|profile| profile.version)
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

    fn bind_agent_player_access(
        &mut self,
        agent_id: &str,
        player_id: &str,
        public_key: Option<&str>,
    ) -> Result<(), PromptControlError> {
        ensure_agent_player_access(self.kernel(), agent_id, player_id, public_key)?;
        let needs_bind = self.kernel.player_binding_for_agent(agent_id).is_none()
            || (public_key.is_some()
                && self.kernel.public_key_binding_for_agent(agent_id).is_none());
        if needs_bind {
            self.kernel
                .bind_agent_player(agent_id, player_id, public_key)
                .map_err(|message| PromptControlError {
                    code: "player_bind_failed".to_string(),
                    message,
                    agent_id: Some(agent_id.to_string()),
                    current_version: self
                        .kernel
                        .model()
                        .agent_prompt_profiles
                        .get(agent_id)
                        .map(|profile| profile.version),
                })?;
        }
        Ok(())
    }

    fn bind_agent_player_access_for_chat(
        &mut self,
        agent_id: &str,
        player_id: &str,
        public_key: Option<&str>,
    ) -> Result<(), AgentChatError> {
        let mapped = ensure_agent_player_access(self.kernel(), agent_id, player_id, public_key)
            .map_err(|err| AgentChatError {
                code: "agent_control_forbidden".to_string(),
                message: err.message,
                agent_id: err.agent_id,
            });
        mapped?;
        let needs_bind = self.kernel.player_binding_for_agent(agent_id).is_none()
            || (public_key.is_some()
                && self.kernel.public_key_binding_for_agent(agent_id).is_none());
        if needs_bind {
            self.kernel
                .bind_agent_player(agent_id, player_id, public_key)
                .map_err(|message| AgentChatError {
                    code: "player_bind_failed".to_string(),
                    message,
                    agent_id: Some(agent_id.to_string()),
                })?;
        }
        Ok(())
    }
}

fn map_auth_verify_error_code(message: &str) -> &'static str {
    if message.contains("nonce") {
        return "auth_nonce_invalid";
    }
    if message.contains("signature") || message.contains("awviewauth:v1") {
        return "auth_signature_invalid";
    }
    if message.contains("player_id") || message.contains("public_key") {
        return "auth_claim_mismatch";
    }
    if message.contains("required") || message.contains("empty") {
        return "auth_claim_invalid";
    }
    "auth_invalid"
}

#[derive(Debug, Clone)]
struct LiveScript {
    agent_id: Option<String>,
    locations: Vec<String>,
    target_index: usize,
}
