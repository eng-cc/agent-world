# Shared function library for viewer-texture-inspector.sh
# Keep CLI parsing and main execution in the entry script.

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
    echo "agent location asset power_plant"
    return 0
  fi

  local parsed=()
  IFS=',' read -r -a parsed <<<"$normalized"
  local item
  for item in "${parsed[@]}"; do
    case "$item" in
      agent|location|asset|power_plant)
        ;;
      *)
        echo "invalid inspect entity: $item" >&2
        echo "supported entities: agent,location,asset,power_plant,all" >&2
        exit 2
        ;;
    esac
  done
  echo "${parsed[*]}"
}

resolve_default_preset_file_for_pack() {
  case "$1" in
    industrial_v3)
      echo "crates/agent_world_viewer/assets/themes/industrial_v3/presets/industrial_v3_default.env"
      ;;
    industrial_v2)
      echo "crates/agent_world_viewer/assets/themes/industrial_v2/presets/industrial_v2_default.env"
      ;;
    industrial_v1)
      echo "crates/agent_world_viewer/assets/themes/industrial_v1/presets/industrial_default.env"
      ;;
    *)
      return 1
      ;;
  esac
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

viewer_env_key() {
  local suffix=$1
  echo "OASIS7_VIEWER_${suffix}"
}

viewer_legacy_env_key() {
  local suffix=$1
  echo "AGENT_WORLD_VIEWER_${suffix}"
}

set_or_unset_viewer_env() {
  local suffix=$1
  local value=$2
  local key
  local legacy_key
  key=$(viewer_env_key "$suffix")
  legacy_key=$(viewer_legacy_env_key "$suffix")
  if [[ -n "$value" ]]; then
    export "$key=$value"
    unset "$legacy_key" || true
  else
    unset "$key" || true
    unset "$legacy_key" || true
  fi
}

viewer_env_value() {
  local suffix=$1
  local key
  local legacy_key
  key=$(viewer_env_key "$suffix")
  legacy_key=$(viewer_legacy_env_key "$suffix")
  if [[ -n "${!key-}" ]]; then
    printf '%s' "${!key}"
  else
    printf '%s' "${!legacy_key-}"
  fi
}

promote_legacy_viewer_envs() {
  local legacy_name
  local value
  local suffix
  local key
  while IFS='=' read -r legacy_name value; do
    [[ "$legacy_name" == AGENT_WORLD_VIEWER_* ]] || continue
    suffix=${legacy_name#AGENT_WORLD_VIEWER_}
    key=$(viewer_env_key "$suffix")
    if [[ -z "${!key+x}" ]]; then
      export "$key=$value"
    fi
  done < <(env)
}

capture_status_value() {
  local status_file=$1
  local key=$2
  awk -F'=' -v wanted="$key" '
    $1 == wanted {
      value = substr($0, index($0, "=") + 1)
    }
    END {
      print value
    }
  ' "$status_file"
}

upper_key_name() {
  local raw=${1:-}
  echo "$raw" | tr '[:lower:]' '[:upper:]' | tr '-' '_'
}

resource_pack_value() {
  local entity=$1
  local variant=$2
  local field=$3
  local entity_key
  entity_key=$(upper_key_name "$entity")
  local variant_key
  variant_key=$(upper_key_name "$variant")
  local variant_level_key="TEXTURE_INSPECTOR_RESOURCE_${entity_key}_${variant_key}_${field}"
  local entity_level_key="TEXTURE_INSPECTOR_RESOURCE_${entity_key}_${field}"

  local value="${!variant_level_key-}"
  if [[ -n "$value" ]]; then
    echo "$value"
    return 0
  fi

  echo "${!entity_level_key-}"
}

resolve_focus_target_for_entity() {
  local entity=$1
  local scenario_name=${2:-}
  case "$entity" in
    agent)
      echo "first_agent"
      ;;
    asset)
      echo "first_asset"
      ;;
    location)
      echo "first_location"
      ;;
    power_plant)
      if [[ "$scenario_name" == *power* ]]; then
        echo "first_power_plant"
      else
        echo "first_location"
      fi
      ;;
    *)
      echo "first_location"
      ;;
  esac
}

resolve_capture_scenario_for_entity() {
  local entity=$1
  local scenario_name=${2:-llm_bootstrap}
  local preview_mode_effective=${3:-scene_proxy}
  if [[ "$preview_mode_effective" == "direct_entity" && "$entity" == "power_plant" && "$scenario_name" != *power* ]]; then
    echo "power_bootstrap"
    return 0
  fi
  echo "$scenario_name"
}

resolve_power_pose() {
  local pose_kind=$1
  local focus_target=$2
  local profile=${3:-legacy}
  local focus_bucket="entity"
  if [[ "$focus_target" == "first_location" ]]; then
    focus_bucket="location"
  fi

  case "${profile}:${pose_kind}:${focus_bucket}" in
    art_review_v2:hero:location)
      echo "pan=0.3,0,0;zoom=1.4;orbit=18,-28;wait=0.7"
      ;;
    art_review_v2:hero:entity)
      echo "zoom=3.2;orbit=18,-28;wait=0.7"
      ;;
    art_review_v2:closeup:location)
      # Wider baseline closeup keeps giant proxy meshes readable.
      echo "pan=0.45,0,0;zoom=1.75;orbit=24,-24;wait=0.9"
      ;;
    art_review_v2:closeup:entity)
      # direct_entity power meshes need a safer radius to avoid clipping inside geometry.
      echo "zoom=3.2;orbit=36,-18;wait=0.9"
      ;;
    art_review_v2:fallback:location)
      echo "pan=0.5,0,0;zoom=1.95;orbit=30,-20;wait=1.0"
      ;;
    art_review_v2:fallback:entity)
      echo "zoom=3.6;orbit=42,-16;wait=1.0"
      ;;
    *:hero:location)
      echo "pan=0.4,0,0;zoom=1.6;orbit=20,-30;wait=0.6"
      ;;
    *:hero:entity)
      echo "zoom=3.0;orbit=20,-30;wait=0.6"
      ;;
    *:closeup:location)
      echo "pan=0.6,0,0;zoom=2.2;orbit=20,-30;wait=0.8"
      ;;
    *:closeup:entity)
      echo "zoom=3.0;orbit=34,-20;wait=0.8"
      ;;
    *:fallback:location)
      echo "pan=0.6,0,0;zoom=2.6;orbit=20,-30;wait=0.9"
      ;;
    *:fallback:entity)
      echo "zoom=3.3;orbit=40,-16;wait=0.9"
      ;;
    *)
      echo "zoom=3.0;orbit=34,-20;wait=0.8"
      ;;
  esac
}

emit_power_retry_closeup_candidates() {
  local focus_target=$1
  local profile=${2:-legacy}
  if [[ "$profile" != "art_review_v2" ]]; then
    echo "fallback|mode=3d;wait=1.0;select=${focus_target};focus=${focus_target};$(resolve_power_pose fallback "$focus_target" "$profile")"
    return 0
  fi

  if [[ "$focus_target" == "first_location" ]]; then
    echo "wide_a|mode=3d;wait=1.0;select=${focus_target};focus=${focus_target};pan=0.52,0,0;zoom=2.05;orbit=22,-24;wait=1.0"
    echo "wide_b|mode=3d;wait=1.0;select=${focus_target};focus=${focus_target};pan=0.36,0,0;zoom=1.85;orbit=34,-18;wait=1.0"
    echo "silhouette|mode=3d;wait=1.0;select=${focus_target};focus=${focus_target};pan=0.28,0,0;zoom=1.65;orbit=48,-14;wait=1.1"
  else
    echo "wide_a|mode=3d;wait=1.0;select=${focus_target};focus=${focus_target};zoom=3.6;orbit=28,-22;wait=1.0"
    echo "three_quarter|mode=3d;wait=1.0;select=${focus_target};focus=${focus_target};pan=0.08,0,0;zoom=3.2;orbit=42,-18;wait=1.0"
    echo "silhouette|mode=3d;wait=1.0;select=${focus_target};focus=${focus_target};zoom=3.9;orbit=58,-12;wait=1.1"
  fi
}

default_automation_steps_for_entity() {
  local entity=$1
  local scenario_name=${2:-$scenario}
  local focus_target
  focus_target=$(resolve_focus_target_for_entity "$entity" "$scenario_name")
  local power_hero_pose
  power_hero_pose=$(resolve_power_pose hero "$focus_target" "$composition_profile")
  case "$1" in
    agent)
      echo "mode=3d;wait=1.0;select=${focus_target};focus=${focus_target};zoom=3.1;orbit=24,-22;wait=0.7"
      ;;
    location)
      echo "mode=3d;wait=1.0;select=${focus_target};focus=${focus_target};zoom=1.7;orbit=14,-24;wait=0.6"
      ;;
    asset)
      echo "mode=3d;wait=1.0;select=${focus_target};focus=${focus_target};zoom=3.0;orbit=22,-24;wait=0.7"
      ;;
    power_plant)
      echo "mode=3d;wait=1.0;select=${focus_target};focus=${focus_target};${power_hero_pose}"
      ;;
    *)
      echo "mode=3d;focus=first_location;pan=0,2,0;zoom=1.2;orbit=10,-25;select=first_location;wait=0.4"
      ;;
  esac
}

default_closeup_automation_steps_for_entity() {
  local entity=$1
  local scenario_name=${2:-$scenario}
  local focus_target
  focus_target=$(resolve_focus_target_for_entity "$entity" "$scenario_name")
  local power_closeup_pose
  power_closeup_pose=$(resolve_power_pose closeup "$focus_target" "$composition_profile")
  case "$1" in
    agent)
      echo "mode=3d;wait=1.0;select=${focus_target};focus=${focus_target};zoom=3.6;orbit=36,-16;wait=0.9"
      ;;
    location)
      echo "mode=3d;wait=1.0;select=${focus_target};focus=${focus_target};zoom=1.25;orbit=28,-20;wait=0.8"
      ;;
    asset)
      echo "mode=3d;wait=1.0;select=${focus_target};focus=${focus_target};zoom=3.4;orbit=34,-18;wait=0.9"
      ;;
    power_plant)
      echo "mode=3d;wait=1.0;select=${focus_target};focus=${focus_target};${power_closeup_pose}"
      ;;
    *)
      echo "mode=3d;focus=first_location;zoom=0.18;orbit=30,-22;wait=0.7"
      ;;
  esac
}

fallback_closeup_automation_steps_for_entity() {
  local entity=$1
  local scenario_name=${2:-$scenario}
  local focus_target
  focus_target=$(resolve_focus_target_for_entity "$entity" "$scenario_name")
  local power_fallback_pose
  power_fallback_pose=$(resolve_power_pose fallback "$focus_target" "$composition_profile")
  case "$1" in
    power_plant)
      echo "mode=3d;wait=1.0;select=${focus_target};focus=${focus_target};${power_fallback_pose}"
      ;;
    *)
      echo "mode=3d;wait=1.0;select=${focus_target};focus=${focus_target};zoom=0.95;orbit=42,-16;wait=0.9"
      ;;
  esac
}

retry_closeup_candidate_specs_for_entity() {
  local entity=$1
  local scenario_name=${2:-$scenario}
  local focus_target
  focus_target=$(resolve_focus_target_for_entity "$entity" "$scenario_name")
  case "$entity" in
    power_plant)
      emit_power_retry_closeup_candidates "$focus_target" "$composition_profile"
      ;;
    *)
      echo "fallback|$(fallback_closeup_automation_steps_for_entity "$entity" "$scenario_name")"
      ;;
  esac
}

expected_selection_kind_for_entity() {
  local entity=$1
  local preview_mode_effective=$2
  if [[ "$preview_mode_effective" != "direct_entity" ]]; then
    echo "location"
    return 0
  fi
  case "$entity" in
    power_plant)
      echo "power_plant"
      ;;
    agent)
      echo "agent"
      ;;
    location)
      echo "location"
      ;;
    asset)
      echo "asset"
      ;;
    *)
      echo ""
      ;;
  esac
}

semantic_gate_enforced_for_entity() {
  local entity=$1
  local mode=$2
  local art_capture_flag=$3
  local preview_mode_effective=$4
  local _unused=$entity
  local _unused_preview=$preview_mode_effective
  case "$mode" in
    off)
      return 1
      ;;
    strict)
      return 0
      ;;
    auto)
      if [[ "$art_capture_flag" -eq 1 ]]; then
        return 0
      fi
      return 1
      ;;
    *)
      return 1
      ;;
  esac
}

parse_crop_window() {
  local raw=${1:-none}
  local normalized
  normalized=$(echo "$raw" | tr '[:upper:]' '[:lower:]')
  if [[ "$normalized" == "auto" ]]; then
    echo "auto"
    return 0
  fi
  if [[ "$normalized" == "none" || "$normalized" == "off" || "$normalized" == "disable" ]]; then
    echo "none"
    return 0
  fi

  IFS=':' read -r crop_w crop_h crop_x crop_y extra <<<"$raw"
  if [[ -n "${extra:-}" || -z "${crop_w:-}" || -z "${crop_h:-}" || -z "${crop_x:-}" || -z "${crop_y:-}" ]]; then
    echo "invalid --crop-window: $raw (expected w:h:x:y, auto, or none)" >&2
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

resolve_effective_crop_window() {
  local crop_window_request=$1
  local entity=$2
  local art_capture_flag=$3
  local panel_hidden_flag=$4

  if [[ "$crop_window_request" != "auto" ]]; then
    echo "$crop_window_request"
    return 0
  fi

  if [[ "$art_capture_flag" -ne 1 ]]; then
    echo "none"
    return 0
  fi

  if [[ "$panel_hidden_flag" -eq 1 ]]; then
    case "$entity" in
      power_plant)
        echo "920:700:120:50"
        ;;
      *)
        echo "900:680:130:60"
        ;;
    esac
    return 0
  fi

  case "$entity" in
    power_plant)
      echo "600:620:0:120"
      ;;
    *)
      echo "600:620:0:120"
      ;;
  esac
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

parse_non_negative_float_or_exit() {
  local raw=$1
  local option_name=$2
  if ! awk -v value="$raw" 'BEGIN { exit !(value ~ /^[0-9]*\.?[0-9]+$/ && value >= 0) }'; then
    echo "invalid ${option_name}: $raw (expected >= 0)" >&2
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

meta_value() {
  local meta_file=$1
  local key=$2
  if [[ ! -f "$meta_file" ]]; then
    echo ""
    return 0
  fi
  awk -F'=' -v wanted="$key" '
    $1 == wanted {
      value = substr($0, index($0, "=") + 1)
    }
    END {
      print value
    }
  ' "$meta_file"
}

image_edge_energy() {
  local path=$1
  if [[ ! -f "$path" ]]; then
    echo "0"
    return 0
  fi

  local value
  value=$(ffmpeg -i "$path" -vf "format=gray,edgedetect=low=0.08:high=0.2,signalstats,metadata=print:file=-" -f null - 2>/dev/null | grep 'lavfi.signalstats.YAVG=' | tail -n 1 | sed -E 's/.*lavfi.signalstats.YAVG=([0-9.]+).*/\1/' || true)
  if [[ -z "$value" ]]; then
    echo "0"
    return 0
  fi
  echo "$value"
}

variant_pair_ssim_value() {
  local root=$1
  local entity=$2
  local a=$3
  local b=$4
  local path_a="$root/$entity/$a/viewer_art_closeup_ssim.png"
  local path_b="$root/$entity/$b/viewer_art_closeup_ssim.png"
  if [[ ! -f "$path_a" ]]; then
    path_a="$root/$entity/$a/viewer_art_closeup.png"
  fi
  if [[ ! -f "$path_b" ]]; then
    path_b="$root/$entity/$b/viewer_art_closeup.png"
  fi

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

clear_material_overrides() {
  set_or_unset_viewer_env "MATERIAL_AGENT_ROUGHNESS" ""
  set_or_unset_viewer_env "MATERIAL_AGENT_METALLIC" ""
  set_or_unset_viewer_env "MATERIAL_ASSET_ROUGHNESS" ""
  set_or_unset_viewer_env "MATERIAL_ASSET_METALLIC" ""
  set_or_unset_viewer_env "MATERIAL_FACILITY_ROUGHNESS" ""
  set_or_unset_viewer_env "MATERIAL_FACILITY_METALLIC" ""
  set_or_unset_viewer_env "MATERIAL_POWER_PLANT_ROUGHNESS" ""
  set_or_unset_viewer_env "MATERIAL_POWER_PLANT_METALLIC" ""
  set_or_unset_viewer_env "MATERIAL_POWER_PLANT_EMISSIVE_BOOST" ""
  set_or_unset_viewer_env "POWER_PLANT_BASE_COLOR" ""
  set_or_unset_viewer_env "POWER_PLANT_EMISSIVE_COLOR" ""
}

apply_variant_material_profile() {
  local profile=$1
  local entity=$2
  local variant=$3

  clear_material_overrides
  if [[ "$profile" != "art_review_v1" ]]; then
    return 0
  fi

  if [[ "$entity" == "power_plant" ]]; then
    local roughness
    local metallic
    local emissive_boost
    local base_color
    local emissive_color
    case "$variant" in
      default)
        roughness="0.45"
        metallic="0.28"
        emissive_boost="0.10"
        base_color="#F36934"
        emissive_color="#FF7F4A"
        ;;
      matte)
        roughness="1.00"
        metallic="0.00"
        emissive_boost="0.00"
        base_color="#140F0A"
        emissive_color="#000000"
        ;;
      glossy)
        roughness="0.00"
        metallic="1.00"
        emissive_boost="1.20"
        base_color="#FFF5DD"
        emissive_color="#FFFFFF"
        ;;
      *)
        return 0
        ;;
    esac

    set_or_unset_viewer_env "MATERIAL_POWER_PLANT_ROUGHNESS" "$roughness"
    set_or_unset_viewer_env "MATERIAL_POWER_PLANT_METALLIC" "$metallic"
    set_or_unset_viewer_env "MATERIAL_POWER_PLANT_EMISSIVE_BOOST" "$emissive_boost"
    set_or_unset_viewer_env "POWER_PLANT_BASE_COLOR" "$base_color"
    set_or_unset_viewer_env "POWER_PLANT_EMISSIVE_COLOR" "$emissive_color"
    return 0
  fi

  local material_group
  case "$entity" in
    agent)
      material_group="agent"
      ;;
    asset)
      material_group="asset"
      ;;
    location)
      material_group="facility"
      ;;
    *)
      return 0
      ;;
  esac

  local roughness
  local metallic
  case "$material_group:$variant" in
    agent:default)
      roughness="0.38"
      metallic="0.08"
      ;;
    agent:matte)
      roughness="0.70"
      metallic="0.03"
      ;;
    agent:glossy)
      roughness="0.22"
      metallic="0.22"
      ;;
    asset:default)
      roughness="0.55"
      metallic="0.12"
      ;;
    asset:matte)
      roughness="0.78"
      metallic="0.04"
      ;;
    asset:glossy)
      roughness="0.28"
      metallic="0.28"
      ;;
    facility:default)
      roughness="0.48"
      metallic="0.20"
      ;;
    facility:matte)
      roughness="0.82"
      metallic="0.05"
      ;;
    facility:glossy)
      roughness="0.18"
      metallic="0.42"
      ;;
    *)
      return 0
      ;;
  esac

  case "$material_group" in
    agent)
      set_or_unset_viewer_env "MATERIAL_AGENT_ROUGHNESS" "$roughness"
      set_or_unset_viewer_env "MATERIAL_AGENT_METALLIC" "$metallic"
      ;;
    asset)
      set_or_unset_viewer_env "MATERIAL_ASSET_ROUGHNESS" "$roughness"
      set_or_unset_viewer_env "MATERIAL_ASSET_METALLIC" "$metallic"
      ;;
    facility)
      set_or_unset_viewer_env "MATERIAL_FACILITY_ROUGHNESS" "$roughness"
      set_or_unset_viewer_env "MATERIAL_FACILITY_METALLIC" "$metallic"
      ;;
  esac
}

apply_art_lighting_profile() {
  local profile=$1
  local entity=$2
  local variant=$3

  case "$profile" in
    art_review_v1)
      set_or_unset_viewer_env "TONEMAPPING" "aces"
      set_or_unset_viewer_env "BLOOM_ENABLED" "0"
      set_or_unset_viewer_env "BLOOM_INTENSITY" "0"
      set_or_unset_viewer_env "COLOR_GRADING_EXPOSURE" "-0.35"
      set_or_unset_viewer_env "AMBIENT_BRIGHTNESS" "95"
      set_or_unset_viewer_env "FILL_LIGHT_RATIO" "0.46"
      set_or_unset_viewer_env "RIM_LIGHT_RATIO" "0.32"
      set_or_unset_viewer_env "EXPOSURE_EV100" "12.8"
      ;;
    art_review_v2)
      set_or_unset_viewer_env "TONEMAPPING" "aces"
      set_or_unset_viewer_env "BLOOM_ENABLED" "0"
      set_or_unset_viewer_env "BLOOM_INTENSITY" "0"

      case "$entity" in
        power_plant)
          set_or_unset_viewer_env "COLOR_GRADING_EXPOSURE" "-0.05"
          set_or_unset_viewer_env "AMBIENT_BRIGHTNESS" "88"
          set_or_unset_viewer_env "FILL_LIGHT_RATIO" "0.36"
          set_or_unset_viewer_env "RIM_LIGHT_RATIO" "0.58"
          set_or_unset_viewer_env "EXPOSURE_EV100" "12.4"
          ;;
        asset|agent)
          set_or_unset_viewer_env "COLOR_GRADING_EXPOSURE" "-0.16"
          set_or_unset_viewer_env "AMBIENT_BRIGHTNESS" "92"
          set_or_unset_viewer_env "FILL_LIGHT_RATIO" "0.42"
          set_or_unset_viewer_env "RIM_LIGHT_RATIO" "0.44"
          set_or_unset_viewer_env "EXPOSURE_EV100" "12.6"
          ;;
        *)
          set_or_unset_viewer_env "COLOR_GRADING_EXPOSURE" "-0.22"
          set_or_unset_viewer_env "AMBIENT_BRIGHTNESS" "94"
          set_or_unset_viewer_env "FILL_LIGHT_RATIO" "0.40"
          set_or_unset_viewer_env "RIM_LIGHT_RATIO" "0.40"
          set_or_unset_viewer_env "EXPOSURE_EV100" "12.7"
          ;;
      esac

      if [[ "$variant" == "matte" ]]; then
        set_or_unset_viewer_env "COLOR_GRADING_EXPOSURE" "0.00"
      elif [[ "$variant" == "glossy" ]]; then
        set_or_unset_viewer_env "COLOR_GRADING_EXPOSURE" "-0.10"
      fi
      ;;
    *)
      ;;
  esac
}

resolve_variant_texture_override() {
  local shared_override=$1
  local template_override=$2
  local variant=$3

  if [[ -n "$template_override" ]]; then
    echo "${template_override//\{variant\}/$variant}"
    return 0
  fi
  if [[ -n "$shared_override" ]]; then
    echo "$shared_override"
    return 0
  fi
  echo ""
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

variant_min_edge_energy() {
  local root=$1
  local entity=$2
  local min_edge="999999"
  local variant
  for variant in default matte glossy; do
    local path="$root/$entity/$variant/viewer_art_closeup.png"
    local value
    value=$(image_edge_energy "$path")
    if float_lt "$value" "$min_edge"; then
      min_edge="$value"
    fi
  done
  if [[ "$min_edge" == "999999" ]]; then
    echo "0"
  else
    echo "$min_edge"
  fi
}

variant_semantic_fail_count() {
  local root=$1
  local entity=$2
  local fail_count=0
  local variant
  for variant in default matte glossy; do
    local meta_file="$root/$entity/$variant/meta.txt"
    local pass
    pass=$(meta_value "$meta_file" "selection_gate_pass")
    if [[ "$pass" == "0" ]]; then
      fail_count=$((fail_count + 1))
    fi
  done
  echo "$fail_count"
}
