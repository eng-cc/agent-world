#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  oasis7-run.sh play [options]
  oasis7-run.sh smoke [options]

Options:
  --base-url <url>                OpenClaw local provider base url (default: http://127.0.0.1:5841)
  --agent-id <id>                 OpenClaw runtime agent id (default: agent_world_runtime)
  --agent-profile <profile>       OpenClaw agent profile (default: agent_world_p0_low_freq_npc)
  --scenario <name>               Gameplay scenario (default: llm_bootstrap)
  --timeout-ms <ms>               Smoke timeout budget (default: 15000)
  --connect-timeout-ms <ms>       Provider connect timeout (default: 15000)
  --samples <n>                   Smoke samples (default: 1)
  --ticks <n>                     Smoke ticks (default: 4)
  --bridge-log <path>             Bridge log path (default: .tmp/oasis7-bridge.log)
  --skip-agent-setup              Skip runtime agent bootstrap
  --reuse-bridge                  Reuse existing bridge at --base-url
  --no-open-browser               Pass through to world_game_launcher
  -h, --help                      Show help
USAGE
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

wait_for_http() {
  local url="$1"
  local attempts="${2:-40}"
  local sleep_s="${3:-0.5}"
  local i
  for ((i=0; i<attempts; i+=1)); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      return 0
    fi
    sleep "$sleep_s"
  done
  echo "timed out waiting for $url" >&2
  return 1
}

mode="${1:-}"
if [[ -z "$mode" || "$mode" == "-h" || "$mode" == "--help" ]]; then
  usage
  exit 0
fi
shift

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"
base_url="http://127.0.0.1:5841"
agent_id="agent_world_runtime"
agent_profile="agent_world_p0_low_freq_npc"
scenario="llm_bootstrap"
timeout_ms="15000"
connect_timeout_ms="15000"
samples="1"
ticks="4"
bridge_log="$repo_root/.tmp/oasis7-bridge.log"
skip_agent_setup="0"
reuse_bridge="0"
open_browser="1"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --base-url)
      base_url="$2"
      shift 2
      ;;
    --agent-id)
      agent_id="$2"
      shift 2
      ;;
    --agent-profile)
      agent_profile="$2"
      shift 2
      ;;
    --scenario)
      scenario="$2"
      shift 2
      ;;
    --timeout-ms)
      timeout_ms="$2"
      shift 2
      ;;
    --connect-timeout-ms)
      connect_timeout_ms="$2"
      shift 2
      ;;
    --samples)
      samples="$2"
      shift 2
      ;;
    --ticks)
      ticks="$2"
      shift 2
      ;;
    --bridge-log)
      bridge_log="$2"
      shift 2
      ;;
    --skip-agent-setup)
      skip_agent_setup="1"
      shift
      ;;
    --reuse-bridge)
      reuse_bridge="1"
      shift
      ;;
    --no-open-browser)
      open_browser="0"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

require_cmd openclaw
require_cmd curl
require_cmd cargo
mkdir -p "$(dirname "$bridge_log")"

cleanup_bridge_pid=""
cleanup() {
  if [[ -n "$cleanup_bridge_pid" ]]; then
    kill "$cleanup_bridge_pid" >/dev/null 2>&1 || true
    wait "$cleanup_bridge_pid" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

if [[ "$skip_agent_setup" != "1" ]]; then
  "$repo_root/scripts/setup-openclaw-agent-world-runtime.sh" "$agent_id"
fi

wait_for_http "http://127.0.0.1:18789/health" 20 0.5

if [[ "$reuse_bridge" != "1" ]]; then
  (
    cd "$repo_root"
    exec env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_openclaw_local_bridge -- --openclaw-agent "$agent_id"
  ) >"$bridge_log" 2>&1 &
  cleanup_bridge_pid="$!"
fi

wait_for_http "$base_url/v1/provider/health" 40 0.5

case "$mode" in
  play)
    cd "$repo_root"
    cmd=(env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_game_launcher --
      --scenario "$scenario"
      --with-llm
      --agent-provider-mode openclaw_local_http
      --openclaw-base-url "$base_url"
      --openclaw-connect-timeout-ms "$connect_timeout_ms"
      --openclaw-agent-profile "$agent_profile")
    if [[ "$open_browser" != "1" ]]; then
      cmd+=(--no-open-browser)
    fi
    printf 'Running: %q ' "${cmd[@]}"
    printf '\n'
    "${cmd[@]}"
    ;;
  smoke)
    cd "$repo_root"
    cmd=(bash scripts/openclaw-parity-p0.sh
      --openclaw-only
      --samples "$samples"
      --ticks "$ticks"
      --timeout-ms "$timeout_ms"
      --openclaw-base-url "$base_url"
      --openclaw-connect-timeout-ms "$connect_timeout_ms"
      --openclaw-agent-profile "$agent_profile")
    printf 'Running: %q ' "${cmd[@]}"
    printf '\n'
    "${cmd[@]}"
    ;;
  *)
    echo "unknown mode: $mode" >&2
    usage >&2
    exit 1
    ;;
esac
