#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

usage() {
  cat <<'USAGE'
Usage: ./scripts/viewer-release-full-coverage.sh [options]

Purpose:
  Run a release-grade full coverage gate for viewer/game quality:
  1) Web usability/semantic gate
  2) Visual asset/theme gate
  3) Gameplay loop coverage gate (industrial + governance/crisis/economic)

Options:
  --scenario <name>             Scenario for viewer/live + gameplay gate (default: llm_bootstrap)
  --theme-pack <name>           Theme pack: industrial_v2,industrial_v1 (default: industrial_v2)
  --variants <list>             Theme variants: default,matte,glossy,all (default: default,matte,glossy)
  --inspect <list>              Texture inspector entities: agent,location,asset,power_plant,power_storage,all (default: all)
  --ticks-industrial <n>        Industrial loop ticks (default: 100)
  --base-port <port>            Base port for native capture scripts (default: 6423)
  --out-dir <path>              Output root (default: output/playwright/viewer/release_full/<timestamp>)
  --quick                       Quick mode: smaller samples and shorter ticks for smoke checks
  --headed                      Run web browser in headed mode for web gate
  --skip-web-qa                 Skip web usability gate
  --skip-web-visual-baseline    Skip viewer-visual-baseline in web gate
  --skip-theme-preview          Skip theme multi-variant preview
  --skip-texture-inspector      Skip texture inspector matrix capture
  --skip-gameplay               Skip gameplay loop gates
  --no-prewarm                  Pass --no-prewarm to theme/texture capture scripts
  --keep-out-dir                Allow writing into a non-empty out dir
  -h, --help                    Show help

Outputs:
  <out-dir>/web_qa/*
  <out-dir>/theme_preview/*
  <out-dir>/texture_inspector/*
  <out-dir>/gameplay_industrial/*
  <out-dir>/gameplay_governance/*
  <out-dir>/release-full-summary-<timestamp>.md
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

resolve_variants() {
  local raw=$1
  local normalized
  normalized=$(printf '%s' "$raw" | tr '[:upper:]' '[:lower:]')
  if [[ -z "$normalized" || "$normalized" == "all" ]]; then
    echo "default matte glossy"
    return 0
  fi

  local parsed=()
  local item=""
  IFS=',' read -r -a parsed <<<"$normalized"
  for item in "${parsed[@]}"; do
    item=$(trim_whitespace "$item")
    case "$item" in
      default|matte|glossy)
        ;;
      *)
        echo "invalid --variants item: $item" >&2
        echo "supported: default,matte,glossy,all" >&2
        exit 2
        ;;
    esac
  done
  echo "${parsed[*]}"
}

resolve_inspect_entities() {
  local raw=$1
  local normalized
  normalized=$(printf '%s' "$raw" | tr '[:upper:]' '[:lower:]')
  if [[ -z "$normalized" || "$normalized" == "all" ]]; then
    echo "agent location asset power_plant power_storage"
    return 0
  fi

  local parsed=()
  local item=""
  IFS=',' read -r -a parsed <<<"$normalized"
  for item in "${parsed[@]}"; do
    item=$(trim_whitespace "$item")
    case "$item" in
      agent|location|asset|power_plant|power_storage)
        ;;
      *)
        echo "invalid --inspect item: $item" >&2
        echo "supported: agent,location,asset,power_plant,power_storage,all" >&2
        exit 2
        ;;
    esac
  done
  echo "${parsed[*]}"
}

run_step() {
  local status_var=$1
  local note_var=$2
  shift 2
  set +e
  "$@"
  local code=$?
  set -e
  if [[ "$code" -eq 0 ]]; then
    printf -v "$status_var" '%s' "passed"
    printf -v "$note_var" '%s' "ok"
  else
    printf -v "$status_var" '%s' "failed"
    printf -v "$note_var" '%s' "exit_code=$code"
    overall_pass=0
  fi
}

fail_step() {
  local status_var=$1
  local note_var=$2
  local message=$3
  printf -v "$status_var" '%s' "failed"
  printf -v "$note_var" '%s' "$message"
  overall_pass=0
}

extract_summary_value() {
  local file_path=$1
  local key=$2
  if [[ ! -f "$file_path" ]]; then
    echo ""
    return 0
  fi
  grep -E "^${key}=" "$file_path" | tail -n 1 | cut -d'=' -f2-
}

count_connected_captures() {
  local capture_root=$1
  local status_count=0
  local connected_count=0
  local capture_status_file=""
  local status_value=""
  local snapshot_ready_value=""

  while IFS= read -r capture_status_file; do
    [[ -z "$capture_status_file" ]] && continue
    status_count=$((status_count + 1))
    status_value=$(extract_summary_value "$capture_status_file" "connection_status")
    snapshot_ready_value=$(extract_summary_value "$capture_status_file" "snapshot_ready")
    if [[ "$status_value" == "connected" && "$snapshot_ready_value" == "1" ]]; then
      connected_count=$((connected_count + 1))
    fi
  done < <(find "$capture_root" -type f -name 'capture_status.txt' | sort)

  echo "$status_count $connected_count"
}

scenario="llm_bootstrap"
theme_pack="industrial_v2"
variants_raw="default,matte,glossy"
inspect_raw="all"
ticks_industrial=100
base_port=6423
out_dir=""
quick=0
headed=0
skip_web_qa=0
skip_web_visual_baseline=0
skip_theme_preview=0
skip_texture_inspector=0
skip_gameplay=0
no_prewarm=0
keep_out_dir=0

user_set_variants=0
user_set_inspect=0
user_set_ticks_industrial=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --scenario)
      scenario=${2:-}
      shift 2
      ;;
    --theme-pack)
      theme_pack=${2:-}
      shift 2
      ;;
    --variants)
      variants_raw=${2:-}
      user_set_variants=1
      shift 2
      ;;
    --inspect)
      inspect_raw=${2:-}
      user_set_inspect=1
      shift 2
      ;;
    --ticks-industrial)
      ticks_industrial=${2:-}
      user_set_ticks_industrial=1
      shift 2
      ;;
    --base-port)
      base_port=${2:-}
      shift 2
      ;;
    --out-dir)
      out_dir=${2:-}
      shift 2
      ;;
    --quick)
      quick=1
      shift
      ;;
    --headed)
      headed=1
      shift
      ;;
    --skip-web-qa)
      skip_web_qa=1
      shift
      ;;
    --skip-web-visual-baseline)
      skip_web_visual_baseline=1
      shift
      ;;
    --skip-theme-preview)
      skip_theme_preview=1
      shift
      ;;
    --skip-texture-inspector)
      skip_texture_inspector=1
      shift
      ;;
    --skip-gameplay)
      skip_gameplay=1
      shift
      ;;
    --no-prewarm)
      no_prewarm=1
      shift
      ;;
    --keep-out-dir)
      keep_out_dir=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

ensure_positive_int "--ticks-industrial" "$ticks_industrial"
ensure_positive_int "--base-port" "$base_port"

if (( quick == 1 )); then
  if (( user_set_variants == 0 )); then
    variants_raw="default"
  fi
  if (( user_set_inspect == 0 )); then
    inspect_raw="agent,location,asset"
  fi
  if (( user_set_ticks_industrial == 0 )); then
    ticks_industrial=36
  fi
  if (( skip_web_visual_baseline == 0 )); then
    skip_web_visual_baseline=1
  fi
fi

IFS=' ' read -r -a variants <<<"$(resolve_variants "$variants_raw")"
IFS=' ' read -r -a inspect_entities <<<"$(resolve_inspect_entities "$inspect_raw")"

theme_dir=""
theme_profile=""
theme_default_preset_file=""
case "$theme_pack" in
  industrial_v2)
    theme_dir="crates/agent_world_viewer/assets/themes/industrial_v2"
    theme_profile="v2"
    theme_default_preset_file="$theme_dir/presets/industrial_v2_default.env"
    ;;
  industrial_v1)
    theme_dir="crates/agent_world_viewer/assets/themes/industrial_v1"
    theme_profile="v1"
    theme_default_preset_file="$theme_dir/presets/industrial_default.env"
    ;;
  *)
    echo "invalid --theme-pack: $theme_pack (expected industrial_v2|industrial_v1)" >&2
    exit 2
    ;;
esac

if [[ ! -f "$theme_default_preset_file" ]]; then
  echo "missing theme preset file: $theme_default_preset_file" >&2
  exit 1
fi

if [[ -z "$out_dir" ]]; then
  timestamp=$(date '+%Y%m%d-%H%M%S')
  out_dir="output/playwright/viewer/release_full/$timestamp"
else
  timestamp=$(date '+%Y%m%d-%H%M%S')
fi

if [[ -d "$out_dir" && "$keep_out_dir" -eq 0 ]]; then
  existing=$(find "$out_dir" -mindepth 1 -maxdepth 1 -print -quit 2>/dev/null || true)
  if [[ -n "$existing" ]]; then
    echo "error: out dir is not empty: $out_dir (use --keep-out-dir to reuse)" >&2
    exit 2
  fi
fi
run mkdir -p "$out_dir"

overall_pass=1

theme_validate_status="skipped"
theme_validate_note="not requested"
web_qa_status="skipped"
web_qa_note="not requested"
theme_preview_status="skipped"
theme_preview_note="not requested"
texture_status="skipped"
texture_note="not requested"
industrial_status="skipped"
industrial_note="not requested"
governance_status="skipped"
governance_note="not requested"

web_out="$out_dir/web_qa"
theme_preview_out="$out_dir/theme_preview"
texture_out="$out_dir/texture_inspector"
industrial_out="$out_dir/gameplay_industrial"
governance_out="$out_dir/gameplay_governance"

if (( skip_theme_preview == 0 || skip_texture_inspector == 0 )); then
  run_step theme_validate_status theme_validate_note \
    python3 scripts/validate-viewer-theme-pack.py --theme-dir "$theme_dir" --profile "$theme_profile"
fi

if (( skip_web_qa == 0 )); then
  web_cmd=(
    ./scripts/viewer-release-qa-loop.sh
    --scenario "$scenario"
    --out-dir "$web_out"
  )
  if (( skip_web_visual_baseline == 1 )); then
    web_cmd+=(--skip-visual-baseline)
  fi
  if (( headed == 1 )); then
    web_cmd+=(--headed)
  fi
  run_step web_qa_status web_qa_note "${web_cmd[@]}"
  if [[ "$web_qa_status" == "passed" ]]; then
    web_summary=$(ls -t "$web_out"/release-qa-summary-*.md 2>/dev/null | head -n 1 || true)
    if [[ -z "$web_summary" ]]; then
      fail_step web_qa_status web_qa_note "missing release-qa summary file"
    elif ! grep -q -- "- Overall: PASS" "$web_summary"; then
      fail_step web_qa_status web_qa_note "summary overall is not PASS: $web_summary"
    else
      web_qa_note="summary=$web_summary"
    fi
  fi
fi

if (( skip_theme_preview == 0 )); then
  preview_cmd=(
    ./scripts/viewer-theme-pack-preview.sh
    --scenario "$scenario"
    --theme-pack "$theme_pack"
    --variants "$variants_raw"
    --base-port "$base_port"
    --out-dir "$theme_preview_out"
  )
  if (( no_prewarm == 1 )); then
    preview_cmd+=(--no-prewarm)
  fi
  run_step theme_preview_status theme_preview_note "${preview_cmd[@]}"
  if [[ "$theme_preview_status" == "passed" ]]; then
    preview_count=$(find "$theme_preview_out" -type f -name 'viewer.png' -size +0c | wc -l | tr -d ' ')
    expected_preview_count=${#variants[@]}
    if (( preview_count < expected_preview_count )); then
      fail_step theme_preview_status theme_preview_note \
        "viewer.png count too small: got=$preview_count expected>=$expected_preview_count"
    else
      read -r preview_status_count preview_connected_count <<<"$(count_connected_captures "$theme_preview_out")"
      if (( preview_status_count < expected_preview_count )); then
        fail_step theme_preview_status theme_preview_note \
          "capture_status count too small: got=$preview_status_count expected>=$expected_preview_count"
      elif (( preview_connected_count < expected_preview_count )); then
        fail_step theme_preview_status theme_preview_note \
          "connected captures too small: got=$preview_connected_count expected>=$expected_preview_count"
      else
        theme_preview_note="viewer.png count=$preview_count expected>=$expected_preview_count; connected captures=$preview_connected_count/$expected_preview_count"
      fi
    fi
  fi
fi

if (( skip_texture_inspector == 0 )); then
  texture_base_port=$((base_port + 100))
  texture_cmd=(
    ./scripts/viewer-texture-inspector.sh
    --preset-file "$theme_default_preset_file"
    --inspect "$inspect_raw"
    --variants "$variants_raw"
    --scenario "$scenario"
    --base-port "$texture_base_port"
    --out-dir "$texture_out"
  )
  if (( no_prewarm == 1 )); then
    texture_cmd+=(--no-prewarm)
  fi
  run_step texture_status texture_note "${texture_cmd[@]}"
  if [[ "$texture_status" == "passed" ]]; then
    texture_count=$(find "$texture_out" -type f -name 'viewer.png' -size +0c | wc -l | tr -d ' ')
    expected_texture_count=$(( ${#inspect_entities[@]} * ${#variants[@]} ))
    if (( texture_count < expected_texture_count )); then
      fail_step texture_status texture_note \
        "viewer.png count too small: got=$texture_count expected>=$expected_texture_count"
    else
      read -r texture_status_count texture_connected_count <<<"$(count_connected_captures "$texture_out")"
      if (( texture_status_count < expected_texture_count )); then
        fail_step texture_status texture_note \
          "capture_status count too small: got=$texture_status_count expected>=$expected_texture_count"
      elif (( texture_connected_count < expected_texture_count )); then
        fail_step texture_status texture_note \
          "connected captures too small: got=$texture_connected_count expected>=$expected_texture_count"
      else
        texture_note="viewer.png count=$texture_count expected>=$expected_texture_count; connected captures=$texture_connected_count/$expected_texture_count"
      fi
    fi
  fi
fi

industrial_summary_file="$industrial_out/summary.txt"
industrial_action_counts=""
governance_log_file="$governance_out/run.log"

if (( skip_gameplay == 0 )); then
  industrial_cmd=(
    ./scripts/llm-longrun-stress.sh
    --scenario "$scenario"
    --ticks "$ticks_industrial"
    --prompt-pack industrial_baseline
    --runtime-gameplay-bridge
    --release-gate
    --release-gate-profile industrial
    --min-active-ticks 1
    --max-llm-errors 999
    --max-parse-errors 999
    --max-repair-rounds-max 999
    --no-llm-io
    --out-dir "$industrial_out"
  )
  run_step industrial_status industrial_note "${industrial_cmd[@]}"
  if [[ "$industrial_status" == "passed" ]]; then
    industrial_action_counts=$(extract_summary_value "$industrial_summary_file" "action_kind_counts")
    industrial_note="summary=$industrial_summary_file"
  fi

  run mkdir -p "$governance_out"
  governance_cmd=(bash -lc "./scripts/llm-baseline-fixture-smoke.sh | tee \"$governance_log_file\"")
  run_step governance_status governance_note "${governance_cmd[@]}"
  if [[ "$governance_status" == "passed" ]]; then
    governance_note="log=$governance_log_file"
  fi
fi

overall_label="PASS"
if (( overall_pass == 0 )); then
  overall_label="FAIL"
fi

summary_path="$out_dir/release-full-summary-${timestamp}.md"
{
  echo "# Viewer Release Full Coverage Summary"
  echo ""
  echo "- Timestamp: $(date '+%Y-%m-%d %H:%M:%S %Z')"
  echo "- Scenario: \`$scenario\`"
  echo "- Theme pack: \`$theme_pack\`"
  echo "- Variants: \`$variants_raw\`"
  echo "- Inspect entities: \`$inspect_raw\`"
  echo "- Overall: $overall_label"
  echo ""
  echo "## Step Status"
  echo "- Theme pack validation: $theme_validate_status ($theme_validate_note)"
  echo "- Web usability gate: $web_qa_status ($web_qa_note)"
  echo "- Theme preview gate: $theme_preview_status ($theme_preview_note)"
  echo "- Texture inspector gate: $texture_status ($texture_note)"
  echo "- Gameplay industrial gate: $industrial_status ($industrial_note)"
  echo "- Gameplay governance/crisis/economic gate: $governance_status ($governance_note)"
  echo ""
  echo "## Coverage Evidence"
  echo "- Web QA artifacts: \`$web_out\`"
  echo "- Theme preview artifacts: \`$theme_preview_out\`"
  echo "- Texture inspector artifacts: \`$texture_out\`"
  echo "- Industrial gameplay artifacts: \`$industrial_out\`"
  echo "- Governance gameplay artifacts: \`$governance_out\`"
  if [[ -n "$industrial_action_counts" ]]; then
    echo "- Industrial action_kind_counts: \`$industrial_action_counts\`"
  fi
  echo "- Governance/economic deterministic gate: \`scripts/llm-baseline-fixture-smoke.sh\`"
} >"$summary_path"

echo "full coverage summary: $summary_path"

if (( overall_pass == 0 )); then
  exit 1
fi
