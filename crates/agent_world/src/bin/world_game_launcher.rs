use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{self, Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use agent_world::simulator::ProviderExecutionMode;
use agent_world_proto::storage_profile::StorageProfile;

const DEFAULT_SCENARIO: &str = "llm_bootstrap";
const DEFAULT_LIVE_BIND: &str = "127.0.0.1:5023";
const DEFAULT_WEB_BIND: &str = "127.0.0.1:5011";
const DEFAULT_VIEWER_HOST: &str = "127.0.0.1";
const DEFAULT_VIEWER_PORT: u16 = 4173;
const DEFAULT_VIEWER_STATIC_DIR: &str = "web";
const GAME_STATIC_DIR_ENV: &str = "OASIS7_GAME_STATIC_DIR";
const LEGACY_GAME_STATIC_DIR_ENV: &str = "AGENT_WORLD_GAME_STATIC_DIR";
const WORLD_VIEWER_LIVE_BIN_ENV: &str = "OASIS7_WORLD_VIEWER_LIVE_BIN";
const LEGACY_WORLD_VIEWER_LIVE_BIN_ENV: &str = "AGENT_WORLD_WORLD_VIEWER_LIVE_BIN";
const WORLD_CHAIN_RUNTIME_BIN_ENV: &str = "OASIS7_WORLD_CHAIN_RUNTIME_BIN";
const LEGACY_WORLD_CHAIN_RUNTIME_BIN_ENV: &str = "AGENT_WORLD_WORLD_CHAIN_RUNTIME_BIN";
const BUILTIN_LLM_PROVIDER_MODE: &str = "builtin_llm";
const OPENCLAW_LOCAL_HTTP_PROVIDER_MODE: &str = "openclaw_local_http";
const DEFAULT_OPENCLAW_BASE_URL: &str = "http://127.0.0.1:5841";
const DEFAULT_OPENCLAW_CONNECT_TIMEOUT_MS: u64 = 3_000;
const DEFAULT_OPENCLAW_AGENT_PROFILE: &str = "oasis7_p0_low_freq_npc";
const VIEWER_AGENT_PROVIDER_MODE_ENV: &str = "OASIS7_AGENT_PROVIDER_MODE";
const LEGACY_VIEWER_AGENT_PROVIDER_MODE_ENV: &str = "AGENT_WORLD_AGENT_PROVIDER_MODE";
const VIEWER_OPENCLAW_BASE_URL_ENV: &str = "OASIS7_OPENCLAW_BASE_URL";
const LEGACY_VIEWER_OPENCLAW_BASE_URL_ENV: &str = "AGENT_WORLD_OPENCLAW_BASE_URL";
const VIEWER_OPENCLAW_AUTH_TOKEN_ENV: &str = "OASIS7_OPENCLAW_AUTH_TOKEN";
const LEGACY_VIEWER_OPENCLAW_AUTH_TOKEN_ENV: &str = "AGENT_WORLD_OPENCLAW_AUTH_TOKEN";
const VIEWER_OPENCLAW_CONNECT_TIMEOUT_MS_ENV: &str = "OASIS7_OPENCLAW_CONNECT_TIMEOUT_MS";
const LEGACY_VIEWER_OPENCLAW_CONNECT_TIMEOUT_MS_ENV: &str =
    "AGENT_WORLD_OPENCLAW_CONNECT_TIMEOUT_MS";
const VIEWER_OPENCLAW_AGENT_PROFILE_ENV: &str = "OASIS7_OPENCLAW_AGENT_PROFILE";
const LEGACY_VIEWER_OPENCLAW_AGENT_PROFILE_ENV: &str = "AGENT_WORLD_OPENCLAW_AGENT_PROFILE";
const VIEWER_OPENCLAW_EXECUTION_MODE_ENV: &str = "OASIS7_OPENCLAW_EXECUTION_MODE";
const LEGACY_VIEWER_OPENCLAW_EXECUTION_MODE_ENV: &str = "AGENT_WORLD_OPENCLAW_EXECUTION_MODE";
const DEFAULT_VIEWER_PLAYER_ID: &str = "viewer-player";
const DEFAULT_CHAIN_STATUS_BIND: &str = "127.0.0.1:5121";
const DEFAULT_CHAIN_NODE_ID: &str = "viewer-live-node";
const DEFAULT_CHAIN_NODE_ROLE: &str = "sequencer";
const DEFAULT_CHAIN_NODE_TICK_MS: u64 = 200;
const DEFAULT_CHAIN_POS_SLOT_DURATION_MS: u64 = 12_000;
const DEFAULT_CHAIN_POS_TICKS_PER_SLOT: u64 = 10;
const DEFAULT_CHAIN_POS_PROPOSAL_TICK_PHASE: u64 = 9;
const DEFAULT_CHAIN_POS_MAX_PAST_SLOT_LAG: u64 = 256;
const VIEWER_PLAYER_ID_ENV: &str = "OASIS7_VIEWER_PLAYER_ID";
const LEGACY_VIEWER_PLAYER_ID_ENV: &str = "AGENT_WORLD_VIEWER_PLAYER_ID";
const VIEWER_AUTH_PUBLIC_KEY_ENV: &str = "OASIS7_VIEWER_AUTH_PUBLIC_KEY";
const LEGACY_VIEWER_AUTH_PUBLIC_KEY_ENV: &str = "AGENT_WORLD_VIEWER_AUTH_PUBLIC_KEY";
const VIEWER_AUTH_PRIVATE_KEY_ENV: &str = "OASIS7_VIEWER_AUTH_PRIVATE_KEY";
const LEGACY_VIEWER_AUTH_PRIVATE_KEY_ENV: &str = "AGENT_WORLD_VIEWER_AUTH_PRIVATE_KEY";
const VIEWER_AUTH_BOOTSTRAP_OBJECT: &str = "__OASIS7_VIEWER_AUTH_ENV";
const LEGACY_VIEWER_AUTH_BOOTSTRAP_OBJECT: &str = "__AGENT_WORLD_VIEWER_AUTH_ENV";
const NODE_CONFIG_FILE_NAME: &str = "config.toml";
const NODE_TABLE_KEY: &str = "node";
const NODE_PRIVATE_KEY_FIELD: &str = "private_key";
const NODE_PUBLIC_KEY_FIELD: &str = "public_key";
static TERMINATION_REQUESTED: AtomicBool = AtomicBool::new(false);
static SIGNAL_HANDLER_INSTALL: OnceLock<Result<(), String>> = OnceLock::new();

fn default_chain_node_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{DEFAULT_CHAIN_NODE_ID}-fresh-{}-{now}", process::id())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ViewerAuthBootstrap {
    player_id: String,
    public_key: String,
    private_key: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliOptions {
    scenario: String,
    live_bind: String,
    web_bind: String,
    viewer_host: String,
    viewer_port: u16,
    viewer_static_dir: String,
    with_llm: bool,
    agent_provider_mode: String,
    openclaw_base_url: String,
    openclaw_auth_token: String,
    openclaw_connect_timeout_ms: u64,
    openclaw_agent_profile: String,
    openclaw_execution_mode: ProviderExecutionMode,
    open_browser: bool,
    chain_enabled: bool,
    chain_status_bind: String,
    chain_node_id: String,
    chain_storage_profile: StorageProfile,
    chain_world_id: Option<String>,
    chain_node_role: String,
    chain_node_tick_ms: u64,
    chain_pos_slot_duration_ms: u64,
    chain_pos_ticks_per_slot: u64,
    chain_pos_proposal_tick_phase: u64,
    chain_pos_adaptive_tick_scheduler_enabled: bool,
    chain_pos_slot_clock_genesis_unix_ms: Option<i64>,
    chain_pos_max_past_slot_lag: u64,
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
            agent_provider_mode: BUILTIN_LLM_PROVIDER_MODE.to_string(),
            openclaw_base_url: DEFAULT_OPENCLAW_BASE_URL.to_string(),
            openclaw_auth_token: String::new(),
            openclaw_connect_timeout_ms: DEFAULT_OPENCLAW_CONNECT_TIMEOUT_MS,
            openclaw_agent_profile: DEFAULT_OPENCLAW_AGENT_PROFILE.to_string(),
            openclaw_execution_mode: ProviderExecutionMode::HeadlessAgent,
            open_browser: true,
            chain_enabled: true,
            chain_status_bind: DEFAULT_CHAIN_STATUS_BIND.to_string(),
            chain_node_id: default_chain_node_id(),
            chain_storage_profile: StorageProfile::DevLocal,
            chain_world_id: None,
            chain_node_role: DEFAULT_CHAIN_NODE_ROLE.to_string(),
            chain_node_tick_ms: DEFAULT_CHAIN_NODE_TICK_MS,
            chain_pos_slot_duration_ms: DEFAULT_CHAIN_POS_SLOT_DURATION_MS,
            chain_pos_ticks_per_slot: DEFAULT_CHAIN_POS_TICKS_PER_SLOT,
            chain_pos_proposal_tick_phase: DEFAULT_CHAIN_POS_PROPOSAL_TICK_PHASE,
            chain_pos_adaptive_tick_scheduler_enabled: false,
            chain_pos_slot_clock_genesis_unix_ms: None,
            chain_pos_max_past_slot_lag: DEFAULT_CHAIN_POS_MAX_PAST_SLOT_LAG,
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
    install_signal_handler()?;
    TERMINATION_REQUESTED.store(false, Ordering::SeqCst);

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
    let mut server = match start_static_http_server(
        options.viewer_host.as_str(),
        options.viewer_port,
        viewer_static_dir.as_path(),
    ) {
        Ok(server) => server,
        Err(err) => {
            terminate_child(&mut world_child);
            if let Some(child) = chain_child.as_mut() {
                terminate_child(child);
            }
            return Err(err);
        }
    };

    let ready_result = wait_until_ready(options, &mut world_child, chain_child.as_mut());
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

fn install_signal_handler() -> Result<(), String> {
    SIGNAL_HANDLER_INSTALL
        .get_or_init(|| {
            ctrlc::set_handler(|| {
                TERMINATION_REQUESTED.store(true, Ordering::SeqCst);
            })
            .map_err(|err| format!("failed to install signal handler: {err}"))
        })
        .clone()
}

fn spawn_world_viewer_live(path: &Path, options: &CliOptions) -> Result<Child, String> {
    let mut command = Command::new(path);
    command
        .arg(options.scenario.as_str())
        .arg("--bind")
        .arg(options.live_bind.as_str())
        .arg("--web-bind")
        .arg(options.web_bind.as_str());
    for env_name in [
        VIEWER_AGENT_PROVIDER_MODE_ENV,
        LEGACY_VIEWER_AGENT_PROVIDER_MODE_ENV,
        VIEWER_OPENCLAW_BASE_URL_ENV,
        LEGACY_VIEWER_OPENCLAW_BASE_URL_ENV,
        VIEWER_OPENCLAW_AUTH_TOKEN_ENV,
        LEGACY_VIEWER_OPENCLAW_AUTH_TOKEN_ENV,
        VIEWER_OPENCLAW_CONNECT_TIMEOUT_MS_ENV,
        LEGACY_VIEWER_OPENCLAW_CONNECT_TIMEOUT_MS_ENV,
        VIEWER_OPENCLAW_AGENT_PROFILE_ENV,
        LEGACY_VIEWER_OPENCLAW_AGENT_PROFILE_ENV,
        VIEWER_OPENCLAW_EXECUTION_MODE_ENV,
        LEGACY_VIEWER_OPENCLAW_EXECUTION_MODE_ENV,
    ] {
        command.env_remove(env_name);
    }
    if options.with_llm {
        command.arg("--llm");
        if options.agent_provider_mode == OPENCLAW_LOCAL_HTTP_PROVIDER_MODE {
            command.env(
                VIEWER_AGENT_PROVIDER_MODE_ENV,
                OPENCLAW_LOCAL_HTTP_PROVIDER_MODE,
            );
            command.env(
                VIEWER_OPENCLAW_BASE_URL_ENV,
                options.openclaw_base_url.as_str(),
            );
            if !options.openclaw_auth_token.trim().is_empty() {
                command.env(
                    VIEWER_OPENCLAW_AUTH_TOKEN_ENV,
                    options.openclaw_auth_token.as_str(),
                );
            }
            command.env(
                VIEWER_OPENCLAW_CONNECT_TIMEOUT_MS_ENV,
                options.openclaw_connect_timeout_ms.to_string(),
            );
            command.env(
                VIEWER_OPENCLAW_AGENT_PROFILE_ENV,
                options.openclaw_agent_profile.as_str(),
            );
            command.env(
                VIEWER_OPENCLAW_EXECUTION_MODE_ENV,
                options.openclaw_execution_mode.as_str(),
            );
        }
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
    command.args(build_world_chain_runtime_args(options));
    command
        .arg("--node-role")
        .arg(options.chain_node_role.as_str())
        .arg("--node-tick-ms")
        .arg(options.chain_node_tick_ms.to_string())
        .arg("--pos-slot-duration-ms")
        .arg(options.chain_pos_slot_duration_ms.to_string())
        .arg("--pos-ticks-per-slot")
        .arg(options.chain_pos_ticks_per_slot.to_string())
        .arg("--pos-proposal-tick-phase")
        .arg(options.chain_pos_proposal_tick_phase.to_string())
        .arg("--pos-max-past-slot-lag")
        .arg(options.chain_pos_max_past_slot_lag.to_string())
        .arg(if options.chain_pos_adaptive_tick_scheduler_enabled {
            "--pos-adaptive-tick-scheduler"
        } else {
            "--pos-no-adaptive-tick-scheduler"
        });
    if let Some(genesis) = options.chain_pos_slot_clock_genesis_unix_ms {
        command
            .arg("--pos-slot-clock-genesis-unix-ms")
            .arg(genesis.to_string());
    }
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

fn chain_execution_world_dir(node_id: &str) -> String {
    Path::new("output")
        .join("chain-runtime")
        .join(node_id)
        .join("reward-runtime-execution-world")
        .to_string_lossy()
        .into_owned()
}

fn build_world_chain_runtime_args(options: &CliOptions) -> Vec<String> {
    let execution_world_dir = chain_execution_world_dir(options.chain_node_id.as_str());
    vec![
        "--node-id".to_string(),
        options.chain_node_id.clone(),
        "--world-id".to_string(),
        chain_world_id(options),
        "--status-bind".to_string(),
        options.chain_status_bind.clone(),
        "--storage-profile".to_string(),
        options.chain_storage_profile.as_str().to_string(),
        "--execution-world-dir".to_string(),
        execution_world_dir,
    ]
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
            let viewer_auth_bootstrap =
                resolve_viewer_auth_bootstrap_from_path(Path::new(NODE_CONFIG_FILE_NAME)).ok();
            let body = sanitize_index_html_for_embedded_server(
                path.as_path(),
                body.as_slice(),
                viewer_auth_bootstrap.as_ref(),
            );
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

fn sanitize_index_html_for_embedded_server(
    path: &Path,
    body: &[u8],
    viewer_auth_bootstrap: Option<&ViewerAuthBootstrap>,
) -> Vec<u8> {
    if path.extension() != Some(OsStr::new("html")) {
        return body.to_vec();
    }
    let sanitized = if path.file_name() == Some(OsStr::new("index.html")) {
        strip_trunk_autoreload_script(body)
    } else {
        body.to_vec()
    };
    if let Some(viewer_auth_bootstrap) = viewer_auth_bootstrap {
        inject_viewer_auth_bootstrap_script(sanitized.as_slice(), viewer_auth_bootstrap)
    } else {
        sanitized
    }
}

fn strip_trunk_autoreload_script(body: &[u8]) -> Vec<u8> {
    let html = String::from_utf8_lossy(body);
    let marker = ".well-known/trunk/ws";
    let Some(marker_index) = html.find(marker) else {
        return body.to_vec();
    };
    let Some(script_start) = html[..marker_index].rfind("<script") else {
        return body.to_vec();
    };
    let Some(script_end_rel) = html[marker_index..].find("</script>") else {
        return body.to_vec();
    };
    let script_end = marker_index + script_end_rel + "</script>".len();

    let mut sanitized = String::with_capacity(html.len());
    sanitized.push_str(&html[..script_start]);
    sanitized.push_str(&html[script_end..]);
    sanitized.into_bytes()
}

fn inject_viewer_auth_bootstrap_script(body: &[u8], auth: &ViewerAuthBootstrap) -> Vec<u8> {
    let html = String::from_utf8_lossy(body);
    let script = build_viewer_auth_bootstrap_script(auth);
    let insert_at = html
        .rfind("</head>")
        .or_else(|| html.rfind("</body>"))
        .unwrap_or(html.len());
    let mut injected = String::with_capacity(html.len() + script.len() + 1);
    injected.push_str(&html[..insert_at]);
    injected.push_str(script.as_str());
    injected.push_str(&html[insert_at..]);
    injected.into_bytes()
}

fn build_viewer_auth_bootstrap_script(auth: &ViewerAuthBootstrap) -> String {
    let payload = serde_json::json!({
        VIEWER_PLAYER_ID_ENV: auth.player_id,
        LEGACY_VIEWER_PLAYER_ID_ENV: auth.player_id,
        VIEWER_AUTH_PUBLIC_KEY_ENV: auth.public_key,
        LEGACY_VIEWER_AUTH_PUBLIC_KEY_ENV: auth.public_key,
        VIEWER_AUTH_PRIVATE_KEY_ENV: auth.private_key,
        LEGACY_VIEWER_AUTH_PRIVATE_KEY_ENV: auth.private_key,
    });
    let payload = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
    format!(
        "<script>const __oasis7ViewerAuthEnv=Object.freeze({payload});window.{VIEWER_AUTH_BOOTSTRAP_OBJECT}=__oasis7ViewerAuthEnv;window.{LEGACY_VIEWER_AUTH_BOOTSTRAP_OBJECT}=__oasis7ViewerAuthEnv;</script>"
    )
}

fn resolve_viewer_auth_bootstrap_from_path(path: &Path) -> Result<ViewerAuthBootstrap, String> {
    let content =
        fs::read_to_string(path).map_err(|err| format!("read {} failed: {err}", path.display()))?;
    let value: toml::Value = toml::from_str(content.as_str())
        .map_err(|err| format!("parse {} failed: {err}", path.display()))?;
    let node = value
        .get(NODE_TABLE_KEY)
        .and_then(toml::Value::as_table)
        .ok_or_else(|| format!("{NODE_TABLE_KEY} table is missing in {}", path.display()))?;
    let private_key =
        resolve_required_toml_string(node, NODE_PRIVATE_KEY_FIELD, "node.private_key")?;
    let public_key = resolve_required_toml_string(node, NODE_PUBLIC_KEY_FIELD, "node.public_key")?;
    let player_id = env::var(VIEWER_PLAYER_ID_ENV)
        .ok()
        .or_else(|| env::var(LEGACY_VIEWER_PLAYER_ID_ENV).ok())
        .map(|raw| raw.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_VIEWER_PLAYER_ID.to_string());
    Ok(ViewerAuthBootstrap {
        player_id,
        public_key,
        private_key,
    })
}

fn resolve_required_toml_string(
    table: &toml::value::Table,
    key: &str,
    label: &str,
) -> Result<String, String> {
    let value = table
        .get(key)
        .and_then(toml::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{label} is missing or empty"))?;
    Ok(value.to_string())
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

fn wait_until_ready(
    options: &CliOptions,
    world_child: &mut Child,
    mut chain_child: Option<&mut Child>,
) -> Result<(), String> {
    let (viewer_host, viewer_port) = normalize_http_target(
        options.viewer_host.as_str(),
        options.viewer_port,
        "viewer host/port",
    )?;
    poll_startup_health(world_child, chain_child.as_deref_mut())?;
    wait_for_http_ready(
        viewer_host.as_str(),
        viewer_port,
        Duration::from_secs(30),
        world_child,
        chain_child.as_deref_mut(),
    )
    .map_err(|err| {
        format!("viewer HTTP did not become ready at {viewer_host}:{viewer_port}: {err}")
    })?;
    poll_startup_health(world_child, chain_child.as_deref_mut())?;

    let (bridge_host, bridge_port) = parse_host_port(options.web_bind.as_str(), "--web-bind")?;
    wait_for_tcp_ready(
        bridge_host.as_str(),
        bridge_port,
        Duration::from_secs(60),
        world_child,
        chain_child.as_deref_mut(),
    )
    .map_err(|err| {
        format!("web bridge did not become ready at {bridge_host}:{bridge_port}: {err}")
    })?;
    poll_startup_health(world_child, chain_child.as_deref_mut())?;

    if options.chain_enabled {
        let (chain_status_host, chain_status_port) =
            parse_host_port(options.chain_status_bind.as_str(), "--chain-status-bind")?;
        let chain_status_host = normalize_bind_host_for_local_access(chain_status_host.as_str());
        wait_for_http_ready(
            chain_status_host.as_str(),
            chain_status_port,
            Duration::from_secs(30),
            world_child,
            chain_child.as_deref_mut(),
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
        if TERMINATION_REQUESTED.load(Ordering::SeqCst) {
            return Ok(());
        }
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

fn wait_for_tcp_ready(
    host: &str,
    port: u16,
    timeout: Duration,
    world_child: &mut Child,
    mut chain_child: Option<&mut Child>,
) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        poll_startup_health(world_child, chain_child.as_deref_mut())?;
        match TcpStream::connect((host, port)) {
            Ok(_) => {
                poll_startup_health(world_child, chain_child.as_deref_mut())?;
                return Ok(());
            }
            Err(_) => thread::sleep(Duration::from_millis(200)),
        }
    }
    poll_startup_health(world_child, chain_child.as_deref_mut())?;
    Err(format!("timeout after {}s", timeout.as_secs()))
}

fn wait_for_http_ready(
    host: &str,
    port: u16,
    timeout: Duration,
    world_child: &mut Child,
    mut chain_child: Option<&mut Child>,
) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    let request = format!("GET / HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n");

    while Instant::now() < deadline {
        poll_startup_health(world_child, chain_child.as_deref_mut())?;
        if let Ok(mut stream) = TcpStream::connect((host, port)) {
            let _ = stream.write_all(request.as_bytes());
            let mut buf = [0u8; 256];
            match stream.read(&mut buf) {
                Ok(0) => {}
                Ok(bytes) => {
                    let response = String::from_utf8_lossy(&buf[..bytes]);
                    if response.starts_with("HTTP/") {
                        poll_startup_health(world_child, chain_child.as_deref_mut())?;
                        return Ok(());
                    }
                }
                Err(_) => {}
            }
        }
        thread::sleep(Duration::from_millis(200));
    }

    poll_startup_health(world_child, chain_child.as_deref_mut())?;
    Err(format!("timeout after {}s", timeout.as_secs()))
}

fn poll_startup_health(
    world_child: &mut Child,
    chain_child: Option<&mut Child>,
) -> Result<(), String> {
    if TERMINATION_REQUESTED.load(Ordering::SeqCst) {
        return Err("termination requested".to_string());
    }
    if let Some(status) = world_child
        .try_wait()
        .map_err(|err| format!("failed to query world_viewer_live status during startup: {err}"))?
    {
        return Err(format!("world_viewer_live exited during startup: {status}"));
    }
    if let Some(chain_child) = chain_child {
        if let Some(status) = chain_child.try_wait().map_err(|err| {
            format!("failed to query world_chain_runtime status during startup: {err}")
        })? {
            return Err(format!(
                "world_chain_runtime exited during startup: {status}"
            ));
        }
    }
    Ok(())
}

fn build_game_url(options: &CliOptions) -> String {
    let viewer_host = normalize_bind_host_for_local_access(options.viewer_host.as_str());
    let viewer_host = host_for_url(viewer_host.as_str());
    let (bridge_host, bridge_port) = parse_host_port(options.web_bind.as_str(), "--web-bind")
        .unwrap_or_else(|_| ("127.0.0.1".to_string(), 5011));
    let bridge_host = normalize_bind_host_for_local_access(bridge_host.as_str());
    let bridge_host = host_for_url(bridge_host.as_str());
    format!(
        "http://{viewer_host}:{}/?ws=ws://{bridge_host}:{bridge_port}",
        options.viewer_port
    )
}

fn normalize_http_target(host: &str, port: u16, label: &str) -> Result<(String, u16), String> {
    let normalized = normalize_bind_host_for_local_access(host);
    if normalized.trim().is_empty() {
        return Err(format!("{label} host cannot be empty"));
    }
    Ok((normalized, port))
}

fn normalize_bind_host_for_local_access(host: &str) -> String {
    let trimmed = host.trim();
    if trimmed == "0.0.0.0" || trimmed == "::" || trimmed == "[::]" {
        "127.0.0.1".to_string()
    } else {
        trimmed.to_string()
    }
}

fn host_for_url(host: &str) -> String {
    if host.contains(':') && !host.starts_with('[') && !host.ends_with(']') {
        format!("[{host}]")
    } else {
        host.to_string()
    }
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
            "--agent-provider-mode" => {
                options.agent_provider_mode =
                    parse_required_value(&mut iter, "--agent-provider-mode")?;
            }
            "--openclaw-base-url" => {
                options.openclaw_base_url = parse_required_value(&mut iter, "--openclaw-base-url")?;
            }
            "--openclaw-auth-token" => {
                options.openclaw_auth_token =
                    parse_required_value(&mut iter, "--openclaw-auth-token")?;
            }
            "--openclaw-connect-timeout-ms" => {
                let raw = parse_required_value(&mut iter, "--openclaw-connect-timeout-ms")?;
                options.openclaw_connect_timeout_ms = raw.parse::<u64>().map_err(|_| {
                    format!("--openclaw-connect-timeout-ms must be a positive integer, got `{raw}`")
                })?;
                if options.openclaw_connect_timeout_ms == 0 {
                    return Err(
                        "--openclaw-connect-timeout-ms must be a positive integer".to_string()
                    );
                }
            }
            "--openclaw-agent-profile" => {
                options.openclaw_agent_profile =
                    parse_required_value(&mut iter, "--openclaw-agent-profile")?;
            }
            "--openclaw-execution-mode" => {
                let raw = parse_required_value(&mut iter, "--openclaw-execution-mode")?;
                options.openclaw_execution_mode = ProviderExecutionMode::parse(raw.as_str())
                    .ok_or_else(|| {
                        format!(
                            "--openclaw-execution-mode must be one of player_parity or headless_agent, got `{raw}`"
                        )
                    })?;
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
            "--chain-storage-profile" => {
                options.chain_storage_profile =
                    parse_required_value(&mut iter, "--chain-storage-profile")?
                        .parse::<StorageProfile>()?;
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
            "--chain-pos-slot-duration-ms" => {
                let raw = parse_required_value(&mut iter, "--chain-pos-slot-duration-ms")?;
                options.chain_pos_slot_duration_ms = raw.parse::<u64>().map_err(|_| {
                    format!("--chain-pos-slot-duration-ms must be a positive integer, got `{raw}`")
                })?;
                if options.chain_pos_slot_duration_ms == 0 {
                    return Err(
                        "--chain-pos-slot-duration-ms must be a positive integer".to_string()
                    );
                }
            }
            "--chain-pos-ticks-per-slot" => {
                let raw = parse_required_value(&mut iter, "--chain-pos-ticks-per-slot")?;
                options.chain_pos_ticks_per_slot = raw.parse::<u64>().map_err(|_| {
                    format!("--chain-pos-ticks-per-slot must be a positive integer, got `{raw}`")
                })?;
                if options.chain_pos_ticks_per_slot == 0 {
                    return Err("--chain-pos-ticks-per-slot must be a positive integer".to_string());
                }
            }
            "--chain-pos-proposal-tick-phase" => {
                let raw = parse_required_value(&mut iter, "--chain-pos-proposal-tick-phase")?;
                options.chain_pos_proposal_tick_phase = raw.parse::<u64>().map_err(|_| {
                    format!("--chain-pos-proposal-tick-phase must be a non-negative integer, got `{raw}`")
                })?;
            }
            "--chain-pos-adaptive-tick-scheduler" => {
                options.chain_pos_adaptive_tick_scheduler_enabled = true;
            }
            "--chain-pos-no-adaptive-tick-scheduler" => {
                options.chain_pos_adaptive_tick_scheduler_enabled = false;
            }
            "--chain-pos-slot-clock-genesis-unix-ms" => {
                let raw =
                    parse_required_value(&mut iter, "--chain-pos-slot-clock-genesis-unix-ms")?;
                options.chain_pos_slot_clock_genesis_unix_ms =
                    Some(raw.parse::<i64>().map_err(|_| {
                        format!(
                            "--chain-pos-slot-clock-genesis-unix-ms must be an integer, got `{raw}`"
                        )
                    })?);
            }
            "--chain-pos-max-past-slot-lag" => {
                let raw = parse_required_value(&mut iter, "--chain-pos-max-past-slot-lag")?;
                options.chain_pos_max_past_slot_lag = raw.parse::<u64>().map_err(|_| {
                    format!(
                        "--chain-pos-max-past-slot-lag must be a non-negative integer, got `{raw}`"
                    )
                })?;
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
    validate_agent_provider_mode(options.agent_provider_mode.as_str())?;
    if options.agent_provider_mode == OPENCLAW_LOCAL_HTTP_PROVIDER_MODE {
        if options.openclaw_base_url.trim().is_empty() {
            return Err("--openclaw-base-url requires a non-empty value".to_string());
        }
        if options.openclaw_agent_profile.trim().is_empty() {
            return Err("--openclaw-agent-profile requires a non-empty value".to_string());
        }
    }
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
        if options.chain_pos_slot_duration_ms == 0 {
            return Err("--chain-pos-slot-duration-ms must be a positive integer".to_string());
        }
        if options.chain_pos_ticks_per_slot == 0 {
            return Err("--chain-pos-ticks-per-slot must be a positive integer".to_string());
        }
        if options.chain_pos_proposal_tick_phase >= options.chain_pos_ticks_per_slot {
            return Err(format!(
                "--chain-pos-proposal-tick-phase={} must be less than --chain-pos-ticks-per-slot={}",
                options.chain_pos_proposal_tick_phase, options.chain_pos_ticks_per_slot
            ));
        }
        for validator in &options.chain_node_validators {
            validate_chain_node_validator(validator.as_str())?;
        }
    }

    Ok(options)
}

fn validate_agent_provider_mode(raw: &str) -> Result<(), String> {
    match raw.trim() {
        BUILTIN_LLM_PROVIDER_MODE | OPENCLAW_LOCAL_HTTP_PROVIDER_MODE => Ok(()),
        _ => Err("--agent-provider-mode must be builtin_llm or openclaw_local_http".to_string()),
    }
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
    let (host_raw, port_text) = if let Some(rest) = trimmed.strip_prefix('[') {
        let (host, remainder) = rest
            .split_once(']')
            .ok_or_else(|| format!("{label} IPv6 host must be in [addr]:port format"))?;
        let port_text = remainder
            .strip_prefix(':')
            .ok_or_else(|| format!("{label} must be in <host:port> format"))?;
        (host, port_text)
    } else {
        let (host, port_text) = trimmed
            .rsplit_once(':')
            .ok_or_else(|| format!("{label} must be in <host:port> format"))?;
        if host.contains(':') {
            return Err(format!("{label} IPv6 host must be wrapped in []"));
        }
        (host, port_text)
    };
    let host = host_raw.trim();
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
    if let Some((path, env_name)) =
        resolve_non_empty_env_override(WORLD_VIEWER_LIVE_BIN_ENV, LEGACY_WORLD_VIEWER_LIVE_BIN_ENV)
    {
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            return Ok(candidate);
        }
        return Err(format!(
            "{env_name} is set but file does not exist: {}",
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

    Err(format!(
        "failed to locate `world_viewer_live` binary; build it first or set {WORLD_VIEWER_LIVE_BIN_ENV} (legacy: {LEGACY_WORLD_VIEWER_LIVE_BIN_ENV})"
    ))
}

fn resolve_world_chain_runtime_binary() -> Result<PathBuf, String> {
    if let Some((path, env_name)) = resolve_non_empty_env_override(
        WORLD_CHAIN_RUNTIME_BIN_ENV,
        LEGACY_WORLD_CHAIN_RUNTIME_BIN_ENV,
    ) {
        let candidate = PathBuf::from(path);
        if candidate.is_file() {
            return Ok(candidate);
        }
        return Err(format!(
            "{env_name} is set but file does not exist: {}",
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

    Err(format!(
        "failed to locate `world_chain_runtime` binary; build it first or set {WORLD_CHAIN_RUNTIME_BIN_ENV} (legacy: {LEGACY_WORLD_CHAIN_RUNTIME_BIN_ENV})"
    ))
}

fn resolve_viewer_static_dir(raw: &str) -> Result<PathBuf, String> {
    let env_override =
        resolve_non_empty_env_override(GAME_STATIC_DIR_ENV, LEGACY_GAME_STATIC_DIR_ENV);
    resolve_viewer_static_dir_with_override(
        raw,
        env_override
            .as_ref()
            .map(|(value, env_name)| (value.as_str(), *env_name)),
    )
}

fn resolve_viewer_static_dir_with_override(
    raw: &str,
    env_override: Option<(&str, &str)>,
) -> Result<PathBuf, String> {
    if raw == DEFAULT_VIEWER_STATIC_DIR {
        if let Some((override_path, env_name)) = env_override {
            if let Some(dir) = resolve_viewer_static_dir_candidate(override_path) {
                return Ok(dir);
            }
            return Err(format!(
                "{env_name} is set but viewer static dir not found: `{override_path}`"
            ));
        }
    }

    if let Some(dir) = resolve_viewer_static_dir_candidate(raw) {
        return Ok(dir);
    }

    if raw == DEFAULT_VIEWER_STATIC_DIR {
        if let Some(dev_fallback) = viewer_dev_dist_candidates()
            .into_iter()
            .find(|candidate| candidate.is_dir())
        {
            return Ok(dev_fallback);
        }
    }

    Err(format!(
        "viewer static dir not found: `{raw}`; provide --viewer-static-dir <path> (expected trunk build output)"
    ))
}

fn viewer_dev_dist_candidates() -> Vec<PathBuf> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    vec![
        repo_root.join("oasis7_viewer").join("dist"),
        repo_root.join("agent_world_viewer").join("dist"),
    ]
}

fn resolve_viewer_static_dir_candidate(raw: &str) -> Option<PathBuf> {
    let user_path = PathBuf::from(raw);
    if user_path.is_dir() {
        return Some(user_path);
    }

    if user_path.is_relative() {
        if let Ok(current_exe) = env::current_exe() {
            if let Some(bin_dir) = current_exe.parent() {
                let sibling_candidate = bin_dir.join("..").join(&user_path);
                if sibling_candidate.is_dir() {
                    return Some(sibling_candidate);
                }
            }
        }
    }
    None
}

fn resolve_non_empty_env_override(
    primary: &'static str,
    legacy: &'static str,
) -> Option<(String, &'static str)> {
    for env_name in [primary, legacy] {
        if let Ok(value) = env::var(env_name) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some((trimmed.to_string(), env_name));
            }
        }
    }
    None
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
  --chain-storage-profile <name> world_chain_runtime storage profile (default: dev_local)\n\
  --chain-world-id <id>        world_chain_runtime world id (default: live-<scenario>)\n\
  --chain-node-role <role>     world_chain_runtime role (default: {DEFAULT_CHAIN_NODE_ROLE})\n\
  --chain-node-tick-ms <n>     world_chain_runtime worker poll/fallback interval ms (default: {DEFAULT_CHAIN_NODE_TICK_MS})\n\
  --chain-pos-slot-duration-ms <n>\n\
                               world_chain_runtime PoS slot duration ms (default: {DEFAULT_CHAIN_POS_SLOT_DURATION_MS})\n\
  --chain-pos-ticks-per-slot <n>\n\
                               world_chain_runtime PoS logical ticks per slot (default: {DEFAULT_CHAIN_POS_TICKS_PER_SLOT})\n\
  --chain-pos-proposal-tick-phase <n>\n\
                               world_chain_runtime proposal phase in slot tick window (default: {DEFAULT_CHAIN_POS_PROPOSAL_TICK_PHASE})\n\
  --chain-pos-adaptive-tick-scheduler\n\
                               enable world_chain_runtime adaptive tick scheduler\n\
  --chain-pos-no-adaptive-tick-scheduler\n\
                               disable world_chain_runtime adaptive scheduler (default)\n\
  --chain-pos-slot-clock-genesis-unix-ms <n>\n\
                               world_chain_runtime fixed slot clock genesis unix ms (default: auto)\n\
  --chain-pos-max-past-slot-lag <n>\n\
                               world_chain_runtime max accepted stale slot lag (default: {DEFAULT_CHAIN_POS_MAX_PAST_SLOT_LAG})\n\
  --chain-node-validator <v:s> world_chain_runtime validator (repeatable)\n\
  --with-llm                   enable llm mode\n\
  --agent-provider-mode <mode> agent provider: builtin_llm|openclaw_local_http\n\
  --openclaw-base-url <url>    OpenClaw local provider base URL (default: {DEFAULT_OPENCLAW_BASE_URL})\n\
  --openclaw-auth-token <tok>  OpenClaw bearer token\n\
  --openclaw-connect-timeout-ms <ms>\n\
                               OpenClaw connect timeout ms (default: {DEFAULT_OPENCLAW_CONNECT_TIMEOUT_MS})\n\
  --openclaw-agent-profile <id>\n\
                               OpenClaw agent profile (default: {DEFAULT_OPENCLAW_AGENT_PROFILE})\n\
  --openclaw-execution-mode <mode>\n\
                               OpenClaw execution mode: player_parity|headless_agent (default: headless_agent)\n\
  --no-open-browser            do not auto open browser\n\
  -h, --help                   show help\n\n\
Env:\n\
  OASIS7_WORLD_VIEWER_LIVE_BIN        explicit path of world_viewer_live binary (legacy: AGENT_WORLD_WORLD_VIEWER_LIVE_BIN)\n\
  OASIS7_WORLD_CHAIN_RUNTIME_BIN      explicit path of world_chain_runtime binary (legacy: AGENT_WORLD_WORLD_CHAIN_RUNTIME_BIN)\n\
  OASIS7_GAME_STATIC_DIR              override default viewer static dir when --viewer-static-dir is omitted (legacy: AGENT_WORLD_GAME_STATIC_DIR)"
    );
}

#[cfg(test)]
#[path = "world_game_launcher/world_game_launcher_tests.rs"]
mod world_game_launcher_tests;
