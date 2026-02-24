#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

usage() {
  cat <<'USAGE'
Usage: ./scripts/p2p-longrun-soak.sh [options]

Options:
  --profile <name>            soak_smoke | soak_endurance | soak_release (default: soak_smoke)
  --duration-secs <n>         override per-topology soak duration seconds
  --topologies <csv>          comma-separated topologies: triad,triad_distributed
  --scenario <name>           world_viewer_live scenario (default: triad_p2p_bootstrap)
  --llm                       enable LLM mode for world_viewer_live
  --no-llm                    disable LLM mode (default)
  --tick-ms <n>               world_viewer_live tick interval (default: 200)
  --base-port <n>             base port for per-topology allocation (default: 5610)
  --bind-host <host>          bind host for gossip/live endpoints (default: 127.0.0.1)
  --out-dir <path>            output root (default: .tmp/p2p_longrun)
  --startup-timeout-secs <n>  startup grace before monitor loop (default: 20)
  --poll-interval-secs <n>    monitor loop interval (default: 2)
  --no-prewarm                skip cargo build prewarm
  -h, --help                  show help

Profiles:
  soak_smoke      default duration 1500s, default topologies triad,triad_distributed
  soak_endurance  default duration 10800s, default topologies triad_distributed
  soak_release    default duration 28800s, default topologies triad_distributed

Output:
  <out-dir>/<timestamp>/
    run_config.json
    summary.md
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

ensure_positive_int() {
  local flag=$1
  local value=$2
  if [[ ! "$value" =~ ^[0-9]+$ ]] || [[ "$value" -le 0 ]]; then
    echo "invalid $flag: $value" >&2
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

profile="soak_smoke"
duration_secs=""
topologies_csv=""
scenario="triad_p2p_bootstrap"
llm_enabled=0
tick_ms=200
base_port=5610
bind_host="127.0.0.1"
out_root=".tmp/p2p_longrun"
startup_timeout_secs=20
poll_interval_secs=2
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
    --tick-ms)
      tick_ms=${2:-}
      shift 2
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
    ;;
  soak_endurance)
    default_duration_secs=10800
    default_topologies_csv="triad_distributed"
    ;;
  soak_release)
    default_duration_secs=28800
    default_topologies_csv="triad_distributed"
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

ensure_positive_int "--duration-secs" "$duration_secs"
ensure_positive_int "--tick-ms" "$tick_ms"
ensure_positive_int "--base-port" "$base_port"
ensure_positive_int "--startup-timeout-secs" "$startup_timeout_secs"
ensure_positive_int "--poll-interval-secs" "$poll_interval_secs"

if [[ -z "$scenario" ]]; then
  echo "--scenario cannot be empty" >&2
  exit 2
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
{
  echo "{"
  echo "  \"profile\": \"$profile\"," 
  echo "  \"duration_secs\": $duration_secs," 
  echo "  \"scenario\": \"$scenario\"," 
  echo "  \"llm_enabled\": $llm_enabled," 
  echo "  \"tick_ms\": $tick_ms," 
  echo "  \"base_port\": $base_port," 
  echo "  \"bind_host\": \"$bind_host\"," 
  echo "  \"startup_timeout_secs\": $startup_timeout_secs," 
  echo "  \"poll_interval_secs\": $poll_interval_secs," 
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
{
  echo "# P2P Longrun Soak Summary"
  echo
  echo "- run_dir: \`$run_dir\`"
  echo "- profile: \`$profile\`"
  echo "- duration_secs_per_topology: \`$duration_secs\`"
  echo "- scenario: \`$scenario\`"
  echo "- tick_ms: \`$tick_ms\`"
  echo
  echo "| topology | status | started_at | ended_at | notes |"
  echo "|---|---|---|---|---|"
} > "$summary_md"

active_cleanup_done=0
declare -a active_pids=()
declare -a active_nodes=()

declare -a active_topology_dirs=()

append_summary_row() {
  local topology=$1
  local status=$2
  local started_at=$3
  local ended_at=$4
  local notes=$5
  echo "| $topology | $status | $started_at | $ended_at | $notes |" >> "$summary_md"
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
    --tick-ms "$tick_ms"
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

  active_pids+=("$pid")
  active_nodes+=("$node_name")
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

  local topology_dir="$run_dir/$topology"
  active_topology_dirs+=("$topology_dir")
  run mkdir -p "$topology_dir/nodes"

  local case_base_port=$((base_port + index * 100))

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

  local startup_deadline=$(( $(date +%s) + startup_timeout_secs ))
  while :; do
    local all_alive=1
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

  if [[ "$status" == "ok" ]]; then
    local deadline=$(( $(date +%s) + duration_secs ))
    while (( $(date +%s) < deadline )); do
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
  ended_at=$(date '+%Y-%m-%d %H:%M:%S %Z')
  append_summary_row "$topology" "$status" "$started_at" "$ended_at" "$notes"

  if [[ "$status" != "ok" ]]; then
    echo "topology run failed: $topology ($notes)" >&2
    return 1
  fi
  return 0
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

echo "soak run completed:"
echo "  run_dir: $run_dir"
echo "  summary: $summary_md"

exit "$overall_status"
