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
  --art-capture            enable art-review mode (director ui + source mesh + crop output)
  --automation-steps <s>   override viewer automation steps for all captures
  --closeup-automation-steps <s>
                           override closeup automation steps for all captures
  --art-lighting           enable art-review lighting preset
  --no-art-lighting        disable art-review lighting preset
  --variant-ssim-threshold <f>
                           power variant validation threshold (default: 0.9995)
  --crop-window <w:h:x:y>  crop window for viewer_art.png; use 'none' to disable crop
  --preview-mode <mode>    scene_proxy,lookdev (default: scene_proxy)
  --no-prewarm             pass --no-prewarm to all capture runs
  -h, --help               show help

Outputs:
  output/texture_inspector/<timestamp>/<entity>/<variant>/
    viewer.png viewer_art.png viewer_closeup.png viewer_art_closeup.png
    live_server.log viewer.log live_server_closeup.log viewer_closeup.log meta.txt
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

resolve_focus_target_for_entity() {
  local entity=$1
  local scenario_name=${2:-}
  case "$entity" in
    power_plant)
      if [[ "$scenario_name" == *power* ]]; then
        echo "first_power_plant"
      else
        echo "first_location"
      fi
      ;;
    power_storage)
      if [[ "$scenario_name" == *power* ]]; then
        echo "first_power_storage"
      else
        echo "first_location"
      fi
      ;;
    *)
      echo "first_location"
      ;;
  esac
}

resolve_power_closeup_pose() {
  local focus_target=$1
  if [[ "$focus_target" == "first_location" ]]; then
    # In non-power scenarios, fallback to wider closeup to avoid clipping giant source meshes.
    echo "pan=0.6,0,0;zoom=2.2;orbit=20,-30;wait=0.8"
  else
    echo "zoom=1.2;orbit=38,-20;wait=0.8"
  fi
}

resolve_power_fallback_closeup_pose() {
  local focus_target=$1
  if [[ "$focus_target" == "first_location" ]]; then
    echo "pan=0.6,0,0;zoom=2.6;orbit=20,-30;wait=0.9"
  else
    echo "zoom=0.9;orbit=46,-14;wait=0.9"
  fi
}

resolve_power_hero_pose() {
  local focus_target=$1
  if [[ "$focus_target" == "first_location" ]]; then
    echo "pan=0.4,0,0;zoom=1.6;orbit=20,-30;wait=0.6"
  else
    echo "zoom=1.6;orbit=20,-30;wait=0.6"
  fi
}

default_automation_steps_for_entity() {
  local entity=$1
  local focus_target
  focus_target=$(resolve_focus_target_for_entity "$entity" "$scenario")
  local power_hero_pose
  power_hero_pose=$(resolve_power_hero_pose "$focus_target")
  case "$1" in
    agent)
      echo "mode=3d;focus=first_location;zoom=1.5;orbit=18,-26;wait=0.6"
      ;;
    location)
      echo "mode=3d;focus=first_location;zoom=1.7;orbit=14,-24;wait=0.6"
      ;;
    asset)
      echo "mode=3d;focus=first_location;zoom=1.55;orbit=18,-28;wait=0.6"
      ;;
    power_plant)
      echo "mode=3d;focus=${focus_target};${power_hero_pose}"
      ;;
    power_storage)
      echo "mode=3d;focus=${focus_target};${power_hero_pose}"
      ;;
    *)
      echo "mode=3d;focus=first_location;pan=0,2,0;zoom=1.2;orbit=10,-25;select=first_location;wait=0.4"
      ;;
  esac
}

default_closeup_automation_steps_for_entity() {
  local entity=$1
  local focus_target
  focus_target=$(resolve_focus_target_for_entity "$entity" "$scenario")
  local power_closeup_pose
  power_closeup_pose=$(resolve_power_closeup_pose "$focus_target")
  case "$1" in
    agent)
      echo "mode=3d;focus=first_location;zoom=1.15;orbit=30,-20;wait=0.8"
      ;;
    location)
      echo "mode=3d;focus=first_location;zoom=1.25;orbit=28,-20;wait=0.8"
      ;;
    asset)
      echo "mode=3d;focus=first_location;zoom=1.15;orbit=32,-22;wait=0.8"
      ;;
    power_plant)
      echo "mode=3d;focus=${focus_target};${power_closeup_pose}"
      ;;
    power_storage)
      echo "mode=3d;focus=${focus_target};${power_closeup_pose}"
      ;;
    *)
      echo "mode=3d;focus=first_location;zoom=0.18;orbit=30,-22;wait=0.7"
      ;;
  esac
}

fallback_closeup_automation_steps_for_entity() {
  local entity=$1
  local focus_target
  focus_target=$(resolve_focus_target_for_entity "$entity" "$scenario")
  local power_fallback_pose
  power_fallback_pose=$(resolve_power_fallback_closeup_pose "$focus_target")
  case "$1" in
    power_plant)
      echo "mode=3d;focus=${focus_target};${power_fallback_pose}"
      ;;
    power_storage)
      echo "mode=3d;focus=${focus_target};${power_fallback_pose}"
      ;;
    *)
      echo "mode=3d;focus=first_location;zoom=0.95;orbit=42,-16;wait=0.9"
      ;;
  esac
}

parse_crop_window() {
  local raw=${1:-none}
  local normalized
  normalized=$(echo "$raw" | tr '[:upper:]' '[:lower:]')
  if [[ "$normalized" == "none" || "$normalized" == "off" || "$normalized" == "disable" ]]; then
    echo "none"
    return 0
  fi

  IFS=':' read -r crop_w crop_h crop_x crop_y extra <<<"$raw"
  if [[ -n "${extra:-}" || -z "${crop_w:-}" || -z "${crop_h:-}" || -z "${crop_x:-}" || -z "${crop_y:-}" ]]; then
    echo "invalid --crop-window: $raw (expected w:h:x:y or none)" >&2
    exit 2
  fi

  for value in "$crop_w" "$crop_h" "$crop_x" "$crop_y"; do
    if [[ ! "$value" =~ ^[0-9]+$ ]]; then
      echo "invalid --crop-window: $raw (all values must be non-negative integers)" >&2
      exit 2
    fi
  done

  if [[ "$crop_w" -eq 0 || "$crop_h" -eq 0 ]]; then
    echo "invalid --crop-window: $raw (width/height must be > 0)" >&2
    exit 2
  fi

  echo "${crop_w}:${crop_h}:${crop_x}:${crop_y}"
}

crop_or_copy_image() {
  local src=$1
  local dest=$2
  local crop_spec=$3

  if [[ "$crop_spec" == "none" ]]; then
    cp "$src" "$dest"
    echo "passthrough"
    return 0
  fi

  IFS=':' read -r crop_w crop_h crop_x crop_y <<<"$crop_spec"
  if ffmpeg -y -i "$src" -vf "crop=${crop_w}:${crop_h}:${crop_x}:${crop_y}" "$dest" >/dev/null 2>&1; then
    echo "cropped"
  else
    cp "$src" "$dest"
    echo "crop_failed_fallback"
  fi
}

captures_are_all_present() {
  local root=$1
  local entity=$2
  local variant
  for variant in default matte glossy; do
    if [[ ! -f "$root/$entity/$variant/viewer_art_closeup.png" ]]; then
      return 1
    fi
  done
  return 0
}

variant_hash_unique_count() {
  local root=$1
  local entity=$2
  local hashes=()
  local variant
  for variant in default matte glossy; do
    local path="$root/$entity/$variant/viewer_art_closeup.png"
    if [[ ! -f "$path" ]]; then
      echo 0
      return 0
    fi
    hashes+=("$(sha256sum "$path" | awk '{print $1}')")
  done
  printf "%s\n" "${hashes[@]}" | sort -u | wc -l | tr -d ' '
}

parse_unit_interval_float_or_exit() {
  local raw=$1
  local option_name=$2
  if ! awk -v value="$raw" 'BEGIN { exit !(value ~ /^[0-9]*\.?[0-9]+$/ && value >= 0 && value <= 1) }'; then
    echo "invalid ${option_name}: $raw (expected 0..1)" >&2
    exit 2
  fi
  echo "$raw"
}

float_ge() {
  local lhs=$1
  local rhs=$2
  awk -v lhs="$lhs" -v rhs="$rhs" 'BEGIN { exit !(lhs >= rhs) }'
}

float_lt() {
  local lhs=$1
  local rhs=$2
  awk -v lhs="$lhs" -v rhs="$rhs" 'BEGIN { exit !(lhs < rhs) }'
}

variant_pair_ssim_value() {
  local root=$1
  local entity=$2
  local a=$3
  local b=$4
  local path_a="$root/$entity/$a/viewer_art_closeup.png"
  local path_b="$root/$entity/$b/viewer_art_closeup.png"

  if [[ ! -f "$path_a" || ! -f "$path_b" ]]; then
    echo "1"
    return 0
  fi

  local line
  line=$(ffmpeg -i "$path_a" -i "$path_b" -lavfi ssim -f null - 2>&1 | grep 'All:' | tail -n 1 || true)
  if [[ -z "$line" ]]; then
    echo "1"
    return 0
  fi

  local value
  value=$(echo "$line" | sed -E 's/.*All:([0-9.]+).*/\1/' || true)
  if [[ -z "$value" ]]; then
    echo "1"
    return 0
  fi

  echo "$value"
}

variant_min_pair_ssim() {
  local root=$1
  local entity=$2
  local min_ssim="1"
  local pairs=("default matte" "default glossy" "matte glossy")
  local pair
  for pair in "${pairs[@]}"; do
    read -r a b <<<"$pair"
    local value
    value=$(variant_pair_ssim_value "$root" "$entity" "$a" "$b")
    if float_lt "$value" "$min_ssim"; then
      min_ssim="$value"
    fi
  done
  echo "$min_ssim"
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
art_capture=0
automation_steps_override=""
closeup_automation_steps_override=""
art_lighting_mode="auto"
variant_ssim_threshold="0.9995"
crop_window_raw=""
preview_mode="scene_proxy"

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
    --art-capture)
      art_capture=1
      shift
      ;;
    --automation-steps)
      automation_steps_override=${2:-}
      shift 2
      ;;
    --closeup-automation-steps)
      closeup_automation_steps_override=${2:-}
      shift 2
      ;;
    --art-lighting)
      art_lighting_mode="on"
      shift
      ;;
    --no-art-lighting)
      art_lighting_mode="off"
      shift
      ;;
    --variant-ssim-threshold)
      variant_ssim_threshold=${2:-}
      shift 2
      ;;
    --crop-window)
      crop_window_raw=${2:-}
      shift 2
      ;;
    --preview-mode)
      preview_mode=${2:-}
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

case "$preview_mode" in
  scene_proxy|lookdev)
    ;;
  *)
    echo "invalid --preview-mode: $preview_mode" >&2
    echo "supported preview modes: scene_proxy,lookdev" >&2
    exit 2
    ;;
esac

if [[ -z "$out_dir" ]]; then
  timestamp=$(date '+%Y%m%d_%H%M%S')
  out_dir="output/texture_inspector/$timestamp"
fi

variant_ssim_threshold=$(parse_unit_interval_float_or_exit "$variant_ssim_threshold" "--variant-ssim-threshold")

entities=($(resolve_entities "$inspect_raw"))
variants=($(resolve_variants "$variants_raw"))
mkdir -p "$out_dir"

default_automation_steps="mode=3d;focus=first_location;pan=0,2,0;zoom=1.2;orbit=10,-25;select=first_location;wait=0.4"
if [[ "$art_capture" -eq 1 && "$use_source_mesh" -eq 0 ]]; then
  use_source_mesh=1
fi
if [[ -z "$crop_window_raw" ]]; then
  if [[ "$art_capture" -eq 1 ]]; then
    crop_window_raw="600:620:0:120"
  else
    crop_window_raw="none"
  fi
fi
crop_window=$(parse_crop_window "$crop_window_raw")

art_lighting_enabled=0
if [[ "$art_lighting_mode" == "on" ]]; then
  art_lighting_enabled=1
elif [[ "$art_lighting_mode" == "auto" && "$art_capture" -eq 1 ]]; then
  art_lighting_enabled=1
fi

capture_index=0
capture_variant_bundle() {
  local entity=$1
  local variant=$2
  local variant_dir=$3
  local port=$4
  local hero_steps=$5
  local closeup_steps=$6
  local no_prewarm_arg=$7
  local retry_attempt=$8
  local src_prefix
  src_prefix=$(entity_prefix "$entity")

  mkdir -p "$variant_dir"

  (
    # Load base theme preset first, then pin variant and inspector overrides.
    source "$preset_file"

    export AGENT_WORLD_VIEWER_MATERIAL_VARIANT_PRESET="$variant"
    export AGENT_WORLD_VIEWER_RENDER_PROFILE="$render_profile"
    export AGENT_WORLD_VIEWER_FRAGMENT_MATERIAL_STRATEGY="$fragment_strategy"
    export AGENT_WORLD_VIEWER_SHOW_LOCATIONS=1
    export AGENT_WORLD_VIEWER_SHOW_AGENTS=0
    if [[ "$preview_mode" == "lookdev" ]]; then
      set_or_unset_env "AGENT_WORLD_VIEWER_LOCATION_SHELL_ENABLED" "0"
      set_or_unset_env "AGENT_WORLD_VIEWER_LOCATION_RADIATION_GLOW" "0"
      set_or_unset_env "AGENT_WORLD_VIEWER_LOCATION_DAMAGE_VISUAL" "0"
    else
      set_or_unset_env "AGENT_WORLD_VIEWER_LOCATION_SHELL_ENABLED" ""
      set_or_unset_env "AGENT_WORLD_VIEWER_LOCATION_RADIATION_GLOW" ""
      set_or_unset_env "AGENT_WORLD_VIEWER_LOCATION_DAMAGE_VISUAL" ""
    fi
    if [[ "$art_capture" -eq 1 ]]; then
      export AGENT_WORLD_VIEWER_EXPERIENCE_MODE="director"
      export AGENT_WORLD_VIEWER_PANEL_MODE="observe"
      export AGENT_WORLD_VIEWER_SHOW_OPS_NAV=0
    fi
    if [[ "$art_lighting_enabled" -eq 1 ]]; then
      export AGENT_WORLD_VIEWER_TONEMAPPING="aces"
      export AGENT_WORLD_VIEWER_BLOOM_ENABLED=0
      export AGENT_WORLD_VIEWER_BLOOM_INTENSITY=0
      export AGENT_WORLD_VIEWER_COLOR_GRADING_EXPOSURE=-0.35
      export AGENT_WORLD_VIEWER_AMBIENT_BRIGHTNESS=95
      export AGENT_WORLD_VIEWER_FILL_LIGHT_RATIO=0.46
      export AGENT_WORLD_VIEWER_RIM_LIGHT_RATIO=0.32
      export AGENT_WORLD_VIEWER_EXPOSURE_EV100=12.8
    fi

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
      --automation-steps "$hero_steps" \
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
    viewer_art_capture_status=$(crop_or_copy_image "$variant_dir/viewer.png" "$variant_dir/viewer_art.png" "$crop_window")
    if [[ "$viewer_art_capture_status" == "crop_failed_fallback" ]]; then
      echo "warn: crop failed, fallback to viewer.png (entity=$entity variant=$variant crop_window=$crop_window)" >&2
    fi

    capture_connection_status_closeup="$capture_connection_status"
    capture_snapshot_ready_closeup="$capture_snapshot_ready"
    viewer_art_closeup_capture_status="passthrough"

    if [[ "$art_capture" -eq 1 ]]; then
      run ./scripts/capture-viewer-frame.sh \
        --scenario "$scenario" \
        --addr "127.0.0.1:$port" \
        --viewer-wait "$viewer_wait" \
        --auto-focus-target first_location \
        --automation-steps "$closeup_steps" \
        --keep-tmp \
        --no-prewarm

      capture_status_file=".tmp/screens/capture_status.txt"
      if [[ ! -s "$capture_status_file" ]]; then
        echo "missing capture status file: $capture_status_file (entity=$entity variant=$variant closeup=1)" >&2
        exit 1
      fi
      capture_connection_status_closeup=$(capture_status_value "$capture_status_file" "connection_status")
      capture_snapshot_ready_closeup=$(capture_status_value "$capture_status_file" "snapshot_ready")
      capture_last_error_closeup=$(capture_status_value "$capture_status_file" "last_error")
      if [[ "$capture_connection_status_closeup" != "connected" || "$capture_snapshot_ready_closeup" != "1" ]]; then
        echo "texture inspector closeup capture connectivity gate failed: entity=$entity variant=$variant connection_status=${capture_connection_status_closeup:-unknown} snapshot_ready=${capture_snapshot_ready_closeup:-unknown}" >&2
        if [[ -n "$capture_last_error_closeup" ]]; then
          echo "last_error=$capture_last_error_closeup" >&2
        fi
        cat "$capture_status_file" >&2 || true
        exit 1
      fi

      cp .tmp/screens/window.png "$variant_dir/viewer_closeup.png"
      cp .tmp/screens/live_server.log "$variant_dir/live_server_closeup.log"
      cp .tmp/screens/viewer.log "$variant_dir/viewer_closeup.log"
      cp "$capture_status_file" "$variant_dir/capture_status_closeup.txt"
      viewer_art_closeup_capture_status=$(crop_or_copy_image "$variant_dir/viewer_closeup.png" "$variant_dir/viewer_art_closeup.png" "$crop_window")
      if [[ "$viewer_art_closeup_capture_status" == "crop_failed_fallback" ]]; then
        echo "warn: closeup crop failed, fallback to viewer_closeup.png (entity=$entity variant=$variant crop_window=$crop_window)" >&2
      fi
    else
      cp "$variant_dir/viewer.png" "$variant_dir/viewer_closeup.png"
      cp "$variant_dir/live_server.log" "$variant_dir/live_server_closeup.log"
      cp "$variant_dir/viewer.log" "$variant_dir/viewer_closeup.log"
      cp "$variant_dir/capture_status.txt" "$variant_dir/capture_status_closeup.txt"
      cp "$variant_dir/viewer_art.png" "$variant_dir/viewer_art_closeup.png"
    fi

    cat >"$variant_dir/meta.txt" <<META
preset_file=$preset_file
scenario=$scenario
entity=$entity
variant=$variant
port=$port
render_profile=$render_profile
fragment_strategy=$fragment_strategy
art_capture=$art_capture
preview_mode=$preview_mode
hero_automation_steps=$hero_steps
closeup_automation_steps=$closeup_steps
crop_window=$crop_window
viewer_art_capture_status=$viewer_art_capture_status
viewer_art_closeup_capture_status=$viewer_art_closeup_capture_status
retry_attempt=$retry_attempt
art_lighting_enabled=$art_lighting_enabled
variant_ssim_threshold=$variant_ssim_threshold
use_source_mesh=$use_source_mesh
lookdev_location_shell_enabled=${AGENT_WORLD_VIEWER_LOCATION_SHELL_ENABLED:-}
lookdev_location_radiation_glow=${AGENT_WORLD_VIEWER_LOCATION_RADIATION_GLOW:-}
lookdev_location_damage_visual=${AGENT_WORLD_VIEWER_LOCATION_DAMAGE_VISUAL:-}
location_mesh_asset=${AGENT_WORLD_VIEWER_LOCATION_MESH_ASSET:-}
location_base_texture_asset=${AGENT_WORLD_VIEWER_LOCATION_BASE_TEXTURE_ASSET:-}
location_normal_texture_asset=${AGENT_WORLD_VIEWER_LOCATION_NORMAL_TEXTURE_ASSET:-}
location_metallic_roughness_texture_asset=${AGENT_WORLD_VIEWER_LOCATION_METALLIC_ROUGHNESS_TEXTURE_ASSET:-}
location_emissive_texture_asset=${AGENT_WORLD_VIEWER_LOCATION_EMISSIVE_TEXTURE_ASSET:-}
capture_connection_status=$capture_connection_status
capture_snapshot_ready=$capture_snapshot_ready
capture_connection_status_closeup=$capture_connection_status_closeup
capture_snapshot_ready_closeup=$capture_snapshot_ready_closeup
META
  )
}

for entity in "${entities[@]}"; do
  entity_default_automation_steps=$(default_automation_steps_for_entity "$entity")
  entity_default_closeup_steps=$(default_closeup_automation_steps_for_entity "$entity")
  entity_retry_closeup_steps=$(fallback_closeup_automation_steps_for_entity "$entity")

  for variant in "${variants[@]}"; do
    port=$((base_port + capture_index))
    capture_index=$((capture_index + 1))
    variant_dir="$out_dir/$entity/$variant"

    if [[ -n "$automation_steps_override" ]]; then
      hero_steps="$automation_steps_override"
    elif [[ "$art_capture" -eq 1 ]]; then
      hero_steps="$entity_default_automation_steps"
    else
      hero_steps="$default_automation_steps"
    fi

    if [[ -n "$closeup_automation_steps_override" ]]; then
      closeup_steps="$closeup_automation_steps_override"
    elif [[ "$art_capture" -eq 1 ]]; then
      closeup_steps="$entity_default_closeup_steps"
    else
      closeup_steps="$hero_steps"
    fi

    no_prewarm_arg=""
    if [[ "$force_no_prewarm" -eq 1 || "$capture_index" -gt 1 ]]; then
      no_prewarm_arg="--no-prewarm"
    fi

    capture_variant_bundle "$entity" "$variant" "$variant_dir" "$port" "$hero_steps" "$closeup_steps" "$no_prewarm_arg" "0"
  done

  if [[ "$art_capture" -eq 1 && ( "$entity" == "power_plant" || "$entity" == "power_storage" ) ]]; then
    if captures_are_all_present "$out_dir" "$entity"; then
      unique_count=$(variant_hash_unique_count "$out_dir" "$entity")
      min_ssim=$(variant_min_pair_ssim "$out_dir" "$entity")
      unique_count_retry="$unique_count"
      min_ssim_retry="$min_ssim"
      initial_high_ssim=0
      retry_high_ssim=0
      validation_retry_reason="none"
      validation_status="passed"
      if float_ge "$min_ssim" "$variant_ssim_threshold"; then
        initial_high_ssim=1
      fi
      if [[ "$unique_count" -eq 1 || "$initial_high_ssim" -eq 1 ]]; then
        if [[ "$unique_count" -eq 1 && "$initial_high_ssim" -eq 1 ]]; then
          validation_retry_reason="identical_hash_and_high_ssim"
        elif [[ "$unique_count" -eq 1 ]]; then
          validation_retry_reason="identical_hash"
        else
          validation_retry_reason="high_ssim"
        fi
        validation_status="retrying"
        echo "warn: material variant validation triggered entity=$entity reason=$validation_retry_reason unique_count=$unique_count min_ssim=$min_ssim threshold=$variant_ssim_threshold; retry with fallback closeup camera" >&2
        for retry_variant in default matte glossy; do
          retry_dir="$out_dir/$entity/$retry_variant"
          if [[ ! -d "$retry_dir" ]]; then
            continue
          fi
          retry_port=$((base_port + capture_index))
          capture_index=$((capture_index + 1))
          capture_variant_bundle "$entity" "$retry_variant" "$retry_dir" "$retry_port" "$entity_default_automation_steps" "$entity_retry_closeup_steps" "--no-prewarm" "1"
        done
        unique_count_retry=$(variant_hash_unique_count "$out_dir" "$entity")
        min_ssim_retry=$(variant_min_pair_ssim "$out_dir" "$entity")
        if float_ge "$min_ssim_retry" "$variant_ssim_threshold"; then
          retry_high_ssim=1
        fi
        if [[ "$unique_count_retry" -eq 1 || "$retry_high_ssim" -eq 1 ]]; then
          validation_status="failed_after_retry"
          echo "warn: material variant validation still failed after retry entity=$entity unique_count=$unique_count_retry min_ssim=$min_ssim_retry threshold=$variant_ssim_threshold" >&2
        else
          validation_status="passed_after_retry"
        fi
      fi

      cat >"$out_dir/$entity/variant_validation.txt" <<VALIDATION
entity=$entity
status=$validation_status
retry_reason=$validation_retry_reason
unique_count_initial=$unique_count
unique_count_after_retry=$unique_count_retry
ssim_threshold=$variant_ssim_threshold
min_pair_ssim_initial=$min_ssim
min_pair_ssim_after_retry=$min_ssim_retry
VALIDATION

      for retry_variant in default matte glossy; do
        retry_meta="$out_dir/$entity/$retry_variant/meta.txt"
        if [[ -f "$retry_meta" ]]; then
          echo "variant_validation=$validation_status" >>"$retry_meta"
          echo "variant_validation_retry_reason=$validation_retry_reason" >>"$retry_meta"
          echo "variant_validation_unique_count_initial=$unique_count" >>"$retry_meta"
          echo "variant_validation_unique_count_after_retry=$unique_count_retry" >>"$retry_meta"
          echo "variant_validation_ssim_threshold=$variant_ssim_threshold" >>"$retry_meta"
          echo "variant_validation_min_pair_ssim_initial=$min_ssim" >>"$retry_meta"
          echo "variant_validation_min_pair_ssim_after_retry=$min_ssim_retry" >>"$retry_meta"
        fi
      done
    fi
  fi
done

echo "texture inspector artifacts: $out_dir"
