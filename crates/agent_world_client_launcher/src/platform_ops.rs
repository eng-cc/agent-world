#[cfg(not(target_arch = "wasm32"))]
use std::env;
use std::path::PathBuf;
#[cfg(not(target_arch = "wasm32"))]
use std::process::Command;

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn resolve_launcher_binary_path() -> PathBuf {
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

#[cfg(target_arch = "wasm32")]
pub(crate) fn resolve_launcher_binary_path() -> PathBuf {
    PathBuf::from(binary_name("world_game_launcher"))
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn resolve_chain_runtime_binary_path() -> PathBuf {
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

#[cfg(target_arch = "wasm32")]
pub(crate) fn resolve_chain_runtime_binary_path() -> PathBuf {
    PathBuf::from(binary_name("world_chain_runtime"))
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn resolve_static_dir_path() -> PathBuf {
    if let Ok(path) = env::var("AGENT_WORLD_GAME_STATIC_DIR") {
        return PathBuf::from(path);
    }

    if let Ok(current_exe) = env::current_exe() {
        if let Some(bin_dir) = current_exe.parent() {
            return bin_dir.join("..").join("web");
        }
    }

    PathBuf::from("web")
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn resolve_static_dir_path() -> PathBuf {
    PathBuf::from("web")
}

pub(crate) fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window().ok_or_else(|| "window is not available".to_string())?;
        window
            .open_with_url(url)
            .map_err(|err| format!("window.open failed: {err:?}"))?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        let status = Command::new("open")
            .arg(url)
            .status()
            .map_err(|err| format!("run open failed: {err}"))?;
        if status.success() {
            return Ok(());
        }
        return Err(format!("open exited with {status}"));
    }

    #[cfg(target_os = "windows")]
    {
        let status = Command::new("cmd")
            .arg("/C")
            .arg("start")
            .arg("")
            .arg(url)
            .status()
            .map_err(|err| format!("run cmd /C start failed: {err}"))?;
        if status.success() {
            return Ok(());
        }
        return Err(format!("cmd /C start exited with {status}"));
    }

    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "windows"),
        not(target_os = "macos")
    ))]
    {
        let status = Command::new("xdg-open")
            .arg(url)
            .status()
            .map_err(|err| format!("run xdg-open failed: {err}"))?;
        if status.success() {
            return Ok(());
        }
        Err(format!("xdg-open exited with {status}"))
    }
}

fn binary_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}
