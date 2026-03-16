# OpenClaw Agent 双轨模式（player_parity / headless_agent / debug_viewer）（2026-03-16）项目管理文档

- 对应需求文档: `doc/world-simulator/llm/llm-openclaw-agent-dual-mode-2026-03-16.prd.md`
- 关联专题:
  - `doc/world-simulator/llm/llm-openclaw-agent-experience-parity-2026-03-12.project.md`
  - `doc/world-simulator/viewer/viewer-web-software-safe-mode-2026-03-16.project.md`

审计轮次: 1

## 任务拆解（含 PRD-ID 映射）
- [x] T0 (PRD-WORLD_SIMULATOR-040) [test_tier_required]: 完成 OpenClaw 双轨模式专题 PRD / Project 建模，并回写模块主文档、索引与 devlog。
- [x] T1 (PRD-WORLD_SIMULATOR-040) [test_tier_required]: 由 `agent_engineer` 牵头冻结 `player_parity` / `headless_agent` 的 observation/action contract、schema version 与禁止泄露的真值边界，并形成 supporting spec `openclaw-agent-dual-mode-contract-2026-03-16.md`。
- [x] T2 (PRD-WORLD_SIMULATOR-040) [test_tier_required]: 由 `runtime_engineer` 落地 mode metadata、统一 replay/summary 追踪字段，并确保所有模式共享权威动作校验。
- [x] T3 (PRD-WORLD_SIMULATOR-040) [test_tier_required]: 由 `viewer_engineer` 把 `debug_viewer` 明确收口为旁路订阅层，并补 mode/fallback 可观测性与 software-safe 对照入口。
- [ ] T4 (PRD-WORLD_SIMULATOR-040) [test_tier_full]: 由 `qa_engineer` / `producer_system_designer` 对同一 OpenClaw 场景执行 `player_parity` vs `headless_agent` 对照采证，形成默认模式与阻断结论。

## 依赖
- `doc/world-simulator/llm/llm-openclaw-agent-experience-parity-2026-03-12.prd.md`
- `doc/world-simulator/llm/llm-openclaw-agent-experience-parity-2026-03-12.project.md`
- `doc/world-simulator/viewer/viewer-web-software-safe-mode-2026-03-16.prd.md`
- `doc/world-simulator/viewer/viewer-web-software-safe-mode-2026-03-16.project.md`
- `doc/world-simulator/llm/llm-openclaw-local-http-provider-integration-2026-03-12.prd.md`
- `doc/world-simulator/llm/llm-decision-provider-standard-openclaw-feasibility-2026-03-12.prd.md`
- `doc/world-simulator/prd.md`
- `doc/world-simulator/project.md`
- `doc/world-simulator/prd.index.md`
- `doc/world-simulator/llm/openclaw-agent-dual-mode-contract-2026-03-16.md`
- `testing-manual.md`

## 状态
- 最近更新：2026-03-16
- 当前阶段: T4 pending
- 当前任务: `由 qa_engineer / producer_system_designer 执行 player_parity vs headless_agent 对照采证，并形成默认模式结论`
- owner: `qa_engineer`
- 联审: `agent_engineer`、`runtime_engineer`、`viewer_engineer`、`qa_engineer`
- 发起建模: `producer_system_designer`
- 备注: 本专题只定义“双轨模式”的产品目标与执行边界；它不替代 `PRD-WORLD_SIMULATOR-038` 的 parity 门禁，而是为 parity/回归/观战拆出各自清晰口径。
