use std::env;
use std::process;
use std::thread;

use agent_world::simulator::WorldScenario;
use agent_world::viewer::{
    ViewerLiveDecisionMode, ViewerLiveServer, ViewerLiveServerConfig, ViewerRuntimeLiveServer,
    ViewerRuntimeLiveServerConfig, ViewerWebBridge, ViewerWebBridgeConfig,
};

const DEFAULT_SCENARIO: &str = "llm_bootstrap";
const DEFAULT_BIND: &str = "127.0.0.1:5023";
const DEFAULT_WEB_BIND: &str = "127.0.0.1:5011";
const REMOVAL_HINT: &str =
    "embedded node flags were removed from world_viewer_live; use world_chain_runtime (normally launched by world_game_launcher)";

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliOptions {
    scenario: WorldScenario,
    bind_addr: String,
    web_bind_addr: Option<String>,
    llm_mode: bool,
    runtime_world: bool,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            scenario: WorldScenario::LlmBootstrap,
            bind_addr: DEFAULT_BIND.to_string(),
            web_bind_addr: Some(DEFAULT_WEB_BIND.to_string()),
            llm_mode: false,
            runtime_world: false,
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

    if let Err(err) = run_viewer(options) {
        eprintln!("world_viewer_live failed: {err}");
        process::exit(1);
    }
}

fn run_viewer(options: CliOptions) -> Result<(), String> {
    if let Some(web_bind_addr) = options.web_bind_addr.clone() {
        let upstream_addr = options.bind_addr.clone();
        thread::spawn(move || {
            let bridge = ViewerWebBridge::new(ViewerWebBridgeConfig::new(
                web_bind_addr.clone(),
                upstream_addr,
            ));
            if let Err(err) = bridge.run() {
                eprintln!("viewer web bridge failed on {web_bind_addr}: {err:?}");
            }
        });
    }

    if options.runtime_world {
        let config = ViewerRuntimeLiveServerConfig::new(options.scenario)
            .with_bind_addr(options.bind_addr)
            .with_decision_mode(if options.llm_mode {
                ViewerLiveDecisionMode::Llm
            } else {
                ViewerLiveDecisionMode::Script
            });
        let mut server = ViewerRuntimeLiveServer::new(config)
            .map_err(|err| format!("failed to create runtime viewer server: {err:?}"))?;
        server
            .run()
            .map_err(|err| format!("runtime viewer server exited with error: {err:?}"))
    } else {
        let config = ViewerLiveServerConfig::new(options.scenario)
            .with_bind_addr(options.bind_addr)
            .with_decision_mode(if options.llm_mode {
                ViewerLiveDecisionMode::Llm
            } else {
                ViewerLiveDecisionMode::Script
            });
        let mut server = ViewerLiveServer::new(config)
            .map_err(|err| format!("failed to create viewer server: {err:?}"))?;
        server
            .run()
            .map_err(|err| format!("viewer server exited with error: {err:?}"))
    }
}

fn parse_options<'a>(args: impl Iterator<Item = &'a str>) -> Result<CliOptions, String> {
    let mut options = CliOptions::default();
    let mut iter = args.peekable();
    let mut scenario_set = false;

    while let Some(arg) = iter.next() {
        if !arg.starts_with('-') {
            if scenario_set {
                return Err(format!("unexpected positional argument `{arg}`"));
            }
            options.scenario = parse_world_scenario(arg)?;
            scenario_set = true;
            continue;
        }

        match arg {
            "--bind" => {
                options.bind_addr = parse_required_value(&mut iter, "--bind")?;
            }
            "--web-bind" => {
                options.web_bind_addr = Some(parse_required_value(&mut iter, "--web-bind")?);
            }
            "--no-web-bind" => {
                options.web_bind_addr = None;
            }
            "--llm" => {
                options.llm_mode = true;
            }
            "--no-llm" => {
                options.llm_mode = false;
            }
            "--runtime-world" => {
                options.runtime_world = true;
            }
            "--no-runtime-world" => {
                options.runtime_world = false;
            }
            "--topology" | "--no-node" | "--viewer-no-consensus-gate" => {
                return Err(format!("`{arg}` is no longer supported: {REMOVAL_HINT}"));
            }
            _ if arg.starts_with("--node-")
                || arg.starts_with("--triad-")
                || arg.starts_with("--reward-runtime-") =>
            {
                return Err(format!("`{arg}` is no longer supported: {REMOVAL_HINT}"));
            }
            _ => {
                return Err(format!("unknown option: {arg}"));
            }
        }
    }

    parse_socket_addr(options.bind_addr.as_str(), "--bind")?;
    if let Some(web_bind_addr) = options.web_bind_addr.as_deref() {
        parse_socket_addr(web_bind_addr, "--web-bind")?;
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

fn parse_socket_addr(raw: &str, label: &str) -> Result<std::net::SocketAddr, String> {
    raw.parse::<std::net::SocketAddr>()
        .map_err(|_| format!("{label} must be in <host:port> format"))
}

fn parse_world_scenario(raw: &str) -> Result<WorldScenario, String> {
    let normalized = raw.trim();
    if normalized.is_empty() {
        return Err("scenario cannot be empty".to_string());
    }
    WorldScenario::parse(normalized).ok_or_else(|| {
        format!(
            "unknown scenario `{normalized}`; supported: {}",
            WorldScenario::variants().join(", ")
        )
    })
}

fn print_help() {
    println!(
        "Usage: world_viewer_live [scenario] [options]\n\n\
Starts pure viewer live server (no embedded chain/node runtime).\n\n\
Options:\n\
  [scenario]                world scenario (default: {DEFAULT_SCENARIO})\n\
  --bind <host:port>        viewer live server bind (default: {DEFAULT_BIND})\n\
  --web-bind <host:port>    websocket bridge bind (default: {DEFAULT_WEB_BIND})\n\
  --no-web-bind             disable websocket bridge\n\
  --runtime-world           run live server on runtime/world\n\
  --no-runtime-world        force simulator live server (default)\n\
  --llm                     enable llm mode\n\
  --no-llm                  disable llm mode (default)\n\
  -h, --help                show help\n\n\
Removed:\n\
  all --node-*, --topology, --triad-*, --reward-runtime-*, --no-node, --viewer-no-consensus-gate\n\
  -> use world_chain_runtime (usually managed by world_game_launcher)"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_options_defaults() {
        let options = parse_options(std::iter::empty()).expect("defaults");
        assert_eq!(options.scenario, WorldScenario::LlmBootstrap);
        assert_eq!(options.bind_addr, DEFAULT_BIND);
        assert_eq!(options.web_bind_addr.as_deref(), Some(DEFAULT_WEB_BIND));
        assert!(!options.llm_mode);
        assert!(!options.runtime_world);
    }

    #[test]
    fn parse_options_reads_custom_values() {
        let options = parse_options(
            [
                "asteroid_fragment",
                "--bind",
                "127.0.0.1:6200",
                "--web-bind",
                "127.0.0.1:6300",
                "--llm",
            ]
            .into_iter(),
        )
        .expect("custom values");
        assert_eq!(options.scenario, WorldScenario::AsteroidFragmentBootstrap);
        assert_eq!(options.bind_addr, "127.0.0.1:6200");
        assert_eq!(options.web_bind_addr.as_deref(), Some("127.0.0.1:6300"));
        assert!(options.llm_mode);
        assert!(!options.runtime_world);
    }

    #[test]
    fn parse_options_supports_no_web_bind() {
        let options = parse_options(["--no-web-bind"].into_iter()).expect("no web bind");
        assert_eq!(options.web_bind_addr, None);
    }

    #[test]
    fn parse_options_rejects_invalid_bind() {
        let err = parse_options(["--bind", "bad-bind"].into_iter()).expect_err("invalid bind");
        assert!(err.contains("--bind"));
    }

    #[test]
    fn parse_options_rejects_legacy_node_flags() {
        let err = parse_options(["--no-node"].into_iter()).expect_err("legacy flag should fail");
        assert!(err.contains("no longer supported"));
        assert!(err.contains("world_chain_runtime"));
    }

    #[test]
    fn parse_options_rejects_legacy_node_prefix_flags() {
        let err = parse_options(["--node-id", "n1"].into_iter()).expect_err("node-id should fail");
        assert!(err.contains("no longer supported"));
        assert!(err.contains("world_chain_runtime"));
    }

    #[test]
    fn parse_options_rejects_unknown_option() {
        let err = parse_options(["--wat"].into_iter()).expect_err("unknown option");
        assert!(err.contains("unknown option"));
    }

    #[test]
    fn parse_options_rejects_unknown_scenario() {
        let err = parse_options(["wat"].into_iter()).expect_err("unknown scenario");
        assert!(err.contains("unknown scenario"));
    }

    #[test]
    fn parse_options_supports_runtime_world_flag() {
        let options = parse_options(["--runtime-world"].into_iter()).expect("runtime world flag");
        assert!(options.runtime_world);
    }

    #[test]
    fn parse_options_supports_runtime_world_with_llm() {
        let options = parse_options(["--runtime-world", "--llm"].into_iter())
            .expect("runtime world + llm should be allowed");
        assert!(options.runtime_world);
        assert!(options.llm_mode);
    }
}
