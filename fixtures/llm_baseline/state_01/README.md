# LLM Baseline State 01

- Source run output: `.tmp/llm_baseline/chunked_1200/state_01`
- Created at: `2026-02-21` (copied into git-tracked fixtures)
- Scenario: `llm_bootstrap` with 5 agents and runtime gameplay bridge
- Prompt pack: `industrial_baseline`
- Intended use: load as a reusable industrial-stage world baseline for governance/crisis follow-up tests

## Contents

- `snapshot.json`: simulator kernel snapshot (world model + config + ids + pending actions)
- `journal.json`: simulator event journal used for state consistency checks/replay boundaries

## Example

```bash
./scripts/llm-longrun-stress.sh \
  --scenario llm_bootstrap \
  --ticks 80 \
  --load-state-dir fixtures/llm_baseline/state_01 \
  --prompt-pack civic_operator \
  --runtime-gameplay-bridge \
  --release-gate \
  --release-gate-profile gameplay \
  --no-llm-io \
  --out-dir .tmp/llm_baseline_from_fixture
```
