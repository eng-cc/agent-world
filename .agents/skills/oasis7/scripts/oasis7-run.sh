#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  oasis7-run.sh play [options]
  oasis7-run.sh smoke [options]
  oasis7-run.sh doctor [options]

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
  --json                          Emit machine-readable JSON for doctor mode
  -h, --help                      Show help
USAGE
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

http_get() {
  local url="$1"
  curl -fsS "$url"
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

encode_json_string() {
  python - <<'PY' "$1"
import json, sys
print(json.dumps(sys.argv[1]))
PY
}

encode_b64() {
  python - <<'PY' "$1"
import base64, sys
print(base64.b64encode(sys.argv[1].encode()).decode())
PY
}

print_doctor_status() {
  local level="$1"
  local label="$2"
  local detail="$3"
  if [[ "$json_output" != "1" ]]; then
    printf '[%s] %s: %s\n' "$level" "$label" "$detail"
  fi
  if [[ -n "$doctor_records_file" ]]; then
    printf '%s\t%s\t%s\n' "$level" "$label" "$(encode_b64 "$detail")" >>"$doctor_records_file"
  fi
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
json_output="0"
doctor_records_file=""

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
    --json)
      json_output="1"
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
if [[ "$mode" != "doctor" ]]; then
  require_cmd cargo
fi
mkdir -p "$(dirname "$bridge_log")"

cleanup_bridge_pid=""
cleanup() {
  if [[ -n "$cleanup_bridge_pid" ]]; then
    kill "$cleanup_bridge_pid" >/dev/null 2>&1 || true
    wait "$cleanup_bridge_pid" >/dev/null 2>&1 || true
  fi
  if [[ -n "$doctor_records_file" && -f "$doctor_records_file" ]]; then
    rm -f "$doctor_records_file"
  fi
}
trap cleanup EXIT

emit_doctor_json() {
  local failures="$1"
  python - <<'PY' "$doctor_records_file" "$failures" "$base_url" "$agent_id" "$agent_profile" "$scenario"
import base64, json, sys
records_path, failures, base_url, agent_id, agent_profile, scenario = sys.argv[1:]
checks = []
with open(records_path, "r", encoding="utf-8") as handle:
    for line in handle:
        level, label, detail_b64 = line.rstrip("\n").split("\t", 2)
        detail = base64.b64decode(detail_b64.encode()).decode()
        checks.append({"level": level, "label": label, "detail": detail})
print(json.dumps({
    "ok": int(failures) == 0,
    "failures": int(failures),
    "base_url": base_url,
    "agent_id": agent_id,
    "agent_profile": agent_profile,
    "scenario": scenario,
    "checks": checks,
}, ensure_ascii=False))
PY
}

run_doctor() {
  local failures=0
  local gateway_json=""
  local bridge_health_json=""
  local provider_info_json=""
  local agents_json=""
  doctor_records_file="$(mktemp)"

  print_doctor_status INFO config "base_url=$base_url agent_id=$agent_id agent_profile=$agent_profile scenario=$scenario"

  if command -v openclaw >/dev/null 2>&1; then
    print_doctor_status OK command "openclaw=$(command -v openclaw)"
  else
    print_doctor_status FAIL command "openclaw not found"
    failures=$((failures + 1))
  fi

  if command -v cargo >/dev/null 2>&1; then
    print_doctor_status OK command "cargo=$(command -v cargo)"
  else
    print_doctor_status FAIL command "cargo not found"
    failures=$((failures + 1))
  fi

  if gateway_json="$(http_get 'http://127.0.0.1:18789/health' 2>/dev/null)"; then
    print_doctor_status OK gateway "$gateway_json"
  else
    print_doctor_status FAIL gateway "cannot reach http://127.0.0.1:18789/health"
    failures=$((failures + 1))
  fi

  if agents_json="$(openclaw agents list --json 2>/dev/null)"; then
    if AGENTS_JSON="$agents_json" AGENT_ID="$agent_id" python - <<'PY' >/dev/null
import json, os, sys
agent_id = os.environ['AGENT_ID']
items = json.loads(os.environ['AGENTS_JSON'])
for item in items:
    if item.get('id') == agent_id:
        sys.exit(0)
sys.exit(1)
PY
    then
      local agent_summary
      agent_summary="$(AGENTS_JSON="$agents_json" AGENT_ID="$agent_id" python - <<'PY'
import json, os
agent_id = os.environ['AGENT_ID']
items = json.loads(os.environ['AGENTS_JSON'])
for item in items:
    if item.get('id') == agent_id:
        print(f"workspace={item.get('workspace','')} model={item.get('model','')}")
        break
PY
)"
      print_doctor_status OK runtime-agent "$agent_summary"
    else
      print_doctor_status FAIL runtime-agent "OpenClaw agent '$agent_id' not found; run scripts/setup-openclaw-agent-world-runtime.sh $agent_id"
      failures=$((failures + 1))
    fi
  else
    print_doctor_status FAIL runtime-agent "failed to query 'openclaw agents list --json'"
    failures=$((failures + 1))
  fi

  if bridge_health_json="$(http_get "$base_url/v1/provider/health" 2>/dev/null)"; then
    print_doctor_status OK bridge-health "$bridge_health_json"
  else
    print_doctor_status FAIL bridge-health "cannot reach $base_url/v1/provider/health"
    failures=$((failures + 1))
  fi

  if provider_info_json="$(http_get "$base_url/v1/provider/info" 2>/dev/null)"; then
    local provider_summary
    provider_summary="$(PROVIDER_INFO_JSON="$provider_info_json" python - <<'PY'
import json, os
value = json.loads(os.environ['PROVIDER_INFO_JSON'])
provider_id = value.get('provider_id', '')
provider_version = value.get('provider_version', '')
protocol_version = value.get('protocol_version', '')
print(f"provider_id={provider_id} provider_version={provider_version} protocol_version={protocol_version}")
PY
)"
    print_doctor_status OK provider-info "$provider_summary"
  else
    print_doctor_status FAIL provider-info "cannot reach $base_url/v1/provider/info"
    failures=$((failures + 1))
  fi

  if [[ -f "$bridge_log" ]]; then
    print_doctor_status INFO bridge-log "$bridge_log"
  else
    print_doctor_status INFO bridge-log "not created yet ($bridge_log)"
  fi

  if [[ "$failures" -eq 0 ]]; then
    print_doctor_status OK summary "doctor checks passed"
    if [[ "$json_output" == "1" ]]; then
      emit_doctor_json "$failures"
    fi
    return 0
  fi

  print_doctor_status FAIL summary "$failures check(s) failed"
  if [[ "$json_output" == "1" ]]; then
    emit_doctor_json "$failures"
  fi
  return 1
}

if [[ "$mode" == "doctor" ]]; then
  run_doctor
  exit $?
fi

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
