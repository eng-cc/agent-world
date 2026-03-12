#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

BUNDLE_DIR="output/release/game-launcher-producer-local"
PROFILE="release"
REBUILD=0
OPEN_HEADED=0
SESSION_NAME="producer-playtest"
STARTUP_TIMEOUT_SECS=120
STACK_ARGS=()

usage() {
  cat <<'USAGE'
Usage: ./scripts/run-producer-playtest.sh [options] [run-game-test options...]

Prepare a bundle-first Web stack for producer manual play.

Default behavior:
- reuse `output/release/game-launcher-producer-local` if it already exists
- otherwise build a fresh bundle there
- then start `./scripts/run-game-test.sh --bundle-dir <bundle>`

Options:
  --bundle-dir <path>      Bundle directory to reuse/build (default: output/release/game-launcher-producer-local)
  --profile <name>         Bundle build profile: release|dev (default: release)
  --rebuild                Force rebuild even if bundle already exists
  --open-headed            After stack ready, auto-open the Viewer URL in headed `agent-browser`
  --session <name>         `agent-browser` session name for `--open-headed` (default: producer-playtest)
  --startup-timeout <secs> Wait timeout for stack URL when `--open-headed` is used (default: 120)
  -h, --help               Show this help

Examples:
  ./scripts/run-producer-playtest.sh --no-llm
  ./scripts/run-producer-playtest.sh --profile dev --no-llm
  ./scripts/run-producer-playtest.sh --no-llm --open-headed
  ./scripts/run-producer-playtest.sh --bundle-dir output/release/game-launcher-local --no-llm
USAGE
}

require_cmd() {
  local cmd=$1
  command -v "$cmd" >/dev/null 2>&1 || {
    echo "error: missing required command: $cmd" >&2
    exit 1
  }
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bundle-dir)
      BUNDLE_DIR="${2:-}"
      shift 2
      ;;
    --profile)
      PROFILE="${2:-}"
      shift 2
      ;;
    --rebuild)
      REBUILD=1
      shift
      ;;
    --open-headed)
      OPEN_HEADED=1
      shift
      ;;
    --session)
      SESSION_NAME="${2:-}"
      shift 2
      ;;
    --startup-timeout)
      STARTUP_TIMEOUT_SECS="${2:-}"
      shift 2
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

[[ -n "$BUNDLE_DIR" ]] || { echo "error: --bundle-dir cannot be empty" >&2; exit 2; }
[[ "$PROFILE" == "release" || "$PROFILE" == "dev" ]] || { echo "error: --profile must be release or dev" >&2; exit 2; }
[[ -n "$SESSION_NAME" ]] || { echo "error: --session cannot be empty" >&2; exit 2; }
[[ "$STARTUP_TIMEOUT_SECS" =~ ^[0-9]+$ ]] && [[ "$STARTUP_TIMEOUT_SECS" -gt 0 ]] || { echo "error: --startup-timeout must be a positive integer" >&2; exit 2; }

if [[ "$BUNDLE_DIR" != /* ]]; then
  ABS_BUNDLE_DIR="$ROOT_DIR/$BUNDLE_DIR"
else
  ABS_BUNDLE_DIR="$BUNDLE_DIR"
fi

if [[ "$REBUILD" == "1" || ! -x "$ABS_BUNDLE_DIR/run-game.sh" ]]; then
  echo "info: preparing producer playtest bundle at $ABS_BUNDLE_DIR (profile=$PROFILE)"
  ./scripts/build-game-launcher-bundle.sh --profile "$PROFILE" --out-dir "$ABS_BUNDLE_DIR"
else
  echo "info: reusing existing producer playtest bundle at $ABS_BUNDLE_DIR"
fi

if [[ "$OPEN_HEADED" != "1" ]]; then
  exec ./scripts/run-game-test.sh --bundle-dir "$ABS_BUNDLE_DIR" "${STACK_ARGS[@]}"
fi

require_cmd agent-browser
mkdir -p "$ROOT_DIR/output/playwright/playability"
RUN_ID="$(date +%Y%m%d-%H%M%S)"
RUN_LOG="$ROOT_DIR/output/playwright/playability/producer-launch-${RUN_ID}.log"
STACK_PID=""

cleanup() {
  local exit_code=$?
  trap - EXIT INT TERM
  if [[ -n "$STACK_PID" ]] && kill -0 "$STACK_PID" >/dev/null 2>&1; then
    kill "$STACK_PID" >/dev/null 2>&1 || true
    wait "$STACK_PID" >/dev/null 2>&1 || true
  fi
  exit "$exit_code"
}
trap cleanup EXIT INT TERM

if command -v stdbuf >/dev/null 2>&1; then
  stdbuf -oL -eL ./scripts/run-game-test.sh --bundle-dir "$ABS_BUNDLE_DIR" "${STACK_ARGS[@]}" > >(tee "$RUN_LOG") 2>&1 &
else
  ./scripts/run-game-test.sh --bundle-dir "$ABS_BUNDLE_DIR" "${STACK_ARGS[@]}" > >(tee "$RUN_LOG") 2>&1 &
fi
STACK_PID=$!

GAME_URL=""
STACK_OUTPUT_DIR=""
for ((i = 0; i < STARTUP_TIMEOUT_SECS; i++)); do
  if ! kill -0 "$STACK_PID" >/dev/null 2>&1; then
    echo "error: producer playtest stack exited unexpectedly" >&2
    tail -n 120 "$RUN_LOG" >&2 || true
    exit 1
  fi
  GAME_URL="$(sed -n 's/^- URL: \(http[^[:space:]]*\)$/\1/p' "$RUN_LOG" | tail -n 1)"
  STACK_OUTPUT_DIR="$(sed -n 's/^- Logs: \(.*\)$/\1/p' "$RUN_LOG" | tail -n 1)"
  [[ -n "$GAME_URL" ]] && break
  sleep 1
done

if [[ -z "$GAME_URL" ]]; then
  echo "error: timeout waiting for game URL from run-game-test.sh" >&2
  tail -n 120 "$RUN_LOG" >&2 || true
  exit 1
fi

echo "info: opening headed browser session '$SESSION_NAME' -> $GAME_URL"
AGENT_BROWSER_SESSION="$SESSION_NAME" agent-browser --headed open "$GAME_URL"
AGENT_BROWSER_SESSION="$SESSION_NAME" agent-browser wait --load networkidle >/dev/null 2>&1 || true

echo "info: browser session: $SESSION_NAME"
echo "info: startup log: $RUN_LOG"
echo "info: stack logs: ${STACK_OUTPUT_DIR:-unknown}"

echo "Press Ctrl+C to stop the producer playtest stack."
wait "$STACK_PID"
