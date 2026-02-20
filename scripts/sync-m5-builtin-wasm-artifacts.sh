#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

"$ROOT_DIR/scripts/sync-m1-builtin-wasm-artifacts.sh" \
  --module-ids-path "$ROOT_DIR/crates/agent_world/src/runtime/world/artifacts/m5_builtin_module_ids.txt" \
  --hash-path "$ROOT_DIR/crates/agent_world/src/runtime/world/artifacts/m5_builtin_modules.sha256" \
  "$@"
