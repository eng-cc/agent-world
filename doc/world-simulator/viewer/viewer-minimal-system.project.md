# oasis7: Minimal System Run (Viewer Demo) - Project Plan

- 对应设计文档: `doc/world-simulator/viewer/viewer-minimal-system.design.md`
- 对应需求文档: `doc/world-simulator/viewer/viewer-minimal-system.prd.md`

审计轮次: 5
## 任务拆解（含 PRD-ID 映射）
- [x] M1 Implement demo data generator API (plan actions + save snapshot/journal)
- [x] M1 Add unit tests for demo action planning and persistence output
- [x] M2 Add `world_viewer_demo` binary + update docs (README / visualization quick start)

## 依赖
- doc/world-simulator/viewer/viewer-minimal-system.prd.md
- `WorldInitConfig` / `WorldScenario` / `initialize_kernel`
- `WorldKernel` persistence (`snapshot.json` / `journal.json`)
- Viewer server (`world_viewer_server`) and viewer client (`agent_world_viewer`)

## 状态
- 最近更新：2026-03-06（ROUND-005 I5-001 字段补齐）
- Current stage: M2 complete (tests done)
