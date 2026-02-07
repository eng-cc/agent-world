use std::collections::HashSet;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::geometry::space_distance_cm;
use crate::simulator::{
    initialize_kernel, Action, AgentDecisionTrace, AgentRunner, LlmAgentBehavior,
    LlmAgentBuildError, OpenAiChatCompletionClient, ResourceKind, ResourceOwner, RunnerMetrics,
    WorldConfig, WorldEvent, WorldInitConfig, WorldInitError, WorldKernel, WorldScenario,
    WorldSnapshot,
};

use super::protocol::{
    ViewerControl, ViewerEventKind, ViewerRequest, ViewerResponse, ViewerStream,
    VIEWER_PROTOCOL_VERSION,
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

struct LiveStepResult {
    event: Option<WorldEvent>,
    decision_trace: Option<AgentDecisionTrace>,
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
                let behavior = LlmAgentBehavior::from_env(agent_id)?;
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
                    if tick == 0 {
                        world.reset()?;
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
                    } else {
                        send_response(
                            writer,
                            &ViewerResponse::Error {
                                message: "live mode only supports seek to tick 0".to_string(),
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
mod tests {
    use super::*;

    #[test]
    fn live_script_moves_between_locations() {
        let config = WorldConfig::default();
        let init = WorldInitConfig::from_scenario(WorldScenario::TwinRegionBootstrap, &config);
        let (mut kernel, _) = initialize_kernel(config, init).expect("init ok");

        let mut script = LiveScript::new(&kernel);
        let mut moved = false;
        for _ in 0..2 {
            let action = script.next_action(&kernel).expect("action");
            kernel.submit_action(action);
            kernel.step_until_empty();

            let agent = kernel.model().agents.get("agent-0").expect("agent exists");
            if agent.location_id == "region-b" {
                moved = true;
                break;
            }
        }

        assert!(moved);
    }

    #[test]
    fn live_world_reset_rebuilds_kernel() {
        let config = WorldConfig::default();
        let init = WorldInitConfig::from_scenario(WorldScenario::Minimal, &config);
        let mut world =
            LiveWorld::new(config, init, ViewerLiveDecisionMode::Script).expect("init ok");

        for _ in 0..5 {
            let _ = world.step().expect("step");
            if world.kernel.time() > 0 {
                break;
            }
        }
        assert!(world.kernel.time() > 0);

        world.reset().expect("reset ok");
        assert_eq!(world.kernel.time(), 0);
    }

    #[test]
    fn live_server_config_supports_llm_mode() {
        let config = ViewerLiveServerConfig::new(WorldScenario::Minimal);
        assert_eq!(config.decision_mode, ViewerLiveDecisionMode::Script);

        let llm_config = config.clone().with_llm_mode(true);
        assert_eq!(llm_config.decision_mode, ViewerLiveDecisionMode::Llm);

        let script_config = llm_config.with_decision_mode(ViewerLiveDecisionMode::Script);
        assert_eq!(script_config.decision_mode, ViewerLiveDecisionMode::Script);
    }
}
