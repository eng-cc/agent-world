---
name: oasis7
description: OpenClaw real-play and parity workflow for Agent World. Use when the user wants to configure, start, validate, or debug a real local OpenClaw gameplay path, including installing the lightweight runtime agent, starting the local bridge, launching `world_game_launcher`, probing `openclaw_local_http`, or running parity smoke for OpenClaw NPC behavior.
---

# Oasis7

## Overview

`oasis7` is the repo-local workflow for running a real OpenClaw-backed Agent World NPC.
Use it for “能不能真跑起来”, “怎么配 OpenClaw 试玩”, “起 bridge / launcher / parity”, and first-line debugging of the local `openclaw_local_http` path.

## When To Use

Use this skill when the task involves any of these:

- Configure a real OpenClaw gameplay run instead of mock provider tests
- Install or refresh the lightweight OpenClaw runtime agent
- Start or debug `world_openclaw_local_bridge`
- Launch the product path with `world_game_launcher` in `openclaw_local_http` mode
- Run `P0-001` parity smoke or inspect OpenClaw latency / wait-only failures
- Explain which OpenClaw settings are required for a real local试玩

Do not use this skill for:

- Generic LLM provider work unrelated to OpenClaw
- Editing OpenClaw third-party source under `third_party/`
- Viewer-only UI styling tasks with no OpenClaw runtime involvement

## Core Workflow

### 1. Verify local prerequisites

Check these first:

- `openclaw` CLI exists in `PATH`
- OpenClaw Gateway is live on `127.0.0.1:18789`
- Repo bridge will bind to `127.0.0.1:5841`
- Cargo commands use `env -u RUSTC_WRAPPER cargo ...`

Useful probes:

```bash
openclaw --version
curl -sS http://127.0.0.1:18789/health
```

### 2. Install the lightweight runtime agent

For real gameplay or parity, prefer the repo-owned lightweight agent instead of the user’s default OpenClaw workspace.

```bash
scripts/setup-openclaw-agent-world-runtime.sh
```

Defaults:

- agent id: `agent_world_runtime`
- workspace: `tools/openclaw/agent_world_runtime_workspace`
- model: `custom-right-codes/gpt-5.4`

The runtime workspace is intentionally slim and is not meant for daily chat.

## 3. Start the bridge

Run the local compatibility bridge that exposes world-simulator provider endpoints:

```bash
env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_openclaw_local_bridge -- --openclaw-agent agent_world_runtime
```

Expected local provider URL:

- `http://127.0.0.1:5841`

Health probes:

```bash
curl -sS http://127.0.0.1:5841/v1/provider/info | jq .
curl -sS http://127.0.0.1:5841/v1/provider/health | jq .
```

## 4. Launch a real gameplay run

Use the product path, not just ad-hoc provider calls:

```bash
env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_game_launcher -- \
  --scenario llm_bootstrap \
  --with-llm \
  --agent-provider-mode openclaw_local_http \
  --openclaw-base-url http://127.0.0.1:5841 \
  --openclaw-connect-timeout-ms 15000 \
  --openclaw-agent-profile agent_world_p0_low_freq_npc
```

Required real-play settings:

- `agent_provider_mode=openclaw_local_http`
- `openclaw_base_url=http://127.0.0.1:5841`
- `openclaw_connect_timeout_ms=15000`
- `openclaw_agent_profile=agent_world_p0_low_freq_npc`

## 5. Run parity smoke

Use this as the fastest real verification path:

```bash
bash scripts/openclaw-parity-p0.sh \
  --openclaw-only \
  --samples 1 \
  --ticks 4 \
  --timeout-ms 15000 \
  --openclaw-base-url http://127.0.0.1:5841 \
  --openclaw-connect-timeout-ms 15000 \
  --openclaw-agent-profile agent_world_p0_low_freq_npc
```

Primary success target today:

- `P0-001`
- `status=passed`
- `goal_completed=true`
- `invalid_action_count=0`

## Debug Checklist

If the run fails, inspect in this order:

1. Gateway health: `http://127.0.0.1:18789/health`
2. Bridge health: `http://127.0.0.1:5841/v1/provider/health`
3. Wrong provider mode or missing profile
4. Bridge not started with the lightweight agent
5. Parity artifacts under `artifacts/openclaw_parity_*`

Current known reality:

- Correctness is largely working for `P0-001`
- Latency is still high in real parity, so treat the path as `experimental`
- `agent_chat` and `prompt_control` are still unsupported in OpenClaw mode

## Repo Anchors

Use these files as the source of truth:

- Bridge entry: `crates/agent_world/src/bin/world_openclaw_local_bridge.rs`
- Launcher entry: `crates/agent_world/src/bin/world_game_launcher.rs`
- Runtime workspace installer: `scripts/setup-openclaw-agent-world-runtime.sh`
- Runtime workspace policy: `tools/openclaw/agent_world_runtime_workspace/AGENTS.md`
- Module tracker: `doc/world-simulator/project.md`
- Daily log: `doc/devlog/2026-03-13.md`

## Output Expectations

When using this skill:

- Prefer exact commands over abstract advice
- State which process provides `127.0.0.1:18789` and which provides `127.0.0.1:5841`
- Distinguish “runtime agent workspace/profile” from Codex repo skills
- If you changed behavior or tooling, update `doc/world-simulator/project.md` and `doc/devlog/YYYY-MM-DD.md`
