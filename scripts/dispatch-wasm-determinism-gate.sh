#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

usage() {
  cat <<'USAGE'
Usage: ./scripts/dispatch-wasm-determinism-gate.sh [options]

Purpose:
  Trigger the wasm-determinism-gate workflow with an external Docker-capable
  summary bundle so full-tier cross-host evidence can be archived once a real
  darwin-arm64 bundle exists.

Requirements:
  - GitHub CLI (`gh`) installed and authenticated
  - A reachable bundle URL for `external_summary_bundle_url`

Options:
  --bundle-url <url>          External summary bundle URL (required)
  --runner-label <label>      Expected external runner label (default: darwin-arm64)
  --workflow <file-or-name>   Workflow identifier (default: wasm-determinism-gate.yml)
  --ref <git-ref>             Git ref to dispatch (default: current branch)
  --repo <owner/name>         Optional GitHub repo override for `gh`
  -h, --help                  Show help
USAGE
}

bundle_url=""
runner_label="darwin-arm64"
workflow_id="wasm-determinism-gate.yml"
git_ref="$(git rev-parse --abbrev-ref HEAD)"
repo_slug=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bundle-url)
      bundle_url=${2:-}
      shift 2
      ;;
    --runner-label)
      runner_label=${2:-}
      shift 2
      ;;
    --workflow)
      workflow_id=${2:-}
      shift 2
      ;;
    --ref)
      git_ref=${2:-}
      shift 2
      ;;
    --repo)
      repo_slug=${2:-}
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ -z "$bundle_url" ]]; then
  echo "error: --bundle-url is required" >&2
  usage >&2
  exit 2
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "error: gh CLI is required" >&2
  exit 2
fi

gh_cmd=(
  gh workflow run "$workflow_id"
  --ref "$git_ref"
  -f "external_summary_bundle_url=$bundle_url"
  -f "external_summary_runner_label=$runner_label"
)
if [[ -n "$repo_slug" ]]; then
  gh_cmd+=(--repo "$repo_slug")
fi

"${gh_cmd[@]}"

run_list_cmd=(gh run list --workflow "$workflow_id")
run_view_cmd=(gh run view "<run-id>" --log-failed)
if [[ -n "$repo_slug" ]]; then
  run_list_cmd+=(--repo "$repo_slug")
  run_view_cmd+=(--repo "$repo_slug")
fi

echo "dispatched wasm determinism gate:"
echo "  workflow=$workflow_id"
echo "  ref=$git_ref"
if [[ -n "$repo_slug" ]]; then
  echo "  repo=$repo_slug"
fi
echo "  external_summary_bundle_url=$bundle_url"
echo "  external_summary_runner_label=$runner_label"
echo "next:"
printf '  %q' "${run_list_cmd[@]}"
printf '\n'
printf '  %q' "${run_view_cmd[@]}"
printf '\n'
