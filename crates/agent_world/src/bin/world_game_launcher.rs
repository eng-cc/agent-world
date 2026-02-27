use std::env;
use std::ffi::OsStr;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{self, Child, Command};
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_SCENARIO: &str = "llm_bootstrap";
const DEFAULT_LIVE_BIND: &str = "127.0.0.1:5023";
const DEFAULT_WEB_BIND: &str = "127.0.0.1:5011";
const DEFAULT_VIEWER_HOST: &str = "127.0.0.1";
const DEFAULT_VIEWER_PORT: u16 = 4173;
const DEFAULT_VIEWER_WEB_SCRIPT: &str = "scripts/run-viewer-web.sh";

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliOptions {
    scenario: String,
    live_bind: String,
    web_bind: String,
    viewer_host: String,
    viewer_port: u16,
    viewer_web_script: String,
    with_llm: bool,
    open_browser: bool,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            scenario: DEFAULT_SCENARIO.to_string(),
            live_bind: DEFAULT_LIVE_BIND.to_string(),
            web_bind: DEFAULT_WEB_BIND.to_string(),
            viewer_host: DEFAULT_VIEWER_HOST.to_string(),
            viewer_port: DEFAULT_VIEWER_PORT,
            viewer_web_script: DEFAULT_VIEWER_WEB_SCRIPT.to_string(),
            with_llm: false,
            open_browser: true,
        }
    }
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
    let viewer_web_script = resolve_viewer_web_script(options.viewer_web_script.as_str())?;

    let mut world_child = spawn_world_viewer_live(&world_viewer_live_bin, options)?;
    let mut web_child = spawn_viewer_web(&viewer_web_script, options)?;

    let ready_result = wait_until_ready(options);
    if let Err(err) = ready_result {
        terminate_child(&mut web_child);
        terminate_child(&mut world_child);
        return Err(err);
    }

    let game_url = build_game_url(options);
    println!("Launcher stack is ready.");
    println!("- URL: {game_url}");
    println!("- world_viewer_live pid: {}", world_child.id());
    println!("- web viewer pid: {}", web_child.id());
    println!("Press Ctrl+C to stop.");

    if options.open_browser {
        if let Err(err) = open_browser(&game_url) {
            eprintln!("warning: failed to open browser automatically: {err}");
            eprintln!("open this URL manually: {game_url}");
        }
    }

    monitor_children(&mut world_child, &mut web_child)
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
        .arg("--viewer-no-consensus-gate");
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

fn spawn_viewer_web(path: &Path, options: &CliOptions) -> Result<Child, String> {
    let mut command = Command::new(path);
    command
        .arg("--address")
        .arg(options.viewer_host.as_str())
        .arg("--port")
        .arg(options.viewer_port.to_string());
    command.spawn().map_err(|err| {
        format!(
            "failed to start viewer web script `{}`: {err}",
            path.display()
        )
    })
}

fn wait_until_ready(options: &CliOptions) -> Result<(), String> {
    let (viewer_host, viewer_port) = normalize_http_target(
        options.viewer_host.as_str(),
        options.viewer_port,
        "viewer host/port",
    )?;
    wait_for_http_ready(viewer_host.as_str(), viewer_port, Duration::from_secs(180)).map_err(
        |err| format!("viewer HTTP did not become ready at {viewer_host}:{viewer_port}: {err}"),
    )?;

    let (bridge_host, bridge_port) = parse_host_port(options.web_bind.as_str(), "--web-bind")?;
    wait_for_tcp_ready(bridge_host.as_str(), bridge_port, Duration::from_secs(60)).map_err(|err| {
        format!("web bridge did not become ready at {bridge_host}:{bridge_port}: {err}")
    })
}

fn monitor_children(world_child: &mut Child, web_child: &mut Child) -> Result<(), String> {
    loop {
        if let Some(status) = world_child
            .try_wait()
            .map_err(|err| format!("failed to query world_viewer_live status: {err}"))?
        {
            terminate_child(web_child);
            return Err(format!("world_viewer_live exited unexpectedly: {status}"));
        }

        if let Some(status) = web_child
            .try_wait()
            .map_err(|err| format!("failed to query web viewer process status: {err}"))?
        {
            terminate_child(world_child);
            return Err(format!("viewer web process exited unexpectedly: {status}"));
        }

        thread::sleep(Duration::from_millis(400));
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
            "--viewer-web-script" => {
                options.viewer_web_script = parse_required_value(&mut iter, "--viewer-web-script")?;
            }
            "--with-llm" => {
                options.with_llm = true;
            }
            "--no-open-browser" => {
                options.open_browser = false;
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

fn resolve_viewer_web_script(raw: &str) -> Result<PathBuf, String> {
    let user_path = PathBuf::from(raw);
    if user_path.is_file() {
        return Ok(user_path);
    }

    let manifest_fallback = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(raw);
    if manifest_fallback.is_file() {
        return Ok(manifest_fallback);
    }

    Err(format!(
        "viewer web script not found: `{raw}` (also checked workspace fallback)"
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
- start world_viewer_live\n\
- start web viewer (run-viewer-web.sh)\n\
- print URL and optionally open browser\n\n\
Options:\n\
  --scenario <name>            world_viewer_live scenario (default: {DEFAULT_SCENARIO})\n\
  --live-bind <host:port>      world_viewer_live bind (default: {DEFAULT_LIVE_BIND})\n\
  --web-bind <host:port>       world_viewer_live web bridge bind (default: {DEFAULT_WEB_BIND})\n\
  --viewer-host <host>         web viewer host (default: {DEFAULT_VIEWER_HOST})\n\
  --viewer-port <port>         web viewer port (default: {DEFAULT_VIEWER_PORT})\n\
  --viewer-web-script <path>   viewer web startup script (default: {DEFAULT_VIEWER_WEB_SCRIPT})\n\
  --with-llm                   enable llm mode\n\
  --no-open-browser            do not auto open browser\n\
  -h, --help                   show help\n\n\
Env:\n\
  AGENT_WORLD_WORLD_VIEWER_LIVE_BIN   explicit path of world_viewer_live binary"
    );
}

#[cfg(test)]
mod world_game_launcher_tests {
    use super::{
        build_game_url, parse_host_port, parse_options, CliOptions, DEFAULT_LIVE_BIND,
        DEFAULT_SCENARIO,
    };

    #[test]
    fn parse_options_defaults() {
        let options = parse_options(std::iter::empty()).expect("parse should succeed");
        assert_eq!(options.scenario, DEFAULT_SCENARIO);
        assert_eq!(options.live_bind, DEFAULT_LIVE_BIND);
        assert!(!options.with_llm);
        assert!(options.open_browser);
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
                "--viewer-web-script",
                "custom.sh",
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
        assert_eq!(options.viewer_web_script, "custom.sh");
        assert!(options.with_llm);
        assert!(!options.open_browser);
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
}
