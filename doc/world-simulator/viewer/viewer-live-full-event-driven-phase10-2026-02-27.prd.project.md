# Viewer Live 完全事件驱动改造 Phase 10（项目管理）

审计轮次: 2

## ROUND-002 主从口径
- 本文件为主项目入口（master）。
- `doc/world-simulator/viewer/viewer-live-full-event-driven-phase8-2026-02-27.prd.project.md` 与 `doc/world-simulator/viewer/viewer-live-full-event-driven-phase9-2026-02-27.prd.project.md` 为增量计划文档（slave）。

## 任务拆解（含 PRD-ID 映射）
- [x] T0 建档：设计文档 + 项目管理文档
- [x] T1 代码收敛：`viewer/server` 删除 `tick_interval` 与定时回放推进
- [x] T2 代码收敛：`viewer/web_bridge` 删除 `poll_interval` 与轮询 sleep
- [x] T3 收口：活跃手册/入口示例与测试同步清理
- [x] T4 回归与结项：required 测试 + 阶段文档收口

## 依赖
- doc/world-simulator/viewer/viewer-live-full-event-driven-phase10-2026-02-27.prd.md
- `crates/agent_world/src/viewer/server.rs`
- `crates/agent_world/src/viewer/web_bridge.rs`
- `crates/agent_world/tests/viewer_offline_integration.rs`
- `site/index.html`
- `site/en/index.html`
- `testing-manual.md` / `doc/world-simulator/viewer/viewer-manual.md`（如需）

## 状态
- 当前阶段：已完成（T0~T4）
