use std::path::Path;

fn write_bytes_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    super::write_bytes_atomic(path, bytes)
}

#[path = "../world_viewer_live/execution_bridge.rs"]
mod real_execution_bridge;
