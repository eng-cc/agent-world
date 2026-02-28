#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

usage() {
  cat <<'USAGE'
Usage: ./scripts/s10-five-node-game-soak.sh [options]

Options:
  --duration-secs <n>              soak duration seconds (default: 1800)
  --scenario <name>                world_viewer_live scenario (default: llm_bootstrap)
  --llm                            enable LLM mode for world_viewer_live
  --no-llm                         disable LLM mode (default)
  --reward-runtime-epoch-duration-secs <n>
                                   reward settlement epoch duration seconds (default: 60)
  --reward-points-per-credit <n>   reward points -> credit ratio (default: 100)
  --base-port <n>                  base port for node port allocation (default: 5810)
  --bind-host <host>               bind host for gossip/live/libp2p endpoints (default: 127.0.0.1)
  --out-dir <path>                 output root (default: .tmp/s10_game_longrun)
  --startup-timeout-secs <n>       startup grace before monitor loop (default: 20)
  --poll-interval-secs <n>         monitor loop interval (default: 2)
  --max-stall-secs <n>             gate threshold for max no-progress window (default: 300)
  --max-lag-p95 <n>                gate threshold for p95(network_height - committed_height) (default: 12)
  --max-distfs-failure-ratio <r>   gate threshold for DistFS failed/total ratio (0~1, default: 0.25)
  --max-settlement-apply-failure-ratio <r>
                                   gate threshold for settlement apply failed/attempts ratio (0~1, default: 0)
  --node-auto-attest-all           enable local auto-attesting all validators on all nodes
  --node-no-auto-attest-all        disable local auto-attesting all validators on all nodes
  --node-auto-attest-sequencer-only
                                   enable auto-attest only on sequencer (default)
  --preserve-node-state            keep existing output/node-distfs/s10-* snapshot/state
  --no-prewarm                     skip cargo build prewarm
  --dry-run                        write config and commands only, do not start processes
  -h, --help                       show help

Topology:
  s10-sequencer (stake 35)
  s10-storage-a (stake 20)
  s10-storage-b (stake 20)
  s10-observer-a (stake 15)
  s10-observer-b (stake 10)

Output:
  <out-dir>/<timestamp>/
    run_config.json
    timeline.csv
    summary.json
    summary.md
    failures.md (only when failed)
    nodes/<node_id>/{command.txt,stdout.log,stderr.log,report/}
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

duration_secs=1800
scenario="llm_bootstrap"
llm_enabled=0
reward_runtime_epoch_duration_secs=60
reward_points_per_credit=100
base_port=5810
bind_host="127.0.0.1"
out_root=".tmp/s10_game_longrun"
startup_timeout_secs=20
poll_interval_secs=2
max_stall_secs=300
max_lag_p95=12
max_distfs_failure_ratio="0.25"
max_settlement_apply_failure_ratio="0"
node_auto_attest_mode=1
isolate_node_state=1
prewarm=1
dry_run=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --duration-secs)
      duration_secs=${2:-}
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
    --reward-runtime-epoch-duration-secs)
      reward_runtime_epoch_duration_secs=${2:-}
      shift 2
      ;;
    --reward-points-per-credit)
      reward_points_per_credit=${2:-}
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
    --max-settlement-apply-failure-ratio)
      max_settlement_apply_failure_ratio=${2:-}
      shift 2
      ;;
    --node-auto-attest-all)
      node_auto_attest_mode=2
      shift
      ;;
    --node-no-auto-attest-all)
      node_auto_attest_mode=0
      shift
      ;;
    --node-auto-attest-sequencer-only)
      node_auto_attest_mode=1
      shift
      ;;
    --preserve-node-state)
      isolate_node_state=0
      shift
      ;;
    --no-prewarm)
      prewarm=0
      shift
      ;;
    --dry-run)
      dry_run=1
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

ensure_positive_int "--duration-secs" "$duration_secs"
ensure_positive_int "--reward-runtime-epoch-duration-secs" "$reward_runtime_epoch_duration_secs"
ensure_positive_int "--reward-points-per-credit" "$reward_points_per_credit"
ensure_positive_int "--base-port" "$base_port"
ensure_positive_int "--startup-timeout-secs" "$startup_timeout_secs"
ensure_positive_int "--poll-interval-secs" "$poll_interval_secs"
ensure_non_negative_int "--max-stall-secs" "$max_stall_secs"
ensure_non_negative_int "--max-lag-p95" "$max_lag_p95"
ensure_ratio_between_zero_and_one "--max-distfs-failure-ratio" "$max_distfs_failure_ratio"
ensure_ratio_between_zero_and_one "--max-settlement-apply-failure-ratio" "$max_settlement_apply_failure_ratio"

scenario=$(trim "$scenario")
if [[ -z "$scenario" ]]; then
  echo "--scenario cannot be empty" >&2
  exit 2
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required for metrics aggregation but not found in PATH" >&2
  exit 1
fi

if [[ "$prewarm" -eq 1 ]] && [[ "$dry_run" -eq 0 ]]; then
  run env -u RUSTC_WRAPPER cargo build -p agent_world --bin world_viewer_live
fi

live_bin="$repo_root/target/debug/world_viewer_live"
if [[ "$dry_run" -eq 0 ]] && [[ ! -x "$live_bin" ]]; then
  echo "world_viewer_live binary not found: $live_bin" >&2
  echo "run with prewarm enabled or build it manually first" >&2
  exit 1
fi

timestamp=$(date +%Y%m%d-%H%M%S)
run_dir="$out_root/$timestamp"
run mkdir -p "$run_dir"

declare -a node_ids=(
  "s10-sequencer"
  "s10-storage-a"
  "s10-storage-b"
  "s10-observer-a"
  "s10-observer-b"
)
declare -a node_roles=(
  "sequencer"
  "storage"
  "storage"
  "observer"
  "observer"
)
declare -a node_stakes=(35 20 20 15 10)
node_count=${#node_ids[@]}

declare -a validator_specs=()
for idx in "${!node_ids[@]}"; do
  validator_specs+=("${node_ids[$idx]}:${node_stakes[$idx]}")
done

node_gossip_port() {
  local idx=$1
  printf '%s' $((base_port + idx + 1))
}

node_viewer_bind_port() {
  local idx=$1
  printf '%s' $((base_port + idx + 11))
}

node_repl_port() {
  local idx=$1
  printf '%s' $((base_port + idx + 31))
}

node_gossip_addr() {
  local idx=$1
  printf '%s:%s' "$bind_host" "$(node_gossip_port "$idx")"
}

node_viewer_bind_addr() {
  local idx=$1
  printf '%s:%s' "$bind_host" "$(node_viewer_bind_port "$idx")"
}

node_repl_addr() {
  local idx=$1
  printf '/ip4/%s/tcp/%s' "$bind_host" "$(node_repl_port "$idx")"
}

run_config_json="$run_dir/run_config.json"
summary_md="$run_dir/summary.md"
timeline_csv="$run_dir/timeline.csv"
summary_json="$run_dir/summary.json"
failures_md="$run_dir/failures.md"

node_table_tsv="$run_dir/.nodes.tsv"
: > "$node_table_tsv"
for idx in "${!node_ids[@]}"; do
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
    "${node_ids[$idx]}" \
    "${node_roles[$idx]}" \
    "${node_stakes[$idx]}" \
    "$(node_gossip_addr "$idx")" \
    "$(node_viewer_bind_addr "$idx")" \
    "$(node_repl_addr "$idx")" >> "$node_table_tsv"
done

validators_json=$(printf '%s\n' "${validator_specs[@]}" | jq -R -s '
  split("\n")
  | map(select(length > 0) | split(":") | {
      validator_id: .[0],
      stake: (.[1] | tonumber)
    })
')
nodes_json=$(jq -R -s '
  split("\n")
  | map(select(length > 0) | split("\t") | {
      node_id: .[0],
      role: .[1],
      stake: (.[2] | tonumber),
      gossip_bind: .[3],
      viewer_bind: .[4],
      repl_listen: .[5]
    })
' "$node_table_tsv")

jq -n \
  --arg run_dir "$run_dir" \
  --arg scenario "$scenario" \
  --arg bind_host "$bind_host" \
  --arg out_dir "$out_root" \
  --argjson duration_secs "$duration_secs" \
  --argjson llm_enabled "$llm_enabled" \
  --argjson reward_runtime_epoch_duration_secs "$reward_runtime_epoch_duration_secs" \
  --argjson reward_points_per_credit "$reward_points_per_credit" \
  --argjson base_port "$base_port" \
  --argjson startup_timeout_secs "$startup_timeout_secs" \
  --argjson poll_interval_secs "$poll_interval_secs" \
  --argjson max_stall_secs "$max_stall_secs" \
  --argjson max_lag_p95 "$max_lag_p95" \
  --argjson max_distfs_failure_ratio "$max_distfs_failure_ratio" \
  --argjson max_settlement_apply_failure_ratio "$max_settlement_apply_failure_ratio" \
  --argjson node_auto_attest_mode "$node_auto_attest_mode" \
  --argjson isolate_node_state "$isolate_node_state" \
  --argjson dry_run "$dry_run" \
  --argjson validators "$validators_json" \
  --argjson nodes "$nodes_json" \
  '{
    run_dir: $run_dir,
    scenario: $scenario,
    llm_enabled: ($llm_enabled == 1),
    reward_runtime_epoch_duration_secs: $reward_runtime_epoch_duration_secs,
    reward_points_per_credit: $reward_points_per_credit,
    duration_secs: $duration_secs,
    bind_host: $bind_host,
    base_port: $base_port,
    startup_timeout_secs: $startup_timeout_secs,
    poll_interval_secs: $poll_interval_secs,
    thresholds: {
      max_stall_secs: $max_stall_secs,
      max_lag_p95: $max_lag_p95,
      max_distfs_failure_ratio: $max_distfs_failure_ratio,
      max_settlement_apply_failure_ratio: $max_settlement_apply_failure_ratio
    },
    node_auto_attest_mode: (
      if $node_auto_attest_mode == 2 then "all"
      elif $node_auto_attest_mode == 1 then "sequencer_only"
      else "off"
      end
    ),
    isolate_node_state: ($isolate_node_state == 1),
    dry_run: ($dry_run == 1),
    validators: $validators,
    nodes: $nodes
  }' > "$run_config_json"

{
  echo "# S10 Five-Node Real Game Soak Summary"
  echo
  echo "- run_dir: \`$run_dir\`"
  echo "- duration_secs: \`$duration_secs\`"
  echo "- scenario: \`$scenario\`"
  echo "- llm_enabled: \`$llm_enabled\`"
  echo "- reward_runtime_epoch_duration_secs: \`$reward_runtime_epoch_duration_secs\`"
  echo "- reward_points_per_credit: \`$reward_points_per_credit\`"
  echo "- max_stall_secs: \`$max_stall_secs\`"
  echo "- max_lag_p95: \`$max_lag_p95\`"
  echo "- max_distfs_failure_ratio: \`$max_distfs_failure_ratio\`"
  echo "- max_settlement_apply_failure_ratio: \`$max_settlement_apply_failure_ratio\`"
  node_auto_attest_mode_label="off"
  if [[ "$node_auto_attest_mode" -eq 2 ]]; then
    node_auto_attest_mode_label="all"
  elif [[ "$node_auto_attest_mode" -eq 1 ]]; then
    node_auto_attest_mode_label="sequencer_only"
  fi
  echo "- node_auto_attest_mode: \`$node_auto_attest_mode_label\`"
  echo "- isolate_node_state: \`$isolate_node_state\`"
  echo
  echo "| run | status | process_status | metric_gate | reports | started_at | ended_at | notes |"
  echo "|---|---|---|---|---|---|---|---|"
} > "$summary_md"

echo "node,epoch_index,observed_at_unix_ms,committed_height,network_committed_height,lag,total_checks,failed_checks,distfs_failure_ratio,invariant_ok,total_distributed_points,minted_record_count,settlement_apply_attempts_total,settlement_apply_failures_total,settlement_apply_failure_ratio,report_path" > "$timeline_csv"

append_summary_row() {
  local run_name=$1
  local status=$2
  local process_status=$3
  local metric_gate=$4
  local reports=$5
  local started_at=$6
  local ended_at=$7
  local notes=$8
  echo "| $run_name | $status | $process_status | $metric_gate | $reports | $started_at | $ended_at | $notes |" >> "$summary_md"
}

declare -a prepared_cmd=()
prepare_node_command() {
  local idx=$1
  local node_id=${node_ids[$idx]}
  local role=${node_roles[$idx]}
  local report_dir="$run_dir/nodes/$node_id/report"
  local -a cmd=(
    "$live_bin"
    "$scenario"
    --topology single
    --bind "$(node_viewer_bind_addr "$idx")"
    --node-id "$node_id"
    --node-role "$role"
    --reward-runtime-enable
    --reward-runtime-epoch-duration-secs "$reward_runtime_epoch_duration_secs"
    --reward-points-per-credit "$reward_points_per_credit"
    --reward-runtime-leader-node "${node_ids[0]}"
    --reward-runtime-report-dir "$report_dir"
    --node-gossip-bind "$(node_gossip_addr "$idx")"
    --node-repl-libp2p-listen "$(node_repl_addr "$idx")"
  )

  if [[ "$node_auto_attest_mode" -eq 2 ]] || { [[ "$node_auto_attest_mode" -eq 1 ]] && [[ "$role" == "sequencer" ]]; }; then
    cmd+=(--node-auto-attest-all)
  else
    cmd+=(--node-no-auto-attest-all)
  fi

  if [[ "$llm_enabled" -eq 1 ]]; then
    cmd+=(--llm)
  else
    cmd+=(--no-llm)
  fi

  local validator
  for validator in "${validator_specs[@]}"; do
    cmd+=(--node-validator "$validator")
  done

  local peer_idx
  for peer_idx in "${!node_ids[@]}"; do
    if (( peer_idx == idx )); then
      continue
    fi
    cmd+=(--node-gossip-peer "$(node_gossip_addr "$peer_idx")")
    cmd+=(--node-repl-libp2p-peer "$(node_repl_addr "$peer_idx")")
  done

  prepared_cmd=("${cmd[@]}")
}

isolate_node_state_dirs() {
  local backup_root="$run_dir/node_state_backup"
  local node_id state_dir backup_dir
  for node_id in "${node_ids[@]}"; do
    state_dir="$repo_root/output/node-distfs/$node_id"
    if [[ ! -d "$state_dir" ]]; then
      continue
    fi
    run mkdir -p "$backup_root"
    backup_dir="$backup_root/${node_id}-$(date +%s)"
    while [[ -e "$backup_dir" ]]; do
      backup_dir="${backup_dir}-$RANDOM"
    done
    run mv "$state_dir" "$backup_dir"
  done
}

active_cleanup_done=0
declare -a active_pids=()
declare -a active_nodes=()

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

  local pid
  for pid in "${active_pids[@]}"; do
    wait "$pid" >/dev/null 2>&1 || true
  done
}

cleanup_on_exit() {
  stop_active_processes
}
trap cleanup_on_exit EXIT

launch_node() {
  local node_name=$1
  shift

  local node_dir="$run_dir/nodes/$node_name"
  local report_dir="$node_dir/report"
  local stdout_log="$node_dir/stdout.log"
  local stderr_log="$node_dir/stderr.log"
  local cmd_txt="$node_dir/command.txt"

  run mkdir -p "$report_dir"

  printf '%q ' "$@" > "$cmd_txt"
  printf '\n' >> "$cmd_txt"

  echo "+ $* > $stdout_log 2> $stderr_log"
  "$@" >"$stdout_log" 2>"$stderr_log" &
  local pid=$!
  active_pids+=("$pid")
  active_nodes+=("$node_name")
}

analysis_report_count=0
analysis_gate_status="insufficient_data"
analysis_gate_notes="no_epoch_reports"
analysis_max_stall_secs_observed=0
analysis_lag_p95=0
analysis_distfs_failure_ratio="0.000000"
analysis_distfs_total_checks=0
analysis_distfs_failed_checks=0
analysis_settlement_apply_failure_ratio="0.000000"
analysis_settlement_apply_attempts=0
analysis_settlement_apply_failures=0
analysis_invariant_all_ok=true
analysis_settlement_positive_samples=0
analysis_minted_non_empty_samples=0
analysis_monotonic_ok=true
analysis_monotonic_violation_nodes=""

analyze_metrics() {
  analysis_report_count=0
  analysis_gate_status="insufficient_data"
  analysis_gate_notes="no_epoch_reports"
  analysis_max_stall_secs_observed=0
  analysis_lag_p95=0
  analysis_distfs_failure_ratio="0.000000"
  analysis_distfs_total_checks=0
  analysis_distfs_failed_checks=0
  analysis_settlement_apply_failure_ratio="0.000000"
  analysis_settlement_apply_attempts=0
  analysis_settlement_apply_failures=0
  analysis_invariant_all_ok=true
  analysis_settlement_positive_samples=0
  analysis_minted_non_empty_samples=0
  analysis_monotonic_ok=true
  analysis_monotonic_violation_nodes=""

  local -a report_files=()
  if [[ -d "$run_dir/nodes" ]]; then
    while IFS= read -r report_path; do
      report_files+=("$report_path")
    done < <(find "$run_dir/nodes" -mindepth 3 -maxdepth 6 -type f -name 'epoch-*.json' | sort)
  fi

  analysis_report_count=${#report_files[@]}
  if (( analysis_report_count == 0 )); then
    return 0
  fi

  local samples_tsv="$run_dir/.metric_samples.tsv"
  : > "$samples_tsv"

  declare -A node_total_max=()
  declare -A node_failed_at_max=()
  declare -A node_settlement_attempts_max=()
  declare -A node_settlement_failures_at_max=()
  local invariant_failed=0

  local report_file node_name metrics
  local epoch_index observed_at committed_height network_committed_height total_checks failed_checks
  local invariant_ok total_distributed_points minted_record_count lag ratio
  local settlement_apply_attempts_total settlement_apply_failures_total settlement_apply_ratio
  for report_file in "${report_files[@]}"; do
    node_name=${report_file#"$run_dir/nodes/"}
    node_name=${node_name%%/*}

    if ! metrics=$(jq -r '[
      (.settlement_report.epoch_index // .node_snapshot.consensus.epoch // 0),
      (.observed_at_unix_ms // 0),
      (.node_snapshot.consensus.committed_height // 0),
      (.node_snapshot.consensus.network_committed_height // 0),
      (.distfs_challenge_report.total_checks // 0),
      (.distfs_challenge_report.failed_checks // 0),
      ((.reward_asset_invariant_status.ok // false) | tostring),
      ((.settlement_report.total_distributed_points // 0) | tonumber? // 0),
      ((.minted_records // []) | length),
      ((.reward_settlement_transport.settlement_apply_attempts_total // 0) | tonumber? // 0),
      ((.reward_settlement_transport.settlement_apply_failures_total // 0) | tonumber? // 0)
    ] | @tsv' "$report_file"); then
      echo "warning: failed to parse report JSON: $report_file" >&2
      continue
    fi

    if [[ -z "$metrics" ]]; then
      continue
    fi

    IFS=$'\t' read -r \
      epoch_index \
      observed_at \
      committed_height \
      network_committed_height \
      total_checks \
      failed_checks \
      invariant_ok \
      total_distributed_points \
      minted_record_count \
      settlement_apply_attempts_total \
      settlement_apply_failures_total <<< "$metrics"

    [[ "$epoch_index" =~ ^-?[0-9]+$ ]] || epoch_index=0
    [[ "$observed_at" =~ ^-?[0-9]+$ ]] || observed_at=0
    [[ "$committed_height" =~ ^-?[0-9]+$ ]] || committed_height=0
    [[ "$network_committed_height" =~ ^-?[0-9]+$ ]] || network_committed_height=0
    [[ "$total_checks" =~ ^-?[0-9]+$ ]] || total_checks=0
    [[ "$failed_checks" =~ ^-?[0-9]+$ ]] || failed_checks=0
    [[ "$total_distributed_points" =~ ^-?[0-9]+$ ]] || total_distributed_points=0
    [[ "$minted_record_count" =~ ^-?[0-9]+$ ]] || minted_record_count=0
    [[ "$settlement_apply_attempts_total" =~ ^-?[0-9]+$ ]] || settlement_apply_attempts_total=0
    [[ "$settlement_apply_failures_total" =~ ^-?[0-9]+$ ]] || settlement_apply_failures_total=0

    if (( total_checks < 0 )); then
      total_checks=0
    fi
    if (( failed_checks < 0 )); then
      failed_checks=0
    fi
    if (( total_distributed_points < 0 )); then
      total_distributed_points=0
    fi
    if (( minted_record_count < 0 )); then
      minted_record_count=0
    fi
    if (( settlement_apply_attempts_total < 0 )); then
      settlement_apply_attempts_total=0
    fi
    if (( settlement_apply_failures_total < 0 )); then
      settlement_apply_failures_total=0
    fi

    lag=$((network_committed_height - committed_height))
    if (( lag < 0 )); then
      lag=0
    fi

    ratio=$(awk -v failed="$failed_checks" -v total="$total_checks" 'BEGIN { if (total > 0) printf "%.6f", failed / total; else printf "0.000000"; }')
    settlement_apply_ratio=$(awk -v failed="$settlement_apply_failures_total" -v total="$settlement_apply_attempts_total" 'BEGIN { if (total > 0) printf "%.6f", failed / total; else printf "0.000000"; }')

    printf '"%s",%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,%s,"%s"\n' \
      "$node_name" \
      "$epoch_index" \
      "$observed_at" \
      "$committed_height" \
      "$network_committed_height" \
      "$lag" \
      "$total_checks" \
      "$failed_checks" \
      "$ratio" \
      "$invariant_ok" \
      "$total_distributed_points" \
      "$minted_record_count" \
      "$settlement_apply_attempts_total" \
      "$settlement_apply_failures_total" \
      "$settlement_apply_ratio" \
      "$report_file" >> "$timeline_csv"

    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
      "$observed_at" \
      "$committed_height" \
      "$lag" \
      "$total_checks" \
      "$failed_checks" \
      "$invariant_ok" \
      "$total_distributed_points" \
      "$minted_record_count" \
      "$settlement_apply_attempts_total" \
      "$settlement_apply_failures_total" \
      "$node_name" >> "$samples_tsv"

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

    local prev_settlement_attempts=${node_settlement_attempts_max[$node_name]:--1}
    if (( settlement_apply_attempts_total > prev_settlement_attempts )); then
      node_settlement_attempts_max["$node_name"]=$settlement_apply_attempts_total
      node_settlement_failures_at_max["$node_name"]=$settlement_apply_failures_total
    elif (( settlement_apply_attempts_total == prev_settlement_attempts )); then
      local prev_settlement_failures=${node_settlement_failures_at_max[$node_name]:-0}
      if (( settlement_apply_failures_total > prev_settlement_failures )); then
        node_settlement_failures_at_max["$node_name"]=$settlement_apply_failures_total
      fi
    fi

    if [[ "$invariant_ok" != "true" ]]; then
      invariant_failed=1
    fi
    if (( total_distributed_points > 0 )); then
      analysis_settlement_positive_samples=$((analysis_settlement_positive_samples + 1))
    fi
    if (( minted_record_count > 0 )); then
      analysis_minted_non_empty_samples=$((analysis_minted_non_empty_samples + 1))
    fi
  done

  local node
  for node in "${!node_total_max[@]}"; do
    analysis_distfs_total_checks=$((analysis_distfs_total_checks + node_total_max[$node]))
    analysis_distfs_failed_checks=$((analysis_distfs_failed_checks + ${node_failed_at_max[$node]:-0}))
  done
  analysis_distfs_failure_ratio=$(awk -v failed="$analysis_distfs_failed_checks" -v total="$analysis_distfs_total_checks" 'BEGIN { if (total > 0) printf "%.6f", failed / total; else printf "0.000000"; }')
  for node in "${!node_settlement_attempts_max[@]}"; do
    analysis_settlement_apply_attempts=$((analysis_settlement_apply_attempts + node_settlement_attempts_max[$node]))
    analysis_settlement_apply_failures=$((analysis_settlement_apply_failures + ${node_settlement_failures_at_max[$node]:-0}))
  done
  analysis_settlement_apply_failure_ratio=$(awk -v failed="$analysis_settlement_apply_failures" -v total="$analysis_settlement_apply_attempts" 'BEGIN { if (total > 0) printf "%.6f", failed / total; else printf "0.000000"; }')

  local sorted_samples="$run_dir/.metric_samples.sorted.tsv"
  sort -n -k1,1 "$samples_tsv" > "$sorted_samples"

  local best_height=-1
  local last_progress_ms=0
  local max_stall_ms=0
  local sample_with_time=0
  local sample_observed sample_committed
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

  local sorted_by_node="$run_dir/.metric_samples.by_node.tsv"
  sort -k11,11 -k1,1n "$samples_tsv" > "$sorted_by_node"
  declare -A node_prev_committed=()
  declare -A node_monotonic_violations=()
  local line_observed line_committed line_node
  while IFS=$'\t' read -r line_observed line_committed _ _ _ _ _ _ _ _ line_node; do
    [[ "$line_observed" =~ ^-?[0-9]+$ ]] || continue
    [[ "$line_committed" =~ ^-?[0-9]+$ ]] || continue
    local prev=${node_prev_committed[$line_node]:-}
    if [[ -n "$prev" ]] && (( line_committed < prev )); then
      node_monotonic_violations["$line_node"]=1
    fi
    node_prev_committed["$line_node"]=$line_committed
  done < "$sorted_by_node"
  if (( ${#node_monotonic_violations[@]} > 0 )); then
    analysis_monotonic_ok=false
    analysis_monotonic_violation_nodes=$(printf '%s\n' "${!node_monotonic_violations[@]}" | sort | paste -sd ',' -)
  fi

  local lag_values="$run_dir/.metric_lags.txt"
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

  if (( analysis_settlement_apply_attempts <= 0 )); then
    gate_failures+=("settlement_apply_attempts=0")
  else
    local settlement_ratio_exceeded
    settlement_ratio_exceeded=$(awk -v ratio="$analysis_settlement_apply_failure_ratio" -v max="$max_settlement_apply_failure_ratio" 'BEGIN { if (ratio > max) print 1; else print 0; }')
    if [[ "$settlement_ratio_exceeded" == "1" ]]; then
      gate_failures+=("settlement_apply_failure_ratio=${analysis_settlement_apply_failure_ratio}>max_${max_settlement_apply_failure_ratio}")
    fi
  fi

  if [[ "$analysis_invariant_all_ok" != "true" ]]; then
    gate_failures+=("reward_asset_invariant_not_ok")
  fi

  if (( analysis_settlement_positive_samples == 0 )); then
    gate_failures+=("settlement_total_distributed_points_not_positive")
  fi

  if (( analysis_minted_non_empty_samples == 0 )); then
    gate_failures+=("minted_records_empty")
  fi

  if [[ "$analysis_monotonic_ok" != "true" ]]; then
    gate_failures+=("committed_height_not_monotonic nodes=${analysis_monotonic_violation_nodes}")
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

write_summary_json() {
  local final_status=$1
  local process_status=$2
  local started_at=$3
  local ended_at=$4
  local notes=$5
  local overall_status_code=$6
  local generated_at
  generated_at=$(date -u '+%Y-%m-%dT%H:%M:%SZ')

  jq -n \
    --arg generated_at "$generated_at" \
    --arg run_dir "$run_dir" \
    --arg scenario "$scenario" \
    --arg summary_md "$summary_md" \
    --arg timeline_csv "$timeline_csv" \
    --arg run_config_json "$run_config_json" \
    --arg failures_md "$failures_md" \
    --arg final_status "$final_status" \
    --arg process_status "$process_status" \
    --arg started_at "$started_at" \
    --arg ended_at "$ended_at" \
    --arg notes "$notes" \
    --arg gate_status "$analysis_gate_status" \
    --arg gate_notes "$analysis_gate_notes" \
    --argjson llm_enabled "$llm_enabled" \
    --argjson duration_secs "$duration_secs" \
    --argjson max_stall_secs "$max_stall_secs" \
    --argjson max_lag_p95 "$max_lag_p95" \
    --argjson max_distfs_failure_ratio "$max_distfs_failure_ratio" \
    --argjson max_settlement_apply_failure_ratio "$max_settlement_apply_failure_ratio" \
    --argjson report_samples "$analysis_report_count" \
    --argjson max_stall_secs_observed "$analysis_max_stall_secs_observed" \
    --argjson lag_p95 "$analysis_lag_p95" \
    --argjson distfs_failure_ratio "$analysis_distfs_failure_ratio" \
    --argjson distfs_total_checks "$analysis_distfs_total_checks" \
    --argjson distfs_failed_checks "$analysis_distfs_failed_checks" \
    --argjson settlement_apply_failure_ratio "$analysis_settlement_apply_failure_ratio" \
    --argjson settlement_apply_attempts "$analysis_settlement_apply_attempts" \
    --argjson settlement_apply_failures "$analysis_settlement_apply_failures" \
    --argjson invariant_all_ok "$analysis_invariant_all_ok" \
    --argjson settlement_positive_samples "$analysis_settlement_positive_samples" \
    --argjson minted_non_empty_samples "$analysis_minted_non_empty_samples" \
    --argjson monotonic_ok "$analysis_monotonic_ok" \
    --arg monotonic_violation_nodes "$analysis_monotonic_violation_nodes" \
    --argjson overall_status_code "$overall_status_code" \
    '{
      generated_at_utc: $generated_at,
      run_dir: $run_dir,
      scenario: $scenario,
      llm_enabled: ($llm_enabled == 1),
      duration_secs: $duration_secs,
      thresholds: {
        max_stall_secs: $max_stall_secs,
        max_lag_p95: $max_lag_p95,
        max_distfs_failure_ratio: $max_distfs_failure_ratio,
        max_settlement_apply_failure_ratio: $max_settlement_apply_failure_ratio
      },
      artifacts: {
        run_config_json: $run_config_json,
        timeline_csv: $timeline_csv,
        summary_md: $summary_md,
        failures_md: (if $overall_status_code == 0 then null else $failures_md end)
      },
      run: {
        status: $final_status,
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
          settlement_apply_failure_ratio: $settlement_apply_failure_ratio,
          settlement_apply_attempts: $settlement_apply_attempts,
          settlement_apply_failures: $settlement_apply_failures,
          invariant_all_ok: $invariant_all_ok,
          settlement_positive_samples: $settlement_positive_samples,
          minted_non_empty_samples: $minted_non_empty_samples,
          committed_height_monotonic: $monotonic_ok,
          committed_height_monotonic_violation_nodes: (
            if $monotonic_violation_nodes == "" then []
            else ($monotonic_violation_nodes | split(","))
            end
          )
        }
      },
      overall_status: (if $overall_status_code == 0 then "ok" else "failed" end)
    }' > "$summary_json"
}

append_summary_metrics_section() {
  {
    echo
    echo "## Metrics Artifacts"
    echo
    echo "- timeline_csv: \`$timeline_csv\`"
    echo "- summary_json: \`$summary_json\`"
    echo "- report_samples: \`$analysis_report_count\`"
    echo
    echo "## Gate Metrics"
    echo
    echo "| metric | value |"
    echo "|---|---|"
    echo "| metric_gate | $analysis_gate_status |"
    echo "| metric_gate_notes | $analysis_gate_notes |"
    echo "| max_stall_secs_observed | $analysis_max_stall_secs_observed |"
    echo "| lag_p95 | $analysis_lag_p95 |"
    echo "| distfs_failure_ratio | $analysis_distfs_failure_ratio |"
    echo "| distfs_total_checks | $analysis_distfs_total_checks |"
    echo "| distfs_failed_checks | $analysis_distfs_failed_checks |"
    echo "| settlement_apply_failure_ratio | $analysis_settlement_apply_failure_ratio |"
    echo "| settlement_apply_attempts | $analysis_settlement_apply_attempts |"
    echo "| settlement_apply_failures | $analysis_settlement_apply_failures |"
    echo "| invariant_all_ok | $analysis_invariant_all_ok |"
    echo "| settlement_positive_samples | $analysis_settlement_positive_samples |"
    echo "| minted_non_empty_samples | $analysis_minted_non_empty_samples |"
    echo "| committed_height_monotonic | $analysis_monotonic_ok |"
    echo "| committed_height_monotonic_violation_nodes | ${analysis_monotonic_violation_nodes:--} |"
  } >> "$summary_md"
}

write_failures_md() {
  local final_status=$1
  local process_status=$2
  local notes=$3
  local overall_status_code=$4

  if (( overall_status_code == 0 )); then
    rm -f "$failures_md"
    return 0
  fi

  {
    echo "# S10 Five-Node Real Game Soak Failures"
    echo
    echo "- run_dir: \`$run_dir\`"
    echo "- scenario: \`$scenario\`"
    echo "- final_status: \`$final_status\`"
    echo "- process_status: \`$process_status\`"
    echo "- gate_status: \`$analysis_gate_status\`"
    echo "- notes: \`$notes\`"
    echo
    echo "## Gate Notes"
    echo
    echo "- \`$analysis_gate_notes\`"
  } > "$failures_md"
}

if [[ "$dry_run" -eq 1 ]]; then
  for idx in "${!node_ids[@]}"; do
    node_name=${node_ids[$idx]}
    node_dir="$run_dir/nodes/$node_name"
    report_dir="$node_dir/report"
    cmd_txt="$node_dir/command.txt"
    run mkdir -p "$report_dir"
    prepare_node_command "$idx"
    printf '%q ' "${prepared_cmd[@]}" > "$cmd_txt"
    printf '\n' >> "$cmd_txt"
    echo "+ dry-run command[$node_name]: ${prepared_cmd[*]}"
  done

  append_summary_row "five_node_real_game" "dry_run" "dry_run" "dry_run" "0" "-" "-" "commands_rendered_only"
  jq -n \
    --arg run_dir "$run_dir" \
    --arg scenario "$scenario" \
    --arg summary_md "$summary_md" \
    --arg summary_json "$summary_json" \
    --arg timeline_csv "$timeline_csv" \
    --arg run_config_json "$run_config_json" \
    --argjson llm_enabled "$llm_enabled" \
    --argjson duration_secs "$duration_secs" \
    '{
      run_dir: $run_dir,
      scenario: $scenario,
      llm_enabled: ($llm_enabled == 1),
      duration_secs: $duration_secs,
      artifacts: {
        run_config_json: $run_config_json,
        timeline_csv: $timeline_csv,
        summary_md: $summary_md
      },
      run: {
        status: "dry_run",
        process_status: "dry_run",
        metric_gate: {
          status: "dry_run",
          notes: "commands_rendered_only"
        },
        report_samples: 0
      },
      overall_status: "dry_run"
    }' > "$summary_json"
  rm -f "$failures_md"

  echo "dry-run completed:"
  echo "  run_dir: $run_dir"
  echo "  run_config: $run_config_json"
  echo "  summary: $summary_md"
  echo "  summary_json: $summary_json"
  exit 0
fi

if [[ "$isolate_node_state" -eq 1 ]]; then
  isolate_node_state_dirs
fi

started_at=$(date '+%Y-%m-%d %H:%M:%S %Z')
run_status="ok"
run_notes="-"

for idx in "${!node_ids[@]}"; do
  prepare_node_command "$idx"
  launch_node "${node_ids[$idx]}" "${prepared_cmd[@]}"
done

sleep "$startup_timeout_secs"
for idx in "${!active_pids[@]}"; do
  if ! kill -0 "${active_pids[$idx]}" >/dev/null 2>&1; then
    run_status="startup_failed"
    run_notes="node=${active_nodes[$idx]} exited before monitor loop"
    break
  fi
done

if [[ "$run_status" == "ok" ]]; then
  started_epoch_sec=$(date +%s)
  deadline=$((started_epoch_sec + duration_secs))
  while (( $(date +%s) < deadline )); do
    for idx in "${!active_pids[@]}"; do
      if ! kill -0 "${active_pids[$idx]}" >/dev/null 2>&1; then
        run_status="process_exit"
        run_notes="node=${active_nodes[$idx]} exited during soak"
        break 2
      fi
    done
    sleep "$poll_interval_secs"
  done
fi

stop_active_processes
ended_at=$(date '+%Y-%m-%d %H:%M:%S %Z')
process_status=$run_status

analyze_metrics

final_status=$run_status
declare -a notes_parts=()
if [[ "$run_notes" != "-" ]]; then
  notes_parts+=("$run_notes")
fi
if [[ "$analysis_gate_status" != "pass" ]]; then
  notes_parts+=("metric_gate=$analysis_gate_notes")
  if [[ "$final_status" == "ok" ]]; then
    final_status="metric_gate_failed"
  fi
fi

if (( ${#notes_parts[@]} == 0 )); then
  notes="-"
else
  notes=$(join_by "; " "${notes_parts[@]}")
fi

overall_status=0
if [[ "$final_status" != "ok" ]]; then
  overall_status=1
fi

append_summary_row "five_node_real_game" "$final_status" "$process_status" "$analysis_gate_status" "$analysis_report_count" "$started_at" "$ended_at" "$notes"
write_summary_json "$final_status" "$process_status" "$started_at" "$ended_at" "$notes" "$overall_status"
append_summary_metrics_section
write_failures_md "$final_status" "$process_status" "$notes" "$overall_status"

echo "S10 soak run completed:"
echo "  run_dir: $run_dir"
echo "  summary: $summary_md"
echo "  summary_json: $summary_json"
echo "  timeline_csv: $timeline_csv"
if [[ -f "$failures_md" ]]; then
  echo "  failures: $failures_md"
fi

exit "$overall_status"
