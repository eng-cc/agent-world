#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

run() {
  echo "+ $*"
  "$@"
}

run_cargo() {
  if [[ "${CI_VERBOSE:-}" == "1" ]]; then
    run env -u RUSTC_WRAPPER cargo "$@" --verbose
  else
    run env -u RUSTC_WRAPPER cargo "$@"
  fi
}

run env -u RUSTC_WRAPPER cargo fmt --all -- --check
run ./scripts/check-include-warning-baseline.sh
run_cargo test
run_cargo test -p agent_world_net --features libp2p --lib
run_cargo test -p agent_world --features wasmtime
run_cargo test -p agent_world --test viewer_live_integration --features viewer_live_integration
run_cargo test -p agent_world --test viewer_offline_integration
