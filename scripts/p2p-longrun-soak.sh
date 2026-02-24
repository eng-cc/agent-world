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
  --tick-ms <n>                    world_viewer_live tick interval (default: 200)
  --base-port <n>                  base port for per-topology allocation (default: 5610)
  --bind-host <host>               bind host for gossip/live endpoints (default: 127.0.0.1)
  --out-dir <path>                 output root (default: .tmp/p2p_longrun)
  --startup-timeout-secs <n>       startup grace before monitor loop (default: 20)
  --poll-interval-secs <n>         monitor loop interval (default: 2)
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
ensure_positive_int "--tick-ms" "$tick_ms"
ensure_positive_int "--base-port" "$base_port"
ensure_positive_int "--startup-timeout-secs" "$startup_timeout_secs"
ensure_positive_int "--poll-interval-secs" "$poll_interval_secs"
ensure_non_negative_int "--max-stall-secs" "$max_stall_secs"
ensure_non_negative_int "--max-lag-p95" "$max_lag_p95"
ensure_ratio_between_zero_and_one "--max-distfs-failure-ratio" "$max_distfs_failure_ratio"

if [[ -z "$scenario" ]]; then
  echo "--scenario cannot be empty" >&2
  exit 2
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
topology_summary_ndjson="$run_dir/.topology_summary.ndjson"

{
  echo "# P2P Longrun Soak Summary"
  echo
  echo "- run_dir: \`$run_dir\`"
  echo "- profile: \`$profile\`"
  echo "- duration_secs_per_topology: \`$duration_secs\`"
  echo "- scenario: \`$scenario\`"
  echo "- tick_ms: \`$tick_ms\`"
  echo "- max_stall_secs: \`$max_stall_secs\`"
  echo "- max_lag_p95: \`$max_lag_p95\`"
  echo "- max_distfs_failure_ratio: \`$max_distfs_failure_ratio\`"
  echo
  echo "| topology | status | process_status | metric_gate | reports | started_at | ended_at | notes |"
  echo "|---|---|---|---|---|---|---|---|"
} > "$summary_md"

echo "topology,node,epoch_index,observed_at_unix_ms,committed_height,network_committed_height,lag,total_checks,failed_checks,distfs_failure_ratio,invariant_ok,report_path" > "$timeline_csv"
: > "$topology_summary_ndjson"

active_cleanup_done=0
declare -a active_pids=()
declare -a active_nodes=()

analysis_report_count=0
analysis_gate_status="insufficient_data"
analysis_gate_notes="no_epoch_reports"
analysis_max_stall_secs_observed=0
analysis_lag_p95=0
analysis_distfs_failure_ratio="0.000000"
analysis_distfs_total_checks=0
analysis_distfs_failed_checks=0
analysis_invariant_all_ok=true

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
  elif (( analysis_max_stall_secs_observed > max_stall_secs )); then
    gate_failures+=("stall=${analysis_max_stall_secs_observed}s>max_${max_stall_secs}s")
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

  local topology_dir="$run_dir/$topology"
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

  if [[ "$status" == "ok" ]]; then
    local deadline=$(( $(date +%s) + duration_secs ))
    while (( $(date +%s) < deadline )); do
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
    --argjson max_stall_secs_observed "$analysis_max_stall_secs_observed" \
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
      metric_gate: {
        status: $gate_status,
        notes: $gate_notes
      },
      metrics: {
        max_stall_secs_observed: $max_stall_secs_observed,
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
    --argjson duration_secs "$duration_secs" \
    --argjson max_stall_secs "$max_stall_secs" \
    --argjson max_lag_p95 "$max_lag_p95" \
    --argjson max_distfs_failure_ratio "$max_distfs_failure_ratio" \
    --arg timeline_csv "$timeline_csv" \
    --arg summary_md "$summary_md" \
    --arg run_config_json "$run_config_json" \
    --argjson overall_status_code "$overall_status_code" \
    '
      . as $topologies |
      {
        generated_at_utc: $generated_at,
        run_dir: $run_dir,
        profile: $profile,
        scenario: $scenario,
        duration_secs_per_topology: $duration_secs,
        thresholds: {
          max_stall_secs: $max_stall_secs,
          max_lag_p95: $max_lag_p95,
          max_distfs_failure_ratio: $max_distfs_failure_ratio
        },
        artifacts: {
          run_config_json: $run_config_json,
          timeline_csv: $timeline_csv,
          summary_md: $summary_md
        },
        totals: {
          topology_count: ($topologies | length),
          topology_ok_count: ($topologies | map(select(.status == "ok")) | length),
          topology_failed_count: ($topologies | map(select(.status != "ok")) | length),
          report_samples_total: ($topologies | map(.report_samples) | add // 0)
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

  if [[ -f "$summary_json" ]]; then
    topology_count=$(jq -r '.totals.topology_count // 0' "$summary_json")
    topology_ok_count=$(jq -r '.totals.topology_ok_count // 0' "$summary_json")
    topology_failed_count=$(jq -r '.totals.topology_failed_count // 0' "$summary_json")
    report_samples_total=$(jq -r '.totals.report_samples_total // 0' "$summary_json")
  fi

  {
    echo
    echo "## Metrics Artifacts"
    echo
    echo "- timeline_csv: \`$timeline_csv\`"
    echo "- summary_json: \`$summary_json\`"
    echo "- topology_count: \`$topology_count\` (ok=\`$topology_ok_count\`, failed=\`$topology_failed_count\`)"
    echo "- report_samples_total: \`$report_samples_total\`"
    echo
    echo "## Gate Metrics"
    echo
    echo "| topology | gate | reports | max_stall_s | lag_p95 | distfs_ratio | invariant_all_ok |"
    echo "|---|---|---|---|---|---|---|"
    if [[ -f "$summary_json" ]]; then
      while IFS=$'\t' read -r topology gate reports stall lag ratio invariant; do
        echo "| $topology | $gate | $reports | $stall | $lag | $ratio | $invariant |"
      done < <(jq -r '.topologies[] | [ .topology, .metric_gate.status, .report_samples, .metrics.max_stall_secs_observed, .metrics.lag_p95, .metrics.distfs_failure_ratio, .metrics.invariant_all_ok ] | @tsv' "$summary_json")
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
if [[ -f "$failures_md" ]]; then
  echo "  failures: $failures_md"
fi

exit "$overall_status"
