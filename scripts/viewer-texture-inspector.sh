#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

usage() {
  cat <<'USAGE'
Usage: ./scripts/viewer-texture-inspector.sh [options]

Purpose:
  Preview texture sets on a stable viewer composition and capture screenshots.
  Texture sources can be selected from theme preset entity slots.

Options:
  --preset-file <path>     preset env file (default: industrial_default.env)
  --inspect <list>         entity source list: agent,location,asset,power_plant,power_storage,all (default: all)
  --variants <list>        default,matte,glossy,all (default: all)
  --scenario <name>        world_viewer_live scenario (default: llm_bootstrap)
  --base-port <port>       start port per capture (default: 6123)
  --viewer-wait <sec>      viewer wait before capture (default: 8)
  --render-profile <name>  debug,balanced,cinematic (default: cinematic)
  --fragment-strategy <s>  readability,fidelity (default: fidelity)
  --base-texture <path>    override source base texture
  --normal-texture <path>  override source normal texture
  --mr-texture <path>      override source metallic_roughness texture
  --emissive-texture <p>   override source emissive texture
  --use-source-mesh        use inspected entity mesh as location mesh in preview
  --out-dir <dir>          output root (default: output/texture_inspector/<timestamp>)
  --no-prewarm             pass --no-prewarm to all capture runs
  -h, --help               show help

Outputs:
  output/texture_inspector/<timestamp>/<entity>/<variant>/
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

  if [[ -z "$normalized" || "$normalized" == "all" ]]; then
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

resolve_entities() {
  local raw=$1
  local normalized
  normalized=$(echo "$raw" | tr '[:upper:]' '[:lower:]')

  if [[ -z "$normalized" || "$normalized" == "all" ]]; then
    echo "agent location asset power_plant power_storage"
    return 0
  fi

  local parsed=()
  IFS=',' read -r -a parsed <<<"$normalized"
  local item
  for item in "${parsed[@]}"; do
    case "$item" in
      agent|location|asset|power_plant|power_storage)
        ;;
      *)
        echo "invalid inspect entity: $item" >&2
        echo "supported entities: agent,location,asset,power_plant,power_storage,all" >&2
        exit 2
        ;;
    esac
  done
  echo "${parsed[*]}"
}

entity_prefix() {
  case "$1" in
    agent)
      echo "AGENT"
      ;;
    location)
      echo "LOCATION"
      ;;
    asset)
      echo "ASSET"
      ;;
    power_plant)
      echo "POWER_PLANT"
      ;;
    power_storage)
      echo "POWER_STORAGE"
      ;;
    *)
      echo "unknown entity: $1" >&2
      exit 2
      ;;
  esac
}

set_or_unset_env() {
  local key=$1
  local value=$2
  if [[ -n "$value" ]]; then
    export "$key=$value"
  else
    unset "$key" || true
  fi
}

capture_status_value() {
  local status_file=$1
  local key=$2
  grep -E "^${key}=" "$status_file" | tail -n 1 | cut -d'=' -f2-
}

preset_file="crates/agent_world_viewer/assets/themes/industrial_v1/presets/industrial_default.env"
inspect_raw="all"
variants_raw="all"
scenario="llm_bootstrap"
base_port=6123
viewer_wait=8
render_profile="cinematic"
fragment_strategy="fidelity"
out_dir=""
force_no_prewarm=0
use_source_mesh=0

override_base_texture=""
override_normal_texture=""
override_mr_texture=""
override_emissive_texture=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --preset-file)
      preset_file=${2:-}
      shift 2
      ;;
    --inspect)
      inspect_raw=${2:-}
      shift 2
      ;;
    --variants)
      variants_raw=${2:-}
      shift 2
      ;;
    --scenario)
      scenario=${2:-}
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
    --render-profile)
      render_profile=${2:-}
      shift 2
      ;;
    --fragment-strategy)
      fragment_strategy=${2:-}
      shift 2
      ;;
    --base-texture)
      override_base_texture=${2:-}
      shift 2
      ;;
    --normal-texture)
      override_normal_texture=${2:-}
      shift 2
      ;;
    --mr-texture)
      override_mr_texture=${2:-}
      shift 2
      ;;
    --emissive-texture)
      override_emissive_texture=${2:-}
      shift 2
      ;;
    --use-source-mesh)
      use_source_mesh=1
      shift
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

if [[ ! -f "$preset_file" ]]; then
  echo "missing preset file: $preset_file" >&2
  exit 1
fi

if [[ ! "$base_port" =~ ^[0-9]+$ ]]; then
  echo "--base-port must be an integer" >&2
  exit 2
fi

case "$render_profile" in
  debug|balanced|cinematic)
    ;;
  *)
    echo "invalid --render-profile: $render_profile" >&2
    echo "supported render profiles: debug,balanced,cinematic" >&2
    exit 2
    ;;
esac

case "$fragment_strategy" in
  readability|fidelity)
    ;;
  *)
    echo "invalid --fragment-strategy: $fragment_strategy" >&2
    echo "supported fragment strategies: readability,fidelity" >&2
    exit 2
    ;;
esac

if [[ -z "$out_dir" ]]; then
  timestamp=$(date '+%Y%m%d_%H%M%S')
  out_dir="output/texture_inspector/$timestamp"
fi

entities=($(resolve_entities "$inspect_raw"))
variants=($(resolve_variants "$variants_raw"))
mkdir -p "$out_dir"

automation_steps="mode=3d;focus=first_location;pan=0,2,0;zoom=1.2;orbit=10,-25;select=first_location;wait=0.4"

capture_index=0
for entity in "${entities[@]}"; do
  src_prefix=$(entity_prefix "$entity")
  for variant in "${variants[@]}"; do
    port=$((base_port + capture_index))
    variant_dir="$out_dir/$entity/$variant"
    mkdir -p "$variant_dir"

    no_prewarm_arg=""
    if [[ "$force_no_prewarm" -eq 1 || "$capture_index" -gt 0 ]]; then
      no_prewarm_arg="--no-prewarm"
    fi

    (
      # Load base theme preset first, then pin variant and inspector overrides.
      source "$preset_file"

      export AGENT_WORLD_VIEWER_MATERIAL_VARIANT_PRESET="$variant"
      export AGENT_WORLD_VIEWER_RENDER_PROFILE="$render_profile"
      export AGENT_WORLD_VIEWER_FRAGMENT_MATERIAL_STRATEGY="$fragment_strategy"
      export AGENT_WORLD_VIEWER_SHOW_LOCATIONS=1
      export AGENT_WORLD_VIEWER_SHOW_AGENTS=0

      src_mesh_key="AGENT_WORLD_VIEWER_${src_prefix}_MESH_ASSET"
      src_base_key="AGENT_WORLD_VIEWER_${src_prefix}_BASE_TEXTURE_ASSET"
      src_normal_key="AGENT_WORLD_VIEWER_${src_prefix}_NORMAL_TEXTURE_ASSET"
      src_mr_key="AGENT_WORLD_VIEWER_${src_prefix}_METALLIC_ROUGHNESS_TEXTURE_ASSET"
      src_emissive_key="AGENT_WORLD_VIEWER_${src_prefix}_EMISSIVE_TEXTURE_ASSET"

      src_mesh="${!src_mesh_key:-}"
      src_base="${!src_base_key:-}"
      src_normal="${!src_normal_key:-}"
      src_mr="${!src_mr_key:-}"
      src_emissive="${!src_emissive_key:-}"

      if [[ "$use_source_mesh" -eq 1 ]]; then
        set_or_unset_env "AGENT_WORLD_VIEWER_LOCATION_MESH_ASSET" "$src_mesh"
      fi

      set_or_unset_env "AGENT_WORLD_VIEWER_LOCATION_BASE_TEXTURE_ASSET" "${override_base_texture:-$src_base}"
      set_or_unset_env "AGENT_WORLD_VIEWER_LOCATION_NORMAL_TEXTURE_ASSET" "${override_normal_texture:-$src_normal}"
      set_or_unset_env "AGENT_WORLD_VIEWER_LOCATION_METALLIC_ROUGHNESS_TEXTURE_ASSET" "${override_mr_texture:-$src_mr}"
      set_or_unset_env "AGENT_WORLD_VIEWER_LOCATION_EMISSIVE_TEXTURE_ASSET" "${override_emissive_texture:-$src_emissive}"

      run ./scripts/capture-viewer-frame.sh \
        --scenario "$scenario" \
        --addr "127.0.0.1:$port" \
        --viewer-wait "$viewer_wait" \
        --auto-focus-target first_location \
        --automation-steps "$automation_steps" \
        --keep-tmp \
        ${no_prewarm_arg:+$no_prewarm_arg}

      capture_status_file=".tmp/screens/capture_status.txt"
      if [[ ! -s "$capture_status_file" ]]; then
        echo "missing capture status file: $capture_status_file (entity=$entity variant=$variant)" >&2
        exit 1
      fi
      capture_connection_status=$(capture_status_value "$capture_status_file" "connection_status")
      capture_snapshot_ready=$(capture_status_value "$capture_status_file" "snapshot_ready")
      capture_last_error=$(capture_status_value "$capture_status_file" "last_error")
      if [[ "$capture_connection_status" != "connected" || "$capture_snapshot_ready" != "1" ]]; then
        echo "texture inspector capture connectivity gate failed: entity=$entity variant=$variant connection_status=${capture_connection_status:-unknown} snapshot_ready=${capture_snapshot_ready:-unknown}" >&2
        if [[ -n "$capture_last_error" ]]; then
          echo "last_error=$capture_last_error" >&2
        fi
        cat "$capture_status_file" >&2 || true
        exit 1
      fi

      cp .tmp/screens/window.png "$variant_dir/viewer.png"
      cp .tmp/screens/live_server.log "$variant_dir/live_server.log"
      cp .tmp/screens/viewer.log "$variant_dir/viewer.log"
      cp "$capture_status_file" "$variant_dir/capture_status.txt"

      cat >"$variant_dir/meta.txt" <<META
preset_file=$preset_file
scenario=$scenario
entity=$entity
variant=$variant
port=$port
render_profile=$render_profile
fragment_strategy=$fragment_strategy
use_source_mesh=$use_source_mesh
location_mesh_asset=${AGENT_WORLD_VIEWER_LOCATION_MESH_ASSET:-}
location_base_texture_asset=${AGENT_WORLD_VIEWER_LOCATION_BASE_TEXTURE_ASSET:-}
location_normal_texture_asset=${AGENT_WORLD_VIEWER_LOCATION_NORMAL_TEXTURE_ASSET:-}
location_metallic_roughness_texture_asset=${AGENT_WORLD_VIEWER_LOCATION_METALLIC_ROUGHNESS_TEXTURE_ASSET:-}
location_emissive_texture_asset=${AGENT_WORLD_VIEWER_LOCATION_EMISSIVE_TEXTURE_ASSET:-}
capture_connection_status=$capture_connection_status
capture_snapshot_ready=$capture_snapshot_ready
META
    )

    capture_index=$((capture_index + 1))
  done
done

echo "texture inspector artifacts: $out_dir"
