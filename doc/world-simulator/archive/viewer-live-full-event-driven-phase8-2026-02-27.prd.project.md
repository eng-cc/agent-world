# Viewer Live 完全事件驱动改造 Phase 8（项目管理）

审计轮次: 2

## 审计备注（2026-03-05 ROUND-002 物理合并）
- 本阶段任务已合并入 `doc/world-simulator/viewer/viewer-live-full-event-driven-phase10-2026-02-27.prd.project.md`。
- 当前替代入口：
  - `doc/world-simulator/viewer/viewer-live-full-event-driven-phase10-2026-02-27.prd.md`
  - `doc/world-simulator/viewer/viewer-live-full-event-driven-phase10-2026-02-27.prd.project.md`
- 本文件保留为阶段追溯，不再作为当前执行基线。

## 任务拆解（含 PRD-ID 映射）
- [x] T0 建档：设计文档 + 项目管理文档
- [x] T1 代码收敛：删除 script 回退开关与 `timer_pulse` 回退链路，仅保留 event_drive
- [x] T2 测试改造：更新/清理回退模式断言并完成 required 回归
- [x] T3 文档收口：更新阶段结论与遗留事项

## 依赖
- doc/world-simulator/archive/viewer-live-full-event-driven-phase8-2026-02-27.prd.md
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/tests.rs`
- `crates/agent_world/src/viewer/mod.rs`
- `crates/agent_world/src/bin/world_viewer_live/world_viewer_live_split_part1.rs`

## 状态
- 当前阶段：已完成（T0~T3）
- 备注：Phase 8 已完成，script 链路已收敛为默认且唯一 `event_drive`。
