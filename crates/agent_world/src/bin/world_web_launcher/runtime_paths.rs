use std::env;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_CONSOLE_STATIC_DIR: &str = "web-launcher";

pub(super) fn resolve_world_game_launcher_binary() -> PathBuf {
    if let Ok(path) = env::var("AGENT_WORLD_GAME_LAUNCHER_BIN") {
        return PathBuf::from(path);
    }

    if let Ok(current_exe) = env::current_exe() {
        if let Some(bin_dir) = current_exe.parent() {
            return bin_dir.join(binary_name("world_game_launcher"));
        }
    }

    PathBuf::from(binary_name("world_game_launcher"))
}

pub(super) fn resolve_world_chain_runtime_binary() -> PathBuf {
    if let Ok(path) = env::var("AGENT_WORLD_WORLD_CHAIN_RUNTIME_BIN") {
        return PathBuf::from(path);
    }

    if let Ok(current_exe) = env::current_exe() {
        if let Some(bin_dir) = current_exe.parent() {
            return bin_dir.join(binary_name("world_chain_runtime"));
        }
    }

    PathBuf::from(binary_name("world_chain_runtime"))
}

pub(super) fn resolve_static_dir_path(default_viewer_static_dir: &str) -> PathBuf {
    if let Ok(path) = env::var("AGENT_WORLD_GAME_STATIC_DIR") {
        return PathBuf::from(path);
    }

    if let Ok(current_exe) = env::current_exe() {
        if let Some(bin_dir) = current_exe.parent() {
            return bin_dir.join("..").join("web");
        }
    }

    PathBuf::from(default_viewer_static_dir)
}

pub(super) fn resolve_console_static_dir_path() -> PathBuf {
    if let Ok(path) = env::var("AGENT_WORLD_WEB_LAUNCHER_STATIC_DIR") {
        return PathBuf::from(path);
    }

    if let Ok(current_exe) = env::current_exe() {
        if let Some(bin_dir) = current_exe.parent() {
            return bin_dir.join("..").join(DEFAULT_CONSOLE_STATIC_DIR);
        }
    }

    PathBuf::from(DEFAULT_CONSOLE_STATIC_DIR)
}

pub(super) fn normalize_bind_host_for_local_access(host: &str) -> String {
    let host = host.trim();
    if host == "0.0.0.0" || host == "::" || host == "[::]" {
        "127.0.0.1".to_string()
    } else {
        host.to_string()
    }
}

pub(super) fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn binary_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}
