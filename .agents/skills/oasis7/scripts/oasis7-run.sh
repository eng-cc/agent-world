#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  oasis7-run.sh download [options]
  oasis7-run.sh play [options]
  oasis7-run.sh smoke [options]
  oasis7-run.sh doctor [options]

Options:
  --repo-root <path>              Explicit repo root for repo-backed actions
  --bundle-dir <path>             Extracted release bundle root containing run-game.sh
  --download-release              Download Agent World bundle from GitHub Release before play
  --release-platform <id>         Release asset platform: linux-x64|macos-x64|windows-x64
  --release-tag <tag>             GitHub release tag or latest (default: latest)
  --release-repo <owner/repo>     GitHub repo slug for release download (default: eng-cc/agent-world)
  --download-dir <path>           Release cache/output root (default: ~/.cache/oasis7/releases)
  --force-download                Redownload bundle even if cached bundle already exists
  --base-url <url>                OpenClaw local provider base url (default: http://127.0.0.1:5841)
  --agent-id <id>                 OpenClaw runtime agent id (default: agent_world_runtime)
  --agent-profile <profile>       OpenClaw agent profile (default: agent_world_p0_low_freq_npc)
  --scenario <name>               Gameplay scenario (default: llm_bootstrap)
  --timeout-ms <ms>               Smoke timeout budget (default: 15000)
  --connect-timeout-ms <ms>       Provider connect timeout (default: 15000)
  --samples <n>                   Smoke samples (default: 1)
  --ticks <n>                     Smoke ticks (default: 4)
  --bridge-log <path>             Bridge log path (default: <repo>/.tmp/oasis7-bridge.log or ./.tmp/oasis7-bridge.log)
  --skip-agent-setup              Skip runtime agent bootstrap
  --reuse-bridge                  Reuse existing bridge at --base-url
  --no-open-browser               Pass through to world_game_launcher
  --json                          Emit machine-readable JSON for doctor mode
  -h, --help                      Show help
USAGE
}

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

http_get() {
  local url="$1"
  curl -fsS "$url"
}

wait_for_http() {
  local url="$1"
  local attempts="${2:-40}"
  local sleep_s="${3:-0.5}"
  local i
  for ((i=0; i<attempts; i+=1)); do
    if curl -fsS "$url" >/dev/null 2>&1; then
      return 0
    fi
    sleep "$sleep_s"
  done
  echo "timed out waiting for $url" >&2
  return 1
}

encode_b64() {
  python - <<'PY' "$1"
import base64, sys
print(base64.b64encode(sys.argv[1].encode()).decode())
PY
}

print_doctor_status() {
  local level="$1"
  local label="$2"
  local detail="$3"
  if [[ "$json_output" != "1" ]]; then
    printf '[%s] %s: %s\n' "$level" "$label" "$detail"
  fi
  if [[ -n "$doctor_records_file" ]]; then
    printf '%s\t%s\t%s\n' "$level" "$label" "$(encode_b64 "$detail")" >>"$doctor_records_file"
  fi
}

normalize_path() {
  local value="$1"
  if [[ "$value" == ~* ]]; then
    eval "printf '%s' \"$value\""
    return 0
  fi
  if [[ "$value" == /* ]]; then
    printf '%s' "$value"
  else
    printf '%s/%s' "$PWD" "$value"
  fi
}

validate_repo_root() {
  local candidate="$1"
  [[ -f "$candidate/Cargo.toml" ]] &&
    [[ -f "$candidate/scripts/setup-openclaw-agent-world-runtime.sh" ]] &&
    [[ -f "$candidate/scripts/openclaw-parity-p0.sh" ]]
}

search_repo_root_upwards() {
  local dir="$1"
  while [[ -n "$dir" && "$dir" != "/" ]]; do
    if validate_repo_root "$dir"; then
      printf '%s\n' "$dir"
      return 0
    fi
    dir="$(dirname "$dir")"
  done
  if validate_repo_root "/"; then
    printf '/\n'
    return 0
  fi
  return 1
}

discover_repo_root() {
  local candidate=""
  if [[ -n "$repo_root_override" ]]; then
    candidate="$(normalize_path "$repo_root_override")"
    if validate_repo_root "$candidate"; then
      printf '%s\n' "$candidate"
      return 0
    fi
    echo "error: invalid --repo-root, missing repo markers: $candidate" >&2
    return 1
  fi

  if candidate="$(git rev-parse --show-toplevel 2>/dev/null)" && validate_repo_root "$candidate"; then
    printf '%s\n' "$candidate"
    return 0
  fi

  if candidate="$(search_repo_root_upwards "$PWD" 2>/dev/null)"; then
    printf '%s\n' "$candidate"
    return 0
  fi

  local script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  if candidate="$(search_repo_root_upwards "$script_dir" 2>/dev/null)"; then
    printf '%s\n' "$candidate"
    return 0
  fi

  return 1
}

resolve_bridge_log_default() {
  local root="$1"
  if [[ -n "$bridge_log_override" ]]; then
    printf '%s\n' "$(normalize_path "$bridge_log_override")"
  elif [[ -n "$root" ]]; then
    printf '%s\n' "$root/.tmp/oasis7-bridge.log"
  else
    printf '%s\n' "$PWD/.tmp/oasis7-bridge.log"
  fi
}

validate_bundle_dir() {
  local candidate="$1"
  [[ -x "$candidate/run-game.sh" ]]
}

detect_release_platform() {
  local uname_s
  uname_s="$(uname -s)"
  case "$uname_s" in
    Linux)
      printf 'linux-x64\n'
      ;;
    Darwin)
      printf 'macos-x64\n'
      ;;
    MINGW*|MSYS*|CYGWIN*)
      printf 'windows-x64\n'
      ;;
    *)
      echo "error: unsupported host platform '$uname_s'; pass --release-platform explicitly" >&2
      return 1
      ;;
  esac
}

release_asset_name() {
  case "$1" in
    linux-x64)
      printf 'agent-world-linux-x64.tar.gz\n'
      ;;
    macos-x64)
      printf 'agent-world-macos-x64.tar.gz\n'
      ;;
    windows-x64)
      printf 'agent-world-windows-x64.zip\n'
      ;;
    *)
      echo "error: unsupported --release-platform: $1" >&2
      return 1
      ;;
  esac
}

release_download_url() {
  local repo="$1"
  local tag="$2"
  local asset="$3"
  if [[ "$tag" == "latest" ]]; then
    printf 'https://github.com/%s/releases/latest/download/%s\n' "$repo" "$asset"
  else
    printf 'https://github.com/%s/releases/download/%s/%s\n' "$repo" "$tag" "$asset"
  fi
}

verify_release_checksum() {
  local checksum_path="$1"
  local archive_path="$2"
  python - <<'PY' "$checksum_path" "$archive_path"
import hashlib, pathlib, sys
checksum_path = pathlib.Path(sys.argv[1])
archive_path = pathlib.Path(sys.argv[2])
expected = None
with checksum_path.open('r', encoding='utf-8') as handle:
    for raw in handle:
        parts = raw.strip().split()
        if len(parts) >= 2 and parts[-1] == archive_path.name:
            expected = parts[0]
            break
if expected is None:
    sys.exit(2)
sha = hashlib.sha256()
with archive_path.open('rb') as handle:
    while True:
        chunk = handle.read(1024 * 1024)
        if not chunk:
            break
        sha.update(chunk)
actual = sha.hexdigest()
if actual != expected:
    print(f'checksum mismatch for {archive_path.name}: expected {expected}, got {actual}', file=sys.stderr)
    sys.exit(1)
print(actual)
PY
}

find_extracted_bundle_dir() {
  local extract_root="$1"
  local platform="$2"
  if validate_bundle_dir "$extract_root"; then
    printf '%s\n' "$extract_root"
    return 0
  fi
  if validate_bundle_dir "$extract_root/agent-world-$platform"; then
    printf '%s\n' "$extract_root/agent-world-$platform"
    return 0
  fi
  local marker
  marker="$(find "$extract_root" -maxdepth 3 -type f -name run-game.sh | head -n 1 || true)"
  if [[ -n "$marker" ]]; then
    dirname "$marker"
    return 0
  fi
  echo "error: extracted release bundle does not contain run-game.sh under $extract_root" >&2
  return 1
}

download_release_bundle() {
  require_cmd curl

  local platform="$release_platform"
  if [[ -z "$platform" ]]; then
    platform="$(detect_release_platform)"
  fi
  local asset_name
  asset_name="$(release_asset_name "$platform")"

  local cache_root
  cache_root="$(normalize_path "$download_dir")"
  local repo_key="${release_repo//\//-}"
  local target_root="$cache_root/$repo_key/$release_tag/$platform"
  local bundle_root="$target_root/bundle"
  local archive_path="$target_root/$asset_name"
  local checksum_path="$target_root/agent-world-checksums.txt"
  local extract_root="$target_root/extracted"
  local asset_url
  asset_url="$(release_download_url "$release_repo" "$release_tag" "$asset_name")"
  local checksum_url
  checksum_url="$(release_download_url "$release_repo" "$release_tag" "agent-world-checksums.txt")"

  if [[ "$force_download" != "1" && -x "$bundle_root/run-game.sh" ]]; then
    printf '%s\n' "$bundle_root"
    return 0
  fi

  rm -rf "$target_root"
  mkdir -p "$target_root" "$extract_root"

  echo "Downloading release asset: $asset_url" >&2
  curl -L --fail --show-error --silent -o "$archive_path" "$asset_url"

  if curl -L --fail --show-error --silent -o "$checksum_path" "$checksum_url"; then
    if checksum_value="$(verify_release_checksum "$checksum_path" "$archive_path" 2>/dev/null)"; then
      echo "Verified SHA256: $checksum_value" >&2
    else
      status=$?
      if [[ "$status" -eq 1 ]]; then
        echo "error: release checksum verification failed for $archive_path" >&2
        exit 1
      fi
      echo "warning: checksums file did not contain $asset_name; skipped verification" >&2
    fi
  else
    rm -f "$checksum_path"
    echo "warning: could not download release checksums; skipped verification" >&2
  fi

  case "$asset_name" in
    *.tar.gz)
      require_cmd tar
      tar -xzf "$archive_path" -C "$extract_root"
      ;;
    *.zip)
      require_cmd unzip
      unzip -q "$archive_path" -d "$extract_root"
      ;;
    *)
      echo "error: unsupported release archive format: $asset_name" >&2
      exit 1
      ;;
  esac

  local detected_bundle
  detected_bundle="$(find_extracted_bundle_dir "$extract_root" "$platform")"
  rm -rf "$bundle_root"
  mkdir -p "$bundle_root"
  cp -R "$detected_bundle/." "$bundle_root/"
  printf '%s\n' "$bundle_root"
}

emit_doctor_json() {
  local failures="$1"
  python - <<'PY' "$doctor_records_file" "$failures" "$base_url" "$agent_id" "$agent_profile" "$scenario"
import base64, json, sys
records_path, failures, base_url, agent_id, agent_profile, scenario = sys.argv[1:]
checks = []
with open(records_path, "r", encoding="utf-8") as handle:
    for line in handle:
        level, label, detail_b64 = line.rstrip("\n").split("\t", 2)
        detail = base64.b64decode(detail_b64.encode()).decode()
        checks.append({"level": level, "label": label, "detail": detail})
print(json.dumps({
    "ok": int(failures) == 0,
    "failures": int(failures),
    "base_url": base_url,
    "agent_id": agent_id,
    "agent_profile": agent_profile,
    "scenario": scenario,
    "checks": checks,
}, ensure_ascii=False))
PY
}

run_doctor() {
  local failures=0
  local gateway_json=""
  local provider_info_json=""
  local agents_json=""
  local resolved_repo_root=""
  local resolved_bundle_dir=""
  doctor_records_file="$(mktemp)"

  print_doctor_status INFO config "base_url=$base_url agent_id=$agent_id agent_profile=$agent_profile scenario=$scenario release_repo=$release_repo release_tag=$release_tag"

  if resolved_repo_root="$(discover_repo_root 2>/dev/null)"; then
    print_doctor_status OK repo-root "$resolved_repo_root"
  else
    print_doctor_status INFO repo-root "not resolved (needed only for agent bootstrap, bridge launch, or smoke)"
  fi

  if [[ -n "$bundle_dir" ]]; then
    resolved_bundle_dir="$(normalize_path "$bundle_dir")"
    if validate_bundle_dir "$resolved_bundle_dir"; then
      print_doctor_status OK bundle-dir "$resolved_bundle_dir"
    else
      print_doctor_status FAIL bundle-dir "missing run-game.sh under $resolved_bundle_dir"
      failures=$((failures + 1))
    fi
  elif [[ "$download_release" == "1" ]]; then
    print_doctor_status INFO bundle-dir "download on demand enabled (platform=${release_platform:-auto}, cache=$(normalize_path "$download_dir"))"
  else
    print_doctor_status INFO bundle-dir "not configured"
  fi

  if command -v openclaw >/dev/null 2>&1; then
    print_doctor_status OK command "openclaw=$(command -v openclaw)"
  else
    print_doctor_status FAIL command "openclaw not found"
    failures=$((failures + 1))
  fi

  if command -v cargo >/dev/null 2>&1; then
    print_doctor_status OK command "cargo=$(command -v cargo)"
  else
    print_doctor_status FAIL command "cargo not found"
    failures=$((failures + 1))
  fi

  if gateway_json="$(http_get 'http://127.0.0.1:18789/health' 2>/dev/null)"; then
    print_doctor_status OK gateway "$gateway_json"
  else
    print_doctor_status FAIL gateway "cannot reach http://127.0.0.1:18789/health"
    failures=$((failures + 1))
  fi

  if agents_json="$(openclaw agents list --json 2>/dev/null)"; then
    if AGENTS_JSON="$agents_json" AGENT_ID="$agent_id" python - <<'PY' >/dev/null
import json, os, sys
agent_id = os.environ['AGENT_ID']
items = json.loads(os.environ['AGENTS_JSON'])
for item in items:
    if item.get('id') == agent_id:
        sys.exit(0)
sys.exit(1)
PY
    then
      local agent_summary
      agent_summary="$(AGENTS_JSON="$agents_json" AGENT_ID="$agent_id" python - <<'PY'
import json, os
agent_id = os.environ['AGENT_ID']
items = json.loads(os.environ['AGENTS_JSON'])
for item in items:
    if item.get('id') == agent_id:
        print(f"workspace={item.get('workspace','')} model={item.get('model','')}")
        break
PY
)"
      print_doctor_status OK runtime-agent "$agent_summary"
    else
      if [[ -n "$resolved_repo_root" ]]; then
        print_doctor_status FAIL runtime-agent "OpenClaw agent '$agent_id' not found; run $resolved_repo_root/scripts/setup-openclaw-agent-world-runtime.sh $agent_id"
      else
        print_doctor_status FAIL runtime-agent "OpenClaw agent '$agent_id' not found; provide --repo-root and run scripts/setup-openclaw-agent-world-runtime.sh $agent_id"
      fi
      failures=$((failures + 1))
    fi
  else
    print_doctor_status FAIL runtime-agent "failed to query 'openclaw agents list --json'"
    failures=$((failures + 1))
  fi

  if http_get "$base_url/v1/provider/health" >/dev/null 2>&1; then
    print_doctor_status OK bridge-health "$base_url/v1/provider/health reachable"
  else
    print_doctor_status FAIL bridge-health "cannot reach $base_url/v1/provider/health"
    failures=$((failures + 1))
  fi

  if provider_info_json="$(http_get "$base_url/v1/provider/info" 2>/dev/null)"; then
    local provider_summary
    provider_summary="$(PROVIDER_INFO_JSON="$provider_info_json" python - <<'PY'
import json, os
value = json.loads(os.environ['PROVIDER_INFO_JSON'])
provider_id = value.get('provider_id', '')
provider_version = value.get('provider_version', '')
protocol_version = value.get('protocol_version', '')
print(f"provider_id={provider_id} provider_version={provider_version} protocol_version={protocol_version}")
PY
)"
    print_doctor_status OK provider-info "$provider_summary"
  else
    print_doctor_status FAIL provider-info "cannot reach $base_url/v1/provider/info"
    failures=$((failures + 1))
  fi

  if [[ -f "$bridge_log" ]]; then
    print_doctor_status INFO bridge-log "$bridge_log"
  else
    print_doctor_status INFO bridge-log "not created yet ($bridge_log)"
  fi

  if [[ "$failures" -eq 0 ]]; then
    print_doctor_status OK summary "doctor checks passed"
    if [[ "$json_output" == "1" ]]; then
      emit_doctor_json "$failures"
    fi
    return 0
  fi

  print_doctor_status FAIL summary "$failures check(s) failed"
  if [[ "$json_output" == "1" ]]; then
    emit_doctor_json "$failures"
  fi
  return 1
}

mode="${1:-}"
if [[ -z "$mode" || "$mode" == "-h" || "$mode" == "--help" ]]; then
  usage
  exit 0
fi
shift

repo_root_override=""
bundle_dir=""
download_release="0"
release_platform=""
release_tag="latest"
release_repo="eng-cc/agent-world"
download_dir="~/.cache/oasis7/releases"
force_download="0"
base_url="http://127.0.0.1:5841"
agent_id="agent_world_runtime"
agent_profile="agent_world_p0_low_freq_npc"
scenario="llm_bootstrap"
timeout_ms="15000"
connect_timeout_ms="15000"
samples="1"
ticks="4"
bridge_log_override=""
skip_agent_setup="0"
reuse_bridge="0"
open_browser="1"
json_output="0"
doctor_records_file=""
repo_root=""
bridge_log=""
cleanup_bridge_pid=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo-root)
      repo_root_override="$2"
      shift 2
      ;;
    --bundle-dir)
      bundle_dir="$2"
      shift 2
      ;;
    --download-release)
      download_release="1"
      shift
      ;;
    --release-platform)
      release_platform="$2"
      shift 2
      ;;
    --release-tag)
      release_tag="$2"
      shift 2
      ;;
    --release-repo)
      release_repo="$2"
      shift 2
      ;;
    --download-dir)
      download_dir="$2"
      shift 2
      ;;
    --force-download)
      force_download="1"
      shift
      ;;
    --base-url)
      base_url="$2"
      shift 2
      ;;
    --agent-id)
      agent_id="$2"
      shift 2
      ;;
    --agent-profile)
      agent_profile="$2"
      shift 2
      ;;
    --scenario)
      scenario="$2"
      shift 2
      ;;
    --timeout-ms)
      timeout_ms="$2"
      shift 2
      ;;
    --connect-timeout-ms)
      connect_timeout_ms="$2"
      shift 2
      ;;
    --samples)
      samples="$2"
      shift 2
      ;;
    --ticks)
      ticks="$2"
      shift 2
      ;;
    --bridge-log)
      bridge_log_override="$2"
      shift 2
      ;;
    --skip-agent-setup)
      skip_agent_setup="1"
      shift
      ;;
    --reuse-bridge)
      reuse_bridge="1"
      shift
      ;;
    --no-open-browser)
      open_browser="0"
      shift
      ;;
    --json)
      json_output="1"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

cleanup() {
  if [[ -n "$cleanup_bridge_pid" ]]; then
    kill "$cleanup_bridge_pid" >/dev/null 2>&1 || true
    wait "$cleanup_bridge_pid" >/dev/null 2>&1 || true
  fi
  if [[ -n "$doctor_records_file" && -f "$doctor_records_file" ]]; then
    rm -f "$doctor_records_file"
  fi
}
trap cleanup EXIT

if [[ -n "$bundle_dir" ]]; then
  bundle_dir="$(normalize_path "$bundle_dir")"
fi

if [[ "$mode" == "download" ]]; then
  download_release="1"
fi

if [[ "$download_release" == "1" ]]; then
  bundle_dir="$(download_release_bundle)"
fi

repo_required="0"
need_cargo="0"
need_openclaw="0"
use_bundle_play="0"

case "$mode" in
  download)
    ;;
  doctor)
    require_cmd curl
    ;;
  play)
    if [[ -n "$bundle_dir" ]]; then
      if ! validate_bundle_dir "$bundle_dir"; then
        echo "error: invalid --bundle-dir, missing run-game.sh: $bundle_dir" >&2
        exit 1
      fi
      use_bundle_play="1"
    fi
    if [[ "$skip_agent_setup" != "1" ]]; then
      repo_required="1"
      need_openclaw="1"
    fi
    if [[ "$reuse_bridge" != "1" ]]; then
      repo_required="1"
      need_cargo="1"
    fi
    if [[ "$use_bundle_play" != "1" ]]; then
      repo_required="1"
      need_cargo="1"
    fi
    require_cmd curl
    ;;
  smoke)
    repo_required="1"
    need_cargo="1"
    if [[ "$skip_agent_setup" != "1" ]]; then
      need_openclaw="1"
    fi
    require_cmd curl
    ;;
  *)
    echo "unknown mode: $mode" >&2
    usage >&2
    exit 1
    ;;
esac

if [[ "$repo_required" == "1" ]]; then
  repo_root="$(discover_repo_root || true)"
  if [[ -z "$repo_root" ]]; then
    echo "error: repo root is required for '$mode'; pass --repo-root <path>" >&2
    exit 1
  fi
fi

bridge_log="$(resolve_bridge_log_default "$repo_root")"
mkdir -p "$(dirname "$bridge_log")"

if [[ "$need_openclaw" == "1" ]]; then
  require_cmd openclaw
fi
if [[ "$need_cargo" == "1" ]]; then
  require_cmd cargo
fi

if [[ "$mode" == "doctor" ]]; then
  run_doctor
  exit $?
fi

if [[ "$mode" == "download" ]]; then
  printf '%s\n' "$bundle_dir"
  exit 0
fi

if [[ "$skip_agent_setup" != "1" ]]; then
  wait_for_http "http://127.0.0.1:18789/health" 20 0.5
  "$repo_root/scripts/setup-openclaw-agent-world-runtime.sh" "$agent_id"
fi

if [[ "$reuse_bridge" != "1" ]]; then
  wait_for_http "http://127.0.0.1:18789/health" 20 0.5
  (
    cd "$repo_root"
    exec env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_openclaw_local_bridge -- --openclaw-agent "$agent_id"
  ) >"$bridge_log" 2>&1 &
  cleanup_bridge_pid="$!"
fi

wait_for_http "$base_url/v1/provider/health" 40 0.5

case "$mode" in
  play)
    if [[ "$use_bundle_play" == "1" ]]; then
      cmd=("$bundle_dir/run-game.sh"
        --scenario "$scenario"
        --with-llm
        --agent-provider-mode openclaw_local_http
        --openclaw-base-url "$base_url"
        --openclaw-connect-timeout-ms "$connect_timeout_ms"
        --openclaw-agent-profile "$agent_profile")
    else
      cd "$repo_root"
      cmd=(env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_game_launcher --
        --scenario "$scenario"
        --with-llm
        --agent-provider-mode openclaw_local_http
        --openclaw-base-url "$base_url"
        --openclaw-connect-timeout-ms "$connect_timeout_ms"
        --openclaw-agent-profile "$agent_profile")
    fi
    if [[ "$open_browser" != "1" ]]; then
      cmd+=(--no-open-browser)
    fi
    printf 'Running: %q ' "${cmd[@]}"
    printf '\n'
    "${cmd[@]}"
    ;;
  smoke)
    cd "$repo_root"
    cmd=(bash scripts/openclaw-parity-p0.sh
      --openclaw-only
      --samples "$samples"
      --ticks "$ticks"
      --timeout-ms "$timeout_ms"
      --openclaw-base-url "$base_url"
      --openclaw-connect-timeout-ms "$connect_timeout_ms"
      --openclaw-agent-profile "$agent_profile")
    printf 'Running: %q ' "${cmd[@]}"
    printf '\n'
    "${cmd[@]}"
    ;;
esac
