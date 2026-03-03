# Agent World: Minimal System Run (Viewer Demo) - Project Plan

## 任务拆解
- [x] M1 Implement demo data generator API (plan actions + save snapshot/journal)
- [x] M1 Add unit tests for demo action planning and persistence output
- [x] M2 Add `world_viewer_demo` binary + update docs (README / visualization quick start)

## 依赖
- `WorldInitConfig` / `WorldScenario` / `initialize_kernel`
- `WorldKernel` persistence (`snapshot.json` / `journal.json`)
- Viewer server (`world_viewer_server`) and viewer client (`agent_world_viewer`)

## 状态
- Current stage: M2 complete (tests done)
