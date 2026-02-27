#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

usage() {
  cat <<'USAGE'
Usage: ./scripts/doc-governance-check.sh

Checks:
  1. Non-archive/non-devlog markdown files must not contain absolute /Users/... or /home/... paths.
  2. Non-archive/non-devlog markdown files must be <= 500 lines.
  3. Each non-archive project doc (*.project.md) must include sections:
     任务拆解 / 依赖 / 状态.
  4. Each non-archive project doc must have a paired design doc and that design doc
     must include sections: 目标 / 范围 / 接口/数据 / 里程碑 / 风险.
USAGE
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if [[ $# -ne 0 ]]; then
  usage
  exit 1
fi

failures=0

fail() {
  echo "doc-governance-check: FAIL: $*"
  failures=$((failures + 1))
}

has_heading() {
  local file="$1"
  local pattern="$2"
  rg -q -e "^#{1,6}[[:space:]]*([0-9]+([.][0-9]+)*[[:space:]]*)?${pattern}.*$" "$file"
}

check_required_sections() {
  local file="$1"
  shift
  local missing=()
  local token
  for token in "$@"; do
    if ! has_heading "$file" "$token"; then
      missing+=("$token")
    fi
  done
  if [[ ${#missing[@]} -gt 0 ]]; then
    fail "$file missing sections: ${missing[*]}"
  fi
}

mapfile -t all_doc_files < <(find doc -type f -name '*.md' ! -path 'doc/devlog/*' ! -path '*/archive/*' | sort)
mapfile -t project_docs < <(find doc -type f -name '*.project.md' ! -path '*/archive/*' | sort)

if [[ ${#all_doc_files[@]} -eq 0 ]]; then
  fail "no markdown files found under doc/"
fi

if [[ ${#project_docs[@]} -eq 0 ]]; then
  fail "no project docs found under doc/"
fi

# 1) absolute path check
if abs_hits=$(rg -n -e '/(Users|home)/[^[:space:]]+' "${all_doc_files[@]}"); then
  echo "doc-governance-check: absolute path hits:"
  echo "$abs_hits"
  fail "absolute user-home paths found in non-archive docs"
fi

# 2) line count check
for file in "${all_doc_files[@]}"; do
  line_count=$(wc -l < "$file" | tr -d ' ')
  if ((line_count > 500)); then
    fail "$file exceeds 500 lines (${line_count})"
  fi
done

# 3) project docs required sections + paired design required sections
for project_doc in "${project_docs[@]}"; do
  check_required_sections "$project_doc" "任务拆解" "依赖" "状态"

  design_doc="${project_doc%.project.md}.md"
  if [[ ! -f "$design_doc" ]]; then
    fail "$project_doc has no paired design doc: $design_doc"
    continue
  fi

  check_required_sections "$design_doc" "目标" "范围" "接口[[:space:]]*/[[:space:]]*数据" "里程碑" "风险"
done

if ((failures > 0)); then
  echo "doc-governance-check: failed with ${failures} issue(s)"
  exit 1
fi

echo "doc-governance-check: OK"
