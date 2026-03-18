#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
source "$repo_root/scripts/agent-browser-lib.sh"

usage() {
  cat <<'USAGE'
Usage: ./scripts/viewer-standard-loading-overlay-regression.sh [options]

Verify the standard Web Viewer bootstrap removes the loading overlay once the
wasm viewer starts and the canvas is available.

Options:
  --static-dir <path>   Use an existing static dir instead of resolving/rebuilding
  --out-dir <path>      Artifact root (default: output/playwright/viewer-standard-overlay)
  --headed              Open browser in headed mode
  --headless            Open browser in headless mode (default)
  -h, --help            Show this help
USAGE
}

sleep_ms() {
  python3 - "$1" <<'PY'
import sys
import time

time.sleep(int(sys.argv[1]) / 1000.0)
PY
}

pick_port() {
  python3 - <<'PY'
import socket

with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
}

normalize_eval_token() {
  local raw=${1:-}
  raw=$(printf '%s' "$raw" | tr -d '\r\n')
  raw=${raw#\"}
  raw=${raw%\"}
  printf '%s' "$raw"
}

wait_for_js_true() {
  local session=$1
  local script=$2
  local timeout_ms=${3:-15000}
  local deadline=$((SECONDS * 1000 + timeout_ms))
  while (( SECONDS * 1000 < deadline )); do
    local value
    value=$(normalize_eval_token "$(ab_eval "$session" "$script")")
    if [[ "$value" == "true" ]]; then
      return 0
    fi
    sleep_ms 200
  done
  return 1
}

STATIC_DIR=""
OUT_ROOT="output/playwright/viewer-standard-overlay"
HEADED=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --static-dir)
      STATIC_DIR=$2
      shift 2
      ;;
    --out-dir)
      OUT_ROOT=$2
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
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

ab_require

run_id="$(date +%Y%m%d_%H%M%S)"
artifact_dir="$OUT_ROOT/$run_id"
mkdir -p "$artifact_dir"
ab_log="$artifact_dir/agent-browser.log"

if [[ -z "$STATIC_DIR" ]]; then
  STATIC_DIR="$(resolve_viewer_static_dir_for_web_closure "$repo_root" web "$repo_root/output/viewer-standard-overlay-regression")"
fi

if [[ ! -f "$STATIC_DIR/index.html" ]]; then
  echo "error: static dir missing index.html: $STATIC_DIR" >&2
  exit 1
fi

port="$(pick_port)"
server_log="$artifact_dir/static-server.log"
python3 -m http.server "$port" --bind 127.0.0.1 --directory "$STATIC_DIR" >"$server_log" 2>&1 &
server_pid=$!
session="viewer-standard-overlay-$run_id"

cleanup() {
  kill "$server_pid" >/dev/null 2>&1 || true
  wait "$server_pid" >/dev/null 2>&1 || true
  ab_run "$session" close >/dev/null 2>&1 || true
}
trap cleanup EXIT

url="http://127.0.0.1:${port}/?render_mode=standard&test_api=1&ws=ws://127.0.0.1:9"
{
  echo "static_dir=$STATIC_DIR"
  echo "url=$url"
} >"$artifact_dir/context.txt"

ab_open "$session" "$HEADED" "$url" | tee "$ab_log"

wait_for_js_true "$session" 'document.querySelector("canvas") !== null' 20000
wait_for_js_true "$session" 'document.querySelector(".viewer-loading") === null' 20000

summary_json="$(ab_eval "$session" '(() => {
  const canvas = document.querySelector("canvas");
  const rect = canvas ? canvas.getBoundingClientRect() : null;
  return {
    canvasCount: document.querySelectorAll("canvas").length,
    overlayPresent: !!document.querySelector(".viewer-loading"),
    bodyTextContainsLoading: document.body.innerText.includes("Loading standard viewer"),
    canvasWidth: rect ? rect.width : null,
    viewportWidth: window.innerWidth,
    canvasNearlyFullWidth: !!rect && rect.width >= window.innerWidth * 0.95,
    state: window.__AW_TEST__?.getState?.() ?? null,
  };
})()')"
json_to_file "$summary_json" "$artifact_dir/summary.json"

ab_screenshot "$session" "$artifact_dir/final.png" | tee -a "$ab_log"

overlay_present="$(json_get "$summary_json" overlayPresent)"
loading_text_present="$(json_get "$summary_json" bodyTextContainsLoading)"
canvas_count="$(json_get "$summary_json" canvasCount)"
canvas_full_width="$(json_get "$summary_json" canvasNearlyFullWidth)"

if [[ "$overlay_present" != "false" ]]; then
  echo "error: loading overlay still present after standard viewer startup" >&2
  exit 1
fi

if [[ "$loading_text_present" != "false" ]]; then
  echo "error: body text still contains loading overlay copy after startup" >&2
  exit 1
fi

if [[ "$canvas_count" != "1" ]]; then
  echo "error: expected exactly one canvas, got $canvas_count" >&2
  exit 1
fi

if [[ "$canvas_full_width" != "true" ]]; then
  echo "error: canvas did not recover near-full viewport width" >&2
  exit 1
fi

echo "viewer standard loading overlay regression passed"
