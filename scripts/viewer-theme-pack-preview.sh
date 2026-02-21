#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

usage() {
  cat <<'USAGE'
Usage: ./scripts/viewer-theme-pack-preview.sh [options]

Options:
  --scenario <name>        world_viewer_live scenario (default: llm_bootstrap)
  --base-port <port>       starting port for per-variant capture (default: 5423)
  --tick-ms <ms>           live tick interval (default: 300)
  --viewer-wait <sec>      viewer wait before capture (default: 10)
  --variants <list>        comma-separated variants: default,matte,glossy,all (default: all)
  --out-dir <dir>          output root (default: output/theme_preview/<timestamp>)
  --no-prewarm             pass --no-prewarm to all capture runs
  -h, --help               show help

Outputs:
  output/theme_preview/<timestamp>/<variant>/
    viewer.png live_server.log viewer.log meta.txt
USAGE
}

run() {
  echo "+ $*"
  "$@"
}

resolve_variants() {
  local raw=$1
  local normalized
  normalized=$(echo "$raw" | tr '[:upper:]' '[:lower:]')
  if [[ "$normalized" == "all" || -z "$normalized" ]]; then
    echo "default matte glossy"
    return 0
  fi

  local parsed=()
  IFS=',' read -r -a parsed <<<"$normalized"
  local item
  for item in "${parsed[@]}"; do
    case "$item" in
      default|matte|glossy)
        ;;
      *)
        echo "invalid variant: $item" >&2
        echo "supported variants: default,matte,glossy,all" >&2
        exit 2
        ;;
    esac
  done
  echo "${parsed[*]}"
}

scenario="llm_bootstrap"
base_port=5423
tick_ms=300
viewer_wait=10
variants_raw="all"
out_dir=""
force_no_prewarm=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --scenario)
      scenario=${2:-}
      shift 2
      ;;
    --base-port)
      base_port=${2:-}
      shift 2
      ;;
    --tick-ms)
      tick_ms=${2:-}
      shift 2
      ;;
    --viewer-wait)
      viewer_wait=${2:-}
      shift 2
      ;;
    --variants)
      variants_raw=${2:-}
      shift 2
      ;;
    --out-dir)
      out_dir=${2:-}
      shift 2
      ;;
    --no-prewarm)
      force_no_prewarm=1
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

if [[ -z "$out_dir" ]]; then
  timestamp=$(date '+%Y%m%d_%H%M%S')
  out_dir="output/theme_preview/$timestamp"
fi

if [[ ! "$base_port" =~ ^[0-9]+$ ]]; then
  echo "--base-port must be an integer" >&2
  exit 2
fi

variants=($(resolve_variants "$variants_raw"))
mkdir -p "$out_dir"

preset_dir="crates/agent_world_viewer/assets/themes/industrial_v1/presets"
automation_steps="mode=3d;focus=first_agent;zoom=0.8;select=first_agent"

index=0
for variant in "${variants[@]}"; do
  port=$((base_port + index))
  variant_dir="$out_dir/$variant"
  mkdir -p "$variant_dir"
  preset_file="$preset_dir/industrial_${variant}.env"
  if [[ ! -f "$preset_file" ]]; then
    echo "missing preset: $preset_file" >&2
    exit 1
  fi

  no_prewarm_flag=()
  if [[ "$force_no_prewarm" -eq 1 || "$index" -gt 0 ]]; then
    no_prewarm_flag+=(--no-prewarm)
  fi

  (
    source "$preset_file"
    run ./scripts/capture-viewer-frame.sh \
      --scenario "$scenario" \
      --addr "127.0.0.1:$port" \
      --tick-ms "$tick_ms" \
      --viewer-wait "$viewer_wait" \
      --auto-focus-target first_agent \
      --automation-steps "$automation_steps" \
      --keep-tmp \
      "${no_prewarm_flag[@]}"
  )

  cp .tmp/screens/window.png "$variant_dir/viewer.png"
  cp .tmp/screens/live_server.log "$variant_dir/live_server.log"
  cp .tmp/screens/viewer.log "$variant_dir/viewer.log"
  cat >"$variant_dir/meta.txt" <<META
scenario=$scenario
variant=$variant
port=$port
tick_ms=$tick_ms
viewer_wait=$viewer_wait
preset_file=$preset_file
META

  index=$((index + 1))
done

echo "theme preview artifacts: $out_dir"
