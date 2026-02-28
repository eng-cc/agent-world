use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{self, Child, Command};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_SCENARIO: &str = "llm_bootstrap";
const DEFAULT_LIVE_BIND: &str = "127.0.0.1:5023";
const DEFAULT_WEB_BIND: &str = "127.0.0.1:5011";
const DEFAULT_VIEWER_HOST: &str = "127.0.0.1";
const DEFAULT_VIEWER_PORT: u16 = 4173;
const DEFAULT_VIEWER_STATIC_DIR: &str = "web";
const DEFAULT_CHAIN_STATUS_BIND: &str = "127.0.0.1:5121";
const DEFAULT_CHAIN_NODE_ID: &str = "viewer-live-node";
const DEFAULT_CHAIN_NODE_ROLE: &str = "sequencer";
const DEFAULT_CHAIN_NODE_TICK_MS: u64 = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliOptions {
    scenario: String,
    live_bind: String,
    web_bind: String,
    viewer_host: String,
    viewer_port: u16,
    viewer_static_dir: String,
    with_llm: bool,
    open_browser: bool,
    chain_enabled: bool,
    chain_status_bind: String,
    chain_node_id: String,
    chain_world_id: Option<String>,
    chain_node_role: String,
    chain_node_tick_ms: u64,
    chain_node_validators: Vec<String>,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            scenario: DEFAULT_SCENARIO.to_string(),
            live_bind: DEFAULT_LIVE_BIND.to_string(),
            web_bind: DEFAULT_WEB_BIND.to_string(),
            viewer_host: DEFAULT_VIEWER_HOST.to_string(),
            viewer_port: DEFAULT_VIEWER_PORT,
            viewer_static_dir: DEFAULT_VIEWER_STATIC_DIR.to_string(),
            with_llm: false,
            open_browser: true,
            chain_enabled: true,
            chain_status_bind: DEFAULT_CHAIN_STATUS_BIND.to_string(),
            chain_node_id: DEFAULT_CHAIN_NODE_ID.to_string(),
            chain_world_id: None,
            chain_node_role: DEFAULT_CHAIN_NODE_ROLE.to_string(),
            chain_node_tick_ms: DEFAULT_CHAIN_NODE_TICK_MS,
            chain_node_validators: Vec::new(),
        }
    }
}

#[derive(Debug)]
struct StaticHttpServer {
    stop_tx: Sender<()>,
    error_rx: Receiver<String>,
    join_handle: Option<thread::JoinHandle<()>>,
}

fn main() {
    let raw_args: Vec<String> = env::args().skip(1).collect();
    if raw_args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return;
    }

    let options = match parse_options(raw_args.iter().map(|arg| arg.as_str())) {
        Ok(options) => options,
        Err(err) => {
            eprintln!("{err}");
            print_help();
            process::exit(1);
        }
    };

    if let Err(err) = run_launcher(&options) {
        eprintln!("launcher failed: {err}");
        process::exit(1);
    }
}

fn run_launcher(options: &CliOptions) -> Result<(), String> {
    let world_viewer_live_bin = resolve_world_viewer_live_binary()?;
    let world_chain_runtime_bin = if options.chain_enabled {
        Some(resolve_world_chain_runtime_binary()?)
    } else {
        None
    };
    let viewer_static_dir = resolve_viewer_static_dir(options.viewer_static_dir.as_str())?;

    let mut chain_child = if let Some(chain_bin) = world_chain_runtime_bin.as_ref() {
        Some(spawn_world_chain_runtime(chain_bin.as_path(), options)?)
    } else {
        None
    };
    let mut world_child = match spawn_world_viewer_live(&world_viewer_live_bin, options) {
        Ok(child) => child,
        Err(err) => {
            if let Some(child) = chain_child.as_mut() {
                terminate_child(child);
            }
            return Err(err);
        }
    };
    let mut server = start_static_http_server(
        options.viewer_host.as_str(),
        options.viewer_port,
        viewer_static_dir.as_path(),
    )?;

    let ready_result = wait_until_ready(options);
    if let Err(err) = ready_result {
        stop_static_http_server(&mut server);
        terminate_child(&mut world_child);
        if let Some(child) = chain_child.as_mut() {
            terminate_child(child);
        }
        return Err(err);
    }

    let game_url = build_game_url(options);
    println!("Launcher stack is ready.");
    println!("- URL: {game_url}");
    println!("- world_viewer_live pid: {}", world_child.id());
    if let Some(chain_child) = chain_child.as_ref() {
        println!("- world_chain_runtime pid: {}", chain_child.id());
        println!(
            "- chain status: http://{}/v1/chain/status",
            options.chain_status_bind
        );
    } else {
        println!("- world_chain_runtime: disabled");
    }
    println!("- web static root: {}", viewer_static_dir.display());
    println!("Press Ctrl+C to stop.");

    if options.open_browser {
        if let Err(err) = open_browser(&game_url) {
            eprintln!("warning: failed to open browser automatically: {err}");
            eprintln!("open this URL manually: {game_url}");
        }
    }

    let monitor_result =
        monitor_world_chain_and_server(&mut world_child, chain_child.as_mut(), &mut server);
    stop_static_http_server(&mut server);
    terminate_child(&mut world_child);
    if let Some(child) = chain_child.as_mut() {
        terminate_child(child);
    }
    monitor_result
}

fn spawn_world_viewer_live(path: &Path, options: &CliOptions) -> Result<Child, String> {
    let mut command = Command::new(path);
    command
        .arg(options.scenario.as_str())
        .arg("--bind")
        .arg(options.live_bind.as_str())
        .arg("--web-bind")
        .arg(options.web_bind.as_str())
        .arg("--topology")
        .arg("single")
        .arg("--viewer-no-consensus-gate")
        .arg("--no-node");
    if options.with_llm {
        command.arg("--llm");
    } else {
        command.arg("--no-llm");
    }
    command.spawn().map_err(|err| {
        format!(
            "failed to start world_viewer_live from `{}`: {err}",
            path.display()
        )
    })
}

fn spawn_world_chain_runtime(path: &Path, options: &CliOptions) -> Result<Child, String> {
    let mut command = Command::new(path);
    command
        .arg("--node-id")
        .arg(options.chain_node_id.as_str())
        .arg("--world-id")
        .arg(chain_world_id(options).as_str())
        .arg("--status-bind")
        .arg(options.chain_status_bind.as_str())
        .arg("--node-role")
        .arg(options.chain_node_role.as_str())
        .arg("--node-tick-ms")
        .arg(options.chain_node_tick_ms.to_string());
    for validator in &options.chain_node_validators {
        command.arg("--node-validator").arg(validator.as_str());
    }
    command.spawn().map_err(|err| {
        format!(
            "failed to start world_chain_runtime from `{}`: {err}",
            path.display()
        )
    })
}

fn chain_world_id(options: &CliOptions) -> String {
    options
        .chain_world_id
        .clone()
        .unwrap_or_else(|| format!("live-{}", options.scenario))
}

fn start_static_http_server(
    host: &str,
    port: u16,
    root_dir: &Path,
) -> Result<StaticHttpServer, String> {
    let listener = TcpListener::bind((host, port))
        .map_err(|err| format!("failed to bind static HTTP server at {host}:{port}: {err}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|err| format!("failed to set static HTTP listener nonblocking: {err}"))?;

    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let (error_tx, error_rx) = mpsc::channel::<String>();
    let root_dir = Arc::new(root_dir.to_path_buf());
    let join_handle = thread::spawn(move || {
        if let Err(err) = run_static_http_loop(listener, root_dir, stop_rx) {
            let _ = error_tx.send(err);
        }
    });

    Ok(StaticHttpServer {
        stop_tx,
        error_rx,
        join_handle: Some(join_handle),
    })
}

fn run_static_http_loop(
    listener: TcpListener,
    root_dir: Arc<PathBuf>,
    stop_rx: Receiver<()>,
) -> Result<(), String> {
    loop {
        match stop_rx.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => return Ok(()),
            Err(TryRecvError::Empty) => {}
        }

        match listener.accept() {
            Ok((stream, _addr)) => {
                let root_dir = Arc::clone(&root_dir);
                thread::spawn(move || {
                    if let Err(err) = handle_http_connection(stream, root_dir.as_path()) {
                        eprintln!("warning: static HTTP connection failed: {err}");
                    }
                });
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20));
            }
            Err(err) => {
                return Err(format!("static HTTP accept failed: {err}"));
            }
        }
    }
}

fn handle_http_connection(mut stream: TcpStream, root_dir: &Path) -> Result<(), String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|err| format!("failed to set read timeout: {err}"))?;

    let mut buffer = [0u8; 8192];
    let bytes = stream
        .read(&mut buffer)
        .map_err(|err| format!("failed to read request: {err}"))?;
    if bytes == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes]);
    let Some(line) = request.lines().next() else {
        write_http_response(&mut stream, 400, "text/plain", b"Bad Request", false)
            .map_err(|err| format!("failed to write 400 response: {err}"))?;
        return Ok(());
    };

    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("");

    let head_only = method.eq_ignore_ascii_case("HEAD");
    if !method.eq_ignore_ascii_case("GET") && !head_only {
        write_http_response(&mut stream, 405, "text/plain", b"Method Not Allowed", false)
            .map_err(|err| format!("failed to write 405 response: {err}"))?;
        return Ok(());
    }

    let resolved = match resolve_static_asset_path(root_dir, target) {
        Ok(resolved) => resolved,
        Err(_) => {
            write_http_response(&mut stream, 400, "text/plain", b"Bad Request", head_only)
                .map_err(|err| format!("failed to write 400 response: {err}"))?;
            return Ok(());
        }
    };

    match resolved {
        Some(path) => {
            let body = fs::read(&path).map_err(|err| {
                format!("failed to read static asset `{}`: {err}", path.display())
            })?;
            write_http_response(
                &mut stream,
                200,
                content_type_for_path(path.as_path()),
                body.as_slice(),
                head_only,
            )
            .map_err(|err| format!("failed to write 200 response: {err}"))?;
        }
        None => {
            write_http_response(&mut stream, 404, "text/plain", b"Not Found", head_only)
                .map_err(|err| format!("failed to write 404 response: {err}"))?;
        }
    }

    Ok(())
}

fn resolve_static_asset_path(root_dir: &Path, raw_target: &str) -> Result<Option<PathBuf>, String> {
    let path_only = raw_target
        .split('?')
        .next()
        .unwrap_or(raw_target)
        .split('#')
        .next()
        .unwrap_or(raw_target);

    let relative = sanitize_relative_request_path(path_only)?;
    let direct_path = if relative.as_os_str().is_empty() {
        root_dir.join("index.html")
    } else {
        root_dir.join(relative.as_path())
    };

    if direct_path.is_file() {
        return Ok(Some(direct_path));
    }

    let has_extension = Path::new(path_only)
        .file_name()
        .and_then(|name| Path::new(name).extension())
        .is_some();
    if !has_extension {
        let spa_index = root_dir.join("index.html");
        if spa_index.is_file() {
            return Ok(Some(spa_index));
        }
    }

    Ok(None)
}

fn sanitize_relative_request_path(raw_path: &str) -> Result<PathBuf, String> {
    let trimmed = raw_path.trim();
    if trimmed.is_empty() {
        return Ok(PathBuf::new());
    }

    let normalized = trimmed.strip_prefix('/').unwrap_or(trimmed);
    let mut cleaned = PathBuf::new();
    for segment in normalized.split('/') {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." || segment.contains('\\') {
            return Err("path traversal is not allowed".to_string());
        }
        cleaned.push(segment);
    }

    Ok(cleaned)
}

fn content_type_for_path(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("wasm") => "application/wasm",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("map") => "application/json; charset=utf-8",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn write_http_response(
    stream: &mut TcpStream,
    status_code: u16,
    content_type: &str,
    body: &[u8],
    head_only: bool,
) -> std::io::Result<()> {
    let status_text = match status_code {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Internal Server Error",
    };
    let headers = format!(
        "HTTP/1.1 {status_code} {status_text}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(headers.as_bytes())?;
    if !head_only {
        stream.write_all(body)?;
    }
    stream.flush()?;
    Ok(())
}

fn wait_until_ready(options: &CliOptions) -> Result<(), String> {
    let (viewer_host, viewer_port) = normalize_http_target(
        options.viewer_host.as_str(),
        options.viewer_port,
        "viewer host/port",
    )?;
    wait_for_http_ready(viewer_host.as_str(), viewer_port, Duration::from_secs(30)).map_err(
        |err| format!("viewer HTTP did not become ready at {viewer_host}:{viewer_port}: {err}"),
    )?;

    let (bridge_host, bridge_port) = parse_host_port(options.web_bind.as_str(), "--web-bind")?;
    wait_for_tcp_ready(bridge_host.as_str(), bridge_port, Duration::from_secs(60)).map_err(
        |err| format!("web bridge did not become ready at {bridge_host}:{bridge_port}: {err}"),
    )?;

    if options.chain_enabled {
        let (chain_status_host, chain_status_port) =
            parse_host_port(options.chain_status_bind.as_str(), "--chain-status-bind")?;
        let chain_status_host = if chain_status_host == "0.0.0.0" {
            "127.0.0.1".to_string()
        } else {
            chain_status_host
        };
        wait_for_http_ready(
            chain_status_host.as_str(),
            chain_status_port,
            Duration::from_secs(30),
        )
        .map_err(|err| {
            format!(
                "chain status HTTP did not become ready at {}:{}: {}",
                chain_status_host, chain_status_port, err
            )
        })?;
    }
    Ok(())
}

fn monitor_world_chain_and_server(
    world_child: &mut Child,
    mut chain_child: Option<&mut Child>,
    server: &mut StaticHttpServer,
) -> Result<(), String> {
    loop {
        if let Some(status) = world_child
            .try_wait()
            .map_err(|err| format!("failed to query world_viewer_live status: {err}"))?
        {
            return Err(format!("world_viewer_live exited unexpectedly: {status}"));
        }
        if let Some(chain_child) = chain_child.as_deref_mut() {
            if let Some(status) = chain_child
                .try_wait()
                .map_err(|err| format!("failed to query world_chain_runtime status: {err}"))?
            {
                return Err(format!("world_chain_runtime exited unexpectedly: {status}"));
            }
        }

        match server.error_rx.try_recv() {
            Ok(err) => return Err(format!("static HTTP server failed: {err}")),
            Err(TryRecvError::Disconnected) => {
                return Err("static HTTP server channel disconnected unexpectedly".to_string());
            }
            Err(TryRecvError::Empty) => {}
        }

        if let Some(handle) = server.join_handle.as_ref() {
            if handle.is_finished() {
                return Err("static HTTP server exited unexpectedly".to_string());
            }
        }

        thread::sleep(Duration::from_millis(400));
    }
}

fn stop_static_http_server(server: &mut StaticHttpServer) {
    let _ = server.stop_tx.send(());
    if let Some(handle) = server.join_handle.take() {
        let _ = handle.join();
    }
}

fn terminate_child(child: &mut Child) {
    if let Ok(None) = child.try_wait() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn wait_for_tcp_ready(host: &str, port: u16, timeout: Duration) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        match TcpStream::connect((host, port)) {
            Ok(_) => return Ok(()),
            Err(_) => thread::sleep(Duration::from_millis(200)),
        }
    }
    Err(format!("timeout after {}s", timeout.as_secs()))
}

fn wait_for_http_ready(host: &str, port: u16, timeout: Duration) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    let request = format!("GET / HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n");

    while Instant::now() < deadline {
        if let Ok(mut stream) = TcpStream::connect((host, port)) {
            let _ = stream.write_all(request.as_bytes());
            let mut buf = [0u8; 256];
            match stream.read(&mut buf) {
                Ok(0) => {}
                Ok(_) => return Ok(()),
                Err(_) => {}
            }
        }
        thread::sleep(Duration::from_millis(200));
    }

    Err(format!("timeout after {}s", timeout.as_secs()))
}

fn build_game_url(options: &CliOptions) -> String {
    let viewer_host = if options.viewer_host == "0.0.0.0" {
        "127.0.0.1"
    } else {
        options.viewer_host.as_str()
    };
    let (bridge_host, bridge_port) = parse_host_port(options.web_bind.as_str(), "--web-bind")
        .unwrap_or_else(|_| ("127.0.0.1".to_string(), 5011));
    let bridge_host = if bridge_host == "0.0.0.0" {
        "127.0.0.1".to_string()
    } else {
        bridge_host
    };
    format!(
        "http://{viewer_host}:{}/?ws=ws://{bridge_host}:{bridge_port}",
        options.viewer_port
    )
}

fn normalize_http_target(host: &str, port: u16, label: &str) -> Result<(String, u16), String> {
    if host.trim().is_empty() {
        return Err(format!("{label} host cannot be empty"));
    }
    Ok((
        if host == "0.0.0.0" {
            "127.0.0.1".to_string()
        } else {
            host.to_string()
        },
        port,
    ))
}

fn parse_options<'a>(args: impl Iterator<Item = &'a str>) -> Result<CliOptions, String> {
    let mut options = CliOptions::default();
    let mut iter = args.peekable();

    while let Some(arg) = iter.next() {
        match arg {
            "--scenario" => {
                options.scenario = parse_required_value(&mut iter, "--scenario")?;
            }
            "--live-bind" => {
                options.live_bind = parse_required_value(&mut iter, "--live-bind")?;
            }
            "--web-bind" => {
                options.web_bind = parse_required_value(&mut iter, "--web-bind")?;
            }
            "--viewer-host" => {
                options.viewer_host = parse_required_value(&mut iter, "--viewer-host")?;
            }
            "--viewer-port" => {
                let raw = parse_required_value(&mut iter, "--viewer-port")?;
                options.viewer_port = raw.parse::<u16>().map_err(|_| {
                    format!("--viewer-port must be an integer in 1..=65535, got `{raw}`")
                })?;
                if options.viewer_port == 0 {
                    return Err("--viewer-port must be in 1..=65535".to_string());
                }
            }
            "--viewer-static-dir" => {
                options.viewer_static_dir = parse_required_value(&mut iter, "--viewer-static-dir")?;
            }
            "--with-llm" => {
                options.with_llm = true;
            }
            "--no-open-browser" => {
                options.open_browser = false;
            }
            "--chain-enable" => {
                options.chain_enabled = true;
            }
            "--chain-disable" => {
                options.chain_enabled = false;
            }
            "--chain-status-bind" => {
                options.chain_status_bind = parse_required_value(&mut iter, "--chain-status-bind")?;
            }
            "--chain-node-id" => {
                options.chain_node_id = parse_required_value(&mut iter, "--chain-node-id")?;
            }
            "--chain-world-id" => {
                options.chain_world_id = Some(parse_required_value(&mut iter, "--chain-world-id")?);
            }
            "--chain-node-role" => {
                let raw = parse_required_value(&mut iter, "--chain-node-role")?;
                options.chain_node_role = parse_chain_node_role(raw.as_str())?;
            }
            "--chain-node-tick-ms" => {
                let raw = parse_required_value(&mut iter, "--chain-node-tick-ms")?;
                options.chain_node_tick_ms = raw.parse::<u64>().map_err(|_| {
                    format!("--chain-node-tick-ms must be a positive integer, got `{raw}`")
                })?;
                if options.chain_node_tick_ms == 0 {
                    return Err("--chain-node-tick-ms must be a positive integer".to_string());
                }
            }
            "--chain-node-validator" => {
                let value = parse_required_value(&mut iter, "--chain-node-validator")?;
                validate_chain_node_validator(value.as_str())?;
                options.chain_node_validators.push(value);
            }
            _ => return Err(format!("unknown option: {arg}")),
        }
    }

    let _ = parse_host_port(options.live_bind.as_str(), "--live-bind")?;
    let _ = parse_host_port(options.web_bind.as_str(), "--web-bind")?;
    normalize_http_target(
        options.viewer_host.as_str(),
        options.viewer_port,
        "viewer host/port",
    )?;
    if options.chain_enabled {
        let _ = parse_host_port(options.chain_status_bind.as_str(), "--chain-status-bind")?;
        if options.chain_node_id.trim().is_empty() {
            return Err("--chain-node-id requires a non-empty value".to_string());
        }
        parse_chain_node_role(options.chain_node_role.as_str())?;
        if options.chain_node_tick_ms == 0 {
            return Err("--chain-node-tick-ms must be a positive integer".to_string());
        }
        for validator in &options.chain_node_validators {
            validate_chain_node_validator(validator.as_str())?;
        }
    }

    Ok(options)
}

fn parse_required_value<'a, I>(
    iter: &mut std::iter::Peekable<I>,
    flag: &str,
) -> Result<String, String>
where
    I: Iterator<Item = &'a str>,
{
    let Some(value) = iter.next() else {
        return Err(format!("{flag} requires a value"));
    };
    let value = value.trim();
    if value.is_empty() {
        return Err(format!("{flag} requires a non-empty value"));
    }
    Ok(value.to_string())
}

fn parse_host_port(raw: &str, label: &str) -> Result<(String, u16), String> {
    let trimmed = raw.trim();
    let (host, port_text) = trimmed
        .rsplit_once(':')
        .ok_or_else(|| format!("{label} must be in <host:port> format"))?;
    if host.trim().is_empty() {
        return Err(format!("{label} host cannot be empty"));
    }
    let port = port_text
        .parse::<u16>()
        .map_err(|_| format!("{label} port must be an integer in 1..=65535"))?;
    if port == 0 {
        return Err(format!("{label} port must be in 1..=65535"));
    }
    Ok((host.trim().to_string(), port))
}

fn parse_chain_node_role(raw: &str) -> Result<String, String> {
    let normalized = raw.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "sequencer" | "storage" | "observer" => Ok(normalized),
        _ => Err("--chain-node-role must be one of: sequencer, storage, observer".to_string()),
    }
}

fn validate_chain_node_validator(raw: &str) -> Result<(), String> {
    let (validator_id, stake) = raw.rsplit_once(':').ok_or_else(|| {
        "--chain-node-validator must be in <validator_id:stake> format".to_string()
    })?;
    if validator_id.trim().is_empty() {
        return Err("--chain-node-validator validator_id cannot be empty".to_string());
    }
    let stake = stake
        .parse::<u64>()
        .map_err(|_| "--chain-node-validator stake must be a positive integer".to_string())?;
    if stake == 0 {
        return Err("--chain-node-validator stake must be a positive integer".to_string());
    }
    Ok(())
}

fn resolve_world_viewer_live_binary() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("AGENT_WORLD_WORLD_VIEWER_LIVE_BIN") {
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            return Ok(candidate);
        }
        return Err(format!(
            "AGENT_WORLD_WORLD_VIEWER_LIVE_BIN is set but file does not exist: {}",
            candidate.display()
        ));
    }

    let mut candidates = Vec::new();
    if let Ok(current_exe) = env::current_exe() {
        if let Some(dir) = current_exe.parent() {
            candidates.push(dir.join(binary_name("world_viewer_live")));
            candidates.push(
                dir.join("..")
                    .join(binary_name("world_viewer_live"))
                    .to_path_buf(),
            );
        }
    }

    if let Some(path_entry) = find_on_path(OsStr::new(&binary_name("world_viewer_live"))) {
        candidates.push(path_entry);
    }

    for candidate in candidates {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err("failed to locate `world_viewer_live` binary; build it first or set AGENT_WORLD_WORLD_VIEWER_LIVE_BIN".to_string())
}

fn resolve_world_chain_runtime_binary() -> Result<PathBuf, String> {
    if let Ok(path) = env::var("AGENT_WORLD_WORLD_CHAIN_RUNTIME_BIN") {
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            return Ok(candidate);
        }
        return Err(format!(
            "AGENT_WORLD_WORLD_CHAIN_RUNTIME_BIN is set but file does not exist: {}",
            candidate.display()
        ));
    }

    let mut candidates = Vec::new();
    if let Ok(current_exe) = env::current_exe() {
        if let Some(dir) = current_exe.parent() {
            candidates.push(dir.join(binary_name("world_chain_runtime")));
            candidates.push(
                dir.join("..")
                    .join(binary_name("world_chain_runtime"))
                    .to_path_buf(),
            );
        }
    }

    if let Some(path_entry) = find_on_path(OsStr::new(&binary_name("world_chain_runtime"))) {
        candidates.push(path_entry);
    }

    for candidate in candidates {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err("failed to locate `world_chain_runtime` binary; build it first or set AGENT_WORLD_WORLD_CHAIN_RUNTIME_BIN".to_string())
}

fn resolve_viewer_static_dir(raw: &str) -> Result<PathBuf, String> {
    let user_path = PathBuf::from(raw);
    if user_path.is_dir() {
        return Ok(user_path);
    }

    if user_path.is_relative() {
        if let Ok(current_exe) = env::current_exe() {
            if let Some(bin_dir) = current_exe.parent() {
                let sibling_candidate = bin_dir.join("..").join(&user_path);
                if sibling_candidate.is_dir() {
                    return Ok(sibling_candidate);
                }
            }
        }
    }

    let dev_fallback = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("agent_world_viewer")
        .join("dist");
    if raw == DEFAULT_VIEWER_STATIC_DIR && dev_fallback.is_dir() {
        return Ok(dev_fallback);
    }

    Err(format!(
        "viewer static dir not found: `{raw}`; provide --viewer-static-dir <path> (expected trunk build output)"
    ))
}

fn binary_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}

fn find_on_path(file_name: &OsStr) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(file_name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let status = Command::new("open")
            .arg(url)
            .status()
            .map_err(|err| format!("failed to execute `open`: {err}"))?;
        if status.success() {
            return Ok(());
        }
        return Err(format!("`open` exited with status {status}"));
    }

    #[cfg(target_os = "windows")]
    {
        let status = Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg("")
            .arg(url)
            .status()
            .map_err(|err| format!("failed to execute `cmd /C start`: {err}"))?;
        if status.success() {
            return Ok(());
        }
        return Err(format!("`cmd /C start` exited with status {status}"));
    }

    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        let status = Command::new("xdg-open")
            .arg(url)
            .status()
            .map_err(|err| format!("failed to execute `xdg-open`: {err}"))?;
        if status.success() {
            return Ok(());
        }
        Err(format!("`xdg-open` exited with status {status}"))
    }
}

fn print_help() {
    println!(
        "Usage: world_game_launcher [options]\n\n\
Start player stack with one command:\n\
- start world_chain_runtime (default)\n\
- start world_viewer_live\n\
- start built-in static web server\n\
- print URL and optionally open browser\n\n\
Options:\n\
  --scenario <name>            world_viewer_live scenario (default: {DEFAULT_SCENARIO})\n\
  --live-bind <host:port>      world_viewer_live bind (default: {DEFAULT_LIVE_BIND})\n\
  --web-bind <host:port>       world_viewer_live web bridge bind (default: {DEFAULT_WEB_BIND})\n\
  --viewer-host <host>         web viewer host (default: {DEFAULT_VIEWER_HOST})\n\
  --viewer-port <port>         web viewer port (default: {DEFAULT_VIEWER_PORT})\n\
  --viewer-static-dir <path>   prebuilt web asset dir (default: {DEFAULT_VIEWER_STATIC_DIR})\n\
  --chain-enable               enable world_chain_runtime (default)\n\
  --chain-disable              disable world_chain_runtime\n\
  --chain-status-bind <addr>   world_chain_runtime status bind (default: {DEFAULT_CHAIN_STATUS_BIND})\n\
  --chain-node-id <id>         world_chain_runtime node id (default: {DEFAULT_CHAIN_NODE_ID})\n\
  --chain-world-id <id>        world_chain_runtime world id (default: live-<scenario>)\n\
  --chain-node-role <role>     world_chain_runtime role (default: {DEFAULT_CHAIN_NODE_ROLE})\n\
  --chain-node-tick-ms <n>     world_chain_runtime tick ms (default: {DEFAULT_CHAIN_NODE_TICK_MS})\n\
  --chain-node-validator <v:s> world_chain_runtime validator (repeatable)\n\
  --with-llm                   enable llm mode\n\
  --no-open-browser            do not auto open browser\n\
  -h, --help                   show help\n\n\
Env:\n\
  AGENT_WORLD_WORLD_VIEWER_LIVE_BIN   explicit path of world_viewer_live binary\n\
  AGENT_WORLD_WORLD_CHAIN_RUNTIME_BIN explicit path of world_chain_runtime binary"
    );
}

#[cfg(test)]
mod world_game_launcher_tests {
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        build_game_url, content_type_for_path, parse_host_port, parse_options,
        resolve_static_asset_path, sanitize_relative_request_path, CliOptions,
        DEFAULT_CHAIN_NODE_ID, DEFAULT_CHAIN_STATUS_BIND, DEFAULT_LIVE_BIND, DEFAULT_SCENARIO,
    };

    #[test]
    fn parse_options_defaults() {
        let options = parse_options(std::iter::empty()).expect("parse should succeed");
        assert_eq!(options.scenario, DEFAULT_SCENARIO);
        assert_eq!(options.live_bind, DEFAULT_LIVE_BIND);
        assert!(!options.with_llm);
        assert!(options.open_browser);
        assert_eq!(options.viewer_static_dir, "web");
        assert!(options.chain_enabled);
        assert_eq!(options.chain_status_bind, DEFAULT_CHAIN_STATUS_BIND);
        assert_eq!(options.chain_node_id, DEFAULT_CHAIN_NODE_ID);
        assert_eq!(options.chain_node_role, "sequencer");
    }

    #[test]
    fn parse_options_accepts_overrides() {
        let options = parse_options(
            [
                "--scenario",
                "twin_region_bootstrap",
                "--live-bind",
                "127.0.0.1:6200",
                "--web-bind",
                "127.0.0.1:6201",
                "--viewer-host",
                "0.0.0.0",
                "--viewer-port",
                "4777",
                "--viewer-static-dir",
                "dist",
                "--chain-status-bind",
                "127.0.0.1:6331",
                "--chain-node-id",
                "chain-a",
                "--chain-world-id",
                "live-chain-a",
                "--chain-node-role",
                "storage",
                "--chain-node-tick-ms",
                "350",
                "--chain-node-validator",
                "chain-a:55",
                "--with-llm",
                "--no-open-browser",
            ]
            .into_iter(),
        )
        .expect("parse should succeed");

        assert_eq!(options.scenario, "twin_region_bootstrap");
        assert_eq!(options.live_bind, "127.0.0.1:6200");
        assert_eq!(options.web_bind, "127.0.0.1:6201");
        assert_eq!(options.viewer_host, "0.0.0.0");
        assert_eq!(options.viewer_port, 4777);
        assert_eq!(options.viewer_static_dir, "dist");
        assert_eq!(options.chain_status_bind, "127.0.0.1:6331");
        assert_eq!(options.chain_node_id, "chain-a");
        assert_eq!(options.chain_world_id, Some("live-chain-a".to_string()));
        assert_eq!(options.chain_node_role, "storage");
        assert_eq!(options.chain_node_tick_ms, 350);
        assert_eq!(
            options.chain_node_validators,
            vec!["chain-a:55".to_string()]
        );
        assert!(options.with_llm);
        assert!(!options.open_browser);
    }

    #[test]
    fn parse_options_accepts_chain_disable() {
        let options = parse_options(["--chain-disable"].into_iter()).expect("parse should succeed");
        assert!(!options.chain_enabled);
    }

    #[test]
    fn parse_options_rejects_invalid_chain_role() {
        let err =
            parse_options(["--chain-node-role", "invalid"].into_iter()).expect_err("should fail");
        assert!(err.contains("sequencer, storage, observer"));
    }

    #[test]
    fn parse_options_rejects_unknown_option() {
        let err = parse_options(["--unknown"].into_iter()).expect_err("should fail");
        assert!(err.contains("unknown option"));
    }

    #[test]
    fn parse_options_rejects_missing_value() {
        let err = parse_options(["--viewer-port"].into_iter()).expect_err("should fail");
        assert!(err.contains("requires a value"));
    }

    #[test]
    fn parse_options_rejects_invalid_port() {
        let err = parse_options(["--viewer-port", "70000"].into_iter()).expect_err("should fail");
        assert!(err.contains("integer"));
    }

    #[test]
    fn parse_options_rejects_invalid_bind_format() {
        let err = parse_options(["--live-bind", "127.0.0.1"].into_iter()).expect_err("should fail");
        assert!(err.contains("<host:port>"));
    }

    #[test]
    fn parse_host_port_parses_valid_value() {
        let (host, port) = parse_host_port("127.0.0.1:5011", "--web-bind").expect("ok");
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 5011);
    }

    #[test]
    fn parse_host_port_rejects_zero_port() {
        let err = parse_host_port("127.0.0.1:0", "--web-bind").expect_err("should fail");
        assert!(err.contains("1..=65535"));
    }

    #[test]
    fn build_game_url_rewrites_zero_bind_host_to_loopback() {
        let options = CliOptions {
            viewer_host: "0.0.0.0".to_string(),
            viewer_port: 4173,
            web_bind: "0.0.0.0:5011".to_string(),
            ..CliOptions::default()
        };
        let url = build_game_url(&options);
        assert_eq!(url, "http://127.0.0.1:4173/?ws=ws://127.0.0.1:5011");
    }

    #[test]
    fn sanitize_relative_request_path_rejects_traversal() {
        let err = sanitize_relative_request_path("/../etc/passwd").expect_err("should fail");
        assert!(err.contains("traversal"));
    }

    #[test]
    fn resolve_static_asset_path_supports_spa_fallback() {
        let temp_dir = make_temp_dir("spa_fallback");
        fs::write(temp_dir.join("index.html"), "<html>ok</html>").expect("write index");
        let resolved = resolve_static_asset_path(temp_dir.as_path(), "/app/route?x=1")
            .expect("resolve should succeed")
            .expect("should fallback to index");
        assert_eq!(resolved, temp_dir.join("index.html"));
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn resolve_static_asset_path_returns_none_for_missing_static_asset() {
        let temp_dir = make_temp_dir("missing_asset");
        fs::write(temp_dir.join("index.html"), "<html>ok</html>").expect("write index");
        let resolved = resolve_static_asset_path(temp_dir.as_path(), "/assets/missing.js")
            .expect("resolve should succeed");
        assert!(resolved.is_none());
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn content_type_for_path_covers_wasm_and_js() {
        assert_eq!(
            content_type_for_path(Path::new("a.wasm")),
            "application/wasm"
        );
        assert_eq!(
            content_type_for_path(Path::new("a.js")),
            "text/javascript; charset=utf-8"
        );
    }

    fn make_temp_dir(label: &str) -> PathBuf {
        let mut path = env::temp_dir();
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        path.push(format!(
            "agent_world_launcher_test_{label}_{}_{}",
            std::process::id(),
            stamp
        ));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }
}
