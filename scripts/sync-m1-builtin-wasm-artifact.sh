#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODULE_ID="m1.rule.move"
PROFILE="release"
OUT_DIR="$ROOT_DIR/.tmp/builtin-wasm-sync"
ARTIFACT_PATH="$ROOT_DIR/crates/agent_world/src/runtime/world/artifacts/m1_builtin_modules.wasm"
HASH_PATH="${ARTIFACT_PATH}.sha256"
CHECK_ONLY=0

usage() {
  cat <<'USAGE'
Usage:
  scripts/sync-m1-builtin-wasm-artifact.sh [options]

Options:
  --check              Build and verify embedded artifact/hash only, do not overwrite files
  --module-id <id>     Module id used to trigger wasm build (default: m1.rule.move)
  --profile <name>     Cargo profile forwarded to wasm build suite (default: release)
  --out-dir <dir>      Build output directory (default: .tmp/builtin-wasm-sync)
  --artifact-path <p>  Embedded artifact path (default: crates/.../m1_builtin_modules.wasm)
  --hash-path <p>      Artifact hash file path (default: <artifact-path>.sha256)
  -h, --help           Show this help
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

while [[ $# -gt 0 ]]; do
  case "$1" in
    --check)
      CHECK_ONLY=1
      shift
      ;;
    --module-id)
      [[ $# -ge 2 ]] || { echo "error: --module-id requires a value" >&2; exit 2; }
      MODULE_ID="$2"
      shift 2
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
    --artifact-path)
      [[ $# -ge 2 ]] || { echo "error: --artifact-path requires a value" >&2; exit 2; }
      ARTIFACT_PATH="$2"
      shift 2
      ;;
    --hash-path)
      [[ $# -ge 2 ]] || { echo "error: --hash-path requires a value" >&2; exit 2; }
      HASH_PATH="$2"
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

mkdir -p "$OUT_DIR"
"$ROOT_DIR/scripts/build-builtin-wasm-modules.sh" \
  --module-id "$MODULE_ID" \
  --out-dir "$OUT_DIR" \
  --profile "$PROFILE"

BUILT_WASM_PATH="$OUT_DIR/$MODULE_ID.wasm"
if [[ ! -f "$BUILT_WASM_PATH" ]]; then
  echo "error: built wasm not found: $BUILT_WASM_PATH" >&2
  exit 1
fi

BUILT_HASH="$(sha256_file "$BUILT_WASM_PATH")"
BUILT_SIZE="$(wc -c < "$BUILT_WASM_PATH" | tr -d '[:space:]')"

if [[ "$CHECK_ONLY" -eq 1 ]]; then
  if [[ ! -f "$ARTIFACT_PATH" ]]; then
    echo "error: embedded wasm artifact missing: $ARTIFACT_PATH" >&2
    exit 1
  fi
  if [[ ! -f "$HASH_PATH" ]]; then
    echo "error: embedded wasm hash file missing: $HASH_PATH" >&2
    exit 1
  fi

  EMBED_HASH="$(sha256_file "$ARTIFACT_PATH")"
  EXPECTED_HASH="$(tr -d '[:space:]' < "$HASH_PATH")"

  if [[ "$EXPECTED_HASH" != "$EMBED_HASH" ]]; then
    echo "error: hash file does not match embedded artifact" >&2
    echo "  hash_file=$HASH_PATH" >&2
    echo "  expected =$EXPECTED_HASH" >&2
    echo "  actual   =$EMBED_HASH" >&2
    exit 1
  fi

  if [[ "$BUILT_HASH" != "$EMBED_HASH" ]]; then
    echo "error: embedded artifact is stale vs built wasm" >&2
    echo "  built    =$BUILT_HASH" >&2
    echo "  embedded =$EMBED_HASH" >&2
    echo "hint: run scripts/sync-m1-builtin-wasm-artifact.sh" >&2
    exit 1
  fi

  echo "check ok: embedded artifact and hash are in sync"
  echo "  module_id=$MODULE_ID"
  echo "  sha256=$EMBED_HASH"
  echo "  size_bytes=$BUILT_SIZE"
  exit 0
fi

mkdir -p "$(dirname "$ARTIFACT_PATH")"
cp "$BUILT_WASM_PATH" "$ARTIFACT_PATH"
printf "%s\n" "$BUILT_HASH" > "$HASH_PATH"

echo "synced embedded wasm artifact"
echo "  module_id=$MODULE_ID"
echo "  artifact_path=$ARTIFACT_PATH"
echo "  hash_path=$HASH_PATH"
echo "  sha256=$BUILT_HASH"
echo "  size_bytes=$BUILT_SIZE"
