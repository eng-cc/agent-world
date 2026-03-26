use super::*;
use oasis7::simulator::{WorldEvent, WorldEventKind, WorldSnapshot};
use oasis7::viewer::{ViewerRequest, ViewerResponse, ViewerStream, VIEWER_PROTOCOL_VERSION};

const HOSTED_SESSION_RUNTIME_PROBE_TIMEOUT_MS: u64 = 300;
const HOSTED_SESSION_RUNTIME_PROBE_INTERVAL_MS: u64 = 1_000;
const HOSTED_SESSION_RUNTIME_MONITOR_READ_TIMEOUT_MS: u64 = 100;
const RUNTIME_PRESENCE_MONITOR_CLIENT: &str = "oasis7_game_launcher_runtime_presence_monitor";
const RUNTIME_PRESENCE_PROBE_CLIENT: &str = "oasis7_game_launcher_hosted_session_probe";

pub(super) fn run_runtime_presence_monitor(
    stop_requested: Arc<AtomicBool>,
    live_bind: Arc<String>,
    hosted_session_issuer: Arc<Mutex<HostedPlayerSessionIssuer>>,
) {
    let reconnect_backoff = Duration::from_millis(250);
    while !stop_requested.load(Ordering::SeqCst) {
        let result =
            RuntimePresenceMonitorClient::connect(live_bind.as_str()).and_then(|mut client| {
                observe_runtime_presence_snapshot(&hosted_session_issuer, client.active_players());
                let mut next_snapshot_at = Instant::now()
                    + Duration::from_millis(HOSTED_SESSION_RUNTIME_PROBE_INTERVAL_MS);
                loop {
                    if stop_requested.load(Ordering::SeqCst) {
                        return Ok(());
                    }
                    if Instant::now() >= next_snapshot_at {
                        client.request_snapshot_sync()?;
                        observe_runtime_presence_snapshot(
                            &hosted_session_issuer,
                            client.active_players(),
                        );
                        next_snapshot_at = Instant::now()
                            + Duration::from_millis(HOSTED_SESSION_RUNTIME_PROBE_INTERVAL_MS);
                        continue;
                    }
                    if client.poll_once()? {
                        observe_runtime_presence_snapshot(
                            &hosted_session_issuer,
                            client.active_players(),
                        );
                    }
                }
            });
        if stop_requested.load(Ordering::SeqCst) {
            return;
        }
        if let Err(err) = result {
            if let Ok(mut issuer) = hosted_session_issuer.lock() {
                issuer.record_runtime_probe_failure(err);
            }
            thread::sleep(reconnect_backoff);
        }
    }
}

pub(super) fn query_runtime_bound_players(live_bind: &str) -> Result<BTreeSet<String>, String> {
    let mut client = ViewerRuntimeProbeClient::connect(
        live_bind,
        Duration::from_millis(HOSTED_SESSION_RUNTIME_PROBE_TIMEOUT_MS),
        RUNTIME_PRESENCE_PROBE_CLIENT,
    )?;
    client.request_snapshot()?;
    client.wait_for_snapshot()
}

fn observe_runtime_presence_snapshot(
    hosted_session_issuer: &Arc<Mutex<HostedPlayerSessionIssuer>>,
    active_players: &BTreeSet<String>,
) {
    if let Ok(mut issuer) = hosted_session_issuer.lock() {
        issuer.observe_runtime_active_players(active_players.iter().map(String::as_str));
    }
}

struct RuntimePresenceMonitorClient {
    probe: ViewerRuntimeProbeClient,
    active_players: BTreeSet<String>,
    subscription_mode: Option<RuntimePresenceSubscriptionMode>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RuntimePresenceSubscriptionMode {
    EventsOnly,
    EventsAndSnapshot,
}

impl RuntimePresenceMonitorClient {
    fn connect(live_bind: &str) -> Result<Self, String> {
        let probe = ViewerRuntimeProbeClient::connect(
            live_bind,
            Duration::from_millis(HOSTED_SESSION_RUNTIME_MONITOR_READ_TIMEOUT_MS),
            RUNTIME_PRESENCE_MONITOR_CLIENT,
        )?;
        let mut client = Self {
            probe,
            active_players: BTreeSet::new(),
            subscription_mode: None,
        };
        client.request_snapshot_sync()?;
        Ok(client)
    }

    fn active_players(&self) -> &BTreeSet<String> {
        &self.active_players
    }

    fn poll_once(&mut self) -> Result<bool, String> {
        match self.probe.read_response_line()? {
            ViewerResponseLine::Response(response) => Ok(self.apply_response(response)),
            ViewerResponseLine::Timeout => Ok(false),
            ViewerResponseLine::Closed => {
                Err("runtime presence connection closed unexpectedly".to_string())
            }
        }
    }

    fn request_snapshot_sync(&mut self) -> Result<(), String> {
        self.set_subscription_mode(RuntimePresenceSubscriptionMode::EventsAndSnapshot)?;
        self.probe.request_snapshot()?;
        let snapshot = loop {
            match self.probe.read_response_line()? {
                ViewerResponseLine::Response(response) => {
                    if let Some(snapshot_players) = runtime_players_from_response(&response) {
                        self.active_players = snapshot_players.clone();
                        break snapshot_players;
                    }
                    self.apply_response(response);
                }
                ViewerResponseLine::Timeout => {
                    return Err(
                        "runtime presence monitor timed out waiting for snapshot".to_string()
                    )
                }
                ViewerResponseLine::Closed => {
                    return Err(
                        "runtime presence connection closed before snapshot sync completed"
                            .to_string(),
                    )
                }
            }
        };
        self.active_players = snapshot;
        self.set_subscription_mode(RuntimePresenceSubscriptionMode::EventsOnly)?;
        Ok(())
    }

    fn set_subscription_mode(
        &mut self,
        mode: RuntimePresenceSubscriptionMode,
    ) -> Result<(), String> {
        if self.subscription_mode == Some(mode) {
            return Ok(());
        }
        let streams = match mode {
            RuntimePresenceSubscriptionMode::EventsOnly => vec![ViewerStream::Events],
            RuntimePresenceSubscriptionMode::EventsAndSnapshot => {
                vec![ViewerStream::Events, ViewerStream::Snapshot]
            }
        };
        self.probe.write_request_line(&ViewerRequest::Subscribe {
            streams,
            event_kinds: Vec::new(),
        })?;
        self.subscription_mode = Some(mode);
        Ok(())
    }

    fn apply_response(&mut self, response: ViewerResponse) -> bool {
        if let Some(snapshot_players) = runtime_players_from_response(&response) {
            self.active_players = snapshot_players;
            return true;
        }
        match response {
            ViewerResponse::Event { event } => match runtime_players_from_event(&event) {
                Some(RuntimePlayerPresenceDelta::Bound(player_id)) => {
                    self.active_players.insert(player_id)
                }
                Some(RuntimePlayerPresenceDelta::Unbound(player_id)) => {
                    self.active_players.remove(player_id.as_str())
                }
                None => false,
            },
            _ => false,
        }
    }
}

struct ViewerRuntimeProbeClient {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl ViewerRuntimeProbeClient {
    fn connect(live_bind: &str, timeout: Duration, client_name: &str) -> Result<Self, String> {
        let (host, port) = parse_host_port(live_bind, "--live-bind")?;
        let stream = TcpStream::connect((host.as_str(), port))
            .map_err(|err| format!("connect runtime live {live_bind} failed: {err}"))?;
        stream
            .set_nodelay(true)
            .map_err(|err| format!("set_nodelay on runtime presence client failed: {err}"))?;
        stream
            .set_read_timeout(Some(timeout))
            .map_err(|err| format!("set_read_timeout on runtime presence client failed: {err}"))?;
        stream
            .set_write_timeout(Some(timeout))
            .map_err(|err| format!("set_write_timeout on runtime presence client failed: {err}"))?;
        let reader_stream = stream
            .try_clone()
            .map_err(|err| format!("clone runtime presence stream failed: {err}"))?;
        let mut client = Self {
            reader: BufReader::new(reader_stream),
            writer: BufWriter::new(stream),
        };
        client.write_request_line(&ViewerRequest::Hello {
            client: client_name.to_string(),
            version: VIEWER_PROTOCOL_VERSION,
        })?;
        client.wait_for_hello_ack()?;
        Ok(client)
    }

    fn request_snapshot(&mut self) -> Result<(), String> {
        self.write_request_line(&ViewerRequest::RequestSnapshot)
    }

    fn wait_for_snapshot(&mut self) -> Result<BTreeSet<String>, String> {
        loop {
            match self.read_response_line()? {
                ViewerResponseLine::Response(response) => {
                    if let Some(active_players) = runtime_players_from_response(&response) {
                        return Ok(active_players);
                    }
                }
                ViewerResponseLine::Timeout => {
                    return Err("runtime probe timed out waiting for snapshot".to_string())
                }
                ViewerResponseLine::Closed => {
                    return Err("runtime probe closed before snapshot".to_string())
                }
            }
        }
    }

    fn wait_for_hello_ack(&mut self) -> Result<(), String> {
        loop {
            match self.read_response_line()? {
                ViewerResponseLine::Response(ViewerResponse::HelloAck { .. }) => return Ok(()),
                ViewerResponseLine::Response(_) => {}
                ViewerResponseLine::Timeout => {
                    return Err("runtime probe timed out waiting for hello_ack".to_string())
                }
                ViewerResponseLine::Closed => {
                    return Err("runtime probe closed before hello_ack".to_string())
                }
            }
        }
    }

    fn write_request_line(&mut self, request: &ViewerRequest) -> Result<(), String> {
        let payload = serde_json::to_string(request)
            .map_err(|err| format!("serialize runtime presence request failed: {err}"))?;
        self.writer
            .write_all(payload.as_bytes())
            .map_err(|err| format!("write runtime presence request failed: {err}"))?;
        self.writer
            .write_all(b"\n")
            .map_err(|err| format!("write runtime presence delimiter failed: {err}"))?;
        self.writer
            .flush()
            .map_err(|err| format!("flush runtime presence request failed: {err}"))
    }

    fn read_response_line(&mut self) -> Result<ViewerResponseLine, String> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => Ok(ViewerResponseLine::Closed),
            Ok(_) => serde_json::from_str(line.trim_end())
                .map(ViewerResponseLine::Response)
                .map_err(|err| format!("decode runtime presence response failed: {err}")),
            Err(err) if is_timeout_error(&err) => Ok(ViewerResponseLine::Timeout),
            Err(err) => Err(format!("read runtime presence response failed: {err}")),
        }
    }
}

enum ViewerResponseLine {
    Response(ViewerResponse),
    Timeout,
    Closed,
}

enum RuntimePlayerPresenceDelta {
    Bound(String),
    Unbound(String),
}

fn runtime_players_from_response(response: &ViewerResponse) -> Option<BTreeSet<String>> {
    let ViewerResponse::Snapshot { snapshot } = response else {
        return None;
    };
    Some(runtime_players_from_snapshot(snapshot))
}

fn runtime_players_from_snapshot(snapshot: &WorldSnapshot) -> BTreeSet<String> {
    snapshot
        .model
        .agent_player_bindings
        .values()
        .map(|player_id| player_id.trim())
        .filter(|player_id| !player_id.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn runtime_players_from_event(event: &WorldEvent) -> Option<RuntimePlayerPresenceDelta> {
    match &event.kind {
        WorldEventKind::AgentPlayerBound { player_id, .. } => {
            let player_id = player_id.trim();
            (!player_id.is_empty())
                .then(|| RuntimePlayerPresenceDelta::Bound(player_id.to_string()))
        }
        WorldEventKind::AgentPlayerUnbound { player_id, .. } => {
            let player_id = player_id.trim();
            (!player_id.is_empty())
                .then(|| RuntimePlayerPresenceDelta::Unbound(player_id.to_string()))
        }
        _ => None,
    }
}

fn is_timeout_error(err: &std::io::Error) -> bool {
    matches!(
        err.kind(),
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use oasis7::simulator::{WorldConfig, WorldModel};
    use std::io::{BufRead, BufReader, BufWriter, Write};
    use std::net::TcpListener;
    use std::thread;

    #[test]
    fn runtime_presence_monitor_client_merges_event_and_snapshot_correction() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind mock");
        let addr = listener.local_addr().expect("local addr");
        let handle = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept");
            let reader_stream = stream.try_clone().expect("clone");
            let mut reader = BufReader::new(reader_stream);
            let mut writer = BufWriter::new(stream);

            expect_request_type(&mut reader, |request| {
                matches!(
                    request,
                    ViewerRequest::Hello {
                        version: VIEWER_PROTOCOL_VERSION,
                        ..
                    }
                )
            });
            write_response(
                &mut writer,
                &ViewerResponse::HelloAck {
                    server: "oasis7".to_string(),
                    version: VIEWER_PROTOCOL_VERSION,
                    world_id: "test-world".to_string(),
                    control_profile: oasis7::viewer::ViewerControlProfile::Live,
                },
            );

            expect_subscribe(&mut reader, &[ViewerStream::Events, ViewerStream::Snapshot]);
            expect_request_type(&mut reader, |request| {
                matches!(request, ViewerRequest::RequestSnapshot)
            });
            write_response(
                &mut writer,
                &ViewerResponse::Snapshot {
                    snapshot: world_snapshot(["player-a"]),
                },
            );
            expect_subscribe(&mut reader, &[ViewerStream::Events]);

            write_response(
                &mut writer,
                &ViewerResponse::Event {
                    event: WorldEvent {
                        id: 1,
                        time: 1,
                        kind: WorldEventKind::AgentPlayerBound {
                            agent_id: "agent-b".to_string(),
                            player_id: "player-b".to_string(),
                            public_key: None,
                        },
                        runtime_event: None,
                    },
                },
            );
            write_response(
                &mut writer,
                &ViewerResponse::Event {
                    event: WorldEvent {
                        id: 2,
                        time: 2,
                        kind: WorldEventKind::AgentPlayerUnbound {
                            agent_id: "agent-a".to_string(),
                            player_id: "player-a".to_string(),
                            public_key: None,
                        },
                        runtime_event: None,
                    },
                },
            );

            expect_subscribe(&mut reader, &[ViewerStream::Events, ViewerStream::Snapshot]);
            expect_request_type(&mut reader, |request| {
                matches!(request, ViewerRequest::RequestSnapshot)
            });
            write_response(
                &mut writer,
                &ViewerResponse::Snapshot {
                    snapshot: world_snapshot(["player-b"]),
                },
            );
            expect_subscribe(&mut reader, &[ViewerStream::Events]);
        });

        let mut client =
            RuntimePresenceMonitorClient::connect(format!("{addr}").as_str()).expect("connect");
        assert_eq!(
            client.active_players(),
            &BTreeSet::from(["player-a".to_string()])
        );

        assert!(client.poll_once().expect("poll event"));
        assert_eq!(
            client.active_players(),
            &BTreeSet::from(["player-a".to_string(), "player-b".to_string()])
        );

        assert!(client.poll_once().expect("poll unbind event"));
        assert_eq!(
            client.active_players(),
            &BTreeSet::from(["player-b".to_string()])
        );

        client.request_snapshot_sync().expect("snapshot sync");
        assert_eq!(
            client.active_players(),
            &BTreeSet::from(["player-b".to_string()])
        );

        handle.join().expect("join mock");
    }

    fn expect_subscribe(reader: &mut BufReader<TcpStream>, expected_streams: &[ViewerStream]) {
        expect_request_type(reader, |request| match request {
            ViewerRequest::Subscribe {
                streams,
                event_kinds,
            } => streams == expected_streams && event_kinds.is_empty(),
            _ => false,
        });
    }

    fn expect_request_type<F>(reader: &mut BufReader<TcpStream>, predicate: F)
    where
        F: FnOnce(&ViewerRequest) -> bool,
    {
        let mut line = String::new();
        reader.read_line(&mut line).expect("read request");
        let request: ViewerRequest = serde_json::from_str(line.trim_end()).expect("decode request");
        assert!(predicate(&request), "unexpected request: {request:?}");
    }

    fn write_response(writer: &mut BufWriter<TcpStream>, response: &ViewerResponse) {
        serde_json::to_writer(&mut *writer, response).expect("write response");
        writer.write_all(b"\n").expect("write newline");
        writer.flush().expect("flush response");
    }

    fn world_snapshot<const N: usize>(players: [&str; N]) -> WorldSnapshot {
        let mut model = WorldModel::default();
        for (index, player_id) in players.iter().enumerate() {
            model
                .agent_player_bindings
                .insert(format!("agent-{index}"), (*player_id).to_string());
        }
        WorldSnapshot {
            version: 1,
            chunk_generation_schema_version: 1,
            time: 0,
            config: WorldConfig::default(),
            model,
            runtime_snapshot: None,
            player_gameplay: None,
            chunk_runtime: Default::default(),
            next_event_id: 0,
            next_action_id: 0,
            pending_actions: Vec::new(),
            journal_len: 0,
        }
    }
}
