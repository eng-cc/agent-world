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
  --reuse-bridge \
  --skip-agent-setup \
  --no-open-browser
```

## Repo-Backed Components

Current boundary:

- runtime agent installer is repo-backed: `scripts/setup-openclaw-agent-world-runtime.sh`
- local compatibility bridge is repo-backed: `world_openclaw_local_bridge`
- parity smoke is repo-backed: `scripts/openclaw-parity-p0.sh`

So a downloaded game bundle is enough for real play. If bridge and runtime agent are already running, prefer `--reuse-bridge --skip-agent-setup` as the no-`cargo` path; only auto bridge bootstrap, runtime-agent install, source-tree launch, and `smoke` still need repo access + `cargo`.

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

## Doctor Contract

`oasis7-run.sh doctor` now reports two operator-facing readiness tracks separately:

- `bundle-play`: whether a valid bundle plus reachable bridge can support no-`cargo` real play via `--reuse-bridge --skip-agent-setup`
- `repo-bootstrap`: whether repo root + `cargo` are available for auto runtime-agent/bootstrap work

## Shutdown Contract

Killing `oasis7-run.sh play` now performs best-effort teardown of the launched play subtree. In the bundle-backed path this includes the wrapper-started launcher stack instead of leaving residual `world_game_launcher` / `world_chain_runtime` / `world_viewer_live` behind.
