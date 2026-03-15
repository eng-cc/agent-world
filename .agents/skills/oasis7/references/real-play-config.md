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

## Bundle-First Entry

For a real试玩，先下载 GitHub Release 的游戏包，再把 OpenClaw provider 配到 bundle：

- latest Linux bundle: `https://github.com/eng-cc/agent-world/releases/latest/download/agent-world-linux-x64.tar.gz`
- latest macOS bundle: `https://github.com/eng-cc/agent-world/releases/latest/download/agent-world-macos-x64.tar.gz`
- latest Windows bundle: `https://github.com/eng-cc/agent-world/releases/latest/download/agent-world-windows-x64.zip`
- checksums: `https://github.com/eng-cc/agent-world/releases/latest/download/agent-world-checksums.txt`

One-command download via `oasis7`:

```bash
bundle_dir="$(.agents/skills/oasis7/scripts/oasis7-run.sh download)"
printf '%s\n' "$bundle_dir"
```

The returned directory must contain `run-game.sh`.
If you use `--download-dir ~/.cache/oasis7/releases`, the helper expands the current user's `~` and returns an absolute bundle path.

## Preferred Runtime Agent

Prefer the repo-owned lightweight runtime agent:

- agent id: `agent_world_runtime`
- installer: `scripts/setup-openclaw-agent-world-runtime.sh`
- workspace: `tools/openclaw/agent_world_runtime_workspace`

## Product Launch Command

### Release bundle path

```bash
./run-game.sh \
  --scenario llm_bootstrap \
  --with-llm \
  --agent-provider-mode openclaw_local_http \
  --openclaw-base-url http://127.0.0.1:5841 \
  --openclaw-connect-timeout-ms 15000 \
  --openclaw-agent-profile agent_world_p0_low_freq_npc
```

### Repo source path

```bash
env -u RUSTC_WRAPPER cargo run -p agent_world --bin world_game_launcher -- \
  --scenario llm_bootstrap \
  --with-llm \
  --agent-provider-mode openclaw_local_http \
  --openclaw-base-url http://127.0.0.1:5841 \
  --openclaw-connect-timeout-ms 15000 \
  --openclaw-agent-profile agent_world_p0_low_freq_npc
```

### Bundle + wrapper path

```bash
bundle_dir="$(.agents/skills/oasis7/scripts/oasis7-run.sh download)"
.agents/skills/oasis7/scripts/oasis7-run.sh play \
  --bundle-dir "$bundle_dir" \
  --no-open-browser
```

## Repo-Backed Components

Current boundary:

- runtime agent installer is repo-backed: `scripts/setup-openclaw-agent-world-runtime.sh`
- local compatibility bridge is repo-backed: `world_openclaw_local_bridge`
- parity smoke is repo-backed: `scripts/openclaw-parity-p0.sh`

So a downloaded game bundle is enough for real play, but `smoke` and auto bridge bootstrap still need repo access unless you pass `--reuse-bridge --skip-agent-setup`.

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
