# Agent World Simulator：Viewer Location 开采损耗可视化（项目管理文档）

- 对应设计文档: `doc/world-simulator/viewer/viewer-location-depletion-visualization.design.md`
- 对应需求文档: `doc/world-simulator/viewer/viewer-location-depletion-visualization.prd.md`

审计轮次: 5
## 任务拆解（含 PRD-ID 映射）
- [x] LDV1：输出设计文档与项目管理文档
- [x] LDV2：接入 `fragment_budget` 到 location 渲染缩放
- [x] LDV3：补充详情面板损耗指标
- [x] LDV4：新增/更新测试并通过回归
- [x] LDV5：更新说明书与 devlog 收口

## 依赖
- doc/world-simulator/viewer/viewer-location-depletion-visualization.prd.md
- `crates/agent_world_viewer/src/scene_helpers.rs`
- `crates/agent_world_viewer/src/ui_text.rs`
- `crates/agent_world_viewer/src/tests.rs`

## 状态
- 当前阶段：已完成
- 最近更新：完成 LDV5（损耗缩放渲染 + 详情指标 + 测试回归，2026-02-10）
