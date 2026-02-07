#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

usage() {
  cat <<'USAGE'
Usage: ./scripts/capture-viewer-frame.sh [options]

Options:
  --scenario <name>       world_viewer_live scenario (default: llm_bootstrap)
  --addr <host:port>      bind/viewer address (default: 127.0.0.1:5023)
  --tick-ms <ms>          live tick interval in ms (default: 300)
  --display <display>     Xvfb display id for Linux mode (default: :100)
  --width <px>            virtual screen width (default: 1280)
  --height <px>           virtual screen height (default: 800)
  --viewer-wait <sec>     wait before capture (default: 8)
  --llm                   enable --llm on world_viewer_live
  --keep-tmp              do not clear .tmp at start
  -h, --help              show help

Behavior:
  - Linux: uses Xvfb + xwininfo + ffmpeg
  - macOS: uses screencapture + osascript (window info best-effort)

Output:
  .tmp/screens/
    live_server.log viewer.log xvfb.log
    root.png window.png window_line.txt window_geom.txt
USAGE
}

require_cmd() {
  local cmd=$1
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "missing required command: $cmd" >&2
    exit 1
  fi
}

run() {
  echo "+ $*"
  "$@"
}

detect_platform() {
  case "$(uname -s)" in
    Linux)
      echo "linux"
      ;;
    Darwin)
      echo "macos"
      ;;
    *)
      echo "unsupported"
      ;;
  esac
}

wait_linux_window_line() {
  local display=$1
  local line=""
  for _ in $(seq 1 30); do
    line=$(DISPLAY="$display" xwininfo -root -tree 2>/dev/null | grep "Agent World Viewer" | head -n1 || true)
    if [[ -n "$line" ]]; then
      echo "$line"
      return 0
    fi
    sleep 1
  done
  return 1
}

wait_macos_window_info() {
  local info=""
  for _ in $(seq 1 30); do
    info=$(osascript <<'APPLESCRIPT' 2>/dev/null || true
tell application "System Events"
  repeat with p in (every process whose background only is false)
    try
      tell p
        repeat with w in windows
          set windowTitle to ""
          try
            set windowTitle to name of w
          end try
          if windowTitle contains "Agent World Viewer" then
            set {xPos, yPos} to position of w
            set {wSize, hSize} to size of w
            set winId to ""
            try
              set winId to value of attribute "AXWindowNumber" of w
            end try
            return (name of p) & "|" & xPos & "|" & yPos & "|" & wSize & "|" & hSize & "|" & winId
          end if
        end repeat
      end tell
    end try
  end repeat
end tell
return ""
APPLESCRIPT
)
    if [[ -n "$info" ]]; then
      echo "$info"
      return 0
    fi
    sleep 1
  done
  return 1
}

capture_linux() {
  local display=$1
  local width=$2
  local height=$3
  local viewer_wait=$4
  local addr=$5
  local viewer_log=$6
  local xvfb_log=$7
  local root_png=$8
  local window_png=$9
  local window_line_txt=${10}
  local window_geom_txt=${11}

  echo "+ Xvfb $display -screen 0 ${width}x${height}x24 > $xvfb_log"
  Xvfb "$display" -screen 0 "${width}x${height}x24" >"$xvfb_log" 2>&1 &
  XVFB_PID=$!

  sleep 2

  echo "+ DISPLAY=$display env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- $addr > $viewer_log"
  DISPLAY="$display" env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- "$addr" >"$viewer_log" 2>&1 &
  VIEWER_PID=$!

  local window_line
  if ! window_line=$(wait_linux_window_line "$display"); then
    echo "failed to find window: Agent World Viewer" >&2
    exit 2
  fi
  echo "$window_line" > "$window_line_txt"

  sleep "$viewer_wait"

  run ffmpeg -y -f x11grab -video_size "${width}x${height}" -i "${display}.0" -frames:v 1 "$root_png"

  local window_geom
  window_geom=$(echo "$window_line" | sed -n 's/.* \([0-9]\+x[0-9]\++[0-9]\++[0-9]\+\).*/\1/p')
  if [[ -n "$window_geom" ]]; then
    echo "$window_geom" > "$window_geom_txt"
    local window_size window_x window_y
    window_size=$(echo "$window_geom" | cut -d+ -f1)
    window_x=$(echo "$window_geom" | cut -d+ -f2)
    window_y=$(echo "$window_geom" | cut -d+ -f3)
    run ffmpeg -y -f x11grab -video_size "$window_size" -i "${display}.0+${window_x},${window_y}" -frames:v 1 "$window_png"
  fi
}

capture_macos() {
  local viewer_wait=$1
  local addr=$2
  local viewer_log=$3
  local xvfb_log=$4
  local root_png=$5
  local window_png=$6
  local window_line_txt=$7
  local window_geom_txt=$8

  echo "macOS mode: no Xvfb required" > "$xvfb_log"

  echo "+ env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- $addr > $viewer_log"
  env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- "$addr" >"$viewer_log" 2>&1 &
  VIEWER_PID=$!

  local window_info
  window_info=$(wait_macos_window_info || true)

  sleep "$viewer_wait"
  run screencapture -x "$root_png"

  if [[ -z "$window_info" ]]; then
    echo "window info unavailable (macOS accessibility may be restricted); using full-screen fallback" > "$window_line_txt"
    cp "$root_png" "$window_png"
    echo "unknown" > "$window_geom_txt"
    return 0
  fi

  IFS='|' read -r process_name window_x window_y window_w window_h window_id <<<"$window_info"
  echo "${process_name} Agent World Viewer ${window_w}x${window_h}+${window_x}+${window_y}" > "$window_line_txt"
  echo "${window_w}x${window_h}+${window_x}+${window_y}" > "$window_geom_txt"

  if [[ -n "${window_id:-}" ]]; then
    run screencapture -x -l "$window_id" "$window_png"
  else
    run screencapture -x -R "${window_x},${window_y},${window_w},${window_h}" "$window_png"
  fi
}

scenario="llm_bootstrap"
addr="127.0.0.1:5023"
tick_ms="300"
display=":100"
width="1280"
height="800"
viewer_wait="8"
enable_llm=0
keep_tmp=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --scenario)
      scenario=${2:-}
      shift 2
      ;;
    --addr)
      addr=${2:-}
      shift 2
      ;;
    --tick-ms)
      tick_ms=${2:-}
      shift 2
      ;;
    --display)
      display=${2:-}
      shift 2
      ;;
    --width)
      width=${2:-}
      shift 2
      ;;
    --height)
      height=${2:-}
      shift 2
      ;;
    --viewer-wait)
      viewer_wait=${2:-}
      shift 2
      ;;
    --llm)
      enable_llm=1
      shift
      ;;
    --keep-tmp)
      keep_tmp=1
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

platform=$(detect_platform)
if [[ "$platform" == "unsupported" ]]; then
  echo "unsupported platform: $(uname -s)" >&2
  exit 1
fi

require_cmd cargo
if [[ "$platform" == "linux" ]]; then
  require_cmd Xvfb
  require_cmd xwininfo
  require_cmd ffmpeg
else
  require_cmd screencapture
  require_cmd osascript
fi

if [[ $keep_tmp -eq 0 ]]; then
  run rm -rf .tmp
fi

out_dir=".tmp/screens"
run mkdir -p "$out_dir"

server_log="$out_dir/live_server.log"
viewer_log="$out_dir/viewer.log"
xvfb_log="$out_dir/xvfb.log"
root_png="$out_dir/root.png"
window_png="$out_dir/window.png"
window_line_txt="$out_dir/window_line.txt"
window_geom_txt="$out_dir/window_geom.txt"

cleanup() {
  local pid
  for pid in "${VIEWER_PID:-}" "${SERVER_PID:-}" "${XVFB_PID:-}"; do
    if [[ -n "${pid:-}" ]] && kill -0 "$pid" 2>/dev/null; then
      kill "$pid" >/dev/null 2>&1 || true
      wait "$pid" >/dev/null 2>&1 || true
    fi
  done
}
trap cleanup EXIT

server_cmd=(env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- "$scenario" --bind "$addr" --tick-ms "$tick_ms")
if [[ $enable_llm -eq 1 ]]; then
  server_cmd+=(--llm)
fi

echo "+ ${server_cmd[*]} > $server_log"
"${server_cmd[@]}" >"$server_log" 2>&1 &
SERVER_PID=$!

if [[ "$platform" == "linux" ]]; then
  capture_linux "$display" "$width" "$height" "$viewer_wait" "$addr" "$viewer_log" "$xvfb_log" "$root_png" "$window_png" "$window_line_txt" "$window_geom_txt"
else
  capture_macos "$viewer_wait" "$addr" "$viewer_log" "$xvfb_log" "$root_png" "$window_png" "$window_line_txt" "$window_geom_txt"
fi

echo "capture complete"
echo "  mode:   $platform"
echo "  root:   $root_png"
echo "  window: $window_png"
echo "  logs:   $server_log, $viewer_log"
