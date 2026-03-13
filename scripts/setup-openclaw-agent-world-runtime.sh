#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AGENT_ID="${1:-agent_world_runtime}"
WORKSPACE_DIR="${OPENCLAW_AGENT_WORLD_WORKSPACE:-$ROOT_DIR/tools/openclaw/agent_world_runtime_workspace}"
MODEL_ID="${OPENCLAW_AGENT_WORLD_MODEL:-custom-right-codes/gpt-5.4}"

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
