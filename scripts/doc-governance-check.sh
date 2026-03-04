#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"

usage() {
  cat <<'USAGE'
Usage: ./scripts/doc-governance-check.sh

Checks:
  1. Non-archive/non-devlog markdown files must not contain absolute /Users/... or /home/... paths.
  2. Non-archive/non-devlog markdown files must be <= 1000 lines.
  3. Each non-archive project doc (*.project.md) must include sections:
     任务拆解 / 依赖 / 状态.
  4. Each non-archive project doc must have a paired design doc and that design doc
     must include either:
       - Legacy sections: 目标 / 范围 / 接口/数据 / 里程碑 / 风险
       - Strict PRD sections: 1..6 chapter structure
     (except whitelisted project docs).
  5. Root-level markdown files under doc/ must match the tracked allowlist.
  6. Root-level markdown files under each module (doc/<module>/*.md) must match
     the tracked allowlist (archive/devlog/.governance excluded).
  7. Active topic PRD pairs (non-archive, non-devlog, excluding module main
     doc/<module>/prd*.md) must contain bidirectional references:
       - topic design doc (*.prd.md) includes its *.prd.project.md path
       - topic project doc (*.prd.project.md) includes its *.prd.md path
  8. Non-archive/non-devlog markdown files must not reference missing markdown
     paths under doc/ (wildcards/templates and explicit exemption docs excluded).
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

# Some handbooks are intentionally concise and do not follow design-doc section template.
# Whitelist is keyed by project doc path to keep exemptions explicit and reviewable.
readonly DESIGN_SECTION_EXEMPT_PROJECT_DOCS=(
  "doc/playability_test_result/game-test.prd.project.md"
)
readonly REFERENCE_EXISTENCE_EXEMPT_DOCS=(
  "doc/engineering/doc-migration/legacy-doc-migration-backlog-2026-03-03.md"
)
readonly DOC_ROOT_MD_ALLOWLIST_FILE="doc/.governance/doc-root-md-allowlist.txt"
readonly MODULE_ROOT_MD_ALLOWLIST_FILE="doc/.governance/module-root-md-allowlist.txt"

fail() {
  echo "doc-governance-check: FAIL: $*"
  failures=$((failures + 1))
}

regex_match_file() {
  local regex="$1"
  local file="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -q -e "$regex" "$file"
    return $?
  fi
  grep -Eq -- "$regex" "$file"
}

regex_match_with_line_numbers() {
  local regex="$1"
  shift
  if command -v rg >/dev/null 2>&1; then
    rg -n -e "$regex" "$@"
    return $?
  fi
  grep -nE -- "$regex" "$@"
}

contains_literal() {
  local needle="$1"
  local file="$2"
  if command -v rg >/dev/null 2>&1; then
    rg -Fq -- "$needle" "$file"
    return $?
  fi
  grep -Fq -- "$needle" "$file"
}

has_heading() {
  local file="$1"
  local pattern="$2"
  regex_match_file "^#{1,6}[[:space:]]*([0-9]+([.][0-9]+)*[.]?[[:space:]]*)?${pattern}.*$" "$file"
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

has_strict_prd_sections() {
  local file="$1"
  has_heading "$file" "Executive Summary" \
    && has_heading "$file" "User Experience[[:space:]]*&[[:space:]]*Functionality" \
    && has_heading "$file" "AI System Requirements[[:space:]]*\\(If Applicable\\)" \
    && has_heading "$file" "Technical Specifications" \
    && has_heading "$file" "Risks[[:space:]]*&[[:space:]]*Roadmap" \
    && has_heading "$file" "Validation[[:space:]]*&[[:space:]]*Decision Record"
}

check_allowlist_match() {
  local label="$1"
  local allowlist_file="$2"
  local actual_file="$3"
  local allowlist_tmp
  allowlist_tmp=$(mktemp)

  if [[ ! -f "$allowlist_file" ]]; then
    fail "${label} allowlist file missing: ${allowlist_file}"
    rm -f "$allowlist_tmp"
    return
  fi

  grep -Ev '^[[:space:]]*($|#)' "$allowlist_file" | sort -u > "$allowlist_tmp"
  sort -u -o "$actual_file" "$actual_file"

  local unexpected missing
  unexpected=$(comm -23 "$actual_file" "$allowlist_tmp" || true)
  missing=$(comm -13 "$actual_file" "$allowlist_tmp" || true)

  if [[ -n "$unexpected" ]]; then
    echo "doc-governance-check: ${label} unexpected entries:"
    echo "$unexpected"
    fail "${label} contains paths not tracked in allowlist"
  fi

  if [[ -n "$missing" ]]; then
    echo "doc-governance-check: ${label} missing entries (stale allowlist):"
    echo "$missing"
    fail "${label} allowlist contains paths that no longer exist"
  fi

  rm -f "$allowlist_tmp"
}

is_design_section_exempt_project_doc() {
  local project_doc="$1"
  local exempt
  for exempt in "${DESIGN_SECTION_EXEMPT_PROJECT_DOCS[@]}"; do
    if [[ "$project_doc" == "$exempt" ]]; then
      return 0
    fi
  done
  return 1
}

is_topic_project_doc() {
  local project_doc="$1"
  [[ ! "$project_doc" =~ ^doc/[^/]+/prd\.project\.md$ ]]
}

is_reference_exempt_doc() {
  local doc_file="$1"
  local exempt
  for exempt in "${REFERENCE_EXISTENCE_EXEMPT_DOCS[@]}"; do
    if [[ "$doc_file" == "$exempt" ]]; then
      return 0
    fi
  done
  return 1
}

extract_doc_markdown_references() {
  local file="$1"
  if command -v rg >/dev/null 2>&1; then
    rg -o --no-filename 'doc/[A-Za-z0-9_./-]+\.md' "$file" | sort -u
    return
  fi
  grep -oE 'doc/[A-Za-z0-9_./-]+\.md' "$file" | sort -u
}

check_doc_path_references() {
  local file="$1"
  local ref_path

  if is_reference_exempt_doc "$file"; then
    return
  fi

  while IFS= read -r ref_path; do
    [[ -z "$ref_path" ]] && continue
    case "$ref_path" in
      *'*'*|*'?'*|*'['*|*']'*|*'{'*|*'}'*|*'YYYY-MM-DD'*)
        continue
        ;;
    esac
    if [[ ! -f "$ref_path" ]]; then
      fail "$file references missing markdown path: $ref_path"
    fi
  done < <(extract_doc_markdown_references "$file")
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
if abs_hits=$(regex_match_with_line_numbers '/(Users|home)/[^[:space:]]+' "${all_doc_files[@]}"); then
  echo "doc-governance-check: absolute path hits:"
  echo "$abs_hits"
  fail "absolute user-home paths found in non-archive docs"
fi

# 2) line count check
for file in "${all_doc_files[@]}"; do
  line_count=$(wc -l < "$file" | tr -d ' ')
  if ((line_count > 1000)); then
    fail "$file exceeds 1000 lines (${line_count})"
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

  if is_topic_project_doc "$project_doc"; then
    if ! contains_literal "$project_doc" "$design_doc"; then
      fail "$design_doc missing bidirectional link to paired project doc: $project_doc"
    fi
    if ! contains_literal "$design_doc" "$project_doc"; then
      fail "$project_doc missing bidirectional link to paired design doc: $design_doc"
    fi
  fi

  if is_design_section_exempt_project_doc "$project_doc"; then
    continue
  fi

  if has_strict_prd_sections "$design_doc"; then
    check_required_sections "$design_doc" \
      "Executive Summary" \
      "User Experience[[:space:]]*&[[:space:]]*Functionality" \
      "AI System Requirements[[:space:]]*\\(If Applicable\\)" \
      "Technical Specifications" \
      "Risks[[:space:]]*&[[:space:]]*Roadmap" \
      "Validation[[:space:]]*&[[:space:]]*Decision Record"
  else
    check_required_sections "$design_doc" "目标" "范围" "接口[[:space:]]*/[[:space:]]*数据" "里程碑" "风险"
  fi
done

# 4) markdown doc path references must exist (except explicit exemptions)
for file in "${all_doc_files[@]}"; do
  check_doc_path_references "$file"
done

doc_root_actual_tmp=$(mktemp)
module_root_actual_tmp=$(mktemp)

find doc -mindepth 1 -maxdepth 1 -type f -name '*.md' | sort > "$doc_root_actual_tmp"
find doc -mindepth 2 -maxdepth 2 -type f -name '*.md' \
  ! -path 'doc/archive/*' \
  ! -path 'doc/devlog/*' \
  ! -path 'doc/.governance/*' \
  | sort > "$module_root_actual_tmp"

check_allowlist_match "doc root markdown set" "$DOC_ROOT_MD_ALLOWLIST_FILE" "$doc_root_actual_tmp"
check_allowlist_match "module root markdown set" "$MODULE_ROOT_MD_ALLOWLIST_FILE" "$module_root_actual_tmp"

rm -f "$doc_root_actual_tmp" "$module_root_actual_tmp"

if ((failures > 0)); then
  echo "doc-governance-check: failed with ${failures} issue(s)"
  exit 1
fi

echo "doc-governance-check: OK"
