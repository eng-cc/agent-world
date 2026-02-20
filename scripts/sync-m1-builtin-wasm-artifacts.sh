#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$ROOT_DIR/.tmp/builtin-wasm-sync-modules"
PROFILE="release"
CHECK_ONLY=0
MODULE_IDS_PATH="$ROOT_DIR/crates/agent_world/src/runtime/world/artifacts/m1_builtin_module_ids.txt"
MODULE_MANIFEST_MAP_PATH="$ROOT_DIR/crates/agent_world/src/runtime/world/artifacts/builtin_module_manifest_map.txt"
HASH_MANIFEST_PATH="$ROOT_DIR/crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256"
IDENTITY_MANIFEST_PATH=""
DISTFS_ROOT="$ROOT_DIR/.distfs/builtin_wasm"
DISTFS_BLOBS_DIR="$DISTFS_ROOT/blobs"
CANONICAL_PLATFORMS_CSV="${AGENT_WORLD_WASM_CANONICAL_PLATFORMS:-darwin-arm64,linux-x86_64}"
CURRENT_PLATFORM=""
CURRENT_PLATFORM_IS_CANONICAL=0

declare -a MODULE_IDS=()
declare -a CANONICAL_PLATFORMS=()

usage() {
  cat <<'USAGE'
Usage:
  scripts/sync-m1-builtin-wasm-artifacts.sh [options]

Options:
  --check                 Build and verify hash manifest vs built wasm, then hydrate DistFS blobs
  --profile <name>        Cargo profile forwarded to wasm build suite (default: release)
  --out-dir <dir>         Build output directory (default: .tmp/builtin-wasm-sync-modules)
  --module-ids-path <p>   Module id manifest path (default: crates/.../m1_builtin_module_ids.txt)
  --module-manifest-map-path <p>
                          Module id -> Cargo.toml map path
                          (default: crates/.../builtin_module_manifest_map.txt)
  --hash-path <p>         Hash manifest path tracked by git (default: crates/.../m1_builtin_modules.sha256)
  --identity-path <p>     Identity manifest path tracked by git
                          (default: <hash-path with .sha256 replaced by .identity.json>)
  --distfs-root <p>       DistFS builtin wasm root (default: .distfs/builtin_wasm)
  --artifact-dir <p>      Deprecated alias of DistFS blobs dir (default: <distfs-root>/blobs)
  -h, --help              Show this help

Env:
  AGENT_WORLD_WASM_CANONICAL_PLATFORMS
      Comma-separated canonical platforms in <os>-<arch> format.
      Default: darwin-arm64,linux-x86_64
USAGE
}

sha256_file() {
  local path="$1"
  if command -v shasum >/dev/null 2>&1; then
    LC_ALL=C LANG=C shasum -a 256 "$path" | awk '{print $1}'
    return 0
  fi
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$path" | awk '{print $1}'
    return 0
  fi
  if command -v openssl >/dev/null 2>&1; then
    openssl dgst -sha256 "$path" | awk '{print $NF}'
    return 0
  fi
  echo "error: no SHA-256 command found (need shasum/sha256sum/openssl)" >&2
  return 127
}

is_sha256_hex() {
  local value="$1"
  [[ "$value" =~ ^[0-9a-f]{64}$ ]]
}

normalize_platform_os() {
  local raw="$1"
  case "$raw" in
    Darwin) echo "darwin" ;;
    Linux) echo "linux" ;;
    *)
      echo "$raw" | tr '[:upper:]' '[:lower:]'
      ;;
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

detect_current_platform() {
  local os arch
  os="$(normalize_platform_os "$(uname -s)")"
  arch="$(normalize_platform_arch "$(uname -m)")"
  echo "${os}-${arch}"
}

read_canonical_platforms() {
  local raw_entries=()
  local entry
  IFS=',' read -r -a raw_entries <<< "$CANONICAL_PLATFORMS_CSV"
  for entry in "${raw_entries[@]}"; do
    entry="$(echo "$entry" | tr -d '[:space:]')"
    [[ -z "$entry" ]] && continue
    if ! array_contains "$entry" "${CANONICAL_PLATFORMS[@]-}"; then
      CANONICAL_PLATFORMS+=("$entry")
    fi
  done

  if [[ "${#CANONICAL_PLATFORMS[@]}" -eq 0 ]]; then
    echo "error: AGENT_WORLD_WASM_CANONICAL_PLATFORMS has no valid entries" >&2
    exit 2
  fi

  if array_contains "$CURRENT_PLATFORM" "${CANONICAL_PLATFORMS[@]-}"; then
    CURRENT_PLATFORM_IS_CANONICAL=1
  fi
}

require_current_platform_supported() {
  if [[ "$CURRENT_PLATFORM_IS_CANONICAL" -ne 1 ]]; then
    echo "error: current platform is not in canonical platform set" >&2
    echo "  current_platform=$CURRENT_PLATFORM" >&2
    echo "  canonical_platforms=${CANONICAL_PLATFORMS[*]}" >&2
    echo "hint: set AGENT_WORLD_WASM_CANONICAL_PLATFORMS to include current platform" >&2
    exit 1
  fi
}

read_module_ids() {
  if [[ ! -f "$MODULE_IDS_PATH" ]]; then
    echo "error: module id manifest missing: $MODULE_IDS_PATH" >&2
    exit 1
  fi

  while IFS= read -r module_id; do
    [[ -z "$module_id" ]] && continue
    MODULE_IDS+=("$module_id")
  done < "$MODULE_IDS_PATH"

  if [[ ${#MODULE_IDS[@]} -eq 0 ]]; then
    echo "error: module id manifest has no module ids: $MODULE_IDS_PATH" >&2
    exit 1
  fi
}

manifest_tokens_for() {
  local module_id="$1"
  awk -v m="$module_id" '
    $1 == m {
      for (i = 2; i <= NF; i++) {
        print $i
      }
      exit
    }
  ' "$HASH_MANIFEST_PATH"
}

manifest_entry_count() {
  awk 'NF > 0 { count += 1 } END { print count + 0 }' "$HASH_MANIFEST_PATH"
}

array_contains() {
  local needle="$1"
  shift
  local candidate
  for candidate in "$@"; do
    if [[ "$candidate" == "$needle" ]]; then
      return 0
    fi
  done
  return 1
}

token_platform_key() {
  local token="$1"
  if [[ "$token" == *=* ]]; then
    echo "${token%%=*}"
  fi
}

token_hash_value() {
  local token="$1"
  if [[ "$token" == *=* ]]; then
    echo "${token#*=}"
  else
    echo "$token"
  fi
}

hydrate_distfs_blobs() {
  env -u RUSTC_WRAPPER cargo run --quiet -p agent_world_distfs --bin hydrate_builtin_wasm -- \
    --root "$DISTFS_ROOT" \
    --manifest "$HASH_MANIFEST_PATH" \
    --built-dir "$OUT_DIR"
}

sync_identity_manifest() {
  local mode="$1"
  local canonical_platforms_joined
  canonical_platforms_joined="$(IFS=, ; echo "${CANONICAL_PLATFORMS[*]}")"

  cmd=(
    env -u RUSTC_WRAPPER cargo run --quiet -p agent_world_distfs --bin sync_builtin_wasm_identity --
    --module-ids-path "$MODULE_IDS_PATH"
    --module-manifest-map-path "$MODULE_MANIFEST_MAP_PATH"
    --hash-manifest-path "$HASH_MANIFEST_PATH"
    --identity-manifest-path "$IDENTITY_MANIFEST_PATH"
    --metadata-dir "$OUT_DIR"
    --workspace-root "$ROOT_DIR"
    --profile "$PROFILE"
    --canonical-platforms "$canonical_platforms_joined"
  )
  if [[ "$mode" == "check" ]]; then
    cmd+=(--check)
  fi

  "${cmd[@]}"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --check)
      CHECK_ONLY=1
      shift
      ;;
    --profile)
      [[ $# -ge 2 ]] || { echo "error: --profile requires a value" >&2; exit 2; }
      PROFILE="$2"
      shift 2
      ;;
    --out-dir)
      [[ $# -ge 2 ]] || { echo "error: --out-dir requires a value" >&2; exit 2; }
      OUT_DIR="$2"
      shift 2
      ;;
    --module-ids-path)
      [[ $# -ge 2 ]] || { echo "error: --module-ids-path requires a value" >&2; exit 2; }
      MODULE_IDS_PATH="$2"
      shift 2
      ;;
    --module-manifest-map-path)
      [[ $# -ge 2 ]] || { echo "error: --module-manifest-map-path requires a value" >&2; exit 2; }
      MODULE_MANIFEST_MAP_PATH="$2"
      shift 2
      ;;
    --hash-path)
      [[ $# -ge 2 ]] || { echo "error: --hash-path requires a value" >&2; exit 2; }
      HASH_MANIFEST_PATH="$2"
      shift 2
      ;;
    --identity-path)
      [[ $# -ge 2 ]] || { echo "error: --identity-path requires a value" >&2; exit 2; }
      IDENTITY_MANIFEST_PATH="$2"
      shift 2
      ;;
    --distfs-root)
      [[ $# -ge 2 ]] || { echo "error: --distfs-root requires a value" >&2; exit 2; }
      DISTFS_ROOT="$2"
      DISTFS_BLOBS_DIR="$DISTFS_ROOT/blobs"
      shift 2
      ;;
    --artifact-dir)
      [[ $# -ge 2 ]] || { echo "error: --artifact-dir requires a value" >&2; exit 2; }
      DISTFS_BLOBS_DIR="$2"
      DISTFS_ROOT="$(dirname "$DISTFS_BLOBS_DIR")"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown option: $1" >&2
      usage
      exit 2
      ;;
  esac
done

if [[ -z "$IDENTITY_MANIFEST_PATH" ]]; then
  if [[ "$HASH_MANIFEST_PATH" == *.sha256 ]]; then
    IDENTITY_MANIFEST_PATH="${HASH_MANIFEST_PATH%.sha256}.identity.json"
  else
    IDENTITY_MANIFEST_PATH="${HASH_MANIFEST_PATH}.identity.json"
  fi
fi

CURRENT_PLATFORM="$(detect_current_platform)"
read_canonical_platforms

read_module_ids
mkdir -p "$OUT_DIR"
mkdir -p "$DISTFS_BLOBS_DIR"

"$ROOT_DIR/scripts/build-builtin-wasm-modules.sh" \
  --module-ids-path "$MODULE_IDS_PATH" \
  --out-dir "$OUT_DIR" \
  --profile "$PROFILE"

if [[ ! -f "$HASH_MANIFEST_PATH" ]]; then
  if [[ "$CHECK_ONLY" -eq 1 ]]; then
    echo "error: hash manifest missing: $HASH_MANIFEST_PATH" >&2
    echo "hint: run scripts/sync-m1-builtin-wasm-artifacts.sh" >&2
    exit 1
  fi
fi

if [[ "$CHECK_ONLY" -eq 1 ]]; then
  manifest_line_count="$(manifest_entry_count)"
  if [[ "$manifest_line_count" -ne "${#MODULE_IDS[@]}" ]]; then
    echo "error: hash manifest line count mismatch" >&2
    echo "  manifest_lines=$manifest_line_count" >&2
    echo "  module_count=${#MODULE_IDS[@]}" >&2
    exit 1
  fi

  for module_id in "${MODULE_IDS[@]}"; do
    built_path="$OUT_DIR/$module_id.wasm"
    if [[ ! -f "$built_path" ]]; then
      echo "error: built wasm not found: $built_path" >&2
      exit 1
    fi

    built_hash="$(sha256_file "$built_path")"
    manifest_tokens=()
    while IFS= read -r manifest_token; do
      [[ -z "$manifest_token" ]] && continue
      manifest_tokens+=("$manifest_token")
    done < <(manifest_tokens_for "$module_id")

    if [[ "${#manifest_tokens[@]}" -eq 0 ]]; then
      echo "error: module hash missing in manifest: $module_id" >&2
      exit 1
    fi

    keyed_tokens=0
    legacy_tokens=0
    platform_keys=()
    platform_hashes=()
    legacy_hashes=()

    for token in "${manifest_tokens[@]}"; do
      hash_value="$(token_hash_value "$token")"
      if ! is_sha256_hex "$hash_value"; then
        echo "error: manifest hash is not valid sha256 module_id=$module_id token=$token" >&2
        exit 1
      fi

      platform_key="$(token_platform_key "$token")"
      if [[ -n "$platform_key" ]]; then
        keyed_tokens=$((keyed_tokens + 1))
        if ! array_contains "$platform_key" "${CANONICAL_PLATFORMS[@]-}"; then
          echo "error: manifest platform is not allowed module_id=$module_id platform=$platform_key" >&2
          echo "  allowed_platforms=${CANONICAL_PLATFORMS[*]}" >&2
          exit 1
        fi
        if array_contains "$platform_key" "${platform_keys[@]-}"; then
          echo "error: duplicate manifest platform entry module_id=$module_id platform=$platform_key" >&2
          exit 1
        fi
        platform_keys+=("$platform_key")
        platform_hashes+=("$hash_value")
      else
        legacy_tokens=$((legacy_tokens + 1))
        legacy_hashes+=("$hash_value")
      fi
    done

    if [[ "$keyed_tokens" -gt 0 && "$legacy_tokens" -gt 0 ]]; then
      echo "error: mixed keyed/legacy hash tokens are not allowed module_id=$module_id" >&2
      exit 1
    fi

    if [[ "$keyed_tokens" -gt 0 ]]; then
      require_current_platform_supported

      expected_platform_hash=""
      for idx in "${!platform_keys[@]}"; do
        if [[ "${platform_keys[$idx]}" == "$CURRENT_PLATFORM" ]]; then
          expected_platform_hash="${platform_hashes[$idx]}"
          break
        fi
      done

      if [[ -z "$expected_platform_hash" ]]; then
        echo "error: manifest missing canonical hash for current platform" >&2
        echo "  module_id=$module_id" >&2
        echo "  current_platform=$CURRENT_PLATFORM" >&2
        echo "  manifest_tokens=${manifest_tokens[*]}" >&2
        exit 1
      fi

      if [[ "$built_hash" != "$expected_platform_hash" ]]; then
        echo "error: canonical hash mismatch for current platform module_id=$module_id" >&2
        echo "  platform=$CURRENT_PLATFORM" >&2
        echo "  built   =$built_hash" >&2
        echo "  manifest=$expected_platform_hash" >&2
        echo "hint: run scripts/sync-m1-builtin-wasm-artifacts.sh" >&2
        exit 1
      fi
    else
      if ! array_contains "$built_hash" "${legacy_hashes[@]-}"; then
        echo "error: hash manifest is stale vs built wasm for module: $module_id" >&2
        echo "  built   =$built_hash" >&2
        echo "  manifest=${legacy_hashes[*]}" >&2
        echo "hint: run scripts/sync-m1-builtin-wasm-artifacts.sh" >&2
        exit 1
      fi
    fi
  done

  sync_identity_manifest check
  hydrate_distfs_blobs

  echo "check ok: hash manifest is in sync with built wasm"
  echo "  module_count=${#MODULE_IDS[@]}"
  echo "  current_platform=$CURRENT_PLATFORM"
  echo "  canonical_platforms=${CANONICAL_PLATFORMS[*]}"
  echo "  hash_manifest=$HASH_MANIFEST_PATH"
  echo "  identity_manifest=$IDENTITY_MANIFEST_PATH"
  echo "  distfs_blobs_dir=$DISTFS_BLOBS_DIR"
  exit 0
fi

tmp_manifest="$(mktemp)"
trap 'rm -f "$tmp_manifest"' EXIT

for module_id in "${MODULE_IDS[@]}"; do
  built_path="$OUT_DIR/$module_id.wasm"
  if [[ ! -f "$built_path" ]]; then
    echo "error: built wasm not found: $built_path" >&2
    exit 1
  fi

  built_hash="$(sha256_file "$built_path")"
  manifest_tokens=()
  if [[ -f "$HASH_MANIFEST_PATH" ]]; then
    while IFS= read -r token; do
      [[ -z "$token" ]] && continue
      manifest_tokens+=("$token")
    done < <(manifest_tokens_for "$module_id")
  fi

  keyed_tokens=0
  legacy_tokens=0
  platform_keys=()
  platform_hashes=()
  for token in "${manifest_tokens[@]}"; do
    hash_value="$(token_hash_value "$token")"
    if ! is_sha256_hex "$hash_value"; then
      echo "error: manifest hash is not valid sha256 module_id=$module_id token=$token" >&2
      exit 1
    fi

    platform_key="$(token_platform_key "$token")"
    if [[ -n "$platform_key" ]]; then
      keyed_tokens=$((keyed_tokens + 1))
      if ! array_contains "$platform_key" "${CANONICAL_PLATFORMS[@]-}"; then
        echo "error: manifest platform is not allowed module_id=$module_id platform=$platform_key" >&2
        echo "  allowed_platforms=${CANONICAL_PLATFORMS[*]}" >&2
        exit 1
      fi
      if array_contains "$platform_key" "${platform_keys[@]-}"; then
        echo "error: duplicate manifest platform entry module_id=$module_id platform=$platform_key" >&2
        exit 1
      fi
      platform_keys+=("$platform_key")
      platform_hashes+=("$hash_value")
    else
      legacy_tokens=$((legacy_tokens + 1))
    fi
  done

  if [[ "$keyed_tokens" -gt 0 && "$legacy_tokens" -gt 0 ]]; then
    echo "error: mixed keyed/legacy hash tokens are not allowed module_id=$module_id" >&2
    exit 1
  fi

  if [[ "$keyed_tokens" -gt 0 ]]; then
    require_current_platform_supported

    updated=0
    for idx in "${!platform_keys[@]}"; do
      if [[ "${platform_keys[$idx]}" == "$CURRENT_PLATFORM" ]]; then
        platform_hashes[$idx]="$built_hash"
        updated=1
        break
      fi
    done
    if [[ "$updated" -ne 1 ]]; then
      platform_keys+=("$CURRENT_PLATFORM")
      platform_hashes+=("$built_hash")
    fi

    printf "%s" "$module_id" >> "$tmp_manifest"
    emitted=0
    for platform in "${CANONICAL_PLATFORMS[@]}"; do
      for idx in "${!platform_keys[@]}"; do
        if [[ "${platform_keys[$idx]}" == "$platform" ]]; then
          printf " %s=%s" "$platform" "${platform_hashes[$idx]}" >> "$tmp_manifest"
          emitted=1
          break
        fi
      done
    done
    if [[ "$emitted" -ne 1 ]]; then
      echo "error: no canonical platform entries emitted for module_id=$module_id" >&2
      exit 1
    fi
    printf "\n" >> "$tmp_manifest"
  else
    printf "%s %s\n" "$module_id" "$built_hash" >> "$tmp_manifest"
  fi
done

mkdir -p "$(dirname "$HASH_MANIFEST_PATH")"
mv "$tmp_manifest" "$HASH_MANIFEST_PATH"
trap - EXIT

sync_identity_manifest sync
hydrate_distfs_blobs

echo "synced builtin wasm hash/identity manifest + DistFS blobs"
echo "  module_count=${#MODULE_IDS[@]}"
echo "  current_platform=$CURRENT_PLATFORM"
echo "  canonical_platforms=${CANONICAL_PLATFORMS[*]}"
echo "  hash_manifest=$HASH_MANIFEST_PATH"
echo "  identity_manifest=$IDENTITY_MANIFEST_PATH"
echo "  distfs_blobs_dir=$DISTFS_BLOBS_DIR"
