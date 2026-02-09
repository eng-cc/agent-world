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
  - macOS: uses Bevy internal screenshot (no system screen-recording permission)

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


VALID_SCENARIOS=(
  minimal
  two_bases
  llm_bootstrap
  power_bootstrap
  resource_bootstrap
  twin_region_bootstrap
  triad_region_bootstrap
  triad_p2p_bootstrap
  asteroid_fragment_bootstrap
  asteroid_fragment_twin_region_bootstrap
  asteroid_fragment_triad_region_bootstrap
)

normalize_scenario_alias() {
  case "$1" in
    triad)
      echo "triad_region_bootstrap"
      ;;
    triad_p2p)
      echo "triad_p2p_bootstrap"
      ;;
    twin|twin_region)
      echo "twin_region_bootstrap"
      ;;
    asteroid_fragment)
      echo "asteroid_fragment_bootstrap"
      ;;
    asteroid_fragment_twin)
      echo "asteroid_fragment_twin_region_bootstrap"
      ;;
    asteroid_fragment_triad)
      echo "asteroid_fragment_triad_region_bootstrap"
      ;;
    *)
      echo "$1"
      ;;
  esac
}

is_valid_scenario() {
  local target=$1
  local item
  for item in "${VALID_SCENARIOS[@]}"; do
    if [[ "$item" == "$target" ]]; then
      return 0
    fi
  done
  return 1
}

scenario_list_csv() {
  local first=1
  local item
  for item in "${VALID_SCENARIOS[@]}"; do
    if [[ $first -eq 1 ]]; then
      printf "%s" "$item"
      first=0
    else
      printf ", %s" "$item"
    fi
  done
  printf "\n"
}

validate_scenario_or_exit() {
  local raw=$1
  local normalized
  normalized=$(normalize_scenario_alias "$raw")

  if [[ "$normalized" != "$raw" ]]; then
    echo "scenario alias mapped: $raw -> $normalized" >&2
  fi

  if is_valid_scenario "$normalized"; then
    echo "$normalized"
    return 0
  fi

  echo "invalid scenario: $raw" >&2
  echo "supported scenarios: $(scenario_list_csv)" >&2
  echo "common aliases: triad, triad_p2p, twin, asteroid_fragment" >&2
  exit 2
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

wait_for_file() {
  local path=$1
  local timeout_secs=$2
  local steps=$((timeout_secs * 2))
  if [[ "$steps" -lt 1 ]]; then
    steps=1
  fi
  for _ in $(seq 1 "$steps"); do
    if [[ -s "$path" ]]; then
      return 0
    fi
    sleep 0.5
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

  local viewer_wait_int=${viewer_wait%.*}
  if [[ -z "$viewer_wait_int" ]]; then
    viewer_wait_int=8
  fi
  local capture_max_wait=$((viewer_wait_int + 20))

  echo "macOS mode: Bevy internal screenshot (no Xvfb)" > "$xvfb_log"
  echo "bevy_internal_capture Agent World Viewer" > "$window_line_txt"
  echo "internal" > "$window_geom_txt"

  echo "+ AGENT_WORLD_VIEWER_CAPTURE_PATH=$window_png env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- $addr > $viewer_log"
  AGENT_WORLD_VIEWER_CAPTURE_PATH="$window_png" \
  AGENT_WORLD_VIEWER_CAPTURE_DELAY_SECS="$viewer_wait" \
  AGENT_WORLD_VIEWER_CAPTURE_MAX_WAIT_SECS="$capture_max_wait" \
  env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- "$addr" >"$viewer_log" 2>&1 &
  VIEWER_PID=$!

  if ! wait_for_file "$window_png" "$capture_max_wait"; then
    echo "failed to generate internal viewer capture: $window_png" >&2
    exit 3
  fi

  cp "$window_png" "$root_png"

  # viewer should exit automatically after capture; tolerate delayed termination
  wait "$VIEWER_PID" >/dev/null 2>&1 || true
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

scenario=$(validate_scenario_or_exit "$scenario")

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
