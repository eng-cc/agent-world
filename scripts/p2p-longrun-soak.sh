#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

usage() {
  cat <<'USAGE'
Usage: ./scripts/p2p-longrun-soak.sh [options]

Options:
  --profile <name>                 soak_smoke | soak_endurance | soak_release (default: soak_smoke)
  --duration-secs <n>              override per-topology soak duration seconds
  --topologies <csv>               comma-separated topologies: triad,triad_distributed
  --scenario <name>                world_viewer_live scenario (default: triad_p2p_bootstrap)
  --llm                            enable LLM mode for world_viewer_live
  --no-llm                         disable LLM mode (default)
  --base-port <n>                  base port for per-topology allocation (default: 5610)
  --bind-host <host>               bind host for gossip/live endpoints (default: 127.0.0.1)
  --out-dir <path>                 output root (default: .tmp/p2p_longrun)
  --startup-timeout-secs <n>       startup grace before monitor loop (default: 20)
  --poll-interval-secs <n>         monitor loop interval (default: 2)
  --chaos-plan <path>              JSON chaos plan for restart/pause injections
  --chaos-continuous-enable        continuously inject chaos events during soak window
  --chaos-continuous-interval-secs <n>
                                    interval seconds between continuous chaos events (default: 30)
  --chaos-continuous-start-sec <n> start second offset for continuous chaos injection (default: 30)
  --chaos-continuous-max-events <n>
                                    max continuous events per topology, 0 = unlimited (default: 0)
  --chaos-continuous-actions <csv> comma-separated actions from restart,pause,disconnect (default: restart,pause)
  --chaos-continuous-seed <n>      deterministic seed for continuous chaos selection (default: unix timestamp)
  --chaos-continuous-restart-down-secs <n>
                                    down seconds for generated restart events (default: 1)
  --chaos-continuous-pause-duration-secs <n>
                                    pause duration seconds for generated pause/disconnect events (default: 2)
  --max-stall-secs <n>             gate threshold for max no-progress window
  --max-lag-p95 <n>                gate threshold for p95(network_height - committed_height)
  --max-distfs-failure-ratio <r>   gate threshold for DistFS failed/total ratio (0~1)
  --no-prewarm                     skip cargo build prewarm
  -h, --help                       show help

Profiles:
  soak_smoke      default duration 1500s, default topologies triad,triad_distributed
  soak_endurance  default duration 10800s, default topologies triad_distributed
  soak_release    default duration 28800s, default topologies triad_distributed

Output:
  <out-dir>/<timestamp>/
    run_config.json
    timeline.csv
    summary.json
    summary.md
    failures.md (only when failed)
    chaos_events.log
    <topology>/nodes/<node_id>/{stdout.log,stderr.log,command.txt,report/}
USAGE
}

run() {
  echo "+ $*"
  "$@"
}

trim() {
  local value=$1
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  printf '%s' "$value"
}

join_by() {
  local sep=$1
  shift || true
  local out=""
  local first=1
  local item
  for item in "$@"; do
    [[ -z "$item" ]] && continue
    if [[ "$first" -eq 1 ]]; then
      out="$item"
      first=0
    else
      out="${out}${sep}${item}"
    fi
  done
  printf '%s' "$out"
}

ensure_positive_int() {
  local flag=$1
  local value=$2
  if [[ ! "$value" =~ ^[0-9]+$ ]] || [[ "$value" -le 0 ]]; then
    echo "invalid $flag: $value" >&2
    exit 2
  fi
}

ensure_non_negative_int() {
  local flag=$1
  local value=$2
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    echo "invalid $flag: $value" >&2
    exit 2
  fi
}

ensure_ratio_between_zero_and_one() {
  local flag=$1
  local value=$2
  if ! awk -v value="$value" 'BEGIN { exit !(value ~ /^([0-9]+(\.[0-9]+)?|\.[0-9]+)$/ && value >= 0.0 && value <= 1.0) }'; then
    echo "invalid $flag: $value (expected 0~1)" >&2
    exit 2
  fi
}

ensure_supported_topology() {
  local value=$1
  case "$value" in
    triad|triad_distributed) ;;
    *)
      echo "unsupported topology: $value (expected triad|triad_distributed)" >&2
      exit 2
      ;;
  esac
}

ensure_supported_chaos_action() {
  local value=$1
  case "$value" in
    restart|pause|disconnect) ;;
    *)
      echo "unsupported chaos action: $value (expected restart|pause|disconnect)" >&2
      exit 2
      ;;
  esac
}

chaos_rng_state=1
chaos_rng_seed() {
  local seed=$1
  local normalized=$((seed % 2147483647))
  if (( normalized <= 0 )); then
    normalized=$((normalized + 2147483646))
  fi
  chaos_rng_state=$normalized
}

chaos_rng_next() {
  chaos_rng_state=$(( (chaos_rng_state * 48271) % 2147483647 ))
  printf '%s' "$chaos_rng_state"
}

profile="soak_smoke"
duration_secs=""
topologies_csv=""
scenario="triad_p2p_bootstrap"
llm_enabled=0
base_port=5610
bind_host="127.0.0.1"
out_root=".tmp/p2p_longrun"
startup_timeout_secs=20
poll_interval_secs=2
chaos_plan_path=""
chaos_continuous_enabled=0
chaos_continuous_interval_secs=30
chaos_continuous_start_sec=30
chaos_continuous_max_events=0
chaos_continuous_actions_csv="restart,pause"
chaos_continuous_seed=""
chaos_continuous_restart_down_secs=1
chaos_continuous_pause_duration_secs=2
max_stall_secs=""
max_lag_p95=""
max_distfs_failure_ratio=""
prewarm=1

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      profile=${2:-}
      shift 2
      ;;
    --duration-secs)
      duration_secs=${2:-}
      shift 2
      ;;
    --topologies)
      topologies_csv=${2:-}
      shift 2
      ;;
    --scenario)
      scenario=${2:-}
      shift 2
      ;;
    --llm)
      llm_enabled=1
      shift
      ;;
    --no-llm)
      llm_enabled=0
      shift
      ;;
    --base-port)
      base_port=${2:-}
      shift 2
      ;;
    --bind-host)
      bind_host=${2:-}
      shift 2
      ;;
    --out-dir)
      out_root=${2:-}
      shift 2
      ;;
    --startup-timeout-secs)
      startup_timeout_secs=${2:-}
      shift 2
      ;;
    --poll-interval-secs)
      poll_interval_secs=${2:-}
      shift 2
      ;;
    --chaos-plan)
      chaos_plan_path=${2:-}
      shift 2
      ;;
    --chaos-continuous-enable)
      chaos_continuous_enabled=1
      shift
      ;;
    --chaos-continuous-interval-secs)
      chaos_continuous_interval_secs=${2:-}
      shift 2
      ;;
    --chaos-continuous-start-sec)
      chaos_continuous_start_sec=${2:-}
      shift 2
      ;;
    --chaos-continuous-max-events)
      chaos_continuous_max_events=${2:-}
      shift 2
      ;;
    --chaos-continuous-actions)
      chaos_continuous_actions_csv=${2:-}
      shift 2
      ;;
    --chaos-continuous-seed)
      chaos_continuous_seed=${2:-}
      shift 2
      ;;
    --chaos-continuous-restart-down-secs)
      chaos_continuous_restart_down_secs=${2:-}
      shift 2
      ;;
    --chaos-continuous-pause-duration-secs)
      chaos_continuous_pause_duration_secs=${2:-}
      shift 2
      ;;
    --max-stall-secs)
      max_stall_secs=${2:-}
      shift 2
      ;;
    --max-lag-p95)
      max_lag_p95=${2:-}
      shift 2
      ;;
    --max-distfs-failure-ratio)
      max_distfs_failure_ratio=${2:-}
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
      exit 2
      ;;
  esac
done

case "$profile" in
  soak_smoke)
    default_duration_secs=1500
    default_topologies_csv="triad,triad_distributed"
    default_max_stall_secs=300
    default_max_lag_p95=12
    default_max_distfs_failure_ratio="0.25"
    ;;
  soak_endurance)
    default_duration_secs=10800
    default_topologies_csv="triad_distributed"
    default_max_stall_secs=420
    default_max_lag_p95=8
    default_max_distfs_failure_ratio="0.15"
    ;;
  soak_release)
    default_duration_secs=28800
    default_topologies_csv="triad_distributed"
    default_max_stall_secs=600
    default_max_lag_p95=5
    default_max_distfs_failure_ratio="0.10"
    ;;
  *)
    echo "invalid --profile: $profile (expected soak_smoke|soak_endurance|soak_release)" >&2
    exit 2
    ;;
esac

if [[ -z "$duration_secs" ]]; then
  duration_secs=$default_duration_secs
fi
if [[ -z "$topologies_csv" ]]; then
  topologies_csv=$default_topologies_csv
fi
if [[ -z "$max_stall_secs" ]]; then
  max_stall_secs=$default_max_stall_secs
fi
if [[ -z "$max_lag_p95" ]]; then
  max_lag_p95=$default_max_lag_p95
fi
if [[ -z "$max_distfs_failure_ratio" ]]; then
  max_distfs_failure_ratio=$default_max_distfs_failure_ratio
fi

ensure_positive_int "--duration-secs" "$duration_secs"
ensure_positive_int "--base-port" "$base_port"
ensure_positive_int "--startup-timeout-secs" "$startup_timeout_secs"
ensure_positive_int "--poll-interval-secs" "$poll_interval_secs"
ensure_non_negative_int "--max-stall-secs" "$max_stall_secs"
ensure_non_negative_int "--max-lag-p95" "$max_lag_p95"
ensure_ratio_between_zero_and_one "--max-distfs-failure-ratio" "$max_distfs_failure_ratio"
ensure_non_negative_int "--chaos-continuous-start-sec" "$chaos_continuous_start_sec"
ensure_non_negative_int "--chaos-continuous-max-events" "$chaos_continuous_max_events"
ensure_non_negative_int "--chaos-continuous-restart-down-secs" "$chaos_continuous_restart_down_secs"
ensure_non_negative_int "--chaos-continuous-pause-duration-secs" "$chaos_continuous_pause_duration_secs"
if [[ "$chaos_continuous_enabled" -eq 1 ]]; then
  ensure_positive_int "--chaos-continuous-interval-secs" "$chaos_continuous_interval_secs"
fi

if [[ -z "$scenario" ]]; then
  echo "--scenario cannot be empty" >&2
  exit 2
fi

if [[ -n "$chaos_plan_path" ]]; then
  if [[ ! -f "$chaos_plan_path" ]]; then
    echo "chaos plan file not found: $chaos_plan_path" >&2
    exit 2
  fi
  if ! jq -e '(.events // []) | type == "array"' "$chaos_plan_path" >/dev/null; then
    echo "invalid chaos plan format: expected JSON object with .events array" >&2
    exit 2
  fi
fi

declare -a chaos_continuous_actions=()
if [[ "$chaos_continuous_enabled" -eq 1 ]]; then
  if [[ -z "$chaos_continuous_seed" ]]; then
    chaos_continuous_seed=$(date +%s)
  fi
  ensure_non_negative_int "--chaos-continuous-seed" "$chaos_continuous_seed"

  mapfile -t chaos_continuous_actions < <(printf '%s' "$chaos_continuous_actions_csv" | tr ',' '\n' | sed '/^$/d')
  if (( ${#chaos_continuous_actions[@]} == 0 )); then
    echo "--chaos-continuous-actions resolved to empty list" >&2
    exit 2
  fi
  for i in "${!chaos_continuous_actions[@]}"; do
    chaos_continuous_actions[$i]=$(trim "${chaos_continuous_actions[$i]}")
    ensure_supported_chaos_action "${chaos_continuous_actions[$i]}"
  done
else
  chaos_continuous_seed=0
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required for metrics aggregation but not found in PATH" >&2
  exit 1
fi

if [[ "$prewarm" -eq 1 ]]; then
  run env -u RUSTC_WRAPPER cargo build -p agent_world --bin world_viewer_live
fi

live_bin="$repo_root/target/debug/world_viewer_live"
if [[ ! -x "$live_bin" ]]; then
  echo "world_viewer_live binary not found: $live_bin" >&2
  echo "run with prewarm enabled or build it manually first" >&2
  exit 1
fi

timestamp=$(date +%Y%m%d-%H%M%S)
run_dir="$out_root/$timestamp"
run mkdir -p "$run_dir"

mapfile -t topologies < <(printf '%s' "$topologies_csv" | tr ',' '\n' | sed '/^$/d')
if (( ${#topologies[@]} == 0 )); then
  echo "--topologies resolved to empty list" >&2
  exit 2
fi

for i in "${!topologies[@]}"; do
  topologies[$i]=$(trim "${topologies[$i]}")
  ensure_supported_topology "${topologies[$i]}"
done

run_config_json="$run_dir/run_config.json"
chaos_enabled=0
if [[ -n "$chaos_plan_path" ]] || [[ "$chaos_continuous_enabled" -eq 1 ]]; then
  chaos_enabled=1
fi
{
  echo "{"
  echo "  \"profile\": \"$profile\","
  echo "  \"duration_secs\": $duration_secs,"
  echo "  \"scenario\": \"$scenario\","
  echo "  \"llm_enabled\": $llm_enabled,"
  echo "  \"base_port\": $base_port,"
  echo "  \"bind_host\": \"$bind_host\","
  echo "  \"startup_timeout_secs\": $startup_timeout_secs,"
  echo "  \"poll_interval_secs\": $poll_interval_secs,"
  echo "  \"chaos_enabled\": $chaos_enabled,"
  if [[ -n "$chaos_plan_path" ]]; then
    echo "  \"chaos_plan_path\": \"$chaos_plan_path\","
  else
    echo "  \"chaos_plan_path\": null,"
  fi
  echo "  \"chaos_continuous_enabled\": $chaos_continuous_enabled,"
  echo "  \"chaos_continuous_interval_secs\": $chaos_continuous_interval_secs,"
  echo "  \"chaos_continuous_start_sec\": $chaos_continuous_start_sec,"
  echo "  \"chaos_continuous_max_events\": $chaos_continuous_max_events,"
  echo "  \"chaos_continuous_actions_csv\": \"$chaos_continuous_actions_csv\","
  if [[ "$chaos_continuous_enabled" -eq 1 ]]; then
    echo "  \"chaos_continuous_seed\": $chaos_continuous_seed,"
  else
    echo "  \"chaos_continuous_seed\": null,"
  fi
  echo "  \"chaos_continuous_restart_down_secs\": $chaos_continuous_restart_down_secs,"
  echo "  \"chaos_continuous_pause_duration_secs\": $chaos_continuous_pause_duration_secs,"
  echo "  \"max_stall_secs\": $max_stall_secs,"
  echo "  \"max_lag_p95\": $max_lag_p95,"
  echo "  \"max_distfs_failure_ratio\": $max_distfs_failure_ratio,"
  echo "  \"topologies\": ["
  for i in "${!topologies[@]}"; do
    suffix=","
    if (( i == ${#topologies[@]} - 1 )); then
      suffix=""
    fi
    echo "    \"${topologies[$i]}\"$suffix"
  done
  echo "  ]"
  echo "}"
} > "$run_config_json"

summary_md="$run_dir/summary.md"
timeline_csv="$run_dir/timeline.csv"
summary_json="$run_dir/summary.json"
failures_md="$run_dir/failures.md"
chaos_events_log="$run_dir/chaos_events.log"
topology_summary_ndjson="$run_dir/.topology_summary.ndjson"

{
  echo "# P2P Longrun Soak Summary"
  echo
  echo "- run_dir: \`$run_dir\`"
  echo "- profile: \`$profile\`"
  echo "- duration_secs_per_topology: \`$duration_secs\`"
  echo "- scenario: \`$scenario\`"
  echo "- max_stall_secs: \`$max_stall_secs\`"
  echo "- max_lag_p95: \`$max_lag_p95\`"
  echo "- max_distfs_failure_ratio: \`$max_distfs_failure_ratio\`"
  if [[ -n "$chaos_plan_path" ]]; then
    echo "- chaos_plan: \`$chaos_plan_path\`"
  else
    echo "- chaos_plan: \`disabled\`"
  fi
  if [[ "$chaos_continuous_enabled" -eq 1 ]]; then
    echo "- chaos_continuous: \`enabled\` (interval=${chaos_continuous_interval_secs}s, start=${chaos_continuous_start_sec}s, max_events=${chaos_continuous_max_events}, actions=${chaos_continuous_actions_csv}, seed=${chaos_continuous_seed})"
    echo "- chaos_continuous_durations: \`restart_down=${chaos_continuous_restart_down_secs}s,pause=${chaos_continuous_pause_duration_secs}s\`"
  else
    echo "- chaos_continuous: \`disabled\`"
  fi
  echo
  echo "| topology | status | process_status | metric_gate | reports | started_at | ended_at | notes |"
  echo "|---|---|---|---|---|---|---|---|"
} > "$summary_md"

echo "topology,node,epoch_index,observed_at_unix_ms,committed_height,network_committed_height,lag,total_checks,failed_checks,distfs_failure_ratio,invariant_ok,report_path" > "$timeline_csv"
: > "$topology_summary_ndjson"
echo "timestamp|topology|event_id|phase|action|node|detail" > "$chaos_events_log"

active_cleanup_done=0
declare -a active_pids=()
declare -a active_nodes=()
declare -A node_cmd_file_by_name=()
declare -A node_stdout_log_by_name=()
declare -A node_stderr_log_by_name=()
declare -A chaos_exempt_secs_by_topology=()
declare -A chaos_events_executed_by_topology=()
declare -A chaos_plan_events_executed_by_topology=()
declare -A chaos_continuous_events_executed_by_topology=()

analysis_report_count=0
analysis_gate_status="insufficient_data"
analysis_gate_notes="no_epoch_reports"
analysis_max_stall_secs_observed=0
analysis_lag_p95=0
analysis_distfs_failure_ratio="0.000000"
analysis_distfs_total_checks=0
analysis_distfs_failed_checks=0
analysis_invariant_all_ok=true
analysis_chaos_exempt_secs=0
analysis_effective_max_stall_secs=0

append_summary_row() {
  local topology=$1
  local status=$2
  local process_status=$3
  local metric_gate=$4
  local reports=$5
  local started_at=$6
  local ended_at=$7
  local notes=$8
  echo "| $topology | $status | $process_status | $metric_gate | $reports | $started_at | $ended_at | $notes |" >> "$summary_md"
}

find_node_index_by_name() {
  local target_node=$1
  local idx
  for idx in "${!active_nodes[@]}"; do
    if [[ "${active_nodes[$idx]}" == "$target_node" ]]; then
      printf '%s' "$idx"
      return 0
    fi
  done
  return 1
}

log_chaos_event() {
  local topology=$1
  local event_id=$2
  local phase=$3
  local action=$4
  local node=$5
  local detail=$6
  local ts
  ts=$(date '+%Y-%m-%d %H:%M:%S %Z')
  echo "$ts|$topology|$event_id|$phase|$action|$node|$detail" >> "$chaos_events_log"
}

relaunch_node_from_saved_command() {
  local node_name=$1
  local idx cmd_txt stdout_log stderr_log cmd_line

  if ! idx=$(find_node_index_by_name "$node_name"); then
    echo "node not found for relaunch: $node_name" >&2
    return 1
  fi

  cmd_txt=${node_cmd_file_by_name[$node_name]:-}
  stdout_log=${node_stdout_log_by_name[$node_name]:-}
  stderr_log=${node_stderr_log_by_name[$node_name]:-}
  if [[ -z "$cmd_txt" ]] || [[ -z "$stdout_log" ]] || [[ -z "$stderr_log" ]]; then
    echo "missing node command metadata for relaunch: $node_name" >&2
    return 1
  fi

  cmd_line=$(tr -d '\n' < "$cmd_txt")
  if [[ -z "$cmd_line" ]]; then
    echo "empty command file for relaunch: $cmd_txt" >&2
    return 1
  fi

  echo "+ $cmd_line >> $stdout_log 2>> $stderr_log"
  bash -lc "$cmd_line" >>"$stdout_log" 2>>"$stderr_log" &
  active_pids[$idx]=$!
  return 0
}

execute_chaos_event() {
  local topology=$1
  local event_id=$2
  local action=$3
  local node_name=$4
  local at_sec=$5
  local down_secs=$6
  local duration_secs=$7
  local idx pid

  if ! idx=$(find_node_index_by_name "$node_name"); then
    log_chaos_event "$topology" "$event_id" "failed" "$action" "$node_name" "node_not_found"
    return 1
  fi
  pid=${active_pids[$idx]}

  case "$action" in
    restart)
      log_chaos_event "$topology" "$event_id" "start" "$action" "$node_name" "at_sec=$at_sec,down_secs=$down_secs,pid=$pid"
      if ! kill -0 "$pid" >/dev/null 2>&1; then
        log_chaos_event "$topology" "$event_id" "failed" "$action" "$node_name" "pid_not_alive=$pid"
        return 1
      fi
      kill "$pid" >/dev/null 2>&1 || true
      wait "$pid" >/dev/null 2>&1 || true
      if (( down_secs > 0 )); then
        sleep "$down_secs"
      fi
      if ! relaunch_node_from_saved_command "$node_name"; then
        log_chaos_event "$topology" "$event_id" "failed" "$action" "$node_name" "relaunch_failed"
        return 1
      fi
      log_chaos_event "$topology" "$event_id" "completed" "$action" "$node_name" "new_pid=${active_pids[$idx]}"
      ;;
    pause|disconnect)
      log_chaos_event "$topology" "$event_id" "start" "$action" "$node_name" "at_sec=$at_sec,duration_secs=$duration_secs,pid=$pid"
      if ! kill -0 "$pid" >/dev/null 2>&1; then
        log_chaos_event "$topology" "$event_id" "failed" "$action" "$node_name" "pid_not_alive=$pid"
        return 1
      fi
      if ! kill -STOP "$pid" >/dev/null 2>&1; then
        log_chaos_event "$topology" "$event_id" "failed" "$action" "$node_name" "sigstop_failed"
        return 1
      fi
      if (( duration_secs > 0 )); then
        sleep "$duration_secs"
      fi
      if ! kill -CONT "$pid" >/dev/null 2>&1; then
        log_chaos_event "$topology" "$event_id" "failed" "$action" "$node_name" "sigcont_failed"
        return 1
      fi
      log_chaos_event "$topology" "$event_id" "completed" "$action" "$node_name" "pid=$pid"
      ;;
    *)
      log_chaos_event "$topology" "$event_id" "failed" "$action" "$node_name" "unknown_action"
      return 1
      ;;
  esac

  return 0
}

stop_active_processes() {
  if [[ "$active_cleanup_done" -eq 1 ]]; then
    return 0
  fi
  active_cleanup_done=1

  local idx
  for idx in "${!active_pids[@]}"; do
    local pid=${active_pids[$idx]}
    if kill -0 "$pid" >/dev/null 2>&1; then
      kill "$pid" >/dev/null 2>&1 || true
    fi
  done

  local deadline=$(( $(date +%s) + 8 ))
  while :; do
    local any_alive=0
    local pid
    for pid in "${active_pids[@]}"; do
      if kill -0 "$pid" >/dev/null 2>&1; then
        any_alive=1
        break
      fi
    done
    if [[ "$any_alive" -eq 0 ]]; then
      break
    fi
    if (( $(date +%s) >= deadline )); then
      for pid in "${active_pids[@]}"; do
        if kill -0 "$pid" >/dev/null 2>&1; then
          kill -9 "$pid" >/dev/null 2>&1 || true
        fi
      done
      break
    fi
    sleep 1
  done

  for pid in "${active_pids[@]}"; do
    wait "$pid" >/dev/null 2>&1 || true
  done
}

cleanup_on_exit() {
  stop_active_processes
}
trap cleanup_on_exit EXIT

launch_node() {
  local topology_dir=$1
  local node_name=$2
  shift 2
  local node_dir="$topology_dir/nodes/$node_name"
  local report_dir="$node_dir/report"
  local stdout_log="$node_dir/stdout.log"
  local stderr_log="$node_dir/stderr.log"
  local cmd_txt="$node_dir/command.txt"

  run mkdir -p "$report_dir"

  local -a cmd=(
    "$live_bin"
    "$scenario"
    --no-llm
    --reward-runtime-enable
    --reward-runtime-report-dir "$report_dir"
    "$@"
  )

  if [[ "$llm_enabled" -eq 1 ]]; then
    cmd=("${cmd[@]/--no-llm/--llm}")
  fi

  printf '%q ' "${cmd[@]}" > "$cmd_txt"
  printf '\n' >> "$cmd_txt"

  echo "+ ${cmd[*]} > $stdout_log 2> $stderr_log"
  "${cmd[@]}" >"$stdout_log" 2>"$stderr_log" &
  local pid=$!

  node_cmd_file_by_name["$node_name"]="$cmd_txt"
  node_stdout_log_by_name["$node_name"]="$stdout_log"
  node_stderr_log_by_name["$node_name"]="$stderr_log"
  active_pids+=("$pid")
  active_nodes+=("$node_name")
}

analyze_topology_metrics() {
  local topology=$1
  local topology_dir=$2

  analysis_report_count=0
  analysis_gate_status="insufficient_data"
  analysis_gate_notes="no_epoch_reports"
  analysis_max_stall_secs_observed=0
  analysis_lag_p95=0
  analysis_distfs_failure_ratio="0.000000"
  analysis_distfs_total_checks=0
  analysis_distfs_failed_checks=0
  analysis_invariant_all_ok=true
  analysis_chaos_exempt_secs=${chaos_exempt_secs_by_topology[$topology]:-0}
  analysis_effective_max_stall_secs=$((max_stall_secs + analysis_chaos_exempt_secs))

  local -a report_files=()
  if [[ -d "$topology_dir/nodes" ]]; then
    while IFS= read -r report_path; do
      report_files+=("$report_path")
    done < <(find "$topology_dir/nodes" -mindepth 3 -maxdepth 3 -type f -name 'epoch-*.json' | sort)
  fi

  analysis_report_count=${#report_files[@]}
  if (( analysis_report_count == 0 )); then
    return 0
  fi

  local samples_tsv="$topology_dir/.metric_samples.tsv"
  : > "$samples_tsv"

  declare -A node_total_max=()
  declare -A node_failed_at_max=()
  local invariant_failed=0

  local report_file node_name metrics
  local epoch_index observed_at committed_height network_committed_height total_checks failed_checks invariant_ok
  local lag ratio

  for report_file in "${report_files[@]}"; do
    node_name=$(basename "$(dirname "$(dirname "$report_file")")")

    if ! metrics=$(jq -r '[
      (.settlement_report.epoch_index // .node_snapshot.consensus.epoch // 0),
      (.observed_at_unix_ms // 0),
      (.node_snapshot.consensus.committed_height // 0),
      (.node_snapshot.consensus.network_committed_height // 0),
      (.distfs_challenge_report.total_checks // 0),
      (.distfs_challenge_report.failed_checks // 0),
      ((.reward_asset_invariant_status.ok // false) | tostring)
    ] | @tsv' "$report_file"); then
      echo "warning: failed to parse report JSON: $report_file" >&2
      continue
    fi

    if [[ -z "$metrics" ]]; then
      continue
    fi

    IFS=$'\t' read -r epoch_index observed_at committed_height network_committed_height total_checks failed_checks invariant_ok <<< "$metrics"

    [[ "$epoch_index" =~ ^-?[0-9]+$ ]] || epoch_index=0
    [[ "$observed_at" =~ ^-?[0-9]+$ ]] || observed_at=0
    [[ "$committed_height" =~ ^-?[0-9]+$ ]] || committed_height=0
    [[ "$network_committed_height" =~ ^-?[0-9]+$ ]] || network_committed_height=0
    [[ "$total_checks" =~ ^-?[0-9]+$ ]] || total_checks=0
    [[ "$failed_checks" =~ ^-?[0-9]+$ ]] || failed_checks=0

    if (( total_checks < 0 )); then
      total_checks=0
    fi
    if (( failed_checks < 0 )); then
      failed_checks=0
    fi

    lag=$((network_committed_height - committed_height))
    if (( lag < 0 )); then
      lag=0
    fi

    ratio=$(awk -v failed="$failed_checks" -v total="$total_checks" 'BEGIN { if (total > 0) printf "%.6f", failed / total; else printf "0.000000"; }')

    printf '"%s","%s",%s,%s,%s,%s,%s,%s,%s,%s,%s,"%s"\n' \
      "$topology" "$node_name" "$epoch_index" "$observed_at" "$committed_height" "$network_committed_height" "$lag" "$total_checks" "$failed_checks" "$ratio" "$invariant_ok" "$report_file" >> "$timeline_csv"

    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
      "$observed_at" "$committed_height" "$lag" "$total_checks" "$failed_checks" "$invariant_ok" "$node_name" >> "$samples_tsv"

    local prev_total=${node_total_max[$node_name]:--1}
    if (( total_checks > prev_total )); then
      node_total_max["$node_name"]=$total_checks
      node_failed_at_max["$node_name"]=$failed_checks
    elif (( total_checks == prev_total )); then
      local prev_failed=${node_failed_at_max[$node_name]:-0}
      if (( failed_checks > prev_failed )); then
        node_failed_at_max["$node_name"]=$failed_checks
      fi
    fi

    if [[ "$invariant_ok" != "true" ]]; then
      invariant_failed=1
    fi
  done

  local node
  for node in "${!node_total_max[@]}"; do
    analysis_distfs_total_checks=$((analysis_distfs_total_checks + node_total_max[$node]))
    analysis_distfs_failed_checks=$((analysis_distfs_failed_checks + ${node_failed_at_max[$node]:-0}))
  done
  analysis_distfs_failure_ratio=$(awk -v failed="$analysis_distfs_failed_checks" -v total="$analysis_distfs_total_checks" 'BEGIN { if (total > 0) printf "%.6f", failed / total; else printf "0.000000"; }')

  local sorted_samples="$topology_dir/.metric_samples.sorted.tsv"
  sort -n -k1,1 "$samples_tsv" > "$sorted_samples"

  local best_height=-1
  local last_progress_ms=0
  local max_stall_ms=0
  local sample_with_time=0
  local sample_observed sample_committed _
  while IFS=$'\t' read -r sample_observed sample_committed _; do
    [[ "$sample_observed" =~ ^-?[0-9]+$ ]] || continue
    [[ "$sample_committed" =~ ^-?[0-9]+$ ]] || continue
    if (( sample_observed <= 0 )); then
      continue
    fi
    sample_with_time=$((sample_with_time + 1))
    if (( best_height < 0 )); then
      best_height=$sample_committed
      last_progress_ms=$sample_observed
    elif (( sample_committed > best_height )); then
      best_height=$sample_committed
      last_progress_ms=$sample_observed
    fi
    local stall_gap=$((sample_observed - last_progress_ms))
    if (( stall_gap > max_stall_ms )); then
      max_stall_ms=$stall_gap
    fi
  done < "$sorted_samples"
  analysis_max_stall_secs_observed=$((max_stall_ms / 1000))

  local lag_values="$topology_dir/.metric_lags.txt"
  cut -f3 "$samples_tsv" | sed '/^[[:space:]]*$/d' | sort -n > "$lag_values"
  local lag_count
  lag_count=$(wc -l < "$lag_values" | tr -d ' ')
  if [[ -z "$lag_count" ]]; then
    lag_count=0
  fi
  if (( lag_count > 0 )); then
    local p95_rank=$(( (95 * lag_count + 99) / 100 ))
    if (( p95_rank < 1 )); then
      p95_rank=1
    fi
    analysis_lag_p95=$(sed -n "${p95_rank}p" "$lag_values")
    [[ "$analysis_lag_p95" =~ ^-?[0-9]+$ ]] || analysis_lag_p95=0
  else
    analysis_lag_p95=0
  fi

  analysis_invariant_all_ok=true
  if (( invariant_failed == 1 )); then
    analysis_invariant_all_ok=false
  fi

  local -a gate_failures=()
  local -a gate_warnings=()

  if (( sample_with_time == 0 )); then
    gate_warnings+=("observed_at_missing")
  elif (( analysis_max_stall_secs_observed > analysis_effective_max_stall_secs )); then
    gate_failures+=("stall=${analysis_max_stall_secs_observed}s>max_${analysis_effective_max_stall_secs}s")
  fi

  if (( analysis_lag_p95 > max_lag_p95 )); then
    gate_failures+=("lag_p95=${analysis_lag_p95}>max_${max_lag_p95}")
  fi

  if (( analysis_distfs_total_checks > 0 )); then
    local ratio_exceeded
    ratio_exceeded=$(awk -v ratio="$analysis_distfs_failure_ratio" -v max="$max_distfs_failure_ratio" 'BEGIN { if (ratio > max) print 1; else print 0; }')
    if [[ "$ratio_exceeded" == "1" ]]; then
      gate_failures+=("distfs_failure_ratio=${analysis_distfs_failure_ratio}>max_${max_distfs_failure_ratio}")
    fi
  else
    gate_warnings+=("distfs_checks=0")
  fi

  if [[ "$analysis_invariant_all_ok" != "true" ]]; then
    gate_failures+=("reward_asset_invariant_not_ok")
  fi

  if (( ${#gate_failures[@]} > 0 )); then
    analysis_gate_status="fail"
    analysis_gate_notes=$(join_by "; " "${gate_failures[@]}")
    if (( ${#gate_warnings[@]} > 0 )); then
      analysis_gate_notes="${analysis_gate_notes}; $(join_by "; " "${gate_warnings[@]}")"
    fi
  elif (( ${#gate_warnings[@]} > 0 )); then
    analysis_gate_status="insufficient_data"
    analysis_gate_notes=$(join_by "; " "${gate_warnings[@]}")
  else
    analysis_gate_status="pass"
    analysis_gate_notes="all_gates_passed"
  fi
}

run_topology() {
  local topology=$1
  local index=$2

  local started_at ended_at status notes
  started_at=$(date '+%Y-%m-%d %H:%M:%S %Z')
  status="ok"
  notes="-"

  active_cleanup_done=0
  active_pids=()
  active_nodes=()
  node_cmd_file_by_name=()
  node_stdout_log_by_name=()
  node_stderr_log_by_name=()

  local topology_dir="$run_dir/$topology"
  run mkdir -p "$topology_dir/nodes"

  local case_base_port=$((base_port + index * 100))
  local -a chaos_event_ids=()
  local -a chaos_event_at_secs=()
  local -a chaos_event_nodes=()
  local -a chaos_event_actions=()
  local -a chaos_event_down_secs=()
  local -a chaos_event_duration_secs=()
  local -a chaos_event_done=()
  local continuous_enabled=0
  local continuous_next_at_sec=0
  local continuous_generated=0

  case "$topology" in
    triad)
      launch_node "$topology_dir" "triad" \
        --topology triad \
        --bind "$bind_host:$((case_base_port + 10))"
      ;;
    triad_distributed)
      local seq_gossip="$bind_host:$((case_base_port + 1))"
      local storage_gossip="$bind_host:$((case_base_port + 2))"
      local observer_gossip="$bind_host:$((case_base_port + 3))"
      local node_id_base="soak-${timestamp}-${index}"

      launch_node "$topology_dir" "sequencer" \
        --topology triad_distributed \
        --node-id "$node_id_base" \
        --node-role sequencer \
        --triad-sequencer-gossip "$seq_gossip" \
        --bind "$bind_host:$((case_base_port + 11))"

      launch_node "$topology_dir" "storage" \
        --topology triad_distributed \
        --node-id "$node_id_base" \
        --node-role storage \
        --triad-sequencer-gossip "$seq_gossip" \
        --triad-storage-gossip "$storage_gossip" \
        --bind "$bind_host:$((case_base_port + 12))"

      launch_node "$topology_dir" "observer" \
        --topology triad_distributed \
        --node-id "$node_id_base" \
        --node-role observer \
        --triad-sequencer-gossip "$seq_gossip" \
        --triad-observer-gossip "$observer_gossip" \
        --bind "$bind_host:$((case_base_port + 13))"
      ;;
  esac

  if [[ -n "$chaos_plan_path" ]]; then
    local event_id at_sec node action down_secs event_duration_secs_raw
    while IFS=$'\t' read -r event_id at_sec node action down_secs event_duration_secs_raw; do
      [[ -z "$event_id" ]] && continue
      [[ "$at_sec" =~ ^[0-9]+$ ]] || at_sec=0
      [[ "$down_secs" =~ ^[0-9]+$ ]] || down_secs=0
      [[ "$event_duration_secs_raw" =~ ^[0-9]+$ ]] || event_duration_secs_raw=0

      if [[ -z "$node" ]]; then
        log_chaos_event "$topology" "$event_id" "skipped" "$action" "none" "missing_node"
        continue
      fi

      case "$action" in
        restart|pause|disconnect) ;;
        "")
          action="restart"
          ;;
        *)
          log_chaos_event "$topology" "$event_id" "failed" "$action" "$node" "unsupported_action"
          status="chaos_plan_invalid"
          notes="invalid chaos action for event=$event_id"
          break
          ;;
      esac

      chaos_event_ids+=("$event_id")
      chaos_event_at_secs+=("$at_sec")
      chaos_event_nodes+=("$node")
      chaos_event_actions+=("$action")
      chaos_event_down_secs+=("$down_secs")
      chaos_event_duration_secs+=("$event_duration_secs_raw")
      chaos_event_done+=("0")
    done < <(
      jq -r --arg topology "$topology" '
        (.events // [] | to_entries[]) as $entry
        | ($entry.value) as $event
        | ($event.topology // "all") as $event_topology
        | select($event_topology == "all" or $event_topology == $topology)
        | [
            ($event.id // ("event-" + ($entry.key | tostring))),
            ($event.at_sec // 0),
            ($event.node // ""),
            ($event.action // "restart"),
            ($event.down_secs // 0),
            ($event.duration_secs // 0)
          ] | @tsv
      ' "$chaos_plan_path"
    )
  fi

  if [[ "$chaos_continuous_enabled" -eq 1 ]]; then
    continuous_enabled=1
    continuous_next_at_sec=$chaos_continuous_start_sec
    chaos_rng_seed $((chaos_continuous_seed + (index + 1) * 104729))
  fi

  if [[ "$status" == "ok" ]]; then
    local startup_deadline=$(( $(date +%s) + startup_timeout_secs ))
    while :; do
      local all_alive=1
      local idx
      for idx in "${!active_pids[@]}"; do
        if ! kill -0 "${active_pids[$idx]}" >/dev/null 2>&1; then
          all_alive=0
          status="startup_failed"
          notes="node=${active_nodes[$idx]} exited before startup window"
          break
        fi
      done
      if [[ "$all_alive" -eq 1 ]]; then
        break
      fi
      if (( $(date +%s) >= startup_deadline )); then
        break
      fi
      sleep 1
    done
  fi

  if [[ "$status" == "ok" ]]; then
    local started_epoch_sec
    started_epoch_sec=$(date +%s)
    local deadline=$(( started_epoch_sec + duration_secs ))
    while (( $(date +%s) < deadline )); do
      local now_sec elapsed_sec
      now_sec=$(date +%s)
      elapsed_sec=$((now_sec - started_epoch_sec))

      local event_idx
      for event_idx in "${!chaos_event_ids[@]}"; do
        if [[ "${chaos_event_done[$event_idx]}" == "1" ]]; then
          continue
        fi
        local event_at=${chaos_event_at_secs[$event_idx]}
        if (( elapsed_sec < event_at )); then
          continue
        fi

        local event_id=${chaos_event_ids[$event_idx]}
        local event_node=${chaos_event_nodes[$event_idx]}
        local event_action=${chaos_event_actions[$event_idx]}
        local event_down_secs=${chaos_event_down_secs[$event_idx]}
        local event_duration_secs=${chaos_event_duration_secs[$event_idx]}

        chaos_event_done[$event_idx]="1"
        if ! execute_chaos_event "$topology" "$event_id" "$event_action" "$event_node" "$event_at" "$event_down_secs" "$event_duration_secs"; then
          status="chaos_failed"
          notes="chaos_event=${event_id} failed"
          break 2
        fi

        local exempt_secs=0
        if [[ "$event_action" == "restart" ]]; then
          exempt_secs=$event_down_secs
        else
          exempt_secs=$event_duration_secs
        fi
        chaos_exempt_secs_by_topology["$topology"]=$(( ${chaos_exempt_secs_by_topology[$topology]:-0} + exempt_secs ))
        chaos_events_executed_by_topology["$topology"]=$(( ${chaos_events_executed_by_topology[$topology]:-0} + 1 ))
        chaos_plan_events_executed_by_topology["$topology"]=$(( ${chaos_plan_events_executed_by_topology[$topology]:-0} + 1 ))
      done

      if [[ "$continuous_enabled" -eq 1 ]]; then
        while (( elapsed_sec >= continuous_next_at_sec )); do
          if (( chaos_continuous_max_events > 0 && continuous_generated >= chaos_continuous_max_events )); then
            continuous_enabled=0
            break
          fi

          local rand node_idx action_idx
          local generated_event_id generated_node generated_action
          local generated_down_secs generated_duration_secs

          rand=$(chaos_rng_next)
          node_idx=$((rand % ${#active_nodes[@]}))
          generated_node=${active_nodes[$node_idx]}

          rand=$(chaos_rng_next)
          action_idx=$((rand % ${#chaos_continuous_actions[@]}))
          generated_action=${chaos_continuous_actions[$action_idx]}

          generated_down_secs=0
          generated_duration_secs=0
          if [[ "$generated_action" == "restart" ]]; then
            generated_down_secs=$chaos_continuous_restart_down_secs
          else
            generated_duration_secs=$chaos_continuous_pause_duration_secs
          fi

          generated_event_id="continuous-${topology}-${continuous_generated}"
          if ! execute_chaos_event "$topology" "$generated_event_id" "$generated_action" "$generated_node" "$continuous_next_at_sec" "$generated_down_secs" "$generated_duration_secs"; then
            status="chaos_failed"
            notes="chaos_event=${generated_event_id} failed"
            break 2
          fi

          local generated_exempt_secs=0
          if [[ "$generated_action" == "restart" ]]; then
            generated_exempt_secs=$generated_down_secs
          else
            generated_exempt_secs=$generated_duration_secs
          fi
          chaos_exempt_secs_by_topology["$topology"]=$(( ${chaos_exempt_secs_by_topology[$topology]:-0} + generated_exempt_secs ))
          chaos_events_executed_by_topology["$topology"]=$(( ${chaos_events_executed_by_topology[$topology]:-0} + 1 ))
          chaos_continuous_events_executed_by_topology["$topology"]=$(( ${chaos_continuous_events_executed_by_topology[$topology]:-0} + 1 ))

          continuous_generated=$((continuous_generated + 1))
          continuous_next_at_sec=$((continuous_next_at_sec + chaos_continuous_interval_secs))
        done
      fi

      local idx
      for idx in "${!active_pids[@]}"; do
        if ! kill -0 "${active_pids[$idx]}" >/dev/null 2>&1; then
          status="process_exit"
          notes="node=${active_nodes[$idx]} exited during soak"
          break 2
        fi
      done
      sleep "$poll_interval_secs"
    done
  fi

  stop_active_processes

  local process_status=$status
  analyze_topology_metrics "$topology" "$topology_dir"

  local final_status=$status
  local -a notes_parts=()
  if [[ "$notes" != "-" ]]; then
    notes_parts+=("$notes")
  fi
  local chaos_event_count=${chaos_events_executed_by_topology[$topology]:-0}
  local chaos_plan_event_count=${chaos_plan_events_executed_by_topology[$topology]:-0}
  local chaos_continuous_event_count=${chaos_continuous_events_executed_by_topology[$topology]:-0}
  local chaos_exempt_secs=${chaos_exempt_secs_by_topology[$topology]:-0}
  if (( chaos_event_count > 0 )); then
    notes_parts+=("chaos_events=${chaos_event_count}")
    notes_parts+=("chaos_plan_events=${chaos_plan_event_count}")
    notes_parts+=("chaos_continuous_events=${chaos_continuous_event_count}")
    notes_parts+=("chaos_exempt_secs=${chaos_exempt_secs}")
  fi

  if [[ "$analysis_gate_status" == "fail" ]]; then
    notes_parts+=("metric_gate=$analysis_gate_notes")
    if [[ "$final_status" == "ok" ]]; then
      final_status="metric_gate_failed"
    fi
  elif [[ "$analysis_gate_status" == "insufficient_data" ]]; then
    notes_parts+=("metric_data=$analysis_gate_notes")
    if [[ "$profile" != "soak_smoke" ]] && [[ "$final_status" == "ok" ]]; then
      notes_parts+=("profile_${profile}_requires_metrics")
      final_status="metric_gate_failed"
    fi
  fi

  if (( ${#notes_parts[@]} == 0 )); then
    notes="-"
  else
    notes=$(join_by "; " "${notes_parts[@]}")
  fi

  ended_at=$(date '+%Y-%m-%d %H:%M:%S %Z')
  status=$final_status

  append_summary_row "$topology" "$status" "$process_status" "$analysis_gate_status" "$analysis_report_count" "$started_at" "$ended_at" "$notes"

  jq -n \
    --arg topology "$topology" \
    --arg status "$status" \
    --arg process_status "$process_status" \
    --arg started_at "$started_at" \
    --arg ended_at "$ended_at" \
    --arg notes "$notes" \
    --arg gate_status "$analysis_gate_status" \
    --arg gate_notes "$analysis_gate_notes" \
    --argjson report_samples "$analysis_report_count" \
    --argjson chaos_events "$chaos_event_count" \
    --argjson chaos_plan_events "$chaos_plan_event_count" \
    --argjson chaos_continuous_events "$chaos_continuous_event_count" \
    --argjson max_stall_secs_observed "$analysis_max_stall_secs_observed" \
    --argjson chaos_exempt_secs "$analysis_chaos_exempt_secs" \
    --argjson effective_max_stall_secs "$analysis_effective_max_stall_secs" \
    --argjson lag_p95 "$analysis_lag_p95" \
    --argjson distfs_failure_ratio "$analysis_distfs_failure_ratio" \
    --argjson distfs_total_checks "$analysis_distfs_total_checks" \
    --argjson distfs_failed_checks "$analysis_distfs_failed_checks" \
    --argjson invariant_all_ok "$analysis_invariant_all_ok" \
    '{
      topology: $topology,
      status: $status,
      process_status: $process_status,
      started_at: $started_at,
      ended_at: $ended_at,
      notes: $notes,
      report_samples: $report_samples,
      chaos_events: $chaos_events,
      chaos_plan_events: $chaos_plan_events,
      chaos_continuous_events: $chaos_continuous_events,
      metric_gate: {
        status: $gate_status,
        notes: $gate_notes
      },
      metrics: {
        max_stall_secs_observed: $max_stall_secs_observed,
        chaos_exempt_secs: $chaos_exempt_secs,
        effective_max_stall_secs: $effective_max_stall_secs,
        lag_p95: $lag_p95,
        distfs_failure_ratio: $distfs_failure_ratio,
        distfs_total_checks: $distfs_total_checks,
        distfs_failed_checks: $distfs_failed_checks,
        invariant_all_ok: $invariant_all_ok
      }
    }' >> "$topology_summary_ndjson"

  if [[ "$status" != "ok" ]]; then
    echo "topology run failed: $topology ($notes)" >&2
    return 1
  fi
  return 0
}

write_summary_json() {
  local overall_status_code=$1
  local generated_at
  generated_at=$(date -u '+%Y-%m-%dT%H:%M:%SZ')

  jq -s \
    --arg generated_at "$generated_at" \
    --arg run_dir "$run_dir" \
    --arg profile "$profile" \
    --arg scenario "$scenario" \
    --arg chaos_plan_path "${chaos_plan_path:-}" \
    --arg chaos_continuous_actions_csv "$chaos_continuous_actions_csv" \
    --argjson duration_secs "$duration_secs" \
    --argjson max_stall_secs "$max_stall_secs" \
    --argjson max_lag_p95 "$max_lag_p95" \
    --argjson max_distfs_failure_ratio "$max_distfs_failure_ratio" \
    --argjson chaos_continuous_enabled "$chaos_continuous_enabled" \
    --argjson chaos_continuous_interval_secs "$chaos_continuous_interval_secs" \
    --argjson chaos_continuous_start_sec "$chaos_continuous_start_sec" \
    --argjson chaos_continuous_max_events "$chaos_continuous_max_events" \
    --argjson chaos_continuous_seed "$chaos_continuous_seed" \
    --argjson chaos_continuous_restart_down_secs "$chaos_continuous_restart_down_secs" \
    --argjson chaos_continuous_pause_duration_secs "$chaos_continuous_pause_duration_secs" \
    --arg timeline_csv "$timeline_csv" \
    --arg summary_md "$summary_md" \
    --arg chaos_events_log "$chaos_events_log" \
    --arg run_config_json "$run_config_json" \
    --argjson overall_status_code "$overall_status_code" \
    '
      . as $topologies |
      {
        generated_at_utc: $generated_at,
        run_dir: $run_dir,
        profile: $profile,
        scenario: $scenario,
        chaos_plan: (if $chaos_plan_path == "" then null else $chaos_plan_path end),
        chaos_continuous: {
          enabled: ($chaos_continuous_enabled == 1),
          interval_secs: $chaos_continuous_interval_secs,
          start_sec: $chaos_continuous_start_sec,
          max_events: $chaos_continuous_max_events,
          actions_csv: $chaos_continuous_actions_csv,
          seed: (if $chaos_continuous_enabled == 1 then $chaos_continuous_seed else null end),
          restart_down_secs: $chaos_continuous_restart_down_secs,
          pause_duration_secs: $chaos_continuous_pause_duration_secs
        },
        duration_secs_per_topology: $duration_secs,
        thresholds: {
          max_stall_secs: $max_stall_secs,
          max_lag_p95: $max_lag_p95,
          max_distfs_failure_ratio: $max_distfs_failure_ratio
        },
        artifacts: {
          run_config_json: $run_config_json,
          timeline_csv: $timeline_csv,
          summary_md: $summary_md,
          chaos_events_log: $chaos_events_log
        },
        totals: {
          topology_count: ($topologies | length),
          topology_ok_count: ($topologies | map(select(.status == "ok")) | length),
          topology_failed_count: ($topologies | map(select(.status != "ok")) | length),
          report_samples_total: ($topologies | map(.report_samples) | add // 0),
          chaos_plan_events_total: ($topologies | map(.chaos_plan_events) | add // 0),
          chaos_continuous_events_total: ($topologies | map(.chaos_continuous_events) | add // 0),
          chaos_events_total: ($topologies | map(.chaos_events) | add // 0)
        },
        gate_failures: (
          $topologies
          | map(select(.metric_gate.status == "fail") | {
              topology,
              reason: .metric_gate.notes
            })
        ),
        topologies: $topologies,
        overall_status: (if $overall_status_code == 0 then "ok" else "failed" end)
      }
    ' "$topology_summary_ndjson" > "$summary_json"
}

append_summary_metrics_section() {
  local topology_count=0
  local topology_ok_count=0
  local topology_failed_count=0
  local report_samples_total=0
  local chaos_plan_events_total=0
  local chaos_continuous_events_total=0
  local chaos_events_total=0

  if [[ -f "$summary_json" ]]; then
    topology_count=$(jq -r '.totals.topology_count // 0' "$summary_json")
    topology_ok_count=$(jq -r '.totals.topology_ok_count // 0' "$summary_json")
    topology_failed_count=$(jq -r '.totals.topology_failed_count // 0' "$summary_json")
    report_samples_total=$(jq -r '.totals.report_samples_total // 0' "$summary_json")
    chaos_plan_events_total=$(jq -r '.totals.chaos_plan_events_total // 0' "$summary_json")
    chaos_continuous_events_total=$(jq -r '.totals.chaos_continuous_events_total // 0' "$summary_json")
    chaos_events_total=$(jq -r '.totals.chaos_events_total // 0' "$summary_json")
  fi

  {
    echo
    echo "## Metrics Artifacts"
    echo
    echo "- timeline_csv: \`$timeline_csv\`"
    echo "- summary_json: \`$summary_json\`"
    echo "- chaos_events_log: \`$chaos_events_log\`"
    echo "- topology_count: \`$topology_count\` (ok=\`$topology_ok_count\`, failed=\`$topology_failed_count\`)"
    echo "- report_samples_total: \`$report_samples_total\`"
    echo "- chaos_plan_events_total: \`$chaos_plan_events_total\`"
    echo "- chaos_continuous_events_total: \`$chaos_continuous_events_total\`"
    echo "- chaos_events_total: \`$chaos_events_total\`"
    echo
    echo "## Gate Metrics"
    echo
    echo "| topology | gate | reports | chaos_plan | chaos_continuous | chaos_events | chaos_exempt_s | max_stall_s | max_stall_s_effective | lag_p95 | distfs_ratio | invariant_all_ok |"
    echo "|---|---|---|---|---|---|---|---|---|---|---|---|"
    if [[ -f "$summary_json" ]]; then
      while IFS=$'\t' read -r topology gate reports chaos_plan chaos_continuous chaos_events chaos_exempt stall stall_effective lag ratio invariant; do
        echo "| $topology | $gate | $reports | $chaos_plan | $chaos_continuous | $chaos_events | $chaos_exempt | $stall | $stall_effective | $lag | $ratio | $invariant |"
      done < <(jq -r '.topologies[] | [ .topology, .metric_gate.status, .report_samples, .chaos_plan_events, .chaos_continuous_events, .chaos_events, .metrics.chaos_exempt_secs, .metrics.max_stall_secs_observed, .metrics.effective_max_stall_secs, .metrics.lag_p95, .metrics.distfs_failure_ratio, .metrics.invariant_all_ok ] | @tsv' "$summary_json")
    fi
  } >> "$summary_md"
}

write_failures_md() {
  local overall_status_code=$1
  if (( overall_status_code == 0 )); then
    rm -f "$failures_md"
    return 0
  fi

  {
    echo "# P2P Longrun Soak Failures"
    echo
    echo "- run_dir: \`$run_dir\`"
    echo "- profile: \`$profile\`"
    echo "- scenario: \`$scenario\`"
    echo
    echo "## Failed Topologies"
    if [[ -f "$summary_json" ]]; then
      jq -r '.topologies[] | select(.status != "ok") | "- topology=\(.topology) status=\(.status) process=\(.process_status) gate=\(.metric_gate.status) reports=\(.report_samples) notes=\(.notes)"' "$summary_json"
    fi
  } > "$failures_md"
}

overall_status=0
for idx in "${!topologies[@]}"; do
  topology="${topologies[$idx]}"
  echo "== topology: $topology =="
  if ! run_topology "$topology" "$idx"; then
    overall_status=1
    break
  fi
done

write_summary_json "$overall_status"
append_summary_metrics_section
write_failures_md "$overall_status"

echo "soak run completed:"
echo "  run_dir: $run_dir"
echo "  summary: $summary_md"
echo "  summary_json: $summary_json"
echo "  timeline_csv: $timeline_csv"
echo "  chaos_events_log: $chaos_events_log"
if [[ -f "$failures_md" ]]; then
  echo "  failures: $failures_md"
fi

exit "$overall_status"
