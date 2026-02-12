#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="$ROOT_DIR/.tmp/builtin-wasm"
PROFILE="release"
DRY_RUN=0
LIST_ONLY=0

declare -a REQUESTED_MODULE_IDS=()

usage() {
  cat <<'USAGE'
Usage:
  scripts/build-builtin-wasm-modules.sh [options]

Options:
  --module-id <id>   Build only one module id (repeatable). Default: build all known ids.
  --out-dir <dir>    Output directory. Default: .tmp/builtin-wasm
  --profile <name>   Cargo profile for wasm build suite. Default: release
  --dry-run          Resolve paths only, do not execute cargo build
  --list             Print known builtin module ids and exit
  -h, --help         Show this help message
USAGE
}

module_manifest_path() {
  local module_id="$1"
  case "$module_id" in
    m1.rule.move)
      echo "$ROOT_DIR/crates/agent_world_builtin_wasm/Cargo.toml"
      ;;
    *)
      return 1
      ;;
  esac
}

all_module_ids() {
  cat <<'EOF_IDS'
m1.rule.move
EOF_IDS
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --module-id)
      [[ $# -ge 2 ]] || { echo "error: --module-id requires a value" >&2; exit 2; }
      REQUESTED_MODULE_IDS+=("$2")
      shift 2
      ;;
    --out-dir)
      [[ $# -ge 2 ]] || { echo "error: --out-dir requires a value" >&2; exit 2; }
      OUT_DIR="$2"
      shift 2
      ;;
    --profile)
      [[ $# -ge 2 ]] || { echo "error: --profile requires a value" >&2; exit 2; }
      PROFILE="$2"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --list)
      LIST_ONLY=1
      shift
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

if [[ "$LIST_ONLY" -eq 1 ]]; then
  all_module_ids
  exit 0
fi

if [[ ${#REQUESTED_MODULE_IDS[@]} -eq 0 ]]; then
  while IFS= read -r module_id; do
    [[ -z "$module_id" ]] && continue
    REQUESTED_MODULE_IDS+=("$module_id")
  done < <(all_module_ids)
fi

mkdir -p "$OUT_DIR"

for module_id in "${REQUESTED_MODULE_IDS[@]}"; do
  manifest_path="$(module_manifest_path "$module_id")" || {
    echo "error: unsupported module id: $module_id" >&2
    exit 2
  }
  cmd=(
    "$ROOT_DIR/scripts/build-wasm-module.sh"
    --module-id "$module_id"
    --manifest-path "$manifest_path"
    --out-dir "$OUT_DIR"
    --profile "$PROFILE"
  )
  if [[ "$DRY_RUN" -eq 1 ]]; then
    cmd+=(--dry-run)
  fi
  "${cmd[@]}"
done
