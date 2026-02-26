#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

VIEWER_HOST="127.0.0.1"
VIEWER_PORT="4173"
LIVE_BIND_ADDR="127.0.0.1:5023"
WEB_BRIDGE_ADDR="127.0.0.1:5011"
TICK_MS="300"
ENABLE_LLM="0"

usage() {
  cat <<'USAGE'
Usage: ./scripts/run-game-test.sh [options]

Start a stable web playability test stack with safe defaults:
- world_viewer_live: llm_bootstrap --no-llm --bind 127.0.0.1:5023 --web-bind 127.0.0.1:5011
- web viewer: run-viewer-web.sh --address 127.0.0.1 --port 4173

Options:
  --viewer-host <host>     Viewer HTTP host (default: 127.0.0.1)
  --viewer-port <port>     Viewer HTTP port (default: 4173)
  --live-bind <addr:port>  world_viewer_live TCP bind (default: 127.0.0.1:5023)
  --web-bind <addr:port>   WebSocket bridge bind (default: 127.0.0.1:5011)
  --tick-ms <ms>           Tick interval in milliseconds (default: 300)
  --with-llm               Enable LLM mode (default: disabled)
  -h, --help               Show this help
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --viewer-host)
      VIEWER_HOST="${2:-}"
      shift 2
      ;;
    --viewer-port)
      VIEWER_PORT="${2:-}"
      shift 2
      ;;
    --live-bind)
      LIVE_BIND_ADDR="${2:-}"
      shift 2
      ;;
    --web-bind)
      WEB_BRIDGE_ADDR="${2:-}"
      shift 2
      ;;
    --tick-ms)
      TICK_MS="${2:-}"
      shift 2
      ;;
    --with-llm)
      ENABLE_LLM="1"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ -z "$VIEWER_HOST" || -z "$VIEWER_PORT" || -z "$LIVE_BIND_ADDR" || -z "$WEB_BRIDGE_ADDR" || -z "$TICK_MS" ]]; then
  echo "error: empty argument is not allowed" >&2
  exit 1
fi

if ! [[ "$VIEWER_PORT" =~ ^[0-9]+$ ]]; then
  echo "error: --viewer-port must be numeric" >&2
  exit 1
fi

if ! [[ "$TICK_MS" =~ ^[0-9]+$ ]]; then
  echo "error: --tick-ms must be numeric" >&2
  exit 1
fi

if [[ "$LIVE_BIND_ADDR" != *:* || "$WEB_BRIDGE_ADDR" != *:* ]]; then
  echo "error: --live-bind/--web-bind must be in <host:port> format" >&2
  exit 1
fi

LIVE_BIND_HOST="${LIVE_BIND_ADDR%:*}"
LIVE_BIND_PORT="${LIVE_BIND_ADDR##*:}"
WEB_BRIDGE_HOST="${WEB_BRIDGE_ADDR%:*}"
WEB_BRIDGE_PORT="${WEB_BRIDGE_ADDR##*:}"

if [[ -z "$LIVE_BIND_HOST" || -z "$LIVE_BIND_PORT" || -z "$WEB_BRIDGE_HOST" || -z "$WEB_BRIDGE_PORT" ]]; then
  echo "error: invalid bind address" >&2
  exit 1
fi

if ! [[ "$LIVE_BIND_PORT" =~ ^[0-9]+$ && "$WEB_BRIDGE_PORT" =~ ^[0-9]+$ ]]; then
  echo "error: bind ports must be numeric" >&2
  exit 1
fi

port_in_use() {
  local port="$1"
  if command -v lsof >/dev/null 2>&1; then
    lsof -iTCP:"$port" -sTCP:LISTEN -n -P >/dev/null 2>&1
    return $?
  fi

  if command -v ss >/dev/null 2>&1; then
    ss -ltn | grep -Eq "[:.]${port}[[:space:]]"
    return $?
  fi

  return 1
}

print_port_owner() {
  local port="$1"
  if command -v lsof >/dev/null 2>&1; then
    lsof -iTCP:"$port" -sTCP:LISTEN -n -P || true
  elif command -v ss >/dev/null 2>&1; then
    ss -ltnp | grep -E "[:.]${port}[[:space:]]" || true
  fi
}

check_port_free() {
  local port="$1"
  if port_in_use "$port"; then
    echo "error: port ${port} is already in use" >&2
    print_port_owner "$port" >&2
    exit 1
  fi
}

wait_for_http_ready() {
  local url="$1"
  local timeout_secs="$2"
  local i
  for ((i = 0; i < timeout_secs; i++)); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  return 1
}

wait_for_tcp_listener_ready() {
  local port="$1"
  local timeout_secs="$2"
  local i
  if ! command -v lsof >/dev/null 2>&1 && ! command -v ss >/dev/null 2>&1; then
    echo "warning: neither lsof nor ss found; skip passive listener probe for port ${port}" >&2
    return 0
  fi
  for ((i = 0; i < timeout_secs; i++)); do
    if port_in_use "$port"; then
      return 0
    fi
    sleep 1
  done
  return 1
}

tail_logs_on_error() {
  echo "--- world_viewer_live.log (tail) ---" >&2
  tail -n 80 "$WORLD_LOG" >&2 || true
  echo "--- web_viewer.log (tail) ---" >&2
  tail -n 80 "$WEB_LOG" >&2 || true
}

check_port_free "$VIEWER_PORT"
check_port_free "$WEB_BRIDGE_PORT"

RUN_ID="$(date +%Y%m%d-%H%M%S)"
OUTPUT_DIR="$ROOT_DIR/output/playwright/playability/startup-${RUN_ID}"
mkdir -p "$OUTPUT_DIR"

WORLD_LOG="$OUTPUT_DIR/world_viewer_live.log"
WEB_LOG="$OUTPUT_DIR/web_viewer.log"
META_FILE="$OUTPUT_DIR/session.meta"

WORLD_PID=""
WEB_PID=""

cleanup() {
  local exit_code=$?
  trap - EXIT INT TERM

  if [[ -n "$WEB_PID" ]] && kill -0 "$WEB_PID" >/dev/null 2>&1; then
    kill "$WEB_PID" >/dev/null 2>&1 || true
  fi
  if [[ -n "$WORLD_PID" ]] && kill -0 "$WORLD_PID" >/dev/null 2>&1; then
    kill "$WORLD_PID" >/dev/null 2>&1 || true
  fi

  wait "$WEB_PID" >/dev/null 2>&1 || true
  wait "$WORLD_PID" >/dev/null 2>&1 || true

  exit "$exit_code"
}
trap cleanup EXIT INT TERM

WORLD_ARGS=(
  llm_bootstrap
  --bind "$LIVE_BIND_ADDR"
  --web-bind "$WEB_BRIDGE_ADDR"
  --tick-ms "$TICK_MS"
  --topology single
  --viewer-no-consensus-gate
)
if [[ "$ENABLE_LLM" == "1" ]]; then
  WORLD_ARGS+=(--llm)
else
  WORLD_ARGS+=(--no-llm)
fi

(
  cd "$ROOT_DIR"
  env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- "${WORLD_ARGS[@]}" >"$WORLD_LOG" 2>&1
) &
WORLD_PID=$!

(
  cd "$ROOT_DIR"
  env -u NO_COLOR ./scripts/run-viewer-web.sh --address "$VIEWER_HOST" --port "$VIEWER_PORT" >"$WEB_LOG" 2>&1
) &
WEB_PID=$!

{
  echo "RUN_ID=$RUN_ID"
  echo "OUTPUT_DIR=$OUTPUT_DIR"
  echo "WORLD_PID=$WORLD_PID"
  echo "WEB_PID=$WEB_PID"
  echo "LIVE_BIND_ADDR=$LIVE_BIND_ADDR"
  echo "WEB_BRIDGE_ADDR=$WEB_BRIDGE_ADDR"
  echo "VIEWER_HOST=$VIEWER_HOST"
  echo "VIEWER_PORT=$VIEWER_PORT"
} >"$META_FILE"

if ! wait_for_http_ready "http://${VIEWER_HOST}:${VIEWER_PORT}/" 180; then
  echo "error: viewer HTTP did not become ready in time" >&2
  tail_logs_on_error
  exit 1
fi

if ! wait_for_tcp_listener_ready "$WEB_BRIDGE_PORT" 60; then
  echo "error: web bridge port ${WEB_BRIDGE_PORT} did not become ready in time" >&2
  tail_logs_on_error
  exit 1
fi

URL_VIEWER_HOST="$VIEWER_HOST"
if [[ "$URL_VIEWER_HOST" == "0.0.0.0" ]]; then
  URL_VIEWER_HOST="127.0.0.1"
fi
URL_WS_HOST="$WEB_BRIDGE_HOST"
if [[ "$URL_WS_HOST" == "0.0.0.0" ]]; then
  URL_WS_HOST="127.0.0.1"
fi

GAME_URL="http://${URL_VIEWER_HOST}:${VIEWER_PORT}/?ws=ws://${URL_WS_HOST}:${WEB_BRIDGE_PORT}&test_api=1"

cat <<INFO
Game test stack is ready.
- URL: $GAME_URL
- Logs: $OUTPUT_DIR

Playwright example:
  PLAYWRIGHT_CLI_SESSION=game-test-open \\
  ./.codex/skills/playwright/scripts/playwright_cli.sh open "$GAME_URL" --headed

Press Ctrl+C to stop both processes.
INFO

while true; do
  if ! kill -0 "$WORLD_PID" >/dev/null 2>&1; then
    echo "error: world_viewer_live exited unexpectedly" >&2
    tail_logs_on_error
    exit 1
  fi
  if ! kill -0 "$WEB_PID" >/dev/null 2>&1; then
    echo "error: run-viewer-web.sh exited unexpectedly" >&2
    tail_logs_on_error
    exit 1
  fi
  sleep 1
done
