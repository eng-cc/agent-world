#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

openclaw_env_or_default() {
  local suffix="$1"
  local default_value="${2-}"
  local key="OPENCLAW_OASIS7_${suffix}"
  if [[ -n "${!key+x}" ]]; then
    printf '%s\n' "${!key}"
  else
    printf '%s\n' "$default_value"
  fi
}

AGENT_ID="${1:-$(openclaw_env_or_default AGENT_ID oasis7_openclaw_agent)}"
WORKSPACE_DIR="$(openclaw_env_or_default WORKSPACE "$ROOT_DIR/tools/openclaw/oasis7_openclaw_workspace")"
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
