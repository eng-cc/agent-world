#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

tmp_dir="$repo_root/.tmp/include-warning-baseline"
mkdir -p "$tmp_dir"

include_list="$tmp_dir/runtime-include-modules.txt"
check_log_default="$tmp_dir/agent_world_all_targets.check.log"
check_log_wasmtime="$tmp_dir/agent_world_all_targets_wasmtime.check.log"
warning_log="$tmp_dir/warnings.log"

echo "+ rg -n \"include!\\(\" crates/agent_world/src/runtime | sort > $include_list"
{ rg -n "include!\\(" crates/agent_world/src/runtime || true; } | sort > "$include_list"

echo "+ env -u RUSTC_WRAPPER cargo check -p agent_world --all-targets"
env -u RUSTC_WRAPPER cargo check -p agent_world --all-targets 2>&1 | tee "$check_log_default"

echo "+ env -u RUSTC_WRAPPER cargo check -p agent_world --all-targets --features wasmtime"
env -u RUSTC_WRAPPER cargo check -p agent_world --all-targets --features wasmtime 2>&1 | tee "$check_log_wasmtime"

rm -f "$warning_log"
rg -n "warning:" "$check_log_default" "$check_log_wasmtime" > "$warning_log" || true

if [[ -s "$warning_log" ]]; then
  echo "include warning baseline check failed: warnings detected"
  cat "$warning_log"
  exit 1
fi

echo "include warning baseline check passed: no warnings found"
