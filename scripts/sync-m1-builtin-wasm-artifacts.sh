#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$ROOT_DIR/.tmp/builtin-wasm-sync-modules"
PROFILE="release"
CHECK_ONLY=0
MODULE_IDS_PATH="$ROOT_DIR/crates/agent_world/src/runtime/world/artifacts/m1_builtin_module_ids.txt"
ARTIFACT_DIR="$ROOT_DIR/crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules"
HASH_MANIFEST_PATH="$ROOT_DIR/crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256"

declare -a MODULE_IDS=()

usage() {
  cat <<'USAGE'
Usage:
  scripts/sync-m1-builtin-wasm-artifacts.sh [options]

Options:
  --check                 Build and verify embedded per-module artifacts/hash manifest only
  --profile <name>        Cargo profile forwarded to wasm build suite (default: release)
  --out-dir <dir>         Build output directory (default: .tmp/builtin-wasm-sync-modules)
  --module-ids-path <p>   Module id manifest path (default: crates/.../m1_builtin_module_ids.txt)
  --artifact-dir <p>      Embedded artifact directory (default: crates/.../m1_builtin_modules)
  --hash-path <p>         Embedded hash manifest path (default: crates/.../m1_builtin_modules.sha256)
  -h, --help              Show this help
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

manifest_hash_for() {
  local module_id="$1"
  awk -v m="$module_id" '$1 == m { print $2; exit }' "$HASH_MANIFEST_PATH"
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
    --artifact-dir)
      [[ $# -ge 2 ]] || { echo "error: --artifact-dir requires a value" >&2; exit 2; }
      ARTIFACT_DIR="$2"
      shift 2
      ;;
    --hash-path)
      [[ $# -ge 2 ]] || { echo "error: --hash-path requires a value" >&2; exit 2; }
      HASH_MANIFEST_PATH="$2"
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

read_module_ids
mkdir -p "$OUT_DIR"

"$ROOT_DIR/scripts/build-builtin-wasm-modules.sh" \
  --out-dir "$OUT_DIR" \
  --profile "$PROFILE"

if [[ "$CHECK_ONLY" -eq 1 ]]; then
  if [[ ! -d "$ARTIFACT_DIR" ]]; then
    echo "error: artifact directory missing: $ARTIFACT_DIR" >&2
    exit 1
  fi
  if [[ ! -f "$HASH_MANIFEST_PATH" ]]; then
    echo "error: hash manifest missing: $HASH_MANIFEST_PATH" >&2
    exit 1
  fi

  manifest_line_count="$(wc -l < "$HASH_MANIFEST_PATH" | tr -d '[:space:]')"
  if [[ "$manifest_line_count" -ne "${#MODULE_IDS[@]}" ]]; then
    echo "error: hash manifest line count mismatch" >&2
    echo "  manifest_lines=$manifest_line_count" >&2
    echo "  module_count=${#MODULE_IDS[@]}" >&2
    exit 1
  fi

  for module_id in "${MODULE_IDS[@]}"; do
    built_path="$OUT_DIR/$module_id.wasm"
    embedded_path="$ARTIFACT_DIR/$module_id.wasm"

    if [[ ! -f "$built_path" ]]; then
      echo "error: built wasm not found: $built_path" >&2
      exit 1
    fi
    if [[ ! -f "$embedded_path" ]]; then
      echo "error: embedded wasm not found: $embedded_path" >&2
      exit 1
    fi

    built_hash="$(sha256_file "$built_path")"
    embedded_hash="$(sha256_file "$embedded_path")"
    manifest_hash="$(manifest_hash_for "$module_id")"
    if [[ -z "$manifest_hash" ]]; then
      echo "error: module hash missing in manifest: $module_id" >&2
      exit 1
    fi

    if [[ "$manifest_hash" != "$embedded_hash" ]]; then
      echo "error: hash manifest mismatch for module: $module_id" >&2
      echo "  manifest=$manifest_hash" >&2
      echo "  embedded=$embedded_hash" >&2
      exit 1
    fi
    if [[ "$built_hash" != "$embedded_hash" ]]; then
      echo "error: embedded artifact is stale vs built wasm for module: $module_id" >&2
      echo "  built   =$built_hash" >&2
      echo "  embedded=$embedded_hash" >&2
      echo "hint: run scripts/sync-m1-builtin-wasm-artifacts.sh" >&2
      exit 1
    fi
  done

  echo "check ok: per-module embedded artifacts and hash manifest are in sync"
  echo "  module_count=${#MODULE_IDS[@]}"
  echo "  artifact_dir=$ARTIFACT_DIR"
  echo "  hash_manifest=$HASH_MANIFEST_PATH"
  exit 0
fi

mkdir -p "$ARTIFACT_DIR"
tmp_manifest="$(mktemp)"
trap 'rm -f "$tmp_manifest"' EXIT

for module_id in "${MODULE_IDS[@]}"; do
  built_path="$OUT_DIR/$module_id.wasm"
  if [[ ! -f "$built_path" ]]; then
    echo "error: built wasm not found: $built_path" >&2
    exit 1
  fi

  embedded_path="$ARTIFACT_DIR/$module_id.wasm"
  cp "$built_path" "$embedded_path"
  module_hash="$(sha256_file "$built_path")"
  printf "%s %s\n" "$module_id" "$module_hash" >> "$tmp_manifest"
done

mkdir -p "$(dirname "$HASH_MANIFEST_PATH")"
mv "$tmp_manifest" "$HASH_MANIFEST_PATH"
trap - EXIT

echo "synced per-module embedded wasm artifacts"
echo "  module_count=${#MODULE_IDS[@]}"
echo "  artifact_dir=$ARTIFACT_DIR"
echo "  hash_manifest=$HASH_MANIFEST_PATH"
