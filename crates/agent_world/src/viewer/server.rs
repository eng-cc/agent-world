use std::collections::HashSet;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use crate::simulator::{
    PersistError, RunnerMetrics, WorldEvent, WorldJournal, WorldSnapshot, WorldTime,
};

use super::protocol::{
    ViewerControl, ViewerEventKind, ViewerRequest, ViewerResponse, ViewerStream,
    VIEWER_PROTOCOL_VERSION,
};

#[derive(Debug, Clone)]
pub struct ViewerServerConfig {
    pub bind_addr: String,
    pub snapshot_path: PathBuf,
    pub journal_path: PathBuf,
    pub tick_interval: Duration,
    pub world_id: String,
}

impl ViewerServerConfig {
    pub fn from_dir(dir: impl AsRef<Path>) -> Self {
        let dir = dir.as_ref();
        let snapshot_path = dir.join("snapshot.json");
        let journal_path = dir.join("journal.json");
        let world_id = dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("world")
            .to_string();
        Self {
            bind_addr: "127.0.0.1:5010".to_string(),
            snapshot_path,
            journal_path,
            tick_interval: Duration::from_millis(50),
            world_id,
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
}

#[derive(Debug)]
pub enum ViewerServerError {
    Io(String),
    Serde(String),
    Persist(String),
}

impl From<io::Error> for ViewerServerError {
    fn from(err: io::Error) -> Self {
        ViewerServerError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for ViewerServerError {
    fn from(err: serde_json::Error) -> Self {
        ViewerServerError::Serde(err.to_string())
    }
}

impl From<PersistError> for ViewerServerError {
    fn from(err: PersistError) -> Self {
        ViewerServerError::Persist(format!("{err:?}"))
    }
}

pub struct ViewerServer {
    config: ViewerServerConfig,
    snapshot: WorldSnapshot,
    journal: WorldJournal,
}

impl ViewerServer {
    pub fn load(config: ViewerServerConfig) -> Result<Self, ViewerServerError> {
        let snapshot = WorldSnapshot::load_json(&config.snapshot_path)?;
        let journal = WorldJournal::load_json(&config.journal_path)?;
        Ok(Self {
            config,
            snapshot,
            journal,
        })
    }

    pub fn run(&self) -> Result<(), ViewerServerError> {
        let listener = TcpListener::bind(&self.config.bind_addr)?;
        for incoming in listener.incoming() {
            let stream = incoming?;
            if let Err(err) = self.serve_stream(stream) {
                eprintln!("viewer server error: {err:?}");
            }
        }
        Ok(())
    }

    pub fn run_once(&self) -> Result<(), ViewerServerError> {
        let listener = TcpListener::bind(&self.config.bind_addr)?;
        let (stream, _) = listener.accept()?;
        self.serve_stream(stream)?;
        Ok(())
    }

    fn serve_stream(&self, stream: TcpStream) -> Result<(), ViewerServerError> {
        stream.set_nodelay(true)?;
        let reader_stream = stream.try_clone()?;
        let mut writer = BufWriter::new(stream);
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || read_requests(reader_stream, tx));

        let mut session = ViewerSession::new(&self.journal.events, self.config.tick_interval);
        let mut last_tick = Instant::now();

        loop {
            match rx.recv_timeout(self.config.tick_interval) {
                Ok(command) => {
                    if !session.handle_request(
                        command,
                        &mut writer,
                        &self.snapshot,
                        &self.config.world_id,
                    )? {
                        break;
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }

            if session.should_emit_event() && last_tick.elapsed() >= session.tick_interval {
                if let Some(event) = session.next_event() {
                    let time = event.time;
                    send_response(&mut writer, &ViewerResponse::Event { event })?;
                    session.update_metrics_time(time);
                    session.emit_metrics(&mut writer)?;
                    last_tick = Instant::now();
                } else {
                    session.playing = false;
                }
            }
        }
        Ok(())
    }
}

struct ViewerSession<'a> {
    events: &'a [WorldEvent],
    subscribed: HashSet<ViewerStream>,
    event_filters: Option<HashSet<ViewerEventKind>>,
    cursor: usize,
    playing: bool,
    tick_interval: Duration,
    metrics: RunnerMetrics,
}

impl<'a> ViewerSession<'a> {
    fn new(events: &'a [WorldEvent], tick_interval: Duration) -> Self {
        Self {
            events,
            subscribed: HashSet::new(),
            event_filters: None,
            cursor: 0,
            playing: false,
            tick_interval,
            metrics: RunnerMetrics::default(),
        }
    }

    fn handle_request(
        &mut self,
        request: ViewerRequest,
        writer: &mut BufWriter<TcpStream>,
        snapshot: &WorldSnapshot,
        world_id: &str,
    ) -> Result<bool, ViewerServerError> {
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
                    send_response(writer, &ViewerResponse::Snapshot {
                        snapshot: snapshot.clone(),
                    })?;
                }
                if self.subscribed.contains(&ViewerStream::Metrics) {
                    self.metrics = metrics_from_snapshot(snapshot);
                    send_response(
                        writer,
                        &ViewerResponse::Metrics {
                            time: Some(snapshot.time),
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
                        if let Some(event) = self.next_event() {
                            let time = event.time;
                            send_response(writer, &ViewerResponse::Event { event })?;
                            self.update_metrics_time(time);
                            self.emit_metrics(writer)?;
                        } else {
                            break;
                        }
                    }
                    self.playing = false;
                }
                ViewerControl::Seek { tick } => {
                    self.cursor = seek_to_tick(self.events, tick);
                    self.update_metrics_time(tick);
                    self.emit_metrics(writer)?;
                }
            },
        }
        Ok(true)
    }

    fn should_emit_event(&self) -> bool {
        self.playing && self.subscribed.contains(&ViewerStream::Events)
    }

    fn next_event(&mut self) -> Option<WorldEvent> {
        while self.cursor < self.events.len() {
            let event = self.events.get(self.cursor).cloned();
            self.cursor = self.cursor.saturating_add(1);
            if let Some(event) = event {
                if self.event_allowed(&event) {
                    return Some(event);
                }
            }
        }
        None
    }

    fn event_allowed(&self, event: &WorldEvent) -> bool {
        match &self.event_filters {
            Some(filters) => filters.iter().any(|filter| filter.matches(&event.kind)),
            None => true,
        }
    }

    fn update_metrics_time(&mut self, time: WorldTime) {
        self.metrics.total_ticks = time;
    }

    fn emit_metrics(&self, writer: &mut BufWriter<TcpStream>) -> Result<(), ViewerServerError> {
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

fn seek_to_tick(events: &[WorldEvent], tick: WorldTime) -> usize {
    events
        .iter()
        .position(|event| event.time >= tick)
        .unwrap_or(events.len())
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
                    Err(_) => {
                        // Ignore malformed requests for now.
                    }
                }
            }
            Err(_) => break,
        }
    }
}

fn send_response(
    writer: &mut BufWriter<TcpStream>,
    response: &ViewerResponse,
) -> Result<(), ViewerServerError> {
    serde_json::to_writer(&mut *writer, response)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

fn metrics_from_snapshot(snapshot: &WorldSnapshot) -> RunnerMetrics {
    RunnerMetrics {
        total_ticks: snapshot.time,
        total_agents: snapshot.model.agents.len(),
        agents_active: snapshot.model.agents.len(),
        agents_quota_exhausted: 0,
        total_actions: 0,
        total_decisions: 0,
        actions_per_tick: 0.0,
        decisions_per_tick: 0.0,
        success_rate: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulator::{RejectReason, WorldEventKind};

    fn make_event(id: u64, time: WorldTime) -> WorldEvent {
        WorldEvent {
            id,
            time,
            kind: WorldEventKind::ActionRejected {
                reason: RejectReason::InvalidAmount { amount: 1 },
            },
        }
    }

    #[test]
    fn seek_to_tick_finds_first_event_at_or_after_time() {
        let events = vec![make_event(1, 10), make_event(2, 20), make_event(3, 30)];
        assert_eq!(seek_to_tick(&events, 0), 0);
        assert_eq!(seek_to_tick(&events, 10), 0);
        assert_eq!(seek_to_tick(&events, 15), 1);
        assert_eq!(seek_to_tick(&events, 25), 2);
        assert_eq!(seek_to_tick(&events, 35), 3);
    }
}
