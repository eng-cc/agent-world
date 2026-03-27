#!/usr/bin/env bash
set -euo pipefail

CALLER_DIR="$(pwd -P)"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"
source "$ROOT_DIR/scripts/worktree-harness-lib.sh"

usage() {
  cat <<'USAGE'
Usage: ./scripts/new-task-worktree.sh <module> <task> [options]

Create or attach a standardized git worktree for one task slice.

Default conventions:
- branch: task/<module>-<task>
- worktrees root: <repo-parent>/worktrees
- worktree path: <worktrees root>/<repo-name>-<module>-<task>
- base ref: HEAD

Options:
  --base <ref>            Base ref for a new branch (default: HEAD)
  --branch <name>         Override branch name
  --path <path>           Override target worktree path
  --worktrees-root <dir>  Override default worktrees root
  --allow-dirty-source    Allow creating from a dirty source worktree
  --json                  Print machine-readable JSON summary only
  -h, --help              Show this help

Examples:
  ./scripts/new-task-worktree.sh scripts task-worktree-bootstrap
  ./scripts/new-task-worktree.sh viewer hud-redesign --base main
  ./scripts/new-task-worktree.sh p2p hosted-flow --json --path ../worktrees/oasis7-codex-p2p-hosted-flow
USAGE
}

wh_require_git_worktree

ALLOW_DIRTY_SOURCE=0
OUTPUT_JSON=0
BASE_REF="HEAD"
BRANCH_NAME=""
TARGET_PATH=""
WORKTREES_ROOT=""
POSITIONAL=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --base)
      BASE_REF="${2:-}"
      shift 2
      ;;
    --branch)
      BRANCH_NAME="${2:-}"
      shift 2
      ;;
    --path)
      TARGET_PATH="${2:-}"
      shift 2
      ;;
    --worktrees-root)
      WORKTREES_ROOT="${2:-}"
      shift 2
      ;;
    --allow-dirty-source)
      ALLOW_DIRTY_SOURCE=1
      shift
      ;;
    --json)
      OUTPUT_JSON=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      POSITIONAL+=("$1")
      shift
      ;;
  esac
done

if [[ "${#POSITIONAL[@]}" -ne 2 ]]; then
  echo "error: expected <module> and <task>" >&2
  usage >&2
  exit 2
fi

MODULE_INPUT="${POSITIONAL[0]}"
TASK_INPUT="${POSITIONAL[1]}"
[[ -n "$MODULE_INPUT" && -n "$TASK_INPUT" ]] || { echo "error: <module> and <task> cannot be empty" >&2; exit 2; }
[[ -n "$BASE_REF" ]] || { echo "error: --base cannot be empty" >&2; exit 2; }

slugify() {
  python3 - "$1" <<'PY'
from __future__ import annotations

import re
import sys

value = sys.argv[1].strip().lower()
value = re.sub(r"[^a-z0-9]+", "-", value)
value = re.sub(r"-{2,}", "-", value).strip("-")
print(value)
PY
}

resolve_abs_path() {
  python3 - "$CALLER_DIR" "$1" <<'PY'
from __future__ import annotations

from pathlib import Path
import sys

base = Path(sys.argv[1]).resolve()
raw = Path(sys.argv[2])
if raw.is_absolute():
    print(raw.resolve())
else:
    print((base / raw).resolve())
PY
}

branch_checkout_path() {
  git --git-dir="$COMMON_GIT_DIR" worktree list --porcelain | python3 - "$1" <<'PY'
from __future__ import annotations

import sys

target = f"refs/heads/{sys.argv[1]}"
current: dict[str, str] = {}

def emit(record: dict[str, str]) -> None:
    if record.get("branch") == target:
        print(record.get("worktree", ""))
        raise SystemExit(0)

for raw in sys.stdin.read().splitlines():
    if not raw:
      if current:
        emit(current)
        current = {}
      continue
    key, _, value = raw.partition(" ")
    current[key] = value

if current:
    emit(current)

raise SystemExit(1)
PY
}

MODULE_SLUG="$(slugify "$MODULE_INPUT")"
TASK_SLUG="$(slugify "$TASK_INPUT")"
[[ -n "$MODULE_SLUG" ]] || { echo "error: <module> becomes empty after slug normalization" >&2; exit 2; }
[[ -n "$TASK_SLUG" ]] || { echo "error: <task> becomes empty after slug normalization" >&2; exit 2; }

REPO_ROOT="$(wh_repo_root)"
REPO_NAME="$(basename "$REPO_ROOT")"
COMMON_GIT_DIR="$(cd "$(git rev-parse --git-common-dir)" && pwd -P)"
DEFAULT_WORKTREES_ROOT="$(dirname "$REPO_ROOT")/worktrees"
if [[ -n "$WORKTREES_ROOT" ]]; then
  WORKTREES_ROOT="$(resolve_abs_path "$WORKTREES_ROOT")"
else
  WORKTREES_ROOT="$DEFAULT_WORKTREES_ROOT"
fi

if [[ -z "$BRANCH_NAME" ]]; then
  BRANCH_NAME="task/${MODULE_SLUG}-${TASK_SLUG}"
fi

if [[ -n "$TARGET_PATH" ]]; then
  TARGET_PATH="$(resolve_abs_path "$TARGET_PATH")"
else
  TARGET_PATH="$(resolve_abs_path "$WORKTREES_ROOT/$REPO_NAME-$MODULE_SLUG-$TASK_SLUG")"
fi

if [[ "$ALLOW_DIRTY_SOURCE" != "1" ]] && [[ -n "$(git status --short)" ]]; then
  echo "error: source worktree is dirty; commit/stash changes first or rerun with --allow-dirty-source" >&2
  exit 1
fi

if ! git rev-parse --verify --quiet "$BASE_REF^{commit}" >/dev/null; then
  echo "error: base ref not found: $BASE_REF" >&2
  exit 1
fi

if [[ -e "$TARGET_PATH" ]]; then
  echo "error: target worktree path already exists: $TARGET_PATH" >&2
  echo "hint: choose a different task slug/path or remove the old directory first" >&2
  exit 1
fi

if existing_branch_path="$(branch_checkout_path "$BRANCH_NAME" 2>/dev/null)"; then
  echo "error: branch is already checked out in another worktree: $BRANCH_NAME" >&2
  echo "hint: existing worktree path: $existing_branch_path" >&2
  exit 1
fi

mkdir -p "$(dirname "$TARGET_PATH")"

MODE="create_new_branch"
if git show-ref --verify --quiet "refs/heads/$BRANCH_NAME"; then
  MODE="attach_existing_branch"
  git worktree add --quiet "$TARGET_PATH" "$BRANCH_NAME"
else
  git worktree add --quiet -b "$BRANCH_NAME" "$TARGET_PATH" "$BASE_REF"
fi

SUMMARY_JSON="$(python3 - "$MODULE_INPUT" "$TASK_INPUT" "$MODULE_SLUG" "$TASK_SLUG" "$BRANCH_NAME" "$TARGET_PATH" "$BASE_REF" "$MODE" "$REPO_ROOT" <<'PY'
from __future__ import annotations

import json
import sys

payload = {
    "module": sys.argv[1],
    "task": sys.argv[2],
    "module_slug": sys.argv[3],
    "task_slug": sys.argv[4],
    "branch": sys.argv[5],
    "worktree_path": sys.argv[6],
    "base_ref": sys.argv[7],
    "mode": sys.argv[8],
    "repo_root": sys.argv[9],
}
print(json.dumps(payload, ensure_ascii=False))
PY
)"

if [[ "$OUTPUT_JSON" == "1" ]]; then
  printf '%s\n' "$SUMMARY_JSON"
  exit 0
fi

cat <<INFO
Task worktree is ready.
- module: $MODULE_INPUT
- task: $TASK_INPUT
- branch: $BRANCH_NAME
- path: $TARGET_PATH
- base ref: $BASE_REF
- mode: $MODE

Next:
  cd $TARGET_PATH
  sed -n '1,160p' doc/$MODULE_SLUG/prd.md
  sed -n '1,160p' doc/$MODULE_SLUG/project.md
  git status -sb
INFO
