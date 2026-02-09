#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

print_help() {
  cat <<'USAGE'
Usage: ./scripts/llm-longrun-stress.sh [options]

Options:
  --scenario <name>            Scenario for world_llm_agent_demo (default: llm_bootstrap)
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

scenario="llm_bootstrap"
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

while [[ $# -gt 0 ]]; do
  case "$1" in
    --scenario)
      scenario=${2:-}
      shift 2
      ;;
    --ticks)
      ticks=${2:-}
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

ensure_positive_int "--ticks" "$ticks"
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

cmd=(
  env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_llm_agent_demo --
  "$scenario"
  --ticks "$ticks"
  --report-json "$report_json"
)
if [[ $print_llm_io -eq 1 ]]; then
  cmd+=(--print-llm-io)
  if [[ -n "$llm_io_max_chars" ]]; then
    cmd+=(--llm-io-max-chars "$llm_io_max_chars")
  fi
fi

echo "+ ${cmd[*]} | tee $log_file"
set +e
"${cmd[@]}" 2>&1 | tee "$log_file"
run_exit=${PIPESTATUS[0]}
set -e
if [[ $run_exit -ne 0 ]]; then
  echo "pressure run failed with exit code $run_exit" >&2
  exit $run_exit
fi

if [[ ! -s "$report_json" ]]; then
  echo "missing report json: $report_json" >&2
  exit 3
fi

if command -v jq >/dev/null 2>&1; then
  active_ticks=$(jq -r '.active_ticks // 0' "$report_json")
  total_decisions=$(jq -r '.total_decisions // 0' "$report_json")
  total_actions=$(jq -r '.total_actions // 0' "$report_json")
  llm_errors=$(jq -r '.trace_counts.llm_errors // 0' "$report_json")
  parse_errors=$(jq -r '.trace_counts.parse_errors // 0' "$report_json")
  repair_rounds_total=$(jq -r '.trace_counts.repair_rounds_total // 0' "$report_json")
  repair_rounds_max=$(jq -r '.trace_counts.repair_rounds_max // 0' "$report_json")
  llm_input_chars_avg=$(jq -r '.trace_counts.llm_input_chars_avg // 0' "$report_json")
  llm_input_chars_max=$(jq -r '.trace_counts.llm_input_chars_max // 0' "$report_json")
  clipped_sections=$(jq -r '.trace_counts.prompt_section_clipped // 0' "$report_json")
  decision_wait=$(jq -r '.decision_counts.wait // 0' "$report_json")
  decision_wait_ticks=$(jq -r '.decision_counts.wait_ticks // 0' "$report_json")
  decision_act=$(jq -r '.decision_counts.act // 0' "$report_json")
elif command -v python3 >/dev/null 2>&1; then
  report_metrics=$(python3 - "$report_json" <<'__PYJSON__'
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
    "trace_counts.llm_errors",
    "trace_counts.parse_errors",
    "trace_counts.repair_rounds_total",
    "trace_counts.repair_rounds_max",
    "trace_counts.llm_input_chars_avg",
    "trace_counts.llm_input_chars_max",
    "trace_counts.prompt_section_clipped",
    "decision_counts.wait",
    "decision_counts.wait_ticks",
    "decision_counts.act",
]
for key in keys:
    print(get(key, 0))
__PYJSON__
)
  active_ticks=$(printf '%s\n' "$report_metrics" | sed -n '1p')
  total_decisions=$(printf '%s\n' "$report_metrics" | sed -n '2p')
  total_actions=$(printf '%s\n' "$report_metrics" | sed -n '3p')
  llm_errors=$(printf '%s\n' "$report_metrics" | sed -n '4p')
  parse_errors=$(printf '%s\n' "$report_metrics" | sed -n '5p')
  repair_rounds_total=$(printf '%s\n' "$report_metrics" | sed -n '6p')
  repair_rounds_max=$(printf '%s\n' "$report_metrics" | sed -n '7p')
  llm_input_chars_avg=$(printf '%s\n' "$report_metrics" | sed -n '8p')
  llm_input_chars_max=$(printf '%s\n' "$report_metrics" | sed -n '9p')
  clipped_sections=$(printf '%s\n' "$report_metrics" | sed -n '10p')
  decision_wait=$(printf '%s\n' "$report_metrics" | sed -n '11p')
  decision_wait_ticks=$(printf '%s\n' "$report_metrics" | sed -n '12p')
  decision_act=$(printf '%s\n' "$report_metrics" | sed -n '13p')
  active_ticks=${active_ticks:-0}
  total_decisions=${total_decisions:-0}
  total_actions=${total_actions:-0}
  llm_errors=${llm_errors:-0}
  parse_errors=${parse_errors:-0}
  repair_rounds_total=${repair_rounds_total:-0}
  repair_rounds_max=${repair_rounds_max:-0}
  llm_input_chars_avg=${llm_input_chars_avg:-0}
  llm_input_chars_max=${llm_input_chars_max:-0}
  clipped_sections=${clipped_sections:-0}
  decision_wait=${decision_wait:-0}
  decision_wait_ticks=${decision_wait_ticks:-0}
  decision_act=${decision_act:-0}
else
  active_ticks=$(extract_metric_from_log "active_ticks" "$log_file" || echo 0)
  total_decisions=$(extract_metric_from_log "total_decisions" "$log_file" || echo 0)
  total_actions=$(extract_metric_from_log "total_actions" "$log_file" || echo 0)
  llm_errors=$(extract_metric_from_log "llm_errors" "$log_file" || echo 0)
  parse_errors=$(extract_metric_from_log "parse_errors" "$log_file" || echo 0)
  repair_rounds_total=$(extract_metric_from_log "repair_rounds_total" "$log_file" || echo 0)
  repair_rounds_max=$(extract_metric_from_log "repair_rounds_max" "$log_file" || echo 0)
  llm_input_chars_avg=$(extract_metric_from_log "llm_input_chars_avg" "$log_file" || echo 0)
  llm_input_chars_max=$(extract_metric_from_log "llm_input_chars_max" "$log_file" || echo 0)
  clipped_sections=0
  decision_wait=$(extract_metric_from_log "decision_wait" "$log_file" || echo 0)
  decision_wait_ticks=$(extract_metric_from_log "decision_wait_ticks" "$log_file" || echo 0)
  decision_act=$(extract_metric_from_log "decision_act" "$log_file" || echo 0)
fi

{
  echo "scenario=$scenario"
  echo "ticks=$ticks"
  echo "active_ticks=$active_ticks"
  echo "total_decisions=$total_decisions"
  echo "total_actions=$total_actions"
  echo "llm_errors=$llm_errors"
  echo "parse_errors=$parse_errors"
  echo "repair_rounds_total=$repair_rounds_total"
  echo "repair_rounds_max=$repair_rounds_max"
  echo "llm_input_chars_avg=$llm_input_chars_avg"
  echo "llm_input_chars_max=$llm_input_chars_max"
  echo "prompt_section_clipped=$clipped_sections"
  echo "decision_wait=$decision_wait"
  echo "decision_wait_ticks=$decision_wait_ticks"
  echo "decision_act=$decision_act"
  echo "llm_io_logged=$print_llm_io"
  echo "llm_io_max_chars=${llm_io_max_chars:-none}"
  echo "report_json=$report_json"
  echo "run_log=$log_file"
} > "$summary_file"

echo "pressure summary:"
cat "$summary_file"

if (( active_ticks < min_active_ticks )); then
  echo "failed: active_ticks($active_ticks) < min_active_ticks($min_active_ticks)" >&2
  exit 10
fi
if (( llm_errors > max_llm_errors )); then
  echo "failed: llm_errors($llm_errors) > max_llm_errors($max_llm_errors)" >&2
  exit 11
fi
if (( parse_errors > max_parse_errors )); then
  echo "failed: parse_errors($parse_errors) > max_parse_errors($max_parse_errors)" >&2
  exit 12
fi
if (( repair_rounds_max > max_repair_rounds_max )); then
  echo "failed: repair_rounds_max($repair_rounds_max) > max_repair_rounds_max($max_repair_rounds_max)" >&2
  exit 13
fi

echo "pressure run passed"
