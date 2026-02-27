#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR=""
PROFILE="release"
WEB_DIST_SOURCE=""
DRY_RUN=0

usage() {
  cat <<'USAGE'
Usage: ./scripts/build-game-launcher-bundle.sh [options]

Build a distributable launcher bundle:
- bin/agent_world_client_launcher
- bin/world_game_launcher
- bin/world_viewer_live
- web/ (prebuilt viewer static assets)
- run-client.sh (desktop client launcher entry)
- run-game.sh (one-command entry)

Options:
  --out-dir <path>       output directory (default: output/release/game-launcher-<timestamp>)
  --profile <name>       cargo profile: release|dev (default: release)
  --web-dist <path>      use existing prebuilt web dist instead of trunk build
  --dry-run              print commands only; do not execute
  -h, --help             show this help
USAGE
}

run() {
  echo "+ $*"
  if [[ "$DRY_RUN" == "1" ]]; then
    return 0
  fi
  "$@"
}

ensure_command() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "error: required command not found: $cmd" >&2
    exit 1
  fi
}

resolve_binary_name() {
  local base="$1"
  if [[ "$(uname -s)" == MINGW* || "$(uname -s)" == MSYS* || "$(uname -s)" == CYGWIN* ]]; then
    echo "${base}.exe"
  else
    echo "$base"
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --out-dir)
      OUT_DIR="${2:-}"
      shift 2
      ;;
    --profile)
      PROFILE="${2:-}"
      shift 2
      ;;
    --web-dist)
      WEB_DIST_SOURCE="${2:-}"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown option: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ -z "$OUT_DIR" ]]; then
  ts="$(date +%Y%m%d-%H%M%S)"
  OUT_DIR="$ROOT_DIR/output/release/game-launcher-$ts"
fi

if [[ "$PROFILE" != "release" && "$PROFILE" != "dev" ]]; then
  echo "error: --profile must be release or dev" >&2
  exit 1
fi

if [[ -n "$WEB_DIST_SOURCE" && ! -d "$WEB_DIST_SOURCE" ]]; then
  echo "error: --web-dist path does not exist: $WEB_DIST_SOURCE" >&2
  exit 1
fi

LAUNCHER_BIN_NAME="$(resolve_binary_name world_game_launcher)"
LIVE_BIN_NAME="$(resolve_binary_name world_viewer_live)"
CLIENT_LAUNCHER_BIN_NAME="$(resolve_binary_name agent_world_client_launcher)"
TARGET_SUBDIR="$PROFILE"
if [[ "$PROFILE" == "dev" ]]; then
  TARGET_SUBDIR="debug"
fi

BUNDLE_BIN_DIR="$OUT_DIR/bin"
BUNDLE_WEB_DIR="$OUT_DIR/web"

run mkdir -p "$BUNDLE_BIN_DIR" "$BUNDLE_WEB_DIR"

# 1) Build native binaries for launcher/live/client launcher.
if [[ "$PROFILE" == "release" ]]; then
  run env -u RUSTC_WRAPPER cargo build --release -p agent_world --bin world_game_launcher --bin world_viewer_live
  run env -u RUSTC_WRAPPER cargo build --release -p agent_world_client_launcher
else
  run env -u RUSTC_WRAPPER cargo build -p agent_world --bin world_game_launcher --bin world_viewer_live
  run env -u RUSTC_WRAPPER cargo build -p agent_world_client_launcher
fi

LAUNCHER_SRC="$ROOT_DIR/target/$TARGET_SUBDIR/$LAUNCHER_BIN_NAME"
LIVE_SRC="$ROOT_DIR/target/$TARGET_SUBDIR/$LIVE_BIN_NAME"
CLIENT_LAUNCHER_SRC="$ROOT_DIR/target/$TARGET_SUBDIR/$CLIENT_LAUNCHER_BIN_NAME"

if [[ "$DRY_RUN" != "1" ]]; then
  [[ -f "$LAUNCHER_SRC" ]] || { echo "error: launcher binary not found: $LAUNCHER_SRC" >&2; exit 1; }
  [[ -f "$LIVE_SRC" ]] || { echo "error: world_viewer_live binary not found: $LIVE_SRC" >&2; exit 1; }
  [[ -f "$CLIENT_LAUNCHER_SRC" ]] || { echo "error: client launcher binary not found: $CLIENT_LAUNCHER_SRC" >&2; exit 1; }
fi

run cp "$LAUNCHER_SRC" "$BUNDLE_BIN_DIR/$LAUNCHER_BIN_NAME"
run cp "$LIVE_SRC" "$BUNDLE_BIN_DIR/$LIVE_BIN_NAME"
run cp "$CLIENT_LAUNCHER_SRC" "$BUNDLE_BIN_DIR/$CLIENT_LAUNCHER_BIN_NAME"

# 2) Prepare web dist (trunk build by default).
if [[ -n "$WEB_DIST_SOURCE" ]]; then
  run rm -rf "$BUNDLE_WEB_DIR"
  run mkdir -p "$BUNDLE_WEB_DIR"
  run cp -R "$WEB_DIST_SOURCE/." "$BUNDLE_WEB_DIR/"
else
  ensure_command trunk
  if [[ "$DRY_RUN" == "1" ]]; then
    echo "+ rustup target list --installed | rg -x 'wasm32-unknown-unknown'"
  else
    if ! rustup target list --installed | rg -x "wasm32-unknown-unknown" >/dev/null 2>&1; then
      echo "error: rust target wasm32-unknown-unknown is not installed" >&2
      echo "hint: rustup target add wasm32-unknown-unknown" >&2
      exit 1
    fi
  fi

  run rm -rf "$BUNDLE_WEB_DIR"
  run mkdir -p "$BUNDLE_WEB_DIR"
  if [[ "$PROFILE" == "release" ]]; then
    run bash -lc "cd '$ROOT_DIR/crates/agent_world_viewer' && trunk build --release --dist '$BUNDLE_WEB_DIR'"
  else
    run bash -lc "cd '$ROOT_DIR/crates/agent_world_viewer' && trunk build --dist '$BUNDLE_WEB_DIR'"
  fi
fi

# 3) Generate desktop client wrapper + one-command CLI wrapper and readme.
run bash -lc "cat > '$OUT_DIR/run-client.sh' <<'LAUNCH'
#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR=\"\$(cd \"\$(dirname \"\${BASH_SOURCE[0]}\")\" && pwd)\"
AGENT_WORLD_GAME_LAUNCHER_BIN=\"\$ROOT_DIR/bin/$LAUNCHER_BIN_NAME\" \
AGENT_WORLD_GAME_STATIC_DIR=\"\$ROOT_DIR/web\" \
\"\$ROOT_DIR/bin/$CLIENT_LAUNCHER_BIN_NAME\" \"\$@\"
LAUNCH"
run chmod +x "$OUT_DIR/run-client.sh"

run bash -lc "cat > '$OUT_DIR/run-game.sh' <<'LAUNCH'
#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR=\"\$(cd \"\$(dirname \"\${BASH_SOURCE[0]}\")\" && pwd)\"
\"\$ROOT_DIR/bin/$LAUNCHER_BIN_NAME\" --viewer-static-dir \"\$ROOT_DIR/web\" \"\$@\"
LAUNCH"
run chmod +x "$OUT_DIR/run-game.sh"

run bash -lc "cat > '$OUT_DIR/README.txt' <<'README'
Agent World Launcher Bundle

Quick start:
1) Desktop launcher: ./run-client.sh
2) CLI launcher: ./run-game.sh
3) Open URL printed by launcher (CLI path defaults auto-open browser).

Optional:
- Desktop launcher can start/stop game stack from GUI and open URL in one click.
- Enable LLM mode: ./run-game.sh --with-llm
- Disable auto-open browser: ./run-game.sh --no-open-browser

Bundle layout:
- bin/agent_world_client_launcher
- bin/world_game_launcher
- bin/world_viewer_live
- web/
- run-client.sh
- run-game.sh
README"

cat <<INFO
Bundle ready: $OUT_DIR
- client launcher: $BUNDLE_BIN_DIR/$CLIENT_LAUNCHER_BIN_NAME
- launcher:        $BUNDLE_BIN_DIR/$LAUNCHER_BIN_NAME
- live:            $BUNDLE_BIN_DIR/$LIVE_BIN_NAME
- web:             $BUNDLE_WEB_DIR
- entries:         $OUT_DIR/run-client.sh, $OUT_DIR/run-game.sh
INFO
