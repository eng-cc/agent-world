#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

usage() {
  cat <<'USAGE'
Usage: ./scripts/viewer-owr4-stress.sh [options]

Options:
  --duration-secs <n>      duration per scenario in seconds (default: 45)
  --tick-ms <n>            world_viewer_live tick interval (default: 120)
  --base-port <n>          first port for scenario runs (default: 5420)
  --out-dir <path>         output root directory (default: .tmp/viewer_owr4_stress)
  --event-window-size <n>  viewer event window cap for stress capture (default: 50000)
  --scenarios <csv>        scenario list, comma separated (default: triad_region_bootstrap,llm_bootstrap)
  --no-prewarm             skip cargo build prewarm
  -h, --help               show help

Behavior:
  - Scenario "llm_bootstrap" will be launched with --llm automatically.
  - viewer runs in headless-online mode and reports event counters to logs.
  - output includes per-scenario logs + CSV + Markdown summary.
USAGE
}

run() {
  echo "+ $*"
  "$@"
}

duration_secs=45
tick_ms=120
base_port=5420
out_root=".tmp/viewer_owr4_stress"
event_window_size=50000
scenarios_csv="triad_region_bootstrap,llm_bootstrap"
prewarm=1

while [[ $# -gt 0 ]]; do
  case "$1" in
    --duration-secs)
      duration_secs=${2:-}
      shift 2
      ;;
    --tick-ms)
      tick_ms=${2:-}
      shift 2
      ;;
    --base-port)
      base_port=${2:-}
      shift 2
      ;;
    --out-dir)
      out_root=${2:-}
      shift 2
      ;;
    --event-window-size)
      event_window_size=${2:-}
      shift 2
      ;;
    --scenarios)
      scenarios_csv=${2:-}
      shift 2
      ;;
    --no-prewarm)
      prewarm=0
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

for value_name in duration_secs tick_ms base_port event_window_size; do
  value=${!value_name}
  if [[ ! "$value" =~ ^[0-9]+$ ]] || [[ "$value" -le 0 ]]; then
    echo "invalid $value_name: $value" >&2
    exit 1
  fi
done

if [[ -z "$scenarios_csv" ]]; then
  echo "scenarios list cannot be empty" >&2
  exit 1
fi

if [[ "$prewarm" == "1" ]]; then
  run env -u RUSTC_WRAPPER cargo build -p agent_world --bin world_viewer_live
  run env -u RUSTC_WRAPPER cargo build -p agent_world_viewer
fi

timestamp=$(date +%Y%m%d-%H%M%S)
run_dir="$out_root/$timestamp"
run mkdir -p "$run_dir"

summary_csv="$run_dir/metrics.csv"
summary_md="$run_dir/summary.md"
echo "scenario,mode,duration_secs,tick_ms,final_events,event_rate,status,server_log,viewer_log" > "$summary_csv"

server_pid=""
viewer_pid=""

cleanup() {
  for pid in "$viewer_pid" "$server_pid"; do
    if [[ -n "$pid" ]] && kill -0 "$pid" >/dev/null 2>&1; then
      kill "$pid" >/dev/null 2>&1 || true
      wait "$pid" >/dev/null 2>&1 || true
    fi
  done
  viewer_pid=""
  server_pid=""
}
trap cleanup EXIT

IFS=',' read -r -a scenarios <<< "$scenarios_csv"

index=0
for scenario_raw in "${scenarios[@]}"; do
  scenario=$(echo "$scenario_raw" | xargs)
  if [[ -z "$scenario" ]]; then
    continue
  fi

  port=$((base_port + index))
  addr="127.0.0.1:${port}"
  scenario_dir="$run_dir/$scenario"
  run mkdir -p "$scenario_dir"
  server_log="$scenario_dir/live_server.log"
  viewer_log="$scenario_dir/viewer.log"

  mode="script"
  server_cmd=(env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_viewer_live -- "$scenario" --bind "$addr" --tick-ms "$tick_ms")
  if [[ "$scenario" == "llm_bootstrap" ]]; then
    mode="llm"
    server_cmd+=(--llm)
  fi

  echo "== scenario: $scenario ($mode) =="
  echo "+ ${server_cmd[*]} > $server_log"
  "${server_cmd[@]}" >"$server_log" 2>&1 &
  server_pid=$!
  sleep 2

  viewer_cmd=(env -u RUSTC_WRAPPER cargo run -p agent_world_viewer -- "$addr")
  echo "+ AGENT_WORLD_VIEWER_HEADLESS=1 AGENT_WORLD_VIEWER_FORCE_ONLINE=1 AGENT_WORLD_VIEWER_EVENT_WINDOW_SIZE=$event_window_size ${viewer_cmd[*]} > $viewer_log"
  AGENT_WORLD_VIEWER_HEADLESS=1 \
  AGENT_WORLD_VIEWER_FORCE_ONLINE=1 \
  AGENT_WORLD_VIEWER_EVENT_WINDOW_SIZE="$event_window_size" \
  "${viewer_cmd[@]}" >"$viewer_log" 2>&1 &
  viewer_pid=$!

  sleep "$duration_secs"
  cleanup

  final_events=$(awk '/viewer events:/ {value=$3} END {if (value=="") value=0; print value}' "$viewer_log")
  status=$(awk -F': ' '/viewer status:/ {value=$2} END {if (value=="") value="unknown"; print value}' "$viewer_log")
  event_rate=$(awk -v events="$final_events" -v secs="$duration_secs" 'BEGIN { printf "%.2f", (secs > 0 ? events / secs : 0) }')

  echo "$scenario,$mode,$duration_secs,$tick_ms,$final_events,$event_rate,$status,$server_log,$viewer_log" >> "$summary_csv"
  index=$((index + 1))
done

{
  echo "# OWR4 压测结果摘要"
  echo
  echo "- 运行目录：\`$run_dir\`"
  echo "- 运行时长（每场景）：\`$duration_secs\` 秒"
  echo "- Tick 间隔：\`$tick_ms\` ms"
  echo "- 参考模板：\`doc/world-simulator/viewer-open-world-sandbox-readiness.stress-report.template.md\`"
  echo
  echo "| Scenario | Mode | Duration(s) | Tick(ms) | Final Events | Events/s | Viewer Status |"
  echo "|---|---:|---:|---:|---:|---:|---|"
  tail -n +2 "$summary_csv" | while IFS=',' read -r scenario mode dur tick events rate status _server_log _viewer_log; do
    echo "| $scenario | $mode | $dur | $tick | $events | $rate | $status |"
  done
} > "$summary_md"

echo "stress run completed:"
echo "  csv: $summary_csv"
echo "  md:  $summary_md"
