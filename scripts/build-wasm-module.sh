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

# Rustup may expose multiple toolchain directory aliases (for example stable-* and
# pinned version names). Remap all discovered toolchain roots to one stable prefix
# so host-specific alias choice cannot leak into wasm bytes.
RUSTUP_HOME_DIR="${RUSTUP_HOME:-}"
if [[ -z "$RUSTUP_HOME_DIR" && -n "${HOME:-}" ]]; then
  RUSTUP_HOME_DIR="$HOME/.rustup"
fi
if [[ -n "$RUSTUP_HOME_DIR" && -d "$RUSTUP_HOME_DIR/toolchains" ]]; then
  for toolchain_dir in "$RUSTUP_HOME_DIR"/toolchains/*; do
    [[ -d "$toolchain_dir" ]] || continue
    append_rustflag_once "--remap-path-prefix=$toolchain_dir=/rustup/toolchain"
  done
fi

# Rust std source paths include host triple under sysroot (for example aarch64-apple-darwin
# or x86_64-unknown-linux-gnu). Remap the concrete sysroot path to avoid cross-host hash drift.
RUSTC_SYSROOT="$(rustc --print sysroot 2>/dev/null || true)"
if [[ -n "$RUSTC_SYSROOT" ]]; then
  append_rustflag_once "--remap-path-prefix=$RUSTC_SYSROOT=/rustup/toolchain"
fi
export RUSTFLAGS="$RUSTFLAGS_EFFECTIVE"

env -u RUSTC_WRAPPER cargo run \
  --quiet \
  --manifest-path "$ROOT_DIR/tools/wasm_build_suite/Cargo.toml" \
  -- \
  build "$@"
