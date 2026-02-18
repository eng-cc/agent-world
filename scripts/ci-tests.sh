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

run_agent_world_required_tier_tests() {
  run_cargo test -p agent_world --tests --features test_tier_required
}

run_agent_world_full_tier_tests() {
  run_cargo test -p agent_world --tests --features "test_tier_full,wasmtime,viewer_live_integration"
}

run_required_builtin_wasm_checks() {
  if [[ "${CI:-}" == "true" || "${AGENT_WORLD_FORCE_M1_WASM_CHECK:-}" == "1" ]]; then
    run ./scripts/sync-m1-builtin-wasm-artifacts.sh --check
  else
    echo "+ skip m1 wasm hash check (set CI=true or AGENT_WORLD_FORCE_M1_WASM_CHECK=1 to enable)"
  fi
}

run_required_gate_checks() {
  run env -u RUSTC_WRAPPER cargo fmt --all -- --check
  run_required_builtin_wasm_checks
}

echo "+ ci test tier: $tier"
run_required_gate_checks
if [[ "$tier" == "required" ]]; then
  run_agent_world_required_tier_tests
else
  run_agent_world_full_tier_tests
  run_cargo test -p agent_world --features wasmtime --lib --bins
  run_cargo test -p agent_world_net --features libp2p --lib
fi
