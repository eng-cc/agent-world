#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

SOURCE_MANUAL="${REPO_ROOT}/doc/world-simulator/viewer/viewer-manual.md"
MIRROR_MANUALS=(
  "${REPO_ROOT}/site/doc/cn/viewer-manual.html"
  "${REPO_ROOT}/site/doc/en/viewer-manual.html"
)

REQUIRED_PATTERNS=(
  'export PWCLI="${CODEX_HOME:-$HOME/.codex}/skills/playwright/scripts/playwright_cli.sh"'
  '[ -f "$PWCLI" ] || { echo "missing playwright cli wrapper: $PWCLI" >&2; exit 1; }'
)

FORBIDDEN_PATTERNS=(
  'export REPO_ROOT="$(pwd)"'
  '$REPO_ROOT/.codex/skills/playwright/scripts/playwright_cli.sh'
  './.codex/skills/playwright/scripts/playwright_cli.sh'
)

contains_fixed_pattern() {
  local pattern="$1"
  local file_path="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -Fq -- "$pattern" "$file_path"
    return $?
  fi
  grep -Fq -- "$pattern" "$file_path"
}

check_required_patterns() {
  local file_path="$1"
  local pattern
  for pattern in "${REQUIRED_PATTERNS[@]}"; do
    if ! contains_fixed_pattern "$pattern" "$file_path"; then
      echo "error: missing required pattern in ${file_path}: ${pattern}" >&2
      return 1
    fi
  done
}

check_forbidden_patterns() {
  local file_path="$1"
  local pattern
  for pattern in "${FORBIDDEN_PATTERNS[@]}"; do
    if contains_fixed_pattern "$pattern" "$file_path"; then
      echo "error: found deprecated pattern in ${file_path}: ${pattern}" >&2
      return 1
    fi
  done
}

check_required_patterns "${SOURCE_MANUAL}"
check_forbidden_patterns "${SOURCE_MANUAL}"

for mirror in "${MIRROR_MANUALS[@]}"; do
  check_required_patterns "${mirror}"
  check_forbidden_patterns "${mirror}"
done

echo "ok: viewer manual static mirrors are synced with Playwright path baseline"
