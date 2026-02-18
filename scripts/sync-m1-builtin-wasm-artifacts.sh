#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$ROOT_DIR/.tmp/builtin-wasm-sync-modules"
PROFILE="release"
CHECK_ONLY=0
MODULE_IDS_PATH="$ROOT_DIR/crates/agent_world/src/runtime/world/artifacts/m1_builtin_module_ids.txt"
HASH_MANIFEST_PATH="$ROOT_DIR/crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.sha256"
DISTFS_ROOT="$ROOT_DIR/.distfs/builtin_wasm"
DISTFS_BLOBS_DIR="$DISTFS_ROOT/blobs"

declare -a MODULE_IDS=()

usage() {
  cat <<'USAGE'
Usage:
  scripts/sync-m1-builtin-wasm-artifacts.sh [options]

Options:
  --check                 Build and verify hash manifest vs built wasm, then hydrate DistFS blobs
  --profile <name>        Cargo profile forwarded to wasm build suite (default: release)
  --out-dir <dir>         Build output directory (default: .tmp/builtin-wasm-sync-modules)
  --module-ids-path <p>   Module id manifest path (default: crates/.../m1_builtin_module_ids.txt)
  --hash-path <p>         Hash manifest path tracked by git (default: crates/.../m1_builtin_modules.sha256)
  --distfs-root <p>       DistFS builtin wasm root (default: .distfs/builtin_wasm)
  --artifact-dir <p>      Deprecated alias of DistFS blobs dir (default: <distfs-root>/blobs)
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

manifest_hashes_for() {
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

hydrate_distfs_blobs() {
  env -u RUSTC_WRAPPER cargo run --quiet -p agent_world_distfs --bin hydrate_builtin_wasm -- \
    --root "$DISTFS_ROOT" \
    --manifest "$HASH_MANIFEST_PATH" \
    --built-dir "$OUT_DIR"
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
    --hash-path)
      [[ $# -ge 2 ]] || { echo "error: --hash-path requires a value" >&2; exit 2; }
      HASH_MANIFEST_PATH="$2"
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
    manifest_hashes=()
    while IFS= read -r manifest_hash; do
      [[ -z "$manifest_hash" ]] && continue
      manifest_hashes+=("$manifest_hash")
    done < <(manifest_hashes_for "$module_id")

    if [[ "${#manifest_hashes[@]}" -eq 0 ]]; then
      echo "error: module hash missing in manifest: $module_id" >&2
      exit 1
    fi

    if ! array_contains "$built_hash" "${manifest_hashes[@]}"; then
      echo "error: hash manifest is stale vs built wasm for module: $module_id" >&2
      echo "  built   =$built_hash" >&2
      echo "  manifest=${manifest_hashes[*]}" >&2
      echo "hint: run scripts/sync-m1-builtin-wasm-artifacts.sh" >&2
      exit 1
    fi
  done

  hydrate_distfs_blobs

  echo "check ok: hash manifest is in sync with built wasm"
  echo "  module_count=${#MODULE_IDS[@]}"
  echo "  hash_manifest=$HASH_MANIFEST_PATH"
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
  merged_hashes=()
  if [[ -f "$HASH_MANIFEST_PATH" ]]; then
    while IFS= read -r existing_hash; do
      [[ -z "$existing_hash" ]] && continue
      if ! array_contains "$existing_hash" "${merged_hashes[@]}"; then
        merged_hashes+=("$existing_hash")
      fi
    done < <(manifest_hashes_for "$module_id")
  fi
  if ! array_contains "$built_hash" "${merged_hashes[@]}"; then
    merged_hashes+=("$built_hash")
  fi

  printf "%s" "$module_id" >> "$tmp_manifest"
  for module_hash in "${merged_hashes[@]}"; do
    printf " %s" "$module_hash" >> "$tmp_manifest"
  done
  printf "\n" >> "$tmp_manifest"
done

mkdir -p "$(dirname "$HASH_MANIFEST_PATH")"
mv "$tmp_manifest" "$HASH_MANIFEST_PATH"
trap - EXIT

hydrate_distfs_blobs

echo "synced builtin wasm hash manifest + DistFS blobs"
echo "  module_count=${#MODULE_IDS[@]}"
echo "  hash_manifest=$HASH_MANIFEST_PATH"
echo "  distfs_blobs_dir=$DISTFS_BLOBS_DIR"
