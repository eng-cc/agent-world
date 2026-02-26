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
}

#[derive(Clone)]
struct PlaybackPulseControl {
    state: Arc<(Mutex<PlaybackPulseState>, Condvar)>,
}

#[derive(Debug, Default)]
struct PlaybackPulseState {
    enabled: bool,
}

impl PlaybackPulseControl {
    fn new() -> Self {
        Self {
            state: Arc::new((Mutex::new(PlaybackPulseState::default()), Condvar::new())),
        }
    }

    fn set_enabled(&self, enabled: bool) {
        let (state_lock, signal) = &*self.state;
        let mut state = state_lock
            .lock()
            .expect("playback pulse control mutex poisoned");
        if state.enabled != enabled {
            state.enabled = enabled;
            signal.notify_all();
        }
    }

    fn notify(&self) {
        let (_, signal) = &*self.state;
        signal.notify_all();
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
                    let steps = count.max(1);
                    for _ in 0..steps {
                        world.request_llm_decision();
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
                        }

                        self.update_metrics(world.metrics());
                        self.emit_metrics(writer)?;
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
                    self.update_metrics(world.metrics());
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
        Ok(ViewerLiveRequestOutcome {
            continue_running: true,
            request_llm_decision,
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
    tx: mpsc::Sender<LiveLoopSignal>,
    loop_running: Arc<AtomicBool>,
    playback_control: PlaybackPulseControl,
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
                            playback_control.notify();
                            break;
                        }
                    }
                    Err(_) => {}
                }
            }
            Err(_) => {
                loop_running.store(false, Ordering::SeqCst);
                playback_control.notify();
                break;
            }
        }
    }
    loop_running.store(false, Ordering::SeqCst);
    playback_control.notify();
}

fn emit_playback_pulses(
    tx: mpsc::Sender<LiveLoopSignal>,
    tick_interval: Duration,
    loop_running: Arc<AtomicBool>,
    playback_control: PlaybackPulseControl,
) {
    let pulse_interval = if tick_interval.is_zero() {
        Duration::from_millis(1)
    } else {
        tick_interval
    };

    let (state_lock, signal) = &*playback_control.state;
    let mut state = state_lock
        .lock()
        .expect("playback pulse control mutex poisoned");
    while loop_running.load(Ordering::SeqCst) {
        while loop_running.load(Ordering::SeqCst) && !state.enabled {
            state = signal
                .wait(state)
                .expect("playback pulse control mutex poisoned");
        }
        if !loop_running.load(Ordering::SeqCst) {
            break;
        }

        let (next_state, wait_result) = signal
            .wait_timeout(state, pulse_interval)
            .expect("playback pulse control mutex poisoned");
        state = next_state;
        if !loop_running.load(Ordering::SeqCst) {
            break;
        }
        if !state.enabled {
            continue;
        }
        if wait_result.timed_out() {
            drop(state);
            if tx.send(LiveLoopSignal::PlaybackPulse).is_err() {
                break;
            }
            state = state_lock
                .lock()
                .expect("playback pulse control mutex poisoned");
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
#[path = "live/tests.rs"]
mod tests;
