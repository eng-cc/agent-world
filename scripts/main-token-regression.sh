#!/usr/bin/env bash
set -euo pipefail

TIER="${1:-required}"

usage() {
  cat <<'USAGE'
Usage: scripts/main-token-regression.sh [required|full]

Runs main-token and NodePoints-bridge focused regression suites.
USAGE
}

run() {
  echo "+ $*"
  "$@"
}

case "${TIER}" in
  required)
    run env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required runtime::tests::main_token:: -- --nocapture
    run env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required runtime::tests::reward_asset_settlement_action:: -- --nocapture
    ;;
  full)
    run env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required runtime::tests::main_token:: -- --nocapture
    run env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_required runtime::tests::reward_asset_settlement_action:: -- --nocapture
    run env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_full runtime::tests::main_token:: -- --nocapture
    run env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_full runtime::tests::reward_asset_settlement_action:: -- --nocapture
    run env -u RUSTC_WRAPPER cargo test -p agent_world --features test_tier_full runtime::tests::reward_asset:: -- --nocapture
    ;;
  *)
    usage
    exit 1
    ;;
esac
