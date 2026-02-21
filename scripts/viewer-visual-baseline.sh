#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

snapshot_dir="crates/agent_world_viewer/tests/snapshots"

required_snapshots=(
  "viewer_overview_live.png"
  "viewer_overview_manual_high_risk.png"
)

run() {
  echo "+ $*"
  "$@"
}

check_snapshot_baselines() {
  local missing=0
  local snapshot_path=""
  for snapshot in "${required_snapshots[@]}"; do
    snapshot_path="$snapshot_dir/$snapshot"
    if [[ ! -s "$snapshot_path" ]]; then
      echo "error: missing snapshot baseline: $snapshot_path" >&2
      missing=1
    fi
  done
  if [[ "$missing" -ne 0 ]]; then
    echo "error: viewer visual baseline check failed because required snapshot files are missing." >&2
    exit 1
  fi
}

check_snapshot_baselines
run env -u RUSTC_WRAPPER cargo test -p agent_world_viewer egui_kittest_snapshot_
