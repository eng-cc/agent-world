#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

openclaw_env_or_default() {
  local suffix="$1"
  local default_value="${2-}"
  local key="OPENCLAW_OASIS7_${suffix}"
  local compat_old_brand_key="OPENCLAW_AGENT_WORLD_${suffix}"
  if [[ -n "${!key+x}" ]]; then
    printf '%s\n' "${!key}"
  elif [[ -n "${!compat_old_brand_key+x}" ]]; then
    printf '%s\n' "${!compat_old_brand_key}"
  else
    printf '%s\n' "$default_value"
  fi
}

AGENT_ID="${1:-$(openclaw_env_or_default AGENT_ID oasis7_runtime)}"
WORKSPACE_DIR="$(openclaw_env_or_default WORKSPACE "$ROOT_DIR/tools/openclaw/oasis7_runtime_workspace")"
MODEL_ID="$(openclaw_env_or_default MODEL custom-right-codes/gpt-5.4)"

if ! command -v openclaw >/dev/null 2>&1; then
  echo "openclaw CLI not found in PATH" >&2
  exit 1
fi

if [ ! -d "$WORKSPACE_DIR" ]; then
  echo "workspace directory not found: $WORKSPACE_DIR" >&2
  exit 1
fi

if openclaw agents list --json | jq -e --arg id "$AGENT_ID" '.[] | select(.id == $id)' >/dev/null; then
  echo "OpenClaw agent already exists: $AGENT_ID"
  openclaw agents list --json | jq --arg id "$AGENT_ID" '.[] | select(.id == $id)'
  exit 0
fi

openclaw agents add "$AGENT_ID" \
  --workspace "$WORKSPACE_DIR" \
  --model "$MODEL_ID" \
  --non-interactive \
  --json
