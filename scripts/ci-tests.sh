#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

tier="${1:-full}"

usage() {
  cat <<'USAGE'
Usage: ./scripts/ci-tests.sh [required|full|full-core|full-support]

  required      Run fast required checks for local commit and PR gate.
  full          Run required checks plus all extended feature/integration tests.
  full-core     Run doc/fmt plus the heaviest `agent_world` full-tier shard.
  full-support  Run the remaining full-tier support crates/viewer shard.

Default: full
USAGE
}

if [[ $# -gt 1 ]]; then
  usage
  exit 1
fi

case "$tier" in
  required|full|full-core|full-support) ;;
  *)
    usage
    exit 1
    ;;
esac

run() {
  echo "+ $*"
  "$@"
}

run_with_retries() {
  local max_attempts=$1
  shift
  local attempt=1
  local exit_code=0
  while (( attempt <= max_attempts )); do
    set +e
    "$@"
    exit_code=$?
    set -e
    if [[ "$exit_code" -eq 0 ]]; then
      return 0
    fi
    if (( attempt == max_attempts )); then
      return "$exit_code"
    fi
    echo "retry: attempt $attempt/$max_attempts failed (exit=$exit_code), retrying..." >&2
    attempt=$((attempt + 1))
    sleep 1
  done
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
  run_cargo test -p agent_world --tests --features "test_tier_full,wasmtime,viewer_live_integration" -- --skip live_server_accepts_client_and_emits_snapshot_and_event
  run_with_retries 3 \
    run_cargo test -p agent_world --features "test_tier_full,wasmtime,viewer_live_integration" \
      --test viewer_live_integration live_server_accepts_client_and_emits_snapshot_and_event -- --nocapture
}

run_agent_world_consensus_tests() {
  run_cargo test -p agent_world_consensus --lib
}

run_agent_world_distfs_tests() {
  run_cargo test -p agent_world_distfs --lib
}

run_agent_world_node_tests() {
  run_cargo test -p agent_world_node --lib
}

run_agent_world_net_tests() {
  run_cargo test -p agent_world_net --lib
}

run_agent_world_net_libp2p_tests() {
  run_cargo test -p agent_world_net --features libp2p --lib
}

run_agent_world_llm_baseline_fixture_smoke() {
  run ./scripts/llm-baseline-fixture-smoke.sh
}

run_agent_world_viewer_tests() {
  run_cargo test -p agent_world_viewer
}

run_agent_world_viewer_wasm_check() {
  run_cargo check -p agent_world_viewer --target wasm32-unknown-unknown
}

run_required_gate_checks() {
  run ./scripts/doc-governance-check.sh
  run env -u RUSTC_WRAPPER cargo fmt --all -- --check
}

run_full_core_tier_tests() {
  run_required_gate_checks
  run_agent_world_full_tier_tests
  run_cargo test -p agent_world --features wasmtime --lib --bins
}

run_full_support_tier_tests() {
  run_agent_world_consensus_tests
  run_agent_world_distfs_tests
  run_agent_world_node_tests
  run_agent_world_net_tests
  run_agent_world_net_libp2p_tests
  run_agent_world_llm_baseline_fixture_smoke
  run_agent_world_viewer_tests
  run_agent_world_viewer_wasm_check
}

echo "+ ci test tier: $tier"
case "$tier" in
  required)
    run_required_gate_checks
    run_agent_world_required_tier_tests
    run_agent_world_consensus_tests
    run_agent_world_distfs_tests
    run_agent_world_viewer_tests
    run_agent_world_viewer_wasm_check
    ;;
  full)
    run_full_core_tier_tests
    run_full_support_tier_tests
    ;;
  full-core)
    run_full_core_tier_tests
    ;;
  full-support)
    run_full_support_tier_tests
    ;;
  *)
    usage
    exit 1
    ;;
 esac
