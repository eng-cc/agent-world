# Real-Play Config

## Required Settings

Use these exact values for a real local OpenClaw gameplay run:

- `agent_provider_mode=openclaw_local_http`
- `openclaw_base_url=http://127.0.0.1:5841`
- `openclaw_connect_timeout_ms=15000`
- `openclaw_agent_profile=agent_world_p0_low_freq_npc`

## Process Ownership

- `127.0.0.1:18789`: OpenClaw Gateway
- `127.0.0.1:5841`: Agent World local compatibility bridge

## Preferred Runtime Agent

Prefer the repo-owned lightweight runtime agent:

- agent id: `agent_world_runtime`
- installer: `scripts/setup-openclaw-agent-world-runtime.sh`
- workspace: `tools/openclaw/agent_world_runtime_workspace`

## Product Launch Command

```bash
env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_game_launcher -- \
  --scenario llm_bootstrap \
  --with-llm \
  --agent-provider-mode openclaw_local_http \
  --openclaw-base-url http://127.0.0.1:5841 \
  --openclaw-connect-timeout-ms 15000 \
  --openclaw-agent-profile agent_world_p0_low_freq_npc
```

## Fast Smoke Command

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
