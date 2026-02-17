#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

RUSTFLAGS_EFFECTIVE="${RUSTFLAGS:-}"

append_rustflag_once() {
  local flag="$1"
  if [[ " $RUSTFLAGS_EFFECTIVE " == *" $flag "* ]]; then
    return 0
  fi
  if [[ -n "$RUSTFLAGS_EFFECTIVE" ]]; then
    RUSTFLAGS_EFFECTIVE="$RUSTFLAGS_EFFECTIVE $flag"
  else
    RUSTFLAGS_EFFECTIVE="$flag"
  fi
}

# Normalize host-specific absolute paths so wasm bytes stay stable across machines.
if [[ -n "${HOME:-}" ]]; then
  append_rustflag_once "--remap-path-prefix=$HOME/.cargo=/cargo"
  append_rustflag_once "--remap-path-prefix=$HOME/.rustup=/rustup"
fi
append_rustflag_once "--remap-path-prefix=$ROOT_DIR=/workspace"
export RUSTFLAGS="$RUSTFLAGS_EFFECTIVE"

env -u RUSTC_WRAPPER cargo run \
  --quiet \
  --manifest-path "$ROOT_DIR/tools/wasm_build_suite/Cargo.toml" \
  -- \
  build "$@"
