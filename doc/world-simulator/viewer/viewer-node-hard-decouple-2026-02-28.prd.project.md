# Viewer 与节点彻底拆分（2026-02-28）项目管理

审计轮次: 3

## 任务拆解（含 PRD-ID 映射）
- [x] T0 建档：设计文档 + 项目管理文档。
- [x] T1 代码重构：`world_viewer_live` 入口改为纯 viewer，禁用内嵌节点参数与运行链路。
- [x] T2 回归收口：required 测试、文档状态更新、任务日志。

## 依赖
- doc/world-simulator/viewer/viewer-node-hard-decouple-2026-02-28.prd.md
- `agent_world::viewer::{ViewerLiveServer, ViewerWebBridge}`
- `world_game_launcher`（当前已默认编排 `world_chain_runtime`）

## 状态
- 当前阶段：已完成（T0~T2）。
- 当前任务：无（项目结项）。
