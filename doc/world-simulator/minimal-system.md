# Agent World: Minimal System Run (Viewer Demo)

## 目标
- Provide a deterministic, low-friction way to generate `snapshot.json` + `journal.json` for the viewer.
- Enable a minimal end-to-end run: generate demo data -> start viewer server -> open UI client.
- Keep the demo data small but with at least a few events for the event list.

## 范围

### In Scope
- A new CLI binary `world_viewer_demo` in `crates/agent_world`.
- Deterministic demo script that produces at least one event.
- Output directory containing `snapshot.json` and `journal.json` for `world_viewer_server`.
- Simple CLI flags: scenario selection and output directory.

### Out of Scope
- Live simulation server or streaming kernel loop.
- Complex scripting language for demo actions.
- UI/Bevy feature expansion.

## 接口/数据

### CLI (draft)
- `world_viewer_demo [scenario] [--out <dir>]`
- Defaults:
  - `scenario = twin_region_bootstrap`
  - `out = .data/world_viewer_data`

### Output
- `snapshot.json`: `WorldSnapshot` (time 0 + model after demo actions).
- `journal.json`: `WorldJournal` (events from demo actions).

### Demo Script Strategy
- Select a deterministic agent (lowest id) and its current location.
- If a second location exists:
  - Transfer enough electricity from the current location to cover move cost.
  - Move the agent to the other location.
- If only one location exists:
  - Transfer a small amount of electricity from location to agent (may be rejected if empty).
- The goal is to guarantee at least one event in the journal.

## 里程碑
- **M1**: Implement demo data generator + CLI (`world_viewer_demo`).
- **M2**: Add tests for demo action planning and persistence.
- **M3**: Document the end-to-end run steps.

## 风险
- Demo action may be rejected if resources are insufficient (still produces an event).
- UI build may fail in offline environments due to Bevy dependencies.
- Users may expect live data; this is offline replay only.
