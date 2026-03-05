#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

run() {
  echo "+ $*"
  "$@"
}

latest_summary() {
  local root=$1
  ls -t "$root"/*/release-gate-summary.md 2>/dev/null | head -n 1 || true
}

ensure_file_contains() {
  local file=$1
  local pattern=$2
  if ! rg -q -- "$pattern" "$file"; then
    echo "error: pattern not found: $pattern" >&2
    echo "  file=$file" >&2
    exit 1
  fi
}

smoke_root=".tmp/release_gate_smoke"
pass_root="$smoke_root/pass"
fail_root="$smoke_root/fail"
mkdir -p "$pass_root" "$fail_root"

run ./scripts/release-gate.sh --dry-run --out-dir "$pass_root"
pass_summary=$(latest_summary "$pass_root")
if [[ -z "$pass_summary" ]]; then
  echo "error: pass summary not found under $pass_root" >&2
  exit 1
fi
ensure_file_contains "$pass_summary" "- Overall: PASS"
ensure_file_contains "$pass_summary" "- web_strict: passed \\(dry_run\\)"
ensure_file_contains "$pass_summary" "- s10: passed \\(dry_run\\)"

failure_log="$fail_root/failure.log"
set +e
./scripts/release-gate.sh \
  --dry-run \
  --dry-run-fail-step web_strict \
  --out-dir "$fail_root" \
  >"$failure_log" 2>&1
failure_code=$?
set -e

if [[ "$failure_code" -eq 0 ]]; then
  echo "error: expected failure when injecting dry-run fail step" >&2
  exit 1
fi
ensure_file_contains "$failure_log" "error: release gate failed at step: web_strict"
ensure_file_contains "$failure_log" "hint: inspect step log:"
ensure_file_contains "$failure_log" "hint: gate summary:"

fail_summary=$(latest_summary "$fail_root")
if [[ -z "$fail_summary" ]]; then
  echo "error: fail summary not found under $fail_root" >&2
  exit 1
fi
ensure_file_contains "$fail_summary" "- Overall: FAIL"
ensure_file_contains "$fail_summary" "- web_strict: failed \\(simulated_dry_run_failure\\)"

echo "release gate smoke checks passed"
