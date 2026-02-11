#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

print_help() {
  cat <<'USAGE'
Usage: ./scripts/llm-longrun-stress.sh [options]

Options:
  --scenario <name>            Scenario for world_llm_agent_demo (repeatable)
  --scenarios <a,b,c>          Comma-separated scenario list
  --jobs <n>                   Max parallel scenarios in multi-scenario mode (default: 1)
  --ticks <n>                  Number of ticks to run (default: 240)
  --out-dir <path>             Output directory (default: .tmp/llm_stress)
  --report-json <path>         Report json path (default: <out-dir>/report.json)
  --log-file <path>            Raw command log path (default: <out-dir>/run.log)
  --summary-file <path>        Summary text path (default: <out-dir>/summary.txt)
  --max-llm-errors <n>         Fail if llm_errors > n (default: 0)
  --max-parse-errors <n>       Fail if parse_errors > n (default: 0)
  --max-repair-rounds-max <n>  Fail if repair_rounds_max > n (default: 2)
  --min-active-ticks <n>       Fail if active_ticks < n (default: ticks)
  --no-llm-io                  Disable LLM input/output logging in run.log
  --llm-io-max-chars <n>       Truncate each LLM input/output block to n chars
  --keep-out-dir               Keep existing out dir content
  -h, --help                   Show help

Notes:
  - If no scenario is provided, default is llm_bootstrap.
  - Single-scenario mode keeps legacy output behavior.
  - Multi-scenario mode supports parallel runs via --jobs.
  - Multi-scenario mode writes per-scenario outputs to:
      <out-dir>/scenarios/<scenario>/{report.json,run.log,summary.txt}
    and writes aggregate outputs to report-json/log-file/summary-file.

Output:
  - report json: detailed run metrics emitted by world_llm_agent_demo
  - run log: cargo run stdout/stderr output (includes LLM I/O by default)
  - summary: flattened key metrics for quick comparison
USAGE
}

run() {
  echo "+ $*"
  "$@"
}

ensure_positive_int() {
  local name=$1
  local value=$2
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    echo "invalid integer for $name: $value" >&2
    exit 2
  fi
}

trim_whitespace() {
  local value=$1
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  echo "$value"
}

append_scenario() {
  local raw=$1
  local value
  value=$(trim_whitespace "$raw")
  if [[ -z "$value" ]]; then
    echo "invalid empty scenario" >&2
    exit 2
  fi
  scenario_inputs+=("$value")
}

append_scenarios_from_csv() {
  local csv=$1
  local part
  IFS=',' read -r -a csv_parts <<<"$csv"
  for part in "${csv_parts[@]}"; do
    append_scenario "$part"
  done
}

extract_metric_from_log() {
  local key=$1
  local log_path=$2
  local line
  line=$(grep -E "^${key}: " "$log_path" | tail -n1 || true)
  if [[ -z "$line" ]]; then
    return 1
  fi
  echo "${line##*: }"
}

active_ticks=0
total_decisions=0
total_actions=0
action_success=0
action_failure=0
llm_errors=0
parse_errors=0
repair_rounds_total=0
repair_rounds_max=0
llm_input_chars_total=0
llm_input_chars_avg=0
llm_input_chars_max=0
clipped_sections=0
decision_wait=0
decision_wait_ticks=0
decision_act=0
module_call_count=0
plan_count=0
execute_until_continue_count=0

load_metrics_from_report() {
  local report_path=$1
  local log_path=$2

  if command -v jq >/dev/null 2>&1; then
    active_ticks=$(jq -r '.active_ticks // 0' "$report_path")
    total_decisions=$(jq -r '.total_decisions // 0' "$report_path")
    total_actions=$(jq -r '.total_actions // 0' "$report_path")
    action_success=$(jq -r '.action_success // 0' "$report_path")
    action_failure=$(jq -r '.action_failure // 0' "$report_path")
    llm_errors=$(jq -r '.trace_counts.llm_errors // 0' "$report_path")
    parse_errors=$(jq -r '.trace_counts.parse_errors // 0' "$report_path")
    repair_rounds_total=$(jq -r '.trace_counts.repair_rounds_total // 0' "$report_path")
    repair_rounds_max=$(jq -r '.trace_counts.repair_rounds_max // 0' "$report_path")
    llm_input_chars_total=$(jq -r '.trace_counts.llm_input_chars_total // 0' "$report_path")
    llm_input_chars_avg=$(jq -r '.trace_counts.llm_input_chars_avg // 0' "$report_path")
    llm_input_chars_max=$(jq -r '.trace_counts.llm_input_chars_max // 0' "$report_path")
    clipped_sections=$(jq -r '.trace_counts.prompt_section_clipped // 0' "$report_path")
    decision_wait=$(jq -r '.decision_counts.wait // 0' "$report_path")
    decision_wait_ticks=$(jq -r '.decision_counts.wait_ticks // 0' "$report_path")
    decision_act=$(jq -r '.decision_counts.act // 0' "$report_path")
    module_call_count=$(jq -r '.trace_counts.step_type_counts.module_call // 0' "$report_path")
    plan_count=$(jq -r '.trace_counts.step_type_counts.plan // 0' "$report_path")
    execute_until_continue_count=$(jq -r '.trace_counts.step_type_counts.execute_until_continue // 0' "$report_path")
  elif command -v python3 >/dev/null 2>&1; then
    report_metrics=$(python3 - "$report_path" <<'__PYJSON__'
import json
import sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as fh:
    report = json.load(fh)

def get(path, default=0):
    current = report
    for key in path.split('.'):
        if not isinstance(current, dict):
            return default
        current = current.get(key)
        if current is None:
            return default
    return current

keys = [
    "active_ticks",
    "total_decisions",
    "total_actions",
    "action_success",
    "action_failure",
    "trace_counts.llm_errors",
    "trace_counts.parse_errors",
    "trace_counts.repair_rounds_total",
    "trace_counts.repair_rounds_max",
    "trace_counts.llm_input_chars_total",
    "trace_counts.llm_input_chars_avg",
    "trace_counts.llm_input_chars_max",
    "trace_counts.prompt_section_clipped",
    "decision_counts.wait",
    "decision_counts.wait_ticks",
    "decision_counts.act",
    "trace_counts.step_type_counts.module_call",
    "trace_counts.step_type_counts.plan",
    "trace_counts.step_type_counts.execute_until_continue",
]
for key in keys:
    print(get(key, 0))
__PYJSON__
)
    active_ticks=$(printf '%s\n' "$report_metrics" | sed -n '1p')
    total_decisions=$(printf '%s\n' "$report_metrics" | sed -n '2p')
    total_actions=$(printf '%s\n' "$report_metrics" | sed -n '3p')
    action_success=$(printf '%s\n' "$report_metrics" | sed -n '4p')
    action_failure=$(printf '%s\n' "$report_metrics" | sed -n '5p')
    llm_errors=$(printf '%s\n' "$report_metrics" | sed -n '6p')
    parse_errors=$(printf '%s\n' "$report_metrics" | sed -n '7p')
    repair_rounds_total=$(printf '%s\n' "$report_metrics" | sed -n '8p')
    repair_rounds_max=$(printf '%s\n' "$report_metrics" | sed -n '9p')
    llm_input_chars_total=$(printf '%s\n' "$report_metrics" | sed -n '10p')
    llm_input_chars_avg=$(printf '%s\n' "$report_metrics" | sed -n '11p')
    llm_input_chars_max=$(printf '%s\n' "$report_metrics" | sed -n '12p')
    clipped_sections=$(printf '%s\n' "$report_metrics" | sed -n '13p')
    decision_wait=$(printf '%s\n' "$report_metrics" | sed -n '14p')
    decision_wait_ticks=$(printf '%s\n' "$report_metrics" | sed -n '15p')
    decision_act=$(printf '%s\n' "$report_metrics" | sed -n '16p')
    module_call_count=$(printf '%s\n' "$report_metrics" | sed -n '17p')
    plan_count=$(printf '%s\n' "$report_metrics" | sed -n '18p')
    execute_until_continue_count=$(printf '%s\n' "$report_metrics" | sed -n '19p')
    active_ticks=${active_ticks:-0}
    total_decisions=${total_decisions:-0}
    total_actions=${total_actions:-0}
    action_success=${action_success:-0}
    action_failure=${action_failure:-0}
    llm_errors=${llm_errors:-0}
    parse_errors=${parse_errors:-0}
    repair_rounds_total=${repair_rounds_total:-0}
    repair_rounds_max=${repair_rounds_max:-0}
    llm_input_chars_total=${llm_input_chars_total:-0}
    llm_input_chars_avg=${llm_input_chars_avg:-0}
    llm_input_chars_max=${llm_input_chars_max:-0}
    clipped_sections=${clipped_sections:-0}
    decision_wait=${decision_wait:-0}
    decision_wait_ticks=${decision_wait_ticks:-0}
    decision_act=${decision_act:-0}
    module_call_count=${module_call_count:-0}
    plan_count=${plan_count:-0}
    execute_until_continue_count=${execute_until_continue_count:-0}
  else
    active_ticks=$(extract_metric_from_log "active_ticks" "$log_path" || echo 0)
    total_decisions=$(extract_metric_from_log "total_decisions" "$log_path" || echo 0)
    total_actions=$(extract_metric_from_log "total_actions" "$log_path" || echo 0)
    action_success=$(extract_metric_from_log "action_success" "$log_path" || echo 0)
    action_failure=$(extract_metric_from_log "action_failure" "$log_path" || echo 0)
    llm_errors=$(extract_metric_from_log "llm_errors" "$log_path" || echo 0)
    parse_errors=$(extract_metric_from_log "parse_errors" "$log_path" || echo 0)
    repair_rounds_total=$(extract_metric_from_log "repair_rounds_total" "$log_path" || echo 0)
    repair_rounds_max=$(extract_metric_from_log "repair_rounds_max" "$log_path" || echo 0)
    llm_input_chars_total=$(extract_metric_from_log "llm_input_chars_total" "$log_path" || echo 0)
    llm_input_chars_avg=$(extract_metric_from_log "llm_input_chars_avg" "$log_path" || echo 0)
    llm_input_chars_max=$(extract_metric_from_log "llm_input_chars_max" "$log_path" || echo 0)
    clipped_sections=0
    decision_wait=$(extract_metric_from_log "decision_wait" "$log_path" || echo 0)
    decision_wait_ticks=$(extract_metric_from_log "decision_wait_ticks" "$log_path" || echo 0)
    decision_act=$(extract_metric_from_log "decision_act" "$log_path" || echo 0)
    module_call_count=0
    plan_count=0
    execute_until_continue_count=0
  fi
}

write_summary_file() {
  local summary_path=$1
  local scenario_name=$2
  {
    echo "scenario=$scenario_name"
    echo "ticks=$ticks"
    echo "active_ticks=$active_ticks"
    echo "total_decisions=$total_decisions"
    echo "total_actions=$total_actions"
    echo "action_success=$action_success"
    echo "action_failure=$action_failure"
    echo "llm_errors=$llm_errors"
    echo "parse_errors=$parse_errors"
    echo "repair_rounds_total=$repair_rounds_total"
    echo "repair_rounds_max=$repair_rounds_max"
    echo "llm_input_chars_total=$llm_input_chars_total"
    echo "llm_input_chars_avg=$llm_input_chars_avg"
    echo "llm_input_chars_max=$llm_input_chars_max"
    echo "prompt_section_clipped=$clipped_sections"
    echo "decision_wait=$decision_wait"
    echo "decision_wait_ticks=$decision_wait_ticks"
    echo "decision_act=$decision_act"
    echo "module_call=$module_call_count"
    echo "plan=$plan_count"
    echo "execute_until_continue=$execute_until_continue_count"
    echo "llm_io_logged=$print_llm_io"
    echo "llm_io_max_chars=${llm_io_max_chars:-none}"
    echo "report_json=$scenario_report_json"
    echo "run_log=$scenario_log_file"
  } >"$summary_path"
}

run_scenario_to_log() {
  local scenario_name=$1
  local scenario_report_path=$2
  local scenario_run_log_path=$3
  local -a cmd=(
    env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_llm_agent_demo --
    "$scenario_name"
    --ticks "$ticks"
    --report-json "$scenario_report_path"
  )
  if [[ $print_llm_io -eq 1 ]]; then
    cmd+=(--print-llm-io)
    if [[ -n "$llm_io_max_chars" ]]; then
      cmd+=(--llm-io-max-chars "$llm_io_max_chars")
    fi
  fi

  {
    echo "==== scenario: $scenario_name ===="
    echo "+ ${cmd[*]}"
  } >"$scenario_run_log_path"
  set +e
  "${cmd[@]}" >>"$scenario_run_log_path" 2>&1
  local run_exit=$?
  set -e
  return "$run_exit"
}

wait_parallel_head_job() {
  local pid=${parallel_pids[0]:-}
  local scenario_name=${parallel_scenarios[0]:-unknown}
  if [[ -z "$pid" ]]; then
    return 0
  fi
  local run_exit=0
  set +e
  wait "$pid"
  run_exit=$?
  set -e

  if (( run_exit != 0 )); then
    echo "pressure run failed for scenario=$scenario_name with exit code $run_exit" >&2
    if (( parallel_failed == 0 )); then
      parallel_failed=1
      parallel_failed_exit=$run_exit
    fi
  fi

  if (( ${#parallel_pids[@]} > 1 )); then
    parallel_pids=("${parallel_pids[@]:1}")
    parallel_scenarios=("${parallel_scenarios[@]:1}")
  else
    parallel_pids=()
    parallel_scenarios=()
  fi
}

declare -a scenario_inputs=()
ticks="240"
out_dir=".tmp/llm_stress"
report_json=""
log_file=""
summary_file=""
max_llm_errors="0"
max_parse_errors="0"
max_repair_rounds_max="2"
min_active_ticks=""
print_llm_io=1
llm_io_max_chars=""
keep_out_dir=0
jobs="1"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --scenario)
      append_scenario "${2:-}"
      shift 2
      ;;
    --scenarios)
      append_scenarios_from_csv "${2:-}"
      shift 2
      ;;
    --ticks)
      ticks=${2:-}
      shift 2
      ;;
    --jobs)
      jobs=${2:-}
      shift 2
      ;;
    --out-dir)
      out_dir=${2:-}
      shift 2
      ;;
    --report-json)
      report_json=${2:-}
      shift 2
      ;;
    --log-file)
      log_file=${2:-}
      shift 2
      ;;
    --summary-file)
      summary_file=${2:-}
      shift 2
      ;;
    --max-llm-errors)
      max_llm_errors=${2:-}
      shift 2
      ;;
    --max-parse-errors)
      max_parse_errors=${2:-}
      shift 2
      ;;
    --max-repair-rounds-max)
      max_repair_rounds_max=${2:-}
      shift 2
      ;;
    --min-active-ticks)
      min_active_ticks=${2:-}
      shift 2
      ;;
    --no-llm-io)
      print_llm_io=0
      shift
      ;;
    --llm-io-max-chars)
      llm_io_max_chars=${2:-}
      shift 2
      ;;
    --keep-out-dir)
      keep_out_dir=1
      shift
      ;;
    -h|--help)
      print_help
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      print_help
      exit 2
      ;;
  esac
done

if [[ ${#scenario_inputs[@]} -eq 0 ]]; then
  scenario_inputs=("llm_bootstrap")
fi

declare -a scenarios=()
for candidate in "${scenario_inputs[@]}"; do
  seen=0
  if (( ${#scenarios[@]} > 0 )); then
    for existed in "${scenarios[@]}"; do
      if [[ "$existed" == "$candidate" ]]; then
        seen=1
        break
      fi
    done
  fi
  if (( seen == 0 )); then
    scenarios+=("$candidate")
  fi
done

scenario_count=${#scenarios[@]}
multi_mode=0
if (( scenario_count > 1 )); then
  multi_mode=1
fi
if (( multi_mode == 0 )); then
  jobs=1
elif (( jobs > scenario_count )); then
  jobs=$scenario_count
fi

ensure_positive_int "--ticks" "$ticks"
ensure_positive_int "--jobs" "$jobs"
ensure_positive_int "--max-llm-errors" "$max_llm_errors"
ensure_positive_int "--max-parse-errors" "$max_parse_errors"
ensure_positive_int "--max-repair-rounds-max" "$max_repair_rounds_max"
if [[ -n "$llm_io_max_chars" ]]; then
  ensure_positive_int "--llm-io-max-chars" "$llm_io_max_chars"
fi
if [[ -z "$min_active_ticks" ]]; then
  min_active_ticks="$ticks"
fi
ensure_positive_int "--min-active-ticks" "$min_active_ticks"

if [[ -z "$report_json" ]]; then
  report_json="$out_dir/report.json"
fi
if [[ -z "$log_file" ]]; then
  log_file="$out_dir/run.log"
fi
if [[ -z "$summary_file" ]]; then
  summary_file="$out_dir/summary.txt"
fi

if [[ $keep_out_dir -eq 0 ]]; then
  run rm -rf "$out_dir"
fi
run mkdir -p "$out_dir"
run mkdir -p "$(dirname "$report_json")" "$(dirname "$log_file")" "$(dirname "$summary_file")"

metrics_tsv="$out_dir/scenario_metrics.tsv"
if [[ $multi_mode -eq 1 ]]; then
  : >"$log_file"
  {
    printf '%s\t' scenario
    printf '%s\t' report_json run_log summary_file
    printf '%s\t' active_ticks total_decisions total_actions action_success action_failure
    printf '%s\t' llm_errors parse_errors repair_rounds_total repair_rounds_max
    printf '%s\t' llm_input_chars_total llm_input_chars_avg llm_input_chars_max
    printf '%s\t' prompt_section_clipped decision_wait decision_wait_ticks decision_act
    printf '%s\t' module_call plan execute_until_continue
    printf '%s\n' llm_io_logged
  } >"$metrics_tsv"
fi

parallel_mode=0
if (( multi_mode == 1 && jobs > 1 )); then
  parallel_mode=1
  echo "parallel scenario run enabled: jobs=$jobs"
fi

if (( parallel_mode == 1 )); then
  declare -a parallel_pids=()
  declare -a parallel_scenarios=()
  parallel_failed=0
  parallel_failed_exit=0

  for scenario in "${scenarios[@]}"; do
    scenario_dir="$out_dir/scenarios/$scenario"
    run mkdir -p "$scenario_dir"
    scenario_report_json="$scenario_dir/report.json"
    scenario_log_file="$scenario_dir/run.log"
    run_scenario_to_log "$scenario" "$scenario_report_json" "$scenario_log_file" &
    parallel_pids+=("$!")
    parallel_scenarios+=("$scenario")
    if (( ${#parallel_pids[@]} >= jobs )); then
      wait_parallel_head_job
    fi
  done

  while (( ${#parallel_pids[@]} > 0 )); do
    wait_parallel_head_job
  done

  if (( parallel_failed != 0 )); then
    exit "$parallel_failed_exit"
  fi

  : >"$log_file"
  for scenario in "${scenarios[@]}"; do
    scenario_log_file="$out_dir/scenarios/$scenario/run.log"
    cat "$scenario_log_file" >>"$log_file"
  done
fi

agg_active_ticks=0
agg_total_decisions=0
agg_total_actions=0
agg_action_success=0
agg_action_failure=0
agg_llm_errors=0
agg_parse_errors=0
agg_repair_rounds_total=0
agg_repair_rounds_max_peak=0
agg_llm_input_chars_total=0
agg_llm_input_chars_avg_sum=0
agg_llm_input_chars_max_peak=0
agg_prompt_section_clipped=0
agg_decision_wait=0
agg_decision_wait_ticks=0
agg_decision_act=0
agg_module_call=0
agg_plan=0
agg_execute_until_continue=0

for scenario in "${scenarios[@]}"; do
  if [[ $multi_mode -eq 1 ]]; then
    scenario_dir="$out_dir/scenarios/$scenario"
    run mkdir -p "$scenario_dir"
    scenario_report_json="$scenario_dir/report.json"
    scenario_log_file="$scenario_dir/run.log"
    scenario_summary_file="$scenario_dir/summary.txt"
  else
    scenario_report_json="$report_json"
    scenario_log_file="$log_file"
    scenario_summary_file="$summary_file"
  fi

  if (( parallel_mode == 0 )); then
    cmd=(
      env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_llm_agent_demo --
      "$scenario"
      --ticks "$ticks"
      --report-json "$scenario_report_json"
    )
    if [[ $print_llm_io -eq 1 ]]; then
      cmd+=(--print-llm-io)
      if [[ -n "$llm_io_max_chars" ]]; then
        cmd+=(--llm-io-max-chars "$llm_io_max_chars")
      fi
    fi

    if [[ $multi_mode -eq 1 ]]; then
      {
        echo "==== scenario: $scenario ===="
        echo "+ ${cmd[*]}"
      } | tee -a "$log_file"
      set +e
      "${cmd[@]}" 2>&1 | tee "$scenario_log_file" | tee -a "$log_file"
      run_exit=${PIPESTATUS[0]}
      set -e
    else
      echo "+ ${cmd[*]} | tee $scenario_log_file"
      set +e
      "${cmd[@]}" 2>&1 | tee "$scenario_log_file"
      run_exit=${PIPESTATUS[0]}
      set -e
    fi

    if [[ $run_exit -ne 0 ]]; then
      echo "pressure run failed for scenario=$scenario with exit code $run_exit" >&2
      exit $run_exit
    fi
  fi

  if [[ ! -s "$scenario_report_json" ]]; then
    echo "missing report json for scenario=$scenario: $scenario_report_json" >&2
    exit 3
  fi

  load_metrics_from_report "$scenario_report_json" "$scenario_log_file"
  write_summary_file "$scenario_summary_file" "$scenario"

  echo "pressure summary [$scenario]:"
  cat "$scenario_summary_file"

  if (( active_ticks < min_active_ticks )); then
    echo "failed: scenario=$scenario active_ticks($active_ticks) < min_active_ticks($min_active_ticks)" >&2
    exit 10
  fi
  if (( llm_errors > max_llm_errors )); then
    echo "failed: scenario=$scenario llm_errors($llm_errors) > max_llm_errors($max_llm_errors)" >&2
    exit 11
  fi
  if (( parse_errors > max_parse_errors )); then
    echo "failed: scenario=$scenario parse_errors($parse_errors) > max_parse_errors($max_parse_errors)" >&2
    exit 12
  fi
  if (( repair_rounds_max > max_repair_rounds_max )); then
    echo "failed: scenario=$scenario repair_rounds_max($repair_rounds_max) > max_repair_rounds_max($max_repair_rounds_max)" >&2
    exit 13
  fi

  agg_active_ticks=$((agg_active_ticks + active_ticks))
  agg_total_decisions=$((agg_total_decisions + total_decisions))
  agg_total_actions=$((agg_total_actions + total_actions))
  agg_action_success=$((agg_action_success + action_success))
  agg_action_failure=$((agg_action_failure + action_failure))
  agg_llm_errors=$((agg_llm_errors + llm_errors))
  agg_parse_errors=$((agg_parse_errors + parse_errors))
  agg_repair_rounds_total=$((agg_repair_rounds_total + repair_rounds_total))
  agg_llm_input_chars_total=$((agg_llm_input_chars_total + llm_input_chars_total))
  agg_llm_input_chars_avg_sum=$((agg_llm_input_chars_avg_sum + llm_input_chars_avg))
  agg_prompt_section_clipped=$((agg_prompt_section_clipped + clipped_sections))
  agg_decision_wait=$((agg_decision_wait + decision_wait))
  agg_decision_wait_ticks=$((agg_decision_wait_ticks + decision_wait_ticks))
  agg_decision_act=$((agg_decision_act + decision_act))
  agg_module_call=$((agg_module_call + module_call_count))
  agg_plan=$((agg_plan + plan_count))
  agg_execute_until_continue=$((agg_execute_until_continue + execute_until_continue_count))
  if (( repair_rounds_max > agg_repair_rounds_max_peak )); then
    agg_repair_rounds_max_peak=$repair_rounds_max
  fi
  if (( llm_input_chars_max > agg_llm_input_chars_max_peak )); then
    agg_llm_input_chars_max_peak=$llm_input_chars_max
  fi

  if [[ $multi_mode -eq 1 ]]; then
    {
      printf '%s\t' "$scenario"
      printf '%s\t' "$scenario_report_json" "$scenario_log_file" "$scenario_summary_file"
      printf '%s\t' "$active_ticks" "$total_decisions" "$total_actions" "$action_success" "$action_failure"
      printf '%s\t' "$llm_errors" "$parse_errors" "$repair_rounds_total" "$repair_rounds_max"
      printf '%s\t' "$llm_input_chars_total" "$llm_input_chars_avg" "$llm_input_chars_max"
      printf '%s\t' "$clipped_sections" "$decision_wait" "$decision_wait_ticks" "$decision_act"
      printf '%s\t' "$module_call_count" "$plan_count" "$execute_until_continue_count"
      printf '%s\n' "$print_llm_io"
    } >>"$metrics_tsv"
  fi
done

if [[ $multi_mode -eq 1 ]]; then
  agg_llm_input_chars_avg_mean=$((agg_llm_input_chars_avg_sum / scenario_count))
  scenarios_csv=$(IFS=,; echo "${scenarios[*]}")
  {
    echo "mode=multi_scenario"
    echo "jobs=$jobs"
    echo "scenario_count=$scenario_count"
    echo "scenarios=$scenarios_csv"
    echo "ticks=$ticks"
    echo "active_ticks_total=$agg_active_ticks"
    echo "total_decisions_total=$agg_total_decisions"
    echo "total_actions_total=$agg_total_actions"
    echo "action_success_total=$agg_action_success"
    echo "action_failure_total=$agg_action_failure"
    echo "llm_errors_total=$agg_llm_errors"
    echo "parse_errors_total=$agg_parse_errors"
    echo "repair_rounds_total=$agg_repair_rounds_total"
    echo "repair_rounds_max_peak=$agg_repair_rounds_max_peak"
    echo "llm_input_chars_total=$agg_llm_input_chars_total"
    echo "llm_input_chars_avg_mean=$agg_llm_input_chars_avg_mean"
    echo "llm_input_chars_max_peak=$agg_llm_input_chars_max_peak"
    echo "prompt_section_clipped_total=$agg_prompt_section_clipped"
    echo "decision_wait_total=$agg_decision_wait"
    echo "decision_wait_ticks_total=$agg_decision_wait_ticks"
    echo "decision_act_total=$agg_decision_act"
    echo "module_call_total=$agg_module_call"
    echo "plan_total=$agg_plan"
    echo "execute_until_continue_total=$agg_execute_until_continue"
    echo "llm_io_logged=$print_llm_io"
    echo "llm_io_max_chars=${llm_io_max_chars:-none}"
    echo "report_json=$report_json"
    echo "run_log=$log_file"
    echo "per_scenario_dir=$out_dir/scenarios"
  } >"$summary_file"

  if command -v python3 >/dev/null 2>&1; then
    python3 - "$metrics_tsv" "$report_json" "$ticks" "$scenario_count" "$jobs" "$print_llm_io" "${llm_io_max_chars:-}" <<'__PYAGG__'
import csv
import json
import sys

metrics_tsv, output_path, ticks, scenario_count, jobs, llm_io_logged, llm_io_max_chars = sys.argv[1:]

int_fields = [
    "active_ticks",
    "total_decisions",
    "total_actions",
    "action_success",
    "action_failure",
    "llm_errors",
    "parse_errors",
    "repair_rounds_total",
    "repair_rounds_max",
    "llm_input_chars_total",
    "llm_input_chars_avg",
    "llm_input_chars_max",
    "prompt_section_clipped",
    "decision_wait",
    "decision_wait_ticks",
    "decision_act",
    "module_call",
    "plan",
    "execute_until_continue",
    "llm_io_logged",
]

rows = []
with open(metrics_tsv, "r", encoding="utf-8") as fh:
    reader = csv.DictReader(fh, delimiter="\t")
    for row in reader:
        normalized = dict(row)
        for key in int_fields:
            normalized[key] = int(row.get(key, 0) or 0)
        rows.append(normalized)

def sum_of(key):
    return sum(item[key] for item in rows)

def peak_of(key):
    return max((item[key] for item in rows), default=0)

scenario_names = [item["scenario"] for item in rows]
avg_mean = int(sum_of("llm_input_chars_avg") / max(len(rows), 1))

report = {
    "mode": "multi_scenario",
    "ticks_requested": int(ticks),
    "scenario_count": int(scenario_count),
    "jobs": int(jobs),
    "scenarios": scenario_names,
    "llm_io_logged": int(llm_io_logged),
    "llm_io_max_chars": llm_io_max_chars or "none",
    "totals": {
        "active_ticks": sum_of("active_ticks"),
        "total_decisions": sum_of("total_decisions"),
        "total_actions": sum_of("total_actions"),
        "action_success": sum_of("action_success"),
        "action_failure": sum_of("action_failure"),
        "llm_errors": sum_of("llm_errors"),
        "parse_errors": sum_of("parse_errors"),
        "repair_rounds_total": sum_of("repair_rounds_total"),
        "llm_input_chars_total": sum_of("llm_input_chars_total"),
        "prompt_section_clipped": sum_of("prompt_section_clipped"),
        "decision_wait": sum_of("decision_wait"),
        "decision_wait_ticks": sum_of("decision_wait_ticks"),
        "decision_act": sum_of("decision_act"),
        "module_call": sum_of("module_call"),
        "plan": sum_of("plan"),
        "execute_until_continue": sum_of("execute_until_continue"),
    },
    "peaks": {
        "repair_rounds_max": peak_of("repair_rounds_max"),
        "llm_input_chars_max": peak_of("llm_input_chars_max"),
    },
    "means": {
        "llm_input_chars_avg": avg_mean,
    },
    "per_scenario": rows,
}

with open(output_path, "w", encoding="utf-8") as fh:
    json.dump(report, fh, ensure_ascii=False, indent=2)
__PYAGG__
  else
    cat >"$report_json" <<EOF
{
  "mode": "multi_scenario",
  "ticks_requested": $ticks,
  "scenario_count": $scenario_count,
  "jobs": $jobs
}
EOF
  fi
fi

echo "pressure summary:"
cat "$summary_file"
echo "pressure run passed"
