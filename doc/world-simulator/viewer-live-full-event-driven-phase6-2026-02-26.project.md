# Viewer Live 完全事件驱动改造 Phase 6（项目管理）

## 任务拆解
- [x] T0 建档：设计文档 + 项目管理文档
- [x] T1 非共识驱动事件化：`NonConsensusDriveRequested` 接线
- [x] T2 状态变更节流：有效推进后再发射 metrics/snapshot
- [x] T3 回归测试：非共识驱动语义 + 背压语义
- [x] T4 文档与日志收口

## 依赖
- `crates/agent_world/src/viewer/live_split_part1.rs`
- `crates/agent_world/src/viewer/live_split_part2.rs`
- `crates/agent_world/src/viewer/live/tests.rs`
- `testing-manual.md`

## 状态
- 当前阶段：已完成（T0~T4）
- 备注：Phase 6 聚焦非共识链路去定时化与状态变更驱动收敛。
