#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

usage() {
  cat <<'USAGE'
Usage: ./scripts/wasm-release-evidence-report.sh [options]

Purpose:
  Collect and verify Docker canonical WASM release evidence for builtin module sets.
  The script can:
  - collect current-runner summaries for m1/m4/m5
  - verify per-module-set multi-runner summary directories
  - write a human/machine readable evidence report

Outputs:
  <out-dir>/<timestamp>/
    summary.md
    summary.json
    module_sets.tsv
    summaries/<module-set>/<runner>.json
    logs/<module-set>.verify.log

Options:
  --out-dir <path>           Output root (default: .tmp/wasm_release_evidence_report)
  --module-sets <csv>        Module sets to process (default: m1,m4,m5)
  --runner-label <label>     Runner label used for collection (default: detected host platform)
  --expected-runners <csv>   Expected runner labels for verify (default: current runner only)
  --skip-collect             Verify/report only; do not collect current-runner summaries
  --dry-run                  Print actions and write placeholder report without execution
  -h, --help                 Show help
USAGE
}

normalize_platform_os() {
  local raw="$1"
  case "$raw" in
    Darwin) echo "darwin" ;;
    Linux) echo "linux" ;;
    *) echo "$raw" | tr '[:upper:]' '[:lower:]' ;;
  esac
}

normalize_platform_arch() {
  local raw="$1"
  case "$raw" in
    arm64|aarch64) echo "arm64" ;;
    x86_64|amd64) echo "x86_64" ;;
    *) echo "$raw" ;;
  esac
}

detect_host_platform() {
  local os arch
  os="$(normalize_platform_os "$(uname -s)")"
  arch="$(normalize_platform_arch "$(uname -m)")"
  echo "${os}-${arch}"
}

ensure_csv_non_empty() {
  local flag="$1"
  local csv="$2"
  if [[ -z "$csv" ]]; then
    echo "error: $flag must not be empty" >&2
    exit 2
  fi
}

format_cmd() {
  local formatted=""
  local token=""
  for token in "$@"; do
    local quoted=""
    printf -v quoted '%q' "$token"
    if [[ -z "$formatted" ]]; then
      formatted="$quoted"
    else
      formatted="$formatted $quoted"
    fi
  done
  printf '%s' "$formatted"
}

out_dir=".tmp/wasm_release_evidence_report"
module_sets_csv="m1,m4,m5"
runner_label=""
expected_runners_csv=""
skip_collect=0
dry_run=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out-dir)
      out_dir=${2:-}
      shift 2
      ;;
    --module-sets)
      module_sets_csv=${2:-}
      shift 2
      ;;
    --runner-label)
      runner_label=${2:-}
      shift 2
      ;;
    --expected-runners)
      expected_runners_csv=${2:-}
      shift 2
      ;;
    --skip-collect)
      skip_collect=1
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
      echo "error: unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ -z "$runner_label" ]]; then
  runner_label="$(detect_host_platform)"
fi
if [[ -z "$expected_runners_csv" ]]; then
  expected_runners_csv="$runner_label"
fi
ensure_csv_non_empty "--module-sets" "$module_sets_csv"
ensure_csv_non_empty "--expected-runners" "$expected_runners_csv"

timestamp=$(date '+%Y%m%d-%H%M%S')
run_dir="$out_dir/$timestamp"
summary_md="$run_dir/summary.md"
summary_json="$run_dir/summary.json"
module_sets_tsv="$run_dir/module_sets.tsv"
logs_dir="$run_dir/logs"
summaries_dir="$run_dir/summaries"
mkdir -p "$logs_dir" "$summaries_dir"
: > "$module_sets_tsv"

overall_status="PASS"

IFS=',' read -r -a module_sets <<< "$module_sets_csv"

for module_set in "${module_sets[@]}"; do
  module_set="$(echo "$module_set" | xargs)"
  [[ -n "$module_set" ]] || continue

  module_summary_dir="$summaries_dir/$module_set"
  module_summary_path="$module_summary_dir/$runner_label.json"
  verify_log="$logs_dir/${module_set}.verify.log"
  mkdir -p "$module_summary_dir"

  collect_cmd=(
    ./scripts/ci-m1-wasm-summary.sh
    --module-set "$module_set"
    --runner-label "$runner_label"
    --out "$module_summary_path"
  )
  verify_cmd=(
    python3 ./scripts/ci-verify-m1-wasm-summaries.py
    --module-set "$module_set"
    --summary-dir "$module_summary_dir"
    --expected-runners "$expected_runners_csv"
  )

  collect_status="skipped"
  verify_status="skipped"
  module_note="ok"

  {
    echo "module_set=$module_set"
    echo "runner_label=$runner_label"
    echo "expected_runners=$expected_runners_csv"
    echo "collect_cmd=$(format_cmd "${collect_cmd[@]}")"
    echo "verify_cmd=$(format_cmd "${verify_cmd[@]}")"
  } > "$verify_log"

  if [[ "$dry_run" -eq 1 ]]; then
    if [[ "$skip_collect" -eq 0 ]]; then
      echo "+ $(format_cmd "${collect_cmd[@]}") (dry-run)"
      collect_status="dry_run"
    fi
    echo "+ $(format_cmd "${verify_cmd[@]}") (dry-run)"
    verify_status="dry_run"
    module_note="dry_run"
  else
    if [[ "$skip_collect" -eq 0 ]]; then
      collect_status="passed"
      set +e
      {
        echo "+ $(format_cmd "${collect_cmd[@]}")"
        "${collect_cmd[@]}"
      } >> "$verify_log" 2>&1
      code=$?
      set -e
      if [[ "$code" -ne 0 ]]; then
        collect_status="failed"
        verify_status="skipped"
        module_note="collect_exit_${code}"
        overall_status="FAIL"
      fi
    fi

    if [[ "$collect_status" != "failed" ]]; then
      verify_status="passed"
      set +e
      {
        echo "+ $(format_cmd "${verify_cmd[@]}")"
        "${verify_cmd[@]}"
      } >> "$verify_log" 2>&1
      code=$?
      set -e
      if [[ "$code" -ne 0 ]]; then
        verify_status="failed"
        module_note="verify_exit_${code}"
        overall_status="FAIL"
      fi
    fi
  fi

  printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$module_set" \
    "$collect_status" \
    "$verify_status" \
    "$module_note" \
    "$module_summary_dir" \
    "$verify_log" \
    >> "$module_sets_tsv"
done

python3 - "$module_sets_tsv" "$summary_json" "$run_dir" "$runner_label" "$expected_runners_csv" "$overall_status" "$skip_collect" "$dry_run" <<'PY'
import json
import pathlib
import sys

module_sets_tsv, summary_json, run_dir, runner_label, expected_runners_csv, overall_status, skip_collect, dry_run = sys.argv[1:]

module_sets = []
with open(module_sets_tsv, "r", encoding="utf-8") as fh:
    for raw in fh:
        module_set, collect_status, verify_status, note, summary_dir, verify_log = raw.rstrip("\n").split("\t")
        summary_paths = sorted(pathlib.Path(summary_dir).glob("*.json"))
        found_runners = []
        module_count = None
        if summary_paths:
            try:
                payload = json.loads(summary_paths[0].read_text())
                module_count = payload.get("module_count")
            except Exception:
                module_count = None
            found_runners = [path.stem for path in summary_paths]
        module_sets.append(
            {
                "module_set": module_set,
                "collect_status": collect_status,
                "verify_status": verify_status,
                "note": note,
                "summary_dir": summary_dir,
                "verify_log": verify_log,
                "found_runners": found_runners,
                "module_count": module_count,
            }
        )

payload = {
    "run_dir": run_dir,
    "runner_label": runner_label,
    "expected_runners": [item for item in expected_runners_csv.split(",") if item],
    "overall_status": overall_status,
    "skip_collect": skip_collect == "1",
    "dry_run": dry_run == "1",
    "module_sets": module_sets,
}
with open(summary_json, "w", encoding="utf-8") as fh:
    json.dump(payload, fh, ensure_ascii=True, indent=2)
PY

{
  echo "# WASM Release Evidence Report"
  echo ""
  echo "- Timestamp: $(date '+%Y-%m-%d %H:%M:%S %Z')"
  echo "- Run dir: \`$run_dir\`"
  echo "- Runner label: \`$runner_label\`"
  echo "- Expected runners: \`$expected_runners_csv\`"
  echo "- Skip collect: \`$skip_collect\`"
  echo "- Dry run: \`$dry_run\`"
  echo "- Overall: $overall_status"
  echo ""
  echo "## Module Sets"
  while IFS=$'\t' read -r module_set collect_status verify_status note summary_dir verify_log; do
    [[ -n "$module_set" ]] || continue
    echo "- $module_set: collect=$collect_status verify=$verify_status note=$note"
    echo "  summary_dir: \`$summary_dir\`"
    echo "  verify_log: \`$verify_log\`"
  done < "$module_sets_tsv"
} > "$summary_md"

echo "wasm release evidence summary: $summary_md"
echo "wasm release evidence summary json: $summary_json"
