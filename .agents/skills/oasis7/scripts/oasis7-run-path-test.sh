#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=/dev/null
source "$script_dir/oasis7-run.sh"

tmp_home="$(mktemp -d)"
tmp_pwd="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp_home" "$tmp_pwd"
}
trap cleanup EXIT

(
  export HOME="$tmp_home"
  cd "$tmp_pwd"

  expanded_default="$(normalize_path '~/.cache/oasis7/releases')"
  expected_default="$tmp_home/.cache/oasis7/releases"
  if [[ "$expanded_default" != "$expected_default" ]]; then
    echo "expected default path '$expected_default', got '$expanded_default'" >&2
    exit 1
  fi
  if [[ "$expanded_default" == "$tmp_pwd/~/"* || "$expanded_default" == "$tmp_pwd/~"* ]]; then
    echo "default path incorrectly stayed repo-local: $expanded_default" >&2
    exit 1
  fi

  expanded_override="$(normalize_path '~/custom-cache/oasis7')"
  expected_override="$tmp_home/custom-cache/oasis7"
  if [[ "$expanded_override" != "$expected_override" ]]; then
    echo "expected override path '$expected_override', got '$expanded_override'" >&2
    exit 1
  fi

  relative_path="$(normalize_path 'relative/bundle')"
  expected_relative="$tmp_pwd/relative/bundle"
  if [[ "$relative_path" != "$expected_relative" ]]; then
    echo "expected relative path '$expected_relative', got '$relative_path'" >&2
    exit 1
  fi
)

echo "oasis7-run path tests passed"
