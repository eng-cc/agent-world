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
    metrics: RunnerMetrics,
}

#[derive(Debug, Clone, Copy)]
struct ViewerLiveRequestOutcome {
    continue_running: bool,
    request_llm_decision: bool,
    deferred_control: Option<ViewerLiveDeferredControl>,
}

#[derive(Debug, Clone, Copy)]
enum ViewerLiveDeferredControl {
    Step { count: usize },
}

#[derive(Debug, Clone, Copy)]
enum CoalescedSignalKind {
    LlmDecisionRequested,
    ConsensusCommitted,
    ConsensusDriveRequested,
    NonConsensusDriveRequested,
}

const LIVE_LOOP_SIGNAL_KIND_COUNT: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LiveLoopSignalKind {
    Request,
    LlmDecisionRequested,
    ConsensusCommitted,
    ConsensusDriveRequested,
    NonConsensusDriveRequested,
    StepRequested,
}

impl LiveLoopSignalKind {
    const ALL: [Self; LIVE_LOOP_SIGNAL_KIND_COUNT] = [
        Self::Request,
        Self::LlmDecisionRequested,
        Self::ConsensusCommitted,
        Self::ConsensusDriveRequested,
        Self::NonConsensusDriveRequested,
        Self::StepRequested,
    ];

    fn as_index(self) -> usize {
        match self {
            Self::Request => 0,
            Self::LlmDecisionRequested => 1,
            Self::ConsensusCommitted => 2,
            Self::ConsensusDriveRequested => 3,
            Self::NonConsensusDriveRequested => 4,
            Self::StepRequested => 5,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Request => "request",
            Self::LlmDecisionRequested => "llm_decision",
            Self::ConsensusCommitted => "consensus_committed",
            Self::ConsensusDriveRequested => "consensus_drive",
            Self::NonConsensusDriveRequested => "non_consensus_drive",
            Self::StepRequested => "step",
        }
    }
}

impl LiveLoopSignal {
    fn kind(&self) -> LiveLoopSignalKind {
        match self {
            LiveLoopSignal::Request(_) => LiveLoopSignalKind::Request,
            LiveLoopSignal::LlmDecisionRequested => LiveLoopSignalKind::LlmDecisionRequested,
            LiveLoopSignal::ConsensusCommitted => LiveLoopSignalKind::ConsensusCommitted,
            LiveLoopSignal::ConsensusDriveRequested => LiveLoopSignalKind::ConsensusDriveRequested,
            LiveLoopSignal::NonConsensusDriveRequested => {
                LiveLoopSignalKind::NonConsensusDriveRequested
            }
            LiveLoopSignal::StepRequested { .. } => LiveLoopSignalKind::StepRequested,
        }
    }
}

struct LiveLoopBackpressure {
    merged_llm_decision_requested: AtomicU64,
    merged_consensus_committed: AtomicU64,
    merged_consensus_drive_requested: AtomicU64,
    merged_non_consensus_drive_requested: AtomicU64,
    dropped_llm_decision_requested: AtomicU64,
    dropped_consensus_committed: AtomicU64,
    dropped_consensus_drive_requested: AtomicU64,
    dropped_non_consensus_drive_requested: AtomicU64,
    enqueued_signals: [AtomicU64; LIVE_LOOP_SIGNAL_KIND_COUNT],
    handled_signals: [AtomicU64; LIVE_LOOP_SIGNAL_KIND_COUNT],
    handled_nanos_total: [AtomicU64; LIVE_LOOP_SIGNAL_KIND_COUNT],
    handled_nanos_max: [AtomicU64; LIVE_LOOP_SIGNAL_KIND_COUNT],
}

#[derive(Debug, Default, Clone, Copy)]
struct LiveLoopBackpressureSnapshot {
    merged_llm_decision_requested: u64,
    merged_consensus_committed: u64,
    merged_consensus_drive_requested: u64,
    merged_non_consensus_drive_requested: u64,
    dropped_llm_decision_requested: u64,
    dropped_consensus_committed: u64,
    dropped_consensus_drive_requested: u64,
    dropped_non_consensus_drive_requested: u64,
    signal_stats: [LiveLoopSignalStatsSnapshot; LIVE_LOOP_SIGNAL_KIND_COUNT],
}

#[derive(Debug, Default, Clone, Copy)]
struct LiveLoopSignalStatsSnapshot {
    enqueued: u64,
    handled: u64,
    avg_handle_us: u64,
    max_handle_us: u64,
}

impl Default for LiveLoopBackpressure {
    fn default() -> Self {
        Self {
            merged_llm_decision_requested: AtomicU64::new(0),
            merged_consensus_committed: AtomicU64::new(0),
            merged_consensus_drive_requested: AtomicU64::new(0),
            merged_non_consensus_drive_requested: AtomicU64::new(0),
            dropped_llm_decision_requested: AtomicU64::new(0),
            dropped_consensus_committed: AtomicU64::new(0),
            dropped_consensus_drive_requested: AtomicU64::new(0),
            dropped_non_consensus_drive_requested: AtomicU64::new(0),
            enqueued_signals: std::array::from_fn(|_| AtomicU64::new(0)),
            handled_signals: std::array::from_fn(|_| AtomicU64::new(0)),
            handled_nanos_total: std::array::from_fn(|_| AtomicU64::new(0)),
            handled_nanos_max: std::array::from_fn(|_| AtomicU64::new(0)),
        }
    }
}

impl LiveLoopBackpressureSnapshot {
    fn has_activity(&self) -> bool {
        self.merged_llm_decision_requested > 0
            || self.merged_consensus_committed > 0
            || self.merged_consensus_drive_requested > 0
            || self.merged_non_consensus_drive_requested > 0
            || self.dropped_llm_decision_requested > 0
            || self.dropped_consensus_committed > 0
            || self.dropped_consensus_drive_requested > 0
            || self.dropped_non_consensus_drive_requested > 0
            || self
                .signal_stats
                .iter()
                .any(|stats| stats.enqueued > 0 || stats.handled > 0)
    }

    fn signal_stats(&self, kind: LiveLoopSignalKind) -> LiveLoopSignalStatsSnapshot {
        self.signal_stats[kind.as_index()]
    }
}

impl LiveLoopBackpressure {
    fn record_enqueued(&self, kind: LiveLoopSignalKind) {
        self.enqueued_signals[kind.as_index()].fetch_add(1, Ordering::SeqCst);
    }

    fn record_handled(&self, kind: LiveLoopSignalKind, elapsed: Duration) {
        let index = kind.as_index();
        self.handled_signals[index].fetch_add(1, Ordering::SeqCst);
        let nanos = elapsed.as_nanos().min(u64::MAX as u128) as u64;
        self.handled_nanos_total[index].fetch_add(nanos, Ordering::SeqCst);
        fetch_max_atomic(&self.handled_nanos_max[index], nanos);
    }

    fn record_merged(&self, kind: CoalescedSignalKind) {
        match kind {
            CoalescedSignalKind::LlmDecisionRequested => {
                self.merged_llm_decision_requested
                    .fetch_add(1, Ordering::SeqCst);
            }
            CoalescedSignalKind::ConsensusCommitted => {
                self.merged_consensus_committed
                    .fetch_add(1, Ordering::SeqCst);
            }
            CoalescedSignalKind::ConsensusDriveRequested => {
                self.merged_consensus_drive_requested
                    .fetch_add(1, Ordering::SeqCst);
            }
            CoalescedSignalKind::NonConsensusDriveRequested => {
                self.merged_non_consensus_drive_requested
                    .fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    fn record_dropped(&self, kind: CoalescedSignalKind) {
        match kind {
            CoalescedSignalKind::LlmDecisionRequested => {
                self.dropped_llm_decision_requested
                    .fetch_add(1, Ordering::SeqCst);
            }
            CoalescedSignalKind::ConsensusCommitted => {
                self.dropped_consensus_committed
                    .fetch_add(1, Ordering::SeqCst);
            }
            CoalescedSignalKind::ConsensusDriveRequested => {
                self.dropped_consensus_drive_requested
                    .fetch_add(1, Ordering::SeqCst);
            }
            CoalescedSignalKind::NonConsensusDriveRequested => {
                self.dropped_non_consensus_drive_requested
                    .fetch_add(1, Ordering::SeqCst);
            }
        }
    }

    fn snapshot(&self) -> LiveLoopBackpressureSnapshot {
        LiveLoopBackpressureSnapshot {
            merged_llm_decision_requested: self
                .merged_llm_decision_requested
                .load(Ordering::SeqCst),
            merged_consensus_committed: self.merged_consensus_committed.load(Ordering::SeqCst),
            merged_consensus_drive_requested: self
                .merged_consensus_drive_requested
                .load(Ordering::SeqCst),
            merged_non_consensus_drive_requested: self
                .merged_non_consensus_drive_requested
                .load(Ordering::SeqCst),
            dropped_llm_decision_requested: self
                .dropped_llm_decision_requested
                .load(Ordering::SeqCst),
            dropped_consensus_committed: self.dropped_consensus_committed.load(Ordering::SeqCst),
            dropped_consensus_drive_requested: self
                .dropped_consensus_drive_requested
                .load(Ordering::SeqCst),
            dropped_non_consensus_drive_requested: self
                .dropped_non_consensus_drive_requested
                .load(Ordering::SeqCst),
            signal_stats: std::array::from_fn(|index| {
                let enqueued = self.enqueued_signals[index].load(Ordering::SeqCst);
                let handled = self.handled_signals[index].load(Ordering::SeqCst);
                let total_nanos = self.handled_nanos_total[index].load(Ordering::SeqCst);
                let max_nanos = self.handled_nanos_max[index].load(Ordering::SeqCst);
                LiveLoopSignalStatsSnapshot {
                    enqueued,
                    handled,
                    avg_handle_us: if handled > 0 {
                        total_nanos / handled / 1_000
                    } else {
                        0
                    },
                    max_handle_us: max_nanos / 1_000,
                }
            }),
        }
    }
}

impl CoalescedSignalKind {
    fn signal_kind(self) -> LiveLoopSignalKind {
        match self {
            Self::LlmDecisionRequested => LiveLoopSignalKind::LlmDecisionRequested,
            Self::ConsensusCommitted => LiveLoopSignalKind::ConsensusCommitted,
            Self::ConsensusDriveRequested => LiveLoopSignalKind::ConsensusDriveRequested,
            Self::NonConsensusDriveRequested => LiveLoopSignalKind::NonConsensusDriveRequested,
        }
    }
}

fn fetch_max_atomic(target: &AtomicU64, value: u64) {
    let mut current = target.load(Ordering::SeqCst);
    while value > current {
        match target.compare_exchange(current, value, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => break,
            Err(observed) => current = observed,
        }
    }
}

fn format_live_loop_signal_stats(snapshot: &LiveLoopBackpressureSnapshot) -> String {
    let mut parts = Vec::new();
    for kind in LiveLoopSignalKind::ALL {
        let stats = snapshot.signal_stats(kind);
        if stats.enqueued == 0 && stats.handled == 0 {
            continue;
        }
        parts.push(format!(
            "{}={{in:{}, handled:{}, avg_us:{}, max_us:{}}}",
            kind.as_str(),
            stats.enqueued,
            stats.handled,
            stats.avg_handle_us,
            stats.max_handle_us
        ));
    }
    if parts.is_empty() {
        "none".to_string()
    } else {
        parts.join(", ")
    }
}

fn enqueue_coalesced_signal(
    tx: &mpsc::SyncSender<LiveLoopSignal>,
    signal: LiveLoopSignal,
    queued: &Arc<AtomicBool>,
    kind: CoalescedSignalKind,
    backpressure: &LiveLoopBackpressure,
) {
    if queued.swap(true, Ordering::SeqCst) {
        backpressure.record_merged(kind);
        return;
    }
    match tx.try_send(signal) {
        Ok(()) => {
            backpressure.record_enqueued(kind.signal_kind());
        }
        Err(mpsc::TrySendError::Full(_)) => {
            queued.store(false, Ordering::SeqCst);
            backpressure.record_dropped(kind);
        }
        Err(mpsc::TrySendError::Disconnected(_)) => {
            queued.store(false, Ordering::SeqCst);
        }
    }
}

impl ViewerLiveSession {
    fn new() -> Self {
        Self {
            subscribed: HashSet::new(),
            event_filters: None,
            playing: false,
            metrics: RunnerMetrics::default(),
        }
    }

    fn handle_request(
        &mut self,
        request: ViewerRequest,
        writer: &mut BufWriter<TcpStream>,
        world: &mut LiveWorld,
        world_id: &str,
    ) -> Result<ViewerLiveRequestOutcome, ViewerLiveServerError> {
        let mut request_llm_decision = false;
        let mut deferred_control = None;
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
                    self.update_metrics(world.metrics());
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
                let (result, wake_llm) = match command {
                    PromptControlCommand::Preview { request } => {
                        (world.prompt_control_preview(request), false)
                    }
                    PromptControlCommand::Apply { request } => {
                        (world.prompt_control_apply(request), true)
                    }
                    PromptControlCommand::Rollback { request } => {
                        (world.prompt_control_rollback(request), true)
                    }
                };
                match result {
                    Ok(ack) => {
                        if wake_llm {
                            request_llm_decision = true;
                        }
                        send_response(writer, &ViewerResponse::PromptControlAck { ack })?;
                    }
                    Err(error) => {
                        send_response(writer, &ViewerResponse::PromptControlError { error })?;
                    }
                }
            }
            ViewerRequest::AgentChat { request } => match world.agent_chat(request) {
                Ok(ack) => {
                    request_llm_decision = true;
                    send_response(writer, &ViewerResponse::AgentChatAck { ack })?;
                }
                Err(error) => {
                    send_response(writer, &ViewerResponse::AgentChatError { error })?;
                }
            },
            ViewerRequest::Control { mode } => match mode {
                ViewerControl::Pause => {
                    self.playing = false;
                }
                ViewerControl::Play => {
                    self.playing = true;
                    request_llm_decision = true;
                }
                ViewerControl::Step { count } => {
                    self.playing = false;
                    deferred_control = Some(ViewerLiveDeferredControl::Step {
                        count: count.max(1),
                    });
                }
                ViewerControl::Seek { tick } => {
                    self.playing = false;
                    // P2P live mode is monotonic and does not support rewind/seek semantics.
                    eprintln!(
                        "viewer live: ignore seek control in live mode (target_tick={tick})"
                    );
                }
            },
        }
        Ok(ViewerLiveRequestOutcome {
            continue_running: true,
            request_llm_decision,
            deferred_control,
        })
    }

    fn should_emit_event(&self) -> bool {
        self.playing && self.subscribed.contains(&ViewerStream::Events)
    }

    fn event_allowed(&self, event: &crate::simulator::WorldEvent) -> bool {
        match &self.event_filters {
            Some(filters) => filters
                .iter()
                .any(|filter| viewer_event_kind_matches(filter, &event.kind)),
            None => true,
        }
    }

    fn update_metrics(&mut self, metrics: RunnerMetrics) {
        self.metrics = metrics;
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
        runtime_perf: Default::default(),
    }
}

fn read_requests(
    stream: TcpStream,
    tx: mpsc::SyncSender<LiveLoopSignal>,
    loop_running: Arc<AtomicBool>,
    backpressure: Arc<LiveLoopBackpressure>,
) {
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
                        if tx.send(LiveLoopSignal::Request(request)).is_err() {
                            loop_running.store(false, Ordering::SeqCst);
                            break;
                        }
                        backpressure.record_enqueued(LiveLoopSignalKind::Request);
                    }
                    Err(_) => {}
                }
            }
            Err(_) => {
                loop_running.store(false, Ordering::SeqCst);
                break;
            }
        }
    }
    loop_running.store(false, Ordering::SeqCst);
}

fn emit_consensus_commit_signals(
    tx: mpsc::SyncSender<LiveLoopSignal>,
    loop_running: Arc<AtomicBool>,
    committed_batches: NodeCommittedActionBatchesHandle,
    signal_queued: Arc<AtomicBool>,
    backpressure: Arc<LiveLoopBackpressure>,
) {
    let wait_timeout = Duration::from_millis(50);
    while loop_running.load(Ordering::SeqCst) {
        if !committed_batches.wait_for_batches(wait_timeout) {
            continue;
        }
        if !loop_running.load(Ordering::SeqCst) {
            break;
        }
        enqueue_coalesced_signal(
            &tx,
            LiveLoopSignal::ConsensusCommitted,
            &signal_queued,
            CoalescedSignalKind::ConsensusCommitted,
            backpressure.as_ref(),
        );
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
#[path = "live/tests.rs"]
mod tests;
