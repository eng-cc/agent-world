#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"
source "$ROOT_DIR/scripts/agent-browser-lib.sh"

usage() {
  cat <<'USAGE'
Usage: ./scripts/run-game-test-ab.sh [options] [run-game-test options...]

Run a stable A/B playability loop and emit quantitative metrics:
- A phase: play -> observe -> pause
- B phase: step-chain control probes (no seek)
- Outputs TTFC / effective control hit-rate / max no-progress window

Important guardrail:
- This script is for automated regression probing only.
- It does NOT replace manual long-play sessions or real-player card filling.

Options:
  --url <url>               Use an existing viewer URL; skip stack bootstrap
  --out-dir <path>          Artifact root (default: output/playwright/playability)
  --startup-timeout <secs>  Wait timeout for stack URL (default: 240)
  --headed                  Open browser in headed mode (default)
  --headless                Open browser in headless mode
  -h, --help                Show this help

If --url is omitted, the script starts:
  ./scripts/run-game-test.sh [remaining args...]

Artifacts:
  <out-dir>/<run-id>/agent-browser.log
  <out-dir>/<run-id>/playthrough.webm
  <out-dir>/<run-id>/step0-home.png
  <out-dir>/<run-id>/step1-phase-a.png
  <out-dir>/<run-id>/step2-phase-b.png
  <out-dir>/<run-id>/step3-final.png
  <out-dir>/<run-id>/ab_metrics.json
  <out-dir>/<run-id>/ab_metrics.md
  <out-dir>/<run-id>/card_quant_metrics.md
USAGE
}

sleep_ms() {
  python3 - "$1" <<'PY'
import sys, time
ms = int(sys.argv[1])
time.sleep(ms / 1000.0)
PY
}

GAME_URL=""
OUT_ROOT="output/playwright/playability"
STARTUP_TIMEOUT_SECS=240
HEADED=1
STACK_ARGS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --url)
      GAME_URL="${2:-}"
      shift 2
      ;;
    --out-dir)
      OUT_ROOT="${2:-}"
      shift 2
      ;;
    --startup-timeout)
      STARTUP_TIMEOUT_SECS="${2:-}"
      shift 2
      ;;
    --headed)
      HEADED=1
      shift
      ;;
    --headless)
      HEADED=0
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      STACK_ARGS+=("$1")
      shift
      ;;
  esac
done

[[ -n "$OUT_ROOT" ]] || { echo "error: --out-dir cannot be empty" >&2; exit 2; }
[[ "$STARTUP_TIMEOUT_SECS" =~ ^[0-9]+$ ]] && [[ "$STARTUP_TIMEOUT_SECS" -gt 0 ]] || { echo "error: --startup-timeout must be a positive integer" >&2; exit 2; }

require_cmd python3
require_cmd rg
ab_require

RUN_ID="$(date +%Y%m%d-%H%M%S)"
OUT_DIR="$OUT_ROOT/$RUN_ID"
mkdir -p "$OUT_DIR"

AB_LOG="$OUT_DIR/agent-browser.log"
RUN_GAME_TEST_LOG="$OUT_DIR/run-game-test.log"
CONSOLE_WARNING_LOG="$OUT_DIR/console_warning_dump.log"
CONSOLE_ALL_LOG="$OUT_DIR/console_all_messages.log"
AB_METRICS_JSON="$OUT_DIR/ab_metrics.json"
AB_METRICS_MD="$OUT_DIR/ab_metrics.md"
CARD_METRICS_MD="$OUT_DIR/card_quant_metrics.md"
SESSION="playability-ab-$RUN_ID"

STACK_PID=""
STACK_OUTPUT_DIR=""

ab_log_note() {
  printf '### [%s] %s\n' "$1" "$(date '+%H:%M:%S')" | tee -a "$AB_LOG" >/dev/null
}

ab_state() {
  ab_eval "$SESSION" 'JSON.stringify(window.__AW_TEST__?.getState?.() ?? null)'
}

state_tick() { json_get "$1" tick; }
state_event_seq() { json_get "$1" eventSeq; }
state_connection() { json_get "$1" connectionStatus; }
state_last_feedback_json() { json_get "$1" lastControlFeedback; }

wait_for_api() {
  local timeout_ms=${1:-20000}
  local deadline=$((SECONDS * 1000 + timeout_ms))
  while (( SECONDS * 1000 < deadline )); do
    if [[ "$(ab_eval "$SESSION" 'typeof window.__AW_TEST__ === "object" ? "ready" : "missing"')" == "ready" ]]; then
      return 0
    fi
    sleep_ms 200
  done
  return 1
}

wait_for_connected() {
  local timeout_ms=${1:-20000}
  local deadline=$((SECONDS * 1000 + timeout_ms))
  local state='null'
  while (( SECONDS * 1000 < deadline )); do
    state=$(ab_state)
    if [[ "$(state_connection "$state")" == "connected" ]]; then
      printf '%s\n' "$state"
      return 0
    fi
    sleep_ms 250
  done
  printf '%s\n' "$state"
  return 1
}

send_control_probe() {
  local name=$1
  local action=$2
  local payload_json=$3
  local expect_progress=$4
  local timeout_ms=$5
  local before feedback after last_feedback_json accepted reason effect before_tick after_tick before_event after_event progressed first_progress_ms feedback_stage feedback_reason feedback_hint fail_category

  before=$(ab_state)
  before_tick=$(state_tick "$before"); before_tick=${before_tick:-0}
  before_event=$(state_event_seq "$before"); before_event=${before_event:-0}
  feedback=$(ab_eval "$SESSION" "JSON.stringify((() => { try { return window.__AW_TEST__?.sendControl?.($(json_quote "$action"), ${payload_json}) ?? null; } catch (err) { return { accepted: false, reason: String(err), effect: 'exception on sendControl' }; } })())")
  accepted=$(json_get "$feedback" accepted)
  reason=$(json_get "$feedback" reason)
  effect=$(json_get "$feedback" effect)
  after="$before"
  progressed=false
  first_progress_ms=""
  local started_ms
  started_ms=$(python3 - <<'PY'
import time
print(int(time.time() * 1000))
PY
)
  local deadline_ms=$((started_ms + timeout_ms))

  while :; do
    local now_ms
    now_ms=$(python3 - <<'PY'
import time
print(int(time.time() * 1000))
PY
)
    if (( now_ms >= deadline_ms )); then
      break
    fi
    sleep_ms 250
    after=$(ab_state)
    after_tick=$(state_tick "$after"); after_tick=${after_tick:-0}
    after_event=$(state_event_seq "$after"); after_event=${after_event:-0}
    last_feedback_json=$(state_last_feedback_json "$after")
    feedback_stage=$(json_get "$last_feedback_json" stage)
    feedback_reason=$(json_get "$last_feedback_json" reason)
    feedback_hint=$(json_get "$last_feedback_json" hint)
    if (( ${after_tick%%.*} > ${before_tick%%.*} || ${after_event%%.*} > ${before_event%%.*} )); then
      progressed=true
      first_progress_ms=$((now_ms - started_ms))
      break
    fi
    if [[ "$expect_progress" == "false" ]] && (( now_ms - started_ms >= 1000 )); then
      break
    fi
    if [[ "$feedback_stage" == "completed_no_progress" || "$feedback_stage" == "blocked" ]]; then
      break
    fi
  done

  after_tick=$(state_tick "$after"); after_tick=${after_tick:-0}
  after_event=$(state_event_seq "$after"); after_event=${after_event:-0}
  last_feedback_json=$(state_last_feedback_json "$after")
  feedback_stage=$(json_get "$last_feedback_json" stage)
  feedback_reason=$(json_get "$last_feedback_json" reason)
  feedback_hint=$(json_get "$last_feedback_json" hint)

  if [[ "$accepted" != "true" ]]; then
    fail_category="rejected"
  elif [[ "$progressed" == "true" ]]; then
    fail_category="progressed"
  elif [[ "$feedback_stage" == "completed_no_progress" ]]; then
    fail_category="completed_no_progress"
  elif [[ "$feedback_stage" == "blocked" ]]; then
    fail_category="blocked_after_accept"
  elif [[ "$(state_connection "$after")" != "connected" ]]; then
    fail_category="disconnected"
  else
    fail_category="timeout_no_delta"
  fi

  python3 - <<'PY' \
    "$name" "$action" "$payload_json" "$expect_progress" "$accepted" "$reason" "$effect" \
    "$before_tick" "$after_tick" "$before_event" "$after_event" "$progressed" "$first_progress_ms" \
    "$feedback_stage" "$feedback_reason" "$feedback_hint" "$fail_category"
import json, sys
name, action, payload_json, expect_progress, accepted, reason, effect, before_tick, after_tick, before_event, after_event, progressed, first_progress_ms, feedback_stage, feedback_reason, feedback_hint, fail_category = sys.argv[1:18]
try:
    payload = json.loads(payload_json)
except Exception:
    payload = payload_json
result = {
    "name": name,
    "action": action,
    "payload": payload,
    "expectProgress": expect_progress == "true",
    "accepted": accepted == "true",
    "reason": reason or None,
    "effect": effect or None,
    "beforeTick": int(float(before_tick or 0)),
    "afterTick": int(float(after_tick or 0)),
    "beforeEventSeq": int(float(before_event or 0)),
    "afterEventSeq": int(float(after_event or 0)),
    "progressed": progressed == "true",
    "firstProgressMs": int(first_progress_ms) if first_progress_ms else None,
    "feedbackStage": feedback_stage or None,
    "feedbackReason": feedback_reason or None,
    "feedbackHint": feedback_hint or None,
    "failCategory": fail_category,
}
print(json.dumps(result, ensure_ascii=False))
PY
}

observe_no_progress_window() {
  local duration_ms=$1
  local started_ms
  started_ms=$(python3 - <<'PY'
import time
print(int(time.time() * 1000))
PY
)
  local end_ms=$((started_ms + duration_ms))
  local state last_tick current_tick stagnation_start max_window now_ms final_event
  state=$(ab_state)
  last_tick=$(state_tick "$state"); last_tick=${last_tick:-0}
  stagnation_start=$started_ms
  max_window=0
  while :; do
    now_ms=$(python3 - <<'PY'
import time
print(int(time.time() * 1000))
PY
)
    (( now_ms >= end_ms )) && break
    sleep_ms 250
    state=$(ab_state)
    current_tick=$(state_tick "$state"); current_tick=${current_tick:-0}
    if [[ "$(state_connection "$state")" == "connected" ]] && (( ${current_tick%%.*} == ${last_tick%%.*} )); then
      local current_window=$((now_ms - stagnation_start))
      (( current_window > max_window )) && max_window=$current_window
    else
      last_tick=$current_tick
      stagnation_start=$now_ms
    fi
  done
  python3 - <<'PY' "$max_window" "$last_tick" "$(state_event_seq "$state")"
import json, sys
print(json.dumps({
    "maxNoProgressWindowMs": int(float(sys.argv[1] or 0)),
    "finalObservedTick": int(float(sys.argv[2] or 0)),
    "finalObservedEventSeq": int(float(sys.argv[3] or 0)),
}, ensure_ascii=False))
PY
}

stop_stack() {
  if [[ -n "$STACK_PID" ]] && kill -0 "$STACK_PID" >/dev/null 2>&1; then
    kill "$STACK_PID" >/dev/null 2>&1 || true
    wait "$STACK_PID" >/dev/null 2>&1 || true
  fi
  STACK_PID=""
}

cleanup() {
  local exit_code=$?
  trap - EXIT INT TERM
  ab_cmd "$SESSION" close >/dev/null 2>&1 || true
  stop_stack
  exit "$exit_code"
}
trap cleanup EXIT INT TERM

if [[ -z "$GAME_URL" ]]; then
  {
    echo "### [bootstrap_stack] $(date '+%H:%M:%S')"
    echo "./scripts/run-game-test.sh ${STACK_ARGS[*]}"
    echo
  } | tee -a "$AB_LOG" >/dev/null

  ./scripts/run-game-test.sh "${STACK_ARGS[@]}" >"$RUN_GAME_TEST_LOG" 2>&1 &
  STACK_PID=$!

  for ((i = 0; i < STARTUP_TIMEOUT_SECS; i++)); do
    if ! kill -0 "$STACK_PID" >/dev/null 2>&1; then
      echo "error: run-game-test stack exited unexpectedly" >&2
      tail -n 120 "$RUN_GAME_TEST_LOG" >&2 || true
      exit 1
    fi
    GAME_URL="$(sed -n 's/^- URL: \(http[^[:space:]]*\)$/\1/p' "$RUN_GAME_TEST_LOG" | tail -n 1)"
    STACK_OUTPUT_DIR="$(sed -n 's/^- Logs: \(.*\)$/\1/p' "$RUN_GAME_TEST_LOG" | tail -n 1)"
    [[ -n "$GAME_URL" ]] && break
    sleep 1
  done

  if [[ -z "$GAME_URL" ]]; then
    echo "error: timeout waiting for game URL from run-game-test.sh" >&2
    tail -n 120 "$RUN_GAME_TEST_LOG" >&2 || true
    exit 1
  fi
else
  {
    echo "### [bootstrap_stack] $(date '+%H:%M:%S')"
    echo "skip stack bootstrap; using provided URL: $GAME_URL"
    echo
  } | tee -a "$AB_LOG" >/dev/null
fi

ab_log_note open
ab_open "$SESSION" "$HEADED" "$GAME_URL" 2>&1 | tee -a "$AB_LOG" >/dev/null
ab_log_note wait_network
ab_cmd "$SESSION" wait --load networkidle 2>&1 | tee -a "$AB_LOG" >/dev/null || true

SNAPSHOT_OK=0
for attempt in 1 2 3 4 5; do
  ab_log_note "snapshot_initial_attempt_${attempt}"
  if snapshot_output=$(ab_cmd "$SESSION" snapshot -i 2>&1); then
    printf "%s\n" "$snapshot_output" | tee -a "$AB_LOG" >/dev/null
    SNAPSHOT_OK=1
    break
  fi
  printf "%s\n" "$snapshot_output" | tee -a "$AB_LOG" >/dev/null
  sleep 1
done
if [[ "$SNAPSHOT_OK" -ne 1 ]]; then
  echo "warning: snapshot still failing after retries; continue with eval path" | tee -a "$AB_LOG" >/dev/null
fi

ab_log_note record_start
ab_cmd "$SESSION" record start "$OUT_DIR/playthrough.webm" >>"$AB_LOG" 2>&1 || true

wait_for_api 20000 >/dev/null || { echo "error: __AW_TEST__ unavailable" >&2; exit 1; }
initial=$(wait_for_connected 20000)
ab_cmd "$SESSION" screenshot "$OUT_DIR/step0-home.png" >>"$AB_LOG" 2>&1 || true

phaseA_play=$(send_control_probe phase_a_play play '{}' true 12000)
no_progress_observation=$(observe_no_progress_window 6000)
phaseA_pause=$(send_control_probe phase_a_pause pause '{}' false 2500)
ab_cmd "$SESSION" screenshot "$OUT_DIR/step1-phase-a.png" >>"$AB_LOG" 2>&1 || true

phaseB_step_primary=$(send_control_probe phase_b_step_primary step '{"count":8}' true 6000)
phaseB_step_followup=$(send_control_probe phase_b_step_followup step '{"count":2}' true 6000)
ab_cmd "$SESSION" screenshot "$OUT_DIR/step2-phase-b.png" >>"$AB_LOG" 2>&1 || true

final_state=$(wait_for_connected 8000)
ab_cmd "$SESSION" screenshot "$OUT_DIR/step3-final.png" >>"$AB_LOG" 2>&1 || true

AB_RESULT_JSON=$(python3 - <<'PY' \
  "$RUN_ID" "$GAME_URL" "$initial" "$final_state" \
  "$phaseA_play" "$phaseA_pause" "$phaseB_step_primary" "$phaseB_step_followup" "$no_progress_observation"
import json, sys
run_id, url, initial_raw, final_raw, play_raw, pause_raw, step1_raw, step2_raw, nop_raw = sys.argv[1:10]
initial = json.loads(initial_raw)
final = json.loads(final_raw)
play = json.loads(play_raw)
pause = json.loads(pause_raw)
step1 = json.loads(step1_raw)
step2 = json.loads(step2_raw)
no_progress = json.loads(nop_raw)
commands = [play, pause, step1, step2]
expected = [item for item in commands if item.get('expectProgress')]
progressed = [item for item in expected if item.get('progressed')]
accepted = [item for item in commands if item.get('accepted')]
result = {
    'runId': run_id,
    'url': url,
    'states': {
        'initial': initial,
        'final': final,
    },
    'phases': {
        'phaseA': {
            'play': play,
            'pause': pause,
            'noProgressObservation': no_progress,
            'pass': bool(play.get('progressed') and pause.get('accepted')),
        },
        'phaseB': {
            'stepPrimary': step1,
            'stepFollowup': step2,
            'pass': bool(step1.get('progressed') and step2.get('progressed')),
        },
    },
    'commands': commands,
    'metrics': {
        'ttfcMs': play.get('firstProgressMs'),
        'totalControls': len(commands),
        'acceptedControls': len(accepted),
        'expectedProgressControls': len(expected),
        'effectiveProgressControls': len(progressed),
        'effectiveControlHitRate': (len(progressed) / len(expected)) if expected else 0,
        'maxNoProgressWindowMs': int(no_progress.get('maxNoProgressWindowMs', 0)),
        'initialTick': int(float(initial.get('tick', 0) or 0)),
        'finalTick': int(float(final.get('tick', 0) or 0)),
        'initialEventSeq': int(float(initial.get('eventSeq', 0) or 0)),
        'finalEventSeq': int(float(final.get('eventSeq', 0) or 0)),
    },
}
print(json.dumps(result, ensure_ascii=False))
PY
)

python3 - "$AB_RESULT_JSON" "$AB_METRICS_JSON" "$AB_METRICS_MD" "$CARD_METRICS_MD" <<'PY'
import datetime as dt
import json
import pathlib
import sys

raw = sys.argv[1]
metrics_path = pathlib.Path(sys.argv[2])
summary_path = pathlib.Path(sys.argv[3])
card_path = pathlib.Path(sys.argv[4])

data = json.loads(raw)
metrics = data.get("metrics", {})
phases = data.get("phases", {})
commands = data.get("commands", [])

metrics_path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

ttfc = metrics.get("ttfcMs")
effective = metrics.get("effectiveControlHitRate", 0)
effective_pct = f"{effective * 100:.1f}%"
expected_progress = int(metrics.get("expectedProgressControls", 0))
effective_progress = int(metrics.get("effectiveProgressControls", 0))
accepted = int(metrics.get("acceptedControls", 0))
total = int(metrics.get("totalControls", 0))
max_stall = int(metrics.get("maxNoProgressWindowMs", 0))
initial_tick = int(metrics.get("initialTick", 0))
final_tick = int(metrics.get("finalTick", 0))
initial_event_seq = int(metrics.get("initialEventSeq", 0))
final_event_seq = int(metrics.get("finalEventSeq", 0))

phase_a = phases.get("phaseA", {})
phase_b = phases.get("phaseB", {})
phase_a_pass = bool(phase_a.get("pass"))
phase_b_pass = bool(phase_b.get("pass"))

cmd_lines = []
for item in commands:
    cmd_lines.append(
        f"- `{item.get('name')}` action=`{item.get('action')}` "
        f"accepted={item.get('accepted')} progressed={item.get('progressed')} "
        f"category=`{item.get('failCategory')}` "
        f"tick `{item.get('beforeTick')}` -> `{item.get('afterTick')}` "
        f"eventSeq `{item.get('beforeEventSeq')}` -> `{item.get('afterEventSeq')}` "
        f"stage=`{item.get('feedbackStage')}` reason=`{item.get('feedbackReason')}`"
    )

summary_lines = [
    "# Playability A/B Metrics",
    "",
    f"- Timestamp: {dt.datetime.now().astimezone().strftime('%Y-%m-%d %H:%M:%S %Z')}",
    f"- Run ID: `{data.get('runId')}`",
    f"- URL: `{data.get('url')}`",
    "",
    "## Quant Metrics",
    f"- TTFC(ms): `{ttfc}`" if ttfc is not None else "- TTFC(ms): `null`",
    f"- Effective control hit-rate: `{effective_progress}/{expected_progress}` (`{effective_pct}`)",
    f"- Accepted control rate: `{accepted}/{total}`",
    f"- Max no-progress window(ms): `{max_stall}`",
    f"- Tick: `{initial_tick}` -> `{final_tick}`",
    f"- EventSeq: `{initial_event_seq}` -> `{final_event_seq}`",
    "",
    "## A/B Verdict",
    f"- A (play/pause): `{'PASS' if phase_a_pass else 'FAIL'}`",
    f"- B (step chain): `{'PASS' if phase_b_pass else 'FAIL'}`",
    f"- B step primary category: `{phase_b.get('stepPrimary', {}).get('failCategory')}`",
    f"- B step followup category: `{phase_b.get('stepFollowup', {}).get('failCategory')}`",
    "",
    "## Control Probes",
] + cmd_lines + [""]

summary_path.write_text("\n".join(summary_lines), encoding="utf-8")

card_lines = [
    "# 量化指标（自动填写）",
    "",
    f"- TTFC（首次可控时间，ms）：`{ttfc}`" if ttfc is not None else "- TTFC（首次可控时间，ms）：`null`",
    f"- 有效控制命中率（有效推进控制次数 / 预期推进控制次数）：`{effective_progress}/{expected_progress}`（`{effective_pct}`）",
    f"- 无进展窗口时长（ms，connected 下 tick 不变最长窗口）：`{max_stall}`",
    "- A/B 分段结论：",
    f"  - A（play/pause）：`{'PASS' if phase_a_pass else 'FAIL'}`",
    f"  - B（step链路）：`{'PASS' if phase_b_pass else 'FAIL'}`",
    f"  - B primary 失败分类：`{phase_b.get('stepPrimary', {}).get('failCategory')}`",
    f"  - B followup 失败分类：`{phase_b.get('stepFollowup', {}).get('failCategory')}`",
    "",
]
card_path.write_text("\n".join(card_lines), encoding="utf-8")
PY

ab_log_note console_all
ab_cmd "$SESSION" console >"$CONSOLE_ALL_LOG" 2>&1 || true
python3 - "$CONSOLE_ALL_LOG" "$CONSOLE_WARNING_LOG" <<'PY'
import pathlib, sys
src = pathlib.Path(sys.argv[1])
out = pathlib.Path(sys.argv[2])
if src.exists():
    warnings = [line for line in src.read_text(encoding='utf-8', errors='replace').splitlines() if 'warn' in line.lower() or 'warning' in line.lower()]
    out.write_text("\n".join(warnings) + ("\n" if warnings else ""), encoding='utf-8')
else:
    out.write_text("", encoding='utf-8')
PY

ab_log_note record_stop
ab_cmd "$SESSION" record stop >>"$AB_LOG" 2>&1 || true
ab_log_note close
ab_cmd "$SESSION" close >>"$AB_LOG" 2>&1 || true

if [[ -n "$STACK_OUTPUT_DIR" && -d "$STACK_OUTPUT_DIR" ]]; then
  cp "$STACK_OUTPUT_DIR/session.meta" "$OUT_DIR/startup.session.meta" 2>/dev/null || true
  cp "$STACK_OUTPUT_DIR/world_viewer_live.log" "$OUT_DIR/startup.world.initial.log" 2>/dev/null || true
  cp "$STACK_OUTPUT_DIR/web_viewer.log" "$OUT_DIR/startup.web.initial.log" 2>/dev/null || true
fi

stop_stack

echo "playability A/B run complete"
echo "- run id: $RUN_ID"
echo "- url: $GAME_URL"
echo "- artifacts: $OUT_DIR"
echo "- metrics json: $AB_METRICS_JSON"
echo "- metrics summary: $AB_METRICS_MD"
echo "- card metrics snippet: $CARD_METRICS_MD"
echo "- reminder: regression probe only; still run manual long-play before final judgment"
