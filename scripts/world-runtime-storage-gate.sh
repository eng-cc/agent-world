#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

usage() {
  cat <<'USAGE'
Usage: ./scripts/world-runtime-storage-gate.sh [options]

Purpose:
  Evaluate runtime storage/GC/replay gate from either:
  - a storage metrics JSON file (`reward-runtime-storage-metrics.json`), or
  - a `/v1/chain/status` response JSON containing `.storage`.

Outputs:
  <out-dir>/<timestamp>/
    summary.md
    summary.json

Options:
  --metrics-file <path>          Path to storage metrics JSON
  --status-json <path>           Path to /v1/chain/status JSON
  --status-url <url>             Fetch /v1/chain/status JSON from URL
  --expected-profile <name>      Expected storage profile (dev_local|release_default|soak_forensics)
  --min-checkpoint-count <n>     Minimum checkpoint_count required (default: 1)
  --max-orphan-blob-count <n>    Maximum orphan_blob_count allowed (default: 0)
  --min-retained-height <n>      Minimum latest_retained_height required (optional)
  --require-gc-clean             Require last_gc_result != failed
  --require-no-degraded          Require degraded_reason to be empty
  --out-dir <path>               Output root (default: .tmp/world_runtime_storage_gate)
  --dry-run                      Record config only, skip evaluation
  -h, --help                     Show help
USAGE
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "error: required command not found: $1" >&2
    exit 2
  }
}

ensure_non_negative_int() {
  local flag=$1
  local value=$2
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    echo "error: $flag must be a non-negative integer (got: $value)" >&2
    exit 2
  fi
}

metrics_file=""
status_json=""
status_url=""
expected_profile=""
min_checkpoint_count=1
max_orphan_blob_count=0
min_retained_height=""
require_gc_clean=0
require_no_degraded=0
out_dir=".tmp/world_runtime_storage_gate"
dry_run=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --metrics-file)
      metrics_file=${2:-}
      shift 2
      ;;
    --status-json)
      status_json=${2:-}
      shift 2
      ;;
    --status-url)
      status_url=${2:-}
      shift 2
      ;;
    --expected-profile)
      expected_profile=${2:-}
      shift 2
      ;;
    --min-checkpoint-count)
      min_checkpoint_count=${2:-}
      shift 2
      ;;
    --max-orphan-blob-count)
      max_orphan_blob_count=${2:-}
      shift 2
      ;;
    --min-retained-height)
      min_retained_height=${2:-}
      shift 2
      ;;
    --require-gc-clean)
      require_gc_clean=1
      shift
      ;;
    --require-no-degraded)
      require_no_degraded=1
      shift
      ;;
    --out-dir)
      out_dir=${2:-}
      shift 2
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

ensure_non_negative_int --min-checkpoint-count "$min_checkpoint_count"
ensure_non_negative_int --max-orphan-blob-count "$max_orphan_blob_count"
if [[ -n "$min_retained_height" ]]; then
  ensure_non_negative_int --min-retained-height "$min_retained_height"
fi

inputs=0
[[ -n "$metrics_file" ]] && inputs=$((inputs + 1))
[[ -n "$status_json" ]] && inputs=$((inputs + 1))
[[ -n "$status_url" ]] && inputs=$((inputs + 1))
if (( inputs != 1 )); then
  echo "error: provide exactly one of --metrics-file, --status-json, or --status-url" >&2
  exit 2
fi

need_cmd python3
if [[ -n "$status_url" ]]; then
  need_cmd curl
fi

timestamp=$(date '+%Y%m%d-%H%M%S')
run_dir="$out_dir/$timestamp"
mkdir -p "$run_dir"
summary_md="$run_dir/summary.md"
summary_json="$run_dir/summary.json"
source_json="$run_dir/source.json"

if [[ "$dry_run" -eq 1 ]]; then
  cat > "$summary_md" <<DRY
# Runtime Storage Gate Summary

- status: `dry_run`
- expected_profile: `${expected_profile:-auto}`
- min_checkpoint_count: `$min_checkpoint_count`
- max_orphan_blob_count: `$max_orphan_blob_count`
- min_retained_height: `${min_retained_height:-unset}`
- require_gc_clean: `$require_gc_clean`
- require_no_degraded: `$require_no_degraded`
DRY
  python3 - <<PY > "$summary_json"
import json
print(json.dumps({
  "status": "dry_run",
  "expected_profile": ${expected_profile:+"$expected_profile" or None},
  "min_checkpoint_count": $min_checkpoint_count,
  "max_orphan_blob_count": $max_orphan_blob_count,
  "min_retained_height": ${min_retained_height:-None},
  "require_gc_clean": bool($require_gc_clean),
  "require_no_degraded": bool($require_no_degraded)
}, ensure_ascii=False, indent=2))
PY
  echo "runtime storage gate summary: $summary_md"
  exit 0
fi

if [[ -n "$status_url" ]]; then
  curl -fsS "$status_url" > "$source_json"
elif [[ -n "$status_json" ]]; then
  cp "$status_json" "$source_json"
else
  cp "$metrics_file" "$source_json"
fi

python3 - "$source_json" "$summary_md" "$summary_json" "$expected_profile" "$min_checkpoint_count" "$max_orphan_blob_count" "$min_retained_height" "$require_gc_clean" "$require_no_degraded" <<'PY'
import json, pathlib, sys

source_path = pathlib.Path(sys.argv[1])
summary_md = pathlib.Path(sys.argv[2])
summary_json = pathlib.Path(sys.argv[3])
expected_profile = sys.argv[4] or None
min_checkpoint_count = int(sys.argv[5])
max_orphan_blob_count = int(sys.argv[6])
min_retained_height = int(sys.argv[7]) if sys.argv[7] else None
require_gc_clean = sys.argv[8] == '1'
require_no_degraded = sys.argv[9] == '1'

raw = json.loads(source_path.read_text(encoding='utf-8'))
storage = raw.get('storage', raw)

checks = []
def add(name, passed, actual, expected, note=""):
    checks.append({
        "name": name,
        "passed": bool(passed),
        "actual": actual,
        "expected": expected,
        "note": note,
    })

profile = storage.get('storage_profile')
effective_budget = (storage.get('effective_budget') or {}).get('profile')
checkpoint_count = storage.get('checkpoint_count', 0)
orphan_blob_count = storage.get('orphan_blob_count', 0)
replay_summary = storage.get('replay_summary') or {}
latest_retained_height = replay_summary.get('latest_retained_height')
replay_mode = replay_summary.get('mode')
last_gc_result = storage.get('last_gc_result')
degraded_reason = storage.get('degraded_reason')

if expected_profile:
    add('storage_profile', profile == expected_profile, profile, expected_profile)
else:
    add('storage_profile_matches_budget', profile == effective_budget, profile, effective_budget)

add('checkpoint_count', checkpoint_count >= min_checkpoint_count, checkpoint_count, f'>={min_checkpoint_count}')
add('orphan_blob_count', orphan_blob_count <= max_orphan_blob_count, orphan_blob_count, f'<={max_orphan_blob_count}')
add('replay_summary_mode', replay_mode in {'latest_only', 'full_log_only', 'checkpoint_plus_log'}, replay_mode, 'one of latest_only/full_log_only/checkpoint_plus_log')
if min_retained_height is not None:
    add('latest_retained_height', isinstance(latest_retained_height, int) and latest_retained_height >= min_retained_height, latest_retained_height, f'>={min_retained_height}')
if require_gc_clean:
    add('last_gc_result', last_gc_result != 'failed', last_gc_result, '!= failed')
if require_no_degraded:
    add('degraded_reason', degraded_reason in (None, ''), degraded_reason, 'empty')

overall = 'PASS' if all(item['passed'] for item in checks) else 'FAIL'

summary = {
    'status': overall,
    'source_json': str(source_path),
    'storage_profile': profile,
    'effective_budget_profile': effective_budget,
    'checkpoint_count': checkpoint_count,
    'orphan_blob_count': orphan_blob_count,
    'latest_retained_height': latest_retained_height,
    'replay_summary_mode': replay_mode,
    'last_gc_result': last_gc_result,
    'degraded_reason': degraded_reason,
    'checks': checks,
}
summary_json.write_text(json.dumps(summary, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')

lines = [
    '# Runtime Storage Gate Summary',
    '',
    f'- status: `{overall}`',
    f'- source_json: `{source_path}`',
    f'- storage_profile: `{profile}`',
    f'- effective_budget.profile: `{effective_budget}`',
    f'- checkpoint_count: `{checkpoint_count}`',
    f'- orphan_blob_count: `{orphan_blob_count}`',
    f'- replay_summary.mode: `{replay_mode}`',
    f'- latest_retained_height: `{latest_retained_height}`',
    f'- last_gc_result: `{last_gc_result}`',
    f'- degraded_reason: `{degraded_reason}`',
    '',
    '| check | result | actual | expected | note |',
    '| --- | --- | --- | --- | --- |',
]
for item in checks:
    lines.append(f"| {item['name']} | {'PASS' if item['passed'] else 'FAIL'} | `{item['actual']}` | `{item['expected']}` | {item['note']} |")
summary_md.write_text('\n'.join(lines) + '\n', encoding='utf-8')
PY

echo "runtime storage gate summary: $summary_md"
