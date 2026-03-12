#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

BUNDLE_DIR="output/release/game-launcher-producer-local"
PROFILE="release"
REBUILD=0
STACK_ARGS=()

usage() {
  cat <<'USAGE'
Usage: ./scripts/run-producer-playtest.sh [options] [run-game-test options...]

Prepare a bundle-first Web stack for producer manual play.

Default behavior:
- reuse `output/release/game-launcher-producer-local` if it already exists
- otherwise build a fresh bundle there
- then start `./scripts/run-game-test.sh --bundle-dir <bundle>`

Options:
  --bundle-dir <path>   Bundle directory to reuse/build (default: output/release/game-launcher-producer-local)
  --profile <name>      Bundle build profile: release|dev (default: release)
  --rebuild             Force rebuild even if bundle already exists
  -h, --help            Show this help

Examples:
  ./scripts/run-producer-playtest.sh --no-llm
  ./scripts/run-producer-playtest.sh --profile dev --no-llm
  ./scripts/run-producer-playtest.sh --bundle-dir output/release/game-launcher-local --no-llm
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bundle-dir)
      BUNDLE_DIR="${2:-}"
      shift 2
      ;;
    --profile)
      PROFILE="${2:-}"
      shift 2
      ;;
    --rebuild)
      REBUILD=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      STACK_ARGS+=("$1")
      shift
      ;;
  esac
done

[[ -n "$BUNDLE_DIR" ]] || { echo "error: --bundle-dir cannot be empty" >&2; exit 2; }
[[ "$PROFILE" == "release" || "$PROFILE" == "dev" ]] || { echo "error: --profile must be release or dev" >&2; exit 2; }

if [[ "$BUNDLE_DIR" != /* ]]; then
  ABS_BUNDLE_DIR="$ROOT_DIR/$BUNDLE_DIR"
else
  ABS_BUNDLE_DIR="$BUNDLE_DIR"
fi

if [[ "$REBUILD" == "1" || ! -x "$ABS_BUNDLE_DIR/run-game.sh" ]]; then
  echo "info: preparing producer playtest bundle at $ABS_BUNDLE_DIR (profile=$PROFILE)"
  ./scripts/build-game-launcher-bundle.sh --profile "$PROFILE" --out-dir "$ABS_BUNDLE_DIR"
else
  echo "info: reusing existing producer playtest bundle at $ABS_BUNDLE_DIR"
fi

exec ./scripts/run-game-test.sh --bundle-dir "$ABS_BUNDLE_DIR" "${STACK_ARGS[@]}"
