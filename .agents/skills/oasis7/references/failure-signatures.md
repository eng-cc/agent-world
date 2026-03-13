# Failure Signatures

## `provider_unreachable`

Meaning:

- runtime could not complete a decision call against the local HTTP provider

Check:

1. `curl -sS http://127.0.0.1:5841/v1/provider/health`
2. bridge process is still running
3. `openclaw_base_url` is loopback and correct

## `openclaw_gateway_unreachable`

Meaning:

- bridge could not get a valid response from `openclaw` / Gateway

Check:

1. `curl -sS http://127.0.0.1:18789/health`
2. `openclaw --version`
3. bridge stderr / log file

## `bridge_model_output_invalid`

Meaning:

- OpenClaw returned malformed or non-whitelisted JSON for the world action

Current behavior:

- bridge records structured diagnostics
- some `P0-001` patrol cases may be rerouted by guardrail instead of failing hard

## `unsupported_agent_profile`

Meaning:

- requested `agent_profile` is empty or not supported by the bridge path

Check:

- use `agent_world_p0_low_freq_npc`

## `wait-only` sample with `goal_completed=false`

Meaning:

- run is alive, but the agent is not producing progress

Check:

1. lightweight runtime agent is installed and used
2. bridge is started with `--openclaw-agent agent_world_runtime`
3. parity artifact raw trace under `artifacts/openclaw_parity_*`
4. latency may still be too high even when correctness is fine

## `agent_chat` / `prompt_control` unsupported

Meaning:

- this is expected in current OpenClaw mode

Current boundary:

- real NPC autoplay path is supported
- direct player-side hot control is not yet supported
