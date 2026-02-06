#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

verbose_flags=()
if [[ "${CI_VERBOSE:-}" == "1" ]]; then
  verbose_flags=(--verbose)
fi

run() {
  echo "+ $*"
  "$@"
}

run env -u RUSTC_WRAPPER cargo test "${verbose_flags[@]}"
run env -u RUSTC_WRAPPER cargo test -p agent_world --features wasmtime "${verbose_flags[@]}"
run env -u RUSTC_WRAPPER cargo test -p agent_world --test viewer_live_integration --features viewer_live_integration "${verbose_flags[@]}"
run env -u RUSTC_WRAPPER cargo test -p agent_world --test viewer_offline_integration "${verbose_flags[@]}"
