#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VIEWER_DIR="$ROOT_DIR/crates/agent_world_viewer"

if ! command -v trunk >/dev/null 2>&1; then
  echo "error: trunk is not installed" >&2
  echo "hint: cargo install trunk" >&2
  exit 1
fi

if ! rustup target list --installed | grep -qx "wasm32-unknown-unknown"; then
  echo "error: target wasm32-unknown-unknown is not installed" >&2
  echo "hint: rustup target add wasm32-unknown-unknown" >&2
  exit 1
fi

cd "$VIEWER_DIR"
exec trunk serve "$@"
