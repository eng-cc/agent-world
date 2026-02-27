#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

usage() {
  cat <<'USAGE'
Usage: ./scripts/run-game-test-ab.sh [options] [run-game-test options...]

Run a stable A/B playability loop and emit quantitative metrics:
- A phase: play -> observe -> pause
- B phase: step-chain control probes (no seek)
- Outputs TTFC / effective control hit-rate / max no-progress window

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
  <out-dir>/<run-id>/playwright.log
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

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "error: missing required command: $cmd" >&2
    exit 1
  fi
}

extract_result_json() {
  local content="$1"
  printf "%s\n" "$content" | awk '
    /^### Result$/ { capture=1; next }
    /^### Ran Playwright code$/ { capture=0 }
    capture { print }
  ' | sed '/^[[:space:]]*$/d' | tail -n 1
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

if [[ -z "$OUT_ROOT" ]]; then
  echo "error: --out-dir cannot be empty" >&2
  exit 2
fi
if ! [[ "$STARTUP_TIMEOUT_SECS" =~ ^[0-9]+$ ]] || [[ "$STARTUP_TIMEOUT_SECS" -le 0 ]]; then
  echo "error: --startup-timeout must be a positive integer" >&2
  exit 2
fi

require_cmd npx
require_cmd python3
require_cmd rg

LOCAL_PWCLI="$ROOT_DIR/.codex/skills/playwright/scripts/playwright_cli.sh"
HOME_PWCLI="${CODEX_HOME:-$HOME/.codex}/skills/playwright/scripts/playwright_cli.sh"
if [[ -x "$LOCAL_PWCLI" ]]; then
  PWCLI="$LOCAL_PWCLI"
elif [[ -x "$HOME_PWCLI" ]]; then
  PWCLI="$HOME_PWCLI"
else
  echo "error: playwright wrapper not found (checked: $LOCAL_PWCLI, $HOME_PWCLI)" >&2
  exit 1
fi

RUN_ID="$(date +%Y%m%d-%H%M%S)"
OUT_DIR="$OUT_ROOT/$RUN_ID"
mkdir -p "$OUT_DIR"

PLAYWRIGHT_LOG="$OUT_DIR/playwright.log"
RUN_GAME_TEST_LOG="$OUT_DIR/run-game-test.log"
CONSOLE_WARNING_LOG="$OUT_DIR/console_warning_dump.log"
CONSOLE_ALL_LOG="$OUT_DIR/console_all_messages.log"
AB_METRICS_JSON="$OUT_DIR/ab_metrics.json"
AB_METRICS_MD="$OUT_DIR/ab_metrics.md"
CARD_METRICS_MD="$OUT_DIR/card_quant_metrics.md"
SESSION="playability-ab-$RUN_ID"

STACK_PID=""
STACK_OUTPUT_DIR=""

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
  PLAYWRIGHT_CLI_SESSION="$SESSION" "$PWCLI" close >/dev/null 2>&1 || true
  stop_stack
  exit "$exit_code"
}
trap cleanup EXIT INT TERM

pw_capture() {
  local label="$1"
  shift
  local output
  output=$({
    echo "### [${label}] $(date '+%H:%M:%S')"
    PLAYWRIGHT_CLI_SESSION="$SESSION" "$PWCLI" "$@"
    echo
  } 2>&1)
  printf "%s\n" "$output" | tee -a "$PLAYWRIGHT_LOG" >/dev/null
  printf "%s\n" "$output"
}

pw_capture_allow_fail() {
  local label="$1"
  shift
  local output
  local status=0
  output=$({
    echo "### [${label}] $(date '+%H:%M:%S')"
    PLAYWRIGHT_CLI_SESSION="$SESSION" "$PWCLI" "$@"
    echo
  } 2>&1) || status=$?
  printf "%s\n" "$output" | tee -a "$PLAYWRIGHT_LOG" >/dev/null
  printf "%s\n" "$output"
  return "$status"
}

if [[ -z "$GAME_URL" ]]; then
  {
    echo "### [bootstrap_stack] $(date '+%H:%M:%S')"
    echo "./scripts/run-game-test.sh ${STACK_ARGS[*]}"
    echo
  } | tee -a "$PLAYWRIGHT_LOG" >/dev/null

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
    if [[ -n "$GAME_URL" ]]; then
      break
    fi
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
  } | tee -a "$PLAYWRIGHT_LOG" >/dev/null
fi

OPEN_ARGS=(open "$GAME_URL")
if [[ "$HEADED" -eq 1 ]]; then
  OPEN_ARGS+=(--headed)
fi

pw_capture open "${OPEN_ARGS[@]}" >/dev/null

SNAPSHOT_OK=0
for attempt in 1 2 3 4 5; do
  if snapshot_output="$(pw_capture_allow_fail "snapshot_initial_attempt_${attempt}" snapshot)"; then
    if ! printf "%s\n" "$snapshot_output" | rg -q '^### Error'; then
      SNAPSHOT_OK=1
      break
    fi
  fi
  sleep 1
done
if [[ "$SNAPSHOT_OK" -ne 1 ]]; then
  echo "warning: snapshot still failing after retries; continue with run-code path" | tee -a "$PLAYWRIGHT_LOG" >/dev/null
fi

pw_capture video_start video-start >/dev/null

AB_FLOW_CODE=$(cat <<JS
async (page) => {
  const toNumber = (value, fallback = 0) => {
    const num = Number(value);
    return Number.isFinite(num) ? num : fallback;
  };
  const readState = async () =>
    await page.evaluate(() => window.__AW_TEST__?.getState?.() ?? null);
  const logicalTick = (state) => toNumber(state?.logicalTime ?? state?.tick, 0);
  const eventSeq = (state) => toNumber(state?.eventSeq, 0);

  const waitForApi = async (timeoutMs = 20000) => {
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
      const ready = await page.evaluate(() => typeof window.__AW_TEST__ === "object");
      if (ready) return;
      await page.waitForTimeout(200);
    }
    throw new Error("__AW_TEST__ unavailable");
  };

  const waitForConnected = async (label, timeoutMs = 20000) => {
    const deadline = Date.now() + timeoutMs;
    let state = null;
    while (Date.now() < deadline) {
      state = await readState();
      if (state?.connectionStatus === "connected") return state;
      await page.waitForTimeout(250);
    }
    throw new Error(
      label +
        ": not connected (status=" +
        state?.connectionStatus +
        ", lastError=" +
        state?.lastError +
        ")",
    );
  };

  const sendControlProbe = async ({
    name,
    action,
    payload,
    expectProgress,
    timeoutMs,
  }) => {
    const before = await readState();
    const baselineTick = logicalTick(before);
    const baselineEventSeq = eventSeq(before);
    const startedAt = Date.now();
    const feedback = await page.evaluate(
      ([controlAction, controlPayload]) => {
        try {
          return window.__AW_TEST__?.sendControl?.(controlAction, controlPayload) ?? null;
        } catch (err) {
          return {
            accepted: false,
            reason: String(err),
            effect: "exception on sendControl",
          };
        }
      },
      [action, payload],
    );
    let after = before;
    let progressed = false;
    let firstProgressMs = null;
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
      await page.waitForTimeout(250);
      after = await readState();
      const tickDelta = logicalTick(after) - baselineTick;
      const eventDelta = eventSeq(after) - baselineEventSeq;
      if (tickDelta > 0 || eventDelta > 0) {
        progressed = true;
        firstProgressMs = Date.now() - startedAt;
        break;
      }
      if (!expectProgress && Date.now() - startedAt >= 1000) {
        break;
      }
    }
    return {
      name,
      action,
      payload,
      expectProgress,
      accepted: feedback?.accepted === true,
      reason: feedback?.reason ?? null,
      effect: feedback?.effect ?? null,
      beforeTick: baselineTick,
      afterTick: logicalTick(after),
      beforeEventSeq: baselineEventSeq,
      afterEventSeq: eventSeq(after),
      progressed,
      firstProgressMs,
    };
  };

  const observeNoProgressWindow = async (durationMs) => {
    const startedAt = Date.now();
    let state = await readState();
    let lastTick = logicalTick(state);
    let stagnationStart = Date.now();
    let maxWindowMs = 0;
    while (Date.now() - startedAt < durationMs) {
      await page.waitForTimeout(250);
      state = await readState();
      const tick = logicalTick(state);
      if (state?.connectionStatus === "connected" && tick === lastTick) {
        const current = Date.now() - stagnationStart;
        if (current > maxWindowMs) maxWindowMs = current;
      } else {
        lastTick = tick;
        stagnationStart = Date.now();
      }
    }
    return {
      maxNoProgressWindowMs: maxWindowMs,
      finalObservedTick: logicalTick(state),
      finalObservedEventSeq: eventSeq(state),
    };
  };

  await waitForApi();
  const initial = await waitForConnected("initial");
  await page.screenshot({ path: "${OUT_DIR}/step0-home.png", scale: "css", type: "png" });

  const phaseAPlay = await sendControlProbe({
    name: "phase_a_play",
    action: "play",
    payload: {},
    expectProgress: true,
    timeoutMs: 12000,
  });
  const noProgressObservation = await observeNoProgressWindow(6000);
  const phaseAPause = await sendControlProbe({
    name: "phase_a_pause",
    action: "pause",
    payload: {},
    expectProgress: false,
    timeoutMs: 2500,
  });
  await page.screenshot({ path: "${OUT_DIR}/step1-phase-a.png", scale: "css", type: "png" });

  const phaseBStepPrimary = await sendControlProbe({
    name: "phase_b_step_primary",
    action: "step",
    payload: { count: 8 },
    expectProgress: true,
    timeoutMs: 6000,
  });
  const phaseBStepFollowup = await sendControlProbe({
    name: "phase_b_step_followup",
    action: "step",
    payload: { count: 2 },
    expectProgress: true,
    timeoutMs: 6000,
  });
  const phaseBMoveProbe = await sendControlProbe({
    name: "phase_b_move_probe",
    action: "move",
    payload: { x: 1, y: 0 },
    expectProgress: false,
    timeoutMs: 2000,
  });
  await page.screenshot({ path: "${OUT_DIR}/step2-phase-b.png", scale: "css", type: "png" });

  const finalState = await waitForConnected("final");
  await page.screenshot({ path: "${OUT_DIR}/step3-final.png", scale: "css", type: "png" });

  const commands = [phaseAPlay, phaseAPause, phaseBStepPrimary, phaseBStepFollowup, phaseBMoveProbe];
  const expectedProgressControls = commands.filter((item) => item.expectProgress);
  const progressedControls = expectedProgressControls.filter((item) => item.progressed);
  const acceptedControls = commands.filter((item) => item.accepted);
  const ttfcMs = phaseAPlay.progressed ? phaseAPlay.firstProgressMs : null;
  const effectiveRate =
    expectedProgressControls.length > 0
      ? progressedControls.length / expectedProgressControls.length
      : 0;

  return {
    runId: "${RUN_ID}",
    url: "${GAME_URL}",
    states: {
      initial,
      final: finalState,
    },
    phases: {
      phaseA: {
        play: phaseAPlay,
        pause: phaseAPause,
        noProgressObservation,
        pass: phaseAPlay.progressed && phaseAPause.accepted,
      },
      phaseB: {
        stepPrimary: phaseBStepPrimary,
        stepFollowup: phaseBStepFollowup,
        moveProbe: phaseBMoveProbe,
        pass: phaseBStepPrimary.progressed && phaseBStepFollowup.progressed,
      },
    },
    commands,
    metrics: {
      ttfcMs,
      totalControls: commands.length,
      acceptedControls: acceptedControls.length,
      expectedProgressControls: expectedProgressControls.length,
      effectiveProgressControls: progressedControls.length,
      effectiveControlHitRate: effectiveRate,
      maxNoProgressWindowMs: noProgressObservation.maxNoProgressWindowMs,
      initialTick: logicalTick(initial),
      finalTick: logicalTick(finalState),
      initialEventSeq: eventSeq(initial),
      finalEventSeq: eventSeq(finalState),
    },
  };
}
JS
)

AB_OUTPUT="$(pw_capture ab_flow run-code "$AB_FLOW_CODE")"
if printf "%s\n" "$AB_OUTPUT" | rg -q '^### Error'; then
  echo "error: A/B run-code returned error" >&2
  exit 1
fi

AB_RESULT_JSON="$(extract_result_json "$AB_OUTPUT")"
if [[ -z "$AB_RESULT_JSON" ]]; then
  echo "error: failed to parse A/B result json from run-code output" >&2
  exit 1
fi

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
        f"tick `{item.get('beforeTick')}` -> `{item.get('afterTick')}` "
        f"eventSeq `{item.get('beforeEventSeq')}` -> `{item.get('afterEventSeq')}`"
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
    "",
]
card_path.write_text("\n".join(card_lines), encoding="utf-8")
PY

CONSOLE_WARNING_OUTPUT="$(pw_capture_allow_fail console_warning console warning || true)"
printf "%s\n" "$CONSOLE_WARNING_OUTPUT" >"$CONSOLE_WARNING_LOG"

CONSOLE_ALL_OUTPUT="$(pw_capture_allow_fail console_all console || true)"
printf "%s\n" "$CONSOLE_ALL_OUTPUT" >"$CONSOLE_ALL_LOG"

pw_capture video_stop video-stop --filename "$OUT_DIR/playthrough.webm" >/dev/null
pw_capture_allow_fail close close >/dev/null || true

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
