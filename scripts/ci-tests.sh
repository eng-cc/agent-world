#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

tier="${1:-full}"

usage() {
  cat <<'USAGE'
Usage: ./scripts/ci-tests.sh [required|full]

  required  Run fast required checks for local commit and PR gate.
  full      Run required checks plus extended feature/integration tests.

Default: full
USAGE
}

if [[ $# -gt 1 ]]; then
  usage
  exit 1
fi

case "$tier" in
  required|full) ;;
  *)
    usage
    exit 1
    ;;
esac

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

run_required() {
  run env -u RUSTC_WRAPPER cargo fmt --all -- --check
  run ./scripts/check-include-warning-baseline.sh
  run ./scripts/sync-m1-builtin-wasm-artifacts.sh --check
  run ./scripts/sync-m4-builtin-wasm-artifacts.sh --check
  run_cargo test
}

run_full_only() {
  run_cargo test -p agent_world_net --features libp2p --lib
  run_cargo test -p agent_world --features wasmtime
  run_cargo test -p agent_world --test viewer_live_integration --features viewer_live_integration
  run_cargo test -p agent_world --test viewer_offline_integration
}

echo "+ ci test tier: $tier"
run_required
if [[ "$tier" == "full" ]]; then
  run_full_only
fi
