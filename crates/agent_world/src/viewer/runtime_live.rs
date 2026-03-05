use std::collections::{HashSet, VecDeque};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;

use crate::geometry::{space_distance_cm, GeoPos};
use crate::runtime::{
    Action as RuntimeAction, DomainEvent as RuntimeDomainEvent,
    RejectReason as RuntimeRejectReason, World as RuntimeWorld, WorldError as RuntimeWorldError,
    WorldEvent as RuntimeWorldEvent, WorldEventBody as RuntimeWorldEventBody,
};
use crate::simulator::{
    build_world_model, Agent, ChunkRuntimeConfig, Location, RejectReason as SimulatorRejectReason,
    ResourceKind, ResourceOwner, RunnerMetrics, WorldConfig, WorldEvent, WorldEventKind,
    WorldInitConfig, WorldModel, WorldScenario, WorldSnapshot, CHUNK_GENERATION_SCHEMA_VERSION,
    SNAPSHOT_VERSION,
};

use super::live::ViewerLiveDecisionMode;
use super::protocol::{
    viewer_event_kind_matches, ControlCompletionAck, ControlCompletionStatus, ViewerControl,
    ViewerControlProfile, ViewerEventKind, ViewerRequest, ViewerResponse, ViewerStream,
    VIEWER_PROTOCOL_VERSION,
};
#[path = "runtime_live/control_plane.rs"]
mod control_plane;
use control_plane::RuntimeLlmSidecar;

#[derive(Debug, Clone)]
pub struct ViewerRuntimeLiveServerConfig {
    pub bind_addr: String,
    pub scenario: WorldScenario,
    pub world_id: String,
    pub decision_mode: ViewerLiveDecisionMode,
}

impl ViewerRuntimeLiveServerConfig {
    pub fn new(scenario: WorldScenario) -> Self {
        Self {
            bind_addr: "127.0.0.1:5010".to_string(),
            world_id: format!("live-runtime-{}", scenario.as_str()),
            scenario,
            decision_mode: ViewerLiveDecisionMode::Script,
        }
    }

    pub fn with_bind_addr(mut self, addr: impl Into<String>) -> Self {
        self.bind_addr = addr.into();
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
pub enum ViewerRuntimeLiveServerError {
    Io(io::Error),
    Serde(String),
    Init(String),
    Runtime(RuntimeWorldError),
}

impl From<io::Error> for ViewerRuntimeLiveServerError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<RuntimeWorldError> for ViewerRuntimeLiveServerError {
    fn from(err: RuntimeWorldError) -> Self {
        Self::Runtime(err)
    }
}

pub struct ViewerRuntimeLiveServer {
    config: ViewerRuntimeLiveServerConfig,
    world: RuntimeWorld,
    snapshot_config: WorldConfig,
    script: RuntimeLiveScript,
    llm_sidecar: RuntimeLlmSidecar,
    pending_virtual_events: VecDeque<WorldEvent>,
    next_virtual_event_id: u64,
}

impl ViewerRuntimeLiveServer {
    pub fn new(
        config: ViewerRuntimeLiveServerConfig,
    ) -> Result<Self, ViewerRuntimeLiveServerError> {
        let (world, snapshot_config) =
            bootstrap_runtime_world(config.scenario).map_err(ViewerRuntimeLiveServerError::Init)?;
        let llm_sidecar = RuntimeLlmSidecar::new(config.decision_mode, &world);
        let next_virtual_event_id = latest_runtime_event_seq(&world).saturating_add(1).max(1);
        Ok(Self {
            config,
            world,
            snapshot_config,
            script: RuntimeLiveScript::default(),
            llm_sidecar,
            pending_virtual_events: VecDeque::new(),
            next_virtual_event_id,
        })
    }

    pub fn run(&mut self) -> Result<(), ViewerRuntimeLiveServerError> {
        let listener = TcpListener::bind(&self.config.bind_addr)?;
        for incoming in listener.incoming() {
            let stream = incoming?;
            if let Err(err) = self.serve_stream(stream) {
                eprintln!("viewer runtime live server error: {err:?}");
            }
        }
        Ok(())
    }

    pub fn run_once(&mut self) -> Result<(), ViewerRuntimeLiveServerError> {
        let listener = TcpListener::bind(&self.config.bind_addr)?;
        let (stream, _) = listener.accept()?;
        self.serve_stream(stream)
    }

    fn serve_stream(&mut self, stream: TcpStream) -> Result<(), ViewerRuntimeLiveServerError> {
        stream.set_nodelay(true)?;
        stream.set_read_timeout(Some(Duration::from_millis(50)))?;

        let reader_stream = stream.try_clone()?;
        let mut reader = BufReader::new(reader_stream);
        let mut writer = BufWriter::new(stream);
        let mut session = RuntimeLiveSession::new();

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => return Ok(()),
                Ok(_) => {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        match serde_json::from_str::<ViewerRequest>(trimmed) {
                            Ok(request) => {
                                self.handle_request(request, &mut session, &mut writer)?;
                            }
                            Err(_) => {}
                        }
                    }
                }
                Err(err) if is_timeout_error(&err) => {}
                Err(err) => return Err(ViewerRuntimeLiveServerError::Io(err)),
            }

            if session.playing {
                self.advance_runtime(&mut session, &mut writer, 1, None, false)?;
            }
        }
    }

    fn handle_request(
        &mut self,
        request: ViewerRequest,
        session: &mut RuntimeLiveSession,
        writer: &mut BufWriter<TcpStream>,
    ) -> Result<(), ViewerRuntimeLiveServerError> {
        match request {
            ViewerRequest::Hello { .. } => {
                send_response(
                    writer,
                    &ViewerResponse::HelloAck {
                        server: "agent_world".to_string(),
                        version: VIEWER_PROTOCOL_VERSION,
                        world_id: self.config.world_id.clone(),
                        control_profile: ViewerControlProfile::Live,
                    },
                )?;
            }
            ViewerRequest::Subscribe {
                streams,
                event_kinds,
            } => {
                session.subscribed = streams.into_iter().collect();
                session.event_filters = if event_kinds.is_empty() {
                    None
                } else {
                    Some(event_kinds.into_iter().collect())
                };
            }
            ViewerRequest::RequestSnapshot => {
                if session.subscribed.is_empty()
                    || session.subscribed.contains(&ViewerStream::Snapshot)
                {
                    let snapshot = self.compat_snapshot();
                    send_response(writer, &ViewerResponse::Snapshot { snapshot })?;
                }
                if session.subscribed.contains(&ViewerStream::Metrics) {
                    session.metrics = runtime_metrics(&self.world);
                    send_response(
                        writer,
                        &ViewerResponse::Metrics {
                            time: Some(self.world.state().time),
                            metrics: session.metrics.clone(),
                        },
                    )?;
                }
            }
            ViewerRequest::PlaybackControl { mode, request_id } => {
                self.apply_control_mode(ViewerControl::from(mode), request_id, session, writer)?;
            }
            ViewerRequest::LiveControl { mode, request_id } => {
                self.apply_control_mode(ViewerControl::from(mode), request_id, session, writer)?;
            }
            ViewerRequest::Control { mode, request_id } => {
                self.apply_control_mode(mode, request_id, session, writer)?;
            }
            ViewerRequest::PromptControl { command } => match self.handle_prompt_control(command) {
                Ok(ack) => {
                    send_response(writer, &ViewerResponse::PromptControlAck { ack })?;
                }
                Err(error) => {
                    send_response(writer, &ViewerResponse::PromptControlError { error })?;
                }
            },
            ViewerRequest::AgentChat { request } => match self.handle_agent_chat(request) {
                Ok(ack) => {
                    send_response(writer, &ViewerResponse::AgentChatAck { ack })?;
                }
                Err(error) => {
                    send_response(writer, &ViewerResponse::AgentChatError { error })?;
                }
            },
        }
        Ok(())
    }

    fn apply_control_mode(
        &mut self,
        mode: ViewerControl,
        request_id: Option<u64>,
        session: &mut RuntimeLiveSession,
        writer: &mut BufWriter<TcpStream>,
    ) -> Result<(), ViewerRuntimeLiveServerError> {
        match mode {
            ViewerControl::Pause => {
                session.playing = false;
            }
            ViewerControl::Play => {
                session.playing = true;
            }
            ViewerControl::Step { count } => {
                session.playing = false;
                self.advance_runtime(session, writer, count.max(1), request_id, true)?;
            }
            ViewerControl::Seek { tick } => {
                session.playing = false;
                eprintln!(
                    "viewer runtime live: ignore seek control in live mode (target_tick={tick})"
                );
            }
        }
        Ok(())
    }

    fn advance_runtime(
        &mut self,
        session: &mut RuntimeLiveSession,
        writer: &mut BufWriter<TcpStream>,
        step_count: usize,
        request_id: Option<u64>,
        emit_while_paused: bool,
    ) -> Result<(), ViewerRuntimeLiveServerError> {
        let baseline_logical_time = self.world.state().time;
        let baseline_event_seq = latest_runtime_event_seq(&self.world);

        for _ in 0..step_count.max(1) {
            match self.config.decision_mode {
                ViewerLiveDecisionMode::Script => self.script.enqueue(&mut self.world),
                ViewerLiveDecisionMode::Llm => self.enqueue_llm_action_from_sidecar(),
            }
            let journal_start = self.world.journal().events.len();
            self.world.step()?;

            let new_events: Vec<_> = self.world.journal().events[journal_start..].to_vec();
            let mut mapped_events = Vec::new();
            for runtime_event in &new_events {
                if let Some(event) = map_runtime_event(runtime_event, &self.snapshot_config) {
                    mapped_events.push(event);
                }
            }
            mapped_events.extend(self.pending_virtual_events.drain(..));

            if session.subscribed.contains(&ViewerStream::Events)
                && (emit_while_paused || session.playing)
            {
                for event in &mapped_events {
                    if session.event_allowed(event) {
                        send_response(
                            writer,
                            &ViewerResponse::Event {
                                event: event.clone(),
                            },
                        )?;
                    }
                }
            }

            if session.subscribed.contains(&ViewerStream::Snapshot) {
                let snapshot = self.compat_snapshot();
                send_response(writer, &ViewerResponse::Snapshot { snapshot })?;
            }

            session.metrics = runtime_metrics(&self.world);
            if session.subscribed.contains(&ViewerStream::Metrics) {
                send_response(
                    writer,
                    &ViewerResponse::Metrics {
                        time: Some(self.world.state().time),
                        metrics: session.metrics.clone(),
                    },
                )?;
            }
        }

        if let Some(request_id) = request_id {
            let delta_logical_time = self
                .world
                .state()
                .time
                .saturating_sub(baseline_logical_time);
            let delta_event_seq =
                latest_runtime_event_seq(&self.world).saturating_sub(baseline_event_seq);
            let status = if delta_logical_time > 0 || delta_event_seq > 0 {
                ControlCompletionStatus::Advanced
            } else {
                ControlCompletionStatus::TimeoutNoProgress
            };
            send_response(
                writer,
                &ViewerResponse::ControlCompletionAck {
                    ack: ControlCompletionAck {
                        request_id,
                        status,
                        delta_logical_time,
                        delta_event_seq,
                    },
                },
            )?;
        }

        Ok(())
    }

    fn compat_snapshot(&self) -> WorldSnapshot {
        let runtime_snapshot = self.world.snapshot();
        WorldSnapshot {
            version: SNAPSHOT_VERSION,
            chunk_generation_schema_version: CHUNK_GENERATION_SCHEMA_VERSION,
            time: self.world.state().time,
            config: self.snapshot_config.clone(),
            model: runtime_state_to_simulator_model(self.world.state(), &self.llm_sidecar),
            chunk_runtime: ChunkRuntimeConfig::default(),
            next_event_id: runtime_snapshot.last_event_id.saturating_add(1).max(1),
            next_action_id: runtime_snapshot.next_action_id.max(1),
            pending_actions: Vec::new(),
            journal_len: runtime_snapshot.journal_len,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct RuntimeLiveScript {
    phase: u8,
    move_direction: i64,
}

impl RuntimeLiveScript {
    fn enqueue(&mut self, world: &mut RuntimeWorld) {
        let mut agent_ids: Vec<String> = world.state().agents.keys().cloned().collect();
        agent_ids.sort();

        if agent_ids.is_empty() {
            world.submit_action(RuntimeAction::RegisterAgent {
                agent_id: "runtime-agent-0".to_string(),
                pos: GeoPos::new(0.0, 0.0, 0.0),
            });
            world.submit_action(RuntimeAction::RegisterAgent {
                agent_id: "runtime-agent-1".to_string(),
                pos: GeoPos::new(0.0, 0.0, 0.0),
            });
            return;
        }

        let phase = self.phase;
        self.phase = self.phase.wrapping_add(1) % 4;

        match phase {
            0 => {
                let first = &agent_ids[0];
                let Some(from_pos) = world.state().agents.get(first).map(|cell| cell.state.pos)
                else {
                    return;
                };
                if self.move_direction == 0 {
                    self.move_direction = 1;
                } else {
                    self.move_direction = -self.move_direction;
                }
                let delta_cm = (self.move_direction * 1_000) as f64;
                world.submit_action(RuntimeAction::MoveAgent {
                    agent_id: first.clone(),
                    to: GeoPos::new(from_pos.x_cm + delta_cm, from_pos.y_cm, from_pos.z_cm),
                });
            }
            1 => {
                if agent_ids.len() < 2 {
                    world.submit_action(RuntimeAction::MoveAgent {
                        agent_id: "missing-agent".to_string(),
                        to: GeoPos::new(0.0, 0.0, 0.0),
                    });
                    return;
                }
                let first = &agent_ids[0];
                let second = &agent_ids[1];
                let Some(target) = world.state().agents.get(first).map(|cell| cell.state.pos)
                else {
                    return;
                };
                world.submit_action(RuntimeAction::MoveAgent {
                    agent_id: second.clone(),
                    to: target,
                });
            }
            2 => {
                if agent_ids.len() < 2 {
                    world.submit_action(RuntimeAction::MoveAgent {
                        agent_id: "missing-agent".to_string(),
                        to: GeoPos::new(0.0, 0.0, 0.0),
                    });
                    return;
                }
                let from = &agent_ids[0];
                let to = &agent_ids[1];
                let _ = world.set_agent_resource_balance(from, ResourceKind::Electricity, 64);
                let _ = world.set_agent_resource_balance(to, ResourceKind::Electricity, 64);
                world.submit_action(RuntimeAction::EmitResourceTransfer {
                    from_agent_id: from.clone(),
                    to_agent_id: to.clone(),
                    kind: ResourceKind::Electricity,
                    amount: 1,
                });
            }
            _ => {
                world.submit_action(RuntimeAction::MoveAgent {
                    agent_id: "missing-agent".to_string(),
                    to: GeoPos::new(0.0, 0.0, 0.0),
                });
            }
        }
    }
}

struct RuntimeLiveSession {
    subscribed: HashSet<ViewerStream>,
    event_filters: Option<HashSet<ViewerEventKind>>,
    playing: bool,
    metrics: RunnerMetrics,
}

impl RuntimeLiveSession {
    fn new() -> Self {
        Self {
            subscribed: HashSet::new(),
            event_filters: None,
            playing: false,
            metrics: RunnerMetrics::default(),
        }
    }

    fn event_allowed(&self, event: &WorldEvent) -> bool {
        match &self.event_filters {
            Some(filters) => filters
                .iter()
                .any(|filter| viewer_event_kind_matches(filter, &event.kind)),
            None => true,
        }
    }
}

fn bootstrap_runtime_world(scenario: WorldScenario) -> Result<(RuntimeWorld, WorldConfig), String> {
    let config = WorldConfig::default();
    let init = WorldInitConfig::from_scenario(scenario, &config);
    let (model, _) = build_world_model(&config, &init)
        .map_err(|err| format!("runtime live bootstrap build_world_model failed: {err:?}"))?;

    let mut world = RuntimeWorld::new();
    let mut seed_agents: Vec<(String, GeoPos, i64, i64)> = model
        .agents
        .iter()
        .map(|(agent_id, agent)| {
            (
                agent_id.clone(),
                agent.pos,
                agent.resources.get(ResourceKind::Electricity),
                agent.resources.get(ResourceKind::Data),
            )
        })
        .collect();
    seed_agents.sort_by(|left, right| left.0.cmp(&right.0));

    if seed_agents.is_empty() {
        seed_agents.push((
            "runtime-agent-0".to_string(),
            GeoPos::new(0.0, 0.0, 0.0),
            32,
            8,
        ));
        seed_agents.push((
            "runtime-agent-1".to_string(),
            GeoPos::new(0.0, 0.0, 0.0),
            32,
            8,
        ));
    }

    for (agent_id, pos, _, _) in &seed_agents {
        world.submit_action(RuntimeAction::RegisterAgent {
            agent_id: agent_id.clone(),
            pos: *pos,
        });
    }

    if world.pending_actions_len() > 0 {
        world
            .step()
            .map_err(|err| format!("runtime live bootstrap register step failed: {err:?}"))?;
    }

    for (agent_id, electricity, data) in world
        .state()
        .agents
        .keys()
        .cloned()
        .map(|agent_id| {
            let maybe_seed = seed_agents
                .iter()
                .find(|entry| entry.0 == agent_id)
                .cloned();
            match maybe_seed {
                Some((_, _, electricity, data)) => (agent_id, electricity.max(32), data.max(8)),
                None => (agent_id, 32, 8),
            }
        })
        .collect::<Vec<_>>()
    {
        world
            .set_agent_resource_balance(agent_id.as_str(), ResourceKind::Electricity, electricity)
            .map_err(|err| {
                format!(
                    "runtime live bootstrap set electricity failed agent={} err={err:?}",
                    agent_id
                )
            })?;
        world
            .set_agent_resource_balance(agent_id.as_str(), ResourceKind::Data, data)
            .map_err(|err| {
                format!(
                    "runtime live bootstrap set data failed agent={} err={err:?}",
                    agent_id
                )
            })?;
    }

    Ok((world, config))
}

fn runtime_state_to_simulator_model(
    state: &crate::runtime::WorldState,
    sidecar: &RuntimeLlmSidecar,
) -> WorldModel {
    let mut model = WorldModel::default();

    for (agent_id, cell) in &state.agents {
        let location_id = location_id_for_pos(cell.state.pos);
        model
            .locations
            .entry(location_id.clone())
            .or_insert_with(|| {
                Location::new(
                    location_id.clone(),
                    format!("runtime-{location_id}"),
                    cell.state.pos,
                )
            });

        let mut agent = Agent::new(agent_id.clone(), location_id, cell.state.pos);
        agent.body = cell.state.body.clone();
        agent.resources = cell.state.resources.clone();
        model.agents.insert(agent_id.clone(), agent);
    }

    model.agent_prompt_profiles = sidecar.prompt_profiles.clone();
    model.agent_player_bindings = sidecar.agent_player_bindings.clone();
    model.agent_player_public_key_bindings = sidecar.agent_public_key_bindings.clone();
    model.player_auth_last_nonce = sidecar.player_auth_last_nonce.clone();
    model
}

fn map_runtime_event(
    runtime_event: &RuntimeWorldEvent,
    config: &WorldConfig,
) -> Option<WorldEvent> {
    let kind = match &runtime_event.body {
        RuntimeWorldEventBody::Domain(domain) => map_runtime_domain_event(domain, config)?,
        _ => return None,
    };

    Some(WorldEvent {
        id: runtime_event.id,
        time: runtime_event.time,
        kind,
    })
}

fn map_runtime_domain_event(
    event: &RuntimeDomainEvent,
    config: &WorldConfig,
) -> Option<WorldEventKind> {
    match event {
        RuntimeDomainEvent::AgentRegistered { agent_id, pos } => {
            Some(WorldEventKind::AgentRegistered {
                agent_id: agent_id.clone(),
                location_id: location_id_for_pos(*pos),
                pos: *pos,
            })
        }
        RuntimeDomainEvent::AgentMoved { agent_id, from, to } => {
            let distance_cm = space_distance_cm(*from, *to);
            Some(WorldEventKind::AgentMoved {
                agent_id: agent_id.clone(),
                from: location_id_for_pos(*from),
                to: location_id_for_pos(*to),
                distance_cm,
                electricity_cost: config.movement_cost(distance_cm),
            })
        }
        RuntimeDomainEvent::ResourceTransferred {
            from_agent_id,
            to_agent_id,
            kind,
            amount,
        } => Some(WorldEventKind::ResourceTransferred {
            from: ResourceOwner::Agent {
                agent_id: from_agent_id.clone(),
            },
            to: ResourceOwner::Agent {
                agent_id: to_agent_id.clone(),
            },
            kind: *kind,
            amount: *amount,
        }),
        RuntimeDomainEvent::ActionRejected { reason, .. } => Some(WorldEventKind::ActionRejected {
            reason: runtime_reject_reason_to_simulator(reason),
        }),
        _ => None,
    }
}

fn runtime_reject_reason_to_simulator(reason: &RuntimeRejectReason) -> SimulatorRejectReason {
    match reason {
        RuntimeRejectReason::AgentAlreadyExists { agent_id } => {
            SimulatorRejectReason::AgentAlreadyExists {
                agent_id: agent_id.clone(),
            }
        }
        RuntimeRejectReason::AgentNotFound { agent_id } => SimulatorRejectReason::AgentNotFound {
            agent_id: agent_id.clone(),
        },
        RuntimeRejectReason::AgentsNotCoLocated {
            agent_id,
            other_agent_id,
        } => SimulatorRejectReason::AgentsNotCoLocated {
            agent_id: agent_id.clone(),
            other_agent_id: other_agent_id.clone(),
        },
        RuntimeRejectReason::InvalidAmount { amount } => {
            SimulatorRejectReason::InvalidAmount { amount: *amount }
        }
        RuntimeRejectReason::InsufficientResource {
            agent_id,
            kind,
            requested,
            available,
        } => SimulatorRejectReason::InsufficientResource {
            owner: ResourceOwner::Agent {
                agent_id: agent_id.clone(),
            },
            kind: *kind,
            requested: *requested,
            available: *available,
        },
        RuntimeRejectReason::FactoryNotFound { factory_id } => {
            SimulatorRejectReason::FacilityNotFound {
                facility_id: factory_id.clone(),
            }
        }
        RuntimeRejectReason::RuleDenied { notes } => SimulatorRejectReason::RuleDenied {
            notes: notes.clone(),
        },
        other => SimulatorRejectReason::RuleDenied {
            notes: vec![format!("runtime reject: {other:?}")],
        },
    }
}

fn runtime_metrics(world: &RuntimeWorld) -> RunnerMetrics {
    let total_ticks = world.state().time;
    let total_actions = world.journal().len() as u64;
    let action_rejected = world
        .journal()
        .events
        .iter()
        .filter(|event| {
            matches!(
                event.body,
                RuntimeWorldEventBody::Domain(RuntimeDomainEvent::ActionRejected { .. })
            )
        })
        .count() as u64;

    RunnerMetrics {
        total_ticks,
        total_agents: world.state().agents.len(),
        agents_active: world.state().agents.len(),
        agents_quota_exhausted: 0,
        total_actions,
        total_decisions: 0,
        actions_per_tick: if total_ticks > 0 {
            total_actions as f64 / total_ticks as f64
        } else {
            0.0
        },
        decisions_per_tick: 0.0,
        success_rate: if total_actions > 0 {
            (total_actions.saturating_sub(action_rejected)) as f64 / total_actions as f64
        } else {
            0.0
        },
        runtime_perf: Default::default(),
    }
}

fn latest_runtime_event_seq(world: &RuntimeWorld) -> u64 {
    world
        .journal()
        .events
        .last()
        .map(|event| event.id)
        .unwrap_or(0)
}

fn location_id_for_pos(pos: GeoPos) -> String {
    format!(
        "runtime:{}:{}:{}",
        pos.x_cm.round() as i64,
        pos.y_cm.round() as i64,
        pos.z_cm.round() as i64
    )
}

fn send_response(
    writer: &mut BufWriter<TcpStream>,
    response: &ViewerResponse,
) -> Result<(), ViewerRuntimeLiveServerError> {
    let payload = serde_json::to_string(response)
        .map_err(|err| ViewerRuntimeLiveServerError::Serde(err.to_string()))?;
    writer.write_all(payload.as_bytes())?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

fn is_timeout_error(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut | io::ErrorKind::Interrupted
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;

    fn test_signer(seed: u8) -> (String, String) {
        let private_key = [seed; 32];
        let signing_key = SigningKey::from_bytes(&private_key);
        (
            hex::encode(signing_key.verifying_key().to_bytes()),
            hex::encode(private_key),
        )
    }

    fn signed_prompt_control_apply_request(
        mut request: crate::viewer::PromptControlApplyRequest,
        intent: crate::viewer::PromptControlAuthIntent,
        nonce: u64,
        public_key_hex: &str,
        private_key_hex: &str,
    ) -> crate::viewer::PromptControlApplyRequest {
        request.public_key = Some(public_key_hex.to_string());
        let proof = crate::viewer::sign_prompt_control_apply_auth_proof(
            intent,
            &request,
            nonce,
            public_key_hex,
            private_key_hex,
        )
        .expect("sign prompt auth");
        request.auth = Some(proof);
        request
    }

    #[test]
    fn map_runtime_domain_event_agent_registered_uses_runtime_location_id() {
        let event = RuntimeDomainEvent::AgentRegistered {
            agent_id: "a1".to_string(),
            pos: GeoPos::new(12.0, 34.0, 56.0),
        };
        let mapped =
            map_runtime_domain_event(&event, &WorldConfig::default()).expect("mapped event");
        match mapped {
            WorldEventKind::AgentRegistered {
                agent_id,
                location_id,
                pos,
            } => {
                assert_eq!(agent_id, "a1");
                assert_eq!(location_id, "runtime:12:34:56");
                assert_eq!(pos, GeoPos::new(12.0, 34.0, 56.0));
            }
            other => panic!("unexpected mapped event: {other:?}"),
        }
    }

    #[test]
    fn map_runtime_domain_event_agent_moved_sets_distance_and_cost() {
        let config = WorldConfig::default();
        let event = RuntimeDomainEvent::AgentMoved {
            agent_id: "a1".to_string(),
            from: GeoPos::new(0.0, 0.0, 0.0),
            to: GeoPos::new(100_000.0, 0.0, 0.0),
        };
        let mapped = map_runtime_domain_event(&event, &config).expect("mapped event");
        match mapped {
            WorldEventKind::AgentMoved {
                distance_cm,
                electricity_cost,
                ..
            } => {
                assert_eq!(distance_cm, 100_000);
                assert_eq!(electricity_cost, config.movement_cost(distance_cm));
            }
            other => panic!("unexpected mapped event: {other:?}"),
        }
    }

    #[test]
    fn runtime_reject_reason_maps_agent_not_found() {
        let reason = RuntimeRejectReason::AgentNotFound {
            agent_id: "ghost".to_string(),
        };
        let mapped = runtime_reject_reason_to_simulator(&reason);
        match mapped {
            SimulatorRejectReason::AgentNotFound { agent_id } => {
                assert_eq!(agent_id, "ghost");
            }
            other => panic!("unexpected reject mapping: {other:?}"),
        }
    }

    #[test]
    fn runtime_reject_reason_unmapped_falls_back_to_rule_denied() {
        let reason = RuntimeRejectReason::InsufficientMaterial {
            material_kind: "iron".to_string(),
            requested: 10,
            available: 0,
        };
        let mapped = runtime_reject_reason_to_simulator(&reason);
        match mapped {
            SimulatorRejectReason::RuleDenied { notes } => {
                assert_eq!(notes.len(), 1);
                assert!(notes[0].contains("runtime reject"));
            }
            other => panic!("unexpected reject mapping: {other:?}"),
        }
    }

    #[test]
    fn runtime_simulator_action_mapping_equivalence_covers_core_gameplay_and_economy() {
        let server = ViewerRuntimeLiveServer::new(ViewerRuntimeLiveServerConfig::new(
            WorldScenario::Minimal,
        ))
        .expect("runtime server");
        let assert_mapped = |action: crate::simulator::Action, expected: RuntimeAction| {
            let mapped = control_plane::simulator_action_to_runtime(&action, &server.world)
                .expect("action should map to runtime");
            assert_eq!(mapped, expected);
        };

        let move_target = GeoPos::new(10.0, 20.0, 30.0);
        assert_mapped(
            crate::simulator::Action::MoveAgent {
                agent_id: "agent-1".to_string(),
                to: location_id_for_pos(move_target),
            },
            RuntimeAction::MoveAgent {
                agent_id: "agent-1".to_string(),
                to: move_target,
            },
        );
        assert_mapped(
            crate::simulator::Action::TransferResource {
                from: ResourceOwner::Agent {
                    agent_id: "agent-1".to_string(),
                },
                to: ResourceOwner::Agent {
                    agent_id: "agent-2".to_string(),
                },
                kind: ResourceKind::Electricity,
                amount: 3,
            },
            RuntimeAction::TransferResource {
                from_agent_id: "agent-1".to_string(),
                to_agent_id: "agent-2".to_string(),
                kind: ResourceKind::Electricity,
                amount: 3,
            },
        );
        assert_mapped(
            crate::simulator::Action::DeclareWar {
                initiator_agent_id: "agent-1".to_string(),
                war_id: "war.alpha".to_string(),
                aggressor_alliance_id: "alliance.a".to_string(),
                defender_alliance_id: "alliance.b".to_string(),
                objective: "expand".to_string(),
                intensity: 2,
            },
            RuntimeAction::DeclareWar {
                initiator_agent_id: "agent-1".to_string(),
                war_id: "war.alpha".to_string(),
                aggressor_alliance_id: "alliance.a".to_string(),
                defender_alliance_id: "alliance.b".to_string(),
                objective: "expand".to_string(),
                intensity: 2,
            },
        );
        assert_mapped(
            crate::simulator::Action::OpenEconomicContract {
                creator_agent_id: "agent-1".to_string(),
                contract_id: "contract.alpha".to_string(),
                counterparty_agent_id: "agent-2".to_string(),
                settlement_kind: ResourceKind::Data,
                settlement_amount: 5,
                reputation_stake: 7,
                expires_at: 99,
                description: "trade".to_string(),
            },
            RuntimeAction::OpenEconomicContract {
                creator_agent_id: "agent-1".to_string(),
                contract_id: "contract.alpha".to_string(),
                counterparty_agent_id: "agent-2".to_string(),
                settlement_kind: ResourceKind::Data,
                settlement_amount: 5,
                reputation_stake: 7,
                expires_at: 99,
                description: "trade".to_string(),
            },
        );
    }

    #[test]
    fn runtime_simulator_action_mapping_covers_module_artifact_actions() {
        let server = ViewerRuntimeLiveServer::new(ViewerRuntimeLiveServerConfig::new(
            WorldScenario::Minimal,
        ))
        .expect("runtime server");
        let mut source_files = std::collections::BTreeMap::new();
        source_files.insert("module.toml".to_string(), b"manifest".to_vec());
        source_files.insert("src/lib.rs".to_string(), b"pub fn run() {}".to_vec());

        let compile = crate::simulator::Action::CompileModuleArtifactFromSource {
            publisher_agent_id: "agent-1".to_string(),
            module_id: "module.alpha".to_string(),
            manifest_path: "module.toml".to_string(),
            source_files: source_files.clone(),
        };
        let compile_mapped = control_plane::simulator_action_to_runtime(&compile, &server.world)
            .expect("compile action should map");
        assert_eq!(
            compile_mapped,
            RuntimeAction::CompileModuleArtifactFromSource {
                publisher_agent_id: "agent-1".to_string(),
                module_id: "module.alpha".to_string(),
                source_package: crate::runtime::ModuleSourcePackage {
                    manifest_path: "module.toml".to_string(),
                    files: source_files,
                },
            }
        );

        let deploy = crate::simulator::Action::DeployModuleArtifact {
            publisher_agent_id: "agent-1".to_string(),
            wasm_hash: "hash.alpha".to_string(),
            wasm_bytes: vec![0xAA, 0xBB],
            module_id_hint: Some("module.alpha".to_string()),
        };
        let deploy_mapped = control_plane::simulator_action_to_runtime(&deploy, &server.world)
            .expect("deploy action should map");
        assert_eq!(
            deploy_mapped,
            RuntimeAction::DeployModuleArtifact {
                publisher_agent_id: "agent-1".to_string(),
                wasm_hash: "hash.alpha".to_string(),
                wasm_bytes: vec![0xAA, 0xBB],
            }
        );

        let list = crate::simulator::Action::ListModuleArtifactForSale {
            seller_agent_id: "agent-1".to_string(),
            wasm_hash: "hash.alpha".to_string(),
            price_kind: ResourceKind::Data,
            price_amount: 9,
        };
        let list_mapped = control_plane::simulator_action_to_runtime(&list, &server.world)
            .expect("list action should map");
        assert_eq!(
            list_mapped,
            RuntimeAction::ListModuleArtifactForSale {
                seller_agent_id: "agent-1".to_string(),
                wasm_hash: "hash.alpha".to_string(),
                price_kind: ResourceKind::Data,
                price_amount: 9,
            }
        );

        let buy = crate::simulator::Action::BuyModuleArtifact {
            buyer_agent_id: "agent-2".to_string(),
            wasm_hash: "hash.alpha".to_string(),
        };
        let buy_mapped = control_plane::simulator_action_to_runtime(&buy, &server.world)
            .expect("buy action should map");
        assert_eq!(
            buy_mapped,
            RuntimeAction::BuyModuleArtifact {
                buyer_agent_id: "agent-2".to_string(),
                wasm_hash: "hash.alpha".to_string(),
            }
        );
    }

    #[test]
    fn runtime_simulator_action_mapping_keeps_unmapped_actions_as_none() {
        let server = ViewerRuntimeLiveServer::new(ViewerRuntimeLiveServerConfig::new(
            WorldScenario::Minimal,
        ))
        .expect("runtime server");

        let build_factory = crate::simulator::Action::BuildFactory {
            owner: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            location_id: "loc-1".to_string(),
            factory_id: "factory-1".to_string(),
            factory_kind: "smelter".to_string(),
        };
        assert!(
            control_plane::simulator_action_to_runtime(&build_factory, &server.world).is_none()
        );

        let transfer_to_location = crate::simulator::Action::TransferResource {
            from: ResourceOwner::Agent {
                agent_id: "agent-1".to_string(),
            },
            to: ResourceOwner::Location {
                location_id: "loc-1".to_string(),
            },
            kind: ResourceKind::Electricity,
            amount: 1,
        };
        assert!(
            control_plane::simulator_action_to_runtime(&transfer_to_location, &server.world)
                .is_none()
        );
    }

    #[test]
    fn runtime_prompt_control_script_mode_requires_llm_mode() {
        let mut server = ViewerRuntimeLiveServer::new(
            ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal)
                .with_decision_mode(ViewerLiveDecisionMode::Script),
        )
        .expect("runtime server");
        let agent_id = server
            .world
            .state()
            .agents
            .keys()
            .next()
            .cloned()
            .expect("seed agent");
        let (public_key, private_key) = test_signer(11);
        let request = signed_prompt_control_apply_request(
            crate::viewer::PromptControlApplyRequest {
                agent_id: agent_id.clone(),
                player_id: "player-a".to_string(),
                public_key: None,
                auth: None,
                expected_version: Some(0),
                updated_by: None,
                system_prompt_override: Some(Some("system".to_string())),
                short_term_goal_override: None,
                long_term_goal_override: None,
            },
            crate::viewer::PromptControlAuthIntent::Apply,
            1,
            public_key.as_str(),
            private_key.as_str(),
        );
        let err = server
            .handle_prompt_control(crate::viewer::PromptControlCommand::Apply { request })
            .expect_err("script mode should reject prompt control");
        assert_eq!(err.code, "llm_mode_required");
        assert!(server.llm_sidecar.prompt_profiles.is_empty());
    }

    #[test]
    fn runtime_prompt_control_apply_updates_snapshot_and_bindings() {
        let mut server = ViewerRuntimeLiveServer::new(
            ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal)
                .with_decision_mode(ViewerLiveDecisionMode::Llm),
        )
        .expect("runtime server");
        let agent_id = server
            .world
            .state()
            .agents
            .keys()
            .next()
            .cloned()
            .expect("seed agent");
        let (public_key, private_key) = test_signer(12);
        let request = signed_prompt_control_apply_request(
            crate::viewer::PromptControlApplyRequest {
                agent_id: agent_id.clone(),
                player_id: "player-a".to_string(),
                public_key: None,
                auth: None,
                expected_version: Some(0),
                updated_by: None,
                system_prompt_override: Some(Some("system".to_string())),
                short_term_goal_override: None,
                long_term_goal_override: None,
            },
            crate::viewer::PromptControlAuthIntent::Apply,
            2,
            public_key.as_str(),
            private_key.as_str(),
        );

        let ack = server
            .handle_prompt_control(crate::viewer::PromptControlCommand::Apply { request })
            .expect("llm mode apply");
        assert_eq!(ack.version, 1);
        let snapshot = server.compat_snapshot();
        let profile = snapshot
            .model
            .agent_prompt_profiles
            .get(agent_id.as_str())
            .expect("profile in snapshot");
        assert_eq!(profile.version, 1);
        assert_eq!(
            snapshot
                .model
                .agent_player_bindings
                .get(agent_id.as_str())
                .map(String::as_str),
            Some("player-a")
        );
        assert_eq!(
            snapshot
                .model
                .player_auth_last_nonce
                .get("player-a")
                .copied(),
            Some(2)
        );
    }

    #[test]
    fn runtime_agent_chat_script_mode_requires_llm_mode() {
        let mut server = ViewerRuntimeLiveServer::new(
            ViewerRuntimeLiveServerConfig::new(WorldScenario::Minimal)
                .with_decision_mode(ViewerLiveDecisionMode::Script),
        )
        .expect("runtime server");
        let agent_id = server
            .world
            .state()
            .agents
            .keys()
            .next()
            .cloned()
            .expect("seed agent");
        let err = server
            .handle_agent_chat(crate::viewer::AgentChatRequest {
                agent_id,
                player_id: Some("player-a".to_string()),
                public_key: None,
                auth: None,
                message: "hello".to_string(),
            })
            .expect_err("script mode should reject chat");
        assert_eq!(err.code, "llm_mode_required");
    }
}
