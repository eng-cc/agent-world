#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

usage() {
  cat <<'USAGE'
Usage: ./scripts/viewer-release-art-baseline.sh [options]

Purpose:
  Build a repeatable external-facing art baseline set (wide / medium / closeup)
  backed by strict semantic gate captures.

Options:
  --scenario <name>         Scenario for captures (default: triad_region_bootstrap)
  --theme-pack <name>       Theme pack: industrial_v3,industrial_v2,industrial_v1 (default: industrial_v3)
  --variant <name>          Theme variant: default,matte,glossy (default: default)
  --inspect <list>          Entities: agent,location,asset,power_plant,all (default: agent,location,power_plant)
  --base-port <port>        Base port for native capture scripts (default: 6723)
  --viewer-wait <sec>       Viewer wait seconds per capture (default: 10)
  --ui-profile-file <path>  UI profile env for native captures (default: scripts/viewer-release-ui-profile.env)
  --out-dir <path>          Output root (default: output/playwright/viewer/release_art/<timestamp>)
  --no-prewarm              Pass --no-prewarm to capture scripts
  --keep-out-dir            Allow writing into a non-empty out dir
  -h, --help                Show help

Outputs:
  <out-dir>/theme_preview/*
  <out-dir>/texture_inspector/*
  <out-dir>/baseline_samples/*
  <out-dir>/release-art-baseline-summary-<timestamp>.md
USAGE
}

run() {
  echo "+ $*"
  "$@"
}

trim_whitespace() {
  local value=$1
  value="${value#"${value%%[![:space:]]*}"}"
  value="${value%"${value##*[![:space:]]}"}"
  echo "$value"
}

ensure_positive_int() {
  local name=$1
  local value=$2
  if [[ ! "$value" =~ ^[0-9]+$ ]]; then
    echo "invalid integer for $name: $value" >&2
    exit 2
  fi
}

resolve_inspect_entities() {
  local raw=$1
  local normalized
  normalized=$(printf '%s' "$raw" | tr '[:upper:]' '[:lower:]')
  if [[ -z "$normalized" || "$normalized" == "all" ]]; then
    echo "agent location asset power_plant"
    return 0
  fi

  local parsed=()
  local item=""
  IFS=',' read -r -a parsed <<<"$normalized"
  for item in "${parsed[@]}"; do
    item=$(trim_whitespace "$item")
    case "$item" in
      agent|location|asset|power_plant)
        ;;
      *)
        echo "invalid --inspect item: $item" >&2
        echo "supported: agent,location,asset,power_plant,all" >&2
        exit 2
        ;;
    esac
  done
  echo "${parsed[*]}"
}

extract_meta_value() {
  local file_path=$1
  local key=$2
  if [[ ! -f "$file_path" ]]; then
    echo ""
    return 0
  fi
  grep -E "^${key}=" "$file_path" | tail -n 1 | cut -d'=' -f2-
}

scenario="triad_region_bootstrap"
theme_pack="industrial_v3"
variant="default"
inspect_raw="agent,location,power_plant"
base_port=6723
viewer_wait=10
ui_profile_file="scripts/viewer-release-ui-profile.env"
out_dir=""
no_prewarm=0
keep_out_dir=0

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
    --variant)
      variant=${2:-}
      shift 2
      ;;
    --inspect)
      inspect_raw=${2:-}
      shift 2
      ;;
    --base-port)
      base_port=${2:-}
      shift 2
      ;;
    --viewer-wait)
      viewer_wait=${2:-}
      shift 2
      ;;
    --ui-profile-file)
      ui_profile_file=${2:-}
      shift 2
      ;;
    --out-dir)
      out_dir=${2:-}
      shift 2
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

ensure_positive_int "--base-port" "$base_port"
ensure_positive_int "--viewer-wait" "$viewer_wait"

case "$variant" in
  default|matte|glossy)
    ;;
  *)
    echo "invalid --variant: $variant (expected default|matte|glossy)" >&2
    exit 2
    ;;
esac

if [[ -n "$ui_profile_file" && ! -f "$ui_profile_file" ]]; then
  echo "missing --ui-profile-file: $ui_profile_file" >&2
  exit 1
fi

theme_dir=""
theme_profile=""
theme_default_preset_file=""
case "$theme_pack" in
  industrial_v3)
    theme_dir="crates/oasis7_viewer/assets/themes/industrial_v3"
    theme_profile="v3"
    theme_default_preset_file="$theme_dir/presets/industrial_v3_default.env"
    ;;
  industrial_v2)
    theme_dir="crates/oasis7_viewer/assets/themes/industrial_v2"
    theme_profile="v2"
    theme_default_preset_file="$theme_dir/presets/industrial_v2_default.env"
    ;;
  industrial_v1)
    theme_dir="crates/oasis7_viewer/assets/themes/industrial_v1"
    theme_profile="v1"
    theme_default_preset_file="$theme_dir/presets/industrial_default.env"
    ;;
  *)
    echo "invalid --theme-pack: $theme_pack (expected industrial_v3|industrial_v2|industrial_v1)" >&2
    exit 2
    ;;
esac

if [[ ! -f "$theme_default_preset_file" ]]; then
  echo "missing theme preset file: $theme_default_preset_file" >&2
  exit 1
fi

if [[ -z "$out_dir" ]]; then
  timestamp=$(date '+%Y%m%d-%H%M%S')
  out_dir="output/playwright/viewer/release_art/$timestamp"
else
  timestamp=$(date '+%Y%m%d-%H%M%S')
fi

if [[ -d "$out_dir" ]]; then
  if (( keep_out_dir == 0 )) && find "$out_dir" -mindepth 1 -print -quit >/dev/null 2>&1; then
    echo "output directory is not empty: $out_dir (use --keep-out-dir to allow)" >&2
    exit 1
  fi
fi

mkdir -p "$out_dir"

theme_preview_out="$out_dir/theme_preview"
texture_out="$out_dir/texture_inspector"
baseline_samples_out="$out_dir/baseline_samples"
summary_path="$out_dir/release-art-baseline-summary-${timestamp}.md"

inspect_entities=()
IFS=' ' read -r -a inspect_entities <<<"$(resolve_inspect_entities "$inspect_raw")"
inspect_csv=$(IFS=,; echo "${inspect_entities[*]}")

run python3 scripts/validate-viewer-theme-pack.py --theme-dir "$theme_dir" --profile "$theme_profile"

preview_cmd=(
  ./scripts/viewer-theme-pack-preview.sh
  --scenario "$scenario"
  --theme-pack "$theme_pack"
  --variants "$variant"
  --base-port "$base_port"
  --viewer-wait "$viewer_wait"
  --ui-profile-file "$ui_profile_file"
  --out-dir "$theme_preview_out"
)
if (( no_prewarm == 1 )); then
  preview_cmd+=(--no-prewarm)
fi
run "${preview_cmd[@]}"

texture_cmd=(
  ./scripts/viewer-texture-inspector.sh
  --inspect "$inspect_csv"
  --variants "$variant"
  --preset-file "$theme_default_preset_file"
  --scenario "$scenario"
  --base-port "$((base_port + 100))"
  --viewer-wait "$viewer_wait"
  --out-dir "$texture_out"
  --art-capture
  --preview-mode direct_entity
  --semantic-gate-mode strict
  --ui-profile-file "$ui_profile_file"
)
if (( no_prewarm == 1 )); then
  texture_cmd+=(--no-prewarm)
fi
run "${texture_cmd[@]}"

mkdir -p "$baseline_samples_out"
gate_failures=0

wide_src="$theme_preview_out/$variant/viewer.png"
if [[ ! -f "$wide_src" ]]; then
  echo "missing wide sample: $wide_src" >&2
  exit 1
fi
cp "$wide_src" "$baseline_samples_out/wide_${variant}.png"

for entity in "${inspect_entities[@]}"; do
  entity_dir="$texture_out/$entity/$variant"
  meta_file="$entity_dir/meta.txt"
  medium_src="$entity_dir/viewer_art.png"
  closeup_src="$entity_dir/viewer_art_closeup.png"
  if [[ ! -f "$medium_src" ]]; then
    echo "missing medium sample for $entity: $medium_src" >&2
    exit 1
  fi
  if [[ ! -f "$closeup_src" ]]; then
    echo "missing closeup sample for $entity: $closeup_src" >&2
    exit 1
  fi
  cp "$medium_src" "$baseline_samples_out/${entity}_medium.png"
  cp "$closeup_src" "$baseline_samples_out/${entity}_closeup.png"

  gate_pass=$(extract_meta_value "$meta_file" "selection_gate_pass")
  if [[ "$gate_pass" != "1" ]]; then
    gate_failures=$((gate_failures + 1))
  fi
done

{
  echo "# Viewer Release Art Baseline Summary"
  echo
  echo "- Timestamp: $(date '+%Y-%m-%d %H:%M:%S %Z')"
  echo "- Scenario: \`$scenario\`"
  echo "- Theme pack: \`$theme_pack\`"
  echo "- Theme profile: \`$theme_profile\`"
  echo "- Variant: \`$variant\`"
  echo "- Inspect entities: \`$inspect_csv\`"
  echo "- Semantic gate mode: \`strict\`"
  echo "- UI profile: \`${ui_profile_file:-none}\`"
  echo
  echo "## Artifact Roots"
  echo "- Theme preview: \`$theme_preview_out\`"
  echo "- Texture inspector: \`$texture_out\`"
  echo "- Baseline samples: \`$baseline_samples_out\`"
  echo
  echo "## Entity Gate Snapshot"
  for entity in "${inspect_entities[@]}"; do
    meta_file="$texture_out/$entity/$variant/meta.txt"
    gate_pass=$(extract_meta_value "$meta_file" "selection_gate_pass")
    gate_reason=$(extract_meta_value "$meta_file" "selection_gate_reason")
    expected_kind=$(extract_meta_value "$meta_file" "selection_gate_expected_kind")
    selected_kind=$(extract_meta_value "$meta_file" "selection_gate_selection_kind_closeup")
    edge_energy=$(extract_meta_value "$meta_file" "closeup_edge_energy")
    echo "- $entity: gate=$gate_pass reason=${gate_reason:-n/a} expected=${expected_kind:-n/a} selected=${selected_kind:-n/a} edge=${edge_energy:-n/a}"
  done
  echo
  echo "## Baseline Samples"
  echo "- wide_${variant}.png"
  for entity in "${inspect_entities[@]}"; do
    echo "- ${entity}_medium.png"
    echo "- ${entity}_closeup.png"
  done
} >"$summary_path"

if (( gate_failures > 0 )); then
  echo "release art baseline strict semantic gate failed for $gate_failures entity snapshots" >&2
  exit 1
fi

echo "release art baseline artifacts: $out_dir"
echo "release art baseline summary: $summary_path"
